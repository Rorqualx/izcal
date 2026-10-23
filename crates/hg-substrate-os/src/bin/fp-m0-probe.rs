//! FP-M0 half B: the supervisor that drives the empty-process probe.
//!
//! Spawns `fp-m0-empty` with the IPC endpoint pre-opened (fd 3 — the
//! security-scope 6.7 pattern: the child opens nothing after lockdown),
//! times spawn-to-ready, prints the child's ready line and pid, and
//! keeps the child alive for the external harness (`footprint` / the
//! FP-M0 tree sampler, which samples any PID). SIGINT/SIGTERM kill the
//! child and exit.
#![expect(
    unsafe_code,
    reason = "G5: src/bin/fp-m0-probe.rs (posix_spawn + signals)"
)]

use std::ffi::CString;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use libc::{
    kill, posix_spawn, posix_spawn_file_actions_addclose, posix_spawn_file_actions_adddup2,
    posix_spawn_file_actions_destroy, posix_spawn_file_actions_init, posix_spawn_file_actions_t,
};

/// Set by the signal handler; the supervisor loop reaps on it.
static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn on_signal(_: libc::c_int) {
    // Async-signal-safe: a lock-free store only.
    RUNNING.store(false, Ordering::Relaxed);
}

fn main() {
    // SAFETY: the handler body is async-signal-safe (one lock-free
    // store); the function-pointer-to-integer cast is the documented
    // way to install a C handler.
    unsafe { libc::signal(libc::SIGINT, on_signal as *const () as libc::sighandler_t) };
    // SAFETY: as above, for the second signal.
    unsafe { libc::signal(libc::SIGTERM, on_signal as *const () as libc::sighandler_t) };

    let (supervisor_end, child_end) = UnixStream::pair().expect("socketpair");
    let mut child_fd = child_end.as_raw_fd();
    // fd 3 is the child's pre-opened IPC slot. If the socketpair landed
    // on fd 3 itself (stdio closed at launch), dup2(3,3) would be a
    // no-op and the close action would then kill the child's endpoint.
    let dupd = child_fd == 3;
    if dupd {
        // SAFETY: dup copies the fd to the lowest free slot; the new fd
        // is a separate fd, closed explicitly after the spawn.
        child_fd = unsafe { libc::dup(child_fd) };
        if child_fd < 0 {
            eprintln!("fp-m0-probe: dup failed");
            exit(1);
        }
    }

    // The child sits next to this binary (cargo puts both in the same
    // target dir); a bare name would rely on PATH, which cargo does not
    // set for us.
    let child_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("fp-m0-empty")))
        .unwrap_or_else(|| "fp-m0-empty".into());
    let child = CString::new(child_path.to_str().unwrap_or("fp-m0-empty")).expect("no NUL");
    let mut argv = [child.as_ptr() as *mut libc::c_char, std::ptr::null_mut()];

    let mut pid: libc::pid_t = 0;
    let started = Instant::now();
    // SAFETY: `actions` is a zeroed file-actions object; init prepares
    // it and the following calls use it in order, destroyed at the end.
    let mut actions: posix_spawn_file_actions_t = unsafe { std::mem::zeroed() };
    // SAFETY: `actions` points at the zeroed object init prepares.
    unsafe { posix_spawn_file_actions_init(&mut actions) };
    // SAFETY: `child_fd` becomes the child's fd 3 (the pre-opened IPC
    // endpoint). The dup2 precedes the close: file actions apply in
    // order, and closing first would make the dup2 an EBADF.
    unsafe { posix_spawn_file_actions_adddup2(&mut actions, child_fd, 3) };
    // SAFETY: `child_fd` is the socketpair end the child must not keep
    // under its original number; the close applies to the child only.
    unsafe { posix_spawn_file_actions_addclose(&mut actions, child_fd) };
    // SAFETY: `argv` is a null-terminated array of valid C strings;
    // `pid` is a valid out-pointer; `actions` holds the two prepared
    // actions. The call only creates the child process.
    // The child's environment: minimal and scrubbed — the supervisor's
    // own environment (DYLD_* linker variables included) acts at exec,
    // before the child's sandbox applies, and must not leak through
    // (security scope's environment surface).
    let clean_env = [CString::new("PATH=/usr/bin:/bin").expect("no NUL")];
    let mut envp: Vec<*mut libc::c_char> = clean_env
        .iter()
        .map(|e| e.as_ptr() as *mut libc::c_char)
        .collect();
    envp.push(std::ptr::null_mut());
    // SAFETY: `argv` and `envp` are null-terminated arrays of valid C
    // strings; `pid` is a valid out-pointer; `actions` holds the two
    // prepared actions. The call only creates the child process.
    let rc = unsafe {
        posix_spawn(
            &mut pid,
            child.as_ptr(),
            &actions,
            std::ptr::null_mut(),
            argv.as_mut_ptr(),
            envp.as_mut_ptr(),
        )
    };
    // SAFETY: `actions` is the initialized object; this frees it.
    unsafe { posix_spawn_file_actions_destroy(&mut actions) };
    drop(child_end); // closes the original fd; the dup'd one, if any, remains
    if dupd {
        // SAFETY: child_fd is the dup made above — a separate fd that
        // `child_end` does not own; this is its parent-side release.
        unsafe { libc::close(child_fd) };
    }
    if rc != 0 {
        eprintln!("fp-m0-probe: posix_spawn failed: rc={rc}");
        exit(1);
    }

    let mut buf = [0u8; 512];
    let mut ready = supervisor_end;
    let n = ready.read(&mut buf).unwrap_or(0); // blocks until the child is up
    println!(
        "fp-m0-probe: child pid={pid} spawn-to-ready={:.3} ms",
        started.elapsed().as_secs_f64() * 1e3
    );
    if n > 0 {
        print!("fp-m0-probe: {}", String::from_utf8_lossy(&buf[..n]));
    }

    while RUNNING.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // SAFETY: `pid` is our child process.
    unsafe { kill(pid, libc::SIGKILL) };
    // SAFETY: `pid` is the killed child; the null status out-pointer is
    // the documented "no status wanted" form.
    unsafe { libc::waitpid(pid, std::ptr::null_mut(), 0) };
}
