//! `hg-encoding`'s tests.
//!
//! The decoders and encoders are written from the standard's prose, so
//! the tests are about the things prose alone does not settle: that
//! every byte sequence the indexes define round-trips, that the
//! masking defence of Section 2 actually holds, that streaming gives
//! the same answer as one call, and that the cases the standard writes
//! out in its own notes come out as written.

mod cases;
mod labels;
mod roundtrip;
mod streaming;
