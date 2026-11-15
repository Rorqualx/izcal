//! The budget the OS gives, and what may be allocated against it.
//!
//! FP 6.6: "Each GPU process reads the OS budget (DXGI
//! `QueryVideoMemoryInfo` and its change event; Vulkan `heapBudget`;
//! on Metal, the device's recommended working set) and reacts to
//! **budget changes**, not only to system memory pressure", because
//! D3D12 budgets "fluctuate dramatically when the user switches away
//! to another application".
//!
//! # Why being over budget is a state and not an error
//!
//! The budget arrives asynchronously and can shrink **below what the
//! process has already allocated**, with nothing done wrong by
//! anyone. So "over budget" is a condition to drain, not a fault, and
//! the interface has to stay usable inside it. The way that goes
//! wrong is specific and circular: refuse every allocation while over
//! budget and the process cannot allocate the swapchain image it
//! needs in order to present the frame whose completion releases the
//! tiles. The budget then never recovers.
//!
//! [`Class`] is the way out. One class is admitted whatever the
//! budget says, because the alternative is that deadlock; one is
//! admitted while there is room; one is refused first and reclaimed
//! first, which is FP 6.6's "the prepaint margin shrinks first under
//! pressure" stated once for every kind rather than for tiles alone.
//!
//! # Why the class is not a function of the kind
//!
//! A tile inside the viewport and a prepaint tile beyond it are the
//! same [`Kind`](crate::ledger::Kind) — FP 6.6 counts both in the
//! same 2.5-viewport budget — and they are not the same class: the
//! prepaint one is what shrinks first. So the kind answers "which
//! ledger line" and the class answers "what happens under pressure",
//! and a single field cannot answer both. A [`Request`] carries both,
//! and that is the whole reason it exists rather than the caller
//! passing a kind and a size.

use crate::ledger::{Kind, Tag};

/// What the OS says, at the moment it was asked.
///
/// Both numbers are the OS's, not Izcal's: `usage` is what the OS
/// thinks this process has, which is not the ledger's `charged()` —
/// the driver's own allocations are in one and not the other. They
/// are kept apart on purpose, because reconciling them is a
/// measurement (GR-S1) and not an assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    /// The bytes the OS is willing to give this process now.
    pub bytes: u64,
    /// The bytes the OS believes it is already using.
    pub usage: u64,
}

impl Budget {
    /// A budget of nothing, which is a real answer on a device under
    /// pressure and not a placeholder.
    pub const NONE: Self = Self { bytes: 0, usage: 0 };

    /// Whether the OS already reports more usage than budget.
    #[must_use]
    pub const fn is_over(&self) -> bool {
        self.usage > self.bytes
    }

    /// The bytes left, or zero. Never negative, and never wrapping:
    /// a wrapped headroom reads as a device with 16 exabytes free.
    #[must_use]
    pub const fn headroom(&self) -> u64 {
        self.bytes.saturating_sub(self.usage)
    }
}

/// What happens to an allocation under pressure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// Reclaimed first, refused first: the work is worth doing when
    /// there is room and costs only latency when there is not.
    /// Prepaint tiles beyond the viewport, atlas growth beyond the
    /// first 256 KiB, staging.
    Discretionary,
    /// What the visible page needs: viewport tiles, the images in
    /// view, a canvas the page is drawing into.
    Working,
    /// Admitted whatever the budget says, because refusing it
    /// deadlocks the thing that would free memory. Swapchain images,
    /// and nothing that grows with page content.
    Reserved,
}

impl Class {
    /// Every class, cheapest to keep first.
    pub const ALL: [Self; 3] = [Self::Discretionary, Self::Working, Self::Reserved];

    /// The order reclamation walks: lower goes first.
    ///
    /// This is a total order and not a per-back-end judgement, which
    /// is the point of stating it here: three back ends under
    /// pressure should drop the same things, or the same page behaves
    /// differently on Metal and on Vulkan for reasons no test would
    /// catch.
    #[must_use]
    pub const fn reclaim_rank(self) -> u8 {
        match self {
            Self::Discretionary => 0,
            Self::Working => 1,
            Self::Reserved => 2,
        }
    }

    /// Whether memory of this class may ever be reclaimed to make
    /// room.
    #[must_use]
    pub const fn is_reclaimable(self) -> bool {
        !matches!(self, Self::Reserved)
    }
}

/// One allocation asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Request {
    /// Which ledger line it joins.
    pub tag: Tag,
    /// What happens to it under pressure.
    pub class: Class,
    /// How big it is.
    pub bytes: u64,
}

/// Whether an allocation may proceed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Allocate it.
    Admit,
    /// Refuse it. The caller's fallback is its own: re-raster later
    /// for a tile, re-rasterize a glyph for an atlas, fail the
    /// context for WebGPU.
    Refuse,
}

/// Whether `request` may be allocated.
///
/// `charged` is the ledger's own total (live plus pinned, G71) and
/// `reserved_cap` is the room [`Class::Reserved`] is guaranteed —
/// which is why it is a parameter at all. Without it a run of
/// discretionary allocations can fill the budget to the last byte,
/// and the swapchain that must then be admitted puts the process over
/// with no way back. With it, discretionary work is held off the
/// reserved classes' room and the deadlock above cannot start.
///
/// The cap itself is [`crate::caps`]' to compute, and it is never a
/// function of this budget: FP-D19.
#[must_use]
pub const fn admit(
    class: Class,
    charged: u64,
    request: u64,
    budget: &Budget,
    reserved_cap: u64,
) -> Decision {
    // Before the class, deliberately: an unrepresentable total is not
    // the condition `Reserved` exists for. That condition is the
    // presentation deadlock, where admitting over budget is what
    // frees the memory; a sum that does not fit in 64 bits is 18
    // exabytes already charged, where admitting helps nobody. So this
    // refusal reaches every class, `Reserved` included.
    let after = match charged.checked_add(request) {
        Some(n) => n,
        None => return Decision::Refuse,
    };
    match class {
        // Admitted over budget on purpose — but not past the check
        // above. Its bound is its own cap, which the caller checks
        // against geometry, not this budget.
        Class::Reserved => Decision::Admit,
        Class::Working => {
            if after <= budget.bytes {
                Decision::Admit
            } else {
                Decision::Refuse
            }
        }
        Class::Discretionary => {
            if after <= budget.bytes.saturating_sub(reserved_cap) {
                Decision::Admit
            } else {
                Decision::Refuse
            }
        }
    }
}

/// The class a kind takes when nothing else is known.
///
/// A default, not a rule: [`Request`] carries the class because the
/// caller knows things this function cannot, above all whether a tile
/// is in the viewport or beyond it. A kind whose default is wrong for
/// a given allocation is the normal case, not an error.
#[must_use]
pub const fn default_class(kind: Kind) -> Class {
    match kind {
        Kind::Swapchain => Class::Reserved,
        Kind::Tile
        | Kind::ImageTexture
        | Kind::GlyphAtlasA8
        | Kind::GlyphAtlasRgba8
        | Kind::Canvas
        | Kind::WebGpu
        | Kind::Video
        | Kind::ViewTransitionSnapshot
        | Kind::OffscreenCanvasSurface => Class::Working,
        Kind::Staging => Class::Discretionary,
    }
}
