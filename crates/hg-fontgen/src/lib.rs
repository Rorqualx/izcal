//! `hg-fontgen`: the `.hgfont` description language and its generator.
//! Tier T, layer `tools` (fonts and text 6.2 and 6.15, G29; TX-M0).
//!
//! A description, then a generator, then code: font tables are described
//! in `.hgfont`, written from the OpenType prose, and this crate emits
//! the reader, the proofs, the fuzz grammar and the anchors. `hg-fontfmt`
//! holds nothing hand-written but its byte readers, and G29 makes that
//! stick — a hand edit fails CI.
//!
//! ```text
//! hg-fontgen generate      write hg-fontfmt's generated files
//! hg-fontgen check         fail if what is checked in is stale
//! ```
//!
//! Nothing here derives from another project's description of OpenType:
//! no official machine-readable one exists, and the community ones are
//! partial or another project's artifact (M-1 part 3).

pub mod check;
pub mod cli;
pub mod emit;
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
