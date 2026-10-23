//! Virtual memory, written from each OS's documentation.
//!
//! | Operation | unix (`mmap(2)`, `mprotect(2)`, `madvise(2)`) | Windows (`VirtualAlloc`, `VirtualFree`, `DiscardVirtualMemory`) |
//! |---|---|---|
//! | reserve | `mmap` `PROT_NONE`, private anonymous | `VirtualAlloc` `MEM_RESERVE`, `PAGE_NOACCESS` |
//! | commit | `mprotect` read/write (macOS also `MADV_FREE_REUSE`) | `VirtualAlloc` `MEM_COMMIT`, `PAGE_READWRITE` |
//! | decommit | `mmap` `MAP_FIXED` `PROT_NONE` over the range (drops the pages, keeps the reservation) | `VirtualFree` `MEM_DECOMMIT` |
//! | purge | Linux `MADV_FREE`; macOS `MADV_FREE_REUSABLE` (FP-M0 measured: it releases the `phys_footprint` charge) | `DiscardVirtualMemory` |
//! | release | `munmap` | `VirtualFree` `MEM_RELEASE` |
//!
//! Addresses are passed as `usize` and every range is page-aligned by the
//! caller (`crate::memory`).
#![expect(unsafe_code, reason = "G5: os::vm (virtual memory calls)")]

use super::OsError;

fn last_error() -> OsError {
    OsError(i64::from(
        std::io::Error::last_os_error().raw_os_error().unwrap_or(-1),
    ))
}

/// The OS page size in bytes.
#[cfg(unix)]
pub fn page_size() -> usize {
    // SAFETY: sysconf takes no pointers and only reads a system value.
    let n = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    usize::try_from(n).unwrap_or(4096)
}

/// Reserves `len` bytes of inaccessible address space; returns its base.
///
/// # Errors
/// The OS error if the reservation fails.
#[cfg(unix)]
pub fn reserve(len: usize) -> Result<usize, OsError> {
    #[cfg(target_os = "linux")]
    let flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE;
    #[cfg(not(target_os = "linux"))]
    let flags = libc::MAP_PRIVATE | libc::MAP_ANON;
    // SAFETY: a null hint and an anonymous mapping touch no existing
    // memory; the result is checked against MAP_FAILED.
    let p = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_NONE, flags, -1, 0) };
    if p == libc::MAP_FAILED {
        return Err(last_error());
    }
    Ok(p as usize)
}

/// Makes `[base, base + len)` readable and writable.
///
/// # Errors
/// The OS error if the pages cannot be committed.
///
/// # Safety
/// The range must lie inside a reservation from [`reserve`] that has not
/// been released.
#[cfg(unix)]
pub unsafe fn commit(base: usize, len: usize) -> Result<(), OsError> {
    let p = base as *mut libc::c_void;
    // SAFETY: the caller's contract: the range is inside a live
    // reservation, so changing its protection affects only that mapping.
    let rc = unsafe { libc::mprotect(p, len, libc::PROT_READ | libc::PROT_WRITE) };
    if rc != 0 {
        return Err(last_error());
    }
    #[cfg(target_os = "macos")]
    {
        // Pages purged with MADV_FREE_REUSABLE are charged again only after
        // MADV_FREE_REUSE; on pages never purged it has no effect.
        // SAFETY: same range as above, now mapped read/write.
        let _ = unsafe { libc::madvise(p, len, libc::MADV_FREE_REUSE) };
    }
    Ok(())
}

/// Drops the pages of `[base, base + len)` and makes them inaccessible;
/// the address range stays reserved.
///
/// # Errors
/// The OS error if the range cannot be replaced.
///
/// # Safety
/// The range must lie inside a live reservation, and no reference into it
/// may be used afterwards.
#[cfg(unix)]
pub unsafe fn decommit(base: usize, len: usize) -> Result<(), OsError> {
    #[cfg(target_os = "linux")]
    let flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE | libc::MAP_FIXED;
    #[cfg(not(target_os = "linux"))]
    let flags = libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_FIXED;
    // SAFETY: the caller's contract: the range is inside our own
    // reservation, so MAP_FIXED replaces only our pages.
    let p = unsafe {
        libc::mmap(
            base as *mut libc::c_void,
            len,
            libc::PROT_NONE,
            flags,
            -1,
            0,
        )
    };
    if p == libc::MAP_FAILED {
        return Err(last_error());
    }
    Ok(())
}

/// Tells the OS the contents of `[base, base + len)` are no longer
/// needed; the pages stay mapped and writable.
///
/// # Errors
/// The OS error if the advice is refused.
///
/// # Safety
/// The range must be committed, and its contents are undefined afterwards.
#[cfg(unix)]
pub unsafe fn purge(base: usize, len: usize) -> Result<(), OsError> {
    #[cfg(target_os = "macos")]
    let advice = libc::MADV_FREE_REUSABLE;
    #[cfg(not(target_os = "macos"))]
    let advice = libc::MADV_FREE;
    // SAFETY: the caller's contract: the range is committed memory of
    // ours whose contents may be discarded.
    let rc = unsafe { libc::madvise(base as *mut libc::c_void, len, advice) };
    if rc != 0 {
        return Err(last_error());
    }
    Ok(())
}

/// Releases a whole reservation.
///
/// # Errors
/// The OS error if the unmap fails.
///
/// # Safety
/// `(base, len)` must be exactly one reservation from [`reserve`], and no
/// reference into it may be used afterwards.
#[cfg(unix)]
pub unsafe fn release(base: usize, len: usize) -> Result<(), OsError> {
    // SAFETY: the caller's contract: this is exactly one of our
    // reservations and nothing refers into it.
    let rc = unsafe { libc::munmap(base as *mut libc::c_void, len) };
    if rc != 0 {
        return Err(last_error());
    }
    Ok(())
}

/// The OS page size in bytes.
#[cfg(windows)]
pub fn page_size() -> usize {
    use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
    // SAFETY: a zeroed struct is a valid initial value.
    let mut info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
    // SAFETY: valid out-pointer.
    unsafe { GetSystemInfo(&mut info) };
    usize::try_from(info.dwPageSize).unwrap_or(4096)
}

/// Reserves `len` bytes of inaccessible address space; returns its base.
///
/// # Errors
/// The OS error if the reservation fails.
#[cfg(windows)]
pub fn reserve(len: usize) -> Result<usize, OsError> {
    use windows_sys::Win32::System::Memory::{MEM_RESERVE, PAGE_NOACCESS, VirtualAlloc};
    // SAFETY: a null address asks the OS to choose; the result is checked.
    let p = unsafe { VirtualAlloc(std::ptr::null(), len, MEM_RESERVE, PAGE_NOACCESS) };
    if p.is_null() {
        return Err(last_error());
    }
    Ok(p as usize)
}

/// Makes `[base, base + len)` readable and writable.
///
/// # Errors
/// The OS error if the pages cannot be committed.
///
/// # Safety
/// The range must lie inside a reservation from [`reserve`] that has not
/// been released.
#[cfg(windows)]
pub unsafe fn commit(base: usize, len: usize) -> Result<(), OsError> {
    use windows_sys::Win32::System::Memory::{MEM_COMMIT, PAGE_READWRITE, VirtualAlloc};
    // SAFETY: the caller's contract: the range is inside a live
    // reservation of ours.
    let p = unsafe { VirtualAlloc(base as *const _, len, MEM_COMMIT, PAGE_READWRITE) };
    if p.is_null() {
        return Err(last_error());
    }
    Ok(())
}

/// Decommits `[base, base + len)`; the address range stays reserved.
///
/// # Errors
/// The OS error if the decommit fails.
///
/// # Safety
/// The range must lie inside a live reservation, and no reference into it
/// may be used afterwards.
#[cfg(windows)]
pub unsafe fn decommit(base: usize, len: usize) -> Result<(), OsError> {
    use windows_sys::Win32::System::Memory::{MEM_DECOMMIT, VirtualFree};
    // SAFETY: the caller's contract: the range is inside our reservation.
    let ok = unsafe { VirtualFree(base as *mut _, len, MEM_DECOMMIT) };
    if ok == 0 {
        return Err(last_error());
    }
    Ok(())
}

/// Tells the OS the contents of `[base, base + len)` are no longer
/// needed; the pages stay committed.
///
/// # Errors
/// The Win32 error code `DiscardVirtualMemory` returned.
///
/// # Safety
/// The range must be committed, and its contents are undefined afterwards.
#[cfg(windows)]
pub unsafe fn purge(base: usize, len: usize) -> Result<(), OsError> {
    use windows_sys::Win32::System::Memory::DiscardVirtualMemory;
    // SAFETY: the caller's contract: the range is committed memory of
    // ours whose contents may be discarded.
    let rc = unsafe { DiscardVirtualMemory(base as *mut _, len) };
    if rc != 0 {
        return Err(OsError(i64::from(rc)));
    }
    Ok(())
}

/// Releases a whole reservation.
///
/// # Errors
/// The OS error if the release fails.
///
/// # Safety
/// `base` must be the base of one reservation from [`reserve`], and no
/// reference into it may be used afterwards.
#[cfg(windows)]
pub unsafe fn release(base: usize, _len: usize) -> Result<(), OsError> {
    use windows_sys::Win32::System::Memory::{MEM_RELEASE, VirtualFree};
    // SAFETY: the caller's contract: `base` starts one of our
    // reservations; MEM_RELEASE requires size 0.
    let ok = unsafe { VirtualFree(base as *mut _, 0, MEM_RELEASE) };
    if ok == 0 {
        return Err(last_error());
    }
    Ok(())
}
