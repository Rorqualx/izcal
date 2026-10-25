//! `hg-substrate-sim` integration tests (coding standard CS-TE-2: one
//! binary per crate; medium tests run under the simulator).

#[cfg(test)]
mod sim;

#[cfg(test)]
mod ipc;

#[cfg(test)]
mod os;
