//! A `Memory` that delegates to the OS substrate and records what it was
//! asked for. The cage reserves itself through `Memory`, so this is the
//! only way the harness learns the cage's base address (needed to find
//! its mappings in the OS's own view) and counts the substrate calls each
//! leg makes.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use hg_substrate::{Access, Memory, MemoryError, MemoryKind, MemoryStats, Region};
use hg_substrate_os::memory::OsMemory;

/// One reservation this memory handed out.
#[derive(Debug, Clone, Copy)]
pub struct Reservation {
    /// Its base address.
    pub base: usize,
    /// Its length in bytes.
    pub len: usize,
}

/// Counts of the substrate calls made through this memory.
#[derive(Debug, Clone, Copy, Default)]
pub struct Calls {
    /// `reserve` calls that succeeded.
    pub reserves: u64,
    /// `reserve` calls that failed.
    pub reserve_failures: u64,
    /// `commit` calls that succeeded, and the bytes they named.
    pub commits: u64,
    /// Bytes named by successful commits (pages already committed count
    /// again: this is the call's length, not the ledger's delta).
    pub commit_bytes: u64,
    /// `decommit` calls that succeeded.
    pub decommits: u64,
    /// `decommit` calls the OS refused.
    pub decommit_failures: u64,
    /// `release` calls.
    pub releases: u64,
}

/// The OS memory, with a record of every reservation and call.
#[derive(Debug)]
pub struct SpyMemory {
    inner: OsMemory,
    live: Mutex<Vec<Reservation>>,
    reserves: AtomicU64,
    reserve_failures: AtomicU64,
    commits: AtomicU64,
    commit_bytes: AtomicU64,
    decommits: AtomicU64,
    decommit_failures: AtomicU64,
    releases: AtomicU64,
}

impl Default for SpyMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl SpyMemory {
    /// A fresh recording memory over the OS substrate.
    pub fn new() -> Self {
        Self {
            inner: OsMemory::new(),
            live: Mutex::new(Vec::new()),
            reserves: AtomicU64::new(0),
            reserve_failures: AtomicU64::new(0),
            commits: AtomicU64::new(0),
            commit_bytes: AtomicU64::new(0),
            decommits: AtomicU64::new(0),
            decommit_failures: AtomicU64::new(0),
            releases: AtomicU64::new(0),
        }
    }

    /// The reservations still live, in the order they were made.
    pub fn reservations(&self) -> Vec<Reservation> {
        self.live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// The substrate calls made so far.
    pub fn calls(&self) -> Calls {
        Calls {
            reserves: self.reserves.load(Ordering::Relaxed),
            reserve_failures: self.reserve_failures.load(Ordering::Relaxed),
            commits: self.commits.load(Ordering::Relaxed),
            commit_bytes: self.commit_bytes.load(Ordering::Relaxed),
            decommits: self.decommits.load(Ordering::Relaxed),
            decommit_failures: self.decommit_failures.load(Ordering::Relaxed),
            releases: self.releases.load(Ordering::Relaxed),
        }
    }
}

impl Memory for SpyMemory {
    fn page_size(&self) -> usize {
        self.inner.page_size()
    }

    fn reserve(&self, bytes: usize, kind: MemoryKind) -> Result<Region, MemoryError> {
        match self.inner.reserve(bytes, kind) {
            Ok(r) => {
                self.reserves.fetch_add(1, Ordering::Relaxed);
                self.live
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(Reservation {
                        base: r.base(),
                        len: r.len(),
                    });
                Ok(r)
            }
            Err(e) => {
                self.reserve_failures.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    fn commit(
        &self,
        region: &Region,
        offset: usize,
        len: usize,
        kind: MemoryKind,
    ) -> Result<(), MemoryError> {
        let r = self.inner.commit(region, offset, len, kind);
        if r.is_ok() {
            self.commits.fetch_add(1, Ordering::Relaxed);
            self.commit_bytes
                .fetch_add(u64::try_from(len).unwrap_or(0), Ordering::Relaxed);
        }
        r
    }

    fn decommit(&self, region: &Region, offset: usize, len: usize) -> Result<(), MemoryError> {
        let r = self.inner.decommit(region, offset, len);
        match r {
            Ok(()) => self.decommits.fetch_add(1, Ordering::Relaxed),
            Err(_) => self.decommit_failures.fetch_add(1, Ordering::Relaxed),
        };
        r
    }

    fn purge(&self, region: &Region, offset: usize, len: usize) {
        self.inner.purge(region, offset, len);
    }

    fn protect(
        &self,
        region: &Region,
        offset: usize,
        len: usize,
        access: Access,
    ) -> Result<(), MemoryError> {
        self.inner.protect(region, offset, len, access)
    }

    fn release(&self, region: Region) {
        self.releases.fetch_add(1, Ordering::Relaxed);
        self.live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|r| r.base != region.base());
        self.inner.release(region);
    }

    fn stats(&self) -> MemoryStats {
        self.inner.stats()
    }
}
