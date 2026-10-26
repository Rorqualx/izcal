//! The Linux sandbox backend: a seccomp-bpf allowlist with an
//! architecture check and argument checks, written from the kernel's
//! seccomp documentation (security scope 6.7, the strictest tier:
//! compute plus IPC — no filesystem, no network, no new processes or
//! namespaces). The program is built as plain data by
//! [`super::seccomp`], where the table and every entry's reason live and
//! the builder is tested; anything not allowed gets EPERM (deny,
//! observable — the tier probe reads the errno). The black-box method of
//! the spec applies: the list starts from what an empty process proves
//! it needs, and every later addition carries a written reason.
//!
//! The whole module is the crate's inventoried `unsafe` (CS-TY-13,
//! `unsafe-inventory.toml`, entry `sandbox::linux`): the prctl and
//! seccomp FFI.
#![expect(unsafe_code, reason = "G5: sandbox::linux (seccomp BPF)")]

use super::seccomp::{ARCH_AARCH64, ARCH_X86_64, Insn, program, steady_rules};

/// The program header, matching the kernel's `sock_fprog`.
#[repr(C)]
struct SockFprog {
    len: u16,
    filter: *const Insn,
}

/// `SECCOMP_SET_MODE_FILTER` and `SECCOMP_FILTER_FLAG_TSYNC` (seccomp(2)).
const SECCOMP_SET_MODE_FILTER: libc::c_ulong = 1;
const SECCOMP_FILTER_FLAG_TSYNC: libc::c_ulong = 1;

/// Applies the empty-process filter: no_new_privs first (seccomp's
/// precondition), then the filter on every thread. Returns the error on
/// failure.
pub fn apply_empty_process_profile() -> Result<(), String> {
    let arch = if cfg!(target_arch = "x86_64") {
        ARCH_X86_64
    } else {
        ARCH_AARCH64
    };
    let own = std::process::id();
    let rules = steady_rules(arch, own);
    let filter = program(arch, &rules).map_err(|e| format!("seccomp program: {e:?}"))?;
    let len = u16::try_from(filter.len()).map_err(|_e| "seccomp program too long".to_string())?;
    // SAFETY: prctl with PR_SET_NO_NEW_PRIVS takes no pointers.
    let rc = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if rc != 0 {
        return Err(format!("PR_SET_NO_NEW_PRIVS: rc={rc}"));
    }
    let prog = SockFprog {
        len,
        filter: filter.as_ptr(),
    };
    // TSYNC applies the filter to every thread of the process at once,
    // so substrate threads started before lockdown are filtered too; a
    // thread that cannot be synchronised makes the call fail (it returns
    // that thread's ID), never a partly filtered process (seccomp(2)).
    // SAFETY: `prog.filter` points at `filter`, kept alive across the
    // call; the kernel copies the program.
    let rc = unsafe {
        libc::syscall(
            libc::SYS_seccomp,
            SECCOMP_SET_MODE_FILTER,
            SECCOMP_FILTER_FLAG_TSYNC,
            &raw const prog,
        )
    };
    if rc != 0 {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        return Err(format!(
            "seccomp(SET_MODE_FILTER, TSYNC): rc={rc} errno={errno}"
        ));
    }
    Ok(())
}

/// The behavioral tier probe: attempts the two operations the tier must
/// deny and reports the errno of each. `EPERM` for both is the filter
/// working as written; anything else is a reduced tier, recorded, never
/// silent (security 6.7).
pub fn probe_line() -> String {
    // SAFETY: a static NUL-terminated path; openat is a plain syscall
    // wrapper; the seccomp filter decides the outcome.
    let open_rc = unsafe { libc::openat(libc::AT_FDCWD, c"/etc/hosts".as_ptr(), libc::O_RDONLY) };
    let open_errno = if open_rc < 0 {
        std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
    } else {
        0
    };
    // SAFETY: socket with constant arguments; the filter decides.
    let sock_rc = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    let sock_errno = if sock_rc < 0 {
        std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
    } else {
        0
    };
    format!("openat-errno={open_errno} socket-errno={sock_errno}")
}

/// Probes process creation under the filter, for the IPC transport's
/// credential guarantee: returns the errno of a fork-style `clone` and of
/// `unshare(CLONE_NEWUSER)`. `0` means the call was allowed (a forked
/// child exits at once), which a sandboxed process must never see.
pub fn probe_process_creation() -> (i32, i32) {
    // SAFETY: a fork-style clone (no shared memory, SIGCHLD on exit);
    // the child side only calls `_exit`.
    let rc = unsafe {
        libc::syscall(
            libc::SYS_clone,
            libc::c_ulong::from(libc::SIGCHLD.unsigned_abs()),
            0usize,
            0usize,
            0usize,
            0usize,
        )
    };
    if rc == 0 {
        // SAFETY: the child ends immediately.
        unsafe { libc::_exit(0) };
    }
    let fork_errno = if rc < 0 {
        std::io::Error::last_os_error().raw_os_error().unwrap_or(-1)
    } else {
        0
    };
    // SAFETY: unshare takes a flag word.
    let rc = unsafe { libc::syscall(libc::SYS_unshare, libc::CLONE_NEWUSER) };
    let userns_errno = if rc < 0 {
        std::io::Error::last_os_error().raw_os_error().unwrap_or(-1)
    } else {
        0
    };
    (fork_errno, userns_errno)
}
