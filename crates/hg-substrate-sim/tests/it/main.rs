//! `hg-substrate-sim` integration tests (coding standard CS-TE-2: one
//! binary per crate; medium tests run under the simulator).

#![forbid(unsafe_code)] // G5, as in the crate root

#[cfg(test)]
mod sim;

#[cfg(test)]
mod ipc;

#[cfg(test)]
mod region;

#[cfg(test)]
mod os;

#[cfg(test)]
mod install;

#[cfg(test)]
mod launch;

#[cfg(test)]
mod faults;
