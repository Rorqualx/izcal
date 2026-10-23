//! The Windows sandbox backend: **AppContainer without capabilities,
//! lockdown token, own job, Untrusted integrity** (security scope 6.7's
//! utility row, its first four mitigations). The remaining mitigations
//! of that row — win32k lockdown, ACG, CIG and CET — are the recorded
//! follow-up and are reported as NOT applied: a tier below the process
//! type's floor is a visible reduced mode, never silent (SEC-8).
//!
//! The supervisor starts the child with `CreateProcessAsUserW` under a
//! restricted version of its own token (every privilege but
//! SeChangeNotify removed), inside an AppContainer with no
//! capabilities. The AppContainer is what denies reads of the user's
//! files: they carry no ACE for the container or for ALL APPLICATION
//! PACKAGES. Windows starts every AppContainer process at Low
//! integrity; the child lowers itself to Untrusted first thing in
//! `main` ([`apply_empty_process_profile`]). All of it measured on
//! rig253, FP-M0 decision log, 23 October 2026.
//!
//! The whole module is the crate's inventoried `unsafe` (CS-TY-13,
//! `unsafe-inventory.toml`, entry `sandbox::windows`): the Win32 FFI.
#![expect(unsafe_code, reason = "G5: sandbox::windows (Win32 token APIs)")]

use std::mem::size_of;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ACCESS_DENIED, GENERIC_READ, GetLastError, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Security::Isolation::{
    CreateAppContainerProfile, DeriveAppContainerSidFromAppContainerName,
};
use windows_sys::Win32::Security::{
    CreateRestrictedToken, CreateWellKnownSid, FreeSid, GetSidSubAuthority,
    GetSidSubAuthorityCount, GetTokenInformation, PSID, SECURITY_MAX_SID_SIZE, SID_AND_ATTRIBUTES,
    SetTokenInformation, TOKEN_ADJUST_DEFAULT, TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE,
    TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TokenIntegrityLevel, TokenIsAppContainer, TokenPrivileges,
    WinUntrustedLabelSid,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// `SE_GROUP_INTEGRITY` from the Win32 headers (the integrity attribute
/// bit on a mandatory-label SID).
const SE_GROUP_INTEGRITY: u32 = 0x20;

/// `DISABLE_MAX_PRIVILEGE`: remove every privilege except
/// SeChangeNotifyPrivilege. (SANDBOX_INERT is deliberately not used: it
/// turns AppLocker and Software Restriction Policy checks *off*.)
const DISABLE_MAX_PRIVILEGE: u32 = 0x1;

/// `HRESULT_FROM_WIN32(ERROR_ALREADY_EXISTS)`, returned by
/// `CreateAppContainerProfile` when the profile is already registered.
const HR_ALREADY_EXISTS: i32 = 0x8007_00B7_u32 as i32;

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

/// An AppContainer SID, freed on drop.
pub struct AppContainerSid(PSID);

impl Drop for AppContainerSid {
    fn drop(&mut self) {
        // SAFETY: the SID came from CreateAppContainerProfile or
        // DeriveAppContainerSidFromAppContainerName, whose documented
        // release is FreeSid.
        unsafe { FreeSid(self.0) };
    }
}

impl AppContainerSid {
    /// The raw SID for the process-creation attribute.
    pub const fn as_psid(&self) -> PSID {
        self.0
    }
}

/// Returns the SID of the AppContainer `name`, registering its profile
/// on first use. A launch needs the registered profile: with only a
/// derived SID, process creation fails with ERROR_FILE_NOT_FOUND (2).
/// The profile is per user and persists, like an installed app's.
pub fn app_container(name: &str, display: &str) -> Result<AppContainerSid, String> {
    let name_w = wide(name);
    let display_w = wide(display);
    let mut sid: PSID = ptr::null_mut();
    // SAFETY: NUL-terminated strings valid for the call; no
    // capabilities (null array, count 0); valid out-pointer.
    let hr = unsafe {
        CreateAppContainerProfile(
            name_w.as_ptr(),
            display_w.as_ptr(),
            display_w.as_ptr(),
            ptr::null(),
            0,
            &mut sid,
        )
    };
    if hr == 0 {
        return Ok(AppContainerSid(sid));
    }
    if hr != HR_ALREADY_EXISTS {
        return Err(format!("CreateAppContainerProfile: hr={hr:#x}"));
    }
    // SAFETY: NUL-terminated name; valid out-pointer.
    let hr = unsafe { DeriveAppContainerSidFromAppContainerName(name_w.as_ptr(), &mut sid) };
    if hr != 0 {
        return Err(format!(
            "DeriveAppContainerSidFromAppContainerName: hr={hr:#x}"
        ));
    }
    Ok(AppContainerSid(sid))
}

/// Builds the lockdown token: a restricted version of this process's
/// own primary token with every privilege but SeChangeNotify removed.
///
/// It is made from the process token directly (no duplicate first):
/// `CreateProcessAsUserW` accepts a restricted version of the caller's
/// own token from a non-admin caller, while a duplicated token failed
/// with ERROR_PRIVILEGE_NOT_HELD (1314). The integrity level is not set
/// here: Windows starts an AppContainer process at Low whatever the
/// token says, and the child lowers itself to Untrusted.
pub fn restricted_token() -> Result<OwnedHandle, String> {
    let mut current: HANDLE = ptr::null_mut();
    // SAFETY: the process pseudo-handle; needs no close.
    let process = unsafe { GetCurrentProcess() };
    // SAFETY: valid out-pointer; the pseudo-handle needs no close.
    let rc = unsafe {
        OpenProcessToken(
            process,
            TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ASSIGN_PRIMARY,
            &mut current,
        )
    };
    if rc == 0 {
        // SAFETY: GetLastError reads the thread's error slot only.
        return Err(format!("OpenProcessToken: error={}", unsafe {
            GetLastError()
        }));
    }

    let mut restricted: HANDLE = ptr::null_mut();
    // SAFETY: `current` is the process token opened above with
    // TOKEN_DUPLICATE; no SIDs to disable, privileges to delete or SIDs
    // to restrict (null arrays, zero counts); valid out-pointer.
    let rc = unsafe {
        CreateRestrictedToken(
            current,
            DISABLE_MAX_PRIVILEGE,
            0,
            ptr::null(),
            0,
            ptr::null(),
            0,
            ptr::null(),
            &mut restricted,
        )
    };
    // SAFETY: GetLastError reads the thread's error slot only; read
    // before CloseHandle can overwrite it.
    let err = unsafe { GetLastError() };
    // SAFETY: `current` is the handle from OpenProcessToken above.
    unsafe { CloseHandle(current) };
    if rc == 0 {
        return Err(format!("CreateRestrictedToken: error={err}"));
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

/// The child's own lockdown step: lower this process's token to
/// Untrusted integrity. Windows starts an AppContainer process at Low;
/// lowering is always allowed, raising never is. Called first in the
/// child's `main`, before anything else runs.
pub fn apply_empty_process_profile() -> Result<(), String> {
    let mut token: HANDLE = ptr::null_mut();
    // SAFETY: the process pseudo-handle; needs no close.
    let process = unsafe { GetCurrentProcess() };
    // SAFETY: valid out-pointer.
    let rc = unsafe { OpenProcessToken(process, TOKEN_ADJUST_DEFAULT | TOKEN_QUERY, &mut token) };
    if rc == 0 {
        // SAFETY: GetLastError reads the thread's error slot only.
        return Err(format!("OpenProcessToken: error={}", unsafe {
            GetLastError()
        }));
    }
    let token = OwnedHandle(token);

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
        // SAFETY: GetLastError reads the thread's error slot only.
        return Err(format!("CreateWellKnownSid: error={}", unsafe {
            GetLastError()
        }));
    }
    let label = TOKEN_MANDATORY_LABEL {
        Label: SID_AND_ATTRIBUTES {
            Sid: sid_buf.as_mut_ptr() as *mut _,
            Attributes: SE_GROUP_INTEGRITY,
        },
    };
    // SAFETY: the label's Sid lives in `sid_buf` for the call's
    // duration; the length covers the struct and the SID it points to.
    let rc = unsafe {
        SetTokenInformation(
            token.as_handle(),
            TokenIntegrityLevel,
            &label as *const _ as *const _,
            size_of::<TOKEN_MANDATORY_LABEL>() as u32 + sid_len,
        )
    };
    if rc == 0 {
        // SAFETY: GetLastError reads the thread's error slot only.
        let err = unsafe { GetLastError() };
        return Err(format!("SetTokenInformation(integrity): error={err}"));
    }
    Ok(())
}

/// The child's tier probe. Reads its own token (integrity level and
/// whether it is an AppContainer) and attempts two reads the tier must
/// deny: `user_file`, a file of the user's that the supervisor created
/// (the AppContainer's job), and the SAM hive. Reports what happened —
/// a denial is data, never silence (SEC-8).
pub fn probe_line(user_file: Option<&Path>) -> String {
    let facts = token_facts();
    let in_job = in_job();
    let user = match user_file {
        Some(p) => format!("user-file-read={}", read_verdict(&p.to_string_lossy())),
        None => "user-file-read=not-probed".to_string(),
    };
    let sam = read_verdict("C:\\Windows\\System32\\config\\SAM");

    // The mitigation list is what this process observes about itself,
    // never a fixed claim (SEC-8): a process started outside the
    // supervisor reports what it lacks.
    let checks = [
        ("appcontainer", facts.appcontainer == "yes"),
        ("lockdown", facts.privileges == Some(1)),
        ("job", in_job == Some(true)),
        ("untrusted", facts.integrity == "untrusted"),
    ];
    let applied: Vec<&str> = checks.iter().filter(|c| c.1).map(|c| c.0).collect();
    let mut missing: Vec<&str> = checks.iter().filter(|c| !c.1).map(|c| c.0).collect();
    missing.extend(["win32k", "acg", "cig", "cet"]);
    let privileges = facts
        .privileges
        .map_or_else(|| "query-failed".to_string(), |n| n.to_string());
    format!(
        "integrity={} appcontainer={} privileges={privileges} {user} sam-read={sam} \
         mitigations={} (not-applied:{})",
        facts.integrity,
        facts.appcontainer,
        if applied.is_empty() {
            "none".to_string()
        } else {
            applied.join("+")
        },
        missing.join(","),
    )
}

/// What this process's own token says about the tier.
struct TokenFacts {
    integrity: String,
    appcontainer: String,
    /// Privileges in the token; the lockdown token keeps exactly one
    /// (SeChangeNotifyPrivilege).
    privileges: Option<u32>,
}

/// Whether this process is in a job that kills it when the job's last
/// handle closes (`None` if the query failed). Being in *some* job is
/// not enough: a remote shell's session job holds every process it
/// starts, so the check reads the limit the mitigation depends on.
fn in_job() -> Option<bool> {
    use windows_sys::Win32::System::JobObjects::{
        IsProcessInJob, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectExtendedLimitInformation, QueryInformationJobObject,
    };

    let mut result: i32 = 0;
    // SAFETY: the process pseudo-handle; needs no close.
    let process = unsafe { GetCurrentProcess() };
    // SAFETY: a valid process handle; a null job asks about any job;
    // valid out-pointer.
    if unsafe { IsProcessInJob(process, ptr::null_mut(), &mut result) } == 0 {
        return None;
    }
    if result == 0 {
        return Some(false);
    }
    // SAFETY: a zeroed struct is the documented initial state.
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: a null job handle queries this process's own job; the
    // buffer is the documented struct for this class, size passed.
    let ok = unsafe {
        QueryInformationJobObject(
            ptr::null_mut(),
            JobObjectExtendedLimitInformation,
            &mut limits as *mut _ as *mut _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ptr::null_mut(),
        )
    };
    (ok != 0).then_some(
        limits.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE != 0,
    )
}

/// The integrity level, the AppContainer flag and the privilege count
/// from this process's own token.
fn token_facts() -> TokenFacts {
    let mut token: HANDLE = ptr::null_mut();
    // SAFETY: the process pseudo-handle; needs no close.
    let process = unsafe { GetCurrentProcess() };
    // SAFETY: valid out-pointer.
    if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
        return TokenFacts {
            integrity: "token-open-failed".into(),
            appcontainer: "unknown".into(),
            privileges: None,
        };
    }
    let token = OwnedHandle(token);

    // A SID_AND_ATTRIBUTES header plus the SID; u64 for alignment.
    let mut buf = [0u64; 16];
    let mut len: u32 = 0;
    // SAFETY: the buffer is valid and its size is passed; the API
    // writes a TOKEN_MANDATORY_LABEL and the SID it points to.
    let ok = unsafe {
        GetTokenInformation(
            token.as_handle(),
            TokenIntegrityLevel,
            buf.as_mut_ptr() as *mut _,
            size_of::<[u64; 16]>() as u32,
            &mut len,
        )
    };
    let integrity = if ok == 0 {
        "query-failed".to_string()
    } else {
        // SAFETY: the API filled `buf` with a TOKEN_MANDATORY_LABEL;
        // the buffer is 8-byte aligned and large enough.
        let sid = unsafe { (*(buf.as_ptr() as *const TOKEN_MANDATORY_LABEL)).Label.Sid };
        // SAFETY: `sid` points into `buf`, a valid integrity SID.
        let count_ptr = unsafe { GetSidSubAuthorityCount(sid) };
        // SAFETY: the API returned a pointer into that SID's header.
        let count = unsafe { *count_ptr } as u32;
        // SAFETY: index count-1 is the SID's last subauthority, the RID.
        let rid_ptr = unsafe { GetSidSubAuthority(sid, count.saturating_sub(1)) };
        // SAFETY: the API returned a pointer into that SID's body.
        let rid = unsafe { *rid_ptr };
        match rid {
            0x0000 => "untrusted".to_string(),
            0x1000 => "low".to_string(),
            0x2000 => "medium".to_string(),
            r => format!("rid-{r:#x}"),
        }
    };

    let mut is_ac: u32 = 0;
    // SAFETY: a u32 out-buffer for TokenIsAppContainer, size passed.
    let ok = unsafe {
        GetTokenInformation(
            token.as_handle(),
            TokenIsAppContainer,
            &mut is_ac as *mut u32 as *mut _,
            4,
            &mut len,
        )
    };
    let appcontainer = match (ok, is_ac) {
        (0, _) => "query-failed",
        (_, 0) => "no",
        _ => "yes",
    };
    // TOKEN_PRIVILEGES starts with its u32 count; u64 for alignment.
    let mut privs = [0u64; 64];
    // SAFETY: the buffer is valid and its size is passed; the API
    // writes a TOKEN_PRIVILEGES.
    let ok = unsafe {
        GetTokenInformation(
            token.as_handle(),
            TokenPrivileges,
            privs.as_mut_ptr() as *mut _,
            size_of::<[u64; 64]>() as u32,
            &mut len,
        )
    };
    // The count is the first u32 of the structure the API wrote.
    let privileges = (ok != 0).then_some(privs[0] as u32);
    TokenFacts {
        integrity,
        appcontainer: appcontainer.to_string(),
        privileges,
    }
}

/// Opens `path` for reading and says how it went: `denied` only for
/// ERROR_ACCESS_DENIED, `ALLOWED` when it opened, `failed-<code>`
/// otherwise (a failure that is not a denial proves nothing).
fn read_verdict(path: &str) -> String {
    let w = wide(path);
    // SAFETY: the wide path is NUL-terminated and valid for the call.
    let h = unsafe {
        CreateFileW(
            w.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    };
    if h == INVALID_HANDLE_VALUE {
        // SAFETY: GetLastError reads the thread's error slot only.
        let err = unsafe { GetLastError() };
        if err == ERROR_ACCESS_DENIED {
            "denied".to_string()
        } else {
            format!("failed-{err}")
        }
    } else {
        // SAFETY: the handle is ours and valid.
        unsafe { CloseHandle(h) };
        "ALLOWED".to_string()
    }
}

/// A UTF-16, NUL-terminated string for the FFI.
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
