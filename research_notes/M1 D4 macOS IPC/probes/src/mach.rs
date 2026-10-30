//! Small helpers over the raw bindings: build and parse messages using only the
//! layouts determined black-box (see ffi::k).
use crate::ffi::{self, k, me};
use std::ptr::null_mut;

pub fn alloc_recv() -> u32 {
    let mut n = 0;
    let kr = unsafe { ffi::mach_port_allocate(me(), k::RIGHT_RECEIVE, &mut n) };
    assert_eq!(kr, 0, "mach_port_allocate");
    n
}
pub fn alloc_recv_send() -> u32 {
    let n = alloc_recv();
    assert_eq!(
        unsafe { ffi::mach_port_insert_right(me(), n, n, k::MAKE_SEND) },
        0
    );
    n
}
pub fn alloc_set() -> u32 {
    let mut n = 0;
    assert_eq!(
        unsafe { ffi::mach_port_allocate(me(), k::RIGHT_PORT_SET, &mut n) },
        0
    );
    n
}
pub fn ptype(n: u32) -> Result<u32, i32> {
    let mut t = 0;
    let kr = unsafe { ffi::mach_port_type(me(), n, &mut t) };
    if kr == 0 {
        Ok(t)
    } else {
        Err(kr)
    }
}
pub fn refs(n: u32, right: u32) -> Result<u32, i32> {
    let mut u = 0;
    let kr = unsafe { ffi::mach_port_get_refs(me(), n, right, &mut u) };
    if kr == 0 {
        Ok(u)
    } else {
        Err(kr)
    }
}
pub fn status(n: u32) -> Result<[i32; 10], i32> {
    let mut s = [0i32; 10];
    let mut c = 10u32;
    let kr = unsafe {
        ffi::mach_port_get_attributes(me(), n, k::PORT_RECEIVE_STATUS, s.as_mut_ptr(), &mut c)
    };
    if kr == 0 {
        Ok(s)
    } else {
        Err(kr)
    }
}
pub fn set_qlimit(n: u32, q: i32) -> i32 {
    let mut v = [q];
    unsafe { ffi::mach_port_set_attributes(me(), n, k::PORT_LIMITS_INFO, v.as_mut_ptr(), 1) }
}
pub fn get_qlimit(n: u32) -> Result<i32, i32> {
    let mut v = [0i32];
    let mut c = 1u32;
    let kr = unsafe {
        ffi::mach_port_get_attributes(me(), n, k::PORT_LIMITS_INFO, v.as_mut_ptr(), &mut c)
    };
    if kr == 0 {
        Ok(v[0])
    } else {
        Err(kr)
    }
}
pub fn request(n: u32, id: i32, sync: u32, notify: u32, disp: u32) -> (i32, u32) {
    let mut prev = 0;
    let kr =
        unsafe { ffi::mach_port_request_notification(me(), n, id, sync, notify, disp, &mut prev) };
    (kr, prev)
}
pub fn kobject(n: u32) -> String {
    let (mut ty, mut addr) = (0u32, 0u64);
    let kr = unsafe { ffi::mach_port_kobject(me(), n, &mut ty, &mut addr) };
    let mut desc = [0 as ffi::c_char; 512];
    let (mut ty2, mut addr2) = (0u32, 0u64);
    let kr2 = unsafe {
        ffi::mach_port_kobject_description(me(), n, &mut ty2, &mut addr2, desc.as_mut_ptr())
    };
    let d = unsafe { std::ffi::CStr::from_ptr(desc.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    format!(
        "kobject kr={kr:#x} type={ty} addr={addr:#x} | description kr={kr2:#x} type={ty2} desc={d:?}"
    )
}
pub fn audit_token_of_task(task: u32) -> Result<[u32; 8], i32> {
    let mut out = [0i32; 8];
    let mut cnt = 8u32;
    let kr = unsafe { libc::task_info(task, k::TASK_AUDIT_TOKEN, out.as_mut_ptr(), &mut cnt) };
    if kr == 0 {
        Ok(out.map(|x| x as u32))
    } else {
        Err(kr)
    }
}
pub fn names_count() -> usize {
    let (mut names, mut nc, mut types, mut tc) = (null_mut(), 0u32, null_mut(), 0u32);
    let kr = unsafe { ffi::mach_port_names(me(), &mut names, &mut nc, &mut types, &mut tc) };
    if kr != 0 {
        return usize::MAX;
    }
    unsafe {
        libc::vm_deallocate(me(), names as usize, (nc as usize) * 4);
        libc::vm_deallocate(me(), types as usize, (tc as usize) * 4);
    }
    nc as usize
}
pub fn now_ns() -> u64 {
    let mut tb = libc::mach_timebase_info { numer: 0, denom: 0 };
    unsafe { libc::mach_timebase_info(&mut tb) };
    let t = unsafe { libc::mach_absolute_time() };
    t * tb.numer as u64 / tb.denom as u64
}

/// An outgoing message.
#[derive(Default, Clone)]
pub struct Out {
    pub dest: u32,
    pub dest_disp: u32,
    pub reply: u32,
    pub reply_disp: u32,
    pub id: i32,
    pub ports: Vec<(u32, u32)>,
    /// (address, size, deallocate, copy)
    pub ools: Vec<(u64, u32, bool, u8)>,
    pub body: Vec<u8>,
    /// forced value for msgh_size (default: computed)
    pub size_override: Option<u32>,
    /// extra bytes placed after the message in the buffer (not counted in send_size)
    pub after: Vec<u8>,
    /// extra bytes counted in send_size but not in msgh_size
    pub extra_send: Vec<u8>,
}
impl Out {
    pub fn to(dest: u32, disp: u32, id: i32) -> Out {
        Out {
            dest,
            dest_disp: disp,
            id,
            ..Default::default()
        }
    }
    pub fn text(mut self, s: &str) -> Out {
        self.body = s.as_bytes().to_vec();
        self
    }
    pub fn encode(&self) -> (Vec<u64>, u32) {
        let mut w: Vec<u32> = vec![0; 6];
        let complex = !self.ports.is_empty() || !self.ools.is_empty();
        w[0] = k::msgh_bits(self.dest_disp, self.reply_disp)
            | if complex { k::MSGH_BITS_COMPLEX } else { 0 };
        w[2] = self.dest;
        w[3] = self.reply;
        w[5] = self.id as u32;
        if complex {
            w.push((self.ports.len() + self.ools.len()) as u32);
            for &(name, disp) in &self.ports {
                w.extend_from_slice(&[name, 0, (disp << 16) | (k::PORT_DESCRIPTOR << 24)]);
            }
            for &(addr, size, dealloc, copy) in &self.ools {
                w.extend_from_slice(&[
                    addr as u32,
                    (addr >> 32) as u32,
                    (dealloc as u32) | ((copy as u32) << 8) | (k::OOL_DESCRIPTOR << 24),
                    size,
                ]);
            }
        }
        let mut bytes: Vec<u8> = w.iter().flat_map(|x| x.to_ne_bytes()).collect();
        bytes.extend_from_slice(&self.body);
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }
        let msgh_size = bytes.len() as u32;
        bytes.extend_from_slice(&self.extra_send);
        let send_size = bytes.len() as u32;
        bytes.extend_from_slice(&self.after);
        let sz = self.size_override.unwrap_or(msgh_size);
        bytes[4..8].copy_from_slice(&sz.to_ne_bytes());
        let mut buf = vec![0u64; bytes.len() / 8 + 2];
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                buf.as_mut_ptr().cast::<u8>(),
                bytes.len(),
            )
        };
        (buf, send_size)
    }
    /// Send with extra options and timeout. Returns (kr, buffer after the call — pseudo-receive may rewrite it).
    pub fn send_opts(&self, opts: i32, timeout: u32) -> (i32, Vec<u64>) {
        let (mut buf, sz) = self.encode();
        let kr = unsafe {
            ffi::mach_msg_overwrite(
                buf.as_mut_ptr().cast(),
                k::SEND_MSG | opts,
                sz,
                0,
                0,
                timeout,
                0,
                null_mut(),
                0,
            )
        };
        (kr, buf)
    }
    pub fn send(&self) -> i32 {
        self.send_opts(0, 0).0
    }
    /// Non-blocking send (timeout 0).
    pub fn send_nb(&self) -> i32 {
        self.send_opts(k::SEND_TIMEOUT, 0).0
    }
}

#[derive(Debug, Clone, Default)]
pub struct Rcvd {
    pub bits: u32,
    pub size: u32,
    pub remote: u32,
    pub local: u32,
    pub id: i32,
    /// (name, received disposition)
    pub ports: Vec<(u32, u32)>,
    /// (address, size, raw flags word)
    pub ools: Vec<(u64, u32, u32)>,
    pub body: Vec<u8>,
    pub trailer: Vec<u32>,
    pub raw_words: Vec<u32>,
}
impl Rcvd {
    pub fn audit(&self) -> Option<[u32; 8]> {
        if self.trailer.len() >= 13 {
            let mut a = [0; 8];
            a.copy_from_slice(&self.trailer[5..13]);
            Some(a)
        } else {
            None
        }
    }
    pub fn pid(&self) -> i32 {
        self.audit().map(|a| a[k::AT_PID] as i32).unwrap_or(-1)
    }
    pub fn pidver(&self) -> u32 {
        self.audit().map(|a| a[k::AT_PIDVERSION]).unwrap_or(0)
    }
    pub fn seqno(&self) -> u32 {
        self.trailer.get(2).copied().unwrap_or(u32::MAX)
    }
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body)
            .trim_end_matches('\0')
            .to_string()
    }
    pub fn remote_disp(&self) -> u32 {
        self.bits & 0x1f
    }
    pub fn local_disp(&self) -> u32 {
        (self.bits >> 8) & 0x1f
    }
}

/// Receive with the audit trailer. timeout_ms None = block forever.
pub fn recv_opts(
    port: u32,
    timeout_ms: Option<u32>,
    bufsize: usize,
    extra: i32,
) -> Result<Rcvd, i32> {
    let mut buf = vec![0u64; bufsize / 8 + 1];
    let mut opts = k::RCV_MSG | k::rcv_trailer_elements(k::AUDIT_ELEMENTS) | extra;
    if timeout_ms.is_some() {
        opts |= k::RCV_TIMEOUT;
    }
    let kr = unsafe {
        ffi::mach_msg_overwrite(
            buf.as_mut_ptr().cast(),
            opts,
            0,
            bufsize as u32,
            port,
            timeout_ms.unwrap_or(0),
            0,
            null_mut(),
            0,
        )
    };
    if kr != 0 {
        return Err(kr);
    }
    Ok(parse(&buf))
}
pub fn recv(port: u32, timeout_ms: Option<u32>) -> Result<Rcvd, i32> {
    recv_opts(port, timeout_ms, 256 * 1024, 0)
}

pub fn parse(buf: &[u64]) -> Rcvd {
    let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), buf.len() * 8) };
    let w = |i: usize| u32::from_ne_bytes(bytes[i..i + 4].try_into().unwrap());
    let mut r = Rcvd {
        bits: w(0),
        size: w(4),
        remote: w(8),
        local: w(12),
        id: w(20) as i32,
        ..Default::default()
    };
    let size = r.size as usize;
    let mut off = 24;
    if r.bits & k::MSGH_BITS_COMPLEX != 0 {
        let n = w(24) as usize;
        off = 28;
        for _ in 0..n {
            let ty = w(off + 8) >> 24;
            if ty == k::PORT_DESCRIPTOR {
                r.ports.push((w(off), (w(off + 8) >> 16) & 0xff));
                off += 12;
            } else {
                let addr = w(off) as u64 | ((w(off + 4) as u64) << 32);
                r.ools.push((addr, w(off + 12), w(off + 8)));
                off += 16;
            }
        }
    }
    if size >= off && size <= bytes.len() {
        r.body = bytes[off..size].to_vec();
    }
    let toff = (size + 3) & !3;
    if toff + 8 <= bytes.len() {
        let tsize = w(toff + 4) as usize;
        r.trailer = (0..tsize / 4).map(|i| w(toff + i * 4)).collect();
    }
    r.raw_words = (0..(size / 4).min(16)).map(|i| w(i * 4)).collect();
    r
}

/// Release every right and OOL region a received message carries (our own cleanup; P22).
pub fn release(r: &Rcvd) {
    for &(name, disp) in &r.ports {
        if name == 0 || name == u32::MAX {
            continue;
        }
        unsafe {
            if disp == k::PORT_RECEIVE {
                ffi::mach_port_mod_refs(me(), name, k::RIGHT_RECEIVE, -1);
            } else {
                ffi::mach_port_deallocate(me(), name);
            }
        }
    }
    for &(addr, size, _) in &r.ools {
        if size > 0 {
            unsafe { ffi::mach_vm_deallocate(me(), addr, size as u64) };
        }
    }
    if r.remote != 0 {
        unsafe { ffi::mach_port_deallocate(me(), r.remote) };
    }
}

pub fn drain(port: u32) -> Vec<Rcvd> {
    let mut v = vec![];
    while let Ok(m) = recv(port, Some(0)) {
        v.push(m);
    }
    v
}

pub fn hex(kr: i32) -> String {
    format!("{:#x}", kr as u32)
}
