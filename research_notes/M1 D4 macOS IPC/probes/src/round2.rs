//! Round 2 (D4 design review): robustness checks of the transport's own
//! error paths. Same rules as round 1: black-box only, every constant from
//! ffi::k (found by `discover`).
use crate::ffi::{self, k, me};
use crate::mach::*;
use std::ffi::CString;

fn hx(kr: i32) -> String {
    hex(kr)
}

fn msgcount(p: u32) -> i32 {
    status(p).map(|s| s[4]).unwrap_or(-1)
}

/// R1: a message too large for the buffer, received through a port set.
pub fn r1() {
    let set = alloc_set();
    let p = alloc_recv_send();
    assert_eq!(unsafe { ffi::mach_port_insert_member(me(), p, set) }, 0);
    // A carried right whose no-senders tells us when it is released.
    let q = alloc_recv_send();
    let n = alloc_recv();
    let (kr, _) = request(q, k::NOTIFY_NO_SENDERS, 1, n, k::MAKE_SEND_ONCE);
    assert_eq!(kr, 0);
    let mut o = Out::to(p, k::COPY_SEND, 4242);
    o.body = vec![7u8; 1000];
    o.ports = vec![(q, k::MOVE_SEND)];
    assert_eq!(o.send(), 0);
    // RCV_LARGE on the set, 64-byte buffer.
    for attempt in 0..2 {
        let mut buf = [0u64; 16];
        let kr = unsafe {
            ffi::mach_msg_overwrite(
                buf.as_mut_ptr().cast(),
                k::RCV_MSG
                    | k::RCV_LARGE
                    | k::RCV_TIMEOUT
                    | k::rcv_trailer_elements(k::AUDIT_ELEMENTS),
                0,
                64,
                set,
                0,
                0,
                std::ptr::null_mut(),
                0,
            )
        };
        let w: Vec<u32> = buf
            .iter()
            .flat_map(|x| [*x as u32, (*x >> 32) as u32])
            .collect();
        println!(
            "R1 set RCV_LARGE attempt {attempt}: kr={} header words [bits={:#x} size={} remote={:#x} local={:#x} voucher={:#x} id={}] (p={p:#x}) msgcount(p)={} trailer-ish words 6..16={:x?}",
            hx(kr), w[0], w[1], w[2], w[3], w[4], w[5] as i32, msgcount(p), &w[6..16]
        );
    }
    // Extract the member, receive without RCV_LARGE into the small buffer.
    assert_eq!(unsafe { ffi::mach_port_extract_member(me(), p, set) }, 0);
    let mut buf = [0u64; 16];
    let kr = unsafe {
        ffi::mach_msg_overwrite(
            buf.as_mut_ptr().cast(),
            k::RCV_MSG | k::RCV_TIMEOUT | k::rcv_trailer_elements(k::AUDIT_ELEMENTS),
            0,
            64,
            p,
            0,
            0,
            std::ptr::null_mut(),
            0,
        )
    };
    let ns = recv(n, Some(200));
    println!(
        "R1 direct receive without RCV_LARGE: kr={} msgcount(p) after={} carried right released (no-senders on q)={}",
        hx(kr),
        msgcount(p),
        matches!(&ns, Ok(m) if m.id == k::NOTIFY_NO_SENDERS)
    );
}

/// R2: notification requests and the rights they hold.
pub fn r2() {
    // (a) A holder of a send-once right to port P names it as the notify
    // port of a request on its own port Q.
    let p = alloc_recv();
    let mut so = 0u32;
    let mut so_t = 0u32;
    let kr =
        unsafe { ffi::mach_port_extract_right(me(), p, k::MAKE_SEND_ONCE, &mut so, &mut so_t) };
    println!(
        "R2a make send-once to P: kr={} name={so:#x} type={so_t}",
        hx(kr)
    );
    let q = alloc_recv_send();
    let (kr, prev) = request(q, k::NOTIFY_NO_SENDERS, 1, so, k::MOVE_SEND_ONCE);
    println!(
        "R2a no-senders on Q aimed at the send-once right to P: kr={} prev={prev:#x}",
        hx(kr)
    );
    unsafe { ffi::mach_port_deallocate(me(), q) };
    match recv(p, Some(300)) {
        Ok(m) => println!(
            "R2a P received: id={} complex={} size={} token pid={} ver={} body={:x?}",
            m.id,
            m.bits & k::MSGH_BITS_COMPLEX != 0,
            m.size,
            m.pid(),
            m.pidver(),
            m.body
        ),
        Err(e) => println!("R2a P received nothing: {}", hx(e)),
    }
    // (b) Re-arming returns the previous notify right.
    let r = alloc_recv_send();
    let n1 = alloc_recv();
    let n2 = alloc_recv();
    let (kr1, prev1) = request(r, k::NOTIFY_NO_SENDERS, 1, n1, k::MAKE_SEND_ONCE);
    let (kr2, prev2) = request(r, k::NOTIFY_NO_SENDERS, 1, n2, k::MAKE_SEND_ONCE);
    println!(
        "R2b arm n1: kr={} prev={prev1:#x}; re-arm n2: kr={} prev={prev2:#x} type(prev)={:?}",
        hx(kr1),
        hx(kr2),
        ptype(prev2).map(|t| format!("{t:#x}"))
    );
    // (c) Cancel: NULL notify returns the armed right; destroying it.
    let (kr3, prev3) = request(r, k::NOTIFY_NO_SENDERS, 1, 0, k::MAKE_SEND_ONCE);
    println!(
        "R2c cancel: kr={} prev={prev3:#x} type={:?}",
        hx(kr3),
        ptype(prev3).map(|t| format!("{t:#x}"))
    );
    unsafe { ffi::mach_port_deallocate(me(), prev2) };
    unsafe { ffi::mach_port_deallocate(me(), prev3) };
    let a = recv(n1, Some(200)).map(|m| (m.id, m.pid(), m.size));
    let b = recv(n2, Some(200)).map(|m| (m.id, m.pid(), m.size));
    println!("R2c after destroying the returned rights: n1 got {a:?}, n2 got {b:?}");
    // (d) No-senders aimed at the data port itself, behind queued data.
    for trial in 0..3 {
        let d = alloc_recv_send();
        let (kr, prev) = request(d, k::NOTIFY_NO_SENDERS, 1, d, k::MAKE_SEND_ONCE);
        assert_eq!(kr, 0, "arm on self");
        assert_eq!(prev, 0);
        for i in 0..3 {
            assert_eq!(Out::to(d, k::COPY_SEND, 100 + i).send(), 0);
        }
        unsafe { ffi::mach_port_deallocate(me(), d) };
        let mut ids = vec![];
        while let Ok(m) = recv(d, Some(100)) {
            ids.push((m.id, m.pid(), m.size, m.bits & k::MSGH_BITS_COMPLEX != 0));
        }
        println!("R2d trial {trial}: order on the data port (id, pid, size, complex) = {ids:?}");
    }
}

/// R5: a send right this task already holds, sent back to it; a reply
/// right made with COPY_SEND.
pub fn r5() {
    let p = alloc_recv();
    let mut bp = 0u32;
    unsafe { ffi::task_get_special_port(me(), k::TASK_BOOTSTRAP_PORT, &mut bp) };
    let before = refs(bp, k::RIGHT_SEND);
    let mut o = Out::to(p, k::MAKE_SEND, 1);
    o.ports = vec![(bp, k::COPY_SEND)];
    let reply = alloc_recv_send();
    o.reply = reply;
    o.reply_disp = k::COPY_SEND;
    assert_eq!(o.send(), 0);
    let m = recv(p, Some(200)).unwrap();
    println!(
        "R5 bootstrap {bp:#x}: received name={:#x} disp={} same name={} urefs before={before:?} after={:?} type={:?}; reply remote={:#x} remote_disp={} reply urefs={:?}",
        m.ports[0].0,
        m.ports[0].1,
        m.ports[0].0 == bp,
        refs(bp, k::RIGHT_SEND),
        ptype(bp).map(|t| format!("{t:#x}")),
        m.remote,
        m.remote_disp(),
        refs(reply, k::RIGHT_SEND)
    );
}

/// R7: a send that fails because the destination is gone, carrying a
/// moved receive right and a moved send right: are they back?
pub fn r7() {
    let d = alloc_recv_send();
    let x = alloc_recv();
    let y = alloc_recv_send();
    let (before_y, _) = (refs(y, k::RIGHT_SEND), 0);
    // Destroy d's receive right: our name becomes a dead name.
    unsafe { ffi::mach_port_mod_refs(me(), d, k::RIGHT_RECEIVE, -1) };
    let mut o = Out::to(d, k::COPY_SEND, 1);
    o.ports = vec![(x, k::MOVE_RECEIVE), (y, k::MOVE_SEND)];
    let (kr, buf) = o.send_opts(k::SEND_TIMEOUT, 0);
    let w: Vec<u32> = buf
        .iter()
        .flat_map(|v| [*v as u32, (*v >> 32) as u32])
        .collect();
    println!(
        "R7 send to a dead port: kr={} type(x)={:?} type(y)={:?} y urefs before={before_y:?} after={:?} buffer port names now [{:#x}, {:#x}]",
        hx(kr),
        ptype(x).map(|t| format!("{t:#x}")),
        ptype(y).map(|t| format!("{t:#x}")),
        refs(y, k::RIGHT_SEND),
        w[7],
        w[10]
    );
}

fn self_exe() -> CString {
    CString::new(std::env::current_exe().unwrap().to_str().unwrap()).unwrap()
}

fn spawn_with_boot(args: &[&str], boot: u32) -> i32 {
    let exe = self_exe();
    let mut cs: Vec<CString> = vec![CString::new("d4-probes").unwrap()];
    cs.extend(args.iter().map(|a| CString::new(*a).unwrap()));
    let mut argv: Vec<*mut libc::c_char> = cs.iter().map(|a| a.as_ptr() as *mut _).collect();
    argv.push(std::ptr::null_mut());
    let mut attr: libc::posix_spawnattr_t = std::ptr::null_mut();
    let mut pid = 0;
    unsafe {
        libc::posix_spawnattr_init(&mut attr);
        libc::posix_spawnattr_setflags(&mut attr, libc::POSIX_SPAWN_START_SUSPENDED as i16);
        ffi::posix_spawnattr_setspecialport_np(&mut attr, boot, k::TASK_BOOTSTRAP_PORT);
        extern "C" {
            static environ: *const *mut libc::c_char;
        }
        let r = libc::posix_spawn(
            &mut pid,
            exe.as_ptr(),
            std::ptr::null(),
            &attr,
            argv.as_ptr(),
            environ,
        );
        libc::posix_spawnattr_destroy(&mut attr);
        assert_eq!(r, 0, "posix_spawn");
    }
    pid
}

fn wait_code(pid: i32) -> (i32, u64) {
    let t = now_ns();
    let mut st = 0;
    unsafe { libc::waitpid(pid, &mut st, 0) };
    let code = if libc::WIFEXITED(st) {
        libc::WEXITSTATUS(st)
    } else {
        128 + libc::WTERMSIG(st)
    };
    (code, (now_ns() - t) / 1_000_000)
}

/// R4 child: optionally says hello on its bootstrap port, then applies
/// the deny-default profile; exits 0 on success, 3 if it failed. Prints
/// the time sandbox_init took.
pub fn r4child(hello_first: bool) {
    if hello_first {
        let mut bp = 0u32;
        unsafe { ffi::task_get_special_port(me(), k::TASK_BOOTSTRAP_PORT, &mut bp) };
        let _ = Out::to(bp, k::COPY_SEND, 1000).send_nb();
        // Give the parent time to destroy the port.
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    let t = now_ns();
    let r = crate::child::apply_sandbox("(version 1)(deny default)(allow sysctl-read)");
    eprintln!(
        "R4 child: sandbox_init {:?} in {} us",
        r.as_ref().map(|_| ()),
        (now_ns() - t) / 1000
    );
    std::process::exit(if r.is_ok() { 0 } else { 3 });
}

/// R4: a child whose bootstrap port is destroyed (a) before it starts,
/// (b) after its hello but before it applies its sandbox; and 50
/// concurrent children each with their own bootstrap port.
pub fn r4() {
    // (a)
    let boot = alloc_recv_send();
    let pid = spawn_with_boot(&["r4child", "no-hello"], boot);
    unsafe { ffi::mach_port_mod_refs(me(), boot, k::RIGHT_RECEIVE, -1) };
    unsafe { ffi::mach_port_deallocate(me(), boot) };
    unsafe { libc::kill(pid, libc::SIGCONT) };
    let (code, ms) = wait_code(pid);
    println!("R4a bootstrap destroyed before start: child exit {code} after {ms} ms");
    // (b)
    let boot = alloc_recv_send();
    let pid = spawn_with_boot(&["r4child", "hello"], boot);
    unsafe { libc::kill(pid, libc::SIGCONT) };
    let mut pre = 0;
    let got = loop {
        match recv(boot, Some(5000)) {
            Ok(m) if m.id == 1000 => break true,
            Ok(m) => {
                pre += 1;
                release(&m);
            }
            Err(_) => break false,
        }
    };
    unsafe { ffi::mach_port_mod_refs(me(), boot, k::RIGHT_RECEIVE, -1) };
    unsafe { ffi::mach_port_deallocate(me(), boot) };
    let (code, ms) = wait_code(pid);
    println!("R4b hello seen={got} (after {pre} other messages); bootstrap destroyed; child exit {code} after {ms} ms");
    // 50 concurrent children, each with its own port; each is destroyed
    // after that child's hello.
    let mut kids = vec![];
    for _ in 0..50 {
        let b = alloc_recv_send();
        let pid = spawn_with_boot(&["r4child", "hello"], b);
        unsafe { libc::kill(pid, libc::SIGCONT) };
        kids.push((pid, b));
    }
    let mut hellos = 0;
    for &(_, b) in &kids {
        loop {
            match recv(b, Some(5000)) {
                Ok(m) if m.id == 1000 => {
                    hellos += 1;
                    break;
                }
                Ok(m) => release(&m),
                Err(_) => break,
            }
        }
        unsafe { ffi::mach_port_mod_refs(me(), b, k::RIGHT_RECEIVE, -1) };
        unsafe { ffi::mach_port_deallocate(me(), b) };
    }
    let codes: Vec<i32> = kids.iter().map(|&(p, _)| wait_code(p).0).collect();
    println!(
        "R4c 50 children, own bootstrap ports: hellos={hellos} exit codes all 0={} ({:?})",
        codes.iter().all(|&c| c == 0),
        codes.iter().filter(|&&c| c != 0).count()
    );
}

/// R6 child: receives a send right from the parent on its bootstrap port
/// and sends 100 messages on it at once.
pub fn r6child() {
    let mut bp = 0u32;
    unsafe { ffi::task_get_special_port(me(), k::TASK_BOOTSTRAP_PORT, &mut bp) };
    let mine = alloc_recv();
    let mut o = Out::to(bp, k::COPY_SEND, 1000);
    o.reply = mine;
    o.reply_disp = k::MAKE_SEND_ONCE;
    let _ = o.send();
    let m = recv(mine, Some(5000)).unwrap();
    let d = m.ports[0].0;
    for i in 0..100 {
        let _ = Out::to(d, k::COPY_SEND, 2000 + i).send();
    }
    std::process::exit(0);
}

/// R6: process A sends 100 on a send right, then moves it to B, which
/// sends 100 at once: are any of B's dequeued before A's?
pub fn r6() {
    let mut overtakes = 0;
    let trials = 20;
    for _ in 0..trials {
        let d = alloc_recv_send();
        set_qlimit(d, 1024);
        let boot = alloc_recv_send();
        let pid = spawn_with_boot(&["r6child"], boot);
        unsafe { libc::kill(pid, libc::SIGCONT) };
        let hello = loop {
            match recv(boot, Some(5000)) {
                Ok(m) if m.id == 1000 => break m,
                Ok(m) => release(&m),
                Err(e) => panic!("no hello {}", hx(e)),
            }
        };
        for i in 0..100 {
            assert_eq!(Out::to(d, k::COPY_SEND, 1000 + i).send(), 0);
        }
        let mut give = Out::to(hello.remote, k::MOVE_SEND_ONCE, 1);
        give.ports = vec![(d, k::MOVE_SEND)];
        assert_eq!(give.send(), 0);
        let mut ids = vec![];
        while ids.len() < 200 {
            match recv(d, Some(3000)) {
                Ok(m) => ids.push(m.id),
                Err(_) => break,
            }
        }
        let last_a = ids.iter().rposition(|&i| i < 2000).unwrap_or(0);
        let first_b = ids.iter().position(|&i| i >= 2000).unwrap_or(ids.len());
        if first_b < last_a {
            overtakes += 1;
        }
        let _ = wait_code(pid);
        unsafe { ffi::mach_port_mod_refs(me(), d, k::RIGHT_RECEIVE, -1) };
        unsafe { ffi::mach_port_mod_refs(me(), boot, k::RIGHT_RECEIVE, -1) };
    }
    println!(
        "R6 hand-off ordering: B's messages before A's last in {overtakes} of {trials} trials"
    );
}

/// R3: kernel memory for queued messages. A child sends `n` messages of
/// `mib` MiB inline to our port and waits; we read both processes'
/// footprint (the `footprint` tool) before and after.
pub fn r3(n: usize, mib: usize) {
    let p = alloc_recv_send();
    set_qlimit(p, 1024);
    let boot = alloc_recv_send();
    let pid = spawn_with_boot(&["r3child", &n.to_string(), &mib.to_string()], boot);
    unsafe { libc::kill(pid, libc::SIGCONT) };
    let hello = loop {
        match recv(boot, Some(5000)) {
            Ok(m) if m.id == 1000 => break m,
            Ok(m) => release(&m),
            Err(e) => panic!("no hello {}", hx(e)),
        }
    };
    let fp = |who: i32| -> String {
        let o = std::process::Command::new("footprint")
            .arg("-p")
            .arg(who.to_string())
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
        o.lines()
            .find(|l| l.contains("phys_footprint:") || l.contains("Footprint:"))
            .unwrap_or("?")
            .trim()
            .to_string()
    };
    let me_pid = std::process::id() as i32;
    let before = (fp(me_pid), fp(pid));
    let mut give = Out::to(hello.remote, k::MOVE_SEND_ONCE, 1);
    give.ports = vec![(p, k::MAKE_SEND)];
    assert_eq!(give.send(), 0);
    std::thread::sleep(std::time::Duration::from_secs(3));
    let after = (fp(me_pid), fp(pid));
    println!(
        "R3 {n} x {mib} MiB queued (msgcount {}): receiver footprint {} -> {}; sender footprint {} -> {}",
        msgcount(p),
        before.0,
        after.0,
        before.1,
        after.1
    );
    unsafe { libc::kill(pid, libc::SIGKILL) };
    let _ = wait_code(pid);
    unsafe { ffi::mach_port_mod_refs(me(), p, k::RIGHT_RECEIVE, -1) };
}

/// R3 child: sends n messages of mib MiB inline, then waits.
pub fn r3child(n: usize, mib: usize) {
    let mut bp = 0u32;
    unsafe { ffi::task_get_special_port(me(), k::TASK_BOOTSTRAP_PORT, &mut bp) };
    let mine = alloc_recv();
    let mut o = Out::to(bp, k::COPY_SEND, 1000);
    o.reply = mine;
    o.reply_disp = k::MAKE_SEND_ONCE;
    let _ = o.send();
    let m = recv(mine, Some(5000)).unwrap();
    let d = m.ports[0].0;
    let mut out = Out::to(d, k::COPY_SEND, 3000);
    out.body = vec![1u8; mib * 1024 * 1024];
    let mut sent = 0;
    for _ in 0..n {
        if out.send_nb() == 0 {
            sent += 1;
        }
    }
    eprintln!("R3 child sent {sent}");
    std::thread::sleep(std::time::Duration::from_secs(30));
    std::process::exit(0);
}
