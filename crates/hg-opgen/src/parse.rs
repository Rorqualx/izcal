//! The `.hgop` parser: recursive descent over the grammar of the
//! language note's Section 3, with the JS-M0 extensions recorded in the
//! crate documentation (`units` declarations, field categories and
//! `repr`, `layout ... of`/`is`, qualified calls and `ensures result is`).

use crate::ast::*;
use crate::diag::{Diag, Span};
use crate::lex::{Tok, Token};

/// Parses one file's tokens into items.
///
/// # Errors
/// The first syntax error (check C1).
pub fn parse(tokens: &[Token]) -> Result<Vec<Item>, Diag> {
    let mut p = Parser {
        toks: tokens,
        at: 0,
    };
    let mut items = Vec::new();
    while !p.is_eof() {
        items.push(p.item()?);
    }
    Ok(items)
}

struct Parser<'a> {
    toks: &'a [Token],
    at: usize,
}

type R<T> = Result<T, Diag>;

impl Parser<'_> {
    fn peek(&self) -> &Tok {
        self.toks.get(self.at).map_or(&Tok::Eof, |t| &t.tok)
    }

    fn peek_at(&self, n: usize) -> &Tok {
        self.toks.get(self.at + n).map_or(&Tok::Eof, |t| &t.tok)
    }

    fn span(&self) -> Span {
        self.toks
            .get(self.at)
            .or_else(|| self.toks.last())
            .map(|t| t.span)
            .unwrap_or_default()
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek(), Tok::Eof)
    }

    fn err<T>(&self, msg: impl Into<String>) -> R<T> {
        Err(Diag::new(self.span(), "OC-1", msg))
    }

    fn is_punct(&self, p: &str) -> bool {
        matches!(self.peek(), Tok::Punct(q) if *q == p)
    }

    fn is_kw(&self, k: &str) -> bool {
        matches!(self.peek(), Tok::Ident(s) if s == k)
    }

    fn eat_punct(&mut self, p: &str) -> bool {
        if self.is_punct(p) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, k: &str) -> bool {
        if self.is_kw(k) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn punct(&mut self, p: &str) -> R<()> {
        if self.eat_punct(p) {
            Ok(())
        } else {
            self.err(format!("expected `{p}`, found {}", describe(self.peek())))
        }
    }

    fn kw(&mut self, k: &str) -> R<()> {
        if self.eat_kw(k) {
            Ok(())
        } else {
            self.err(format!("expected `{k}`, found {}", describe(self.peek())))
        }
    }

    fn name(&mut self) -> R<Name> {
        let span = self.span();
        match self.peek().clone() {
            Tok::Ident(text) => {
                self.at += 1;
                Ok(Name { text, span })
            }
            other => self.err(format!("expected a name, found {}", describe(&other))),
        }
    }

    fn string(&mut self) -> R<String> {
        match self.peek().clone() {
            Tok::Str(s) => {
                self.at += 1;
                Ok(s)
            }
            other => self.err(format!("expected a string, found {}", describe(&other))),
        }
    }

    // --- Items ---------------------------------------------------------

    fn item(&mut self) -> R<Item> {
        if self.eat_kw("const") {
            let name = self.name()?;
            self.punct(":")?;
            let ty = self.ty()?;
            self.punct("=")?;
            let negative = self.eat_punct("-");
            let value = self.literal()?;
            self.punct(";")?;
            return Ok(Item::Const(ConstDecl {
                name,
                ty,
                value,
                negative,
            }));
        }
        if self.eat_kw("units") {
            let name = self.name()?;
            let params = self.params()?;
            self.punct("=")?;
            let body = self.expr()?;
            self.punct(";")?;
            return Ok(Item::Units(UnitsDecl { name, params, body }));
        }
        if self.eat_kw("category") {
            let name = self.name()?;
            let param = if self.eat_punct("(") {
                let p = self.name()?;
                self.punct(":")?;
                let t = self.ty()?;
                self.punct(")")?;
                Some((p, t))
            } else {
                None
            };
            let under = if self.eat_kw("under") {
                Some(self.name()?)
            } else {
                None
            };
            let gc_mutable = self.eat_kw("gc_mutable");
            self.punct(";")?;
            return Ok(Item::Category(CategoryDecl {
                name,
                param,
                under,
                gc_mutable,
            }));
        }
        if self.eat_kw("fact") {
            let name = self.name()?;
            self.kw("on")?;
            let mut on = vec![self.cat_ref()?];
            while self.eat_punct(",") {
                on.push(self.cat_ref()?);
            }
            self.punct(";")?;
            return Ok(Item::Fact(FactDecl { name, on }));
        }
        if self.eat_kw("layout") {
            let kind = self.name()?;
            let is = if self.eat_kw("is") {
                Some(self.name()?)
            } else {
                None
            };
            self.punct("{")?;
            let mut fields = Vec::new();
            while !self.eat_punct("}") {
                let name = self.name()?;
                self.punct(":")?;
                let ty = self.ty()?;
                let mutability = if self.eat_kw("immutable") {
                    Mutability::Immutable
                } else if self.eat_kw("in") {
                    Mutability::In(self.cat_ref()?)
                } else {
                    return self
                        .err("a field is `immutable` or names its category with `in <category>`");
                };
                let repr = self.eat_kw("repr");
                self.punct(";")?;
                fields.push(FieldDecl {
                    name,
                    ty,
                    mutability,
                    repr,
                });
            }
            return Ok(Item::Layout(LayoutDecl { kind, is, fields }));
        }
        if self.eat_kw("runtime") {
            let name = self.name()?;
            let params = self.params()?;
            self.punct("->")?;
            let ret = self.ty()?;
            let effects = self.effects()?;
            let facts = self.facts()?;
            let anchor = self.anchor()?;
            let ensures = if self.is_kw("ensures") {
                let span = self.span();
                self.at += 1;
                self.kw("result")?;
                self.kw("is")?;
                Some(Contract {
                    pattern: self.pattern()?,
                    span,
                })
            } else {
                None
            };
            self.punct(";")?;
            return Ok(Item::Runtime(RuntimeDecl {
                name,
                params,
                ret,
                effects,
                facts,
                anchor,
                ensures,
            }));
        }
        if self.eat_kw("helper") {
            let name = self.name()?;
            return Ok(Item::Helper(self.fn_rest(name, false)?));
        }
        if self.eat_kw("intrinsic") {
            let span = self.span();
            let Tok::Intrinsic(text) = self.peek().clone() else {
                return self.err("an intrinsic's name starts with `%`");
            };
            self.at += 1;
            return Ok(Item::Intrinsic(self.fn_rest(Name { text, span }, true)?));
        }
        if self.eat_kw("op") {
            let name = self.name()?;
            self.punct("(")?;
            let mut operands = Vec::new();
            if !self.eat_punct(")") {
                loop {
                    let oname = self.name()?;
                    self.punct(":")?;
                    let kind = self.operand_kind()?;
                    operands.push(Operand { name: oname, kind });
                    if self.eat_punct(")") {
                        break;
                    }
                    self.punct(",")?;
                }
            }
            self.kw("acc")?;
            let acc = if self.eat_kw("none") {
                AccMode::None
            } else if self.eat_kw("in") {
                AccMode::In
            } else if self.eat_kw("out") {
                AccMode::Out
            } else if self.eat_kw("inout") {
                AccMode::InOut
            } else {
                return self.err("expected `none`, `in`, `out` or `inout`");
            };
            let after_call = self.after_call()?;
            let effects = self.effects()?;
            let facts = self.facts()?;
            let anchor = self.anchor()?;
            let body = self.block()?;
            return Ok(Item::Op(OpDecl {
                name,
                operands,
                acc,
                after_call,
                effects,
                facts,
                anchor,
                body,
            }));
        }
        self.err(format!(
            "expected an item (`const`, `units`, `layout`, `runtime`, `helper`, `intrinsic` or `op`), found {}",
            describe(self.peek())
        ))
    }

    fn fn_rest(&mut self, name: Name, intrinsic: bool) -> R<FnDecl> {
        let params = self.params()?;
        self.punct("->")?;
        let ret = self.ty()?;
        let after_call = if intrinsic { self.after_call()? } else { None };
        let effects = self.effects()?;
        let facts = self.facts()?;
        let anchor = self.anchor()?;
        let body = self.block()?;
        Ok(FnDecl {
            name,
            params,
            ret,
            after_call,
            effects,
            facts,
            anchor,
            body,
        })
    }

    fn after_call(&mut self) -> R<Option<AfterCall>> {
        if !self.eat_kw("after_call") {
            return Ok(None);
        }
        if self.eat_kw("result") {
            return Ok(Some(AfterCall::Result));
        }
        if self.eat_kw("preserve") {
            return Ok(Some(AfterCall::Preserve));
        }
        self.err("expected `result` or `preserve`")
    }

    fn cat_ref(&mut self) -> R<CatRef> {
        let name = self.name()?;
        let arg = if self.eat_punct("(") {
            let a = match self.peek().clone() {
                Tok::Ident(s) => s,
                Tok::Int(v) => v.to_string(),
                other => {
                    return self.err(format!(
                        "a category parameter is a name or an integer, not {}",
                        describe(&other)
                    ));
                }
            };
            self.at += 1;
            self.punct(")")?;
            Some(a)
        } else {
            None
        };
        Ok(CatRef { name, arg })
    }

    fn facts(&mut self) -> R<Vec<FactClause>> {
        let mut out = Vec::new();
        loop {
            if self.eat_kw("invalidates") || self.is_kw("results") {
                let results = self.eat_kw("results");
                self.punct("(")?;
                let mut names = vec![self.name()?];
                while self.eat_punct(",") {
                    names.push(self.name()?);
                }
                self.punct(")")?;
                out.push(if results {
                    FactClause::Results(names)
                } else {
                    FactClause::Invalidates(names)
                });
            } else if self.eat_kw("transfer") {
                out.push(FactClause::Transfer(self.name()?));
            } else if self.eat_kw("smt") {
                out.push(FactClause::Smt(self.name()?));
            } else if self.is_kw("frame_charge") {
                let span = self.span();
                self.at += 1;
                let Tok::Int(v) = self.peek().clone() else {
                    return self.err("`frame_charge` takes a number of canonical depth bytes");
                };
                self.at += 1;
                out.push(FactClause::FrameCharge(
                    u64::try_from(v).unwrap_or(u64::MAX),
                    span,
                ));
            } else {
                return Ok(out);
            }
        }
    }

    fn params(&mut self) -> R<Vec<Param>> {
        self.punct("(")?;
        let mut out = Vec::new();
        if self.eat_punct(")") {
            return Ok(out);
        }
        loop {
            let name = self.name()?;
            self.punct(":")?;
            let frame = self.eat_kw("frame");
            let ty = self.ty()?;
            out.push(Param { name, frame, ty });
            if self.eat_punct(")") {
                return Ok(out);
            }
            self.punct(",")?;
        }
    }

    fn ty(&mut self) -> R<TypeExpr> {
        let span = self.span();
        if self.is_kw("Ref") && matches!(self.peek_at(1), Tok::Punct("<")) {
            self.at += 2;
            let k = self.name()?;
            self.punct(">")?;
            return Ok(TypeExpr::Ref(k));
        }
        if self.is_kw("Empty") && matches!(self.peek_at(1), Tok::Punct("|")) {
            self.at += 2;
            self.kw("Value")?;
            return Ok(TypeExpr::Weak(span));
        }
        Ok(TypeExpr::Named(self.name()?))
    }

    fn literal(&mut self) -> R<Literal> {
        match self.peek().clone() {
            Tok::Int(v) => {
                self.at += 1;
                Ok(Literal::Int(v))
            }
            Tok::Float(f) => {
                self.at += 1;
                Ok(Literal::Float(f))
            }
            Tok::Ident(s) if s == "true" || s == "false" => {
                self.at += 1;
                Ok(Literal::Bool(s == "true"))
            }
            other => self.err(format!("expected a literal, found {}", describe(&other))),
        }
    }

    fn operand_kind(&mut self) -> R<OperandKind> {
        let n = self.name()?;
        Ok(match n.text.as_str() {
            "reg" => OperandKind::Reg,
            "cnt" => OperandKind::Cnt,
            "imm" => OperandKind::Imm,
            "fwd" => OperandKind::Fwd,
            "back" => OperandKind::Back,
            "intr" => OperandKind::Intr,
            "const" => {
                self.punct("<")?;
                let mut kinds = vec![self.name()?];
                while self.eat_punct("|") {
                    kinds.push(self.name()?);
                }
                self.punct(">")?;
                OperandKind::Const(kinds)
            }
            "slot" => {
                self.punct("<")?;
                // Slot kinds such as `keyed-load` contain a hyphen.
                let mut k = self.name()?;
                while self.eat_punct("-") {
                    let more = self.name()?;
                    k.text.push('-');
                    k.text.push_str(&more.text);
                }
                self.punct(">")?;
                OperandKind::Slot(k)
            }
            other => {
                return Err(Diag::new(
                    n.span,
                    "OC-13",
                    format!("unknown operand kind `{other}`"),
                ));
            }
        })
    }

    fn effects(&mut self) -> R<Vec<Effect>> {
        self.kw("effects")?;
        self.punct("{")?;
        let mut out = Vec::new();
        if self.eat_punct("}") {
            return Ok(out);
        }
        loop {
            out.push(self.effect()?);
            if self.eat_punct("}") {
                return Ok(out);
            }
            self.punct(",")?;
        }
    }

    fn effect(&mut self) -> R<Effect> {
        let n = self.name()?;
        let span = n.span;
        Ok(match n.text.as_str() {
            "alloc" => {
                self.punct("(")?;
                let u = self.name()?;
                self.punct(")")?;
                Effect::Alloc(u)
            }
            "alloc_internal" => Effect::AllocInternal(span),
            "reads" | "writes" => {
                self.punct("(")?;
                let mut cats = Vec::new();
                loop {
                    cats.push(self.cat_ref()?);
                    if self.eat_punct(")") {
                        break;
                    }
                    self.punct(",")?;
                }
                if n.text == "reads" {
                    Effect::Reads(cats, span)
                } else {
                    Effect::Writes(cats, span)
                }
            }
            "throw" => {
                self.punct("(")?;
                let mut kinds = Vec::new();
                loop {
                    kinds.push(self.name()?);
                    if self.eat_punct(")") {
                        break;
                    }
                    self.punct(",")?;
                }
                Effect::Throw(kinds)
            }
            "call_user" => Effect::CallUser(span),
            "host" => {
                self.punct("(")?;
                let h = self.name()?;
                self.punct(")")?;
                Effect::Host(h)
            }
            "input" => {
                self.punct("(")?;
                let k = self.name()?;
                self.punct(")")?;
                match k.text.as_str() {
                    "entropy" => Effect::Input(InputKind::Entropy, span),
                    "clock" => Effect::Input(InputKind::Clock, span),
                    _ => {
                        return Err(Diag::new(
                            k.span,
                            "OC-11",
                            "an input is `entropy` or `clock`",
                        ));
                    }
                }
            }
            "interrupt" => Effect::Interrupt(span),
            other => return Err(Diag::new(span, "OC-1", format!("unknown effect `{other}`"))),
        })
    }

    fn anchor(&mut self) -> R<Anchor> {
        let span = self.span();
        self.kw("anchor")?;
        let s = self.name()?;
        if s.text == "izcal" {
            let reason = self.string()?;
            return Ok(Anchor::Izcal { reason, span });
        }
        let Some(spec) = Spec::parse(&s.text) else {
            return Err(Diag::new(
                s.span,
                "OC-12",
                format!("unknown specification `{}`", s.text),
            ));
        };
        let clause = self.string()?;
        let step = if self.eat_kw("step") {
            Some(self.string()?)
        } else {
            None
        };
        Ok(Anchor::Spec {
            spec,
            clause,
            step,
            span,
        })
    }

    // --- Blocks and statements ----------------------------------------

    fn block(&mut self) -> R<Block> {
        let span = self.span();
        self.punct("{")?;
        let mut stmts = Vec::new();
        loop {
            if self.eat_punct("}") {
                return Ok(Block {
                    stmts,
                    tail: None,
                    span,
                });
            }
            match self.stmt_or_tail()? {
                StmtOrTail::Stmt(s) => stmts.push(s),
                StmtOrTail::Tail(e) => {
                    self.punct("}")?;
                    return Ok(Block {
                        stmts,
                        tail: Some(Box::new(e)),
                        span,
                    });
                }
            }
        }
    }

    fn stmt_or_tail(&mut self) -> R<StmtOrTail> {
        let span = self.span();
        if self.eat_kw("let") {
            let name = self.name()?;
            let ty = if self.eat_punct(":") {
                Some(self.ty()?)
            } else {
                None
            };
            self.punct("=")?;
            let value = self.expr()?;
            self.punct(";")?;
            return Ok(StmtOrTail::Stmt(Stmt::Let { name, ty, value }));
        }
        if self.is_kw("acc") && matches!(self.peek_at(1), Tok::Punct("=")) {
            self.at += 2;
            let value = self.expr()?;
            self.punct(";")?;
            return Ok(StmtOrTail::Stmt(Stmt::Assign {
                place: Place::Acc(span),
                value,
            }));
        }
        if self.is_kw("reg") && matches!(self.peek_at(1), Tok::Punct("[")) {
            // `reg[x] = e;` or an expression starting with `reg[x]`.
            let save = self.at;
            self.at += 2;
            let r = self.name()?;
            self.punct("]")?;
            if self.eat_punct("=") {
                let value = self.expr()?;
                self.punct(";")?;
                return Ok(StmtOrTail::Stmt(Stmt::Assign {
                    place: Place::Reg(r),
                    value,
                }));
            }
            self.at = save;
        }
        if self.is_kw("if") {
            return Ok(StmtOrTail::Stmt(self.if_stmt()?));
        }
        if self.is_kw("match") {
            let m = self.match_()?;
            if self.is_punct("}") {
                return Ok(StmtOrTail::Tail(Expr::Match(m)));
            }
            return Ok(StmtOrTail::Stmt(Stmt::Match(m)));
        }
        if self.is_kw("path") {
            return Ok(StmtOrTail::Stmt(Stmt::Path(self.path()?)));
        }
        if self.eat_kw("charge") {
            self.punct("(")?;
            let units = self.name()?;
            let args = self.args()?;
            self.punct(")")?;
            self.punct(";")?;
            return Ok(StmtOrTail::Stmt(Stmt::Charge { units, args, span }));
        }
        if let Some(t) = self.terminal()? {
            return Ok(StmtOrTail::Stmt(Stmt::Terminal(t)));
        }
        let e = self.expr()?;
        if self.eat_punct(";") {
            return Ok(StmtOrTail::Stmt(Stmt::Expr(e)));
        }
        if self.is_punct("}") {
            return Ok(StmtOrTail::Tail(e));
        }
        self.err(format!(
            "expected `;` or `}}` after an expression, found {}",
            describe(self.peek())
        ))
    }

    fn terminal(&mut self) -> R<Option<Terminal>> {
        let span = self.span();
        if self.eat_kw("next") {
            self.punct(";")?;
            return Ok(Some(Terminal::Next(span)));
        }
        if self.eat_kw("jump") {
            let t = self.name()?;
            self.punct(";")?;
            return Ok(Some(Terminal::Jump(t)));
        }
        if self.eat_kw("return") {
            if self.eat_punct(";") {
                return Ok(Some(Terminal::Return(None, span)));
            }
            let e = self.expr()?;
            self.punct(";")?;
            return Ok(Some(Terminal::Return(Some(Box::new(e)), span)));
        }
        if self.is_kw("throw") {
            self.at += 1;
            let k = self.name()?;
            self.punct(";")?;
            return Ok(Some(Terminal::Throw(k)));
        }
        if self.is_kw("call_user") {
            self.at += 1;
            let k = self.name()?;
            let Some(kind) = CallKind::parse(&k.text) else {
                return Err(Diag::new(
                    k.span,
                    "OC-17",
                    "a tail user call is `call`, `construct`, `getter`, `setter` or `iter_step`",
                ));
            };
            let args = self.args()?;
            self.punct(";")?;
            return Ok(Some(Terminal::TailCall(kind, args, span)));
        }
        Ok(None)
    }

    fn if_stmt(&mut self) -> R<Stmt> {
        let span = self.span();
        self.kw("if")?;
        let cond = self.expr()?;
        let then = self.block()?;
        let els = if self.eat_kw("else") {
            if self.is_kw("if") {
                let s = self.span();
                let inner = self.if_stmt()?;
                Some(Block {
                    stmts: vec![inner],
                    tail: None,
                    span: s,
                })
            } else {
                Some(self.block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If {
            cond,
            then,
            els,
            span,
        })
    }

    fn path(&mut self) -> R<PathDecl> {
        self.kw("path")?;
        let name = self.name()?;
        let mut anchor = None;
        let mut speculate = None;
        loop {
            if self.is_kw("anchor") {
                if anchor.is_some() {
                    return Err(Diag::new(
                        self.span(),
                        "OC-12",
                        "a path has exactly one anchor",
                    ));
                }
                anchor = Some(self.anchor()?);
            } else if self.eat_kw("speculate") {
                let slot = self.name()?;
                self.punct(".")?;
                let pred = self.name()?;
                speculate = Some((slot, pred));
            } else {
                break;
            }
        }
        let body = self.block()?;
        Ok(PathDecl {
            name,
            anchor,
            speculate,
            body,
        })
    }

    fn match_(&mut self) -> R<Match> {
        let span = self.span();
        self.kw("match")?;
        let scrutinee = Box::new(self.expr()?);
        self.punct("{")?;
        let mut arms = Vec::new();
        while !self.eat_punct("}") {
            let pattern = self.pattern()?;
            let guard = if self.eat_kw("if") {
                Some(self.expr()?)
            } else {
                None
            };
            self.punct("=>")?;
            let body = if self.is_kw("path") {
                let p = self.path()?;
                self.eat_punct(",");
                ArmBody::Path(p)
            } else if self.is_punct("{") {
                let b = self.block()?;
                self.eat_punct(",");
                ArmBody::Block(b)
            } else if let Some(t) = self.terminal()? {
                ArmBody::Terminal(t)
            } else {
                let e = self.expr()?;
                if !self.is_punct("}") {
                    self.punct(",")?;
                }
                ArmBody::Expr(e)
            };
            arms.push(Arm {
                pattern,
                guard,
                body,
            });
        }
        if arms.is_empty() {
            return Err(Diag::new(span, "OC-2", "a match needs at least one arm"));
        }
        Ok(Match {
            scrutinee,
            arms,
            span,
        })
    }

    fn pattern(&mut self) -> R<Pattern> {
        let first = self.pattern1()?;
        if !self.is_punct("|") {
            return Ok(first);
        }
        let mut alts = vec![first];
        while self.eat_punct("|") {
            alts.push(self.pattern1()?);
        }
        Ok(Pattern::Or(alts))
    }

    fn pattern1(&mut self) -> R<Pattern> {
        let span = self.span();
        if self.eat_punct("(") {
            let mut ps = vec![self.pattern()?];
            while self.eat_punct(",") {
                ps.push(self.pattern()?);
            }
            self.punct(")")?;
            if ps.len() < 2 {
                return Err(Diag::new(
                    span,
                    "OC-2",
                    "a tuple pattern has two or more parts",
                ));
            }
            return Ok(Pattern::Tuple(ps, span));
        }
        let n = self.name()?;
        Ok(match n.text.as_str() {
            "_" => Pattern::Wild(span),
            "Undefined" | "Null" | "True" | "False" | "Empty" => Pattern::Oddball(n),
            "Smi" | "Num" | "Value" => {
                self.punct("(")?;
                let b = self.name()?;
                self.punct(")")?;
                match n.text.as_str() {
                    "Smi" => Pattern::Smi(b),
                    "Num" => Pattern::Num(b),
                    _ => Pattern::Value(b),
                }
            }
            "Ref" => {
                self.punct("<")?;
                let k = self.name()?;
                self.punct(">")?;
                self.punct("(")?;
                let b = self.name()?;
                self.punct(")")?;
                Pattern::Ref(k, b)
            }
            other => {
                return Err(Diag::new(
                    n.span,
                    "OC-2",
                    format!("unknown pattern `{other}`"),
                ));
            }
        })
    }

    // --- Expressions ---------------------------------------------------

    fn args(&mut self) -> R<Vec<Expr>> {
        self.punct("(")?;
        let mut out = Vec::new();
        if self.eat_punct(")") {
            return Ok(out);
        }
        loop {
            out.push(self.expr()?);
            if self.eat_punct(")") {
                return Ok(out);
            }
            self.punct(",")?;
        }
    }

    fn expr(&mut self) -> R<Expr> {
        self.binary(1)
    }

    fn binop(&self) -> Option<(BinOp, u8)> {
        let Tok::Punct(p) = self.peek() else {
            return None;
        };
        Some(match *p {
            "||" => (BinOp::Or, 1),
            "&&" => (BinOp::And, 2),
            "==" => (BinOp::Eq, 3),
            "!=" => (BinOp::Ne, 3),
            "<" => (BinOp::Lt, 4),
            "<=" => (BinOp::Le, 4),
            ">" => (BinOp::Gt, 4),
            ">=" => (BinOp::Ge, 4),
            "|" => (BinOp::BitOr, 5),
            "^" => (BinOp::BitXor, 6),
            "&" => (BinOp::BitAnd, 7),
            "<<" => (BinOp::Shl, 8),
            ">>" => (BinOp::Sar, 8),
            ">>>" => (BinOp::Shr, 8),
            "+" => (BinOp::Add, 9),
            "-" => (BinOp::Sub, 9),
            "*" => (BinOp::Mul, 10),
            "/" => (BinOp::Div, 10),
            "%" => (BinOp::Rem, 10),
            _ => return None,
        })
    }

    fn binary(&mut self, min: u8) -> R<Expr> {
        let mut lhs = self.unary()?;
        while let Some((op, prec)) = self.binop() {
            if prec < min {
                break;
            }
            let span = self.span();
            self.at += 1;
            let rhs = self.binary(prec + 1)?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs), span);
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> R<Expr> {
        let span = self.span();
        if self.eat_punct("!") {
            return Ok(Expr::Unary(UnOp::Not, Box::new(self.unary()?), span));
        }
        if self.eat_punct("-") {
            return Ok(Expr::Unary(UnOp::Neg, Box::new(self.unary()?), span));
        }
        if self.eat_punct("~") {
            return Ok(Expr::Unary(UnOp::BitNot, Box::new(self.unary()?), span));
        }
        self.primary()
    }

    fn primary(&mut self) -> R<Expr> {
        let span = self.span();
        match self.peek().clone() {
            Tok::Int(_) | Tok::Float(_) => {
                let l = self.literal()?;
                return Ok(Expr::Lit(l, span));
            }
            Tok::Punct("(") => {
                self.at += 1;
                let first = self.expr()?;
                if self.eat_punct(")") {
                    return Ok(first);
                }
                let mut parts = vec![first];
                while self.eat_punct(",") {
                    parts.push(self.expr()?);
                }
                self.punct(")")?;
                return Ok(Expr::Tuple(parts, span));
            }
            Tok::Punct("{") => return Ok(Expr::Block(self.block()?)),
            Tok::Ident(_) => {}
            other => {
                return self.err(format!(
                    "expected an expression, found {}",
                    describe(&other)
                ));
            }
        }
        if self.is_kw("true") || self.is_kw("false") {
            let l = self.literal()?;
            return Ok(Expr::Lit(l, span));
        }
        if self.is_kw("acc") {
            self.at += 1;
            return Ok(Expr::Acc(span));
        }
        if (self.is_kw("reg") || self.is_kw("const")) && matches!(self.peek_at(1), Tok::Punct("["))
        {
            let is_reg = self.is_kw("reg");
            self.at += 2;
            let n = self.name()?;
            self.punct("]")?;
            return Ok(if is_reg {
                Expr::Reg(n)
            } else {
                Expr::ConstPool(n)
            });
        }
        if self.is_kw("match") {
            return Ok(Expr::Match(self.match_()?));
        }
        if self.eat_kw("alloc") {
            let kind = self.name()?;
            self.punct("{")?;
            let mut fields = Vec::new();
            if !self.eat_punct("}") {
                loop {
                    let f = self.name()?;
                    self.punct(":")?;
                    let e = self.expr()?;
                    fields.push((f, e));
                    if self.eat_punct("}") {
                        break;
                    }
                    self.punct(",")?;
                }
            }
            return Ok(Expr::Alloc(kind, fields));
        }
        let n = self.name()?;
        if self.is_punct("(") {
            let args = self.args()?;
            return Ok(Expr::Call(n, args));
        }
        if self.is_punct(".") && matches!(self.peek_at(1), Tok::Ident(_)) {
            self.at += 1;
            let f = self.name()?;
            let args = self.args()?;
            return Ok(Expr::QCall(n, f, args));
        }
        Ok(Expr::Ident(n))
    }
}

enum StmtOrTail {
    Stmt(Stmt),
    Tail(Expr),
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Ident(s) => format!("`{s}`"),
        Tok::Intrinsic(s) => format!("`%{s}`"),
        Tok::Int(v) => format!("`{v}`"),
        Tok::Float(f) => format!("`{f}`"),
        Tok::Str(_) => "a string".to_string(),
        Tok::Punct(p) => format!("`{p}`"),
        Tok::Eof => "the end of the file".to_string(),
    }
}
