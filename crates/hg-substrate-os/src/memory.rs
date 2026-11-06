//! OS memory behind the substrate's `Memory` trait (Stage 1 design 3.1).
//!
//! Each region's committed part is a page-aligned prefix. The counters are
//! the process's own ledger: bytes reserved and bytes committed, as the
//! substrate granted them (the OS's resident figure is measured
//! separately, FP-M0).

use std::collections::BTreeMap;
use std::sync::Mutex;

use hg_substrate::{Access, Memory, MemoryError, MemoryKind, MemoryStats, Region, check_range};
use hg_types::HashMap;

use crate::os::vm;

/// One live reservation: its length and, per committed page (by index),
/// the ledger line the page is charged to.
#[derive(Debug)]
struct Reservation {
    len: usize,
    committed: BTreeMap<usize, MemoryKind>,
}

#[derive(Debug, Default)]
struct Ledger {
    regions: HashMap<usize, Reservation>,
    reserved: u64,
    committed_by_kind: [u64; 3],
    protect_calls: u64,
}

impl Ledger {
    fn stats(&self) -> MemoryStats {
        MemoryStats {
            reserved: self.reserved,
            committed: self.committed_by_kind.iter().sum(),
            committed_by_kind: self.committed_by_kind,
            protect_calls: self.protect_calls,
        }
    }
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

/// The maximal runs of pages in `[first, end)` whose committed state is
/// `committed`, as `(first page, page count)`.
fn runs(
    pages: &BTreeMap<usize, MemoryKind>,
    first: usize,
    end: usize,
    committed: bool,
) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut push = |p: usize| match out.last_mut() {
        Some((start, n)) if *start + *n == p => *n += 1,
        _ => out.push((p, 1)),
    };
    if committed {
        // Only the committed pages, however large the range.
        for &p in pages.range(first..end).map(|(p, _)| p) {
            push(p);
        }
    } else {
        for p in first..end {
            if !pages.contains_key(&p) {
                push(p);
            }
        }
    }
    out
}

impl Memory for OsMemory {
    fn page_size(&self) -> usize {
        self.page
    }

    fn reserve(&self, bytes: usize, kind: MemoryKind) -> Result<Region, MemoryError> {
        let len = bytes
            .max(1)
            .checked_next_multiple_of(self.page)
            .ok_or(MemoryError::OutOfAddressSpace)?;
        let base = vm::reserve(len).map_err(|_os| MemoryError::OutOfAddressSpace)?;
        let mut l = self.ledger();
        l.regions.insert(
            base,
            Reservation {
                len,
                committed: BTreeMap::new(),
            },
        );
        l.reserved = l.reserved.saturating_add(to_u64(len));
        Ok(Region::new(base, len, kind))
    }

    fn commit(
        &self,
        region: &Region,
        offset: usize,
        len: usize,
        kind: MemoryKind,
    ) -> Result<(), MemoryError> {
        let page = self.page;
        let mut l = self.ledger();
        let r = l
            .regions
            .get(&region.base())
            .ok_or(MemoryError::OutOfRange)?;
        let end = check_range(page, r.len, offset, len)?;
        let todo = runs(&r.committed, offset / page, end / page, false);
        let mut done: Vec<(usize, usize)> = Vec::new();
        for &(first, n) in &todo {
            // The run lies inside the live reservation (check_range above),
            // checked under the ledger lock.
            if unsafe_commit(region.base() + first * page, n * page).is_err() {
                // Undo this call's runs, so a failure commits nothing new.
                for &(f, m) in &done {
                    let _ = unsafe_decommit(region.base() + f * page, m * page);
                }
                return Err(MemoryError::CommitFailed);
            }
            done.push((first, n));
        }
        let mut grown = 0usize;
        if let Some(r) = l.regions.get_mut(&region.base()) {
            for &(first, n) in &todo {
                for p in first..first + n {
                    r.committed.insert(p, kind);
                }
                grown += n * page;
            }
        }
        let k = kind.index();
        l.committed_by_kind[k] = l.committed_by_kind[k].saturating_add(to_u64(grown));
        Ok(())
    }

    fn decommit(&self, region: &Region, offset: usize, len: usize) -> Result<(), MemoryError> {
        let page = self.page;
        let mut l = self.ledger();
        let Some(r) = l.regions.get(&region.base()) else {
            return Err(MemoryError::OutOfRange);
        };
        let end = check_range(page, r.len, offset, len)?;
        let todo = runs(&r.committed, offset / page, end / page, true);
        let mut freed = [0u64; 3];
        let mut refused = false;
        for &(first, n) in &todo {
            if unsafe_decommit(region.base() + first * page, n * page).is_err() {
                // These pages stay committed, charged and in the ledger.
                refused = true;
                continue;
            }
            if let Some(r) = l.regions.get_mut(&region.base()) {
                for p in first..first + n {
                    if let Some(k) = r.committed.remove(&p) {
                        freed[k.index()] = freed[k.index()].saturating_add(to_u64(page));
                    }
                }
            }
        }
        for (c, f) in l.committed_by_kind.iter_mut().zip(freed) {
            *c = c.saturating_sub(f);
        }
        if refused {
            Err(MemoryError::DecommitFailed)
        } else {
            Ok(())
        }
    }

    fn purge(&self, region: &Region, offset: usize, len: usize) {
        let page = self.page;
        let l = self.ledger();
        let Some(r) = l.regions.get(&region.base()) else {
            return;
        };
        let Ok(end) = check_range(page, r.len, offset, len) else {
            return;
        };
        for (first, n) in runs(&r.committed, offset / page, end / page, true) {
            // Purge is advice: a refusal leaves the pages as they were.
            let _ = unsafe_purge(region.base() + first * page, n * page);
        }
    }

    fn protect(
        &self,
        region: &Region,
        offset: usize,
        len: usize,
        access: Access,
    ) -> Result<(), MemoryError> {
        let page = self.page;
        let mut l = self.ledger();
        let r = l
            .regions
            .get(&region.base())
            .ok_or(MemoryError::OutOfRange)?;
        let end = check_range(page, r.len, offset, len)?;
        // Counts only the committed pages in the range, so the check costs
        // what is committed, not the size of the range.
        if r.committed.range(offset / page..end / page).count() != len / page {
            return Err(MemoryError::NotCommitted);
        }
        if len == 0 {
            return Ok(());
        }
        // Every page of the range is committed (checked just above, under
        // the ledger lock).
        unsafe_protect(region.base() + offset, len, access)
            .map_err(|_os| MemoryError::ProtectFailed)?;
        l.protect_calls = l.protect_calls.saturating_add(1);
        Ok(())
    }

    fn release(&self, region: Region) {
        let page = self.page;
        let mut l = self.ledger();
        let Some(r) = l.regions.get(&region.base()) else {
            return;
        };
        let len = r.len;
        // The ledger lock is held across the call, so no other caller can
        // reach this reservation; it leaves the ledger only once the OS has
        // released it, so a refusal keeps the counters true.
        if unsafe_release(region.base(), len).is_ok() {
            let mut freed = [0u64; 3];
            if let Some(r) = l.regions.remove(&region.base()) {
                for k in r.committed.values() {
                    freed[k.index()] = freed[k.index()].saturating_add(to_u64(page));
                }
            }
            for (c, f) in l.committed_by_kind.iter_mut().zip(freed) {
                *c = c.saturating_sub(f);
            }
            l.reserved = l.reserved.saturating_sub(to_u64(len));
        }
    }

    fn stats(&self) -> MemoryStats {
        self.ledger().stats()
    }
}

// The five calls below are this module's only route into the unsafe
// `os::vm` functions. Each is reached only with a range the ledger proves
// is inside a live reservation of ours (checked by the callers above
// while they hold the ledger lock), which is the functions' contract.

#[expect(unsafe_code, reason = "G5: memory (ledger-checked calls into os::vm)")]
fn unsafe_commit(base: usize, len: usize) -> Result<(), crate::os::OsError> {
    // SAFETY: callers pass a range inside a live reservation (see above).
    unsafe { vm::commit(base, len) }
}

#[expect(unsafe_code, reason = "G5: memory (ledger-checked calls into os::vm)")]
fn unsafe_decommit(base: usize, len: usize) -> Result<(), crate::os::OsError> {
    // SAFETY: callers pass a range inside a live reservation; the Memory
    // contract makes the decommitted contents unusable afterwards.
    unsafe { vm::decommit(base, len) }
}

#[expect(unsafe_code, reason = "G5: memory (ledger-checked calls into os::vm)")]
fn unsafe_purge(base: usize, len: usize) -> Result<(), crate::os::OsError> {
    // SAFETY: callers pass a committed range; the Memory contract makes
    // its contents undefined afterwards.
    unsafe { vm::purge(base, len) }
}

#[expect(unsafe_code, reason = "G5: memory (ledger-checked calls into os::vm)")]
fn unsafe_protect(base: usize, len: usize, access: Access) -> Result<(), crate::os::OsError> {
    let a = match access {
        Access::None => 0,
        Access::Read => 1,
        Access::ReadWrite => 2,
    };
    // SAFETY: callers pass a range whose every page is committed, inside a
    // live reservation; the Memory contract makes the caller responsible
    // for not using the pages against their new protection.
    unsafe { vm::protect(base, len, a) }
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

    #[expect(unsafe_code, reason = "G5: memory (the test touches committed pages)")]
    fn fill(r: &Region, offset: usize, len: usize) {
        // SAFETY: the test committed `[offset, offset + len)` of `r`
        // read/write just before calling this.
        unsafe { std::ptr::write_bytes((r.base() + offset) as *mut u8, 0xAB, len) };
    }

    #[expect(unsafe_code, reason = "G5: memory (the test reads committed pages)")]
    fn read(r: &Region, offset: usize) -> u8 {
        // SAFETY: the test committed the page at `offset` readable.
        unsafe { std::ptr::read_volatile((r.base() + offset) as *const u8) }
    }

    #[test]
    fn ranges_commit_anywhere_and_keep_a_ledger_per_kind() {
        let m = OsMemory::new();
        let page = m.page_size();
        let r = m.reserve(8 * page, MemoryKind::ProcessPart).unwrap();
        assert_eq!(r.len(), 8 * page);
        assert_eq!(m.stats().reserved, to_u64(8 * page));

        // Two separate ranges, charged to two lines.
        m.commit(&r, 2 * page, 2 * page, MemoryKind::PerAgent)
            .unwrap();
        m.commit(&r, 6 * page, page, MemoryKind::ProcessPart)
            .unwrap();
        fill(&r, 2 * page, 2 * page);
        fill(&r, 6 * page, page);
        let s = m.stats();
        assert_eq!(s.committed, to_u64(3 * page));
        assert_eq!(s.committed_by_kind, [to_u64(page), to_u64(2 * page), 0]);

        // Committing over committed pages charges only the new ones and
        // keeps the old ones' contents and line.
        m.commit(&r, page, 3 * page, MemoryKind::CanonicalExemptCache)
            .unwrap();
        assert_eq!(read(&r, 2 * page), 0xAB);
        assert_eq!(
            m.stats().committed_by_kind,
            [to_u64(page), to_u64(2 * page), to_u64(page)]
        );

        // Decommitting a range returns each page to its own line.
        assert_eq!(m.decommit(&r, 0, 4 * page), Ok(()));
        assert_eq!(m.stats().committed_by_kind, [to_u64(page), 0, 0]);
        assert_eq!(m.decommit(&r, 1, page), Err(MemoryError::Misaligned));

        // Misaligned and outside ranges are refused, never rounded.
        assert_eq!(
            m.commit(&r, 1, page, MemoryKind::PerAgent),
            Err(MemoryError::Misaligned)
        );
        assert_eq!(
            m.commit(&r, 7 * page, 2 * page, MemoryKind::PerAgent),
            Err(MemoryError::OutOfRange)
        );

        m.purge(&r, 6 * page, page); // advice only: the ledger is unchanged
        assert_eq!(m.stats().committed, to_u64(page));

        m.release(r);
        assert_eq!(m.stats(), MemoryStats::default());
    }

    #[test]
    fn protect_needs_committed_pages_and_counts_flips() {
        let m = OsMemory::new();
        let page = m.page_size();
        let r = m.reserve(4 * page, MemoryKind::ProcessPart).unwrap();
        m.commit(&r, 0, 2 * page, MemoryKind::ProcessPart).unwrap();
        fill(&r, 0, 2 * page);

        assert_eq!(
            m.protect(&r, page, 2 * page, Access::Read),
            Err(MemoryError::NotCommitted)
        );
        m.protect(&r, 0, 2 * page, Access::Read).unwrap();
        assert_eq!(read(&r, page), 0xAB); // still readable
        m.protect(&r, 0, 2 * page, Access::ReadWrite).unwrap();
        fill(&r, 0, page); // writable again
        assert_eq!(m.stats().protect_calls, 2);

        m.release(r);
    }
}
