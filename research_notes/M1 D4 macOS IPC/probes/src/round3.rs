//! Round 3 (D4 code review): receive-status accounting, notification
//! sizes, oversize messages on a set, and descriptor types the kernel
//! delivers. Same rules: black-box only, constants from ffi::k.
use crate::ffi::{self, k, me};
use crate::mach::*;

fn hx(kr: i32) -> String {
    hex(kr)
}

/// Status words 4 (msgcount), 5 (sorights), 6 (srights), 7 (pdrequest),
/// 8 (nsrequest).
fn st(p: u32) -> String {
    match status(p) {
        Ok(s) => format!(
            "msgcount={} sorights={} srights={} pdrequest={} nsrequest={}",
            s[4], s[5], s[6], s[7], s[8]
        ),
        Err(e) => format!("status failed {}", hx(e)),
    }
}

/// Moves `p`'s receive right to ourselves through carrier port `c` and
/// returns the name it arrives under.
fn move_receive_through(p: u32, c: u32) -> Option<u32> {
    let mut o = Out::to(c, k::MAKE_SEND, 7);
    o.ports = vec![(p, k::MOVE_RECEIVE)];
    let kr = o.send();
    if kr != 0 {
        println!("  move failed {}", hx(kr));
        return None;
    }
    let m = recv(c, Some(200)).ok()?;
    m.ports.first().map(|&(n, _)| n)
}

fn make_send_once(p: u32) -> u32 {
    let mut so = 0u32;
    let mut t = 0u32;
    let kr = unsafe { ffi::mach_port_extract_right(me(), p, k::MAKE_SEND_ONCE, &mut so, &mut t) };
    assert_eq!(kr, 0, "make send-once");
    so
}

/// R8: what sorights and pdrequest count, and whether they travel with
/// a moved receive right.
pub fn r8() {
    let c = alloc_recv();
    // (a) no-senders aimed at the port itself.
    let d = alloc_recv_send();
    println!("R8a fresh port: {}", st(d));
    let (kr, prev) = request(d, k::NOTIFY_NO_SENDERS, 1, d, k::MAKE_SEND_ONCE);
    println!(
        "R8a armed no-senders on itself (kr={} prev={prev:#x}): {}",
        hx(kr),
        st(d)
    );
    let (kr, prev) = request(d, k::NOTIFY_NO_SENDERS, 0, 0, k::MAKE_SEND_ONCE);
    println!(
        "R8a cancelled (kr={} returned={prev:#x} type={:?}): {}",
        hx(kr),
        ptype(prev).map(|t| format!("{t:#x}")),
        st(d)
    );
    unsafe { ffi::mach_port_deallocate(me(), prev) };
    println!("R8a returned right destroyed: {}", st(d));
    let msgs: Vec<(i32, u32, i32)> = drain(d).iter().map(|m| (m.id, m.size, m.pid())).collect();
    println!(
        "R8a queued on the port after: (id, size, pid) {msgs:?}; now {}",
        st(d)
    );
    // (b) send-once rights made by the receive holder, moved with the port.
    let e = alloc_recv();
    let s1 = make_send_once(e);
    let s2 = make_send_once(e);
    let s3 = make_send_once(e);
    println!("R8b three send-once rights made: {}", st(e));
    if let Some(e2) = move_receive_through(e, c) {
        println!(
            "R8b after a move to ourselves (name {e:#x} -> {e2:#x}): {}",
            st(e2)
        );
        unsafe { ffi::mach_port_deallocate(me(), s1) };
        println!("R8b one send-once destroyed: {}", st(e2));
        let _ = (s2, s3);
    }
    // (c) send-once rights in queued messages count too.
    let f = alloc_recv();
    let g = alloc_recv_send();
    for i in 0..3 {
        let mut o = Out::to(g, k::COPY_SEND, 200 + i);
        o.reply = f;
        o.reply_disp = k::MAKE_SEND_ONCE;
        assert_eq!(o.send(), 0);
    }
    println!(
        "R8c three messages on another port carry reply rights to F: F {}",
        st(f)
    );
    // (d) port-destroyed request on a moved port.
    let h = alloc_recv();
    let q = alloc_recv();
    let (kr, prev) = request(h, k::NOTIFY_PORT_DESTROYED, 0, q, k::MAKE_SEND_ONCE);
    println!(
        "R8d port-destroyed requested (kr={} prev={prev:#x}): H {}",
        hx(kr),
        st(h)
    );
    if let Some(h2) = move_receive_through(h, c) {
        println!("R8d after a move: {}", st(h2));
        let kr = unsafe { ffi::mach_port_mod_refs(me(), h2, k::RIGHT_RECEIVE, -1) };
        match recv(q, Some(200)) {
            Ok(m) => println!(
                "R8d destroyed (kr={}): Q got id={} size={} complex={} pid={} ports={:?}",
                hx(kr),
                m.id,
                m.size,
                m.bits & k::MSGH_BITS_COMPLEX != 0,
                m.pid(),
                m.ports
            ),
            Err(e) => println!("R8d destroyed (kr={}): Q got nothing {}", hx(kr), hx(e)),
        }
    }
}

/// R9: the sizes of the notifications the transport acts on.
pub fn r9() {
    // No-senders (70) aimed at the port itself.
    let d = alloc_recv_send();
    let (kr, _) = request(d, k::NOTIFY_NO_SENDERS, 1, d, k::MAKE_SEND_ONCE);
    assert_eq!(kr, 0);
    unsafe { ffi::mach_port_deallocate(me(), d) };
    if let Ok(m) = recv(d, Some(200)) {
        println!(
            "R9 no-senders: id={} size={} bits={:#x} complex={} pid={} body={:x?}",
            m.id,
            m.size,
            m.bits,
            m.bits & k::MSGH_BITS_COMPLEX != 0,
            m.pid(),
            m.body
        );
    }
    // Send-possible (66): fill a qlimit-1 queue, fail a send with 0x80,
    // arm, dequeue.
    let p = alloc_recv_send();
    let n = alloc_recv();
    set_qlimit(p, 1);
    assert_eq!(Out::to(p, k::COPY_SEND, 1).send_nb(), 0);
    let (kr_full, _) = Out::to(p, k::COPY_SEND, 2).send_opts(k::SEND_TIMEOUT | 0x80, 0);
    let (kr, _) = request(p, k::NOTIFY_SEND_POSSIBLE, 0, n, k::MAKE_SEND_ONCE);
    let _ = recv(p, Some(100));
    match recv(n, Some(300)) {
        Ok(m) => println!(
            "R9 send-possible (full send kr={}, arm kr={}): id={} size={} bits={:#x} complex={} pid={} body={:x?}",
            hx(kr_full),
            hx(kr),
            m.id,
            m.size,
            m.bits,
            m.bits & k::MSGH_BITS_COMPLEX != 0,
            m.pid(),
            m.body
        ),
        Err(e) => println!("R9 send-possible: nothing ({})", hx(e)),
    }
    // Send-once (71): a send-once right destroyed unused.
    let s = alloc_recv();
    let so = make_send_once(s);
    unsafe { ffi::mach_port_deallocate(me(), so) };
    if let Ok(m) = recv(s, Some(200)) {
        println!("R9 send-once: id={} size={} pid={}", m.id, m.size, m.pid());
    }
    // Cancelling a send-possible request.
    let (kr2, prev) = request(p, k::NOTIFY_SEND_POSSIBLE, 0, 0, k::MAKE_SEND_ONCE);
    println!(
        "R9 cancel send-possible with nothing armed: kr={} prev={prev:#x}",
        hx(kr2)
    );
    assert_eq!(Out::to(p, k::COPY_SEND, 3).send_nb(), 0);
    let (kr_full, _) = Out::to(p, k::COPY_SEND, 4).send_opts(k::SEND_TIMEOUT | 0x80, 0);
    let (kr3, _) = request(p, k::NOTIFY_SEND_POSSIBLE, 0, n, k::MAKE_SEND_ONCE);
    let (kr4, prev) = request(p, k::NOTIFY_SEND_POSSIBLE, 0, 0, k::MAKE_SEND_ONCE);
    println!(
        "R9 armed (full kr={}, arm kr={}) then cancelled: kr={} prev={prev:#x} type={:?}",
        hx(kr_full),
        hx(kr3),
        hx(kr4),
        ptype(prev).map(|t| format!("{t:#x}"))
    );
    if prev != 0 {
        unsafe { ffi::mach_port_deallocate(me(), prev) };
    }
    let _ = recv(p, Some(100));
    let after: Vec<(i32, u32)> = drain(n).iter().map(|m| (m.id, m.size)).collect();
    println!("R9 after the cancel and a dequeue, the notify port holds (id, size) {after:?}");
}

/// R10: an oversize message on a port set, received WITHOUT RCV_LARGE.
pub fn r10() {
    for trial in 0..3 {
        let set = alloc_set();
        let p = alloc_recv_send();
        assert_eq!(unsafe { ffi::mach_port_insert_member(me(), p, set) }, 0);
        let q = alloc_recv_send();
        let n = alloc_recv();
        let (kr, _) = request(q, k::NOTIFY_NO_SENDERS, 1, n, k::MAKE_SEND_ONCE);
        assert_eq!(kr, 0);
        let mut o = Out::to(p, k::COPY_SEND, 4343);
        o.body = vec![7u8; 1000];
        o.ports = vec![(q, k::MOVE_SEND)];
        assert_eq!(o.send(), 0);
        let mut buf = [0u64; 32];
        let kr = unsafe {
            ffi::mach_msg_overwrite(
                buf.as_mut_ptr().cast(),
                k::RCV_MSG | k::RCV_TIMEOUT | k::rcv_trailer_elements(k::AUDIT_ELEMENTS),
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
        let ns = recv(n, Some(200));
        println!(
            "R10 trial {trial}: kr={} header [bits={:#x} size={} remote={:#x} local={:#x} id={}] (member p={p:#x}) words 6..20={:x?} msgcount(p) after={} carried right released={}",
            hx(kr),
            w[0],
            w[1],
            w[2],
            w[3],
            w[5] as i32,
            &w[6..20],
            status(p).map(|s| s[4]).unwrap_or(-1),
            matches!(&ns, Ok(m) if m.id == k::NOTIFY_NO_SENDERS)
        );
    }
}

/// R11: descriptor types beyond port (0), OOL (1) and OOL ports (2):
/// which the kernel accepts from a sender, and what the receiver sees.
pub fn r11() {
    let p = alloc_recv_send();
    let page = vec![9u8; 4096];
    for ty in 3u32..=6 {
        for len in [12usize, 16] {
            let carried = alloc_recv_send();
            // Hand-built: header, count 1, one descriptor of `len` bytes,
            // 4 body bytes. For 16 bytes, an OOL-shaped descriptor
            // (address, flags with the type, size); for 12, a
            // port-shaped one (name, pad, flags with the type).
            let mut w: Vec<u32> = vec![
                k::msgh_bits(k::COPY_SEND, 0) | k::MSGH_BITS_COMPLEX,
                0,
                p,
                0,
                0,
                5000 + ty,
                1,
            ];
            if len == 12 {
                w.extend_from_slice(&[carried, 0, (k::MOVE_SEND << 16) | (ty << 24)]);
            } else {
                let addr = page.as_ptr() as u64;
                w.extend_from_slice(&[addr as u32, (addr >> 32) as u32, ty << 24, 4096]);
            }
            w.push(0xABCD);
            let size = (w.len() * 4) as u32;
            w[1] = size;
            let mut buf: Vec<u64> = vec![0; w.len() / 2 + 2];
            unsafe {
                std::ptr::copy_nonoverlapping(w.as_ptr(), buf.as_mut_ptr().cast::<u32>(), w.len())
            };
            let kr = unsafe {
                ffi::mach_msg_overwrite(
                    buf.as_mut_ptr().cast(),
                    k::SEND_MSG | k::SEND_TIMEOUT,
                    size,
                    0,
                    0,
                    0,
                    0,
                    std::ptr::null_mut(),
                    0,
                )
            };
            let got = if kr == 0 {
                match recv(p, Some(100)) {
                    Ok(m) => format!("received size={} words={:x?}", m.size, m.raw_words),
                    Err(e) => format!("nothing ({})", hx(e)),
                }
            } else {
                "not sent".into()
            };
            println!("R11 type {ty} as {len} bytes: send kr={} {got}", hx(kr));
        }
    }
}
