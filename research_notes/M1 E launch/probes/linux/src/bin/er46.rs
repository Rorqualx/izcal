//! M1 phase E, round 2: the two probes the design's E-R4 and E-R6 rest on.
//! Written from man pages only: clone(2), execveat(2), openat(2),
//! unix(7), pidfd_open(2), proc(5), user_namespaces(7), pid_namespaces(7).
//!
//! `er46 dirfd <dir> <name>`: E-R4. The parent opens `<dir>` O_PATH |
//! O_DIRECTORY before the clone; the child, in new user (+ mount) namespaces,
//! opens `<name>` relative to it and execs the result.
//! `er46 creds`: E-R6 (a). broker -> spawner (plain child) -> grandchild in
//! new user + PID namespaces; who sees which pid and uid.
//! `er46 pidfd-ino [churn]`: E-R6 (b). Is a pidfd's inode a generation?
#![allow(unsafe_code, clippy::all)]

use std::ffi::CString;
use std::os::fd::RawFd;

const CLONE_PIDFD: u64 = 0x1000;

#[repr(C)]
#[derive(Default)]
struct CloneArgs {
    flags: u64,
    pidfd: u64,
    child_tid: u64,
    parent_tid: u64,
    exit_signal: u64,
    stack: u64,
    stack_size: u64,
    tls: u64,
    set_tid: u64,
    set_tid_size: u64,
    cgroup: u64,
}

fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(-1)
}

fn clone3(flags: u64, pidfd: &mut i32) -> i64 {
    let args = CloneArgs {
        flags: flags | CLONE_PIDFD,
        pidfd: (pidfd as *mut i32) as u64,
        exit_signal: libc::SIGCHLD as u64,
        ..Default::default()
    };
    unsafe {
        libc::syscall(
            libc::SYS_clone3,
            &raw const args,
            std::mem::size_of::<CloneArgs>(),
        )
    }
}

fn passcred(fd: RawFd) {
    let one: i32 = 1;
    unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PASSCRED,
            (&raw const one).cast(),
            4,
        )
    };
}

fn peercred(fd: RawFd) -> String {
    let mut u: libc::ucred = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<libc::ucred>() as u32;
    let rc = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&raw mut u).cast(),
            &mut len,
        )
    };
    if rc < 0 {
        format!("err={}", errno())
    } else {
        format!("pid={} uid={}", u.pid, u.uid)
    }
}

fn send(fd: RawFd, msg: &str) {
    unsafe { libc::send(fd, msg.as_ptr().cast(), msg.len(), libc::MSG_NOSIGNAL) };
}

/// One record and its SCM_CREDENTIALS.
fn recv(fd: RawFd) -> (String, String) {
    let mut buf = [0u8; 512];
    let mut cbuf = [0u64; 16];
    let mut iov = libc::iovec {
        iov_base: buf.as_mut_ptr().cast(),
        iov_len: buf.len(),
    };
    let mut h: libc::msghdr = unsafe { std::mem::zeroed() };
    h.msg_iov = &mut iov;
    h.msg_iovlen = 1;
    h.msg_control = cbuf.as_mut_ptr().cast();
    h.msg_controllen = std::mem::size_of_val(&cbuf);
    let n = unsafe { libc::recvmsg(fd, &mut h, 0) };
    if n < 0 {
        return (format!("recv-err={}", errno()), String::new());
    }
    let mut creds = "no-creds".to_string();
    unsafe {
        let mut c = libc::CMSG_FIRSTHDR(&h);
        while !c.is_null() {
            if (*c).cmsg_level == libc::SOL_SOCKET && (*c).cmsg_type == libc::SCM_CREDENTIALS {
                let u = libc::CMSG_DATA(c).cast::<libc::ucred>().read_unaligned();
                creds = format!("pid={} uid={}", u.pid, u.uid);
            }
            c = libc::CMSG_NXTHDR(&h, c);
        }
    }
    (
        String::from_utf8_lossy(&buf[..n as usize]).to_string(),
        creds,
    )
}

fn dirfd_probe(dir: &str, name: &str) {
    let d = CString::new(dir).unwrap();
    let n = CString::new(name).unwrap();
    let low = unsafe {
        libc::open(
            d.as_ptr(),
            libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC,
        )
    };
    assert!(low >= 0, "open dir: {}", errno());
    let dfd = unsafe { libc::fcntl(low, libc::F_DUPFD_CLOEXEC, 100) };
    unsafe { libc::close(low) };
    let user = libc::CLONE_NEWUSER as u64;
    let all =
        (libc::CLONE_NEWUSER | libc::CLONE_NEWPID | libc::CLONE_NEWNET | libc::CLONE_NEWNS) as u64;
    for (label, flags, open_flags) in [
        ("user only, O_PATH", user, libc::O_PATH),
        (
            "user+mnt, O_PATH",
            user | libc::CLONE_NEWNS as u64,
            libc::O_PATH,
        ),
        (
            "user+mnt, O_RDONLY",
            user | libc::CLONE_NEWNS as u64,
            libc::O_RDONLY,
        ),
        ("user+pid+net+mnt, O_PATH", all, libc::O_PATH),
    ] {
        let mut rp = [0; 2];
        unsafe { libc::pipe2(rp.as_mut_ptr(), libc::O_CLOEXEC) };
        let mut pidfd = -1;
        let rc = clone3(flags, &mut pidfd);
        if rc == 0 {
            let f = unsafe { libc::openat(dfd, n.as_ptr(), open_flags | libc::O_CLOEXEC) };
            let open_s = if f < 0 {
                format!("openat errno={}", errno())
            } else {
                "openat ok".into()
            };
            // Keep the pipe's write end so a failed exec can report; the
            // child exec'd prints its own line on stdout ("child" mode of
            // the main probe: it needs fd 3, so give it the pipe there).
            unsafe { libc::dup2(rp[1], 3) };
            let a0 = c"er46-child";
            let argv = [a0.as_ptr(), std::ptr::null()];
            let envp: [*const libc::c_char; 1] = [std::ptr::null()];
            let msg0 = format!("{open_s}; ");
            unsafe { libc::write(3, msg0.as_ptr().cast(), msg0.len()) };
            let x = unsafe {
                libc::syscall(
                    libc::SYS_execveat,
                    f,
                    c"".as_ptr(),
                    argv.as_ptr(),
                    envp.as_ptr(),
                    libc::AT_EMPTY_PATH,
                )
            };
            let msg = format!("execveat rc={x} errno={}", errno());
            unsafe {
                libc::write(3, msg.as_ptr().cast(), msg.len());
                libc::_exit(127)
            }
        }
        unsafe { libc::close(rp[1]) };
        let mut out = String::new();
        let mut b = [0u8; 512];
        loop {
            let k = unsafe { libc::read(rp[0], b.as_mut_ptr().cast(), b.len()) };
            if k <= 0 {
                break;
            }
            out.push_str(&String::from_utf8_lossy(&b[..k as usize]));
        }
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        unsafe { libc::waitid(3, pidfd as u32, &mut info, libc::WEXITED) };
        let status = unsafe { info.si_status() };
        println!("E-R4 {label}: {out} | exit status {status}");
        unsafe {
            libc::close(rp[0]);
            libc::close(pidfd);
        }
    }
}

fn creds_probe() {
    // The broker makes both pairs: one to the spawner, one for the
    // grandchild (passed down by fork inheritance, as SCM_RIGHTS would).
    let mut bs = [0; 2];
    let mut bc = [0; 2];
    unsafe {
        libc::socketpair(
            libc::AF_UNIX,
            libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
            0,
            bs.as_mut_ptr(),
        );
        libc::socketpair(
            libc::AF_UNIX,
            libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
            0,
            bc.as_mut_ptr(),
        );
    }
    for fd in [bs[0], bs[1], bc[0], bc[1]] {
        passcred(fd);
    }
    let broker = unsafe { libc::getpid() };
    let spawner = unsafe { libc::fork() };
    if spawner == 0 {
        // Spawner: also makes a pair of its own for the grandchild, to see
        // what a spawner-created pair shows.
        let mut sc = [0; 2];
        unsafe {
            libc::socketpair(
                libc::AF_UNIX,
                libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
                0,
                sc.as_mut_ptr(),
            )
        };
        passcred(sc[0]);
        passcred(sc[1]);
        let mut pidfd = -1;
        let rc = clone3(
            (libc::CLONE_NEWUSER | libc::CLONE_NEWPID) as u64,
            &mut pidfd,
        );
        if rc == 0 {
            // Grandchild, inside its namespaces.
            let me = unsafe { libc::getpid() };
            let r1 = format!(
                "grandchild getpid={me}; SO_PEERCRED on broker-made end [{}]; on spawner-made end [{}]",
                peercred(bc[1]),
                peercred(sc[1])
            );
            send(bc[1], &r1);
            send(sc[1], "to-spawner");
            let (t1, c1) = recv(bc[1]);
            let (t2, c2) = recv(sc[1]);
            send(
                bc[1],
                &format!(
                    "grandchild saw broker's '{t1}' creds[{c1}]; spawner's '{t2}' creds[{c2}]"
                ),
            );
            unsafe { libc::_exit(0) };
        }
        let (t, c) = recv(sc[0]);
        send(sc[0], "from-spawner");
        send(
            bs[1],
            &format!(
                "spawner pid={} saw grandchild '{t}' creds[{c}]; clone3 outer pid={rc}; SO_PEERCRED on its own end [{}]",
                unsafe { libc::getpid() },
                peercred(sc[0])
            ),
        );
        let mut st = 0;
        unsafe { libc::waitpid(rc as i32, &mut st, 0) };
        unsafe { libc::_exit(0) };
    }
    unsafe { libc::close(bc[1]) };
    println!("E-R6a broker pid={broker} spawner pid={spawner}");
    let (t, c) = recv(bc[0]);
    println!("  broker got from grandchild: '{t}' creds[{c}]");
    println!(
        "  broker SO_PEERCRED on its end of the broker-made pair: [{}]",
        peercred(bc[0])
    );
    send(bc[0], "from-broker");
    let (t, c) = recv(bc[0]);
    println!("  broker got: '{t}' creds[{c}]");
    let (t, c) = recv(bs[0]);
    println!("  broker got from spawner: '{t}' creds[{c}]");
    let mut st = 0;
    unsafe { libc::waitpid(spawner, &mut st, 0) };
}

fn fd_ino(fd: RawFd) -> (u64, u64) {
    let mut s: libc::stat = unsafe { std::mem::zeroed() };
    unsafe { libc::fstat(fd, &mut s) };
    (s.st_dev as u64, s.st_ino as u64)
}

fn pidfd_ino_probe(churn: usize) {
    // Two live children at once, then one after the first is reaped.
    let mut out = Vec::new();
    let mut fds = Vec::new();
    for _ in 0..2 {
        let mut pidfd = -1;
        let rc = clone3(0, &mut pidfd);
        if rc == 0 {
            unsafe { libc::usleep(200_000) };
            unsafe { libc::_exit(0) };
        }
        let (dev, ino) = fd_ino(pidfd);
        out.push(format!("pid={rc} pidfd dev={dev} ino={ino}"));
        fds.push((rc, pidfd));
    }
    let link = std::fs::read_link(format!("/proc/self/fd/{}", fds[0].1))
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    println!("E-R6b two live children: {} | link {link}", out.join(" ; "));
    // Uniqueness across exit: reap, make a new one.
    for (_, fd) in &fds {
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        unsafe { libc::waitid(3, *fd as u32, &mut info, libc::WEXITED) };
    }
    let mut pidfd = -1;
    let rc = clone3(0, &mut pidfd);
    if rc == 0 {
        unsafe { libc::_exit(0) };
    }
    let (dev, ino) = fd_ino(pidfd);
    println!("  after both exited: new pid={rc} ino={ino} dev={dev}");
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    unsafe { libc::waitid(3, pidfd as u32, &mut info, libc::WEXITED) };
    // Churn until a pid recurs (only when asked): vfork-style children,
    // pid_max 4194304 on hueyj, so this is millions of forks.
    if churn > 0 {
        let first = rc;
        let first_ino = ino;
        let t0 = std::time::Instant::now();
        for i in 0..churn {
            let mut fd = -1;
            let p = clone3(0, &mut fd);
            if p == 0 {
                unsafe { libc::_exit(0) };
            }
            let (_, ino2) = fd_ino(fd);
            let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
            unsafe { libc::waitid(3, fd as u32, &mut info, libc::WEXITED) };
            unsafe { libc::close(fd) };
            if p == first {
                println!(
                    "  pid {first} recurred after {i} forks, {} s: ino then {first_ino}, now {ino2}",
                    t0.elapsed().as_secs()
                );
                return;
            }
        }
        println!(
            "  no recurrence of pid {first} in {churn} forks ({} s)",
            t0.elapsed().as_secs()
        );
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.first().map(String::as_str) == Some("er46-child") {
        let msg = format!("exec ok, child pid={}", unsafe { libc::getpid() });
        unsafe { libc::write(3, msg.as_ptr().cast(), msg.len()) };
        return;
    }
    match a.get(1).map(String::as_str) {
        Some("dirfd") => dirfd_probe(&a[2], &a[3]),
        Some("creds") => creds_probe(),
        Some("pidfd-ino") => pidfd_ino_probe(a.get(2).and_then(|s| s.parse().ok()).unwrap_or(0)),
        _ => eprintln!("usage: er46 dirfd <dir> <name> | creds | pidfd-ino [churn]"),
    }
}
