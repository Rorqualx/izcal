//! M1 D5 probes: shared helpers. Written from Microsoft Learn reference
//! pages and the windows-sys items as compiled; no header or crate source.
#![allow(clippy::missing_safety_doc, dead_code, unsafe_op_in_unsafe_fn)]

/// The repo's sandbox backend, copied verbatim from
/// crates/hg-substrate-os/src/sandbox/windows.rs (m1d4 worktree).
pub mod sandbox_windows;

use std::ffi::c_void;
use std::io::{BufRead, BufReader, Write};
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Security::Authorization::*;
use windows_sys::Win32::Security::*;
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::Win32::System::IO::*;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Pipes::*;
use windows_sys::Win32::System::Threading::*;

pub type H = HANDLE;

pub fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
pub fn le() -> u32 {
    unsafe { GetLastError() }
}
pub fn hv(h: H) -> isize {
    h as isize
}
pub fn hp(v: isize) -> H {
    v as H
}

/// 128 random bits as hex (BCryptGenRandom, system-preferred RNG).
pub fn rand_hex() -> String {
    use windows_sys::Win32::Security::Cryptography::{
        BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
    };
    let mut b = [0u8; 16];
    let st = unsafe {
        BCryptGenRandom(
            null_mut(),
            b.as_mut_ptr(),
            16,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    };
    assert_eq!(st, 0, "BCryptGenRandom");
    b.iter().map(|x| format!("{x:02x}")).collect()
}

// ---------- NtQueryObject (ntdll, by GetProcAddress) ----------
type NtQo = unsafe extern "system" fn(H, i32, *mut c_void, u32, *mut u32) -> i32;
fn ntqo() -> NtQo {
    unsafe {
        let m = GetModuleHandleW(w("ntdll.dll").as_ptr());
        let p = GetProcAddress(m, c"NtQueryObject".as_ptr() as *const u8).expect("NtQueryObject");
        std::mem::transmute::<_, NtQo>(p)
    }
}
/// Raw call: (NTSTATUS, ReturnLength, buffer).
pub fn nt_query_raw(h: H, class: i32) -> (i32, u32, Vec<u64>) {
    nt_query_len(h, class, 4096)
}
pub fn nt_query_len(h: H, class: i32, len: u32) -> (i32, u32, Vec<u64>) {
    let mut buf = vec![0u64; 512];
    let mut rl = 0u32;
    let st = unsafe { ntqo()(h, class, buf.as_mut_ptr() as *mut c_void, len, &mut rl) };
    (st, rl, buf)
}
/// ObjectTypeInformation; the class number is the one the P16 probe
/// identified (2), checked again at run time by the shape of the result.
pub fn type_name(h: H) -> String {
    let (st, _rl, buf) = nt_query_raw(h, 2);
    if st != 0 {
        return format!("ntstatus={:#x}", st as u32);
    }
    unicode_string_at(&buf)
}
fn unicode_string_at(buf: &[u64]) -> String {
    // UNICODE_STRING { Length: u16, MaximumLength: u16, (pad), Buffer: *u16 }
    let len = (buf[0] & 0xffff) as usize / 2;
    let p = buf[1] as *const u16;
    if p.is_null() {
        return String::new();
    }
    let s = unsafe { std::slice::from_raw_parts(p, len) };
    String::from_utf16_lossy(s)
}
/// ObjectBasicInformation GrantedAccess (class 0; second u32).
/// PUBLIC_OBJECT_BASIC_INFORMATION is 56 bytes (4 ULONGs + 10 reserved
/// ULONGs, per the NtQueryObject page); the call wants that exact size.
pub fn granted_access(h: H) -> Option<u32> {
    let (st, _rl, buf) = nt_query_len(h, 0, 56);
    (st == 0).then(|| (buf[0] >> 32) as u32)
}
pub fn handle_count(h: H) -> Option<u32> {
    let (st, _rl, buf) = nt_query_len(h, 0, 56);
    (st == 0).then(|| (buf[1] & 0xffff_ffff) as u32)
}
pub fn describe(h: H) -> String {
    format!(
        "type={} access={}",
        type_name(h),
        granted_access(h).map_or_else(
            || format!("?(status {:#x})", nt_query_len(h, 0, 56).0 as u32),
            |a| format!("{a:#x}")
        )
    )
}

// ---------- security descriptors ----------
pub fn sddl_of(h: H) -> String {
    let mut sd: PSECURITY_DESCRIPTOR = null_mut();
    let info = OWNER_SECURITY_INFORMATION
        | GROUP_SECURITY_INFORMATION
        | DACL_SECURITY_INFORMATION
        | LABEL_SECURITY_INFORMATION;
    let rc = unsafe {
        GetSecurityInfo(
            h,
            SE_KERNEL_OBJECT,
            info,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            &mut sd,
        )
    };
    if rc != 0 {
        return format!("GetSecurityInfo-err={rc}");
    }
    let mut s: *mut u16 = null_mut();
    let mut n = 0u32;
    let ok = unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            info,
            &mut s,
            &mut n,
        )
    };
    let out = if ok == 0 {
        format!("convert-err={}", le())
    } else {
        let sl = unsafe { std::slice::from_raw_parts(s, n as usize) };
        String::from_utf16_lossy(sl)
            .trim_end_matches('\0')
            .to_string()
    };
    unsafe {
        LocalFree(s as *mut c_void);
        LocalFree(sd as *mut c_void);
    }
    out
}
/// A SECURITY_ATTRIBUTES from SDDL. The descriptor is leaked (probe).
pub fn sa_from_sddl(sddl: &str, inherit: bool) -> SECURITY_ATTRIBUTES {
    let mut sd: PSECURITY_DESCRIPTOR = null_mut();
    let ok = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            w(sddl).as_ptr(),
            SDDL_REVISION_1,
            &mut sd,
            null_mut(),
        )
    };
    assert!(ok != 0, "bad sddl {sddl}: {}", le());
    SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd,
        bInheritHandle: inherit as i32,
    }
}
pub fn sid_string(sid: PSID) -> String {
    let mut s: *mut u16 = null_mut();
    if unsafe { ConvertSidToStringSidW(sid, &mut s) } == 0 {
        return format!("sid-err={}", le());
    }
    let mut n = 0;
    while unsafe { *s.add(n) } != 0 {
        n += 1;
    }
    let out = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(s, n) });
    unsafe { LocalFree(s as *mut c_void) };
    out
}
/// This process's user SID.
pub fn user_sid() -> String {
    let mut tok: H = null_mut();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut tok) };
    let mut buf = [0u64; 64];
    let mut n = 0;
    let ok = unsafe {
        GetTokenInformation(tok, TokenUser, buf.as_mut_ptr() as *mut c_void, 512, &mut n)
    };
    unsafe { CloseHandle(tok) };
    assert!(ok != 0);
    let tu = buf.as_ptr() as *const TOKEN_USER;
    sid_string(unsafe { (*tu).User.Sid })
}
pub fn session_id() -> u32 {
    let mut s = 0u32;
    unsafe {
        windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId(
            GetCurrentProcessId(),
            &mut s,
        )
    };
    s
}

// ---------- pipes ----------
pub const MSG: u32 =
    PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS;

/// A broker-made pair: random name, first instance, reject remote, one
/// instance, overlapped, explicit DACL (`sddl`; None = NULL SD).
/// Returns (server, client, name). The client is opened with
/// read+write data and write attributes, switched to message read mode.
pub fn make_pair(sddl: Option<&str>, out_buf: u32, in_buf: u32) -> Result<(H, H, String), String> {
    let name = format!(r"\\.\pipe\izcal-d5-{}", rand_hex());
    let sa = sddl.map(|s| sa_from_sddl(s, false));
    let sap = sa.as_ref().map_or(null(), |s| s as *const _);
    let server = unsafe {
        CreateNamedPipeW(
            w(&name).as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
            MSG,
            1,
            out_buf,
            in_buf,
            0,
            sap,
        )
    };
    if server == INVALID_HANDLE_VALUE {
        return Err(format!("CreateNamedPipeW err={}", le()));
    }
    let client = unsafe {
        CreateFileW(
            w(&name).as_ptr(),
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
        return Err(format!("CreateFileW(client) err={}", le()));
    }
    let mode = PIPE_READMODE_MESSAGE;
    if unsafe { SetNamedPipeHandleState(client, &mode, null(), null()) } == 0 {
        return Err(format!("SetNamedPipeHandleState err={}", le()));
    }
    // Connect: the client already opened, so expect ERROR_PIPE_CONNECTED.
    let mut ov: OVERLAPPED = unsafe { zeroed() };
    let ok = unsafe { ConnectNamedPipe(server, &mut ov) };
    let e = le();
    if ok == 0 && e != ERROR_PIPE_CONNECTED {
        return Err(format!("ConnectNamedPipe err={e}"));
    }
    Ok((server, client, name))
}

pub struct Ov {
    pub ov: Box<OVERLAPPED>,
    pub buf: Vec<u8>,
}
pub fn event() -> H {
    unsafe { CreateEventW(null(), 1, 0, null()) }
}
/// Starts an overlapped write. Returns (immediate-ok, error, op).
pub fn write_start(h: H, data: Vec<u8>) -> (bool, u32, Ov) {
    let mut ov: Box<OVERLAPPED> = Box::new(unsafe { zeroed() });
    ov.hEvent = event();
    let mut n = 0u32;
    let ok = unsafe { WriteFile(h, data.as_ptr(), data.len() as u32, &mut n, &mut *ov) };
    let e = if ok == 0 { le() } else { 0 };
    (ok != 0, e, Ov { ov, buf: data })
}
/// Waits for an op (timeout ms). Ok(bytes) | Err(error) | Err(WAIT_TIMEOUT).
pub fn finish(h: H, op: &mut Ov, ms: u32) -> Result<u32, u32> {
    let r = unsafe { WaitForSingleObject(op.ov.hEvent, ms) };
    if r == WAIT_TIMEOUT {
        return Err(WAIT_TIMEOUT);
    }
    let mut n = 0u32;
    let ok = unsafe { GetOverlappedResult(h, &*op.ov, &mut n, 0) };
    if ok == 0 { Err(le()) } else { Ok(n) }
}
pub fn close_ov(op: Ov) {
    unsafe { CloseHandle(op.ov.hEvent) };
}
/// Synchronous (event-waited) message write.
pub fn write_msg(h: H, data: Vec<u8>) -> Result<u32, u32> {
    let (ok, e, mut op) = write_start(h, data);
    let r = if ok {
        let mut n = 0;
        unsafe { GetOverlappedResult(h, &*op.ov, &mut n, 0) };
        Ok(n)
    } else if e == ERROR_IO_PENDING {
        finish(h, &mut op, 10_000)
    } else {
        Err(e)
    };
    close_ov(op);
    r
}
/// One read with a timeout: Ok(bytes) or Err((error, bytes)). On a
/// timeout the read is cancelled and Err((WAIT_TIMEOUT,0)) returned.
pub fn read_msg(h: H, cap: usize, ms: u32) -> Result<Vec<u8>, (u32, Vec<u8>)> {
    let mut ov: Box<OVERLAPPED> = Box::new(unsafe { zeroed() });
    ov.hEvent = event();
    let mut buf = vec![0u8; cap];
    let mut n = 0u32;
    let ok = unsafe { ReadFile(h, buf.as_mut_ptr(), cap as u32, &mut n, &mut *ov) };
    let res = if ok != 0 {
        unsafe { GetOverlappedResult(h, &*ov, &mut n, 0) };
        buf.truncate(n as usize);
        Ok(buf)
    } else {
        let e = le();
        if e != ERROR_IO_PENDING {
            unsafe { GetOverlappedResult(h, &*ov, &mut n, 0) };
            buf.truncate(n as usize);
            Err((e, buf))
        } else {
            let r = unsafe { WaitForSingleObject(ov.hEvent, ms) };
            if r == WAIT_TIMEOUT {
                unsafe { CancelIoEx(h, &*ov) };
                unsafe { GetOverlappedResult(h, &*ov, &mut n, 1) };
                Err((WAIT_TIMEOUT, vec![]))
            } else {
                let ok = unsafe { GetOverlappedResult(h, &*ov, &mut n, 0) };
                let e = le();
                buf.truncate(n as usize);
                if ok != 0 { Ok(buf) } else { Err((e, buf)) }
            }
        }
    };
    unsafe { CloseHandle(ov.hEvent) };
    res
}
pub fn fmt_read(r: &Result<Vec<u8>, (u32, Vec<u8>)>) -> String {
    match r {
        Ok(b) => format!(
            "ok len={} first={}",
            b.len(),
            b.first().map_or(-1, |x| *x as i32)
        ),
        Err((e, b)) => format!("err={} len={}", e, b.len()),
    }
}
/// Reads until an error; one line per result.
pub fn read_all(h: H, cap: usize, ms: u32) -> Vec<String> {
    let mut out = vec![];
    for _ in 0..64 {
        let r = read_msg(h, cap, ms);
        out.push(fmt_read(&r));
        if r.is_err() {
            break;
        }
    }
    out
}
/// DuplicateHandle wrapper.
pub fn dup(sp: H, h: H, tp: H, access: u32, opts: u32) -> Result<H, u32> {
    let mut out: H = null_mut();
    let ok = unsafe { DuplicateHandle(sp, h, tp, &mut out, access, 0, opts) };
    if ok == 0 { Err(le()) } else { Ok(out) }
}
pub fn me() -> H {
    unsafe { GetCurrentProcess() }
}
pub fn pipe_info(h: H) -> String {
    let (mut f, mut o, mut i, mut m) = (0u32, 0u32, 0u32, 0u32);
    let ok = unsafe { GetNamedPipeInfo(h, &mut f, &mut o, &mut i, &mut m) };
    if ok == 0 {
        format!("GetNamedPipeInfo err={}", le())
    } else {
        format!("GetNamedPipeInfo flags={f:#x} out={o} in={i} max={m}")
    }
}

// ---------- launching a sandboxed child ----------
pub struct Child {
    pub process: H,
    pub thread: H,
    pub pid: u32,
    pub job: Option<sandbox_windows::OwnedHandle>,
    pub to: std::fs::File,
    pub lines: Receiver<String>,
}
pub struct Launch<'a> {
    pub container: &'a str,
    pub sandbox: bool,
    pub extra_inherit: Vec<H>,
    pub args: String,
    /// PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY value (two u64 words).
    pub mitigation: Option<[u64; 2]>,
    /// For P12: no handle list at all (inherit every inheritable handle).
    pub no_list: bool,
}
impl Default for Launch<'_> {
    fn default() -> Self {
        Launch {
            container: "izcal.d5probe.a",
            sandbox: true,
            extra_inherit: vec![],
            args: String::new(),
            mitigation: None,
            no_list: false,
        }
    }
}
fn anon_pipe() -> (H, H) {
    let sa = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: 1,
    };
    let (mut r, mut wr): (H, H) = (null_mut(), null_mut());
    assert!(
        unsafe { CreatePipe(&mut r, &mut wr, &sa, 0) } != 0,
        "CreatePipe {}",
        le()
    );
    (r, wr)
}
/// Starts d5child suspended (sandboxed: lockdown token, AppContainer
/// `container` without capabilities, kill-on-close job assigned before
/// it runs), with an inherited stdin/stdout pipe pair (handle list).
/// The caller duplicates handles in and then calls `resume`.
pub fn launch(l: &Launch) -> Result<Child, String> {
    use sandbox_windows::*;
    let (in_r, in_w) = anon_pipe(); // child's stdin
    let (out_r, out_w) = anon_pipe(); // child's stdout
    unsafe {
        SetHandleInformation(in_w, HANDLE_FLAG_INHERIT, 0);
        SetHandleInformation(out_r, HANDLE_FLAG_INHERIT, 0);
    }
    let mut n_attr = 1u32;
    if l.sandbox {
        n_attr += 1;
    }
    if l.mitigation.is_some() {
        n_attr += 1;
    }
    let mut size = 0usize;
    unsafe { InitializeProcThreadAttributeList(null_mut(), n_attr, 0, &mut size) };
    let mut mem = vec![0u64; size.div_ceil(8)];
    let attrs = mem.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;
    assert!(unsafe { InitializeProcThreadAttributeList(attrs, n_attr, 0, &mut size) } != 0);
    let container = if l.sandbox {
        Some(app_container(l.container, l.container)?)
    } else {
        None
    };
    let mut caps: SECURITY_CAPABILITIES = unsafe { zeroed() };
    if let Some(c) = &container {
        caps.AppContainerSid = c.as_psid();
        let ok = unsafe {
            UpdateProcThreadAttribute(
                attrs,
                0,
                PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
                &mut caps as *mut _ as *mut c_void,
                size_of::<SECURITY_CAPABILITIES>(),
                null_mut(),
                null(),
            )
        };
        assert!(ok != 0);
    }
    let mut list = vec![in_r, out_w];
    list.extend(l.extra_inherit.iter().copied());
    if std::env::var_os("D5_LISTLOG").is_some() {
        eprintln!(
            "[launch] sandbox={} HANDLE_LIST={:?}",
            l.sandbox,
            list.iter().map(|h| hv(*h)).collect::<Vec<_>>()
        );
    }
    if !l.no_list {
        let ok = unsafe {
            UpdateProcThreadAttribute(
                attrs,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                list.as_mut_ptr() as *mut c_void,
                size_of::<H>() * list.len(),
                null_mut(),
                null(),
            )
        };
        assert!(ok != 0);
    }
    let mut mit = l.mitigation.unwrap_or([0, 0]);
    if l.mitigation.is_some() {
        let ok = unsafe {
            UpdateProcThreadAttribute(
                attrs,
                0,
                PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY as usize,
                mit.as_mut_ptr() as *mut c_void,
                16,
                null_mut(),
                null(),
            )
        };
        if ok == 0 {
            return Err(format!(
                "UpdateProcThreadAttribute(mitigation) err={}",
                le()
            ));
        }
    }
    let exe = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("d5child.exe");
    let mut cmd = w(&format!(
        "\"{}\" {}{}",
        exe.display(),
        if l.sandbox { "" } else { "--nosandbox " },
        l.args
    ));
    let exe_w = w(&exe.to_string_lossy());
    let mut si: STARTUPINFOEXW = unsafe { zeroed() };
    si.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    si.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    si.StartupInfo.hStdInput = in_r;
    si.StartupInfo.hStdOutput = out_w;
    si.StartupInfo.hStdError = out_w;
    if !l.no_list || l.mitigation.is_some() || l.sandbox {
        si.lpAttributeList = attrs;
    }
    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
    let flags = EXTENDED_STARTUPINFO_PRESENT | CREATE_SUSPENDED | CREATE_NO_WINDOW;
    let token = if l.sandbox {
        Some(restricted_token()?)
    } else {
        None
    };
    let rc = unsafe {
        match &token {
            Some(t) => CreateProcessAsUserW(
                t.as_handle(),
                exe_w.as_ptr(),
                cmd.as_mut_ptr(),
                null(),
                null(),
                1,
                flags,
                null(),
                null(),
                &si.StartupInfo,
                &mut pi,
            ),
            None => CreateProcessW(
                exe_w.as_ptr(),
                cmd.as_mut_ptr(),
                null(),
                null(),
                1,
                flags,
                null(),
                null(),
                &si.StartupInfo,
                &mut pi,
            ),
        }
    };
    let e = le();
    unsafe {
        DeleteProcThreadAttributeList(attrs);
        CloseHandle(in_r);
        CloseHandle(out_w);
    }
    if rc == 0 {
        return Err(format!("CreateProcess err={e}"));
    }
    let job = if l.sandbox {
        let j = job_object()?;
        unsafe { assign_job(&j, pi.hProcess)? };
        Some(j)
    } else {
        None
    };
    let (tx, rx) = channel();
    let out_raw = out_r as usize;
    std::thread::spawn(move || {
        use std::os::windows::io::FromRawHandle;
        let f = unsafe { std::fs::File::from_raw_handle(out_raw as *mut c_void) };
        for line in BufReader::new(f).lines() {
            match line {
                Ok(s) => {
                    if tx.send(s).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    use std::os::windows::io::FromRawHandle;
    let to = unsafe { std::fs::File::from_raw_handle(in_w as *mut c_void) };
    Ok(Child {
        process: pi.hProcess,
        thread: pi.hThread,
        pid: pi.dwProcessId,
        job,
        to,
        lines: rx,
    })
}
impl Child {
    pub fn resume(&self) -> u32 {
        unsafe { ResumeThread(self.thread) }
    }
    /// Sends one command line and collects reply lines up to "END".
    pub fn cmd(&mut self, c: &str) -> Vec<String> {
        let _ = writeln!(self.to, "{c}");
        let _ = self.to.flush();
        self.collect(Duration::from_secs(60))
    }
    pub fn collect(&mut self, t: Duration) -> Vec<String> {
        let mut out = vec![];
        loop {
            match self.lines.recv_timeout(t) {
                Ok(s) if s == "END" => break,
                Ok(s) => out.push(s),
                Err(e) => {
                    out.push(format!("<no END: {e:?}; {}>", self.state()));
                    break;
                }
            }
        }
        out
    }
    pub fn state(&self) -> String {
        let mut code = 0u32;
        unsafe { GetExitCodeProcess(self.process, &mut code) };
        if code == STILL_ACTIVE as u32 {
            "running".into()
        } else {
            format!("exited code={code:#x}")
        }
    }
    /// Duplicates a broker handle into the child.
    pub fn give(&self, h: H, access: u32, opts: u32) -> Result<isize, u32> {
        dup(me(), h, self.process, access, opts).map(hv)
    }
    pub fn kill(mut self) -> String {
        drop(self.job.take());
        unsafe { WaitForSingleObject(self.process, 5000) };
        self.state()
    }
}
