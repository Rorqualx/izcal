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

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use hg_substrate::{Memory, MemoryError, MemoryKind, Region};

use crate::slot::MAX_CAGE_BYTES;

/// Bytes the cage hands an agent heap at a time: a multiple of every page
/// size Izcal runs on (4, 16 and 64 KiB), so commits are aligned on every
/// OS and the simulator's ledger is the same everywhere (heap design H-5).
pub(crate) const CHUNK: u64 = 1 << 20;

/// Which chunks are free. Chunk 0 is never handed out, so no object lives
/// at cage offset 0.
#[derive(Debug)]
struct Pool {
    /// The first chunk never handed out.
    next: u64,
    /// Chunks handed out and returned, lowest first.
    free: BTreeSet<u64>,
    /// Chunks returned whose decommit the OS refused. They keep their old
    /// contents, so they are never handed out again; they stay committed
    /// and charged until the cage goes.
    kept: u64,
}

/// The cage: its reservation, the memory interface that made it, and the
/// pool of chunks its agent heaps draw from (JS-D17: one cage per
/// process; each agent heap owns its own chunks).
pub(crate) struct Cage {
    mem: Arc<dyn Memory>,
    region: Option<Region>,
    base: *mut u8,
    len: u64,
    pool: Mutex<Pool>,
}

// SAFETY: the raw base pointer is never written after `new`, every access
// through it is atomic (this module's condition), and the chunk pool is
// behind a mutex; so agent heaps on different threads may share the cage.
unsafe impl Send for Cage {}
// SAFETY: as for `Send`: shared references only perform atomic accesses
// and locked pool operations.
unsafe impl Sync for Cage {}

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
            pool: Mutex::new(Pool {
                next: CHUNK,
                free: BTreeSet::new(),
                kept: 0,
            }),
        })
    }

    /// Hands out `n` adjacent chunks, committed and charged to the
    /// per-agent ledger line; returns the first one's offset. The lowest
    /// run of `n` returned chunks is reused; otherwise the run comes from
    /// fresh space.
    pub(crate) fn acquire(&self, n: u64) -> Result<u64, CageError> {
        let mut pool = self
            .pool
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let reuse = Self::free_run(&pool.free, n);
        if let Some(at) = reuse {
            for i in 0..n {
                pool.free.remove(&(at + i * CHUNK));
            }
        }
        let at = match reuse {
            Some(at) => at,
            None => {
                let at = pool.next;
                let end = n
                    .checked_mul(CHUNK)
                    .and_then(|b| at.checked_add(b))
                    .ok_or(CageError::BadLength)?;
                if end > self.len {
                    return Err(CageError::BadLength);
                }
                pool.next = end;
                at
            }
        };
        if let Err(e) = self.commit(at, n * CHUNK, MemoryKind::PerAgent) {
            // Nothing was committed: the chunks go back as they were.
            if reuse.is_some() {
                for i in 0..n {
                    pool.free.insert(at + i * CHUNK);
                }
            } else if pool.next == at + n * CHUNK {
                pool.next = at;
            }
            return Err(e);
        }
        Ok(at)
    }

    /// The lowest run of `n` adjacent chunks in `free`, if any.
    fn free_run(free: &BTreeSet<u64>, n: u64) -> Option<u64> {
        let mut run_start = 0;
        let mut run_len = 0;
        let mut prev = None;
        for &c in free {
            if prev.is_some_and(|p: u64| p.checked_add(CHUNK) == Some(c)) {
                run_len += 1;
            } else {
                run_start = c;
                run_len = 1;
            }
            if run_len == n {
                return Some(run_start);
            }
            prev = Some(c);
        }
        None
    }

    /// Takes back the chunk at `offset`: its pages are decommitted, so
    /// they read as zero, or fault, never as old data (I-2), and it is
    /// reused. If the decommit is refused, the chunk still holds the old
    /// heap's data: it is kept out of the pool for the cage's life.
    pub(crate) fn release(&self, offset: u64) {
        let decommitted = match (
            self.region.as_ref(),
            usize::try_from(offset),
            usize::try_from(CHUNK),
        ) {
            (Some(region), Ok(o), Ok(l)) => self.mem.decommit(region, o, l).is_ok(),
            _ => false,
        };
        let mut pool = self
            .pool
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if decommitted {
            pool.free.insert(offset);
        } else {
            pool.kept += 1;
        }
    }

    /// How many returned chunks were kept out of the pool because their
    /// decommit was refused.
    #[cfg(test)]
    pub(crate) fn kept(&self) -> u64 {
        self.pool
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .kept
    }

    /// Commits `[offset, offset + len)`, charged to `kind`.
    fn commit(&self, offset: u64, len: u64, kind: MemoryKind) -> Result<(), CageError> {
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
    use std::sync::atomic::{AtomicBool, Ordering};

    use hg_substrate::{Access, Memory, MemoryError, MemoryKind, MemoryStats, Region};

    /// Memory for tests: each reservation is a zeroed allocation, and a
    /// decommit zeroes its range, as the OS's fresh pages read.
    #[derive(Default)]
    pub(crate) struct TestMemory {
        live: Mutex<Vec<(usize, std::alloc::Layout)>>,
        protects: Mutex<Vec<(usize, usize, Access)>>,
        refuse_decommit: AtomicBool,
    }

    impl TestMemory {
        /// Makes every later decommit fail, leaving its pages as they are.
        pub(crate) fn refuse_decommit(&self, refuse: bool) {
            self.refuse_decommit.store(refuse, Ordering::SeqCst);
        }

        /// Every protection change asked for, in order.
        pub(crate) fn protects(&self) -> Vec<(usize, usize, Access)> {
            self.protects.lock().unwrap().clone()
        }
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
        fn decommit(&self, region: &Region, offset: usize, len: usize) -> Result<(), MemoryError> {
            if self.refuse_decommit.load(Ordering::SeqCst) {
                return Err(MemoryError::DecommitFailed);
            }
            let end = offset.checked_add(len).ok_or(MemoryError::OutOfRange)?;
            if end > region.len() {
                return Err(MemoryError::OutOfRange);
            }
            // SAFETY: `[offset, end)` is inside the live allocation behind
            // `region` (checked above), and the cage does not access a
            // chunk while it is being returned.
            unsafe {
                std::ptr::with_exposed_provenance_mut::<u8>(region.base() + offset)
                    .write_bytes(0, len);
            }
            Ok(())
        }
        fn purge(&self, _: &Region, _: usize, _: usize) {}
        fn protect(&self, _: &Region, o: usize, l: usize, a: Access) -> Result<(), MemoryError> {
            self.protects.lock().unwrap().push((o, l, a));
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::test_memory::TestMemory;
    use super::{CHUNK, Cage};

    /// A released chunk comes back zeroed. One whose decommit was refused
    /// still holds the old heap's data, so it is never handed out again.
    #[test]
    fn a_chunk_whose_decommit_was_refused_is_never_reused() {
        let mem = Arc::new(TestMemory::default());
        let cage = Cage::new(mem.clone(), 8 * CHUNK).unwrap();
        let a = cage.acquire(1).unwrap();
        cage.store64(a, 0x5EC2E7);
        cage.release(a);
        assert_eq!(cage.acquire(1).unwrap(), a, "a decommitted chunk is reused");
        assert_eq!(cage.load64(a), 0, "and reads as zero");

        cage.store64(a, 0x5EC2E7);
        mem.refuse_decommit(true);
        cage.release(a);
        mem.refuse_decommit(false);
        assert_eq!(cage.kept(), 1);
        let b = cage.acquire(1).unwrap();
        assert_ne!(b, a, "a chunk that kept its data is not handed out");
        assert_eq!(cage.load64(a), 0x5EC2E7);
        assert_eq!(cage.load64(b), 0);
    }

    /// A run of chunks reuses adjacent returned chunks, so heaps that come
    /// and go cannot use up the cage for large objects.
    #[test]
    fn a_run_reuses_adjacent_returned_chunks() {
        // Chunk 0 is never handed out, so this cage has three to give.
        let cage = Cage::new(Arc::new(TestMemory::default()), 4 * CHUNK).unwrap();
        let chunks: Vec<u64> = (0..3).map(|_| cage.acquire(1).unwrap()).collect();
        assert!(cage.acquire(1).is_err(), "the cage is full");
        // Returned out of order, with a gap: no run of three yet.
        cage.release(chunks[2]);
        cage.release(chunks[0]);
        assert!(
            cage.acquire(3).is_err(),
            "two returned chunks are no run of three"
        );
        assert_eq!(
            cage.acquire(2).ok(),
            None,
            "nor of two: they are not adjacent"
        );
        cage.release(chunks[1]);
        assert_eq!(cage.acquire(3).unwrap(), chunks[0]);
        assert!(cage.acquire(1).is_err(), "the run took all three");
    }
}
