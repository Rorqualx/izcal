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
use hg_substrate_os::sandbox::{apply, probe_line};
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
                // Block until woken: no timer, so no idle wakeups
                // (footprint 6.8: no periodic timers, no polling).
                // park() may return spuriously, hence the loop.
                while !done.load(Ordering::Relaxed) {
                    std::thread::park();
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
    // Hold the process, blocked with no timer, until the supervisor ends
    // it.
    loop {
        std::thread::park();
    }
}

/// The Windows main: the supervisor started this process in an
/// AppContainer under the lockdown token, in its job, with no console.
/// It lowers itself to Untrusted, probes its tier, spawns the two
/// threads, reports over its stdout (the pipe the supervisor passed as
/// the IPC endpoint) and waits.
#[cfg(windows)]
fn main() {
    use std::io::Write;

    // stdout is the IPC pipe and stderr is null, so every failure is
    // reported down the pipe as one `fp-m0-empty: error:` line, which
    // the supervisor prints with the child's exit code (SEC-8).
    fn fail(msg: &str) -> ! {
        // One line, one write: the supervisor reads a single message, so
        // line breaks inside msg (a panic's location and payload) are
        // flattened.
        let line = format!("fp-m0-empty: error: {}\n", msg.replace(['\r', '\n'], " "));
        let mut out = std::io::stdout().lock();
        let _ = out.write_all(line.as_bytes()).and_then(|()| out.flush());
        std::process::exit(1);
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "CS-ER-14: this is the process's crash handler; with stderr null it reports a panic down the IPC pipe"
    )]
    std::panic::set_hook(Box::new(|info| fail(&format!("panic: {info}"))));

    // 1. Lock down first: lower from the AppContainer's Low integrity
    // to Untrusted before anything else runs.
    if let Err(e) = hg_substrate_os::sandbox::apply() {
        fail(&format!("sandbox failed to apply: {e}"));
    }

    // 2. The tier probe; argv[1] is the supervisor's canary, a file of
    // the user's that the AppContainer must keep this process from
    // reading.
    let canary = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    let probe = hg_substrate_os::sandbox::windows::probe_line(canary.as_deref());

    let done = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();
    for name in ["fp-m0-thread-a", "fp-m0-thread-b"] {
        let done = Arc::clone(&done);
        match thread::spawn(name, move || {
            // Block until woken: no timer, so no idle wakeups
            // (footprint 6.8: no periodic timers, no polling).
            // park() may return spuriously, hence the loop.
            while !done.load(Ordering::Relaxed) {
                std::thread::park();
            }
        }) {
            Ok(h) => handles.push(h),
            Err(e) => fail(&format!("thread spawn ({name}): {e}")),
        }
    }

    let line = format!(
        "sandbox=applied {probe} threads=2 pid={}\n",
        std::process::id(),
    );
    let mut ipc = std::io::stdout().lock();
    if ipc
        .write_all(line.as_bytes())
        .and_then(|()| ipc.flush())
        .is_err()
    {
        // The pipe itself failed, so nothing can be reported down it;
        // exiting lets the supervisor report the closed pipe and this
        // exit code instead of waiting on a silent child.
        std::process::exit(2);
    }
    drop(ipc);

    // Hold the process, blocked with no timer, until the supervisor ends
    // it.
    loop {
        std::thread::park();
    }
}
