//! Simulated memory: the backing (supplied by whoever builds the world)
//! provides real addresses; the simulator keeps each process's ledger in
//! its own page size, enforces the budget and injects commit failures.
//! A crash releases everything the process held.
//!
//! The trace records sizes, never addresses, so it does not depend on
//! where the host put a region.

use std::collections::BTreeMap;
use std::sync::Arc;

use hg_substrate::{Access, Memory, MemoryError, MemoryKind, MemoryStats, Region, check_range};

use crate::world::{Proc, ProcessId, Reservation, Shared, State, below};

pub(crate) struct SimMemory {
    pub(crate) shared: Arc<Shared>,
    pub(crate) pid: ProcessId,
    pub(crate) generation: u32,
}

/// The simulator's page: its configured page or the backing's, whichever
/// is larger, so every range it passes on is aligned for the backing. A
/// caller that uses a fixed block size (a multiple of every page size)
/// sees the same ledger and trace on every OS.
fn page(st: &State, p: &Proc) -> usize {
    usize::try_from(st.config.page_size)
        .unwrap_or(4096)
        .max(p.backing.page_size())
}

fn to_u64(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

/// The maximal runs of pages in `[first, end)` whose committed state is
/// `committed`, as `(first page, page count)`.
fn runs(
    pages: &BTreeMap<u64, MemoryKind>,
    first: u64,
    end: u64,
    committed: bool,
) -> Vec<(u64, u64)> {
    let mut out: Vec<(u64, u64)> = Vec::new();
    let mut push = |p: u64| match out.last_mut() {
        Some((start, n)) if *start + *n == p => *n += 1,
        _ => out.push((p, 1)),
    };
    if committed {
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

/// The live reservation of `region`, and the range's first and end page.
fn locate<'a>(
    p: &'a Proc,
    page: usize,
    region: &Region,
    offset: usize,
    len: usize,
) -> Result<(&'a Reservation, u64, u64), MemoryError> {
    let r = p
        .regions
        .get(&region.base())
        .ok_or(MemoryError::OutOfRange)?;
    let rlen = usize::try_from(r.len).map_err(|_| MemoryError::OutOfRange)?;
    let end = check_range(page, rlen, offset, len)?;
    Ok((r, to_u64(offset / page), to_u64(end / page)))
}

impl Memory for SimMemory {
    fn page_size(&self) -> usize {
        let st = self.shared.lock();
        st.proc(self.pid, self.generation)
            .map_or(4096, |p| page(&st, p))
    }

    fn reserve(&self, bytes: usize, kind: MemoryKind) -> Result<Region, MemoryError> {
        let mut st = self.shared.lock();
        let Some(p) = st.proc(self.pid, self.generation) else {
            return Err(MemoryError::OutOfAddressSpace);
        };
        let len = bytes
            .max(1)
            .checked_next_multiple_of(page(&st, p))
            .ok_or(MemoryError::OutOfAddressSpace)?;
        let Some(p) = st.proc_mut(self.pid, self.generation) else {
            return Err(MemoryError::OutOfAddressSpace);
        };
        let region = p.backing.reserve(len, kind)?;
        p.regions.insert(
            region.base(),
            Reservation {
                len: to_u64(len),
                committed: BTreeMap::new(),
                kind,
                host_len: region.len(),
            },
        );
        p.reserved = p.reserved.saturating_add(to_u64(len));
        let (name, generation) = (p.name, p.generation);
        st.trace
            .record(format_args!("reserve {name}.{generation} {len}"));
        // The engine sees only the simulated length: a backing that
        // rounded further keeps its extra pages to itself.
        Ok(Region::new(region.base(), len, kind))
    }

    fn commit(
        &self,
        region: &Region,
        offset: usize,
        len: usize,
        kind: MemoryKind,
    ) -> Result<(), MemoryError> {
        let mut st = self.shared.lock();
        let budget = st.config.memory_budget;
        let fail_rate = st.config.faults.commit_fail_per_million;
        let Some(p) = st.proc(self.pid, self.generation) else {
            return Err(MemoryError::CommitFailed);
        };
        let pg = page(&st, p);
        let (name, generation) = (p.name, p.generation);
        let (r, first, end) = locate(p, pg, region, offset, len)?;
        let todo = runs(&r.committed, first, end, false);
        let grow: u64 = todo.iter().map(|&(_, n)| n).sum::<u64>() * to_u64(pg);
        if grow == 0 {
            return Ok(());
        }
        if budget.is_some_and(|b| p.committed.saturating_add(grow) > b) {
            st.trace.record(format_args!(
                "commit-over-budget {name}.{generation} {grow}"
            ));
            return Err(MemoryError::CommitFailed);
        }
        if fail_rate > 0 && below(&mut st.fault, 1_000_000) < u64::from(fail_rate) {
            st.trace
                .record(format_args!("commit-fault {name}.{generation} {grow}"));
            return Err(MemoryError::CommitFailed);
        }
        let Some(p) = st.proc_mut(self.pid, self.generation) else {
            return Err(MemoryError::CommitFailed);
        };
        let mut done: Vec<(u64, u64)> = Vec::new();
        for &(f, n) in &todo {
            let (o, l) = (
                usize::try_from(f).unwrap_or(usize::MAX) * pg,
                usize::try_from(n).unwrap_or(0) * pg,
            );
            if p.backing.commit(region, o, l, kind).is_err() {
                for &(df, dn) in &done {
                    let (o, l) = (
                        usize::try_from(df).unwrap_or(usize::MAX) * pg,
                        usize::try_from(dn).unwrap_or(0) * pg,
                    );
                    // A rollback the backing refuses leaves pages it
                    // holds but this ledger does not count; the process
                    // still owns them, inside its own reservation.
                    let _ = p.backing.decommit(region, o, l);
                }
                return Err(MemoryError::CommitFailed);
            }
            done.push((f, n));
        }
        if let Some(r) = p.regions.get_mut(&region.base()) {
            for &(f, n) in &todo {
                for pgi in f..f + n {
                    r.committed.insert(pgi, kind);
                }
            }
        }
        p.committed = p.committed.saturating_add(grow);
        let k = kind.index();
        p.committed_by_kind[k] = p.committed_by_kind[k].saturating_add(grow);
        st.trace
            .record(format_args!("commit {name}.{generation} {grow}"));
        Ok(())
    }

    fn decommit(&self, region: &Region, offset: usize, len: usize) -> Result<(), MemoryError> {
        let mut st = self.shared.lock();
        let Some(p) = st.proc(self.pid, self.generation) else {
            return Err(MemoryError::OutOfRange);
        };
        let pg = page(&st, p);
        let (r, first, end) = locate(p, pg, region, offset, len)?;
        let todo = runs(&r.committed, first, end, true);
        if todo.is_empty() {
            return Ok(());
        }
        let Some(p) = st.proc_mut(self.pid, self.generation) else {
            return Err(MemoryError::OutOfRange);
        };
        let mut freed = [0u64; 3];
        let mut refused = false;
        for &(f, n) in &todo {
            let (o, l) = (
                usize::try_from(f).unwrap_or(usize::MAX) * pg,
                usize::try_from(n).unwrap_or(0) * pg,
            );
            if p.backing.decommit(region, o, l).is_err() {
                // The backing kept these pages: they stay committed here too.
                refused = true;
                continue;
            }
            if let Some(r) = p.regions.get_mut(&region.base()) {
                for pgi in f..f + n {
                    if let Some(k) = r.committed.remove(&pgi) {
                        freed[k.index()] = freed[k.index()].saturating_add(to_u64(pg));
                    }
                }
            }
        }
        let shrink: u64 = freed.iter().sum();
        p.committed = p.committed.saturating_sub(shrink);
        for (c, f) in p.committed_by_kind.iter_mut().zip(freed) {
            *c = c.saturating_sub(f);
        }
        let (name, generation) = (p.name, p.generation);
        st.trace
            .record(format_args!("decommit {name}.{generation} {shrink}"));
        if refused {
            Err(MemoryError::DecommitFailed)
        } else {
            Ok(())
        }
    }

    fn purge(&self, region: &Region, offset: usize, len: usize) {
        let mut st = self.shared.lock();
        let Some(p) = st.proc(self.pid, self.generation) else {
            return;
        };
        let pg = page(&st, p);
        let Ok((r, first, end)) = locate(p, pg, region, offset, len) else {
            return;
        };
        for (f, n) in runs(&r.committed, first, end, true) {
            let (o, l) = (
                usize::try_from(f).unwrap_or(usize::MAX) * pg,
                usize::try_from(n).unwrap_or(0) * pg,
            );
            p.backing.purge(region, o, l);
        }
        let (name, generation) = (p.name, p.generation);
        st.trace.record(format_args!("purge {name}.{generation}"));
    }

    fn protect(
        &self,
        region: &Region,
        offset: usize,
        len: usize,
        access: Access,
    ) -> Result<(), MemoryError> {
        let mut st = self.shared.lock();
        let Some(p) = st.proc(self.pid, self.generation) else {
            return Err(MemoryError::OutOfRange);
        };
        let pg = page(&st, p);
        let (r, first, end) = locate(p, pg, region, offset, len)?;
        if to_u64(r.committed.range(first..end).count()) != end.saturating_sub(first) {
            return Err(MemoryError::NotCommitted);
        }
        if len == 0 {
            return Ok(());
        }
        p.backing.protect(region, offset, len, access)?;
        let Some(p) = st.proc_mut(self.pid, self.generation) else {
            return Err(MemoryError::OutOfRange);
        };
        p.protect_calls = p.protect_calls.saturating_add(1);
        let (name, generation) = (p.name, p.generation);
        st.trace
            .record(format_args!("protect {name}.{generation} {len} {access:?}"));
        Ok(())
    }

    fn release(&self, region: Region) {
        let mut st = self.shared.lock();
        let Some(p) = st.proc(self.pid, self.generation) else {
            return;
        };
        let pg = to_u64(page(&st, p));
        let Some(p) = st.proc_mut(self.pid, self.generation) else {
            return;
        };
        let Some(r) = p.regions.remove(&region.base()) else {
            return;
        };
        let mut freed = [0u64; 3];
        for k in r.committed.values() {
            freed[k.index()] = freed[k.index()].saturating_add(pg);
        }
        p.reserved = p.reserved.saturating_sub(r.len);
        p.committed = p.committed.saturating_sub(freed.iter().sum());
        for (c, f) in p.committed_by_kind.iter_mut().zip(freed) {
            *c = c.saturating_sub(f);
        }
        p.backing
            .release(Region::new(region.base(), r.host_len, r.kind));
        let (name, generation) = (p.name, p.generation);
        st.trace
            .record(format_args!("release {name}.{generation} {}", r.len));
    }

    fn stats(&self) -> MemoryStats {
        let st = self.shared.lock();
        st.proc(self.pid, self.generation)
            .map_or_else(MemoryStats::default, |p| MemoryStats {
                reserved: p.reserved,
                committed: p.committed,
                committed_by_kind: p.committed_by_kind,
                protect_calls: p.protect_calls,
            })
    }
}
