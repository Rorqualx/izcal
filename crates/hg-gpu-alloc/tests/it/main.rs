//! `hg-gpu-alloc`'s tests: the six decisions of 6.2 and 6.22's
//! allocator, each held to by something that can fail.

#![allow(clippy::arithmetic_side_effects, reason = "test arithmetic is literal")]

#[cfg(test)]
mod contract;
