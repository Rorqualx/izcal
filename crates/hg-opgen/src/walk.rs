//! Visits every expression of a typed body, in source order.

use crate::tir::*;

/// Calls `f` on every expression in `b`, outermost first.
pub fn block(b: &TBlock, f: &mut dyn FnMut(&TExpr)) {
    for s in &b.stmts {
        stmt(s, f);
    }
    if let Some(t) = &b.tail {
        expr(t, f);
    }
}

fn stmt(s: &TStmt, f: &mut dyn FnMut(&TExpr)) {
    match s {
        TStmt::Let(_, e) | TStmt::SetAcc(e, _) | TStmt::SetReg(_, e, _) | TStmt::Expr(e) => {
            expr(e, f);
        }
        TStmt::If(c, t, e) => {
            expr(c, f);
            block(t, f);
            if let Some(e) = e {
                block(e, f);
            }
        }
        TStmt::Match(m) => match_(m, f),
        TStmt::Path(p) => block(&p.body, f),
        TStmt::Charge(_, args, _) => {
            for a in args {
                expr(a, f);
            }
        }
        TStmt::Term(t) => term(t, f),
    }
}

fn term(t: &TTerm, f: &mut dyn FnMut(&TExpr)) {
    match t {
        TTerm::Return(Some(e), _) => expr(e, f),
        TTerm::TailCall(_, args, _) => {
            for a in args {
                expr(a, f);
            }
        }
        _ => {}
    }
}

fn match_(m: &TMatch, f: &mut dyn FnMut(&TExpr)) {
    for s in &m.scrutinees {
        expr(s, f);
    }
    for a in &m.arms {
        if let Some(g) = &a.guard {
            expr(g, f);
        }
        match &a.body {
            TArmBody::Path(p) => block(&p.body, f),
            TArmBody::Block(b) => block(b, f),
            TArmBody::Term(t) => term(t, f),
            TArmBody::Expr(e) => expr(e, f),
        }
    }
}

/// Calls `f` on `e` and every expression inside it.
pub fn expr(e: &TExpr, f: &mut dyn FnMut(&TExpr)) {
    f(e);
    match &e.kind {
        TE::Runtime(_, args) | TE::Helper(_, args) | TE::Prim(_, args) | TE::Alloc(_, args) => {
            for a in args {
                expr(a, f);
            }
        }
        TE::Field(_, _, a)
        | TE::KeptPush(a)
        | TE::Smi(a)
        | TE::Number(a)
        | TE::BoolValue(a)
        | TE::Conv(_, a)
        | TE::Un(_, a) => expr(a, f),
        TE::Bin(_, a, b) | TE::Same(a, b, _) => {
            expr(a, f);
            expr(b, f);
        }
        TE::Match(m) => match_(m, f),
        TE::Block(b) => block(b, f),
        TE::Int(_)
        | TE::Float(_)
        | TE::Bool(_)
        | TE::Local(_)
        | TE::Acc
        | TE::Reg(_)
        | TE::ConstPool(..)
        | TE::Imm(_)
        | TE::Oddball(_)
        | TE::ObjKind(_)
        | TE::Intrinsic { .. }
        | TE::Poll
        | TE::UnitStrict => {}
    }
}
