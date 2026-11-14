//! `hg-url`'s own tests.
//!
//! WPT's `urltestdata.json` is the oracle and it is run by
//! `hg-wpt-url`. These are the things it does not reach: the pieces of
//! the standard that have no entry point through the parser, the
//! invariants a passing suite would not notice were broken, and the
//! cases the standard states in prose and gives no test for.

mod ext;
mod host;
mod parse;
mod percent;
