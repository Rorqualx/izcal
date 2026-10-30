//! The Linux system calls behind the IPC transport, as a small safe
//! interface. Written from socketpair(2), sendmsg(2), recvmsg(2),
//! unix(7), cmsg(3), socket(7), epoll(7), epoll_ctl(2), eventfd(2) and
//! getrlimit(2); no other project's code is a source.
//!
//! This is the only `unsafe` in the transport (CS-TY-13,
//! `unsafe-inventory.toml`, entry `ipc::linux::sys`), and the only code
//! that calls `sendmsg` and `recvmsg` (security scope SEC-4 (d)). It
//! moves bytes and file descriptors and reports raw kernel facts;
//! deciding what to accept is the pure code in `ipc::unix`.
//!
//! Every send and receive passes `MSG_DONTWAIT` explicitly (sends also
//! `MSG_NOSIGNAL`): a socket's `O_NONBLOCK` belongs to its shared open
//! file description, which a former holder of a transferred end can
//! still change, so the transport never relies on it.
#![expect(unsafe_code, reason = "G5: ipc::linux::sys (socket, epoll FFI)")]

use std::collections::BTreeMap;
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};

use core::ptr::NonNull;

use hg_substrate::ipc::Mapping;

use crate::ipc::shm::{OsMapping, Raw};
use crate::ipc::unix::accept::FdFacts;
use crate::ipc::unix::cmsg;

/// The control buffer a record is received into: room for 16 fds and one
/// set of credentials, 8-byte aligned as `cmsghdr` requires.
pub(crate) const CONTROL_LEN: usize = cmsg::CONTROL_LEN;

/// An 8-byte-aligned control buffer.
#[repr(C, align(8))]
struct ControlBuf([u8; CONTROL_LEN]);

impl ControlBuf {
    /// Zeroed.
    const fn new() -> Self {
        Self([0; CONTROL_LEN])
    }
}

fn last_errno() -> i32 {
    io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

/// Converts a syscall's `-1`-on-error return.
fn check(rc: libc::c_int) -> io::Result<libc::c_int> {
    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(rc)
    }
}

/// A new channel's two ends: `socketpair(AF_UNIX, SOCK_SEQPACKET |
/// SOCK_CLOEXEC | SOCK_NONBLOCK, 0)`.
pub(crate) fn socketpair() -> io::Result<(OwnedFd, OwnedFd)> {
    let mut sv: [libc::c_int; 2] = [-1, -1];
    // SAFETY: `sv` is a valid two-int array for the call to fill.
    check(unsafe {
        libc::socketpair(
            libc::AF_UNIX,
            libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
            0,
            sv.as_mut_ptr(),
        )
    })?;
    // SAFETY: on success both are new descriptors this process owns.
    let a = unsafe { OwnedFd::from_raw_fd(sv[0]) };
    // SAFETY: as above.
    let b = unsafe { OwnedFd::from_raw_fd(sv[1]) };
    Ok((a, b))
}

fn setsockopt_int(fd: BorrowedFd<'_>, name: libc::c_int, value: libc::c_int) -> io::Result<()> {
    // SAFETY: a valid fd and a pointer to a live int of the stated size.
    check(unsafe {
        libc::setsockopt(
            fd.as_raw_fd(),
            libc::SOL_SOCKET,
            name,
            (&raw const value).cast(),
            socklen_of::<libc::c_int>(),
        )
    })
    .map(|_| ())
}

fn getsockopt_int(fd: BorrowedFd<'_>, name: libc::c_int) -> io::Result<libc::c_int> {
    let mut value: libc::c_int = 0;
    let mut len = socklen_of::<libc::c_int>();
    // SAFETY: a valid fd and pointers to a live int and its length.
    check(unsafe {
        libc::getsockopt(
            fd.as_raw_fd(),
            libc::SOL_SOCKET,
            name,
            (&raw mut value).cast(),
            &raw mut len,
        )
    })?;
    Ok(value)
}

const fn socklen_of<T>() -> libc::socklen_t {
    // Sizes of small C structs; they fit a socklen_t.
    #[expect(clippy::cast_possible_truncation, reason = "small C struct sizes")]
    {
        core::mem::size_of::<T>() as libc::socklen_t
    }
}

/// Turns on `SO_PASSCRED`: every record received on this end carries the
/// sender's kernel-attested credentials (unix(7)).
pub(crate) fn set_passcred(fd: BorrowedFd<'_>) -> io::Result<()> {
    setsockopt_int(fd, libc::SO_PASSCRED, 1)
}

/// Requests a send buffer of `bytes` and returns what the kernel set (it
/// doubles the request and caps it at `wmem_max`, socket(7)).
pub(crate) fn set_sndbuf(fd: BorrowedFd<'_>, bytes: usize) -> io::Result<usize> {
    let want = libc::c_int::try_from(bytes).unwrap_or(libc::c_int::MAX);
    setsockopt_int(fd, libc::SO_SNDBUF, want)?;
    getsockopt_int(fd, libc::SO_SNDBUF).map(|v| usize::try_from(v).unwrap_or(0))
}

/// Reads and clears a socket's pending error (`SO_ERROR`): after a peer
/// closes with unread data the next read reports `ECONNRESET` once and
/// queued records stay readable (measured on Linux 6.8, D3 probe 2).
pub(crate) fn take_error(fd: BorrowedFd<'_>) -> io::Result<i32> {
    getsockopt_int(fd, libc::SO_ERROR)
}

/// The raw facts handle acceptance decides on (`ipc::unix::accept`).
///
/// `fstat` is the raw system call, not the C library's wrapper, which
/// may use `newfstatat` or `statx` instead (D6 review minor 4): the
/// sandbox filter allows exactly this call. On both supported arches the
/// kernel's `struct stat` is the C library's (x86_64's own layout,
/// aarch64's generic one), so `libc::stat` is the buffer.
///
/// # Errors
/// The `fstat` error; a failed socket or seal query is not an error, it
/// leaves those facts at -1 (or 0 for the credentials).
pub(crate) fn fd_facts(fd: BorrowedFd<'_>) -> io::Result<FdFacts> {
    // SAFETY: an all-zero `stat` is a valid value to be overwritten.
    let mut st: libc::stat = unsafe { core::mem::zeroed() };
    // SAFETY: a valid fd and a pointer to a live `stat` the kernel fills.
    let rc = unsafe { libc::syscall(libc::SYS_fstat, fd.as_raw_fd(), &raw mut st) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    let is_socket = st.st_mode & libc::S_IFMT == libc::S_IFSOCK;
    let is_regular = st.st_mode & libc::S_IFMT == libc::S_IFREG;
    let (domain, sock_type, protocol, peercred_pid) = if is_socket {
        (
            getsockopt_int(fd, libc::SO_DOMAIN).unwrap_or(-1),
            getsockopt_int(fd, libc::SO_TYPE).unwrap_or(-1),
            getsockopt_int(fd, libc::SO_PROTOCOL).unwrap_or(-1),
            peer_pid(fd).unwrap_or(0),
        )
    } else {
        (-1, -1, -1, 0)
    };
    let seals = if is_regular {
        get_seals(fd).unwrap_or(-1)
    } else {
        -1
    };
    Ok(FdFacts {
        fd: fd.as_raw_fd(),
        is_socket,
        domain,
        sock_type,
        protocol,
        dev: st.st_dev,
        ino: st.st_ino,
        peercred_pid,
        is_regular,
        size: st.st_size,
        seals,
    })
}

/// The creator of the socket pair `fd` belongs to (`SO_PEERCRED`): for a
/// launch channel end, the broker that launched this process.
pub(crate) fn creator_pid(fd: BorrowedFd<'_>) -> Option<i32> {
    peer_pid(fd).ok().filter(|&p| p > 0)
}

// ---- shared memory (D6), from memfd_create(2), fcntl(2) "File
// sealing", ftruncate(2), mmap(2) and munmap(2) --------------------------

/// `F_ADD_SEALS` and `F_GET_SEALS` (fcntl(2)).
const F_ADD_SEALS: libc::c_int = 1033;
const F_GET_SEALS: libc::c_int = 1034;
/// memfd_create(2) flags: close-on-exec, sealable, and (Linux 6.3) not
/// executable with the exec seal set.
const MFD_CLOEXEC: libc::c_uint = 0x1;
const MFD_ALLOW_SEALING: libc::c_uint = 0x2;
const MFD_NOEXEC_SEAL: libc::c_uint = 0x8;

/// A new memfd of `len` bytes, zeroed, made with `MFD_NOEXEC_SEAL`
/// (review M5), then grown to `len` and sealed against growing and
/// shrinking: its seals are then exactly `GROW | SHRINK | EXEC`.
///
/// # Errors
/// The failing call's error (`EINVAL` from a kernel without
/// `MFD_NOEXEC_SEAL`).
pub(crate) fn memfd_region(len: usize, grow_shrink: libc::c_int) -> io::Result<OwnedFd> {
    let size =
        libc::off_t::try_from(len).map_err(|_e| io::Error::from_raw_os_error(libc::EINVAL))?;
    // SAFETY: a NUL-terminated name and flags; no pointer is kept.
    let rc = unsafe {
        libc::syscall(
            libc::SYS_memfd_create,
            c"izcal-region".as_ptr(),
            MFD_CLOEXEC | MFD_ALLOW_SEALING | MFD_NOEXEC_SEAL,
        )
    };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    let raw = RawFd::try_from(rc).map_err(|_e| io::Error::from_raw_os_error(libc::EBADF))?;
    // SAFETY: the kernel just returned this new descriptor; nothing else
    // owns it.
    let fd = unsafe { OwnedFd::from_raw_fd(raw) };
    // SAFETY: a valid fd and a size.
    check(unsafe { libc::ftruncate(fd.as_raw_fd(), size) })?;
    add_seals(fd.as_fd(), grow_shrink)?;
    Ok(fd)
}

/// `fcntl(F_ADD_SEALS)`.
///
/// # Errors
/// `EBUSY` for `F_SEAL_WRITE` while a shared mapping made through a
/// writable descriptor exists (fact L1b), `EPERM` once `F_SEAL_SEAL` is
/// set.
pub(crate) fn add_seals(fd: BorrowedFd<'_>, seals: libc::c_int) -> io::Result<()> {
    // SAFETY: F_ADD_SEALS takes an int.
    check(unsafe { libc::fcntl(fd.as_raw_fd(), F_ADD_SEALS, seals) }).map(|_| ())
}

/// `fcntl(F_GET_SEALS)`: the seals, or `EINVAL` for a file that cannot be
/// sealed.
pub(crate) fn get_seals(fd: BorrowedFd<'_>) -> io::Result<libc::c_int> {
    // SAFETY: F_GET_SEALS takes no argument.
    check(unsafe { libc::fcntl(fd.as_raw_fd(), F_GET_SEALS) })
}

/// `mmap(NULL, len, PROT_READ [| PROT_WRITE], MAP_SHARED, fd, 0)`; never
/// `PROT_EXEC`.
fn mmap_shared(fd: BorrowedFd<'_>, len: usize, writable: bool) -> io::Result<NonNull<u8>> {
    let prot = if writable {
        libc::PROT_READ | libc::PROT_WRITE
    } else {
        libc::PROT_READ
    };
    // SAFETY: a new mapping at an address the kernel picks; no existing
    // memory is touched.
    let p = unsafe {
        libc::mmap(
            core::ptr::null_mut(),
            len,
            prot,
            libc::MAP_SHARED,
            fd.as_raw_fd(),
            0,
        )
    };
    if p == libc::MAP_FAILED {
        return Err(io::Error::last_os_error());
    }
    NonNull::new(p.cast::<u8>()).ok_or_else(|| io::Error::from_raw_os_error(libc::EFAULT))
}

/// A live `mmap_shared` mapping, unmapped when dropped.
struct Mapped {
    base: NonNull<u8>,
    len: usize,
}

impl Mapped {
    fn new(fd: BorrowedFd<'_>, len: usize, writable: bool) -> io::Result<Self> {
        mmap_shared(fd, len, writable).map(|base| Self { base, len })
    }
}

impl Drop for Mapped {
    fn drop(&mut self) {
        // SAFETY: the one live mapping `new` made; nothing uses it after.
        unsafe { libc::munmap(self.base.as_ptr().cast::<libc::c_void>(), self.len) };
    }
}

// SAFETY: `Mapped` only names an address range; unmapping it from any
// thread is sound.
unsafe impl Send for Mapped {}
// SAFETY: `&Mapped` gives no access to the memory.
unsafe impl Sync for Mapped {}

/// Maps `fd` for `len` bytes, read-write or read-only, as a [`Mapping`]:
/// every access checked and atomic (`ipc::shm`), unmapped when dropped.
///
/// # Errors
/// The kernel's error: `EPERM` for a write mapping of a sealed region.
pub(crate) fn map_region(fd: BorrowedFd<'_>, len: usize, writable: bool) -> io::Result<Mapping> {
    let m = Mapped::new(fd, len, writable)?;
    // SAFETY: `m.base` is a fresh page-aligned mapping of `len` bytes,
    // readable, and writable if asked; `OsMapping` owns `m`, which unmaps
    // only when the `OsMapping` (and its `Raw`) is dropped.
    let raw = unsafe { Raw::new(m.base, len, writable) };
    Ok(Mapping::new(Box::new(OsMapping::new(raw, m))))
}

/// The self-test's probe of fact L1b: the error `F_SEAL_WRITE` gets while
/// a read-only shared mapping through `fd` exists.
pub(crate) fn seal_write_with_reader(fd: BorrowedFd<'_>, len: usize) -> Option<i32> {
    let m = Mapped::new(fd, len, false).ok()?;
    let e = add_seals(fd, 0x8).err().and_then(|e| e.raw_os_error());
    drop(m);
    e
}

/// The self-test's probe that a read-only mapping of `fd` cannot be made
/// writable with `mprotect`.
pub(crate) fn can_widen_reader(fd: BorrowedFd<'_>, len: usize) -> io::Result<bool> {
    let m = Mapped::new(fd, len, false)?;
    // SAFETY: `m` is a live mapping of `len` bytes; only its protection
    // changes, and it is unmapped right after.
    let rc = unsafe {
        libc::mprotect(
            m.base.as_ptr().cast::<libc::c_void>(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
        )
    };
    Ok(rc == 0)
}

/// `pwrite` of one byte at 0: the self-test's check that a sealed
/// region refuses writes through its descriptor.
pub(crate) fn try_pwrite(fd: BorrowedFd<'_>) -> io::Result<()> {
    let b = [1u8];
    // SAFETY: a valid fd and a one-byte buffer.
    let n = unsafe { libc::pwrite(fd.as_raw_fd(), b.as_ptr().cast::<libc::c_void>(), 1, 0) };
    if n < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// `SO_PEERCRED`'s pid: for a `socketpair` end, the process that created
/// the pair, fixed for the socket's life (unix(7); D3 probe 1).
fn peer_pid(fd: BorrowedFd<'_>) -> io::Result<i32> {
    let mut cred = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len = socklen_of::<libc::ucred>();
    // SAFETY: a valid fd and pointers to a live `ucred` and its length.
    check(unsafe {
        libc::getsockopt(
            fd.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&raw mut cred).cast(),
            &raw mut len,
        )
    })?;
    Ok(cred.pid)
}

/// How a send ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SendOutcome {
    /// The whole record is in the peer's queue.
    Sent,
    /// No room now (`EAGAIN`): retry when the socket is writable.
    WouldBlock,
    /// The peer is gone (`EPIPE`, `ECONNRESET`).
    PeerGone,
    /// Any other error: the record size against a shrunk buffer
    /// (`EMSGSIZE`), too many descriptors in flight (`ETOOMANYREFS`), or
    /// another kernel refusal. Carries the errno.
    Failed(i32),
}

/// Sends one record with `fds` attached (`SCM_RIGHTS`), never blocking.
#[expect(
    clippy::disallowed_methods,
    reason = "SEC-4 (d): this is the IPC backend"
)]
pub(crate) fn send_record(fd: BorrowedFd<'_>, bytes: &[u8], fds: &[BorrowedFd<'_>]) -> SendOutcome {
    let mut iov = libc::iovec {
        iov_base: bytes.as_ptr().cast_mut().cast(),
        iov_len: bytes.len(),
    };
    let mut ctl = ControlBuf::new();
    // SAFETY: an all-zero msghdr is valid (null pointers, zero lengths).
    let mut msg: libc::msghdr = unsafe { core::mem::zeroed() };
    msg.msg_iov = &raw mut iov;
    msg.msg_iovlen = 1;
    if !fds.is_empty() {
        let raw: Vec<i32> = fds.iter().map(AsRawFd::as_raw_fd).collect();
        let Some(len) = cmsg::encode_rights(&raw, &mut ctl.0) else {
            return SendOutcome::Failed(libc::EINVAL);
        };
        msg.msg_control = ctl.0.as_mut_ptr().cast();
        msg.msg_controllen = len;
    }
    // SAFETY: `msg` points at `iov` and `ctl`, both live across the call;
    // `iov` describes `bytes`, which the kernel only reads.
    let rc = unsafe {
        libc::sendmsg(
            fd.as_raw_fd(),
            &raw const msg,
            libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL,
        )
    };
    if rc >= 0 {
        // SEQPACKET sends a record whole or not at all.
        return SendOutcome::Sent;
    }
    match last_errno() {
        libc::EAGAIN => SendOutcome::WouldBlock,
        libc::EPIPE | libc::ECONNRESET => SendOutcome::PeerGone,
        e => SendOutcome::Failed(e),
    }
}

/// One received record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Received {
    /// Bytes in the record (as much as fit the buffer).
    pub(crate) len: usize,
    /// The record was longer than the buffer (`MSG_TRUNC`).
    pub(crate) truncated: bool,
    /// Ancillary data was cut (`MSG_CTRUNC`): too small a control
    /// buffer, or no free descriptor slots here (unix(7)).
    pub(crate) control_truncated: bool,
    /// Bytes of ancillary data the kernel wrote.
    pub(crate) control_len: usize,
}

/// How a receive ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecvOutcome<T = Received> {
    /// A record (a zero `len` is an empty record or end of file; the
    /// caller tells them apart by the hang-up event, D3 probe 1 test 5).
    Record(T),
    /// Nothing queued now.
    WouldBlock,
    /// `ECONNRESET`: the peer closed with records it never read. Reported
    /// once; queued records are still readable after it.
    Reset,
    /// Any other errno.
    Failed(i32),
}

/// Receives one record with no control buffer, never blocking: no
/// descriptor can be installed (any sent are discarded by the kernel and
/// `control_truncated` is set). The credit-record path, and the probe
/// that tells end of file from an empty record.
pub(crate) fn recv_record(fd: BorrowedFd<'_>, buf: &mut [u8]) -> RecvOutcome {
    recvmsg(fd, buf, None)
}

/// A data record with what came with it.
#[derive(Debug)]
pub(crate) struct WithRights {
    /// The record.
    pub(crate) rec: Received,
    /// Its parsed control data.
    pub(crate) control: cmsg::ParsedControl,
    /// Every descriptor the kernel installed for it, owned, by number:
    /// each closes exactly once, when dropped, unless the caller keeps it.
    pub(crate) owned: BTreeMap<RawFd, OwnedFd>,
}

/// Receives one data record and its ancillary data, never blocking, and
/// takes ownership of every descriptor the kernel installed for it, in
/// the same call, so a number is never owned twice. Received descriptors
/// are close-on-exec (`MSG_CMSG_CLOEXEC`).
pub(crate) fn recv_with_rights(fd: BorrowedFd<'_>, buf: &mut [u8]) -> RecvOutcome<WithRights> {
    let mut ctl = ControlBuf::new();
    let rec = match recvmsg(fd, buf, Some(&mut ctl)) {
        RecvOutcome::Record(r) => r,
        RecvOutcome::WouldBlock => return RecvOutcome::WouldBlock,
        RecvOutcome::Reset => return RecvOutcome::Reset,
        RecvOutcome::Failed(e) => return RecvOutcome::Failed(e),
    };
    let control_len = rec.control_len.min(CONTROL_LEN);
    let control = cmsg::parse(ctl.0.get(..control_len).unwrap_or(&[]));
    let mut owned = BTreeMap::new();
    for &raw in control.close_set() {
        if raw < 0 || owned.contains_key(&raw) || !is_open(raw) {
            continue;
        }
        // The parser reads descriptor numbers only from `SCM_RIGHTS`
        // messages in the control bytes the kernel wrote (unix(7)); each
        // installed number is new and distinct, and no other code has seen
        // it. The map check above keeps one owner per number.
        // SAFETY: `raw` was installed in this process by the recvmsg above
        // and has no other owner (see above).
        let fd = unsafe { OwnedFd::from_raw_fd(raw) };
        owned.insert(raw, fd);
    }
    RecvOutcome::Record(WithRights {
        rec,
        control,
        owned,
    })
}

#[expect(
    clippy::disallowed_methods,
    reason = "SEC-4 (d): this is the IPC backend"
)]
fn recvmsg(fd: BorrowedFd<'_>, buf: &mut [u8], ctl: Option<&mut ControlBuf>) -> RecvOutcome {
    let mut iov = libc::iovec {
        iov_base: buf.as_mut_ptr().cast(),
        iov_len: buf.len(),
    };
    // SAFETY: an all-zero msghdr is valid.
    let mut msg: libc::msghdr = unsafe { core::mem::zeroed() };
    msg.msg_iov = &raw mut iov;
    msg.msg_iovlen = 1;
    if let Some(c) = ctl {
        msg.msg_control = c.0.as_mut_ptr().cast();
        msg.msg_controllen = CONTROL_LEN;
    }
    // SAFETY: `msg` points at `iov` (describing `buf`) and the control
    // buffer, all live and writable across the call.
    let rc = unsafe {
        libc::recvmsg(
            fd.as_raw_fd(),
            &raw mut msg,
            libc::MSG_DONTWAIT | libc::MSG_CMSG_CLOEXEC,
        )
    };
    if rc < 0 {
        return match last_errno() {
            libc::EAGAIN => RecvOutcome::WouldBlock,
            libc::ECONNRESET => RecvOutcome::Reset,
            e => RecvOutcome::Failed(e),
        };
    }
    RecvOutcome::Record(Received {
        len: usize::try_from(rc).unwrap_or(0),
        truncated: msg.msg_flags & libc::MSG_TRUNC != 0,
        control_truncated: msg.msg_flags & libc::MSG_CTRUNC != 0,
        control_len: msg.msg_controllen,
    })
}

/// Takes the two descriptors a test child was started with at `fds`
/// (tests only). A once-only claim: a second call in the same process
/// gets `None`, so the numbers are never owned twice.
#[cfg(test)]
pub(crate) fn take_test_child_fds(fds: (RawFd, RawFd)) -> Option<(OwnedFd, OwnedFd)> {
    use std::sync::atomic::{AtomicBool, Ordering};
    static TAKEN: AtomicBool = AtomicBool::new(false);
    if fds.0 == fds.1 || !is_open(fds.0) || !is_open(fds.1) || TAKEN.swap(true, Ordering::SeqCst) {
        return None;
    }
    // SAFETY: the test harness started this process with exactly these
    // descriptors placed by `spawn_with_fds`, and the once-only claim
    // above makes this the only owner in the process.
    let up = unsafe { OwnedFd::from_raw_fd(fds.0) };
    // SAFETY: as above, for the second descriptor.
    let down = unsafe { OwnedFd::from_raw_fd(fds.1) };
    Some((up, down))
}

/// Whether `fd` names an open descriptor (`fcntl(F_GETFD)`).
pub(crate) fn is_open(fd: RawFd) -> bool {
    // SAFETY: F_GETFD takes no pointer; an invalid fd yields EBADF.
    unsafe { libc::fcntl(fd, libc::F_GETFD) >= 0 }
}

/// Sets close-on-exec.
pub(crate) fn set_cloexec(fd: BorrowedFd<'_>) -> io::Result<()> {
    // SAFETY: F_SETFD takes an int flag.
    check(unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) }).map(|_| ())
}

/// `shutdown(SHUT_WR)`: the peer reads what was sent, then end of file.
pub(crate) fn shutdown_write(fd: BorrowedFd<'_>) -> io::Result<()> {
    // SAFETY: plain syscall on a valid fd.
    check(unsafe { libc::shutdown(fd.as_raw_fd(), libc::SHUT_WR) }).map(|_| ())
}

/// The soft `RLIMIT_NOFILE`.
pub(crate) fn fd_limit() -> usize {
    let mut r = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: a pointer to a live rlimit.
    if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &raw mut r) } != 0 {
        return 0;
    }
    usize::try_from(r.rlim_cur).unwrap_or(usize::MAX)
}

/// The number of open descriptors below `bound`, counted with
/// `fcntl(F_GETFD)`: the transport's starting point for its descriptor
/// headroom (taken once, when the transport starts).
pub(crate) fn count_open(bound: usize) -> usize {
    let top = RawFd::try_from(bound).unwrap_or(RawFd::MAX);
    (0..top).filter(|&fd| is_open(fd)).count()
}

/// An epoll instance (epoll(7)).
#[derive(Debug)]
pub(crate) struct Epoll(OwnedFd);

/// Readiness for one registration, by its token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ready {
    /// The token the registration was made with.
    pub(crate) token: u64,
    /// Readable.
    pub(crate) input: bool,
    /// Writable.
    pub(crate) output: bool,
    /// The peer hung up or shut down writing, or an error is pending.
    pub(crate) hangup: bool,
}

/// What to wait for. Channel ends are one-shot (re-armed after each
/// event, so a paused receiver is not reported over and over); the wake
/// eventfd is level-triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Interest {
    /// Readable, or hung up.
    pub(crate) read: bool,
    /// Writable.
    pub(crate) write: bool,
    /// One-shot.
    pub(crate) oneshot: bool,
}

impl Interest {
    /// The wake eventfd: readable, level-triggered.
    pub(crate) const WAKER: Self = Self {
        read: true,
        write: false,
        oneshot: false,
    };

    /// A channel end: one-shot, reading and/or writing.
    pub(crate) const fn end(read: bool, write: bool) -> Self {
        Self {
            read,
            write,
            oneshot: true,
        }
    }

    fn bits(self) -> u32 {
        let mut b = 0;
        if self.read {
            b |= libc::EPOLLIN | libc::EPOLLRDHUP;
        }
        if self.write {
            b |= libc::EPOLLOUT;
        }
        if self.oneshot {
            b |= libc::EPOLLONESHOT;
        }
        // Small positive flag bits; the cast keeps them.
        #[expect(clippy::cast_sign_loss, reason = "epoll flag bits")]
        {
            b as u32
        }
    }
}

impl Epoll {
    /// `epoll_create1(EPOLL_CLOEXEC)`.
    pub(crate) fn new() -> io::Result<Self> {
        // SAFETY: plain syscall.
        let fd = check(unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) })?;
        // SAFETY: a new descriptor this process owns.
        Ok(Self(unsafe { OwnedFd::from_raw_fd(fd) }))
    }

    fn ctl(
        &self,
        op: libc::c_int,
        fd: BorrowedFd<'_>,
        interest: Option<Interest>,
        token: u64,
    ) -> io::Result<()> {
        let mut ev = libc::epoll_event {
            events: interest.map_or(0, Interest::bits),
            u64: token,
        };
        // SAFETY: valid fds and a pointer to a live epoll_event.
        check(unsafe { libc::epoll_ctl(self.0.as_raw_fd(), op, fd.as_raw_fd(), &raw mut ev) })
            .map(|_| ())
    }

    /// Registers `fd` with `token`.
    pub(crate) fn add(&self, fd: BorrowedFd<'_>, interest: Interest, token: u64) -> io::Result<()> {
        self.ctl(libc::EPOLL_CTL_ADD, fd, Some(interest), token)
    }

    /// Re-arms or changes a registration.
    pub(crate) fn modify(
        &self,
        fd: BorrowedFd<'_>,
        interest: Interest,
        token: u64,
    ) -> io::Result<()> {
        self.ctl(libc::EPOLL_CTL_MOD, fd, Some(interest), token)
    }

    /// Removes a registration. Always before closing the descriptor: epoll
    /// tracks open file descriptions, and a transferred end's description
    /// outlives this process's descriptor number (epoll(7), Q6).
    pub(crate) fn delete(&self, fd: BorrowedFd<'_>) -> io::Result<()> {
        self.ctl(libc::EPOLL_CTL_DEL, fd, None, 0)
    }

    /// Waits and fills `out`; returns how many entries are ready. With
    /// `timeout_ms` `None` it waits for an event only, so an idle process
    /// makes no wakeups (footprint FP-18); a timeout is used only while a
    /// transfer is waiting on a deadline.
    pub(crate) fn wait(
        &self,
        out: &mut Vec<Ready>,
        max: usize,
        timeout_ms: Option<u32>,
    ) -> io::Result<usize> {
        let cap = max.clamp(1, 256);
        let mut evs = vec![libc::epoll_event { events: 0, u64: 0 }; cap];
        let n = loop {
            // SAFETY: `evs` holds `cap` live entries for the kernel to fill;
            // a null signal mask means "unchanged".
            let rc = unsafe {
                libc::epoll_pwait(
                    self.0.as_raw_fd(),
                    evs.as_mut_ptr(),
                    libc::c_int::try_from(cap).unwrap_or(1),
                    timeout_ms.map_or(-1, |t| libc::c_int::try_from(t).unwrap_or(libc::c_int::MAX)),
                    core::ptr::null(),
                )
            };
            if rc >= 0 {
                break usize::try_from(rc).unwrap_or(0);
            }
            if last_errno() != libc::EINTR {
                return Err(io::Error::last_os_error());
            }
        };
        out.clear();
        let (inp, outp, hup) = (
            libc::EPOLLIN as u32,
            libc::EPOLLOUT as u32,
            (libc::EPOLLHUP | libc::EPOLLRDHUP | libc::EPOLLERR) as u32,
        );
        for ev in evs.iter().take(n) {
            let bits = ev.events;
            out.push(Ready {
                token: ev.u64,
                input: bits & inp != 0,
                output: bits & outp != 0,
                hangup: bits & hup != 0,
            });
        }
        Ok(n)
    }
}

/// An eventfd for waking the IO thread (eventfd(2)).
#[derive(Debug)]
pub(crate) struct Waker(OwnedFd);

impl Waker {
    /// `eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK)`.
    pub(crate) fn new() -> io::Result<Self> {
        // SAFETY: plain syscall.
        let fd = check(unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) })?;
        // SAFETY: a new descriptor this process owns.
        Ok(Self(unsafe { OwnedFd::from_raw_fd(fd) }))
    }

    /// Its descriptor, to register.
    pub(crate) fn fd(&self) -> BorrowedFd<'_> {
        use std::os::fd::AsFd;
        self.0.as_fd()
    }

    /// Adds one to the counter; the IO thread wakes.
    pub(crate) fn wake(&self) {
        let one: u64 = 1;
        // SAFETY: writes 8 bytes from a live u64. A full counter (EAGAIN)
        // already means a wake is pending.
        let _ = unsafe { libc::write(self.0.as_raw_fd(), (&raw const one).cast(), 8) };
    }

    /// Clears the counter.
    pub(crate) fn drain(&self) {
        let mut v: u64 = 0;
        // SAFETY: reads 8 bytes into a live u64; EAGAIN when already zero.
        let _ = unsafe { libc::read(self.0.as_raw_fd(), (&raw mut v).cast(), 8) };
    }
}

/// Starts `program` with `args` and the extra environment `env`, with
/// each `(source, target)` descriptor placed at `target` in the child
/// (posix_spawn(3), `posix_spawn_file_actions_adddup2`; `dup2` clears
/// close-on-exec on the target). Every other descriptor here is
/// close-on-exec, so the child gets stdio plus exactly these. Returns the
/// child's process ID. For the launcher (phase E) and the transport's
/// cross-process tests.
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
pub(crate) fn spawn_with_fds(
    program: &std::ffi::CStr,
    args: &[&std::ffi::CStr],
    env: &[&std::ffi::CStr],
    fds: &[(BorrowedFd<'_>, RawFd)],
) -> io::Result<libc::pid_t> {
    let mut argv: Vec<*mut libc::c_char> = Vec::with_capacity(args.len() + 2);
    argv.push(program.as_ptr().cast_mut());
    argv.extend(args.iter().map(|a| a.as_ptr().cast_mut()));
    argv.push(core::ptr::null_mut());
    // The child's environment: this process's, plus `env`.
    let inherited: Vec<std::ffi::CString> = std::env::vars_os()
        .filter_map(|(k, v)| {
            let mut kv = k.into_encoded_bytes();
            kv.push(b'=');
            kv.extend(v.into_encoded_bytes());
            std::ffi::CString::new(kv).ok()
        })
        .collect();
    let mut envp: Vec<*mut libc::c_char> =
        inherited.iter().map(|e| e.as_ptr().cast_mut()).collect();
    envp.extend(env.iter().map(|e| e.as_ptr().cast_mut()));
    envp.push(core::ptr::null_mut());
    // SAFETY: an all-zero file-actions object is valid storage for init.
    let mut actions: libc::posix_spawn_file_actions_t = unsafe { core::mem::zeroed() };
    // SAFETY: initialises the object just declared.
    if unsafe { libc::posix_spawn_file_actions_init(&raw mut actions) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let mut rc = 0;
    for (src, target) in fds {
        // SAFETY: an initialised file-actions object and two descriptors.
        rc = unsafe {
            libc::posix_spawn_file_actions_adddup2(&raw mut actions, src.as_raw_fd(), *target)
        };
        if rc != 0 {
            break;
        }
    }
    let mut pid: libc::pid_t = 0;
    if rc == 0 {
        // SAFETY: `program`, `argv` and `envp` are NUL-terminated and
        // null-terminated arrays of live C strings; `actions` is
        // initialised; the attributes pointer may be null.
        rc = unsafe {
            libc::posix_spawn(
                &raw mut pid,
                program.as_ptr(),
                &raw const actions,
                core::ptr::null(),
                argv.as_ptr(),
                envp.as_ptr(),
            )
        };
    }
    // SAFETY: destroys the object initialised above.
    unsafe { libc::posix_spawn_file_actions_destroy(&raw mut actions) };
    if rc == 0 {
        Ok(pid)
    } else {
        Err(io::Error::from_raw_os_error(rc))
    }
}

/// Waits for child `pid` to end; returns its exit status or the signal
/// that ended it (negative).
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
pub(crate) fn wait_child(pid: libc::pid_t) -> io::Result<i32> {
    let mut status: libc::c_int = 0;
    loop {
        // SAFETY: a pointer to a live int.
        let rc = unsafe { libc::waitpid(pid, &raw mut status, 0) };
        if rc == pid {
            break;
        }
        if rc < 0 && last_errno() != libc::EINTR {
            return Err(io::Error::last_os_error());
        }
    }
    if libc::WIFEXITED(status) {
        Ok(libc::WEXITSTATUS(status))
    } else {
        Ok(-libc::WTERMSIG(status))
    }
}

/// Clears `O_NONBLOCK` on a descriptor's open file description, as a
/// former holder of a transferred end could (tests only).
#[cfg(test)]
pub(crate) fn clear_nonblock(fd: BorrowedFd<'_>) {
    // SAFETY: F_GETFL takes no pointer.
    let fl = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) };
    // SAFETY: F_SETFL takes an int flag word.
    unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, fl & !libc::O_NONBLOCK) };
}

/// How many descriptors below `bound` in this process refer to the
/// same file as `fd` (same `st_dev` and `st_ino`), `fd` included (tests
/// only). Unlike [`count_open`] it cannot be moved by other threads
/// opening and closing unrelated descriptors.
#[cfg(test)]
pub(crate) fn count_refs(fd: BorrowedFd<'_>, bound: usize) -> usize {
    let Ok(want) = fd_facts(fd) else {
        return 0;
    };
    let top = RawFd::try_from(bound).unwrap_or(RawFd::MAX);
    (0..top)
        .filter(|&n| {
            // SAFETY: fstat on any number: an unused one yields EBADF,
            // and the stat is a live local.
            let mut st: libc::stat = unsafe { core::mem::zeroed() };
            // SAFETY: as above.
            let ok = unsafe { libc::fstat(n, &raw mut st) } == 0;
            ok && st.st_dev == want.dev && st.st_ino == want.ino
        })
        .count()
}

/// Sets the soft and hard `RLIMIT_NOFILE` (tests only, in a child).
#[cfg(test)]
pub(crate) fn set_fd_limit(n: usize) {
    let v = libc::rlim_t::try_from(n).unwrap_or(libc::rlim_t::MAX);
    let r = libc::rlimit {
        rlim_cur: v,
        rlim_max: v,
    };
    // SAFETY: a pointer to a live rlimit.
    unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &raw const r) };
}

/// Tests only: the errno of an executable shared mapping of `fd` (0 if
/// it was made), and of `mprotect` to executable on a fresh private page
/// (0 if allowed). Inside the sandbox both are refused (review M5).
#[cfg(test)]
pub(crate) fn try_exec(fd: BorrowedFd<'_>, len: usize) -> (i32, i32) {
    // SAFETY: a new mapping the kernel places; unmapped below if made.
    let p = unsafe {
        libc::mmap(
            core::ptr::null_mut(),
            len,
            libc::PROT_READ | libc::PROT_EXEC,
            libc::MAP_SHARED,
            fd.as_raw_fd(),
            0,
        )
    };
    let map = if p == libc::MAP_FAILED {
        last_errno()
    } else {
        // SAFETY: the mapping just made, unused.
        unsafe { libc::munmap(p, len) };
        0
    };
    // SAFETY: a new private anonymous page.
    let q = unsafe {
        libc::mmap(
            core::ptr::null_mut(),
            4096,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if q == libc::MAP_FAILED {
        return (map, last_errno());
    }
    // SAFETY: `q` is the page just mapped; only its protection changes.
    let rc = unsafe { libc::mprotect(q, 4096, libc::PROT_READ | libc::PROT_EXEC) };
    let protect = if rc == 0 { 0 } else { last_errno() };
    // SAFETY: as above, unused after.
    unsafe { libc::munmap(q, 4096) };
    (map, protect)
}

#[cfg(test)]
mod tests {
    use std::os::fd::AsFd;

    use super::*;

    /// The pure parser's layout constants are libc's (cmsg(3)).
    #[test]
    fn cmsg_layout_matches_libc() {
        // SAFETY: CMSG_* compute sizes only.
        let len0 = unsafe { libc::CMSG_LEN(0) } as usize;
        // SAFETY: as above.
        let space4 = unsafe { libc::CMSG_SPACE(4) } as usize;
        // SAFETY: as above.
        let space_cred =
            unsafe { libc::CMSG_SPACE(core::mem::size_of::<libc::ucred>() as u32) } as usize;
        assert_eq!(len0, cmsg::cmsg_len(0));
        assert_eq!(space4, cmsg::cmsg_space(4));
        assert_eq!(space_cred, cmsg::cmsg_space(cmsg::UCRED_LEN));
        assert_eq!(cmsg::SOL_SOCKET, libc::SOL_SOCKET);
        assert_eq!(cmsg::SCM_RIGHTS, libc::SCM_RIGHTS);
        assert_eq!(cmsg::SCM_CREDENTIALS, libc::SCM_CREDENTIALS);
        assert_eq!(cmsg::UCRED_LEN, core::mem::size_of::<libc::ucred>());
    }

    #[test]
    fn a_record_with_fds_round_trips_and_carries_credentials() {
        let (a, b) = socketpair().unwrap();
        set_passcred(b.as_fd()).unwrap();
        let (x, _y) = socketpair().unwrap();
        assert_eq!(
            send_record(a.as_fd(), b"hello", &[x.as_fd()]),
            SendOutcome::Sent
        );
        let mut buf = [0u8; 64];
        let RecvOutcome::Record(r) = recv_with_rights(b.as_fd(), &mut buf) else {
            panic!("no record");
        };
        assert_eq!(&buf[..r.rec.len], b"hello");
        assert!(!r.rec.truncated && !r.rec.control_truncated);
        let parsed = &r.control;
        assert!(parsed.outcome.is_ok());
        assert_eq!(parsed.fd_count(), 1);
        let pid = i32::try_from(std::process::id()).unwrap();
        assert_eq!(parsed.cred.map(|c| c.pid), Some(pid));
        assert_eq!(r.owned.len(), 1);
        for fd in parsed.fds() {
            let owned = &r.owned[fd];
            let facts = fd_facts(owned.as_fd()).unwrap();
            assert!(facts.is_socket);
            assert_eq!(facts.domain, libc::AF_UNIX);
            assert_eq!(facts.sock_type, libc::SOCK_SEQPACKET);
            assert_eq!(facts.peercred_pid, pid);
        }
    }

    #[test]
    fn epoll_and_waker() {
        let ep = Epoll::new().unwrap();
        let w = Waker::new().unwrap();
        ep.add(w.fd(), Interest::WAKER, 7).unwrap();
        w.wake();
        let mut out = Vec::new();
        assert_eq!(ep.wait(&mut out, 8, None).unwrap(), 1);
        assert_eq!(out[0].token, 7);
        assert!(out[0].input);
        w.drain();
    }
}
