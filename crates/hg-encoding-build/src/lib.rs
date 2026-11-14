//! `hg-encoding-build`: `hg-encoding`'s tables, from the Encoding
//! Standard's own data. Tier T, layer `tools` (style spec 6.6, ST-D16).
//!
//! The standard publishes two files beside its prose: `encodings.json`,
//! which lists every encoding and the labels that select it, and
//! `indexes.json`, which holds the index tables the legacy decoders and
//! encoders are defined over. They are pinned by content hash, fetched
//! into a cache and never committed; the tables compiled from them are
//! checked in and ship, as WHATWG standards data (HGBA-2 2.3.2 M-1
//! part 1, style spec 7).
//!
//! `encoding_rs` is excluded under M-1: it was written for Gecko and
//! does browser-engine work (ST-D16).
//!
//! The outputs:
//!
//! - `crates/hg-encoding/src/generated/encodings.rs` — the `Encoding`
//!   enum, its names and the label table;
//! - `.../indexes.rs` — every index, by pointer and by code point;
//! - `.../encodings.tsv` — the names, groups and labels, for review.

pub mod cli;
pub mod data;
pub mod emit;

pub use data::{Data, Encoding, Index};
