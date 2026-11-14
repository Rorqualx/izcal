//! `hg-fragment`: the fragment store. L4, tier R.
//!
//! Layout is a pure function from recorded inputs to an **immutable
//! fragment tree**. This crate owns that tree: the records, the read
//! API every consumer uses, the side tables, and reclamation.
//!
//! **G38.** Paint, `hg-hit`, `hg-layout-query`, `hg-semantic` and
//! `hg-inspect-renderer` depend on this crate's read API and never on
//! `hg-layout` or `hg-box-tree` internals. That is what lets the
//! store's representation change without touching a consumer.
//!
//! # Identity, which is the subtle part (LY-D18, G45)
//!
//! A fragment id is a **store index**. An incremental pass keeps the
//! ids of the subtrees it reused; a from-scratch pass allocates all of
//! them fresh. So **ids depend on history**, and two runs that produce
//! the same page can hold different ids for the same box.
//!
//! Three rules follow, and they are the whole of the design:
//!
//! 1. **Ids are never compared or hashed.** What is compared is the
//!    [`canonical`] tree — a pre-order serialization with geometry,
//!    side-table contents and break tokens inlined and no ids or store
//!    offsets anywhere. DET-1's fragment-tree hash, LY-3, LY-32 and
//!    LY-40 all use it.
//! 2. **Every reference to a fragment carries an epoch**
//!    ([`Handle`]), checked on use, so a reclaimed slot cannot be read
//!    through a stale reference. That includes the store's **own child
//!    links**: a parent that outlives a reclaimed child holds a handle
//!    that no longer resolves, rather than a bare index that the next
//!    fragment to take the slot would answer to. Without it a reused
//!    slot could be given its own ancestor as a child, and a walk of
//!    that tree would never end — reachable from content, so a denial
//!    of service.
//! 3. **Primary state stores no fragment id at all.** `hg-layout-state`
//!    and `hg-scroll` hold boxes, not fragments.
//!
//! # Reclamation
//!
//! The store is class (a) derived data (FP §6.4): dropping it and
//! laying out again gives a bit-identical tree (LY-32). Freed slots go
//! on a free list and are reused with the epoch stepped. Footprint
//! counts **live** records only, so canonical bytes do not depend on
//! how much the page has churned (FP-24).

use std::collections::BTreeSet;

use hg_types::geometry::{LayoutUnit, LogicalRect, LogicalSize};

/// A box's identity in `hg-box-tree`'s columns.
///
/// Carried here so a consumer can go from a fragment back to the box
/// that produced it without this crate depending on `hg-box-tree`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BoxId(pub u32);

/// A break token's identity in the token side table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BreakTokenId(pub u32);

/// A fragment's index in the store.
///
/// **Not an identity across runs.** See the crate docs: ids depend on
/// history, so they are never compared, hashed or stored in primary
/// state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FragmentId(u32);

impl FragmentId {
    /// The index, for a store lookup.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// A fragment reference that survives the pass that made it (G45).
///
/// The epoch is what makes it safe: a reclaimed slot is reused with a
/// stepped epoch, so a handle to the old occupant no longer resolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle {
    id: FragmentId,
    epoch: u32,
}

impl Handle {
    /// The fragment this handle names, if it is still live.
    #[must_use]
    pub const fn id(self) -> FragmentId {
        self.id
    }

    /// The epoch it was taken in.
    #[must_use]
    pub const fn epoch(self) -> u32 {
        self.epoch
    }
}

/// What a fragment is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FragmentKind {
    /// A box fragment: one fragment of a box's border box.
    #[default]
    Box,
    /// A line box.
    Line,
    /// A run of text, whose glyphs live in a side table.
    Text,
}

/// Where a box's baselines sit, measured from its block-start edge.
///
/// `None` means the box has no baseline of that kind, which is not the
/// same as a baseline at zero — alignment asks which, and conflating
/// them is a classic bug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Baselines {
    /// The first baseline.
    pub first: Option<LayoutUnit>,
    /// The last baseline.
    pub last: Option<LayoutUnit>,
}

/// One immutable fragment.
///
/// Children are a contiguous run in the store's child array, so a
/// fragment is a fixed size and a walk over children is one slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    /// The box that produced it.
    pub box_id: BoxId,
    /// What kind of fragment it is.
    pub kind: FragmentKind,
    /// Its border-box rectangle, in its parent's coordinates and its
    /// parent's writing mode.
    pub rect: LogicalRect,
    /// Its baselines.
    pub baselines: Baselines,
    /// Its ink and scrollable overflow, relative to its own origin.
    pub overflow: LogicalRect,
    /// The break token it ended with, when it broke.
    pub break_token: Option<BreakTokenId>,
    /// Whether this is the last fragment of its box.
    pub is_last: bool,
    /// The children, as a run in the store's child array.
    children: Children,
}

/// A contiguous run of children in the store's child array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Children {
    start: u32,
    len: u32,
}

/// What went wrong reading the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The id names no slot.
    NoSuchFragment,
    /// The slot is free.
    Reclaimed,
    /// The store has no room for another fragment or child link.
    Full,
    /// The handle's epoch does not match the slot's: the fragment it
    /// named was reclaimed and the slot reused.
    StaleHandle,
}

/// One slot of the store.
#[derive(Debug, Clone)]
struct Slot {
    /// `None` when the slot is free.
    fragment: Option<Fragment>,
    /// Stepped every time the slot is reused.
    epoch: u32,
}

/// The fragment store.
#[derive(Debug, Clone, Default)]
pub struct Store {
    slots: Vec<Slot>,
    /// Child links, as epoch-checked handles. Append-only within a
    /// pass; [`Store::clear`] is what reclaims it, because the store
    /// is class (a) derived data that is dropped and rebuilt whole.
    children: Vec<Handle>,
    free: Vec<FragmentId>,
}

impl Store {
    /// An empty store.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            children: Vec::new(),
            free: Vec::new(),
        }
    }

    /// How many fragments are live.
    ///
    /// Footprint counts live records only (FP-24), so this is the
    /// number the budget ledger uses — not `capacity`, which depends
    /// on how much the page has churned.
    #[must_use]
    pub fn live(&self) -> usize {
        self.slots.iter().filter(|s| s.fragment.is_some()).count()
    }

    /// How many slots the store holds, live and free.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// How many child links the store holds.
    ///
    /// The child array is append-only within a pass: reclaiming a
    /// fragment frees its slot but not its run of links, because the
    /// run is identified by an offset that nothing else may move. The
    /// store is class (a) derived data (FP §6.4), so what reclaims the
    /// array is [`Store::clear`] and a rebuild — which gives a
    /// bit-identical tree (LY-32).
    #[must_use]
    pub fn child_links(&self) -> usize {
        self.children.len()
    }

    /// Drops everything. The store is derived data; rebuilding gives a
    /// bit-identical tree.
    pub fn clear(&mut self) {
        self.slots.clear();
        self.children.clear();
        self.free.clear();
    }

    /// Adds a fragment with the given children, in order.
    ///
    /// # Errors
    /// [`Error::NoSuchFragment`] when a child id names no live
    /// fragment, so a tree can never be built over a reclaimed slot,
    /// and [`Error::Full`] when an index would not fit — saturating
    /// there would corrupt a run's offset rather than refuse.
    pub fn push(
        &mut self,
        mut fragment: Fragment,
        children: &[FragmentId],
    ) -> Result<FragmentId, Error> {
        // Resolved now, so a child reclaimed later leaves a handle
        // that no longer resolves instead of an index the next
        // occupant of that slot would answer to.
        let links: Vec<Handle> = children
            .iter()
            .map(|c| self.handle(*c))
            .collect::<Result<_, _>>()?;
        let start = u32::try_from(self.children.len()).ok().ok_or(Error::Full)?;
        let len = u32::try_from(links.len()).ok().ok_or(Error::Full)?;
        self.children.extend_from_slice(&links);
        fragment.children = Children { start, len };
        if let Some(id) = self.free.pop() {
            let at = usize::try_from(id.index())
                .ok()
                .ok_or(Error::NoSuchFragment)?;
            let slot = self.slots.get_mut(at).ok_or(Error::NoSuchFragment)?;
            slot.fragment = Some(fragment);
            return Ok(id);
        }
        let next = u32::try_from(self.slots.len()).ok().ok_or(Error::Full)?;
        self.slots.push(Slot {
            fragment: Some(fragment),
            epoch: 0,
        });
        Ok(FragmentId(next))
    }

    /// The fragment at `id`.
    ///
    /// # Errors
    /// [`Error::NoSuchFragment`] or [`Error::Reclaimed`].
    pub fn get(&self, id: FragmentId) -> Result<&Fragment, Error> {
        let at = usize::try_from(id.index())
            .ok()
            .ok_or(Error::NoSuchFragment)?;
        let slot = self.slots.get(at).ok_or(Error::NoSuchFragment)?;
        slot.fragment.as_ref().ok_or(Error::Reclaimed)
    }

    /// A handle to `id`, for holding beyond this pass (G45).
    ///
    /// # Errors
    /// As [`Store::get`].
    pub fn handle(&self, id: FragmentId) -> Result<Handle, Error> {
        let at = usize::try_from(id.index())
            .ok()
            .ok_or(Error::NoSuchFragment)?;
        let slot = self.slots.get(at).ok_or(Error::NoSuchFragment)?;
        if slot.fragment.is_none() {
            return Err(Error::Reclaimed);
        }
        Ok(Handle {
            id,
            epoch: slot.epoch,
        })
    }

    /// The fragment a handle names, checking the epoch.
    ///
    /// # Errors
    /// [`Error::StaleHandle`] when the slot has been reused since —
    /// which is the whole reason a handle is not an id.
    pub fn resolve(&self, handle: Handle) -> Result<&Fragment, Error> {
        let at = usize::try_from(handle.id.index())
            .ok()
            .ok_or(Error::NoSuchFragment)?;
        let slot = self.slots.get(at).ok_or(Error::NoSuchFragment)?;
        if slot.epoch != handle.epoch {
            return Err(Error::StaleHandle);
        }
        slot.fragment.as_ref().ok_or(Error::Reclaimed)
    }

    /// The children of `id`, in order, as epoch-checked handles.
    ///
    /// A handle rather than an id, because a child may have been
    /// reclaimed since: [`Store::resolve`] then says so rather than
    /// handing back whatever now occupies the slot.
    #[must_use]
    pub fn children(&self, id: FragmentId) -> &[Handle] {
        let Ok(fragment) = self.get(id) else {
            return &[];
        };
        let start = usize::try_from(fragment.children.start).unwrap_or(usize::MAX);
        let len = usize::try_from(fragment.children.len).unwrap_or(0);
        let end = start.saturating_add(len);
        self.children.get(start..end).unwrap_or(&[])
    }

    /// The children of `id` that are still live, in order.
    #[must_use]
    pub fn live_children(&self, id: FragmentId) -> Vec<FragmentId> {
        self.children(id)
            .iter()
            .filter(|h| self.resolve(**h).is_ok())
            .map(|h| h.id())
            .collect()
    }

    /// Reclaims a fragment's slot, stepping its epoch so every handle
    /// to it goes stale.
    ///
    /// Children are **not** reclaimed: a reused subtree is shared, so
    /// reclaiming reachable children would cut a live tree. Dropping
    /// a whole tree is [`Store::reclaim_tree`].
    ///
    /// # Errors
    /// [`Error::NoSuchFragment`] or [`Error::Reclaimed`].
    pub fn reclaim(&mut self, id: FragmentId) -> Result<(), Error> {
        let at = usize::try_from(id.index())
            .ok()
            .ok_or(Error::NoSuchFragment)?;
        let slot = self.slots.get_mut(at).ok_or(Error::NoSuchFragment)?;
        if slot.fragment.is_none() {
            return Err(Error::Reclaimed);
        }
        slot.fragment = None;
        slot.epoch = slot.epoch.wrapping_add(1);
        self.free.push(id);
        Ok(())
    }

    /// Reclaims `root` and everything below it, each slot once.
    ///
    /// # Errors
    /// As [`Store::reclaim`], for the root.
    pub fn reclaim_tree(&mut self, root: FragmentId) -> Result<usize, Error> {
        // A `BTreeSet` rather than a scan of what has been seen: a
        // fragment tree is as large as the page, and an O(n²) visited
        // check is a processor denial of service on content.
        let mut seen: BTreeSet<FragmentId> = BTreeSet::new();
        let mut order = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            if self.get(id).is_err() {
                continue;
            }
            order.push(id);
            stack.extend(self.live_children(id));
        }
        let mut n = 0_usize;
        for id in order {
            if self.reclaim(id).is_ok() {
                n = n.saturating_add(1);
            }
        }
        if n == 0 {
            return Err(Error::NoSuchFragment);
        }
        Ok(n)
    }

    /// Every fragment of the tree at `root`, in pre-order.
    ///
    /// Each fragment once. Epoch-checked child links make a cycle
    /// impossible, and the visited set makes a walk that met one
    /// terminate anyway — a walk reachable from content must not be
    /// able to run forever whatever the store holds.
    #[must_use]
    pub fn pre_order(&self, root: FragmentId) -> Vec<FragmentId> {
        let mut seen: BTreeSet<FragmentId> = BTreeSet::new();
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if !seen.insert(id) || self.get(id).is_err() {
                continue;
            }
            out.push(id);
            let mut kids = self.live_children(id);
            kids.reverse();
            stack.extend(kids);
        }
        out
    }
}

/// A fragment as the canonical tree records it: **no id anywhere**.
///
/// This is what LY-D18 means by "an id-free canonical fragment tree".
/// Two runs that laid out the same page produce the same canonical
/// tree whatever ids their stores happened to allocate, so it is what
/// DET-1's hash and every reference comparison use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalFragment {
    /// The box that produced it.
    pub box_id: BoxId,
    /// What kind of fragment it is.
    pub kind: FragmentKind,
    /// Its rectangle.
    pub rect: LogicalRect,
    /// Its baselines.
    pub baselines: Baselines,
    /// Its overflow.
    pub overflow: LogicalRect,
    /// Whether it broke, and whether it is the last fragment.
    ///
    /// The break **token's contents** are inlined by the caller that
    /// owns the token table; what is recorded here is that there was
    /// one, because an id is not comparable.
    pub broke: bool,
    /// Whether this is the last fragment of its box.
    pub is_last: bool,
    /// How many children it has — the child order is the position of
    /// the records that follow in pre-order.
    pub children: u32,
}

/// The canonical fragment tree of `root`: a pre-order serialization
/// with no ids and no store offsets (LY-D18).
#[must_use]
pub fn canonical(store: &Store, root: FragmentId) -> Vec<CanonicalFragment> {
    store
        .pre_order(root)
        .into_iter()
        .filter_map(|id| {
            let f = store.get(id).ok()?;
            Some(CanonicalFragment {
                box_id: f.box_id,
                kind: f.kind,
                rect: f.rect,
                baselines: f.baselines,
                overflow: f.overflow,
                broke: f.break_token.is_some(),
                is_last: f.is_last,
                children: f.children.len,
            })
        })
        .collect()
}

impl Fragment {
    /// A box fragment of `size` at the origin, with no children.
    #[must_use]
    pub fn box_fragment(box_id: BoxId, size: LogicalSize) -> Self {
        Self {
            box_id,
            kind: FragmentKind::Box,
            rect: LogicalRect::new(hg_types::geometry::LogicalPoint::default(), size),
            baselines: Baselines::default(),
            overflow: LogicalRect::new(hg_types::geometry::LogicalPoint::default(), size),
            break_token: None,
            is_last: true,
            children: Children::default(),
        }
    }

    /// How many children it has.
    #[must_use]
    pub const fn child_count(&self) -> u32 {
        self.children.len
    }
}
