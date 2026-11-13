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
//! Work in progress: the language and its checker are here; the emitter
//! and the state descriptions follow.

pub mod check;
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
