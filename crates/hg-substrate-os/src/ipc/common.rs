//! The parts of the OS transports every backend shares: the record
//! formats and the flow counters. Pure code over plain values, so it
//! compiles and is tested on every OS and the Kani harnesses prove it
//! (security scope SEC-4 (b), VER-1).
//!
//! - [`codec`]: the data and credit record formats (IPC-R6).
//! - [`flow`]: the counters behind `Full` and the credits.
//!
//! These modules follow the tier P rules (coding standard CS-ER-3,
//! CS-TY-12) although the crate is tier O: they parse what a peer sends.
//! The lints below enforce that here until the generated tier table does.

// Outside the tests and the Kani harnesses, the Linux and Windows
// transports use all of this code; the macOS transport uses only part
// of it, so on macOS (and any OS without a transport) some is unused.
#![cfg_attr(
    not(any(test, kani, target_os = "linux", windows)),
    expect(dead_code, reason = "the macOS transport uses only part of it")
)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::todo,
    clippy::unimplemented,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::get_unwrap,
    clippy::arithmetic_side_effects,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::checked_conversions,
    clippy::wildcard_enum_match_arm,
    clippy::map_err_ignore
)]

pub(crate) mod codec;
pub(crate) mod flow;
