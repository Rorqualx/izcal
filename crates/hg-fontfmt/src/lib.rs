//! `hg-fontfmt`: readers for OpenType tables, generated from `.hgfont`.
//!
//! Everything but [`read`] is written by `hg-fontgen` from the
//! descriptions in `tables/`, and G29 keeps it that way: a hand edit
//! fails CI. The descriptions are the one source of truth, as `.hgop` is
//! for the engine's operations.
//!
//! **Memory safety never depends on validation** (fonts and text 6.2).
//! Every accessor checks its bounds on every read and returns `None`
//! rather than panic, for any bytes at all — which is what TX-11's Kani
//! harnesses prove, one per accessor. Validation decides *acceptance*,
//! which is a separate, web-observable thing and arrives with the
//! loader.

pub mod generated;
pub mod read;

pub use generated::tables;
