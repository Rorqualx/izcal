//! Simulated memory: the backing (supplied by whoever builds the world)
//! provides real addresses; the simulator keeps each process's ledger in
//! its own page size, enforces the budget and injects commit failures.
//! A crash releases everything the process held.
//!
//! The trace records sizes, never addresses, so it does not depend on
//! where the host put a region.

use std::sync::Arc;

use hg_substrate::{Memory, MemoryError, MemoryKind, MemoryStats, Region};

use crate::world::{ProcessId, Reservation, Shared, State, below};

pub(crate) struct SimMemory {
    pub(crate) shared: Arc<Shared>,
    pub(crate) pid: ProcessId,
    pub(crate) generation: u32,
}

fn pages(st: &State, bytes: usize) -> Option<u64> {
    u64::try_from(bytes)
        .ok()?
        .checked_next_multiple_of(st.config.page_size)
}

impl Memory for SimMemory {
    fn reserve(&self, bytes: usize, kind: MemoryKind) -> Result<Region, MemoryError> {
        let mut st = self.shared.lock();
        let len = pages(&st, bytes.max(1)).ok_or(MemoryError::OutOfAddressSpace)?;
        let Some(p) = st.proc_mut(self.pid, self.generation) else {
            return Err(MemoryError::OutOfAddressSpace);
        };
        let region = p.backing.reserve(bytes, kind)?;
        p.regions.insert(
            region.base(),
            Reservation {
                len,
                committed: 0,
                kind,
                host_len: region.len(),
            },
        );
        p.reserved = p.reserved.saturating_add(len);
        let (name, generation) = (p.name, p.generation);
        st.trace
            .record(format_args!("reserve {name}.{generation} {len}"));
        Ok(region)
    }

    fn commit(&self, region: &Region, bytes: usize) -> Result<(), MemoryError> {
        let mut st = self.shared.lock();
        let want = pages(&st, bytes).ok_or(MemoryError::OutOfRange)?;
        let budget = st.config.memory_budget;
        let fail_rate = st.config.faults.commit_fail_per_million;
        let Some(p) = st.proc(self.pid, self.generation) else {
            return Err(MemoryError::CommitFailed);
        };
        let (name, generation) = (p.name, p.generation);
        let r = *p
            .regions
            .get(&region.base())
            .ok_or(MemoryError::OutOfRange)?;
        if want > r.len {
            return Err(MemoryError::OutOfRange);
        }
        if want <= r.committed {
            return Ok(());
        }
        let grow = want.saturating_sub(r.committed);
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
        p.backing.commit(region, bytes)?;
        if let Some(e) = p.regions.get_mut(&region.base()) {
            e.committed = want;
        }
        p.committed = p.committed.saturating_add(grow);
        st.trace
            .record(format_args!("commit {name}.{generation} {grow}"));
        Ok(())
    }

    fn decommit(&self, region: &Region, bytes: usize) {
        let mut st = self.shared.lock();
        let Some(keep) = pages(&st, bytes) else {
            return;
        };
        let Some(p) = st.proc_mut(self.pid, self.generation) else {
            return;
        };
        let Some(r) = p.regions.get_mut(&region.base()) else {
            return;
        };
        if keep >= r.committed {
            return;
        }
        let shrink = r.committed.saturating_sub(keep);
        r.committed = keep;
        p.committed = p.committed.saturating_sub(shrink);
        p.backing.decommit(region, bytes);
        let (name, generation) = (p.name, p.generation);
        st.trace
            .record(format_args!("decommit {name}.{generation} {shrink}"));
    }

    fn purge(&self, region: &Region, bytes: usize) {
        let mut st = self.shared.lock();
        let Some(p) = st.proc(self.pid, self.generation) else {
            return;
        };
        if !p.regions.contains_key(&region.base()) {
            return;
        }
        p.backing.purge(region, bytes);
        let (name, generation) = (p.name, p.generation);
        st.trace.record(format_args!("purge {name}.{generation}"));
    }

    fn release(&self, region: Region) {
        let mut st = self.shared.lock();
        let Some(p) = st.proc_mut(self.pid, self.generation) else {
            return;
        };
        let Some(r) = p.regions.remove(&region.base()) else {
            return;
        };
        p.reserved = p.reserved.saturating_sub(r.len);
        p.committed = p.committed.saturating_sub(r.committed);
        p.backing.release(region);
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
            })
    }
}
