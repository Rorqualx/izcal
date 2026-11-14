//! `hg-crypto`: the one place cryptography is implemented (NW-D23).
//!
//! G36: "Only `hg-crypto` implements a cryptographic primitive or
//! depends on a cryptography crate." The rule exists because the
//! constant-time requirement needs a boundary — code that must not
//! branch or index on a secret cannot be audited if it is spread
//! across eleven protocol crates — and because one implementation is
//! the only way the update verifier, the trust-data verifier, TLS and
//! the WebCrypto spec's primitives can be the same code.
//!
//! §6.21 puts the crate in **L2s**, the shared sub-layer, for exactly
//! that reason.
//!
//! # What NW-M0 asks for, and what is here
//!
//! The milestone row asks for "`hg-crypto`'s interfaces with every pin
//! resolved and licence-checked". So:
//!
//! - the **interfaces** are [`Hash`], [`Mac`] and [`Kdf`], plus the
//!   secret types of [`secret`] and the constant-time helpers of
//!   [`ct`];
//! - the **pins** are `pins.toml`, one row per primitive, naming where
//!   it will come from and under what licence. The test that checks it
//!   is in `hg-crypto-check`, because G3 forbids a tier P crate
//!   ambient file access and CS-AS-1 binds that to its tests too;
//! - three primitives are implemented, because the pins alone would
//!   leave the oracle with nothing to run: **SHA-256** (FIPS 180-4),
//!   **HMAC** (RFC 2104, FIPS 198-1) and **HKDF** (RFC 5869), each
//!   Izcal's own code written from the published standard. They are
//!   the key schedule's floor: TLS 1.3's `HKDF-Expand-Label`
//!   (RFC 8446 §7.1) is built from them and nothing else.
//!
//! Everything else — X25519, P-256, P-384, Ed25519, RSA, AES-GCM,
//! ChaCha20-Poly1305, HPKE — has a pin row and no code. NW-M1 is where
//! they land, and NW-50's constant-time gate is what lets them merge.
//!
//! # Constant time
//!
//! §9.1: "Secret-dependent code in `hg-crypto` merges only after
//! `hg-crypto-check` passes on every target ISA (G36, NW-50)." A
//! timing leak cannot be found by functional tests, so the gate is a
//! separate tool and not a test in this crate. What this crate owes it
//! is a marked boundary: every function whose running time must not
//! depend on a secret says so in its own documentation, and the ones
//! here are in [`ct`].

pub mod ct;
pub mod hash;
pub mod hkdf;
pub mod hmac;
pub mod secret;

pub use hash::{Hash, Sha256};
pub use hkdf::Kdf;
pub use hmac::Mac;
pub use secret::Secret;
