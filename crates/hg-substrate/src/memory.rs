//! Address space (Stage 1 design 3.1). Production is the OS's virtual
//! memory; the simulator counts committed and touched bytes and fails on
//! overcommit. Every region carries a [`MemoryKind`], the footprint
//! ledger line it is charged to (footprint spec 6.2).

/// The footprint ledger line a region is charged to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryKind {
    /// The process's fixed part: memory every process of its kind has.
    ProcessPart,
    /// A per-agent part: memory that grows with each agent (realm group).
    PerAgent,
    /// A cache exempt from canonical accounting, reported as its cap.
    CanonicalExemptCache,
}

/// A reserved range of address space. Created by a [`Memory`]
/// implementation; other code passes it back to the same implementation.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Region {
    base: usize,
    len: usize,
    kind: MemoryKind,
}

impl Region {
    /// A region at `base` of `len` bytes. For implementations only.
    pub const fn new(base: usize, len: usize, kind: MemoryKind) -> Self {
        Self { base, len, kind }
    }

    /// The region's start address.
    pub const fn base(&self) -> usize {
        self.base
    }

    /// The region's length in bytes.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the region is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The ledger line the region is charged to.
    pub const fn kind(&self) -> MemoryKind {
        self.kind
    }
}

/// Why a memory operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    /// No address space left for the reservation.
    OutOfAddressSpace,
    /// The OS (or the simulator's budget) refused to commit the pages.
    CommitFailed,
    /// The byte count is not within the region.
    OutOfRange,
}

impl core::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::OutOfAddressSpace => "no address space left for the reservation",
            Self::CommitFailed => "the pages could not be committed",
            Self::OutOfRange => "the byte count is outside the region",
        })
    }
}

impl std::error::Error for MemoryError {}

/// Counters for one process's memory, by state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryStats {
    /// Bytes of address space reserved.
    pub reserved: u64,
    /// Bytes committed (charged to the process).
    pub committed: u64,
}

/// The memory interface.
pub trait Memory: Send + Sync {
    /// Reserves `bytes` of address space, charged to `kind`'s ledger line.
    ///
    /// # Errors
    /// [`MemoryError::OutOfAddressSpace`] if the reservation fails.
    fn reserve(&self, bytes: usize, kind: MemoryKind) -> Result<Region, MemoryError>;

    /// Commits the first `bytes` of `region`.
    ///
    /// # Errors
    /// [`MemoryError::CommitFailed`] if the pages cannot be committed;
    /// [`MemoryError::OutOfRange`] if `bytes` exceeds the region.
    fn commit(&self, region: &Region, bytes: usize) -> Result<(), MemoryError>;

    /// Decommits the first `bytes` of `region`; the reservation stays.
    fn decommit(&self, region: &Region, bytes: usize);

    /// Drops the pages of the first `bytes` of `region` but keeps them
    /// committed for reuse; the ledger records the purge.
    fn purge(&self, region: &Region, bytes: usize);

    /// Releases the whole region.
    fn release(&self, region: Region);

    /// The process's current counters.
    fn stats(&self) -> MemoryStats;
}
