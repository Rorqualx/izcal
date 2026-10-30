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
//! The launch path's only `unsafe` on macOS (CS-TY-13,
//! `unsafe-inventory.toml`, entry `launch::macos::sys`).
#![expect(
    unsafe_code,
    reason = "G5: launch::macos::sys (Security framework FFI)"
)]

use core::ffi::c_void;

type CfRef = *const c_void;
type OsStatus = i32;

/// `kCFNumberSInt32Type`.
const CF_NUMBER_SINT32: isize = 3;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
    fn CFURLCreateFromFileSystemRepresentation(
        alloc: CfRef,
        buffer: *const u8,
        len: isize,
        is_dir: u8,
    ) -> CfRef;
    fn CFNumberCreate(alloc: CfRef, kind: isize, value: *const c_void) -> CfRef;
    fn CFDictionaryCreate(
        alloc: CfRef,
        keys: *const CfRef,
        values: *const CfRef,
        n: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> CfRef;
    fn CFDictionaryGetValue(dict: CfRef, key: CfRef) -> CfRef;
    fn CFGetTypeID(cf: CfRef) -> usize;
    fn CFDataGetTypeID() -> usize;
    fn CFDataGetLength(data: CfRef) -> isize;
    fn CFDataGetBytePtr(data: CfRef) -> *const u8;
    fn CFRelease(cf: CfRef);
}

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    static kSecCodeInfoUnique: CfRef;
    static kSecGuestAttributePid: CfRef;
    fn SecStaticCodeCreateWithPath(path: CfRef, flags: u32, code: *mut CfRef) -> OsStatus;
    fn SecCodeCopyGuestWithAttributes(
        host: CfRef,
        attributes: CfRef,
        flags: u32,
        guest: *mut CfRef,
    ) -> OsStatus;
    fn SecCodeCopySigningInformation(code: CfRef, flags: u32, info: *mut CfRef) -> OsStatus;
}

/// A Core Foundation object this code owns (released on drop).
struct Owned(CfRef);

impl Owned {
    fn new(r: CfRef) -> Option<Self> {
        (!r.is_null()).then_some(Self(r))
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: a non-null object this code created or copied (the
        // Create/Copy rule), released once.
        unsafe { CFRelease(self.0) };
    }
}

/// The code-directory hash of a code object, `None` if it has none.
fn unique_of(code: &Owned) -> Option<Vec<u8>> {
    let mut info: CfRef = core::ptr::null();
    // SAFETY: a live code object and a valid out-pointer; flags 0.
    if unsafe { SecCodeCopySigningInformation(code.0, 0, &raw mut info) } != 0 {
        return None;
    }
    let info = Owned::new(info)?;
    // SAFETY: a live dictionary and the framework's key constant; the
    // value is not owned here (the Get rule).
    let data = unsafe { CFDictionaryGetValue(info.0, kSecCodeInfoUnique) };
    if data.is_null() {
        return None;
    }
    // SAFETY: CFGetTypeID on a non-null object.
    let type_id = unsafe { CFGetTypeID(data) };
    // SAFETY: CFDataGetTypeID takes nothing.
    if type_id != unsafe { CFDataGetTypeID() } {
        return None;
    }
    // SAFETY: a live CFData.
    let len = usize::try_from(unsafe { CFDataGetLength(data) }).ok()?;
    // SAFETY: a live CFData; its bytes stay valid while `info` lives.
    let ptr = unsafe { CFDataGetBytePtr(data) };
    if ptr.is_null() || len == 0 {
        return None;
    }
    // SAFETY: `ptr` points at `len` bytes owned by `data`, alive here.
    Some(unsafe { core::slice::from_raw_parts(ptr, len) }.to_vec())
}

/// The code-directory hash of the executable at `path`.
pub(crate) fn file_cdhash(path: &[u8]) -> Option<Vec<u8>> {
    let len = isize::try_from(path.len()).ok()?;
    // SAFETY: `path` is `len` live bytes; default allocator.
    let url = Owned::new(unsafe {
        CFURLCreateFromFileSystemRepresentation(core::ptr::null(), path.as_ptr(), len, 0)
    })?;
    let mut code: CfRef = core::ptr::null();
    // SAFETY: a live URL and a valid out-pointer.
    if unsafe { SecStaticCodeCreateWithPath(url.0, 0, &raw mut code) } != 0 {
        return None;
    }
    unique_of(&Owned::new(code)?)
}

/// The code-directory hash of running process `pid` (the kernel's view of
/// what it executed), read while the process is still suspended.
pub(crate) fn process_cdhash(pid: libc::pid_t) -> Option<Vec<u8>> {
    // SAFETY: `pid` is a live int; default allocator.
    let num = Owned::new(unsafe {
        CFNumberCreate(core::ptr::null(), CF_NUMBER_SINT32, (&raw const pid).cast())
    })?;
    // SAFETY: reads the framework's key constant.
    let key = unsafe { kSecGuestAttributePid };
    let keys = [key];
    let values = [num.0];
    // SAFETY: one live key and value; the documented type callbacks.
    let attrs = Owned::new(unsafe {
        CFDictionaryCreate(
            core::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &raw const kCFTypeDictionaryKeyCallBacks,
            &raw const kCFTypeDictionaryValueCallBacks,
        )
    })?;
    let mut code: CfRef = core::ptr::null();
    // SAFETY: no host (the system's), live attributes, a valid
    // out-pointer.
    if unsafe { SecCodeCopyGuestWithAttributes(core::ptr::null(), attrs.0, 0, &raw mut code) } != 0
    {
        return None;
    }
    unique_of(&Owned::new(code)?)
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
