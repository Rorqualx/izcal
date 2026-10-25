//! The Win32 calls behind the Windows transport: named pipes,
//! overlapped I/O with completion routines, handle duplication and the
//! kernel's facts about a handle. The only `unsafe` in the transport
//! (CS-TY-13, `unsafe-inventory.toml` entry `ipc::windows::sys`).
//! Written from the Microsoft Learn reference pages for each call and
//! the D5 probes; no header or other project's code.
//!
//! **I/O model.** Every read and write is started on the transport's IO
//! thread with `ReadFileEx`/`WriteFileEx`, and completes there: Windows
//! queues the completion routine as an APC to the thread that started
//! the call, and runs it only while that thread waits alertably. The IO
//! thread's wait has no timeout unless a transfer deadline is pending,
//! so an idle process has no wakeups (FP-18). Other threads wake it with
//! `QueueUserAPC`. No completion port is used: a port is bound to the
//! file object, which moves between processes with the handle.
//!
//! **Ownership.** A started operation owns its buffer and a reference to
//! its handle (`Arc<OwnedHandle>`) until its completion routine runs,
//! exactly once, so neither can be freed while the kernel writes to it.

#![expect(
    unsafe_code,
    reason = "G5: ipc::windows::sys (named pipes, overlapped I/O, DuplicateHandle)"
)]

use core::ffi::c_void;
use core::mem::{size_of, zeroed};
use core::ptr::{null, null_mut};
use std::cell::RefCell;
use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, OwnedHandle, RawHandle};
use std::sync::Arc;

use windows_sys::Wdk::Foundation::NtQueryObject;
use windows_sys::Win32::Foundation::{
    DUPLICATE_CLOSE_SOURCE, DUPLICATE_SAME_ACCESS, DuplicateHandle, ERROR_IO_PENDING,
    ERROR_PIPE_CONNECTED, GetLastError, HANDLE, INVALID_HANDLE_VALUE, LocalFree, WAIT_OBJECT_0,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_APPCONTAINER_INFORMATION,
    TOKEN_QUERY, TokenAppContainerSid,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, FILE_NAME_INFO,
    FILE_READ_ATTRIBUTES, FILE_READ_DATA, FILE_TYPE_PIPE, FILE_WRITE_ATTRIBUTES, FILE_WRITE_DATA,
    FileNameInfo, GetFileInformationByHandleEx, GetFileType, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
    ReadFile, ReadFileEx, SYNCHRONIZE, WriteFileEx,
};
use windows_sys::Win32::System::IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED};
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, GetNamedPipeInfo, PIPE_READMODE_MESSAGE,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_SERVER_END, PIPE_TYPE_MESSAGE, PIPE_WAIT,
    SetNamedPipeHandleState,
};
use windows_sys::Win32::System::Threading::{
    CreateEventW, GetCurrentProcess, GetCurrentProcessId, GetCurrentThread, OpenProcessToken,
    QueueUserAPC, SleepEx, WaitForSingleObject,
};

/// The access a channel end is given when it moves: read and write data
/// (data one way, credits the other), read attributes (so the broker can
/// classify it on its next move) and synchronize. Never
/// `DUPLICATE_SAME_ACCESS` onward (IPC-R1): in particular no
/// `FILE_WRITE_ATTRIBUTES`, so a holder cannot change the pipe's read
/// mode.
pub(crate) const MOVE_ACCESS: u32 =
    FILE_READ_DATA | FILE_WRITE_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE;

/// Each pipe direction's buffer, in bytes: room for one longest data
/// message and its credits (with no buffer every write pends until the
/// reader reads, probe "full pipe").
const PIPE_BUFFER: u32 = 128 * 1024;

/// `ERROR_MORE_DATA`: a message longer than the read buffer.
pub(crate) const ERROR_MORE_DATA: u32 = 234;

/// `ERROR_OPERATION_ABORTED`: the operation was cancelled.
pub(crate) const ERROR_OPERATION_ABORTED: u32 = 995;

/// `STATUS_SUCCESS`.
const STATUS_SUCCESS: i32 = 0;

/// `ObjectBasicInformation`, `NtQueryObject`'s class 0.
const OBJECT_BASIC_INFORMATION: i32 = 0;

/// `PUBLIC_OBJECT_BASIC_INFORMATION`, as the `NtQueryObject` reference
/// page lays it out: four ULONGs, then ten reserved (56 bytes, the size
/// the call wants, probe P16).
#[repr(C)]
#[derive(Default)]
struct ObjectBasicInformation {
    attributes: u32,
    granted_access: u32,
    handle_count: u32,
    pointer_count: u32,
    reserved: [u32; 10],
}

const _: () = assert!(size_of::<ObjectBasicInformation>() == 56);

/// The current-process pseudo-handle (needs no close).
fn current_process() -> HANDLE {
    // SAFETY: no arguments; returns a constant pseudo-handle.
    unsafe { GetCurrentProcess() }
}

/// The current-thread pseudo-handle (needs no close).
fn current_thread_pseudo() -> HANDLE {
    // SAFETY: no arguments; returns a constant pseudo-handle.
    unsafe { GetCurrentThread() }
}

/// The last error of this thread.
fn last_error() -> u32 {
    // SAFETY: reads the calling thread's error slot only.
    unsafe { GetLastError() }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(core::iter::once(0)).collect()
}

/// The raw handle of an owned one.
fn raw(h: &OwnedHandle) -> HANDLE {
    h.as_raw_handle().cast()
}

/// A handle value as the u64 the control records carry.
pub(crate) fn value(h: &OwnedHandle) -> u64 {
    u64::try_from(h.as_raw_handle().addr()).unwrap_or(0)
}

/// Gives up ownership of a handle without closing it: its value has
/// been, or is being, pulled out of this process by the broker
/// (`DUPLICATE_CLOSE_SOURCE` closes it here).
pub(crate) fn forget(h: OwnedHandle) {
    let _ = h.into_raw_handle();
}

/// Takes ownership of handle value `v`. Private: every caller goes
/// through [`adopt_pipe`], which checks the value first.
fn adopt(v: u64) -> Option<OwnedHandle> {
    let addr = usize::try_from(v).ok().filter(|&a| a != 0)?;
    let raw_handle: RawHandle = core::ptr::without_provenance_mut(addr);
    // SAFETY: `adopt_pipe` checked `v` is a well-formed handle value and
    // owns what it adopts only once it names the expected pipe end; the
    // broker that put it in this table handed it over to this process.
    Some(unsafe { OwnedHandle::from_raw_handle(raw_handle) })
}

/// Adopts a pipe end the broker reports having duplicated into this
/// process (a `Hello` or `Redeemed` record): `None`, leaving the value
/// alone, unless it is a well-formed handle value naming a message-mode
/// pipe's server end (`server_end`) or client end. The broker is trusted
/// to name only handles it gave; this keeps a wrong value from taking
/// over anything but a pipe end, and the transport refuses values of
/// ends it already holds.
pub(crate) fn adopt_pipe(v: u64, server_end: bool) -> Option<OwnedHandle> {
    crate::ipc::win::accept::check_value(v).ok()?;
    let h = adopt(v)?;
    let f = facts(&h);
    if f.is_pipe && f.message && f.server_end == server_end {
        Some(h)
    } else {
        forget(h);
        None
    }
}

/// Whether this process runs in an AppContainer, and its container SID
/// as a string if so.
pub(crate) fn app_container_sid() -> Option<String> {
    let mut token: HANDLE = null_mut();
    // SAFETY: the current-process pseudo-handle needs no close; valid
    // out-pointer.
    let ok = unsafe { OpenProcessToken(current_process(), TOKEN_QUERY, &raw mut token) };
    if ok == 0 {
        return None;
    }
    // SAFETY: `token` was just opened and is owned here.
    let token = unsafe { OwnedHandle::from_raw_handle(token.cast()) };
    let mut buf = [0u64; 64];
    let mut len = 0u32;
    // SAFETY: the buffer is 512 bytes, as passed; valid out-pointer.
    let ok = unsafe {
        GetTokenInformation(
            raw(&token),
            TokenAppContainerSid,
            buf.as_mut_ptr().cast::<c_void>(),
            512,
            &raw mut len,
        )
    };
    if ok == 0 {
        return None;
    }
    // SAFETY: on success the buffer starts with the information
    // struct; the 8-byte alignment of `buf` suits its pointer field.
    let info = unsafe { &*buf.as_ptr().cast::<TOKEN_APPCONTAINER_INFORMATION>() };
    if info.TokenAppContainer.is_null() {
        return None;
    }
    let mut s: *mut u16 = null_mut();
    // SAFETY: the SID pointer is valid while `buf` lives; valid
    // out-pointer.
    if unsafe { ConvertSidToStringSidW(info.TokenAppContainer, &raw mut s) } == 0 {
        return None;
    }
    let out = wide_to_string(s);
    // SAFETY: the string was allocated by the call for us to free with
    // LocalFree.
    unsafe { LocalFree(s.cast()) };
    Some(out)
}

/// A security descriptor from SDDL, freed on drop.
struct Descriptor(PSECURITY_DESCRIPTOR);

impl Drop for Descriptor {
    fn drop(&mut self) {
        // SAFETY: allocated by ConvertStringSecurityDescriptor..., whose
        // documented release is LocalFree.
        unsafe { LocalFree(self.0) };
    }
}

fn descriptor(sddl: &str) -> io::Result<Descriptor> {
    let w = wide(sddl);
    let mut sd: PSECURITY_DESCRIPTOR = null_mut();
    // SAFETY: NUL-terminated string; valid out-pointer; no size wanted.
    let ok = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            w.as_ptr(),
            SDDL_REVISION_1,
            &raw mut sd,
            null_mut(),
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(Descriptor(sd))
}

/// Makes a channel's pipe: a one-instance, duplex, message-mode pipe
/// with a random name, and its client end opened and connected at once
/// (so no other client can ever open it, probe "Pair"). Returns (server
/// end, client end): the receiver and the sender.
///
/// The name is `\\.\pipe\izcal-<32 hex>`, or under `LOCAL\` in an
/// AppContainer, which puts it in the container's own namespace. The
/// DACL grants the owner only (and, in a container, the container):
/// an object made at Untrusted integrity is also labelled so, which no
/// other process can write up to.
pub(crate) fn pipe_pair(
    container: Option<&str>,
    id: &[u8; 16],
) -> io::Result<(OwnedHandle, OwnedHandle)> {
    named_pair(crate::ipc::win::accept::CHANNEL_PREFIX, container, id)
}

/// Makes a control link: as [`pipe_pair`], under a name no channel has
/// (so a link end is never taken for a channel end). The broker makes
/// it, in the global namespace; (server, client) = (broker's, child's).
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
pub(crate) fn link_pair(id: &[u8; 16]) -> io::Result<(OwnedHandle, OwnedHandle)> {
    named_pair("izcal-link-", None, id)
}

fn named_pair(
    prefix: &str,
    container: Option<&str>,
    id: &[u8; 16],
) -> io::Result<(OwnedHandle, OwnedHandle)> {
    let hex: String = id.iter().map(|b| format!("{b:02x}")).collect();
    let (name, sddl) = match container {
        Some(sid) => (
            format!(r"\\.\pipe\LOCAL\{prefix}{hex}"),
            format!("D:P(A;;FA;;;OW)(A;;FA;;;{sid})"),
        ),
        None => (
            format!(r"\\.\pipe\{prefix}{hex}"),
            "D:P(A;;FA;;;OW)".to_owned(),
        ),
    };
    let sd = descriptor(&sddl)?;
    let sa = SECURITY_ATTRIBUTES {
        nLength: u32::try_from(size_of::<SECURITY_ATTRIBUTES>()).unwrap_or(0),
        lpSecurityDescriptor: sd.0,
        bInheritHandle: 0,
    };
    let w = wide(&name);
    // SAFETY: NUL-terminated name; the attributes live for the call.
    let server = unsafe {
        CreateNamedPipeW(
            w.as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            PIPE_BUFFER,
            PIPE_BUFFER,
            0,
            &raw const sa,
        )
    };
    if server == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a new handle owned here.
    let server = unsafe { OwnedHandle::from_raw_handle(server.cast()) };
    // SAFETY: NUL-terminated name; no security attributes or template.
    let client = unsafe {
        CreateFileW(
            w.as_ptr(),
            FILE_READ_DATA
                | FILE_WRITE_DATA
                | FILE_READ_ATTRIBUTES
                | FILE_WRITE_ATTRIBUTES
                | SYNCHRONIZE,
            0,
            null(),
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            null_mut(),
        )
    };
    if client == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a new handle owned here.
    let client = unsafe { OwnedHandle::from_raw_handle(client.cast()) };
    let mode = PIPE_READMODE_MESSAGE;
    // SAFETY: valid handle; the mode lives for the call; no counts.
    if unsafe { SetNamedPipeHandleState(raw(&client), &raw const mode, null(), null()) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a zeroed OVERLAPPED is its documented initial state.
    let mut ov: OVERLAPPED = unsafe { zeroed() };
    // SAFETY: valid server handle; the client is already open, so the
    // call returns at once with ERROR_PIPE_CONNECTED and keeps no
    // reference to `ov`.
    let ok = unsafe { ConnectNamedPipe(raw(&server), &raw mut ov) };
    let err = last_error();
    if ok == 0 && err != ERROR_PIPE_CONNECTED {
        return Err(io::Error::from_raw_os_error(
            i32::try_from(err).unwrap_or(0),
        ));
    }
    Ok((server, client))
}

/// What the kernel says about a pipe handle (the raw facts behind
/// [`crate::ipc::win::accept::Facts`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RawFacts {
    /// `GetFileType` is `FILE_TYPE_PIPE`.
    pub(crate) is_pipe: bool,
    /// The server end.
    pub(crate) server_end: bool,
    /// A message-type pipe.
    pub(crate) message: bool,
    /// The most instances the pipe allows.
    pub(crate) max_instances: u32,
    /// `GetFileInformationByHandleEx(FileNameInfo)`: the pipe's name
    /// relative to the pipe device.
    pub(crate) name: Option<String>,
    /// Handles to the object in every process.
    pub(crate) handle_count: u32,
}

/// Asks the kernel about `h`. A query that fails leaves a value no rule
/// accepts.
pub(crate) fn facts(h: &OwnedHandle) -> RawFacts {
    // SAFETY: valid handle.
    let is_pipe = unsafe { GetFileType(raw(h)) } == FILE_TYPE_PIPE;
    if !is_pipe {
        return RawFacts::default();
    }
    let (mut flags, mut out_size, mut in_size, mut max) = (0u32, 0u32, 0u32, 0u32);
    // SAFETY: valid handle and out-pointers.
    let info_ok = unsafe {
        GetNamedPipeInfo(
            raw(h),
            &raw mut flags,
            &raw mut out_size,
            &raw mut in_size,
            &raw mut max,
        )
    } != 0;
    RawFacts {
        is_pipe,
        server_end: info_ok && flags & PIPE_SERVER_END != 0,
        message: info_ok && flags & PIPE_TYPE_MESSAGE != 0,
        max_instances: if info_ok { max } else { 0 },
        name: pipe_name(h),
        handle_count: handle_count(h),
    }
}

/// `GetFileInformationByHandleEx(FileNameInfo)`: the name relative to
/// the pipe device.
fn pipe_name(h: &OwnedHandle) -> Option<String> {
    let mut buf = [0u32; 260];
    let len = u32::try_from(size_of::<[u32; 260]>()).unwrap_or(0);
    // SAFETY: valid handle; the buffer is `len` bytes and 4-aligned, as
    // FILE_NAME_INFO needs.
    let ok =
        unsafe { GetFileInformationByHandleEx(raw(h), FileNameInfo, buf.as_mut_ptr().cast(), len) };
    if ok == 0 {
        return None;
    }
    // SAFETY: on success the buffer holds a FILE_NAME_INFO.
    let info = unsafe { &*buf.as_ptr().cast::<FILE_NAME_INFO>() };
    let units = usize::try_from(info.FileNameLength / 2).unwrap_or(0);
    let cap = (size_of::<[u32; 260]>() - 4) / 2;
    if units > cap {
        return None;
    }
    // SAFETY: FileName holds `units` UTF-16 units inside `buf` (checked
    // against its size above).
    let name = unsafe { core::slice::from_raw_parts(info.FileName.as_ptr(), units) };
    Some(String::from_utf16_lossy(name))
}

/// `NtQueryObject` basic information: handles to the object in every
/// process (probe W1); 0 if the query fails.
fn handle_count(h: &OwnedHandle) -> u32 {
    let mut info = ObjectBasicInformation::default();
    let mut ret = 0u32;
    // SAFETY: valid handle; the buffer is the 56-byte struct the class
    // fills; valid out-pointer.
    let st = unsafe {
        NtQueryObject(
            raw(h),
            OBJECT_BASIC_INFORMATION,
            (&raw mut info).cast(),
            56,
            &raw mut ret,
        )
    };
    if st == STATUS_SUCCESS {
        info.handle_count
    } else {
        0
    }
}

/// A process this one may duplicate handles into and out of (the
/// broker's handle to a child, with `PROCESS_DUP_HANDLE`).
#[derive(Debug)]
pub(crate) struct Process(pub(crate) OwnedHandle);

/// Pulls `v` out of `p`'s table into this process (`DUPLICATE_CLOSE_SOURCE`
/// when `close_source`: the value is gone from `p` whatever the
/// outcome).
#[expect(
    clippy::disallowed_methods,
    reason = "SEC-4 (d): this is the Windows IPC backend"
)]
pub(crate) fn pull(p: &Process, v: u64, close_source: bool) -> Result<OwnedHandle, u32> {
    let addr = usize::try_from(v).map_err(|_e| 0u32)?;
    let src: HANDLE = core::ptr::without_provenance_mut(addr);
    let mut out: HANDLE = null_mut();
    let opts = DUPLICATE_SAME_ACCESS
        | if close_source {
            DUPLICATE_CLOSE_SOURCE
        } else {
            0
        };
    let (from, me) = (raw(&p.0), current_process());
    // `from` has PROCESS_DUP_HANDLE; `src` is only a value in its table,
    // never used here; the pseudo-handle `me` needs no close.
    // SAFETY: valid process handles and out-pointer, as above.
    let ok = unsafe { DuplicateHandle(from, src, me, &raw mut out, 0, 0, opts) };
    if ok == 0 {
        return Err(last_error());
    }
    // SAFETY: a new handle in this process's table, owned here.
    Ok(unsafe { OwnedHandle::from_raw_handle(out.cast()) })
}

/// Puts a copy of `h` into `p`'s table with [`MOVE_ACCESS`]; returns its
/// value there. The caller closes its own copy.
#[expect(
    clippy::disallowed_methods,
    reason = "SEC-4 (d): this is the Windows IPC backend"
)]
pub(crate) fn give(p: &Process, h: &OwnedHandle) -> Result<u64, u32> {
    let mut out: HANDLE = null_mut();
    // SAFETY: valid handles; an explicit, narrowed access mask, not
    // inheritable; valid out-pointer.
    let ok = unsafe {
        DuplicateHandle(
            current_process(),
            raw(h),
            raw(&p.0),
            &raw mut out,
            MOVE_ACCESS,
            0,
            0,
        )
    };
    if ok == 0 {
        return Err(last_error());
    }
    Ok(u64::try_from(out.addr()).unwrap_or(0))
}

/// A second handle to the same object, same access, not inheritable.
///
/// # Errors
/// The OS error.
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
#[expect(
    clippy::disallowed_methods,
    reason = "SEC-4 (d): this is the Windows IPC backend"
)]
pub(crate) fn duplicate(h: &OwnedHandle) -> io::Result<OwnedHandle> {
    let mut out: HANDLE = null_mut();
    // SAFETY: valid handle; same access, not inheritable; valid
    // out-pointer.
    let ok = unsafe {
        DuplicateHandle(
            current_process(),
            raw(h),
            current_process(),
            &raw mut out,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a new handle owned here.
    Ok(unsafe { OwnedHandle::from_raw_handle(out.cast()) })
}

/// A SID as its string form.
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
pub(crate) fn sid_string(sid: &crate::sandbox::windows::AppContainerSid) -> Option<String> {
    let mut s: *mut u16 = null_mut();
    // SAFETY: a valid SID owned by `sid`; valid out-pointer.
    if unsafe { ConvertSidToStringSidW(sid.as_psid(), &raw mut s) } == 0 {
        return None;
    }
    let out = wide_to_string(s);
    // SAFETY: allocated by the call for us to free with LocalFree.
    unsafe { LocalFree(s.cast()) };
    Some(out)
}

/// A NUL-terminated UTF-16 string a call returned.
fn wide_to_string(s: *const u16) -> String {
    let mut n = 0usize;
    loop {
        // SAFETY: the caller passes a NUL-terminated string; `n` is at
        // most the index of its terminator, so the pointer stays inside.
        let at = unsafe { s.add(n) };
        // SAFETY: as above: a unit of the string or its terminator.
        let unit = unsafe { *at };
        if unit == 0 {
            break;
        }
        n = n.saturating_add(1);
    }
    // SAFETY: `n` UTF-16 units precede the terminator.
    String::from_utf16_lossy(unsafe { core::slice::from_raw_parts(s, n) })
}

/// An inheritable copy of `h` for a child's handle list.
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
#[expect(
    clippy::disallowed_methods,
    reason = "SEC-4 (d): this is the Windows IPC backend"
)]
pub(crate) fn inheritable(h: &OwnedHandle) -> io::Result<OwnedHandle> {
    let mut out: HANDLE = null_mut();
    // SAFETY: valid handle; same access, inheritable; valid out-pointer.
    let ok = unsafe {
        DuplicateHandle(
            current_process(),
            raw(h),
            current_process(),
            &raw mut out,
            0,
            1,
            DUPLICATE_SAME_ACCESS,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a new handle owned here.
    Ok(unsafe { OwnedHandle::from_raw_handle(out.cast()) })
}

/// This process's ID.
pub(crate) fn process_id() -> u32 {
    // SAFETY: no arguments.
    unsafe { GetCurrentProcessId() }
}

/// Cancels every operation this process has started on `h`; each still
/// completes, with [`ERROR_OPERATION_ABORTED`] unless it finished first.
pub(crate) fn cancel(h: &OwnedHandle) {
    // SAFETY: valid handle; a null OVERLAPPED cancels all of them.
    let _ = unsafe { CancelIoEx(raw(h), null()) };
}

/// One finished operation.
#[derive(Debug)]
pub(crate) struct Completion {
    /// The token given when it started.
    pub(crate) token: u64,
    /// A write (else a read).
    pub(crate) write: bool,
    /// 0, or the Win32 error.
    pub(crate) error: u32,
    /// The buffer: for a read, truncated to the bytes read.
    pub(crate) buf: Vec<u8>,
}

/// A started operation. The OVERLAPPED comes first, so the pointer the
/// completion routine gets is the operation's.
#[repr(C)]
struct Op {
    ov: OVERLAPPED,
    token: u64,
    write: bool,
    buf: Vec<u8>,
    /// Keeps the handle open while the operation runs.
    handle: Arc<OwnedHandle>,
}

thread_local! {
    /// Completions the routines ran since the IO thread last took them.
    static DONE: RefCell<Vec<Completion>> = const { RefCell::new(Vec::new()) };
}

/// The completion routine of every operation.
unsafe extern "system" fn on_done(error: u32, bytes: u32, ov: *mut OVERLAPPED) {
    // Every OVERLAPPED this module passes to ReadFileEx or WriteFileEx
    // is the first field of an `Op` leaked with Box::into_raw by `start`.
    // SAFETY: Windows runs this routine exactly once per operation that
    // started, so that box is reclaimed exactly once.
    let op = unsafe { Box::from_raw(ov.cast::<Op>()) };
    let Op {
        token,
        write,
        mut buf,
        handle,
        ..
    } = *op;
    if !write {
        buf.truncate(usize::try_from(bytes).unwrap_or(0));
    }
    DONE.with(|d| {
        d.borrow_mut().push(Completion {
            token,
            write,
            error,
            buf,
        });
    });
    // The handle reference drops here: the operation no longer uses it.
    drop(handle);
}

/// Starts a read of up to `cap` bytes, or a write of `buf`, on `h`.
/// Must run on the IO thread, which takes the completion with
/// [`take_completions`] after an alertable wait.
///
/// # Errors
/// The Win32 error when the operation did not start (no completion will
/// come); the buffer is returned with it.
fn start(
    h: &Arc<OwnedHandle>,
    write: bool,
    buf: Vec<u8>,
    token: u64,
) -> Result<(), (u32, Vec<u8>)> {
    let Ok(len) = u32::try_from(buf.len()) else {
        return Err((0, buf));
    };
    let op = Box::new(Op {
        // SAFETY: a zeroed OVERLAPPED is its documented initial state.
        ov: unsafe { zeroed() },
        token,
        write,
        buf,
        handle: Arc::clone(h),
    });
    let op = Box::into_raw(op);
    // SAFETY: `op` is a live allocation leaked just above; its buffer
    // does not move while the operation runs (the Vec's heap block is
    // owned by the op and never reallocated).
    let data = unsafe { (*op).buf.as_mut_ptr() };
    let ok = if write {
        // SAFETY: valid handle opened for overlapped I/O; `data` holds
        // `len` bytes and, with the OVERLAPPED, lives until the
        // completion routine reclaims `op`.
        unsafe { WriteFileEx(raw(h), data, len, op.cast(), Some(on_done)) }
    } else {
        // SAFETY: as for the write.
        unsafe { ReadFileEx(raw(h), data, len, op.cast(), Some(on_done)) }
    };
    if ok == 0 {
        let err = last_error();
        // SAFETY: the call failed, so no completion routine will run:
        // the box is still only ours.
        let op = unsafe { Box::from_raw(op) };
        return Err((err, op.buf));
    }
    Ok(())
}

/// Starts a read of one message of up to `cap` bytes.
///
/// # Errors
/// The Win32 error when it did not start.
pub(crate) fn start_read(h: &Arc<OwnedHandle>, cap: usize, token: u64) -> Result<(), u32> {
    start(h, false, vec![0u8; cap], token).map_err(|(e, _b)| e)
}

/// Starts writing one message.
///
/// # Errors
/// The Win32 error and the message when it did not start.
pub(crate) fn start_write(
    h: &Arc<OwnedHandle>,
    msg: Vec<u8>,
    token: u64,
) -> Result<(), (u32, Vec<u8>)> {
    start(h, true, msg, token)
}

/// The completions the IO thread's last alertable wait ran.
pub(crate) fn take_completions() -> Vec<Completion> {
    DONE.with(|d| core::mem::take(&mut *d.borrow_mut()))
}

/// Waits alertably: returns after one or more APCs ran (completions or
/// wakes), or after `timeout_ms` (none: no timeout).
pub(crate) fn wait(timeout_ms: Option<u32>) {
    // SAFETY: no pointers; alertable, so queued APCs run here.
    let _ = unsafe { SleepEx(timeout_ms.unwrap_or(u32::MAX), 1) };
}

/// A real handle to a thread, to queue APCs to it.
#[derive(Debug)]
pub(crate) struct ThreadRef(OwnedHandle);

/// The calling thread, as a handle other threads can wake.
///
/// # Errors
/// The OS error.
#[expect(
    clippy::disallowed_methods,
    reason = "SEC-4 (d): this is the Windows IPC backend"
)]
pub(crate) fn current_thread() -> io::Result<ThreadRef> {
    let mut out: HANDLE = null_mut();
    // SAFETY: the pseudo-handles need no close; same access, not
    // inheritable; valid out-pointer.
    let ok = unsafe {
        DuplicateHandle(
            current_process(),
            current_thread_pseudo(),
            current_process(),
            &raw mut out,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a new handle owned here.
    Ok(ThreadRef(unsafe {
        OwnedHandle::from_raw_handle(out.cast())
    }))
}

/// The wake APC: does nothing; running it ends the alertable wait.
unsafe extern "system" fn wake_apc(_data: usize) {}

/// Wakes the thread from its alertable wait.
pub(crate) fn wake(t: &ThreadRef) -> bool {
    // SAFETY: a valid thread handle; the routine is a no-op.
    unsafe { QueueUserAPC(Some(wake_apc), raw(&t.0), 0) != 0 }
}

/// Reads one message from `h` synchronously, waiting at most
/// `timeout_ms` (for the `Hello` a child reads before its IO thread
/// starts).
///
/// # Errors
/// The OS error, or `TimedOut`.
pub(crate) fn read_blocking(h: &OwnedHandle, cap: usize, timeout_ms: u32) -> io::Result<Vec<u8>> {
    let mut buf = vec![0u8; cap];
    let len = u32::try_from(cap).map_err(|_e| io::Error::other("buffer too large"))?;
    // SAFETY: manual-reset, unsignalled, unnamed event.
    let event = unsafe { CreateEventW(null(), 1, 0, null()) };
    if event.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a new event owned here.
    let event = unsafe { OwnedHandle::from_raw_handle(event.cast()) };
    // SAFETY: a zeroed OVERLAPPED is its documented initial state.
    let mut ov: OVERLAPPED = unsafe { zeroed() };
    ov.hEvent = raw(&event);
    let mut n = 0u32;
    // SAFETY: valid handle; `buf` and `ov` outlive the operation: this
    // function waits for it to finish or cancels it and waits again.
    let ok = unsafe { ReadFile(raw(h), buf.as_mut_ptr(), len, &raw mut n, &raw mut ov) };
    if ok == 0 && last_error() != ERROR_IO_PENDING {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a valid event handle.
    let w = unsafe { WaitForSingleObject(raw(&event), timeout_ms) };
    if w != WAIT_OBJECT_0 {
        cancel(h);
    }
    // SAFETY: valid handle and the OVERLAPPED of the read above; wait
    // for it to finish (it has, or was cancelled).
    let ok = unsafe { GetOverlappedResult(raw(h), &raw const ov, &raw mut n, 1) };
    if ok == 0 {
        return Err(if w == WAIT_OBJECT_0 {
            io::Error::last_os_error()
        } else {
            io::ErrorKind::TimedOut.into()
        });
    }
    buf.truncate(usize::try_from(n).unwrap_or(0));
    Ok(buf)
}

/// Starting sandboxed children: for the launcher (phase E) and the
/// tests. The child runs in its own AppContainer (`container`), under
/// the lockdown token, in a kill-on-close job assigned before it runs,
/// with exactly one inherited handle: `inherit`. It starts suspended;
/// the caller duplicates its launch ends in, then [`resume`]s it.
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
pub(crate) mod spawn {
    use super::{Process, last_error, raw, wide};
    use core::mem::{size_of, zeroed};
    use core::ptr::{null, null_mut};
    use std::io;
    use std::os::windows::io::{FromRawHandle, OwnedHandle};

    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Security::SECURITY_CAPABILITIES;
    use windows_sys::Win32::System::Threading::{
        CREATE_NO_WINDOW, CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, CreateProcessAsUserW,
        CreateProcessW, DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT,
        InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
        PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES,
        PROCESS_INFORMATION, ResumeThread, STARTUPINFOEXW, TerminateProcess,
        UpdateProcThreadAttribute, WaitForSingleObject,
    };

    use crate::sandbox::windows::{AppContainerSid, OwnedHandle as SbHandle, assign_job};

    /// A started, suspended child.
    #[derive(Debug)]
    pub(crate) struct Child {
        /// Its process handle (all access: the creator's).
        pub(crate) process: Process,
        /// Its primary thread, suspended.
        thread: OwnedHandle,
        /// Its process ID.
        pub(crate) pid: u32,
    }

    /// What to start.
    pub(crate) struct Spec<'a> {
        /// The executable.
        pub(crate) exe: &'a std::path::Path,
        /// The command line after the executable's quoted path.
        pub(crate) args: &'a str,
        /// Variables added to this process's environment.
        pub(crate) env: &'a [(String, String)],
        /// The one handle the child inherits.
        pub(crate) inherit: &'a OwnedHandle,
        /// Sandboxed: container, lockdown token and job. `None` starts
        /// the child plainly (tests of the transport alone).
        pub(crate) sandbox: Option<(&'a AppContainerSid, &'a SbHandle, &'a SbHandle)>,
    }

    fn env_block(extra: &[(String, String)]) -> Vec<u16> {
        let mut vars: Vec<(String, String)> = std::env::vars()
            .filter(|(k, _)| !extra.iter().any(|(e, _)| e.eq_ignore_ascii_case(k)))
            .collect();
        vars.extend(extra.iter().cloned());
        vars.sort_by_key(|(k, _)| k.to_uppercase());
        let mut out = Vec::new();
        for (k, v) in vars {
            out.extend(format!("{k}={v}").encode_utf16());
            out.push(0);
        }
        out.push(0);
        out
    }

    /// Starts the child, suspended.
    ///
    /// # Errors
    /// The OS error.
    pub(crate) fn start(spec: &Spec<'_>) -> io::Result<Child> {
        let n_attr = if spec.sandbox.is_some() { 2 } else { 1 };
        let mut size = 0usize;
        // SAFETY: the documented size query: null list, out-pointer.
        unsafe { InitializeProcThreadAttributeList(null_mut(), n_attr, 0, &raw mut size) };
        let mut mem = vec![0u64; size.div_ceil(8)];
        let attrs: LPPROC_THREAD_ATTRIBUTE_LIST = mem.as_mut_ptr().cast();
        // SAFETY: `mem` holds `size` bytes, 8-aligned.
        if unsafe { InitializeProcThreadAttributeList(attrs, n_attr, 0, &raw mut size) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let result = start_with(spec, attrs);
        // SAFETY: initialised above and no longer used.
        unsafe { DeleteProcThreadAttributeList(attrs) };
        result
    }

    fn start_with(spec: &Spec<'_>, attrs: LPPROC_THREAD_ATTRIBUTE_LIST) -> io::Result<Child> {
        // SAFETY: a zeroed struct is its documented initial state.
        let mut caps: SECURITY_CAPABILITIES = unsafe { zeroed() };
        if let Some((container, _, _)) = spec.sandbox {
            caps.AppContainerSid = container.as_psid();
            // SAFETY: `caps` lives until the process is created; its
            // size is the struct's.
            let ok = unsafe {
                UpdateProcThreadAttribute(
                    attrs,
                    0,
                    PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
                    (&raw mut caps).cast(),
                    size_of::<SECURITY_CAPABILITIES>(),
                    null_mut(),
                    null(),
                )
            };
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
        }
        let mut list: [HANDLE; 1] = [raw(spec.inherit)];
        // SAFETY: `list` holds one valid, inheritable handle and lives
        // until the process is created.
        let ok = unsafe {
            UpdateProcThreadAttribute(
                attrs,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                list.as_mut_ptr().cast(),
                size_of::<HANDLE>(),
                null_mut(),
                null(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        let exe_w = wide(&spec.exe.to_string_lossy());
        let mut cmd = wide(&format!("\"{}\" {}", spec.exe.display(), spec.args));
        let env = env_block(spec.env);
        // SAFETY: a zeroed struct is its documented initial state.
        let mut si: STARTUPINFOEXW = unsafe { zeroed() };
        si.StartupInfo.cb = u32::try_from(size_of::<STARTUPINFOEXW>()).unwrap_or(0);
        si.lpAttributeList = attrs;
        // SAFETY: a zeroed struct is its documented initial state.
        let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
        // No console: creating one in the sandbox fails in the loader
        // (FP-M0 decision log, 23 October 2026).
        let flags = EXTENDED_STARTUPINFO_PRESENT
            | CREATE_SUSPENDED
            | CREATE_NO_WINDOW
            | CREATE_UNICODE_ENVIRONMENT;
        let ok = match spec.sandbox {
            // Inheritance is on and limited to `list` by the attribute.
            // SAFETY: valid token, exe path, mutable command line,
            // environment block and startup info, all live for the call.
            Some((_, token, _)) => unsafe {
                CreateProcessAsUserW(
                    token.as_handle(),
                    exe_w.as_ptr(),
                    cmd.as_mut_ptr(),
                    null(),
                    null(),
                    1,
                    flags,
                    env.as_ptr().cast(),
                    null(),
                    &raw const si.StartupInfo,
                    &raw mut pi,
                )
            },
            // SAFETY: as above, without a token.
            None => unsafe {
                CreateProcessW(
                    exe_w.as_ptr(),
                    cmd.as_mut_ptr(),
                    null(),
                    null(),
                    1,
                    flags,
                    env.as_ptr().cast(),
                    null(),
                    &raw const si.StartupInfo,
                    &raw mut pi,
                )
            },
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: new handles owned here.
        let process = unsafe { OwnedHandle::from_raw_handle(pi.hProcess.cast()) };
        // SAFETY: as above.
        let thread = unsafe { OwnedHandle::from_raw_handle(pi.hThread.cast()) };
        if let Some((_, _, job)) = spec.sandbox {
            // SAFETY: the process handle is valid for the call.
            if let Err(e) = unsafe { assign_job(job, raw(&process)) } {
                // SAFETY: valid handle; the child never ran.
                unsafe { TerminateProcess(raw(&process), 1) };
                return Err(io::Error::other(e));
            }
        }
        Ok(Child {
            process: Process(process),
            thread,
            pid: pi.dwProcessId,
        })
    }

    /// Lets the child run.
    ///
    /// # Errors
    /// The OS error.
    pub(crate) fn resume(c: &Child) -> io::Result<()> {
        // SAFETY: the child's suspended primary thread.
        if unsafe { ResumeThread(raw(&c.thread)) } == u32::MAX {
            return Err(io::Error::from_raw_os_error(
                i32::try_from(last_error()).unwrap_or(0),
            ));
        }
        Ok(())
    }

    /// A second handle to the child's process, for the broker's table.
    ///
    /// # Errors
    /// The OS error.
    pub(crate) fn process_handle(c: &Child) -> io::Result<Process> {
        super::duplicate(&c.process.0).map(Process)
    }

    /// Unregisters AppContainer profile `name` (each child gets its own;
    /// the profile persists per user until removed).
    pub(crate) fn delete_container(name: &str) {
        use windows_sys::Win32::Security::Isolation::DeleteAppContainerProfile;
        let w = super::wide(name);
        // SAFETY: a NUL-terminated name valid for the call.
        let _ = unsafe { DeleteAppContainerProfile(w.as_ptr()) };
    }

    /// Ends the child.
    pub(crate) fn kill(c: &Child) {
        // SAFETY: valid process handle.
        unsafe { TerminateProcess(raw(&c.process.0), 1) };
    }

    /// Waits for the child to exit; returns its exit code.
    pub(crate) fn wait(c: &Child, timeout_ms: u32) -> Option<u32> {
        use windows_sys::Win32::System::Threading::GetExitCodeProcess;
        // SAFETY: valid process handle.
        if unsafe { WaitForSingleObject(raw(&c.process.0), timeout_ms) } != 0 {
            return None;
        }
        let mut code = 0u32;
        // SAFETY: valid process handle and out-pointer.
        (unsafe { GetExitCodeProcess(raw(&c.process.0), &raw mut code) } != 0).then_some(code)
    }
}
