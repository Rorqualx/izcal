//! M1 D5 probe broker (Medium IL, the user's own token). Runs the probes
//! named on the command line (default: all) and prints one block per
//! probe. Children are d5child under the repo's sandbox (lockdown token,
//! AppContainer without capabilities, kill-on-close job, Untrusted).
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::time::{Duration, Instant};

use d5probe::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::Win32::System::IO::*;
use windows_sys::Win32::System::Memory::*;
use windows_sys::Win32::System::Pipes::*;
use windows_sys::Win32::System::Threading::*;

const RECV: u32 = FILE_READ_DATA | SYNCHRONIZE;
const SEND: u32 = FILE_WRITE_DATA | SYNCHRONIZE;
const GENERIC_WRITE_: u32 = 0x4000_0000;

fn say(tag: &str, s: impl AsRef<str>) {
    println!("[{tag}] {}", s.as_ref());
}
fn says(tag: &str, v: Vec<String>) {
    for s in v {
        say(tag, format!("  child: {s}"));
    }
}
fn user_dacl() -> String {
    // Only the broker's user SID; no label (unlabelled = Medium, NW).
    format!("D:P(A;;GA;;;{})", user_sid())
}
fn start(container: &str) -> Child {
    let mut c = launch(&Launch {
        container,
        ..Default::default()
    })
    .expect("launch");
    c.resume();
    let hello = c.collect(Duration::from_secs(20));
    for h in hello {
        say("child", format!("pid={} {h}", c.pid));
    }
    c
}
fn info_field(lines: &[String], key: &str) -> String {
    for l in lines {
        for part in l.split_whitespace() {
            if let Some(v) = part.strip_prefix(&format!("{key}=")) {
                return v.to_string();
            }
        }
    }
    String::new()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("p26a") {
        return p26_broker_strict(true);
    }
    if args.first().map(|s| s.as_str()) == Some("p26b") {
        return p26_broker_strict(false);
    }
    if args.first().map(|s| s.as_str()) == Some("--medium") {
        return relaunch_medium(&args[1..]);
    }
    let all = args.is_empty();
    say(
        "env",
        format!(
            "broker token: {}",
            d5probe::sandbox_windows::probe_line(None)
        ),
    );
    let want = |p: &str| all || args.iter().any(|a| a == p);
    say(
        "env",
        format!(
            "broker pid={} user={} session={}",
            std::process::id(),
            user_sid(),
            session_id()
        ),
    );
    if want("p1") {
        p1_to_p4();
    }
    if want("p5") {
        p5();
    }
    if want("pair") {
        pair_into_child();
    }
    if want("p6") {
        p6_p7();
    }
    if want("p8") {
        p8();
    }
    if want("p9") {
        p9();
    }
    if want("p10") {
        p10();
    }
    if want("p11") {
        p11();
    }
    if want("p12") {
        p12();
    }
    if want("p13") {
        p13();
    }
    if want("p14") {
        p14();
    }
    if want("p16") {
        p16_p17();
    }
    if want("p18") {
        p18_p19_p15();
    }
    if want("p20") {
        p20();
    }
    if want("p20b") {
        p20b();
    }
    if want("p22") {
        p22();
    }
    if want("p22x") {
        p22x();
    }
    if want("p26x") {
        p26x();
    }
    if want("p23") {
        p23_p25();
    }
    if want("p26") {
        p26();
    }
    if want("p27") {
        p27();
    }
    if want("w1") {
        w1();
    }
    if want("w2") {
        w2();
    }
    if want("w3") {
        w3();
    }
    if want("w4") {
        w4();
    }
    if want("w4prof") {
        w4_profiles();
    }
    if want("w5") {
        w5();
    }
    if want("w6") {
        w6();
    }
    if want("w7") {
        w7();
    }
    if want("rtt") {
        rtt();
    }
    if want("cleanup") {
        cleanup();
    }
}

// ---------------- P1-P4: pipes created inside the sandbox ----------------
fn p1_to_p4() {
    let t = "P1-4";
    let mut a = start("izcal.d5probe.a");
    let info = a.cmd("info");
    says(t, info.clone());
    let sess = info_field(&info, "session");
    let acpath = info_field(&info, "ac-path");
    let acsid = info_field(&info, "ac-sid");
    let r = rand_hex();
    let names = [
        format!(r"\\.\pipe\izcal-d5-{r}-plain"),
        format!(r"\\.\pipe\LOCAL\izcal-d5-{r}-local"),
        format!(r"\\.\pipe\Sessions\{sess}\{acpath}\izcal-d5-{r}-full"),
    ];
    let mut created = vec![];
    for nm in &names {
        say(
            t,
            format!(
                "P1/P2 child CreateNamedPipeW({nm}) [INBOUND|OVERLAPPED|FIRST_INSTANCE, MESSAGE|READMODE_MESSAGE|REJECT_REMOTE, 1,0,0,0,NULL]"
            ),
        );
        let rep = a.cmd(&format!("mkpipe {nm}"));
        says(t, rep.clone());
        let h = info_field(&rep, "h");
        let fname = info_field(&rep, "filename");
        if !h.is_empty() {
            created.push((nm.clone(), h, fname));
        }
    }
    // P3 with the default SD, then with an explicit DACL.
    for (nm, _, _) in &created {
        say(t, format!("P3 same child CreateFileW({nm}, GENERIC_WRITE)"));
        says(t, a.cmd(&format!("open {nm} {GENERIC_WRITE_:#x}")));
    }
    let sd = format!("D:(A;;GA;;;{acsid})(A;;GA;;;RC)(A;;GA;;;OW)");
    for (i, nm) in names.iter().enumerate() {
        let nm2 = format!("{nm}-sd{i}");
        say(t, format!("P3 child CreateNamedPipeW({nm2}) with SD {sd}"));
        let rep = a.cmd(&format!("mkpipe_sd {nm2} {sd}"));
        says(t, rep.clone());
        if !info_field(&rep, "h").is_empty() {
            says(t, a.cmd(&format!("open {nm2} {GENERIC_WRITE_:#x}")));
        }
    }
    // P4: the broker inspects what the child created and tries to open it.
    let mut b = start("izcal.d5probe.b");
    for (nm, h, fname) in &created {
        match dup(a.process, hp(h.parse().unwrap()), me(), READ_CONTROL, 0) {
            Ok(bh) => say(
                t,
                format!(
                    "P4 broker GetSecurityInfo on child's {nm}: {} [{}]",
                    sddl_of(bh),
                    describe(bh)
                ),
            ),
            Err(e) => say(
                t,
                format!("P4 broker dup(READ_CONTROL) of child's {nm} FAILED err={e}"),
            ),
        }
        let bname = format!(r"\\.\pipe{fname}");
        for target in [nm.clone(), bname.clone()] {
            let hb = unsafe {
                CreateFileW(
                    w(&target).as_ptr(),
                    GENERIC_WRITE_,
                    0,
                    null(),
                    OPEN_EXISTING,
                    FILE_FLAG_OVERLAPPED,
                    null_mut(),
                )
            };
            say(
                t,
                format!(
                    "P4 broker CreateFileW({target}, GENERIC_WRITE): {}",
                    if hb == INVALID_HANDLE_VALUE {
                        format!("FAILED err={}", le())
                    } else {
                        "OK".into()
                    }
                ),
            );
            if hb != INVALID_HANDLE_VALUE {
                unsafe { CloseHandle(hb) };
            }
        }
        say(
            t,
            format!("P4 child B (other AppContainer) CreateFileW({bname})"),
        );
        says(t, b.cmd(&format!("open {bname} {GENERIC_WRITE_:#x}")));
    }
    // P4 again on a child pipe nobody has connected (so a busy instance
    // cannot hide the access result).
    let free = format!(r"\\.\pipe\LOCAL\izcal-d5-{}-free", rand_hex());
    let rep = a.cmd(&format!("mkpipe {free}"));
    says(t, rep.clone());
    let bname = format!(r"\\.\pipe{}", info_field(&rep, "filename"));
    for acc in [GENERIC_WRITE_, SEND, FILE_READ_DATA | SYNCHRONIZE] {
        let hb = unsafe {
            CreateFileW(
                w(&bname).as_ptr(),
                acc,
                0,
                null(),
                OPEN_EXISTING,
                FILE_FLAG_OVERLAPPED,
                null_mut(),
            )
        };
        say(
            t,
            format!(
                "P4 broker CreateFileW(unconnected child pipe, {acc:#x}): {}",
                if hb == INVALID_HANDLE_VALUE {
                    format!("FAILED err={}", le())
                } else {
                    format!("OK {}", describe(hb))
                }
            ),
        );
        if hb != INVALID_HANDLE_VALUE {
            unsafe { CloseHandle(hb) };
            break;
        }
    }
    let free2 = format!(r"\\.\pipe\LOCAL\izcal-d5-{}-free2", rand_hex());
    let rep = a.cmd(&format!("mkpipe {free2}"));
    let bname2 = format!(r"\\.\pipe{}", info_field(&rep, "filename"));
    for acc in [GENERIC_WRITE_, SEND] {
        say(
            t,
            format!("P4 child B CreateFileW(unconnected child-A pipe, {acc:#x}):"),
        );
        says(t, b.cmd(&format!("open {bname2} {acc:#x}")));
    }
    a.kill();
    b.kill();
}

// ---------------- P5: broker pipes, child opening by name ----------------
fn p5() {
    let t = "P5";
    let mut a = start("izcal.d5probe.a");
    let acsid = info_field(&a.cmd("info"), "ac-sid");
    let cases: Vec<(&str, Option<String>)> = vec![
        ("NULL SD", None),
        ("explicit DACL user-only", Some(user_dacl())),
        (
            "DACL user + AC SID, no label",
            Some(format!("D:P(A;;GA;;;{})(A;;GRGW;;;{acsid})", user_sid())),
        ),
        (
            "DACL user + AC SID, label Untrusted NW",
            Some(format!(
                "D:P(A;;GA;;;{})(A;;GRGW;;;{acsid})S:(ML;;NW;;;S-1-16-0)",
                user_sid()
            )),
        ),
    ];
    for (label, sddl) in cases {
        let name = format!(r"\\.\pipe\izcal-d5-{}", rand_hex());
        let sa = sddl.as_ref().map(|s| sa_from_sddl(s, false));
        let h = unsafe {
            CreateNamedPipeW(
                w(&name).as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
                MSG,
                1,
                0,
                0,
                0,
                sa.as_ref().map_or(null(), |s| s as *const _),
            )
        };
        if h == INVALID_HANDLE_VALUE {
            say(t, format!("{label}: CreateNamedPipeW FAILED err={}", le()));
            continue;
        }
        say(t, format!("{label}: broker pipe SD = {}", sddl_of(h)));
        for acc in [GENERIC_WRITE_, SEND, FILE_READ_DATA | SYNCHRONIZE] {
            say(t, format!("{label}: child CreateFileW(access {acc:#x})"));
            says(t, a.cmd(&format!("open {name} {acc:#x}")));
        }
        // First-instance check: a second instance of the same name.
        let h2 = unsafe {
            CreateNamedPipeW(
                w(&name).as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
                MSG,
                1,
                0,
                0,
                0,
                null(),
            )
        };
        say(
            t,
            format!(
                "{label}: second CreateNamedPipeW same name (FIRST_INSTANCE): {}",
                if h2 == INVALID_HANDLE_VALUE {
                    format!("FAILED err={}", le())
                } else {
                    "SUCCEEDED".into()
                }
            ),
        );
        unsafe { CloseHandle(h) };
    }
    // FIRST_INSTANCE against an existing pipe that allows 2 instances.
    let name = format!(r"\\.\pipe\izcal-d5-{}", rand_hex());
    let h1 = unsafe {
        CreateNamedPipeW(
            w(&name).as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
            MSG,
            2,
            0,
            0,
            0,
            null(),
        )
    };
    let h2 = unsafe {
        CreateNamedPipeW(
            w(&name).as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
            MSG,
            2,
            0,
            0,
            0,
            null(),
        )
    };
    say(
        t,
        format!(
            "existing pipe (max 2, no FIRST_INSTANCE) ok={}; then CreateNamedPipeW with FIRST_INSTANCE: {}",
            h1 != INVALID_HANDLE_VALUE,
            if h2 == INVALID_HANDLE_VALUE {
                format!("FAILED err={}", le())
            } else {
                "SUCCEEDED".into()
            }
        ),
    );
    let h3 = unsafe {
        CreateNamedPipeW(
            w(&name).as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
            MSG,
            2,
            0,
            0,
            0,
            null(),
        )
    };
    say(
        t,
        format!(
            "  and a second instance without FIRST_INSTANCE: {}",
            if h3 == INVALID_HANDLE_VALUE {
                format!("FAILED err={}", le())
            } else {
                "SUCCEEDED".into()
            }
        ),
    );
    a.kill();
}

// ---------------- the design pair, both ends into a child ----------------
fn pair_into_child() {
    let t = "PAIR";
    let (s, c, name) = make_pair(Some(&user_dacl()), 65536, 65536).expect("pair");
    say(
        t,
        format!(
            "broker pair {name}: server {} | client {} | SD {}",
            describe(s),
            describe(c),
            sddl_of(s)
        ),
    );
    say(
        t,
        format!("server {}; client {}", pipe_info(s), pipe_info(c)),
    );
    let c2 = unsafe {
        CreateFileW(
            w(&name).as_ptr(),
            GENERIC_WRITE_,
            0,
            null(),
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            null_mut(),
        )
    };
    say(
        t,
        format!(
            "second client CreateFileW: {}",
            if c2 == INVALID_HANDLE_VALUE {
                format!("FAILED err={}", le())
            } else {
                "SUCCEEDED".into()
            }
        ),
    );
    let mut a = launch(&Launch::default()).expect("launch");
    // P21: duplicate into the suspended child (moves: CLOSE_SOURCE).
    let rs = a.give(s, RECV, DUPLICATE_CLOSE_SOURCE);
    let wc = a.give(c, SEND, DUPLICATE_CLOSE_SOURCE);
    say(
        t,
        format!(
            "P21 DuplicateHandle into SUSPENDED sandboxed child: receiver(server)={rs:?} sender(client)={wc:?}"
        ),
    );
    a.resume();
    says(t, a.collect(Duration::from_secs(20)));
    let (rs, wc) = (rs.unwrap(), wc.unwrap());
    says(t, a.cmd(&format!("access {rs}")));
    says(t, a.cmd(&format!("access {wc}")));
    says(t, a.cmd(&format!("write {wc} 3 100")));
    says(t, a.cmd(&format!("read {rs} 4096 1000")));
    says(t, a.cmd(&format!("read {rs} 4096 1000")));
    says(t, a.cmd(&format!("read {rs} 4096 1000")));
    says(t, a.cmd(&format!("read {rs} 4096 300")));
    a.kill();
}

// ---------------- P6/P7: when does an overlapped write pend ----------------
fn p6_p7() {
    let t = "P6-7";
    for buf in [0u32, 4096, 65536] {
        for len in [1usize, 4096, 65536 + 64] {
            let (s, c, _) = make_pair(Some(&user_dacl()), buf, buf).unwrap();
            let info = pipe_info(c);
            let mut ops = vec![];
            let mut pended_at = None;
            let mut err = 0;
            let mut bytes = 0usize;
            for i in 0..20000usize {
                let (ok, e, op) = write_start(c, vec![(i % 251) as u8; len]);
                ops.push(op);
                if !ok {
                    if e == ERROR_IO_PENDING {
                        pended_at = Some(i);
                    } else {
                        err = e;
                    }
                    break;
                }
                bytes += len;
            }
            say(
                t,
                format!(
                    "bufs={buf} msg={len}B: client {info}; writes completed immediately before the first pend: {} msgs / {bytes} B; first non-immediate: {}",
                    pended_at.map_or("none".into(), |x| x.to_string()),
                    if pended_at.is_some() {
                        "ERROR_IO_PENDING".to_string()
                    } else {
                        format!("err={err}")
                    }
                ),
            );
            // P7: read everything; check order and boundaries.
            let total = ops.len();
            let mut good = 0;
            let mut bad = vec![];
            for i in 0..total {
                match read_msg(s, len + 64, 2000) {
                    Ok(b) if b.len() == len && b.iter().all(|x| *x == (i % 251) as u8) => good += 1,
                    r => {
                        bad.push(format!("#{i}: {}", fmt_read(&r)));
                        break;
                    }
                }
            }
            let last = ops.last_mut().unwrap();
            let fin = finish(c, last, 2000);
            say(
                t,
                format!(
                    "  read back {good}/{total} intact in order {bad:?}; pended write completion after reads: {fin:?}"
                ),
            );
            for op in ops {
                close_ov(op);
            }
            unsafe {
                CloseHandle(s);
                CloseHandle(c);
            }
        }
    }
}

// ---------------- P8/P8b ----------------
fn p8() {
    let t = "P8";
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let (_, _, mut op) = write_start(c, vec![7u8; 65537]);
    let r1 = read_msg(s, 65536, 2000);
    let r2 = read_msg(s, 65536, 2000);
    say(
        t,
        format!(
            "64KiB+1 message into 64KiB read: first {} ; second read {} ; write {:?}",
            fmt_read(&r1),
            fmt_read(&r2),
            finish(c, &mut op, 2000)
        ),
    );
    let wz = write_msg(c, vec![]);
    let rz = read_msg(s, 100, 1000);
    say(
        t,
        format!("0-byte message: write {wz:?} read {}", fmt_read(&rz)),
    );
    for size in [1usize << 20, 16 << 20, 64 << 20] {
        let (ok, e, mut op) = write_start(c, vec![9u8; size]);
        let r = read_msg(s, size + 16, 10000);
        say(
            t,
            format!(
                "P8b {size} B single message: write immediate={ok} err={e}; read {}; write completion {:?}",
                fmt_read(&r),
                finish(c, &mut op, 5000)
            ),
        );
        close_ov(op);
    }
    unsafe {
        CloseHandle(s);
        CloseHandle(c);
    }
}

// ---------------- P9: close with queued messages ----------------
fn p9() {
    let t = "P9";
    for (label, writer_is_client) in [
        ("writer=client, reader=server", true),
        ("writer=server, reader=client", false),
    ] {
        let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let (wr, rd) = if writer_is_client { (c, s) } else { (s, c) };
        for i in 0..3u8 {
            write_msg(wr, vec![i; 10 + i as usize]).unwrap();
        }
        unsafe { CloseHandle(wr) };
        say(
            t,
            format!(
                "{label}: 3 queued then CloseHandle(writer); reader sees: {:?}",
                read_all(rd, 4096, 1000)
            ),
        );
        let w2 = write_msg(rd, vec![1]);
        say(
            t,
            format!("{label}:   reader-side write after peer close: {w2:?}"),
        );
        unsafe { CloseHandle(rd) };
    }
    // Writer in a sandboxed child, killed by its job (kill-on-close).
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let mut a = launch(&Launch::default()).unwrap();
    let wc = a.give(c, SEND, DUPLICATE_CLOSE_SOURCE).unwrap();
    a.resume();
    a.collect(Duration::from_secs(20));
    says(t, a.cmd(&format!("write {wc} 3 10")));
    let st = a.kill();
    say(
        t,
        format!(
            "writer in child, child killed by closing its job ({st}); reader sees: {:?}",
            read_all(s, 4096, 1000)
        ),
    );
    unsafe { CloseHandle(s) };
}

// ---------------- P10: close the reader ----------------
fn p10() {
    let t = "P10";
    for (label, reader_is_server) in [
        ("reader=server closed, writer=client", true),
        ("reader=client closed, writer=server", false),
    ] {
        let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let (wr, rd) = if reader_is_server { (c, s) } else { (s, c) };
        write_msg(wr, vec![1; 10]).unwrap();
        unsafe { CloseHandle(rd) };
        let r1 = write_msg(wr, vec![2; 10]);
        let r2 = write_msg(wr, vec![3; 10]);
        let rr = read_msg(wr, 100, 500);
        say(
            t,
            format!(
                "{label}: next write {r1:?}, again {r2:?}; read on writer end {}",
                fmt_read(&rr)
            ),
        );
        unsafe { CloseHandle(wr) };
    }
    // A write pending when the reader closes.
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let mut ops = vec![];
    for _ in 0..1000 {
        let (ok, e, op) = write_start(c, vec![5; 4096]);
        ops.push(op);
        if !ok {
            say(
                t,
                format!("filled: write #{} returned err={e}", ops.len() - 1),
            );
            break;
        }
    }
    let last = ops.last_mut().unwrap();
    say(
        t,
        format!(
            "pending write state just before the reader closes: {:?}",
            finish(c, last, 300)
        ),
    );
    unsafe { CloseHandle(s) };
    say(
        t,
        format!(
            "pending write after reader CloseHandle completes with: {:?}",
            finish(c, last, 3000)
        ),
    );
    let after = write_msg(c, vec![6; 10]);
    say(t, format!("next write after that: {after:?}"));
    unsafe { CloseHandle(c) };
}

// ---------------- P11: a copy elsewhere keeps the end open ----------------
fn p11() {
    let t = "P11";
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let mut a = launch(&Launch::default()).unwrap();
    let wc = a.give(c, SEND, 0).unwrap(); // a copy, not a move
    a.resume();
    a.collect(Duration::from_secs(20));
    unsafe { CloseHandle(c) };
    say(
        t,
        format!(
            "broker closed its writer, child still holds a copy; read on server: {}",
            fmt_read(&read_msg(s, 100, 1000))
        ),
    );
    says(t, a.cmd(&format!("close {wc}")));
    say(
        t,
        format!(
            "after the child closed its copy; read on server: {}",
            fmt_read(&read_msg(s, 100, 1000))
        ),
    );
    a.kill();
    unsafe { CloseHandle(s) };
}

// ---------------- P12: inheritable handle into a list-less CreateProcess ----------------
fn p12() {
    let t = "P12";
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    unsafe { SetHandleInformation(c, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
    let mut x = launch(&Launch {
        sandbox: false,
        no_list: true,
        ..Default::default()
    })
    .unwrap();
    x.resume();
    x.collect(Duration::from_secs(20));
    say(
        t,
        format!(
            "inheritable pipe end value {} in broker; plain CreateProcess(bInheritHandles=TRUE, no list) child checks it:",
            hv(c)
        ),
    );
    says(t, x.cmd(&format!("checkh {}", hv(c))));
    x.cmd("exit");
    let mut y = launch(&Launch {
        sandbox: false,
        ..Default::default()
    })
    .unwrap();
    y.resume();
    y.collect(Duration::from_secs(20));
    say(
        t,
        "same, but the child is created with a HANDLE_LIST that omits it:",
    );
    says(t, y.cmd(&format!("checkh {}", hv(c))));
    y.cmd("exit");
    unsafe {
        CloseHandle(s);
        CloseHandle(c);
    }
}

// ---------------- P13: client PID after the end moves ----------------
fn p13() {
    let t = "P13";
    let pid_of = |s: H| {
        let (mut cp, mut sp, mut cs) = (0u32, 0u32, 0u32);
        let a = unsafe { GetNamedPipeClientProcessId(s, &mut cp) };
        let ea = le();
        let b = unsafe { GetNamedPipeServerProcessId(s, &mut sp) };
        let c = unsafe { GetNamedPipeClientSessionId(s, &mut cs) };
        format!(
            "ClientPid={} (ok={a} err={}) ServerPid={sp} (ok={b}) ClientSession={cs} (ok={c})",
            cp,
            if a == 0 { ea } else { 0 }
        )
    };
    let mut a = start("izcal.d5probe.a");
    let mut b = start("izcal.d5probe.b");
    say(
        t,
        format!(
            "broker pid={} childA pid={} childB pid={}",
            std::process::id(),
            a.pid,
            b.pid
        ),
    );
    // (a) the design: broker opens the client, moves it to A, then to B.
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    say(t, format!("(a) broker-opened client: {}", pid_of(s)));
    let ca = a.give(c, SEND, DUPLICATE_CLOSE_SOURCE).unwrap();
    says(t, a.cmd(&format!("write {ca} 1 5")));
    let _ = read_msg(s, 100, 1000);
    say(
        t,
        format!("(a) after move into A and A wrote: {}", pid_of(s)),
    );
    let cb = hv(dup(a.process, hp(ca), b.process, SEND, DUPLICATE_CLOSE_SOURCE).unwrap());
    says(t, b.cmd(&format!("write {cb} 1 5")));
    let _ = read_msg(s, 100, 1000);
    say(t, format!("(a) after move A->B and B wrote: {}", pid_of(s)));
    unsafe { CloseHandle(s) };
    // (b) child A connects itself (DACL grants its AppContainer SID).
    let acsid = info_field(&a.cmd("info"), "ac-sid");
    let sd = format!(
        "D:P(A;;GA;;;{})(A;;GRGW;;;{acsid})S:(ML;;NW;;;S-1-16-0)",
        user_sid()
    );
    let name = format!(r"\\.\pipe\izcal-d5-{}", rand_hex());
    let sa = sa_from_sddl(&sd, false);
    let s = unsafe {
        CreateNamedPipeW(
            w(&name).as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
            MSG,
            1,
            0,
            0,
            0,
            &sa,
        )
    };
    let rep = a.cmd(&format!("open {name} {SEND:#x}"));
    says(t, rep.clone());
    let ch = info_field(&rep, "h");
    if !ch.is_empty() {
        say(t, format!("(b) A opened by name: {}", pid_of(s)));
        let cb = hv(dup(
            a.process,
            hp(ch.parse().unwrap()),
            b.process,
            SEND,
            DUPLICATE_CLOSE_SOURCE,
        )
        .unwrap());
        says(t, b.cmd(&format!("write {cb} 1 5")));
        let _ = read_msg(s, 100, 1000);
        say(t, format!("(b) after move A->B and B wrote: {}", pid_of(s)));
        a.kill();
        say(t, format!("(b) after A exited: {}", pid_of(s)));
    }
    b.kill();
}

// ---------------- P14: which values DuplicateHandle takes from a child ----------------
fn p14() {
    let t = "P14";
    let mut a = start("izcal.d5probe.a");
    says(t, a.cmd("info"));
    let my_tid = unsafe { GetCurrentThreadId() };
    let child_tid = unsafe { GetThreadId(a.thread) };
    say(
        t,
        format!(
            "broker pid={} tid={my_tid}; child pid={} main tid={child_tid}",
            std::process::id(),
            a.pid
        ),
    );
    let vals: [isize; 13] = [0, -1, -2, -3, -4, -5, -6, -7, -8, -9, 4, 8, 0x7FFF_FFFF];
    for v in vals {
        match dup(a.process, hp(v), me(), 0, DUPLICATE_SAME_ACCESS) {
            Ok(h) => {
                let ty = type_name(h);
                let extra = match ty.as_str() {
                    "Process" => format!(" GetProcessId={}", unsafe { GetProcessId(h) }),
                    "Thread" => format!(
                        " GetThreadId={} owner-pid={}",
                        unsafe { GetThreadId(h) },
                        unsafe { GetProcessIdOfThread(h) }
                    ),
                    _ => String::new(),
                };
                say(
                    t,
                    format!("DuplicateHandle(child, {v}) OK -> {}{extra}", describe(h)),
                );
                unsafe { CloseHandle(h) };
            }
            Err(e) => say(t, format!("DuplicateHandle(child, {v}) FAILED err={e}")),
        }
    }
    a.kill();
}

// ---------------- P16/P17: classification ----------------
fn p16_p17() {
    let t = "P16";
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    // NtQueryObject class numbers: shape of each class's output.
    for class in 0..4 {
        let (st, rl, buf) = nt_query_raw(s, class);
        say(
            t,
            format!(
                "NtQueryObject(pipe server, class {class}): status={:#x} ReturnLength={rl} first words={:#x} {:#x}",
                st as u32, buf[0], buf[1]
            ),
        );
    }
    let mut anon_r: H = null_mut();
    let mut anon_w: H = null_mut();
    unsafe { CreatePipe(&mut anon_r, &mut anon_w, null(), 0) };
    let sock = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    use std::os::windows::io::AsRawSocket;
    let sock_h = sock.as_raw_socket() as H;
    let file = unsafe {
        CreateFileW(
            w(&std::env::current_exe().unwrap().to_string_lossy()).as_ptr(),
            FILE_READ_DATA,
            FILE_SHARE_READ,
            null(),
            OPEN_EXISTING,
            0,
            null_mut(),
        )
    };
    let sect = unsafe {
        CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            null(),
            PAGE_READWRITE,
            0,
            65536,
            null(),
        )
    };
    let ev = event();
    let proc_ = dup(me(), me(), me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    let thr = dup(
        me(),
        unsafe { GetCurrentThread() },
        me(),
        0,
        DUPLICATE_SAME_ACCESS,
    )
    .unwrap();
    let mut tok: H = null_mut();
    unsafe { windows_sys::Win32::System::Threading::OpenProcessToken(me(), 0x8, &mut tok) };
    let port = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, null_mut(), 0, 1) };
    for (label, h) in [
        ("pipe server", s),
        ("pipe client", c),
        ("anon read", anon_r),
        ("anon write", anon_w),
        ("socket", sock_h),
        ("disk file", file),
        ("section", sect),
        ("event", ev),
        ("process", proc_),
        ("thread", thr),
        ("token", tok),
        ("iocp", port),
    ] {
        say(
            t,
            format!(
                "{label}: {} GetFileType={} {}",
                describe(h),
                unsafe { GetFileType(h) },
                pipe_info(h)
            ),
        );
    }
    // P17
    let t = "P17";
    let cmp = |a: H, b: H| {
        let ok = unsafe { CompareObjectHandles(a, b) };
        format!("{ok} (err={})", if ok == 0 { le() } else { 0 })
    };
    let s2 = dup(me(), s, me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    let mut a = launch(&Launch::default()).unwrap();
    let in_child = a.give(s, RECV, 0).unwrap();
    a.resume();
    a.collect(Duration::from_secs(20));
    let back = dup(a.process, hp(in_child), me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    say(
        t,
        format!(
            "server vs own dup: {}; server vs client of same instance: {}; server vs copy pulled back from child: {}; server vs other pair's server: {}",
            cmp(s, s2),
            cmp(s, c),
            cmp(s, back),
            {
                let (s9, _c9, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
                cmp(s, s9)
            }
        ),
    );
    let n = 100_000;
    let t0 = Instant::now();
    for _ in 0..n {
        unsafe { CompareObjectHandles(s, back) };
    }
    let same = t0.elapsed().as_secs_f64() * 1e9 / n as f64;
    let t0 = Instant::now();
    for _ in 0..n {
        unsafe { CompareObjectHandles(s, c) };
    }
    say(
        t,
        format!(
            "CompareObjectHandles cost: {same:.0} ns (same object), {:.0} ns (different)",
            t0.elapsed().as_secs_f64() * 1e9 / n as f64
        ),
    );
    a.kill();
}

// ---------------- P18/P19/P15: narrowed ends in the child ----------------
fn p18_p19_p15() {
    let t = "P18";
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let (s2, _c2b, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let mut a = launch(&Launch::default()).unwrap();
    let r = a.give(s, RECV, 0).unwrap();
    let r_attr = a.give(s2, RECV | FILE_READ_ATTRIBUTES, 0).unwrap();
    let wsend = a.give(c, SEND, 0).unwrap();
    let (s4, c2, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let r_fresh = a.give(s4, RECV, 0).unwrap();
    a.resume();
    a.collect(Duration::from_secs(20));
    say(
        t,
        format!("receiver FILE_READ_DATA|SYNCHRONIZE ({RECV:#x}) ops:"),
    );
    says(t, a.cmd(&format!("ops {r}")));
    say(t, format!("receiver + FILE_READ_ATTRIBUTES ops:"));
    says(t, a.cmd(&format!("ops {r_attr}")));
    say(
        t,
        format!("sender FILE_WRITE_DATA|SYNCHRONIZE ({SEND:#x}) ops:"),
    );
    says(t, a.cmd(&format!("ops {wsend}")));
    write_msg(c2, vec![3; 33]).unwrap();
    say(
        t,
        "fresh receiver (FILE_READ_DATA|SYNCHRONIZE) read via completion port (broker wrote 33 B):",
    );
    says(t, a.cmd(&format!("iocpread {r_fresh} 4096")));
    say(t, "sender write (child), broker reads:");
    // the child's sender is the client end; the broker reads from... the
    // server end is in the child as `r`, so read there.
    says(t, a.cmd(&format!("write {wsend} 1 21")));
    says(t, a.cmd(&format!("read {r} 4096 1000")));
    // P19: widen within the child.
    let t = "P19";
    for acc in [
        FILE_READ_DATA | FILE_WRITE_DATA | SYNCHRONIZE,
        RECV | FILE_WRITE_ATTRIBUTES,
        RECV | FILE_READ_ATTRIBUTES,
        GENERIC_WRITE_,
        0x1F01FF,
    ] {
        says(t, a.cmd(&format!("widen {r} {acc:#x}")));
    }
    // P15: pull back with SAME_ACCESS and compare GrantedAccess.
    let t = "P15";
    says(t, a.cmd(&format!("access {r}")));
    let back = dup(a.process, hp(r), me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    say(
        t,
        format!(
            "pulled back into broker with SAME_ACCESS: {}",
            describe(back)
        ),
    );
    a.kill();
}

// ---------------- P20: idle wakeups with 64 pipes ----------------
fn p20() {
    let t = "P20";
    let mut a = launch(&Launch::default()).unwrap();
    let mut vals = vec![];
    let mut clients = vec![];
    for _ in 0..64 {
        let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        vals.push(a.give(s, RECV, DUPLICATE_CLOSE_SOURCE).unwrap().to_string());
        clients.push(c);
    }
    a.resume();
    a.collect(Duration::from_secs(20));
    says(t, a.cmd(&format!("idle {}", vals.join(" "))));
    std::thread::sleep(Duration::from_secs(2));
    say(
        t,
        format!("child pid={}; counting context switches for 30 s", a.pid),
    );
    if std::env::var_os("D5_P20_EXTERNAL").is_some() {
        // The counter runs from the (High) ssh shell: WMI perf data is
        // denied to the Medium token.
        say(t, "holding the idle child 45 s for the external counter");
        std::thread::sleep(Duration::from_secs(45));
        says(t, a.cmd("wakes"));
        says(t, a.cmd("post"));
        write_msg(clients[5], vec![1; 8]).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        say(t, "after one message on pipe 5:");
        says(t, a.cmd("wakes"));
        a.kill();
        return;
    }
    let ps = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("wakeups-windows.ps1");
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&ps)
        .args(["-Name", "d5child", "-Seconds", "30"])
        .output()
        .unwrap();
    say(t, String::from_utf8_lossy(&out.stdout).trim());
    say(t, String::from_utf8_lossy(&out.stderr).trim());
    says(t, a.cmd("wakes"));
    says(t, a.cmd("post"));
    write_msg(clients[5], vec![1; 8]).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    say(t, "after one message on pipe 5:");
    says(t, a.cmd("wakes"));
    a.kill();
}

// ---------------- P20b: cancel and close with a pending read ----------------
fn p20b() {
    let t = "P20b";
    for cancel_first in [true, false] {
        let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let port = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, null_mut(), 0, 1) };
        unsafe { CreateIoCompletionPort(s, port, 1, 1) };
        let mut ov: Box<OVERLAPPED> = Box::new(unsafe { zeroed() });
        let mut buf = vec![0u8; 4096];
        let ok = unsafe { ReadFile(s, buf.as_mut_ptr(), 4096, null_mut(), &mut *ov) };
        let e = le();
        if cancel_first {
            let ok = unsafe { CancelIoEx(s, &*ov) };
            say(t, format!("read posted ok={ok}/{e}; CancelIoEx ok={ok}"));
        }
        let mut packets = vec![];
        let mut gq = |ms| {
            let (mut b, mut k, mut p) = (0u32, 0usize, null_mut::<OVERLAPPED>());
            let ok = unsafe { GetQueuedCompletionStatus(port, &mut b, &mut k, &mut p, ms) };
            format!(
                "ok={ok} err={} bytes={b} key={k} ov={}",
                if ok == 0 { le() } else { 0 },
                !p.is_null()
            )
        };
        if cancel_first {
            packets.push(format!("after cancel: {}", gq(1000)));
            unsafe { CloseHandle(s) };
            packets.push(format!("after close: {}", gq(500)));
        } else {
            unsafe { CloseHandle(s) };
            packets.push(format!("after CloseHandle alone: {}", gq(1000)));
            packets.push(format!("again: {}", gq(500)));
        }
        say(t, format!("cancel_first={cancel_first}: {packets:?}"));
        unsafe {
            CloseHandle(c);
            CloseHandle(port);
        }
    }
}

// ---------------- P22: handle-list route ----------------
fn p22() {
    let t = "P22";
    for sandbox in [false, true] {
        let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let (s3, _c3, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        for h in [s, c, s3] {
            unsafe { SetHandleInformation(h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
        }
        say(
            t,
            format!(
                "sandbox={sandbox} broker: listed {} ({}), {} ({}); unlisted inheritable {}",
                hv(s),
                describe(s),
                hv(c),
                describe(c),
                hv(s3)
            ),
        );
        let mut a = launch(&Launch {
            sandbox,
            extra_inherit: vec![s, c],
            ..Default::default()
        })
        .unwrap();
        a.resume();
        says(t, a.collect(Duration::from_secs(20)));
        says(t, a.cmd(&format!("checkh {} {}", hv(s), hv(c))));
        // The unlisted value: CloseHandle reports 6 without raising (the
        // sandboxed child runs with strict handle checks, see P26x).
        says(t, a.cmd(&format!("inv CloseHandle {}", hv(s3))));
        a.kill();
        for h in [s, c, s3] {
            unsafe { CloseHandle(h) };
        }
    }
}

fn p22x() {
    let t = "P22x";
    for (label, kind, nolower) in [
        ("server end", 0, false),
        ("client end", 1, false),
        ("anon pipe read end", 2, false),
        ("event", 3, false),
        ("server end, no Untrusted lowering", 0, true),
    ] {
        let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let (mut ar, mut aw): (H, H) = (null_mut(), null_mut());
        unsafe { CreatePipe(&mut ar, &mut aw, null(), 0) };
        let ev = event();
        let h = [s, c, ar, ev][kind];
        unsafe { SetHandleInformation(h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
        let mut a = launch(&Launch {
            extra_inherit: vec![h],
            args: if nolower {
                "--nolower".into()
            } else {
                String::new()
            },
            ..Default::default()
        })
        .unwrap();
        a.resume();
        let hello = a.collect(Duration::from_secs(20));
        let ok = hello.iter().any(|l| l.starts_with("hello"));
        say(
            t,
            format!(
                "sandboxed child, HANDLE_LIST + {label} ({}): {}",
                describe(h),
                if ok {
                    a.cmd(&format!("checkh {}", hv(h))).join(" ")
                } else {
                    hello.join(" ")
                }
            ),
        );
        a.kill();
    }
    for (label, two, unlisted, sandbox) in [
        ("two listed (server+client)", true, false, true),
        ("one listed + one unlisted inheritable", false, true, true),
        ("two listed + unlisted, not sandboxed", true, true, false),
        ("two listed (server + event)", false, false, true),
    ] {
        let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let (s3, _c3, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let ev = event();
        let mut list = vec![s];
        if two {
            list.push(c)
        } else if !unlisted {
            list.push(ev)
        }
        for h in [s, c, ev] {
            unsafe { SetHandleInformation(h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
        }
        if unlisted {
            unsafe { SetHandleInformation(s3, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
        }
        let mut a = launch(&Launch {
            sandbox,
            extra_inherit: list.clone(),
            ..Default::default()
        })
        .unwrap();
        a.resume();
        let hello = a.collect(Duration::from_secs(20));
        let ok = hello.iter().any(|l| l.starts_with("hello"));
        say(
            t,
            format!(
                "sandbox={sandbox} {label}: {}",
                if ok {
                    a.cmd(&format!(
                        "checkh {} {}",
                        list.iter()
                            .map(|h| hv(*h).to_string())
                            .collect::<Vec<_>>()
                            .join(" "),
                        hv(s3)
                    ))
                    .join(" | ")
                } else {
                    hello.join(" ")
                }
            ),
        );
        a.kill();
        for h in [s, c, s3, ev] {
            unsafe { CloseHandle(h) };
        }
    }
    // Pinpoint: ends of different pipes vs both ends of one pipe.
    for (label, case) in [
        ("listed: server of pipe 1 + client of pipe 2", 0),
        (
            "listed: server of pipe 1; its own client inheritable, unlisted",
            1,
        ),
        ("listed: server + client of pipe 1", 2),
        ("listed: server of pipe 1 + server of pipe 2", 3),
    ] {
        let (s1, c1, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let (s2, c2, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let (list, inh): (Vec<H>, Vec<H>) = match case {
            0 => (vec![s1, c2], vec![s1, c2]),
            1 => (vec![s1], vec![s1, c1]),
            2 => (vec![s1, c1], vec![s1, c1]),
            _ => (vec![s1, s2], vec![s1, s2]),
        };
        for h in &inh {
            unsafe { SetHandleInformation(*h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
        }
        let mut a = launch(&Launch {
            extra_inherit: list.clone(),
            ..Default::default()
        })
        .unwrap();
        a.resume();
        let hello = a.collect(Duration::from_secs(20));
        let ok = hello.iter().any(|l| l.starts_with("hello"));
        say(
            t,
            format!(
                "PIN sandboxed, {label}: {}",
                if ok {
                    "child started OK".to_string()
                } else {
                    hello.join(" ")
                }
            ),
        );
        a.kill();
        for h in [s1, c1, s2, c2] {
            unsafe { CloseHandle(h) };
        }
    }
    for (label, case) in [
        (
            "T1 listed s1; unlisted inheritable: server of another pipe",
            1,
        ),
        ("T2 listed s1; unlisted inheritable: event", 2),
        ("T3 listed event2; unlisted inheritable: event", 3),
        ("T4 listed s1; unlisted inheritable: anon pipe read end", 4),
        (
            "T5 listed event2; unlisted inheritable: server of another pipe",
            5,
        ),
    ] {
        let (s1, c1, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let (s2, c2, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let (ev, ev2) = (event(), event());
        let (mut ar, mut aw): (H, H) = (null_mut(), null_mut());
        unsafe { CreatePipe(&mut ar, &mut aw, null(), 0) };
        let (listed, unl) = match case {
            1 => (s1, s2),
            2 => (s1, ev),
            3 => (ev2, ev),
            4 => (s1, ar),
            _ => (ev2, s2),
        };
        for h in [listed, unl] {
            unsafe { SetHandleInformation(h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
        }
        let mut a = launch(&Launch {
            extra_inherit: vec![listed],
            ..Default::default()
        })
        .unwrap();
        a.resume();
        let hello = a.collect(Duration::from_secs(20));
        let ok = hello.iter().any(|l| l.starts_with("hello"));
        say(
            t,
            format!(
                "TT sandboxed, {label} (listed {} unlisted {}): {}",
                hv(listed),
                hv(unl),
                if ok {
                    "child started OK".to_string()
                } else {
                    hello.join(" ")
                }
            ),
        );
        a.kill();
        for h in [s1, c1, s2, c2, ev, ev2, ar, aw] {
            unsafe { CloseHandle(h) };
        }
    }
    // Inheritable named-pipe server ends (each from its own pair):
    // listed L, unlisted U.
    for (l_n, u_n) in [
        (0usize, 3usize),
        (1, 2),
        (2, 1),
        (3, 0),
        (0, 2),
        (1, 1),
        (2, 0),
        (1, 3),
        (4, 0),
    ] {
        let pairs: Vec<(H, H)> = (0..l_n + u_n)
            .map(|_| {
                let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
                (s, c)
            })
            .collect();
        for (s, _) in &pairs {
            unsafe { SetHandleInformation(*s, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
        }
        let listed: Vec<H> = pairs[..l_n].iter().map(|p| p.0).collect();
        let mut a = launch(&Launch {
            extra_inherit: listed,
            ..Default::default()
        })
        .unwrap();
        a.resume();
        let hello = a.collect(Duration::from_secs(20));
        let ok = hello.iter().any(|l| l.starts_with("hello"));
        say(
            t,
            format!(
                "PIPES sandboxed listed-pipe-ends={l_n} unlisted-inheritable-pipe-ends={u_n}: {}",
                if ok {
                    "OK".to_string()
                } else {
                    hello.join(" ")
                }
            ),
        );
        a.kill();
        for (s, c) in pairs {
            unsafe {
                CloseHandle(s);
                CloseHandle(c);
            }
        }
    }
    // Client ends (CreateFileW-opened) among the inheritable handles.
    // spec: (listed, unlisted) as strings of s=server c=client e=event.
    for (ls, us) in [
        ("s", "ce"),
        ("s", "ee"),
        ("c", "ee"),
        ("cee", ""),
        ("c", ""),
        ("cc", ""),
        ("ccc", ""),
        ("", "ccc"),
        ("sss", ""),
        ("", "cee"),
    ] {
        let mut keep: Vec<H> = vec![];
        let mut mk = |k: char| -> H {
            let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
            keep.push(s);
            keep.push(c);
            let h = match k {
                's' => s,
                'c' => c,
                _ => {
                    let e = event();
                    keep.push(e);
                    e
                }
            };
            unsafe { SetHandleInformation(h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
            h
        };
        let listed: Vec<H> = ls.chars().map(&mut mk).collect();
        let _unl: Vec<H> = us.chars().map(&mut mk).collect();
        let mut a = launch(&Launch {
            extra_inherit: listed,
            ..Default::default()
        })
        .unwrap();
        a.resume();
        let hello = a.collect(Duration::from_secs(20));
        let ok = hello.iter().any(|l| l.starts_with("hello"));
        say(
            t,
            format!(
                "CLI sandboxed listed='{ls}' unlisted-inheritable='{us}': {}",
                if ok {
                    "OK".to_string()
                } else {
                    hello.join(" ")
                }
            ),
        );
        a.kill();
        for h in keep {
            unsafe { CloseHandle(h) };
        }
    }
    // Both ends of ONE pipe inheritable, plus a third inheritable handle.
    for (label, case) in [
        ("listed s1,c1 + unlisted event", 0),
        ("listed s1 + unlisted c1,event", 1),
        ("listed s1,c1,event", 2),
        ("listed s1,c1 only", 3),
        ("unlisted s1,c1,event; nothing extra listed", 4),
        ("listed s1,c1 + unlisted s2", 5),
        ("listed s1,c1 + unlisted event; NOT sandboxed", 6),
    ] {
        let (s1, c1, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let (s2, c2, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let ev = event();
        let (listed, unl): (Vec<H>, Vec<H>) = match case {
            0 | 6 => (vec![s1, c1], vec![ev]),
            1 => (vec![s1], vec![c1, ev]),
            2 => (vec![s1, c1, ev], vec![]),
            3 => (vec![s1, c1], vec![]),
            4 => (vec![], vec![s1, c1, ev]),
            _ => (vec![s1, c1], vec![s2]),
        };
        for h in listed.iter().chain(unl.iter()) {
            unsafe { SetHandleInformation(*h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
        }
        let mut a = launch(&Launch {
            sandbox: case != 6,
            extra_inherit: listed,
            ..Default::default()
        })
        .unwrap();
        a.resume();
        let hello = a.collect(Duration::from_secs(20));
        let ok = hello.iter().any(|l| l.starts_with("hello"));
        say(
            t,
            format!(
                "SAME {label}: {}",
                if ok {
                    "OK".to_string()
                } else {
                    hello.join(" ")
                }
            ),
        );
        a.kill();
        for h in [s1, c1, s2, c2, ev] {
            unsafe { CloseHandle(h) };
        }
    }
    // Minimal: no extra listed handles; one unlisted inheritable event.
    for (label, sandbox, args) in [
        ("sandboxed", true, ""),
        ("sandboxed, no Untrusted lowering", true, "--nolower"),
        ("not sandboxed", false, ""),
    ] {
        for unlisted in [false, true] {
            let ev = event();
            if unlisted {
                unsafe { SetHandleInformation(ev, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
            }
            let mut a = launch(&Launch {
                sandbox,
                args: args.into(),
                ..Default::default()
            })
            .unwrap();
            a.resume();
            let hello = a.collect(Duration::from_secs(20));
            let ok = hello.iter().any(|l| l.starts_with("hello"));
            say(
                t,
                format!(
                    "MIN {label}, only stdin/stdout listed, unlisted inheritable event present={unlisted}: {}",
                    if ok {
                        "child started OK".to_string()
                    } else {
                        hello.join(" ")
                    }
                ),
            );
            a.kill();
            unsafe { CloseHandle(ev) };
        }
    }
}

// ---------------- P23-P25: sections ----------------
fn p23_p25() {
    let t = "P23";
    let mut a = start("izcal.d5probe.a");
    let acsid = info_field(&a.cmd("info"), "ac-sid");
    let mk = |sddl: Option<String>, name: Option<&str>| unsafe {
        let sa = sddl.map(|s| sa_from_sddl(&s, false));
        let h = CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            sa.as_ref().map_or(null(), |s| s as *const _),
            PAGE_READWRITE | SEC_COMMIT,
            0,
            65536,
            name.map_or(null(), |n| w(n).leak().as_ptr()),
        );
        assert!(!h.is_null(), "CreateFileMappingW {}", le());
        let v = MapViewOfFile(h, FILE_MAP_WRITE, 0, 0, 0);
        std::ptr::copy_nonoverlapping(b"hello".as_ptr(), v.Value as *mut u8, 5);
        (h, v.Value as *mut u8)
    };
    let cases = [
        ("P23 default SD", None),
        (
            "P24 DACL: user GA, AppContainer FILE_MAP_READ only",
            Some(format!(
                "D:P(A;;GA;;;{})(A;;{:#x};;;{acsid})",
                user_sid(),
                FILE_MAP_READ
            )),
        ),
        (
            "P24 DACL: user GA only (AppContainer absent)",
            Some(format!("D:P(A;;GA;;;{})", user_sid())),
        ),
    ];
    for (label, sddl) in cases {
        let (h, view) = mk(sddl, None);
        say(
            t,
            format!("{label}: broker section {} SD {}", describe(h), sddl_of(h)),
        );
        let ch = a.give(h, FILE_MAP_READ, 0).unwrap();
        says(t, a.cmd(&format!("section {ch}")));
        let b100 = unsafe { *view.add(100) };
        let b200 = unsafe { *view.add(200) };
        say(
            t,
            format!(
                "{label}: broker view byte100={:?} byte200={:?} (W = child widened and wrote; C = copy view leaked)",
                b100 as char, b200 as char
            ),
        );
    }
    // Reopen by name: a named section in the broker's session namespace.
    let nm = format!("izcal-d5-{}", rand_hex());
    let (_h, _) = mk(None, Some(&format!("Local\\{nm}")));
    let sess = session_id();
    for (label, path) in [
        ("Local\\", format!("Local\\{nm}")),
        (
            "full",
            format!("\\Sessions\\{sess}\\BaseNamedObjects\\{nm}"),
        ),
        ("Global\\", format!("Global\\{nm}")),
    ] {
        say(
            t,
            format!("reopen named section by {label} name {path} with FILE_MAP_WRITE:"),
        );
        says(t, a.cmd(&format!("openmap {path} {:#x}", FILE_MAP_WRITE)));
    }
    let t = "P25";
    says(t, a.cmd("mksection"));
    a.kill();
}

// ---------------- P26: strict handle checks ----------------
fn p26_broker_strict(child_source: bool) {
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
    println!("strict policy set ok={ok} err={}", le());
    let a = launch(&Launch::default()).unwrap();
    a.resume();
    if child_source {
        for v in [0x1234_5678isize, 0x1000, -2] {
            let r = dup(a.process, hp(v), me(), 0, DUPLICATE_SAME_ACCESS);
            println!("broker(strict) DuplicateHandle(child, {v:#x}) -> {r:?} (no exception)");
        }
        let r = dup(
            a.process,
            hp(0x1234_5678),
            me(),
            0,
            DUPLICATE_CLOSE_SOURCE | DUPLICATE_SAME_ACCESS,
        );
        println!(
            "broker(strict) DuplicateHandle(child, 0x12345678, CLOSE_SOURCE) -> {r:?} (no exception)"
        );
        let r = dup(me(), hp(0x1234_5678), a.process, 0, DUPLICATE_SAME_ACCESS);
        println!(
            "broker(strict) DuplicateHandle(self, 0x12345678 -> child) -> {r:?} (no exception)"
        );
    } else {
        let ok = unsafe { CloseHandle(hp(0x1234_5678)) };
        println!(
            "broker(strict) CloseHandle(0x12345678) returned {ok} err={} (no exception)",
            le()
        );
        let mut f = 0u32;
        let ok = unsafe { GetHandleInformation(hp(0x1234_5678), &mut f) };
        println!(
            "broker(strict) GetHandleInformation(0x12345678) returned {ok} err={} (no exception)",
            le()
        );
    }
    a.kill();
}
fn p26() {
    let t = "P26";
    for sub in ["p26a", "p26b"] {
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .arg(sub)
            .output()
            .unwrap();
        say(
            t,
            format!(
                "{sub}: exit status {:?} ({:#x})",
                out.status.code(),
                out.status.code().unwrap_or(0) as u32
            ),
        );
        for l in String::from_utf8_lossy(&out.stdout).lines() {
            say(t, format!("  {l}"));
        }
    }
    let mut a = start("izcal.d5probe.a");
    say(t, "child: strict on, then CloseHandle(0x12345678):");
    says(t, a.cmd("strict 0x12345678"));
    unsafe { WaitForSingleObject(a.process, 2000) };
    say(t, format!("child state after: {}", a.state()));
    // And a child DuplicateHandle source from a strict child (broker not strict).
    let mut b = start("izcal.d5probe.a");
    says(t, b.cmd("strict"));
    let r = dup(b.process, hp(0x1234_5678), me(), 0, DUPLICATE_SAME_ACCESS);
    say(
        t,
        format!(
            "broker (not strict) DuplicateHandle(strict child, 0x12345678) -> {r:?}; child state {}",
            b.state()
        ),
    );
    a.kill();
    b.kill();
}

/// Which calls on an invalid handle value kill the sandboxed child, and
/// its strict-handle-check policy as launched.
fn p26x() {
    let t = "P26x";
    for sandbox in [true, false] {
        let mut a = launch(&Launch {
            sandbox,
            ..Default::default()
        })
        .unwrap();
        a.resume();
        a.collect(Duration::from_secs(20));
        say(
            t,
            format!("sandbox={sandbox}: {}", a.cmd("mitig").join(" ")),
        );
        a.kill();
    }
    let mut a = launch(&Launch {
        args: "--nolower".into(),
        ..Default::default()
    })
    .unwrap();
    a.resume();
    a.collect(Duration::from_secs(20));
    say(
        t,
        format!(
            "sandboxed, before Untrusted lowering: {}",
            a.cmd("mitig").join(" ")
        ),
    );
    a.kill();
    for api in [
        "GetHandleInformation",
        "GetFileType",
        "NtQueryObject",
        "CloseHandle",
        "ReadFile",
        "WaitForSingleObject",
        "DuplicateHandle",
    ] {
        for v in ["0x12344", "-3"] {
            for sandbox in [true, false] {
                let mut a = launch(&Launch {
                    sandbox,
                    ..Default::default()
                })
                .unwrap();
                a.resume();
                a.collect(Duration::from_secs(20));
                let r = a.cmd(&format!("inv {api} {v}"));
                unsafe { WaitForSingleObject(a.process, 1000) };
                say(
                    t,
                    format!(
                        "sandbox={sandbox} {api}({v}): {} [child {}]",
                        r.join(" "),
                        a.state()
                    ),
                );
                a.kill();
            }
        }
    }
}

// ---------------- P27: FSCTL system-call disable ----------------
fn p27() {
    let t = "P27";
    for (label, mit) in [
        ("control (no policy)", None),
        ("FSCTL disable (POLICY2 1<<56)", Some([0u64, 1u64 << 56])),
    ] {
        let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        let mut a = match launch(&Launch {
            mitigation: mit,
            ..Default::default()
        }) {
            Ok(a) => a,
            Err(e) => {
                say(t, format!("{label}: launch FAILED {e}"));
                continue;
            }
        };
        let rs = a.give(s, 0, DUPLICATE_SAME_ACCESS).unwrap();
        let rc = a.give(c, 0, DUPLICATE_SAME_ACCESS).unwrap();
        a.resume();
        says(t, a.collect(Duration::from_secs(20)));
        says(t, a.cmd("mitig"));
        say(t, format!("{label}: full-access ends; server ops:"));
        says(t, a.cmd(&format!("ops {rs}")));
        says(t, a.cmd(&format!("write {rc} 1 12")));
        says(t, a.cmd(&format!("read {rs} 4096 1000")));
        says(t, a.cmd(&format!("iocpread {rs} 4096")));
        a.kill();
    }
}

// ---------------- round-trip cost ----------------
fn rtt() {
    let t = "RTT";
    // broker -> child on pair1, child -> broker on pair2; the child
    // echoes through a completion port.
    let (s1, c1, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let (s2, c2, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let mut a = launch(&Launch::default()).unwrap();
    let rin = a.give(s1, RECV, DUPLICATE_CLOSE_SOURCE).unwrap();
    let wout = a.give(c2, SEND, DUPLICATE_CLOSE_SOURCE).unwrap();
    a.resume();
    a.collect(Duration::from_secs(20));
    says(t, a.cmd(&format!("echo {rin} {wout}")));
    for size in [64usize, 4096, 65536] {
        let iters = if size == 65536 { 2000 } else { 10000 };
        let mut v = Vec::with_capacity(iters);
        for i in 0..iters + 200 {
            let t0 = Instant::now();
            write_msg(c1, vec![1; size]).unwrap();
            let r = read_msg(s2, size + 64, 2000).unwrap();
            assert_eq!(r.len(), size);
            if i >= 200 {
                v.push(t0.elapsed().as_secs_f64() * 1e6);
            }
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        say(
            t,
            format!(
                "broker<->sandboxed child round trip, {size} B messages, n={iters}: median {:.1} us, p10 {:.1}, p90 {:.1}, p99 {:.1}, mean {:.1}",
                v[v.len() / 2],
                v[v.len() / 10],
                v[v.len() * 9 / 10],
                v[v.len() * 99 / 100],
                mean
            ),
        );
    }
    a.kill();
    // In-process baseline: write+read on one pair in this thread.
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let n = 20000;
    let t0 = Instant::now();
    for _ in 0..n {
        write_msg(c, vec![1; 64]).unwrap();
        read_msg(s, 4096, 1000).unwrap();
    }
    say(
        t,
        format!(
            "in-process write+read, 64 B, same thread: {:.2} us each",
            t0.elapsed().as_secs_f64() * 1e6 / n as f64
        ),
    );
}

/// The ssh session's token is an elevated admin's (High IL, BA enabled).
/// A browser broker runs with the UAC-filtered token: BA deny-only,
/// privileges stripped, Medium IL. This builds that shape as a
/// restricted version of our own token (BA and S-1-5-114 deny-only,
/// DISABLE_MAX_PRIVILEGE, label lowered to Medium) and re-runs this
/// program under it with the same stdout/stderr.
fn relaunch_medium(rest: &[String]) {
    use windows_sys::Win32::Security::Authorization::ConvertStringSidToSidW;
    use windows_sys::Win32::Security::*;
    use windows_sys::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};
    let mut cur: H = null_mut();
    unsafe {
        OpenProcessToken(
            me(),
            TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ASSIGN_PRIMARY | TOKEN_ADJUST_DEFAULT,
            &mut cur,
        )
    };
    let mut sids: Vec<Vec<u8>> = vec![];
    for s in ["S-1-5-32-544", "S-1-5-114"] {
        let mut p: PSID = null_mut();
        assert!(unsafe { ConvertStringSidToSidW(w(s).as_ptr(), &mut p) } != 0);
        let len = unsafe { GetLengthSid(p) } as usize;
        sids.push(unsafe { std::slice::from_raw_parts(p as *const u8, len) }.to_vec());
    }
    let disable: Vec<SID_AND_ATTRIBUTES> = sids
        .iter_mut()
        .map(|s| SID_AND_ATTRIBUTES {
            Sid: s.as_mut_ptr() as PSID,
            Attributes: 0,
        })
        .collect();
    let mut tok: H = null_mut();
    let ok = unsafe {
        CreateRestrictedToken(
            cur,
            0x1, /* DISABLE_MAX_PRIVILEGE */
            2,
            disable.as_ptr(),
            0,
            null(),
            0,
            null(),
            &mut tok,
        )
    };
    assert!(ok != 0, "CreateRestrictedToken {}", le());
    let mut medium: PSID = null_mut();
    unsafe { ConvertStringSidToSidW(w("S-1-16-8192").as_ptr(), &mut medium) };
    let label = TOKEN_MANDATORY_LABEL {
        Label: SID_AND_ATTRIBUTES {
            Sid: medium,
            Attributes: 0x20,
        },
    };
    let ok = unsafe {
        SetTokenInformation(
            tok,
            TokenIntegrityLevel,
            &label as *const _ as *const c_void,
            size_of::<TOKEN_MANDATORY_LABEL>() as u32 + unsafe { GetLengthSid(medium) },
        )
    };
    assert!(ok != 0, "SetTokenInformation {}", le());
    // Owner = the user and a default DACL of user + SYSTEM, as in a
    // filtered token (the elevated token's owner and default DACL name
    // BA, which is deny-only here).
    let user = user_sid();
    let mut usid: PSID = null_mut();
    unsafe { ConvertStringSidToSidW(w(&user).as_ptr(), &mut usid) };
    let owner = TOKEN_OWNER { Owner: usid };
    let ok = unsafe {
        SetTokenInformation(
            tok,
            TokenOwner,
            &owner as *const _ as *const c_void,
            size_of::<TOKEN_OWNER>() as u32,
        )
    };
    assert!(ok != 0, "SetTokenInformation(owner) {}", le());
    let sa = sa_from_sddl(&format!("D:(A;;GA;;;{user})(A;;GA;;;SY)"), false);
    let (mut present, mut defaulted, mut dacl) = (0i32, 0i32, null_mut::<ACL>());
    unsafe {
        GetSecurityDescriptorDacl(
            sa.lpSecurityDescriptor,
            &mut present,
            &mut dacl,
            &mut defaulted,
        )
    };
    let dd = TOKEN_DEFAULT_DACL { DefaultDacl: dacl };
    let ok = unsafe {
        SetTokenInformation(
            tok,
            TokenDefaultDacl,
            &dd as *const _ as *const c_void,
            size_of::<TOKEN_DEFAULT_DACL>() as u32,
        )
    };
    assert!(ok != 0, "SetTokenInformation(default dacl) {}", le());
    let exe = std::env::current_exe().unwrap();
    let mut cmd = w(&format!("\"{}\" {}", exe.display(), rest.join(" ")));
    let mut si: STARTUPINFOW = unsafe { zeroed() };
    si.cb = size_of::<STARTUPINFOW>() as u32;
    si.dwFlags = STARTF_USESTDHANDLES;
    unsafe {
        si.hStdOutput = GetStdHandle(STD_OUTPUT_HANDLE);
        si.hStdError = GetStdHandle(STD_ERROR_HANDLE);
        SetHandleInformation(si.hStdOutput, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT);
        SetHandleInformation(si.hStdError, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT);
    }
    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
    let rc = unsafe {
        CreateProcessAsUserW(
            tok,
            w(&exe.to_string_lossy()).as_ptr(),
            cmd.as_mut_ptr(),
            null(),
            null(),
            1,
            0,
            null(),
            null(),
            &si,
            &mut pi,
        )
    };
    if rc == 0 {
        println!("[env] Medium relaunch FAILED err={}", le());
        return;
    }
    unsafe { WaitForSingleObject(pi.hProcess, INFINITE) };
    let mut code = 0u32;
    unsafe { GetExitCodeProcess(pi.hProcess, &mut code) };
    println!("[env] Medium-token broker exited code={code:#x}");
}

// ======================= round 2 (design review) =======================

fn counts_str(h: H) -> String {
    match counts(h) {
        Ok((hc, pc)) => format!("HandleCount={hc} PointerCount={pc}"),
        Err(st) => format!("counts ntstatus={st:#x}"),
    }
}

// W1: does ObjectBasicInformation's HandleCount span processes?
fn w1() {
    let t = "W1";
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    say(t, format!("broker server end, fresh: {}", counts_str(s)));
    let sdup = dup(me(), s, me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    say(
        t,
        format!(
            "broker server end after a 2nd broker-local copy: {}",
            counts_str(s)
        ),
    );
    unsafe { CloseHandle(sdup) };
    // The client end goes into the child; the child makes a 2nd copy of
    // it in its own table; then the broker pulls the end with
    // close-source and reads the counts of the pulled copy.
    let mut a = launch(&Launch::default()).unwrap();
    let cc = a.give(c, SEND, 0).unwrap(); // a copy stays in the broker too
    a.resume();
    a.collect(Duration::from_secs(20));
    say(
        t,
        format!("client end: broker's own copy counts: {}", counts_str(c)),
    );
    says(t, a.cmd(&format!("counts {cc}")));
    says(t, a.cmd(&format!("dupself {cc}")));
    says(t, a.cmd(&format!("counts {cc}")));
    // Broker pulls the child's primary copy with close-source.
    let pulled = dup(
        a.process,
        hp(cc),
        me(),
        0,
        DUPLICATE_CLOSE_SOURCE | DUPLICATE_SAME_ACCESS,
    )
    .expect("pull");
    say(
        t,
        format!(
            "after pull (child had made a 2nd copy; broker also holds one): pulled copy {}",
            counts_str(pulled)
        ),
    );
    unsafe {
        CloseHandle(pulled);
        CloseHandle(c);
        CloseHandle(s);
    }
    a.kill();
}

// W2: do the low two bits of a handle value matter?
fn w2() {
    let t = "W2";
    let mut a = start("izcal.d5probe.a");
    // An event the child owns, at a 4-aligned value if we can get one.
    let rep = a.cmd("mkevent");
    says(t, rep.clone());
    let base: isize = info_field(&rep, "h").parse().unwrap_or(0);
    say(
        t,
        format!("child event handle value={base} (low2={})", base & 3),
    );
    for d in 0..4 {
        let v = base | d;
        says(t, a.cmd(&format!("checkh {v}")));
        // Broker duplicates from the child's table using the tagged value.
        match dup(a.process, hp(v), me(), 0, DUPLICATE_SAME_ACCESS) {
            Ok(h) => {
                say(t, format!("broker dup(child, {v}) OK -> {}", describe(h)));
                unsafe { CloseHandle(h) };
            }
            Err(e) => say(t, format!("broker dup(child, {v}) FAILED err={e}")),
        }
    }
    a.kill();
}

// W3: a pending read across a close-source move.
fn w3() {
    let t = "W3";
    for cancel in [false, true] {
        let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
        // The old holder (child A) gets the server (receiver) end.
        let mut a = launch(&Launch::default()).unwrap();
        let rs = a.give(s, RECV | FILE_READ_ATTRIBUTES, 0).unwrap();
        a.resume();
        a.collect(Duration::from_secs(20));
        says(t, a.cmd(&format!("postread {rs} 4096")));
        if cancel {
            says(t, a.cmd(&format!("cancelread {rs}")));
            std::thread::sleep(Duration::from_millis(100));
        }
        // Broker pulls the receiver end out of A with close-source into B.
        let mut b = launch(&Launch::default()).unwrap();
        let in_b =
            dup(a.process, hp(rs), b.process, RECV, DUPLICATE_CLOSE_SOURCE).expect("pull to B");
        b.resume();
        b.collect(Duration::from_secs(20));
        // Broker writes one message to the pipe (from the client end).
        let w = write_msg(c, vec![42u8; 64]);
        say(
            t,
            format!("cancel_first={cancel}: broker wrote one 64 B message: {w:?}"),
        );
        std::thread::sleep(Duration::from_millis(200));
        say(t, "A (old holder) read state:");
        says(t, a.cmd(&format!("readpoll {rs}")));
        say(t, "B (new holder) read:");
        says(t, b.cmd(&format!("read {} 4096 1000", hv(in_b))));
        a.kill();
        b.kill();
        unsafe {
            CloseHandle(c);
            CloseHandle(s);
        }
    }
}

// W4: two children in the SAME AppContainer profile.
fn w4() {
    let t = "W4";
    let mut a = launch(&Launch {
        container: "izcal.d5probe.shared",
        ..Default::default()
    })
    .unwrap();
    a.resume();
    let ha = a.collect(Duration::from_secs(20));
    for l in &ha {
        say(t, format!("A: {l}"));
    }
    let mut b = launch(&Launch {
        container: "izcal.d5probe.shared",
        ..Default::default()
    })
    .unwrap();
    b.resume();
    b.collect(Duration::from_secs(20));
    // A creates a LOCAL\ pipe (server) and reports the handle + name.
    let name = format!(r"\\.\pipe\LOCAL\izcal-d5-{}", rand_hex());
    let rep = a.cmd(&format!("mkpipe {name}"));
    says(t, rep.clone());
    let a_server: isize = info_field(&rep, "h").parse().unwrap_or(0);
    let fname = info_field(&rep, "filename");
    let full = format!(r"\\.\pipe{fname}");
    // B opens it by name.
    say(
        t,
        format!("B opens A's LOCAL pipe by its full name {full}:"),
    );
    let repb = b.cmd(&format!("open {full} {SEND:#x}"));
    says(t, repb.clone());
    if !info_field(&repb, "h").is_empty() {
        std::thread::sleep(Duration::from_millis(100));
        say(
            t,
            format!(
                "A GetNamedPipeClientProcessId on its server end (expect B pid={}):",
                b.pid
            ),
        );
        says(t, a.cmd(&format!("clientpid {a_server}")));
    }
    a.kill();
    b.kill();
}

// W4 (profiles): create/delete 100 derived AppContainer profiles, timed.
fn w4_profiles() {
    use windows_sys::Win32::Security::Isolation::{
        CreateAppContainerProfile, DeleteAppContainerProfile,
    };
    let t = "W4prof";
    let n = 100;
    let mut create_us = vec![];
    let mut del_us = vec![];
    for i in 0..n {
        let name = format!("izcal.d5probe.perf.{i}");
        let nw = w(&name);
        let mut sid: windows_sys::Win32::Security::PSID = null_mut();
        let t0 = Instant::now();
        let hr = unsafe {
            CreateAppContainerProfile(nw.as_ptr(), nw.as_ptr(), nw.as_ptr(), null(), 0, &mut sid)
        };
        create_us.push(t0.elapsed().as_secs_f64() * 1e6);
        if hr != 0 {
            say(
                t,
                format!("profile {i}: CreateAppContainerProfile hr={hr:#x}"),
            );
            break;
        }
    }
    for i in 0..n {
        let name = format!("izcal.d5probe.perf.{i}");
        let t0 = Instant::now();
        let hr = unsafe { DeleteAppContainerProfile(w(&name).as_ptr()) };
        del_us.push(t0.elapsed().as_secs_f64() * 1e6);
        if hr != 0 {
            say(t, format!("profile {i}: Delete hr={hr:#x}"));
        }
    }
    let med = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (
            v[v.len() / 2],
            v[v.len() - 1],
            v.iter().sum::<f64>() / v.len() as f64,
        )
    };
    let (cm, cmax, cmean) = med(&mut create_us);
    let (dm, dmax, dmean) = med(&mut del_us);
    say(
        t,
        format!(
            "{n} profiles: Create median {cm:.0} us (mean {cmean:.0}, max {cmax:.0}); Delete median {dm:.0} us (mean {dmean:.0}, max {dmax:.0})"
        ),
    );
}

// W5: read a pipe's name and end from a handle, incl. one from a child.
fn w5() {
    let t = "W5";
    let (s, c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    for (label, h) in [("server end", s), ("client end", c)] {
        say(
            t,
            format!(
                "{label}: FileNameInfo={} NtQueryObject-name={} {} flags-serverbit={}",
                file_name(h),
                object_name(h),
                pipe_info(h),
                pipe_info(h).contains("flags=0xd")
            ),
        );
    }
    // A child holds the client end; broker pulls it and reads the name.
    let mut a = launch(&Launch::default()).unwrap();
    let cc = a.give(c, SEND, 0).unwrap();
    a.resume();
    a.collect(Duration::from_secs(20));
    let pulled = dup(a.process, hp(cc), me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    say(
        t,
        format!(
            "client end pulled from child: FileNameInfo={} NtQueryObject-name={} {}",
            file_name(pulled),
            object_name(pulled),
            pipe_info(pulled)
        ),
    );
    unsafe {
        CloseHandle(pulled);
        CloseHandle(s);
        CloseHandle(c);
    }
    a.kill();
}

// W6: cost of duplicating 16 ends into a child in one batch.
fn w6() {
    let t = "W6";
    let mut a = launch(&Launch::default()).unwrap();
    a.resume();
    a.collect(Duration::from_secs(20));
    let mut pairs = vec![];
    for _ in 0..16 {
        pairs.push(make_pair(Some(&user_dacl()), 65536, 65536).unwrap());
    }
    // Warm the AppContainer/target-process path once, then time 5 batches.
    let mut times = vec![];
    for round in 0..6 {
        let t0 = Instant::now();
        let mut hs = vec![];
        for (_, c, _) in &pairs {
            hs.push(dup(me(), *c, a.process, SEND, 0).unwrap());
        }
        let us = t0.elapsed().as_secs_f64() * 1e6;
        if round > 0 {
            times.push(us);
        }
        // Close the duplicates in the child before the next round.
        for h in hs {
            let _ = dup(a.process, h, me(), 0, DUPLICATE_CLOSE_SOURCE);
        }
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    say(
        t,
        format!(
            "DuplicateHandle x16 into a sandboxed child: median {:.1} us/batch ({:.2} us/handle), min {:.1}, max {:.1}",
            times[times.len() / 2],
            times[times.len() / 2] / 16.0,
            times[0],
            times[times.len() - 1]
        ),
    );
    for (s, c, _) in pairs {
        unsafe {
            CloseHandle(s);
            CloseHandle(c);
        }
    }
    a.kill();
}

// W7: bootstrap over one inherited pipe; confirm isolation.
fn w7() {
    let t = "W7";
    // The bootstrap pipe: broker keeps the server end, the child gets the
    // client end as its ONE extra inheritable handle (beyond stdio).
    let (boot_s, boot_c, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    unsafe { SetHandleInformation(boot_c, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
    // Two channel ends to hand over after creation.
    let (s1, c1, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    let (s2, c2, _) = make_pair(Some(&user_dacl()), 65536, 65536).unwrap();
    // A few broker-only handles whose values we will test in the child.
    let probe_handles = [event(), boot_s, s1, s2];
    let mut a = launch(&Launch {
        extra_inherit: vec![boot_c],
        ..Default::default()
    })
    .unwrap();
    // Duplicate the two channel ends into the suspended child.
    let v1 = a.give(c1, SEND, DUPLICATE_CLOSE_SOURCE).unwrap();
    let v2 = a.give(s2, RECV, DUPLICATE_CLOSE_SOURCE).unwrap();
    a.resume();
    let hello = a.collect(Duration::from_secs(20));
    for l in &hello {
        say(t, format!("child: {l}"));
    }
    // The child reports the value of its inherited bootstrap end; compare.
    says(t, a.cmd(&format!("checkh {}", hv(boot_c))));
    say(t, format!("broker's boot_c value was {}", hv(boot_c)));
    // Broker sends the two channel values over the bootstrap pipe; the
    // child reads them from the bootstrap end and uses them.
    let msg = format!("{v1} {v2}");
    write_msg(boot_s, msg.into_bytes()).unwrap();
    says(t, a.cmd(&format!("read {} 256 1000", hv(boot_c))));
    says(t, a.cmd(&format!("access {v1}")));
    says(t, a.cmd(&format!("access {v2}")));
    // Use them: broker writes on s1 -> child reads v1? v1 is the SENDER
    // (client) end, so the child writes and the broker reads on s1.
    says(t, a.cmd(&format!("write {v1} 1 20")));
    say(
        t,
        format!("broker reads on s1: {}", fmt_read(&read_msg(s1, 256, 1000))),
    );
    // Isolation: test known broker handle values in the child with
    // dupself (DuplicateHandle self->self), which returns err 6 for an
    // invalid value instead of raising under strict handle checks (so it
    // does not kill the child). An OK would mean the value names a live
    // object in the child, i.e. a leak.
    say(
        t,
        "isolation: broker-only handle values checked in the child (expect all FAILED err=6):",
    );
    for h in probe_handles {
        says(t, a.cmd(&format!("dupself {}", hv(h))));
    }
    a.kill();
    unsafe {
        CloseHandle(boot_s);
        CloseHandle(boot_c);
        // c1 and s2 went to the child with DUPLICATE_CLOSE_SOURCE: the
        // broker no longer holds them.
        CloseHandle(s1);
        CloseHandle(c2);
        for h in probe_handles {
            if h != boot_s && h != s1 && h != s2 {
                CloseHandle(h);
            }
        }
    }
}

fn cleanup() {
    use windows_sys::Win32::Security::Isolation::DeleteAppContainerProfile;
    for n in ["izcal.d5probe.a", "izcal.d5probe.b", "izcal.d5probe.shared"] {
        let hr = unsafe { DeleteAppContainerProfile(w(n).as_ptr()) };
        say(
            "cleanup",
            format!("DeleteAppContainerProfile({n}) hr={hr:#x}"),
        );
    }
}
