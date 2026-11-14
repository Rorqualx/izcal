//! `hg-crypto`'s tests: the published vectors and the constant-time
//! helpers.
//!
//! The `pins.toml` check is in `hg-crypto-check`, because G3 forbids a
//! tier P crate ambient file access and CS-AS-1 binds that to its
//! tests as well.

#[cfg(test)]
mod vectors;
