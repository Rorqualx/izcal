//! `hg-style-reference`: the independent reference style engine
//! (ST-D12, G20). Tools, tier T.
//!
//! **What makes it worth having.** `hg-style` is fast: a rule index,
//! an ancestor filter, invalidation sets, style sharing, parallel
//! traversal. Every one of those is an optimization, and an
//! optimization is a claim — "this answer equals the answer you would
//! get by doing it the slow way". This crate *is* the slow way. It
//! matches every rule against every element, cascades the whole list,
//! and keeps nothing between passes. ST-4 and ST-6 are both ship
//! blockers phrased against it.
//!
//! **G20: it shares no code with the production engine.** Not the
//! tokenizer, not the selector parser, not the cascade. Its only
//! project-local dependency is [`hg_tree_traits`] — plus `hg-types`
//! and webref's data when those arrive. The two remaining common-mode
//! risks are named in the style spec's Section 10 and are exactly
//! those shared things; everything else is written twice, from the
//! prose, so that agreement means something.
//!
//! # What is here at ST-M0
//!
//! - [`token`] — the CSS Syntax 3 tokenizer, §4, with §3.3's
//!   preprocessing folded in.
//! - [`selector`] — Selectors 3: the grammar, specificity, and a
//!   matcher over [`hg_tree_traits::Tree`].
//! - [`cascade`] — origins, cascade layers, importance, specificity
//!   and order of appearance.
//!
//! # What is not here yet
//!
//! The spec-literal value interpreter over webref's grammars, the
//! substitution interpreter, `:has()`, container queries, nesting and
//! the conditional group rules (`@media`, `@supports`). Each is named
//! in the ST-M0 review's agenda; none is stubbed here, because a stub
//! that answered would be worse than nothing in an oracle. What
//! [`sheet`] cannot read it records in [`sheet::Sheet::skipped`] and
//! [`sheet::Sheet::refused`], so a differential never mistakes "did
//! not read it" for "read it and agreed".

pub mod cascade;
pub mod selector;
pub mod sheet;
pub mod token;

pub use cascade::{Cascade, Declaration, Origin, Rule};
pub use selector::{Selector, SelectorList, Specificity};
pub use token::{Token, tokenize};
