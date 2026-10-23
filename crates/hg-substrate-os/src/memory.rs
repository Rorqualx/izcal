//! OS memory behind the substrate's `Memory` trait (Stage 1 design 3.1).
//!
//! Each region's committed part is a page-aligned prefix. The counters are
//! the process's own ledger: bytes reserved and bytes committed, as the
//! substrate granted them (the OS's resident figure is measured
//! separately, FP-M0).

use std::sync::Mutex;

use hg_substrate::{Memory, MemoryError, MemoryKind, MemoryStats, Region};
use hg_types::HashMap;

use crate::os::vm;

/// One live reservation.
#[derive(Debug, Clone, Copy)]
struct Reservation {
    len: usize,
    committed: usize,
}

#[derive(Debug, Default)]
struct Ledger {
    regions: HashMap<usize, Reservation>,
    reserved: u64,
    committed: u64,
}

/// OS virtual memory.
#[derive(Debug)]
pub struct OsMemory {
    page: usize,
    ledger: Mutex<Ledger>,
}

impl OsMemory {
    /// The OS memory interface.
    pub fn new() -> Self {
        Self {
            page: vm::page_size(),
            ledger: Mutex::new(Ledger::default()),
        }
    }

    /// The OS page size.
    pub const fn page_size(&self) -> usize {
        self.page
    }

    /// `bytes` rounded up to whole pages, or `None` on overflow.
    fn round_up(&self, bytes: usize) -> Option<usize> {
        bytes.checked_next_multiple_of(self.page)
    }

    fn ledger(&self) -> std::sync::MutexGuard<'_, Ledger> {
        // A poisoned lock means a panic mid-update, and every panic aborts
        // the process (CS-ER-2), so the data is still consistent here.
        self.ledger
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Default for OsMemory {
    fn default() -> Self {
        Self::new()
    }
}

fn to_u64(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

impl Memory for OsMemory {
    fn reserve(&self, bytes: usize, kind: MemoryKind) -> Result<Region, MemoryError> {
        let len = self
            .round_up(bytes.max(1))
            .ok_or(MemoryError::OutOfAddressSpace)?;
        let base = vm::reserve(len).map_err(|_os| MemoryError::OutOfAddressSpace)?;
        let mut l = self.ledger();
        l.regions.insert(base, Reservation { len, committed: 0 });
        l.reserved = l.reserved.saturating_add(to_u64(len));
        Ok(Region::new(base, len, kind))
    }

    fn commit(&self, region: &Region, bytes: usize) -> Result<(), MemoryError> {
        let want = self.round_up(bytes).ok_or(MemoryError::OutOfRange)?;
        let mut l = self.ledger();
        let Some(r) = l.regions.get(&region.base()).copied() else {
            return Err(MemoryError::OutOfRange);
        };
        if want > r.len {
            return Err(MemoryError::OutOfRange);
        }
        if want <= r.committed {
            return Ok(());
        }
        let start = region
            .base()
            .checked_add(r.committed)
            .ok_or(MemoryError::OutOfRange)?;
        let grow = want.saturating_sub(r.committed);
        // [start, start + grow) lies inside the live reservation `r`
        // (want <= r.len), checked under the ledger lock.
        unsafe_commit(start, grow)?;
        if let Some(e) = l.regions.get_mut(&region.base()) {
            e.committed = want;
        }
        l.committed = l.committed.saturating_add(to_u64(grow));
        Ok(())
    }

    fn decommit(&self, region: &Region, bytes: usize) {
        let Some(keep) = self.round_up(bytes) else {
            return;
        };
        let mut l = self.ledger();
        let Some(r) = l.regions.get(&region.base()).copied() else {
            return;
        };
        if keep >= r.committed {
            return;
        }
        let Some(start) = region.base().checked_add(keep) else {
            return;
        };
        let shrink = r.committed.saturating_sub(keep);
        if unsafe_decommit(start, shrink).is_ok() {
            if let Some(e) = l.regions.get_mut(&region.base()) {
                e.committed = keep;
            }
            l.committed = l.committed.saturating_sub(to_u64(shrink));
        }
    }

    fn purge(&self, region: &Region, bytes: usize) {
        let Some(keep) = self.round_up(bytes) else {
            return;
        };
        let l = self.ledger();
        let Some(r) = l.regions.get(&region.base()).copied() else {
            return;
        };
        if keep >= r.committed {
            return;
        }
        let Some(start) = region.base().checked_add(keep) else {
            return;
        };
        // Purge is advice: a refusal leaves the pages as they were.
        let _ = unsafe_purge(start, r.committed.saturating_sub(keep));
    }

    fn release(&self, region: Region) {
        let mut l = self.ledger();
        let Some(r) = l.regions.get(&region.base()).copied() else {
            return;
        };
        // The ledger lock is held across the call, so no other caller can
        // reach this reservation; it leaves the ledger only once the OS has
        // released it, so a refusal keeps the counters true.
        if unsafe_release(region.base(), r.len).is_ok() {
            l.regions.remove(&region.base());
            l.reserved = l.reserved.saturating_sub(to_u64(r.len));
            l.committed = l.committed.saturating_sub(to_u64(r.committed));
        }
    }

    fn stats(&self) -> MemoryStats {
        let l = self.ledger();
        MemoryStats {
            reserved: l.reserved,
            committed: l.committed,
        }
    }
}

// The four calls below are this module's only route into the unsafe
// `os::vm` functions. Each is reached only with a range the ledger proves
// is inside a live reservation of ours (checked by the callers above
// while they hold the ledger lock), which is the functions' contract.

#[expect(unsafe_code, reason = "G5: memory (ledger-checked calls into os::vm)")]
fn unsafe_commit(base: usize, len: usize) -> Result<(), MemoryError> {
    // SAFETY: callers pass a range inside a live reservation (see above).
    unsafe { vm::commit(base, len) }.map_err(|_os| MemoryError::CommitFailed)
}

#[expect(unsafe_code, reason = "G5: memory (ledger-checked calls into os::vm)")]
fn unsafe_decommit(base: usize, len: usize) -> Result<(), crate::os::OsError> {
    // SAFETY: callers pass a range inside a live reservation; the Memory
    // contract makes contents past the kept prefix unusable afterwards.
    unsafe { vm::decommit(base, len) }
}

#[expect(unsafe_code, reason = "G5: memory (ledger-checked calls into os::vm)")]
fn unsafe_purge(base: usize, len: usize) -> Result<(), crate::os::OsError> {
    // SAFETY: callers pass a committed range; the Memory contract makes
    // its contents undefined afterwards.
    unsafe { vm::purge(base, len) }
}

#[expect(unsafe_code, reason = "G5: memory (ledger-checked calls into os::vm)")]
fn unsafe_release(base: usize, len: usize) -> Result<(), crate::os::OsError> {
    // SAFETY: callers pass exactly one live reservation, under the ledger
    // lock, and `release` takes the Region by value, so nothing else can
    // reach it.
    unsafe { vm::release(base, len) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[expect(unsafe_code, reason = "G5: memory (the test writes committed pages)")]
    fn fill_committed(r: &Region, len: usize) {
        // SAFETY: the test committed the first `len` bytes of `r`
        // read/write just before calling this.
        unsafe { std::ptr::write_bytes(r.base() as *mut u8, 0xAB, len) };
    }

    #[test]
    fn reserve_commit_purge_decommit_release_keep_the_ledger() {
        let m = OsMemory::new();
        let page = m.page_size();
        let r = m.reserve(4 * page, MemoryKind::ProcessPart).unwrap();
        assert_eq!(r.len(), 4 * page);
        assert_eq!(
            m.stats(),
            MemoryStats {
                reserved: to_u64(4 * page),
                committed: 0
            }
        );

        m.commit(&r, page + 1).unwrap(); // rounds up to two pages
        assert_eq!(m.stats().committed, to_u64(2 * page));
        // The committed pages are writable.
        fill_committed(&r, 2 * page);

        m.purge(&r, page); // advice only: the ledger is unchanged
        assert_eq!(m.stats().committed, to_u64(2 * page));

        m.decommit(&r, page);
        assert_eq!(m.stats().committed, to_u64(page));

        assert_eq!(m.commit(&r, 5 * page), Err(MemoryError::OutOfRange));

        m.release(r);
        assert_eq!(m.stats(), MemoryStats::default());
    }
}
