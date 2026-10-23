//! The Windows sandbox backend, first leg: **untrusted integrity,
//! lockdown token, own job** (security scope 6.7's utility row, the
//! first three of its mitigations). The remaining mitigations of the
//! full utility tier — AppContainer without capabilities, win32k
//! lockdown, ACG and CIG/CET — are the recorded follow-up and are
//! reported as NOT applied: a tier below the process type's floor is a
//! visible reduced mode, never silent (SEC-8).
//!
//! The token is created by the supervisor before the child starts, so
//! the empty process is restricted from birth — no post-start lockdown
//! window. The child probes its own tier (the integrity label from its
//! token, plus a file-open that must fail) and reports it.
//!
//! The whole module is the crate's inventoried `unsafe` (CS-TY-13,
//! `unsafe-inventory.toml`, entry `sandbox::windows`): the Win32 FFI.
#![expect(unsafe_code, reason = "G5: sandbox::windows (Win32 token APIs)")]

use std::mem::size_of;
use std::ptr;

use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ACCESS_DENIED, GENERIC_READ, HANDLE};
use windows_sys::Win32::Security::{
    CreateRestrictedToken, CreateWellKnownSid, DuplicateTokenEx, GetTokenInformation,
    SECURITY_MAX_SID_SIZE, SID_AND_ATTRIBUTES, SecurityImpersonation, SetTokenInformation,
    TOKEN_ALL_ACCESS, TOKEN_DUPLICATE, TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TokenIntegrityLevel,
    TokenPrimary, WinUntrustedLabelSid,
};
use windows_sys::Win32::Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, OPEN_EXISTING};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// `SE_GROUP_INTEGRITY` from the Win32 headers (the integrity attribute
/// bit on a mandatory-label SID).
const SE_GROUP_INTEGRITY: u32 = 0x40;

/// `DISABLE_MAX_PRIVILEGE` and `SANDBOX_INERT` (the documented lockdown
/// flag combination for `CreateRestrictedToken`).
const DISABLE_MAX_PRIVILEGE: u32 = 0x1;
const SANDBOX_INERT: u32 = 0x2;

/// An owned HANDLE that closes itself (RAII for the token and job).
pub struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and valid until
        // this close.
        unsafe { CloseHandle(self.0) };
    }
}

impl OwnedHandle {
    /// The raw handle for the Win32 calls.
    pub const fn as_handle(&self) -> HANDLE {
        self.0
    }
}

/// Builds the restricted, untrusted-integrity token for the child.
///
/// `DISABLE_MAX_PRIVILEGE` strips every privilege; `SANDBOX_INERT`
/// makes the token a fixed sandbox token (no further privilege
/// restoration); the Untrusted label SID (S-1-16-0) sets the lowest
/// integrity level. The returned token is what `CreateProcessAsUserW`
/// starts the child with.
pub fn restricted_token() -> Result<OwnedHandle, String> {
    let mut current: HANDLE = ptr::null_mut();
    // SAFETY: the process pseudo-handle; needs no close.
    let process = unsafe { GetCurrentProcess() };
    // SAFETY: valid out-pointer; the pseudo-handle needs no close.
    let rc = unsafe { OpenProcessToken(process, TOKEN_DUPLICATE | TOKEN_QUERY, &mut current) };
    if rc == 0 {
        return Err(format!("OpenProcessToken: rc={rc}"));
    }

    let mut dup: HANDLE = ptr::null_mut();
    // SAFETY: `current` is the process token; the duplicate is closed
    // on failure and owned on success.
    let rc = unsafe {
        DuplicateTokenEx(
            current,
            TOKEN_ALL_ACCESS,
            ptr::null(),
            SecurityImpersonation,
            TokenPrimary,
            &mut dup,
        )
    };
    // SAFETY: `current` is the handle from OpenProcessToken above.
    unsafe { CloseHandle(current) };
    if rc == 0 {
        return Err(format!("DuplicateTokenEx: rc={rc}"));
    }

    let mut restricted: HANDLE = ptr::null_mut();
    // SAFETY: the documented lockdown flags; the out-handle is closed
    // on the failure path.
    let rc = unsafe {
        CreateRestrictedToken(
            dup,
            DISABLE_MAX_PRIVILEGE | SANDBOX_INERT,
            0,
            ptr::null(),
            0,
            ptr::null(),
            0,
            ptr::null(),
            &mut restricted,
        )
    };
    // SAFETY: `dup` is the duplicated token above.
    unsafe { CloseHandle(dup) };
    if rc == 0 {
        return Err(format!("CreateRestrictedToken: rc={rc}"));
    }

    // The Untrusted label SID, then the mandatory label on the token.
    let mut sid_buf = [0u8; SECURITY_MAX_SID_SIZE as usize];
    let mut sid_len: u32 = sid_buf.len() as u32;
    // SAFETY: the SID buffer is sized per the constant; the API fills
    // it for WinUntrustedLabelSid.
    let rc = unsafe {
        CreateWellKnownSid(
            WinUntrustedLabelSid,
            ptr::null_mut(),
            sid_buf.as_mut_ptr() as *mut _,
            &mut sid_len,
        )
    };
    if rc == 0 {
        // SAFETY: `restricted` is the restricted token above.
        unsafe { CloseHandle(restricted) };
        return Err(format!("CreateWellKnownSid: rc={rc}"));
    }
    let label = TOKEN_MANDATORY_LABEL {
        Label: SID_AND_ATTRIBUTES {
            Sid: sid_buf.as_ptr() as *mut _,
            Attributes: SE_GROUP_INTEGRITY,
        },
    };
    // SAFETY: the label's Sid lives in `sid_buf` for the call's
    // duration; the size is the struct's.
    let rc = unsafe {
        SetTokenInformation(
            restricted,
            TokenIntegrityLevel,
            &label as *const _ as *const _,
            size_of::<TOKEN_MANDATORY_LABEL>() as u32,
        )
    };
    if rc == 0 {
        // SAFETY: `restricted` is the restricted token above.
        unsafe { CloseHandle(restricted) };
        return Err(format!("SetTokenInformation(integrity): rc={rc}"));
    }
    Ok(OwnedHandle(restricted))
}

/// Creates the child's job object with kill-on-close: if the
/// supervisor dies, the job (and its process) dies with it.
pub fn job_object() -> Result<OwnedHandle, String> {
    use windows_sys::Win32::System::JobObjects::{
        CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectExtendedLimitInformation, SetInformationJobObject,
    };

    // SAFETY: null attributes and name (default security, unnamed job).
    let job = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
    if job.is_null() {
        return Err("CreateJobObjectW failed".to_string());
    }
    // SAFETY: a zeroed struct is the documented initial state.
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    // SAFETY: `limits` is the documented struct for this information
    // class, and the size is the struct's.
    let rc = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };
    if rc == 0 {
        // SAFETY: the handle is ours and valid.
        unsafe { CloseHandle(job) };
        return Err("SetInformationJobObject failed".to_string());
    }
    Ok(OwnedHandle(job))
}

/// Puts the child process in the job.
///
/// # Safety
/// `process` must be a valid process handle for the call's duration.
pub unsafe fn assign_job(job: &OwnedHandle, process: HANDLE) -> Result<(), String> {
    use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;

    // SAFETY: both handles are valid for the call's duration (the
    // caller's contract).
    let rc = unsafe { AssignProcessToJobObject(job.as_handle(), process) };
    if rc == 0 {
        return Err("AssignProcessToJobObject failed".to_string());
    }
    Ok(())
}

/// The child's tier probe: reads its own integrity label and attempts
/// a file open that the Untrusted label must deny. Reports both — a
/// denial is data, never silence (SEC-8).
pub fn probe_line() -> String {
    // The integrity SID from the child's own token.
    let mut token: HANDLE = ptr::null_mut();
    // SAFETY: the process pseudo-handle; needs no close.
    let process = unsafe { GetCurrentProcess() };
    // SAFETY: valid out-pointer; the pseudo-handle needs no close.
    let rc = unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) };
    let integrity = if rc == 0 {
        "token-open-failed".to_string()
    } else {
        let mut buf = [0u8; SECURITY_MAX_SID_SIZE as usize];
        let mut len: u32 = 0;
        // SAFETY: the buffer is sized per the constant; the API writes
        // the label and its length.
        let ok = unsafe {
            GetTokenInformation(
                token,
                TokenIntegrityLevel,
                buf.as_mut_ptr() as *mut _,
                buf.len() as u32,
                &mut len,
            )
        };
        // SAFETY: `token` is the handle from OpenProcessToken above.
        unsafe { CloseHandle(token) };
        if ok == 0 {
            "integrity-query-failed".to_string()
        } else {
            // SAFETY: buf holds a TOKEN_MANDATORY_LABEL of `len` bytes
            // filled by the API above; the read is within bounds.
            let label = unsafe { &*(buf.as_ptr() as *const TOKEN_MANDATORY_LABEL) };
            format!("integrity-label-present={}", !label.Label.Sid.is_null())
        }
    };

    // The must-deny probe: opening a system file at Untrusted
    // integrity must fail with ACCESS_DENIED.
    let hosts = wide("C:\\Windows\\System32\\drivers\\etc\\hosts");
    // SAFETY: the wide path is NUL-terminated and valid for the call.
    let h = unsafe {
        CreateFileW(
            hosts.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ,
            ptr::null(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    };
    let file_verdict = if h == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        // SAFETY: GetLastError reads the thread's error slot only.
        let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        format!(
            "file-read={}",
            if err == ERROR_ACCESS_DENIED {
                "denied"
            } else {
                "unexpected"
            }
        )
    } else {
        // SAFETY: the handle is ours and valid.
        unsafe { CloseHandle(h) };
        "file-read=ALLOWED".to_string()
    };

    format!(
        "{integrity} {file_verdict} mitigations=untrusted+lockdown+job (appcontainer,win32k,acg:not-applied)"
    )
}

/// A UTF-16, NUL-terminated string for the FFI.
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
