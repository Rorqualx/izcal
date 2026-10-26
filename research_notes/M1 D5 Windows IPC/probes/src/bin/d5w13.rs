//! M1 D5 probe W13, for the fresh-pipe-per-move design (written from
//! Microsoft Learn reference pages: ReadFile, WriteFile, OVERLAPPED,
//! CancelIoEx, SetNamedPipeHandleState, GetProcessMemoryInfo; no header or
//! crate source).
//!
//! A compromised former holder can post several overlapped reads on a
//! receiver end before it gives the end away. W10 showed that pending reads
//! complete in issue order. This probe asks:
//! - W13a: with N reads pending and the handle pulled with close-source (as
//!   the broker does), how many of M later records does the old holder take?
//! - W13b: how many reads can one process leave pending, and what does each
//!   cost in its nonpaged-pool quota?
//! - W13c: a stale read with a 1-byte or 0-byte buffer: does it split a
//!   record, leaving a fragment for the next reader?
//! - W13d: can the broker stop the stale reads before it hands the pipe on?
//!   CancelIoEx on its own handles to that file object, nowait mode on its
//!   full-access handle, and (a writer's lever, for comparison) zero-length
//!   writes.
#![allow(clippy::missing_safety_doc)]

use std::io::{BufRead, BufReader, Read, Write};
use std::mem::zeroed;
use std::os::windows::io::AsRawHandle;
use std::process::{Command, Stdio};
use std::ptr::{null, null_mut};
use std::time::{Duration, Instant};

use d5probe::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Storage::FileSystem::ReadFile;
use windows_sys::Win32::System::IO::{CancelIoEx, OVERLAPPED};
use windows_sys::Win32::System::Pipes::{
    PIPE_NOWAIT, PIPE_READMODE_MESSAGE, SetNamedPipeHandleState,
};
use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, TerminateProcess};

fn say(tag: &str, s: String) {
    println!("[{tag}] {s}");
}

const STATUS_PENDING: usize = 0x103;

fn nonpaged() -> usize {
    let mut c: PROCESS_MEMORY_COUNTERS = unsafe { zeroed() };
    c.cb = size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
    c.QuotaNonPagedPoolUsage
}

/// The child (former holder): posts `n` overlapped reads of `cap` bytes on
/// the inherited server end, says how many are pending, waits for stdin to
/// close, then reports each read from its OVERLAPPED status block (the
/// handle may have been pulled from it by then).
fn child(v: &str, n: usize, cap: usize, wait_go: bool) {
    let h = hp(v.parse().unwrap());
    if wait_go {
        // In a job: the parent assigns it, then says go.
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
    }
    let np0 = nonpaged();
    let mut ops: Vec<(Box<OVERLAPPED>, Vec<u8>)> = Vec::with_capacity(n);
    let mut fail = 0u32;
    for _ in 0..n {
        let mut ov: Box<OVERLAPPED> = Box::new(unsafe { zeroed() });
        ov.hEvent = event();
        let mut buf = vec![0u8; cap.max(1)];
        let ok = unsafe { ReadFile(h, buf.as_mut_ptr(), cap as u32, null_mut(), &mut *ov) };
        let e = if ok == 0 { le() } else { 0 };
        if ok != 0 || e != ERROR_IO_PENDING {
            fail = if ok != 0 { 1 } else { e };
            ops.push((ov, buf));
            break;
        }
        ops.push((ov, buf));
    }
    let np1 = nonpaged();
    println!(
        "posted {} (stop: {}), nonpaged quota {} -> {} bytes",
        ops.len(),
        if fail == 0 {
            "none".to_owned()
        } else {
            format!("{fail}")
        },
        np0,
        np1
    );
    std::io::stdout().flush().unwrap();
    let mut s = String::new();
    let _ = std::io::stdin().read_to_string(&mut s);
    std::thread::sleep(Duration::from_millis(200));
    let mut got: Vec<String> = vec![];
    let mut errs: std::collections::BTreeMap<String, usize> = Default::default();
    let mut pending = 0usize;
    for (ov, buf) in &ops {
        let st = ov.Internal;
        let n = ov.InternalHigh;
        if st == STATUS_PENDING {
            pending += 1;
        } else if st == 0 {
            if n >= 4 {
                got.push(format!(
                    "{}",
                    u32::from_le_bytes(buf[..4].try_into().unwrap())
                ));
            } else {
                got.push(format!("{n}B"));
            }
        } else {
            *errs.entry(format!("{st:#x}/{n}B")).or_default() += 1;
        }
    }
    let shown = if got.len() > 12 {
        format!("{}..{} ({} records)", got[0], got[got.len() - 1], got.len())
    } else {
        format!("{got:?}")
    };
    println!("child took {shown}; other status {errs:?}; still pending {pending}");
}

struct Holder {
    c: std::process::Child,
    inh: isize,
}

/// Starts the former holder with an inheritable duplicate of `server`, and
/// waits until its reads are posted.
fn spawn_holder(server: H, n: usize, cap: usize) -> (Holder, String) {
    spawn_holder_in(server, n, cap, None)
}

/// A job with the given extended-limit flags and process/job memory
/// limits (JOBOBJECT_EXTENDED_LIMIT_INFORMATION, class 9).
fn job(flags: u32, process_mem: usize, job_mem: usize) -> H {
    use windows_sys::Win32::System::JobObjects::*;
    let j = unsafe { CreateJobObjectW(null(), null()) };
    let mut l: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    l.BasicLimitInformation.LimitFlags = flags | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    l.ProcessMemoryLimit = process_mem;
    l.JobMemoryLimit = job_mem;
    let ok = unsafe {
        SetInformationJobObject(
            j,
            JobObjectExtendedLimitInformation,
            &l as *const _ as *const core::ffi::c_void,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };
    assert!(ok != 0, "SetInformationJobObject err={}", le());
    j
}

fn spawn_holder_in(server: H, n: usize, cap: usize, in_job: Option<H>) -> (Holder, String) {
    let inh = dup(me(), server, me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    unsafe { SetHandleInformation(inh, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
    let mut c = Command::new(std::env::current_exe().unwrap())
        .args([
            "child",
            &hv(inh).to_string(),
            &n.to_string(),
            &cap.to_string(),
            if in_job.is_some() { "wait" } else { "go" },
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    unsafe { CloseHandle(inh) };
    if let Some(j) = in_job {
        use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;
        let ok = unsafe { AssignProcessToJobObject(j, hp(c.as_raw_handle() as isize)) };
        assert!(ok != 0, "AssignProcessToJobObject err={}", le());
        c.stdin.as_mut().unwrap().write_all(b"go\n").unwrap();
        c.stdin.as_mut().unwrap().flush().unwrap();
    }
    let mut line = String::new();
    BufReader::new(c.stdout.as_mut().unwrap())
        .read_line(&mut line)
        .unwrap();
    (Holder { c, inh: hv(inh) }, line.trim().to_owned())
}

/// The broker's pull: the holder's value, with DUPLICATE_CLOSE_SOURCE.
fn pull(h: &Holder) -> Result<H, u32> {
    let cp = hp(h.c.as_raw_handle() as isize);
    dup(
        cp,
        hp(h.inh),
        me(),
        0,
        DUPLICATE_SAME_ACCESS | DUPLICATE_CLOSE_SOURCE,
    )
}

fn finish_holder(mut h: Holder) -> String {
    drop(h.c.stdin.take());
    let _ = h.c.wait();
    let mut s = String::new();
    if let Some(mut o) = h.c.stdout.take() {
        let _ = o.read_to_string(&mut s);
    }
    s.trim().to_owned()
}

fn rec(seq: u32) -> Vec<u8> {
    let mut v = seq.to_le_bytes().to_vec();
    v.extend_from_slice(&[0xAB; 4]);
    v
}

/// Reads until an error, summarising the sequence numbers.
fn drain(h: H) -> String {
    let mut seqs = vec![];
    let mut tail = String::new();
    for _ in 0..5000 {
        match read_msg(h, 64, 3000) {
            Ok(b) if b.len() >= 4 => seqs.push(u32::from_le_bytes(b[..4].try_into().unwrap())),
            Ok(b) => {
                tail = format!("short ok len={}", b.len());
                break;
            }
            Err((e, b)) => {
                tail = format!("err={e} len={}", b.len());
                if !b.is_empty() {
                    tail += &format!(" bytes={:02x?}", b);
                }
                break;
            }
        }
    }
    let s = if seqs.len() > 12 {
        format!(
            "{}..{} ({} records)",
            seqs[0],
            seqs[seqs.len() - 1],
            seqs.len()
        )
    } else {
        format!("{seqs:?}")
    };
    format!("{s} then {tail}")
}

fn w13a(n: usize) {
    let (server, client, _) = make_pair(None, 65536, 65536).unwrap();
    let (h, posted) = spawn_holder(server, n, 64);
    let pulled = pull(&h);
    let m = n as u32 + 4;
    let t0 = Instant::now();
    let mut werr = 0;
    for s in 1..=m {
        if write_msg(client, rec(s)).is_err() {
            werr += 1;
        }
    }
    let wms = t0.elapsed().as_millis();
    unsafe { CloseHandle(client) };
    let rest = drain(server);
    say(
        "W13a",
        format!(
            "N={n} reads pending ({posted}); pull with close-source: {}; wrote 1..={m} ({werr} write errors, {wms} ms); new holder reads {rest}; {}",
            if pulled.is_ok() { "ok" } else { "FAILED" },
            finish_holder(h)
        ),
    );
    if let Ok(p) = pulled {
        unsafe { CloseHandle(p) };
    }
    unsafe { CloseHandle(server) };
}

/// W13e: does a job's commit limit bound the pending reads? The repo's
/// job sets only KILL_ON_JOB_CLOSE; the documented memory limits are
/// ProcessMemoryLimit and JobMemoryLimit (committed memory).
fn w13e(label: &str, flags: u32, pm: usize, jm: usize, n: usize) {
    let (server, client, _) = make_pair(None, 65536, 65536).unwrap();
    let j = job(flags, pm, jm);
    let (h, posted) = spawn_holder_in(server, n, 64, Some(j));
    let mut c = h.c;
    let kill = unsafe { TerminateProcess(hp(c.as_raw_handle() as isize), 0) };
    let _ = c.wait();
    say(
        "W13e",
        format!("{label}: asked for {n} pending reads: {posted}; holder killed (ok={kill})"),
    );
    unsafe {
        CloseHandle(j);
        CloseHandle(client);
        CloseHandle(server);
    }
}

fn w13b(n: usize) {
    let (server, client, _) = make_pair(None, 65536, 65536).unwrap();
    let (h, posted) = spawn_holder(server, n, 64);
    let mut c = h.c;
    let kill = unsafe { TerminateProcess(hp(c.as_raw_handle() as isize), 0) };
    let _ = c.wait();
    say(
        "W13b",
        format!("asked for {n} pending reads: {posted}; holder killed (ok={kill})"),
    );
    unsafe {
        CloseHandle(client);
        CloseHandle(server);
    }
}

fn w13c(cap: usize) {
    let (server, client, _) = make_pair(None, 65536, 65536).unwrap();
    let (h, posted) = spawn_holder(server, 1, cap);
    let pulled = pull(&h);
    let w = write_msg(client, rec(1));
    let w2 = write_msg(client, rec(2));
    std::thread::sleep(Duration::from_millis(200));
    unsafe { CloseHandle(client) };
    let rest = drain(server);
    say(
        "W13c",
        format!(
            "one stale read of {cap} bytes ({posted}), pulled {}; wrote two 8-byte records {w:?} {w2:?}; new holder reads {rest}; {}",
            pulled.is_ok(),
            finish_holder(h)
        ),
    );
    unsafe { CloseHandle(server) };
}

fn w13d() {
    // Lever 1 and 2: the broker's CancelIoEx and nowait mode.
    let (server, client, _) = make_pair(None, 65536, 65536).unwrap();
    let (h, posted) = spawn_holder(server, 4, 64);
    let pulled = pull(&h).unwrap();
    let c1 = unsafe { CancelIoEx(pulled, null()) };
    let e1 = le();
    let c2 = unsafe { CancelIoEx(server, null()) };
    let e2 = le();
    let mode = PIPE_READMODE_MESSAGE | PIPE_NOWAIT;
    let s1 = unsafe { SetNamedPipeHandleState(server, &mode, null(), null()) };
    let es = le();
    for s in 1..=6 {
        let _ = write_msg(client, rec(s));
    }
    unsafe { CloseHandle(client) };
    let rest = drain(server);
    say(
        "W13d",
        format!(
            "4 stale reads ({posted}); CancelIoEx(pulled) ok={c1} err={e1}; CancelIoEx(own server handle) ok={c2} err={e2}; SetNamedPipeHandleState(nowait) ok={s1} err={es}; wrote 1..=6; new holder reads {rest}; {}",
            finish_holder(h)
        ),
    );
    unsafe {
        CloseHandle(pulled);
        CloseHandle(server);
    }

    // Lever 3 (the writer's, not the broker's): zero-length messages.
    let (server, client, _) = make_pair(None, 65536, 65536).unwrap();
    let (h, posted) = spawn_holder(server, 4, 64);
    let pulled = pull(&h).unwrap();
    let mut zs = vec![];
    for _ in 0..4 {
        zs.push(write_msg(client, vec![]));
    }
    for s in 1..=2 {
        let _ = write_msg(client, rec(s));
    }
    unsafe { CloseHandle(client) };
    let rest = drain(server);
    say(
        "W13d",
        format!(
            "4 stale reads ({posted}); 4 zero-length writes {zs:?}, then 1..=2; new holder reads {rest}; {}",
            finish_holder(h)
        ),
    );
    unsafe {
        CloseHandle(pulled);
        CloseHandle(server);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("child") {
        child(
            &args[1],
            args[2].parse().unwrap(),
            args[3].parse().unwrap(),
            args.get(4).map(String::as_str) == Some("wait"),
        );
        return;
    }
    if args.first().map(String::as_str) == Some("jobs") {
        use windows_sys::Win32::System::JobObjects::*;
        const MB: usize = 1 << 20;
        w13e("job, no memory limit", 0, 0, 0, 131072);
        w13e(
            "job, ProcessMemoryLimit 32 MB",
            JOB_OBJECT_LIMIT_PROCESS_MEMORY,
            32 * MB,
            0,
            131072,
        );
        w13e(
            "job, JobMemoryLimit 32 MB",
            JOB_OBJECT_LIMIT_JOB_MEMORY,
            0,
            32 * MB,
            131072,
        );
        w13e(
            "job, ProcessMemoryLimit 8 MB",
            JOB_OBJECT_LIMIT_PROCESS_MEMORY,
            8 * MB,
            0,
            131072,
        );
        w13b(262144);
        return;
    }
    for n in [1usize, 8, 64, 1024] {
        for _ in 0..3 {
            w13a(n);
        }
    }
    for n in [1024usize, 16384, 65536] {
        w13b(n);
    }
    for cap in [1usize, 0] {
        for _ in 0..3 {
            w13c(cap);
        }
    }
    for _ in 0..3 {
        w13d();
    }
}
