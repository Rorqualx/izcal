//! Volatile, then release — and what that costs on three back ends.
//!
//! FP 6.6's hidden-tab rule: "After 30 s† hidden, every tile and
//! image texture of the tab is released ... within 30 s they are
//! marked volatile or low residency first." GR 6.2 names the
//! mechanisms: "Volatile and low-residency states
//! (`setPurgeableState`, `SetResidencyPriority`) implement FP 6.6's
//! 'volatile, then release'".
//!
//! # The three back ends do not mean the same thing by it
//!
//! - **Metal** `setPurgeableState(_:)` with `volatile`: the system
//!   "can reclaim the underlying storage at any time **without
//!   informing the app**" (GA 4.10, in `research_notes/Izcal graphics
//!   and rendering/gpu_apis_platforms.md`). So the contents can be
//!   gone and no callback says so.
//! - **Direct3D 12** `Evict`/`MakeResident` with
//!   `SetResidencyPriority` (GA 3.18) is *residency*: the allocation
//!   is paged out, not discarded.
//! - **Vulkan** has no purgeable state in the APIs this project
//!   surveyed — `VK_EXT_memory_budget` "reports per-heap budget and
//!   usage" (GA 5.3) and nothing more — so volatility has to be
//!   emulated by freeing and reallocating, which loses the contents
//!   by construction.
//!
//! Two of the three therefore cannot promise the pixels survive, so
//! **the contract is that they do not**. D3D12's stronger guarantee
//! is invisible here, which is the right way round: an interface
//! written to D3D12's behaviour would show stale or foreign pixels on
//! the other two, on the path that runs when a tab has been hidden
//! for half a minute — the least-tested path there is.
//!
//! # Why this is a safety property and not a correctness one
//!
//! Sampling a texture whose storage the system reclaimed reads
//! whatever is there now. That is the same class of mistake
//! `hg-display-list`'s bounds check exists to stop, where "whatever
//! the sampler then reads is memory the renderer did not put there",
//! and it is the second-largest Chrome graphics CVE class
//! (uninitialized use, GR 9's table). So [`Allocation::can_sample`]
//! is false until the allocation has been written since it came back,
//! and no amount of residency alone makes it true.

use crate::ledger::{AllocationId, Tag};

/// Where the allocation's storage is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Residency {
    /// Present and usable.
    Resident,
    /// Offered to the system. On Metal the storage may already be
    /// gone; on D3D12 it is paged out; on Vulkan it is freed.
    Volatile,
    /// Given up. There is nothing to come back to.
    Released,
}

/// Whether what was in the allocation is still in it.
///
/// `#[must_use]` because the whole mechanism fails if a caller takes
/// an allocation back and does not look at the answer: on Metal that
/// is the case where the pixels are gone and nothing said so.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Contents {
    /// Still there: the allocation may be sampled.
    Intact,
    /// Gone. The allocation is a destination and not a source until
    /// something writes it.
    Lost,
}

/// What a residency transition refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidencyError {
    /// A released allocation cannot come back. Its memory is not
    /// reserved for it and may be anything now; the caller allocates
    /// a new one, which is also what the Vulkan path does for every
    /// volatile allocation.
    Released,
    /// Already in that state, which is a caller that has lost track
    /// of it rather than a no-op worth hiding.
    AlreadyThere,
}

/// One allocation, as the allocator's interface sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Allocation {
    id: AllocationId,
    tag: Tag,
    bytes: u64,
    residency: Residency,
    /// Whether the bytes now in it were written since the last time
    /// the system could have taken them.
    written: bool,
}

impl Allocation {
    /// A fresh allocation: resident, and **not** sampleable.
    ///
    /// New GPU memory is not zero on every back end and the engine
    /// does not get to assume it is (GR 9, lazy clearing), so a
    /// just-made allocation is in exactly the position a recovered
    /// one is: a destination until written.
    #[must_use]
    pub const fn new(id: AllocationId, tag: Tag, bytes: u64) -> Self {
        Self {
            id,
            tag,
            bytes,
            residency: Residency::Resident,
            written: false,
        }
    }

    /// Which allocation.
    #[must_use]
    pub const fn id(&self) -> AllocationId {
        self.id
    }

    /// Its ledger tag.
    #[must_use]
    pub const fn tag(&self) -> Tag {
        self.tag
    }

    /// Its size.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Where its storage is.
    #[must_use]
    pub const fn residency(&self) -> Residency {
        self.residency
    }

    /// Whether it may be read.
    ///
    /// Resident **and** written since the last moment the system
    /// could have reclaimed it. The second half is the one that
    /// matters: residency says the memory is addressable, not that
    /// the pixels in it are Izcal's.
    #[must_use]
    pub const fn can_sample(&self) -> bool {
        matches!(self.residency, Residency::Resident) && self.written
    }

    /// Whether it may be written.
    #[must_use]
    pub const fn can_write(&self) -> bool {
        matches!(self.residency, Residency::Resident)
    }

    /// Record that something wrote the whole allocation.
    ///
    /// # Errors
    ///
    /// [`ResidencyError::Released`] if there is nothing to write, or
    /// [`ResidencyError::AlreadyThere`] if it is volatile — writing
    /// through a volatile handle is the bug this stops, because on
    /// Metal the write can land in storage the system has taken back.
    pub const fn mark_written(&mut self) -> Result<(), ResidencyError> {
        match self.residency {
            Residency::Resident => {
                self.written = true;
                Ok(())
            }
            Residency::Volatile => Err(ResidencyError::AlreadyThere),
            Residency::Released => Err(ResidencyError::Released),
        }
    }

    /// Offer the storage to the system.
    ///
    /// # Errors
    ///
    /// [`ResidencyError::AlreadyThere`] if it is already volatile,
    /// [`ResidencyError::Released`] if it is gone.
    pub const fn make_volatile(&mut self) -> Result<(), ResidencyError> {
        match self.residency {
            Residency::Resident => {
                self.residency = Residency::Volatile;
                // Whatever was written is now the system's to keep or
                // discard, so the engine knows nothing about it.
                self.written = false;
                Ok(())
            }
            Residency::Volatile => Err(ResidencyError::AlreadyThere),
            Residency::Released => Err(ResidencyError::Released),
        }
    }

    /// Take the storage back, and say whether the contents came with
    /// it.
    ///
    /// `survived` is the back end's report. A back end that cannot
    /// tell must pass `false`: the only sound default is that the
    /// pixels are gone, and that is the Vulkan path's answer always.
    ///
    /// # Errors
    ///
    /// [`ResidencyError::Released`] if it was released, which is not
    /// recoverable, or [`ResidencyError::AlreadyThere`] if it was
    /// never volatile.
    pub const fn make_resident(&mut self, survived: bool) -> Result<Contents, ResidencyError> {
        match self.residency {
            Residency::Volatile => {
                self.residency = Residency::Resident;
                self.written = survived;
                if survived {
                    Ok(Contents::Intact)
                } else {
                    Ok(Contents::Lost)
                }
            }
            Residency::Resident => Err(ResidencyError::AlreadyThere),
            Residency::Released => Err(ResidencyError::Released),
        }
    }

    /// Give the allocation up for good.
    pub const fn release(&mut self) {
        self.residency = Residency::Released;
        self.written = false;
    }
}
