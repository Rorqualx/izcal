//! The Linux sandbox backend: a seccomp-bpf allowlist with an
//! architecture check, written from the kernel's seccomp documentation
//! (security scope 6.7, the strictest tier: compute plus IPC — no
//! filesystem, no network). The profile is generated from the static
//! allowlist below, one `jeq` per syscall jumping to a shared ALLOW;
//! anything else gets EPERM (deny, observable — the tier probe reads
//! the errno). The black-box method of the spec applies:
//! the list starts from what an empty process proves it needs and every
//! later addition carries a written reason, never a guess.
//!
//! The whole module is the crate's inventoried `unsafe` (CS-TY-13,
//! `unsafe-inventory.toml`, entry `sandbox::linux`): the prctl FFI and
//! the raw BPF program bytes.
#![expect(unsafe_code, reason = "G5: sandbox::linux (seccomp BPF)")]

/// `AUDIT_ARCH_X86_64`, `AUDIT_ARCH_AARCH64` (the two lab arches).
const ARCH_X86_64: u32 = 0xC000_003E;
const ARCH_AARCH64: u32 = 0xC000_00B7;

/// x86_64 syscall numbers this filter allows (the `seccomp_data.nr`
/// field). Each entry has its reason; the empty process's needs only.
const ALLOWLIST_X86_64: &[(i64, &str)] = &[
    (0, "read — the pre-opened IPC endpoint"),
    (1, "write — IPC, stderr"),
    (3, "close"),
    (5, "fstat — std's fd queries"),
    (8, "lseek"),
    (9, "mmap — the allocator"),
    (10, "mprotect — the allocator"),
    (11, "munmap — the allocator"),
    (12, "brk — the allocator"),
    (13, "rt_sigaction — thread setup"),
    (14, "rt_sigprocmask — thread setup"),
    (15, "rt_sigreturn — signal return"),
    (16, "ioctl — std's fd queries"),
    (24, "sched_yield"),
    (28, "madvise — the allocator"),
    (35, "nanosleep"),
    (44, "sendto — the IPC ready byte (std uses MSG_NOSIGNAL)"),
    (39, "getpid"),
    (56, "clone — the sanctioned thread spawn"),
    (60, "exit"),
    (63, "uname — std's platform queries"),
    (97, "getrlimit — std's resource queries"),
    (131, "sigaltstack — signal handling"),
    (157, "prctl — PR_SET_NO_NEW_PRIVS and friends"),
    (158, "arch_prctl — thread TLS"),
    (186, "gettid"),
    (202, "futex — thread parking"),
    (204, "sched_getaffinity — std's CPU queries"),
    (218, "set_tid_address — thread teardown"),
    (228, "clock_gettime"),
    (230, "clock_nanosleep — std::thread::sleep"),
    (231, "exit_group"),
    (234, "tgkill — thread signalling"),
    (273, "set_robust_list — glibc thread teardown"),
    (262, "newfstatat — std's fd queries"),
    (302, "prlimit64 — std's resource queries"),
    (318, "getrandom — std's hash seeds"),
    (334, "rseq — glibc's restartable sequences"),
    (435, "clone3 — glibc's thread spawn"),
];

/// aarch64 syscall numbers for the same operations (the lab Mac's arch
/// when the probe runs there).
const ALLOWLIST_AARCH64: &[(i64, &str)] = &[
    (29, "ioctl"),
    (57, "close"),
    (62, "lseek"),
    (63, "read"),
    (64, "write"),
    (79, "newfstatat"),
    (80, "fstat"),
    (93, "exit"),
    (94, "exit_group"),
    (96, "set_tid_address"),
    (98, "futex"),
    (99, "set_robust_list — glibc thread teardown"),
    (101, "nanosleep"),
    (113, "clock_gettime"),
    (115, "clock_nanosleep"),
    (123, "sched_getaffinity"),
    (206, "sendto — the IPC ready byte (std uses MSG_NOSIGNAL)"),
    (124, "sched_yield"),
    (131, "tgkill"),
    (132, "sigaltstack"),
    (134, "rt_sigaction"),
    (135, "rt_sigprocmask"),
    (139, "rt_sigreturn"),
    (160, "uname"),
    (163, "getrlimit"),
    (167, "prctl"),
    (172, "getpid"),
    (178, "gettid"),
    (214, "brk"),
    (215, "munmap"),
    (220, "clone"),
    (222, "mmap"),
    (226, "mprotect"),
    (233, "madvise"),
    (261, "prlimit64"),
    (278, "getrandom"),
    (293, "rseq"),
    (435, "clone3"),
];

/// BPF instruction opcodes (kernel `linux/filter.h`).
const BPF_LD_W_ABS: u16 = 0x20;
const BPF_JEQ_K: u16 = 0x15;
const BPF_RET_K: u16 = 0x06;

/// seccomp return values (kernel `linux/seccomp.h`).
///
/// The deny action is EPERM, not KILL: a denied operation is observable
/// to the process (the tier probe reads the errno and reports it), and
/// every legitimate operation of the empty process is allowlisted, so a
/// denial means a bug or an attempt — either way it is recorded, never
/// silent (security 6.7).
const RET_DENY: u32 = 0x0005_0000 | libc::EPERM as u32; // SECCOMP_RET_ERRNO | EPERM
const RET_ALLOW: u32 = 0x7fff_0000;

/// One BPF instruction, matching the kernel's `sock_filter`.
#[repr(C)]
#[derive(Clone, Copy)]
struct SockFilter {
    code: u16,
    jt: u8,
    jf: u8,
    k: u32,
}

/// The program header, matching the kernel's `sock_fprog`.
#[repr(C)]
struct SockFprog {
    len: u16,
    filter: *const SockFilter,
}

/// Builds the allowlist filter for the running arch. Layout:
///
/// ```text
/// 0: ld  arch          (offset 4 in seccomp_data)
/// 1: jeq x86_64  jt=2 (to the nr load)      jf=0 (try aarch64)
/// 2: jeq aarch64 jt=1 (to the nr load)      jf=0 (kill)
/// 3: ret deny (EPERM)
/// 4: ld  nr
/// 5..5+n-1: jeq nr_i  jt=n-i (to the shared ALLOW)  jf=0
/// 5+n: ret deny (EPERM)
/// 6+n: ret allow   (shared target)
/// ```
///
/// A `jeq nr_i` at index `5+i` lands on the shared ALLOW at index
/// `6+n`, so its jt is `n-i` (jt counts instructions after this one).
fn build_filter() -> Vec<SockFilter> {
    let table: &[(i64, &str)] = if cfg!(target_arch = "x86_64") {
        ALLOWLIST_X86_64
    } else {
        ALLOWLIST_AARCH64
    };
    let n = table.len();

    // The arch check (load arch, accept the two lab arches, else deny),
    // then the syscall-number load.
    let mut f = vec![
        SockFilter {
            code: BPF_LD_W_ABS,
            jt: 0,
            jf: 0,
            k: 4,
        },
        SockFilter {
            code: BPF_JEQ_K,
            jt: 2,
            jf: 0,
            k: ARCH_X86_64,
        },
        SockFilter {
            code: BPF_JEQ_K,
            jt: 1,
            jf: 0,
            k: ARCH_AARCH64,
        },
        SockFilter {
            code: BPF_RET_K,
            jt: 0,
            jf: 0,
            k: RET_DENY,
        },
        SockFilter {
            code: BPF_LD_W_ABS,
            jt: 0,
            jf: 0,
            k: 0,
        },
    ];
    for (i, (nr, _reason)) in table.iter().enumerate() {
        f.push(SockFilter {
            code: BPF_JEQ_K,
            jt: (n - i) as u8,
            jf: 0,
            k: *nr as u32,
        });
    }
    f.push(SockFilter {
        code: BPF_RET_K,
        jt: 0,
        jf: 0,
        k: RET_DENY,
    });
    f.push(SockFilter {
        code: BPF_RET_K,
        jt: 0,
        jf: 0,
        k: RET_ALLOW,
    });
    f
}

/// Applies the empty-process filter: no_new_privs first (seccomp's
/// precondition), then the filter. Returns the prctl error on failure.
pub fn apply_empty_process_profile() -> Result<(), String> {
    // SAFETY: prctl with PR_SET_NO_NEW_PRIVS takes no pointers.
    let rc = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if rc != 0 {
        return Err(format!("PR_SET_NO_NEW_PRIVS: rc={rc}"));
    }
    let filter = build_filter();
    let prog = SockFprog {
        len: filter.len() as u16,
        filter: filter.as_ptr(),
    };
    // SAFETY: `prog.filter` points at `filter`, kept alive across the
    // call; the kernel copies the program.
    let rc = unsafe {
        libc::prctl(
            libc::PR_SET_SECCOMP,
            libc::SECCOMP_MODE_FILTER,
            &prog as *const SockFprog as libc::c_ulong,
            0,
            0,
        )
    };
    if rc != 0 {
        return Err(format!("PR_SET_SECCOMP: rc={rc}"));
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
