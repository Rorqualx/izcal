#![allow(deprecated)]
//! D4 Mach IPC black-box probes (Izcal M1 phase D4). See ffi.rs for the source of
//! every binding; constant values are determined by `discover`, never from headers.
mod child;
mod ffi;
mod mach;
mod parent;
mod round2;

use ffi::{k, me};
use mach::*;

macro_rules! out { ($p:expr, $($a:tt)*) => { println!("{} {}", $p, format!($($a)*)) } }
pub(crate) use out;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(|s| s.as_str()) {
        Some("child") => child::child_main(&a),
        Some("discover") => discover(),
        Some("inproc") => inproc_suite("[parent]"),
        Some("cross") => parent::cross(a.iter().any(|x| x == "--sandbox")),
        Some("p3b") => parent::p3b(a.get(2).and_then(|x| x.parse().ok()).unwrap_or(50)),
        Some("p10guard") => p10guard(),
        Some("p14") => parent::p14(),
        Some("p23") => parent::p23(a.get(2).and_then(|x| x.parse().ok()).unwrap_or(60)),
        Some("p28") => parent::p28(),
        Some("p29") => parent::p29(),
        Some("p34") => p34(),
        Some("p19x") => p19x(),
        Some("p29x") => p29x(a.get(2).map(|x| x.as_str()).unwrap_or("first")),
        Some("p2bit") => p2bit(a[2].parse().unwrap()),
        Some("p38") => p38(),
        Some("sbxop") => sbxop(&a[2]),
        Some("sbxcheck") => sbxcheck(),
        Some("p31") => parent::p31(),
        Some("p31x") => parent::p31x(),
        Some("oolports") => oolports(),
        Some("sbxmach") => sbxmach(&a[2]),
        Some("dbg") => parent::dbg(),
        Some("r1") => round2::r1(),
        Some("r2") => round2::r2(),
        Some("r3") => round2::r3(
            a.get(2).and_then(|x| x.parse().ok()).unwrap_or(64),
            a.get(3).and_then(|x| x.parse().ok()).unwrap_or(4),
        ),
        Some("r3child") => round2::r3child(a[2].parse().unwrap(), a[3].parse().unwrap()),
        Some("r4") => round2::r4(),
        Some("r4child") => round2::r4child(a.get(2).map(|x| x.as_str()) == Some("hello")),
        Some("r5") => round2::r5(),
        Some("r6") => round2::r6(),
        Some("r6child") => round2::r6child(),
        Some("r7") => round2::r7(),
        _ => eprintln!(
            "usage: d4-probes discover|inproc|cross [--sandbox]|p3b N|p10guard|p14|p23 SECS|p28|p29|p34|p38|sbxop OP"
        ),
    }
}

/// Re-checks every constant in ffi::k by behaviour. Panics if one no longer holds.
fn discover() {
    let t = me();
    // rights / type bits
    let r = alloc_recv();
    assert_eq!(ptype(r), Ok(k::type_bit(k::RIGHT_RECEIVE)));
    assert_eq!(refs(r, k::RIGHT_RECEIVE), Ok(1));
    let s = alloc_set();
    assert_eq!(
        unsafe { ffi::mach_port_insert_member(t, r, s) },
        0,
        "receive joins port set"
    );
    assert_eq!(ptype(s), Ok(k::type_bit(k::RIGHT_PORT_SET)));
    let mut d = 0;
    unsafe { ffi::mach_port_allocate(t, k::RIGHT_DEAD_NAME, &mut d) };
    assert_eq!(ptype(d), Ok(k::type_bit(k::RIGHT_DEAD_NAME)));
    assert_eq!(
        unsafe { ffi::mach_port_insert_right(t, r, r, k::MAKE_SEND) },
        0
    );
    assert_eq!(refs(r, k::RIGHT_SEND), Ok(1));
    assert_eq!(
        unsafe { ffi::mach_port_insert_right(t, r, r, k::COPY_SEND) },
        0
    );
    assert_eq!(refs(r, k::RIGHT_SEND), Ok(2));
    let (mut so, mut sot) = (0u32, 0u32);
    assert_eq!(
        unsafe { ffi::mach_port_extract_right(t, r, k::MAKE_SEND_ONCE, &mut so, &mut sot) },
        0
    );
    assert_eq!(sot, k::PORT_SEND_ONCE);
    let (mut a1, mut a1t) = (0u32, 0u32);
    assert_eq!(
        unsafe { ffi::mach_port_extract_right(t, r, k::COPY_SEND, &mut a1, &mut a1t) },
        0
    );
    assert_eq!(a1t, k::PORT_SEND);
    // message round trip with the audit trailer
    unsafe { ffi::mach_port_extract_member(t, r, s) };
    assert_eq!(Out::to(r, k::COPY_SEND, 77).text("x").send(), 0);
    let m = recv(r, Some(100)).unwrap();
    assert_eq!(m.id, 77);
    assert_eq!(m.local_disp(), k::PORT_SEND);
    assert_eq!(m.trailer[1], 52);
    assert_eq!(m.pid(), std::process::id() as i32);
    assert_eq!(audit_token_of_task(t).unwrap(), m.audit().unwrap());
    assert_eq!(recv(r, Some(0)).err(), Some(0x10004003));
    assert_eq!(get_qlimit(r), Ok(5));
    for _ in 0..5 {
        assert_eq!(Out::to(r, k::COPY_SEND, 1).send_nb(), 0);
    }
    assert_eq!(Out::to(r, k::COPY_SEND, 1).send_nb(), 0x10000004);
    assert_eq!(status(r).unwrap()[4], 5, "msgcount slot");
    drain(r);
    let mut bp = 0;
    unsafe { ffi::task_get_special_port(t, k::TASK_BOOTSTRAP_PORT, &mut bp) };
    assert_eq!(bp, unsafe { ffi::bootstrap_port });
    // notifications by event
    let n = alloc_recv();
    let x = alloc_recv();
    assert_eq!(
        request(x, k::NOTIFY_NO_SENDERS, 0, n, k::MAKE_SEND_ONCE).0,
        0
    );
    assert_eq!(recv(n, Some(100)).unwrap().id, k::NOTIFY_NO_SENDERS);
    assert_eq!(
        request(x, k::NOTIFY_PORT_DESTROYED, 0, n, k::MAKE_SEND_ONCE).0,
        0
    );
    unsafe { ffi::mach_port_mod_refs(t, x, k::RIGHT_RECEIVE, -1) };
    let pd = recv(n, Some(100)).unwrap();
    assert_eq!(pd.id, k::NOTIFY_PORT_DESTROYED);
    assert_eq!(pd.ports[0].1, k::PORT_RECEIVE);
    let y = alloc_recv_send();
    assert_eq!(
        request(y, k::NOTIFY_DEAD_NAME, 0, n, k::MAKE_SEND_ONCE).0,
        0
    );
    unsafe { ffi::mach_port_mod_refs(t, y, k::RIGHT_RECEIVE, -1) };
    assert_eq!(recv(n, Some(100)).unwrap().id, k::NOTIFY_DEAD_NAME);
    println!("discover: every constant check above holds");
}

/// In-process probes. Run unsandboxed in the parent and inside the deny-default sandbox in a child.
pub fn inproc_suite(p: &str) {
    let t = me();
    let pid = std::process::id();
    // ---- P25 basics: allocation / insert / msg on held rights
    let r = alloc_recv_send();
    out!(
        p,
        "P25 mach_port_allocate+insert_right OK name={r:#x} type={:#x}",
        ptype(r).unwrap()
    );
    // ---- P1 trailer sizes
    let mut sizes = vec![];
    for el in 0..9u32 {
        Out::to(r, k::COPY_SEND, 1).send();
        let mut buf = [0u64; 64];
        let kr = unsafe {
            ffi::mach_msg_overwrite(
                buf.as_mut_ptr().cast(),
                k::RCV_MSG | k::RCV_TIMEOUT | k::rcv_trailer_elements(el),
                0,
                512,
                r,
                0,
                0,
                std::ptr::null_mut(),
                0,
            )
        };
        let m = parse(&buf);
        sizes.push(format!(
            "{el}:{}{}",
            m.trailer.get(1).copied().unwrap_or(0),
            if kr != 0 {
                format!("(kr {})", hex(kr))
            } else {
                String::new()
            }
        ));
    }
    for ty in [1u32, 2, 4, 8, 15] {
        Out::to(r, k::COPY_SEND, 1).send();
        let mut buf = [0u64; 64];
        let kr = unsafe {
            ffi::mach_msg_overwrite(
                buf.as_mut_ptr().cast(),
                k::RCV_MSG | k::RCV_TIMEOUT | ((ty << 28) as i32) | k::rcv_trailer_elements(3),
                0,
                512,
                r,
                0,
                0,
                std::ptr::null_mut(),
                0,
            )
        };
        let m = parse(&buf);
        sizes.push(format!(
            "type{ty}<<28:kr={} size={}",
            hex(kr),
            m.trailer.get(1).copied().unwrap_or(0)
        ));
        drain(r);
    }
    out!(p, "P1 trailer bytes by element count: {}", sizes.join(" "));
    Out::to(r, k::COPY_SEND, 1).send();
    let m = recv(r, Some(100)).unwrap();
    let ti = audit_token_of_task(t);
    out!(
        p,
        "P1 audit trailer {:?} | task_info(15) {:?} | getpid={pid} euid={} egid={} -> trailer==task_info: {}",
        m.audit(),
        ti,
        unsafe { libc::geteuid() },
        unsafe { libc::getegid() },
        ti.ok() == m.audit()
    );
    // ---- P2 forgery
    let fake: Vec<u8> = [0u32, 52, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1]
        .iter()
        .flat_map(|x| x.to_ne_bytes())
        .collect();
    let mut o = Out::to(r, k::COPY_SEND, 21).text("forge");
    o.extra_send = fake.clone();
    let ka = o.send();
    let ra = recv(r, Some(100));
    out!(
        p,
        "P2a fake trailer inside send_size (msgh_size excludes it): send={} -> rcv pid={:?} body_len={:?}",
        hex(ka),
        ra.as_ref().map(|x| x.pid()),
        ra.as_ref().map(|x| x.body.len())
    );
    let mut o = Out::to(r, k::COPY_SEND, 22).text("forge");
    o.after = fake.clone();
    let kb = o.send();
    let rb = recv(r, Some(100));
    out!(
        p,
        "P2b fake trailer after send_size: send={} -> rcv pid={:?}",
        hex(kb),
        rb.as_ref().map(|x| x.pid())
    );
    let mut o = Out::to(r, k::COPY_SEND, 23).text("forge");
    o.extra_send = fake.clone();
    o.size_override = Some(24 + 8 + 52);
    let kc = o.send();
    let rc = recv(r, Some(100));
    out!(
        p,
        "P2c msgh_size covers the fake trailer: send={} -> rcv pid={:?}",
        hex(kc),
        rc.as_ref().map(|x| x.pid())
    );
    out!(
        p,
        "P2d (option-bit scan runs per bit in its own process: `d4-probes p2bit N`)"
    );
    // ---- P4 kernel-originated notification token
    let n = alloc_recv();
    let x = alloc_recv();
    request(x, k::NOTIFY_NO_SENDERS, 0, n, k::MAKE_SEND_ONCE);
    let ns = recv(n, Some(100));
    let y = alloc_recv_send();
    request(y, k::NOTIFY_DEAD_NAME, 0, n, k::MAKE_SEND_ONCE);
    unsafe { ffi::mach_port_mod_refs(t, y, k::RIGHT_RECEIVE, -1) };
    let dn = recv(n, Some(100));
    out!(
        p,
        "P4 no-senders id={:?} token={:?} | dead-name id={:?} token={:?} body={:?}",
        ns.as_ref().map(|m| m.id),
        ns.as_ref().map(|m| m.audit()),
        dn.as_ref().map(|m| m.id),
        dn.as_ref().map(|m| m.audit()),
        dn.as_ref().map(|m| m.raw_words.clone())
    );
    // ---- P5 port set: per-member seqno, local name, token
    let set = alloc_set();
    let (a1, a2) = (alloc_recv_send(), alloc_recv_send());
    unsafe {
        ffi::mach_port_insert_member(t, a1, set);
        ffi::mach_port_insert_member(t, a2, set)
    };
    for i in 0..3 {
        Out::to(a1, k::COPY_SEND, 500 + i).send();
        Out::to(a2, k::COPY_SEND, 600 + i).send();
    }
    let v: Vec<String> = drain(set)
        .iter()
        .map(|m| {
            format!(
                "{}@{}:seq{}:pid{}",
                m.id,
                if m.local == a1 {
                    "a1"
                } else if m.local == a2 {
                    "a2"
                } else {
                    "?"
                },
                m.seqno(),
                m.pid()
            )
        })
        .collect();
    out!(p, "P5 via set: {}", v.join(" "));
    out!(
        p,
        "P25 port set alloc/insert_member/receive-from-set OK (above)"
    );
    // ---- P7 type bits
    let so_holder = alloc_recv();
    let (mut so, mut sot) = (0u32, 0u32);
    unsafe { ffi::mach_port_extract_right(t, so_holder, k::MAKE_SEND_ONCE, &mut so, &mut sot) };
    let send_only = {
        let q = alloc_recv();
        let (mut s1, mut s1t) = (0u32, 0u32);
        unsafe { ffi::mach_port_extract_right(t, q, k::MAKE_SEND, &mut s1, &mut s1t) };
        s1
    };
    let mut dead = 0;
    unsafe { ffi::mach_port_allocate(t, k::RIGHT_DEAD_NAME, &mut dead) };
    let dnr = alloc_recv_send();
    let nn = alloc_recv();
    let k_dnr = request(dnr, k::NOTIFY_DEAD_NAME, 0, nn, k::MAKE_SEND_ONCE).0;
    out!(
        p,
        "P7 mach_port_type: receive={:#x} send(name differs? {})={:#x} recv+send={:#x} send-once={:#x} port-set={:#x} dead-name={:#x} recv+send+dead-name-request(kr {})={:#x} invalid-name={:?}",
        ptype(so_holder).unwrap(),
        send_only != so_holder,
        ptype(send_only).unwrap_or(0),
        ptype(r).unwrap(),
        ptype(so).unwrap(),
        ptype(set).unwrap(),
        ptype(dead).unwrap(),
        hex(k_dnr),
        ptype(dnr).unwrap(),
        ptype(0x12345).err().map(hex)
    );
    // ---- P8 kobject classification on local objects
    let mut tn = 0u32;
    let ktn = unsafe { ffi::task_name_for_pid(t, pid as i32, &mut tn) };
    let thr = unsafe { libc::mach_thread_self() };
    let host = unsafe { libc::mach_host_self() };
    let mut sem = 0u32;
    let ksem = unsafe { ffi::semaphore_create(t, &mut sem, 0, 0) };
    let mut buf_addr = 0u64;
    unsafe { ffi::mach_vm_allocate(t, &mut buf_addr, 16384, 1) };
    let mut msz = 16384u64;
    let mut me_h = 0u32;
    let kme = unsafe { ffi::mach_make_memory_entry_64(t, &mut msz, buf_addr, 1, &mut me_h, 0) };
    let mut bp = 0u32;
    let kbp = unsafe { ffi::task_get_special_port(t, k::TASK_BOOTSTRAP_PORT, &mut bp) };
    let svc = unsafe {
        ffi::IOServiceGetMatchingService(
            ffi::kIOMainPortDefault,
            ffi::IOServiceMatching(c"IOSurfaceRoot".as_ptr()),
        )
    };
    let mut conn = 0u32;
    let kconn = if svc != 0 {
        unsafe { ffi::IOServiceOpen(svc, t, 0, &mut conn) }
    } else {
        -1
    };
    out!(
        p,
        "P8 setup: task_name_for_pid(self)={} semaphore_create={} make_memory_entry_64(R)={} task_get_special_port(bootstrap)={} IOServiceGetMatchingService(IOSurfaceRoot)={svc:#x} IOServiceOpen={}",
        hex(ktn),
        hex(ksem),
        hex(kme),
        hex(kbp),
        hex(kconn)
    );
    for (label, name) in [
        ("task_self", t),
        ("task_name", tn),
        ("thread_self", thr),
        ("host", host),
        ("semaphore", sem),
        ("mem_entry", me_h),
        ("bootstrap(send)", bp),
        ("iokit_service", svc),
        ("iokit_connect", conn),
        ("plain recv+send", r),
        ("plain send-only", send_only),
        ("send-once", so),
        ("dead_name", dead),
        ("port_set", set),
        ("null", 0),
    ] {
        out!(
            p,
            "P8 {label:16} name={name:#x} type={:?} {}",
            ptype(name).map(|t| format!("{t:#x}")).map_err(hex),
            kobject(name)
        );
    }
    // ---- P11 sizes
    let mut sz = vec![];
    for (np, no) in [(0usize, 0usize), (1, 0), (16, 0), (0, 1), (0, 2), (16, 2)] {
        let mut o = Out::to(r, k::COPY_SEND, 11).text("abcd");
        o.ports = (0..np).map(|_| (r, k::MAKE_SEND)).collect();
        o.ools = (0..no).map(|_| (buf_addr, 4096, false, 1)).collect();
        let (enc, ssz) = o.encode();
        let _ = enc;
        let ks = o.send();
        let m = recv(r, Some(100));
        if let Ok(m) = &m {
            release(m);
        }
        sz.push(format!(
            "{np}p+{no}ool: send_size={ssz} send={} rcv_msgh_size={:?}",
            hex(ks),
            m.map(|m| m.size).map_err(hex)
        ));
    }
    out!(p, "P11 {}", sz.join(" | "));
    // ---- P13 max inline size
    let big = alloc_recv_send();
    let (mut lo, mut hi) = (24u32, 512 * 1024 * 1024u32);
    let mut last_err = 0;
    while hi - lo > 4 {
        let mid = (lo + (hi - lo) / 2) & !3;
        let mut o = Out::to(big, k::COPY_SEND, 13);
        o.body = vec![0u8; (mid - 24) as usize];
        let kr = o.send_nb();
        if kr == 0 {
            lo = mid;
            drain_big(big, mid as usize + 1024);
        } else {
            hi = mid;
            last_err = kr;
        }
    }
    let mut o = Out::to(big, k::COPY_SEND, 13);
    o.body = vec![0u8; 65536];
    o.ports = (0..16).map(|_| (r, k::MAKE_SEND)).collect();
    let k64 = o.send_nb();
    let m64 = recv_opts(big, Some(100), 70000, 0);
    let ok64 = m64
        .as_ref()
        .map(|m| (m.ports.len(), m.body.len(), m.size))
        .map_err(|e| hex(*e));
    if let Ok(m) = &m64 {
        release(m);
    }
    out!(
        p,
        "P13 max inline msgh_size accepted={lo} first refused={hi} err={} | 64KiB+16 port descriptors: send={} received (ports, body, msgh_size)={:?}",
        hex(last_err),
        hex(k64),
        ok64
    );
    // ---- P16 queue limits
    let q = alloc_recv_send();
    let dflt = get_qlimit(q);
    let mut acc = vec![];
    for v in [0, 1, 1024, 1025, 2048] {
        acc.push(format!("{v}:{}", hex(set_qlimit(q, v))));
    }
    set_qlimit(q, 0);
    let k0 = Out::to(q, k::COPY_SEND, 1).send_nb();
    set_qlimit(q, 1024);
    let mut n64 = 0;
    let mut e64 = 0;
    loop {
        let mut o = Out::to(q, k::COPY_SEND, 16);
        o.body = vec![1u8; 65536];
        let kr = o.send_nb();
        if kr != 0 {
            e64 = kr;
            break;
        }
        n64 += 1;
        if n64 > 5000 {
            break;
        }
    }
    while recv_opts(q, Some(0), 70000, 0).is_ok() {}
    out!(
        p,
        "P16 default qlimit={dflt:?} set: [{}] | qlimit 0 -> send {} | qlimit 1024 filled with 64KiB msgs: {n64} accepted then {}",
        acc.join(" "),
        hex(k0),
        hex(e64)
    );
    // ---- P17 non-blocking send on a full queue
    let f = alloc_recv_send();
    set_qlimit(f, 1);
    Out::to(f, k::COPY_SEND, 1).send();
    let mut lat = vec![];
    let mut codes = std::collections::BTreeMap::new();
    for _ in 0..10000 {
        let t0 = now_ns();
        let kr = Out::to(f, k::COPY_SEND, 2).send_nb();
        lat.push(now_ns() - t0);
        *codes.entry(hex(kr)).or_insert(0) += 1;
    }
    lat.sort();
    out!(
        p,
        "P17 timeout-0 send to full queue x10000: codes={codes:?} latency ns min={} p50={} p99={} max={}",
        lat[0],
        lat[5000],
        lat[9900],
        lat[9999]
    );
    // rights and OOL in a failed send
    let carried = alloc_recv_send();
    let before = (refs(carried, k::RIGHT_SEND), ptype(carried));
    let movr = alloc_recv();
    let mut ool_addr = 0u64;
    unsafe {
        ffi::mach_vm_allocate(t, &mut ool_addr, 16384, 1);
        *(ool_addr as *mut u8) = 0x42
    };
    let mut o = Out::to(f, k::COPY_SEND, 3);
    o.ports = vec![(carried, k::MOVE_SEND), (movr, k::MOVE_RECEIVE)];
    o.ools = vec![(ool_addr, 16384, true, 1)];
    let (kr, after_buf) = o.send_opts(k::SEND_TIMEOUT, 0);
    let pr = parse(&after_buf);
    let ool_mapped = unsafe { ffi::mach_vm_protect(t, ool_addr, 16384, 0, 3) } == 0;
    out!(
        p,
        "P17 failed send carrying MOVE_SEND({carried:#x}), MOVE_RECEIVE({movr:#x}), OOL dealloc: kr={} | carried send refs before {:?} after {:?} | moved receive type after {:?} | OOL original still mapped={ool_mapped} | buffer after call: ports={:x?} ools={:x?}",
        hex(kr),
        before,
        (refs(carried, k::RIGHT_SEND), ptype(carried)),
        ptype(movr).map(|t| format!("{t:#x}")).map_err(hex),
        pr.ports,
        pr.ools
    );
    // resend works after drain
    drain(f);
    let mut ab = after_buf.clone();
    let ssz = {
        let (_, z) = o.encode();
        z
    };
    let kre2 = unsafe {
        ffi::mach_msg_overwrite(
            ab.as_mut_ptr().cast(),
            k::SEND_MSG | k::SEND_TIMEOUT,
            ssz,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            0,
        )
    };
    let back = recv(f, Some(100));
    out!(
        p,
        "P17 resend after drain using the buffer as rewritten by the failed call -> {} (received ports {:?}, OOL first byte ok={:?})",
        hex(kre2),
        back.as_ref().map(|m| m.ports.clone()).map_err(|e| hex(*e)),
        back.as_ref()
            .map(|m| unsafe { *(m.ools[0].0 as *const u8) } == 0x42)
    );
    let carried2 = alloc_recv_send();
    let movr2 = alloc_recv();
    Out::to(f, k::COPY_SEND, 1).send();
    let mut o2 = Out::to(f, k::COPY_SEND, 3);
    o2.ports = vec![(carried2, k::MOVE_SEND), (movr2, k::MOVE_RECEIVE)];
    let (k2, _) = o2.send_opts(k::SEND_TIMEOUT, 0);
    drain(f);
    let kre = o2.send_nb();
    out!(
        p,
        "P17 resend re-encoded with the ORIGINAL names after a failed send ({}): {} | carried send right afterwards type={:?}",
        hex(k2),
        hex(kre),
        ptype(carried2).map(|x| format!("{x:#x}")).map_err(hex)
    );
    drain(f);
    // signal storm
    p17_signals(p, f);
    // ---- P18 send-once ignores qlimit
    let so_port = alloc_recv_send();
    set_qlimit(so_port, 1);
    Out::to(so_port, k::COPY_SEND, 1).send();
    let mut nso = 0;
    let mut eso = 0;
    for _ in 0..20000 {
        let (mut sorr, mut sot) = (0u32, 0u32);
        unsafe { ffi::mach_port_extract_right(t, so_port, k::MAKE_SEND_ONCE, &mut sorr, &mut sot) };
        let kr = Out::to(sorr, k::MOVE_SEND_ONCE, 18).send_nb();
        if kr != 0 {
            eso = kr;
            unsafe { ffi::mach_port_deallocate(t, sorr) };
            break;
        }
        nso += 1;
    }
    out!(
        p,
        "P18 qlimit 1 port already full; send-once sends accepted={nso} (cap 20000) first error={} msgcount={:?}",
        hex(eso),
        status(so_port).map(|s| s[4])
    );
    drain(so_port);
    // ---- P19 send-possible
    p19(p);
    // ---- P20 no-senders
    p20_local(p);
    // ---- P22 cleanup
    p22(p);
    // ---- P24 oversize receive
    let o24 = alloc_recv_send();
    let carried = alloc_recv(); // receive right only: the message's MAKE_SEND is the only send right
    let nn24 = alloc_recv();
    let (kreq, _) = request(carried, k::NOTIFY_NO_SENDERS, 1, nn24, k::MAKE_SEND_ONCE);
    let mut o = Out::to(o24, k::COPY_SEND, 24);
    o.ports = vec![(carried, k::MAKE_SEND)];
    o.body = vec![7; 1000];
    o.send();
    let small = recv_opts(o24, Some(0), 64, 0);
    let after_small = status(o24).map(|s| s[4]);
    let ns24 = recv(nn24, Some(200));
    let mut o = Out::to(o24, k::COPY_SEND, 25);
    o.body = vec![7; 1000];
    o.send();
    let large = recv_opts(o24, Some(0), 64, k::RCV_LARGE);
    let after_large = status(o24).map(|s| s[4]);
    let full = recv(o24, Some(0));
    out!(
        p,
        "P24 64-byte buffer, no RCV_LARGE: {:?} msgcount after={after_small:?} no-senders on carried port (req {}): {:?} | with RCV_LARGE: {:?} msgcount after={after_large:?} then full receive ok={}",
        small.err().map(hex),
        hex(kreq),
        ns24.map(|m| m.id).map_err(hex),
        large.as_ref().err().map(|e| hex(*e)),
        full.is_ok()
    );
    // ---- P32 alias + uref limit
    let al = alloc_recv_send();
    let tgt = alloc_recv_send();
    let mut o = Out::to(al, k::COPY_SEND, 32);
    o.ports = (0..16).map(|_| (tgt, k::COPY_SEND)).collect();
    let before = refs(tgt, k::RIGHT_SEND);
    o.send();
    let m = recv(al, Some(100)).unwrap();
    let names: std::collections::BTreeSet<u32> = m.ports.iter().map(|x| x.0).collect();
    out!(
        p,
        "P32 16x COPY_SEND of one right: received names={names:x?} (own name {tgt:#x}) urefs before={before:?} after receive={:?}",
        refs(tgt, k::RIGHT_SEND)
    );
    let mut maxref = 0;
    let mut kov = 0;
    for _ in 0..200000 {
        let kr = unsafe { ffi::mach_port_mod_refs(t, tgt, k::RIGHT_SEND, 1) };
        if kr != 0 {
            kov = kr;
            break;
        }
    }
    if let Ok(u) = refs(tgt, k::RIGHT_SEND) {
        maxref = u;
    }
    let mut o = Out::to(al, k::COPY_SEND, 33);
    o.ports = vec![(tgt, k::COPY_SEND)];
    let ks = o.send();
    let rr = recv(al, Some(100));
    out!(
        p,
        "P32 uref limit: mod_refs +1 stops at urefs={maxref} with {} | COPY_SEND of a right at max: send={} receive={:?} urefs after={:?}",
        hex(kov),
        hex(ks),
        rr.as_ref().map(|m| m.ports.clone()).map_err(|e| hex(*e)),
        refs(tgt, k::RIGHT_SEND)
    );
    // ---- P33 local memory entry
    let mut src = 0u64;
    unsafe {
        ffi::mach_vm_allocate(t, &mut src, 16384, 1);
        *(src as *mut u8) = 0x11
    };
    let mut msz = 16384u64;
    let mut ro = 0u32;
    let kro = unsafe { ffi::mach_make_memory_entry_64(t, &mut msz, src, 1, &mut ro, 0) };
    let creator_write = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        unsafe { *(src as *mut u8) = 0x22 }
    });
    let s = child::mementry_tests(ro);
    creator_write.join().unwrap();
    out!(p, "P33 local read-only entry (make kr {}): {s}", hex(kro));
    // ---- mach_ports_register (P28 local)
    let reg = alloc_recv_send();
    let mut arr = [reg];
    let krg = unsafe { ffi::mach_ports_register(t, arr.as_mut_ptr(), 1) };
    let (mut la, mut lc) = (std::ptr::null_mut::<u32>(), 0u32);
    let klu = unsafe { ffi::mach_ports_lookup(t, &mut la, &mut lc) };
    out!(
        p,
        "P28 mach_ports_register(self,[port])={} mach_ports_lookup={} count={lc}",
        hex(krg),
        hex(klu)
    );
    // ---- task_get_bootstrap_port equivalent and bootstrap_check_in from here
    let mut sp = 0u32;
    let kcheck =
        unsafe { ffi::bootstrap_check_in(bp, c"org.izcal.d4probe.unlisted".as_ptr(), &mut sp) };
    out!(
        p,
        "P29 bootstrap_check_in(bootstrap_port, unlisted name)={} ({})",
        hex(kcheck),
        kcheck
    );
    out!(p, "inproc done");
}

fn p2bit(bit: u32) {
    let pid = std::process::id() as i32;
    let r = alloc_recv_send();
    let fake: Vec<u8> = [0u32, 52, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1]
        .iter()
        .flat_map(|x| x.to_ne_bytes())
        .collect();
    let mut o = Out::to(r, k::COPY_SEND, 24).text("forge");
    o.extra_send = fake;
    let (kr, _) = o.send_opts(k::SEND_TIMEOUT | (1i32 << bit), 0);
    let got = recv(r, Some(0));
    println!(
        "P2d bit{bit:2} ({:#010x}): send kr={} received pid={:?} (own {pid}) body_len={:?}",
        1u32 << bit,
        hex(kr),
        got.as_ref().map(|m| m.pid()).map_err(|e| hex(*e)),
        got.as_ref().map(|m| m.body.len())
    );
}

fn p19x() {
    let nport = alloc_recv();
    for bit in [
        5u32, 6, 7, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    ] {
        for order in ["req-then-send", "send-then-req"] {
            let f = alloc_recv_send();
            set_qlimit(f, 1);
            Out::to(f, k::COPY_SEND, 1).send();
            let mut kq = 0;
            if order == "req-then-send" {
                kq = request(f, k::NOTIFY_SEND_POSSIBLE, 0, nport, k::MAKE_SEND_ONCE).0;
            }
            let (ks, _) = Out::to(f, k::COPY_SEND, 2).send_opts(k::SEND_TIMEOUT | (1 << bit), 0);
            if order == "send-then-req" {
                kq = request(f, k::NOTIFY_SEND_POSSIBLE, 0, nport, k::MAKE_SEND_ONCE).0;
            }
            let pre = recv(nport, Some(20)).map(|m| m.id);
            recv(f, Some(0)).ok();
            let post = recv(nport, Some(100)).map(|m| m.id);
            if pre.is_ok() || post.is_ok() || ks != 0x10000004 {
                println!(
                    "P19x bit{bit} ({:#x}) {order}: req={} send={} notification before dequeue={:?} after={:?}",
                    1u32 << bit,
                    hex(kq),
                    hex(ks),
                    pre.map_err(hex),
                    post.map_err(hex)
                );
            }
            unsafe { ffi::mach_port_mod_refs(me(), f, k::RIGHT_RECEIVE, -1) };
            drain(nport);
        }
    }
    println!("P19x scan done (only bits that changed anything are printed)");
}

fn p29x(role: &str) {
    let mut sp = 0u32;
    let kr = unsafe {
        ffi::bootstrap_check_in(
            ffi::bootstrap_port,
            c"org.izcal.d4probe.unlisted".as_ptr(),
            &mut sp,
        )
    };
    println!(
        "P29 [{role} pid {}] bootstrap_check_in(unlisted)={} ({kr}) port={sp:#x} type={:?} {}",
        std::process::id(),
        hex(kr),
        ptype(sp).map(|x| format!("{x:#x}")).map_err(hex),
        if sp != 0 { kobject(sp) } else { String::new() }
    );
    if kr == 0 {
        let mut sp2 = 0u32;
        let kr2 = unsafe {
            ffi::bootstrap_check_in(
                ffi::bootstrap_port,
                c"org.izcal.d4probe.unlisted".as_ptr(),
                &mut sp2,
            )
        };
        println!(
            "P29 [{role}] second check_in same name same process={} ({kr2}) port={sp2:#x}",
            hex(kr2)
        );
    }
    if role == "first" {
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}

fn drain_big(port: u32, sz: usize) {
    while recv_opts(port, Some(0), sz, 0).is_ok() {}
}

extern "C" fn on_usr1(_: i32) {}
fn p17_signals(p: &str, f: u32) {
    unsafe { libc::signal(libc::SIGUSR1, on_usr1 as usize) };
    let me_thread = unsafe { libc::pthread_self() } as usize;
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let st = stop.clone();
    let h = std::thread::spawn(move || {
        let mut n = 0u64;
        while !st.load(std::sync::atomic::Ordering::Relaxed) {
            unsafe { libc::pthread_kill(me_thread as libc::pthread_t, libc::SIGUSR1) };
            n += 1;
            std::thread::sleep(std::time::Duration::from_micros(20));
        }
        n
    });
    Out::to(f, k::COPY_SEND, 1).send_nb();
    let mut codes = std::collections::BTreeMap::new();
    let mut maxl = 0;
    for _ in 0..20000 {
        let t0 = now_ns();
        let kr = Out::to(f, k::COPY_SEND, 2).send_nb();
        maxl = maxl.max(now_ns() - t0);
        *codes.entry(hex(kr)).or_insert(0) += 1;
    }
    // blocking send (no timeout) while signals arrive; a helper drains after 200 ms
    let drainer = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(200));
        recv(f, Some(1000)).map(|m| m.id)
    });
    let t0 = now_ns();
    let kb = Out::to(f, k::COPY_SEND, 3).send();
    let blocked_ms = (now_ns() - t0) / 1_000_000;
    let _ = drainer.join();
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let nsig = h.join().unwrap();
    drain(f);
    unsafe { libc::signal(libc::SIGUSR1, libc::SIG_DFL) };
    out!(
        p,
        "P17 signal storm ({nsig} SIGUSR1 to the sending thread): timeout-0 sends x20000 codes={codes:?} max latency {maxl} ns | blocking send (no SEND_INTERRUPT) on full queue: kr={} returned after {blocked_ms} ms (drained at 200 ms)",
        hex(kb)
    );
}

fn p19(p: &str) {
    let t = me();
    let nport = alloc_recv();
    // a) not full: does a request fire at once?
    let a = alloc_recv_send();
    let (ka, _) = request(a, k::NOTIFY_SEND_POSSIBLE, 0, nport, k::MAKE_SEND_ONCE);
    let ra = recv(nport, Some(100));
    out!(
        p,
        "P19a queue not full, request(66, sync 0)={} -> notification within 100 ms: {:?}",
        hex(ka),
        ra.as_ref()
            .map(|m| (m.id, m.raw_words.clone()))
            .map_err(|e| hex(*e))
    );
    // b) full, request, dequeue one
    let b = alloc_recv_send();
    set_qlimit(b, 1);
    Out::to(b, k::COPY_SEND, 1).send();
    let (kb, _) = request(b, k::NOTIFY_SEND_POSSIBLE, 0, nport, k::MAKE_SEND_ONCE);
    let pre = recv(nport, Some(50));
    let t0 = now_ns();
    recv(b, Some(0)).ok();
    let rb = recv(nport, Some(200));
    let dt = now_ns() - t0;
    out!(
        p,
        "P19b full queue, request={} (pre-dequeue notification: {:?}); after dequeue: {:?} after {} us",
        hex(kb),
        pre.as_ref().map(|m| m.id).map_err(|e| hex(*e)),
        rb.as_ref()
            .map(|m| (m.id, m.raw_words.clone(), m.audit().map(|a| a[5])))
            .map_err(|e| hex(*e)),
        dt / 1000
    );
    // c) failed send first, then request, then dequeue
    let c = alloc_recv_send();
    set_qlimit(c, 1);
    Out::to(c, k::COPY_SEND, 1).send();
    let kf = Out::to(c, k::COPY_SEND, 2).send_nb();
    let (kc, _) = request(c, k::NOTIFY_SEND_POSSIBLE, 0, nport, k::MAKE_SEND_ONCE);
    let prec = recv(nport, Some(50));
    recv(c, Some(0)).ok();
    let rc = recv(nport, Some(200));
    // one-shot? fill and dequeue again
    Out::to(c, k::COPY_SEND, 3).send_nb();
    Out::to(c, k::COPY_SEND, 4).send_nb();
    recv(c, Some(0)).ok();
    let rc2 = recv(nport, Some(200));
    out!(
        p,
        "P19c failed send ({}) then request={} pre={:?} after dequeue={:?} second fill+dequeue without re-arm={:?}",
        hex(kf),
        hex(kc),
        prec.as_ref().map(|m| m.id).map_err(|e| hex(*e)),
        rc.as_ref().map(|m| m.id).map_err(|e| hex(*e)),
        rc2.as_ref().map(|m| m.id).map_err(|e| hex(*e))
    );
    // d) sync = 1
    let d = alloc_recv_send();
    set_qlimit(d, 1);
    Out::to(d, k::COPY_SEND, 1).send();
    let (kd, _) = request(d, k::NOTIFY_SEND_POSSIBLE, 1, nport, k::MAKE_SEND_ONCE);
    let pred = recv(nport, Some(50));
    out!(
        p,
        "P19d full queue request(sync 1)={} -> immediate: {:?}",
        hex(kd),
        pred.as_ref().map(|m| m.id).map_err(|e| hex(*e))
    );
    let (ka2, _) = request(a, k::NOTIFY_SEND_POSSIBLE, 1, nport, k::MAKE_SEND_ONCE);
    let ra2 = recv(nport, Some(50));
    out!(
        p,
        "P19d' not-full queue request(sync 1)={} -> immediate: {:?}",
        hex(ka2),
        ra2.as_ref().map(|m| m.id).map_err(|e| hex(*e))
    );
    drain(nport);
    // e) port dies with request armed
    let e = alloc_recv_send();
    set_qlimit(e, 1);
    Out::to(e, k::COPY_SEND, 1).send();
    request(e, k::NOTIFY_SEND_POSSIBLE, 0, nport, k::MAKE_SEND_ONCE);
    unsafe { ffi::mach_port_mod_refs(t, e, k::RIGHT_RECEIVE, -1) };
    let re = drain(nport);
    out!(
        p,
        "P19e receive right destroyed with send-possible armed: notifications {:?}",
        re.iter().map(|m| m.id).collect::<Vec<_>>()
    );
}

fn p20_local(p: &str) {
    let t = me();
    let nport = alloc_recv();
    // (c) a send right in flight delays no-senders
    let pp = alloc_recv();
    let mut s = 0u32;
    let mut st = 0u32;
    unsafe { ffi::mach_port_extract_right(t, pp, k::MAKE_SEND, &mut s, &mut st) }; // mscount 1, name s (merges into pp)
    let (kr, _) = request(pp, k::NOTIFY_NO_SENDERS, 1, nport, k::MAKE_SEND_ONCE);
    let holder = alloc_recv_send();
    let mut o = Out::to(holder, k::COPY_SEND, 1);
    o.ports = vec![(pp, k::COPY_SEND)];
    o.send();
    unsafe { ffi::mach_port_deallocate(t, pp) }; // drop our send right; the copy is in flight
    let early = recv(nport, Some(100));
    let m = recv(holder, Some(100)).unwrap();
    let still = recv(nport, Some(50));
    release(&m);
    let late = recv(nport, Some(100));
    out!(
        p,
        "P20c request(sync 1)={} | in-flight copy queued, own right dropped -> notification in 100 ms: {:?} | after receive (right now held): {:?} | after releasing it: {:?}",
        hex(kr),
        early.map(|m| m.id).map_err(hex),
        still.map(|m| m.id).map_err(hex),
        late.as_ref()
            .map(|m| (m.id, m.body.clone()))
            .map_err(|e| hex(*e))
    );
    // (c2) destroyed in queue: right inside a message destroyed with its port
    let pp2 = alloc_recv();
    let (mut s2, mut s2t) = (0u32, 0u32);
    unsafe { ffi::mach_port_extract_right(t, pp2, k::MAKE_SEND, &mut s2, &mut s2t) };
    request(pp2, k::NOTIFY_NO_SENDERS, 1, nport, k::MAKE_SEND_ONCE);
    let holder2 = alloc_recv_send();
    let mut o = Out::to(holder2, k::COPY_SEND, 1);
    o.ports = vec![(pp2, k::MOVE_SEND)];
    o.send();
    let e2 = recv(nport, Some(50));
    unsafe { ffi::mach_port_mod_refs(t, holder2, k::RIGHT_RECEIVE, -1) };
    let l2 = recv(nport, Some(100));
    out!(
        p,
        "P20c' right moved into a queued message: before destroying holder {:?}; after destroying holder's receive right {:?}",
        e2.map(|m| m.id).map_err(hex),
        l2.map(|m| m.id).map_err(hex)
    );
    // (d) sync threshold
    let x = alloc_recv();
    let (mut xs, mut xst) = (0u32, 0u32);
    unsafe { ffi::mach_port_extract_right(t, x, k::MAKE_SEND, &mut xs, &mut xst) }; // mscount 1
    unsafe { ffi::mach_port_deallocate(t, x) }; // no senders now, mscount 1
    let (k2, _) = request(x, k::NOTIFY_NO_SENDERS, 2, nport, k::MAKE_SEND_ONCE);
    let d1 = recv(nport, Some(50));
    unsafe { ffi::mach_port_extract_right(t, x, k::MAKE_SEND, &mut xs, &mut xst) }; // mscount 2
    let d2 = recv(nport, Some(50));
    unsafe { ffi::mach_port_deallocate(t, x) };
    let d3 = recv(nport, Some(100));
    let (k1, _) = request(x, k::NOTIFY_NO_SENDERS, 1, nport, k::MAKE_SEND_ONCE);
    let d4 = recv(nport, Some(50));
    out!(
        p,
        "P20d mscount=1 no senders: request(sync 2)={} fires? {:?}; make-send (mscount 2): {:?}; drop it: {:?} (count word={:?}); re-request(sync 1) with mscount 2 and no senders={} -> {:?}",
        hex(k2),
        d1.map(|m| m.id).map_err(hex),
        d2.map(|m| m.id).map_err(hex),
        d3.as_ref().map(|m| m.id).map_err(|e| hex(*e)),
        d3.as_ref().ok().and_then(|m| m
            .body
            .get(8..12)
            .map(|b| u32::from_ne_bytes(b.try_into().unwrap()))),
        hex(k1),
        d4.map(|m| m.id).map_err(hex)
    );
}

fn p22(p: &str) {
    let t = me();
    let port = alloc_recv_send();
    set_qlimit(port, 1024);
    // warm-up so name-table growth is not counted
    for _ in 0..3 {
        let x = alloc_recv();
        unsafe { ffi::mach_port_mod_refs(t, x, k::RIGHT_RECEIVE, -1) };
    }
    let before = names_count();
    let mut sent_regions = vec![];
    for _ in 0..100 {
        let rcv = alloc_recv();
        let snd = alloc_recv_send(); // we keep its receive right; send one MAKE_SEND
        let so = alloc_recv();
        let mut addr = 0u64;
        unsafe { ffi::mach_vm_allocate(t, &mut addr, 16384, 1) };
        sent_regions.push(addr);
        let mut o = Out::to(port, k::COPY_SEND, 22);
        o.ports = vec![
            (rcv, k::MOVE_RECEIVE),
            (snd, k::MAKE_SEND),
            (so, k::MAKE_SEND_ONCE),
            (0, k::COPY_SEND),
            (u32::MAX, k::COPY_SEND),
        ];
        o.ools = vec![(addr, 16384, true, 1)];
        let kr = o.send();
        assert_eq!(kr, 0, "p22 send");
        unsafe {
            ffi::mach_port_mod_refs(t, snd, k::RIGHT_RECEIVE, -1);
            ffi::mach_port_deallocate(t, snd);
            ffi::mach_port_mod_refs(t, so, k::RIGHT_RECEIVE, -1)
        };
    }
    let mid = names_count();
    let mut regions = vec![];
    let mut kinds = std::collections::BTreeSet::new();
    for m in drain(port) {
        for &(n, d) in &m.ports {
            kinds.insert((
                d,
                if n == 0 {
                    "null"
                } else if n == u32::MAX {
                    "dead"
                } else {
                    "name"
                },
            ));
        }
        for &(a, _, _) in &m.ools {
            regions.push(a);
        }
        release(&m);
    }
    let after = names_count();
    let still_mapped = regions
        .iter()
        .filter(|&&a| unsafe { ffi::mach_vm_protect(t, a, 16384, 0, 1) } == 0)
        .count();
    out!(
        p,
        "P22 names: before={before} after-send={mid} after-receive+release={after} (diff {}) | received descriptor kinds {kinds:?} | received OOL regions still mapped after vm_deallocate: {still_mapped}/{}",
        after as i64 - before as i64,
        regions.len()
    );
}

/// P10: guarded receive right moved in a message (may raise EXC_GUARD, so it runs in its own process).
fn p10guard() {
    let t = me();
    let r = alloc_recv();
    let kg = unsafe { ffi::mach_port_guard(t, r, 0xfeed, 0) };
    println!("P10g mach_port_guard(non-strict)={}", hex(kg));
    let dst = alloc_recv_send();
    let mut o = Out::to(dst, k::COPY_SEND, 10);
    o.ports = vec![(r, k::MOVE_RECEIVE)];
    let ks = o.send();
    println!(
        "P10g send MOVE_RECEIVE of a guarded receive right: kr={} type after={:?}",
        hex(ks),
        ptype(r).map(|x| format!("{x:#x}")).map_err(hex)
    );
    let kd = unsafe { ffi::mach_port_mod_refs(t, r, k::RIGHT_RECEIVE, -1) };
    println!(
        "P10g mod_refs(RECEIVE,-1) on guarded right: {} (process survived)",
        hex(kd)
    );
    let r2 = alloc_recv();
    let kg2 = unsafe { ffi::mach_port_guard(t, r2, 0xbeef, 1) };
    println!("P10g mach_port_guard(strict)={}", hex(kg2));
    use std::io::Write;
    std::io::stdout().flush().ok();
    let mut o = Out::to(dst, k::COPY_SEND, 10);
    o.ports = vec![(r2, k::MOVE_RECEIVE)];
    let ks2 = o.send();
    println!(
        "P10g send MOVE_RECEIVE of a STRICT-guarded receive right: kr={} (process survived)",
        hex(ks2)
    );
    let mut opts = [0u32; 16];
    let mut name = 0u32;
    let kc = unsafe { ffi::mach_port_construct(t, opts.as_mut_ptr().cast(), 0, &mut name) };
    println!(
        "P10g mach_port_construct(all-zero options)={} name type={:?}",
        hex(kc),
        ptype(name).map(|x| format!("{x:#x}")).map_err(hex)
    );
    opts[0] = 0; // flags unknown: no documented values, not scanned further here
}

/// P34: allocate receive rights until failure (cap 250 000).
fn p34() {
    let t = me();
    let t0 = now_ns();
    let mut n = 0;
    let mut err = 0;
    for _ in 0..250_000 {
        let mut x = 0;
        let kr = unsafe { ffi::mach_port_allocate(t, k::RIGHT_RECEIVE, &mut x) };
        if kr != 0 {
            err = kr;
            break;
        }
        n += 1;
    }
    println!(
        "P34 allocated {n} receive rights in {} ms; stop code {} ; names_count={}",
        (now_ns() - t0) / 1_000_000,
        hex(err),
        names_count()
    );
}

/// P38: many threads poke a qlimit-1 wake port with timeout-0 sends while an IO thread is blocked on it.
fn p38() {
    let set = alloc_set();
    let w = alloc_recv_send();
    set_qlimit(w, 1);
    unsafe { ffi::mach_port_insert_member(me(), w, set) };
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let st = stop.clone();
    let io = std::thread::spawn(move || {
        let mut n = 0u64;
        loop {
            match recv(set, Some(200)) {
                Ok(_) => n += 1,
                Err(_) => {
                    if st.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                }
            }
        }
        n
    });
    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(move || {
                let (mut ok, mut full, mut other, mut maxl) = (0u64, 0u64, 0u64, 0u64);
                for _ in 0..20000 {
                    let t0 = now_ns();
                    let kr = Out::to(w, k::COPY_SEND, 38).send_nb();
                    maxl = maxl.max(now_ns() - t0);
                    match kr {
                        0 => ok += 1,
                        0x10000004 => full += 1,
                        _ => other += 1,
                    }
                }
                (ok, full, other, maxl)
            })
        })
        .collect();
    let res: Vec<_> = threads.into_iter().map(|h| h.join().unwrap()).collect();
    std::thread::sleep(std::time::Duration::from_millis(300));
    stop.store(true, std::sync::atomic::Ordering::SeqCst);
    let got = io.join().unwrap();
    let ok: u64 = res.iter().map(|r| r.0).sum();
    let full: u64 = res.iter().map(|r| r.1).sum();
    let other: u64 = res.iter().map(|r| r.2).sum();
    let maxl = res.iter().map(|r| r.3).max().unwrap();
    println!(
        "P38 8 threads x 20000 timeout-0 sends to qlimit-1 wake port: accepted={ok} full(0x10000004)={full} other={other} max send latency {maxl} ns | IO thread received {got} (== accepted: {})",
        got == ok
    );
}

/// P25: does a profile that allows one operation name compile?
fn sbxop(op: &str) {
    let prof = format!("(version 1)\n(deny default)\n(allow sysctl-read)\n({op})\n");
    let r = child::apply_sandbox(&prof);
    println!(
        "P25op {op:60} -> {}",
        match r {
            Ok(()) => "compiles+applied".to_string(),
            Err(e) => format!("ERR {}", e.lines().next().unwrap_or("")),
        }
    );
}

/// P25: what sandbox_check says about mach operations under the empty-process profile.
fn sbxcheck() {
    extern "C" {
        fn sandbox_check(pid: i32, op: *const ffi::c_char, ty: i32, ...) -> i32;
    }
    let r = child::apply_sandbox(child::SBPL);
    println!("P25chk applied: {r:?}");
    for op in [
        "mach-lookup",
        "mach-register",
        "mach-bootstrap",
        "mach-per-user-lookup",
        "mach-priv-host-port",
        "mach-priv-task-port",
        "mach-task-name",
        "mach-kernel-endpoint",
        "syscall-mach",
        "syscall-mig",
        "mach-task-special-port-get",
        "mach-task-special-port-set",
        "iokit-open",
    ] {
        let c = std::ffi::CString::new(op).unwrap();
        let v = unsafe { sandbox_check(-1, c.as_ptr(), 0) };
        println!("P25chk sandbox_check({op}) = {v}");
    }
    // behaviour after sandboxing: lookups and special-port operations
    let mut sp = 0u32;
    let kci = unsafe {
        ffi::bootstrap_check_in(
            ffi::bootstrap_port,
            c"org.izcal.d4probe.unlisted".as_ptr(),
            &mut sp,
        )
    };
    let mut tn = 0u32;
    let ktn = unsafe { ffi::task_name_for_pid(me(), std::process::id() as i32, &mut tn) };
    let mut tp = 0u32;
    let ktp = unsafe { libc::task_for_pid(me(), 1, &mut tp) };
    let svc = unsafe {
        ffi::IOServiceGetMatchingService(
            ffi::kIOMainPortDefault,
            ffi::IOServiceMatching(c"IOSurfaceRoot".as_ptr()),
        )
    };
    let mut conn = 0u32;
    let kconn = if svc != 0 {
        unsafe { ffi::IOServiceOpen(svc, me(), 0, &mut conn) }
    } else {
        -1
    };
    let w = alloc_recv_send();
    let kset = unsafe { ffi::task_set_special_port(me(), k::TASK_BOOTSTRAP_PORT, w) };
    println!(
        "P25chk sandboxed: bootstrap_check_in={} task_name_for_pid(self)={} task_for_pid(1)={} IOServiceGetMatchingService(IOSurfaceRoot)={svc:#x} IOServiceOpen={} task_set_special_port(bootstrap)={}",
        hex(kci),
        hex(ktn),
        hex(ktp),
        hex(kconn),
        hex(kset)
    );
}

/// P25: apply deny-default + one extra rule, then run a minimal Mach sequence.
fn sbxmach(rule: &str) {
    use std::io::Write;
    let r0 = alloc_recv_send(); // set up before lockdown, like the IO thread
    let prof = format!("{}({rule})\n", child::SBPL);
    let r = child::apply_sandbox(&prof);
    println!("P25m rule ({rule}) applied: {r:?}");
    std::io::stdout().flush().ok();
    let mut n = 0u32;
    let ka = unsafe { ffi::mach_port_allocate(me(), k::RIGHT_RECEIVE, &mut n) };
    println!("P25m   mach_port_allocate={}", hex(ka));
    std::io::stdout().flush().ok();
    let ki = unsafe { ffi::mach_port_insert_right(me(), n, n, k::MAKE_SEND) };
    println!("P25m   mach_port_insert_right={}", hex(ki));
    std::io::stdout().flush().ok();
    let ks = Out::to(r0, k::COPY_SEND, 5).send();
    let kr = recv(r0, Some(100)).map(|m| m.pid());
    println!(
        "P25m   mach_msg send on held right={} receive={:?}",
        hex(ks),
        kr.map_err(hex)
    );
    std::io::stdout().flush().ok();
    let kq = set_qlimit(r0, 64);
    println!("P25m   mach_port_set_attributes={}", hex(kq));
    std::io::stdout().flush().ok();
    println!("P25m   {}", kobject(me()));
    std::io::stdout().flush().ok();
    let mut a = 0u64;
    unsafe { ffi::mach_vm_allocate(me(), &mut a, 16384, 1) };
    let mut sz = 16384u64;
    let mut h = 0u32;
    let km = unsafe { ffi::mach_make_memory_entry_64(me(), &mut sz, a, 1, &mut h, 0) };
    println!("P25m   mach_make_memory_entry_64={}", hex(km));
}

/// P30: an OOL port-array descriptor (type 2: {address u64, deallocate:8 | copy:8 | disposition:8 | type:8, count}).
fn oolports() {
    use std::io::Write;
    let dst = alloc_recv_send();
    let a = alloc_recv_send();
    let b = alloc_recv_send();
    let arr = [a, b];
    let addr = arr.as_ptr() as u64;
    let mut w: Vec<u32> = vec![
        k::MSGH_BITS_COMPLEX | k::msgh_bits(k::COPY_SEND, 0),
        0,
        dst,
        0,
        0,
        30,
        1,
        addr as u32,
        (addr >> 32) as u32,
        (k::COPY_SEND << 16) | (2 << 24),
        2,
    ];
    let sz = (w.len() * 4) as u32;
    w[1] = sz;
    println!("P30 sending an OOL port array (2 x COPY_SEND) ...");
    std::io::stdout().flush().ok();
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
    let m = recv(dst, Some(100));
    println!(
        "P30 OOL port array: send={} received descriptors={:?} words={:x?}",
        hex(kr),
        m.as_ref().map(|m| m.ools.clone()).map_err(|e| hex(*e)),
        m.as_ref().map(|m| m.raw_words.clone())
    );
    if let Ok(m) = m {
        if let Some(&(pa, cnt, _)) = m.ools.first() {
            let names: Vec<u32> = (0..cnt as usize)
                .map(|i| unsafe { *(pa as *const u32).add(i) })
                .collect();
            println!("P30 received port names {names:x?} (own {a:#x} {b:#x})");
        }
    }
}
