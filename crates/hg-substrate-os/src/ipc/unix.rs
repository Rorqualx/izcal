//! The libc-free core of the Unix IPC transport (M1 phase D3).
//!
//! Everything here is safe Rust over plain values. The syscall layer
//! (`sys`, a later D3 step and the only `unsafe` in the transport)
//! translates the kernel's structures into these types and back, so the
//! decisions that keep a hostile peer out are made in code that compiles
//! and is tested on every OS, and that Kani proves (security scope
//! SEC-4 (b), VER-1).
//!
//! - [`codec`]: the data and credit record formats (IPC-R6).
//! - [`cmsg`]: our own parser of the ancillary data `recvmsg(2)` returns.
//! - [`accept`]: which received descriptors become handles (IPC-R1).
//! - [`flow`]: the counters behind `Full` and the credits.
//! - [`decide`]: the decision on one received record.
//!
//! Sources: the Linux man pages unix(7), cmsg(3), socket(7) and
//! recvmsg(2) only.
//!
//! These modules follow the tier P rules (coding standard CS-ER-3,
//! CS-TY-12) although the crate is tier O: they parse what a peer sends.
//! The lints below enforce that here until the generated tier table does.

// The sys and io modules (later D3 steps) are the callers; until they
// land, only the tests and the Kani harnesses use these items.
#![cfg_attr(
    not(any(test, kani)),
    expect(dead_code, reason = "used by the sys and io modules of later D3 steps")
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

pub(crate) mod accept;
pub(crate) mod cmsg;
pub(crate) mod codec;
pub(crate) mod decide;
pub(crate) mod flow;
