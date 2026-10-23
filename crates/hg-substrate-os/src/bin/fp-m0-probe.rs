//! FP-M0 half B: the supervisor that drives the empty-process probe.
//!
//! Spawns `fp-m0-empty` with the IPC endpoint pre-opened (fd 3 on unix,
//! an inherited pipe handle on Windows — the security-scope 6.7
//! pattern: the child opens nothing after lockdown), times
//! spawn-to-ready, prints the child's ready line and pid, and keeps
//! the child alive for the external harness (which samples any PID).
//! SIGINT/SIGTERM kill the unix child and exit; the Windows pair is
//! killed by the measurement.
#![expect(
    unsafe_code,
    reason = "G5: src/bin/fp-m0-probe.rs (posix_spawn + signals)"
)]

#[cfg(unix)]
use std::ffi::CString;
#[cfg(unix)]
use std::io::Read;
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::process::exit;
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(unix)]
use std::time::Instant;

#[cfg(unix)]
use libc::{
    kill, posix_spawn, posix_spawn_file_actions_addclose, posix_spawn_file_actions_adddup2,
    posix_spawn_file_actions_destroy, posix_spawn_file_actions_init, posix_spawn_file_actions_t,
};

/// Set by the signal handler; the supervisor loop reaps on it.
#[cfg(unix)]
static RUNNING: AtomicBool = AtomicBool::new(true);

#[cfg(unix)]
extern "C" fn on_signal(_: libc::c_int) {
    // Async-signal-safe: a lock-free store only.
    RUNNING.store(false, Ordering::Relaxed);
}

#[cfg(unix)]
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

/// The Windows supervisor: creates the IPC pipe (inheritable), the
/// restricted untrusted token and the job, starts the child under the
/// token suspended so the job assignment completes before it runs
/// (fatal on failure), resumes it, times spawn-to-ready, prints the
/// child's ready line, and holds it for the harness.
#[cfg(windows)]
fn main() {
    use std::mem::size_of;
    use std::ptr;
    use std::time::Instant;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, SetHandleInformation};
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::Storage::FileSystem::ReadFile;
    use windows_sys::Win32::System::Pipes::CreatePipe;
    use windows_sys::Win32::System::Threading::{
        CreateProcessWithTokenW, PROCESS_INFORMATION, ResumeThread, STARTUPINFOW, Sleep,
        TerminateProcess,
    };

    use hg_substrate_os::sandbox::windows::{assign_job, job_object, restricted_token};

    // 1. The IPC pipe, inheritable (the child receives the write end).
    let sa = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    };
    let mut read_h: HANDLE = ptr::null_mut();
    let mut write_h: HANDLE = ptr::null_mut();
    // SAFETY: the out-handles are valid; the attributes request
    // inheritance for the child's copy of the write end.
    if unsafe { CreatePipe(&mut read_h, &mut write_h, &sa, 0) } == 0 {
        eprintln!("fp-m0-probe: CreatePipe failed");
        exit(1);
    }
    // The child must inherit only the write end (I-2: a handle passed
    // further than needed). Clear the read end's inherit flag.
    // SAFETY: read_h is the just-created pipe handle; the flag change
    // applies to this process's copy only.
    if unsafe {
        SetHandleInformation(read_h, 1 /* HANDLE_FLAG_INHERIT */, 0)
    } == 0
    {
        eprintln!("fp-m0-probe: SetHandleInformation failed");
        exit(1);
    }

    // 2. The restricted token and the job.
    let token = match restricted_token() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("fp-m0-probe: token: {e}");
            exit(1);
        }
    };
    let job = match job_object() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("fp-m0-probe: job: {e}");
            exit(1);
        }
    };

    // 3. The child command line: the exe next to this one plus the
    // inherited pipe handle.
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("fp-m0-empty.exe")))
        .expect("exe path");
    let mut cmdline: Vec<u16> = format!("\"{}\" {}", exe.display(), write_h as usize)
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let exe_w: Vec<u16> = exe
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: a zeroed struct is the documented initial state.
    let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
    si.cb = size_of::<STARTUPINFOW>() as u32;
    // SAFETY: a zeroed struct is the documented initial state.
    let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    let started = Instant::now();

    // The child's environment: inherited for now. Both the hand-built
    // and the filtered-from-GetEnvironmentStringsW blocks made every
    // CreateProcess* variant fail ERROR_INVALID_PARAMETER (87) on
    // rig253 — recorded in the FP-M0 decision log; the scrub is a
    // Windows-tier follow-up alongside AppContainer/win32k/ACG.
    //
    // CreateProcessWithTokenW, not AsUser: the latter demands
    // SE_INCREASE_QUOTA_NAME (1314) from a non-admin caller. The
    // child starts suspended so the job assignment below completes
    // BEFORE it runs — its mitigation claim is true when it reports.
    const CREATE_SUSPENDED: u32 = 0x4;
    // SAFETY: valid exe path, command line and startup info;
    // inheritance is on so the child gets the write end.
    let rc = unsafe {
        CreateProcessWithTokenW(
            token.as_handle(),
            0,
            exe_w.as_ptr(),
            cmdline.as_mut_ptr(),
            CREATE_SUSPENDED,
            ptr::null(),
            ptr::null(),
            &si,
            &mut pi,
        )
    };
    // SAFETY: the parent's copy of the write end is no longer needed.
    unsafe { CloseHandle(write_h) };
    if rc == 0 {
        // SAFETY: GetLastError reads the thread's error slot only.
        let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        eprintln!("fp-m0-probe: CreateProcessWithTokenW failed: error={err}");
        exit(1);
    }
    // SAFETY: pi.hProcess is the just-created child process handle.
    // Fatal on failure: the child must never claim a mitigation it
    // does not have (SEC-8: silent reduced modes are forbidden).
    if let Err(e) = unsafe { assign_job(&job, pi.hProcess) } {
        eprintln!("fp-m0-probe: job assign failed: {e}");
        // SAFETY: the child is suspended and never ran; killing it
        // here is the clean teardown.
        unsafe { TerminateProcess(pi.hProcess, 1) };
        exit(1);
    }
    // SAFETY: pi.hThread is the suspended child's primary thread.
    if unsafe { ResumeThread(pi.hThread) } == u32::MAX {
        eprintln!("fp-m0-probe: ResumeThread failed");
        exit(1);
    }

    // 4. The ready line (blocking read on the pipe).
    let mut buf = [0u8; 512];
    let mut n: u32 = 0;
    // SAFETY: the buffer is valid for the read; the read blocks until
    // the child writes or the pipe closes.
    unsafe { ReadFile(read_h, buf.as_mut_ptr(), 512, &mut n, ptr::null_mut()) };
    // SAFETY: the read end is ours and no longer needed.
    unsafe { CloseHandle(read_h) };
    println!(
        "fp-m0-probe: child pid={} spawn-to-ready={:.3} ms",
        pi.dwProcessId,
        started.elapsed().as_secs_f64() * 1e3
    );
    if n > 0 {
        print!(
            "fp-m0-probe: {}",
            String::from_utf8_lossy(&buf[..n as usize])
        );
    }

    // 5. Hold the child for the harness; the measurement kills us.
    loop {
        // SAFETY: a fixed 1-second sleep with no return value.
        unsafe { Sleep(1000) };
    }
}
