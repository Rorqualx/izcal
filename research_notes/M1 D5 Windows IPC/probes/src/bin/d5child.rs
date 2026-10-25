//! M1 D5 probe child: lowers itself to Untrusted first (the repo's
//! `apply_empty_process_profile`), then runs commands read from its
//! inherited stdin pipe and answers on its stdout pipe, ending each
//! answer with a line "END". Handle values arrive as decimal integers.
// The GUI subsystem, like fp-m0-empty: no console under the sandbox.
#![windows_subsystem = "windows"]
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

use std::ffi::c_void;
use std::io::{BufRead, Write};
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use d5probe::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Security::Isolation::GetAppContainerNamedObjectPath;
use windows_sys::Win32::Security::*;
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::Win32::System::IO::*;
use windows_sys::Win32::System::Memory::*;
use windows_sys::Win32::System::Pipes::*;
use windows_sys::Win32::System::Threading::*;

struct St {
    port: H,
    wakes: Arc<AtomicU32>,
}

fn main() {
    {
        let mut o = std::io::stdout();
        let _ = writeln!(o, "boot (main reached)");
        let _ = o.flush();
    }
    let args: Vec<String> = std::env::args().collect();
    let sandboxed = !args.iter().any(|a| a == "--nosandbox");
    let lowered = if sandboxed && !args.iter().any(|a| a == "--nolower") {
        match d5probe::sandbox_windows::apply_empty_process_profile() {
            Ok(()) => "ok".to_string(),
            Err(e) => e,
        }
    } else {
        "skipped".to_string()
    };
    let mut out = std::io::stdout();
    let _ = writeln!(
        out,
        "hello pid={} lowered={} args={:?} {}",
        std::process::id(),
        lowered,
        &args[1..],
        d5probe::sandbox_windows::probe_line(None)
    );
    let _ = writeln!(out, "END");
    let _ = out.flush();
    let mut st = St {
        port: null_mut(),
        wakes: Arc::new(AtomicU32::new(0)),
    };
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        if parts[0] == "exit" {
            break;
        }
        let reply = run(&parts, &mut st);
        for r in reply {
            let _ = writeln!(out, "{r}");
        }
        let _ = writeln!(out, "END");
        let _ = out.flush();
    }
}

fn n(s: &str) -> isize {
    if let Some(x) = s.strip_prefix("0x") {
        isize::from_str_radix(x, 16).unwrap()
    } else {
        s.parse().unwrap()
    }
}
fn u(s: &str) -> u32 {
    n(s) as u32
}

fn token_ac_sid() -> String {
    let mut tok: H = null_mut();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut tok) };
    let mut buf = [0u64; 64];
    let mut len = 0;
    let ok = unsafe {
        GetTokenInformation(
            tok,
            TokenAppContainerSid,
            buf.as_mut_ptr() as *mut c_void,
            512,
            &mut len,
        )
    };
    unsafe { CloseHandle(tok) };
    if ok == 0 {
        return format!("err={}", le());
    }
    let p = buf.as_ptr() as *const TOKEN_APPCONTAINER_INFORMATION;
    let sid = unsafe { (*p).TokenAppContainer };
    if sid.is_null() {
        "none".into()
    } else {
        sid_string(sid)
    }
}

fn ac_path() -> String {
    let mut b = [0u16; 512];
    let mut rl = 0u32;
    let ok = unsafe {
        GetAppContainerNamedObjectPath(null_mut(), null_mut(), 512, b.as_mut_ptr(), &mut rl)
    };
    if ok == 0 {
        return format!("err={}", le());
    }
    let len = b.iter().position(|&c| c == 0).unwrap_or(0);
    String::from_utf16_lossy(&b[..len])
}

fn file_name(h: H) -> String {
    let mut buf = [0u32; 256];
    let ok = unsafe {
        GetFileInformationByHandleEx(h, FileNameInfo, buf.as_mut_ptr() as *mut c_void, 1024)
    };
    if ok == 0 {
        return format!("name-err={}", le());
    }
    let len = buf[0] as usize / 2;
    let p = unsafe { (buf.as_ptr() as *const u16).add(2) };
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(p, len) })
}

fn run(p: &[&str], st: &mut St) -> Vec<String> {
    let mut o = vec![];
    match p[0] {
        "info" => {
            o.push(format!(
                "pid={} session={} ac-sid={} ac-path={}",
                std::process::id(),
                session_id(),
                token_ac_sid(),
                ac_path()
            ));
            o.push(format!(
                "GetCurrentProcess={} GetCurrentThread={}",
                hv(unsafe { GetCurrentProcess() }),
                hv(unsafe { GetCurrentThread() })
            ));
            // Which small negative values act as token handles here
            // (the token pseudo-handles' values are not documented).
            for v in -8isize..=-1 {
                let mut len = 0u32;
                let ok =
                    unsafe { GetTokenInformation(hp(v), TokenStatistics, null_mut(), 0, &mut len) };
                let e = le();
                let mut tok_type = String::new();
                if e == ERROR_INSUFFICIENT_BUFFER {
                    let mut buf = [0u64; 64];
                    let ok2 = unsafe {
                        GetTokenInformation(
                            hp(v),
                            TokenStatistics,
                            buf.as_mut_ptr() as *mut c_void,
                            512,
                            &mut len,
                        )
                    };
                    if ok2 != 0 {
                        let ts = buf.as_ptr() as *const TOKEN_STATISTICS;
                        tok_type = format!(
                            " token-type={} impersonation-level={}",
                            unsafe { (*ts).TokenType },
                            unsafe { (*ts).ImpersonationLevel }
                        );
                    }
                }
                o.push(format!(
                    "value {v}: GetTokenInformation ok={ok} err={e}{tok_type} ntq={}",
                    type_name(hp(v))
                ));
            }
        }
        "mkpipe" | "mkpipe_sd" => {
            let name = p[1];
            let sa = if p[0] == "mkpipe_sd" {
                Some(sa_from_sddl(p[2], false))
            } else {
                None
            };
            let h = unsafe {
                CreateNamedPipeW(
                    w(name).as_ptr(),
                    PIPE_ACCESS_INBOUND | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
                    PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_REJECT_REMOTE_CLIENTS,
                    1,
                    0,
                    0,
                    0,
                    sa.as_ref().map_or(null(), |s| s as *const _),
                )
            };
            if h == INVALID_HANDLE_VALUE {
                o.push(format!("CreateNamedPipeW FAILED err={}", le()));
            } else {
                o.push(format!(
                    "CreateNamedPipeW OK h={} filename={} sd={}",
                    hv(h),
                    file_name(h),
                    sddl_of(h)
                ));
            }
        }
        "open" => {
            let h = unsafe {
                CreateFileW(
                    w(p[1]).as_ptr(),
                    u(p[2]),
                    0,
                    null(),
                    OPEN_EXISTING,
                    FILE_FLAG_OVERLAPPED,
                    null_mut(),
                )
            };
            if h == INVALID_HANDLE_VALUE {
                o.push(format!("CreateFileW FAILED err={}", le()));
            } else {
                o.push(format!("CreateFileW OK h={} {}", hv(h), describe(h)));
            }
        }
        "write" => {
            // write <h> <count> <len>: message i is `len` bytes of value i.
            let h = hp(n(p[1]));
            for i in 0..u(p[2]) {
                let r = write_msg(h, vec![i as u8; u(p[3]) as usize]);
                o.push(format!("write {i}: {r:?}"));
            }
        }
        "read" => {
            let r = read_msg(hp(n(p[1])), u(p[2]) as usize, u(p[3]));
            o.push(fmt_read(&r));
        }
        "close" => {
            let ok = unsafe { CloseHandle(hp(n(p[1]))) };
            o.push(format!(
                "CloseHandle ok={ok} err={}",
                if ok == 0 { le() } else { 0 }
            ));
        }
        "access" => o.push(describe(hp(n(p[1])))),
        "checkh" => {
            for v in &p[1..] {
                let h = hp(n(v));
                let mut flags = 0u32;
                let ok = unsafe { GetHandleInformation(h, &mut flags) };
                let e = le();
                o.push(format!(
                    "value {v}: GetHandleInformation ok={ok} err={} flags={flags} filetype={} {}",
                    if ok == 0 { e } else { 0 },
                    unsafe { GetFileType(h) },
                    describe(h)
                ));
            }
        }
        "ops" => {
            // P18/P27: what each pipe call does with this handle's access.
            let h = hp(n(p[1]));
            o.push(format!("{} filetype={}", describe(h), unsafe {
                GetFileType(h)
            }));
            let (mut avail, mut left) = (0u32, 0u32);
            let ok = unsafe { PeekNamedPipe(h, null_mut(), 0, null_mut(), &mut avail, &mut left) };
            o.push(format!(
                "PeekNamedPipe ok={ok} err={} avail={avail} left={left}",
                if ok == 0 { le() } else { 0 }
            ));
            o.push(pipe_info(h));
            let mode = PIPE_READMODE_MESSAGE;
            let ok = unsafe { SetNamedPipeHandleState(h, &mode, null(), null()) };
            o.push(format!(
                "SetNamedPipeHandleState(MESSAGE) ok={ok} err={}",
                if ok == 0 { le() } else { 0 }
            ));
            let mut state = 0u32;
            let ok = unsafe {
                GetNamedPipeHandleStateW(
                    h,
                    &mut state,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    0,
                )
            };
            o.push(format!(
                "GetNamedPipeHandleState ok={ok} err={} state={state:#x}",
                if ok == 0 { le() } else { 0 }
            ));
            let port = unsafe { CreateIoCompletionPort(h, null_mut(), 7, 1) };
            o.push(format!(
                "CreateIoCompletionPort(assoc) {}",
                if port.is_null() {
                    format!("FAILED err={}", le())
                } else {
                    "ok".into()
                }
            ));
            if !port.is_null() {
                unsafe { CloseHandle(port) };
            }
        }
        "iocpread" => {
            // A read through a completion port (the design's IO path).
            let h = hp(n(p[1]));
            let port = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, null_mut(), 0, 1) };
            let a = unsafe { CreateIoCompletionPort(h, port, 42, 1) };
            if a.is_null() {
                o.push(format!("assoc FAILED err={}", le()));
                return o;
            }
            let mut ov: Box<OVERLAPPED> = Box::new(unsafe { zeroed() });
            let mut buf = vec![0u8; u(p[2]) as usize];
            let ok =
                unsafe { ReadFile(h, buf.as_mut_ptr(), buf.len() as u32, null_mut(), &mut *ov) };
            o.push(format!(
                "ReadFile ok={ok} err={}",
                if ok == 0 { le() } else { 0 }
            ));
            let (mut bytes, mut key, mut pov) = (0u32, 0usize, null_mut::<OVERLAPPED>());
            let ok =
                unsafe { GetQueuedCompletionStatus(port, &mut bytes, &mut key, &mut pov, 5000) };
            o.push(format!(
                "GQCS ok={ok} err={} bytes={bytes} key={key} ov-match={}",
                if ok == 0 { le() } else { 0 },
                pov == &mut *ov as *mut _
            ));
            unsafe { CloseHandle(port) };
        }
        "widen" => {
            // P19/P23(iii): DuplicateHandle(self -> self) asking for more.
            let h = hp(n(p[1]));
            let r = dup(me(), h, me(), u(p[2]), 0);
            match r {
                Ok(h2) => o.push(format!(
                    "DuplicateHandle(self, {:#x}) OK -> {}",
                    u(p[2]),
                    describe(h2)
                )),
                Err(e) => o.push(format!(
                    "DuplicateHandle(self, {:#x}) FAILED err={e}",
                    u(p[2])
                )),
            }
        }
        "section" => {
            // P23: a section handle the broker gave with FILE_MAP_READ.
            let h = hp(n(p[1]));
            o.push(format!("{} (FILE_MAP_READ={:#x} FILE_MAP_WRITE={:#x} FILE_MAP_COPY={:#x} FILE_MAP_ALL_ACCESS={:#x} SECTION_QUERY={:#x})", describe(h), FILE_MAP_READ, FILE_MAP_WRITE, FILE_MAP_COPY, FILE_MAP_ALL_ACCESS, SECTION_QUERY));
            let v = unsafe { MapViewOfFile(h, FILE_MAP_WRITE, 0, 0, 0) };
            o.push(format!(
                "(i) MapViewOfFile(WRITE) {}",
                if v.Value.is_null() {
                    format!("FAILED err={}", le())
                } else {
                    "MAPPED".into()
                }
            ));
            let v = unsafe { MapViewOfFile(h, FILE_MAP_READ, 0, 0, 0) };
            if v.Value.is_null() {
                o.push(format!("(ii) MapViewOfFile(READ) FAILED err={}", le()));
            } else {
                let first = unsafe { std::slice::from_raw_parts(v.Value as *const u8, 5) }.to_vec();
                let mut old = 0u32;
                let ok = unsafe { VirtualProtect(v.Value, 4096, PAGE_READWRITE, &mut old) };
                o.push(format!(
                    "(ii) read view ok, data={:?}; VirtualProtect(RW) ok={ok} err={} old={old:#x}",
                    String::from_utf8_lossy(&first),
                    if ok == 0 { le() } else { 0 }
                ));
            }
            for (label, acc) in [
                ("FILE_MAP_WRITE", FILE_MAP_WRITE),
                ("FILE_MAP_ALL_ACCESS", FILE_MAP_ALL_ACCESS),
                ("FILE_MAP_READ|WRITE", FILE_MAP_READ | FILE_MAP_WRITE),
            ] {
                match dup(me(), h, me(), acc, 0) {
                    Ok(h2) => {
                        let v = unsafe { MapViewOfFile(h2, FILE_MAP_WRITE, 0, 0, 0) };
                        let wrote = if v.Value.is_null() {
                            format!("write-map FAILED err={}", le())
                        } else {
                            unsafe { *(v.Value as *mut u8).add(100) = b'W' };
                            "write-map OK, wrote byte 100='W'".into()
                        };
                        o.push(format!(
                            "(iii) DuplicateHandle(self, {label}) OK -> {}; {wrote}",
                            describe(h2)
                        ));
                    }
                    Err(e) => o.push(format!(
                        "(iii) DuplicateHandle(self, {label}) FAILED err={e}"
                    )),
                }
            }
            let v = unsafe { MapViewOfFile(h, FILE_MAP_COPY, 0, 0, 0) };
            if v.Value.is_null() {
                o.push(format!("(v) MapViewOfFile(COPY) FAILED err={}", le()));
            } else {
                unsafe { *(v.Value as *mut u8).add(200) = b'C' };
                o.push("(v) copy view mapped, wrote byte 200='C' privately".into());
            }
        }
        "openmap" => {
            let h = unsafe { OpenFileMappingW(u(p[2]), 0, w(p[1]).as_ptr()) };
            o.push(if h.is_null() {
                format!("OpenFileMappingW({}) FAILED err={}", p[1], le())
            } else {
                format!("OpenFileMappingW({}) OK {}", p[1], describe(h))
            });
        }
        "mksection" => {
            let h = unsafe {
                CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    null(),
                    PAGE_READWRITE | SEC_COMMIT,
                    0,
                    65536,
                    null(),
                )
            };
            o.push(if h.is_null() {
                format!("CreateFileMappingW FAILED err={}", le())
            } else {
                format!("CreateFileMappingW OK {}", describe(h))
            });
        }
        "idle" => {
            // P20: one port, every given handle associated with a standing
            // read, one IO thread in GetQueuedCompletionStatusEx(INFINITE).
            let port = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, null_mut(), 0, 1) };
            st.port = port;
            let mut ok_n = 0;
            for (i, v) in p[1..].iter().enumerate() {
                let h = hp(n(v));
                if unsafe { CreateIoCompletionPort(h, port, i + 1, 1) }.is_null() {
                    o.push(format!("assoc {v} FAILED err={}", le()));
                    continue;
                }
                let ov: &'static mut OVERLAPPED = Box::leak(Box::new(unsafe { zeroed() }));
                let buf: &'static mut [u8] = Box::leak(vec![0u8; 4096].into_boxed_slice());
                let ok = unsafe { ReadFile(h, buf.as_mut_ptr(), 4096, null_mut(), ov) };
                if ok == 0 && le() == ERROR_IO_PENDING {
                    ok_n += 1;
                } else {
                    o.push(format!("read {v}: ok={ok} err={}", le()));
                }
            }
            let wakes = st.wakes.clone();
            let port_raw = port as usize;
            std::thread::spawn(move || {
                let port = port_raw as H;
                loop {
                    let mut ents: [OVERLAPPED_ENTRY; 16] = unsafe { zeroed() };
                    let mut got = 0u32;
                    let ok = unsafe {
                        GetQueuedCompletionStatusEx(
                            port,
                            ents.as_mut_ptr(),
                            16,
                            &mut got,
                            INFINITE,
                            0,
                        )
                    };
                    wakes.fetch_add(1, Ordering::SeqCst);
                    if ok == 0 {
                        break;
                    }
                }
            });
            o.push(format!("pending reads={ok_n} io thread started"));
        }
        "post" => {
            let ok = unsafe { PostQueuedCompletionStatus(st.port, 0, 999, null_mut()) };
            std::thread::sleep(std::time::Duration::from_millis(200));
            o.push(format!(
                "posted ok={ok} wakes={}",
                st.wakes.load(Ordering::SeqCst)
            ));
        }
        "wakes" => o.push(format!("wakes={}", st.wakes.load(Ordering::SeqCst))),
        "echo" => {
            // Round trip: read on <hin>, write the same bytes on <hout>,
            // both through one completion port, until a read fails.
            let (hin, hout) = (hp(n(p[1])), hp(n(p[2])));
            let (a, b) = (hin as usize, hout as usize);
            std::thread::spawn(move || echo(a as H, b as H));
            o.push("echo started".into());
        }
        "strict" => {
            // P26 child side: strict handle checks on, then a bad close.
            // PROCESS_MITIGATION_STRICT_HANDLE_CHECK_POLICY is one DWORD (Flags):
            // bit 0 RaiseExceptionOnInvalidHandleReference, bit 1
            // HandleExceptionsPermanentlyEnabled (Microsoft Learn struct page).
            let pol: u32 = 0b11;
            let ok = unsafe {
                SetProcessMitigationPolicy(
                    ProcessStrictHandleCheckPolicy,
                    &pol as *const _ as *const c_void,
                    size_of::<u32>(),
                )
            };
            o.push(format!(
                "SetProcessMitigationPolicy(strict) ok={ok} err={}",
                if ok == 0 { le() } else { 0 }
            ));
            if p.len() > 1 {
                let ok = unsafe { CloseHandle(hp(n(p[1]))) };
                o.push(format!(
                    "CloseHandle({}) returned ok={ok} err={} (no exception)",
                    p[1],
                    le()
                ));
            }
        }
        "mitig" => {
            let mut f: u32 = 0;
            let ok = unsafe {
                GetProcessMitigationPolicy(
                    GetCurrentProcess(),
                    ProcessStrictHandleCheckPolicy,
                    &mut f as *mut u32 as *mut c_void,
                    4,
                )
            };
            o.push(format!(
                "StrictHandleCheckPolicy flags={f:#x} (ok={ok} err={})",
                if ok == 0 { le() } else { 0 }
            ));
            let mut g: u32 = 0;
            let ok = unsafe {
                GetProcessMitigationPolicy(
                    GetCurrentProcess(),
                    ProcessSystemCallDisablePolicy,
                    &mut g as *mut u32 as *mut c_void,
                    4,
                )
            };
            o.push(format!(
                "SystemCallDisablePolicy flags={g:#x} (ok={ok} err={})",
                if ok == 0 { le() } else { 0 }
            ));
        }
        "inv" => {
            // One API on one (invalid) value, so the one that raises is known.
            let h = hp(n(p[2]));
            let r = match p[1] {
                "GetHandleInformation" => {
                    let mut f = 0u32;
                    let ok = unsafe { GetHandleInformation(h, &mut f) };
                    format!("ok={ok} err={}", le())
                }
                "GetFileType" => format!("type={} err={}", unsafe { GetFileType(h) }, le()),
                "NtQueryObject" => type_name(h),
                "CloseHandle" => {
                    let ok = unsafe { CloseHandle(h) };
                    format!("ok={ok} err={}", le())
                }
                "ReadFile" => {
                    let mut b = [0u8; 4];
                    let mut n2 = 0u32;
                    let ok = unsafe { ReadFile(h, b.as_mut_ptr(), 4, &mut n2, null_mut()) };
                    format!("ok={ok} err={}", le())
                }
                "WaitForSingleObject" => {
                    format!("r={:#x} err={}", unsafe { WaitForSingleObject(h, 0) }, le())
                }
                "DuplicateHandle" => format!("{:?}", dup(me(), h, me(), 0, DUPLICATE_SAME_ACCESS)),
                x => format!("unknown api {x}"),
            };
            o.push(format!("{} on {}: returned {r}", p[1], p[2]));
        }
        "time" => {
            // In-process timing of a self round trip on a given pair (h1 -> h2).
            let (a, b) = (hp(n(p[1])), hp(n(p[2])));
            let iters = 2000;
            let t = Instant::now();
            for _ in 0..iters {
                write_msg(a, vec![1u8; 64]).unwrap();
                read_msg(b, 4096, 1000).unwrap();
            }
            o.push(format!(
                "{} us per write+read",
                t.elapsed().as_secs_f64() * 1e6 / iters as f64
            ));
        }
        x => o.push(format!("unknown command {x}")),
    }
    o
}

fn echo(hin: H, hout: H) {
    let port = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, null_mut(), 0, 1) };
    unsafe {
        CreateIoCompletionPort(hin, port, 1, 1);
        CreateIoCompletionPort(hout, port, 2, 1);
    }
    let mut rov: Box<OVERLAPPED> = Box::new(unsafe { zeroed() });
    let mut wov: Box<OVERLAPPED> = Box::new(unsafe { zeroed() });
    let mut rbuf = vec![0u8; 65536 + 512];
    let mut wbuf: Vec<u8> = vec![];
    let post_read = |rov: &mut OVERLAPPED, rbuf: &mut Vec<u8>| unsafe {
        *rov = zeroed();
        let ok = ReadFile(hin, rbuf.as_mut_ptr(), rbuf.len() as u32, null_mut(), rov);
        ok != 0 || GetLastError() == ERROR_IO_PENDING
    };
    if !post_read(&mut rov, &mut rbuf) {
        return;
    }
    loop {
        let (mut bytes, mut key, mut pov) = (0u32, 0usize, null_mut::<OVERLAPPED>());
        let ok =
            unsafe { GetQueuedCompletionStatus(port, &mut bytes, &mut key, &mut pov, INFINITE) };
        if key == 1 {
            if ok == 0 {
                return;
            }
            wbuf.clear();
            wbuf.extend_from_slice(&rbuf[..bytes as usize]);
            unsafe {
                *wov = zeroed();
                let okw = WriteFile(
                    hout,
                    wbuf.as_ptr(),
                    wbuf.len() as u32,
                    null_mut(),
                    &mut *wov,
                );
                if okw == 0 && GetLastError() != ERROR_IO_PENDING {
                    return;
                }
            }
            if !post_read(&mut rov, &mut rbuf) {
                return;
            }
        }
    }
}
