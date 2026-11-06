//! `hg-test262`: Izcal's test262 conformance runner. Tier T, layer
//! `tools` (coding standard 6.2; JS engine spec, crate table: "test262
//! runner (both strictness modes, `$262` host)").
//!
//! ORC-1 asks for an engine's oracle to run in CI before any engine code
//! is generated. At JS-M0 Izcal has no parser yet, so the runner is
//! validated against engine262, used strictly as a black-box, test-only
//! oracle ([`engine262`]); at JS-M1 Izcal's engine implements the same
//! [`engine::Engine`] trait.
//!
//! The method comes from test262's own prose, INTERPRETING.md and
//! CONTRIBUTING.md, at the pinned revision (`tools/test-data.toml`); no
//! other runner's code is read (HGBA-2 M-1 part 3).
//!
//! # Pipeline
//!
//! 1. [`runner::discover`] lists `test/**/*.js`, skipping `_FIXTURE` files.
//! 2. [`frontmatter`] parses each file's `/*--- ---*/` block.
//! 3. [`exclusions`] and the engine's [`engine::HostCapabilities`] decide
//!    what is not run, and say why.
//! 4. [`expand`] turns a file into runs: modes, the strict prefix, the
//!    harness files.
//! 5. An [`engine::Engine`] runs each [`engine::Job`]; [`classify`] turns
//!    the [`engine::Outcome`] into a [`classify::Status`].
//! 6. [`ledger`] records the runs, summarises them by directory and by
//!    feature, and compares two ledgers.

pub mod classify;
pub mod data;
pub mod engine;
pub mod engine262;
pub mod exclusions;
pub mod expand;
pub mod frontmatter;
pub mod ledger;
pub mod runner;
