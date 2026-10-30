//! Access to a mapped shared-memory region (M1 phase D6; security scope
//! IPC-R7), for every OS backend's mappings.
//!
//! Another process may write the region at any time, so no byte is ever
//! read or written as plain memory: every access is atomic (relaxed for
//! bulk copies, as the caller orders for words), which is sound under
//! concurrent writes and never lets the compiler read a byte twice. Every
//! access first checks its range with `check_range` or `check_word`, the
//! rule in `hg_substrate::ipc` that the Kani proofs cover, so no input
//! reaches outside the mapping.
//!
//! This is the only `unsafe` here (CS-TY-13, `unsafe-inventory.toml`,
//! entry `ipc::shm`).
#![expect(
    unsafe_code,
    reason = "G5: ipc::shm (atomic access to a mapped region)"
)]

use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};

use hg_substrate::ipc::{AccessError, MappingObject, check_range, check_word};

/// A live mapping's address range.
#[derive(Debug)]
pub(crate) struct Raw {
    base: NonNull<u8>,
    len: usize,
    writable: bool,
}

// SAFETY: `Raw` names memory shared with other processes; every access
// through it is atomic, so it may be used from any thread.
unsafe impl Send for Raw {}
// SAFETY: as above; `&Raw` only performs atomic accesses.
unsafe impl Sync for Raw {}

impl Raw {
    /// Wraps a mapping.
    ///
    /// # Safety
    /// `base` is the start of a mapping of at least `len` bytes, aligned
    /// to a page, readable, and writable if `writable`; it stays mapped
    /// until this `Raw` is dropped (its owner unmaps after).
    pub(crate) const unsafe fn new(base: NonNull<u8>, len: usize, writable: bool) -> Self {
        Self {
            base,
            len,
            writable,
        }
    }

    /// The address `off` bytes in; computing it is safe, using it is not.
    fn at(&self, off: usize) -> *mut u8 {
        self.base.as_ptr().wrapping_add(off)
    }

    /// The byte at `off`, which the caller has checked is in bounds.
    fn byte(&self, off: usize) -> &AtomicU8 {
        // `AtomicU8` has the size and alignment of `u8`, and every access
        // to the mapping is atomic.
        // SAFETY: the caller checked `off < len` (`check_range`), so the
        // byte is inside the live mapping (`new`'s contract).
        unsafe { AtomicU8::from_ptr(self.at(off)) }
    }

    /// Copies `out.len()` bytes from `off`.
    pub(crate) fn read(&self, off: usize, out: &mut [u8]) -> Result<(), AccessError> {
        check_range(off, out.len(), self.len)?;
        for (i, b) in out.iter_mut().enumerate() {
            *b = self.byte(off.wrapping_add(i)).load(Ordering::Relaxed);
        }
        Ok(())
    }

    /// Copies `data` to `off`.
    pub(crate) fn write(&self, off: usize, data: &[u8]) -> Result<(), AccessError> {
        check_range(off, data.len(), self.len)?;
        if !self.writable {
            return Err(AccessError::ReadOnly);
        }
        for (i, b) in data.iter().enumerate() {
            self.byte(off.wrapping_add(i)).store(*b, Ordering::Relaxed);
        }
        Ok(())
    }

    pub(crate) fn load_u32(&self, off: usize, order: Ordering) -> Result<u32, AccessError> {
        check_word(off, 4, self.len)?;
        // SAFETY: in bounds and 4-aligned (`check_word`; the base is page
        // aligned), inside the live mapping; every access is atomic.
        let w = unsafe { AtomicU32::from_ptr(self.at(off).cast::<u32>()) };
        Ok(w.load(load_order(order)))
    }

    pub(crate) fn store_u32(&self, off: usize, v: u32, order: Ordering) -> Result<(), AccessError> {
        check_word(off, 4, self.len)?;
        if !self.writable {
            return Err(AccessError::ReadOnly);
        }
        // SAFETY: as in `load_u32`, and the mapping is writable.
        let w = unsafe { AtomicU32::from_ptr(self.at(off).cast::<u32>()) };
        w.store(v, store_order(order));
        Ok(())
    }

    pub(crate) fn load_u64(&self, off: usize, order: Ordering) -> Result<u64, AccessError> {
        check_word(off, 8, self.len)?;
        // SAFETY: in bounds and 8-aligned (`check_word`; the base is page
        // aligned), inside the live mapping; every access is atomic.
        let w = unsafe { AtomicU64::from_ptr(self.at(off).cast::<u64>()) };
        Ok(w.load(load_order(order)))
    }

    pub(crate) fn store_u64(&self, off: usize, v: u64, order: Ordering) -> Result<(), AccessError> {
        check_word(off, 8, self.len)?;
        if !self.writable {
            return Err(AccessError::ReadOnly);
        }
        // SAFETY: as in `load_u64`, and the mapping is writable.
        let w = unsafe { AtomicU64::from_ptr(self.at(off).cast::<u64>()) };
        w.store(v, store_order(order));
        Ok(())
    }
}

/// A backend's mapping: [`Raw`]'s checked accesses over the mapping that
/// `guard` owns and undoes when dropped. The fields drop in order, so the
/// `Raw` is gone before the guard unmaps.
pub(crate) struct OsMapping<G: Send + Sync> {
    raw: Raw,
    guard: G,
}

impl<G: Send + Sync> OsMapping<G> {
    pub(crate) const fn new(raw: Raw, guard: G) -> Self {
        Self { raw, guard }
    }
}

impl<G: Send + Sync> core::fmt::Debug for OsMapping<G> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let _ = &self.guard;
        f.debug_struct("OsMapping")
            .field("len", &self.raw.len)
            .field("writable", &self.raw.writable)
            .finish_non_exhaustive()
    }
}

impl<G: Send + Sync> MappingObject for OsMapping<G> {
    fn len(&self) -> usize {
        self.raw.len
    }
    fn writable(&self) -> bool {
        self.raw.writable
    }
    fn read(&self, off: usize, out: &mut [u8]) -> Result<(), AccessError> {
        self.raw.read(off, out)
    }
    fn write(&self, off: usize, data: &[u8]) -> Result<(), AccessError> {
        self.raw.write(off, data)
    }
    fn load_u32(&self, off: usize, order: Ordering) -> Result<u32, AccessError> {
        self.raw.load_u32(off, order)
    }
    fn store_u32(&self, off: usize, v: u32, order: Ordering) -> Result<(), AccessError> {
        self.raw.store_u32(off, v, order)
    }
    fn load_u64(&self, off: usize, order: Ordering) -> Result<u64, AccessError> {
        self.raw.load_u64(off, order)
    }
    fn store_u64(&self, off: usize, v: u64, order: Ordering) -> Result<(), AccessError> {
        self.raw.store_u64(off, v, order)
    }
    fn base(&self) -> NonNull<u8> {
        self.raw.base
    }
}

/// A load ordering: release has none, so it becomes relaxed.
const fn load_order(o: Ordering) -> Ordering {
    match o {
        Ordering::Release | Ordering::Relaxed => Ordering::Relaxed,
        Ordering::AcqRel => Ordering::Acquire,
        other => other,
    }
}

/// A store ordering: acquire has none, so it becomes relaxed.
const fn store_order(o: Ordering) -> Ordering {
    match o {
        Ordering::Acquire | Ordering::Relaxed => Ordering::Relaxed,
        Ordering::AcqRel => Ordering::Release,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_access_over_private_memory() {
        let mut page = vec![0u64; 512];
        let base = NonNull::new(page.as_mut_ptr().cast::<u8>()).unwrap();
        // SAFETY: a live, 8-aligned, writable 4 KiB buffer that outlives `r`.
        let r = unsafe { Raw::new(base, 4096, true) };
        r.write(10, b"hello").unwrap();
        let mut out = [0u8; 5];
        r.read(10, &mut out).unwrap();
        assert_eq!(&out, b"hello");
        r.store_u64(8, 7, Ordering::Release).unwrap();
        assert_eq!(r.load_u64(8, Ordering::Acquire), Ok(7));
        assert_eq!(
            r.load_u32(2, Ordering::Acquire),
            Err(AccessError::Misaligned)
        );
        assert_eq!(r.read(4095, &mut [0; 2]), Err(AccessError::OutOfBounds));
        // SAFETY: as above, read-only view.
        let ro = unsafe { Raw::new(base, 4096, false) };
        assert_eq!(ro.write(0, b"x"), Err(AccessError::ReadOnly));
        assert_eq!(
            ro.store_u32(0, 1, Ordering::Release),
            Err(AccessError::ReadOnly)
        );
        let _ = (r, ro);
        drop(page);
    }

    #[test]
    fn a_mapping_s_guard_is_dropped_with_it() {
        use std::sync::Arc;
        use std::sync::atomic::AtomicBool;
        struct Guard(Arc<AtomicBool>);
        impl Drop for Guard {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let mut page = vec![0u64; 512];
        let base = NonNull::new(page.as_mut_ptr().cast::<u8>()).unwrap();
        let undone = Arc::new(AtomicBool::new(false));
        // SAFETY: a live, aligned, writable 4 KiB buffer that outlives `m`.
        let raw = unsafe { Raw::new(base, 4096, true) };
        let m = OsMapping::new(raw, Guard(Arc::clone(&undone)));
        m.write(0, b"x").unwrap();
        assert_eq!(MappingObject::len(&m), 4096);
        assert!(!undone.load(Ordering::SeqCst));
        drop(m);
        assert!(undone.load(Ordering::SeqCst));
        drop(page);
    }
}
