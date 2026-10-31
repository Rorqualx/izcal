//! Emits the reference interpreter (language note Section 7): the
//! `Runtime` trait whose signatures follow from declared effects (5.1),
//! one Rust function per operation, helper and intrinsic, the dispatch
//! loop, and the constants and units formulas.
//!
//! Representation in the generated Rust. A heap value is either *plain*
//! (`Value<'h, 'm>` or `Ref<'h, 'm, K>`, borrowed from the mutator, so it
//! cannot live across a GC point) or *rooted* (`Root<'h, 's, Any>` or
//! `Rooted<'h, 's, K>`, in the instruction's scope). A local is rooted
//! exactly when the flow analysis found it live across a GC point, which
//! OC-6 has already justified; every other local is plain, so a liveness
//! mistake in the generator is a borrow error in rustc (R0-O6's second
//! check). Arguments of a GC point are rooted at the call, as `Init` asks.
//! An intrinsic's parameters are re-read from the frame's argument list at
//! each use.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::ast::{Anchor, BinOp, UnOp};
use crate::flow::{Flows, ItemFlow};
use crate::model::*;
use crate::tir::*;

/// The generated file's text.
pub fn emit(m: &Model, flows: &Flows, header: &str) -> String {
    let mut o = String::new();
    o.push_str(header);
    o.push_str(
        "//! The reference interpreter (design review R0-O9): a plain `match`\n\
         //! loop over the opcode table, one function per operation, helper and\n\
         //! intrinsic, and the `Runtime` trait `hg-js-runtime` implements.\n\
         //! Every function names the ECMA-262 clause it implements (G9).\n\
         \n\
         #![allow(\n    \
         clippy::needless_return,\n    \
         clippy::nonminimal_bool,\n    \
         clippy::collapsible_else_if,\n    \
         clippy::collapsible_if,\n    \
         clippy::too_many_arguments,\n    \
         clippy::match_single_binding,\n    \
         clippy::let_and_return,\n    \
         clippy::bool_comparison,\n    \
         clippy::needless_late_init,\n    \
         clippy::unnecessary_cast,\n    \
         clippy::redundant_locals,\n    \
         clippy::let_unit_value\n\
         )]\n\
         // The allowances above are style lints that generated code trips by\n\
         // construction (every route ends in an explicit `return`, conditions\n\
         // are written as the descriptions write them); hg-opgen's output is\n\
         // reviewed as the generator (CS-ST-17 (f)).\n\n",
    );
    o.push_str(
        "use hg_heap::{Any, Mutator, Root, Scope, Value};\n\
         use hg_js_bytecode::{Instr, Intrinsic, Op, UnitView};\n\
         use hg_substrate::RandomStream;\n\n\
         use crate::heap::{self, Class, ObjectKind, Ref, Rooted, kind};\n\
         use crate::support::{\n    \
         CallKind, Completion, Cx, ErrorKind, Flow, Host, OutOfMemory, Reg, RegList, RuntimeBase, ScriptToken, Throw,\n\
         };\n\
         use crate::support::prim;\n\
         use super::defs::units;\n\n",
    );
    emit_trait(m, &mut o);
    emit_run(m, &mut o);
    for (i, f) in m.helpers.iter().enumerate() {
        let mut g = Gen::new(m, &f.body, &flows.helpers[i], Ctx::Helper(&f.sig));
        g.func_helper(f, i);
        o.push_str(&g.out);
    }
    emit_intrinsic_dispatch(m, &mut o);
    for (i, f) in m.intrinsics.iter().enumerate() {
        let mut g = Gen::new(m, &f.body, &flows.intrinsics[i], Ctx::Intrinsic(&f.sig));
        g.func_intrinsic(f);
        o.push_str(&g.out);
    }
    for (i, op) in m.ops.iter().enumerate() {
        let mut g = Gen::new(m, &op.body, &flows.ops[i], Ctx::Op(op));
        g.func_op(op);
        o.push_str(&g.out);
    }
    o
}

/// The constants and units formulas, as their own generated module.
pub fn emit_consts(m: &Model, header: &str) -> String {
    let mut o = String::new();
    o.push_str(header);
    o.push_str("//! The descriptions' constants and canonical-units formulas (R0-B10),\n//! shared by the generated interpreter and `hg-js-runtime`.\n\n");
    o.push_str("/// The descriptions' named constants.\npub mod consts {\n");
    for c in &m.consts {
        let (ty, v) = match (c.ty, c.value) {
            (t, ConstValue::Int(v)) => (rust_scalar(t), v.to_string()),
            (_, ConstValue::Float(f)) => ("f64", format!("{f:?}")),
            (_, ConstValue::Bool(b)) => ("bool", b.to_string()),
        };
        let _ = writeln!(
            o,
            "    /// `{}` (declared in the descriptions).\n    pub const {}: {ty} = {v};",
            c.name, c.name
        );
    }
    o.push_str("}\n\n/// Canonical-units formulas (bytecode note Section 6): the GC debt each\n/// semantic allocation charges, a function of semantic quantities only.\npub mod units {\n");
    for u in &m.units {
        let params: Vec<String> = u
            .params
            .iter()
            .map(|(n, t)| format!("{n}: {}", rust_scalar(*t)))
            .collect();
        let body = units_expr(&u.body, &u.params);
        let _ = writeln!(
            o,
            "    /// `{}` canonical units.\n    pub fn {}({}) -> u64 {{\n        let v: i128 = {body};\n        u64::try_from(v).unwrap_or(u64::MAX)\n    }}",
            u.name,
            u.name,
            params.join(", ")
        );
    }
    o.push_str("}\n");
    o
}

fn units_expr(e: &TExpr, params: &[(String, Ty)]) -> String {
    match &e.kind {
        TE::Int(v) => format!("{v}"),
        TE::Local(l) => format!("i128::from({})", params[l.0 as usize].0),
        TE::Bin(op, a, b) => format!(
            "{} {} {}",
            p(&units_expr(a, params)),
            op.text(),
            p(&units_expr(b, params))
        ),
        _ => "0".into(),
    }
}

fn rust_scalar(t: Ty) -> &'static str {
    match t {
        Ty::I32 => "i32",
        Ty::U32 => "u32",
        Ty::I64 => "i64",
        Ty::F64 => "f64",
        Ty::Bool => "bool",
        Ty::ObjectKind => "ObjectKind",
        _ => "()",
    }
}

fn anchor_text(a: &Anchor) -> String {
    match a {
        Anchor::Spec {
            spec, clause, step, ..
        } => {
            let base = match spec.name() {
                "ecma262" => "https://tc39.es/ecma262/",
                "temporal" => "https://tc39.es/proposal-temporal/",
                "ecma402" => "https://tc39.es/ecma402/",
                _ => "",
            };
            match step {
                Some(s) => format!("spec: {base}#{clause} step {s}"),
                None => format!("spec: {base}#{clause}"),
            }
        }
        Anchor::Izcal { reason, .. } => format!("izcal: {reason}"),
    }
}

// --- The Runtime trait -----------------------------------------------------

/// How a parameter or result of type `t` is passed in a function with or
/// without a GC point.
fn ty_rust(t: Ty, gc: bool, life: &str) -> String {
    match (t, gc) {
        (Ty::Value, true) => format!("Root<'h, '{life}, Any>"),
        (Ty::Value, false) => format!("Value<'h, '{life}>"),
        (Ty::Ref(k), true) => format!("Rooted<'h, '{life}, {}>", k.rust()),
        (Ty::Ref(k), false) => format!("Ref<'h, '{life}, {}>", k.rust()),
        (Ty::Key, true) => format!("Rooted<'h, '{life}, kind::Key>"),
        (Ty::Key, false) => format!("Ref<'h, '{life}, kind::Key>"),
        (Ty::Weak, _) => format!("Option<Value<'h, '{life}>>"),
        (t, _) => rust_scalar(t).to_string(),
    }
}

/// The signature's environment parameters, as declared effects ask.
fn env_params(e: &Effects) -> (bool, bool, bool, bool) {
    let gc = e.gc();
    let user = e.user();
    let host = !e.host.is_empty();
    let random = e.input.contains(&crate::ast::InputKind::Entropy);
    (gc, user, host, random)
}

fn emit_trait(m: &Model, o: &mut String) {
    o.push_str(
        "/// The runtime functions the descriptions call (one method per\n\
         /// `runtime` declaration). Each signature follows from the declared\n\
         /// effects (language note 5.1): without a GC point a method holds only\n\
         /// `&Mutator` and unrooted arguments, so it cannot allocate; without\n\
         /// `call_user` it has no `ScriptToken`, so it cannot run script; a hook\n\
         /// or the realm's entropy stream is passed only to a method that\n\
         /// declares it. rustc thereby checks the declarations a second time.\n\
         pub trait Runtime<'h>: RuntimeBase<'h> {\n",
    );
    for r in &m.runtimes {
        let e = &r.sig.effects;
        let (gc, user, host, random) = env_params(e);
        let reads_only = e.alloc.is_empty()
            && !e.alloc_internal
            && e.writes.is_empty()
            && !e.call_user
            && e.host.is_empty()
            && e.input.is_empty()
            && !e.interrupt
            && e.throws.is_empty();
        let life = if gc { "s" } else { "m" };
        let mut ps = vec![if reads_only {
            "&self".to_string()
        } else {
            "&mut self".to_string()
        }];
        if gc {
            ps.push("m: &mut Mutator<'h>".into());
            if user {
                ps.push("t: &mut ScriptToken".into());
            }
            ps.push("s: &Scope<'h, 's>".into());
        } else {
            ps.push("m: &'m Mutator<'h>".into());
        }
        if host {
            ps.push("host: &mut dyn Host<'h>".into());
        }
        if random {
            ps.push("random: &mut dyn RandomStream".into());
        }
        for (n, t) in &r.sig.params {
            ps.push(format!("{n}: {}", ty_rust(*t, gc, life)));
        }
        let ret = ty_rust(r.sig.ret, gc, life);
        let ret = if gc {
            if e.throws.is_empty() && !user {
                format!("Result<{ret}, OutOfMemory>")
            } else {
                format!("Result<{ret}, Throw<'h, 's>>")
            }
        } else {
            ret
        };
        let generics = if gc { "<'s>" } else { "<'m>" };
        let _ = writeln!(
            o,
            "    /// `{}`: {}. Effects: {{ {} }}.\n    fn {}{generics}({}) -> {ret};",
            r.name,
            anchor_text(&r.sig.anchor),
            e.text(),
            r.name,
            ps.join(", ")
        );
    }
    o.push_str("}\n\n");
}

// --- The dispatch loop ------------------------------------------------------

fn operand_arg(kind: &OperandKind, i: usize) -> String {
    match kind {
        OperandKind::Reg => format!("i.reg({i})"),
        OperandKind::Imm => format!("i.imm({i})"),
        OperandKind::Fwd => format!("i.forward_target({i})"),
        OperandKind::Back => format!("i.backward_target({i})"),
        OperandKind::Cnt | OperandKind::Intr | OperandKind::Const(_) | OperandKind::Slot(_) => {
            format!("i.index({i})")
        }
    }
}

fn emit_run(m: &Model, o: &mut String) {
    o.push_str(
        "/// Runs `unit` in `cx`'s frame until it returns or throws (bytecode\n\
         /// note 4.7: an abrupt completion leaves the unit). Each instruction runs\n\
         /// in its own root scope, so the roots an instruction makes die with it;\n\
         /// the frame, the realm and the constants outlive it.\n\
         pub fn run<'h, R: Runtime<'h>>(m: &mut Mutator<'h>, cx: &mut Cx<'_, 'h, '_, R>, unit: &UnitView<'_>) -> Completion {\n    \
         let code = unit.code();\n    \
         let mut pc = 0u32;\n    \
         loop {\n        \
         let Ok(i) = hg_js_bytecode::decode(code, pc) else {\n            \
         return Completion::BadBytecode(pc);\n        \
         };\n        \
         if !cx.admits(&i, unit) {\n            \
         return Completion::BadBytecode(pc);\n        \
         }\n        \
         let flow = m.scope(|m, s| {\n            \
         let r = dispatch(m, s, cx, &i);\n            \
         match r {\n                \
         Ok(f) => f,\n                \
         Err(e) => cx.frame.set_thrown(m, e),\n            \
         }\n        \
         });\n        \
         match flow {\n            \
         Flow::Next => pc = i.end,\n            \
         Flow::Jump(target) => pc = target,\n            \
         Flow::Return => return Completion::Normal,\n            \
         Flow::Throw => return Completion::Throw,\n            \
         Flow::Interrupt => return Completion::Interrupted,\n            \
         Flow::Bad => return Completion::BadBytecode(pc),\n        \
         }\n    \
         }\n\
         }\n\n",
    );
    o.push_str(
        "/// One instruction: the operation its opcode names.\n\
         fn dispatch<'h, 's, R: Runtime<'h>>(m: &mut Mutator<'h>, s: &Scope<'h, 's>, cx: &mut Cx<'_, 'h, '_, R>, i: &Instr) -> Result<Flow, Throw<'h, 's>> {\n    \
         match i.op {\n",
    );
    for op in &m.ops {
        let mut args = vec!["m".to_string(), "s".into(), "cx".into()];
        for (j, (_, k)) in op.operands.iter().enumerate() {
            args.push(operand_arg(k, j));
        }
        let _ = writeln!(
            o,
            "        Op::{} => op_{}({}),",
            op.name,
            snake(&op.name),
            args.join(", ")
        );
    }
    o.push_str("    }\n}\n\n");
}

fn emit_intrinsic_dispatch(m: &Model, o: &mut String) {
    o.push_str(
        "/// `CallIntrinsic`'s dispatch: the intrinsic the operand names, over the\n\
         /// argument list (the interpreter admitted the id and the arity).\n\
         fn call_intrinsic<'h, 's, R: Runtime<'h>>(m: &mut Mutator<'h>, s: &Scope<'h, 's>, cx: &mut Cx<'_, 'h, '_, R>, id: u32, list: RegList) -> Result<Root<'h, 's, Any>, Throw<'h, 's>> {\n    \
         match Intrinsic::from_index(id) {\n",
    );
    for f in &m.intrinsics {
        let gc = f.sig.effects.gc();
        let call = format!("intr_{}(m, s, cx, list)", snake(&f.name));
        if gc {
            let _ = writeln!(o, "        Some(Intrinsic::{}) => {call},", f.name);
        } else {
            let _ = writeln!(
                o,
                "        Some(Intrinsic::{}) => {{\n            let v = intr_{}(&*m, cx, list);\n            Ok(s.root_value(m, v))\n        }}",
                f.name,
                snake(&f.name)
            );
        }
    }
    o.push_str("        None => Err(Throw::BadBytecode),\n    }\n}\n\n");
}

// --- Bodies -----------------------------------------------------------------

#[derive(Clone, Copy)]
enum Ctx<'a> {
    Op(&'a Op),
    Helper(&'a Sig),
    Intrinsic(&'a Sig),
}

/// How a value is held in the generated Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rep {
    /// A scalar.
    Scalar,
    /// Borrowed from the mutator.
    Plain,
    /// Rooted in the instruction's scope.
    Rooted,
}

struct Gen<'a> {
    m: &'a Model,
    body: &'a Body,
    flow: &'a ItemFlow,
    ctx: Ctx<'a>,
    out: String,
    indent: usize,
    tmp: usize,
    /// How each local is held.
    store: BTreeMap<LocalId, Rep>,
    gc: bool,
}

impl<'a> Gen<'a> {
    fn new(m: &'a Model, body: &'a Body, flow: &'a ItemFlow, ctx: Ctx<'a>) -> Self {
        let gc = match ctx {
            Ctx::Op(_) => true,
            Ctx::Helper(s) | Ctx::Intrinsic(s) => s.effects.gc(),
        };
        Gen {
            m,
            body,
            flow,
            ctx,
            out: String::new(),
            indent: 1,
            tmp: 0,
            store: BTreeMap::new(),
            gc,
        }
    }

    fn line(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn fresh(&mut self, base: &str) -> String {
        self.tmp += 1;
        format!("{base}{}", self.tmp)
    }

    fn local_name(&self, l: LocalId) -> String {
        let loc = &self.body.locals[l.0 as usize];
        format!("l_{}_{}", loc.name, l.0)
    }

    fn sig(&self) -> Option<&'a Sig> {
        match self.ctx {
            Ctx::Helper(s) | Ctx::Intrinsic(s) => Some(s),
            Ctx::Op(_) => None,
        }
    }

    // --- Function headers ----------------------------------------------

    fn doc_paths(&mut self) {
        for p in &self.body.paths {
            let own = if p.own_anchor { "" } else { " (inherited)" };
            let spec = p
                .speculate
                .as_ref()
                .map(|(_, pr)| format!(", speculable on `{pr}`"))
                .unwrap_or_default();
            let _ = writeln!(
                self.out,
                "/// - path `{}`{spec}: {}{own}",
                p.name,
                anchor_text(&p.anchor)
            );
        }
    }

    fn func_op(&mut self, op: &'a Op) {
        let _ = writeln!(
            self.out,
            "/// `{}` (opcode {:#04x}): effects {{ {} }}.",
            op.name,
            op.opcode,
            op.effects.text()
        );
        self.doc_paths();
        let mut ps = vec![
            "m: &mut Mutator<'h>".to_string(),
            "s: &Scope<'h, 's>".into(),
            "cx: &mut Cx<'_, 'h, '_, R>".into(),
        ];
        for (j, (n, k)) in op.operands.iter().enumerate() {
            let used = self.operand_used(j);
            let name = if used {
                format!("o_{n}")
            } else {
                format!("_o_{n}")
            };
            let ty = match k {
                OperandKind::Reg => "Reg",
                OperandKind::Imm => "i32",
                _ => "u32",
            };
            ps.push(format!("{name}: {ty}"));
        }
        let _ = writeln!(
            self.out,
            "fn op_{}<'h, 's, R: Runtime<'h>>({}) -> Result<Flow, Throw<'h, 's>> {{",
            snake(&op.name),
            ps.join(", ")
        );
        let b = &self.body.block;
        self.block_stmts(b);
        self.out.push_str("}\n\n");
    }

    fn operand_used(&self, j: usize) -> bool {
        let mut used = false;
        crate::walk::block(&self.body.block, &mut |e| match &e.kind {
            TE::Reg(i) | TE::ConstPool(i, _) | TE::Imm(i) if *i == j => used = true,
            TE::Intrinsic { id, args, count } if *id == j || *args == j || *count == j => {
                used = true
            }
            _ => {}
        });
        if used {
            return true;
        }
        stmts_use_operand(&self.body.block, j)
    }

    fn func_helper(&mut self, f: &'a Func, _i: usize) {
        let e = &f.sig.effects;
        let _ = writeln!(
            self.out,
            "/// Helper `{}`: effects {{ {} }}.",
            f.name,
            e.text()
        );
        self.doc_paths();
        let mut ps = Vec::new();
        let life = if self.gc { "s" } else { "m" };
        if self.gc {
            ps.push("m: &mut Mutator<'h>".to_string());
            ps.push("s: &Scope<'h, 's>".into());
        } else {
            ps.push("m: &'m Mutator<'h>".to_string());
        }
        ps.push("cx: &mut Cx<'_, 'h, '_, R>".into());
        for (i, (_, t)) in f.sig.params.iter().enumerate() {
            let id = LocalId(u32::try_from(i).unwrap_or(0));
            let rep = if t.is_heap() {
                if self.gc { Rep::Rooted } else { Rep::Plain }
            } else {
                Rep::Scalar
            };
            self.store.insert(id, rep);
            let name = if self.body.locals[i].used {
                self.local_name(id)
            } else {
                format!("_{}", self.local_name(id))
            };
            ps.push(format!("{name}: {}", ty_rust(*t, self.gc, life)));
        }
        let generics = if self.gc { "'h, 's" } else { "'h, 'm" };
        let ret = self.ret_type(&f.sig);
        let _ = writeln!(
            self.out,
            "fn helper_{}<{generics}, R: Runtime<'h>>({}) -> {ret} {{",
            snake(&f.name),
            ps.join(", ")
        );
        self.fn_body();
        self.out.push_str("}\n\n");
    }

    fn func_intrinsic(&mut self, f: &'a Func) {
        let e = &f.sig.effects;
        let _ = writeln!(
            self.out,
            "/// Intrinsic `%{}`: effects {{ {} }}.",
            f.name,
            e.text()
        );
        self.doc_paths();
        for (i, _) in f.sig.params.iter().enumerate() {
            self.store
                .insert(LocalId(u32::try_from(i).unwrap_or(0)), Rep::Plain);
        }
        let list = if f.sig.params.is_empty() {
            "_list"
        } else {
            "list"
        };
        let ret = self.ret_type(&f.sig);
        if self.gc {
            let _ = writeln!(
                self.out,
                "fn intr_{}<'h, 's, R: Runtime<'h>>(m: &mut Mutator<'h>, s: &Scope<'h, 's>, cx: &mut Cx<'_, 'h, '_, R>, {list}: RegList) -> {ret} {{",
                snake(&f.name)
            );
        } else {
            let _ = writeln!(
                self.out,
                "fn intr_{}<'h, 'm, R: Runtime<'h>>(m: &'m Mutator<'h>, cx: &mut Cx<'_, 'h, '_, R>, {list}: RegList) -> {ret} {{",
                snake(&f.name)
            );
        }
        self.fn_body();
        self.out.push_str("}\n\n");
    }

    fn ret_type(&self, sig: &Sig) -> String {
        let life = if self.gc { "s" } else { "m" };
        let t = ty_rust(sig.ret, self.gc, life);
        if self.gc {
            format!("Result<{t}, Throw<'h, 's>>")
        } else {
            t
        }
    }

    fn fn_body(&mut self) {
        let b = &self.body.block;
        for s in &b.stmts {
            self.stmt(s);
        }
        if let Some(t) = &b.tail {
            if let TE::Match(mm) = &t.kind
                && diverges_match(mm)
            {
                let code = self.match_code(mm, None);
                self.line(&code);
                return;
            }
            let v = self.value_for_return(t);
            self.line(&v);
        }
    }

    /// The function's final value from expression `t`.
    fn value_for_return(&mut self, t: &'a TExpr) -> String {
        let ret = self.sig().map_or(Ty::Unit, |s| s.ret);
        let want = if ret.is_heap() && self.gc {
            Rep::Rooted
        } else if ret.is_heap() {
            Rep::Plain
        } else {
            Rep::Scalar
        };
        let code = self.expr_as(t, want, ret);
        if self.gc { format!("Ok({code})") } else { code }
    }

    // --- Statements ----------------------------------------------------

    fn block_stmts(&mut self, b: &'a TBlock) {
        for s in &b.stmts {
            self.stmt(s);
        }
    }

    fn want_for_local(&self, id: LocalId, ty: Ty) -> Rep {
        if !ty.is_heap() || ty == Ty::Weak {
            if ty == Ty::Weak {
                Rep::Plain
            } else {
                Rep::Scalar
            }
        } else if self.flow.rooted.contains(&id) {
            Rep::Rooted
        } else {
            Rep::Plain
        }
    }

    fn stmt(&mut self, s: &'a TStmt) {
        match s {
            TStmt::Let(id, e) => {
                let ty = self.body.locals[id.0 as usize].ty;
                let want = self.want_for_local(*id, ty);
                let code = self.expr_as(e, want, ty);
                self.store.insert(*id, want);
                let name = if self.body.locals[id.0 as usize].used {
                    self.local_name(*id)
                } else {
                    format!("_{}", self.local_name(*id))
                };
                self.line(&format!("let {name} = {code};"));
            }
            TStmt::SetAcc(e, _) => {
                let v = self.expr_as(e, Rep::Plain, Ty::Value);
                self.line(&format!("cx.frame.set_acc(m, {v});"));
            }
            TStmt::SetReg(r, e, _) => {
                let v = self.expr_as(e, Rep::Plain, Ty::Value);
                let reg = self.operand_name(*r);
                self.line(&format!("cx.frame.set_reg(m, {reg}, {v});"));
            }
            TStmt::If(c, t, e) => {
                let cond = self.expr_as(c, Rep::Scalar, Ty::Bool);
                self.line(&format!("if {cond} {{"));
                self.indent += 1;
                self.block_stmts(t);
                self.indent -= 1;
                match e {
                    Some(e) => {
                        self.line("} else {");
                        self.indent += 1;
                        self.block_stmts(e);
                        self.indent -= 1;
                        self.line("}");
                    }
                    None => self.line("}"),
                }
            }
            TStmt::Match(m) => {
                let code = self.match_code(m, None);
                self.line(&format!("{code};"));
            }
            TStmt::Path(p) => {
                let info = &self.body.paths[p.id];
                let text = format!("// path `{}`: {}", info.name, anchor_text(&info.anchor));
                self.line(&text);
                self.block_stmts(&p.body);
            }
            TStmt::Charge(u, args, _) => {
                let name = self.m.units[*u].name.clone();
                let mut a = Vec::new();
                for x in args {
                    a.push(self.expr_as(x, Rep::Scalar, x.ty));
                }
                self.line(&format!("cx.charge(m, units::{name}({}));", a.join(", ")));
            }
            TStmt::Expr(e) => {
                let code = self.expr_as(
                    e,
                    if e.ty.is_heap() {
                        Rep::Plain
                    } else {
                        Rep::Scalar
                    },
                    e.ty,
                );
                if e.ty == Ty::Unit {
                    self.line(&format!("{code};"));
                } else {
                    self.line(&format!("let _ = {code};"));
                }
            }
            TStmt::Term(t) => self.term(t),
        }
    }

    fn term(&mut self, t: &'a TTerm) {
        match t {
            TTerm::Next(_) => self.line("return Ok(Flow::Next);"),
            TTerm::Jump(op, _, _) => {
                let n = self.operand_name(*op);
                self.line(&format!("return Ok(Flow::Jump({n}));"));
            }
            TTerm::Return(None, _) => self.line("return Ok(Flow::Return);"),
            TTerm::Return(Some(e), _) => {
                let v = self.value_for_return(e);
                self.line(&format!("return {v};"));
            }
            TTerm::Throw(k, _) => {
                self.line("cx.charge(m, units::error_units());");
                self.line(&format!("return Err(Throw::Error(ErrorKind::{k}));"));
            }
            TTerm::TailCall(kind, args, _) => {
                let mut roots = Vec::new();
                for a in args {
                    let v = self.expr_as(a, Rep::Rooted, Ty::Value);
                    let n = self.fresh("a");
                    self.line(&format!("let {n} = {v};"));
                    roots.push(n);
                }
                let (callee, rest) = roots
                    .split_first()
                    .map_or((String::new(), &[][..]), |(c, r)| (c.clone(), r));
                let (this, args) = rest
                    .split_first()
                    .map_or((String::new(), &[][..]), |(c, r)| (c.clone(), r));
                let r = self.fresh("t");
                self.line(&format!(
                    "let {r} = cx.rt.tail_call(m, cx.t, s, {}, {callee}, {this}, &[{}])?;",
                    call_kind_rust(*kind),
                    args.join(", ")
                ));
                self.line(&format!("cx.frame.set_acc(m, {r}.get_value(m));"));
                self.line("return Ok(Flow::Next);");
            }
        }
    }

    fn operand_name(&self, j: usize) -> String {
        match self.ctx {
            Ctx::Op(op) => format!("o_{}", op.operands[j].0),
            _ => "?".into(),
        }
    }

    // --- Matches -------------------------------------------------------

    /// A Rust `match` for `m`; `want` is the representation of a value
    /// match (and its type), `None` for a statement match.
    fn match_code(&mut self, m: &'a TMatch, want: Option<(Rep, Ty)>) -> String {
        // Bind the scrutinees and classify them.
        let mut scr = Vec::new();
        for s in &m.scrutinees {
            let v = self.expr_as(
                s,
                Rep::Plain,
                if s.ty == Ty::Weak {
                    Ty::Weak
                } else {
                    Ty::Value
                },
            );
            let n = self.fresh("v");
            self.line(&format!("let {n} = {v};"));
            scr.push((n, s.ty == Ty::Weak));
        }
        let heads: Vec<String> = scr
            .iter()
            .map(|(n, weak)| {
                if *weak {
                    n.clone()
                } else {
                    format!("heap::classify(m, {n})")
                }
            })
            .collect();
        let head = if heads.len() == 1 {
            heads[0].clone()
        } else {
            format!("({})", heads.join(", "))
        };
        let mut o = format!("match {head} {{\n");
        let saved_indent = self.indent;
        for arm in &m.arms {
            let mut prelude: Vec<(LocalId, String)> = Vec::new();
            let mut pats = Vec::new();
            for (k, (p, (sname, weak))) in arm.pats.iter().zip(&scr).enumerate() {
                let at = format!("c{k}_{}", self.tmp + 1);
                pats.push(self.pattern(p, *weak, sname, &at, &mut prelude));
            }
            self.tmp += 1;
            let pat = if pats.len() == 1 {
                pats[0].clone()
            } else {
                format!("({})", pats.join(", "))
            };
            let mut guard = String::new();
            if let Some(g) = &arm.guard {
                let saved = std::mem::take(&mut self.out);
                let gi = self.indent;
                self.indent = 0;
                let gcode = self.expr_as(g, Rep::Scalar, Ty::Bool);
                let extra = std::mem::replace(&mut self.out, saved);
                self.indent = gi;
                let pre: String = prelude
                    .iter()
                    .filter(|(id, _)| {
                        let n = self.local_name(*id);
                        gcode.contains(&n) || extra.contains(&n)
                    })
                    .map(|(_, p)| format!("{p} "))
                    .collect();
                guard = format!(" if {{ {pre}{}{gcode} }}", extra.replace('\n', " "));
            }
            // The body, generated into its own buffer.
            let saved = std::mem::take(&mut self.out);
            self.indent = saved_indent + 2;
            for (id, p) in &prelude {
                if self.prelude_needed(*id, arm) {
                    self.line(p);
                }
            }
            self.root_binders(arm);
            let value = match &arm.body {
                TArmBody::Path(p) => {
                    let info = &self.body.paths[p.id];
                    let text = format!("// path `{}`: {}", info.name, anchor_text(&info.anchor));
                    self.line(&text);
                    self.block_stmts(&p.body);
                    p.body.tail.as_ref().map(|t| self.tail_value(t, want))
                }
                TArmBody::Block(b) => {
                    self.block_stmts(b);
                    b.tail.as_ref().map(|t| self.tail_value(t, want))
                }
                TArmBody::Term(t) => {
                    self.term(t);
                    None
                }
                TArmBody::Expr(e) => {
                    if want.is_some() {
                        Some(self.tail_value(e, want))
                    } else {
                        let c = self.expr_as(e, Rep::Scalar, e.ty);
                        self.line(&format!("{c};"));
                        None
                    }
                }
            };
            if let Some(v) = value {
                self.line(&v);
            }
            let body = std::mem::replace(&mut self.out, saved);
            self.indent = saved_indent;
            let pad = "    ".repeat(saved_indent + 1);
            let _ = write!(o, "{pad}{pat}{guard} => {{\n{body}{pad}}}\n");
        }
        let pad = "    ".repeat(saved_indent);
        let _ = write!(o, "{pad}}}");
        o
    }

    fn tail_value(&mut self, t: &'a TExpr, want: Option<(Rep, Ty)>) -> String {
        let (rep, ty) = want.unwrap_or((Rep::Scalar, t.ty));
        self.expr_as(t, rep, ty)
    }

    fn prelude_needed(&self, id: LocalId, arm: &TArm) -> bool {
        let id = id.0;
        let mut used = false;
        let mut f = |e: &TExpr| {
            if let TE::Local(l) = e.kind
                && l.0 == id
            {
                used = true;
            }
        };
        match &arm.body {
            TArmBody::Path(p) => crate::walk::block(&p.body, &mut f),
            TArmBody::Block(b) => crate::walk::block(b, &mut f),
            TArmBody::Expr(e) => crate::walk::expr(e, &mut f),
            TArmBody::Term(TTerm::Return(Some(e), _)) => crate::walk::expr(e, &mut f),
            TArmBody::Term(TTerm::TailCall(_, args, _)) => {
                for a in args {
                    crate::walk::expr(a, &mut f);
                }
            }
            TArmBody::Term(_) => {}
        }
        used
    }

    /// Roots the arm's heap binders that are live across a GC point.
    fn root_binders(&mut self, arm: &TArm) {
        let mut ids = Vec::new();
        for p in &arm.pats {
            collect_binders(p, &mut ids);
        }
        for id in ids {
            let ty = self.body.locals[id.0 as usize].ty;
            if !ty.is_heap() {
                self.store.insert(id, Rep::Scalar);
                continue;
            }
            let rep = self.want_for_local(id, ty);
            let used = self.body.locals[id.0 as usize].used;
            if rep == Rep::Rooted && used {
                let n = self.local_name(id);
                let r = match ty {
                    Ty::Value => format!("let {n} = s.root_value(m, {n});"),
                    _ => format!("let {n} = {n}.root(m, s);"),
                };
                self.line(&r);
            }
            self.store.insert(id, if used { rep } else { Rep::Plain });
        }
    }

    /// The Rust pattern for `p`; binders that need extraction add a `let`
    /// to `prelude`. `at` is a fresh name for an `@` binding.
    fn pattern(
        &mut self,
        p: &TPat,
        weak: bool,
        scr: &str,
        at: &str,
        prelude: &mut Vec<(LocalId, String)>,
    ) -> String {
        if weak {
            return match p {
                TPat::Empty => "None".into(),
                TPat::Value(Some(id)) => {
                    let n = self.binder(*id);
                    format!("Some({n})")
                }
                TPat::Value(None) => "Some(_)".into(),
                _ => "_".into(),
            };
        }
        match p {
            TPat::Wild => "_".into(),
            TPat::Oddball(o) => format!("Class::{}", o.name()),
            TPat::Empty => "_".into(),
            TPat::Smi(None) => "Class::Smi(_)".into(),
            TPat::Smi(Some(id)) => {
                let n = self.binder(*id);
                format!("Class::Smi({n})")
            }
            TPat::Num(b) => {
                if let Some(id) = b {
                    let n = self.local_name(*id);
                    prelude.push((*id, format!("let {n} = {at}.number(m);")));
                    format!("{at} @ (Class::Smi(_) | Class::HeapNumber(_))")
                } else {
                    "Class::Smi(_) | Class::HeapNumber(_)".into()
                }
            }
            TPat::Ref(k, b) => {
                let variants: Vec<&str> = match k {
                    Kind::Object => vec!["Dict", "WeakRef", "FinReg", "HostFn"],
                    other => vec![other.name()],
                };
                match (b, variants.len()) {
                    (Some(id), 1) => {
                        let n = self.binder(*id);
                        format!("Class::{}({n})", variants[0])
                    }
                    (Some(id), _) => {
                        let n = self.local_name(*id);
                        prelude.push((*id, format!("let {n} = {at}.object();")));
                        let alts: Vec<String> =
                            variants.iter().map(|v| format!("Class::{v}(_)")).collect();
                        format!("{at} @ ({})", alts.join(" | "))
                    }
                    (None, _) => {
                        let alts: Vec<String> =
                            variants.iter().map(|v| format!("Class::{v}(_)")).collect();
                        alts.join(" | ")
                    }
                }
            }
            TPat::Value(b) => {
                if let Some(id) = b {
                    let n = self.local_name(*id);
                    prelude.push((*id, format!("let {n} = {scr};")));
                }
                "_".into()
            }
            TPat::Or(ps) => {
                let alts: Vec<String> = ps
                    .iter()
                    .map(|q| self.pattern(q, false, scr, at, prelude))
                    .collect();
                alts.join(" | ")
            }
        }
    }

    fn binder(&self, id: LocalId) -> String {
        if self.body.locals[id.0 as usize].used {
            self.local_name(id)
        } else {
            "_".into()
        }
    }

    // --- Expressions ---------------------------------------------------

    /// `e` as Rust code in representation `want`, of type `ty` (the static
    /// type the context expects: an upcast is made when it differs).
    fn expr_as(&mut self, e: &'a TExpr, want: Rep, ty: Ty) -> String {
        let (code, rep) = self.expr(e, want, ty);
        let code = self.upcast(code, e.ty, ty, rep);
        convert(code, rep, want, ty)
    }

    fn upcast(&self, code: String, from: Ty, to: Ty, rep: Rep) -> String {
        match (from, to) {
            (Ty::Ref(a), Ty::Ref(b)) if a != b => format!("{code}.upcast::<{}>()", b.rust()),
            (Ty::Ref(_) | Ty::Key, Ty::Value) => match rep {
                Rep::Rooted => format!("{code}.erase()"),
                _ => format!("{code}.value()"),
            },
            _ => code,
        }
    }

    /// `e` as Rust code, in whichever representation it naturally has.
    fn expr(&mut self, e: &'a TExpr, want: Rep, ty: Ty) -> (String, Rep) {
        let scalar = |c: String| (c, Rep::Scalar);
        match &e.kind {
            TE::Int(v) => scalar(int_lit(*v, e.ty)),
            TE::Float(f) => scalar(float_lit(*f)),
            TE::Bool(b) => scalar(b.to_string()),
            TE::ObjKind(k) => scalar(format!("ObjectKind::{}", k.name())),
            TE::UnitStrict => scalar("cx.strict".into()),
            TE::Imm(i) => scalar(self.operand_name(*i)),
            TE::Local(l) => self.local_use(*l),
            TE::Acc => ("cx.frame.acc(m)".into(), Rep::Plain),
            TE::Reg(r) => {
                let n = self.operand_name(*r);
                (format!("cx.frame.reg(m, {n})"), Rep::Plain)
            }
            TE::ConstPool(i, key) => {
                let n = self.operand_name(*i);
                if *key {
                    (format!("cx.consts.key(m, {n})"), Rep::Plain)
                } else {
                    (format!("cx.consts.get(m, {n})"), Rep::Plain)
                }
            }
            TE::Oddball(o) => (
                // `true` and `false` are keywords: the realm's singletons
                // are `true_value` and `false_value`.
                match o.name() {
                    "True" => "cx.realm.true_value(m)".to_string(),
                    "False" => "cx.realm.false_value(m)".to_string(),
                    n => format!("cx.realm.{}(m)", snake(n)),
                },
                Rep::Plain,
            ),
            TE::Runtime(r, args) => self.runtime_call(*r, args),
            TE::Helper(h, args) => self.helper_call(*h, args),
            TE::Intrinsic { id, args, count } => {
                let id = self.operand_name(*id);
                let a = self.operand_name(*args);
                let c = self.operand_name(*count);
                let t = self.fresh("t");
                self.line(&format!(
                    "let {t} = call_intrinsic(m, s, cx, {id}, RegList::new({a}, {c}))?;"
                ));
                (t, Rep::Rooted)
            }
            TE::Prim(p, args) => {
                let a: Vec<String> = args
                    .iter()
                    .map(|x| self.expr_as(x, Rep::Scalar, x.ty))
                    .collect();
                scalar(prim_code(*p, &a))
            }
            TE::Field(li, fi, arg) => {
                let layout = &self.m.layouts[*li];
                let from = layout.read_from.unwrap_or(Kind::Object);
                let fname = format!("{}_{}", snake(&layout.name), layout.fields[*fi].name);
                let fty = layout.fields[*fi].ty;
                let a = self.expr_as(arg, Rep::Plain, Ty::Ref(from));
                let code = format!("heap::{fname}(m, {a})");
                if fty.is_heap() {
                    (code, Rep::Plain)
                } else {
                    scalar(code)
                }
            }
            TE::KeptPush(a) => {
                let v = self.expr_as(a, Rep::Plain, Ty::Value);
                scalar(format!("heap::kept_push(m, {v})"))
            }
            TE::Poll => {
                let t = self.fresh("p");
                self.line(&format!("let {t} = cx.poll(m);"));
                self.line(&format!("if !{t} {{"));
                self.line("    return Ok(Flow::Interrupt);");
                self.line("}");
                scalar("()".into())
            }
            TE::Smi(a) => {
                let v = self.expr_as(a, Rep::Scalar, a.ty);
                let code = if a.ty == Ty::I32 {
                    format!("heap::smi({v})")
                } else {
                    format!("heap::smi(prim::to_i32({v}))")
                };
                (code, Rep::Plain)
            }
            TE::Number(a) => {
                let v = self.expr_as(a, Rep::Scalar, Ty::F64);
                let t = self.fresh("t");
                self.line(&format!("let {t} = heap::number(m, s, {v})?;"));
                (t, Rep::Rooted)
            }
            TE::BoolValue(a) => {
                let v = self.expr_as(a, Rep::Scalar, Ty::Bool);
                (format!("cx.realm.boolean(m, {v})"), Rep::Plain)
            }
            TE::Alloc(li, fields) => {
                let layout = &self.m.layouts[*li];
                let name = snake(&layout.name);
                let ftys: Vec<Ty> = layout
                    .fields
                    .iter()
                    .map(|f| if f.ty == Ty::Weak { Ty::Value } else { f.ty })
                    .collect();
                let mut args = Vec::new();
                for (f, fty) in fields.iter().zip(ftys) {
                    let rep = if fty.is_heap() {
                        Rep::Rooted
                    } else {
                        Rep::Scalar
                    };
                    let v = self.expr_as(f, rep, fty);
                    let n = self.fresh("a");
                    self.line(&format!("let {n} = {v};"));
                    args.push(n);
                }
                let t = self.fresh("t");
                self.line(&format!(
                    "let {t} = heap::alloc_{name}(m, s, &cx.realm, {})?;",
                    args.join(", ")
                ));
                (t, Rep::Rooted)
            }
            TE::Conv(t, a) => {
                let v = self.expr_as(a, Rep::Scalar, a.ty);
                scalar(conv_code(a.ty, *t, &v))
            }
            TE::Bin(op, a, b) => {
                let x = self.expr_as(a, Rep::Scalar, a.ty);
                let y = self.expr_as(b, Rep::Scalar, b.ty);
                scalar(bin_code(*op, a.ty, &x, &y))
            }
            TE::Same(a, b, neg) => {
                let x = self.expr_as(a, Rep::Plain, Ty::Value);
                let y = self.expr_as(b, Rep::Plain, Ty::Value);
                let c = format!("heap::same({x}, {y})");
                scalar(if *neg { format!("!{c}") } else { c })
            }
            TE::Un(op, a) => {
                let x = self.expr_as(a, Rep::Scalar, a.ty);
                scalar(match op {
                    UnOp::Not | UnOp::BitNot => format!("!{}", p(&x)),
                    UnOp::Neg => format!("-{}", p(&x)),
                })
            }
            TE::Match(mm) => {
                let rep = if ty.is_heap() { want } else { Rep::Scalar };
                let code = self.match_code(mm, Some((rep, ty)));
                (code, rep)
            }
            TE::Block(b) => {
                self.line("// a block value");
                self.block_stmts(b);
                match &b.tail {
                    Some(t) => self.expr(t, want, ty),
                    None => scalar("()".into()),
                }
            }
        }
    }

    fn local_use(&self, l: LocalId) -> (String, Rep) {
        let local = &self.body.locals[l.0 as usize];
        let params = self.sig().map_or(0, |s| s.params.len());
        if let Origin::RegParam(i) = local.origin
            && (l.0 as usize) < params
        {
            return (format!("cx.frame.reg(m, list.at({i}))"), Rep::Plain);
        }
        let rep = self
            .store
            .get(&l)
            .copied()
            .unwrap_or(if local.ty.is_heap() {
                Rep::Plain
            } else {
                Rep::Scalar
            });
        let n = self.local_name(l);
        (n, rep)
    }

    fn call_args(&mut self, args: &'a [TExpr], params: &[(String, Ty)], gc: bool) -> Vec<String> {
        let mut out = Vec::new();
        for (a, (_, pt)) in args.iter().zip(params) {
            let rep = if pt.is_heap() {
                if gc { Rep::Rooted } else { Rep::Plain }
            } else {
                Rep::Scalar
            };
            let v = self.expr_as(a, rep, *pt);
            if gc {
                let n = self.fresh("a");
                self.line(&format!("let {n} = {v};"));
                out.push(n);
            } else {
                out.push(v);
            }
        }
        out
    }

    fn runtime_call(&mut self, r: usize, args: &'a [TExpr]) -> (String, Rep) {
        let rt = &self.m.runtimes[r];
        let e = &rt.sig.effects;
        let (gc, user, host, random) = env_params(e);
        let a = self.call_args(args, &rt.sig.params, gc);
        let mut ps = vec!["m".to_string()];
        if gc {
            if user {
                ps.push("cx.t".into());
            }
            ps.push("s".into());
        }
        if host {
            ps.push("&mut *cx.host".into());
        }
        if random {
            ps.push("&mut *cx.random".into());
        }
        ps.extend(a);
        let call = format!("cx.rt.{}({})", rt.name, ps.join(", "));
        let ret = rt.sig.ret;
        if gc {
            let t = self.fresh("t");
            self.line(&format!("let {t} = {call}?;"));
            (
                t,
                if ret.is_heap() {
                    Rep::Rooted
                } else {
                    Rep::Scalar
                },
            )
        } else {
            (
                call,
                if ret.is_heap() {
                    Rep::Plain
                } else {
                    Rep::Scalar
                },
            )
        }
    }

    fn helper_call(&mut self, h: usize, args: &'a [TExpr]) -> (String, Rep) {
        let hp = &self.m.helpers[h];
        let gc = hp.sig.effects.gc();
        let a = self.call_args(args, &hp.sig.params, gc);
        let mut ps = vec!["m".to_string()];
        if gc {
            ps.push("s".into());
        }
        ps.push("cx".into());
        ps.extend(a);
        let call = format!("helper_{}({})", snake(&hp.name), ps.join(", "));
        let ret = hp.sig.ret;
        if gc {
            let t = self.fresh("t");
            self.line(&format!("let {t} = {call}?;"));
            (
                t,
                if ret.is_heap() {
                    Rep::Rooted
                } else {
                    Rep::Scalar
                },
            )
        } else {
            (
                call,
                if ret.is_heap() {
                    Rep::Plain
                } else {
                    Rep::Scalar
                },
            )
        }
    }
}

fn convert(code: String, from: Rep, to: Rep, ty: Ty) -> String {
    match (from, to) {
        (Rep::Rooted, Rep::Plain) => match ty {
            Ty::Value => format!("{code}.get_value(m)"),
            _ => format!("{code}.get(m)"),
        },
        (Rep::Plain, Rep::Rooted) => match ty {
            Ty::Value => format!("s.root_value(m, {code})"),
            _ => format!("{code}.root(m, s)"),
        },
        _ => code,
    }
}

fn int_lit(v: i128, t: Ty) -> String {
    match t {
        Ty::I32 => format!("{v}_i32"),
        Ty::U32 => format!("{v}_u32"),
        _ => format!("{v}_i64"),
    }
}

fn float_lit(f: f64) -> String {
    format!("{f:?}_f64")
}

/// `x` parenthesized if it is not a single term (a space at nesting
/// depth 0 means an operator), so the output has no redundant parentheses.
fn p(x: &str) -> String {
    let mut depth = 0i32;
    for c in x.chars() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ' ' if depth == 0 => return format!("({x})"),
            _ => {}
        }
    }
    x.to_string()
}

fn prim_code(prim: Prim, a: &[String]) -> String {
    let x = |i: usize| p(&a.get(i).cloned().unwrap_or_default());
    match prim {
        Prim::FAdd => format!("{} + {}", x(0), x(1)),
        Prim::FSub => format!("{} - {}", x(0), x(1)),
        Prim::FMul => format!("{} * {}", x(0), x(1)),
        Prim::FDiv => format!("{} / {}", x(0), x(1)),
        Prim::FNeg => format!("-{}", x(0)),
        Prim::FLt => format!("{} < {}", x(0), x(1)),
        Prim::FLe => format!("{} <= {}", x(0), x(1)),
        Prim::FGt => format!("{} > {}", x(0), x(1)),
        Prim::FGe => format!("{} >= {}", x(0), x(1)),
        Prim::FEq => format!("{} == {}", x(0), x(1)),
        Prim::FIsNan => format!("{}.is_nan()", x(0)),
        Prim::FToInt32 => format!("prim::to_int32({})", x(0)),
        Prim::FToUint32 => format!("prim::to_uint32({})", x(0)),
        Prim::FMod => format!("prim::remainder({}, {})", x(0), x(1)),
    }
}

fn conv_code(from: Ty, to: Ty, v: &str) -> String {
    match (from, to) {
        (a, b) if a == b => v.to_string(),
        (Ty::I32 | Ty::U32, Ty::I64 | Ty::F64) => format!("{}::from({v})", rust_scalar(to)),
        (Ty::I64, Ty::F64) => format!("prim::exact_f64({v})"),
        (_, Ty::I32) => format!("prim::to_i32({v})"),
        (_, Ty::U32) => format!("prim::to_u32({v})"),
        (_, Ty::I64) => format!("i64::from({v})"),
        _ => v.to_string(),
    }
}

fn bin_code(op: BinOp, t: Ty, x: &str, y: &str) -> String {
    match op {
        BinOp::Shl => format!("prim::shl({x}, {y})"),
        BinOp::Sar => format!("prim::sar({x}, {y})"),
        BinOp::Shr => format!("prim::shr({x}, {y})"),
        _ => {
            let _ = t;
            format!("{} {} {}", p(x), op.text(), p(y))
        }
    }
}

fn collect_binders(p: &TPat, out: &mut Vec<LocalId>) {
    match p {
        TPat::Smi(Some(i)) | TPat::Num(Some(i)) | TPat::Ref(_, Some(i)) | TPat::Value(Some(i)) => {
            out.push(*i)
        }
        TPat::Or(ps) => {
            for q in ps {
                collect_binders(q, out);
            }
        }
        _ => {}
    }
}

fn stmts_use_operand(b: &TBlock, j: usize) -> bool {
    fn stmt(s: &TStmt, j: usize) -> bool {
        match s {
            TStmt::SetReg(r, _, _) => *r == j,
            TStmt::If(_, t, e) => {
                stmts_use_operand(t, j) || e.as_ref().is_some_and(|e| stmts_use_operand(e, j))
            }
            TStmt::Match(m) => arms(m, j),
            TStmt::Path(p) => stmts_use_operand(&p.body, j),
            TStmt::Term(TTerm::Jump(o, _, _)) => *o == j,
            TStmt::Let(_, e) | TStmt::Expr(e) | TStmt::SetAcc(e, _) => expr_arms(e, j),
            _ => false,
        }
    }
    fn expr_arms(e: &TExpr, j: usize) -> bool {
        match &e.kind {
            TE::Match(m) => arms(m, j),
            TE::Block(b) => stmts_use_operand(b, j),
            _ => false,
        }
    }
    fn arms(m: &TMatch, j: usize) -> bool {
        m.arms.iter().any(|a| match &a.body {
            TArmBody::Path(p) => stmts_use_operand(&p.body, j),
            TArmBody::Block(b) => stmts_use_operand(b, j),
            TArmBody::Term(TTerm::Jump(o, _, _)) => *o == j,
            _ => false,
        })
    }
    b.stmts.iter().any(|s| stmt(s, j))
}

/// Whether every arm of `m` leaves by a terminal (so the match is a
/// statement, whatever its declared type).
fn diverges_match(m: &TMatch) -> bool {
    m.arms.iter().all(|a| match &a.body {
        TArmBody::Term(_) => true,
        TArmBody::Expr(_) => false,
        TArmBody::Path(p) => block_diverges(&p.body),
        TArmBody::Block(b) => block_diverges(b),
    })
}

fn block_diverges(b: &TBlock) -> bool {
    if b.tail.is_some() {
        return false;
    }
    match b.stmts.last() {
        Some(TStmt::Term(_)) => true,
        Some(TStmt::Match(m)) => diverges_match(m),
        Some(TStmt::If(_, t, Some(e))) => block_diverges(t) && block_diverges(e),
        Some(TStmt::Path(p)) => block_diverges(&p.body),
        _ => false,
    }
}
