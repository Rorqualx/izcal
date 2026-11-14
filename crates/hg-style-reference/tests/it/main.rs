//! `hg-style-reference`'s own tests.
//!
//! An oracle's tests are not a formality: a reference engine that is
//! quietly wrong makes every differential green and hides the bug it
//! was built to find. So each test here names the sentence of the
//! specification it checks.

mod cascade;
mod selector;
mod token;
