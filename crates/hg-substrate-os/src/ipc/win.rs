//! The pure core of the Windows IPC transport (M1 phase D5).
//!
//! Everything here is safe Rust over plain values, so it compiles and is
//! tested on every OS and the Kani harnesses prove it (security scope
//! SEC-4 (b), VER-1). The syscall layer (`ipc::windows::sys`, the only
//! `unsafe` in the transport) asks the kernel about each handle and
//! hands the answers here.
//!
//! - [`frame`]: the ticket prefix on data messages, and the records of
//!   the control link between a process and its broker.
//! - [`accept`]: which handles the broker takes into a relay (IPC-R1).
//! - [`ledger`]: the broker's record of who holds each channel end, the
//!   tickets in flight and the bindings.
//! - [`decide`]: a receiver's decision on one pipe message.
//!
//! Sources: Microsoft Learn's reference pages for named pipes,
//! `DuplicateHandle`, `GetNamedPipeInfo`, `GetFileInformationByHandleEx`
//! and `NtQueryObject`, and the D5 probes
//! (`research_notes/M1 D5 Windows IPC/windows_facts.md`).
//!
//! These modules follow the tier P rules (coding standard CS-ER-3,
//! CS-TY-12) although the crate is tier O: they parse what a peer sends.

// Only the tests and the Kani harnesses call this code until the
// Windows transport lands on top of it.
#![cfg_attr(
    not(any(test, kani)),
    expect(dead_code, reason = "the Windows transport is not wired in yet")
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
pub(crate) mod decide;
pub(crate) mod frame;
pub(crate) mod ledger;
