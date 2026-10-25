//! The Mach calls behind the macOS IPC transport, as a small safe
//! interface. Signatures are from Apple's published documentation pages
//! (declarations only) and the SDK man pages; every constant value is in
//! `ipc::mach::wire`, found black-box (`research_notes/M1 D4 macOS
//! IPC/mach_facts.md`). No header, no kernel source and no other
//! project's code is a source.
//!
//! This is the only `unsafe` in the transport (CS-TY-13,
//! `unsafe-inventory.toml`, entry `ipc::macos::sys`), and the only code
//! that sends or receives Mach messages carrying rights (security scope
//! SEC-4 (d)): the foreign functions are declared privately here, so no
//! other module can call them. It moves bytes and rights and reports raw
//! kernel facts; deciding what to accept is the pure code in
//! `ipc::mach`.
//!
//! Every send passes [`SEND_OPTIONS`] and every receive
//! [`RCV_OPTIONS`] or a subset: an unknown option bit gets the sending
//! process killed (P2).
#![expect(unsafe_code, reason = "G5: ipc::macos::sys (Mach message FFI)")]

use core::ffi::{c_char, c_int, c_void};

use crate::ipc::mach::accept::SlotFacts;
use crate::ipc::mach::wire::{
    self, ATTR_LIMITS, ATTR_RECEIVE_STATUS, Desc, MAKE_SEND, MAKE_SEND_ONCE, NAME_DEAD, NAME_NULL,
    NOTIFY_NO_SENDERS, NOTIFY_SEND_POSSIBLE, OPT_RCV, OPT_RCV_TIMEOUT, OPT_TRAILER_AUDIT,
    PORT_RECEIVE, PORT_SEND, PORT_SEND_ONCE, QLIMIT_MAX, RCV_OPTIONS, RECEIVE_STATUS_WORDS,
    RIGHT_PORT_SET, RIGHT_RECEIVE, RIGHT_SEND, SEND_OPTIONS, TASK_AUDIT_TOKEN, TASK_BOOTSTRAP_PORT,
    Token,
};

type KernReturn = c_int;
type PortName = u32;

// Declarations from developer.apple.com/documentation/kernel (the
// `mach_port_*` and `mach_msg_overwrite` pages), the OSF Mach kernel
// interface pages (`task_get_special_port`) and the SDK man page
// posix_spawnattr_setspecialport_np(3). `mach_msg` itself has no page;
// `mach_msg_overwrite` with a null receive buffer is the documented form.
unsafe extern "C" {
    fn mach_task_self() -> PortName;
    #[cfg(test)]
    fn mach_host_self() -> PortName;
    fn mach_port_allocate(task: PortName, right: u32, name: *mut PortName) -> KernReturn;
    fn mach_port_deallocate(task: PortName, name: PortName) -> KernReturn;
    fn mach_port_mod_refs(task: PortName, name: PortName, right: u32, delta: i32) -> KernReturn;
    fn mach_port_insert_right(
        task: PortName,
        name: PortName,
        poly: PortName,
        poly_poly: u32,
    ) -> KernReturn;
    fn mach_port_insert_member(task: PortName, name: PortName, pset: PortName) -> KernReturn;
    fn mach_port_extract_member(task: PortName, name: PortName, pset: PortName) -> KernReturn;
    fn mach_port_type(task: PortName, name: PortName, ptype: *mut u32) -> KernReturn;
    fn mach_port_get_refs(task: PortName, name: PortName, right: u32, refs: *mut u32)
    -> KernReturn;
    fn mach_port_get_attributes(
        task: PortName,
        name: PortName,
        flavor: c_int,
        info: *mut i32,
        count: *mut u32,
    ) -> KernReturn;
    fn mach_port_set_attributes(
        task: PortName,
        name: PortName,
        flavor: c_int,
        info: *mut i32,
        count: u32,
    ) -> KernReturn;
    fn mach_port_request_notification(
        task: PortName,
        name: PortName,
        msgid: i32,
        sync: u32,
        notify: PortName,
        notify_poly: u32,
        previous: *mut PortName,
    ) -> KernReturn;
    fn mach_port_kobject(
        task: PortName,
        name: PortName,
        object_type: *mut u32,
        object_addr: *mut u64,
    ) -> KernReturn;
    fn mach_msg_overwrite(
        msg: *mut c_void,
        option: i32,
        send_size: u32,
        rcv_size: u32,
        rcv_name: PortName,
        timeout: u32,
        notify: PortName,
        rcv_msg: *mut c_void,
        rcv_limit: u32,
    ) -> KernReturn;
    fn mach_vm_deallocate(task: PortName, address: u64, size: u64) -> KernReturn;
    fn task_get_special_port(task: PortName, which: c_int, port: *mut PortName) -> KernReturn;
    #[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
    fn task_name_for_pid(target: PortName, pid: c_int, name: *mut PortName) -> KernReturn;
    #[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
    fn posix_spawnattr_setspecialport_np(
        attr: *mut libc::posix_spawnattr_t,
        port: PortName,
        which: c_int,
    ) -> c_int;
}

/// This task's own port name. Stable for the life of the process.
fn task() -> PortName {
    // SAFETY: takes no arguments and returns the caller's task port name.
    unsafe { mach_task_self() }
}

/// A kernel return code that is not success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Kr(pub(crate) i32);

fn kr(rc: KernReturn) -> Result<(), Kr> {
    if rc == 0 { Ok(()) } else { Err(Kr(rc)) }
}

// ---- rights ----

/// A new receive right.
pub(crate) fn allocate_receive() -> Result<PortName, Kr> {
    let mut name: PortName = NAME_NULL;
    // SAFETY: `name` is a valid out-pointer for the new name.
    kr(unsafe { mach_port_allocate(task(), RIGHT_RECEIVE, &raw mut name) })?;
    Ok(name)
}

/// A new port set.
pub(crate) fn allocate_set() -> Result<PortName, Kr> {
    let mut name: PortName = NAME_NULL;
    // SAFETY: `name` is a valid out-pointer for the new name.
    kr(unsafe { mach_port_allocate(task(), RIGHT_PORT_SET, &raw mut name) })?;
    Ok(name)
}

/// Makes a send right to `name`'s port under the same name (the kernel
/// gives one port one name per task).
pub(crate) fn make_send(name: PortName) -> Result<(), Kr> {
    // SAFETY: plain values; the kernel checks `name` holds a receive right.
    kr(unsafe { mach_port_insert_right(task(), name, name, u32::from(MAKE_SEND)) })
}

/// Drops one send (or send-once, or dead-name) reference.
pub(crate) fn deallocate(name: PortName) {
    if name == NAME_NULL || name == NAME_DEAD {
        return;
    }
    // SAFETY: plain values; a name this process does not hold is refused.
    let _ = unsafe { mach_port_deallocate(task(), name) };
}

/// Destroys the receive right: queued messages and their rights are
/// released by the kernel, senders see a dead name.
pub(crate) fn destroy_receive(name: PortName) {
    if name == NAME_NULL || name == NAME_DEAD {
        return;
    }
    // SAFETY: plain values; a name without a receive right is refused.
    let _ = unsafe { mach_port_mod_refs(task(), name, RIGHT_RECEIVE, -1) };
}

/// Destroys a port set.
pub(crate) fn destroy_set(name: PortName) {
    // SAFETY: plain values; a name without a port set is refused.
    let _ = unsafe { mach_port_mod_refs(task(), name, RIGHT_PORT_SET, -1) };
}

/// Adds `name` to `set`.
pub(crate) fn insert_member(name: PortName, set: PortName) -> Result<(), Kr> {
    // SAFETY: plain values.
    kr(unsafe { mach_port_insert_member(task(), name, set) })
}

/// Takes `name` out of `set`.
pub(crate) fn extract_member(name: PortName, set: PortName) {
    // SAFETY: plain values.
    let _ = unsafe { mach_port_extract_member(task(), name, set) };
}

/// Sets a receive right's queue limit, capped at the kernel's maximum
/// (P16).
pub(crate) fn set_qlimit(name: PortName, limit: u32) -> Result<(), Kr> {
    let mut v = [i32::try_from(limit.min(QLIMIT_MAX)).unwrap_or(0)];
    // SAFETY: `v` holds the one word the limits flavor takes (P16).
    kr(unsafe { mach_port_set_attributes(task(), name, ATTR_LIMITS, v.as_mut_ptr(), 1) })
}

/// Aims `name`'s no-senders notification at the port itself, firing
/// once its make-send count reaches `sync` and no send right is left
/// (P20). Any request it carried is replaced; the right that request
/// held (a send-once right) is released.
pub(crate) fn request_no_senders(name: PortName, sync: u32) -> Result<(), Kr> {
    let mut previous: PortName = NAME_NULL;
    // SAFETY: plain values and a valid out-pointer for the previous
    // notify right.
    kr(unsafe {
        mach_port_request_notification(
            task(),
            name,
            i32::try_from(NOTIFY_NO_SENDERS).unwrap_or(0),
            sync,
            name,
            u32::from(MAKE_SEND_ONCE),
            &raw mut previous,
        )
    })?;
    deallocate(previous);
    Ok(())
}

/// Arms the send-possible request on send right `name`, notifying
/// `notify` (P19).
pub(crate) fn request_send_possible(name: PortName, notify: PortName) {
    let mut previous: PortName = NAME_NULL;
    // SAFETY: plain values and a valid out-pointer for the previous
    // notify right.
    let rc = unsafe {
        mach_port_request_notification(
            task(),
            name,
            i32::try_from(NOTIFY_SEND_POSSIBLE).unwrap_or(0),
            0,
            notify,
            u32::from(MAKE_SEND_ONCE),
            &raw mut previous,
        )
    };
    if rc == 0 {
        deallocate(previous);
    }
}

/// What the kernel says about one received right, for acceptance.
pub(crate) fn slot_facts(desc: Desc) -> SlotFacts {
    let Desc::Port { name, disposition } = desc else {
        return SlotFacts {
            desc_type: wire::DESC_OOL,
            name: NAME_NULL,
            disposition: 0,
            type_bits: 0,
            send_refs: 0,
            kobject: 0,
            plain_queue: false,
        };
    };
    let mut type_bits = 0u32;
    // SAFETY: a valid out-pointer.
    if unsafe { mach_port_type(task(), name, &raw mut type_bits) } != 0 {
        type_bits = 0;
    }
    let mut send_refs = 0u32;
    // SAFETY: a valid out-pointer.
    if unsafe { mach_port_get_refs(task(), name, RIGHT_SEND, &raw mut send_refs) } != 0 {
        send_refs = 0;
    }
    let mut kobject = 0u32;
    if disposition == PORT_SEND {
        let mut addr = 0u64;
        // SAFETY: valid out-pointers.
        if unsafe { mach_port_kobject(task(), name, &raw mut kobject, &raw mut addr) } != 0 {
            kobject = 0;
        }
    }
    let plain_queue = disposition == PORT_RECEIVE && receive_status(name).is_some();
    SlotFacts {
        desc_type: wire::DESC_PORT,
        name,
        disposition,
        type_bits,
        send_refs,
        kobject,
        plain_queue,
    }
}

/// A receive right's status words (P10), if it is one.
pub(crate) fn receive_status(name: PortName) -> Option<[i32; RECEIVE_STATUS_WORDS]> {
    let mut s = [0i32; RECEIVE_STATUS_WORDS];
    let mut count = u32::try_from(RECEIVE_STATUS_WORDS).unwrap_or(0);
    // SAFETY: `s` has room for `count` words.
    let rc = unsafe {
        mach_port_get_attributes(
            task(),
            name,
            ATTR_RECEIVE_STATUS,
            s.as_mut_ptr(),
            &raw mut count,
        )
    };
    (rc == 0).then_some(s)
}

// ---- messages ----

/// A receive buffer, 8-byte aligned as the kernel writes 64-bit
/// addresses into out-of-line descriptors.
pub(crate) struct Buf {
    words: Vec<u64>,
}

impl Buf {
    /// Room for `bytes` bytes.
    pub(crate) fn new(bytes: usize) -> Self {
        Self {
            words: vec![0u64; bytes.div_ceil(8)],
        }
    }

    /// The bytes.
    pub(crate) fn bytes(&self) -> &[u8] {
        let len = self.words.len().saturating_mul(8);
        // SAFETY: the words are initialised, u8 has no alignment, and the
        // slice covers exactly the vector's storage for its lifetime.
        unsafe { core::slice::from_raw_parts(self.words.as_ptr().cast::<u8>(), len) }
    }

    /// The bytes, to write a message into.
    pub(crate) fn bytes_mut(&mut self) -> &mut [u8] {
        let len = self.words.len().saturating_mul(8);
        // SAFETY: as `bytes`, with exclusive access through `&mut self`.
        unsafe { core::slice::from_raw_parts_mut(self.words.as_mut_ptr().cast::<u8>(), len) }
    }

    fn ptr(&mut self) -> *mut c_void {
        self.words.as_mut_ptr().cast()
    }

    fn len32(&self) -> u32 {
        u32::try_from(self.words.len().saturating_mul(8)).unwrap_or(u32::MAX)
    }
}

/// Sends the message in `buf` (`size` bytes) without waiting. On a full
/// queue ([`wire::SEND_TIMED_OUT`]) the kernel hands every carried right
/// back into `buf`, a moved receive right under a new name (P17), so a
/// retry sends `buf` as it now stands.
pub(crate) fn send(buf: &mut Buf, size: usize) -> Result<(), Kr> {
    let Ok(size) = u32::try_from(size) else {
        return Err(Kr(wire::SEND_INVALID_DEST));
    };
    let ptr = buf.ptr();
    // SAFETY: `buf` holds a message of `size` bytes built by
    // `wire::encode`; no receive is requested.
    kr(unsafe {
        mach_msg_overwrite(
            ptr,
            SEND_OPTIONS,
            size,
            0,
            NAME_NULL,
            0,
            NAME_NULL,
            core::ptr::null_mut(),
            0,
        )
    })
}

/// Receives one message from `port` (a port or a port set) into `buf`,
/// waiting with no timeout. A message too large stays queued and
/// [`wire::RCV_TOO_LARGE`] is returned, its size in the header (P24).
pub(crate) fn receive(port: PortName, buf: &mut Buf) -> Result<(), Kr> {
    let len = buf.len32();
    let ptr = buf.ptr();
    // SAFETY: `buf` has room for `len` bytes, which the kernel never
    // exceeds; no send is requested.
    kr(unsafe {
        mach_msg_overwrite(
            ptr,
            RCV_OPTIONS,
            0,
            len,
            port,
            0,
            NAME_NULL,
            core::ptr::null_mut(),
            0,
        )
    })
}

/// Receives one message from `port` if one is queued, without waiting.
/// A message too large for `buf` is destroyed by the kernel, its rights
/// released (P24).
pub(crate) fn receive_now(port: PortName, buf: &mut Buf) -> Result<(), Kr> {
    let len = buf.len32();
    let ptr = buf.ptr();
    // SAFETY: as `receive`, with a zero timeout and without RCV_LARGE.
    kr(unsafe {
        mach_msg_overwrite(
            ptr,
            OPT_RCV | OPT_RCV_TIMEOUT | OPT_TRAILER_AUDIT,
            0,
            len,
            port,
            0,
            NAME_NULL,
            core::ptr::null_mut(),
            0,
        )
    })
}

/// Receives one message from `port`, waiting up to `ms` milliseconds.
pub(crate) fn receive_within(port: PortName, buf: &mut Buf, ms: u32) -> Result<(), Kr> {
    let len = buf.len32();
    let ptr = buf.ptr();
    // SAFETY: as `receive_now`, with a timeout.
    kr(unsafe {
        mach_msg_overwrite(
            ptr,
            OPT_RCV | OPT_RCV_TIMEOUT | OPT_TRAILER_AUDIT,
            0,
            len,
            port,
            ms,
            NAME_NULL,
            core::ptr::null_mut(),
            0,
        )
    })
}

/// Releases every right and region a received message carries, and its
/// reply and voucher rights, as far as its layout reads.
pub(crate) fn release_received(bytes: &[u8]) {
    let Ok(l) = wire::layout(bytes) else {
        return;
    };
    deallocate(l.header.remote);
    deallocate(l.header.voucher);
    for d in wire::descs(bytes, &l) {
        release_desc(d);
    }
}

fn release_desc(d: Desc) {
    match d {
        Desc::Port { name, disposition } => release_right(name, disposition),
        Desc::Ool { address, size } => {
            if size > 0 {
                // SAFETY: the kernel mapped this region into this task
                // for the received message; nothing else refers to it.
                let _ = unsafe { mach_vm_deallocate(task(), address, u64::from(size)) };
            }
        }
        Desc::OolPorts {
            address,
            count,
            disposition,
        } => {
            let n = usize::try_from(count).unwrap_or(0);
            if n > 0 && address != 0 {
                // SAFETY: the kernel mapped an array of `count` port names
                // at `address` for the received message.
                let names = unsafe { core::slice::from_raw_parts(address as *const u32, n) };
                for &name in names {
                    release_right(name, disposition);
                }
                let bytes = u64::from(count).saturating_mul(4);
                // SAFETY: the array's region, mapped by the kernel for this
                // message and read above; nothing else refers to it.
                let _ = unsafe { mach_vm_deallocate(task(), address, bytes) };
            }
        }
    }
}

/// Releases one right by its disposition: as received (16 receive,
/// 17 send, 18 send-once, P12), and equally in our own unsent message,
/// where a moved receive right reads 16 and a moved send right 17; a
/// make-disposition creates nothing until sent.
pub(crate) fn release_right(name: PortName, disposition: u8) {
    match disposition {
        PORT_RECEIVE => destroy_receive(name),
        PORT_SEND | PORT_SEND_ONCE => deallocate(name),
        _ => {}
    }
}

/// Releases the rights an outgoing message of ours holds while unsent:
/// its moved receive and send rights (P17: after a failed send they are
/// back in this task, under the names now written in the message).
pub(crate) fn release_outgoing(bytes: &[u8]) {
    let Ok(l) = wire::layout(bytes) else {
        return;
    };
    for d in wire::descs(bytes, &l) {
        if let Desc::Port { name, disposition } = d {
            release_right(name, disposition);
        }
    }
}

// ---- processes ----

/// This process's own audit token (P1).
pub(crate) fn own_token() -> Option<Token> {
    task_token(task())
}

fn task_token(t: PortName) -> Option<Token> {
    let mut words = [0i32; 8];
    let mut count = 8u32;
    // SAFETY: `words` has room for the eight words of the flavor (P1).
    let rc = unsafe { libc::task_info(t, TASK_AUDIT_TOKEN, words.as_mut_ptr(), &raw mut count) };
    if rc != 0 || count != 8 {
        return None;
    }
    let w = |i: usize| words.get(i).map(|&v| u32::from_ne_bytes(v.to_ne_bytes()));
    Some(Token {
        pid: w(5)?,
        version: w(7)?,
    })
}

/// This process's bootstrap special port (P27).
pub(crate) fn bootstrap_port() -> Option<PortName> {
    let mut p: PortName = NAME_NULL;
    // SAFETY: a valid out-pointer.
    let rc = unsafe { task_get_special_port(task(), TASK_BOOTSTRAP_PORT, &raw mut p) };
    (rc == 0 && p != NAME_NULL).then_some(p)
}

/// A child started by [`spawn`].
#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
pub(crate) struct Spawned {
    /// Its process ID.
    pub(crate) pid: libc::pid_t,
    /// Its audit token, read while it was suspended (P3).
    pub(crate) token: Token,
}

#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
/// Starts `program` with `bootstrap` as its bootstrap port (P27) and no
/// descriptor beyond stdio inherited, suspended until its audit token is
/// read (P3), then resumed.
pub(crate) fn spawn(
    program: &std::ffi::CStr,
    args: &[&std::ffi::CStr],
    env: &[&std::ffi::CStr],
    bootstrap: PortName,
) -> std::io::Result<Spawned> {
    let mut argv: Vec<*mut c_char> = Vec::with_capacity(args.len().saturating_add(2));
    argv.push(program.as_ptr().cast_mut());
    argv.extend(args.iter().map(|a| a.as_ptr().cast_mut()));
    argv.push(core::ptr::null_mut());
    let inherited: Vec<std::ffi::CString> = std::env::vars_os()
        .filter_map(|(k, v)| {
            let mut kv = k.into_encoded_bytes();
            kv.push(b'=');
            kv.extend(v.into_encoded_bytes());
            std::ffi::CString::new(kv).ok()
        })
        .collect();
    let mut envp: Vec<*mut c_char> = inherited.iter().map(|e| e.as_ptr().cast_mut()).collect();
    envp.extend(env.iter().map(|e| e.as_ptr().cast_mut()));
    envp.push(core::ptr::null_mut());

    // SAFETY: an all-zero attributes object is valid storage for init.
    let mut attr: libc::posix_spawnattr_t = unsafe { core::mem::zeroed() };
    // SAFETY: initialises the object just declared.
    if unsafe { libc::posix_spawnattr_init(&raw mut attr) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let flags = libc::POSIX_SPAWN_START_SUSPENDED | libc::POSIX_SPAWN_CLOEXEC_DEFAULT;
    // SAFETY: an initialised attributes object and documented flags.
    let mut rc =
        unsafe { libc::posix_spawnattr_setflags(&raw mut attr, i16::try_from(flags).unwrap_or(0)) };
    if rc == 0 {
        // SAFETY: an initialised attributes object; the kernel checks the
        // port name.
        rc = unsafe {
            posix_spawnattr_setspecialport_np(&raw mut attr, bootstrap, TASK_BOOTSTRAP_PORT)
        };
    }
    // POSIX_SPAWN_CLOEXEC_DEFAULT closes every descriptor not named in
    // file actions, so stdio is named to stay.
    // SAFETY: an all-zero file-actions object is valid storage for init.
    let mut actions: libc::posix_spawn_file_actions_t = unsafe { core::mem::zeroed() };
    if rc == 0 {
        // SAFETY: initialises the object just declared.
        rc = unsafe { libc::posix_spawn_file_actions_init(&raw mut actions) };
    }
    for fd in 0..3 {
        if rc == 0 {
            // SAFETY: an initialised file-actions object; stdio onto itself.
            rc = unsafe { libc::posix_spawn_file_actions_adddup2(&raw mut actions, fd, fd) };
        }
    }
    let mut pid: libc::pid_t = 0;
    if rc == 0 {
        // SAFETY: `program`, `argv` and `envp` are NUL-terminated and
        // null-terminated arrays of live C strings; both objects are
        // initialised.
        rc = unsafe {
            libc::posix_spawn(
                &raw mut pid,
                program.as_ptr(),
                &raw const actions,
                &raw const attr,
                argv.as_ptr(),
                envp.as_ptr(),
            )
        };
    }
    // SAFETY: destroys the object initialised above.
    let _ = unsafe { libc::posix_spawn_file_actions_destroy(&raw mut actions) };
    // SAFETY: destroys the object initialised above.
    let _ = unsafe { libc::posix_spawnattr_destroy(&raw mut attr) };
    if rc != 0 {
        return Err(std::io::Error::from_raw_os_error(rc));
    }
    let token = child_token(pid);
    // SAFETY: resumes the suspended child (P3: SIGCONT ends the suspended
    // start).
    let _ = unsafe { libc::kill(pid, libc::SIGCONT) };
    match token {
        Some(token) => Ok(Spawned { pid, token }),
        None => Err(std::io::Error::other(
            "the child's audit token could not be read",
        )),
    }
}

#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
fn child_token(pid: libc::pid_t) -> Option<Token> {
    let mut name: PortName = NAME_NULL;
    // SAFETY: a valid out-pointer.
    if unsafe { task_name_for_pid(task(), pid, &raw mut name) } != 0 {
        return None;
    }
    let t = task_token(name);
    deallocate(name);
    t
}

#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
/// Waits for child `pid` and returns its exit code (or 128 + signal).
pub(crate) fn wait_child(pid: libc::pid_t) -> std::io::Result<i32> {
    let mut status: c_int = 0;
    loop {
        // SAFETY: a valid out-pointer for the status.
        let rc = unsafe { libc::waitpid(pid, &raw mut status, 0) };
        if rc == pid {
            break;
        }
        let e = std::io::Error::last_os_error();
        if e.raw_os_error() != Some(libc::EINTR) {
            return Err(e);
        }
    }
    if libc::WIFEXITED(status) {
        Ok(libc::WEXITSTATUS(status))
    } else {
        Ok(128_i32.saturating_add(libc::WTERMSIG(status)))
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    //! Raw access for the transport's hostile-peer tests: sends crafted
    //! messages that the transport itself never would.

    use super::{Buf, Kr, PortName, send};

    use crate::ipc::mach::wire::{self, Outgoing};

    /// Encodes and sends `m`, without waiting.
    pub(crate) fn send_raw(m: &Outgoing<'_>) -> Result<(), Kr> {
        let room = wire::RECEIVE_BUFFER.max(m.body.len().saturating_add(4096));
        let mut buf = Buf::new(room);
        let Some(size) = wire::encode(m, buf.bytes_mut()) else {
            return Err(Kr(-1));
        };
        send(&mut buf, size)
    }

    /// A send right to the host name port: a kernel object a hostile
    /// peer might send. (Its own task port is the other obvious one, but
    /// under Enhanced Security sending that kills the sender, P30.)
    pub(crate) fn host_port() -> PortName {
        // SAFETY: takes no arguments and returns a send right to the
        // host name port, which the caller owns.
        unsafe { super::mach_host_self() }
    }

    /// Sends `bytes` as a message exactly as given, without waiting.
    pub(crate) fn send_bytes(bytes: &[u8]) -> Result<(), Kr> {
        let mut buf = Buf::new(bytes.len().max(64));
        if let Some(dst) = buf.bytes_mut().get_mut(..bytes.len()) {
            dst.copy_from_slice(bytes);
        }
        send(&mut buf, bytes.len())
    }

    /// A new receive right with a send right beside it.
    pub(crate) fn port_with_send() -> PortName {
        let p = super::allocate_receive().unwrap_or(0);
        let _ = super::make_send(p);
        p
    }
}
