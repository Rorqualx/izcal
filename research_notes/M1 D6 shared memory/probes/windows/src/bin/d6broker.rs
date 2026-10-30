//! M1 D6 probe broker (Medium IL, the user's own token). Children are
//! d6child under the repo's sandbox (lockdown token, AppContainer without
//! capabilities, kill-on-close job, Untrusted), one container each.
//! Sources: Microsoft Learn reference pages only.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

use std::ptr::null;
use std::time::{Duration, Instant};

use d6probe::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Memory::*;
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

fn say(tag: &str, s: impl AsRef<str>) {
    println!("[{tag}] {}", s.as_ref());
}
fn says(tag: &str, v: Vec<String>) {
    for s in v {
        say(tag, format!("  child: {s}"));
    }
}
fn start(container: &str) -> Child {
    let mut c = launch(&Launch {
        container,
        ..Default::default()
    })
    .expect("launch");
    c.resume();
    for h in c.collect(Duration::from_secs(20)) {
        say("child", format!("pid={} {h}", c.pid));
    }
    c
}
fn field(lines: &[String], key: &str) -> String {
    lines
        .iter()
        .flat_map(|l| l.split_whitespace())
        .find_map(|p| p.strip_prefix(&format!("{key}=")).map(str::to_string))
        .unwrap_or_default()
}
fn section(sddl: &str, size: u32) -> H {
    let sa = sa_from_sddl(sddl, false);
    let h = unsafe {
        CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            &sa,
            PAGE_READWRITE | SEC_COMMIT,
            0,
            size,
            null(),
        )
    };
    assert!(!h.is_null(), "CreateFileMappingW {}", le());
    h
}

type Cmp = unsafe extern "system" fn(H, H) -> i32;
fn compare(a: H, b: H) -> i32 {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    let m = unsafe { GetModuleHandleW(w("kernelbase.dll").as_ptr()) };
    let f = unsafe { GetProcAddress(m, c"CompareObjectHandles".as_ptr().cast()) }
        .expect("CompareObjectHandles");
    let f: Cmp = unsafe { std::mem::transmute(f) };
    unsafe { f(a, b) }
}

fn main() {
    let which: Vec<String> = std::env::args().skip(1).collect();
    let on = |s: &str| which.is_empty() || which.iter().any(|w| w == s);
    if on("s1") {
        s1_s4();
    }
    if on("s5") {
        s5_costs();
    }
    if on("s6") {
        s6_commit();
    }
    if on("s7") {
        s7_counts();
    }
    if on("s9") {
        s9_dacl();
    }
    if on("s10") {
        s10_charge();
    }
    if on("s11") {
        s11_offer();
    }
}

/// S9: a 0x6 handle the broker duplicates into a child whose AppContainer
/// SID is not in the section's DACL (DACL = the broker's user GA only).
fn s9_dacl() {
    let mut c = start("izcal.d6probe.producer");
    let sddl = format!("D:P(A;;GA;;;{})", user_sid());
    let h = section(&sddl, 65536);
    say("S9", format!("section SD {}", sddl_of(h)));
    let ch = c.give(h, FILE_MAP_READ | FILE_MAP_WRITE, 0).unwrap();
    says("S9", c.cmd(&format!("query {ch}")));
    says("S9", c.cmd(&format!("map {ch} rw")));
    says("S9", c.cmd("write 0 s9-child"));
    let bv = unsafe { MapViewOfFile(h, FILE_MAP_READ, 0, 0, 0) };
    let b = unsafe { std::slice::from_raw_parts(bv.Value as *const u8, 8) }.to_vec();
    unsafe { UnmapViewOfFile(bv) };
    say(
        "S9",
        format!("broker reads {:?}", String::from_utf8_lossy(&b)),
    );
    say(
        "S9",
        "the child tries to widen past 0x6 (DACL names only the broker):",
    );
    says("S9", c.cmd(&format!("widen {ch}")));
    // A read-only handle in the same child cannot be widened to write.
    let rh = c.give(h, FILE_MAP_READ, 0).unwrap();
    say("S9", "a FILE_MAP_READ handle in that child:");
    says("S9", c.cmd(&format!("widen {rh}")));
    c.kill();
}

fn mem(p: H) -> String {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX,
    };
    let mut m: PROCESS_MEMORY_COUNTERS_EX = unsafe { std::mem::zeroed() };
    m.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
    let ok = unsafe { GetProcessMemoryInfo(p, (&raw mut m).cast(), m.cb) };
    let mut ms: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    ms.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    unsafe { GlobalMemoryStatusEx(&mut ms) };
    format!(
        "ok={ok} PrivateUsage={} KiB WorkingSet={} KiB PagefileUsage={} KiB | system commit used={} MiB",
        m.PrivateUsage >> 10,
        m.WorkingSetSize >> 10,
        m.PagefileUsage >> 10,
        (ms.ullTotalPageFile - ms.ullAvailPageFile) >> 20
    )
}

/// S10: who is charged for a 64 MiB SEC_COMMIT section: the creator, the
/// mapper, the toucher, or only the system?
fn s10_charge() {
    let mut c = start("izcal.d6probe.consumer");
    let me_ = unsafe { windows_sys::Win32::System::Threading::GetCurrentProcess() };
    let size = 64u32 << 20;
    say(
        "S10",
        format!("before: broker {} | child {}", mem(me_), mem(c.process)),
    );
    let sddl = format!("D:P(A;;GA;;;{})", user_sid());
    let h = section(&sddl, size);
    say(
        "S10",
        format!(
            "broker created 64 MiB SEC_COMMIT: broker {} | child {}",
            mem(me_),
            mem(c.process)
        ),
    );
    let ch = c.give(h, FILE_MAP_READ | FILE_MAP_WRITE, 0).unwrap();
    says("S10", c.cmd(&format!("map {ch} rw")));
    say(
        "S10",
        format!(
            "child mapped: broker {} | child {}",
            mem(me_),
            mem(c.process)
        ),
    );
    says("S10", c.cmd(&format!("fill {size}")));
    say(
        "S10",
        format!(
            "child wrote every page: broker {} | child {}",
            mem(me_),
            mem(c.process)
        ),
    );
    says("S10", c.cmd("unmap"));
    say(
        "S10",
        format!(
            "child unmapped: broker {} | child {}",
            mem(me_),
            mem(c.process)
        ),
    );
    unsafe { CloseHandle(h) };
    say(
        "S10",
        format!(
            "broker closed (child still has a handle): broker {} | child {}",
            mem(me_),
            mem(c.process)
        ),
    );
    c.kill();
    say("S10", format!("child exited: broker {}", mem(me_)));
}

/// S11: can a consumer with a read view offer or discard pages of a
/// shared section, and does the producer's data change?
fn s11_offer() {
    let mut c = start("izcal.d6probe.consumer");
    let sddl = format!("D:P(A;;GA;;;{})", user_sid());
    let size = 1u32 << 20;
    let h = section(&sddl, size);
    let bv = unsafe { MapViewOfFile(h, FILE_MAP_WRITE, 0, 0, 0) };
    let bp = bv.Value as *mut u8;
    for i in (0..size as usize).step_by(4096) {
        unsafe { *bp.add(i) = 0x5a };
    }
    let ch = c.give(h, FILE_MAP_READ, 0).unwrap();
    says("S11", c.cmd(&format!("map {ch} r")));
    says("S11", c.cmd(&format!("touch {size}")));
    says("S11", c.cmd(&format!("offer {size}")));
    let bad = (0..size as usize)
        .step_by(4096)
        .filter(|&i| unsafe { *bp.add(i) } != 0x5a)
        .count();
    say(
        "S11",
        format!(
            "broker's view after the consumer's offer/discard: {bad} of {} pages changed",
            size / 4096
        ),
    );
    says("S11", c.cmd("read 0 1"));
    unsafe { UnmapViewOfFile(bv) };
    c.kill();
}

/// S7: can the broker see that a child still holds a view of a section,
/// after pulling the child's only handle with DUPLICATE_CLOSE_SOURCE?
fn s7_counts() {
    let mut c = start("izcal.d6probe.producer");
    let sid = field(&c.cmd("info"), "ac-sid");
    let sddl = format!(
        "D:P(A;;GA;;;{})(A;;{:#x};;;{sid})",
        user_sid(),
        FILE_MAP_READ | FILE_MAP_WRITE
    );
    let h = section(&sddl, 65536);
    let show = |label: &str, x: H| {
        say(
            "S7",
            format!("{label}: (HandleCount, PointerCount) = {:?}", counts(x)),
        )
    };
    show("broker's handle only, no view", h);
    let v = unsafe { MapViewOfFile(h, FILE_MAP_READ, 0, 0, 0) };
    show("broker's handle + one broker view", h);
    unsafe { UnmapViewOfFile(v) };
    show("broker view unmapped", h);
    let ch = c.give(h, FILE_MAP_READ | FILE_MAP_WRITE, 0).unwrap();
    show("given to the child (two handles)", h);
    says("S7", c.cmd(&format!("map {ch} rw")));
    show("child mapped a view", h);
    let pulled = dup(
        c.process,
        hp(ch),
        me(),
        0,
        DUPLICATE_SAME_ACCESS | DUPLICATE_CLOSE_SOURCE,
    )
    .unwrap();
    show(
        "child's handle pulled with CLOSE_SOURCE; child keeps its view",
        h,
    );
    says("S7", c.cmd("write 0 after-pull"));
    let bv = unsafe { MapViewOfFile(h, FILE_MAP_READ, 0, 0, 0) };
    let b = unsafe { std::slice::from_raw_parts(bv.Value as *const u8, 10) }.to_vec();
    unsafe { UnmapViewOfFile(bv) };
    say(
        "S7",
        format!(
            "broker reads {:?}: the child's view still writes",
            String::from_utf8_lossy(&b)
        ),
    );
    says("S7", c.cmd("unmap"));
    show("child unmapped its view", h);
    unsafe { CloseHandle(pulled) };
    show("pulled handle closed", h);
    c.kill();
}

/// S1 pair with a per-SID DACL; S2 what the consumer can learn; S3 the
/// broker identifies a relayed section; S4 lifetime after the broker
/// lets go.
fn s1_s4() {
    let mut prod = start("izcal.d6probe.producer");
    let mut cons = start("izcal.d6probe.consumer");
    let psid = field(&prod.cmd("info"), "ac-sid");
    let csid = field(&cons.cmd("info"), "ac-sid");
    say(
        "S1",
        format!("producer AC SID {psid}; consumer AC SID {csid}"),
    );
    let size = 100_000u32; // not a page multiple
    let sddl = format!(
        "D:P(A;;GA;;;{})(A;;{:#x};;;{psid})",
        user_sid(),
        FILE_MAP_READ | FILE_MAP_WRITE
    );
    let h = section(&sddl, size);
    say(
        "S1",
        format!(
            "broker section {} size {size} SD {}",
            describe(h),
            sddl_of(h)
        ),
    );
    let ph = prod.give(h, FILE_MAP_READ | FILE_MAP_WRITE, 0).unwrap();
    let ch = cons.give(h, FILE_MAP_READ, 0).unwrap();
    says("S1", prod.cmd(&format!("map {ph} rw")));
    says("S1", prod.cmd("write 0 hello-1"));
    says("S1", cons.cmd(&format!("map {ch} r")));
    says("S1", cons.cmd("read 0 7"));
    says("S1", prod.cmd("write 0 hello-2"));
    says("S1", cons.cmd("read 0 7"));
    say("S1", "consumer tries to widen:");
    says("S1", cons.cmd(&format!("widen {ch}")));
    say("S1", "producer tries to widen beyond its grant:");
    says("S1", prod.cmd(&format!("widen {ph}")));
    say("S2", "what the consumer can learn about its handle:");
    says("S2", cons.cmd(&format!("query {ch}")));
    say("S2", "the same for the producer's handle:");
    says("S2", prod.cmd(&format!("query {ph}")));
    // S3: pull handles back from the consumer and compare with ours.
    let pulled = dup(cons.process, hp(ch), me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    say(
        "S3",
        format!(
            "pulled consumer's handle: {} CompareObjectHandles(ours)={}",
            describe(pulled),
            compare(pulled, h)
        ),
    );
    let made = cons.cmd("mksection");
    says("S3", made.clone());
    let mv: isize = made
        .iter()
        .find_map(|l| {
            l.strip_prefix("made ")
                .and_then(|r| r.split_whitespace().next())
                .map(|v| v.parse().unwrap())
        })
        .unwrap_or(0);
    if mv != 0 {
        let p2 = dup(cons.process, hp(mv), me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
        say(
            "S3",
            format!(
                "pulled a child-made section: {} CompareObjectHandles(ours)={}",
                describe(p2),
                compare(p2, h)
            ),
        );
    }
    // S4: the broker closes its handle; the producer exits; the consumer keeps reading.
    unsafe { CloseHandle(h) };
    say(
        "S4",
        format!("broker closed its handle; producer killed: {}", prod.kill()),
    );
    says("S4", cons.cmd("read 0 7"));
    say("S4", format!("consumer: {}", cons.kill()));
}

fn s5_costs() {
    for size in [64 * 1024u32, 4 * 1024 * 1024] {
        let n = 2000;
        let sddl = format!("D:P(A;;GA;;;{})", user_sid());
        let t = Instant::now();
        for _ in 0..n {
            let h = section(&sddl, size);
            unsafe { CloseHandle(h) };
        }
        let create = t.elapsed().as_nanos() as f64 / n as f64 / 1000.0;
        let h = section(&sddl, size);
        let t = Instant::now();
        for _ in 0..n {
            let v = unsafe { MapViewOfFile(h, FILE_MAP_READ, 0, 0, 0) };
            unsafe { UnmapViewOfFile(v) };
        }
        let mapc = t.elapsed().as_nanos() as f64 / n as f64 / 1000.0;
        let t = Instant::now();
        let m = n / 10;
        for _ in 0..m {
            let v = unsafe { MapViewOfFile(h, FILE_MAP_READ, 0, 0, 0) };
            let p = v.Value as *const u8;
            let mut s = 0u64;
            for i in 0..(size as usize / 4096) {
                s = s.wrapping_add(unsafe { std::ptr::read_volatile(p.add(i * 4096)) } as u64);
            }
            std::hint::black_box(s);
            unsafe { UnmapViewOfFile(v) };
        }
        let touched = t.elapsed().as_nanos() as f64 / m as f64 / 1000.0;
        say(
            "S5",
            format!(
                "{} KiB: create(with DACL)+close {create:.2} µs; map+unmap {mapc:.2} µs; map+read every page+unmap {touched:.2} µs",
                size / 1024
            ),
        );
        unsafe { CloseHandle(h) };
    }
}

fn s6_commit() {
    let mut ms: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    ms.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    unsafe { GlobalMemoryStatusEx(&mut ms) };
    say(
        "S6",
        format!(
            "commit limit (ullTotalPageFile) {} MiB, available {} MiB, physical {} MiB",
            ms.ullTotalPageFile >> 20,
            ms.ullAvailPageFile >> 20,
            ms.ullTotalPhys >> 20
        ),
    );
    let sddl = format!("D:P(A;;GA;;;{})", user_sid());
    let sa = sa_from_sddl(&sddl, false);
    for gib in [1u64, 16, 256] {
        let bytes = gib << 30;
        for (label, flag) in [("SEC_COMMIT", SEC_COMMIT), ("SEC_RESERVE", SEC_RESERVE)] {
            let h = unsafe {
                CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    &sa,
                    PAGE_READWRITE | flag,
                    (bytes >> 32) as u32,
                    bytes as u32,
                    null(),
                )
            };
            say(
                "S6",
                format!(
                    "{gib} GiB {label}: {}",
                    if h.is_null() {
                        format!("FAILED err={}", le())
                    } else {
                        "created".into()
                    }
                ),
            );
            if !h.is_null() {
                unsafe { CloseHandle(h) };
            }
        }
    }
}
