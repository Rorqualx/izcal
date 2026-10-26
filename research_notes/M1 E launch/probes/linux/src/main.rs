//! M1 phase E: black-box probes of the Linux spawn path.
//!
//! Written from man pages only: clone(2) (clone3), user_namespaces(7),
//! pid_namespaces(7), mount_namespaces(7), pivot_root(2), landlock(7),
//! seccomp(2), execveat(2), close_range(2), unix(7), pidfd_open(2).
//!
//! Usage: `e-linux-probes probe <child-binary> [runs]`. The child binary
//! is this program (dynamic or static build); it is exec'd by fd, so the
//! parent opens it O_PATH before any namespace is entered.
//!
//! The parent is single-threaded, so the cloned child (no CLONE_VM) may
//! allocate and format before it execs.
#![allow(unsafe_code, clippy::all)]

use std::ffi::CString;
use std::io::Write;
use std::os::fd::RawFd;
use std::time::Instant;

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

fn rc_errno(rc: i64) -> String {
    if rc < 0 {
        format!("err={}", errno())
    } else {
        format!("ok={rc}")
    }
}

fn write_file(path: &str, data: &str) -> String {
    let c = CString::new(path).unwrap();
    let fd = unsafe { libc::open(c.as_ptr(), libc::O_WRONLY | libc::O_CLOEXEC) };
    if fd < 0 {
        return format!("open-err={}", errno());
    }
    let n = unsafe { libc::write(fd, data.as_ptr().cast(), data.len()) };
    let e = errno();
    unsafe { libc::close(fd) };
    if n < 0 {
        format!("write-err={e}")
    } else {
        "ok".into()
    }
}

fn read_text(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| format!("read-err={e}"))
}

fn cap_eff() -> String {
    read_text("/proc/self/status")
        .lines()
        .find(|l| l.starts_with("CapEff"))
        .unwrap_or("CapEff: ?")
        .split_whitespace()
        .nth(1)
        .unwrap_or("?")
        .to_string()
}

/// Landlock ABI version (landlock(7): LANDLOCK_CREATE_RULESET_VERSION).
fn landlock_abi() -> i64 {
    unsafe { libc::syscall(libc::SYS_landlock_create_ruleset, 0usize, 0usize, 1u32) }
}

/// Restricts all filesystem access the ABI knows, plus TCP when ABI >= 4.
/// Returns (create rc, restrict rc, open-/etc/hosts errno, elapsed us).
fn landlock_lockdown(abi: i64) -> String {
    landlock_lockdown_with(abi, -1)
}

/// As [`landlock_lockdown`], plus one rule: EXECUTE|READ_FILE on the file
/// `allow_fd` (an O_PATH fd) when it is not -1 (landlock(7),
/// LANDLOCK_RULE_PATH_BENEATH).
fn landlock_lockdown_with(abi: i64, allow_fd: RawFd) -> String {
    #[repr(C)]
    struct Attr {
        fs: u64,
        net: u64,
        scoped: u64,
    }
    // landlock(7): access rights by ABI. v1: bits 0..12 (13 rights);
    // v2 adds REFER (13); v3 TRUNCATE (14); v5 IOCTL_DEV (15).
    let mut fs: u64 = (1 << 13) - 1;
    if abi >= 2 {
        fs |= 1 << 13;
    }
    if abi >= 3 {
        fs |= 1 << 14;
    }
    if abi >= 5 {
        fs |= 1 << 15;
    }
    let net: u64 = if abi >= 4 { 0b11 } else { 0 };
    let scoped: u64 = if abi >= 6 { 0b11 } else { 0 };
    let attr = Attr { fs, net, scoped };
    let size = if abi >= 6 {
        24
    } else if abi >= 4 {
        16
    } else {
        8
    };
    let t0 = Instant::now();
    let fd = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            &raw const attr,
            size as usize,
            0u32,
        )
    };
    if fd < 0 {
        return format!("create-err={}", errno());
    }
    let mut rule_s = String::new();
    if allow_fd >= 0 {
        #[repr(C, packed)]
        struct PathBeneath {
            allowed: u64,
            parent_fd: i32,
        }
        let r = PathBeneath {
            allowed: 0b1 | 0b100,
            parent_fd: allow_fd,
        }; // EXECUTE | READ_FILE
        let rc =
            unsafe { libc::syscall(libc::SYS_landlock_add_rule, fd, 1u32, &raw const r, 0u32) };
        rule_s = format!(" add_rule(exe) {}", rc_errno(rc));
    }
    unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    let rc = unsafe { libc::syscall(libc::SYS_landlock_restrict_self, fd, 0u32) };
    let us = t0.elapsed().as_micros();
    let rc_s = rc_errno(rc);
    let h = unsafe { libc::open(c"/etc/hosts".as_ptr(), libc::O_RDONLY) };
    let open_s = if h < 0 {
        format!("open-errno={}", errno())
    } else {
        "open-ALLOWED".into()
    };
    let s = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    let conn_s = if s < 0 {
        format!("socket-errno={}", errno())
    } else {
        let addr = libc::sockaddr_in {
            sin_family: libc::AF_INET as u16,
            sin_port: 80u16.to_be(),
            sin_addr: libc::in_addr {
                s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be(),
            },
            sin_zero: [0; 8],
        };
        let r = unsafe {
            libc::connect(
                s,
                (&raw const addr).cast(),
                std::mem::size_of::<libc::sockaddr_in>() as u32,
            )
        };
        if r < 0 {
            format!("connect-errno={}", errno())
        } else {
            "connect-ALLOWED".into()
        }
    };
    format!("restrict {rc_s}{rule_s} {open_s} {conn_s} setup_us={us}")
}

/// A seccomp filter of `n` compare-and-jump rules that allows everything
/// (its cost is the point), installed with TSYNC; returns elapsed us.
fn seccomp_cost(n: usize) -> String {
    #[repr(C)]
    struct Insn {
        code: u16,
        jt: u8,
        jf: u8,
        k: u32,
    }
    #[repr(C)]
    struct Prog {
        len: u16,
        filter: *const Insn,
    }
    let mut p = vec![Insn {
        code: 0x20,
        jt: 0,
        jf: 0,
        k: 0,
    }]; // ld [nr]
    for i in 0..n {
        // jeq #(1000+i), allow-at-end : next
        let to_end = (n - i) as u8;
        p.push(Insn {
            code: 0x15,
            jt: to_end.saturating_sub(1).max(0),
            jf: 0,
            k: 1000 + i as u32,
        });
    }
    p.push(Insn {
        code: 0x06,
        jt: 0,
        jf: 0,
        k: 0x7fff_0000,
    }); // ret ALLOW
    let prog = Prog {
        len: p.len() as u16,
        filter: p.as_ptr(),
    };
    unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    let t0 = Instant::now();
    let rc = unsafe { libc::syscall(libc::SYS_seccomp, 1u64, 1u64, &raw const prog) };
    format!(
        "seccomp({n}) {} us={}",
        rc_errno(rc),
        t0.elapsed().as_micros()
    )
}

/// Lists open fds 0..1024 (F_GETFD succeeds) with their CLOEXEC bit.
fn open_fds() -> String {
    let mut v = Vec::new();
    for fd in 0..1024 {
        let f = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if f >= 0 {
            v.push(format!(
                "{fd}{}",
                if f & libc::FD_CLOEXEC != 0 { "c" } else { "" }
            ));
        }
    }
    v.join(",")
}

fn send_with_creds(fd: RawFd, msg: &[u8]) -> String {
    let mut iov = libc::iovec {
        iov_base: msg.as_ptr() as *mut _,
        iov_len: msg.len(),
    };
    let mut h: libc::msghdr = unsafe { std::mem::zeroed() };
    h.msg_iov = &mut iov;
    h.msg_iovlen = 1;
    let n = unsafe { libc::sendmsg(fd, &h, libc::MSG_NOSIGNAL) };
    if n < 0 {
        format!("send-err={}", errno())
    } else {
        "sent".into()
    }
}

/// Receives one record; returns (text, SCM_CREDENTIALS pid/uid/gid).
fn recv_with_creds(fd: RawFd) -> (String, String) {
    let mut buf = [0u8; 512];
    let mut cbuf = [0u64; 32];
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
    let text = String::from_utf8_lossy(&buf[..n as usize]).to_string();
    let mut creds = "no-creds".to_string();
    unsafe {
        let mut c = libc::CMSG_FIRSTHDR(&h);
        while !c.is_null() {
            if (*c).cmsg_level == libc::SOL_SOCKET && (*c).cmsg_type == libc::SCM_CREDENTIALS {
                let u = libc::CMSG_DATA(c).cast::<libc::ucred>().read_unaligned();
                creds = format!("pid={} uid={} gid={}", u.pid, u.uid, u.gid);
            }
            c = libc::CMSG_NXTHDR(&h, c);
        }
    }
    (text, creds)
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
        format!("peercred-err={}", errno())
    } else {
        format!("pid={} uid={}", u.pid, u.uid)
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

/// The exec'd child: reports what it sees, over fd 3 (the kept end).
fn child_main() -> ! {
    let pid = unsafe { libc::getpid() };
    let uid = unsafe { libc::getuid() };
    let fds = open_fds();
    let peer = peercred(3);
    passcred(3);
    let sent = send_with_creds(
        3,
        format!(
            "ready pid={pid} uid={uid} fds={fds} peercred={peer} capeff={}",
            cap_eff()
        )
        .as_bytes(),
    );
    let (text, creds) = recv_with_creds(3);
    let _ = send_with_creds(
        3,
        format!("child-saw '{text}' creds[{creds}] {sent}").as_bytes(),
    );
    unsafe { libc::_exit(0) }
}

struct Opts {
    /// 0: execveat on the fd opened before clone; 1: execve by path;
    /// 2: execveat on an O_PATH fd the child opens after clone.
    exec_mode: u8,
    flags: u64,
    pivot: bool,
    landlock: bool,
    seccomp: usize,
}

/// One spawn. Returns (report lines, spawn-to-ready us).
fn spawn(exe_fd: RawFd, o: &Opts, verbose: bool) -> (Vec<String>, u128) {
    let mut sv = [0; 2];
    unsafe {
        libc::socketpair(
            libc::AF_UNIX,
            libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
            0,
            sv.as_mut_ptr(),
        )
    };
    passcred(sv[0]);
    // A pipe for the pre-exec report, and a stray non-CLOEXEC fd the
    // child must not inherit.
    let mut rp = [0; 2];
    unsafe { libc::pipe2(rp.as_mut_ptr(), libc::O_CLOEXEC) };
    let stray = unsafe { libc::open(c"/dev/null".as_ptr(), libc::O_RDONLY) };
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    let mut pidfd: i32 = -1;
    let args = CloneArgs {
        flags: o.flags | CLONE_PIDFD,
        pidfd: (&raw mut pidfd) as u64,
        exit_signal: libc::SIGCHLD as u64,
        ..Default::default()
    };
    let t0 = Instant::now();
    let rc = unsafe {
        libc::syscall(
            libc::SYS_clone3,
            &raw const args,
            std::mem::size_of::<CloneArgs>(),
        )
    };
    if rc < 0 {
        let e = errno();
        unsafe {
            libc::close(sv[0]);
            libc::close(sv[1]);
            libc::close(rp[0]);
            libc::close(rp[1]);
            libc::close(stray);
        }
        return (vec![format!("clone3 err={e}")], 0);
    }
    if rc == 0 {
        // Child, before exec.
        let mut r = Vec::new();
        r.push(format!(
            "pre-exec getpid={} uid={} capeff={}",
            unsafe { libc::getpid() },
            unsafe { libc::getuid() },
            cap_eff()
        ));
        if o.flags & libc::CLONE_NEWUSER as u64 != 0 {
            r.push(format!(
                "setgroups deny: {}",
                write_file("/proc/self/setgroups", "deny")
            ));
            r.push(format!(
                "uid_map: {}",
                write_file("/proc/self/uid_map", &format!("0 {uid} 1"))
            ));
            r.push(format!(
                "gid_map: {}",
                write_file("/proc/self/gid_map", &format!("0 {gid} 1"))
            ));
            r.push(format!(
                "after-map uid={} capeff={}",
                unsafe { libc::getuid() },
                cap_eff()
            ));
        }
        if o.flags & libc::CLONE_NEWNET as u64 != 0 {
            let s = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
            let addr = libc::sockaddr_in {
                sin_family: libc::AF_INET as u16,
                sin_port: 53u16.to_be(),
                sin_addr: libc::in_addr {
                    s_addr: u32::from_be_bytes([1, 1, 1, 1]).to_be(),
                },
                sin_zero: [0; 8],
            };
            let c = unsafe { libc::connect(s, (&raw const addr).cast(), 16) };
            r.push(format!(
                "netns socket={} connect-1.1.1.1 {}",
                s,
                if c < 0 {
                    format!("errno={}", errno())
                } else {
                    "ok".into()
                }
            ));
            unsafe { libc::close(s) };
        }
        if o.pivot {
            let m1 = unsafe {
                libc::mount(
                    std::ptr::null(),
                    c"/".as_ptr(),
                    std::ptr::null(),
                    libc::MS_REC | libc::MS_PRIVATE,
                    std::ptr::null(),
                )
            };
            let e1 = if m1 < 0 { errno() } else { 0 };
            // An empty tmpfs root: mount it on /tmp's mount point name is
            // not needed; use a fresh directory under /proc's sibling.
            let dir = c"/tmp";
            let m2 = unsafe {
                libc::mount(
                    c"none".as_ptr(),
                    dir.as_ptr(),
                    c"tmpfs".as_ptr(),
                    0,
                    c"size=64k,mode=0700".as_ptr().cast(),
                )
            };
            let e2 = if m2 < 0 { errno() } else { 0 };
            let ch = unsafe { libc::chdir(dir.as_ptr()) };
            let pr = unsafe { libc::syscall(libc::SYS_pivot_root, c".".as_ptr(), c".".as_ptr()) };
            let e3 = if pr < 0 { errno() } else { 0 };
            let um = unsafe { libc::umount2(c".".as_ptr(), libc::MNT_DETACH) };
            let e4 = if um < 0 { errno() } else { 0 };
            let _ = unsafe { libc::chdir(c"/".as_ptr()) };
            let probe = unsafe { libc::open(c"/etc/hosts".as_ptr(), libc::O_RDONLY) };
            r.push(format!(
                "pivot: private={e1} tmpfs={e2} chdir={ch} pivot_root={e3} umount_old={e4} /etc/hosts-after={}",
                if probe < 0 { format!("errno={}", errno()) } else { "VISIBLE".into() }
            ));
        }
        let child_fd = if o.exec_mode == 3 {
            // Reopen the pre-opened fd through procfs inside the new
            // mount namespace (no path to the build is named).
            let p = CString::new(format!("/proc/self/fd/{exe_fd}")).unwrap();
            let fd = unsafe { libc::open(p.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
            r.push(format!(
                "reopen via /proc/self/fd: {}",
                if fd < 0 {
                    format!("errno={}", errno())
                } else {
                    "ok".into()
                }
            ));
            unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 200) }
        } else if o.exec_mode == 2 {
            let p = CString::new(
                std::fs::read_link(format!("/proc/self/fd/{exe_fd}"))
                    .map(|p| p.display().to_string())
                    .unwrap_or_default(),
            )
            .unwrap();
            let fd = unsafe { libc::open(p.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
            unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 200) }
        } else {
            exe_fd
        };
        if o.landlock {
            let abi = landlock_abi();
            r.push(format!(
                "landlock abi={abi} {}",
                landlock_lockdown_with(abi, child_fd)
            ));
        }
        if o.seccomp > 0 {
            r.push(seccomp_cost(o.seccomp));
        }
        // Close everything but the kept end: mark all CLOEXEC, then keep
        // sv[1] as fd 3.
        let d = unsafe { libc::dup2(sv[1], 3) };
        let cr = unsafe {
            libc::syscall(
                libc::SYS_close_range,
                4u32,
                u32::MAX,
                4u32, /* CLOSE_RANGE_CLOEXEC */
            )
        };
        let cl = unsafe { libc::fcntl(3, libc::F_SETFD, 0) };
        r.push(format!(
            "dup2={d} close_range_cloexec={} clear-cloexec-3={cl}",
            rc_errno(cr)
        ));
        let line = r.join("\n") + "\n";
        unsafe { libc::write(rp[1], line.as_ptr().cast(), line.len()) };
        let a0 = c"child";
        let argv = [a0.as_ptr(), std::ptr::null()];
        let envp: [*const libc::c_char; 1] = [std::ptr::null()];
        let x = match o.exec_mode {
            1 => {
                let p = CString::new(
                    std::fs::read_link(format!("/proc/self/fd/{exe_fd}"))
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                )
                .unwrap();
                unsafe { libc::execve(p.as_ptr(), argv.as_ptr(), envp.as_ptr()) as i64 }
            }
            2 | 3 => unsafe {
                libc::syscall(
                    libc::SYS_execveat,
                    child_fd,
                    c"".as_ptr(),
                    argv.as_ptr(),
                    envp.as_ptr(),
                    libc::AT_EMPTY_PATH,
                )
            },
            _ => unsafe {
                libc::syscall(
                    libc::SYS_execveat,
                    exe_fd,
                    c"".as_ptr(),
                    argv.as_ptr(),
                    envp.as_ptr(),
                    libc::AT_EMPTY_PATH,
                )
            },
        };
        let msg = format!("execveat failed rc={x} errno={}\n", errno());
        unsafe {
            libc::write(rp[1], msg.as_ptr().cast(), msg.len());
            libc::_exit(127)
        }
    }
    // Parent.
    let outer_pid = rc;
    unsafe {
        libc::close(sv[1]);
        libc::close(rp[1]);
        libc::close(stray);
    }
    let (ready, creds) = recv_with_creds(sv[0]);
    let us = t0.elapsed().as_micros();
    let mut lines = Vec::new();
    if verbose {
        let mut pre = String::new();
        let mut b = [0u8; 4096];
        loop {
            let n = unsafe { libc::read(rp[0], b.as_mut_ptr().cast(), b.len()) };
            if n <= 0 {
                break;
            }
            pre.push_str(&String::from_utf8_lossy(&b[..n as usize]));
        }
        lines.extend(pre.lines().map(|l| format!("  {l}")));
        lines.push(format!(
            "  parent: clone3 returned outer pid={outer_pid} pidfd={pidfd}"
        ));
        lines.push(format!("  parent got: '{ready}' creds[{creds}]"));
        passcred(sv[0]);
        lines.push(format!(
            "  parent SO_PEERCRED on its end: {}",
            peercred(sv[0])
        ));
        let _ = send_with_creds(sv[0], b"hello-from-parent");
        let (back, c2) = recv_with_creds(sv[0]);
        lines.push(format!("  child reply: '{back}' creds[{c2}]"));
    }
    // Reap through the pidfd (waitid P_PIDFD).
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    unsafe {
        libc::close(sv[0]);
        libc::waitid(3 /* P_PIDFD */, pidfd as u32, &mut info, libc::WEXITED);
        libc::close(pidfd);
        libc::close(rp[0]);
    }
    (lines, us)
}

/// Broker -> spawner -> grandchild in a new user+pid namespace. The
/// spawner sends the grandchild's pidfd to the broker over SCM_RIGHTS
/// with the outer pid as data; the broker, not the parent, reads the
/// pid from the pidfd's fdinfo and polls it for exit (pidfd_open(2)).
fn pidfd_probe() {
    let mut sv = [0; 2];
    unsafe {
        libc::socketpair(
            libc::AF_UNIX,
            libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
            0,
            sv.as_mut_ptr(),
        )
    };
    let spawner = unsafe { libc::fork() };
    if spawner == 0 {
        let mut pidfd: i32 = -1;
        let args = CloneArgs {
            flags: CLONE_PIDFD | (libc::CLONE_NEWUSER | libc::CLONE_NEWPID) as u64,
            pidfd: (&raw mut pidfd) as u64,
            exit_signal: libc::SIGCHLD as u64,
            ..Default::default()
        };
        let rc = unsafe {
            libc::syscall(
                libc::SYS_clone3,
                &raw const args,
                std::mem::size_of::<CloneArgs>(),
            )
        };
        if rc == 0 {
            unsafe { libc::usleep(300_000) };
            unsafe { libc::_exit(7) };
        }
        let data = format!("{rc}");
        let mut iov = libc::iovec {
            iov_base: data.as_ptr() as *mut _,
            iov_len: data.len(),
        };
        let mut cbuf = [0u64; 8];
        let mut h: libc::msghdr = unsafe { std::mem::zeroed() };
        h.msg_iov = &mut iov;
        h.msg_iovlen = 1;
        h.msg_control = cbuf.as_mut_ptr().cast();
        h.msg_controllen = unsafe { libc::CMSG_SPACE(4) } as usize;
        unsafe {
            let c = libc::CMSG_FIRSTHDR(&h);
            (*c).cmsg_level = libc::SOL_SOCKET;
            (*c).cmsg_type = libc::SCM_RIGHTS;
            (*c).cmsg_len = libc::CMSG_LEN(4) as usize;
            libc::CMSG_DATA(c).cast::<i32>().write_unaligned(pidfd);
            libc::sendmsg(sv[1], &h, 0);
        }
        // The spawner reaps its child (it is the parent).
        let mut st = 0;
        unsafe { libc::waitpid(rc as i32, &mut st, 0) };
        unsafe { libc::_exit(0) };
    }
    let mut buf = [0u8; 32];
    let mut cbuf = [0u64; 8];
    let mut iov = libc::iovec {
        iov_base: buf.as_mut_ptr().cast(),
        iov_len: buf.len(),
    };
    let mut h: libc::msghdr = unsafe { std::mem::zeroed() };
    h.msg_iov = &mut iov;
    h.msg_iovlen = 1;
    h.msg_control = cbuf.as_mut_ptr().cast();
    h.msg_controllen = std::mem::size_of_val(&cbuf);
    let n = unsafe { libc::recvmsg(sv[0], &mut h, 0) };
    let said = String::from_utf8_lossy(&buf[..n.max(0) as usize]).to_string();
    let first = unsafe { libc::CMSG_FIRSTHDR(&h) };
    if n <= 0 || first.is_null() {
        println!(
            "pidfd via SCM_RIGHTS: no fd received (recvmsg rc={n} errno={})",
            errno()
        );
        return;
    }
    let fd = unsafe { libc::CMSG_DATA(first).cast::<i32>().read_unaligned() };
    let info = read_text(&format!("/proc/self/fdinfo/{fd}"));
    let pid_line = info
        .lines()
        .find(|l| l.starts_with("Pid:"))
        .unwrap_or("Pid: ?")
        .to_string();
    let nspid = info
        .lines()
        .find(|l| l.starts_with("NSpid:"))
        .unwrap_or("NSpid: ?")
        .to_string();
    let t0 = Instant::now();
    let mut p = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let pr = unsafe { libc::poll(&mut p, 1, 2000) };
    let ms = t0.elapsed().as_millis();
    let mut info2: libc::siginfo_t = unsafe { std::mem::zeroed() };
    let w = unsafe { libc::waitid(3, fd as u32, &mut info2, libc::WEXITED | libc::WNOHANG) };
    let werr = if w < 0 {
        format!("waitid-errno={}", errno())
    } else {
        "waitid-ok".into()
    };
    let sig = unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            fd,
            0,
            std::ptr::null::<u8>(),
            0,
        )
    };
    println!(
        "pidfd via SCM_RIGHTS: spawner said outer pid={said}; broker fdinfo {pid_line} | {nspid}; poll rc={pr} revents={:#x} after {ms} ms; non-parent {werr}; send_signal(0) after exit {}",
        p.revents,
        rc_errno(sig)
    );
    let mut st = 0;
    unsafe { libc::waitpid(spawner, &mut st, 0) };
}

fn median(mut v: Vec<u128>) -> u128 {
    v.sort_unstable();
    v[v.len() / 2]
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.first().map(String::as_str) == Some("child") {
        child_main();
    }
    if a.get(1).map(String::as_str) == Some("pidfd") {
        pidfd_probe();
        return;
    }
    if a.get(1).map(String::as_str) != Some("probe") {
        eprintln!("usage: e-linux-probes probe <child-binary> [runs]");
        std::process::exit(2);
    }
    let child = CString::new(a[2].clone()).unwrap();
    let runs: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(20);
    let low = unsafe { libc::open(child.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
    assert!(low >= 0, "open O_PATH: {}", errno());
    // Keep the executable's fd clear of 3, which the child's kept end
    // takes (the first run of this probe overwrote it there).
    let exe_fd = unsafe { libc::fcntl(low, libc::F_DUPFD_CLOEXEC, 100) };
    unsafe { libc::close(low) };
    let mut out = std::io::stdout().lock();
    writeln!(
        out,
        "kernel={} host-uid={} child={}",
        read_text("/proc/sys/kernel/osrelease").trim(),
        unsafe { libc::getuid() },
        a[2]
    )
    .unwrap();
    for f in [
        "/proc/sys/kernel/apparmor_restrict_unprivileged_userns",
        "/proc/sys/kernel/unprivileged_userns_clone",
        "/proc/sys/user/max_user_namespaces",
        "/proc/sys/kernel/yama/ptrace_scope",
    ] {
        writeln!(out, "{f}={}", read_text(f).trim()).unwrap();
    }
    writeln!(out, "landlock abi (host)={}", landlock_abi()).unwrap();
    let ns =
        (libc::CLONE_NEWUSER | libc::CLONE_NEWPID | libc::CLONE_NEWNET | libc::CLONE_NEWNS) as u64;
    let u = libc::CLONE_NEWUSER as u64;
    let only: Option<&str> = a.get(4).map(String::as_str);
    let cases: [(&str, Opts); 14] = [
        (
            "plain (no namespaces)",
            Opts {
                exec_mode: 0,
                flags: 0,
                pivot: false,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "user only",
            Opts {
                exec_mode: 0,
                flags: u,
                pivot: false,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "user+pid",
            Opts {
                exec_mode: 0,
                flags: u | libc::CLONE_NEWPID as u64,
                pivot: false,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "user+net",
            Opts {
                exec_mode: 0,
                flags: u | libc::CLONE_NEWNET as u64,
                pivot: false,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "user+mnt",
            Opts {
                exec_mode: 0,
                flags: u | libc::CLONE_NEWNS as u64,
                pivot: false,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "user+mnt exec-by-path",
            Opts {
                exec_mode: 1,
                flags: u | libc::CLONE_NEWNS as u64,
                pivot: false,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "user+mnt exec-fd-opened-in-child",
            Opts {
                exec_mode: 2,
                flags: u | libc::CLONE_NEWNS as u64,
                pivot: false,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "FULL: user+pid+net+mnt, fd-in-child, landlock(exe rule), seccomp(200)",
            Opts {
                exec_mode: 2,
                flags: ns,
                pivot: false,
                landlock: true,
                seccomp: 200,
            },
        ),
        (
            "user+mnt exec-procfs-reopen",
            Opts {
                exec_mode: 3,
                flags: u | libc::CLONE_NEWNS as u64,
                pivot: false,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "pid only (no user ns)",
            Opts {
                exec_mode: 0,
                flags: libc::CLONE_NEWPID as u64,
                pivot: false,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "user+pid+net+mnt",
            Opts {
                exec_mode: 0,
                flags: ns,
                pivot: false,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "user+pid+net+mnt, pivot to empty tmpfs",
            Opts {
                exec_mode: 0,
                flags: ns,
                pivot: true,
                landlock: false,
                seccomp: 0,
            },
        ),
        (
            "all ns + landlock + seccomp(200)",
            Opts {
                exec_mode: 0,
                flags: ns,
                pivot: false,
                landlock: true,
                seccomp: 200,
            },
        ),
        (
            "all ns + pivot + landlock + seccomp(200)",
            Opts {
                exec_mode: 0,
                flags: ns,
                pivot: true,
                landlock: true,
                seccomp: 200,
            },
        ),
    ];
    for (name, o) in &cases {
        if let Some(f) = only {
            if !name.contains(f) {
                continue;
            }
        }
        writeln!(out, "== {name}").unwrap();
        let (lines, us) = spawn(exe_fd, o, true);
        for l in lines {
            writeln!(out, "{l}").unwrap();
        }
        writeln!(out, "  first spawn-to-ready us={us}").unwrap();
        let v: Vec<u128> = (0..runs).map(|_| spawn(exe_fd, o, false).1).collect();
        let mx = *v.iter().max().unwrap();
        writeln!(
            out,
            "  spawn-to-ready over {runs} runs: median us={} max us={mx}",
            median(v)
        )
        .unwrap();
    }
}
