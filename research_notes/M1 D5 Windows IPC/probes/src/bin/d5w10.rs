//! M1 D5 probes W10 and W11, for the fresh-pipe-per-move design (written
//! from Microsoft Learn reference pages; no header or crate source).
//!
//! A child process holds a duplicate of a pipe's server end (the same file
//! object) and leaves an overlapped read pending on it, as a former holder
//! could. The parent then reads on its own handle, as the new holder would.
//!
//! W10: with the child's read pending first and the parent's second, which
//! read does the next record complete? Then, with records queued and the
//! writer closing, what does each side see (the drain-to-the-end the
//! design relies on)?
//! W11: the writer closes while the child's read is pending on an empty
//! pipe. Does that read fail, and with what error? What does a later read
//! on the parent's handle return?
#![allow(clippy::missing_safety_doc)]

use std::io::Read;
use std::os::windows::io::AsRawHandle;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use d5probe::*;
use windows_sys::Win32::Foundation::*;

fn say(tag: &str, s: String) {
    println!("[{tag}] {s}");
}

/// The child: one overlapped read on the inherited server end, reported
/// with its timing.
fn child(v: &str) {
    let h = hp(v.parse().unwrap());
    let t0 = Instant::now();
    let r = read_msg(h, 64, 15_000);
    println!(
        "child read: {} after {} ms",
        fmt_read(&r),
        t0.elapsed().as_millis()
    );
}

/// Starts a child with an inheritable duplicate of `server` and waits for
/// its read to be pending.
fn spawn_reader(server: H) -> std::process::Child {
    let inh = dup(me(), server, me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    unsafe { SetHandleInformation(inh, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
    let c = Command::new(std::env::current_exe().unwrap())
        .args(["child", &hv(inh).to_string()])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    unsafe { CloseHandle(inh) };
    std::thread::sleep(Duration::from_millis(700));
    c
}

fn child_said(mut c: std::process::Child) -> String {
    let _ = c.wait();
    let mut s = String::new();
    if let Some(mut o) = c.stdout.take() {
        let _ = o.read_to_string(&mut s);
    }
    s.trim().to_owned()
}

/// A read on the parent's handle, on its own thread.
fn parent_read(server: H, ms: u32) -> std::sync::mpsc::Receiver<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let v = hv(server);
    std::thread::spawn(move || {
        let r = read_msg(hp(v), 64, ms);
        let _ = tx.send(fmt_read(&r));
    });
    rx
}

fn w10(round: u32) {
    let (server, client, _) = make_pair(None, 65536, 65536).unwrap();
    let c = spawn_reader(server);
    let p = parent_read(server, 10_000);
    std::thread::sleep(Duration::from_millis(300));
    let w1 = write_msg(client, vec![1u8; 8]);
    let w2 = write_msg(client, vec![2u8; 8]);
    let got = p
        .recv_timeout(Duration::from_secs(12))
        .unwrap_or("no answer".into());
    say(
        "W10",
        format!(
            "round {round}: child read first, parent second; writes {w1:?} {w2:?}; parent: {got}; {}",
            child_said(c)
        ),
    );
    unsafe {
        CloseHandle(client);
        CloseHandle(server);
    }
}

fn w10_drain() {
    let (server, client, _) = make_pair(None, 65536, 65536).unwrap();
    let c = spawn_reader(server);
    for b in [1u8, 2, 3] {
        let _ = write_msg(client, vec![b; 8]);
    }
    unsafe { CloseHandle(client) };
    std::thread::sleep(Duration::from_millis(300));
    let rest = read_all(server, 64, 3000);
    say(
        "W10b",
        format!(
            "child read pending, three records queued, writer closed; {}; parent then reads: {rest:?}",
            child_said(c)
        ),
    );
    unsafe { CloseHandle(server) };
}

fn w11() {
    let (server, client, _) = make_pair(None, 65536, 65536).unwrap();
    let c = spawn_reader(server);
    let t0 = Instant::now();
    unsafe { CloseHandle(client) };
    let said = child_said(c);
    let later = read_msg(server, 64, 3000);
    say(
        "W11",
        format!(
            "child read pending on an empty pipe, writer closed at t0; {said} (child exited {} ms after t0); a later read on the parent's handle: {}",
            t0.elapsed().as_millis(),
            fmt_read(&later)
        ),
    );
    unsafe { CloseHandle(server) };
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("child") {
        child(&args[1]);
        return;
    }
    for round in 1..=5 {
        w10(round);
    }
    for _ in 0..3 {
        w10_drain();
    }
    for _ in 0..3 {
        w11();
    }
    let _ = std::io::stdout().as_raw_handle();
}
