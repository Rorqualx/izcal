//! `hg-unicode`'s tests: cases taken from UAX #15's prose.
//!
//! TX-1's conformance run against the pinned `NormalizationTest.txt` is
//! `hg-unicode-build`'s, because reading a pinned data file is an
//! ambient effect and this crate is tier P. Every oracle in the tree
//! sits in a tier T crate for the same reason (`hg-test262`, `hg-wast`,
//! `hg-js-oracle`).

#[cfg(test)]
mod units;
