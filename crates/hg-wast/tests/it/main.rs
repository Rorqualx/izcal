//! `hg-wast` integration tests (CS-TE-2: one binary per crate): the
//! reader over whole script files on disk (`wast/`, written for these
//! tests in the suite's shape, none copied from the WebAssembly
//! specification's test suite, which is fetched at a pin and never
//! committed).

#[cfg(test)]
mod run;
