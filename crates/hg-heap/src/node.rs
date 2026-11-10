//! The node space: an external, non-moving space the one collector
//! traces (JS-D4 (a); JS spec 6.2 and 6.5; JS-M0 plan Phase 2 item 7).
//!
//! Node structure lives **outside the cage**, in columns the embedder
//! owns, indexed by a 32-bit slot with a 32-bit epoch. The collector
//! owns two things about it: a **side mark bitmap**, one bit per slot,
//! and the **worklist**. The space owns everything else and answers
//! questions about its own slots through [`NodeSpace`], a pull interface:
//! `hg-heap` asks, the space answers.
//!
//! Why a pull interface and not a registry of slot addresses the space
//! hands over — the three written constraints admit only one shape:
//!
//! * `hg-node-store` has **no `hg-heap` dependency** (JS spec 6.2), so it
//!   cannot hold `hg-heap` types in its columns;
//! * external traced slots **are** `hg-heap` field types held outside the
//!   cage (JS spec 6.2), so they cannot be plain words the embedder
//!   invents;
//! * the wrapper's slot is **a field of the node's rare data** (DOM spec
//!   6.4), and the rare-data tables belong to `hg-dom`, which may depend
//!   on `hg-heap`.
//!
//! Together those say the links come to the collector through a trait
//! implemented over dependency-free columns, while the external traced
//! slots are [`ExternalSlot`]s living in the embedder's rare-data tables.
//! A push registry of slot addresses would force `hg-node-store` itself
//! to hold `hg-heap` types, which the first constraint forbids.
//!
//! The real space is `hg-dom` over `hg-node-store`, on the DOM track at
//! M2. JS-M0 builds this interface and the [`TestNodeSpace`] that
//! exercises it.
//!
//! No `unsafe` code: the columns are ordinary Rust data outside the cage,
//! and every cage access still goes through the `cage` module.

use core::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::heap::{Gc, Mutator};
use crate::object::Referent;
use crate::slot::Slot;

/// The ordering every traced slot uses (JS spec 6.5: mutator stores and
/// marker loads are `Relaxed`).
const ORD: Ordering = Ordering::Relaxed;

/// A reference to a node of the heap's node space: a 32-bit slot index
/// and the 32-bit epoch it was taken at (heap design H-4 as amended by
/// decision 1, and Stage 1 5.2's `NodeId`).
///
/// Slot 0 is never a node, so [`NodeRef::NONE`] is the empty reference.
/// A `NodeRef` read out of the cage is untrusted (H-2): the collector
/// bounds-checks the index and compares the epoch with the space's own
/// before following it, and the binding path re-checks it on every call
/// (JS spec 6.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NodeRef {
    /// The node's slot index.
    pub id: u32,
    /// The epoch the reference was taken at.
    pub epoch: u32,
}

impl NodeRef {
    /// The empty reference.
    pub const NONE: Self = Self { id: 0, epoch: 0 };

    /// The reference to slot `id` at `epoch`.
    pub const fn new(id: u32, epoch: u32) -> Self {
        Self { id, epoch }
    }

    /// Whether this is the empty reference.
    pub const fn is_none(self) -> bool {
        self.id == 0
    }

    /// The reference as the 64 bits a node field holds.
    pub const fn to_bits(self) -> u64 {
        ((self.epoch as u64) << 32) | self.id as u64
    }

    /// The reference from 64 bits, which may be any bits at all when they
    /// come out of the cage (H-2).
    pub const fn from_bits(bits: u64) -> Self {
        Self {
            id: (bits & 0xFFFF_FFFF) as u32,
            epoch: (bits >> 32) as u32,
        }
    }
}

/// A traced reference held **outside** the cage: JS spec 6.2's "external
/// traced slot". The wrapper of a node, an event listener in its rare
/// data, a per-world wrapper-map entry are all of these.
///
/// Its storage is an `AtomicU32`, like every traced slot (JS spec 6.5),
/// and its value is reached only through a branded [`Mutator`], so the
/// embedder cannot dereference it raw. [`ExternalSlot::set`] is the one
/// place an external edge is written, which is where the SATB barrier
/// will sit when concurrent marking arrives.
///
/// # The obligation this type carries, and how it is checked
///
/// A cage-side [`Field`](crate::Traced) is traced because its object's
/// layout declares it. An external slot has no layout: it is traced
/// because the space's [`NodeSpace::trace`] visits it. **Every external
/// slot the space owns must be visited by `trace` for the node that owns
/// it.** A slot that `trace` never visits is never updated, so after a
/// cycle that moved its target [`ExternalSlot::get`] would return a
/// reference to where the object used to be. H-2 keeps that inside the
/// cage — a wrong answer, not an invalid access — but it is still a wrong
/// answer the rooting rules are supposed to make impossible, and the type
/// system cannot see it: the space's own `trace` is the only thing that
/// knows which slots a node owns.
///
/// What the type system cannot check, a debug build does. Each slot
/// remembers the cycle in which it was last written or traced.
/// [`ExternalSlot::set`] stamps it with the number of cycles completed so
/// far; the collector stamps it again as it traces it; and
/// [`ExternalSlot::get`] asserts that the stamp is not older than the
/// last completed cycle. A slot the space forgot to visit therefore
/// panics on the first read after a cycle, naming the fault, instead of
/// quietly answering with a stale offset. The stamp is a second
/// `AtomicU32` present only in debug builds, so a release build pays
/// nothing and the slot stays four bytes.
///
/// The check cannot tell that fault from the other reason a live-looking
/// slot goes untraced: its node died in the last cycle and the embedder
/// has not released it yet. That read was already a wrong answer — see
/// [`ExternalSlot::get`] — so it is right that it now reports instead of
/// answering, and the panic names both causes. An embedder that reads
/// its nodes' slots between a cycle and
/// [`Mutator::take_node_releases`](crate::Mutator::take_node_releases)
/// will meet it, which is the intended outcome and not a false positive.
///
/// A debug build also catches the cross-heap half of the same hazard:
/// [`ExternalSlot::set`] asserts that the reference it is given belongs
/// to the writing mutator's own heap.
#[derive(Debug, Default)]
pub struct ExternalSlot<T: Referent> {
    cell: ExternalCell,
    _t: PhantomData<fn(T) -> T>,
}

impl<T: Referent> ExternalSlot<T> {
    /// An empty slot.
    pub const fn empty() -> Self {
        Self {
            cell: ExternalCell::empty(),
            _t: PhantomData,
        }
    }

    /// The reference this slot holds, for as long as the mutator is
    /// borrowed; `None` if it holds none.
    ///
    /// Read it only for a node that is still live. Between the cycle that
    /// finds a node dead and the embedder's release of it
    /// ([`Mutator::take_node_releases`](crate::Mutator::take_node_releases)),
    /// the node's slots were not traced, so this would return a reference
    /// to an object that has been swept or moved — a wrong answer inside
    /// the cage, which H-2 allows, but a wrong answer. The release step
    /// clears the slots, which closes the window.
    ///
    /// # Panics
    /// In a debug build, if the slot holds a reference the last cycle did
    /// not trace: either a node space whose `trace` does not visit the
    /// slot (see this type's documentation) or a read inside the release
    /// window above. The panic names both, because the stamp cannot tell
    /// them apart and both are faults.
    pub fn get<'h, 'm>(&self, m: &'m Mutator<'h>) -> Option<Gc<'h, 'm, T>> {
        let s = self.cell.load();
        if !s.is_reference() {
            return None;
        }
        debug_assert!(
            self.cell.seen() >= m.cycles_run(),
            "an external traced slot holding a reference was not traced by \
             the last cycle: either the node space's `trace` does not visit \
             it, or its node is dead and the embedder has not released it yet"
        );
        Some(Gc::from_slot(s))
    }

    /// Points this slot at `v`. The barrier's place for external edges.
    ///
    /// # Panics
    /// In a debug build, if `v` does not belong to `m`'s heap. The brands
    /// cannot catch this the way they catch a cross-heap cage store: the
    /// slot lives in a table that outlives any one `Heap::enter`, so it
    /// cannot carry a per-`enter` brand. The assertion stands in for it,
    /// as soundness step 6's debug check stands in for the same thing at
    /// a GC point.
    pub fn set<'h>(&self, m: &Mutator<'h>, v: Option<Gc<'h, '_, T>>) {
        debug_assert!(
            v.is_none_or(|g| m.owns(g.slot())),
            "an external traced slot was given a reference from another heap"
        );
        self.cell.store(v.map_or(Slot::ZERO, |g| g.slot()));
        // The next cycle is the one that must visit it, so the stamp is
        // the count of cycles already completed: after that next cycle,
        // a slot it traced carries the new count and a slot it missed
        // still carries this one.
        self.cell.stamp(m.cycles_run());
    }

    /// The untyped view the collector traces through.
    pub fn cell(&self) -> &ExternalCell {
        &self.cell
    }
}

/// The untyped view of an [`ExternalSlot`], which is all the collector
/// needs: it reads the slot, follows the reference, and writes back
/// wherever the target now lives, without knowing its type.
#[derive(Debug, Default)]
pub struct ExternalCell {
    slot: AtomicU32,
    /// The cycle in which this slot was last written or traced, for the
    /// debug check in [`ExternalSlot::get`]. Debug builds only, so a
    /// release build's slot is four bytes.
    #[cfg(debug_assertions)]
    seen: AtomicU32,
}

impl ExternalCell {
    /// An empty cell.
    pub const fn empty() -> Self {
        Self {
            slot: AtomicU32::new(0),
            #[cfg(debug_assertions)]
            seen: AtomicU32::new(0),
        }
    }

    pub(crate) fn load(&self) -> Slot {
        Slot::from_raw(self.slot.load(ORD))
    }

    pub(crate) fn store(&self, s: Slot) {
        self.slot.store(s.raw(), ORD);
    }

    /// Records that this slot was written or traced in cycle `cycle`.
    pub(crate) fn stamp(&self, cycle: u32) {
        #[cfg(debug_assertions)]
        self.seen.store(cycle, ORD);
        #[cfg(not(debug_assertions))]
        let _ = cycle;
    }

    /// The cycle this slot was last written or traced in. A release
    /// build keeps no stamp, so it answers `u32::MAX` and the check in
    /// [`ExternalSlot::get`] is vacuous there — as `debug_assert!`
    /// already makes it.
    pub(crate) fn seen(&self) -> u32 {
        #[cfg(debug_assertions)]
        return self.seen.load(ORD);
        #[cfg(not(debug_assertions))]
        return u32::MAX;
    }
}

/// What the collector calls as it traces one node's edges.
pub trait NodeVisitor {
    /// A link to another node slot: one of the five tree links, or any
    /// other node-to-node edge the space holds.
    fn node(&mut self, id: u32);

    /// An external traced slot this node holds.
    fn external(&mut self, cell: &ExternalCell);
}

/// An external node space the one collector traces (JS-D4 (a)).
///
/// Every method takes `&self`: the columns and the external slots are
/// atomics, so the embedder needs no mutable borrow to write a link, and
/// the collector needs none to trace one. That is what lets one `Rc` be
/// shared between the two without a `RefCell` whose borrow could straddle
/// a GC point.
pub trait NodeSpace {
    /// One past the highest slot index the space can hand out. The
    /// collector sizes its side mark bitmap from this and bounds-checks
    /// every index it reads out of the cage against it (H-2).
    fn slots(&self) -> u32;

    /// The epoch of slot `id`, or `None` if the slot holds no live node.
    /// The collector compares this with the epoch in a [`NodeRef`] before
    /// following it, so a stale or forged reference is not followed.
    fn epoch(&self, id: u32) -> Option<u32>;

    /// Calls `v` once for every edge slot `id` holds: `v.node` for each
    /// link to another slot, `v.external` for each external traced slot.
    /// Called only for a slot the collector has just marked live.
    ///
    /// **This must visit every external traced slot the node owns.** An
    /// [`ExternalSlot`] it does not visit is not traced, and so is not
    /// updated when its target moves; see that type's documentation.
    fn trace(&self, id: u32, v: &mut dyn NodeVisitor);
}

/// A handle to a node root: an entry in the heap's node-root list, which
/// is what keeps a node alive that nothing else names — the document, and
/// any detached subtree the embedder holds by ID alone (decision 3).
///
/// The list is the embedder's to manage, exactly as the kept-alive list
/// is: [`Mutator::remove_node_root`](crate::Mutator::remove_node_root)
/// releases one and
/// [`Mutator::clear_node_roots`](crate::Mutator::clear_node_roots)
/// empties the list at a defined point.
///
/// The handle carries a generation as well as an index, for the same
/// reason a [`NodeRef`] does: entries are reused, so without one a stale
/// or twice-removed handle would unroot whichever node now holds that
/// index — and unrooting a live node releases it. A handle whose
/// generation does not match its entry is ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeRootId {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

/// One entry of the node-root list.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NodeRootEntry {
    /// What the entry roots, or [`NodeRef::NONE`] when it is free.
    pub(crate) node: NodeRef,
    /// Raised each time the entry is reused.
    pub(crate) generation: u32,
}

/// A scope for short-lived node roots (decision 3). Roots taken through
/// it are released when it returns, so a pass that needs a set of nodes
/// alive for its own duration — a tree walk, a range's endpoints while a
/// mutation runs — does not have to pair every `add` with a `remove`.
/// Long-lived roots, like the document's, use
/// [`Mutator::add_node_root`](crate::Mutator::add_node_root) directly.
#[derive(Debug, Default)]
pub struct NodeScope {
    taken: core::cell::RefCell<Vec<NodeRootId>>,
}

impl NodeScope {
    /// Roots `r` for as long as this scope lives.
    pub fn root(&self, m: &Mutator<'_>, r: NodeRef) -> NodeRootId {
        let id = m.add_node_root(r);
        self.taken.borrow_mut().push(id);
        id
    }

    /// The roots this scope holds, for the mutator to release on exit.
    pub(crate) fn take(&self) -> Vec<NodeRootId> {
        core::mem::take(&mut *self.taken.borrow_mut())
    }
}

/// One bit per node slot: the side mark bitmap of JS spec 6.2. It lives
/// here, outside the cage and outside the node space, for the reason
/// `Space`'s own metadata does — corrupt cage contents can never reach
/// the collector's bookkeeping (H-2) — and because `hg-node-store` may
/// not depend on `hg-heap` and so cannot hold it.
#[derive(Debug, Default)]
pub(crate) struct NodeMarks {
    bits: Vec<u64>,
}

impl NodeMarks {
    /// A bitmap with room for `slots` slots, every bit clear.
    pub(crate) fn for_slots(slots: u32) -> Self {
        Self {
            bits: vec![0u64; (slots as usize).div_ceil(64)],
        }
    }

    /// Marks `id`; returns whether it was already marked.
    pub(crate) fn mark(&mut self, id: u32) -> bool {
        let (w, b) = ((id as usize) / 64, (id as usize) % 64);
        match self.bits.get_mut(w) {
            Some(word) => {
                let was = *word & (1 << b) != 0;
                *word |= 1 << b;
                was
            }
            // Out of range: treat as marked, so it is never followed.
            None => true,
        }
    }

    /// Whether `id` is marked.
    pub(crate) fn is_marked(&self, id: u32) -> bool {
        self.bits
            .get((id as usize) / 64)
            .is_some_and(|w| w & (1 << ((id as usize) % 64)) != 0)
    }
}

// --- a test node space ------------------------------------------------------

/// A node space for tests: five link columns, an epoch column, and one
/// external traced slot per node for its wrapper. It is what JS-M0 has in
/// place of `hg-node-store`, which belongs to the DOM track at M2.
///
/// The columns are `AtomicU32` and the methods take `&self`, as the real
/// store's do (JS spec 6.5), so a test can mutate the tree while the
/// heap holds its `Rc`.
#[cfg(any(test, feature = "testing"))]
#[derive(Debug)]
pub struct TestNodeSpace {
    /// parent, first child, last child, previous and next sibling.
    links: [Vec<AtomicU32>; 5],
    /// The generation of each slot, raised each time the slot is reused,
    /// so a reference taken at an older one is detected and never
    /// aliased onto a new node (Stage 1 5.2, release-by-ID).
    generations: Vec<AtomicU32>,
    /// Whether each slot holds a live node. Kept apart from the
    /// generation, because a released slot keeps its generation (the next
    /// use raises it) while holding no node at all.
    live: Vec<AtomicU32>,
    /// One external traced slot per node: its wrapper, held strongly once
    /// created (JS-D4 (a)).
    wrappers: Vec<ExternalSlot<crate::Any>>,
}

#[cfg(any(test, feature = "testing"))]
impl TestNodeSpace {
    /// Which link column is which.
    pub const PARENT: usize = 0;
    /// The first-child column.
    pub const FIRST_CHILD: usize = 1;
    /// The last-child column.
    pub const LAST_CHILD: usize = 2;
    /// The previous-sibling column.
    pub const PREV: usize = 3;
    /// The next-sibling column.
    pub const NEXT: usize = 4;

    /// A space with room for `slots` slots. Slot 0 is never a node.
    pub fn new(slots: u32) -> Self {
        let col = || (0..slots).map(|_| AtomicU32::new(0)).collect::<Vec<_>>();
        Self {
            links: [col(), col(), col(), col(), col()],
            generations: col(),
            live: col(),
            wrappers: (0..slots).map(|_| ExternalSlot::empty()).collect(),
        }
    }

    /// Creates a node in slot `id`, as a parser does, and returns its
    /// reference. The slot's generation is raised, so any reference to
    /// its previous occupant is now stale.
    ///
    /// # Panics
    /// If the slot already holds a live node.
    pub fn create(&self, id: u32) -> NodeRef {
        assert!(!self.is_allocated(id), "slot {id} already holds a node");
        let g = self.generations[id as usize]
            .load(ORD)
            .wrapping_add(1)
            .max(1);
        self.generations[id as usize].store(g, ORD);
        self.live[id as usize].store(1, ORD);
        NodeRef::new(id, g)
    }

    /// The reference to the node in slot `id`, or `None` if the slot
    /// holds none.
    pub fn node_ref(&self, id: u32) -> Option<NodeRef> {
        self.epoch(id).map(|g| NodeRef::new(id, g))
    }

    /// Links `child` under `parent`, as the last child.
    pub fn append(&self, parent: u32, child: u32) {
        self.links[Self::PARENT][child as usize].store(parent, ORD);
        let prev = self.links[Self::LAST_CHILD][parent as usize].load(ORD);
        if prev == 0 {
            self.links[Self::FIRST_CHILD][parent as usize].store(child, ORD);
        } else {
            self.links[Self::PREV][child as usize].store(prev, ORD);
            self.links[Self::NEXT][prev as usize].store(child, ORD);
        }
        self.links[Self::LAST_CHILD][parent as usize].store(child, ORD);
    }

    /// Links `child` under `parent` immediately before `before`, which
    /// must already be a child of `parent`: the node-space half of
    /// `insertBefore`, which CS-D3 criterion 4 names as the mutation to
    /// exercise. `child` must be detached first.
    pub fn insert_before(&self, parent: u32, child: u32, before: u32) {
        let prev = self.links[Self::PREV][before as usize].load(ORD);
        self.links[Self::PARENT][child as usize].store(parent, ORD);
        self.links[Self::NEXT][child as usize].store(before, ORD);
        self.links[Self::PREV][before as usize].store(child, ORD);
        if prev == 0 {
            self.links[Self::FIRST_CHILD][parent as usize].store(child, ORD);
        } else {
            self.links[Self::PREV][child as usize].store(prev, ORD);
            self.links[Self::NEXT][prev as usize].store(child, ORD);
        }
    }

    /// The children of `parent`, in order.
    pub fn children(&self, parent: u32) -> Vec<u32> {
        let mut out = Vec::new();
        let mut at = self.links[Self::FIRST_CHILD][parent as usize].load(ORD);
        while at != 0 && out.len() <= self.live.len() {
            out.push(at);
            at = self.links[Self::NEXT][at as usize].load(ORD);
        }
        out
    }

    /// Unlinks `child` from its parent, leaving it detached. Its own
    /// children stay attached to it.
    pub fn detach(&self, child: u32) {
        let p = self.links[Self::PARENT][child as usize].swap(0, ORD);
        if p == 0 {
            return;
        }
        let (prev, next) = (
            self.links[Self::PREV][child as usize].swap(0, ORD),
            self.links[Self::NEXT][child as usize].swap(0, ORD),
        );
        if prev == 0 {
            self.links[Self::FIRST_CHILD][p as usize].store(next, ORD);
        } else {
            self.links[Self::NEXT][prev as usize].store(next, ORD);
        }
        if next == 0 {
            self.links[Self::LAST_CHILD][p as usize].store(prev, ORD);
        } else {
            self.links[Self::PREV][next as usize].store(prev, ORD);
        }
    }

    /// The node's wrapper slot: an external traced slot, in what stands
    /// in here for the rare-data table (DOM spec 6.4).
    pub fn wrapper(&self, id: u32) -> &ExternalSlot<crate::Any> {
        &self.wrappers[id as usize]
    }

    /// Releases the node `r` names: its links and wrapper slot are
    /// cleared and the slot stops holding a node (JS spec 6.5,
    /// release-by-ID). The embedder calls this for each entry of
    /// [`Mutator::take_node_releases`](crate::Mutator::take_node_releases).
    ///
    /// A reference whose generation does not match the slot's is
    /// **ignored**: that is what the generation is for, and it is why a
    /// stale release can never free a node that has taken the slot since
    /// (Stage 1 5.2, "A stale ID is detected by its epoch, never aliased
    /// onto a new node").
    pub fn release(&self, r: NodeRef) {
        if self.epoch(r.id) != Some(r.epoch) {
            return;
        }
        self.release_slot(r.id);
    }

    /// Releases slot `id` without checking a generation. For a test that
    /// is deliberately simulating a stale release.
    pub fn release_slot(&self, id: u32) {
        for c in &self.links {
            c[id as usize].store(0, ORD);
        }
        self.wrappers[id as usize].cell().store(Slot::ZERO);
        // The generation stays; the next `create` raises it. The slot
        // holds no node now, so `epoch` reports `None` and no reference
        // taken at the old generation is followed.
        self.live[id as usize].store(0, ORD);
    }

    /// Whether slot `id` holds a live node.
    pub fn is_allocated(&self, id: u32) -> bool {
        self.live.get(id as usize).is_some_and(|e| e.load(ORD) != 0)
    }
}

#[cfg(any(test, feature = "testing"))]
impl NodeSpace for TestNodeSpace {
    fn slots(&self) -> u32 {
        u32::try_from(self.live.len()).unwrap_or(u32::MAX)
    }

    fn epoch(&self, id: u32) -> Option<u32> {
        (self.live.get(id as usize)?.load(ORD) != 0)
            .then(|| self.generations[id as usize].load(ORD))
    }

    fn trace(&self, id: u32, v: &mut dyn NodeVisitor) {
        for c in &self.links {
            let t = c[id as usize].load(ORD);
            if t != 0 {
                v.node(t);
            }
        }
        v.external(self.wrappers[id as usize].cell());
    }
}
