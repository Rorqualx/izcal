//! Name resolution and type checking (checks OC-1 and OC-2), with the
//! declaration-level parts of OC-5 (`throw(any)` needs user code), OC-8
//! (units formulas), OC-11 (inputs name their hook), OC-13 (encoding) and
//! OC-14 (no direct heap-number stores). The route checks (OC-3 to OC-9,
//! OC-17) are in `flow`.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{self, *};
use crate::diag::{Diags, Span};
use crate::model::OperandKind;
use crate::model::*;
use crate::print;
use crate::tir::*;

/// The parsed input: each file's name and items.
pub struct Input {
    /// File names (relative paths, forward slashes).
    pub files: Vec<String>,
    /// Items per file, in file order.
    pub items: Vec<Vec<Item>>,
}

/// Resolves and type-checks everything. Bodies that fail keep going so
/// that one run reports as much as it can.
pub fn check(input: &Input, diags: &mut Diags) -> Model {
    let mut cx = Decls::default();
    // Pass 1: every name, so that declarations are order-free.
    let mut seen: BTreeMap<String, Span> = BTreeMap::new();
    for items in &input.items {
        for item in items {
            let (name, span) = match item {
                Item::Category(c) => (format!("category {}", c.name.text), c.name.span),
                Item::Fact(f) => (format!("fact {}", f.name.text), f.name.span),
                Item::Const(c) => (c.name.text.clone(), c.name.span),
                Item::Units(u) => (u.name.text.clone(), u.name.span),
                Item::Layout(l) => (format!("layout {}", l.kind.text), l.kind.span),
                Item::Runtime(r) => (r.name.text.clone(), r.name.span),
                Item::Helper(h) => (h.name.text.clone(), h.name.span),
                Item::Intrinsic(i) => (format!("%{}", i.name.text), i.name.span),
                Item::Op(o) => (format!("op {}", o.name.text), o.name.span),
            };
            if let Some(prev) = seen.insert(name.clone(), span) {
                diags.err(
                    span,
                    "OC-1",
                    format!("`{name}` is declared twice (first at line {})", prev.line),
                );
            }
        }
    }
    // The category tree and the fact kinds (R0-O12) come first: effects
    // and fields name categories.
    for items in &input.items {
        for item in items {
            if let Item::Category(c) = item {
                let param = c.param.as_ref().map(|(_, t)| resolve_type(t, diags));
                cx.cats.cats.push(CategoryInfo {
                    name: c.name.text.clone(),
                    param,
                    parent: None,
                    gc_mutable: c.gc_mutable,
                    span: c.name.span,
                });
            }
        }
    }
    for items in &input.items {
        for item in items {
            if let Item::Category(c) = item {
                let me = cx.cats.find(&c.name.text);
                match (&c.under, me) {
                    (Some(p), Some(i)) => match cx.cats.find(&p.text) {
                        Some(j) => cx.cats.cats[i].parent = Some(j),
                        None => diags.err(p.span, "OC-1", format!("unknown category `{}`", p.text)),
                    },
                    (None, _) if c.name.text != HEAP => diags.err(
                        c.name.span,
                        "OC-1",
                        format!("every category but `{HEAP}` is `under` a parent"),
                    ),
                    _ => {}
                }
            }
        }
    }
    if !cx.cats.cats.is_empty() && cx.cats.find(HEAP).is_none() {
        diags.err(
            Span::default(),
            "OC-1",
            format!("the category tree has no root `{HEAP}`"),
        );
    }
    // A cycle in `under` would make coverage loop.
    for (i, _) in cx.cats.cats.iter().enumerate() {
        let mut at = cx.cats.cats[i].parent;
        let mut steps = 0;
        while let Some(j) = at {
            steps += 1;
            if steps > cx.cats.cats.len() {
                diags.err(
                    cx.cats.cats[i].span,
                    "OC-1",
                    "the category tree has a cycle",
                );
                break;
            }
            at = cx.cats.cats[j].parent;
        }
    }
    for items in &input.items {
        for item in items {
            if let Item::Fact(f) = item {
                let on = f.on.iter().map(|c| resolve_cat(&cx, c, diags)).collect();
                cx.fact_kinds.push(FactKind {
                    name: f.name.text.clone(),
                    on,
                });
            }
        }
    }
    // Constants first: units and bodies use them.
    for items in &input.items {
        for item in items {
            if let Item::Const(c) = item {
                cx.consts.push(resolve_const(c, diags));
            }
        }
    }
    for items in &input.items {
        for item in items {
            if let Item::Units(u) = item {
                let units = resolve_units(&cx, u, diags);
                cx.units.push(units);
            }
        }
    }
    for items in &input.items {
        for item in items {
            if let Item::Layout(l) = item {
                let layout = resolve_layout(&cx, l, diags);
                cx.layouts.push(layout);
            }
        }
    }
    for items in &input.items {
        for item in items {
            if let Item::Runtime(r) = item {
                let rt = resolve_runtime(&cx, r, diags);
                cx.runtimes.push(rt);
            }
        }
    }
    for (fi, items) in input.items.iter().enumerate() {
        for item in items {
            match item {
                Item::Helper(h) => {
                    let sig = resolve_sig(&cx, h, diags);
                    cx.helpers.push((h.name.text.clone(), sig, fi));
                }
                Item::Intrinsic(i) => {
                    let sig = resolve_sig(&cx, i, diags);
                    for (p, (_, ty)) in i.params.iter().zip(&sig.params) {
                        if *ty != Ty::Value {
                            diags.err(
                                p.name.span,
                                "OC-13",
                                "an intrinsic's parameters are registers of its argument list, so each is a `Value`",
                            );
                        }
                    }
                    cx.intrinsics.push((i.name.text.clone(), sig, fi));
                }
                _ => {}
            }
        }
    }
    let mut ops_sigs = Vec::new();
    for items in &input.items {
        for item in items {
            if let Item::Op(o) = item {
                ops_sigs.push(resolve_op_sig(&cx, o, diags));
            }
        }
    }
    cx.ops = ops_sigs;
    if cx.ops.len() > 0xFC {
        diags.err(
            Span::default(),
            "OC-13",
            format!(
                "{} operations do not fit the opcode space 0x01-0xFC (0xFD is the reserved Extended prefix, 0xFE and 0xFF the scaling prefixes)",
                cx.ops.len()
            ),
        );
    }
    if cx.intrinsics.len() > 256 {
        diags.err(
            Span::default(),
            "OC-13",
            "more than 256 intrinsics: a narrow `intr` operand cannot name them all",
        );
    }

    // Pass 2: bodies.
    let mut model = Model {
        files: input.files.clone(),
        cats: cx.cats.clone(),
        fact_kinds: cx.fact_kinds.clone(),
        consts: cx.consts.clone(),
        units: cx.units.clone(),
        layouts: cx.layouts.clone(),
        runtimes: cx.runtimes.clone(),
        ..Model::default()
    };
    let mut op_index = 0usize;
    for (fi, items) in input.items.iter().enumerate() {
        let file = u16::try_from(fi).unwrap_or(u16::MAX);
        for item in items {
            match item {
                Item::Helper(h) => {
                    let idx = model.helpers.len();
                    let sig = cx.helpers[idx].1.clone();
                    let body = check_body(&cx, Owner::Helper, &sig, &[], &h.body, diags);
                    model.helpers.push(Func {
                        name: h.name.text.clone(),
                        sig,
                        body,
                        span: h.name.span,
                        file,
                    });
                }
                Item::Intrinsic(i) => {
                    let idx = model.intrinsics.len();
                    let sig = cx.intrinsics[idx].1.clone();
                    let body = check_body(&cx, Owner::Intrinsic, &sig, &[], &i.body, diags);
                    model.intrinsics.push(Func {
                        name: i.name.text.clone(),
                        sig,
                        body,
                        span: i.name.span,
                        file,
                    });
                }
                Item::Op(o) => {
                    let os = cx.ops[op_index].clone();
                    let sig = Sig {
                        params: Vec::new(),
                        frame_params: Vec::new(),
                        after_call: os.after_call,
                        facts: os.facts.clone(),
                        ret: Ty::Unit,
                        effects: os.effects.clone(),
                        anchor: o.anchor.clone(),
                    };
                    let body = check_body(&cx, Owner::Op, &sig, &os.operands, &o.body, diags);
                    model.ops.push(Op {
                        name: o.name.text.clone(),
                        opcode: u8::try_from(op_index + 1).unwrap_or(0xFC),
                        operands: os.operands,
                        acc: o.acc,
                        after_call: os.after_call,
                        facts: os.facts,
                        effects: os.effects,
                        anchor: o.anchor.clone(),
                        body,
                        span: o.name.span,
                        file,
                    });
                    op_index += 1;
                }
                _ => {}
            }
        }
    }
    check_helper_cycles(&model, diags);
    model
}

#[derive(Default)]
struct Decls {
    cats: CatTree,
    fact_kinds: Vec<FactKind>,
    consts: Vec<Const>,
    units: Vec<Units>,
    layouts: Vec<Layout>,
    runtimes: Vec<Runtime>,
    helpers: Vec<(String, Sig, usize)>,
    intrinsics: Vec<(String, Sig, usize)>,
    ops: Vec<OpSig>,
}

#[derive(Clone)]
struct OpSig {
    operands: Vec<(String, OperandKind)>,
    after_call: Option<AfterCall>,
    facts: ItemFacts,
    effects: Effects,
}

fn resolve_cat(cx: &Decls, c: &CatRef, diags: &mut Diags) -> Cat {
    match cx.cats.find(&c.name.text) {
        Some(i) => {
            let info = &cx.cats.cats[i];
            match (&info.param, &c.arg) {
                (Some(_), None) => diags.err(
                    c.name.span,
                    "OC-1",
                    format!(
                        "category `{}` takes a parameter (`any` for every value)",
                        c.name.text
                    ),
                ),
                (None, Some(_)) => diags.err(
                    c.name.span,
                    "OC-1",
                    format!("category `{}` takes no parameter", c.name.text),
                ),
                _ => {}
            }
        }
        None => diags.err(
            c.name.span,
            "OC-1",
            format!("unknown category `{}`", c.name.text),
        ),
    }
    Cat {
        name: c.name.text.clone(),
        arg: c.arg.clone(),
    }
}

fn resolve_facts(cx: &Decls, facts: &[FactClause], diags: &mut Diags) -> ItemFacts {
    let mut out = ItemFacts::default();
    for f in facts {
        match f {
            FactClause::Invalidates(ns) => {
                for n in ns {
                    if !cx.fact_kinds.iter().any(|k| k.name == n.text) {
                        diags.err(n.span, "OC-1", format!("unknown fact kind `{}`", n.text));
                    }
                    out.invalidates.push(n.text.clone());
                }
            }
            FactClause::Results(ns) => {
                for n in ns {
                    if !["smi", "f64", "tagged", "bool"].contains(&n.text.as_str()) {
                        diags.err(
                            n.span,
                            "OC-1",
                            format!(
                                "`{}` is not a representation (smi, f64, tagged, bool)",
                                n.text
                            ),
                        );
                    }
                    out.results.push(n.text.clone());
                }
            }
            FactClause::Transfer(n) => out.transfer = Some(n.text.clone()),
            FactClause::Smt(n) => out.smt = Some(n.text.clone()),
            FactClause::FrameCharge(v, _) => out.frame_charge = Some(*v),
        }
    }
    out
}

impl Decls {
    fn konst(&self, name: &str) -> Option<usize> {
        self.consts.iter().position(|c| c.name == name)
    }
    fn units(&self, name: &str) -> Option<usize> {
        self.units.iter().position(|u| u.name == name)
    }
    fn layout(&self, name: &str) -> Option<usize> {
        self.layouts.iter().position(|l| l.name == name)
    }
    fn runtime(&self, name: &str) -> Option<usize> {
        self.runtimes.iter().position(|r| r.name == name)
    }
    fn helper(&self, name: &str) -> Option<usize> {
        self.helpers.iter().position(|h| h.0 == name)
    }
}

fn resolve_type(t: &TypeExpr, diags: &mut Diags) -> Ty {
    match t {
        TypeExpr::Weak(_) => Ty::Weak,
        TypeExpr::Ref(k) => match Kind::parse(&k.text) {
            Some(kind) => Ty::Ref(kind),
            None => {
                diags.err(k.span, "OC-1", format!("unknown kind `{}`", k.text));
                Ty::Never
            }
        },
        TypeExpr::Named(n) => match n.text.as_str() {
            "Value" => Ty::Value,
            "Key" => Ty::Key,
            "i32" => Ty::I32,
            "u32" => Ty::U32,
            "i64" => Ty::I64,
            "f64" => Ty::F64,
            "bool" => Ty::Bool,
            "ObjectKind" => Ty::ObjectKind,
            other => {
                diags.err(n.span, "OC-1", format!("unknown type `{other}`"));
                Ty::Never
            }
        },
    }
}

fn resolve_const(c: &ConstDecl, diags: &mut Diags) -> Const {
    let ty = resolve_type(&c.ty, diags);
    let value = match (&c.value, ty) {
        (Literal::Int(v), t) if t.is_int() => {
            let v = if c.negative { -*v } else { *v };
            if let Some((lo, hi)) = t.range()
                && !(lo..=hi).contains(&v)
            {
                diags.err(c.name.span, "OC-2", format!("{v} does not fit {t}"));
            }
            ConstValue::Int(v)
        }
        (Literal::Float(f), Ty::F64) => {
            let v: f64 = f.parse().unwrap_or(f64::NAN);
            ConstValue::Float(if c.negative { -v } else { v })
        }
        (Literal::Int(v), Ty::F64) => {
            #[expect(clippy::cast_precision_loss, reason = "tier T: checked exact below")]
            let f = *v as f64;
            if f as i128 != *v {
                diags.err(c.name.span, "OC-2", "the integer is not exact as an f64");
            }
            ConstValue::Float(if c.negative { -f } else { f })
        }
        (Literal::Bool(b), Ty::Bool) if !c.negative => ConstValue::Bool(*b),
        _ => {
            diags.err(
                c.name.span,
                "OC-2",
                format!("the literal does not have the constant's type {ty}"),
            );
            ConstValue::Int(0)
        }
    };
    Const {
        name: c.name.text.clone(),
        ty,
        value,
        span: c.name.span,
    }
}

fn resolve_units(cx: &Decls, u: &UnitsDecl, diags: &mut Diags) -> Units {
    let mut params = Vec::new();
    for p in &u.params {
        let ty = resolve_type(&p.ty, diags);
        if !ty.is_int() {
            diags.err(
                p.name.span,
                "OC-8",
                "a units formula's parameters are semantic quantities: integers",
            );
        }
        params.push((p.name.text.clone(), ty));
    }
    let body = units_expr(cx, &params, &u.body, diags);
    if !body.ty.is_int() && body.ty != Ty::Never {
        diags.err(u.name.span, "OC-8", "a units formula is an integer");
    }
    Units {
        name: u.name.text.clone(),
        params,
        body,
        span: u.name.span,
    }
}

/// A units formula: parameters, constants, literals and `+ - * /` only
/// (C8: semantic quantities, never a representation test).
fn units_expr(cx: &Decls, params: &[(String, Ty)], e: &Expr, diags: &mut Diags) -> TExpr {
    let span = e.span();
    match e {
        Expr::Lit(Literal::Int(v), _) => TExpr {
            kind: TE::Int(*v),
            ty: Ty::I64,
            span,
        },
        Expr::Ident(n) => {
            if let Some(i) = params.iter().position(|p| p.0 == n.text) {
                return TExpr {
                    kind: TE::Local(LocalId(u32::try_from(i).unwrap_or(0))),
                    ty: Ty::I64,
                    span,
                };
            }
            if let Some(c) = cx.konst(&n.text)
                && let ConstValue::Int(v) = cx.consts[c].value
            {
                return TExpr {
                    kind: TE::Int(v),
                    ty: Ty::I64,
                    span,
                };
            }
            diags.err(
                span,
                "OC-8",
                format!("`{}` is not a parameter or an integer constant", n.text),
            );
            never(span)
        }
        Expr::Binary(op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div), l, r, _) => {
            let l = units_expr(cx, params, l, diags);
            let r = units_expr(cx, params, r, diags);
            TExpr {
                kind: TE::Bin(*op, Box::new(l), Box::new(r)),
                ty: Ty::I64,
                span,
            }
        }
        _ => {
            diags.err(
                span,
                "OC-8",
                "a units formula uses only its parameters, constants and + - * /",
            );
            never(span)
        }
    }
}

fn never(span: Span) -> TExpr {
    TExpr {
        kind: TE::Int(0),
        ty: Ty::Never,
        span,
    }
}

fn resolve_layout(cx: &Decls, l: &LayoutDecl, diags: &mut Diags) -> Layout {
    let name = l.kind.text.clone();
    let own = Kind::parse(&name);
    let known = own.is_some() || name == "Rope";
    if !known {
        diags.err(
            l.kind.span,
            "OC-1",
            format!("`{name}` is not a heap kind hg-js-ops knows (a Kind, or `Rope`)"),
        );
    }
    let is = l.is.as_ref().and_then(|o| {
        let k = Kind::parse(&o.text);
        if k.is_none() {
            diags.err(o.span, "OC-1", format!("unknown kind `{}`", o.text));
        }
        k
    });
    // `is K`: allocation-only (a representation of K); otherwise fields
    // are read from `of` or from the kind itself.
    let read_from = if is.is_some() { None } else { own };
    let alloc_as = if ALLOCATABLE.contains(&name.as_str()) {
        is.or(own)
    } else {
        None
    };
    let mut fields: Vec<Field> = Vec::new();
    for f in &l.fields {
        if fields.iter().any(|g| g.name == f.name.text) {
            diags.err(
                f.name.span,
                "OC-1",
                format!("field `{}` is declared twice", f.name.text),
            );
        }
        let ty = resolve_type(&f.ty, diags);
        let category = match &f.mutability {
            Mutability::Immutable => None,
            Mutability::In(c) => Some(resolve_cat(cx, c, diags)),
        };
        if ty == Ty::Weak && !category.as_ref().is_some_and(|c| cx.cats.gc_mutable(c)) {
            diags.err(
                f.name.span,
                "OC-1",
                "a weak field changes when a canonical cycle clears it: declare it in a `gc_mutable` category",
            );
        }
        fields.push(Field {
            name: f.name.text.clone(),
            ty,
            category,
            repr: f.repr,
        });
    }
    Layout {
        name,
        read_from,
        alloc_as,
        fields,
        span: l.kind.span,
    }
}

fn resolve_effects(cx: &Decls, effects: &[ast::Effect], diags: &mut Diags) -> Effects {
    let mut e = Effects::default();
    for eff in effects {
        match eff {
            ast::Effect::Alloc(u) => {
                if cx.units(&u.text).is_none() {
                    diags.err(
                        u.span,
                        "OC-1",
                        format!("unknown units formula `{}`", u.text),
                    );
                }
                e.alloc.insert(u.text.clone());
            }
            ast::Effect::AllocInternal(_) => e.alloc_internal = true,
            ast::Effect::Reads(cs, _) => {
                for c in cs {
                    e.reads.insert(resolve_cat(cx, c, diags));
                }
            }
            ast::Effect::Writes(cs, _) => {
                for c in cs {
                    e.writes.insert(resolve_cat(cx, c, diags));
                }
            }
            ast::Effect::Throw(ks) => {
                for k in ks {
                    if !ERROR_KINDS.contains(&k.text.as_str()) {
                        diags.err(k.span, "OC-1", format!("unknown error kind `{}`", k.text));
                    }
                    e.throws.insert(k.text.clone());
                }
            }
            ast::Effect::CallUser(_) => e.call_user = true,
            ast::Effect::Host(h) => {
                if !HOOKS.iter().any(|x| x.0 == h.text) {
                    diags.err(h.span, "OC-1", format!("`{}` is not a host hook", h.text));
                }
                e.host.insert(h.text.clone());
            }
            ast::Effect::Input(k, _) => {
                e.input.insert(*k);
            }
            ast::Effect::Interrupt(_) => e.interrupt = true,
        }
    }
    e
}

/// OC-11: a clock read goes through the clock hook, and the clock hook is
/// a clock read; OC-5: only user code throws a user value.
fn check_effect_rules(e: &Effects, span: Span, diags: &mut Diags) {
    let clock_hook = e.host.contains(CLOCK_HOOK);
    if e.input.contains(&InputKind::Clock) && !clock_hook {
        diags.err(
            span,
            "OC-11",
            format!("`input(clock)` reaches the clock only through `host({CLOCK_HOOK})`"),
        );
    }
    if clock_hook && !e.input.contains(&InputKind::Clock) {
        diags.err(
            span,
            "OC-11",
            format!("`host({CLOCK_HOOK})` reads a B1 input: declare `input(clock)`"),
        );
    }
    if e.throws.contains("any") && !e.user() {
        diags.err(
            span,
            "OC-5",
            "`throw(any)` throws a user value, which only user code can produce: declare `call_user` or name the error kinds",
        );
    }
}

fn resolve_sig(cx: &Decls, f: &FnDecl, diags: &mut Diags) -> Sig {
    let mut sig = resolve_sig_parts(cx, &f.params, &f.ret, &f.effects, &f.anchor, diags);
    sig.after_call = f.after_call;
    sig.facts = resolve_facts(cx, &f.facts, diags);
    sig
}

fn resolve_sig_parts(
    cx: &Decls,
    params: &[ast::Param],
    ret: &TypeExpr,
    effects: &[ast::Effect],
    anchor: &Anchor,
    diags: &mut Diags,
) -> Sig {
    let mut ps = Vec::new();
    let mut frame_params = Vec::new();
    for p in params {
        frame_params.push(p.frame);
        if ps.iter().any(|(n, _): &(String, Ty)| *n == p.name.text) {
            diags.err(
                p.name.span,
                "OC-1",
                format!("parameter `{}` is declared twice", p.name.text),
            );
        }
        let ty = resolve_type(&p.ty, diags);
        if ty == Ty::Weak {
            diags.err(
                p.name.span,
                "OC-2",
                "a weak slot's contents are not a parameter type",
            );
        }
        ps.push((p.name.text.clone(), ty));
    }
    let ret_ty = resolve_type(ret, diags);
    let effects = resolve_effects(cx, effects, diags);
    check_effect_rules(&effects, anchor.span(), diags);
    Sig {
        params: ps,
        frame_params,
        after_call: None,
        facts: ItemFacts::default(),
        ret: ret_ty,
        effects,
        anchor: anchor.clone(),
    }
}

fn resolve_runtime(cx: &Decls, r: &RuntimeDecl, diags: &mut Diags) -> Runtime {
    let mut sig = resolve_sig_parts(cx, &r.params, &r.ret, &r.effects, &r.anchor, diags);
    sig.facts = resolve_facts(cx, &r.facts, diags);
    if r.params.iter().any(|p| p.frame) {
        diags.err(
            r.name.span,
            "OC-6",
            "`frame` parameters are a helper's: a runtime function roots its arguments",
        );
    }
    let ensures = r.ensures.as_ref().map(|c| {
        let mut sink = Vec::new();
        let p = pattern_shallow(&c.pattern, sig.ret, &mut sink, diags);
        if !sink.is_empty() {
            diags.err(c.span, "OC-2", "a contract pattern binds nothing");
        }
        p
    });
    Runtime {
        name: r.name.text.clone(),
        sig,
        ensures,
        span: r.name.span,
    }
}

fn resolve_op_sig(cx: &Decls, o: &OpDecl, diags: &mut Diags) -> OpSig {
    let mut operands: Vec<(String, OperandKind)> = Vec::new();
    for (i, op) in o.operands.iter().enumerate() {
        if operands.iter().any(|(n, _)| *n == op.name.text) {
            diags.err(
                op.name.span,
                "OC-13",
                format!("operand `{}` is declared twice", op.name.text),
            );
        }
        let kind = match &op.kind {
            ast::OperandKind::Reg => OperandKind::Reg,
            ast::OperandKind::Cnt => OperandKind::Cnt,
            ast::OperandKind::Imm => OperandKind::Imm,
            ast::OperandKind::Fwd => OperandKind::Fwd,
            ast::OperandKind::Back => OperandKind::Back,
            ast::OperandKind::Intr => OperandKind::Intr,
            ast::OperandKind::Const(ks) => {
                let mut set = BTreeSet::new();
                for k in ks {
                    if !CONST_KINDS.contains(&k.text.as_str()) {
                        diags.err(
                            k.span,
                            "OC-13",
                            format!("unknown constant kind `{}`", k.text),
                        );
                    }
                    set.insert(k.text.clone());
                }
                if set.contains("key") && set.len() > 1 {
                    diags.err(
                        op.name.span,
                        "OC-13",
                        "a `key` constant operand accepts only keys: its value is a Key, the others' a Value",
                    );
                }
                OperandKind::Const(set.into_iter().collect())
            }
            ast::OperandKind::Slot(k) => {
                if !SLOT_KINDS.contains(&k.text.as_str()) {
                    diags.err(k.span, "OC-13", format!("unknown slot kind `{}`", k.text));
                }
                OperandKind::Slot(k.text.clone())
            }
        };
        // A count follows the register it counts; an intrinsic id is
        // followed by its register list.
        if kind == OperandKind::Cnt && (i == 0 || o.operands[i - 1].kind != ast::OperandKind::Reg) {
            diags.err(
                op.name.span,
                "OC-13",
                "a `cnt` operand follows the `reg` operand it counts",
            );
        }
        if kind == OperandKind::Intr {
            let ok = matches!(
                o.operands.get(i + 1).map(|x| &x.kind),
                Some(ast::OperandKind::Reg)
            ) && matches!(
                o.operands.get(i + 2).map(|x| &x.kind),
                Some(ast::OperandKind::Cnt)
            );
            if !ok {
                diags.err(
                    op.name.span,
                    "OC-13",
                    "an `intr` operand is followed by its argument list: `reg`, `cnt`",
                );
            }
        }
        operands.push((op.name.text.clone(), kind));
    }
    if matches!(
        o.name.text.as_str(),
        "Invalid" | "Wide2" | "Wide4" | "Extended"
    ) {
        diags.err(
            o.name.span,
            "OC-13",
            format!("`{}` is a reserved opcode name", o.name.text),
        );
    }
    let effects = resolve_effects(cx, &o.effects, diags);
    check_effect_rules(&effects, o.anchor.span(), diags);
    OpSig {
        operands,
        after_call: o.after_call,
        facts: resolve_facts(cx, &o.facts, diags),
        effects,
    }
}

/// A pattern with no binders allowed (for contracts and alternatives).
fn pattern_shallow(p: &Pattern, ty: Ty, binders: &mut Vec<String>, diags: &mut Diags) -> TPat {
    match p {
        Pattern::Wild(_) => TPat::Wild,
        Pattern::Oddball(n) => {
            if n.text == "Empty" {
                if ty != Ty::Weak {
                    diags.err(
                        n.span,
                        "OC-2",
                        "`Empty` matches only a weak slot's contents",
                    );
                }
                TPat::Empty
            } else {
                TPat::Oddball(Oddball::parse(&n.text).unwrap_or(Oddball::Undefined))
            }
        }
        Pattern::Or(ps) => TPat::Or(
            ps.iter()
                .map(|q| pattern_shallow(q, ty, binders, diags))
                .collect(),
        ),
        Pattern::Smi(b) | Pattern::Num(b) | Pattern::Value(b) | Pattern::Ref(_, b) => {
            binders.push(b.text.clone());
            diags.err(b.span, "OC-2", "no binders here");
            TPat::Wild
        }
        Pattern::Tuple(_, s) => {
            diags.err(*s, "OC-2", "no tuple here");
            TPat::Wild
        }
    }
}

// --- Bodies ----------------------------------------------------------------

#[derive(Clone, Copy)]
enum Owner {
    Op,
    Helper,
    Intrinsic,
}

struct BodyCx<'a> {
    decls: &'a Decls,
    owner: Owner,
    operands: &'a [(String, OperandKind)],
    ret: Ty,
    locals: Vec<Local>,
    scopes: Vec<Vec<(String, LocalId)>>,
    paths: Vec<PathInfo>,
    path_stack: Vec<usize>,
    guard: String,
    diags: &'a mut Diags,
}

fn check_body(
    decls: &Decls,
    owner: Owner,
    sig: &Sig,
    operands: &[(String, OperandKind)],
    block: &Block,
    diags: &mut Diags,
) -> Body {
    let gc = sig.effects.gc();
    let mut locals = Vec::new();
    let mut scope = Vec::new();
    for (i, (name, ty)) in sig.params.iter().enumerate() {
        let frame = sig.frame_params.get(i).copied().unwrap_or(false);
        let origin = match owner {
            Owner::Intrinsic => Origin::RegParam(i),
            Owner::Helper if frame => Origin::FrameParam(i),
            Owner::Helper if gc => Origin::RootedParam(i),
            _ => Origin::Param(i),
        };
        let id = LocalId(u32::try_from(locals.len()).unwrap_or(u32::MAX));
        locals.push(Local {
            name: name.clone(),
            ty: *ty,
            span: block.span,
            origin,
            used: false,
        });
        scope.push((name.clone(), id));
    }
    let mut b = BodyCx {
        decls,
        owner,
        operands,
        ret: sig.ret,
        locals,
        scopes: vec![scope],
        paths: vec![PathInfo {
            name: "main".into(),
            anchor: sig.anchor.clone(),
            own_anchor: true,
            speculate: None,
            parent: None,
            guard: String::new(),
            span: block.span,
        }],
        path_stack: vec![0],
        guard: String::new(),
        diags,
    };
    let want = match owner {
        Owner::Op => Ty::Unit,
        _ => sig.ret,
    };
    let tb = b.block(block, want);
    Body {
        block: tb,
        locals: b.locals,
        paths: b.paths,
    }
}

impl BodyCx<'_> {
    fn err(&mut self, span: Span, check: &'static str, msg: impl Into<String>) {
        self.diags.err(span, check, msg);
    }

    fn is_op(&self) -> bool {
        matches!(self.owner, Owner::Op)
    }

    fn lookup(&self, name: &str) -> Option<LocalId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.iter().rev().find(|(n, _)| n == name).map(|(_, id)| *id))
    }

    fn bind(&mut self, name: &str, ty: Ty, span: Span, origin: Origin) -> Option<LocalId> {
        if name == "_" {
            return None;
        }
        let id = LocalId(u32::try_from(self.locals.len()).unwrap_or(u32::MAX));
        self.locals.push(Local {
            name: name.to_string(),
            ty,
            span,
            origin,
            used: false,
        });
        if let Some(s) = self.scopes.last_mut() {
            s.push((name.to_string(), id));
        }
        Some(id)
    }

    fn operand(&self, name: &str) -> Option<(usize, &OperandKind)> {
        self.operands
            .iter()
            .enumerate()
            .find(|(_, (n, _))| n == name)
            .map(|(i, (_, k))| (i, k))
    }

    // --- Blocks and statements -----------------------------------------

    /// A block whose value has type `want` (`Unit` for a statement block).
    fn block(&mut self, b: &Block, want: Ty) -> TBlock {
        self.scopes.push(Vec::new());
        let mut stmts = Vec::new();
        let n = b.stmts.len();
        let mut tail = None;
        for (i, s) in b.stmts.iter().enumerate() {
            // A value block may end in a match statement that is its value.
            if i + 1 == n
                && b.tail.is_none()
                && want != Ty::Unit
                && let Stmt::Match(m) = s
            {
                tail = Some(self.match_expr(m, want));
                break;
            }
            stmts.push(self.stmt(s));
        }
        if let Some(t) = &b.tail {
            if want == Ty::Unit {
                if let Expr::Match(m) = t.as_ref() {
                    stmts.push(TStmt::Match(self.match_stmt(m)));
                } else {
                    let e = self.expr(t, None);
                    if e.ty != Ty::Unit && e.ty != Ty::Never {
                        self.err(
                            t.span(),
                            "OC-2",
                            format!(
                                "a value of type {} where a statement is expected (add `;`)",
                                e.ty
                            ),
                        );
                    }
                    stmts.push(TStmt::Expr(e));
                }
            } else {
                let e = self.expr(t, Some(want));
                self.expect_ty(&e, want);
                tail = Some(e);
            }
        }
        self.scopes.pop();
        TBlock { stmts, tail }
    }

    fn expect_ty(&mut self, e: &TExpr, want: Ty) {
        if !e.ty.fits(want) {
            self.err(e.span, "OC-2", format!("expected {want}, found {}", e.ty));
        }
    }

    fn origin_of(&self, e: &TExpr) -> Origin {
        match &e.kind {
            TE::Acc => Origin::Frame(FrameLoc::Acc),
            TE::Reg(n) => Origin::Frame(FrameLoc::Reg(*n)),
            TE::ConstPool(..) => Origin::Const,
            TE::Local(l) => self.locals[l.0 as usize].origin,
            _ => Origin::Temp,
        }
    }

    fn stmt(&mut self, s: &Stmt) -> TStmt {
        match s {
            Stmt::Let { name, ty, value } => {
                let want = ty.as_ref().map(|t| resolve_type(t, self.diags));
                let e = match value {
                    Expr::Match(m) => self.match_expr(m, want.unwrap_or(Ty::Never)),
                    _ => self.expr(value, want),
                };
                if let Some(w) = want {
                    self.expect_ty(&e, w);
                }
                let ty = match want {
                    Some(w) => w,
                    None if e.ty == Ty::Never => Ty::Never,
                    None => e.ty,
                };
                if ty == Ty::Unit {
                    self.err(
                        name.span,
                        "OC-2",
                        "`let` binds a value; this expression has none",
                    );
                }
                let origin = self.origin_of(&e);
                let id = self.bind(&name.text, ty, name.span, origin);
                match id {
                    Some(id) => TStmt::Let(id, e),
                    None => TStmt::Expr(e),
                }
            }
            Stmt::Assign { place, value } => {
                if !self.is_op() {
                    self.err(value.span(), "OC-4", "only an operation writes the frame");
                }
                let e = self.expr(value, Some(Ty::Value));
                self.expect_ty(&e, Ty::Value);
                match place {
                    Place::Acc(sp) => TStmt::SetAcc(e, *sp),
                    Place::Reg(r) => match self.operand(&r.text) {
                        Some((i, OperandKind::Reg)) => TStmt::SetReg(i, e, r.span),
                        _ => {
                            self.err(
                                r.span,
                                "OC-4",
                                format!("`{}` is not a `reg` operand", r.text),
                            );
                            TStmt::Expr(e)
                        }
                    },
                }
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                let c = self.expr(cond, Some(Ty::Bool));
                self.expect_ty(&c, Ty::Bool);
                let saved = std::mem::replace(&mut self.guard, format!("if {}", print::expr(cond)));
                let t = self.block(then, Ty::Unit);
                self.guard = format!("if !({})", print::expr(cond));
                let e = els.as_ref().map(|b| self.block(b, Ty::Unit));
                self.guard = saved;
                TStmt::If(c, t, e)
            }
            Stmt::Match(m) => TStmt::Match(self.match_stmt(m)),
            Stmt::Path(p) => TStmt::Path(self.path(p, Ty::Unit)),
            Stmt::Charge { units, args, span } => {
                let Some(u) = self.decls.units(&units.text) else {
                    self.err(
                        units.span,
                        "OC-1",
                        format!("unknown units formula `{}`", units.text),
                    );
                    return TStmt::Expr(never(*span));
                };
                let params = self.decls.units[u].params.clone();
                if params.len() != args.len() {
                    self.err(
                        *span,
                        "OC-2",
                        format!("`{}` takes {} arguments", units.text, params.len()),
                    );
                }
                let targs: Vec<TExpr> = args
                    .iter()
                    .zip(params.iter().map(|p| p.1).chain(std::iter::repeat(Ty::I64)))
                    .map(|(a, t)| {
                        let e = self.expr(a, Some(t));
                        self.expect_ty(&e, t);
                        e
                    })
                    .collect();
                TStmt::Charge(u, targs, *span)
            }
            Stmt::Expr(e) => {
                let t = self.expr(e, None);
                TStmt::Expr(t)
            }
            Stmt::Terminal(t) => TStmt::Term(self.terminal(t)),
        }
    }

    fn terminal(&mut self, t: &Terminal) -> TTerm {
        match t {
            Terminal::Next(s) => {
                if !self.is_op() {
                    self.err(
                        *s,
                        "OC-4",
                        "`next` ends an operation; a helper or intrinsic returns",
                    );
                }
                TTerm::Next(*s)
            }
            Terminal::Jump(n) => {
                if !self.is_op() {
                    self.err(n.span, "OC-4", "only an operation jumps");
                }
                match self.operand(&n.text) {
                    Some((i, OperandKind::Fwd)) => TTerm::Jump(i, false, n.span),
                    Some((i, OperandKind::Back)) => TTerm::Jump(i, true, n.span),
                    _ => {
                        self.err(
                            n.span,
                            "OC-4",
                            format!("`{}` is not a `fwd` or `back` operand", n.text),
                        );
                        TTerm::Next(n.span)
                    }
                }
            }
            Terminal::Return(e, s) => {
                if self.is_op() {
                    if e.is_some() {
                        self.err(
                            *s,
                            "OC-4",
                            "an operation returns the accumulator: `return;`",
                        );
                    }
                    return TTerm::Return(None, *s);
                }
                match e {
                    Some(e) => {
                        let ret = self.ret;
                        let te = self.expr(e, Some(ret));
                        self.expect_ty(&te, ret);
                        TTerm::Return(Some(te), *s)
                    }
                    None => {
                        self.err(*s, "OC-2", format!("return a value of type {}", self.ret));
                        TTerm::Return(None, *s)
                    }
                }
            }
            Terminal::Throw(k) => {
                if !ERROR_KINDS.contains(&k.text.as_str()) || k.text == "any" {
                    self.err(k.span, "OC-1", format!("`{}` is not an error kind", k.text));
                }
                TTerm::Throw(k.text.clone(), k.span)
            }
            Terminal::TailCall(kind, args, s) => {
                if !self.is_op() {
                    self.err(
                        *s,
                        "OC-17",
                        "a tail user call ends an operation's path, then `next`",
                    );
                }
                if args.len() < 2 {
                    self.err(
                        *s,
                        "OC-17",
                        "a tail user call takes the callee, `this` and the arguments",
                    );
                }
                let targs: Vec<TExpr> = args
                    .iter()
                    .map(|a| {
                        let e = self.expr(a, Some(Ty::Value));
                        self.expect_ty(&e, Ty::Value);
                        e
                    })
                    .collect();
                TTerm::TailCall(*kind, targs, *s)
            }
        }
    }

    fn path(&mut self, p: &PathDecl, want: Ty) -> TPath {
        let parent = self.path_stack.last().copied();
        let inherited = parent
            .map(|i| self.paths[i].anchor.clone())
            .unwrap_or(Anchor::Izcal {
                reason: String::new(),
                span: p.name.span,
            });
        if self.paths.iter().any(|q| q.name == p.name.text) {
            self.err(
                p.name.span,
                "OC-12",
                format!("path `{}` is named twice", p.name.text),
            );
        }
        let speculate =
            p.speculate
                .as_ref()
                .and_then(|(slot, pred)| match self.operand(&slot.text) {
                    Some((i, OperandKind::Slot(k))) => {
                        if predicates(k).contains(&pred.text.as_str()) {
                            Some((i, pred.text.clone()))
                        } else {
                            let k = k.clone();
                            self.err(
                                pred.span,
                                "OC-7",
                                format!(
                                    "a `{k}` slot offers no feedback predicate `{}`",
                                    pred.text
                                ),
                            );
                            None
                        }
                    }
                    _ => {
                        self.err(
                            slot.span,
                            "OC-7",
                            format!("`{}` is not a `slot` operand", slot.text),
                        );
                        None
                    }
                });
        let id = self.paths.len();
        self.paths.push(PathInfo {
            name: p.name.text.clone(),
            anchor: p.anchor.clone().unwrap_or(inherited),
            own_anchor: p.anchor.is_some(),
            speculate,
            parent,
            guard: self.guard.clone(),
            span: p.name.span,
        });
        self.path_stack.push(id);
        let body = self.block(&p.body, want);
        self.path_stack.pop();
        TPath { id, body }
    }

    // --- Matches -------------------------------------------------------

    fn scrutinees(&mut self, e: &Expr) -> Vec<TExpr> {
        let parts: Vec<&Expr> = match e {
            Expr::Tuple(ps, _) => ps.iter().collect(),
            _ => vec![e],
        };
        parts
            .into_iter()
            .map(|p| {
                let t = self.expr(p, None);
                if t.ty.atoms().is_none() && t.ty != Ty::Never {
                    self.err(
                        t.span,
                        "OC-2",
                        format!("a match is over JS values or a weak slot, not {}", t.ty),
                    );
                }
                t
            })
            .collect()
    }

    fn match_stmt(&mut self, m: &ast::Match) -> TMatch {
        self.match_common(m, Ty::Unit)
    }

    /// A match expression; `want` is the expected type or `Never` when
    /// unknown (then the arms decide).
    fn match_expr(&mut self, m: &ast::Match, want: Ty) -> TExpr {
        let tm = self.match_common(m, want);
        TExpr {
            ty: tm.ty,
            span: m.span,
            kind: TE::Match(Box::new(tm)),
        }
    }

    fn match_common(&mut self, m: &ast::Match, want: Ty) -> TMatch {
        let scr = self.scrutinees(&m.scrutinee);
        let mut arms = Vec::new();
        let mut ty = if want == Ty::Never { None } else { Some(want) };
        for arm in &m.arms {
            self.scopes.push(Vec::new());
            let pats = self.arm_patterns(&arm.pattern, &scr);
            let guard = arm.guard.as_ref().map(|g| {
                let e = self.expr(g, Some(Ty::Bool));
                self.expect_ty(&e, Ty::Bool);
                e
            });
            let mut text = print::pattern(&arm.pattern);
            if let Some(g) = &arm.guard {
                text = format!("{text} if {}", print::expr(g));
            }
            let saved = std::mem::replace(&mut self.guard, text.clone());
            let body_want = ty.unwrap_or(Ty::Never);
            let body = match &arm.body {
                ArmBody::Path(p) => {
                    let w = if body_want == Ty::Never && want != Ty::Unit {
                        // Infer from the path's tail.
                        p.body.tail.as_ref().map_or(Ty::Unit, |t| self.peek_type(t))
                    } else {
                        body_want
                    };
                    let tp = self.path(p, w);
                    if ty.is_none() && tp.body.tail.is_some() {
                        ty = tp.body.tail.as_ref().map(|t| t.ty);
                    }
                    TArmBody::Path(tp)
                }
                ArmBody::Block(b) => {
                    let w = if body_want == Ty::Never && want != Ty::Unit {
                        b.tail.as_ref().map_or(Ty::Unit, |t| self.peek_type(t))
                    } else {
                        body_want
                    };
                    let tb = self.block(b, w);
                    if ty.is_none() && tb.tail.is_some() {
                        ty = tb.tail.as_ref().map(|t| t.ty);
                    }
                    TArmBody::Block(tb)
                }
                ArmBody::Terminal(t) => TArmBody::Term(self.terminal(t)),
                ArmBody::Expr(e) => {
                    let te = self.expr(e, ty);
                    match ty {
                        Some(w) => self.expect_ty(&te, w),
                        None => ty = Some(te.ty),
                    }
                    TArmBody::Expr(te)
                }
            };
            self.guard = saved;
            self.scopes.pop();
            arms.push(TArm {
                pats,
                guard,
                body,
                text,
            });
        }
        let ty = ty.unwrap_or(Ty::Unit);
        let tm = TMatch {
            scrutinees: scr,
            arms,
            ty,
            span: m.span,
        };
        self.exhaustive(&tm);
        tm
    }

    /// The type an expression would have, without reporting (used to
    /// infer a value match's type from its first block arm).
    fn peek_type(&mut self, e: &Expr) -> Ty {
        let mut sink = Diags::default();
        let saved_locals = self.locals.len();
        let saved_paths = self.paths.len();
        let real = std::mem::take(self.diags);
        let t = self.expr(e, None).ty;
        sink.0.append(&mut self.diags.0);
        *self.diags = real;
        self.locals.truncate(saved_locals);
        self.paths.truncate(saved_paths);
        t
    }

    fn arm_patterns(&mut self, p: &Pattern, scr: &[TExpr]) -> Vec<TPat> {
        if scr.len() == 1 {
            return vec![self.pattern(p, &scr[0])];
        }
        match p {
            Pattern::Tuple(ps, span) => {
                if ps.len() != scr.len() {
                    self.err(
                        *span,
                        "OC-2",
                        format!("the scrutinee has {} parts", scr.len()),
                    );
                }
                ps.iter()
                    .zip(scr)
                    .map(|(q, s)| self.pattern(q, s))
                    .collect()
            }
            Pattern::Wild(_) => vec![TPat::Wild; scr.len()],
            _ => {
                self.err(
                    p.span(),
                    "OC-2",
                    "a tuple scrutinee needs a tuple pattern or `_`",
                );
                vec![TPat::Wild; scr.len()]
            }
        }
    }

    fn pattern(&mut self, p: &Pattern, scr: &TExpr) -> TPat {
        let sty = scr.ty;
        let origin = self.origin_of(scr);
        let weak = sty == Ty::Weak;
        match p {
            Pattern::Wild(_) => TPat::Wild,
            Pattern::Oddball(n) => {
                if n.text == "Empty" {
                    if !weak {
                        self.err(
                            n.span,
                            "OC-2",
                            "`Empty` matches only a weak slot's contents",
                        );
                    }
                    return TPat::Empty;
                }
                if weak {
                    self.err(
                        n.span,
                        "OC-2",
                        "a weak slot's contents match `Empty` or `Value(x)`",
                    );
                }
                TPat::Oddball(Oddball::parse(&n.text).unwrap_or(Oddball::Undefined))
            }
            Pattern::Smi(b) | Pattern::Num(b) => {
                if weak {
                    self.err(
                        b.span,
                        "OC-2",
                        "a weak slot's contents match `Empty` or `Value(x)`",
                    );
                }
                let is_smi = matches!(p, Pattern::Smi(_));
                let ty = if is_smi { Ty::I32 } else { Ty::F64 };
                let id = self.bind(&b.text, ty, b.span, Origin::Temp);
                if is_smi { TPat::Smi(id) } else { TPat::Num(id) }
            }
            Pattern::Ref(k, b) => {
                if weak {
                    self.err(
                        k.span,
                        "OC-2",
                        "a weak slot's contents match `Empty` or `Value(x)`",
                    );
                }
                let Some(kind) = Kind::parse(&k.text) else {
                    self.err(k.span, "OC-1", format!("unknown kind `{}`", k.text));
                    return TPat::Wild;
                };
                let id = self.bind(&b.text, Ty::Ref(kind), b.span, origin);
                TPat::Ref(kind, id)
            }
            Pattern::Value(b) => {
                let ty = if weak { Ty::Value } else { sty };
                let o = if weak { Origin::Temp } else { origin };
                let id = self.bind(&b.text, ty, b.span, o);
                TPat::Value(id)
            }
            Pattern::Tuple(_, s) => {
                self.err(*s, "OC-2", "a nested tuple pattern");
                TPat::Wild
            }
            Pattern::Or(ps) => {
                let mut out = Vec::new();
                for q in ps {
                    let before = self.locals.len();
                    let t = self.pattern(q, scr);
                    if self.locals.len() != before {
                        self.err(q.span(), "OC-2", "alternatives bind nothing");
                    }
                    out.push(t);
                }
                TPat::Or(out)
            }
        }
    }

    /// C2: every match is exhaustive over its scrutinee's atoms, and no
    /// arm is unreachable (the generated Rust would not compile cleanly).
    fn exhaustive(&mut self, m: &TMatch) {
        let universes: Vec<Vec<Atom>> = m
            .scrutinees
            .iter()
            .map(|s| s.ty.atoms().unwrap_or_else(|| Atom::VALUE.to_vec()))
            .collect();
        // Every combination of atoms, one per scrutinee.
        let mut combos: Vec<Vec<Atom>> = vec![Vec::new()];
        for u in &universes {
            let mut next = Vec::new();
            for c in &combos {
                for a in u {
                    let mut c2 = c.clone();
                    c2.push(*a);
                    next.push(c2);
                }
            }
            combos = next;
        }
        let mut covered = vec![false; combos.len()];
        for arm in &m.arms {
            let sets: Vec<BTreeSet<Atom>> = arm
                .pats
                .iter()
                .zip(&universes)
                .map(|(p, u)| pat_atoms(p, u))
                .collect();
            let mut new = false;
            for (i, c) in combos.iter().enumerate() {
                if c.iter().zip(&sets).all(|(a, s)| s.contains(a)) && !covered[i] {
                    new = true;
                    if arm.guard.is_none() {
                        covered[i] = true;
                    }
                }
            }
            if !new {
                self.err(
                    m.span,
                    "OC-2",
                    format!(
                        "the arm `{}` is unreachable: earlier arms cover it",
                        arm.text
                    ),
                );
            }
        }
        let missing: Vec<String> = combos
            .iter()
            .zip(&covered)
            .filter(|(_, c)| !**c)
            .map(|(c, _)| {
                let parts: Vec<String> = c.iter().map(|a| format!("{a:?}")).collect();
                parts.join(", ")
            })
            .collect();
        if let Some(first) = missing.first() {
            self.err(
                m.span,
                "OC-2",
                format!(
                    "the match is not exhaustive: ({first}) is not handled{}",
                    if missing.len() > 1 {
                        format!(", nor {} other cases", missing.len() - 1)
                    } else {
                        String::new()
                    }
                ),
            );
        }
    }

    // --- Expressions ---------------------------------------------------

    fn expr(&mut self, e: &Expr, want: Option<Ty>) -> TExpr {
        let span = e.span();
        let mk = |kind, ty| TExpr { kind, ty, span };
        match e {
            Expr::Lit(Literal::Int(v), _) => {
                let ty = match want {
                    Some(t) if t.is_int() => t,
                    Some(Ty::F64) => {
                        #[expect(clippy::cast_precision_loss, reason = "tier T: checked exact")]
                        let f = *v as f64;
                        if f as i128 != *v {
                            self.err(span, "OC-3", "the literal is not exact as an f64");
                        }
                        return mk(TE::Float(f), Ty::F64);
                    }
                    _ => Ty::I64,
                };
                if let Some((lo, hi)) = ty.range()
                    && !(lo..=hi).contains(v)
                {
                    self.err(span, "OC-3", format!("{v} does not fit {ty}"));
                }
                mk(TE::Int(*v), ty)
            }
            Expr::Lit(Literal::Float(f), _) => {
                mk(TE::Float(f.parse().unwrap_or(f64::NAN)), Ty::F64)
            }
            Expr::Lit(Literal::Bool(b), _) => mk(TE::Bool(*b), Ty::Bool),
            Expr::Ident(n) => self.ident(n, want),
            Expr::Acc(_) => {
                if !self.is_op() {
                    self.err(span, "OC-4", "only an operation reads the accumulator");
                }
                mk(TE::Acc, Ty::Value)
            }
            Expr::Reg(n) => match self.operand(&n.text) {
                Some((i, OperandKind::Reg)) => mk(TE::Reg(i), Ty::Value),
                _ => {
                    self.err(
                        n.span,
                        "OC-4",
                        format!("`{}` is not a `reg` operand", n.text),
                    );
                    never(span)
                }
            },
            Expr::ConstPool(n) => match self.operand(&n.text) {
                Some((i, OperandKind::Const(ks))) => {
                    let key = ks.iter().any(|k| k == "key");
                    mk(TE::ConstPool(i, key), if key { Ty::Key } else { Ty::Value })
                }
                _ => {
                    self.err(
                        n.span,
                        "OC-4",
                        format!("`{}` is not a `const` operand", n.text),
                    );
                    never(span)
                }
            },
            Expr::Call(n, args) => self.call(n, args, want),
            Expr::QCall(q, f, args) => self.qcall(q, f, args),
            Expr::Alloc(k, fields) => self.alloc(k, fields),
            Expr::Match(m) => self.match_expr(m, want.unwrap_or(Ty::Never)),
            Expr::Block(b) => {
                let w =
                    want.unwrap_or_else(|| b.tail.as_ref().map_or(Ty::Unit, |t| self.peek_type(t)));
                let tb = self.block(b, w);
                let ty = tb.tail.as_ref().map_or(Ty::Unit, |t| t.ty);
                mk(TE::Block(Box::new(tb)), ty)
            }
            Expr::Binary(op, l, r, _) => self.binary(*op, l, r, span),
            Expr::Unary(op, x, _) => {
                let t = self.expr(x, want);
                let ty = match op {
                    UnOp::Not => {
                        self.expect_ty(&t, Ty::Bool);
                        Ty::Bool
                    }
                    UnOp::Neg => {
                        if !matches!(t.ty, Ty::I32 | Ty::I64 | Ty::F64 | Ty::Never) {
                            self.err(span, "OC-2", format!("`-` on {}", t.ty));
                        }
                        t.ty
                    }
                    UnOp::BitNot => {
                        if !matches!(t.ty, Ty::I32 | Ty::U32 | Ty::Never) {
                            self.err(span, "OC-2", format!("`~` on {}", t.ty));
                        }
                        t.ty
                    }
                };
                mk(TE::Un(*op, Box::new(t)), ty)
            }
            Expr::Tuple(_, _) => {
                self.err(span, "OC-2", "a tuple is only a match scrutinee");
                never(span)
            }
        }
    }

    fn ident(&mut self, n: &Name, want: Option<Ty>) -> TExpr {
        let span = n.span;
        let mk = |kind, ty| TExpr { kind, ty, span };
        if let Some(id) = self.lookup(&n.text) {
            let l = &mut self.locals[id.0 as usize];
            l.used = true;
            return mk(TE::Local(id), l.ty);
        }
        if let Some((i, k)) = self.operand(&n.text) {
            if *k == OperandKind::Imm {
                return mk(TE::Imm(i), Ty::I32);
            }
            let k = k.text();
            self.err(
                span,
                "OC-4",
                format!("the `{k}` operand `{}` is not a value", n.text),
            );
            return never(span);
        }
        if let Some(c) = self.decls.konst(&n.text) {
            let c = &self.decls.consts[c];
            return match c.value {
                ConstValue::Int(v) => {
                    let ty = match want {
                        Some(t) if t.is_int() => t,
                        _ => c.ty,
                    };
                    mk(TE::Int(v), ty)
                }
                ConstValue::Float(f) => mk(TE::Float(f), Ty::F64),
                ConstValue::Bool(b) => mk(TE::Bool(b), Ty::Bool),
            };
        }
        if let Some(o) = Oddball::parse(&n.text) {
            return mk(TE::Oddball(o), Ty::Value);
        }
        if let Some(k) = Kind::parse(&n.text)
            && k.is_object_kind()
        {
            return mk(TE::ObjKind(k), Ty::ObjectKind);
        }
        self.err(span, "OC-1", format!("unknown name `{}`", n.text));
        never(span)
    }

    fn call(&mut self, n: &Name, args: &[Expr], _want: Option<Ty>) -> TExpr {
        let span = n.span;
        let mk = |kind, ty| TExpr { kind, ty, span };
        let one = |this: &mut Self, want: Option<Ty>| -> TExpr {
            if args.len() != 1 {
                this.err(span, "OC-2", format!("`{}` takes one argument", n.text));
            }
            match args.first() {
                Some(a) => this.expr(a, want),
                None => never(span),
            }
        };
        match n.text.as_str() {
            "i32" | "u32" | "i64" | "f64" => {
                let ty = match n.text.as_str() {
                    "i32" => Ty::I32,
                    "u32" => Ty::U32,
                    "i64" => Ty::I64,
                    _ => Ty::F64,
                };
                let a = one(self, None);
                if !a.ty.is_numeric() && a.ty != Ty::Never {
                    self.err(
                        span,
                        "OC-2",
                        format!("`{}` converts a number, not {}", n.text, a.ty),
                    );
                }
                if ty.is_int() && a.ty == Ty::F64 {
                    self.err(
                        span,
                        "OC-14",
                        "converting an f64 to an integer is `f64.to_int32` or `f64.to_uint32` (ToInt32's modular semantics)",
                    );
                }
                return mk(TE::Conv(ty, Box::new(a)), ty);
            }
            "smi" => {
                let a = one(self, Some(Ty::I32));
                if !a.ty.is_int() && a.ty != Ty::Never {
                    self.err(span, "OC-2", "`smi` takes an integer");
                }
                return mk(TE::Smi(Box::new(a)), Ty::Value);
            }
            "number" => {
                let a = one(self, Some(Ty::F64));
                self.expect_ty(&a, Ty::F64);
                return mk(TE::Number(Box::new(a)), Ty::Value);
            }
            "bool" => {
                let a = one(self, Some(Ty::Bool));
                self.expect_ty(&a, Ty::Bool);
                return mk(TE::BoolValue(Box::new(a)), Ty::Value);
            }
            "intrinsic" => {
                if !self.is_op() || args.len() != 3 {
                    self.err(span, "OC-13", "`intrinsic(id, list, count)` is an operation's dispatch over its `intr`, `reg` and `cnt` operands");
                    return never(span);
                }
                let names: Vec<Option<&str>> = args
                    .iter()
                    .map(|a| match a {
                        Expr::Ident(n) => Some(n.text.as_str()),
                        _ => None,
                    })
                    .collect();
                let find = |this: &Self, i: usize, want: &OperandKind| -> Option<usize> {
                    let name = names[i]?;
                    this.operand(name)
                        .filter(|(_, k)| *k == want)
                        .map(|(j, _)| j)
                };
                let (Some(id), Some(list), Some(count)) = (
                    find(self, 0, &OperandKind::Intr),
                    find(self, 1, &OperandKind::Reg),
                    find(self, 2, &OperandKind::Cnt),
                ) else {
                    self.err(
                        span,
                        "OC-13",
                        "`intrinsic` takes the operation's `intr`, `reg` and `cnt` operands",
                    );
                    return never(span);
                };
                return mk(
                    TE::Intrinsic {
                        id,
                        args: list,
                        count,
                    },
                    Ty::Value,
                );
            }
            _ => {}
        }
        if let Some(r) = self.decls.runtime(&n.text) {
            let sig = self.decls.runtimes[r].sig.clone();
            let targs = self.args(n, args, &sig.params);
            return mk(TE::Runtime(r, targs), sig.ret);
        }
        if let Some(h) = self.decls.helper(&n.text) {
            let sig = self.decls.helpers[h].1.clone();
            let targs = self.args(n, args, &sig.params);
            return mk(TE::Helper(h, targs), sig.ret);
        }
        self.err(span, "OC-1", format!("unknown function `{}`", n.text));
        for a in args {
            let _ = self.expr(a, None);
        }
        never(span)
    }

    fn args(&mut self, n: &Name, args: &[Expr], params: &[(String, Ty)]) -> Vec<TExpr> {
        if args.len() != params.len() {
            self.err(
                n.span,
                "OC-2",
                format!(
                    "`{}` takes {} arguments, not {}",
                    n.text,
                    params.len(),
                    args.len()
                ),
            );
        }
        args.iter()
            .enumerate()
            .map(|(i, a)| {
                let want = params.get(i).map(|p| p.1);
                let e = self.expr(a, want);
                if let Some(w) = want {
                    self.expect_ty(&e, w);
                }
                e
            })
            .collect()
    }

    fn qcall(&mut self, q: &Name, f: &Name, args: &[Expr]) -> TExpr {
        let span = q.span;
        let mk = |kind, ty| TExpr { kind, ty, span };
        match (q.text.as_str(), f.text.as_str()) {
            ("f64", name) => {
                let Some((prim, params, ret)) = Prim::parse(name) else {
                    self.err(
                        f.span,
                        "OC-14",
                        format!("`f64.{name}` is not a primitive (implementation-approximated functions are runtime calls to hg-math; there is no fused multiply-add)"),
                    );
                    return never(span);
                };
                let ps: Vec<(String, Ty)> = params.iter().map(|t| (String::new(), *t)).collect();
                let targs = self.args(f, args, &ps);
                mk(TE::Prim(prim, targs), ret)
            }
            ("kept", "push") => {
                let ps = vec![(String::new(), Ty::Value)];
                let mut targs = self.args(f, args, &ps);
                let a = targs.pop().unwrap_or_else(|| never(span));
                mk(TE::KeptPush(Box::new(a)), Ty::Unit)
            }
            ("interrupt", "poll") => {
                let _ = self.args(f, args, &[]);
                mk(TE::Poll, Ty::Unit)
            }
            ("unit", "strict") => {
                let _ = self.args(f, args, &[]);
                mk(TE::UnitStrict, Ty::Bool)
            }
            (k, field) => {
                let Some(li) = self.decls.layout(k) else {
                    self.err(
                        q.span,
                        "OC-1",
                        format!("unknown kind or typed operation `{k}.{field}`"),
                    );
                    return never(span);
                };
                let layout = &self.decls.layouts[li];
                let Some(from) = layout.read_from else {
                    self.err(
                        f.span,
                        "OC-1",
                        format!("`{k}` is allocation-only: read its fields through the kind it represents"),
                    );
                    return never(span);
                };
                let Some(fi) = layout.fields.iter().position(|x| x.name == field) else {
                    self.err(f.span, "OC-1", format!("`{k}` has no field `{field}`"));
                    return never(span);
                };
                let fty = layout.fields[fi].ty;
                let ps = vec![(String::new(), Ty::Ref(from))];
                let mut targs = self.args(f, args, &ps);
                let a = targs.pop().unwrap_or_else(|| never(span));
                mk(TE::Field(li, fi, Box::new(a)), fty)
            }
        }
    }

    fn alloc(&mut self, k: &Name, fields: &[(Name, Expr)]) -> TExpr {
        let span = k.span;
        let Some(li) = self.decls.layout(&k.text) else {
            self.err(span, "OC-1", format!("unknown kind `{}`", k.text));
            return never(span);
        };
        let layout = self.decls.layouts[li].clone();
        if layout.name == "HeapNumber" {
            self.err(
                span,
                "OC-14",
                "a heap number is made by `number(f)`, which keeps the canonical representation and canonicalizes NaN (JS-D12 (a))",
            );
        }
        let Some(as_kind) = layout.alloc_as else {
            self.err(
                span,
                "OC-1",
                format!("`{}` is not a fixed-size kind hg-js-ops allocates", k.text),
            );
            return never(span);
        };
        let mut values: Vec<Option<TExpr>> = vec![None; layout.fields.len()];
        for (fname, e) in fields {
            match layout.fields.iter().position(|f| f.name == fname.text) {
                Some(i) => {
                    let want = if layout.fields[i].ty == Ty::Weak {
                        Ty::Value
                    } else {
                        layout.fields[i].ty
                    };
                    let te = self.expr(e, Some(want));
                    self.expect_ty(&te, want);
                    if values[i].is_some() {
                        self.err(
                            fname.span,
                            "OC-2",
                            format!("field `{}` is set twice", fname.text),
                        );
                    }
                    values[i] = Some(te);
                }
                None => {
                    self.err(
                        fname.span,
                        "OC-1",
                        format!("`{}` has no field `{}`", k.text, fname.text),
                    );
                }
            }
        }
        let mut out = Vec::new();
        for (i, v) in values.into_iter().enumerate() {
            match v {
                Some(v) => out.push(v),
                None => {
                    self.err(
                        span,
                        "OC-2",
                        format!("field `{}` is not initialized", layout.fields[i].name),
                    );
                    out.push(never(span));
                }
            }
        }
        TExpr {
            kind: TE::Alloc(li, out),
            ty: Ty::Ref(as_kind),
            span,
        }
    }

    fn binary(&mut self, op: BinOp, l: &Expr, r: &Expr, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        if matches!(op, BinOp::And | BinOp::Or) {
            let a = self.expr(l, Some(Ty::Bool));
            let b = self.expr(r, Some(Ty::Bool));
            self.expect_ty(&a, Ty::Bool);
            self.expect_ty(&b, Ty::Bool);
            return mk(TE::Bin(op, Box::new(a), Box::new(b)), Ty::Bool);
        }
        // Type the non-literal side first, so a literal takes its type.
        let lit = |e: &Expr| matches!(e, Expr::Lit(Literal::Int(_), _));
        let (a, b) = if lit(l) && !lit(r) {
            let b = self.expr(r, None);
            let a = self.expr(l, Some(b.ty));
            (a, b)
        } else {
            let a = self.expr(l, None);
            let want = if matches!(op, BinOp::Shl | BinOp::Sar | BinOp::Shr) {
                None
            } else {
                Some(a.ty)
            };
            let b = self.expr(r, want);
            (a, b)
        };
        let never_ty = a.ty == Ty::Never || b.ty == Ty::Never;
        match op {
            BinOp::Eq | BinOp::Ne => {
                if a.ty.is_heap() && b.ty.is_heap() && a.ty != Ty::Weak && b.ty != Ty::Weak {
                    return mk(
                        TE::Same(Box::new(a), Box::new(b), op == BinOp::Ne),
                        Ty::Bool,
                    );
                }
                let ok = a.ty == b.ty
                    && (a.ty.is_numeric() || matches!(a.ty, Ty::Bool | Ty::ObjectKind));
                if !ok && !never_ty {
                    self.err(
                        span,
                        "OC-2",
                        format!("`{}` compares {} with {}", op.text(), a.ty, b.ty),
                    );
                }
                mk(TE::Bin(op, Box::new(a), Box::new(b)), Ty::Bool)
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if !(a.ty == b.ty && a.ty.is_numeric()) && !never_ty {
                    self.err(
                        span,
                        "OC-2",
                        format!("`{}` compares {} with {}", op.text(), a.ty, b.ty),
                    );
                }
                mk(TE::Bin(op, Box::new(a), Box::new(b)), Ty::Bool)
            }
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                if !(a.ty == b.ty && a.ty.is_numeric()) && !never_ty {
                    self.err(
                        span,
                        "OC-2",
                        format!("`{}` on {} and {}", op.text(), a.ty, b.ty),
                    );
                }
                let ty = a.ty;
                mk(TE::Bin(op, Box::new(a), Box::new(b)), ty)
            }
            BinOp::Rem => {
                if !(a.ty == b.ty && a.ty.is_int()) && !never_ty {
                    self.err(
                        span,
                        "OC-2",
                        format!(
                            "`%` on {} and {} (an f64 remainder is `f64.fmod`)",
                            a.ty, b.ty
                        ),
                    );
                }
                let ty = a.ty;
                mk(TE::Bin(op, Box::new(a), Box::new(b)), ty)
            }
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
                if !(a.ty == b.ty && matches!(a.ty, Ty::I32 | Ty::U32)) && !never_ty {
                    self.err(
                        span,
                        "OC-2",
                        format!("`{}` on {} and {}", op.text(), a.ty, b.ty),
                    );
                }
                let ty = a.ty;
                mk(TE::Bin(op, Box::new(a), Box::new(b)), ty)
            }
            BinOp::Shl | BinOp::Sar | BinOp::Shr => {
                let want_l = if op == BinOp::Shr { Ty::U32 } else { Ty::I32 };
                if (a.ty != want_l || !matches!(b.ty, Ty::I32 | Ty::U32)) && !never_ty {
                    self.err(
                        span,
                        "OC-2",
                        format!(
                            "`{}` shifts an {want_l} by an i32 or u32 count, not {} by {}",
                            op.text(),
                            a.ty,
                            b.ty
                        ),
                    );
                }
                mk(TE::Bin(op, Box::new(a), Box::new(b)), want_l)
            }
            BinOp::And | BinOp::Or => unreachable!("handled above"),
        }
    }
}

fn pat_atoms(p: &TPat, universe: &[Atom]) -> BTreeSet<Atom> {
    match p {
        TPat::Wild | TPat::Value(_) => {
            if universe.contains(&Atom::Present) {
                // `Value(x)` on a weak slot is the present case only.
                if matches!(p, TPat::Value(_)) {
                    return [Atom::Present].into_iter().collect();
                }
            }
            universe.iter().copied().collect()
        }
        TPat::Oddball(o) => [o.atom()].into_iter().collect(),
        TPat::Empty => [Atom::Empty].into_iter().collect(),
        TPat::Smi(_) => [Atom::Smi].into_iter().collect(),
        TPat::Num(_) => [Atom::Smi, Atom::HeapNumber].into_iter().collect(),
        TPat::Ref(k, _) => k.atoms().into_iter().collect(),
        TPat::Or(ps) => ps.iter().flat_map(|q| pat_atoms(q, universe)).collect(),
    }
}

/// C3: the helper call graph is acyclic (no recursion).
fn check_helper_cycles(m: &Model, diags: &mut Diags) {
    let calls: Vec<BTreeSet<usize>> = m
        .helpers
        .iter()
        .map(|h| {
            let mut s = BTreeSet::new();
            crate::walk::block(&h.body.block, &mut |e| {
                if let TE::Helper(i, _) = &e.kind {
                    s.insert(*i);
                }
            });
            s
        })
        .collect();
    // Depth-first search with colours.
    fn visit(
        i: usize,
        calls: &[BTreeSet<usize>],
        colour: &mut [u8],
        stack: &mut Vec<usize>,
    ) -> Option<Vec<usize>> {
        colour[i] = 1;
        stack.push(i);
        for &j in &calls[i] {
            if colour[j] == 1 {
                let at = stack.iter().position(|&k| k == j).unwrap_or(0);
                let mut cyc = stack[at..].to_vec();
                cyc.push(j);
                return Some(cyc);
            }
            if colour[j] == 0
                && let Some(c) = visit(j, calls, colour, stack)
            {
                return Some(c);
            }
        }
        stack.pop();
        colour[i] = 2;
        None
    }
    let mut colour = vec![0u8; calls.len()];
    for i in 0..calls.len() {
        if colour[i] == 0 {
            let mut stack = Vec::new();
            if let Some(cyc) = visit(i, &calls, &mut colour, &mut stack) {
                let names: Vec<&str> = cyc.iter().map(|&k| m.helpers[k].name.as_str()).collect();
                diags.err(
                    m.helpers[cyc[0]].span,
                    "OC-3",
                    format!("helpers recurse: {}", names.join(" -> ")),
                );
                return;
            }
        }
    }
}
