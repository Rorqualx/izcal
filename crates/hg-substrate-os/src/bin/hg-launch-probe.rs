//! `hg-launch-probe`: a test child for the Linux spawner (tests only; it
//! ships nowhere). Started by the spawner in place of a real kind, under
//! the launch filter, it reports what it finds on its bootstrap sender
//! (descriptor 3) as one text record, then waits until its bootstrap
//! receiver (descriptor 4) reaches end of file, and exits.
//!
//! The report is black-box (security 6.7): it performs the calls and
//! records what they returned.
#![cfg_attr(
    target_os = "linux",
    expect(
        unsafe_code,
        reason = "G5: src/bin/hg-launch-probe.rs (raw fd and prctl probes)"
    )
)]

#[cfg(target_os = "linux")]
fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

#[cfg(target_os = "linux")]
fn main() {
    use std::fmt::Write as _;
    let mut r = String::new();
    // SAFETY: getpid takes nothing.
    let pid = unsafe { libc::getpid() };
    let _ = write!(r, "pid={pid}");
    let mut fds = Vec::new();
    for n in 0..256 {
        // SAFETY: F_GETFD takes no pointer; a closed number gives EBADF.
        let fl = unsafe { libc::fcntl(n, libc::F_GETFD) };
        if fl >= 0 {
            fds.push(format!(
                "{n}{}",
                if fl & libc::FD_CLOEXEC != 0 { "c" } else { "" }
            ));
        }
    }
    let _ = write!(r, " fds={}", fds.join(","));
    // SAFETY: a prctl query with no pointers.
    let nnp = unsafe { libc::prctl(libc::PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0) };
    // SAFETY: as above.
    let seccomp = unsafe { libc::prctl(libc::PR_GET_SECCOMP, 0, 0, 0, 0) };
    let _ = write!(r, " nnp={nnp} seccomp={seccomp}");
    // SAFETY: a static path; the sandbox decides.
    let rc = unsafe { libc::open(c"/etc/hosts".as_ptr(), libc::O_RDONLY) };
    let _ = write!(r, " open={}", if rc < 0 { errno() } else { 0 });
    // SAFETY: constant arguments; the sandbox decides.
    let rc = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    let _ = write!(r, " inet={}", if rc < 0 { errno() } else { 0 });
    // SAFETY: a signal to init (pid 1 outside a PID namespace is another
    // process); the sandbox decides.
    let rc = unsafe { libc::kill(1, 0) };
    let _ = write!(r, " kill1={}", if rc < 0 { errno() } else { 0 });
    // SAFETY: writes the live report to descriptor 3.
    unsafe { libc::write(3, r.as_ptr().cast(), r.len()) };
    // The ends are nonblocking (the transport's sockets are), and only
    // epoll may wait under the filter: wait there, then read.
    // SAFETY: epoll_create1 takes a flag word.
    let ep = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
    let mut ev = libc::epoll_event {
        events: u32::try_from(libc::EPOLLIN).unwrap_or(1),
        u64: 4,
    };
    // SAFETY: a live event; descriptor 4 is held.
    unsafe { libc::epoll_ctl(ep, libc::EPOLL_CTL_ADD, 4, &raw mut ev) };
    let mut b = [0u8; 64];
    let rc = loop {
        let mut out = [libc::epoll_event { events: 0, u64: 0 }];
        // SAFETY: one live output slot; no signal mask change.
        unsafe { libc::epoll_pwait(ep, out.as_mut_ptr(), 1, -1, core::ptr::null()) };
        // SAFETY: reads into a live buffer.
        let rc = unsafe { libc::read(4, b.as_mut_ptr().cast(), b.len()) };
        if rc == 0 || (rc < 0 && errno() != libc::EAGAIN) {
            break rc;
        }
    };
    // Why it stopped, as a second record.
    let end = format!("end rc={rc} errno={}", if rc < 0 { errno() } else { 0 });
    // SAFETY: writes the live record to descriptor 3.
    unsafe { libc::write(3, end.as_ptr().cast(), end.len()) };
}

/// On macOS: the hello exchange with its launcher (its launch channels),
/// then nothing, ever: it never sends `ready`, so its launcher's deadline
/// must kill it (the launcher conformance test).
#[cfg(target_os = "macos")]
fn main() {
    let threads = hg_substrate_os::Threads {
        general: 1,
        may_block: 1,
    };
    let Ok(_s) = hg_substrate_os::os_substrate_launched(threads) else {
        return;
    };
    loop {
        std::thread::park();
    }
}

/// Linux and macOS only.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() {}
