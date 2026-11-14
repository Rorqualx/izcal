//! `hg-net-wire`: the budget, the named limits, and the checked kernels.
//!
//! Networking §6.2 (NW-D1) gives every protocol state machine the same
//! shape — bytes and timers in, a bounded list of actions out, no
//! `async`, no executor, no clock read — and two things that shape
//! belongs to every one of them:
//!
//! - a **[`Budget`]**, charged **at allocation time** for every byte or
//!   object a peer can make the machine hold or queue, whose failure is
//!   a typed error and never a panic;
//! - the **arithmetic kernels** that read peer-supplied numbers, each
//!   small enough that Kani can prove it cannot overflow or panic for
//!   *any* input (G33, tier P).
//!
//! Both are here rather than in each protocol crate because a budget
//! per crate is a budget per author, and because an overflow proof is
//! only worth having if there is one place to prove it about.
//!
//! # Why checked arithmetic rather than a release flag
//!
//! Overflow checks in release builds turn an overflow into an abort,
//! which is a denial of service a peer chooses the moment it notices.
//! So the rule is explicit checked arithmetic at every peer-supplied
//! value (§6.2), and the proofs are of the checks rather than of the
//! compiler's flags.
//!
//! # What is not here yet
//!
//! The crate's §6.21 row also gives it the header-field, Structured
//! Fields and MIME type parsers and MIME sniffing. NW-M0's row scopes
//! it to "budgets and Kani-proved kernels", and those parsers arrive
//! with the HTTP/1.1 client at NW-M1. Nothing here anticipates them.

#![cfg_attr(kani, allow(dead_code, reason = "proof-only builds"))]

pub mod ack;
pub mod budget;
pub mod flow;
pub mod limits;
pub mod packet_number;
pub mod varint;

#[cfg(kani)]
mod proofs;

pub use budget::{Budget, Limit, Overrun, charge_all};
