//! `hg-css-props`: Izcal's CSS property table. L2p, tier R (style spec
//! 6.6, ST-D6).
//!
//! Every property the web platform defines, with what the cascade needs
//! to know about it: its initial value, whether it inherits, which
//! conversion takes its specified value to a computed one, what a change
//! to it dirties, and — for a shorthand — the longhands it expands to.
//!
//! **Nothing here is hand-written.** `hg-cssgen` builds it from two
//! inputs: `@webref/css` at a pinned version, which is the W3C's own
//! extraction of the published specifications, and Izcal's `.hgprop`
//! overlay, which carries what the specifications leave as prose. The
//! overlay also *restates* the webref fields the engine was written
//! against, and the build fails when the two disagree, so a
//! specification change arrives as a build error with the difference
//! named rather than as silent drift ([SP 6.6][SP], ST-7).
//!
//! The one thing this crate does not yet do is **convert** a specified
//! value to a computed one. [`ComputedKind`] says which conversion each
//! property needs; the conversions themselves, and the per-property
//! value parsers generated from the grammars, are ST-M1. What exists
//! here is the table those are generated against, and the check that
//! keeps it honest.
//!
//! [SP]: ../../../reports/Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md

mod generated;
mod table;

pub use generated::kinds::ComputedKind;
pub use table::{Affects, Property};

#[cfg(feature = "reference")]
pub use table::reference;
