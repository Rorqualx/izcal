//! Address space (Stage 1 design 3.1). Production is the OS's virtual
//! memory; the simulator counts committed bytes and fails on overcommit.
//!
//! A region is reserved once and then committed, decommitted, purged and
//! protected in page-aligned ranges anywhere inside it, so a heap can
//! commit the blocks it uses and return the ones it frees. Each commit
//! names the footprint ledger line its pages are charged to (footprint
//! spec 6.2), so one reservation, such as the engine's per-process cage,
//! can hold blocks of several agents, each charged to its own line.

/// The footprint ledger line a committed range is charged to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MemoryKind {
    /// The process's fixed part: memory every process of its kind has.
    ProcessPart,
    /// A per-agent part: memory that grows with each agent (realm group).
    PerAgent,
    /// A cache exempt from canonical accounting, reported as its cap.
    CanonicalExemptCache,
}

impl MemoryKind {
    /// Every kind, in [`MemoryStats::committed_by_kind`]'s order.
    pub const ALL: [Self; 3] = [
        Self::ProcessPart,
        Self::PerAgent,
        Self::CanonicalExemptCache,
    ];

    /// This kind's index in [`MemoryStats::committed_by_kind`].
    pub const fn index(self) -> usize {
        match self {
            Self::ProcessPart => 0,
            Self::PerAgent => 1,
            Self::CanonicalExemptCache => 2,
        }
    }
}

/// What the pages of a committed range may be used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Access {
    /// Neither readable nor writable (a guard).
    None,
    /// Readable only.
    Read,
    /// Readable and writable: what [`Memory::commit`] gives.
    ReadWrite,
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

    /// The ledger line the reservation itself is charged to.
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
    /// The range is not inside the region, or the region is not live.
    OutOfRange,
    /// The offset or length is not a multiple of [`Memory::page_size`].
    Misaligned,
    /// [`Memory::protect`] was asked to change pages that are not committed.
    NotCommitted,
    /// The OS refused to change the pages' protection.
    ProtectFailed,
    /// The OS refused to decommit some of the pages; they stay committed,
    /// charged and readable.
    DecommitFailed,
}

impl core::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::OutOfAddressSpace => "no address space left for the reservation",
            Self::CommitFailed => "the pages could not be committed",
            Self::OutOfRange => "the range is outside the region",
            Self::Misaligned => "the range is not page-aligned",
            Self::NotCommitted => "the range has pages that are not committed",
            Self::ProtectFailed => "the pages' protection could not be changed",
            Self::DecommitFailed => "some pages could not be decommitted",
        })
    }
}

impl std::error::Error for MemoryError {}

/// Counters for one process's memory.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryStats {
    /// Bytes of address space reserved.
    pub reserved: u64,
    /// Bytes committed (charged to the process), all ledger lines together.
    pub committed: u64,
    /// Bytes committed per ledger line, indexed by [`MemoryKind::index`].
    pub committed_by_kind: [u64; 3],
    /// Successful [`Memory::protect`] calls: each is a flip the simulator
    /// counts, so a write window's cost is visible.
    pub protect_calls: u64,
}

/// The memory interface.
///
/// Every range is `(offset, len)` within a region. `offset` and `len`
/// must be multiples of [`Memory::page_size`]; a misaligned range is
/// refused with [`MemoryError::Misaligned`] (or ignored, for the calls
/// that return nothing), never rounded, so no call can reach a page the
/// caller did not name.
pub trait Memory: Send + Sync {
    /// The granule of every range, in bytes: a power of two. Callers that
    /// must behave the same on every OS (the engine's heap) use a fixed
    /// block size that is a multiple of it.
    fn page_size(&self) -> usize;

    /// Reserves `bytes` of inaccessible address space (rounded up to whole
    /// pages); `kind` is the ledger line the reservation is charged to.
    ///
    /// # Errors
    /// [`MemoryError::OutOfAddressSpace`] if the reservation fails.
    fn reserve(&self, bytes: usize, kind: MemoryKind) -> Result<Region, MemoryError>;

    /// Commits the pages of `[offset, offset + len)` read/write, charging
    /// the newly committed ones to `kind`. Pages already committed keep
    /// their contents, protection and ledger line.
    ///
    /// # Errors
    /// [`MemoryError::Misaligned`], [`MemoryError::OutOfRange`], or
    /// [`MemoryError::CommitFailed`] if the pages cannot be committed (then
    /// none of the range's new pages is committed).
    fn commit(
        &self,
        region: &Region,
        offset: usize,
        len: usize,
        kind: MemoryKind,
    ) -> Result<(), MemoryError>;

    /// Returns the committed pages of `[offset, offset + len)` to the OS
    /// and makes them inaccessible; the reservation stays. Their contents
    /// are gone. Pages in the range that are not committed are skipped.
    ///
    /// # Errors
    ///
    /// [`MemoryError::Misaligned`] or [`MemoryError::OutOfRange`] for a bad
    /// range, which changes nothing. [`MemoryError::DecommitFailed`] when
    /// the OS refused some pages: those stay committed, charged and with
    /// their contents, so a caller that must not reuse old contents (a
    /// cage handing a chunk to another heap) must not reuse them.
    fn decommit(&self, region: &Region, offset: usize, len: usize) -> Result<(), MemoryError>;

    /// Drops the contents of the committed pages of `[offset, offset +
    /// len)`, keeping them committed and charged for reuse; their contents
    /// are undefined afterwards.
    fn purge(&self, region: &Region, offset: usize, len: usize);

    /// Sets the access of `[offset, offset + len)`, every page of which
    /// must be committed.
    ///
    /// # Errors
    /// [`MemoryError::Misaligned`], [`MemoryError::OutOfRange`],
    /// [`MemoryError::NotCommitted`], or [`MemoryError::ProtectFailed`].
    fn protect(
        &self,
        region: &Region,
        offset: usize,
        len: usize,
        access: Access,
    ) -> Result<(), MemoryError>;

    /// Releases the whole region.
    fn release(&self, region: Region);

    /// The process's current counters.
    fn stats(&self) -> MemoryStats;
}

/// Checks that `[offset, offset + len)` is page-aligned and inside a
/// region of `region_len` bytes; returns the range's end. For
/// implementations.
///
/// # Errors
/// [`MemoryError::Misaligned`] or [`MemoryError::OutOfRange`].
pub fn check_range(
    page: usize,
    region_len: usize,
    offset: usize,
    len: usize,
) -> Result<usize, MemoryError> {
    if page == 0 || !offset.is_multiple_of(page) || !len.is_multiple_of(page) {
        return Err(MemoryError::Misaligned);
    }
    let end = offset.checked_add(len).ok_or(MemoryError::OutOfRange)?;
    if end > region_len {
        return Err(MemoryError::OutOfRange);
    }
    Ok(end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_range_refuses_misaligned_and_outside_ranges() {
        assert_eq!(check_range(4096, 16384, 4096, 8192), Ok(12288));
        assert_eq!(check_range(4096, 16384, 0, 16384), Ok(16384));
        assert_eq!(check_range(4096, 16384, 0, 0), Ok(0));
        assert_eq!(
            check_range(4096, 16384, 1, 4096),
            Err(MemoryError::Misaligned)
        );
        assert_eq!(
            check_range(4096, 16384, 0, 100),
            Err(MemoryError::Misaligned)
        );
        assert_eq!(
            check_range(4096, 16384, 12288, 8192),
            Err(MemoryError::OutOfRange)
        );
        assert_eq!(
            check_range(4096, 16384, usize::MAX - 4095, 4096),
            Err(MemoryError::OutOfRange)
        );
        assert_eq!(check_range(0, 16384, 0, 0), Err(MemoryError::Misaligned));
    }

    #[test]
    fn kind_indices_follow_all() {
        for (i, k) in MemoryKind::ALL.iter().enumerate() {
            assert_eq!(k.index(), i);
        }
    }
}
