//! The route analysis. A description has no loops, so its body unfolds
//! into a finite set of routes from entry to a terminal; every route is
//! walked once with an abstract state, which makes the checks
//! path-sensitive without a fixpoint:
//!
//! - OC-3 totality: every route ends in exactly one terminal; integer
//!   arithmetic is proved in range by interval analysis (with refinement
//!   on comparisons against constants);
//! - OC-4 frame effects agree with the accumulator mode;
//! - OC-5 inferred effects fit the declaration (per path and per item);
//! - OC-6 rooting: a heap value live across a GC point is frame-backed
//!   (unwritten since read) or an argument of that GC point, and every
//!   `frame` helper parameter receives a frame-backed value;
//! - OC-7 a `speculate` guard precedes every effect on its route;
//! - OC-8 charges use semantic quantities only, never a `repr` field;
//! - OC-9 values read from mutable fields are stale after user code or a
//!   hook, and values of `gc_mutable` categories after any GC point;
//! - OC-17 a user call is the last effect of its route, in one of the
//!   three forms, and agrees with the item's `after_call` mode.
//!
//! The walk also yields the facts the emitters need: per-path effects,
//! GC points and their live frame values, charges and calls, and which
//! locals the generated Rust must root.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{AccMode, AfterCall, BinOp, UnOp};
use crate::diag::{Diags, Span};
use crate::model::*;
use crate::tir::*;

/// What the analysis found for one item.
#[derive(Debug, Clone, Default)]
pub struct ItemFlow {
    /// The union of the routes' inferred effects.
    pub effects: Effects,
    /// Per path: the union over the routes through it.
    pub path_effects: Vec<Effects>,
    /// Per path: the GC points on its routes, with the frame values live
    /// across each.
    pub path_gc: Vec<BTreeSet<(String, String)>>,
    /// Per path: the charges, as written.
    pub path_units: Vec<BTreeSet<String>>,
    /// Per path: the runtime functions, helpers and intrinsics called.
    pub path_calls: Vec<BTreeSet<String>>,
    /// Locals live across a GC point: the generated Rust roots them.
    pub rooted: BTreeSet<LocalId>,
    /// Control kinds: `next`, `jump`, `back`, `return`, `throw`, `call`.
    pub control: BTreeSet<&'static str>,
    /// Whether some route reads the accumulator.
    pub acc_read: bool,
    /// Whether some route writes the accumulator.
    pub acc_written: bool,
    /// Registers (operand indices) written.
    pub regs_written: BTreeSet<usize>,
    /// The number of routes.
    pub routes: usize,
}

/// The analysis of every item, indexed like the model's lists.
#[derive(Debug, Clone, Default)]
pub struct Flows {
    /// Per operation.
    pub ops: Vec<ItemFlow>,
    /// Per helper.
    pub helpers: Vec<ItemFlow>,
    /// Per intrinsic.
    pub intrinsics: Vec<ItemFlow>,
}

/// Analyzes every body of `m`.
pub fn analyze(m: &Model, diags: &mut Diags) -> Flows {
    let mut out = Flows::default();
    let all_intr = {
        let mut e = Effects::default();
        for i in &m.intrinsics {
            e.add(&i.sig.effects);
        }
        e
    };
    for h in &m.helpers {
        let f = Walker::run_item(m, Item::Helper, &h.sig, &h.body, &[], &all_intr, diags);
        report_effects(&h.name, &h.sig.effects, &f, h.span, &m.cats, diags);
        out.helpers.push(f);
    }
    for h in &m.intrinsics {
        let f = Walker::run_item(m, Item::Intrinsic, &h.sig, &h.body, &[], &all_intr, diags);
        report_effects(
            &format!("%{}", h.name),
            &h.sig.effects,
            &f,
            h.span,
            &m.cats,
            diags,
        );
        out.intrinsics.push(f);
    }
    for o in &m.ops {
        let sig = Sig {
            params: Vec::new(),
            frame_params: Vec::new(),
            after_call: o.after_call,
            facts: o.facts.clone(),
            ret: Ty::Unit,
            effects: o.effects.clone(),
            anchor: o.anchor.clone(),
        };
        let f = Walker::run_item(m, Item::Op, &sig, &o.body, &o.operands, &all_intr, diags);
        report_effects(&o.name, &o.effects, &f, o.span, &m.cats, diags);
        check_acc(o, &f, diags);
        out.ops.push(f);
    }
    out
}

fn report_effects(
    name: &str,
    declared: &Effects,
    f: &ItemFlow,
    span: Span,
    tree: &CatTree,
    diags: &mut Diags,
) {
    let excess = f.effects.excess_over(declared, tree);
    if !excess.is_empty() {
        diags.err(
            span,
            "OC-5",
            format!("`{name}` may {} but does not declare it", excess.join(", ")),
        );
    }
}

/// OC-4: the inferred accumulator use agrees with the declared mode.
fn check_acc(o: &Op, f: &ItemFlow, diags: &mut Diags) {
    let (want_read, want_write) = match o.acc {
        AccMode::None => (false, false),
        AccMode::In => (true, false),
        AccMode::Out => (false, true),
        AccMode::InOut => (true, true),
    };
    if f.acc_read && !want_read {
        diags.err(
            o.span,
            "OC-4",
            format!(
                "`{}` reads the accumulator but declares `acc {}`",
                o.name,
                o.acc.name()
            ),
        );
    }
    if f.acc_written && !want_write {
        diags.err(
            o.span,
            "OC-4",
            format!(
                "`{}` writes the accumulator but declares `acc {}`",
                o.name,
                o.acc.name()
            ),
        );
    }
    if want_read && !f.acc_read {
        diags.err(
            o.span,
            "OC-4",
            format!(
                "`{}` declares `acc {}` but never reads it",
                o.name,
                o.acc.name()
            ),
        );
    }
    if want_write && !f.acc_written {
        diags.err(
            o.span,
            "OC-4",
            format!(
                "`{}` declares `acc {}` but never writes it",
                o.name,
                o.acc.name()
            ),
        );
    }
}

#[derive(Clone, Copy)]
enum Item {
    Op,
    Helper,
    Intrinsic,
}

/// An abstract value: what the checks need to know about an expression.
#[derive(Debug, Clone, Default)]
struct Av {
    /// The integer interval, for integers.
    iv: Option<(i128, i128)>,
    /// Mutable reads it derives from: (epoch, GC epoch, gc-mutable).
    stale: Vec<(u32, u32, bool)>,
    /// Derived from a `repr` field (OC-8).
    repr: bool,
}

impl Av {
    fn join(mut self, other: &Self) -> Self {
        self.stale.extend(other.stale.iter().copied());
        self.repr |= other.repr;
        self
    }
}

#[derive(Debug, Clone)]
enum Ev {
    Def(LocalId),
    Use(LocalId, Span),
    Write(FrameLoc),
    Gc {
        args: Vec<LocalId>,
        what: String,
        span: Span,
    },
}

#[derive(Clone)]
struct St {
    events: Vec<Ev>,
    entered: BTreeSet<usize>,
    locals: BTreeMap<LocalId, Av>,
    epoch: u32,
    gc_epoch: u32,
    blocked: Option<Span>,
    repr_ctrl: bool,
    effects: Effects,
    value: Av,
    units: BTreeSet<String>,
    calls: BTreeSet<String>,
    acc_read: bool,
    acc_written: bool,
    regs_written: BTreeSet<usize>,
    /// The route's user call, once made (OC-17).
    user: Option<Span>,
    /// The call a statement may make as its user call, and its form.
    allowed: Option<(usize, AfterCall)>,
}

#[derive(Clone)]
enum K<'a> {
    Stmts(&'a [TStmt]),
    Tail(&'a TExpr),
    Bind(LocalId),
    SetAcc(Span),
    Return,
    Repr(bool),
}

struct Walker<'a> {
    m: &'a Model,
    item: Item,
    sig: &'a Sig,
    body: &'a Body,
    operands: &'a [(String, OperandKind)],
    all_intr: &'a Effects,
    diags: &'a mut Diags,
    out: ItemFlow,
    /// Routes beyond this many are not walked (a description this large
    /// is rejected: OC-3 wants a small decision tree).
    budget: usize,
}

const MAX_ROUTES: usize = 4096;

fn ptr(e: &TExpr) -> usize {
    std::ptr::from_ref(e) as usize
}

fn ty_iv(t: Ty) -> Option<(i128, i128)> {
    t.range()
}

const SMI: (i128, i128) = (-(1 << 30), (1 << 30) - 1);

impl<'a> Walker<'a> {
    fn run_item(
        m: &'a Model,
        item: Item,
        sig: &'a Sig,
        body: &'a Body,
        operands: &'a [(String, OperandKind)],
        all_intr: &'a Effects,
        diags: &'a mut Diags,
    ) -> ItemFlow {
        let n = body.paths.len();
        let mut w = Walker {
            m,
            item,
            sig,
            body,
            operands,
            all_intr,
            diags,
            out: ItemFlow {
                path_effects: vec![Effects::default(); n],
                path_gc: vec![BTreeSet::new(); n],
                path_units: vec![BTreeSet::new(); n],
                path_calls: vec![BTreeSet::new(); n],
                ..ItemFlow::default()
            },
            budget: MAX_ROUTES,
        };
        let mut st = St {
            events: Vec::new(),
            entered: [0].into_iter().collect(),
            locals: BTreeMap::new(),
            epoch: 0,
            gc_epoch: 0,
            blocked: None,
            repr_ctrl: false,
            effects: Effects::default(),
            value: Av::default(),
            units: BTreeSet::new(),
            calls: BTreeSet::new(),
            acc_read: false,
            acc_written: false,
            regs_written: BTreeSet::new(),
            user: None,
            allowed: None,
        };
        for (i, l) in body.locals.iter().enumerate() {
            if matches!(
                l.origin,
                Origin::RegParam(_)
                    | Origin::FrameParam(_)
                    | Origin::RootedParam(_)
                    | Origin::Param(_)
            ) {
                let id = LocalId(u32::try_from(i).unwrap_or(0));
                st.events.push(Ev::Def(id));
                st.locals.insert(
                    id,
                    Av {
                        iv: ty_iv(l.ty),
                        ..Av::default()
                    },
                );
            }
        }
        let mut k = Vec::new();
        if let Some(t) = &body.block.tail {
            k.push(K::Return);
            k.push(K::Tail(t));
        }
        k.push(K::Stmts(&body.block.stmts));
        w.walk(st, k);
        if w.budget == 0 {
            w.diags.err(
                body.paths[0].span,
                "OC-3",
                format!("more than {MAX_ROUTES} routes: split the description"),
            );
        }
        w.out
    }

    fn err(&mut self, span: Span, check: &'static str, msg: impl Into<String>) {
        self.diags.err(span, check, msg);
    }

    fn is_op(&self) -> bool {
        matches!(self.item, Item::Op)
    }

    fn local(&self, id: LocalId) -> &Local {
        &self.body.locals[id.0 as usize]
    }

    // --- Routes --------------------------------------------------------

    fn walk(&mut self, mut st: St, mut k: Vec<K<'a>>) {
        loop {
            let Some(top) = k.pop() else {
                // The continuation ran out without a terminal.
                if self.is_op() {
                    let span = self.body.paths[0].span;
                    self.err(
                        span,
                        "OC-3",
                        "a route falls through the end without `next`, `jump`, `return` or `throw`",
                    );
                } else if self.sig.ret != Ty::Unit {
                    let span = self.body.paths[0].span;
                    self.err(
                        span,
                        "OC-3",
                        "a route falls through the end without returning a value",
                    );
                }
                self.finish(st, "return");
                return;
            };
            match top {
                K::Stmts([]) => {}
                K::Stmts([s, rest @ ..]) => {
                    k.push(K::Stmts(rest));
                    if !self.stmt(s, &mut st, &mut k) {
                        return;
                    }
                }
                K::Tail(e) => {
                    self.tail(e, st, k);
                    return;
                }
                K::Bind(id) => {
                    let v = std::mem::take(&mut st.value);
                    self.bind(id, v, &mut st);
                }
                K::SetAcc(span) => {
                    self.write(FrameLoc::Acc, span, &mut st, false);
                }
                K::Return => {
                    self.finish(st, "return");
                    return;
                }
                K::Repr(saved) => st.repr_ctrl = saved,
            }
        }
    }

    /// Runs statement `s`. Returns false when the route was handed off
    /// (forked or finished).
    fn stmt(&mut self, s: &'a TStmt, st: &mut St, k: &mut Vec<K<'a>>) -> bool {
        match s {
            TStmt::Let(id, e) => {
                if branching(e) {
                    k.push(K::Bind(*id));
                    k.push(K::Tail(e));
                } else {
                    let v = self.eval(e, st);
                    self.bind(*id, v, st);
                }
                true
            }
            TStmt::SetAcc(e, span) => {
                if branching(e) {
                    k.push(K::SetAcc(*span));
                    k.push(K::Tail(e));
                    return true;
                }
                st.allowed = self.user_call_at(e).map(|p| (p, AfterCall::Result));
                let before = st.user.is_some();
                let _ = self.eval(e, st);
                st.allowed = None;
                let call_result = !before && st.user.is_some();
                self.write(FrameLoc::Acc, *span, st, call_result);
                true
            }
            TStmt::SetReg(r, e, span) => {
                let _ = self.eval(e, st);
                self.write(FrameLoc::Reg(*r), *span, st, false);
                st.regs_written.insert(*r);
                true
            }
            TStmt::Expr(e) => {
                st.allowed = self.user_call_at(e).map(|p| (p, AfterCall::Preserve));
                let _ = self.eval(e, st);
                st.allowed = None;
                true
            }
            TStmt::If(c, t, e) => {
                let cv = self.eval(c, st);
                let rest = std::mem::take(k);
                for truth in [true, false] {
                    let mut st2 = st.clone();
                    let mut k2 = rest.clone();
                    self.refine(c, truth, &mut st2);
                    if cv.repr {
                        k2.push(K::Repr(st2.repr_ctrl));
                        st2.repr_ctrl = true;
                    }
                    if truth {
                        k2.push(K::Stmts(&t.stmts));
                    } else if let Some(e) = e {
                        k2.push(K::Stmts(&e.stmts));
                    }
                    self.fork(st2, k2);
                }
                false
            }
            TStmt::Match(m) => {
                let rest = std::mem::take(k);
                self.match_(m, st, rest, None);
                false
            }
            TStmt::Path(p) => {
                self.enter_path(p.id, st);
                k.push(K::Stmts(&p.body.stmts));
                true
            }
            TStmt::Charge(u, args, span) => {
                self.charge(*u, args, *span, st);
                true
            }
            TStmt::Term(t) => {
                if matches!(k.last(), Some(K::Stmts(rest)) if !rest.is_empty()) {
                    self.err(
                        term_span(t),
                        "OC-3",
                        "statements after a terminal are unreachable",
                    );
                }
                self.terminal(t, std::mem::replace(st, empty_like(st)));
                false
            }
        }
    }

    fn fork(&mut self, st: St, k: Vec<K<'a>>) {
        if self.budget == 0 {
            return;
        }
        self.walk(st, k);
    }

    /// Evaluates a value position that may branch (a match or a block),
    /// leaving the value in `st.value`, then continues with `k`.
    fn tail(&mut self, e: &'a TExpr, mut st: St, mut k: Vec<K<'a>>) {
        match &e.kind {
            TE::Match(m) => self.match_(m, &mut st, k, Some(())),
            TE::Block(b) => {
                if let Some(t) = &b.tail {
                    k.push(K::Tail(t));
                }
                k.push(K::Stmts(&b.stmts));
                self.walk(st, k);
            }
            _ => {
                let allowed = if matches!(k.last(), Some(K::Return))
                    && matches!(self.item, Item::Intrinsic)
                {
                    self.user_call_at(e).map(|p| (p, AfterCall::Result))
                } else {
                    None
                };
                st.allowed = allowed;
                st.value = self.eval(e, &mut st);
                st.allowed = None;
                self.walk(st, k);
            }
        }
    }

    fn match_(&mut self, m: &'a TMatch, st: &mut St, rest: Vec<K<'a>>, value: Option<()>) {
        let scr: Vec<Av> = m.scrutinees.iter().map(|s| self.eval(s, st)).collect();
        let repr = scr.iter().any(|v| v.repr);
        for arm in &m.arms {
            if self.budget == 0 {
                return;
            }
            let mut st2 = st.clone();
            let mut k2 = rest.clone();
            for (p, sv) in arm.pats.iter().zip(&scr) {
                self.bind_pattern(p, sv, &mut st2);
            }
            if let Some(g) = &arm.guard {
                let gv = self.eval(g, &mut st2);
                self.refine(g, true, &mut st2);
                if gv.repr {
                    k2.push(K::Repr(st2.repr_ctrl));
                    st2.repr_ctrl = true;
                }
            }
            if repr {
                k2.push(K::Repr(st2.repr_ctrl));
                st2.repr_ctrl = true;
            }
            match &arm.body {
                TArmBody::Path(p) => {
                    self.enter_path(p.id, &mut st2);
                    if value.is_some()
                        && let Some(t) = &p.body.tail
                    {
                        k2.push(K::Tail(t));
                    }
                    k2.push(K::Stmts(&p.body.stmts));
                    self.fork(st2, k2);
                }
                TArmBody::Block(b) => {
                    if value.is_some()
                        && let Some(t) = &b.tail
                    {
                        k2.push(K::Tail(t));
                    }
                    k2.push(K::Stmts(&b.stmts));
                    self.fork(st2, k2);
                }
                TArmBody::Term(t) => self.terminal(t, st2),
                TArmBody::Expr(e) => {
                    if value.is_some() {
                        k2.push(K::Tail(e));
                    } else {
                        let _ = self.eval(e, &mut st2);
                    }
                    self.fork(st2, k2);
                }
            }
        }
    }

    fn bind_pattern(&mut self, p: &TPat, sv: &Av, st: &mut St) {
        let (id, iv) = match p {
            TPat::Smi(Some(id)) => (*id, Some(SMI)),
            TPat::Num(Some(id)) | TPat::Ref(_, Some(id)) | TPat::Value(Some(id)) => (*id, None),
            _ => return,
        };
        let v = Av {
            iv,
            stale: sv.stale.clone(),
            repr: sv.repr,
        };
        self.bind(id, v, st);
    }

    fn bind(&mut self, id: LocalId, v: Av, st: &mut St) {
        st.events.push(Ev::Def(id));
        st.locals.insert(id, v);
    }

    fn enter_path(&mut self, id: usize, st: &mut St) {
        st.entered.insert(id);
        let info = &self.body.paths[id];
        if info.speculate.is_some()
            && let Some(at) = st.blocked
        {
            let name = info.name.clone();
            let span = info.span;
            self.err(
                span,
                "OC-7",
                format!(
                    "path `{name}` is speculable, but its guard follows an effect at line {} (a GC point, a write, user code, a hook or an input): re-executing the instruction after a deopt would repeat it",
                    at.line
                ),
            );
        }
    }

    /// A frame write; `call_result` when it stores the value of the user
    /// call the same statement just made (the `acc = rt(...)` form).
    fn write(&mut self, loc: FrameLoc, span: Span, st: &mut St, call_result: bool) {
        if st.user.is_some() && !call_result {
            self.err(span, "OC-17", "a frame write after a call that may run user code: the call must be the path's last effect");
        }
        st.events.push(Ev::Write(loc));
        st.blocked.get_or_insert(span);
        if loc == FrameLoc::Acc {
            st.acc_written = true;
        }
    }

    fn effect(&mut self, e: &Effects, span: Span, st: &mut St, user_call: bool) {
        if !e.is_empty() && st.user.is_some() && !user_call {
            self.err(span, "OC-17", "an effect after a call that may run user code: the call must be the path's last effect");
        }
        if e.blocks_speculation() {
            st.blocked.get_or_insert(span);
        }
        st.effects.add(e);
    }

    fn gc_point(&mut self, args: Vec<LocalId>, what: String, span: Span, st: &mut St) {
        st.events.push(Ev::Gc { args, what, span });
        st.gc_epoch += 1;
        st.blocked.get_or_insert(span);
    }

    fn charge(&mut self, u: usize, args: &[TExpr], span: Span, st: &mut St) {
        let units = &self.m.units[u];
        let mut ivs = Vec::new();
        for a in args {
            let v = self.eval(a, st);
            if v.repr {
                self.err(
                    a.span,
                    "OC-8",
                    "a charge depends on a representation (`repr`) field: it would differ by tier",
                );
            }
            ivs.push(v.iv.unwrap_or((i128::from(i64::MIN), i128::from(i64::MAX))));
        }
        if st.repr_ctrl {
            self.err(span, "OC-8", "a charge made under a test of a representation (`repr`) field: it would differ by tier");
        }
        match units_iv(&units.body, &ivs) {
            Some((lo, hi)) if lo >= 0 && hi <= i128::from(i64::MAX) => {}
            Some(_) => self.err(
                span,
                "OC-3",
                format!(
                    "`{}` may be negative or overflow for these arguments",
                    units.name
                ),
            ),
            None => self.err(
                span,
                "OC-3",
                format!(
                    "`{}` may overflow or divide by zero for these arguments",
                    units.name
                ),
            ),
        }
        let name = units.name.clone();
        let text: Vec<String> = args
            .iter()
            .map(|a| expr_text(a, &self.body.locals))
            .collect();
        st.units.insert(format!("{name}({})", text.join(", ")));
        let mut e = Effects::default();
        e.alloc.insert(name.clone());
        self.effect(&e, span, st, false);
        self.gc_point(Vec::new(), format!("charge({name})"), span, st);
    }

    fn terminal(&mut self, t: &'a TTerm, mut st: St) {
        let kind = match t {
            TTerm::Next(_) => "next",
            TTerm::Jump(_, back, span) => {
                if st.user.is_some() {
                    self.err(*span, "OC-17", "a jump after a call that may run user code: the call is followed by `next`");
                }
                if *back {
                    if !st.effects.interrupt {
                        self.err(
                            *span,
                            "OC-4",
                            "a backward jump passes the interrupt check first (`interrupt.poll()`), so every cycle has a safepoint (bytecode note 4.7)",
                        );
                    }
                    "back"
                } else {
                    "jump"
                }
            }
            TTerm::Return(e, span) => {
                if self.is_op() {
                    st.acc_read = true;
                    if st.user.is_some() {
                        self.err(*span, "OC-17", "`return` after a call that may run user code: the call is followed by `next`");
                    }
                } else if let Some(e) = e {
                    st.allowed = if matches!(self.item, Item::Intrinsic) {
                        self.user_call_at(e).map(|p| (p, AfterCall::Result))
                    } else {
                        None
                    };
                    let _ = self.eval(e, &mut st);
                    st.allowed = None;
                }
                "return"
            }
            TTerm::Throw(kind, span) => {
                let mut e = Effects::default();
                e.throws.insert(kind.clone());
                self.effect(&e, *span, &mut st, false);
                self.gc_point(Vec::new(), format!("throw {kind}"), *span, &mut st);
                "throw"
            }
            TTerm::TailCall(kind, args, span) => {
                let mut arg_locals = Vec::new();
                for a in args {
                    let _ = self.eval(a, &mut st);
                    if let TE::Local(l) = a.kind {
                        arg_locals.push(l);
                    }
                }
                if st.user.is_some() {
                    self.err(
                        *span,
                        "OC-17",
                        "two calls that may run user code on one path",
                    );
                }
                if self.sig.after_call != Some(AfterCall::Result) {
                    self.err(
                        *span,
                        "OC-17",
                        "a tail user call's value is the accumulator: declare `after_call result`",
                    );
                }
                let e = Effects {
                    call_user: true,
                    ..Effects::default()
                };
                self.effect(&e, *span, &mut st, true);
                self.gc_point(
                    arg_locals,
                    format!("call_user {}", kind.name()),
                    *span,
                    &mut st,
                );
                st.user = Some(*span);
                st.acc_written = true;
                "call"
            }
        };
        self.finish(st, kind);
    }

    fn finish(&mut self, st: St, control: &'static str) {
        if self.budget == 0 {
            return;
        }
        self.budget -= 1;
        self.out.routes += 1;
        self.out.control.insert(control);
        if st.effects.throws_engine_error() || !st.effects.throws.is_empty() {
            self.out.control.insert("throw");
        }
        self.out.acc_read |= st.acc_read;
        self.out.acc_written |= st.acc_written;
        self.out
            .regs_written
            .extend(st.regs_written.iter().copied());
        self.out.effects.add(&st.effects);
        let gcs = self.rooting(&st);
        for p in &st.entered {
            self.out.path_effects[*p].add(&st.effects);
            self.out.path_units[*p].extend(st.units.iter().cloned());
            self.out.path_calls[*p].extend(st.calls.iter().cloned());
            self.out.path_gc[*p].extend(gcs.iter().cloned());
        }
    }

    /// OC-6 over one route; returns its GC points with the frame values
    /// live across each.
    fn rooting(&mut self, st: &St) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let evs = &st.events;
        let mut defs: BTreeMap<LocalId, usize> = BTreeMap::new();
        for (i, e) in evs.iter().enumerate() {
            if let Ev::Def(l) = e {
                defs.insert(*l, i);
            }
            let Ev::Gc { args, what, span } = e else {
                continue;
            };
            let mut live_frame = BTreeSet::new();
            // Locals defined before this point and used after it.
            let mut seen = BTreeSet::new();
            for (j, later) in evs.iter().enumerate().skip(i + 1) {
                let Ev::Use(l, use_span) = later else {
                    continue;
                };
                let Some(&d) = defs.get(l) else { continue };
                // A redefinition after the GC point starts a new value.
                if evs[i + 1..j]
                    .iter()
                    .any(|x| matches!(x, Ev::Def(m) if m == l))
                {
                    continue;
                }
                let local = self.local(*l).clone();
                if !local.ty.is_heap() || !seen.insert(*l) {
                    continue;
                }
                self.out.rooted.insert(*l);
                let ok = match local.origin {
                    Origin::Frame(loc) => {
                        let written = evs[d..j]
                            .iter() // Any register write counts: two `reg` operands may name one register.
                            .any(|x| {
                                matches!(
                                    (x, loc),
                                    (Ev::Write(FrameLoc::Acc), FrameLoc::Acc)
                                        | (Ev::Write(FrameLoc::Reg(_)), FrameLoc::Reg(_))
                                )
                            });
                        if !written {
                            live_frame.insert(self.loc_text(loc));
                        }
                        !written || args.contains(l)
                    }
                    Origin::RegParam(i) => {
                        live_frame.insert(format!("param {}", self.body.locals[i].name));
                        true
                    }
                    Origin::FrameParam(_) | Origin::RootedParam(_) | Origin::Const => true,
                    Origin::Param(_) | Origin::Temp => args.contains(l),
                };
                if !ok {
                    let line = span.line;
                    self.err(
                        *use_span,
                        "OC-6",
                        format!(
                            "`{}` is live across the GC point at line {line} ({what}) but is neither held in the frame nor an argument of that GC point",
                            local.name
                        ),
                    );
                }
            }
            let live: Vec<String> = live_frame.into_iter().collect();
            out.push((what.clone(), live.join(" ")));
        }
        out
    }

    fn loc_text(&self, loc: FrameLoc) -> String {
        match loc {
            FrameLoc::Acc => "acc".into(),
            FrameLoc::Reg(r) => format!(
                "reg[{}]",
                self.operands.get(r).map_or("?", |o| o.0.as_str())
            ),
        }
    }

    /// The call expression a statement's value is, if it may run user code.
    fn user_call_at(&self, e: &TExpr) -> Option<usize> {
        let eff = self.callee_effects(e)?;
        eff.user().then(|| ptr(e))
    }

    fn callee_effects(&self, e: &TExpr) -> Option<Effects> {
        Some(match &e.kind {
            TE::Runtime(r, _) => self.m.runtimes[*r].sig.effects.clone(),
            TE::Helper(h, _) => self.m.helpers[*h].sig.effects.clone(),
            TE::Intrinsic { .. } => self.all_intr.clone(),
            _ => return None,
        })
    }

    // --- Expressions ---------------------------------------------------

    fn eval(&mut self, e: &TExpr, st: &mut St) -> Av {
        match &e.kind {
            TE::Int(v) => Av {
                iv: Some((*v, *v)),
                ..Av::default()
            },
            TE::Float(_) | TE::Bool(_) | TE::Oddball(_) | TE::ObjKind(_) | TE::UnitStrict => {
                Av::default()
            }
            TE::Local(l) => {
                st.events.push(Ev::Use(*l, e.span));
                let v = st.locals.get(l).cloned().unwrap_or_default();
                self.check_stale(*l, &v, e.span, st);
                v
            }
            TE::Acc => {
                st.acc_read = true;
                Av::default()
            }
            TE::Reg(_) | TE::ConstPool(..) => Av::default(),
            TE::Imm(_) => Av {
                iv: Some(SMI),
                ..Av::default()
            },
            TE::Runtime(..) | TE::Helper(..) | TE::Intrinsic { .. } => self.call(e, st),
            TE::Prim(p, args) => {
                let mut v = Av::default();
                for a in args {
                    let av = self.eval(a, st);
                    v = v.join(&av);
                }
                v.iv = match p {
                    Prim::FToInt32 => ty_iv(Ty::I32),
                    Prim::FToUint32 => ty_iv(Ty::U32),
                    _ => None,
                };
                v
            }
            TE::Field(li, fi, arg) => {
                let mut v = self.eval(arg, st);
                let field = &self.m.layouts[*li].fields[*fi];
                v.iv = ty_iv(field.ty);
                v.stale.clear();
                if let Some(c) = &field.category {
                    let mut eff = Effects::default();
                    eff.reads.insert(c.clone());
                    self.effect(&eff, e.span, st, false);
                    v.stale
                        .push((st.epoch, st.gc_epoch, self.m.cats.gc_mutable(c)));
                }
                if field.repr {
                    v.repr = true;
                }
                v
            }
            TE::KeptPush(a) => {
                let _ = self.eval(a, st);
                let mut eff = Effects::default();
                eff.writes.insert(Cat {
                    name: "kept".into(),
                    arg: None,
                });
                self.effect(&eff, e.span, st, false);
                Av::default()
            }
            TE::Poll => {
                let eff = Effects {
                    interrupt: true,
                    ..Effects::default()
                };
                self.effect(&eff, e.span, st, false);
                self.gc_point(Vec::new(), "interrupt.poll".into(), e.span, st);
                Av::default()
            }
            TE::Smi(a) => {
                let v = self.eval(a, st);
                match v.iv {
                    Some((lo, hi)) if lo >= SMI.0 && hi <= SMI.1 => {}
                    _ => self.err(e.span, "OC-3", "`smi(e)` needs a proof that `e` is in [-2^30, 2^30 - 1]; use `number(f64(e))` otherwise"),
                }
                Av { iv: None, ..v }
            }
            TE::Number(a) => {
                let v = self.eval(a, st);
                let eff = Effects {
                    alloc_internal: true,
                    ..Effects::default()
                };
                self.effect(&eff, e.span, st, false);
                self.gc_point(Vec::new(), "number".into(), e.span, st);
                Av { iv: None, ..v }
            }
            TE::BoolValue(a) => {
                let v = self.eval(a, st);
                Av { iv: None, ..v }
            }
            TE::Alloc(li, fields) => {
                let mut v = Av::default();
                let mut args = Vec::new();
                for f in fields {
                    let av = self.eval(f, st);
                    v = v.join(&av);
                    if let TE::Local(l) = f.kind {
                        args.push(l);
                    }
                }
                let eff = Effects {
                    alloc_internal: true,
                    ..Effects::default()
                };
                self.effect(&eff, e.span, st, false);
                let name = self.m.layouts[*li].name.clone();
                self.gc_point(args, format!("alloc {name}"), e.span, st);
                Av {
                    iv: None,
                    stale: Vec::new(),
                    repr: v.repr,
                }
            }
            TE::Conv(t, a) => {
                let v = self.eval(a, st);
                let iv = v.iv;
                if t.is_int() {
                    let (lo, hi) = t.range().unwrap_or((0, 0));
                    match iv {
                        Some((a, b)) if a >= lo && b <= hi => {}
                        _ => self.err(
                            e.span,
                            "OC-3",
                            format!("the conversion to {t} needs a range proof"),
                        ),
                    }
                    Av { iv, ..v }
                } else {
                    if a.ty.is_int() {
                        let lim = 1i128 << 53;
                        match iv {
                            Some((x, y)) if x >= -lim && y <= lim => {}
                            _ => self.err(e.span, "OC-3", "the conversion to f64 is exact only for |x| <= 2^53: prove the range"),
                        }
                    }
                    Av { iv: None, ..v }
                }
            }
            TE::Bin(op, a, b) => {
                let va = self.eval(a, st);
                let vb = self.eval(b, st);
                let iv = if e.ty.is_int() {
                    self.int_op(*op, va.iv, vb.iv, e.ty, e.span)
                } else {
                    None
                };
                let mut v = va.join(&vb);
                v.iv = iv;
                v
            }
            TE::Same(a, b, _) => {
                let va = self.eval(a, st);
                let vb = self.eval(b, st);
                let mut v = va.join(&vb);
                v.iv = None;
                v
            }
            TE::Un(op, a) => {
                let mut v = self.eval(a, st);
                v.iv = match (op, v.iv) {
                    (UnOp::Neg, Some((lo, hi))) if e.ty.is_int() => {
                        let r = (-hi, -lo);
                        self.fit(r, e.ty, e.span)
                    }
                    (UnOp::BitNot, Some((lo, hi))) if e.ty == Ty::I32 => Some((-hi - 1, -lo - 1)),
                    (UnOp::Neg | UnOp::BitNot, _) if e.ty.is_int() => {
                        if *op == UnOp::Neg {
                            self.err(e.span, "OC-3", "integer negation needs a range proof");
                        }
                        ty_iv(e.ty)
                    }
                    _ => None,
                };
                v
            }
            TE::Match(_) | TE::Block(_) => {
                self.err(
                    e.span,
                    "OC-2",
                    "a `match` or block value here must be bound with `let` first",
                );
                Av::default()
            }
        }
    }

    fn check_stale(&mut self, l: LocalId, v: &Av, span: Span, st: &St) {
        for (epoch, gc_epoch, gc_mut) in &v.stale {
            let name = self.local(l).name.clone();
            if *epoch < st.epoch {
                self.err(
                    span,
                    "OC-9",
                    format!("`{name}` was read from a mutable heap field before a call that may run user code or reach a host hook, and is used after it: read it again after the call"),
                );
                return;
            }
            if *gc_mut && *gc_epoch < st.gc_epoch {
                self.err(
                    span,
                    "OC-9",
                    format!("`{name}` was read from a category a GC point changes (weak slots) and is used after a GC point: read it again"),
                );
                return;
            }
        }
    }

    fn call(&mut self, e: &TExpr, st: &mut St) -> Av {
        let (name, args, eff, frame_params): (String, &Vec<TExpr>, Effects, Vec<bool>) =
            match &e.kind {
                TE::Runtime(r, args) => {
                    let rt = &self.m.runtimes[*r];
                    (rt.name.clone(), args, rt.sig.effects.clone(), Vec::new())
                }
                TE::Helper(h, args) => {
                    let hp = &self.m.helpers[*h];
                    (
                        hp.name.clone(),
                        args,
                        hp.sig.effects.clone(),
                        hp.sig.frame_params.clone(),
                    )
                }
                TE::Intrinsic { .. } => {
                    let names: Vec<String> = self
                        .m
                        .intrinsics
                        .iter()
                        .map(|i| format!("%{}", i.name))
                        .collect();
                    for n in names {
                        st.calls.insert(n);
                    }
                    (
                        "intrinsic".into(),
                        &EMPTY,
                        self.all_intr.clone(),
                        Vec::new(),
                    )
                }
                _ => return Av::default(),
            };
        let mut v = Av::default();
        let mut arg_locals = Vec::new();
        for (i, a) in args.iter().enumerate() {
            let av = self.eval(a, st);
            v = v.join(&av);
            if let TE::Local(l) = a.kind {
                arg_locals.push(l);
            }
            if frame_params.get(i).copied().unwrap_or(false) && !self.frame_backed(a, st) {
                self.err(
                    a.span,
                    "OC-6",
                    format!("`{name}` declares this parameter `frame`, but the argument is not a frame-backed value (a register or the accumulator, unwritten since read)"),
                );
            }
        }
        let user = eff.user();
        if user {
            if matches!(self.item, Item::Helper) {
                self.err(e.span, "OC-17", "a helper's user call would not end its caller's path: call user code from the operation itself");
            }
            if st.user.is_some() {
                self.err(
                    e.span,
                    "OC-17",
                    "two calls that may run user code on one path",
                );
            } else if st.allowed.is_none_or(|(p, _)| p != ptr(e)) {
                self.err(e.span, "OC-17", "post-call work: a call that may run user code is the whole value of `acc = ...`, a call statement, or a tail user call");
            } else if let Some((_, form)) = st.allowed
                && self.sig.after_call != Some(form)
            {
                let want = match form {
                    AfterCall::Result => "result",
                    AfterCall::Preserve => "preserve",
                };
                self.err(e.span, "OC-17", format!("this user call's form leaves the accumulator as `{want}`: declare `after_call {want}`"));
            }
        }
        st.calls.insert(name.clone());
        self.effect(&eff, e.span, st, user);
        if eff.gc() {
            self.gc_point(arg_locals, name, e.span, st);
        }
        if eff.ends_epoch() {
            st.epoch += 1;
        }
        if user {
            st.user = Some(e.span);
        }
        Av {
            iv: ty_iv(e.ty),
            stale: Vec::new(),
            repr: false,
        }
    }

    fn frame_backed(&self, a: &TExpr, st: &St) -> bool {
        match &a.kind {
            TE::Acc | TE::Reg(_) => true,
            TE::Local(l) => match self.local(*l).origin {
                Origin::Frame(loc) => {
                    let Some(d) = st
                        .events
                        .iter()
                        .rposition(|x| matches!(x, Ev::Def(m) if m == l))
                    else {
                        return false;
                    };
                    !st.events[d..]
                        .iter() // Any register write counts: two `reg` operands may name one register.
                        .any(|x| {
                            matches!(
                                (x, loc),
                                (Ev::Write(FrameLoc::Acc), FrameLoc::Acc)
                                    | (Ev::Write(FrameLoc::Reg(_)), FrameLoc::Reg(_))
                            )
                        })
                }
                Origin::RegParam(_) | Origin::FrameParam(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn fit(&mut self, r: (i128, i128), t: Ty, span: Span) -> Option<(i128, i128)> {
        let (lo, hi) = t.range()?;
        if r.0 < lo || r.1 > hi {
            self.err(
                span,
                "OC-3",
                format!("the result may overflow {t}: prove the range or use a wider type"),
            );
            return Some((lo, hi));
        }
        Some(r)
    }

    fn int_op(
        &mut self,
        op: BinOp,
        a: Option<(i128, i128)>,
        b: Option<(i128, i128)>,
        t: Ty,
        span: Span,
    ) -> Option<(i128, i128)> {
        let full = t.range();
        let (Some(a), Some(b)) = (a, b) else {
            if matches!(
                op,
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
            ) {
                self.err(
                    span,
                    "OC-3",
                    format!("`{}` on {t} needs a range proof", op.text()),
                );
            }
            return full;
        };
        match op {
            BinOp::Add => self.fit((a.0 + b.0, a.1 + b.1), t, span),
            BinOp::Sub => self.fit((a.0 - b.1, a.1 - b.0), t, span),
            BinOp::Mul => {
                let c = [a.0 * b.0, a.0 * b.1, a.1 * b.0, a.1 * b.1];
                let r = (*c.iter().min().unwrap_or(&0), *c.iter().max().unwrap_or(&0));
                self.fit(r, t, span)
            }
            BinOp::Div | BinOp::Rem => {
                if b.0 <= 0 && b.1 >= 0 {
                    self.err(
                        span,
                        "OC-3",
                        format!("`{}` by a value that may be zero", op.text()),
                    );
                    return full;
                }
                if op == BinOp::Div {
                    let c = [a.0 / b.0, a.0 / b.1, a.1 / b.0, a.1 / b.1];
                    Some((*c.iter().min().unwrap_or(&0), *c.iter().max().unwrap_or(&0)))
                } else {
                    let m = b.0.abs().max(b.1.abs()) - 1;
                    Some((if a.0 < 0 { -m } else { 0 }, if a.1 > 0 { m } else { 0 }))
                }
            }
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
                if op == BinOp::BitAnd && (a.0 >= 0 || b.0 >= 0) {
                    let hi = if a.0 >= 0 && b.0 >= 0 {
                        a.1.min(b.1)
                    } else if a.0 >= 0 {
                        a.1
                    } else {
                        b.1
                    };
                    return Some((0, hi));
                }
                // Both in [-2^k, 2^k - 1]: so is the result.
                let bits = |x: (i128, i128)| -> u32 {
                    let mut k = 0u32;
                    while k < 64 && !(x.0 >= -(1i128 << k) && x.1 < (1i128 << k)) {
                        k += 1;
                    }
                    k
                };
                let k = bits(a).max(bits(b));
                let r = (-(1i128 << k), (1i128 << k) - 1);
                match full {
                    Some((lo, hi)) => Some((r.0.max(lo), r.1.min(hi))),
                    None => Some(r),
                }
            }
            BinOp::Shl | BinOp::Sar | BinOp::Shr => full,
            _ => None,
        }
    }

    /// Narrows intervals on a condition known to be `truth`.
    fn refine(&mut self, c: &TExpr, truth: bool, st: &mut St) {
        match &c.kind {
            TE::Un(UnOp::Not, x) => self.refine(x, !truth, st),
            TE::Bin(BinOp::And, a, b) if truth => {
                self.refine(a, true, st);
                self.refine(b, true, st);
            }
            TE::Bin(BinOp::Or, a, b) if !truth => {
                self.refine(a, false, st);
                self.refine(b, false, st);
            }
            TE::Bin(
                op @ (BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Eq | BinOp::Ne),
                a,
                b,
            ) => {
                let (l, k, op) = match (&a.kind, &b.kind) {
                    (TE::Local(l), TE::Int(k)) => (*l, *k, *op),
                    (TE::Int(k), TE::Local(l)) => (*l, *k, flip(*op)),
                    _ => return,
                };
                let op = if truth { op } else { negate(op) };
                let Some(v) = st.locals.get_mut(&l) else {
                    return;
                };
                let Some((lo, hi)) = v.iv else { return };
                let (lo, hi) = match op {
                    BinOp::Lt => (lo, hi.min(k - 1)),
                    BinOp::Le => (lo, hi.min(k)),
                    BinOp::Gt => (lo.max(k + 1), hi),
                    BinOp::Ge => (lo.max(k), hi),
                    BinOp::Eq => (lo.max(k), hi.min(k)),
                    _ => (lo, hi),
                };
                v.iv = Some((lo, hi.max(lo)));
            }
            _ => {}
        }
    }
}

static EMPTY: Vec<TExpr> = Vec::new();

fn flip(op: BinOp) -> BinOp {
    match op {
        BinOp::Lt => BinOp::Gt,
        BinOp::Le => BinOp::Ge,
        BinOp::Gt => BinOp::Lt,
        BinOp::Ge => BinOp::Le,
        o => o,
    }
}

fn negate(op: BinOp) -> BinOp {
    match op {
        BinOp::Lt => BinOp::Ge,
        BinOp::Le => BinOp::Gt,
        BinOp::Gt => BinOp::Le,
        BinOp::Ge => BinOp::Lt,
        BinOp::Eq => BinOp::Ne,
        BinOp::Ne => BinOp::Eq,
        o => o,
    }
}

fn branching(e: &TExpr) -> bool {
    matches!(e.kind, TE::Match(_) | TE::Block(_))
}

fn term_span(t: &TTerm) -> Span {
    match t {
        TTerm::Next(s)
        | TTerm::Jump(_, _, s)
        | TTerm::Return(_, s)
        | TTerm::Throw(_, s)
        | TTerm::TailCall(_, _, s) => *s,
    }
}

fn empty_like(st: &St) -> St {
    St {
        events: Vec::new(),
        entered: BTreeSet::new(),
        locals: BTreeMap::new(),
        epoch: st.epoch,
        gc_epoch: st.gc_epoch,
        blocked: None,
        repr_ctrl: false,
        effects: Effects::default(),
        value: Av::default(),
        units: BTreeSet::new(),
        calls: BTreeSet::new(),
        acc_read: false,
        acc_written: false,
        regs_written: BTreeSet::new(),
        user: None,
        allowed: None,
    }
}

/// The interval of a units formula for argument intervals `args`, or
/// `None` if it may divide by zero.
fn units_iv(e: &TExpr, args: &[(i128, i128)]) -> Option<(i128, i128)> {
    match &e.kind {
        TE::Int(v) => Some((*v, *v)),
        TE::Local(l) => args.get(l.0 as usize).copied(),
        TE::Bin(op, a, b) => {
            let a = units_iv(a, args)?;
            let b = units_iv(b, args)?;
            Some(match op {
                BinOp::Add => (a.0 + b.0, a.1 + b.1),
                BinOp::Sub => (a.0 - b.1, a.1 - b.0),
                BinOp::Mul => {
                    let c = [a.0 * b.0, a.0 * b.1, a.1 * b.0, a.1 * b.1];
                    (*c.iter().min()?, *c.iter().max()?)
                }
                BinOp::Div => {
                    if b.0 <= 0 && b.1 >= 0 {
                        return None;
                    }
                    let c = [a.0 / b.0, a.0 / b.1, a.1 / b.0, a.1 / b.1];
                    (*c.iter().min()?, *c.iter().max()?)
                }
                _ => return None,
            })
        }
        _ => None,
    }
}

/// An argument expression as text, for the facts' charge formulas.
pub fn expr_text(e: &TExpr, locals: &[Local]) -> String {
    match &e.kind {
        TE::Int(v) => v.to_string(),
        TE::Local(l) => locals
            .get(l.0 as usize)
            .map_or_else(|| "?".into(), |x| x.name.clone()),
        TE::Bin(op, a, b) => format!(
            "({} {} {})",
            expr_text(a, locals),
            op.text(),
            expr_text(b, locals)
        ),
        TE::Conv(t, a) => format!("{t}({})", expr_text(a, locals)),
        _ => "…".into(),
    }
}
