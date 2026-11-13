//! `hg-layout-reference`: the independent reference layout engine.
//! Tools, tier T (LY-D5).
//!
//! **Why it exists.** `hg-layout` will have a constraint-space cache,
//! an incremental path, a parallel path and a from-scratch path. Each
//! is a claim that it computes the same answer a slower way would.
//! LY-3, LY-4, LY-32, LY-40 and G42 are those claims, and every one of
//! them is phrased against this engine.
//!
//! **G41: it shares no project-local code** with `hg-layout`,
//! `hg-box-tree`, `hg-layout-math` or `hg-fragment` — only
//! `hg-tree-traits`, `hg-types`' geometry, `hg-style`'s read API and
//! `hg-text`. So it builds its own box tree ([`block::RefBox`]), reads
//! its own computed values ([`style::BoxStyle`]) and produces its own
//! fragments ([`block::RefFragment`]). Two engines that shared any of
//! those would agree about a bug in the shared part, and agreement is
//! the entire output of a differential.
//!
//! # What is here at LY-M0
//!
//! CSS 2.2 **block layout**, written from the prose, step by step:
//! §10.3.3's constraint equation, §10.4's min and max width as the
//! three-step algorithm it is, §10.6.3's four ordered clauses for an
//! automatic height, and §8.3.1's collapsing margins.
//!
//! Inline layout takes a **line count and a line height** as inputs
//! rather than laying lines out, because line breaking needs
//! `hg-text`, which is the text track's. That is a stated limit, not a
//! stub: the clause of §10.6.3 that lines feed is implemented, and
//! what is missing is named.

pub mod block;
pub mod style;

pub use block::{Collapsed, Dir, RefBox, RefFragment, UsedWidth, layout, used_width_constrained};
pub use style::{BoxStyle, Display, MaxSize, MinSize, Overflow, Size};
