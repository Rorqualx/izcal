//! `hg-cssgen`: the `.hgprop` overlay and the property table it builds.
//! Tier T, layer `tools` (style spec 6.6, ST-D6, ST-M0).
//!
//! One description per property, from the specifications' own data.
//! Two inputs:
//!
//! 1. **`@webref/css`** at a pinned version — the W3C's extraction of
//!    CSS features from the published specifications: each property's
//!    value grammar, initial value, inheritance, animation type,
//!    longhands and defining URL.
//! 2. **Izcal's `.hgprop` overlay** — what the specifications leave as
//!    prose, and a restatement of the webref fields the engine was
//!    written against.
//!
//! The restatement is the point. The build **fails when the overlay and
//! webref disagree**, so a specification change reaches Izcal as an
//! error naming the difference rather than as silent drift ([SP 6.6][SP],
//! ST-7). Blink generates its property table from a private one;
//! generating from the specifications' own extracted data, with a
//! failing cross-check, is what makes the table's provenance checkable.
//!
//! Using webref's *data* is M-1-clean. Using CSSTree, which webref's
//! JavaScript API happens to parse grammars with, or any other CSS
//! parser library, is not — so neither is installed and neither is
//! called.
//!
//! The outputs, all checked in:
//!
//! - `crates/hg-css-props/src/generated/kinds.rs` — the closed set of
//!   computed-value kinds;
//! - `.../properties.rs` — the table, the per-property constants and the
//!   name index;
//! - `.../grammars.rs` — the grammars and URLs, behind the `reference`
//!   feature;
//! - `.../properties.tsv` — every column in one table, for triage.
//!
//! [SP]: ../../../reports/Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md

pub mod check;
pub mod cli;
pub mod emit;
pub mod syntax;
pub mod webref;

pub use check::{Disagreement, Model};
pub use syntax::{Error, File};

/// Parses and checks every overlay file, in the order given.
///
/// # Errors
/// [`Error`] naming the file and line of the first problem.
pub fn model(sources: &[(String, String)]) -> Result<Model, Error> {
    let mut files = Vec::new();
    for (name, text) in sources {
        files.push((name.clone(), syntax::parse(text, name)?));
    }
    check::check(&files)
}
