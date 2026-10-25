//! `hg-substrate-sim` integration tests (coding standard CS-TE-2: one
//! binary per crate; medium tests run under the simulator).

#![forbid(unsafe_code)] // G5, as in the crate root

#[cfg(test)]
mod sim;

#[cfg(test)]
mod ipc;
