//! The Windows calls behind the launch path: holding the build's files
//! open so nothing can replace or delete them (`CreateFileW` without
//! `FILE_SHARE_DELETE` or `FILE_SHARE_WRITE`), and ending a child's job
//! (`TerminateJobObject`). Declarations from the Learn pages for those
//! functions; no other project's code is a source.
//!
//! The launch path's only `unsafe` on Windows (CS-TY-13,
//! `unsafe-inventory.toml`, entry `launch::windows::sys`).
#![expect(
    unsafe_code,
    reason = "G5: launch::windows::sys (CreateFileW, TerminateJobObject)"
)]

use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::Path;

use windows_sys::Win32::Foundation::{GENERIC_READ, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ, OPEN_EXISTING,
};

fn wide(p: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    p.as_os_str().encode_wide().chain(Some(0)).collect()
}

/// Opens `path` (a file, or with `dir` a directory) for reading, shared
/// for reading only: while the handle lives, nobody can write, rename or
/// delete it (review M9; DU 13.1).
pub(crate) fn hold(path: &Path, dir: bool) -> io::Result<OwnedHandle> {
    let w = wide(path);
    let flags = if dir { FILE_FLAG_BACKUP_SEMANTICS } else { 0 };
    // SAFETY: a NUL-terminated path, no security attributes, no template.
    let h = unsafe {
        CreateFileW(
            w.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ,
            core::ptr::null(),
            OPEN_EXISTING,
            flags,
            core::ptr::null_mut(),
        )
    };
    if h == INVALID_HANDLE_VALUE || h.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a new handle owned here.
    Ok(unsafe { OwnedHandle::from_raw_handle(h) })
}

/// Ends every process in `job` (`TerminateJobObject`).
pub(crate) fn terminate_job(job: &crate::sandbox::windows::OwnedHandle) {
    use windows_sys::Win32::System::JobObjects::TerminateJobObject;
    // SAFETY: a live job handle.
    let _ = unsafe { TerminateJobObject(job.as_handle(), 1) };
}

/// When process `pid` started (its creation time, in 100 ns units since
/// 1601, `GetProcessTimes`), if it is running now. `None` when no process
/// has that id, it has exited (`GetExitCodeProcess` is not
/// `STILL_ACTIVE`), or it cannot be queried.
pub(crate) fn running_since(pid: u32) -> Option<u64> {
    use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, STILL_ACTIVE};
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    // SAFETY: plain arguments; a null result is checked.
    let h = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if h.is_null() {
        return None;
    }
    let zero = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let (mut created, mut exited, mut kernel, mut user) = (zero, zero, zero, zero);
    let mut code = 0u32;
    // SAFETY: a live process handle and four valid out-pointers.
    let times = unsafe {
        GetProcessTimes(
            h,
            &raw mut created,
            &raw mut exited,
            &raw mut kernel,
            &raw mut user,
        )
    };
    // SAFETY: a live process handle and a valid out-pointer.
    let alive = unsafe { GetExitCodeProcess(h, &raw mut code) } != 0
        && i32::try_from(code).is_ok_and(|c| c == STILL_ACTIVE);
    // SAFETY: the handle opened above, closed once.
    unsafe { CloseHandle(h) };
    (times != 0 && alive)
        .then(|| (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
}

/// Keeps `AsRawHandle` in use for the held handles' type.
#[expect(dead_code, reason = "documents the held handle's raw form")]
fn raw(h: &OwnedHandle) -> *mut core::ffi::c_void {
    h.as_raw_handle()
}

/// FP-27's phase timestamps on Windows (M1 phase G), for measurement
/// builds only (`--features measure-launch`; a production build has none
/// of this). Every stamp is `QueryPerformanceCounter`, one system-wide
/// counter, so the broker's and the child's stamps compare directly.
///
/// - The broker stamps each launch step on the launching thread.
/// - The child stamps its own start, then publishes the stamps as its
///   main thread's description (`SetThreadDescription`); the broker reads
///   them through the thread handle it already holds
///   (`GetThreadDescription`). No handle, pipe or message is added to the
///   launch being measured.
#[cfg(feature = "measure-launch")]
pub mod timing {
    use core::cell::RefCell;
    use std::os::windows::io::{AsRawHandle, OwnedHandle};
    use std::sync::{Mutex, PoisonError};

    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::System::Performance::{
        QueryPerformanceCounter, QueryPerformanceFrequency,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentThread, GetThreadDescription, SetThreadDescription,
    };

    /// The child's description prefix.
    const PREFIX: &str = "izcal-launch-timing:";

    thread_local! {
        static MARKS: RefCell<Vec<(&'static str, i64)>> = const { RefCell::new(Vec::new()) };
    }

    /// One launch: its kind's token, the broker's stamps and a handle to
    /// the child's main thread.
    struct Record {
        kind: &'static str,
        pid: u32,
        broker: Vec<(&'static str, i64)>,
        thread: OwnedHandle,
    }

    static RECORDS: Mutex<Vec<Record>> = Mutex::new(Vec::new());

    fn qpc() -> i64 {
        let mut v = 0i64;
        // SAFETY: a valid out-pointer; cannot fail on Windows XP or later.
        unsafe { QueryPerformanceCounter(&raw mut v) };
        v
    }

    /// Stamps `phase` on this thread.
    pub(crate) fn mark(phase: &'static str) {
        let t = qpc();
        MARKS.with(|m| m.borrow_mut().push((phase, t)));
    }

    /// Takes this thread's stamps.
    pub(crate) fn take() -> Vec<(&'static str, i64)> {
        MARKS.with(|m| core::mem::take(&mut *m.borrow_mut()))
    }

    /// The broker: files this thread's stamps for the child `pid`.
    pub(crate) fn record(kind: &'static str, pid: u32, thread: OwnedHandle) {
        let broker = take();
        RECORDS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Record {
                kind,
                pid,
                broker,
                thread,
            });
    }

    /// The child: publishes this thread's stamps as its description.
    pub(crate) fn publish() {
        let mut s = String::from(PREFIX);
        for (p, t) in take() {
            s.push_str(&format!("{p}={t};"));
        }
        let w: Vec<u16> = s.encode_utf16().chain(Some(0)).collect();
        // SAFETY: takes nothing; returns the current-thread pseudo-handle.
        let me = unsafe { GetCurrentThread() };
        // SAFETY: the pseudo-handle and a NUL-terminated string valid for
        // the call.
        let _ = unsafe { SetThreadDescription(me, w.as_ptr()) };
    }

    fn description(thread: &OwnedHandle) -> Option<String> {
        let mut p: *mut u16 = core::ptr::null_mut();
        // SAFETY: a live thread handle with query access; valid
        // out-pointer.
        let hr = unsafe { GetThreadDescription(thread.as_raw_handle(), &raw mut p) };
        if hr < 0 || p.is_null() {
            return None;
        }
        let mut n = 0usize;
        // SAFETY: the call returned a NUL-terminated string at `p`, so
        // every index up to its NUL is readable.
        while unsafe { p.wrapping_add(n).read() } != 0 {
            n = n.saturating_add(1);
        }
        // SAFETY: `n` u16s are readable at `p`, as counted above.
        let s = String::from_utf16_lossy(unsafe { core::slice::from_raw_parts(p, n) });
        // SAFETY: the string was allocated by the call; LocalFree is its
        // documented release.
        unsafe { LocalFree(p.cast()) };
        Some(s)
    }

    /// Every launch so far, one row per phase: `kind`, `pid`, `side`
    /// (`broker` or `child`), `phase`, milliseconds since the broker's
    /// first stamp for that launch. Reads each child's published stamps
    /// now, so call it after the children are `ready`.
    #[must_use]
    pub fn report() -> Vec<String> {
        let mut f = 0i64;
        // SAFETY: a valid out-pointer.
        unsafe { QueryPerformanceFrequency(&raw mut f) };
        #[expect(clippy::cast_precision_loss, reason = "milliseconds for a TSV")]
        let ms = |t: i64, base: i64| (t.saturating_sub(base)) as f64 * 1e3 / f.max(1) as f64;
        let recs = RECORDS.lock().unwrap_or_else(PoisonError::into_inner);
        let mut out = Vec::new();
        for r in recs.iter() {
            let Some(&(_, base)) = r.broker.first() else {
                continue;
            };
            for &(p, t) in &r.broker {
                out.push(format!(
                    "{}\t{}\tbroker\t{p}\t{:.3}",
                    r.kind,
                    r.pid,
                    ms(t, base)
                ));
            }
            let d = description(&r.thread).unwrap_or_default();
            for kv in d.strip_prefix(PREFIX).unwrap_or("").split(';') {
                if let Some((p, t)) = kv.split_once('=')
                    && let Ok(t) = t.parse::<i64>()
                {
                    out.push(format!(
                        "{}\t{}\tchild\t{p}\t{:.3}",
                        r.kind,
                        r.pid,
                        ms(t, base)
                    ));
                }
            }
        }
        out
    }

    /// Forgets every record (between runs).
    pub fn clear() {
        RECORDS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }
}
