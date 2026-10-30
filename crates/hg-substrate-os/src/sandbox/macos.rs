//! The macOS sandbox backend: custom deny-default SBPL, applied at
//! start through `sandbox_init_with_parameters` (security scope 6.7).
//! The whole module is the crate's inventoried `unsafe` (CS-TY-13,
//! `unsafe-inventory.toml`, entry `sandbox::macos`): the FFI to the
//! sandbox API and nothing else.
#![expect(unsafe_code, reason = "G5: sandbox::macos (sandbox API)")]
//!
//! The built-in named profiles (`kSBXProfile*`) are **never** used:
//! Izcal's own measurement (security scope 6.7, `sec-d6`) shows macOS 27
//! kills processes that opt into them inside `sandbox_init` once the
//! binary declares SDK 27 or later, while custom profiles apply and
//! enforce whatever SDK the binary declares. Izcal writes its SBPL from
//! Apple's manual pages and its own black-box tests, never from another
//! browser's profile files (M-1).
//!
//! # Safety
//!
//! This module is the crate's inventoried `unsafe` (CS-TY-13,
//! `unsafe-inventory.toml`): the FFI to the sandbox API and nothing
//! else. Every unsafe block states its precondition and why it holds.

use super::Verdict;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

/// The FP-M0 empty-process profile, first-principles form.
///
/// An empty process on Izcal's substrate — two threads, one pre-opened
/// IPC endpoint, nothing else — needs, from the OS: its own memory
/// (system calls, not sandbox-gated), thread creation (`bsdthread_create`,
/// not sandbox-gated), its own process information, and read-only sysctl
/// (the tier probe reads it). It needs **no** Mach lookups — the broker
/// does not exist in the FP-M0 probe yet, and the spec's renderer row
/// allows no lookups except the broker's — no WindowServer, no IOKit
/// user clients, no files, no network. So the profile is exactly:
///
/// ```text
/// (version 1)
/// (deny default)
/// (allow sysctl-read)
/// ```
///
/// The tier probe below is the black-box test of that claim; if the
/// empty process needs a grant this profile lacks, the probe shows it
/// and the grant is added with a written reason, never silently.
pub const EMPTY_PROCESS_PROFILE: &str = "(version 1)\n(deny default)\n(allow sysctl-read)\n";

/// SBPL operation names and argument kinds used by the tier probe
/// (Apple's `sandbox_check(3)` manual page).
mod ops {
    use std::ffi::CStr;
    use std::os::raw::c_int;

    /// `sandbox_check`'s type argument: a path filter.
    pub const FILTER_PATH: c_int = 1;

    /// `sandbox_check`'s type argument: no argument follows.
    pub const FILTER_NONE: c_int = 0;

    pub const FILE_READ: &CStr = c"file-read";
    pub const MACH_LOOKUP: &CStr = c"mach-lookup";
    pub const NETWORK_OUTBOUND: &CStr = c"network-outbound";
    pub const SYSCTL_READ: &CStr = c"sysctl-read";
}

/// The sandbox API (deprecated `<sandbox.h>` surface; the spec's
/// measured path — custom profiles apply and enforce on this host).
///
/// `sandbox_check` is declared with a fixed three-pointer tail: the
/// caller passes only the arguments its operation reads and `ptr::null()`
/// for the rest. On Apple Silicon the trailing arguments land in fixed
/// registers, and `sandbox_check` reads only what the operation names —
/// this is the shape the spec's own `sec-d6` probes used.
///
/// # Safety
/// Raw FFI: pointers must be valid C strings for the duration of the
/// call; `sandbox_free_error` frees only buffers the API returned.
mod ffi {
    use super::{c_char, c_int, c_void};

    unsafe extern "C" {
        /// Applies an SBPL profile to this process. `errorbuf`, when
        /// set, is owned by the API and freed with `sandbox_free_error`.
        pub fn sandbox_init_with_parameters(
            profile: *const c_char,
            flags: u64,
            parameters: *const *const c_char,
            errorbuf: *mut *mut c_char,
        ) -> c_int;

        /// Frees an error buffer returned by `sandbox_init_with_parameters`.
        pub fn sandbox_free_error(errorbuf: *mut c_char);

        /// Queries whether an operation would be permitted.
        pub fn sandbox_check(
            pid: c_int,
            operation: *const c_char,
            kind: c_int,
            arg1: *const c_void,
            arg2: *const c_void,
            arg3: *const c_void,
        ) -> c_int;
    }
}

/// Applies the empty-process profile and returns the API's error string
/// on failure. The process opens nothing afterwards (security 6.7:
/// no path opens after lockdown).
pub fn apply_empty_process_profile() -> Result<(), String> {
    let profile = CString::new(EMPTY_PROCESS_PROFILE).expect("profile has no NUL");
    let mut errorbuf: *mut c_char = ptr::null_mut();
    // SAFETY: valid C string, null parameters, valid out-pointer freed below.
    let rc = unsafe {
        ffi::sandbox_init_with_parameters(profile.as_ptr(), 0, ptr::null(), &mut errorbuf)
    };
    if rc == 0 {
        return Ok(());
    }
    let message = if errorbuf.is_null() {
        format!("sandbox_init_with_parameters returned {rc}")
    } else {
        // SAFETY: errorbuf was allocated by the API for this exact free.
        let text = unsafe { CStr::from_ptr(errorbuf) }
            .to_string_lossy()
            .into_owned();
        // SAFETY: errorbuf is an API-owned buffer; freeing it is the
        // documented way to release it.
        unsafe { ffi::sandbox_free_error(errorbuf) };
        text
    };
    Err(message)
}

/// Runs one `sandbox_check` probe. `arg` is the operation's path or
/// service name, or null when the operation takes none.
fn check(operation: &'static CStr, kind: c_int, arg: Option<&CStr>) -> Verdict {
    let arg = arg.map_or(ptr::null(), CStr::as_ptr).cast::<c_void>();
    // SAFETY: valid C strings for the call's duration; the trailing
    // arguments are null because no probed operation reads more than one.
    let rc =
        unsafe { ffi::sandbox_check(-1, operation.as_ptr(), kind, arg, ptr::null(), ptr::null()) };
    match rc {
        0 => Verdict::Allowed,
        1 => Verdict::Denied,
        _ => Verdict::ProbeFailed,
    }
}

/// The tier probe: the four operations that define the empty-process
/// tier, in wire order. A result of `Denied` on the first three and
/// `Allowed` on sysctl-read is the profile working as written; anything
/// else is a reduced tier, recorded, never silent (security 6.7).
fn probe_tier() -> [Verdict; 4] {
    let hosts = CString::new("/etc/hosts").expect("no NUL");
    let windowserver = CString::new("com.apple.WindowServer").expect("no NUL");
    [
        check(ops::FILE_READ, ops::FILTER_PATH, Some(&hosts)),
        check(ops::MACH_LOOKUP, ops::FILTER_PATH, Some(&windowserver)),
        check(ops::NETWORK_OUTBOUND, ops::FILTER_NONE, None),
        check(ops::SYSCTL_READ, ops::FILTER_NONE, None),
    ]
}

/// Whether the profile's floor holds, by the same probes: a file read,
/// a service lookup and an outbound connection are all denied.
#[must_use]
pub fn floor_holds() -> bool {
    matches!(
        probe_tier(),
        [Verdict::Denied, Verdict::Denied, Verdict::Denied, _]
    )
}

/// The ready-line segment for macOS: the sandbox_check verdicts plus
/// the behavioral sysctl probe. `sandbox_check`'s "sysctl-read" name
/// returns a false deny while the profile's `(allow sysctl-read)` rule
/// demonstrably works, so the line reports the real read — what the
/// rule governs — alongside the check's verdict (recorded in the FP-M0
/// decision log, not treated as a tier reduction).
pub fn probe_line() -> String {
    let tier = probe_tier();
    let mut memsize: u64 = 0;
    let mut len: libc::size_t = std::mem::size_of::<u64>();
    // SAFETY: the name is a static C string; `memsize` and `len` are
    // valid out-pointers for the call's duration.
    let sysctl_ok = unsafe {
        libc::sysctlbyname(
            c"hw.memsize".as_ptr(),
            (&mut memsize as *mut u64).cast::<libc::c_void>(),
            &mut len,
            std::ptr::null_mut(),
            0,
        ) == 0
    };
    format!(
        "file-read={} mach-lookup={} network-outbound={} sysctl-check={} sysctl-real={}",
        tier[0].as_char(),
        tier[1].as_char(),
        tier[2].as_char(),
        tier[3].as_char(),
        if sysctl_ok {
            format!("ok:{memsize}")
        } else {
            "failed".to_string()
        },
    )
}
