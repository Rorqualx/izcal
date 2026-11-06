//! `hg-opgen`: Izcal's operation generator (tier T, layer tools; JS spec
//! 6.3, 6.15 and G16; design review R0-O1 to R0-O12). It reads the
//! `.hgop` operation descriptions, checks them (OC-1 to OC-17 of the
//! language note's Section 6) and writes the opcode table, the reference
//! interpreter, the operation facts and the anchor index.
//!
//! Extensions of the language note's grammar that JS-M0 needed, all
//! recorded in the generator's report:
//! - `units name(params) = expr;` declares a canonical-units formula
//!   (R0-B10) in the descriptions themselves;
//! - a mutable layout field names its category (`target: Empty | Value
//!   in weak;`) and may be marked `repr` (OC-8);
//! - `layout Rope is String { ... }` declares an allocation-only
//!   representation of a kind;
//! - qualified calls `kept.push(v)`, `interrupt.poll()`, `unit.strict()`
//!   and `f64.op(...)` are the typed heap operations and primitives;
//! - `intrinsic(id, list, count)` is `CallIntrinsic`'s dispatch;
//! - `ensures result is P` is the contract form.
//!
//! Threading: a single-threaded batch tool.

pub mod anchors;
pub mod ast;
pub mod check;
pub mod cli;
pub mod diag;
pub mod emit_interp;
pub mod emit_tables;
pub mod flow;
pub mod lex;
pub mod model;
pub mod parse;
pub mod print;
pub mod tir;
pub mod walk;

use diag::Diags;

/// Parses and checks the given sources (name, text). Returns the model
/// and the flow analysis, or every rejection.
///
/// # Errors
/// The rejections, sorted by position.
pub fn load(sources: &[(String, String)]) -> Result<(model::Model, flow::Flows), Diags> {
    let mut diags = Diags::default();
    let mut items = Vec::new();
    for (i, (_, text)) in sources.iter().enumerate() {
        let file = u16::try_from(i).unwrap_or(u16::MAX);
        match lex::lex(text, file).and_then(|t| parse::parse(&t)) {
            Ok(it) => items.push(it),
            Err(d) => {
                diags.push(d);
                items.push(Vec::new());
            }
        }
    }
    if !diags.is_empty() {
        return Err(diags.sorted());
    }
    let input = check::Input {
        files: sources.iter().map(|(n, _)| n.clone()).collect(),
        items,
    };
    let model = check::check(&input, &mut diags);
    if !diags.is_empty() {
        return Err(diags.sorted());
    }
    let flows = flow::analyze(&model, &mut diags);
    if !diags.is_empty() {
        return Err(diags.sorted());
    }
    Ok((model, flows))
}

/// OC-5's report: declarations wider than what their routes infer (allowed,
/// but the optimizer facts are then needlessly pessimistic).
pub fn notes(m: &model::Model, f: &flow::Flows) -> Vec<String> {
    let mut out = Vec::new();
    let mut one = |name: &str, declared: &model::Effects, inferred: &model::Effects| {
        let wider = declared.excess_over(inferred, &m.cats);
        if !wider.is_empty() {
            out.push(format!(
                "note: OC-5: `{name}` declares {} that no route infers",
                wider.join(", ")
            ));
        }
    };
    for (o, fl) in m.ops.iter().zip(&f.ops) {
        one(&o.name, &o.effects, &fl.effects);
    }
    for (i, fl) in m.intrinsics.iter().zip(&f.intrinsics) {
        one(&format!("%{}", i.name), &i.sig.effects, &fl.effects);
    }
    for (h, fl) in m.helpers.iter().zip(&f.helpers) {
        one(&h.name, &h.sig.effects, &fl.effects);
    }
    out
}
