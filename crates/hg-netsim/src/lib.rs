//! `hg-netsim`: links, peers and a resolver, with no network.
//!
//! §9.1 orders this crate before any protocol crate: "`hg-net-wire`
//! (budgets, checked kernels, the parsers), the sans-IO conventions,
//! `hg-netsim` and the first `hg-net-oracle` servers come before any
//! protocol crate", because "oracles before code (ORC-1); budgets and
//! kernels cannot be retrofitted".
//!
//! # What a simulated link is for
//!
//! A protocol written against a real socket is tested against the
//! network the test machine happens to have — low latency, almost no
//! loss, no reordering — and the bugs that matter are the ones that
//! need the opposite. Worse, the network is different on every run, so
//! a failure is not reproducible and a passing run proves nothing
//! about the next one.
//!
//! So a link here is a **function of its inputs and one seeded
//! stream**: the same seed gives the same deliveries, in the same
//! order, at the same nanoseconds, on every machine. A failure found
//! at seed 97 is found again at seed 97.
//!
//! # The shape
//!
//! A [`Link`] takes a packet at an instant and returns when it
//! arrives, or that it was dropped. It does not hold a queue of
//! callbacks or drive anything: the caller owns the clock, as
//! [`hg_substrate::Clock`] does for everything else, and the link only
//! answers questions. That is the same sans-IO discipline NW-D1 puts
//! on the protocol crates, for the same reason — the simulator, the
//! fuzzers and replay all run the same code.
//!
//! [`Resolver`] is the same idea for DNS: a zone and a set of
//! failures, answering a query with what a stub resolver would see.
//!
//! # What is not here
//!
//! §6.21 also gives the crate middleboxes and peers. NW-M0's row asks
//! for "`hg-netsim` with links and DNS"; the hostile servers are
//! `hg-net-oracle`'s, and the middleboxes arrive with the protocol
//! that needs them to be wrong about something.

pub mod dns;
pub mod link;

pub use dns::{Answer, Query, Record, Resolver};
pub use link::{Delivery, Link, LinkSpec};
