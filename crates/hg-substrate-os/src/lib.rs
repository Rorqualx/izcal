//! Izcal's OS substrate (tier O, G3 — coding standard 0.3.7 tier table).
//!
//! The one crate allowed ambient OS effects. Holds the sandbox backends
//! (security scope 6.7: `hg-sandbox` lives inside `hg-substrate-os`),
//! the process-kind machinery and the sanctioned thread interface.
//!
//! First drop (FP-M0 half B, 23 October 2026): the macOS sandbox backend
//! with the empty-process profile, the tier probe, the sanctioned thread
//! seed, and the two FP-M0 binaries (`fp-m0-probe`, the supervisor, and
//! `fp-m0-empty`, the sandboxed empty process).

pub mod sandbox;
pub mod thread;
