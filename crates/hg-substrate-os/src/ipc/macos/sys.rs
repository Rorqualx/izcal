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
use core::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ipc::shm::{OsMapping, Raw};

use crate::ipc::mach::accept::SlotFacts;
use crate::ipc::mach::wire::{
    self, ATTR_LIMITS, ATTR_RECEIVE_STATUS, Desc, MAKE_SEND, MAKE_SEND_ONCE, NAME_DEAD, NAME_NULL,
    NOTIFY_NO_SENDERS, NOTIFY_SEND_POSSIBLE, OPT_RCV, OPT_RCV_TIMEOUT, OPT_TRAILER_AUDIT,
    PORT_RECEIVE, PORT_SEND, PORT_SEND_ONCE, QLIMIT_MAX, RCV_OPTIONS, RECEIVE_STATUS_WORDS,
    RIGHT_PORT_SET, RIGHT_RECEIVE, RIGHT_SEND, SEND_OPTIONS, STATUS_DESTROY_REQUEST,
    STATUS_SEND_ONCE, TASK_AUDIT_TOKEN, TASK_BOOTSTRAP_PORT, Token,
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
    fn mach_vm_allocate(task: PortName, address: *mut u64, size: u64, flags: c_int) -> KernReturn;
    fn mach_make_memory_entry_64(
        task: PortName,
        size: *mut u64,
        offset: u64,
        permission: c_int,
        object_handle: *mut PortName,
        parent_entry: PortName,
    ) -> KernReturn;
    fn mach_vm_map(
        task: PortName,
        address: *mut u64,
        size: u64,
        mask: u64,
        flags: c_int,
        object: PortName,
        offset: u64,
        copy: c_int,
        cur_protection: c_int,
        max_protection: c_int,
        inheritance: c_int,
    ) -> KernReturn;
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

/// Adds one send reference to `name`, which holds a send right.
pub(crate) fn add_send_ref(name: PortName) -> Result<(), Kr> {
    // SAFETY: plain values; the kernel checks `name` holds a send right.
    kr(unsafe { mach_port_mod_refs(task(), name, RIGHT_SEND, 1) })
}

/// Drops one send (or send-once, or dead-name) reference.
pub(crate) fn deallocate(name: PortName) {
    if name == NAME_NULL || name == NAME_DEAD {
        return;
    }
    // SAFETY: plain values. The caller must hold `name`: under Enhanced
    // Security releasing a name the task does not hold is fatal
    // (EXC_GUARD INVALID_NAME, R13), not just refused.
    let _ = unsafe { mach_port_deallocate(task(), name) };
}

/// Destroys the receive right: queued messages and their rights are
/// released by the kernel, senders see a dead name.
pub(crate) fn destroy_receive(name: PortName) {
    if name == NAME_NULL || name == NAME_DEAD {
        return;
    }
    // SAFETY: plain values. The caller must hold the receive right: under
    // Enhanced Security releasing a name the task does not hold is fatal
    // (EXC_GUARD INVALID_NAME, R13), not just refused.
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

/// Aims `name`'s no-senders notification at `notify` (a receive right
/// this process holds), firing once its make-send count reaches `sync`
/// and no send right is left (P20). Any request it carried is replaced;
/// the right that request held (a send-once right) is released.
pub(crate) fn request_no_senders(name: PortName, sync: u32, notify: PortName) -> Result<(), Kr> {
    let mut previous: PortName = NAME_NULL;
    // SAFETY: plain values and a valid out-pointer for the previous
    // notify right.
    kr(unsafe {
        mach_port_request_notification(
            task(),
            name,
            i32::try_from(NOTIFY_NO_SENDERS).unwrap_or(0),
            sync,
            notify,
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

/// Takes request `id` (a notification kind) off `name`, and releases
/// the send-once right it held (R2c, R8, R9). Nothing armed is no error.
fn cancel_request(name: PortName, id: u32) {
    let mut previous: PortName = NAME_NULL;
    // SAFETY: plain values (a null notify port cancels) and a valid
    // out-pointer for the previous notify right.
    let rc = unsafe {
        mach_port_request_notification(
            task(),
            name,
            i32::try_from(id).unwrap_or(0),
            0,
            NAME_NULL,
            u32::from(MAKE_SEND_ONCE),
            &raw mut previous,
        )
    };
    if rc == 0 {
        deallocate(previous);
    }
}

/// Cancels this process's no-senders request on receive right `name`
/// before the right moves: the request belongs to this process's closer
/// port, which stays (R8).
pub(crate) fn cancel_no_senders(name: PortName) {
    cancel_request(name, NOTIFY_NO_SENDERS);
}

/// Cancels the send-possible request on send right `name` before the
/// right moves (R9).
pub(crate) fn cancel_send_possible(name: PortName) {
    cancel_request(name, NOTIFY_SEND_POSSIBLE);
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
            send_once: 0,
            destroy_request: true,
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
    let kobject = if disposition == PORT_SEND {
        kobject_type(name).map_or(0, |raw| plain_as_none(raw, plain_kobject()))
    } else {
        0
    };
    let status = if disposition == PORT_RECEIVE {
        receive_status(name)
    } else {
        None
    };
    SlotFacts {
        desc_type: wire::DESC_PORT,
        name,
        disposition,
        type_bits,
        send_refs,
        kobject,
        plain_queue: status.is_some(),
        send_once: status
            .and_then(|w| w.get(STATUS_SEND_ONCE).copied())
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0),
        destroy_request: status
            .and_then(|w| w.get(STATUS_DESTROY_REQUEST).copied())
            .is_none_or(|n| n != 0),
    }
}

/// `mach_port_kobject`'s type for send right `name`, if the query works.
fn kobject_type(name: PortName) -> Option<u32> {
    let mut kobject = 0u32;
    let mut addr = 0u64;
    // SAFETY: valid out-pointers.
    let rc = unsafe { mach_port_kobject(task(), name, &raw mut kobject, &raw mut addr) };
    (rc == 0).then_some(kobject)
}

/// What `mach_port_kobject` says of a plain port on the running kernel,
/// asked once of a port made for the purpose. The value is not stable
/// across releases (R14): 0xFFFFFFFF on macOS 27, 0 on macOS 15, while
/// every kernel object type is the same on both. `None` if the kernel
/// would not say, or says the same of this task's own port (then the
/// query cannot tell a plain port from a task port); with `None` no send
/// right is accepted.
pub(crate) fn plain_kobject() -> Option<u32> {
    static PLAIN: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *PLAIN.get_or_init(|| {
        let name = allocate_receive().ok()?;
        let plain = make_send(name).ok().and_then(|()| {
            let t = kobject_type(name);
            deallocate(name);
            t
        });
        destroy_receive(name);
        plain.filter(|&p| kobject_type(task()).is_some_and(|own| own != p))
    })
}

/// Reports a raw kobject type in the acceptance rules' terms
/// ([`wire::KOBJECT_NONE`] for a plain port): the running kernel's
/// plain-port value becomes `KOBJECT_NONE`, and a raw `KOBJECT_NONE`
/// that is not this kernel's plain value becomes 0, which no rule
/// accepts. Without a plain value nothing is reported plain.
pub(crate) const fn plain_as_none(raw: u32, plain: Option<u32>) -> u32 {
    match plain {
        Some(p) if raw == p => wire::KOBJECT_NONE,
        _ if raw == wire::KOBJECT_NONE => 0,
        _ => raw,
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
///
/// A buffer that a receive filled holds the rights of the message in it
/// until [`release_received`] releases them or [`Buf::adopted`] says
/// they became this process's own: only then, and only once, can they be
/// released. Releasing anything else (an outgoing message, a message
/// already released or adopted) does nothing, so no caller can release
/// names it does not own through this buffer.
pub(crate) struct Buf {
    words: Vec<u64>,
    holds: core::cell::Cell<bool>,
}

impl Buf {
    /// Room for `bytes` bytes.
    pub(crate) fn new(bytes: usize) -> Self {
        Self {
            words: vec![0u64; bytes.div_ceil(8)],
            holds: core::cell::Cell::new(false),
        }
    }

    /// The received message's rights became this process's own (handles,
    /// a kept reply right): they are no longer the buffer's to release.
    pub(crate) fn adopted(&self) {
        self.holds.set(false);
    }

    /// Records the outcome of a receive into this buffer.
    fn received(&self, r: Result<(), Kr>) -> Result<(), Kr> {
        self.holds.set(r.is_ok());
        r
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
/// waiting with no timeout. A message too large is destroyed by the
/// kernel, its rights released, and [`wire::RCV_TOO_LARGE`] is returned
/// with its port's name as the header's local name and no sender (R10).
pub(crate) fn receive(port: PortName, buf: &mut Buf) -> Result<(), Kr> {
    let len = buf.len32();
    let ptr = buf.ptr();
    // SAFETY: `buf` has room for `len` bytes, which the kernel never
    // exceeds; no send is requested.
    let r = kr(unsafe {
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
    });
    buf.received(r)
}

/// Receives one message from `port` if one is queued, without waiting.
/// A message too large for `buf` is destroyed by the kernel, its rights
/// released (P24).
pub(crate) fn receive_now(port: PortName, buf: &mut Buf) -> Result<(), Kr> {
    let len = buf.len32();
    let ptr = buf.ptr();
    // SAFETY: as `receive`, with a zero timeout.
    let r = kr(unsafe {
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
    });
    buf.received(r)
}

/// Receives one message from `port`, waiting up to `ms` milliseconds.
pub(crate) fn receive_within(port: PortName, buf: &mut Buf, ms: u32) -> Result<(), Kr> {
    let len = buf.len32();
    let ptr = buf.ptr();
    // SAFETY: as `receive_now`, with a timeout.
    let r = kr(unsafe {
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
    });
    buf.received(r)
}

/// Releases every right and region the message received into `buf`
/// carries, and its reply and voucher rights, as far as its layout
/// reads; once only, and only for a message received and neither
/// released nor adopted since ([`Buf`]).
pub(crate) fn release_received(buf: &Buf) {
    if !buf.holds.replace(false) {
        return;
    }
    let bytes = buf.bytes();
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
/// its moved receive and send rights, and, after a send that failed, the
/// header's own: the kernel hands the destination back as a moved send
/// right and the reply as a send-once right under a new name (P17,
/// R12). A message never sent still names them as copy and make, which
/// hold nothing.
pub(crate) fn release_outgoing(bytes: &[u8]) {
    let Ok(l) = wire::layout(bytes) else {
        return;
    };
    if l.header.remote_disposition() == PORT_SEND {
        deallocate(l.header.remote);
    }
    if l.header.local_disposition() == PORT_SEND_ONCE {
        deallocate(l.header.local);
    }
    for d in wire::descs(bytes, &l) {
        if let Desc::Port { name, disposition } = d {
            release_right(name, disposition);
        }
    }
}

// ---- memory entries (D6), from the developer.apple.com kernel pages for
// mach_make_memory_entry_64, mach_vm_allocate, mach_vm_map and
// mach_vm_deallocate, and the D6 probes (facts M1 to M10) -------------

/// `VM_PROT_READ`, `VM_PROT_READ | VM_PROT_WRITE`.
const PROT_READ: c_int = 1;
const PROT_RW: c_int = 3;
/// `VM_FLAGS_ANYWHERE`.
const FLAGS_ANYWHERE: c_int = 1;
/// `VM_INHERIT_NONE`: no child inherits a mapping (none forks anyway).
const INHERIT_NONE: c_int = 2;

/// The VM page size (16 KiB on Apple silicon, fact M3).
pub(crate) fn page_size() -> u64 {
    // SAFETY: sysconf takes a constant and reads nothing of ours.
    let p = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    u64::try_from(p).ok().filter(|&p| p > 0).unwrap_or(16384)
}

/// `len` rounded up to whole pages, or `None` past `u64`.
pub(crate) fn pages(len: u64) -> Option<u64> {
    let p = page_size();
    len.checked_add(p - 1).map(|n| n / p * p)
}

/// A new read-write memory entry of at least `len` bytes, zeroed: made
/// from a fresh anonymous range, which is then released (the entry keeps
/// the memory; fact M5).
pub(crate) fn new_entry(len: u64) -> Result<PortName, Kr> {
    let size = pages(len).ok_or(Kr(-1))?;
    let mut addr = 0u64;
    // SAFETY: a valid out-pointer; the kernel picks the address.
    kr(unsafe { mach_vm_allocate(task(), &raw mut addr, size, FLAGS_ANYWHERE) })?;
    let mut got = size;
    let mut name: PortName = NAME_NULL;
    // SAFETY: valid out-pointers; `addr`/`size` is the range just made.
    let made = kr(unsafe {
        mach_make_memory_entry_64(
            task(),
            &raw mut got,
            addr,
            PROT_RW,
            &raw mut name,
            NAME_NULL,
        )
    });
    // SAFETY: the range made above, which nothing else uses.
    let _ = unsafe { mach_vm_deallocate(task(), addr, size) };
    made?;
    if got < size {
        deallocate(name);
        return Err(Kr(-1));
    }
    Ok(name)
}

/// A sub-entry of `parent` covering its first `len` bytes (rounded to
/// pages), read-write or read-only. Fails with 0x2 for a read-write one
/// of a read-only entry (fact M2) and with 0x4 past the entry's size
/// (fact M3).
pub(crate) fn sub_entry(parent: PortName, len: u64, writable: bool) -> Result<PortName, Kr> {
    let mut size = pages(len).ok_or(Kr(-1))?;
    let mut name: PortName = NAME_NULL;
    let perm = if writable { PROT_RW } else { PROT_READ };
    // SAFETY: valid out-pointers; the kernel checks `parent`.
    kr(unsafe {
        mach_make_memory_entry_64(task(), &raw mut size, 0, perm, &raw mut name, parent)
    })?;
    Ok(name)
}

/// What a received entry allows, for `ipc::mach::accept::region_rule`:
/// whether a read-write sub-entry of the declared length can be made, and
/// whether a read-only one can (the entry is at least that long). Both
/// probes are released at once.
pub(crate) fn probe_entry(name: PortName, len: u64) -> (bool, bool) {
    let rw = sub_entry(name, len, true).map(deallocate).is_ok();
    let fits = sub_entry(name, len, false).map(deallocate).is_ok();
    (rw, fits)
}

/// A live `mach_vm_map` range, released when dropped; it counts itself in
/// `live` while it exists.
#[derive(Debug)]
pub(crate) struct VmRange {
    addr: u64,
    size: u64,
    live: Option<Arc<AtomicUsize>>,
}

impl Drop for VmRange {
    fn drop(&mut self) {
        // SAFETY: the one range `map_entry` mapped; the `OsMapping` that
        // owns this guard has dropped its `Raw` first, so nothing uses it.
        let _ = unsafe { mach_vm_deallocate(task(), self.addr, self.size) };
        if let Some(l) = &self.live {
            l.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

/// Maps `entry`'s first `len` bytes: read-write and shared, read-only and
/// shared, or (`copy`) a read-only snapshot of this moment (fact M4).
pub(crate) fn map_entry(
    entry: PortName,
    len: usize,
    writable: bool,
    copy: bool,
    live: Option<Arc<AtomicUsize>>,
) -> Result<OsMapping<VmRange>, Kr> {
    let size = pages(u64::try_from(len).map_err(|_e| Kr(-1))?).ok_or(Kr(-1))?;
    let prot = if writable { PROT_RW } else { PROT_READ };
    let mut addr = 0u64;
    // SAFETY: a valid out-pointer; a new mapping the kernel places, of an
    // entry the kernel checks.
    kr(unsafe {
        mach_vm_map(
            task(),
            &raw mut addr,
            size,
            0,
            FLAGS_ANYWHERE,
            entry,
            0,
            c_int::from(copy),
            prot,
            prot,
            INHERIT_NONE,
        )
    })?;
    if let Some(l) = &live {
        l.fetch_add(1, Ordering::AcqRel);
    }
    let guard = VmRange { addr, size, live };
    let base = usize::try_from(addr)
        .ok()
        .and_then(|a| NonNull::new(core::ptr::with_exposed_provenance_mut::<u8>(a)))
        .ok_or(Kr(-1))?;
    // SAFETY: `base` is a fresh page-aligned mapping of `size >= len`
    // bytes, readable, and writable if asked; `guard` releases it only
    // after the `OsMapping` (and its `Raw`) is dropped.
    let raw = unsafe { Raw::new(base, len, writable) };
    Ok(OsMapping::new(raw, guard))
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
/// Starts `program` with `bootstrap` as its bootstrap port (P27), no
/// descriptor beyond stdio inherited and exactly the environment `env`
/// (nothing of the launcher's: it may hold secrets, and a sandboxed
/// child should get no ambient authority), suspended until its audit
/// token is read (P3), then resumed. With a bare environment the child
/// sends a few messages to its bootstrap port before `main` (P27); the
/// launcher refuses them while it waits for the hello.
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
    let mut envp: Vec<*mut c_char> = env.iter().map(|e| e.as_ptr().cast_mut()).collect();
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
    let mut actions_ready = false;
    if rc == 0 {
        // SAFETY: initialises the object just declared.
        rc = unsafe { libc::posix_spawn_file_actions_init(&raw mut actions) };
        actions_ready = rc == 0;
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
    if actions_ready {
        // SAFETY: destroys the file-actions object, which init set up
        // (`actions_ready`); it is not used after.
        let _ = unsafe { libc::posix_spawn_file_actions_destroy(&raw mut actions) };
    }
    // SAFETY: destroys the attributes object, which init set up (a
    // failed init returned above); it is not used after.
    let _ = unsafe { libc::posix_spawnattr_destroy(&raw mut attr) };
    if rc != 0 {
        return Err(std::io::Error::from_raw_os_error(rc));
    }
    #[cfg(test)]
    test_support::LAST_PID.set(pid);
    let Some(token) = child_token(pid) else {
        // A child whose identity cannot be checked never runs: it is
        // killed while still suspended, and reaped.
        // SAFETY: signals the child just spawned, which has not run.
        let _ = unsafe { libc::kill(pid, libc::SIGKILL) };
        let _ = wait_child(pid);
        return Err(std::io::Error::other(
            "the child's audit token could not be read",
        ));
    };
    // SAFETY: resumes the suspended child (P3: SIGCONT ends the suspended
    // start).
    let _ = unsafe { libc::kill(pid, libc::SIGCONT) };
    Ok(Spawned { pid, token })
}

#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
fn child_token(pid: libc::pid_t) -> Option<Token> {
    #[cfg(test)]
    if test_support::FAIL_CHILD_TOKEN.get() {
        return None;
    }
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

    std::thread_local! {
        /// Makes this thread's next spawns fail to read the child's token.
        pub(crate) static FAIL_CHILD_TOKEN: core::cell::Cell<bool> =
            const { core::cell::Cell::new(false) };
        /// The pid of this thread's last spawned child.
        pub(crate) static LAST_PID: core::cell::Cell<libc::pid_t> =
            const { core::cell::Cell::new(0) };
    }

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

    /// Reaps child `pid` if it has exited, without waiting: its pid, or
    /// 0 while it runs.
    pub(crate) fn reap_now(pid: libc::pid_t) -> std::io::Result<libc::pid_t> {
        let mut status: libc::c_int = 0;
        // SAFETY: a valid out-pointer; WNOHANG never blocks.
        let rc = unsafe { libc::waitpid(pid, &raw mut status, libc::WNOHANG) };
        if rc < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(rc)
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
