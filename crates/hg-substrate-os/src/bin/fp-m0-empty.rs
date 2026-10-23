//! FP-M0 half B: the sandboxed empty process.
//!
//! The substrate floor's probe target (footprint spec Section 9, FP-M0):
//! an empty process on Izcal's substrate with the **full sandbox** of
//! security scope 6.7, **two threads** and an **IPC endpoint**. The IPC
//! endpoint is fd 3, pre-opened by the supervisor (`fp-m0-probe`) — the
//! spec's rule: nothing is opened after lockdown, everything else
//! arrives as a pre-opened handle.
//!
//! Start-up order: apply the sandbox, probe the achieved tier, spawn
//! the two threads, report the ready line over fd 3, then wait for the
//! supervisor.
// Windows: the GUI subsystem, so the sandboxed process gets no console
// and no conhost.exe (a console host costs ~1 MB private and ~6 MB
// working set per process; FP-M0 decision log, 23 October).
#![cfg_attr(windows, windows_subsystem = "windows")]
#![cfg_attr(
    unix,
    expect(
        unsafe_code,
        reason = "G5: src/bin/fp-m0-empty.rs (the pre-opened fd 3; unix only)"
    )
)]

#[cfg(unix)]
use std::io::Write;
#[cfg(unix)]
use std::os::fd::FromRawFd;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(unix)]
use hg_substrate_os::sandbox::apply;
use hg_substrate_os::sandbox::probe_line;
use hg_substrate_os::thread;

#[cfg(unix)]
fn main() {
    // 1. Lock down first: nothing opens after this point.
    if let Err(e) = apply() {
        eprintln!("fp-m0-empty: sandbox failed to apply: {e}");
        std::process::exit(1);
    }

    // 2. The tier probe: record what was achieved, never silent.
    let probe = probe_line();

    // 3. Two threads, through the sanctioned interface.
    let done = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();
    for name in ["fp-m0-thread-a", "fp-m0-thread-b"] {
        let done = Arc::clone(&done);
        handles.push(
            thread::spawn(name, move || {
                while !done.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            })
            .expect("thread spawn"),
        );
    }

    // 4. Report readiness over the pre-opened IPC endpoint.
    // SAFETY: fd 3 is the supervisor's pre-opened socketpair end; the
    // supervisor guarantees it is open and valid for this process.
    let mut ipc = unsafe { UnixStream::from_raw_fd(3) };
    let line = format!(
        "sandbox=applied {probe} threads=2 pid={}\n",
        std::process::id(),
    );
    // The tier line also goes to stderr, for the harness log.
    eprint!("{line}");
    if let Err(e) = ipc.write_all(line.as_bytes()) {
        eprintln!("fp-m0-empty: ready write failed: {e}");
    }

    // 5. Wait forever: the supervisor samples us and then kills us.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}

/// The Windows main: the child starts restricted from birth (the
/// supervisor created it under the untrusted token, with no console),
/// probes its tier, spawns the two threads, reports over its stdout —
/// the pipe the supervisor passed as the IPC endpoint, the one handle
/// path CreateProcessWithTokenW carries — and waits.
#[cfg(windows)]
fn main() {
    use std::io::Write;

    let probe = probe_line();

    let done = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();
    for name in ["fp-m0-thread-a", "fp-m0-thread-b"] {
        let done = Arc::clone(&done);
        handles.push(
            thread::spawn(name, move || {
                while !done.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            })
            .expect("thread spawn"),
        );
    }

    let line = format!(
        "sandbox=applied {probe} threads=2 pid={}\n",
        std::process::id(),
    );
    // The ready line is the only output. A failed write cannot be
    // reported (stderr is null); the supervisor reports the missing
    // line and the child's state instead (SEC-8).
    let mut ipc = std::io::stdout().lock();
    let _ = ipc.write_all(line.as_bytes()).and_then(|()| ipc.flush());

    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
