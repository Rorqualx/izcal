//! The libc-free core of the Unix IPC transport (M1 phase D3).
//!
//! Everything here is safe Rust over plain values. The syscall layer
//! (`ipc::linux::sys`, the only `unsafe` in the transport)
//! translates the kernel's structures into these types and back, so the
//! decisions that keep a hostile peer out are made in code that compiles
//! and is tested on every OS, and that Kani proves (security scope
//! SEC-4 (b), VER-1). The record codec and the flow counters, which
//! every backend shares, are in [`super::common`].
//!
//! - [`cmsg`]: our own parser of the ancillary data `recvmsg(2)` returns.
//! - [`accept`]: which received descriptors become handles (IPC-R1).
//! - [`decide`]: the decision on one received record.
//!
//! Sources: the Linux man pages unix(7), cmsg(3), socket(7) and
//! recvmsg(2) only.
//!
//! These modules follow the tier P rules (coding standard CS-ER-3,
//! CS-TY-12) although the crate is tier O: they parse what a peer sends.
//! The lints below enforce that here until the generated tier table does.

// Only the Linux transport (`ipc::linux`) calls this code outside the
// tests and the Kani harnesses; macOS and Windows have their own cores
// (`ipc::mach`, `ipc::win`).
#![cfg_attr(
    not(any(test, kani, target_os = "linux")),
    expect(dead_code, reason = "only the Linux transport uses it")
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
pub(crate) mod decide;
