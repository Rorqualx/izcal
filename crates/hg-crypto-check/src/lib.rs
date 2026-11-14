//! `hg-crypto-check`: the checks a functional test cannot make (G36).
//!
//! §6.21 gives this crate three jobs — "constant-time checks per ISA
//! (binary taint check, statistical timing tests), Wycheproof and ACVP
//! runners, differential fuzzing of primitives against black-box
//! implementations" — and §9.1 makes the first of them a **merge
//! gate**: "Secret-dependent code in `hg-crypto` merges only after
//! `hg-crypto-check` passes on every target ISA (G36, NW-50)."
//!
//! A gate has to exist before the code it gates. That is why this
//! crate is NW-M0's and not NW-M1's, and why what it holds today is
//! the two parts that can be built without the primitives: the
//! Wycheproof runner, and the timing harness with a planted leak it
//! has to find.
//!
//! # What is here
//!
//! - [`vectors`]: reading Wycheproof's JSON, with the structure the
//!   schemas describe rather than a guess at it.
//! - [`run`]: running the suites `hg-crypto` has a primitive for, and
//!   **classifying the ones it does not**, so the report is a map of
//!   what each primitive still owes rather than a pass rate over the
//!   two that exist.
//! - [`timing`]: the statistical timing test, with the positive
//!   control that makes it mean something.
//!
//! # What is not here, and why
//!
//! - **The ACVP runner.** NW-M0's row names it beside Wycheproof.
//!   ACVP's published vector sets cover AES, SHA, ECDSA, RSA and the
//!   SP 800-56 key agreements — none of which `hg-crypto` implements
//!   yet — so a runner for them would have nothing to run and nothing
//!   to be wrong about. It is NW-M0's remaining work, named in the
//!   design review's agenda rather than stubbed here.
//! - **The binary taint check.** It reads a compiled object per ISA
//!   and needs a disassembler; the statistical test is the half that
//!   runs in CI, and the taint check runs per ISA outside it.
//! - **Differential fuzzing against black-box implementations.** Needs
//!   the primitives.

pub mod run;
pub mod timing;
pub mod vectors;

pub use run::{Outcome, Summary};
pub use vectors::{Case, Group, Suite};

/// How many vector files the pinned set holds, measured 13 November
/// 2026 at `vectors::PIN`.
pub const FILES: usize = 359;

/// How many algorithms they cover. Four of the 113 are container
/// formats rather than primitives (the JSON Web suites), named by
/// their file because the files carry no `algorithm`.
pub const ALGORITHMS: usize = 113;

/// How many cases, over every algorithm.
pub const CASES: usize = 145_424;

/// HMAC-SHA-256 cases that pass, and the number that exist: **174 of
/// 174**, measured 13 November 2026.
///
/// A number that can only go up. The suite covers keys shorter than,
/// equal to and longer than the block, truncated tags down to 8 bits,
/// and tags of the wrong length -- the cases a MAC written from the
/// RFC's worked examples alone passes by luck.
pub const HMAC_SHA256_BASELINE: usize = 174;

/// HKDF-SHA-256 cases that pass: **86 of 86**, measured 13 November
/// 2026. Includes RFC 5869's own three and the lengths at and past
/// 255 times the hash output.
pub const HKDF_SHA256_BASELINE: usize = 86;
