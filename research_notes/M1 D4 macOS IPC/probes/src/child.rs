//! Child side: reached only through its bootstrap special port (no lookups).
use crate::ffi::{self, k, me};
use crate::mach::*;
use std::ffi::CString;

pub const SBPL: &str = "(version 1)\n(deny default)\n(allow sysctl-read)\n"; // = sandbox/macos.rs EMPTY_PROCESS_PROFILE

pub fn apply_sandbox(profile: &str) -> Result<(), String> {
    let p = CString::new(profile).unwrap();
    let mut err: *mut ffi::c_char = std::ptr::null_mut();
    let rc =
        unsafe { ffi::sandbox_init_with_parameters(p.as_ptr(), 0, std::ptr::null(), &mut err) };
    if rc == 0 {
        return Ok(());
    }
    let msg = if err.is_null() {
        format!("rc={rc}")
    } else {
        let s = unsafe { std::ffi::CStr::from_ptr(err) }
            .to_string_lossy()
            .into_owned();
        unsafe { ffi::sandbox_free_error(err) };
        s
    };
    Err(msg)
}

fn reply(to: u32, text: &str, ports: Vec<(u32, u32)>) -> i32 {
    let mut o = Out::to(to, k::MOVE_SEND_ONCE, 3000).text(text);
    o.ports = ports;
    let kr = o.send();
    if to != 0 && (kr != 0 || std::env::var_os("D4DEBUG").is_some()) {
        eprintln!(
            "child[{}] reply to {to:#x} type={:?} kr={}",
            std::process::id(),
            ptype(to).map(|t| format!("{t:#x}")),
            hex(kr)
        );
    }
    kr
}

pub fn child_main(args: &[String]) {
    let role = args.get(2).cloned().unwrap_or_default();
    let sandbox = args.iter().any(|a| a == "--sandbox");
    let mut bp = 0u32;
    let kbp = unsafe { ffi::task_get_special_port(me(), k::TASK_BOOTSTRAP_PORT, &mut bp) };
    let global = unsafe { ffi::bootstrap_port };
    let c = alloc_recv_send();
    let mut hello = Out::to(bp, k::COPY_SEND, 1000).text(&format!(
        "role={role} pid={} task_get_special_port(4)={}:{bp:#x} bootstrap_port_global={global:#x} type={:?}",
        std::process::id(), hex(kbp), ptype(bp)));
    hello.ports = vec![(c, k::MAKE_SEND)];
    let kh = hello.send();
    if kh != 0 {
        eprintln!("child: hello send failed {}", hex(kh));
        std::process::exit(3);
    }
    if role == "specialtest" {
        let w: i32 = args[3].parse().unwrap();
        let mut sp = 0u32;
        let kr = unsafe { ffi::task_get_special_port(me(), w, &mut sp) };
        let r = if kr == 0 && sp != 0 {
            Out::to(sp, k::COPY_SEND, 2800).text("special").send_nb()
        } else {
            -1
        };
        let _ = Out::to(bp, k::COPY_SEND, 1002)
            .text(&format!(
                "special {w}: get={} port={sp:#x} send={}",
                hex(kr),
                hex(r)
            ))
            .send();
        std::process::exit(0);
    }
    if sandbox {
        let r = apply_sandbox(SBPL);
        let ks = Out::to(bp, k::COPY_SEND, 1001)
            .text(&format!("sandbox_init_with_parameters: {r:?}"))
            .send();
        if std::env::var_os("D4DEBUG").is_some() {
            eprintln!(
                "child sandbox {r:?}; post-sandbox send to bootstrap kr={}",
                hex(ks)
            );
        }
    }
    let mut held: Vec<u32> = vec![];
    loop {
        let m = match recv(c, None) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("child recv err {}", hex(e));
                std::process::exit(4)
            }
        };
        let rp = m.remote;
        if std::env::var_os("D4DEBUG").is_some() {
            eprintln!(
                "child[{}] cmd {} remote={rp:#x} bits={:#x} ports={:?}",
                std::process::id(),
                m.id,
                m.bits,
                m.ports
            );
        }
        match m.id {
            10 => {
                // take receive right(s)
                let mut s = String::new();
                for &(n, d) in &m.ports {
                    held.push(n);
                    s += &format!(
                        "got name={n:#x} disp={d} type={:?} status[pset,seqno,mscount,qlimit,msgcount,sorights,srights,pdreq,nsreq,flags]={:?}; ",
                        ptype(n).map(|t| format!("{t:#x}")),
                        status(n)
                    );
                }
                reply(rp, &s, vec![]);
            }
            11 => {
                // drain held[i]
                let i: usize = m.text().parse().unwrap_or(0);
                let v = drain(held[i]);
                let s: Vec<String> = v
                    .iter()
                    .map(|x| format!("id={} pid={} seq={}", x.id, x.pid(), x.seqno()))
                    .collect();
                for x in &v {
                    release(x);
                }
                reply(
                    rp,
                    &format!("drained {} [{}]", v.len(), s.join(", ")),
                    vec![],
                );
            }
            12 => {
                // destroy held[i]
                let i: usize = m.text().parse().unwrap_or(0);
                let kr = unsafe { ffi::mach_port_mod_refs(me(), held[i], k::RIGHT_RECEIVE, -1) };
                reply(rp, &format!("destroy kr={}", hex(kr)), vec![]);
            }
            13 => {
                // classify the ports in this message
                let mut s = String::new();
                for &(n, d) in &m.ports {
                    s += &format!(
                        "[name={n:#x} disp={d} type={:?} status={:?} {}] ",
                        ptype(n).map(|t| format!("{t:#x}")),
                        status(n),
                        kobject(n)
                    );
                }
                reply(rp, &s, vec![]);
            }
            14 | 15 | 16 => {
                // challenge on held[0]
                let r = recv(held[0], Some(3000));
                let s = match r {
                    Err(e) => format!("recv err {}", hex(e)),
                    Ok(x) => {
                        let rr = x.remote;
                        let res = match m.id {
                            14 => Out::to(rr, k::MOVE_SEND_ONCE, 4000).text("pong").send(),
                            15 => unsafe { ffi::mach_port_deallocate(me(), rr) },
                            _ => {
                                let mut o = Out::to(m.ports[0].0, k::COPY_SEND, 4001);
                                o.ports = vec![(rr, k::MOVE_SEND_ONCE)];
                                o.send()
                            }
                        };
                        format!(
                            "challenge from pid {} reply-disp {} action kr={}",
                            x.pid(),
                            x.remote_disp(),
                            hex(res)
                        )
                    }
                };
                reply(rp, &s, vec![]);
            }
            17 => {
                let s = mementry_tests(m.ports[0].0);
                reply(rp, &s, vec![]);
            }
            18 => {
                crate::inproc_suite(if sandbox {
                    "[child-sbx]"
                } else {
                    "[child-plain]"
                });
                reply(rp, "done", vec![]);
            }
            19 => {
                reply(rp, "bye", vec![]);
                std::process::exit(0);
            }
            20 => {
                // burst N to the send right, then drop it
                let n: usize = m.text().parse().unwrap_or(0);
                let p = m.ports[0].0;
                for i in 0..n {
                    let _ = Out::to(p, k::COPY_SEND, 2000 + i as i32).send();
                }
                unsafe { ffi::mach_port_deallocate(me(), p) };
                reply(rp, "burst", vec![]);
            }
            21 => {
                // send our task/thread/host ports, one message each, to ports[0]
                let q = m.ports[0].0;
                let mut tn = 0u32;
                let ktn =
                    unsafe { ffi::task_name_for_pid(me(), std::process::id() as i32, &mut tn) };
                let thr = unsafe { libc::mach_thread_self() };
                let host = unsafe { libc::mach_host_self() };
                let mut s = format!("task_name_for_pid(self)={} ", hex(ktn));
                let only = m.text();
                for (label, name, disp) in [
                    ("task_self", me(), k::COPY_SEND),
                    ("task_name", tn, k::COPY_SEND),
                    ("thread_self", thr, k::COPY_SEND),
                    ("host", host, k::COPY_SEND),
                    ("task_self_MOVE", me(), k::MOVE_SEND),
                ] {
                    if !only.is_empty() && only != label {
                        continue;
                    }
                    let mut o = Out::to(q, k::COPY_SEND, 2100).text(label);
                    o.ports = vec![(name, disp)];
                    s += &format!("{label}:{} ", hex(o.send()));
                }
                reply(rp, &s, vec![]);
            }
            22 => {
                // echo loop on c until id 2
                reply(rp, "echo-ready", vec![]);
                loop {
                    let x = match recv_opts(c, None, 256 * 1024, 0) {
                        Ok(x) => x,
                        Err(_) => break,
                    };
                    if x.id == 2 {
                        reply(x.remote, "echo-done", vec![]);
                        break;
                    }
                    for &(a, sz, _) in &x.ools {
                        unsafe {
                            std::ptr::read_volatile(a as *const u8);
                            ffi::mach_vm_deallocate(me(), a, sz as u64)
                        };
                    }
                    let mut o = Out::to(x.remote, k::MOVE_SEND_ONCE, 1);
                    let _ = &o; // bare ack: one-way payload cost only
                    o.send();
                }
            }
            25 => {
                // hostile-shaped messages to ports[0]
                let q = m.ports[0].0;
                let mut res = vec![];
                let mut o = Out::to(q, k::COPY_SEND, 2501).text("null+dead");
                o.ports = vec![(0, k::COPY_SEND), (u32::MAX, k::COPY_SEND)];
                res.push(format!("null+dead:{}", hex(o.send())));
                let so = alloc_recv();
                let mut o = Out::to(q, k::COPY_SEND, 2502).text("send-once");
                o.ports = vec![(so, k::MAKE_SEND_ONCE)];
                res.push(format!("send-once:{}", hex(o.send())));
                let mut o = Out::to(q, k::COPY_SEND, 2503).text("task_self");
                o.ports = vec![(me(), k::COPY_SEND)];
                res.push(format!("task_self:{}", hex(o.send())));
                let thr = unsafe { libc::mach_thread_self() };
                let mut o = Out::to(q, k::COPY_SEND, 2504).text("thread_self");
                o.ports = vec![(thr, k::COPY_SEND)];
                res.push(format!("thread_self:{}", hex(o.send())));
                // OOL port array, type 2
                let (a, b) = (alloc_recv_send(), alloc_recv_send());
                let arr = [a, b];
                let addr = arr.as_ptr() as u64;
                let mut w: Vec<u32> = vec![
                    k::MSGH_BITS_COMPLEX | k::msgh_bits(k::COPY_SEND, 0),
                    0,
                    q,
                    0,
                    0,
                    2505,
                    1,
                    addr as u32,
                    (addr >> 32) as u32,
                    (k::COPY_SEND << 16) | (2 << 24),
                    2,
                ];
                let sz = (w.len() * 4) as u32;
                w[1] = sz;
                let kr = unsafe {
                    ffi::mach_msg_overwrite(
                        w.as_mut_ptr().cast(),
                        k::SEND_MSG,
                        sz,
                        0,
                        0,
                        0,
                        0,
                        std::ptr::null_mut(),
                        0,
                    )
                };
                res.push(format!("ool-port-array:{}", hex(kr)));
                reply(rp, &res.join(" "), vec![]);
            }
            23 => {
                let _ = Out::to(m.ports[0].0, k::COPY_SEND, 2300)
                    .text("bye-before-exit")
                    .send();
                std::process::exit(0);
            }
            24 => {
                // exec self (POSIX_SPAWN_SETEXEC) as role afterexec
                reply(rp, "execing", vec![]);
                exec_self();
            }
            28 => {
                let (mut arr, mut cnt) = (std::ptr::null_mut::<u32>(), 0u32);
                let kr = unsafe { ffi::mach_ports_lookup(me(), &mut arr, &mut cnt) };
                let v: Vec<String> = (0..cnt as usize)
                    .map(|i| {
                        let n = unsafe { *arr.add(i) };
                        let ks = if n != 0 {
                            Out::to(n, k::COPY_SEND, 2801).send_nb()
                        } else {
                            -1
                        };
                        format!(
                            "{n:#x}:{:?}:send={}",
                            ptype(n).map(|t| format!("{t:#x}")),
                            hex(ks)
                        )
                    })
                    .collect();
                reply(
                    rp,
                    &format!("mach_ports_lookup kr={} cnt={cnt} {v:?}", hex(kr)),
                    vec![],
                );
            }
            30 => {
                // OOL check: wait, then read first byte; report protections of received region
                let (a, sz, fl) = m.ools[0];
                let first0 = unsafe { std::ptr::read_volatile(a as *const u8) };
                std::thread::sleep(std::time::Duration::from_millis(100));
                let first1 = unsafe { std::ptr::read_volatile(a as *const u8) };
                let (mut a2, mut s2, mut info, mut cnt, mut on) =
                    (a, 0u64, [0i32; 16], 16u32, 0u32);
                let kr = unsafe {
                    ffi::mach_vm_region(
                        me(),
                        &mut a2,
                        &mut s2,
                        k::VM_REGION_BASIC_INFO_64,
                        info.as_mut_ptr(),
                        &mut cnt,
                        &mut on,
                    )
                };
                let wr = unsafe { ffi::mach_vm_protect(me(), a, sz as u64, 0, 3) };
                reply(
                    rp,
                    &format!(
                        "ool flags={fl:#x} size={sz} first-at-receive={first0:#x} first-after-100ms={first1:#x} region kr={} prot={} max={} protect(RW)={}",
                        hex(kr),
                        info[0],
                        info[1],
                        hex(wr)
                    ),
                    vec![],
                );
            }
            31 => {
                // libSystem calls that may talk to the (replaced) bootstrap port
                let t0 = now_ns();
                let pw = unsafe { libc::getpwuid(libc::getuid()) };
                let t1 = now_ns();
                let mut hints: libc::addrinfo = unsafe { std::mem::zeroed() };
                hints.ai_family = libc::AF_INET;
                let mut res = std::ptr::null_mut();
                let node = CString::new("localhost").unwrap();
                let g =
                    unsafe { libc::getaddrinfo(node.as_ptr(), std::ptr::null(), &hints, &mut res) };
                let t2 = now_ns();
                reply(
                    rp,
                    &format!(
                        "getpwuid null={} took {}ms; getaddrinfo(localhost)={g} took {}ms",
                        pw.is_null(),
                        (t1 - t0) / 1_000_000,
                        (t2 - t1) / 1_000_000
                    ),
                    vec![],
                );
            }
            _ => {
                reply(rp, &format!("unknown cmd {}", m.id), vec![]);
            }
        }
    }
}

pub fn exec_self() -> ! {
    let exe = CString::new(std::env::current_exe().unwrap().to_str().unwrap()).unwrap();
    let a0 = CString::new("d4-probes").unwrap();
    let a1 = CString::new("child").unwrap();
    let a2 = CString::new("afterexec").unwrap();
    let argv = [
        a0.as_ptr() as *mut _,
        a1.as_ptr() as *mut _,
        a2.as_ptr() as *mut _,
        std::ptr::null_mut(),
    ];
    let mut attr: libc::posix_spawnattr_t = std::ptr::null_mut();
    unsafe {
        libc::posix_spawnattr_init(&mut attr);
        libc::posix_spawnattr_setflags(&mut attr, libc::POSIX_SPAWN_SETEXEC as i16);
        let mut pid = 0;
        let r = libc::posix_spawn(
            &mut pid,
            exe.as_ptr(),
            std::ptr::null(),
            &attr,
            argv.as_ptr(),
            std::ptr::null(),
        );
        eprintln!("exec failed {r}");
        std::process::exit(5)
    }
}

/// Memory-entry tests on a received entry (P33).
pub fn mementry_tests(entry: u32) -> String {
    let mut s = format!(
        "entry type={:?} {} ",
        ptype(entry).map(|t| format!("{t:#x}")),
        kobject(entry)
    );
    let size = 16384u64;
    for (label, cur, max) in [("RW/RW", 3, 3), ("R/RW", 1, 3), ("R/R", 1, 1)] {
        let mut addr = 0u64;
        let kr =
            unsafe { libc::mach_vm_map(me(), &mut addr, size, 0, 1, entry, 0, 0, cur, max, 1) };
        s += &format!("map {label} copy=0: {}", hex(kr));
        if kr == 0 {
            let (mut a2, mut s2, mut info, mut cnt, mut on) = (addr, 0u64, [0i32; 16], 16u32, 0u32);
            unsafe {
                ffi::mach_vm_region(
                    me(),
                    &mut a2,
                    &mut s2,
                    k::VM_REGION_BASIC_INFO_64,
                    info.as_mut_ptr(),
                    &mut cnt,
                    &mut on,
                )
            };
            let first = unsafe { std::ptr::read_volatile(addr as *const u8) };
            std::thread::sleep(std::time::Duration::from_millis(150));
            let later = unsafe { std::ptr::read_volatile(addr as *const u8) };
            let up = unsafe { ffi::mach_vm_protect(me(), addr, size, 0, 3) };
            s += &format!(
                " (prot={} max={} first={first:#x} after-150ms={later:#x} protect->RW={})",
                info[0],
                info[1],
                hex(up)
            );
            unsafe { ffi::mach_vm_deallocate(me(), addr, size) };
        }
        s += "; ";
    }
    // copy=TRUE mapping RW (private copy)
    let mut addr = 0u64;
    let kr = unsafe { libc::mach_vm_map(me(), &mut addr, size, 0, 1, entry, 0, 1, 3, 3, 1) };
    s += &format!("map RW copy=1: {}; ", hex(kr));
    // writable child entry from the received one
    let mut sz = size;
    let mut h = 0u32;
    let kr = unsafe { ffi::mach_make_memory_entry_64(me(), &mut sz, 0, 3, &mut h, entry) };
    s += &format!("make_memory_entry_64(RW, parent=received): {} ", hex(kr));
    s
}
