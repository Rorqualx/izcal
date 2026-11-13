//! `hg-htmlgen`: the `.hgtok` description language and its generator.
//! Tier T, layer `tools` (style spec 6, ST-D3, ST-M0).
//!
//! The HTML tokenizer's states are written in `.hgtok`, from the HTML
//! Standard's prose, and this crate generates the tokenizer, the
//! reference stepper that serves as its oracle, and the transition
//! coverage table the anchors and fuzzers read. The description and the
//! generator are Izcal formats only: no second source language, which is
//! the lesson of Gecko's translator.
//!
//! The outputs:
//!
//! - `generated/states.rs` — the `State` enum, the five states the tree
//!   builder may switch to, and the reference stepper's table;
//! - `generated/tokenizer.rs` — the tokenizer, as plain Rust;
//! - `generated/entities.rs` — the 2,231 named character references,
//!   from WHATWG's pinned `entities.json`;
//! - `generated/transitions.tsv` and `generated/anchors.tsv` — the
//!   coverage table ANC-1 and the fuzzers read, and the anchor index.

pub mod check;
pub mod cli;
pub mod emit;
pub mod entities;
pub mod json;
pub mod syntax;

pub use check::Model;
pub use syntax::{Error, File};

/// Parses and checks every description, in the order given.
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
