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
#![expect(
    unsafe_code,
    reason = "G5: src/bin/fp-m0-empty.rs (the pre-opened fd 3)"
)]

use std::io::Write;
use std::os::fd::FromRawFd;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use hg_substrate_os::sandbox::macos::{apply_empty_process_profile, probe_tier};
use hg_substrate_os::thread;

fn main() {
    // 1. Lock down first: nothing opens after this point.
    if let Err(e) = apply_empty_process_profile() {
        eprintln!("fp-m0-empty: sandbox failed to apply: {e}");
        std::process::exit(1);
    }

    // 2. The tier probe: record what was achieved, never silent.
    let tier = probe_tier();

    // 2a. The behavioral sysctl probe: sandbox_check's "sysctl-read"
    // verdict disagrees with the profile's (allow sysctl-read) rule
    // (first run: D while the process plainly reads sysctls), so the
    // tier line reports the real read, which is what the rule governs.
    let mut memsize: u64 = 0;
    let mut len: libc::size_t = std::mem::size_of::<u64>();
    // SAFETY: the name is a static C string; `memsize` and `len` are
    // valid out-pointers for the call's duration.
    let sysctl_ok = unsafe {
        libc::sysctlbyname(
            c"hw.memsize".as_ptr(),
            (&mut memsize as *mut u64).cast::<libc::c_void>(),
            &mut len,
            std::ptr::null_mut(),
            0,
        ) == 0
    };

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
        "sandbox=applied file-read={} mach-lookup={} network-outbound={} sysctl-check={} sysctl-real={} threads=2 pid={}\n",
        tier[0].as_char(),
        tier[1].as_char(),
        tier[2].as_char(),
        tier[3].as_char(),
        if sysctl_ok {
            format!("ok:{memsize}")
        } else {
            "failed".to_string()
        },
        std::process::id(),
    );
    let _ = ipc.write_all(line.as_bytes());

    // 5. Wait forever: the supervisor samples us and then kills us.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
