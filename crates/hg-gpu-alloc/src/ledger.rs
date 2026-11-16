//! The ledger tag, and who may name a site.
//!
//! FP 6.6: "Every GPU allocation, in whichever GPU process allocated
//! it, carries a ledger tag (site, container, partition, tab, kind)".
//! GR 6.22 adds that "the compositor's tile and surface allocations
//! are charged to the site whose content they hold", and FP 6.6 says
//! why the tag exists at all: "no GPU API knows which tab owns a
//! texture, so each GPU process does".
//!
//! # Why the tag is not five fields the caller fills in
//!
//! The two GPU process kinds stand in a different relation to the
//! site. A **content GPU process** is one per renderer key (GR 6.2:
//! keyed "exactly like the renderer it serves"), so its site,
//! container and partition are fixed before it allocates anything;
//! they are properties of the process, not of the allocation. The
//! **compositor GPU service** is one per browser and holds every
//! site's composited pixels, so for it the site genuinely varies per
//! allocation.
//!
//! An interface that took all five fields per allocation would
//! therefore be giving the content GPU process a parameter it has no
//! right to vary — and the way that goes wrong is silent: bytes
//! charged to a site that did not ask for them, in the one table
//! attribution reads (FP 6.14). So the scope is a type
//! ([`KeyScope`], [`CompositorScope`]), a key-scoped process has no
//! function that accepts a site, and a [`Ledger`] refuses a tag its
//! own scope could not have made.

use hg_gpu_hal::device::Epoch;

/// The site a tag charges, as the renderer key names it.
///
/// Opaque here on purpose: this crate compares sites for equality and
/// does nothing else with them, and the key's own definition belongs
/// to the process model (GR-D2, OD-5), not to the allocator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SiteId(pub u64);

/// The container the site's renderer runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContainerId(pub u64);

/// The storage partition, as the (T, C, X, n) projection fixes it
/// (FP 6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartitionId(pub u64);

/// Which tab's view these bytes are for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TabId(pub u64);

/// What the allocation holds.
///
/// FP 6.6's list, exactly, and closed: "kind is swapchain, tile,
/// image texture, glyph atlas (A8 or RGBA8 pool), canvas, WebGPU,
/// video (including frames of the hardware-decode path), View
/// Transition snapshot, OffscreenCanvas surface, or staging". The two
/// atlas pools are separate variants because FP 6.6 puts them "each
/// on its own ledger line".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// Presentation images for one surface.
    Swapchain,
    /// A rastered tile of a document.
    Tile,
    /// A decoded image's texture.
    ImageTexture,
    /// The A8 coverage-mask glyph pool of one renderer key and
    /// partition.
    GlyphAtlasA8,
    /// The RGBA8 colour-glyph pool of one renderer key and partition.
    GlyphAtlasRgba8,
    /// A canvas backing surface.
    Canvas,
    /// A WebGPU resource.
    WebGpu,
    /// A video frame, the hardware-decode path's included.
    Video,
    /// A View Transition snapshot.
    ViewTransitionSnapshot,
    /// An `OffscreenCanvas` surface.
    OffscreenCanvasSurface,
    /// An upload or read-back staging buffer.
    Staging,
}

impl Kind {
    /// Every kind, for tests and for reporting a ledger line per kind.
    pub const ALL: [Self; 11] = [
        Self::Swapchain,
        Self::Tile,
        Self::ImageTexture,
        Self::GlyphAtlasA8,
        Self::GlyphAtlasRgba8,
        Self::Canvas,
        Self::WebGpu,
        Self::Video,
        Self::ViewTransitionSnapshot,
        Self::OffscreenCanvasSurface,
        Self::Staging,
    ];
}

/// FP 6.6's five fields, together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tag {
    /// Whose content these bytes are.
    pub site: SiteId,
    /// Which container that site's renderer runs in.
    pub container: ContainerId,
    /// Which storage partition.
    pub partition: PartitionId,
    /// Which tab's view.
    pub tab: TabId,
    /// What the allocation holds.
    pub kind: Kind,
}

/// A content GPU process's scope: one renderer key, fixed at start.
///
/// There is deliberately no method here that takes a [`SiteId`]. A
/// content GPU process cannot name another site's bytes, because the
/// interface gives it no way to say one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyScope {
    site: SiteId,
    container: ContainerId,
    partition: PartitionId,
}

impl KeyScope {
    /// The key this process serves.
    #[must_use]
    pub const fn new(site: SiteId, container: ContainerId, partition: PartitionId) -> Self {
        Self {
            site,
            container,
            partition,
        }
    }

    /// Which site this process is.
    #[must_use]
    pub const fn site(&self) -> SiteId {
        self.site
    }

    /// A tag for this process's own key. The site is not a parameter.
    #[must_use]
    pub const fn tag(&self, kind: Kind, tab: TabId) -> Tag {
        Tag {
            site: self.site,
            container: self.container,
            partition: self.partition,
            tab,
            kind,
        }
    }
}

/// The compositor GPU service's scope: every site, one process.
///
/// GR 6.22 charges its tiles and surfaces "to the site whose content
/// they hold", so here the site *is* a parameter — and the reason it
/// may be is that this process is the one I-1 shared-process
/// exception (G63), which the format it reads is built around.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompositorScope;

impl CompositorScope {
    /// A tag for the site whose content the allocation holds.
    #[must_use]
    pub const fn tag(
        &self,
        site: SiteId,
        container: ContainerId,
        partition: PartitionId,
        kind: Kind,
        tab: TabId,
    ) -> Tag {
        Tag {
            site,
            container,
            partition,
            tab,
            kind,
        }
    }
}

/// Which process this ledger belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// One content GPU process per renderer key.
    Key(KeyScope),
    /// The one compositor GPU service.
    Compositor(CompositorScope),
}

impl Scope {
    /// Whether this scope could have produced `tag`.
    ///
    /// The type already stops a key-scoped caller from *writing*
    /// another site's tag; this catches one that arrived some other
    /// way, which is the case a test can plant.
    #[must_use]
    pub const fn could_have_made(&self, tag: &Tag) -> bool {
        match *self {
            Self::Compositor(_) => true,
            Self::Key(k) => {
                k.site.0 == tag.site.0
                    && k.container.0 == tag.container.0
                    && k.partition.0 == tag.partition.0
            }
        }
    }
}

/// Which allocation, in which epoch.
///
/// G45's shape, which the HAL's `ResourceId` already uses: a handle
/// held past a device loss is an (id, epoch) pair checked on use, not
/// an index that silently names whatever took its place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AllocationId {
    /// Which epoch of the device.
    pub epoch: Epoch,
    /// Which allocation within it.
    pub index: u32,
}

/// What a ledger refused, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerError {
    /// A tag naming a site this scope is not. Only a content GPU
    /// process can hit this, and only with a tag it did not make.
    ForeignSite,
    /// An allocation from another epoch: the device was lost and this
    /// handle names memory that no longer exists.
    WrongEpoch,
    /// An id this ledger does not hold.
    NoSuchAllocation,
    /// An id this ledger already holds. A second row under one id is
    /// a row nothing can release: `grant`, `release` and `unpin` act
    /// on the first match, so the duplicate stays charged until the
    /// epoch resets, and the site is charged twice for memory it has
    /// once.
    AlreadyCharged,
    /// A reset to the current epoch or an older one. An epoch that
    /// does not advance is not an epoch: a `Grant` taken before the
    /// reset would reach a new row that reused its index, which is
    /// G45's (id, epoch) check defeated by the epoch standing still.
    EpochNotNewer,
    /// A second grant of an allocation already granted (G71: "granted
    /// once and never re-granted").
    AlreadyGranted,
    /// The byte total would overflow. An interface that cannot say
    /// this has to either panic or wrap, and a wrapped GPU budget
    /// reads as free memory.
    Overflow,
}

/// What releasing an allocation did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Release {
    /// The bytes are gone and the ledger line shrank.
    Freed,
    /// A holder still pins the texture, so the bytes stay charged
    /// (G71: "a holder that keeps a released texture is charged for
    /// it, so pinning memory is charged and killed at budget").
    Pinned,
}

/// A transfer of a shared texture to one holder.
///
/// Deliberately neither `Clone` nor `Copy`: G71 says a shared texture
/// is "granted once and never re-granted", and a grant that can be
/// duplicated is a grant that can be re-granted. The compiler is a
/// better place to say that than a comment.
#[derive(Debug, PartialEq, Eq)]
pub struct Grant {
    id: AllocationId,
    bytes: u64,
}

impl Grant {
    /// Which allocation was granted.
    #[must_use]
    pub const fn id(&self) -> AllocationId {
        self.id
    }

    /// How many bytes travel with it.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

/// One allocation's row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Row {
    id: AllocationId,
    tag: Tag,
    bytes: u64,
    /// Granted once, ever. G71 is about the grant, not the holding,
    /// so this never goes back to `false` — a texture given back is
    /// still a texture that has been granted.
    granted: bool,
    /// A holder has it now.
    held: bool,
    /// The owner has released it. A released row survives only while
    /// `held`, and that is exactly the pinned case.
    released: bool,
}

/// The per-process table FP 6.6 requires, because no GPU API has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ledger {
    scope: Scope,
    epoch: Epoch,
    rows: Vec<Row>,
}

impl Ledger {
    /// A ledger for one process, in one epoch of its device.
    #[must_use]
    pub const fn new(scope: Scope, epoch: Epoch) -> Self {
        Self {
            scope,
            epoch,
            rows: Vec::new(),
        }
    }

    /// This ledger's scope.
    #[must_use]
    pub const fn scope(&self) -> Scope {
        self.scope
    }

    /// The epoch its allocations belong to.
    #[must_use]
    pub const fn epoch(&self) -> Epoch {
        self.epoch
    }

    /// Record an allocation.
    ///
    /// # Errors
    ///
    /// [`LedgerError::ForeignSite`] for a tag this scope could not
    /// have made, [`LedgerError::WrongEpoch`] for an id from another
    /// epoch, [`LedgerError::AlreadyCharged`] for an id already
    /// here, [`LedgerError::Overflow`] if the total would not fit.
    pub fn charge(&mut self, id: AllocationId, tag: Tag, bytes: u64) -> Result<(), LedgerError> {
        if !self.scope.could_have_made(&tag) {
            return Err(LedgerError::ForeignSite);
        }
        if id.epoch != self.epoch {
            return Err(LedgerError::WrongEpoch);
        }
        if self.rows.iter().any(|r| r.id == id) {
            return Err(LedgerError::AlreadyCharged);
        }
        // The sum must be expressible before the row joins it, and
        // the check cannot use `charged()`, which saturates for
        // reporting: a saturating total would report a full budget as
        // a merely large one and admit the allocation that broke it.
        self.rows
            .iter()
            .try_fold(bytes, |sum, r| sum.checked_add(r.bytes))
            .ok_or(LedgerError::Overflow)?;
        self.rows.push(Row {
            id,
            tag,
            bytes,
            granted: false,
            held: false,
            released: false,
        });
        Ok(())
    }

    /// Grant a shared texture to a holder, once.
    ///
    /// # Errors
    ///
    /// [`LedgerError::AlreadyGranted`] on a second grant (G71),
    /// [`LedgerError::NoSuchAllocation`] for an unknown id,
    /// [`LedgerError::WrongEpoch`] for one from another epoch.
    pub fn grant(&mut self, id: AllocationId) -> Result<Grant, LedgerError> {
        if id.epoch != self.epoch {
            return Err(LedgerError::WrongEpoch);
        }
        let row = self
            .rows
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(LedgerError::NoSuchAllocation)?;
        if row.granted {
            return Err(LedgerError::AlreadyGranted);
        }
        row.granted = true;
        row.held = true;
        Ok(Grant {
            id,
            bytes: row.bytes,
        })
    }

    /// Release an allocation. A granted texture whose holder has not
    /// given it back stays charged.
    ///
    /// # Errors
    ///
    /// [`LedgerError::NoSuchAllocation`] for an unknown id,
    /// [`LedgerError::WrongEpoch`] for one from another epoch.
    pub fn release(&mut self, id: AllocationId) -> Result<Release, LedgerError> {
        if id.epoch != self.epoch {
            return Err(LedgerError::WrongEpoch);
        }
        let position = self
            .rows
            .iter()
            .position(|r| r.id == id)
            .ok_or(LedgerError::NoSuchAllocation)?;
        let held = if let Some(row) = self.rows.get_mut(position) {
            row.released = true;
            row.held
        } else {
            return Err(LedgerError::NoSuchAllocation);
        };
        if held {
            return Ok(Release::Pinned);
        }
        let _ = self.rows.remove(position);
        Ok(Release::Freed)
    }

    /// A holder gives a shared texture back, consuming the grant so
    /// it cannot be given back twice.
    ///
    /// The give-back can arrive before or after the owner's release,
    /// and the two orders must end in the same place: if the owner
    /// has released, the row goes; if it has not, the row stays live
    /// and simply stops being held. What does *not* happen either way
    /// is the row becoming grantable again (G71).
    ///
    /// # Errors
    ///
    /// [`LedgerError::WrongEpoch`] for a grant from another epoch —
    /// the device it names is gone and its index may have been
    /// reused — or [`LedgerError::NoSuchAllocation`] if no held row
    /// has that id.
    pub fn unpin(&mut self, grant: Grant) -> Result<(), LedgerError> {
        if grant.id.epoch != self.epoch {
            return Err(LedgerError::WrongEpoch);
        }
        let position = self
            .rows
            .iter()
            .position(|r| r.id == grant.id && r.held)
            .ok_or(LedgerError::NoSuchAllocation)?;
        let released = if let Some(row) = self.rows.get_mut(position) {
            row.held = false;
            row.released
        } else {
            return Err(LedgerError::NoSuchAllocation);
        };
        if released {
            let _ = self.rows.remove(position);
        }
        Ok(())
    }

    /// The bytes this process is charged: live allocations **plus**
    /// released ones a holder still pins.
    ///
    /// This is the number the budget is compared against, and the
    /// reason it is not simply the sum of live allocations is G71. A
    /// ledger that counted only live rows would report a process
    /// under budget while its pinned textures kept the memory, which
    /// is the under-report that makes pinning free.
    #[must_use]
    pub fn charged(&self) -> u64 {
        self.rows
            .iter()
            .fold(0_u64, |sum, r| sum.saturating_add(r.bytes))
    }

    /// The bytes charged for one site.
    #[must_use]
    pub fn charged_to(&self, site: SiteId) -> u64 {
        self.rows
            .iter()
            .filter(|r| r.tag.site == site)
            .fold(0_u64, |sum, r| sum.saturating_add(r.bytes))
    }

    /// The bytes charged on one kind's ledger line.
    #[must_use]
    pub fn charged_for(&self, kind: Kind) -> u64 {
        self.rows
            .iter()
            .filter(|r| r.tag.kind == kind)
            .fold(0_u64, |sum, r| sum.saturating_add(r.bytes))
    }

    /// The bytes held only by pinning.
    #[must_use]
    pub fn pinned(&self) -> u64 {
        self.rows
            .iter()
            .filter(|r| r.released && r.held)
            .fold(0_u64, |sum, r| sum.saturating_add(r.bytes))
    }

    /// How many allocations the ledger holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether it holds none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Start the next epoch: every allocation of the lost device is
    /// gone, pinned ones included, because the memory is gone.
    ///
    /// The HAL's `DeviceLoss::resolve` answers "you are the one"
    /// exactly once per epoch (6.6), so this runs once per loss and
    /// not once per caller who noticed. The epoch must **advance**,
    /// which is how that once-per-loss property is kept here rather
    /// than assumed: a reset to the epoch it is already in would let
    /// a `Grant` from before the reset reach a new row that reused
    /// its index.
    ///
    /// # Errors
    ///
    /// [`LedgerError::EpochNotNewer`] if `epoch` is the current one
    /// or older.
    pub fn reset_for(&mut self, epoch: Epoch) -> Result<(), LedgerError> {
        if epoch.0 <= self.epoch.0 {
            return Err(LedgerError::EpochNotNewer);
        }
        self.epoch = epoch;
        self.rows.clear();
        Ok(())
    }
}
