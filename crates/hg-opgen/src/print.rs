//! Prints patterns and expressions back as `.hgop` text, for the guard
//! column of the operation facts.

use crate::ast::{Expr, Literal, Pattern, UnOp};

/// A pattern as written.
pub fn pattern(p: &Pattern) -> String {
    match p {
        Pattern::Oddball(n) => n.text.clone(),
        Pattern::Smi(b) => format!("Smi({})", b.text),
        Pattern::Num(b) => format!("Num({})", b.text),
        Pattern::Ref(k, b) => format!("Ref<{}>({})", k.text, b.text),
        Pattern::Value(b) => format!("Value({})", b.text),
        Pattern::Tuple(ps, _) => {
            let parts: Vec<String> = ps.iter().map(pattern).collect();
            format!("({})", parts.join(", "))
        }
        Pattern::Or(ps) => {
            let parts: Vec<String> = ps.iter().map(pattern).collect();
            parts.join(" | ")
        }
        Pattern::Wild(_) => "_".into(),
    }
}

/// An expression as written (without its original spacing).
pub fn expr(e: &Expr) -> String {
    match e {
        Expr::Lit(Literal::Int(v), _) => v.to_string(),
        Expr::Lit(Literal::Float(f), _) => f.clone(),
        Expr::Lit(Literal::Bool(b), _) => b.to_string(),
        Expr::Ident(n) => n.text.clone(),
        Expr::Acc(_) => "acc".into(),
        Expr::Reg(n) => format!("reg[{}]", n.text),
        Expr::ConstPool(n) => format!("const[{}]", n.text),
        Expr::Call(n, args) => format!("{}({})", n.text, list(args)),
        Expr::QCall(q, f, args) => format!("{}.{}({})", q.text, f.text, list(args)),
        Expr::Alloc(k, fields) => {
            let parts: Vec<String> = fields
                .iter()
                .map(|(f, e)| format!("{}: {}", f.text, expr(e)))
                .collect();
            format!("alloc {} {{ {} }}", k.text, parts.join(", "))
        }
        Expr::Match(_) => "match …".into(),
        Expr::Block(_) => "{ … }".into(),
        Expr::Binary(op, l, r, _) => format!("{} {} {}", atom(l), op.text(), atom(r)),
        Expr::Unary(op, x, _) => {
            let o = match op {
                UnOp::Not => "!",
                UnOp::Neg => "-",
                UnOp::BitNot => "~",
            };
            format!("{o}{}", atom(x))
        }
        Expr::Tuple(parts, _) => format!("({})", list(parts)),
    }
}

fn atom(e: &Expr) -> String {
    match e {
        Expr::Binary(..) => format!("({})", expr(e)),
        _ => expr(e),
    }
}

fn list(args: &[Expr]) -> String {
    let parts: Vec<String> = args.iter().map(expr).collect();
    parts.join(", ")
}
