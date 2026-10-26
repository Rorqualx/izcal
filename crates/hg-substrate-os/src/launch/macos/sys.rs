//! The macOS calls behind the launch path: a code object's code-directory
//! hash, for a file and for a running (suspended) process, through the
//! Security framework's documented code-signing services
//! (developer.apple.com: `SecStaticCodeCreateWithPath`,
//! `SecCodeCopyGuestWithAttributes`, `SecCodeCopySigningInformation`,
//! `kSecCodeInfoUnique` "the code directory hash", `kSecGuestAttributePid`)
//! and the Core Foundation types they take (`CFURL`, `CFDictionary`,
//! `CFNumber`, `CFData`). Declarations from those pages only; no header
//! and no other project's code is a source.
//!
//! **The frameworks are loaded at run time, by the broker only**
//! (`dlopen(3)`, `dlsym(3)`), never linked. This module is in
//! `hg-substrate-os`, which every child executable links; a `#[link]`
//! would give every renderer and network child load commands for
//! CoreFoundation and Security, and dyld runs their initialisers (and
//! Foundation's, Swift's and XPC's behind them) in a sandboxed process
//! that never calls them. Measured in M1 phase G (macOS 27.0.1): the
//! load alone cost each child about 464 KiB of `phys_footprint`, over
//! FP-2's 4 MiB. Only the launcher's two functions below reach the
//! loader, so a child never loads either framework.
//!
//! The launch path's only `unsafe` on macOS (CS-TY-13,
//! `unsafe-inventory.toml`, entry `launch::macos::sys`).
#![expect(
    unsafe_code,
    reason = "G5: launch::macos::sys (Security framework FFI, loaded at run time)"
)]

use core::ffi::{CStr, c_void};
use std::sync::OnceLock;

type CfRef = *const c_void;
type OsStatus = i32;

/// `kCFNumberSInt32Type`.
const CF_NUMBER_SINT32: isize = 3;

/// The frameworks, by their documented install paths (in the dyld shared
/// cache; `dlopen(3)` loads them by these names).
const CORE_FOUNDATION: &CStr =
    c"/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation";
const SECURITY: &CStr = c"/System/Library/Frameworks/Security.framework/Security";

/// The documented C signatures, one alias per call.
type UrlCreateFn = unsafe extern "C" fn(CfRef, *const u8, isize, u8) -> CfRef;
type NumberCreateFn = unsafe extern "C" fn(CfRef, isize, *const c_void) -> CfRef;
type DictionaryCreateFn = unsafe extern "C" fn(
    CfRef,
    *const CfRef,
    *const CfRef,
    isize,
    *const c_void,
    *const c_void,
) -> CfRef;
type DictionaryGetValueFn = unsafe extern "C" fn(CfRef, CfRef) -> CfRef;
type GetTypeIdFn = unsafe extern "C" fn(CfRef) -> usize;
type TypeIdFn = unsafe extern "C" fn() -> usize;
type DataLengthFn = unsafe extern "C" fn(CfRef) -> isize;
type DataBytesFn = unsafe extern "C" fn(CfRef) -> *const u8;
type ReleaseFn = unsafe extern "C" fn(CfRef);
type StaticCodeCreateFn = unsafe extern "C" fn(CfRef, u32, *mut CfRef) -> OsStatus;
type GuestWithAttributesFn = unsafe extern "C" fn(CfRef, CfRef, u32, *mut CfRef) -> OsStatus;
type SigningInformationFn = unsafe extern "C" fn(CfRef, u32, *mut CfRef) -> OsStatus;

/// The calls this module makes, resolved once from the two frameworks.
struct Api {
    cf_type_dictionary_key_callbacks: *const c_void,
    cf_type_dictionary_value_callbacks: *const c_void,
    cf_url_create_from_file_system_representation: UrlCreateFn,
    cf_number_create: NumberCreateFn,
    cf_dictionary_create: DictionaryCreateFn,
    cf_dictionary_get_value: DictionaryGetValueFn,
    cf_get_type_id: GetTypeIdFn,
    cf_data_get_type_id: TypeIdFn,
    cf_data_get_length: DataLengthFn,
    cf_data_get_byte_ptr: DataBytesFn,
    cf_release: ReleaseFn,
    /// The value of `kSecCodeInfoUnique` (a `CFStringRef` constant).
    sec_code_info_unique: CfRef,
    /// The value of `kSecGuestAttributePid`.
    sec_guest_attribute_pid: CfRef,
    sec_static_code_create_with_path: StaticCodeCreateFn,
    sec_code_copy_guest_with_attributes: GuestWithAttributesFn,
    sec_code_copy_signing_information: SigningInformationFn,
}

// SAFETY: every field is a function address or the address or value of an
// immutable framework constant; the frameworks stay loaded for the
// process's life (never `dlclose`d), so the table may be shared.
unsafe impl Send for Api {}
// SAFETY: as above; nothing in the table is ever written after load.
unsafe impl Sync for Api {}

/// Opens one framework, kept loaded for the process's life.
fn open(path: &CStr) -> Option<*mut c_void> {
    // SAFETY: a valid C string; RTLD_LAZY | RTLD_LOCAL loads a system
    // framework without adding its symbols to the global namespace.
    let h = unsafe { libc::dlopen(path.as_ptr(), libc::RTLD_LAZY | libc::RTLD_LOCAL) };
    (!h.is_null()).then_some(h)
}

/// The address of `name` in `lib`.
fn sym(lib: *mut c_void, name: &CStr) -> Option<*mut c_void> {
    // SAFETY: a handle from `dlopen` that is never closed, a valid C
    // string.
    let p = unsafe { libc::dlsym(lib, name.as_ptr()) };
    (!p.is_null()).then_some(p)
}

/// Resolves a function symbol as the pointer type `$ty`, its documented
/// signature.
macro_rules! func {
    ($lib:expr, $name:literal, $ty:ty) => {{
        let p = sym($lib, $name)?;
        // SAFETY: `p` is the address of framework function `$name`, whose
        // documented C signature is `$ty`; on Apple's ABIs a function
        // address and a function pointer have one size and representation.
        unsafe { core::mem::transmute::<*mut c_void, $ty>(p) }
    }};
}

/// The value of a constant whose symbol holds a `CFStringRef`.
fn constant(lib: *mut c_void, name: &CStr) -> Option<CfRef> {
    let p = sym(lib, name)?;
    // SAFETY: `p` is the address of an immutable, initialised
    // `CFStringRef` the framework exports.
    let v = unsafe { *p.cast::<CfRef>() };
    (!v.is_null()).then_some(v)
}

impl Api {
    fn load() -> Option<Self> {
        let cf = open(CORE_FOUNDATION)?;
        let sec = open(SECURITY)?;
        Some(Self {
            cf_type_dictionary_key_callbacks: sym(cf, c"kCFTypeDictionaryKeyCallBacks")?
                .cast_const(),
            cf_type_dictionary_value_callbacks: sym(cf, c"kCFTypeDictionaryValueCallBacks")?
                .cast_const(),
            cf_url_create_from_file_system_representation: func!(
                cf,
                c"CFURLCreateFromFileSystemRepresentation",
                UrlCreateFn
            ),
            cf_number_create: func!(cf, c"CFNumberCreate", NumberCreateFn),
            cf_dictionary_create: func!(cf, c"CFDictionaryCreate", DictionaryCreateFn),
            cf_dictionary_get_value: func!(cf, c"CFDictionaryGetValue", DictionaryGetValueFn),
            cf_get_type_id: func!(cf, c"CFGetTypeID", GetTypeIdFn),
            cf_data_get_type_id: func!(cf, c"CFDataGetTypeID", TypeIdFn),
            cf_data_get_length: func!(cf, c"CFDataGetLength", DataLengthFn),
            cf_data_get_byte_ptr: func!(cf, c"CFDataGetBytePtr", DataBytesFn),
            cf_release: func!(cf, c"CFRelease", ReleaseFn),
            sec_code_info_unique: constant(sec, c"kSecCodeInfoUnique")?,
            sec_guest_attribute_pid: constant(sec, c"kSecGuestAttributePid")?,
            sec_static_code_create_with_path: func!(
                sec,
                c"SecStaticCodeCreateWithPath",
                StaticCodeCreateFn
            ),
            sec_code_copy_guest_with_attributes: func!(
                sec,
                c"SecCodeCopyGuestWithAttributes",
                GuestWithAttributesFn
            ),
            sec_code_copy_signing_information: func!(
                sec,
                c"SecCodeCopySigningInformation",
                SigningInformationFn
            ),
        })
    }
}

/// The table, loaded on first use (the broker's first `pin`); `None` if a
/// framework or symbol is missing, which fails the launch closed.
fn api() -> Option<&'static Api> {
    static API: OnceLock<Option<Api>> = OnceLock::new();
    API.get_or_init(Api::load).as_ref()
}

/// A Core Foundation object this code owns (released on drop).
struct Owned {
    r: CfRef,
    api: &'static Api,
}

impl Owned {
    fn new(api: &'static Api, r: CfRef) -> Option<Self> {
        (!r.is_null()).then_some(Self { r, api })
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: a non-null object this code created or copied (the
        // Create/Copy rule), released once.
        unsafe { (self.api.cf_release)(self.r) };
    }
}

/// The code-directory hash of a code object, `None` if it has none.
fn unique_of(code: &Owned) -> Option<Vec<u8>> {
    let api = code.api;
    let mut info: CfRef = core::ptr::null();
    // SAFETY: a live code object and a valid out-pointer; flags 0.
    if unsafe { (api.sec_code_copy_signing_information)(code.r, 0, &raw mut info) } != 0 {
        return None;
    }
    let info = Owned::new(api, info)?;
    // SAFETY: a live dictionary and the framework's key constant; the
    // value is not owned here (the Get rule).
    let data = unsafe { (api.cf_dictionary_get_value)(info.r, api.sec_code_info_unique) };
    if data.is_null() {
        return None;
    }
    // SAFETY: CFGetTypeID on a non-null object.
    let type_id = unsafe { (api.cf_get_type_id)(data) };
    // SAFETY: CFDataGetTypeID takes nothing.
    if type_id != unsafe { (api.cf_data_get_type_id)() } {
        return None;
    }
    // SAFETY: a live CFData.
    let len = usize::try_from(unsafe { (api.cf_data_get_length)(data) }).ok()?;
    // SAFETY: a live CFData; its bytes stay valid while `info` lives.
    let ptr = unsafe { (api.cf_data_get_byte_ptr)(data) };
    if ptr.is_null() || len == 0 {
        return None;
    }
    // SAFETY: `ptr` points at `len` bytes owned by `data`, alive here.
    Some(unsafe { core::slice::from_raw_parts(ptr, len) }.to_vec())
}

/// The code-directory hash of the executable at `path`.
pub(crate) fn file_cdhash(path: &[u8]) -> Option<Vec<u8>> {
    let api = api()?;
    let len = isize::try_from(path.len()).ok()?;
    // SAFETY: `path` is `len` live bytes; default allocator.
    let url = Owned::new(api, unsafe {
        (api.cf_url_create_from_file_system_representation)(
            core::ptr::null(),
            path.as_ptr(),
            len,
            0,
        )
    })?;
    let mut code: CfRef = core::ptr::null();
    // SAFETY: a live URL and a valid out-pointer.
    if unsafe { (api.sec_static_code_create_with_path)(url.r, 0, &raw mut code) } != 0 {
        return None;
    }
    unique_of(&Owned::new(api, code)?)
}

/// The code-directory hash of running process `pid` (the kernel's view of
/// what it executed), read while the process is still suspended.
pub(crate) fn process_cdhash(pid: libc::pid_t) -> Option<Vec<u8>> {
    let api = api()?;
    // SAFETY: `pid` is a live int; default allocator.
    let num = Owned::new(api, unsafe {
        (api.cf_number_create)(core::ptr::null(), CF_NUMBER_SINT32, (&raw const pid).cast())
    })?;
    let keys = [api.sec_guest_attribute_pid];
    let values = [num.r];
    // SAFETY: one live key and value; the documented type callbacks.
    let attrs = Owned::new(api, unsafe {
        (api.cf_dictionary_create)(
            core::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            api.cf_type_dictionary_key_callbacks,
            api.cf_type_dictionary_value_callbacks,
        )
    })?;
    let mut code: CfRef = core::ptr::null();
    // SAFETY: no host (the system's), live attributes, a valid
    // out-pointer.
    if unsafe {
        (api.sec_code_copy_guest_with_attributes)(core::ptr::null(), attrs.r, 0, &raw mut code)
    } != 0
    {
        return None;
    }
    unique_of(&Owned::new(api, code)?)
}

/// The path of a held file now (`F_GETPATH`, fcntl(2)): the launcher
/// execs by this path, then checks the child's code-directory hash while
/// it is suspended.
pub(crate) fn path_of(fd: std::os::fd::BorrowedFd<'_>) -> Option<std::ffi::CString> {
    let mut buf = vec![0u8; usize::try_from(libc::PATH_MAX).unwrap_or(1024)];
    // SAFETY: F_GETPATH writes at most PATH_MAX bytes into `buf`, which
    // holds that many.
    let rc = unsafe {
        libc::fcntl(
            std::os::fd::AsRawFd::as_raw_fd(&fd),
            libc::F_GETPATH,
            buf.as_mut_ptr(),
        )
    };
    if rc < 0 {
        return None;
    }
    let end = buf.iter().position(|&b| b == 0)?;
    buf.truncate(end);
    std::ffi::CString::new(buf).ok()
}
