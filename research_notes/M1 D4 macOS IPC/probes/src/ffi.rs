//! Raw bindings. Source of each item is noted:
//!  [libc]  = exported by the `libc` crate (compile-tested, not read)
//!  [adoc]  = signature/field types from developer.apple.com/documentation/kernel/... (declaration shown on the page)
//!  [osf]   = OSF Mach Kernel Interface HTML man page (prose reference)
//!  [man]   = Apple man page installed with the SDK (share/man)
//!  [repo]  = already used by Izcal's crates/hg-substrate-os/src/sandbox/macos.rs
//!  [bb]    = value determined black-box by this program (`d4-probes discover`, evidence printed)
#![allow(non_camel_case_types, dead_code)]
pub use libc::{
    boolean_t, c_char, c_int, c_void, integer_t, kern_return_t, mach_msg_type_number_t,
    mach_port_t, mach_vm_address_t, mach_vm_size_t, natural_t, vm_prot_t,
};
pub type mach_port_name_t = u32; // [adoc] mach_port_name_t; natural_t
pub type mach_port_right_t = natural_t; // [adoc]
pub type mach_port_type_t = natural_t; // [adoc]
pub type mach_msg_type_name_t = u32; // [adoc] unsigned int
pub type mach_port_urefs_t = natural_t; // [adoc]
pub type mach_port_delta_t = integer_t; // [adoc]
pub type mach_port_flavor_t = c_int; // [adoc]
pub type mach_msg_return_t = kern_return_t; // [adoc]
pub type mach_msg_option_t = integer_t; // [adoc]
pub type mach_msg_size_t = natural_t; // [adoc]
pub type mach_msg_timeout_t = natural_t; // [adoc]
pub type mach_msg_id_t = integer_t; // [adoc]

extern "C" {
    // [adoc] kernel/1578704 etc.
    pub fn mach_port_allocate(
        task: mach_port_t,
        right: mach_port_right_t,
        name: *mut mach_port_name_t,
    ) -> kern_return_t;
    pub fn mach_port_deallocate(task: mach_port_t, name: mach_port_name_t) -> kern_return_t;
    pub fn mach_port_destroy(task: mach_port_t, name: mach_port_name_t) -> kern_return_t;
    pub fn mach_port_type(
        task: mach_port_t,
        name: mach_port_name_t,
        ptype: *mut mach_port_type_t,
    ) -> kern_return_t;
    pub fn mach_port_get_refs(
        task: mach_port_t,
        name: mach_port_name_t,
        right: mach_port_right_t,
        refs: *mut mach_port_urefs_t,
    ) -> kern_return_t;
    pub fn mach_port_mod_refs(
        task: mach_port_t,
        name: mach_port_name_t,
        right: mach_port_right_t,
        delta: mach_port_delta_t,
    ) -> kern_return_t;
    pub fn mach_port_insert_right(
        task: mach_port_t,
        name: mach_port_name_t,
        poly: mach_port_t,
        poly_poly: mach_msg_type_name_t,
    ) -> kern_return_t;
    pub fn mach_port_extract_right(
        task: mach_port_t,
        name: mach_port_name_t,
        msgt_name: mach_msg_type_name_t,
        poly: *mut mach_port_t,
        poly_poly: *mut mach_msg_type_name_t,
    ) -> kern_return_t;
    pub fn mach_port_insert_member(
        task: mach_port_t,
        name: mach_port_name_t,
        pset: mach_port_name_t,
    ) -> kern_return_t;
    pub fn mach_port_extract_member(
        task: mach_port_t,
        name: mach_port_name_t,
        pset: mach_port_name_t,
    ) -> kern_return_t;
    pub fn mach_port_move_member(
        task: mach_port_t,
        member: mach_port_name_t,
        after: mach_port_name_t,
    ) -> kern_return_t;
    pub fn mach_port_get_attributes(
        task: mach_port_t,
        name: mach_port_name_t,
        flavor: mach_port_flavor_t,
        info: *mut integer_t,
        cnt: *mut mach_msg_type_number_t,
    ) -> kern_return_t;
    pub fn mach_port_set_attributes(
        task: mach_port_t,
        name: mach_port_name_t,
        flavor: mach_port_flavor_t,
        info: *mut integer_t,
        cnt: mach_msg_type_number_t,
    ) -> kern_return_t;
    pub fn mach_port_request_notification(
        task: mach_port_t,
        name: mach_port_name_t,
        msgid: mach_msg_id_t,
        sync: natural_t,
        notify: mach_port_t,
        notify_poly: mach_msg_type_name_t,
        previous: *mut mach_port_t,
    ) -> kern_return_t;
    pub fn mach_port_kobject(
        task: mach_port_t,
        name: mach_port_name_t,
        object_type: *mut natural_t,
        object_addr: *mut mach_vm_address_t,
    ) -> kern_return_t;
    pub fn mach_port_kobject_description(
        task: mach_port_t,
        name: mach_port_name_t,
        object_type: *mut natural_t,
        object_addr: *mut mach_vm_address_t,
        description: *mut c_char, /* char[512] [adoc] */
    ) -> kern_return_t;
    pub fn mach_port_names(
        task: mach_port_t,
        names: *mut *mut mach_port_name_t,
        names_cnt: *mut mach_msg_type_number_t,
        types: *mut *mut mach_port_type_t,
        types_cnt: *mut mach_msg_type_number_t,
    ) -> kern_return_t;
    pub fn mach_port_guard(
        task: mach_port_t,
        name: mach_port_name_t,
        guard: u64,
        strict: boolean_t,
    ) -> kern_return_t;
    pub fn mach_port_unguard(
        task: mach_port_t,
        name: mach_port_name_t,
        guard: u64,
    ) -> kern_return_t;
    pub fn mach_port_construct(
        task: mach_port_t,
        options: *mut c_void,
        context: u64,
        name: *mut mach_port_name_t,
    ) -> kern_return_t;
    pub fn mach_port_get_srights(
        task: mach_port_t,
        name: mach_port_name_t,
        srights: *mut natural_t,
    ) -> kern_return_t;
    // [adoc] kernel/1528967-mach_msg_overwrite (mach_msg itself has no page; overwrite with rcv_msg = NULL)
    pub fn mach_msg_overwrite(
        msg: *mut c_void,
        option: mach_msg_option_t,
        send_size: mach_msg_size_t,
        rcv_size: mach_msg_size_t,
        rcv_name: mach_port_name_t,
        timeout: mach_msg_timeout_t,
        notify: mach_port_name_t,
        rcv_msg: *mut c_void,
        rcv_limit: mach_msg_size_t,
    ) -> mach_msg_return_t;
    // [adoc] kernel/1585405, 1402149, 1538205
    pub fn mach_make_memory_entry_64(
        target: mach_port_t,
        size: *mut u64,
        offset: u64,
        permission: vm_prot_t,
        handle: *mut mach_port_t,
        parent: mach_port_t,
    ) -> kern_return_t;
    pub fn mach_vm_region(
        target: mach_port_t,
        address: *mut mach_vm_address_t,
        size: *mut mach_vm_size_t,
        flavor: c_int,
        info: *mut c_int,
        cnt: *mut mach_msg_type_number_t,
        object_name: *mut mach_port_t,
    ) -> kern_return_t;
    pub fn mach_vm_deallocate(
        target: mach_port_t,
        address: mach_vm_address_t,
        size: mach_vm_size_t,
    ) -> kern_return_t;
    pub fn mach_vm_allocate(
        target: mach_port_t,
        address: *mut mach_vm_address_t,
        size: mach_vm_size_t,
        flags: c_int,
    ) -> kern_return_t;
    pub fn mach_vm_protect(
        target: mach_port_t,
        address: mach_vm_address_t,
        size: mach_vm_size_t,
        set_max: boolean_t,
        prot: vm_prot_t,
    ) -> kern_return_t;
    pub fn semaphore_create(
        task: mach_port_t,
        sem: *mut mach_port_t,
        policy: c_int,
        value: c_int,
    ) -> kern_return_t;
    // [osf] task_get_special_port(task, int which, mach_port_t *port)
    pub fn task_get_special_port(
        task: mach_port_t,
        which: c_int,
        port: *mut mach_port_t,
    ) -> kern_return_t;
    pub fn task_set_special_port(
        task: mach_port_t,
        which: c_int,
        port: mach_port_t,
    ) -> kern_return_t;
    // [osf] mach_ports_register(task, port_array, count) / mach_ports_lookup(task, &array, &count)
    pub fn mach_ports_register(
        task: mach_port_t,
        ports: *mut mach_port_t,
        count: mach_msg_type_number_t,
    ) -> kern_return_t;
    pub fn mach_ports_lookup(
        task: mach_port_t,
        ports: *mut *mut mach_port_t,
        count: *mut mach_msg_type_number_t,
    ) -> kern_return_t;
    // same shape as libc::task_for_pid ([libc]); task_name_for_pid is named in Apple DTS prose (forums 842442)
    pub fn task_name_for_pid(
        target: mach_port_t,
        pid: c_int,
        tn: *mut mach_port_t,
    ) -> kern_return_t;
    // [man] posix_spawnattr_setspecialport_np(3)
    pub fn posix_spawnattr_setspecialport_np(
        attr: *mut libc::posix_spawnattr_t,
        new_port: mach_port_t,
        which: c_int,
    ) -> c_int;
    // [man] bootstrap_check_in(3) names the `bootstrap_port` global
    pub static bootstrap_port: mach_port_t;
    // [repo] sandbox API as used by sandbox/macos.rs
    pub fn sandbox_init_with_parameters(
        profile: *const c_char,
        flags: u64,
        parameters: *const *const c_char,
        errorbuf: *mut *mut c_char,
    ) -> c_int;
    pub fn sandbox_free_error(errorbuf: *mut c_char);
}
pub fn me() -> mach_port_t {
    unsafe { libc::mach_task_self() }
}

extern "C" {
    // [man] bootstrap_check_in(3): kern_return_t bootstrap_check_in(mach_port_t bp, const name_t service_name, mach_port_t *sp)
    pub fn bootstrap_check_in(
        bp: mach_port_t,
        name: *const c_char,
        sp: *mut mach_port_t,
    ) -> kern_return_t;
}
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    // [adoc] iokit/1514687, 1514535, 1514515, 1514627; kIOMainPortDefault is a mach_port_t ([adoc] iokit/kiomainportdefault)
    pub fn IOServiceMatching(name: *const c_char) -> *mut c_void;
    pub fn IOServiceGetMatchingService(
        main_port: mach_port_t,
        matching: *mut c_void,
    ) -> mach_port_t;
    pub fn IOServiceOpen(
        service: mach_port_t,
        owning_task: mach_port_t,
        ty: u32,
        connect: *mut mach_port_t,
    ) -> kern_return_t;
    pub fn IOObjectRelease(o: mach_port_t) -> kern_return_t;
    pub static kIOMainPortDefault: mach_port_t;
}

/// Constant values. Every value below was determined black-box by `d4-probes discover`
/// (which re-checks each one by behaviour and fails loudly if any no longer holds) on
/// macOS 27.0.1 (26A434), Apple M6, 25 Oct 2026 — none comes from a header.
pub mod k {
    // mach_port_allocate `right` values that succeed are 1, 3, 4. 1 can join a set (insert_member) => RECEIVE;
    // 3 accepts members => PORT_SET; 4 neither => DEAD_NAME. get_refs index 0 counts the right made by
    // MAKE_SEND => SEND; index 2 counts the right made by MAKE_SEND_ONCE => SEND_ONCE. get_refs accepts 0..5.
    pub const RIGHT_SEND: u32 = 0;
    pub const RIGHT_RECEIVE: u32 = 1;
    pub const RIGHT_SEND_ONCE: u32 = 2;
    pub const RIGHT_PORT_SET: u32 = 3;
    pub const RIGHT_DEAD_NAME: u32 = 4;
    /// mach_port_type bit for right r: observed 1 << (16 + r) for r = 0..4.
    pub const fn type_bit(r: u32) -> u32 {
        1 << (16 + r)
    }
    // insert_right/extract_right disposition scan (D3): 20 creates a send right from a receive right (urefs 0->1),
    // 19 adds a uref to an existing send right, 17 moves it (no change in place), 21 makes a send-once right
    // (refused into a name holding the receive right; extract gives received type 18), 18 needs a send-once
    // right, 16 moves the receive right (extract gives received type 16; old name becomes invalid).
    pub const MOVE_RECEIVE: u32 = 16;
    pub const MOVE_SEND: u32 = 17;
    pub const MOVE_SEND_ONCE: u32 = 18;
    pub const COPY_SEND: u32 = 19;
    pub const MAKE_SEND: u32 = 20;
    pub const MAKE_SEND_ONCE: u32 = 21;
    pub const PORT_RECEIVE: u32 = 16; // as received
    pub const PORT_SEND: u32 = 17;
    pub const PORT_SEND_ONCE: u32 = 18;
    // mach_msg options: bit 0 sends (garbage header -> MACH_SEND_* error), bit 1 receives (blocks on an
    // empty port), 0x100 makes an empty receive return 0x10004003, 0x10 makes a send to a full queue return
    // 0x10000004, 0x4 leaves a too-large message queued and reports its size.
    pub const SEND_MSG: i32 = 0x1;
    pub const RCV_MSG: i32 = 0x2;
    pub const RCV_LARGE: i32 = 0x4;
    pub const SEND_TIMEOUT: i32 = 0x10;
    pub const RCV_TIMEOUT: i32 = 0x100;
    /// Trailer element count is the field starting at bit 24 (1 -> 12 B, 2 -> 20 B, 3 -> 52 B, 4 -> 60 B, 8 -> 68 B).
    pub const fn rcv_trailer_elements(n: u32) -> i32 {
        (n << 24) as i32
    }
    pub const AUDIT_ELEMENTS: u32 = 3;
    // header: words [bits, size, remote, local, voucher, id]; bits = remote_disp | local_disp << 8 (+ voucher
    // disp << 16 unprobed); bit 31 is set by the kernel on messages carrying descriptors.
    pub const MSGH_BITS_COMPLEX: u32 = 0x8000_0000;
    pub const fn msgh_bits(remote: u32, local: u32) -> u32 {
        remote | (local << 8)
    }
    // descriptors: port {name, pad1, pad2:16 | disposition:8 | type:8} 12 B, type 0;
    // OOL (64-bit) {address u64, deallocate:8 | copy:8 | pad1:8 | type:8, size} 16 B, type 1.
    pub const PORT_DESCRIPTOR: u32 = 0;
    pub const OOL_DESCRIPTOR: u32 = 1;
    // notification msgh_id values, by which event produced them (D4): accepted request ids are 66, 69, 70, 72.
    pub const NOTIFY_PORT_DELETED: i32 = 65; // dead-name request whose name vanished
    pub const NOTIFY_SEND_POSSIBLE: i32 = 66; // request id; see P19
    pub const NOTIFY_PORT_DESTROYED: i32 = 69; // returns the receive right in a port descriptor
    pub const NOTIFY_NO_SENDERS: i32 = 70; // fires at once on a fresh receive right with sync 0
    pub const NOTIFY_DEAD_NAME: i32 = 72; // carries the dead name
                                          // port attributes: flavor 1 returns 1 word (qlimit, settable 0..1024), flavor 2 returns 10 words:
                                          // [pset, seqno, mscount, qlimit, msgcount, sorights, srights, pdrequest, nsrequest, flags].
    pub const PORT_LIMITS_INFO: i32 = 1;
    pub const PORT_RECEIVE_STATUS: i32 = 2;
    // task_info flavor 15 returns 8 words equal to the audit-trailer token of this process's messages.
    pub const TASK_AUDIT_TOKEN: u32 = 15;
    // task_get_special_port: 4 returns the `bootstrap_port` global, 2 returns mach_host_self().
    pub const TASK_HOST_PORT: i32 = 2;
    pub const TASK_BOOTSTRAP_PORT: i32 = 4;
    // mach_vm_region flavor 9 returns 9 words, [0] = protection, [1] = max_protection.
    pub const VM_REGION_BASIC_INFO_64: i32 = 9;
    // audit token words: [auid, euid, egid, ruid, rgid, pid, asid, pidversion] (pid matched getpid(); pidversion
    // is the word that changes across exec, see P3)
    pub const AT_PID: usize = 5;
    pub const AT_PIDVERSION: usize = 7;
}
