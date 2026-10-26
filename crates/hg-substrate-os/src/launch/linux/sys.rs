//! The Linux system calls behind the launch path, as a small safe
//! interface: `clone3` with namespaces and a pidfd, the child's steps
//! before exec (descriptors, `setsid`, Landlock, the launch seccomp
//! filter, `execveat` of a pre-opened descriptor), pidfds, signals and
//! the reaper. Written from clone(2), close_range(2), execveat(2),
//! landlock(7), landlock_create_ruleset(2), landlock_add_rule(2),
//! landlock_restrict_self(2), seccomp(2), pidfd_open(2),
//! pidfd_send_signal(2), waitid(2), signalfd(2), prctl(2), poll(2),
//! setsid(2), mount(2), user_namespaces(7) and proc(5); no other
//! project's code is a source.
//!
//! This is the launch path's only `unsafe` (CS-TY-13,
//! `unsafe-inventory.toml`, entry `launch::linux::sys`). Descriptors cross
//! between broker and spawner only through `ipc::linux::sys` (SEC-4 (d)).
//!
//! **Child code after `clone3`.** A `clone3` without `CLONE_VM` copies
//! the caller like `fork`. In a process with other threads the copy may
//! hold locks those threads held, so the child side of every clone here
//! makes raw system calls only, on values prepared before the clone, and
//! ends in `execveat` or `_exit`. The one exception is
//! [`spawn_child`]'s filter builder, which allocates: the spawner, its
//! only caller, is single-threaded, so no lock can be held.
#![expect(
    unsafe_code,
    reason = "G5: launch::linux::sys (clone3, Landlock, execveat, pidfd FFI)"
)]

use std::ffi::CStr;
use std::io;
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};

use crate::sandbox::seccomp::Insn;

fn last_errno() -> i32 {
    io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

fn check(rc: libc::c_long) -> io::Result<libc::c_long> {
    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(rc)
    }
}

/// Takes ownership of a raw descriptor returned by a system call.
fn own(rc: libc::c_long) -> io::Result<OwnedFd> {
    let fd = RawFd::try_from(check(rc)?).map_err(|_e| io::Error::other("descriptor range"))?;
    // SAFETY: `fd` was just returned by the kernel to this process and
    // nothing else owns it.
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

// ---- clone(2) -------------------------------------------------------

/// `CLONE_PIDFD`: the kernel writes a pidfd for the child (clone(2)).
pub(crate) const CLONE_PIDFD: u64 = 0x1000;
/// New user namespace.
pub(crate) const CLONE_NEWUSER: u64 = 0x1000_0000;
/// New PID namespace.
pub(crate) const CLONE_NEWPID: u64 = 0x2000_0000;
/// New network namespace.
pub(crate) const CLONE_NEWNET: u64 = 0x4000_0000;
/// New mount namespace.
pub(crate) const CLONE_NEWNS: u64 = 0x0002_0000;

/// `struct clone_args` up to `tls` (`CLONE_ARGS_SIZE_VER0`, 64 bytes).
#[repr(C)]
#[derive(Default)]
struct CloneArgs {
    flags: u64,
    pidfd: u64,
    child_tid: u64,
    parent_tid: u64,
    exit_signal: u64,
    stack: u64,
    stack_size: u64,
    tls: u64,
}

/// What `clone3` returned in the caller.
enum Cloned {
    /// This is the child.
    Child,
    /// This is the caller; the child's outer pid and pidfd.
    Parent(u32, OwnedFd),
}

/// `clone3(flags | CLONE_PIDFD)`, `SIGCHLD` on exit, no new stack.
fn clone3(flags: u64) -> io::Result<Cloned> {
    let mut pidfd: libc::c_int = -1;
    let args = CloneArgs {
        flags: flags | CLONE_PIDFD,
        pidfd: (&raw mut pidfd) as u64,
        exit_signal: u64::from(libc::SIGCHLD.unsigned_abs()),
        ..CloneArgs::default()
    };
    // SAFETY: `args` is a live `clone_args` of the size passed, and its
    // `pidfd` points at a live int. With no CLONE_VM and no stack the
    // child runs on a copy of this address space, like fork.
    let rc = unsafe {
        libc::syscall(
            libc::SYS_clone3,
            &raw const args,
            core::mem::size_of::<CloneArgs>(),
        )
    };
    let rc = check(rc)?;
    if rc == 0 {
        return Ok(Cloned::Child);
    }
    let pid = u32::try_from(rc).map_err(|_e| io::Error::other("pid range"))?;
    // SAFETY: the kernel wrote a new pidfd for the child into `pidfd`
    // (CLONE_PIDFD); nothing else owns it.
    let fd = unsafe { OwnedFd::from_raw_fd(pidfd) };
    Ok(Cloned::Parent(pid, fd))
}

/// Ends a cloned child without running any of the caller's exit code.
fn child_exit(code: i32) -> ! {
    // SAFETY: `_exit` ends the process at once.
    unsafe { libc::_exit(code) }
}

// ---- descriptors ----------------------------------------------------

/// `CLOSE_RANGE_CLOEXEC` (close_range(2)).
const CLOSE_RANGE_CLOEXEC: libc::c_uint = 1 << 2;

/// Closes every descriptor from `first` up.
pub(crate) fn close_from(first: RawFd) {
    let first = libc::c_uint::try_from(first).unwrap_or(libc::c_uint::MAX);
    // SAFETY: close_range takes two numbers and a flag word. Callers own
    // nothing at or above `first` (the spawner at its start; see its use).
    unsafe { libc::syscall(libc::SYS_close_range, first, libc::c_uint::MAX, 0) };
}

/// Claims descriptor `n` this process was started with, if it is open.
/// The spawner's start only, once per number.
pub(crate) fn take_inherited(n: RawFd) -> Option<OwnedFd> {
    // SAFETY: F_GETFD takes no pointer; it fails with EBADF on a closed
    // number.
    if unsafe { libc::fcntl(n, libc::F_GETFD) } < 0 {
        return None;
    }
    // SAFETY: the broker placed `n` before exec (see `spawn_spawner`),
    // and the spawner's start claims each number once.
    Some(unsafe { OwnedFd::from_raw_fd(n) })
}

/// Marks a descriptor close-on-exec.
pub(crate) fn set_cloexec(fd: BorrowedFd<'_>) -> io::Result<()> {
    // SAFETY: F_SETFD takes an int flag word.
    check(unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) }.into())
        .map(|_r| ())
}

/// Whether `fd` names a regular file (an executable's `O_PATH` handle).
pub(crate) fn is_regular(fd: BorrowedFd<'_>) -> bool {
    // SAFETY: an all-zero stat is valid storage.
    let mut st: libc::stat = unsafe { core::mem::zeroed() };
    // SAFETY: fstat on a held descriptor (O_PATH allows it, open(2)), into
    // a live stat.
    let rc = unsafe { libc::fstat(fd.as_raw_fd(), &raw mut st) };
    rc == 0 && st.st_mode & libc::S_IFMT == libc::S_IFREG
}

/// `/dev/null`, read-write, close-on-exec: the child's stdio.
pub(crate) fn open_devnull() -> io::Result<OwnedFd> {
    // SAFETY: a static NUL-terminated path.
    own(unsafe { libc::open(c"/dev/null".as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) }.into())
}

/// A directory handle (`O_PATH | O_DIRECTORY`).
pub(crate) fn open_dir(path: &CStr) -> io::Result<OwnedFd> {
    // SAFETY: a NUL-terminated path.
    own(unsafe {
        libc::open(
            path.as_ptr(),
            libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC,
        )
    }
    .into())
}

/// An executable's handle in `dir` (`O_PATH`, no symlink followed): the
/// one handle it is ever exec'd from (G251, DU 13.1).
pub(crate) fn open_exe(dir: BorrowedFd<'_>, name: &CStr) -> io::Result<OwnedFd> {
    // SAFETY: a held directory descriptor and a NUL-terminated name.
    let fd = own(unsafe {
        libc::openat(
            dir.as_raw_fd(),
            name.as_ptr(),
            libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    }
    .into())?;
    if is_regular(fd.as_fd_ref()) {
        Ok(fd)
    } else {
        Err(io::Error::from_raw_os_error(libc::EACCES))
    }
}

trait AsFdRef {
    fn as_fd_ref(&self) -> BorrowedFd<'_>;
}
impl AsFdRef for OwnedFd {
    fn as_fd_ref(&self) -> BorrowedFd<'_> {
        std::os::fd::AsFd::as_fd(self)
    }
}

/// A close-on-exec pipe: (read end, write end).
pub(crate) fn pipe() -> io::Result<(OwnedFd, OwnedFd)> {
    let mut fds = [-1 as libc::c_int; 2];
    // SAFETY: a pointer to two live ints.
    check(unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) }.into())?;
    let [r, w] = fds;
    // SAFETY: pipe2 just returned this new descriptor.
    let r = unsafe { OwnedFd::from_raw_fd(r) };
    // SAFETY: as above.
    let w = unsafe { OwnedFd::from_raw_fd(w) };
    Ok((r, w))
}

/// Reads the child's failure record: `None` at end of file (the exec
/// succeeded and closed the write end), else `(stage, errno)`.
pub(crate) fn read_failure(fd: BorrowedFd<'_>) -> Option<(u8, i32)> {
    let mut buf = [0u8; 5];
    let mut got = 0usize;
    while got < buf.len() {
        let rest = buf.get_mut(got..).unwrap_or(&mut []);
        // SAFETY: reads into the live remainder of `buf`.
        let rc = unsafe { libc::read(fd.as_raw_fd(), rest.as_mut_ptr().cast(), rest.len()) };
        if rc < 0 && last_errno() == libc::EINTR {
            continue;
        }
        let Ok(n) = usize::try_from(rc) else {
            return Some((0, last_errno()));
        };
        if n == 0 {
            break;
        }
        got = got.saturating_add(n);
    }
    if got == 0 {
        return None;
    }
    let [s, e0, e1, e2, e3] = buf;
    Some((s, i32::from_le_bytes([e0, e1, e2, e3])))
}

// ---- Landlock (landlock(7)) -------------------------------------------

const LANDLOCK_CREATE_RULESET_VERSION: libc::c_uint = 1;
const LANDLOCK_RULE_PATH_BENEATH: libc::c_int = 1;
const FS_EXECUTE: u64 = 1 << 0;
const FS_READ_FILE: u64 = 1 << 2;
/// The filesystem rights of ABI 1 (bits 0 to 12), then REFER (2),
/// TRUNCATE (3) and IOCTL_DEV (5).
const FS_V1: u64 = (1 << 13) - 1;
const FS_REFER: u64 = 1 << 13;
const FS_TRUNCATE: u64 = 1 << 14;
const FS_IOCTL_DEV: u64 = 1 << 15;
/// TCP bind and connect (ABI 4).
const NET_TCP: u64 = (1 << 0) | (1 << 1);
/// Abstract Unix sockets and signals (ABI 6).
const SCOPE_ALL: u64 = (1 << 0) | (1 << 1);

#[repr(C)]
struct RulesetAttr {
    handled_access_fs: u64,
    handled_access_net: u64,
    scoped: u64,
}

#[repr(C, packed)]
struct PathBeneathAttr {
    allowed_access: u64,
    parent_fd: i32,
}

/// The Landlock ABI version, 0 when Landlock is missing or disabled.
pub(crate) fn landlock_abi() -> i32 {
    // SAFETY: the version query takes a null attribute and size 0.
    let rc = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            core::ptr::null::<RulesetAttr>(),
            0usize,
            LANDLOCK_CREATE_RULESET_VERSION,
        )
    };
    i32::try_from(rc).unwrap_or(0).max(0)
}

/// What a ruleset of ABI `abi` handles: every right the ABI has.
fn ruleset_for(abi: i32) -> (RulesetAttr, usize) {
    let mut fs = FS_V1;
    if abi >= 2 {
        fs |= FS_REFER;
    }
    if abi >= 3 {
        fs |= FS_TRUNCATE;
    }
    if abi >= 5 {
        fs |= FS_IOCTL_DEV;
    }
    let attr = RulesetAttr {
        handled_access_fs: fs,
        handled_access_net: if abi >= 4 { NET_TCP } else { 0 },
        scoped: if abi >= 6 { SCOPE_ALL } else { 0 },
    };
    // The kernel rejects a larger struct with nonzero fields it does not
    // know, so pass exactly what the ABI defines.
    let size = match abi {
        1..=3 => 8,
        4 | 5 => 16,
        _ => 24,
    };
    (attr, size)
}

/// Restricts this process with Landlock: every right of `abi` handled,
/// one rule allowing `EXECUTE | READ_FILE` beneath `exe` (launch facts
/// E6, E7). Needs `no_new_privs`. Returns the errno on failure.
fn landlock_exe_only(exe: RawFd, abi: i32) -> Result<(), i32> {
    let (attr, size) = ruleset_for(abi);
    // SAFETY: `attr` is live and at least `size` bytes.
    let rs = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            &raw const attr,
            size,
            0u32,
        )
    };
    if rs < 0 {
        return Err(last_errno());
    }
    let rule = PathBeneathAttr {
        allowed_access: FS_EXECUTE | FS_READ_FILE,
        parent_fd: exe,
    };
    // SAFETY: a ruleset descriptor and a live path-beneath attribute.
    let rc = unsafe {
        libc::syscall(
            libc::SYS_landlock_add_rule,
            rs,
            LANDLOCK_RULE_PATH_BENEATH,
            &raw const rule,
            0u32,
        )
    };
    if rc < 0 {
        return Err(last_errno());
    }
    // SAFETY: a ruleset descriptor and flags 0.
    let rc = unsafe { libc::syscall(libc::SYS_landlock_restrict_self, rs, 0u32) };
    if rc < 0 {
        return Err(last_errno());
    }
    Ok(())
}

// ---- seccomp ----------------------------------------------------------

#[repr(C)]
struct SockFprog {
    len: u16,
    filter: *const Insn,
}

/// Installs `prog` on every thread (`SECCOMP_FILTER_FLAG_TSYNC`). Needs
/// `no_new_privs`.
fn install_filter(prog: &[Insn]) -> Result<(), i32> {
    let len = u16::try_from(prog.len()).map_err(|_e| libc::E2BIG)?;
    let fprog = SockFprog {
        len,
        filter: prog.as_ptr(),
    };
    // SAFETY: `fprog` points at `prog`, live across the call; the kernel
    // copies the program. Mode 1 is SECCOMP_SET_MODE_FILTER, flag 1 TSYNC.
    let rc = unsafe { libc::syscall(libc::SYS_seccomp, 1u32, 1u32, &raw const fprog) };
    if rc == 0 { Ok(()) } else { Err(last_errno()) }
}

// ---- the spawner's child ----------------------------------------------

/// Where a child stopped; the codes are `hg_launch_plan::Stage`'s order
/// (`Descriptors` 3, `Landlock` 4, `Seccomp` 5, `Exec` 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChildStage {
    /// Placing descriptors, `setsid`, `no_new_privs`.
    Descriptors,
    /// Landlock.
    Landlock,
    /// The launch filter.
    Seccomp,
    /// `execveat`.
    Exec,
}

/// A spawn, prepared before the clone.
pub(crate) struct ChildPlan<'a> {
    /// Extra clone flags (namespaces).
    pub(crate) flags: u64,
    /// The bootstrap ends, placed at 3 and 4.
    pub(crate) up: BorrowedFd<'a>,
    /// See `up`.
    pub(crate) down: BorrowedFd<'a>,
    /// Placed at 0, 1 and 2.
    pub(crate) devnull: BorrowedFd<'a>,
    /// The executable's `O_PATH` handle.
    pub(crate) exe: BorrowedFd<'a>,
    /// argv[0].
    pub(crate) name: &'a CStr,
    /// The failure pipe's write end.
    pub(crate) err: BorrowedFd<'a>,
    /// The Landlock ABI.
    pub(crate) abi: i32,
    /// Builds the launch filter for the child's own pid as it sees it.
    pub(crate) filter: fn(u32) -> Option<Vec<Insn>>,
}

/// Starts one sandboxed child (launch design E-R3): returns its outer
/// pid and pidfd. The child writes `(stage, errno)` to the failure pipe
/// and exits 127 if any step before exec fails.
///
/// # Errors
/// `clone3`'s error.
pub(crate) fn spawn_child(plan: &ChildPlan<'_>) -> io::Result<(u32, OwnedFd)> {
    let argv: [*const libc::c_char; 2] = [plan.name.as_ptr(), core::ptr::null()];
    let envp: [*const libc::c_char; 1] = [core::ptr::null()];
    match clone3(plan.flags)? {
        Cloned::Parent(pid, fd) => Ok((pid, fd)),
        Cloned::Child => {
            let (stage, errno) = child_steps(plan, &argv, &envp);
            let [e0, e1, e2, e3] = errno.to_le_bytes();
            let code = match stage {
                ChildStage::Descriptors => 3,
                ChildStage::Landlock => 4,
                ChildStage::Seccomp => 5,
                ChildStage::Exec => 6,
            };
            let rec = [code, e0, e1, e2, e3];
            // SAFETY: writes the live record; the write end is open.
            unsafe { libc::write(plan.err.as_raw_fd(), rec.as_ptr().cast(), rec.len()) };
            child_exit(127)
        }
    }
}

/// `fcntl(fd, F_DUPFD_CLOEXEC, 5)`: a copy above the slots being filled.
fn dup_high(fd: RawFd) -> Result<RawFd, i32> {
    // SAFETY: F_DUPFD_CLOEXEC takes an int.
    let rc = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 5) };
    if rc < 0 { Err(last_errno()) } else { Ok(rc) }
}

/// `dup3(from, to, 0)`: `to` stays open across exec.
fn place(from: RawFd, to: RawFd) -> Result<(), i32> {
    // SAFETY: dup3 takes two numbers and a flag word.
    if unsafe { libc::dup3(from, to, 0) } < 0 {
        Err(last_errno())
    } else {
        Ok(())
    }
}

/// The child's steps; returns only on failure.
fn child_steps(
    plan: &ChildPlan<'_>,
    argv: &[*const libc::c_char; 2],
    envp: &[*const libc::c_char; 1],
) -> (ChildStage, i32) {
    let fail = |s: ChildStage| move |e: i32| (s, e);
    // (1) Descriptors: copies above 4 first, so placing 0..4 clobbers
    // nothing still needed, then everything from 5 up close-on-exec
    // (launch facts E9). The failure pipe and the executable are among
    // them: both close at exec.
    let r = (|| {
        // Every signal unblocked: the spawner blocks SIGCHLD for its
        // signalfd, and a child inherits no mask it did not ask for.
        // SAFETY: an all-zero sigset is valid storage for sigemptyset.
        let mut empty: libc::sigset_t = unsafe { core::mem::zeroed() };
        // SAFETY: initialises the live set.
        unsafe { libc::sigemptyset(&raw mut empty) };
        // SAFETY: a live set; the old mask is not wanted.
        if unsafe { libc::sigprocmask(libc::SIG_SETMASK, &raw const empty, core::ptr::null_mut()) }
            != 0
        {
            return Err(last_errno());
        }
        // The executable's handle may sit at 3 or 4 in the spawner, so it
        // moves above them as well.
        let exe = dup_high(plan.exe.as_raw_fd())?;
        let up = dup_high(plan.up.as_raw_fd())?;
        let down = dup_high(plan.down.as_raw_fd())?;
        let null = dup_high(plan.devnull.as_raw_fd())?;
        place(null, 0)?;
        place(null, 1)?;
        place(null, 2)?;
        place(up, 3)?;
        place(down, 4)?;
        // SAFETY: close_range takes two numbers and a flag word.
        let rc = unsafe {
            libc::syscall(
                libc::SYS_close_range,
                5u32,
                libc::c_uint::MAX,
                CLOSE_RANGE_CLOEXEC,
            )
        };
        if rc < 0 {
            return Err(last_errno());
        }
        // A session of its own: no controlling terminal (review M1).
        // SAFETY: setsid takes nothing.
        if unsafe { libc::setsid() } < 0 {
            return Err(last_errno());
        }
        // SAFETY: prctl with PR_SET_NO_NEW_PRIVS takes no pointers.
        if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0 {
            return Err(last_errno());
        }
        Ok(exe)
    })();
    let exe = match r {
        Ok(x) => x,
        Err(e) => return fail(ChildStage::Descriptors)(e),
    };
    // (2) Landlock: the executable only.
    if let Err(e) = landlock_exe_only(exe, plan.abi) {
        return fail(ChildStage::Landlock)(e);
    }
    // (3) The launch filter: the kind's steady list plus execveat
    // (review B1), for this process's own pid as it sees it.
    let own = std::process::id();
    let Some(prog) = (plan.filter)(own) else {
        return (ChildStage::Seccomp, libc::EINVAL);
    };
    if let Err(e) = install_filter(&prog) {
        return fail(ChildStage::Seccomp)(e);
    }
    // (4) Exec from the handle, no environment.
    // SAFETY: a held O_PATH descriptor, an empty path with AT_EMPTY_PATH,
    // and null-terminated argv and envp arrays of live C strings.
    unsafe {
        libc::syscall(
            libc::SYS_execveat,
            exe,
            c"".as_ptr(),
            argv.as_ptr(),
            envp.as_ptr(),
            libc::AT_EMPTY_PATH,
        )
    };
    (ChildStage::Exec, last_errno())
}

// ---- the spawner's tier test --------------------------------------------

/// Whether a child can be made with `flags`, and if so whether it can
/// make its mount tree private (a mount needs `CAP_SYS_ADMIN` in the new
/// user namespace, which the restricted profile denies, launch facts
/// E1, E2). `None` when the clone itself fails.
pub(crate) fn test_clone(flags: u64) -> Option<bool> {
    let Ok(c) = clone3(flags) else {
        return None;
    };
    match c {
        Cloned::Child => {
            let mut ok = true;
            if flags & CLONE_NEWNS != 0 {
                // SAFETY: mount with a static path and null strings: only
                // changes this child's (new) mount namespace.
                let rc = unsafe {
                    libc::mount(
                        core::ptr::null(),
                        c"/".as_ptr(),
                        core::ptr::null(),
                        libc::MS_REC | libc::MS_PRIVATE,
                        core::ptr::null(),
                    )
                };
                ok = rc == 0;
            }
            child_exit(if ok { 0 } else { 1 })
        }
        Cloned::Parent(_pid, fd) => Some(wait_pidfd(fd.as_fd_ref()) == Some(0)),
    }
}

// ---- reaping and signals ------------------------------------------------

/// `P_PIDFD` (waitid(2)).
const P_PIDFD: libc::idtype_t = 3;

/// Waits for (and reaps) this process's child `pidfd`; its exit code,
/// `None` if killed by a signal or not this process's child.
pub(crate) fn wait_pidfd(pidfd: BorrowedFd<'_>) -> Option<i32> {
    wait_pidfd_flags(pidfd, libc::WEXITED)
}

fn wait_pidfd_flags(pidfd: BorrowedFd<'_>, flags: libc::c_int) -> Option<i32> {
    let id = libc::id_t::try_from(pidfd.as_raw_fd()).ok()?;
    loop {
        // SAFETY: an all-zero siginfo is valid storage.
        let mut info: libc::siginfo_t = unsafe { core::mem::zeroed() };
        // SAFETY: a live siginfo; P_PIDFD with a held pidfd.
        let rc = unsafe { libc::waitid(P_PIDFD, id, &raw mut info, flags) };
        if rc < 0 {
            if last_errno() == libc::EINTR {
                continue;
            }
            return None;
        }
        // SAFETY: waitid filled `info` for a child state change (or left
        // it zero under WNOHANG, where si_pid reads 0).
        let pid = unsafe { info.si_pid() };
        // SAFETY: as above.
        let status = unsafe { info.si_status() };
        let code = info.si_code;
        if pid == 0 {
            return None;
        }
        return (code == libc::CLD_EXITED).then_some(status);
    }
}

/// Reaps every child that has ended, never blocking (the spawner's
/// `SIGCHLD` handler; E-R2).
pub(crate) fn reap_all() {
    loop {
        // SAFETY: an all-zero siginfo is valid storage.
        let mut info: libc::siginfo_t = unsafe { core::mem::zeroed() };
        // SAFETY: a live siginfo; P_ALL ignores the id.
        let rc =
            unsafe { libc::waitid(libc::P_ALL, 0, &raw mut info, libc::WEXITED | libc::WNOHANG) };
        // SAFETY: as in wait_pidfd_flags.
        if rc < 0 || unsafe { info.si_pid() } == 0 {
            return;
        }
    }
}

/// Blocks `SIGCHLD` and returns a signalfd for it, so the spawner's one
/// poll loop sees children end (no idle wakeups). Every other signal is
/// unblocked and at its default.
pub(crate) fn sigchld_fd() -> io::Result<OwnedFd> {
    // SAFETY: an all-zero sigset is valid storage for sigemptyset.
    let mut set: libc::sigset_t = unsafe { core::mem::zeroed() };
    // SAFETY: initialises and fills the live set.
    unsafe {
        libc::sigemptyset(&raw mut set);
    }
    // SAFETY: as above.
    unsafe {
        libc::sigaddset(&raw mut set, libc::SIGCHLD);
    }
    // SAFETY: a live set; the old mask is not wanted.
    unsafe { libc::sigprocmask(libc::SIG_SETMASK, &raw const set, core::ptr::null_mut()) };
    // SAFETY: a live set; new flags.
    own(
        unsafe { libc::signalfd(-1, &raw const set, libc::SFD_CLOEXEC | libc::SFD_NONBLOCK) }
            .into(),
    )
}

/// Empties a nonblocking signalfd.
pub(crate) fn drain(fd: BorrowedFd<'_>) {
    let mut buf = [0u8; 128 * 4];
    // SAFETY: reads into a live buffer until EAGAIN.
    while unsafe { libc::read(fd.as_raw_fd(), buf.as_mut_ptr().cast(), buf.len()) } > 0 {}
}

/// Sends `sig` through a pidfd (pidfd_send_signal(2)): never another
/// process that reused the pid.
pub(crate) fn pidfd_signal(pidfd: BorrowedFd<'_>, sig: libc::c_int) -> io::Result<()> {
    // SAFETY: a held pidfd, a signal number, no siginfo, flags 0.
    check(unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            pidfd.as_raw_fd(),
            sig,
            core::ptr::null::<libc::siginfo_t>(),
            0u32,
        )
    })
    .map(|_r| ())
}

/// `poll(2)` on up to four descriptors for input; their `revents`.
pub(crate) fn poll_in(fds: &[BorrowedFd<'_>], timeout_ms: i32) -> io::Result<[i16; 4]> {
    let mut p = [libc::pollfd {
        fd: -1,
        events: 0,
        revents: 0,
    }; 4];
    for (slot, fd) in p.iter_mut().zip(fds) {
        slot.fd = fd.as_raw_fd();
        slot.events = libc::POLLIN;
    }
    let n = libc::nfds_t::try_from(fds.len().min(4)).unwrap_or(0);
    loop {
        // SAFETY: `p` is live and holds at least `n` entries.
        let rc = unsafe { libc::poll(p.as_mut_ptr(), n, timeout_ms) };
        if rc < 0 && last_errno() == libc::EINTR {
            continue;
        }
        check(rc.into())?;
        let [a, b, c, d] = p;
        return Ok([a.revents, b.revents, c.revents, d.revents]);
    }
}

/// Waits up to `timeout_ms` for `fd` to take output; `true` if it can.
pub(crate) fn poll_out(fd: BorrowedFd<'_>, timeout_ms: i32) -> bool {
    let mut p = libc::pollfd {
        fd: fd.as_raw_fd(),
        events: libc::POLLOUT,
        revents: 0,
    };
    loop {
        // SAFETY: one live pollfd.
        let rc = unsafe { libc::poll(&raw mut p, 1, timeout_ms) };
        if rc < 0 && last_errno() == libc::EINTR {
            continue;
        }
        return rc > 0 && p.revents & libc::POLLOUT != 0;
    }
}

/// Makes this process the reaper of its orphaned descendants
/// (`PR_SET_CHILD_SUBREAPER`, review minor): a child whose spawner died
/// is re-parented here, not to init.
pub(crate) fn set_child_subreaper() -> io::Result<()> {
    // SAFETY: prctl with an int argument.
    check(unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) }.into()).map(|_r| ())
}

/// The `Pid:` and the last `NSpid:` value of a pidfd, from
/// `/proc/self/fdinfo/<fd>` (proc(5); launch facts E16).
pub(crate) fn pidfd_pids(pidfd: BorrowedFd<'_>) -> Option<(u32, u32)> {
    let text = std::fs::read_to_string(format!("/proc/self/fdinfo/{}", pidfd.as_raw_fd())).ok()?;
    let field = |name: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(name))
            .map(str::split_whitespace)
    };
    let pid = field("Pid:")?.next()?.parse().ok()?;
    let ns = field("NSpid:")?.last()?.parse().ok()?;
    Some((pid, ns))
}

// ---- the broker starts the spawner ----------------------------------------

/// Starts the spawner (no namespaces, launch design E-R2): `link` at 3,
/// each executable handle at 4 onwards, `devnull` at 0 and 1, this
/// process's stderr kept at 2, everything else closed at exec, an empty
/// environment, every signal unblocked. Exec'd from `spawner`, a handle
/// the broker opened at its start. Returns its pid and pidfd.
///
/// # Errors
/// `clone3`'s error; an exec failure shows as the spawner never sending
/// `Ready`.
pub(crate) fn spawn_spawner(
    spawner: BorrowedFd<'_>,
    name: &CStr,
    link: BorrowedFd<'_>,
    exes: &[BorrowedFd<'_>; 2],
    devnull: BorrowedFd<'_>,
) -> io::Result<(u32, OwnedFd)> {
    let argv: [*const libc::c_char; 2] = [name.as_ptr(), core::ptr::null()];
    let envp: [*const libc::c_char; 1] = [core::ptr::null()];
    let [r, n] = exes;
    let src = [
        spawner.as_raw_fd(),
        link.as_raw_fd(),
        r.as_raw_fd(),
        n.as_raw_fd(),
        devnull.as_raw_fd(),
    ];
    // SAFETY: an all-zero sigset is valid storage.
    let mut empty: libc::sigset_t = unsafe { core::mem::zeroed() };
    // SAFETY: initialises the live set.
    unsafe { libc::sigemptyset(&raw mut empty) };
    match clone3(0)? {
        Cloned::Parent(pid, fd) => Ok((pid, fd)),
        Cloned::Child => {
            // Raw calls only from here (see the module docs).
            let mut high = [-1; 5];
            for (h, s) in high.iter_mut().zip(src) {
                // SAFETY: F_DUPFD_CLOEXEC takes an int.
                *h = unsafe { libc::fcntl(s, libc::F_DUPFD_CLOEXEC, 6) };
                if *h < 0 {
                    child_exit(126);
                }
            }
            let [exe, l, rr, nn, null] = high;
            for (from, to) in [(null, 0), (null, 1), (l, 3), (rr, 4), (nn, 5)] {
                // SAFETY: dup3 takes two numbers and a flag word.
                if unsafe { libc::dup3(from, to, 0) } < 0 {
                    child_exit(126);
                }
            }
            // SAFETY: close_range takes two numbers and a flag word.
            unsafe {
                libc::syscall(
                    libc::SYS_close_range,
                    6u32,
                    libc::c_uint::MAX,
                    CLOSE_RANGE_CLOEXEC,
                )
            };
            // SAFETY: a live empty set.
            unsafe {
                libc::sigprocmask(libc::SIG_SETMASK, &raw const empty, core::ptr::null_mut())
            };
            // SAFETY: as in child_steps' execveat.
            unsafe {
                libc::syscall(
                    libc::SYS_execveat,
                    exe,
                    c"".as_ptr(),
                    argv.as_ptr(),
                    envp.as_ptr(),
                    libc::AT_EMPTY_PATH,
                )
            };
            child_exit(126)
        }
    }
}
