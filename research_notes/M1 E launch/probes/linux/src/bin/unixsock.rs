//! M1 phase E, review M1: can a child on the restricted tier reach a
//! path-named Unix socket (the D-Bus buses, X11, docker.sock)?
//!
//! Written from man pages only: unix(7), landlock(7), seccomp(2),
//! clone(2) (clone3), user_namespaces(7), pid_namespaces(7),
//! network_namespaces(7), stat(2), sendto(2).
//!
//! Each stage runs in a fresh child and tries, for every target socket:
//! `stat` (the `newfstatat` existence question), `socket`+`connect` with a
//! new socket, `connect` with a socket made before the sandbox, and
//! closes at once. Nothing is ever written to a target the probe does
//! not own: a successful connect is closed without a byte sent. The one
//! `sendto` with an address goes to a datagram socket this probe binds
//! in its own temporary directory.
//!
//! Stages (the restricted tier of launch facts E22, built up):
//! - `plain`: a forked child, no namespaces, no Landlock, no seccomp.
//! - `ns`: clone3(NEWUSER|NEWPID|NEWNET), no mount namespace.
//! - `ns+landlock`: plus Landlock handling every filesystem right of the
//!   ABI and both TCP rights, with no rule.
//! - `ns+landlock+seccomp`: plus the D3 steady filter, installed by
//!   `hg_substrate_os::sandbox::linux::apply_empty_process_profile` (the
//!   real table, not a copy).
//!
//! Usage: `unixsock [extra-socket-path ...]`.
#![allow(unsafe_code, clippy::all)]

use std::ffi::CString;
use std::os::fd::RawFd;

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

fn res(rc: i64) -> String {
    if rc < 0 {
        let e = errno();
        let name = match e {
            1 => "EPERM",
            2 => "ENOENT",
            13 => "EACCES",
            22 => "EINVAL",
            91 => "EPROTOTYPE",
            97 => "EAFNOSUPPORT",
            106 => "EISCONN",
            111 => "ECONNREFUSED",
            _ => "",
        };
        format!("{e}{}{name}", if name.is_empty() { "" } else { ":" })
    } else {
        "OK".into()
    }
}

/// A sockaddr_un for `path`; a leading '@' names an abstract socket.
fn addr(path: &str) -> (libc::sockaddr_un, u32) {
    let mut a: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    a.sun_family = libc::AF_UNIX as u16;
    let bytes = path.as_bytes();
    let (abs, body) = match bytes.first() {
        Some(b'@') => (true, &bytes[1..]),
        _ => (false, bytes),
    };
    let off = usize::from(abs);
    for (i, b) in body.iter().enumerate().take(a.sun_path.len() - 1 - off) {
        a.sun_path[i + off] = *b as libc::c_char;
    }
    let base = std::mem::size_of::<libc::sa_family_t>();
    let len = if abs {
        base + 1 + body.len()
    } else {
        base + body.len() + 1
    };
    (a, len as u32)
}

fn landlock_abi() -> i64 {
    unsafe { libc::syscall(libc::SYS_landlock_create_ruleset, 0usize, 0usize, 1u32) }
}

/// Every filesystem right of the ABI and both TCP rights, no rule
/// (landlock(7)); the child's restricted-tier Landlock with no exe rule.
fn landlock_all(abi: i64) -> String {
    #[repr(C)]
    struct Attr {
        fs: u64,
        net: u64,
    }
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
    let attr = Attr { fs, net };
    let size = if abi >= 4 { 16usize } else { 8 };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            &raw const attr,
            size,
            0u32,
        )
    };
    if fd < 0 {
        return format!("landlock create {}", res(fd));
    }
    unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    let rc = unsafe { libc::syscall(libc::SYS_landlock_restrict_self, fd, 0u32) };
    unsafe { libc::close(fd as i32) };
    format!("landlock abi={abi} restrict={}", res(rc))
}

/// What the probe holds before any sandbox: a stream socket, a datagram
/// socket and a seqpacket pair, and its own datagram target.
struct Pre {
    stream: RawFd,
    dgram: RawFd,
    pair: RawFd,
    own_target: String,
}

fn try_targets(stage: &str, targets: &[String], pre: &Pre) {
    for t in targets {
        let mut line = format!("{stage:<20} {t:<40}");
        if !t.starts_with('@') {
            let c = CString::new(t.as_str()).unwrap();
            let mut st: libc::stat = unsafe { std::mem::zeroed() };
            let rc = unsafe { libc::stat(c.as_ptr(), &mut st) };
            line += &format!(" stat={}", res(rc as i64));
        }
        let (a, len) = addr(t);
        let s = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) };
        line += &format!(" socket={}", res(s as i64));
        if s >= 0 {
            let rc = unsafe { libc::connect(s, (&raw const a).cast(), len) };
            line += &format!(" connect={}", res(rc as i64));
            unsafe { libc::close(s) };
        }
        // A socket made before the sandbox: separates `socket` from
        // `connect`. A stream socket connects once, so a fresh dup is
        // not enough; use it for the first target only.
        if t == &targets[0] {
            let rc = unsafe { libc::connect(pre.stream, (&raw const a).cast(), len) };
            line += &format!(" preconnect={}", res(rc as i64));
        }
        println!("{line}");
    }
    // Datagrams with an address, to the probe's own socket only.
    let (a, len) = addr(&pre.own_target);
    let msg = b"probe";
    let rc = unsafe {
        libc::sendto(
            pre.dgram,
            msg.as_ptr().cast(),
            msg.len(),
            libc::MSG_DONTWAIT,
            (&raw const a).cast(),
            len,
        )
    };
    let rc2 = unsafe {
        libc::sendto(
            pre.pair,
            msg.as_ptr().cast(),
            msg.len(),
            libc::MSG_DONTWAIT,
            (&raw const a).cast(),
            len,
        )
    };
    let sp = {
        let mut sv = [0; 2];
        let r = unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_DGRAM, 0, sv.as_mut_ptr()) };
        res(r as i64)
    };
    println!(
        "{stage:<20} own-dgram-target: sendto(pre-made dgram)={} sendto(seqpacket pair end)={} socketpair(DGRAM)={sp}",
        res(rc as i64),
        res(rc2 as i64)
    );
}

fn run_stage(
    stage: &str,
    flags: u64,
    landlock: bool,
    seccomp: bool,
    targets: &[String],
    pre: &mut Pre,
) {
    use std::io::Write;
    let _ = std::io::stdout().flush();
    // A fresh pre-made stream socket per stage: one that connected in an
    // earlier stage would report EISCONN.
    if pre.stream >= 0 {
        unsafe { libc::close(pre.stream) };
    }
    pre.stream = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
    let args = CloneArgs {
        flags,
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
    if rc < 0 {
        println!("{stage:<20} clone3={}", res(rc));
        return;
    }
    if rc == 0 {
        let mut head = format!(
            "{stage:<20} pid={} uid={}",
            unsafe { libc::getpid() },
            unsafe { libc::getuid() }
        );
        if landlock {
            head += &format!(" {}", landlock_all(landlock_abi()));
        }
        if seccomp {
            let r = hg_substrate_os::sandbox::linux::apply_empty_process_profile();
            head += &format!(
                " seccomp={}",
                if r.is_ok() {
                    "OK".into()
                } else {
                    format!("{r:?}")
                }
            );
        }
        println!("{head}");
        try_targets(stage, targets, pre);
        let _ = std::io::stdout().flush();
        unsafe { libc::_exit(0) };
    }
    let mut status = 0;
    unsafe { libc::waitpid(rc as i32, &mut status, 0) };
    println!("{stage:<20} child exit status={status:#x}");
}

fn main() {
    let uid = unsafe { libc::getuid() };
    let mut targets: Vec<String> = vec![
        format!("/run/user/{uid}/bus"),
        "/run/dbus/system_bus_socket".into(),
        "/var/run/docker.sock".into(),
    ];
    if let Ok(rd) = std::fs::read_dir("/tmp/.X11-unix") {
        let mut xs: Vec<String> = rd
            .filter_map(|e| e.ok())
            .map(|e| e.path().display().to_string())
            .collect();
        xs.sort();
        for x in xs {
            // The X server's abstract twin, reachable only in its own
            // network namespace.
            targets.push(format!("@{x}"));
            targets.push(x);
        }
    } else {
        targets.push("/tmp/.X11-unix/X0".into());
        targets.push("@/tmp/.X11-unix/X0".into());
    }
    targets.extend(std::env::args().skip(1));
    println!(
        "dbus-session-env={:?}",
        std::env::var("DBUS_SESSION_BUS_ADDRESS").ok()
    );
    println!("display-env={:?}", std::env::var("DISPLAY").ok());
    println!("landlock-abi={}", landlock_abi());

    // The probe's own datagram target, in a private temporary directory.
    let dir = std::env::temp_dir().join(format!("izcal-unixsock-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let own_target = dir.join("target.sock").display().to_string();
    let own = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC, 0) };
    let (a, len) = addr(&own_target);
    let b = unsafe { libc::bind(own, (&raw const a).cast(), len) };
    println!("own-target={own_target} bind={}", res(b as i64));

    // The probe's own abstract listener: the abstract namespace belongs
    // to the network namespace (unix(7)), so a child in a new one should
    // not reach it, and Landlock ABI 4 has no abstract-socket scoping.
    let abs_name = format!("@izcal-unixsock-{}", std::process::id());
    let abs = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) };
    let (aa, alen) = addr(&abs_name);
    let ab = unsafe { libc::bind(abs, (&raw const aa).cast(), alen) };
    let al = unsafe { libc::listen(abs, 8) };
    println!(
        "own-abstract={abs_name} bind={} listen={}",
        res(ab as i64),
        res(al as i64)
    );
    targets.push(abs_name);

    let mut pre = Pre {
        stream: unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) },
        dgram: unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_DGRAM, 0) },
        pair: -1,
        own_target,
    };
    let mut sv = [0; 2];
    unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_SEQPACKET, 0, sv.as_mut_ptr()) };
    pre.pair = sv[0];

    let ns = (libc::CLONE_NEWUSER | libc::CLONE_NEWPID | libc::CLONE_NEWNET) as u64;
    run_stage("plain", 0, false, false, &targets, &mut pre);
    run_stage("ns", ns, false, false, &targets, &mut pre);
    run_stage("ns+landlock", ns, true, false, &targets, &mut pre);
    run_stage("ns+landlock+seccomp", ns, true, true, &targets, &mut pre);
    // Landlock alone without namespaces: whether the abstract X11 twin
    // is reachable once the network namespace is not there to hide it.
    run_stage("landlock-only", 0, true, false, &targets, &mut pre);

    // What reached the probe's own target, from which stages.
    let mut got = 0;
    let mut buf = [0u8; 16];
    while unsafe { libc::recv(own, buf.as_mut_ptr().cast(), buf.len(), libc::MSG_DONTWAIT) } > 0 {
        got += 1;
    }
    println!("own-target received {got} datagram(s)");
    unsafe { libc::close(own) };
    let _ = std::fs::remove_dir_all(&dir);
}
