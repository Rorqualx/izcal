//! Parent side of the cross-process probes. Children reach us only through the
//! bootstrap special port set with posix_spawnattr_setspecialport_np.
use crate::ffi::{self, k, me};
use crate::mach::*;
use crate::out;
use std::ffi::CString;

pub struct Child {
    pub pid: i32,
    pub port: u32,
    pub broker: u32,
    pub hello: Rcvd,
    pub pre: Vec<Rcvd>,
    pub suspended_token: Option<[u32; 8]>,
    pub label: &'static str,
    pub sandbox_result: String,
    /// broker traffic after hello and before the sandbox report (from sandbox_init itself)
    pub post: Vec<Rcvd>,
}

pub fn spawn(
    role: &str,
    extra: &[&str],
    suspended: bool,
    special: &[(i32, u32)],
) -> Result<Child, String> {
    spawn_env(role, extra, suspended, special, false)
}
pub fn spawn_env(
    role: &str,
    extra: &[&str],
    suspended: bool,
    special: &[(i32, u32)],
    empty_env: bool,
) -> Result<Child, String> {
    let broker = alloc_recv_send();
    set_qlimit(broker, 1024);
    let exe = CString::new(std::env::var("D4_CHILD_EXE").unwrap_or_else(|_| {
        std::env::current_exe()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }))
    .unwrap();
    let mut args: Vec<CString> = vec![
        CString::new("d4-probes").unwrap(),
        CString::new("child").unwrap(),
        CString::new(role).unwrap(),
    ];
    for e in extra {
        args.push(CString::new(*e).unwrap());
    }
    let mut argv: Vec<*mut libc::c_char> = args.iter().map(|a| a.as_ptr() as *mut _).collect();
    argv.push(std::ptr::null_mut());
    let mut attr: libc::posix_spawnattr_t = std::ptr::null_mut();
    let mut pid = 0;
    unsafe {
        libc::posix_spawnattr_init(&mut attr);
        if suspended {
            libc::posix_spawnattr_setflags(&mut attr, libc::POSIX_SPAWN_START_SUSPENDED as i16);
        }
        let r = ffi::posix_spawnattr_setspecialport_np(&mut attr, broker, k::TASK_BOOTSTRAP_PORT);
        if r != 0 {
            return Err(format!("setspecialport_np(bootstrap) = {r}"));
        }
        for &(w, p) in special {
            let r = ffi::posix_spawnattr_setspecialport_np(&mut attr, p, w);
            if r != 0 {
                return Err(format!("setspecialport_np({w}) = {r}"));
            }
        }
        extern "C" {
            static environ: *const *mut libc::c_char;
        }
        let r = libc::posix_spawn(
            &mut pid,
            exe.as_ptr(),
            std::ptr::null(),
            &attr,
            argv.as_ptr(),
            if empty_env { std::ptr::null() } else { environ },
        );
        libc::posix_spawnattr_destroy(&mut attr);
        if r != 0 {
            return Err(format!("posix_spawn = {r}"));
        }
    }
    let mut suspended_token = None;
    if suspended {
        let mut tn = 0u32;
        let kr = unsafe { ffi::task_name_for_pid(me(), pid, &mut tn) };
        suspended_token = audit_token_of_task(tn).ok();
        if kr != 0 {
            eprintln!("task_name_for_pid(child) = {}", hex(kr));
        }
        unsafe { libc::kill(pid, libc::SIGCONT) };
    }
    let mut pre = vec![];
    loop {
        match recv(broker, Some(10000)) {
            Ok(m) if m.id == 1000 => {
                let port = m.ports.first().map(|p| p.0).unwrap_or(0);
                let mut c = Child {
                    pid,
                    port,
                    broker,
                    hello: m,
                    pre,
                    suspended_token,
                    label: if extra.contains(&"--sandbox") {
                        "sandboxed"
                    } else {
                        "plain"
                    },
                    sandbox_result: String::new(),
                    post: vec![],
                };
                if extra.contains(&"--sandbox") {
                    // service (refuse) broker traffic until the child reports its sandbox result
                    loop {
                        match recv(broker, Some(10000)) {
                            Ok(m) if m.id == 1001 => {
                                c.sandbox_result = m.text();
                                break;
                            }
                            Ok(m) => {
                                c.post.push(m.clone());
                                release(&m);
                            }
                            Err(e) => {
                                c.sandbox_result = format!("no sandbox report: {}", hex(e));
                                break;
                            }
                        }
                    }
                }
                return Ok(c);
            }
            Ok(m) => {
                pre.push(m.clone());
                release(&m);
            }
            Err(e) => {
                return Err(format!(
                    "no hello: {} (pre-hello msgs {})",
                    hex(e),
                    pre.len()
                ));
            }
        }
    }
}

/// Send a command and wait for the reply, servicing (refusing) broker traffic meanwhile.
pub fn cmd(
    c: &mut Child,
    id: i32,
    ports: Vec<(u32, u32)>,
    text: &str,
    ools: Vec<(u64, u32, bool, u8)>,
) -> Result<Rcvd, String> {
    let rp = alloc_recv();
    let set = alloc_set();
    unsafe {
        ffi::mach_port_insert_member(me(), rp, set);
        ffi::mach_port_insert_member(me(), c.broker, set)
    };
    let mut o = Out::to(c.port, k::COPY_SEND, id).text(text);
    o.reply = rp;
    o.reply_disp = k::MAKE_SEND_ONCE;
    o.ports = ports;
    o.ools = ools;
    let kr = o.send();
    if kr != 0 {
        cleanup(c, rp, set);
        return Err(format!("cmd {id} send {}", hex(kr)));
    }
    let res = loop {
        match recv(set, Some(15000)) {
            Ok(m) if m.local == rp => break Ok(m),
            Ok(m) => {
                c.pre.push(m.clone());
                release(&m);
            }
            Err(e) => break Err(format!("cmd {id} no reply {}", hex(e))),
        }
    };
    cleanup(c, rp, set);
    res
}
fn cleanup(c: &Child, rp: u32, set: u32) {
    unsafe {
        ffi::mach_port_extract_member(me(), c.broker, set);
        ffi::mach_port_mod_refs(me(), rp, k::RIGHT_RECEIVE, -1);
        ffi::mach_port_mod_refs(me(), set, k::RIGHT_PORT_SET, -1);
    }
}
fn txt(r: &Result<Rcvd, String>) -> String {
    match r {
        Ok(m) => m.text(),
        Err(e) => e.clone(),
    }
}
fn wait(pid: i32) -> i32 {
    let mut st = 0;
    for _ in 0..200 {
        if unsafe { libc::waitpid(pid, &mut st, libc::WNOHANG) } == pid {
            return st;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    unsafe {
        libc::kill(pid, libc::SIGKILL);
        libc::waitpid(pid, &mut st, 0)
    };
    -1
}
fn until_hello(broker: u32, ms: u32) -> (Option<Rcvd>, Vec<Rcvd>) {
    let mut pre = vec![];
    loop {
        match recv(broker, Some(ms)) {
            Ok(m) if m.id == 1000 => return (Some(m), pre),
            Ok(m) => {
                pre.push(m.clone());
                release(&m);
            }
            Err(_) => return (None, pre),
        }
    }
}

fn describe_pre(v: &[Rcvd]) -> String {
    v.iter()
        .map(|m| {
            format!(
                "{{id={} size={} bits={:#x} remote_disp={} ports={} from_pid={} words={:x?}}}",
                m.id,
                m.size,
                m.bits,
                m.remote_disp(),
                m.ports.len(),
                m.pid(),
                &m.raw_words[..m.raw_words.len().min(8)]
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn cross(sandbox: bool) {
    let extra: Vec<&str> = if sandbox { vec!["--sandbox"] } else { vec![] };
    let p = if sandbox { "[x-sbx]" } else { "[x]" };
    let t = me();
    // ---- P27 + P3: suspended spawn with bootstrap replaced
    let mut c = match spawn("main", &extra, true, &[]) {
        Ok(c) => c,
        Err(e) => {
            out!(p, "SPAWN FAILED {e}");
            return;
        }
    };
    out!(
        p,
        "P27 child pid {} hello: {:?} | hello trailer pid={} pidver={:#x}",
        c.pid,
        c.hello.text(),
        c.hello.pid(),
        c.hello.pidver()
    );
    out!(
        p,
        "P27 messages on the broker port before hello: {} {}",
        c.pre.len(),
        describe_pre(&c.pre)
    );
    if sandbox {
        out!(
            p,
            "P26 child sandbox_init_with_parameters(empty-process SBPL): {} | broker traffic between hello and the sandbox report: {} {}",
            c.sandbox_result,
            c.post.len(),
            describe_pre(&c.post)
        );
    }
    out!(
        p,
        "P3a suspended-spawn token (task_name_for_pid+task_info 15) = {:?} | first-message trailer token = {:?} | equal={}",
        c.suspended_token,
        c.hello.audit(),
        c.suspended_token == c.hello.audit()
    );
    let r = cmd(&mut c, 31, vec![], "", vec![]);
    out!(
        p,
        "P27 child libSystem calls with replaced bootstrap: {} | broker traffic so far: {} {}",
        txt(&r),
        c.pre.len(),
        describe_pre(&c.pre)
    );
    match spawn_env("main", &extra, false, &[], true) {
        Ok(mut ce) => {
            out!(
                p,
                "P27 same child spawned with an EMPTY environment: {} pre-hello messages {}",
                ce.pre.len(),
                describe_pre(&ce.pre)
            );
            let r = cmd(&mut ce, 31, vec![], "", vec![]);
            out!(
                p,
                "P27 (empty env) libSystem calls: {} | broker traffic {}",
                txt(&r),
                ce.pre.len()
            );
            let _ = cmd(&mut ce, 19, vec![], "", vec![]);
            wait(ce.pid);
        }
        Err(e) => out!(p, "P27 empty-env spawn failed: {e}"),
    }
    // ---- P6: parent holds only a send right to the child's port
    out!(
        p,
        "P6 send right to child-held port {:#x}: type={:?} get_attributes(status)={:?} {}",
        c.port,
        ptype(c.port).map(|x| format!("{x:#x}")),
        status(c.port).err().map(hex),
        kobject(c.port)
    );
    // ---- P10 / P37: move a receive right with state into the child
    let nport = alloc_recv();
    let x = alloc_recv_send();
    set_qlimit(x, 77);
    for i in 0..2 {
        Out::to(x, k::COPY_SEND, 100 + i).send();
    }
    let _ = drain(x);
    Out::to(x, k::MAKE_SEND, 200).send(); // mscount bump
    Out::to(x, k::COPY_SEND, 201).send();
    Out::to(x, k::COPY_SEND, 202).send(); // two queued
    let (kns, _) = request(x, k::NOTIFY_NO_SENDERS, 100, nport, k::MAKE_SEND_ONCE);
    let st_before = status(x);
    let r = cmd(&mut c, 10, vec![(x, k::MOVE_RECEIVE)], "", vec![]);
    let notif = drain(nport);
    out!(
        p,
        "P10 before move status={st_before:?} (no-senders armed: {}) | child after MOVE_RECEIVE: {}",
        hex(kns),
        txt(&r)
    );
    out!(
        p,
        "P37 notifications to the old requester after the move: {:?} | parent's name now: type={:?}",
        notif.iter().map(|m| (m.id, m.pid())).collect::<Vec<_>>(),
        ptype(x).map(|v| format!("{v:#x}"))
    );
    let r = cmd(&mut c, 11, vec![], "0", vec![]);
    out!(p, "P37 child drains the moved port: {}", txt(&r));
    let x2 = alloc_recv();
    let (mut x2s, mut x2t) = (0u32, 0u32);
    unsafe { ffi::mach_port_extract_right(t, x2, k::MAKE_SEND, &mut x2s, &mut x2t) };
    let n37 = alloc_recv();
    let (k37, _) = request(x2, k::NOTIFY_NO_SENDERS, 1, n37, k::MAKE_SEND_ONCE);
    let r = cmd(&mut c, 10, vec![(x2, k::MOVE_RECEIVE)], "", vec![]);
    let at_move = drain(n37);
    unsafe { ffi::mach_port_deallocate(t, x2) };
    let after_drop = recv(n37, Some(500));
    out!(
        p,
        "P37b moved receive right with no-senders armed ({}): child sees {} | at move we got {:?} | after we drop our only send right we get {:?} from pid {:?}",
        hex(k37),
        txt(&r),
        at_move.iter().map(|m| m.id).collect::<Vec<_>>(),
        after_drop.as_ref().map(|m| m.id).map_err(|e| hex(*e)),
        after_drop.as_ref().map(|m| m.pid())
    );
    // ---- P6 challenge + P36 send-once semantics on x (child holds receive, we hold send)
    let rep = alloc_recv();
    let mut o = Out::to(x, k::COPY_SEND, 600);
    o.reply = rep;
    o.reply_disp = k::MAKE_SEND_ONCE;
    o.send();
    let r = cmd(&mut c, 14, vec![], "", vec![]);
    let pong = recv(rep, Some(2000));
    out!(
        p,
        "P6 challenge: child says {} | reply trailer pid={:?} (child pid {}) id={:?}",
        txt(&r),
        pong.as_ref().map(|m| m.pid()),
        c.pid,
        pong.as_ref().map(|m| m.id)
    );
    let mut o = Out::to(x, k::COPY_SEND, 601);
    o.reply = rep;
    o.reply_disp = k::MAKE_SEND_ONCE;
    o.send();
    let r = cmd(&mut c, 15, vec![], "", vec![]);
    let son = recv(rep, Some(2000));
    out!(
        p,
        "P36 child destroys the send-once reply right: {} | we get id={:?} size={:?} token pid={:?}",
        txt(&r),
        son.as_ref().map(|m| m.id),
        son.as_ref().map(|m| m.size),
        son.as_ref().map(|m| m.pid())
    );
    let q = alloc_recv_send();
    let mut o = Out::to(x, k::COPY_SEND, 602);
    o.reply = rep;
    o.reply_disp = k::MAKE_SEND_ONCE;
    o.send();
    let r = cmd(&mut c, 16, vec![(q, k::MAKE_SEND)], "", vec![]);
    let fwd = recv(q, Some(2000));
    let fr = fwd.as_ref().ok().and_then(|m| m.ports.first().copied());
    let kfwd = fr
        .map(|(n, _)| Out::to(n, k::MOVE_SEND_ONCE, 603).send())
        .unwrap_or(-1);
    let viaf = recv(rep, Some(2000));
    out!(
        p,
        "P36 child forwards the send-once right to us: {} | forwarded msg from pid={:?} carries {:?}; we reply through it ({}) -> reply trailer pid={:?} (ours {})",
        txt(&r),
        fwd.as_ref().map(|m| m.pid()),
        fr,
        hex(kfwd),
        viaf.as_ref().map(|m| m.pid()),
        std::process::id()
    );
    // ---- P21: receiver destroys its receive right while messages with rights + OOL are queued
    let y_ports: Vec<u32> = (0..3).map(|_| alloc_recv()).collect();
    let n21 = alloc_recv();
    let (kdn, _) = request(x, k::NOTIFY_DEAD_NAME, 0, n21, k::MAKE_SEND_ONCE);
    for &yp in &y_ports {
        request(yp, k::NOTIFY_NO_SENDERS, 1, n21, k::MAKE_SEND_ONCE);
        let mut addr = 0u64;
        unsafe { ffi::mach_vm_allocate(t, &mut addr, 16384, 1) };
        let mut o = Out::to(x, k::COPY_SEND, 210);
        o.ports = vec![(yp, k::MAKE_SEND)];
        o.ools = vec![(addr, 16384, true, 1)];
        o.send();
    }
    let r = cmd(&mut c, 12, vec![], "0", vec![]);
    std::thread::sleep(std::time::Duration::from_millis(100));
    let notes = drain(n21);
    let ksend = Out::to(x, k::COPY_SEND, 211).send_nb();
    out!(
        p,
        "P21 dead-name request on our send right={} | child destroys receive: {} | notifications: {:?} | our name now type={:?} | next send kr={}",
        hex(kdn),
        txt(&r),
        notes
            .iter()
            .map(|m| (m.id, m.raw_words.get(8).copied()))
            .collect::<Vec<_>>(),
        ptype(x).map(|v| format!("{v:#x}")),
        hex(ksend)
    );
    // ---- P8/P9: a separate child per port kind sends it to us (a guard kill ends only that child)
    for label in [
        "task_name",
        "thread_self",
        "host",
        "task_self",
        "task_self_MOVE",
    ] {
        let mut cx = match spawn("main", &extra, false, &[]) {
            Ok(c) => c,
            Err(e) => {
                out!(p, "P9 spawn failed {e}");
                continue;
            }
        };
        let q9 = alloc_recv_send();
        let r = cmd(&mut cx, 21, vec![(q9, k::MAKE_SEND)], label, vec![]);
        let got = drain(q9);
        let mut lines = vec![];
        for m in &got {
            for &(n, d) in &m.ports {
                lines.push(format!("P9   received {} disp={d} name={n:#x} type={:?} {} (classified while sender alive)", m.text(), ptype(n).map(|v| format!("{v:#x}")).map_err(hex), kobject(n)));
            }
        }
        let st = if r.is_err() {
            wait(cx.pid)
        } else {
            let _ = cmd(&mut cx, 19, vec![], "", vec![]);
            wait(cx.pid)
        };
        out!(
            p,
            "P9 {label}: child says {} | child exit status {st:#x} | we received {} message(s)",
            txt(&r),
            got.len()
        );
        for l in lines {
            out!(p, "{l}");
        }
        for m in &got {
            for &(n, _) in &m.ports {
                out!(
                    p,
                    "P9   after the sender exited the same name is type={:?}",
                    ptype(n).map(|v| format!("{v:#x}")).map_err(hex)
                );
            }
            release(m);
        }
    }
    // ---- P8 in the child: classify a memory entry, a task port and a plain port it receives
    let mut buf = 0u64;
    unsafe {
        ffi::mach_vm_allocate(t, &mut buf, 16384, 1);
        *(buf as *mut u8) = 0x33
    };
    let mut msz = 16384u64;
    let mut ro = 0u32;
    let kro = unsafe { ffi::mach_make_memory_entry_64(t, &mut msz, buf, 1, &mut ro, 0) };
    let plain = alloc_recv_send();
    let entitled = std::env::var_os("D4_ENTITLED").is_some();
    let r = if entitled {
        cmd(
            &mut c,
            13,
            vec![
                (ro, k::COPY_SEND),
                (plain, k::MAKE_SEND),
                (unsafe { libc::mach_host_self() }, k::COPY_SEND),
            ],
            "",
            vec![],
        )
    } else {
        cmd(
            &mut c,
            13,
            vec![
                (ro, k::COPY_SEND),
                (t, k::COPY_SEND),
                (plain, k::MAKE_SEND),
                (unsafe { libc::mach_host_self() }, k::COPY_SEND),
            ],
            "",
            vec![],
        )
    };
    out!(
        p,
        "P8 child classifies [mem-entry(R, make {}), {}plain port, host]: {}",
        hex(kro),
        if entitled { "" } else { "parent task port, " },
        txt(&r)
    );
    // ---- P33 in the child
    let bufc = buf;
    let w = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(60));
        unsafe { *(bufc as *mut u8) = 0x44 }
    });
    let r = cmd(&mut c, 17, vec![(ro, k::COPY_SEND)], "", vec![]);
    w.join().unwrap();
    out!(
        p,
        "P33 child maps our read-only entry (we write 0x33->0x44 at ~60 ms): {}",
        txt(&r)
    );
    // ---- P15 OOL time-of-check/time-of-use
    for (copy, dealloc) in [(0u8, false), (1, false)] {
        let mut a = 0u64;
        unsafe {
            ffi::mach_vm_allocate(t, &mut a, 65536, 1);
            std::ptr::write_bytes(a as *mut u8, 0x55, 65536)
        };
        let aa = a;
        let w = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            unsafe { *(aa as *mut u8) = 0x66 }
        });
        let r = cmd(&mut c, 30, vec![], "", vec![(a, 65536, dealloc, copy)]);
        w.join().unwrap();
        out!(
            p,
            "P15 OOL copy={copy} dealloc={dealloc}, sender writes 0x55->0x66 20 ms after send: {}",
            txt(&r)
        );
    }
    // ---- P20b ordering of no-senders vs queued data, cross-process
    let trials = if sandbox { 300 } else { 1500 };
    let nper = 8;
    let (mut early, mut ok, mut missing) = (0, 0, 0);
    let set = alloc_set();
    let n20 = alloc_recv();
    unsafe { ffi::mach_port_insert_member(t, n20, set) };
    for _ in 0..trials {
        let pp = alloc_recv();
        unsafe { ffi::mach_port_insert_member(t, pp, set) };
        let (mut s, mut st) = (0u32, 0u32);
        unsafe { ffi::mach_port_extract_right(t, pp, k::MAKE_SEND, &mut s, &mut st) };
        request(pp, k::NOTIFY_NO_SENDERS, 1, n20, k::MAKE_SEND_ONCE);
        let mut o = Out::to(c.port, k::COPY_SEND, 20).text(&nper.to_string());
        o.ports = vec![(pp, k::MOVE_SEND)];
        o.send();
        let mut data = 0;
        let mut got_ns = false;
        let mut bad = false;
        loop {
            match recv(set, Some(3000)) {
                Ok(m) if m.local == n20 => {
                    if data < nper {
                        bad = true;
                    }
                    got_ns = true;
                }
                Ok(m) if m.local == pp => {
                    data += 1;
                }
                Ok(m) => {
                    release(&m);
                    continue;
                }
                Err(_) => break,
            }
            if got_ns && data == nper {
                break;
            }
            if got_ns && bad {
                let rest = drain(pp).len();
                data += rest;
                break;
            }
        }
        // the child's "burst" reply goes to a dead reply port (none given); nothing to drain
        if bad {
            early += 1
        } else if got_ns && data == nper {
            ok += 1
        } else {
            missing += 1
        }
        unsafe {
            ffi::mach_port_extract_member(t, pp, set);
            ffi::mach_port_mod_refs(t, pp, k::RIGHT_RECEIVE, -1)
        };
    }
    out!(
        p,
        "P20b {trials} trials x {nper} data msgs then sender drops its only send right; notify port and data port in one set: no-senders seen before all data={early} in order={ok} incomplete={missing}"
    );
    // ---- P35: sender exits before we dequeue
    let q35 = alloc_recv_send();
    let mut o = Out::to(c.port, k::COPY_SEND, 23);
    o.ports = vec![(q35, k::MAKE_SEND)];
    o.send();
    let st = wait(c.pid);
    std::thread::sleep(std::time::Duration::from_millis(50));
    let m = recv(q35, Some(1000));
    out!(
        p,
        "P35 child exited (status {st:#x}) before dequeue; message trailer pid={:?} pidver={:?} (hello had pid {} pidver {:#x})",
        m.as_ref().map(|m| m.pid()),
        m.as_ref().map(|m| format!("{:#x}", m.pidver())),
        c.hello.pid(),
        c.hello.pidver()
    );
    // ---- P3a exec: new child, exec, compare
    let mut c2 = match spawn("main", &extra, false, &[]) {
        Ok(c) => c,
        Err(e) => {
            out!(p, "SPAWN2 FAILED {e}");
            return;
        }
    };
    let before = (c2.hello.pid(), c2.hello.pidver());
    let r = cmd(&mut c2, 24, vec![], "", vec![]);
    let (after, pre2) = until_hello(c2.broker, 5000);
    out!(
        p,
        "P3a exec: before (pid,pidver)=({}, {:#x}) cmd={} | exec'd image sent {} messages before its hello: {} | after-exec hello: {:?} (pid,pidver)={:?}",
        before.0,
        before.1,
        txt(&r),
        pre2.len(),
        describe_pre(&pre2),
        after.as_ref().map(|m| m.text()),
        after
            .as_ref()
            .map(|m| (m.pid(), format!("{:#x}", m.pidver())))
    );
    if let Some(m) = after {
        if let Some(&(np, _)) = m.ports.first() {
            c2.port = np;
            let _ = cmd(&mut c2, 19, vec![], "", vec![]);
        }
    }
    wait(c2.pid);
    // ---- sandboxed in-process suite in a fresh child (only meaningful with --sandbox; also run plain for parity)
    let mut c3 = match spawn("main", &extra, false, &[]) {
        Ok(c) => c,
        Err(e) => {
            out!(p, "SPAWN3 FAILED {e}");
            return;
        }
    };
    let r = cmd(&mut c3, 18, vec![], "", vec![]);
    out!(p, "inproc in child ({}) -> {}", c3.label, txt(&r));
    let _ = cmd(&mut c3, 19, vec![], "", vec![]);
    wait(c3.pid);
}

/// P3b: pidversion across many spawns.
pub fn p3b(n: usize) {
    let mut v = vec![];
    for _ in 0..n {
        match spawn("main", &[], false, &[]) {
            Ok(mut c) => {
                v.push((c.pid, c.hello.pidver()));
                let _ = cmd(&mut c, 19, vec![], "", vec![]);
                wait(c.pid);
            }
            Err(e) => println!("P3b spawn err {e}"),
        }
    }
    let mut pv: Vec<u32> = v.iter().map(|x| x.1).collect();
    let increasing = pv.windows(2).all(|w| w[1] > w[0]);
    let diffs: Vec<i64> = pv.windows(2).map(|w| w[1] as i64 - w[0] as i64).collect();
    pv.sort();
    pv.dedup();
    println!(
        "P3b {n} children: (pid,pidver) first/last {:?} {:?} | pidversions distinct={} strictly increasing in spawn order={increasing} | step min={:?} max={:?} | own pidver={:#x}",
        v.first(),
        v.last(),
        pv.len() == v.len(),
        diffs.iter().min(),
        diffs.iter().max(),
        audit_token_of_task(me())
            .map(|a| a[k::AT_PIDVERSION])
            .unwrap_or(0)
    );
}

/// P14: round-trip cost, inline vs OOL.
pub fn p14() {
    let mut c = spawn("main", &[], false, &[]).expect("spawn");
    let r = cmd(&mut c, 22, vec![], "", vec![]);
    println!("P14 echo: {}", txt(&r));
    let rp = alloc_recv();
    let iters = 3000;
    let mut src = 0u64;
    unsafe {
        ffi::mach_vm_allocate(me(), &mut src, 65536, 1);
        std::ptr::write_bytes(src as *mut u8, 1, 65536)
    };
    for (label, size, ool) in [
        ("inline", 1024usize, None),
        ("inline", 16384, None),
        ("inline", 65536, None),
        ("ool-copy0", 65536, Some(0u8)),
        ("ool-copy1", 65536, Some(1u8)),
        ("ool-copy1", 16384, Some(1u8)),
        ("empty", 0, None),
    ] {
        let mut lat = Vec::with_capacity(iters);
        let mut o = Out::to(c.port, k::COPY_SEND, 1);
        o.reply = rp;
        o.reply_disp = k::MAKE_SEND_ONCE;
        if let Some(cp) = ool {
            o.ools = vec![(src, size as u32, false, cp)];
        } else {
            o.body = vec![7u8; size];
        }
        let (proto, ssz) = o.encode();
        for i in 0..iters + 200 {
            let mut b = proto.clone();
            let t0 = now_ns();
            let kr = unsafe {
                ffi::mach_msg_overwrite(
                    b.as_mut_ptr().cast(),
                    k::SEND_MSG,
                    ssz,
                    0,
                    0,
                    0,
                    0,
                    std::ptr::null_mut(),
                    0,
                )
            };
            assert_eq!(kr, 0);
            let m = recv_opts(rp, Some(5000), 70000, 0).expect("echo");
            let dt = now_ns() - t0;
            let _ = m;
            if i >= 200 {
                lat.push(dt);
            }
        }
        lat.sort();
        println!(
            "P14 {label:10} {size:6} B: round trip ns p50={} p90={} p99={} min={}",
            lat[iters / 2],
            lat[iters * 9 / 10],
            lat[iters * 99 / 100],
            lat[0]
        );
    }
    let mut o = Out::to(c.port, k::COPY_SEND, 2);
    o.reply = rp;
    o.reply_disp = k::MAKE_SEND_ONCE;
    o.send();
    let _ = recv(rp, Some(2000));
    let _ = cmd(&mut c, 19, vec![], "", vec![]);
    wait(c.pid);
}

/// P23: idle wakeups of a process whose IO thread blocks on a 1 000-port set.
pub fn p23(secs: u64) {
    let set = alloc_set();
    for _ in 0..1000 {
        let x = alloc_recv();
        unsafe { ffi::mach_port_insert_member(me(), x, set) };
    }
    let wake = alloc_recv_send();
    set_qlimit(wake, 1);
    unsafe { ffi::mach_port_insert_member(me(), wake, set) };
    let io = std::thread::spawn(move || {
        let mut v = vec![];
        loop {
            match recv(set, None) {
                Ok(m) => {
                    v.push(m.id);
                    if m.id == 99 {
                        break;
                    }
                }
                Err(e) => {
                    v.push(-e);
                    break;
                }
            }
        }
        v
    });
    // add a member while the IO thread is blocked, then send to it
    std::thread::sleep(std::time::Duration::from_millis(200));
    let late = alloc_recv_send();
    let ki = unsafe { ffi::mach_port_insert_member(me(), late, set) };
    Out::to(late, k::COPY_SEND, 7).send();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let ke = unsafe { ffi::mach_port_extract_member(me(), late, set) };
    Out::to(late, k::COPY_SEND, 8).send();
    std::thread::sleep(std::time::Duration::from_millis(200));
    println!(
        "P23 pid {} blocked; inserted member while blocked ({}), sent id 7; extracted ({}), sent id 8 (should not reach the set); measuring idle {secs}s",
        std::process::id(),
        hex(ki),
        hex(ke)
    );
    use std::io::Write;
    std::io::stdout().flush().ok();
    std::thread::sleep(std::time::Duration::from_secs(secs));
    Out::to(wake, k::COPY_SEND, 99).send();
    let v = io.join().unwrap();
    println!(
        "P23 IO thread received ids {v:?}; id 8 still queued on extracted port: {}",
        recv(late, Some(0)).map(|m| m.id == 8).unwrap_or(false)
    );
}

/// P28: which special-port indices posix_spawnattr_setspecialport_np accepts and that reach the child; mach_ports_register inheritance.
pub fn p28() {
    let reg = alloc_recv_send();
    let mut arr = [reg];
    let krg = unsafe { ffi::mach_ports_register(me(), arr.as_mut_ptr(), 1) };
    let mut c = spawn("main", &[], false, &[]).expect("spawn");
    let r = cmd(&mut c, 28, vec![], "", vec![]);
    let got = recv(reg, Some(500));
    println!(
        "P28 parent mach_ports_register={} ; child (posix_spawn) mach_ports_lookup: {} | child's send to slot 0 reached our registered port: {}",
        hex(krg),
        txt(&r),
        got.map(|m| m.id == 2801 && m.pid() == c.pid)
            .unwrap_or(false)
    );
    let _ = cmd(&mut c, 19, vec![], "", vec![]);
    wait(c.pid);
    for w in [1i32, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12] {
        let x = alloc_recv_send();
        match spawn("specialtest", &[&w.to_string()], false, &[(w, x)]) {
            Ok(c) => {
                let rep = recv(c.broker, Some(3000));
                let got = recv(x, Some(500));
                println!(
                    "P28 setspecialport_np(which={w}): child says {:?} | message reached our port: {}",
                    rep.map(|m| m.text()),
                    got.map(|m| m.id == 2800).unwrap_or(false)
                );
                wait(c.pid);
            }
            Err(e) => println!("P28 setspecialport_np(which={w}): {e}"),
        }
    }
}

/// P29: dynamic registration from an unprivileged process.
pub fn p29() {
    let mut sp = 0u32;
    let kr = unsafe {
        ffi::bootstrap_check_in(
            ffi::bootstrap_port,
            c"org.izcal.d4probe.unlisted".as_ptr(),
            &mut sp,
        )
    };
    println!(
        "P29 bootstrap_check_in(launchd bootstrap, unlisted)={} ({kr})",
        hex(kr)
    );
}

/// P31: hostile messages to a receiver (run with and without entitlements); dispositions and nulls.
/// P31 cross-process: a (possibly un-entitled) child sends hostile-shaped messages; we receive each and classify.
pub fn p31x() {
    let mut c = spawn("main", &[], false, &[]).expect("spawn");
    let q = alloc_recv_send();
    set_qlimit(q, 64);
    let r = cmd(&mut c, 25, vec![(q, k::MAKE_SEND)], "", vec![]);
    println!(
        "P31x child (exe {:?}) sent: {}",
        std::env::var("D4_CHILD_EXE").ok(),
        txt(&r)
    );
    use std::io::Write;
    loop {
        let m = match recv(q, Some(300)) {
            Ok(m) => m,
            Err(e) => {
                println!("P31x done ({})", hex(e));
                break;
            }
        };
        let kinds: Vec<String> = m
            .ports
            .iter()
            .map(|&(n, d)| {
                format!(
                    "{n:#x}/{d}/{:?}/{}",
                    ptype(n).map(|v| format!("{v:#x}")).map_err(hex),
                    if n != 0 && n != u32::MAX {
                        kobject(n)
                    } else {
                        String::new()
                    }
                )
            })
            .collect();
        println!(
            "P31x received id={} ports={kinds:?} ools={:x?} — receiver alive",
            m.id, m.ools
        );
        std::io::stdout().flush().ok();
        release(&m);
    }
    let _ = cmd(&mut c, 19, vec![], "", vec![]);
    wait(c.pid);
}

pub fn p31() {
    let r = alloc_recv_send();
    let so = alloc_recv();
    let (mut s1, mut s1t) = (0u32, 0u32);
    unsafe { ffi::mach_port_extract_right(me(), so, k::MAKE_SEND_ONCE, &mut s1, &mut s1t) };
    let mut res = vec![];
    for (label, ports) in [
        ("null", vec![(0u32, k::COPY_SEND)]),
        ("dead", vec![(u32::MAX, k::COPY_SEND)]),
        ("send-once", vec![(s1, k::MOVE_SEND_ONCE)]),
        ("task-self", vec![(me(), k::COPY_SEND)]),
    ] {
        let mut o = Out::to(r, k::COPY_SEND, 31);
        o.ports = ports;
        let ks = o.send();
        let m = recv(r, Some(100));
        res.push(format!(
            "{label}: send={} rcv={:?}",
            hex(ks),
            m.as_ref().map(|m| m.ports.clone()).map_err(|e| hex(*e))
        ));
        if let Ok(m) = m {
            release(&m);
        }
    }
    println!("P31 {}", res.join(" | "));
}

pub fn dbg() {
    let mut c = spawn("main", &["--sandbox"], false, &[]).expect("spawn");
    println!(
        "dbg pre {} | sandbox: {} | post-hello traffic {}",
        describe_pre(&c.pre),
        c.sandbox_result,
        describe_pre(&c.post)
    );
    for id in [13, 31, 19] {
        let r = cmd(
            &mut c,
            id,
            if id == 13 {
                vec![(alloc_recv_send(), k::MAKE_SEND)]
            } else {
                vec![]
            },
            "",
            vec![],
        );
        println!("dbg cmd {id}: {}", txt(&r));
    }
    wait(c.pid);
}
