//! `hg-js-ops`: the JS operations hg-opgen generates from `ops/*.hgop`
//! (JS spec 6.3; design review R0-O1). Layer L2e, tier R.
//!
//! The generated files are checked in under `src/generated/`; the
//! hand-written frame and value layer they compile against is added with
//! the interpreter's tests, and the modules are declared then.

#![forbid(unsafe_code)]
