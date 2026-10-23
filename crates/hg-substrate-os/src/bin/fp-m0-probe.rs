//! FP-M0 half B: the supervisor that drives the empty-process probe.
//!
//! Spawns `fp-m0-empty` with the IPC endpoint pre-opened (fd 3 on unix,
//! a pipe passed as the child's stdout on Windows — the security-scope 6.7
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

/// The Windows supervisor: creates the IPC pipe (the child's stdout), a
/// canary file of the user's for the tier probe, the lockdown token,
/// the AppContainer and the job; starts the child suspended in the
/// AppContainer under the token, so the job assignment completes before
/// it runs (fatal on failure); resumes it, times spawn-to-ready, prints
/// the child's ready line, and holds it for the harness. Any other
/// outcome (an error line, a closed pipe, no line within the deadline)
/// prints one `FAILED` line with the child's state and exits 1.
#[cfg(windows)]
fn main() {
    use std::mem::size_of;
    use std::ptr;
    use std::time::Instant;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, SetHandleInformation};
    use windows_sys::Win32::Security::{SECURITY_ATTRIBUTES, SECURITY_CAPABILITIES};
    use windows_sys::Win32::Storage::FileSystem::ReadFile;
    use windows_sys::Win32::System::Pipes::CreatePipe;
    use windows_sys::Win32::System::Threading::{
        CREATE_NO_WINDOW, CREATE_SUSPENDED, CreateProcessAsUserW, DeleteProcThreadAttributeList,
        EXTENDED_STARTUPINFO_PRESENT, InitializeProcThreadAttributeList,
        LPPROC_THREAD_ATTRIBUTE_LIST, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
        PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, PROCESS_INFORMATION, ResumeThread,
        STARTF_USESTDHANDLES, STARTUPINFOEXW, Sleep, TerminateProcess, UpdateProcThreadAttribute,
    };

    use hg_substrate_os::sandbox::windows::{
        app_container, assign_job, job_object, restricted_token,
    };

    // 1. The IPC pipe. The write end becomes the child's stdout. It is
    // inheritable so CreateProcessAsUserW can pass it, and the
    // handle-list attribute below limits inheritance to that one
    // handle (I-2); the read end is not inheritable.
    let sa = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    };
    let mut read_h: HANDLE = ptr::null_mut();
    let mut write_h: HANDLE = ptr::null_mut();
    // SAFETY: the out-handles are valid; the attributes make both ends
    // inheritable, and the read end's flag is cleared next.
    if unsafe { CreatePipe(&mut read_h, &mut write_h, &sa, 0) } == 0 {
        eprintln!("fp-m0-probe: CreatePipe failed");
        exit(1);
    }
    // SAFETY: read_h is the just-created pipe handle; the flag change
    // applies to this process's copy only.
    if unsafe {
        SetHandleInformation(read_h, 1 /* HANDLE_FLAG_INHERIT */, 0)
    } == 0
    {
        eprintln!("fp-m0-probe: SetHandleInformation failed");
        exit(1);
    }

    // 2. The canary: a file of the user's, in the user's temp folder,
    // that the child must not be able to read (the AppContainer's job).
    let canary = std::env::temp_dir().join(format!("fp-m0-canary-{}.txt", std::process::id()));
    if let Err(e) = std::fs::write(&canary, b"izcal fp-m0 canary\n") {
        eprintln!("fp-m0-probe: canary {}: {e}", canary.display());
        exit(1);
    }
    let remove_canary = || {
        let _ = std::fs::remove_file(&canary);
    };

    // 3. The lockdown token, the AppContainer (no capabilities) and the
    // job.
    let token = match restricted_token() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("fp-m0-probe: token: {e}");
            exit(1);
        }
    };
    let container = match app_container("izcal.fp-m0.empty", "Izcal FP-M0 empty process") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("fp-m0-probe: appcontainer: {e}");
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

    // 4. The attribute list: the AppContainer with no capabilities, and
    // the one handle the child may inherit.
    let mut caps = SECURITY_CAPABILITIES {
        AppContainerSid: container.as_psid(),
        Capabilities: ptr::null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    };
    let mut inherit = [write_h];
    let mut attr_size: usize = 0;
    // SAFETY: the documented size query: a null list returns the size
    // needed for two attributes (the call reports failure by design).
    unsafe { InitializeProcThreadAttributeList(ptr::null_mut(), 2, 0, &mut attr_size) };
    let mut attr_mem = vec![0u64; attr_size.div_ceil(8)];
    let attrs = attr_mem.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;
    // SAFETY: `attr_mem` is at least `attr_size` bytes and 8-aligned,
    // and lives until DeleteProcThreadAttributeList below.
    let ok_init = unsafe { InitializeProcThreadAttributeList(attrs, 2, 0, &mut attr_size) };
    // SAFETY: `caps` lives until process creation; size is the struct's.
    let ok_caps = unsafe {
        UpdateProcThreadAttribute(
            attrs,
            0,
            PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
            &mut caps as *mut _ as *mut _,
            size_of::<SECURITY_CAPABILITIES>(),
            ptr::null_mut(),
            ptr::null(),
        )
    };
    // SAFETY: `inherit` holds one valid handle and lives until process
    // creation; size is one HANDLE.
    let ok_list = unsafe {
        UpdateProcThreadAttribute(
            attrs,
            0,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
            inherit.as_mut_ptr() as *mut _,
            size_of::<HANDLE>(),
            ptr::null_mut(),
            ptr::null(),
        )
    };
    if ok_init == 0 || ok_caps == 0 || ok_list == 0 {
        eprintln!("fp-m0-probe: attribute list failed ({ok_init}{ok_caps}{ok_list})");
        remove_canary();
        exit(1);
    }

    // 5. The child command line: the exe next to this one, then the
    // canary path for the tier probe.
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("fp-m0-empty.exe")))
        .expect("exe path");
    let mut cmdline: Vec<u16> = format!("\"{}\" \"{}\"", exe.display(), canary.display())
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let exe_w: Vec<u16> = exe
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: a zeroed struct is the documented initial state.
    let mut si: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    si.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    // The pipe is the child's stdout, the IPC endpoint; stdin and
    // stderr are null.
    si.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    si.StartupInfo.hStdOutput = write_h;
    si.lpAttributeList = attrs;
    // SAFETY: a zeroed struct is the documented initial state.
    let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    let started = Instant::now();

    // The child's environment: inherited (a null block). The scrub is a
    // Windows-tier follow-up alongside win32k lockdown, ACG, CIG, CET.
    //
    // The child starts suspended so the job assignment below completes
    // BEFORE it runs — its mitigation claim is true when it reports. No
    // console window: creating one under the sandbox fails in the
    // loader with STATUS_DLL_INIT_FAILED (0xC0000142); fp-m0-empty is a
    // GUI-subsystem binary, and CREATE_NO_WINDOW covers any
    // console-subsystem child (FP-M0 decision log, 23 October).
    // SAFETY: valid token, exe path, command line and extended startup
    // info; inheritance is on and limited to `inherit` by the list.
    let rc = unsafe {
        CreateProcessAsUserW(
            token.as_handle(),
            exe_w.as_ptr(),
            cmdline.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_SUSPENDED | CREATE_NO_WINDOW,
            ptr::null(),
            ptr::null(),
            &si.StartupInfo,
            &mut pi,
        )
    };
    // SAFETY: GetLastError reads the thread's error slot only; read
    // before the cleanup calls can overwrite it.
    let create_err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    // SAFETY: the list was initialized above and is no longer needed.
    unsafe { DeleteProcThreadAttributeList(attrs) };
    // SAFETY: the parent's copy of the write end is no longer needed.
    unsafe { CloseHandle(write_h) };
    if rc == 0 {
        eprintln!("fp-m0-probe: CreateProcessAsUserW failed: error={create_err}");
        remove_canary();
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
        remove_canary();
        exit(1);
    }
    // SAFETY: pi.hThread is the suspended child's primary thread.
    if unsafe { ResumeThread(pi.hThread) } == u32::MAX {
        eprintln!("fp-m0-probe: ResumeThread failed");
        remove_canary();
        exit(1);
    }

    // 6. The ready line, with a deadline. A reader thread does the
    // blocking read so a child that neither writes nor exits is
    // reported as stuck instead of hanging the supervisor (SEC-8).
    // Every outcome other than a ready line prints the child's state
    // and exits 1; the timing line prints only for a ready child.
    const READY_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    // The handle crosses to the reader thread as an integer (HANDLE is
    // a raw pointer, so not Send); only that thread uses it from here.
    let read_raw = read_h as usize;
    let reader = hg_substrate_os::thread::spawn("fp-m0-probe-reader", move || {
        let read_h = read_raw as HANDLE;
        let mut buf = [0u8; 512];
        let mut n: u32 = 0;
        // SAFETY: the buffer is valid for the read; read_h is the pipe's
        // read end, owned by this thread from here on. The read blocks
        // until the child writes or the pipe closes.
        let ok = unsafe { ReadFile(read_h, buf.as_mut_ptr(), 512, &mut n, ptr::null_mut()) };
        // SAFETY: the read end is ours and no longer needed.
        unsafe { CloseHandle(read_h) };
        let got = if ok == 0 { 0 } else { n as usize };
        let _ = tx.send(buf[..got].to_vec());
    });
    if let Err(e) = reader {
        eprintln!("fp-m0-probe: reader thread: {e}");
        // SAFETY: valid process handle; the child is ours to end.
        unsafe { TerminateProcess(pi.hProcess, 1) };
        exit(1);
    }

    let outcome = rx.recv_timeout(READY_DEADLINE);
    // The child has probed (or failed); the canary is no longer needed.
    remove_canary();
    match outcome {
        Ok(bytes) if bytes.starts_with(b"sandbox=applied") => {
            println!(
                "fp-m0-probe: child pid={} spawn-to-ready={:.3} ms",
                pi.dwProcessId,
                started.elapsed().as_secs_f64() * 1e3
            );
            print!("fp-m0-probe: {}", String::from_utf8_lossy(&bytes));
        }
        Ok(bytes) if !bytes.is_empty() => {
            // The child wrote something other than a ready line: its
            // error report (stderr is null, so stdout carries it).
            println!(
                "fp-m0-probe: FAILED child pid={} reported: {}; {}",
                pi.dwProcessId,
                String::from_utf8_lossy(&bytes).trim_end(),
                child_state(pi.hProcess)
            );
            exit(1);
        }
        Ok(_) => {
            println!(
                "fp-m0-probe: FAILED child pid={} closed the pipe without a ready line; {}",
                pi.dwProcessId,
                child_state(pi.hProcess)
            );
            exit(1);
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            // The reader thread ended without sending: a supervisor
            // fault, not the child's.
            println!(
                "fp-m0-probe: FAILED supervisor reader ended without a result; child pid={}; {}; terminating it",
                pi.dwProcessId,
                child_state(pi.hProcess)
            );
            // SAFETY: valid process handle; the child is ours to end.
            unsafe { TerminateProcess(pi.hProcess, 1) };
            exit(1);
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            println!(
                "fp-m0-probe: FAILED child pid={} no ready line within {} s; {}; terminating it",
                pi.dwProcessId,
                READY_DEADLINE.as_secs(),
                child_state(pi.hProcess)
            );
            // SAFETY: valid process handle; a stuck child is ended.
            unsafe { TerminateProcess(pi.hProcess, 1) };
            exit(1);
        }
    }

    // 7. Hold the child for the harness; the measurement kills us.
    loop {
        // SAFETY: a fixed 1-second sleep with no return value.
        unsafe { Sleep(1000) };
    }
}

/// How the child stands, for the failure lines: its exit code once it
/// has exited (waiting up to 2 s), or "still running" — never the
/// STILL_ACTIVE pseudo exit code (259) read as a real one.
#[cfg(windows)]
fn child_state(process: windows_sys::Win32::Foundation::HANDLE) -> String {
    use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
    use windows_sys::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject};

    // SAFETY: the caller passes the child's process handle, valid for
    // the supervisor's lifetime; the wait is bounded.
    let waited = unsafe { WaitForSingleObject(process, 2000) };
    if waited != WAIT_OBJECT_0 {
        return format!("child still running (wait={waited:#x})");
    }
    let mut code: u32 = 0;
    // SAFETY: valid process handle and out-pointer.
    if unsafe { GetExitCodeProcess(process, &mut code) } == 0 {
        return "child exited; exit code unreadable".to_string();
    }
    format!("child exited with code {code:#010x}")
}
