//! M1 D6 macOS cross-process probes (M8 snapshot cost, M9 who is
//! charged, M10 hostile receiver and lifetime). A child reaches the
//! parent only through its bootstrap special port, as D4's probes do;
//! the message helpers are D4's own (`ffi.rs`, `mach.rs`, included by
//! path). phys_footprint is read with footprint(1), one pid at a time.
use crate::ffi::{self, k, me};
use crate::mach::*;
use std::ffi::CString;

extern "C" {
    fn mach_make_memory_entry_64(
        t: u32,
        size: *mut u64,
        offset: u64,
        perm: i32,
        h: *mut u32,
        parent: u32,
    ) -> i32;
    fn mach_vm_allocate(t: u32, addr: *mut u64, size: u64, flags: i32) -> i32;
    fn mach_vm_deallocate(t: u32, addr: u64, size: u64) -> i32;
    fn mach_vm_protect(t: u32, addr: u64, size: u64, set_max: i32, prot: i32) -> i32;
    fn mach_vm_region(
        t: u32,
        addr: *mut u64,
        size: *mut u64,
        flavor: i32,
        info: *mut i32,
        cnt: *mut u32,
        obj: *mut u32,
    ) -> i32;
    // [adoc] kernel/1402380 mach_vm_purgable_control(target, address, control, *state)
    fn mach_vm_purgable_control(t: u32, addr: u64, control: i32, state: *mut i32) -> i32;
    // [adoc] kernel/1402382 mach_vm_behavior_set(target, address, size, behavior)
    fn mach_vm_behavior_set(t: u32, addr: u64, size: u64, behavior: i32) -> i32;
}

const SZ: u64 = 32 << 20;

pub fn footprint(pid: i32) -> u64 {
    let o = std::process::Command::new("footprint")
        .args(["-f", "bytes", "-p", &pid.to_string()])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&o.stdout);
    s.lines()
        .find_map(|l| {
            l.split("Footprint: ")
                .nth(1)
                .and_then(|r| r.split(' ').next())
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0)
}
fn mib(b: u64) -> String {
    format!("{:.1} MiB", b as f64 / 1048576.0)
}
fn map(entry: u32, size: u64, cur: i32, max: i32, copy: i32) -> Result<u64, i32> {
    let mut addr = 0u64;
    let kr = unsafe { libc::mach_vm_map(me(), &mut addr, size, 0, 1, entry, 0, copy, cur, max, 1) };
    if kr == 0 {
        Ok(addr)
    } else {
        Err(kr)
    }
}
fn prot_of(addr: u64) -> (i32, i32) {
    let (mut a, mut s, mut info, mut cnt, mut obj) = (addr, 0u64, [0i32; 16], 16u32, 0u32);
    unsafe {
        mach_vm_region(
            me(),
            &mut a,
            &mut s,
            9,
            info.as_mut_ptr(),
            &mut cnt,
            &mut obj,
        )
    };
    (info[0], info[1])
}
fn fill(addr: u64, size: u64, v: u8) {
    for i in (0..size).step_by(16384) {
        unsafe { std::ptr::write_volatile((addr + i) as *mut u8, v) };
    }
}
fn changed(addr: u64, size: u64, v: u8) -> usize {
    (0..size)
        .step_by(16384)
        .filter(|&i| unsafe { std::ptr::read_volatile((addr + i) as *const u8) } != v)
        .count()
}

struct Kid {
    pid: i32,
    port: u32,
    broker: u32,
}
fn spawn(role: &str) -> Kid {
    let broker = alloc_recv_send();
    let exe = CString::new(std::env::current_exe().unwrap().to_str().unwrap()).unwrap();
    let args = [
        CString::new("d6").unwrap(),
        CString::new("child").unwrap(),
        CString::new(role).unwrap(),
    ];
    let mut argv: Vec<*mut libc::c_char> = args.iter().map(|a| a.as_ptr() as *mut _).collect();
    argv.push(std::ptr::null_mut());
    let mut attr: libc::posix_spawnattr_t = std::ptr::null_mut();
    let mut pid = 0;
    unsafe {
        libc::posix_spawnattr_init(&mut attr);
        assert_eq!(
            ffi::posix_spawnattr_setspecialport_np(&mut attr, broker, k::TASK_BOOTSTRAP_PORT),
            0
        );
        extern "C" {
            static environ: *const *mut libc::c_char;
        }
        assert_eq!(
            libc::posix_spawn(
                &mut pid,
                exe.as_ptr(),
                std::ptr::null(),
                &attr,
                argv.as_ptr(),
                environ
            ),
            0
        );
    }
    let hello = recv(broker, Some(10000)).expect("hello");
    let port = hello.ports[0].0;
    Kid { pid, port, broker }
}
/// Sends one command with optional ports; returns the reply text and ports.
fn ask(c: &Kid, text: &str, ports: Vec<(u32, u32)>) -> Rcvd {
    let rp = alloc_recv();
    let mut o = Out::to(c.port, k::COPY_SEND, 1).text(text);
    o.reply = rp;
    o.reply_disp = k::MAKE_SEND_ONCE;
    o.ports = ports;
    assert_eq!(o.send(), 0);
    let r = recv(rp, Some(60000)).expect("reply");
    unsafe { ffi::mach_port_mod_refs(me(), rp, k::RIGHT_RECEIVE, -1) };
    r
}
fn reply(to: u32, text: &str, ports: Vec<(u32, u32)>) {
    let mut o = Out::to(to, k::MOVE_SEND_ONCE, 2).text(text);
    o.ports = ports;
    o.send();
}

pub fn child_main(_role: &str) {
    let mut boot = 0u32;
    unsafe { ffi::task_get_special_port(me(), k::TASK_BOOTSTRAP_PORT, &mut boot) };
    let mine = alloc_recv();
    let mut o = Out::to(boot, k::COPY_SEND, 0).text("hello");
    o.ports = vec![(mine, k::MAKE_SEND)];
    o.send();
    let mut held: Vec<(u64, u64)> = vec![];
    loop {
        let Ok(m) = recv(mine, None) else { return };
        let t = m.text();
        let w: Vec<&str> = t.split_whitespace().collect();
        let out = match w.first().copied() {
            Some("exit") => {
                reply(m.remote, "bye", vec![]);
                return;
            }
            // map-shared-rw-fill: map the given RW entry shared and write every page.
            Some("map") => {
                let (cur, max, copy) = match w[1] {
                    "rw" => (3, 3, 0),
                    "r" => (1, 1, 0),
                    _ => (1, 1, 1), // snapshot
                };
                match map(m.ports[0].0, SZ, cur, max, copy) {
                    Ok(a) => {
                        if w.get(2) == Some(&"fill") {
                            fill(a, SZ, 0x77);
                        }
                        if w.get(2) == Some(&"touch") {
                            let _ = changed(a, SZ, 0);
                        }
                        held.push((a, SZ));
                        format!("mapped {} at {a:#x} prot {:?}", w[1], prot_of(a))
                    }
                    Err(e) => format!("map {} failed {e:#x}", w[1]),
                }
            }
            Some("count") => {
                let v: u8 = w[1].parse().unwrap();
                let (a, s) = *held.last().unwrap();
                format!(
                    "{} of {} pages differ from {v:#x}",
                    changed(a, s, v),
                    s / 16384
                )
            }
            Some("unmap") => {
                for (a, s) in held.drain(..) {
                    unsafe { mach_vm_deallocate(me(), a, s) };
                }
                "unmapped".into()
            }
            // hostile: everything a receiver of a read-only entry might try.
            Some("hostile") => hostile(m.ports[0].0),
            // make: create an entry of our own and hand it back, then exit.
            Some("make") => {
                let mut a = 0u64;
                unsafe { mach_vm_allocate(me(), &mut a, SZ, 1) };
                fill(a, SZ, 0x3c);
                let mut s = SZ;
                let mut e = 0u32;
                unsafe { mach_make_memory_entry_64(me(), &mut s, a, 1, &mut e, 0) };
                reply(m.remote, "made", vec![(e, k::MOVE_SEND)]);
                unsafe { mach_vm_deallocate(me(), a, SZ) };
                std::process::exit(0);
            }
            _ => format!("unknown {t:?}"),
        };
        reply(m.remote, &out, vec![]);
    }
}

fn hostile(entry: u32) -> String {
    let mut v = vec![];
    // Permission bits on a sub-entry of the RO entry: any that yield an entry mappable RW?
    let mut ok_bits = vec![];
    for b in 1..32 {
        let bit = 1i32.wrapping_shl(b);
        let mut s = SZ;
        let mut h = 0u32;
        let kr = unsafe { mach_make_memory_entry_64(me(), &mut s, 0, 1 | bit, &mut h, entry) };
        if kr == 0 {
            let rw = map(h, SZ, 3, 3, 0);
            ok_bits.push(format!(
                "{bit:#x}:map-RW={}",
                match rw {
                    Ok(a) => {
                        unsafe {
                            std::ptr::write_volatile(a as *mut u8, 0xee);
                            mach_vm_deallocate(me(), a, SZ)
                        };
                        "OK-WROTE".to_string()
                    }
                    Err(e) => format!("{e:#x}"),
                }
            ));
            unsafe { ffi::mach_port_deallocate(me(), h) };
        }
    }
    v.push(format!(
        "sub-entry perm READ|bit accepted for: [{}]",
        ok_bits.join(", ")
    ));
    // Protection bits on a shared RO mapping.
    let a = map(entry, SZ, 1, 1, 0).unwrap();
    let mut prot_hits = vec![];
    for set_max in [0, 1] {
        for b in 0..32 {
            let p = 3 | 1i32.wrapping_shl(b);
            let kr = unsafe { mach_vm_protect(me(), a, SZ, set_max, p) };
            let (cur, max) = prot_of(a);
            if kr == 0 {
                let wrote = if cur & 2 != 0 {
                    unsafe { std::ptr::write_volatile(a as *mut u8, 0xdd) };
                    "wrote"
                } else {
                    "no-write"
                };
                prot_hits.push(format!(
                    "max={set_max} prot={p:#x}->cur {cur} max {max} {wrote}"
                ));
                unsafe { mach_vm_protect(me(), a, SZ, 0, 1) };
            }
        }
    }
    v.push(format!(
        "mach_vm_protect accepted: [{}]",
        prot_hits.join("; ")
    ));
    // Purgeable control and behaviours on the shared mapping.
    let mut pc = vec![];
    for control in 0..4 {
        for st in [0, 1, 2, 3] {
            let mut s = st;
            let kr = unsafe { mach_vm_purgable_control(me(), a, control, &mut s) };
            if kr == 0 {
                pc.push(format!("c{control}/s{st}->{s}"));
            }
        }
    }
    v.push(format!("purgable_control kr=0 for: [{}]", pc.join(" ")));
    let mut bh = vec![];
    for b in 0..16 {
        if unsafe { mach_vm_behavior_set(me(), a, SZ, b) } == 0 {
            bh.push(b.to_string());
        }
    }
    v.push(format!("behavior_set kr=0 for: [{}]", bh.join(" ")));
    let mut md = vec![];
    for adv in 0..16 {
        if unsafe { libc::madvise(a as *mut libc::c_void, SZ as usize, adv) } == 0 {
            md.push(adv.to_string());
        }
    }
    v.push(format!("madvise ok for advice: [{}]", md.join(" ")));
    v.push(format!(
        "receiver's own map now: {} of {} pages differ from 0x5a",
        changed(a, SZ, 0x5a),
        SZ / 16384
    ));
    unsafe { mach_vm_deallocate(me(), a, SZ) };
    v.join(" | ")
}

pub fn parent_main() {
    let pid = |k: &Kid| k.pid;
    // M9: who is charged for pages the receiver writes into a shared entry?
    let mut src = 0u64;
    unsafe { mach_vm_allocate(me(), &mut src, SZ, 1) };
    let mut s = SZ;
    let mut rw = 0u32;
    unsafe { mach_make_memory_entry_64(me(), &mut s, src, 3, &mut rw, 0) };
    unsafe { mach_vm_deallocate(me(), src, SZ) };
    let me_pid = std::process::id() as i32;
    let c = spawn("m9");
    println!(
        "M9 before: parent {} child {}",
        mib(footprint(me_pid)),
        mib(footprint(pid(&c)))
    );
    println!(
        "M9 child: {}",
        ask(&c, "map rw fill", vec![(rw, k::COPY_SEND)]).text()
    );
    println!("M9 child wrote every page of a 32 MiB RW entry the parent made (parent holds no mapping): parent {} child {}", mib(footprint(me_pid)), mib(footprint(pid(&c))));
    let pm = map(rw, SZ, 1, 1, 0).unwrap();
    let _ = changed(pm, SZ, 0x77);
    println!(
        "M9 parent mapped it and read every page: parent {} child {}",
        mib(footprint(me_pid)),
        mib(footprint(pid(&c)))
    );
    println!("M9 child: {}", ask(&c, "unmap", vec![]).text());
    println!(
        "M9 child unmapped: parent {} child {}",
        mib(footprint(me_pid)),
        mib(footprint(pid(&c)))
    );
    ask(&c, "exit", vec![]);
    unsafe { mach_vm_deallocate(me(), pm, SZ) };

    // M8: snapshot (copy=1) cost when the filler keeps writing.
    let mut src = 0u64;
    unsafe { mach_vm_allocate(me(), &mut src, SZ, 1) };
    fill(src, SZ, 0x11);
    let mut s = SZ;
    let mut ro = 0u32;
    unsafe { mach_make_memory_entry_64(me(), &mut s, src, 1, &mut ro, 0) };
    let c = spawn("m8");
    println!(
        "M8 before: parent (filler, 32 MiB written) {} child {}",
        mib(footprint(me_pid)),
        mib(footprint(pid(&c)))
    );
    println!(
        "M8 child: {}",
        ask(&c, "map snap touch", vec![(ro, k::COPY_SEND)]).text()
    );
    println!(
        "M8 child took a copy=1 snapshot and read every page: parent {} child {}",
        mib(footprint(me_pid)),
        mib(footprint(pid(&c)))
    );
    let t = std::time::Instant::now();
    fill(src, SZ, 0x22);
    let el = t.elapsed();
    println!(
        "M8 parent rewrote every page ({:.2} ms; {:.1} µs per 16 KiB page): parent {} child {}",
        el.as_secs_f64() * 1e3,
        el.as_secs_f64() * 1e6 / (SZ / 16384) as f64,
        mib(footprint(me_pid)),
        mib(footprint(pid(&c)))
    );
    println!("M8 child: {}", ask(&c, "count 17", vec![]).text());
    let t = std::time::Instant::now();
    fill(src, SZ, 0x33);
    println!(
        "M8 parent rewrote every page again ({:.2} ms)",
        t.elapsed().as_secs_f64() * 1e3
    );
    println!("M8 child: {}", ask(&c, "unmap", vec![]).text());
    println!(
        "M8 child dropped its snapshot: parent {} child {}",
        mib(footprint(me_pid)),
        mib(footprint(pid(&c)))
    );
    ask(&c, "exit", vec![]);

    // M10: a hostile receiver of a read-only entry; the parent checks its data after.
    fill(src, SZ, 0x5a);
    let c = spawn("m10");
    println!(
        "M10 hostile child: {}",
        ask(&c, "hostile", vec![(ro, k::COPY_SEND)]).text()
    );
    println!(
        "M10 parent's data after: {} of {} pages differ from 0x5a",
        changed(src, SZ, 0x5a),
        SZ / 16384
    );
    ask(&c, "exit", vec![]);

    // M10 lifetime: the child makes an entry, sends it, and exits; we map it after.
    let c = spawn("m10b");
    let r = ask(&c, "make", vec![]);
    let e = r.ports.first().map(|p| p.0).unwrap_or(0);
    let mut st = 0;
    unsafe { libc::waitpid(c.pid, &mut st, 0) };
    match map(e, SZ, 1, 1, 0) {
        Ok(a) => println!("M10 child exited (status {st}); its entry still maps: {} of {} pages differ from 0x3c; kobject type of the name {}", changed(a, SZ, 0x3c), SZ / 16384, kobject(e)),
        Err(kr) => println!("M10 child exited; its entry no longer maps: {kr:#x}"),
    }
    let _ = c.broker;
}
