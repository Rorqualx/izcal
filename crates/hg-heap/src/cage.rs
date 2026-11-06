//! The cage (JS spec 6.1, JS-D17): one reservation per process that holds
//! every GC object. This module performs every access to cage memory;
//! nothing else in `hg-heap` computes a cage address (heap design H-1).
//!
//! **Verification condition** (stated for arbitrary cage contents, H-2):
//! every access this module performs lies inside `[base, base + len)` of
//! the cage reservation, is aligned to its width, and is atomic. It holds
//! because every offset is masked to `len − 1` (`len` is a power of two)
//! and rounded down to the access width before it is added to `base`,
//! which is page-aligned. An access may land on an uncommitted page and
//! fault: a crash inside the cage, which H-2 allows (JS-10 counts faults
//! outside it). Atomic accesses mean a concurrent corrupting write can
//! change a value but never cause a data race.
#![expect(unsafe_code, reason = "G5: cage (every cage memory access)")]

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use hg_substrate::{Memory, MemoryError, MemoryKind, Region};

use crate::slot::MAX_CAGE_BYTES;

/// The cage: its reservation and the memory interface that made it.
pub(crate) struct Cage {
    mem: Arc<dyn Memory>,
    region: Option<Region>,
    base: *mut u8,
    len: u64,
}

impl core::fmt::Debug for Cage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Cage")
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

/// Why a cage could not be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CageError {
    /// The length is not a power of two between 1 MiB and 16 GiB.
    BadLength,
    /// The reservation or a commit failed.
    Memory(MemoryError),
}

impl Cage {
    /// Reserves a cage of `len` bytes: a power of two, at least 1 MiB and
    /// at most 16 GiB. Its address space is charged to the process part;
    /// committed blocks are charged by [`Cage::commit`].
    pub(crate) fn new(mem: Arc<dyn Memory>, len: u64) -> Result<Self, CageError> {
        if !len.is_power_of_two() || !(1 << 20..=MAX_CAGE_BYTES).contains(&len) {
            return Err(CageError::BadLength);
        }
        let bytes = usize::try_from(len).map_err(|_| CageError::BadLength)?;
        let region = mem
            .reserve(bytes, MemoryKind::ProcessPart)
            .map_err(CageError::Memory)?;
        let base = std::ptr::with_exposed_provenance_mut::<u8>(region.base());
        Ok(Self {
            mem,
            region: Some(region),
            base,
            len,
        })
    }

    /// The cage's length in bytes.
    pub(crate) const fn len(&self) -> u64 {
        self.len
    }

    /// Commits `[offset, offset + len)`, charged to `kind`.
    pub(crate) fn commit(&self, offset: u64, len: u64, kind: MemoryKind) -> Result<(), CageError> {
        let (Some(region), Ok(o), Ok(l)) = (
            self.region.as_ref(),
            usize::try_from(offset),
            usize::try_from(len),
        ) else {
            return Err(CageError::BadLength);
        };
        self.mem
            .commit(region, o, l, kind)
            .map_err(CageError::Memory)
    }

    /// The address of `offset`, masked into the cage and aligned to
    /// `width` (a power of two up to 8).
    fn addr(&self, offset: u64, width: u64) -> *mut u8 {
        let o = offset & (self.len - 1) & !(width - 1);
        // o < len, and len fits in usize (checked in `new`).
        self.base.wrapping_add(usize::try_from(o).unwrap_or(0))
    }

    /// Loads the 32-bit word at `offset`.
    pub(crate) fn load32(&self, offset: u64) -> u32 {
        let p = self.addr(offset, 4).cast::<u32>();
        // SAFETY: `p` is inside the reservation and 4-aligned (this
        // module's condition); every access to it is atomic.
        let a = unsafe { AtomicU32::from_ptr(p) };
        a.load(Ordering::Relaxed)
    }

    /// Stores the 32-bit word `v` at `offset`.
    pub(crate) fn store32(&self, offset: u64, v: u32) {
        let p = self.addr(offset, 4).cast::<u32>();
        // SAFETY: as in `load32`.
        let a = unsafe { AtomicU32::from_ptr(p) };
        a.store(v, Ordering::Relaxed);
    }

    /// Loads the 64-bit word at `offset` (rounded down to 8).
    pub(crate) fn load64(&self, offset: u64) -> u64 {
        let p = self.addr(offset, 8).cast::<u64>();
        // SAFETY: `p` is inside the reservation and 8-aligned (the base is
        // page-aligned); every access to it is atomic.
        let a = unsafe { AtomicU64::from_ptr(p) };
        a.load(Ordering::Relaxed)
    }

    /// Stores the 64-bit word `v` at `offset` (rounded down to 8).
    pub(crate) fn store64(&self, offset: u64, v: u64) {
        let p = self.addr(offset, 8).cast::<u64>();
        // SAFETY: as in `load64`.
        let a = unsafe { AtomicU64::from_ptr(p) };
        a.store(v, Ordering::Relaxed);
    }

    /// Copies `len` bytes (a multiple of 8) from `from` to `to`, a word at
    /// a time through this module's own accesses.
    pub(crate) fn copy(&self, from: u64, to: u64, len: u64) {
        let mut i = 0;
        while i < len {
            self.store64(to.wrapping_add(i), self.load64(from.wrapping_add(i)));
            i += 8;
        }
    }

    /// Writes `pattern` over `len` bytes (a multiple of 8) at `offset`.
    pub(crate) fn fill(&self, offset: u64, len: u64, pattern: u64) {
        let mut i = 0;
        while i < len {
            self.store64(offset.wrapping_add(i), pattern);
            i += 8;
        }
    }
}

impl Drop for Cage {
    fn drop(&mut self) {
        if let Some(r) = self.region.take() {
            self.mem.release(r);
        }
    }
}

/// Memory for tests: each reservation is an ordinary zeroed allocation,
/// so Miri can run the heap (it cannot map a real reservation).
#[cfg(test)]
pub(crate) mod test_memory {
    use std::sync::Mutex;

    use hg_substrate::{Access, Memory, MemoryError, MemoryKind, MemoryStats, Region};

    /// Memory for tests: each reservation is a zeroed allocation.
    #[derive(Default)]
    pub(crate) struct TestMemory {
        live: Mutex<Vec<(usize, std::alloc::Layout)>>,
    }

    impl Memory for TestMemory {
        fn page_size(&self) -> usize {
            4096
        }
        fn reserve(&self, bytes: usize, kind: MemoryKind) -> Result<Region, MemoryError> {
            let layout = std::alloc::Layout::from_size_align(bytes, 4096)
                .map_err(|_| MemoryError::OutOfAddressSpace)?;
            // SAFETY: the layout has a non-zero size (cages are at least 1 MiB).
            let p = unsafe { std::alloc::alloc_zeroed(layout) };
            if p.is_null() {
                return Err(MemoryError::OutOfAddressSpace);
            }
            let base = p.expose_provenance();
            self.live.lock().unwrap().push((base, layout));
            Ok(Region::new(base, bytes, kind))
        }
        fn commit(&self, _: &Region, _: usize, _: usize, _: MemoryKind) -> Result<(), MemoryError> {
            Ok(())
        }
        fn decommit(&self, _: &Region, _: usize, _: usize) -> Result<(), MemoryError> {
            Ok(())
        }
        fn purge(&self, _: &Region, _: usize, _: usize) {}
        fn protect(&self, _: &Region, _: usize, _: usize, _: Access) -> Result<(), MemoryError> {
            Ok(())
        }
        fn release(&self, region: Region) {
            let mut live = self.live.lock().unwrap();
            if let Some(i) = live.iter().position(|(b, _)| *b == region.base()) {
                let (base, layout) = live.remove(i);
                // SAFETY: `base` came from `alloc_zeroed` with `layout`, and the
                // cage that used it is gone.
                unsafe {
                    std::alloc::dealloc(std::ptr::with_exposed_provenance_mut(base), layout);
                }
            }
        }
        fn stats(&self) -> MemoryStats {
            MemoryStats::default()
        }
    }
}
