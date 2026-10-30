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

/// Keeps `AsRawHandle` in use for the held handles' type.
#[expect(dead_code, reason = "documents the held handle's raw form")]
fn raw(h: &OwnedHandle) -> *mut core::ffi::c_void {
    h.as_raw_handle()
}
