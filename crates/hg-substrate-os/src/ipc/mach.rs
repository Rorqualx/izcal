//! The libc-free core of the macOS IPC transport (M1 phase D4): Mach
//! messages as plain bytes and values.
//!
//! Everything here is safe Rust over plain values. The syscall layer
//! (`ipc::macos::sys`, the only `unsafe` in the transport) receives a
//! message into a buffer and asks the kernel about each right it
//! carries; the decisions that keep a hostile peer out are made here, in
//! code that compiles and is tested on every OS and that Kani proves
//! (security scope SEC-4 (b), VER-1). The record codec and the flow
//! counters, which every backend shares, are in [`super::common`].
//!
//! - [`wire`]: the message layout, the constants, and the parser of what
//!   `mach_msg` returns.
//! - [`accept`]: which received rights become handles (IPC-R1).
//! - [`decide`]: the decision on one received data message.
//! - [`notify`]: which messages are the kernel's own notifications.
//!
//! Sources: Apple's published Mach documentation (declarations and
//! field lists only) and the black-box probes in
//! `research_notes/M1 D4 macOS IPC/mach_facts.md`. No header, no
//! kernel source and no other project's code: every constant value and
//! every field order below was found by behaviour, and each cites the
//! probe that found it (`P<n>`).
//!
//! These modules follow the tier P rules (coding standard CS-ER-3,
//! CS-TY-12) although the crate is tier O: they parse what a peer sends.
//! The lints below enforce that here until the generated tier table does.

// Only the macOS transport (`ipc::macos`) uses all of it; elsewhere the
// tests and the Kani harnesses use part.
#![cfg_attr(
    not(target_os = "macos"),
    expect(dead_code, reason = "only the macOS transport uses all of it")
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
pub(crate) mod notify;
pub(crate) mod wire;
