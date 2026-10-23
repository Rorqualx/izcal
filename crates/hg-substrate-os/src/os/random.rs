//! The OS CSPRNG, written from each OS's documentation: `getentropy(2)` on
//! macOS (at most 256 bytes per call), `getrandom(2)` on Linux, and
//! `BCryptGenRandom` with `BCRYPT_USE_SYSTEM_PREFERRED_RNG` on Windows.
#![expect(unsafe_code, reason = "G5: os::random (the OS CSPRNG calls)")]

use super::OsError;

/// Fills `dest` from the OS CSPRNG.
///
/// # Errors
/// The OS error code if the OS refused.
#[cfg(target_os = "macos")]
pub fn fill(dest: &mut [u8]) -> Result<(), OsError> {
    for chunk in dest.chunks_mut(256) {
        // SAFETY: `chunk` is a valid, writable buffer of chunk.len() bytes,
        // at most 256 as getentropy(2) requires.
        let rc = unsafe { libc::getentropy(chunk.as_mut_ptr().cast(), chunk.len()) };
        if rc != 0 {
            return Err(OsError(i64::from(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            )));
        }
    }
    Ok(())
}

/// Fills `dest` from the OS CSPRNG.
///
/// # Errors
/// The OS error code if the OS refused.
#[cfg(target_os = "linux")]
pub fn fill(dest: &mut [u8]) -> Result<(), OsError> {
    let mut rest = dest;
    while !rest.is_empty() {
        // SAFETY: `rest` is a valid, writable buffer of rest.len() bytes;
        // flags 0 reads the urandom source, blocking only until it is
        // first initialized.
        let n = unsafe { libc::getrandom(rest.as_mut_ptr().cast(), rest.len(), 0) };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(OsError(i64::from(err.raw_os_error().unwrap_or(-1))));
        }
        let n = usize::try_from(n).unwrap_or(0);
        rest = rest.get_mut(n..).unwrap_or_default();
    }
    Ok(())
}

/// Fills `dest` from the OS CSPRNG.
///
/// # Errors
/// The NTSTATUS if the OS refused.
#[cfg(windows)]
pub fn fill(dest: &mut [u8]) -> Result<(), OsError> {
    use windows_sys::Win32::Security::Cryptography::{
        BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
    };
    // Chunks of 1 MiB keep each length well inside BCryptGenRandom's u32.
    for chunk in dest.chunks_mut(1 << 20) {
        let len = u32::try_from(chunk.len()).unwrap_or(u32::MAX);
        // SAFETY: `chunk` is a valid, writable buffer of `len` bytes; a
        // null algorithm handle with BCRYPT_USE_SYSTEM_PREFERRED_RNG is
        // the documented way to use the system generator.
        let status = unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                chunk.as_mut_ptr(),
                len,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status != 0 {
            return Err(OsError(i64::from(status)));
        }
    }
    Ok(())
}
