//! Safe Rust bindings for the cppgc (Oilpan) APIs that rusty_v8 152.2.0 does
//! not expose.
//!
//! Two modes:
//! - default (`standalone`): a C++ shim supplies the APIs on top of the
//!   crates.io rusty_v8 (see `standalone.rs`).
//! - feature `rusty_v8_fork`: rusty_v8 is the HGBA fork, which carries the
//!   same APIs natively (`v8::cppgc::*`, compiled inside the V8 build). This
//!   crate is then a thin compatibility layer: it re-exports the fork's types,
//!   keeps its extension traits as forwarders, and its shim shrinks to the ABI
//!   checks (`abi`).
#![allow(clippy::missing_safety_doc)]

#[cfg(not(feature = "rusty_v8_fork"))]
mod standalone;
#[cfg(not(feature = "rusty_v8_fork"))]
pub use standalone::*;

#[cfg(feature = "rusty_v8_fork")]
mod fork;
#[cfg(feature = "rusty_v8_fork")]
pub use fork::*;
