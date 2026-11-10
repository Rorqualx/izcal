//! The agent heap, its branded mutator, roots and the collector (JS spec
//! 6.5; heap design H-7 to H-11).

use core::cell::RefCell;
use core::marker::PhantomData;
use std::any::TypeId;
use std::collections::BTreeMap;

use hg_types::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use hg_substrate::{DecisionKind, DecisionLog, Memory, Position};

use crate::cage::{Cage, CageError};
use crate::node::{
    ExternalCell, NodeMarks, NodeRef, NodeRootEntry, NodeRootId, NodeScope, NodeSpace, NodeVisitor,
};
use crate::object::{Any, Layout, Node, Object, Raw, RawValue, Referent, Tail, Traced, Weak};
use crate::slot::Slot;
use crate::space::{AllocError, Space};
use crate::verify::{Audit, Violation};

/// An invariant lifetime marker: the brands `'h` and scopes `'s`.
type Invariant<'a> = PhantomData<fn(&'a ()) -> &'a ()>;

/// What freed cage memory is overwritten with: an odd value, so a stale
/// slot read decodes as a reference that [`Space::is_object`] refuses.
const POISON: u64 = 0xDEAD_BEEF_DEAD_BEEF;

/// How the heap collects beyond its canonical trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stress {
    /// Only canonical cycles.
    None,
    /// The moving test collector: before every allocation, a cycle that
    /// moves every object (JS spec 6.5, step 8).
    MoveOnAlloc,
    /// GC stress: a cycle that moves every object at every safepoint.
    MoveAtSafepoint,
}

/// A heap's configuration.
#[derive(Debug, Clone, Copy)]
pub struct HeapConfig {
    /// The cage's size: a power of two from 1 MiB to 16 GiB. Production
    /// uses 16 GiB (JS-D17); tests use small cages.
    pub cage_bytes: u64,
    /// Canonical allocation units between canonical cycles, as charged
    /// by [`Mutator::charge`] (heap design H-8).
    pub trigger: u64,
    /// Extra, non-canonical cycles for testing.
    pub stress: Stress,
    /// The agent this heap belongs to, for decision positions.
    pub agent: u32,
}

/// Why a heap could not be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeapError {
    /// The cage could not be made.
    Cage(CageError),
}

/// Counters for one heap.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HeapStats {
    /// Canonical cycles run.
    pub canonical_cycles: u64,
    /// Non-canonical cycles run (stress and test modes).
    pub other_cycles: u64,
    /// Objects moved, in all cycles.
    pub moved: u64,
    /// Weak slots cleared, in all canonical cycles.
    pub cleared: u64,
    /// Blocks committed.
    pub blocks: u64,
    /// Blocks holding objects.
    pub blocks_used: u64,
}

pub(crate) struct State {
    pub(crate) cage: Arc<Cage>,
    pub(crate) space: Space,
    /// The type table: index → layout (outside the cage, H-4).
    types: Vec<Layout>,
    type_index: HashMap<TypeId, u32>,
    /// Root slots, one segment per open scope, outermost first. A root
    /// lives in its own scope's segment, so rooting into an outer scope
    /// from inside an inner one survives the inner scope's exit.
    pub(crate) roots: RefCell<Vec<Vec<Slot>>>,
    /// The agent's [[KeptAlive]] list (ECMA-262 9.10).
    pub(crate) kept: RefCell<Vec<Slot>>,
    /// Objects that had a weak slot cleared by a canonical cycle and that
    /// the embedder has not taken yet ([`Mutator::take_cleared`]). Traced
    /// as roots until taken, in the order the cycle cleared them.
    pub(crate) cleared: RefCell<Vec<Slot>>,
    /// The node space this heap traces, if the embedder set one
    /// (JS-D4 (a)). Every method of it takes `&self`, so sharing one
    /// `Rc` with the embedder needs no `RefCell` whose borrow could
    /// straddle a GC point.
    pub(crate) node_space: Option<Rc<dyn NodeSpace>>,
    /// The node-root list (decision 3): what keeps a node alive that
    /// nothing else names. A released entry becomes [`NodeRef::NONE`]
    /// and its index is reused, so a handle never names another node.
    pub(crate) node_roots: RefCell<Vec<NodeRootEntry>>,
    /// Indices of released node-root entries, for reuse.
    node_root_free: RefCell<Vec<u32>>,
    /// The generation given to the next node root. It is per heap, not
    /// per entry, so that `clear_node_roots` cannot hand a new entry a
    /// generation a handle from before the clear already carries.
    node_root_gen: core::cell::Cell<u32>,
    /// Node slots a canonical cycle found unreachable, for the embedder
    /// to release at an event-loop point (JS spec 6.5, release-by-ID).
    node_release: RefCell<Vec<NodeRef>>,
    /// Which (slot, epoch) pairs are already on `node_release`. A slot
    /// the embedder has not released yet is found dead again by every
    /// later cycle, and listing it twice would let a stale duplicate
    /// release whatever has taken the slot by then. Keyed by the epoch as
    /// well as the slot, so that a slot released, reused and dead again
    /// is listed for its new occupant rather than suppressed.
    node_release_pending: RefCell<std::collections::BTreeSet<(u32, u32)>>,
    /// Objects whose layout has weak slots, to clear after a cycle.
    weak_holders: Vec<u64>,
    debt: u64,
    config: HeapConfig,
    decisions: Arc<dyn DecisionLog>,
    progress: u64,
    stats: HeapStats,
    /// Whether anything has been allocated since the last cycle ended.
    /// Allocation marks an object's lines but not its mark bit, so
    /// between a cycle and the next one a live object can legitimately be
    /// unmarked; the verifier skips its mark checks when this is set.
    pub(crate) allocated_since_cycle: bool,
    /// Whether a cycle runs the heap verifier after itself. On by default
    /// in debug builds (JS-M0 plan Phase 2 item 4) and off in release,
    /// where its walk would cost more than it is worth. A fuzzer that
    /// writes arbitrary words into the cage turns it off: see
    /// [`Mutator::set_verify`].
    verify: bool,
}

/// The process's cage (JS spec 6.1, JS-D17): one reservation that every
/// agent heap of the process allocates its chunks from. Cloning it shares
/// the same cage.
#[derive(Clone)]
pub struct ProcessCage(Arc<Cage>);

impl core::fmt::Debug for ProcessCage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ProcessCage").field(&self.0).finish()
    }
}

impl ProcessCage {
    /// Reserves the process's cage: `bytes` is a power of two from 1 MiB
    /// to 16 GiB (production uses 16 GiB, JS-D17; tests use small cages).
    ///
    /// # Errors
    /// [`HeapError::Cage`] if the cage cannot be reserved.
    pub fn new(mem: Arc<dyn Memory>, bytes: u64) -> Result<Self, HeapError> {
        Ok(Self(Arc::new(
            Cage::new(mem, bytes).map_err(HeapError::Cage)?,
        )))
    }
}

/// One agent heap. It is entered for each turn of its agent's work.
pub struct Heap {
    state: State,
}

impl Drop for Heap {
    fn drop(&mut self) {
        let st = &mut self.state;
        st.space.release(&st.cage);
    }
}

impl core::fmt::Debug for Heap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Heap")
            .field("stats", &self.state.stats)
            .finish_non_exhaustive()
    }
}

/// The only way to allocate, collect or reach a GC point in heap `'h`
/// (JS-D18). `!Send` and `!Sync`; `'h` is a fresh, invariant brand per
/// [`Heap::enter`].
pub struct Mutator<'h> {
    state: &'h mut State,
    _brand: PhantomData<fn(&'h ()) -> &'h ()>,
    _not_send: PhantomData<*const ()>,
}

/// An unrooted reference into heap `'h`, valid while the shared borrow
/// `'m` of its mutator lives, so it cannot survive a GC point. Four bytes.
pub struct Gc<'h, 'm, T: Referent> {
    slot: Slot,
    _borrow: PhantomData<&'m Mutator<'h>>,
    _t: PhantomData<fn(T) -> T>,
}

impl<T: Referent> Clone for Gc<'_, '_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Referent> Copy for Gc<'_, '_, T> {}

impl<T: Referent> core::fmt::Debug for Gc<'_, '_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Gc({:#x})", self.slot.raw())
    }
}

impl<T: Referent> PartialEq for Gc<'_, '_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot
    }
}
impl<T: Referent> Eq for Gc<'_, '_, T> {}

/// A root scope of heap `'h`; `'s` is fresh per [`Mutator::scope`].
pub struct Scope<'h, 's> {
    /// Its segment in [`State::roots`]: how many scopes enclose it.
    level: usize,
    _h: Invariant<'h>,
    _s: Invariant<'s>,
}

/// A rooted reference: survives GC points, and cannot outlive scope `'s`.
pub struct Root<'h, 's, T: Referent> {
    /// Its scope's segment, and its place in it.
    level: usize,
    index: usize,
    _h: Invariant<'h>,
    _s: Invariant<'s>,
    _t: PhantomData<fn(T) -> T>,
}

impl<T: Referent> Clone for Root<'_, '_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Referent> Copy for Root<'_, '_, T> {}

/// A JS value as a slot holds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value<'h, 'm> {
    /// A small integer.
    Small(i32),
    /// A reference.
    Ref(Gc<'h, 'm, Any>),
}

impl Heap {
    /// A heap in a cage of its own, reserved through `mem`, logging its
    /// decisions to `decisions`. A process with several agents makes one
    /// [`ProcessCage`] and puts each agent's heap in it with
    /// [`Heap::in_cage`] (JS-D17).
    ///
    /// # Errors
    /// [`HeapError::Cage`] if the cage cannot be reserved.
    pub fn new(
        mem: Arc<dyn Memory>,
        decisions: Arc<dyn DecisionLog>,
        config: HeapConfig,
    ) -> Result<Self, HeapError> {
        let cage = ProcessCage::new(mem, config.cage_bytes)?;
        Ok(Self::in_cage(&cage, decisions, config))
    }

    /// An agent heap in the process's cage: it owns the chunks it takes
    /// from the cage, and gives them all back when it is dropped (JS spec
    /// 6.5: a destroyed agent's heap is released whole, without a trace).
    /// `config.cage_bytes` is ignored; the cage's own size applies.
    pub fn in_cage(
        cage: &ProcessCage,
        decisions: Arc<dyn DecisionLog>,
        config: HeapConfig,
    ) -> Self {
        Self {
            state: State {
                cage: Arc::clone(&cage.0),
                space: Space::new(),
                types: Vec::new(),
                type_index: HashMap::default(),
                roots: RefCell::new(Vec::new()),
                kept: RefCell::new(Vec::new()),
                cleared: RefCell::new(Vec::new()),
                node_space: None,
                node_roots: RefCell::new(Vec::new()),
                node_root_free: RefCell::new(Vec::new()),
                node_root_gen: core::cell::Cell::new(0),
                node_release: RefCell::new(Vec::new()),
                node_release_pending: RefCell::new(std::collections::BTreeSet::new()),
                weak_holders: Vec::new(),
                debt: 0,
                config,
                decisions,
                progress: 0,
                stats: HeapStats::default(),
                allocated_since_cycle: false,
                verify: cfg!(debug_assertions),
            },
        }
    }

    /// Runs `f` with this heap's mutator under a fresh brand.
    pub fn enter<R>(&mut self, f: impl for<'h> FnOnce(&mut Mutator<'h>) -> R) -> R {
        let mut m = Mutator {
            state: &mut self.state,
            _brand: PhantomData,
            _not_send: PhantomData,
        };
        f(&mut m)
    }

    /// The heap's counters.
    pub fn stats(&self) -> HeapStats {
        self.state.stats
    }

    /// Walks the heap from its roots and checks the collector's
    /// invariants (JS-M0 plan Phase 2 item 4). A debug build runs this
    /// after every cycle by itself; call it directly to check the heap at
    /// a point of your choosing, or in a release build.
    ///
    /// Allocation marks an object's lines but not its mark bit, so
    /// between one cycle and the next a live object is legitimately
    /// unmarked. Called after an allocation, this skips the two mark
    /// checks and says so in [`Audit::marks_checked`]; every other check
    /// runs whenever it is called.
    ///
    /// # Errors
    /// The first [`Violation`] found.
    pub fn verify(&self) -> Result<Audit, Violation> {
        crate::verify::verify(&self.state)
    }

    /// Gives this heap the node space it traces (JS-D4 (a)). Set once,
    /// before the heap runs; a heap without one simply traces no nodes.
    pub fn set_node_space(&mut self, space: Rc<dyn NodeSpace>) {
        self.state.node_space = Some(space);
    }

    /// Turns after-every-cycle verification on or off. It starts on in
    /// debug builds and off in release.
    ///
    /// Turn it **off** around an in-cage corruption fuzzer: under
    /// arbitrary cage contents a slot may legitimately name a dead
    /// object, which H-2 allows and the verifier does not (see the
    /// [`verify`](crate::verify) module).
    pub fn set_verify(&mut self, on: bool) {
        self.state.verify = on;
    }
}

impl<'h> Mutator<'h> {
    /// Runs `f` with a fresh root scope; its roots die when it returns.
    pub fn scope<R>(&mut self, f: impl for<'s> FnOnce(&mut Self, &Scope<'h, 's>) -> R) -> R {
        let level = {
            let mut roots = self.state.roots.borrow_mut();
            roots.push(Vec::new());
            roots.len() - 1
        };
        let s = Scope {
            level,
            _h: PhantomData,
            _s: PhantomData,
        };
        let r = f(self, &s);
        self.state.roots.borrow_mut().truncate(level);
        r
    }

    /// Advances the agent's progress counter by `n` (the interpreter
    /// calls this per operation), which positions decision points.
    pub fn advance(&mut self, n: u64) {
        self.state.progress = self.state.progress.wrapping_add(n);
    }

    /// The agent's progress counter.
    pub fn progress(&self) -> u64 {
        self.state.progress
    }

    /// Allocates a zeroed `T` with a tail of `len` elements (0 for a type
    /// without a tail) and roots it in `s`. A GC point. It charges no
    /// canonical units: the operation that allocates charges what its
    /// description says, through [`Mutator::charge`], so the charge never
    /// depends on representation or tier (design review R0-B10).
    ///
    /// # Errors
    /// [`AllocError::OutOfMemory`] if the cage is full.
    pub fn alloc<'s, T: Object>(
        &mut self,
        s: &Scope<'h, 's>,
        len: u32,
    ) -> Result<Root<'h, 's, T>, AllocError> {
        let st = &mut *self.state;
        let layout = T::LAYOUT;
        let len = if layout.tail == Tail::None { 0 } else { len };
        let size = layout.size(len).ok_or(AllocError::OutOfMemory)?;
        if st.config.stress == Stress::MoveOnAlloc {
            st.collect(false);
        }
        let ty = st.type_of::<T>();
        let at = st.space.alloc(&st.cage, size)?;
        st.cage.fill(at, size, 0);
        st.cage.store32(at, ty);
        st.cage.store32(at + 4, len);
        if layout.weak > 0 {
            st.weak_holders.push(at);
        }
        st.allocated_since_cycle = true;
        let slot = Slot::reference(at).ok_or(AllocError::OutOfMemory)?;
        Ok(push_root(&st.roots, s, slot))
    }

    /// The heap's internals, for the corruption tests.
    #[cfg(test)]
    pub(crate) fn state_for_tests(&self) -> &State {
        self.state
    }

    /// Charges `units` canonical allocation units of GC debt (JS spec
    /// 6.9), and runs a canonical cycle when the debt reaches the
    /// trigger. A GC point. Each operation that semantically allocates
    /// calls it with the units its description fixes.
    pub fn charge(&mut self, units: u64) {
        let st = &mut *self.state;
        st.debt = st.debt.saturating_add(units);
        if st.debt >= st.config.trigger {
            st.collect(true);
        }
    }

    /// An explicit safepoint: a GC point where the stress mode moves
    /// every object.
    pub fn safepoint(&mut self) {
        if self.state.config.stress == Stress::MoveAtSafepoint {
            self.state.collect(false);
        }
    }

    /// Runs a canonical cycle now, as if the trigger had been reached.
    pub fn collect(&mut self) {
        self.state.collect(true);
    }

    /// How many cycles this heap has run, canonical and otherwise: the
    /// stamp [`ExternalSlot`](crate::ExternalSlot) compares against, so
    /// that a slot the node space forgot to trace is caught on the first
    /// read after a cycle rather than answering with a stale offset.
    pub(crate) fn cycles_run(&self) -> u32 {
        let st = &self.state.stats;
        u32::try_from(st.canonical_cycles.saturating_add(st.other_cycles)).unwrap_or(u32::MAX)
    }

    /// Whether `s` names an object of this heap, for the debug assertion
    /// in [`ExternalSlot::set`](crate::ExternalSlot::set).
    pub(crate) fn owns(&self, s: Slot) -> bool {
        s.as_reference()
            .is_some_and(|off| self.state.space.is_object(off))
    }

    /// Adds `target` to the agent's kept-alive list ([[KeptAlive]]), as
    /// `WeakRef.prototype.deref` does.
    pub fn keep_alive<T: Referent>(&self, target: Gc<'h, '_, T>) {
        self.state.kept.borrow_mut().push(target.slot);
    }

    /// Empties the kept-alive list (ECMA-262 ClearKeptObjects).
    pub fn clear_kept_objects(&mut self) {
        self.state.kept.borrow_mut().clear();
    }

    /// Takes the objects whose weak slots canonical cycles have cleared
    /// since the last call, rooted in `s`, in the order the cycles cleared
    /// them (allocation order within a cycle). An object appears once per
    /// cycle that cleared at least one of its weak slots. The engine uses
    /// it to find the finalization cells whose targets died (ECMA-262
    /// [Execution](https://tc39.es/ecma262/#sec-weakref-execution)); until
    /// taken, the objects are roots, so a cell cannot be lost between the
    /// cycle and the embedder's look. Not a GC point.
    pub fn take_cleared<'s>(&self, s: &Scope<'h, 's>) -> Vec<Root<'h, 's, Any>> {
        let taken = core::mem::take(&mut *self.state.cleared.borrow_mut());
        taken
            .into_iter()
            .map(|slot| push_root(&self.state.roots, s, slot))
            .collect()
    }

    /// Roots the node `r` so it stays alive while the root lives: the
    /// document, and any detached subtree the embedder holds by ID alone
    /// (decision 3). Not a GC point.
    ///
    /// The list is the embedder's to manage, as the kept-alive list is:
    /// release one with [`Mutator::remove_node_root`] or empty the list
    /// with [`Mutator::clear_node_roots`]. A forgotten root keeps its
    /// node alive, which is the same exposure `keep_alive` already has.
    pub fn add_node_root(&self, r: NodeRef) -> NodeRootId {
        if r.is_none() {
            // Rooting nothing takes no entry, and the handle it returns
            // matches none: an entry holding `NONE` looks free but would
            // never reach the free list, so each call would leak one.
            return NodeRootId {
                index: u32::MAX,
                generation: 0,
            };
        }
        let mut roots = self.state.node_roots.borrow_mut();
        let index = match self.state.node_root_free.borrow_mut().pop() {
            Some(i) => i,
            None => {
                roots.push(NodeRootEntry::default());
                u32::try_from(roots.len() - 1).unwrap_or(u32::MAX)
            }
        };
        let Some(entry) = roots.get_mut(index as usize) else {
            return NodeRootId {
                index,
                generation: 0,
            };
        };
        // The generation comes from the heap's own counter, so it is
        // fresh across a `clear_node_roots` as well as across a reuse of
        // this entry: a handle from before either no longer matches.
        let g = self.state.node_root_gen.get().wrapping_add(1).max(1);
        self.state.node_root_gen.set(g);
        entry.generation = g;
        entry.node = r;
        NodeRootId {
            index,
            generation: g,
        }
    }

    /// Runs `f` with a scope whose node roots are released when it
    /// returns (decision 3), for a set of nodes that need only live as
    /// long as one pass. Not a GC point in itself; `f` may contain any
    /// number of them.
    ///
    /// The roots are released on a normal return only. A panic inside `f`
    /// leaves them in the list, which keeps their nodes alive; shipping
    /// and simulation builds abort on panic (CS-ER-2), so that window
    /// exists only in a test build, where the roots go with the heap.
    pub fn node_scope<R>(&mut self, f: impl FnOnce(&mut Self, &NodeScope) -> R) -> R {
        let scope = NodeScope::default();
        let r = f(self, &scope);
        for id in scope.take() {
            self.remove_node_root(id);
        }
        r
    }

    /// Releases a node root. Its entry is emptied and its index reused,
    /// so a stale [`NodeRootId`] can never name another node.
    pub fn remove_node_root(&self, id: NodeRootId) {
        let mut roots = self.state.node_roots.borrow_mut();
        // A handle whose generation does not match its entry is stale:
        // the entry has been released and reused since, and honouring it
        // would unroot — and so release — another node.
        if let Some(entry) = roots.get_mut(id.index as usize)
            && entry.generation == id.generation
            && !entry.node.is_none()
        {
            entry.node = NodeRef::NONE;
            self.state.node_root_free.borrow_mut().push(id.index);
        }
    }

    /// Empties the node-root list, as `clear_kept_objects` empties the
    /// kept-alive list.
    /// The heap's generation counter is **not** reset, so a handle issued
    /// before the clear cannot match an entry added after it.
    pub fn clear_node_roots(&mut self) {
        self.state.node_roots.borrow_mut().clear();
        self.state.node_root_free.borrow_mut().clear();
    }

    /// How many node roots are live.
    pub fn node_roots_len(&self) -> usize {
        self.state
            .node_roots
            .borrow()
            .iter()
            .filter(|e| !e.node.is_none())
            .count()
    }

    /// Takes the node slots canonical cycles have found unreachable since
    /// the last call (JS spec 6.5, release-by-ID). The embedder releases
    /// each one's payload at an event-loop point between tasks, never
    /// inside a GC point, and bumps its epoch so a stale reference is
    /// detected rather than aliased onto a new node. Not a GC point.
    pub fn take_node_releases(&self) -> Vec<NodeRef> {
        self.state.node_release_pending.borrow_mut().clear();
        core::mem::take(&mut *self.state.node_release.borrow_mut())
    }

    /// The heap's counters, with the block counts as they are now.
    pub fn stats(&self) -> HeapStats {
        let (blocks, used) = self.state.space.census();
        HeapStats {
            blocks: blocks as u64,
            blocks_used: used as u64,
            ..self.state.stats
        }
    }

    /// Walks the heap from its roots and checks the collector's
    /// invariants. Not a GC point. See [`Heap::verify`] for what is
    /// checked when an allocation has happened since the last cycle.
    ///
    /// # Errors
    /// The first [`Violation`] found.
    pub fn verify(&self) -> Result<Audit, Violation> {
        crate::verify::verify(self.state)
    }

    /// Turns after-every-cycle verification on or off for this heap; see
    /// [`Heap::set_verify`].
    pub fn set_verify(&mut self, on: bool) {
        self.state.verify = on;
    }

    /// A digest of everything reachable from the roots and the kept-alive
    /// list, independent of where objects live: objects are numbered in
    /// the order a breadth-first walk from the roots meets them, and each
    /// contributes its type name, length, slots (references as those
    /// numbers, small integers as themselves), raw words and tail. Two
    /// runs that build the same graph give the same digest, however the
    /// collector moved it.
    pub fn digest(&self) -> u64 {
        use core::hash::Hasher as _;
        let st = &*self.state;
        let mut h =
            hg_types::ProcessHasher::with_keys(0x4947_4845_4150_0001, 0x4947_4845_4150_0002);
        let mut ids: BTreeMap<u64, u64> = BTreeMap::new();
        let mut queue: std::collections::VecDeque<u64> = std::collections::VecDeque::new();
        let name = |off: u64,
                    ids: &mut BTreeMap<u64, u64>,
                    queue: &mut std::collections::VecDeque<u64>|
         -> u64 {
            let next = ids.len() as u64;
            *ids.entry(off).or_insert_with(|| {
                queue.push_back(off);
                next
            })
        };
        let slot_word = |s: Slot,
                         ids: &mut BTreeMap<u64, u64>,
                         queue: &mut std::collections::VecDeque<u64>|
         -> u64 {
            match s.as_reference() {
                Some(off) if st.space.is_object(off) => (name(off, ids, queue) << 2) | 1,
                Some(_) => 3, // a reference to nothing (cleared or corrupt)
                None => u64::from(s.raw()) << 2,
            }
        };
        let roots: Vec<Slot> = st
            .roots
            .borrow()
            .iter()
            .flatten()
            .chain(st.kept.borrow().iter())
            .chain(st.cleared.borrow().iter())
            .copied()
            .collect();
        h.write_u64(roots.len() as u64);
        for s in roots {
            h.write_u64(slot_word(s, &mut ids, &mut queue));
        }
        // The node space. Node slots never move (JS-D4 (a): the columns
        // are non-moving), so walking the **reachable** ones in slot
        // order is stable, and two runs that build the same tree hash the
        // same however the collector evacuated the cage. Only reachable
        // nodes: one a cycle found dead keeps its epoch until the
        // embedder releases it, and that cycle did not trace it, so its
        // external traced slots hold offsets from before the cage moved.
        // Hashed before the cage queue is drained, so the objects the
        // external traced slots name are numbered deterministically.
        if let Some(space) = st.node_space.as_deref() {
            let node_roots: Vec<NodeRef> = st
                .node_roots
                .borrow()
                .iter()
                .map(|e| e.node)
                .filter(|r| !r.is_none())
                .collect();
            h.write_u64(node_roots.len() as u64);
            for r in node_roots {
                h.write_u32(r.id);
                h.write_u32(r.epoch);
            }
            struct D<'a> {
                h: &'a mut hg_types::ProcessHasher,
                ids: &'a mut BTreeMap<u64, u64>,
                queue: &'a mut std::collections::VecDeque<u64>,
                st: &'a State,
            }
            impl NodeVisitor for D<'_> {
                fn node(&mut self, id: u32) {
                    self.h.write_u32(id);
                }
                fn external(&mut self, cell: &ExternalCell) {
                    let s = cell.load();
                    let w = match s.as_reference() {
                        Some(off) if self.st.space.is_object(off) => {
                            let next = self.ids.len() as u64;
                            let n = *self.ids.entry(off).or_insert_with(|| {
                                self.queue.push_back(off);
                                next
                            });
                            (n << 2) | 1
                        }
                        Some(_) => 3,
                        None => u64::from(s.raw()) << 2,
                    };
                    self.h.write_u64(w);
                }
            }
            let live = st.reachable_nodes();
            h.write_u64(live.len() as u64);
            for &id in &live {
                let Some(epoch) = space.epoch(id) else {
                    continue;
                };
                h.write_u32(id);
                h.write_u32(epoch);
                let mut d = D {
                    h: &mut h,
                    ids: &mut ids,
                    queue: &mut queue,
                    st,
                };
                space.trace(id, &mut d);
            }
        }
        while let Some(off) = queue.pop_front() {
            let Some((layout, size)) = st.shape(off) else {
                h.write_u64(u64::MAX);
                continue;
            };
            h.write(layout.name.as_bytes());
            h.write_u32(st.cage.load32(off + 4));
            for i in 0..layout.traced.saturating_add(layout.weak) {
                let s = Slot::from_raw(st.cage.load32(off + Layout::slot_offset(i)));
                h.write_u64(slot_word(s, &mut ids, &mut queue));
            }
            for i in 0..layout.nodes {
                let r = NodeRef::from_bits(st.cage.load64(off + layout.node_offset(i)));
                h.write_u32(r.id);
                h.write_u32(r.epoch);
            }
            for i in 0..layout.raw_words {
                h.write_u64(st.cage.load64(off + layout.raw_offset(i)));
            }
            let mut at = off + layout.tail_offset();
            match layout.tail {
                Tail::Slots => {
                    while at + 4 <= off + size {
                        let s = Slot::from_raw(st.cage.load32(at));
                        h.write_u64(slot_word(s, &mut ids, &mut queue));
                        at += 4;
                    }
                }
                Tail::Bytes => {
                    let len = u64::from(st.cage.load32(off + 4))
                        .min(size.saturating_sub(layout.tail_offset()));
                    let end = at + len;
                    while at < end {
                        let word = st.cage.load32(at & !3);
                        h.write_u8(u8::try_from((word >> ((at & 3) * 8)) & 0xFF).unwrap_or(0));
                        at += 1;
                    }
                }
                Tail::None => {}
            }
        }
        h.finish()
    }
}

/// Roots `slot` in scope `s`'s own segment. The segment exists while `s`
/// does: [`Mutator::scope`] pushes it before `s` is made and drops it
/// only after `s`'s lifetime has ended.
fn push_root<'h, 's, T: Referent>(
    roots: &RefCell<Vec<Vec<Slot>>>,
    s: &Scope<'h, 's>,
    slot: Slot,
) -> Root<'h, 's, T> {
    let mut roots = roots.borrow_mut();
    let index = roots.get_mut(s.level).map_or(0, |seg| {
        seg.push(slot);
        seg.len() - 1
    });
    Root {
        level: s.level,
        index,
        _h: PhantomData,
        _s: PhantomData,
        _t: PhantomData,
    }
}

impl<'h, 's> Scope<'h, 's> {
    /// Roots `gc` in this scope.
    pub fn root<T: Referent>(&self, m: &Mutator<'h>, gc: Gc<'h, '_, T>) -> Root<'h, 's, T> {
        self.root_slot(m, gc.slot)
    }

    /// Roots the JS value `v` in this scope: a reference is traced and
    /// updated like any root; a small integer is kept as it is. Read it
    /// back with [`Root::get_value`].
    pub fn root_value(&self, m: &Mutator<'h>, v: Value<'h, '_>) -> Root<'h, 's, Any> {
        self.root_slot(m, value_slot(v))
    }

    fn root_slot<T: Referent>(&self, m: &Mutator<'h>, slot: Slot) -> Root<'h, 's, T> {
        push_root(&m.state.roots, self, slot)
    }
}

impl<'h, T: Referent> Root<'h, '_, T> {
    /// The rooted reference, for as long as the mutator is borrowed.
    pub fn get<'m>(&self, m: &'m Mutator<'h>) -> Gc<'h, 'm, T> {
        let slot = m
            .state
            .roots
            .borrow()
            .get(self.level)
            .and_then(|seg| seg.get(self.index))
            .copied()
            .unwrap_or(Slot::ZERO);
        Gc {
            slot,
            _borrow: PhantomData,
            _t: PhantomData,
        }
    }

    /// Points this root at `gc` instead.
    pub fn set(&self, m: &Mutator<'h>, gc: Gc<'h, '_, T>) {
        if let Some(s) = m
            .state
            .roots
            .borrow_mut()
            .get_mut(self.level)
            .and_then(|seg| seg.get_mut(self.index))
        {
            *s = gc.slot;
        }
    }
}

impl<'h> Root<'h, '_, Any> {
    /// The rooted JS value, for as long as the mutator is borrowed.
    pub fn get_value<'m>(&self, m: &'m Mutator<'h>) -> Value<'h, 'm> {
        let slot = m
            .state
            .roots
            .borrow()
            .get(self.level)
            .and_then(|seg| seg.get(self.index))
            .copied()
            .unwrap_or(Slot::ZERO);
        slot_value(slot)
    }

    /// Makes this root hold the JS value `v` instead.
    pub fn set_value(&self, m: &Mutator<'h>, v: Value<'h, '_>) {
        if let Some(s) = m
            .state
            .roots
            .borrow_mut()
            .get_mut(self.level)
            .and_then(|seg| seg.get_mut(self.index))
        {
            *s = value_slot(v);
        }
    }
}

impl<'h, 'm, T: Referent> Gc<'h, 'm, T> {
    fn off(self) -> u64 {
        self.slot.as_reference().unwrap_or(0)
    }

    /// A reference from the slot `s`. For the crate's own readers of
    /// traced storage, inside and outside the cage.
    pub(crate) fn from_slot(s: Slot) -> Self {
        Self {
            slot: s,
            _borrow: PhantomData,
            _t: PhantomData,
        }
    }

    /// The slot this reference is stored as.
    pub fn slot(self) -> Slot {
        self.slot
    }

    /// This reference as an untyped one.
    pub fn erase(self) -> Gc<'h, 'm, Any> {
        Gc {
            slot: self.slot,
            _borrow: PhantomData,
            _t: PhantomData,
        }
    }

    /// This reference as a `U`, if the object's header says it is one
    /// (H-2: the header is checked against the type table).
    pub fn cast<U: Object>(self, m: &'m Mutator<'h>) -> Option<Gc<'h, 'm, U>> {
        let ty = m.state.cage.load32(self.off());
        let want = m.state.type_index.get(&TypeId::of::<U>())?;
        (ty == *want).then_some(Gc {
            slot: self.slot,
            _borrow: PhantomData,
            _t: PhantomData,
        })
    }

    /// The length of the object's tail.
    pub fn len(self, m: &'m Mutator<'h>) -> u32 {
        m.state.cage.load32(self.off() + 4)
    }

    /// Whether the object's tail is empty.
    pub fn is_empty(self, m: &'m Mutator<'h>) -> bool {
        self.len(m) == 0
    }
}

impl<'h, 'm, T: Object> Gc<'h, 'm, T> {
    /// Reads strong field `f`: `None` if it holds no reference.
    pub fn get<U: Referent>(self, m: &'m Mutator<'h>, f: Traced<T, U>) -> Option<Gc<'h, 'm, U>> {
        let s = Slot::from_raw(
            m.state
                .cage
                .load32(self.off() + Layout::slot_offset(f.index())),
        );
        s.is_reference().then_some(Gc {
            slot: s,
            _borrow: PhantomData,
            _t: PhantomData,
        })
    }

    /// Writes strong field `f` (the write barrier's place: heap design).
    pub fn set<U: Referent>(self, m: &Mutator<'h>, f: Traced<T, U>, v: Option<Gc<'h, '_, U>>) {
        let s = v.map_or(Slot::ZERO, |g| g.slot);
        m.state
            .cage
            .store32(self.off() + Layout::slot_offset(f.index()), s.raw());
    }

    /// Reads strong field `f` as a JS value.
    pub fn get_value(self, m: &'m Mutator<'h>, f: Traced<T, Any>) -> Value<'h, 'm> {
        let s = Slot::from_raw(
            m.state
                .cage
                .load32(self.off() + Layout::slot_offset(f.index())),
        );
        slot_value(s)
    }

    /// Writes strong field `f` from a JS value.
    pub fn set_value(self, m: &Mutator<'h>, f: Traced<T, Any>, v: Value<'h, '_>) {
        m.state.cage.store32(
            self.off() + Layout::slot_offset(f.index()),
            value_slot(v).raw(),
        );
    }

    /// Reads weak field `f`: `None` if it holds no reference (or was
    /// cleared).
    pub fn get_weak<U: Referent>(self, m: &'m Mutator<'h>, f: Weak<T, U>) -> Option<Gc<'h, 'm, U>> {
        let s = Slot::from_raw(
            m.state
                .cage
                .load32(self.off() + Layout::slot_offset(f.index())),
        );
        s.is_reference().then_some(Gc {
            slot: s,
            _borrow: PhantomData,
            _t: PhantomData,
        })
    }

    /// Writes weak field `f`.
    pub fn set_weak<U: Referent>(self, m: &Mutator<'h>, f: Weak<T, U>, v: Option<Gc<'h, '_, U>>) {
        let s = v.map_or(Slot::ZERO, |g| g.slot);
        m.state
            .cage
            .store32(self.off() + Layout::slot_offset(f.index()), s.raw());
    }

    /// Reads node field `f`: the node it names, or `None` if it names
    /// none. The value comes out of the cage, so it is untrusted (H-2):
    /// the collector bounds-checks it and compares its epoch with the
    /// space's before following it, and the binding path re-checks it on
    /// every call (JS spec 6.2).
    pub fn node(self, m: &'m Mutator<'h>, f: Node<T>) -> Option<NodeRef> {
        let bits = m
            .state
            .cage
            .load64(self.off() + T::LAYOUT.node_offset(f.index()));
        let r = NodeRef::from_bits(bits);
        (!r.is_none()).then_some(r)
    }

    /// Writes node field `f`. Traced strongly, so once a wrapper names
    /// its node the node cannot die while the wrapper lives (decision 2).
    pub fn set_node(self, m: &Mutator<'h>, f: Node<T>, v: Option<NodeRef>) {
        m.state.cage.store64(
            self.off() + T::LAYOUT.node_offset(f.index()),
            v.unwrap_or(NodeRef::NONE).to_bits(),
        );
    }

    /// Reads raw field `f`.
    pub fn raw<V: RawValue>(self, m: &'m Mutator<'h>, f: Raw<T, V>) -> V {
        V::from_bits(
            m.state
                .cage
                .load64(self.off() + T::LAYOUT.raw_offset(f.index())),
        )
    }

    /// Writes raw field `f`.
    pub fn set_raw<V: RawValue>(self, m: &Mutator<'h>, f: Raw<T, V>, v: V) {
        m.state
            .cage
            .store64(self.off() + T::LAYOUT.raw_offset(f.index()), v.to_bits());
    }

    /// Reads tail slot `i` (a type with a slot tail) as a JS value; `None`
    /// past the end.
    pub fn tail(self, m: &'m Mutator<'h>, i: u32) -> Option<Value<'h, 'm>> {
        if T::LAYOUT.tail != Tail::Slots || i >= self.len(m) {
            return None;
        }
        let at = self.off() + T::LAYOUT.tail_offset() + u64::from(i) * 4;
        Some(slot_value(Slot::from_raw(m.state.cage.load32(at))))
    }

    /// Writes tail slot `i`; ignored past the end.
    pub fn set_tail(self, m: &Mutator<'h>, i: u32, v: Value<'h, '_>) {
        if T::LAYOUT.tail != Tail::Slots || i >= self.len(m) {
            return;
        }
        let at = self.off() + T::LAYOUT.tail_offset() + u64::from(i) * 4;
        m.state.cage.store32(at, value_slot(v).raw());
    }

    /// Reads tail byte `i` (a type with a byte tail); `None` past the end.
    pub fn byte(self, m: &'m Mutator<'h>, i: u32) -> Option<u8> {
        if T::LAYOUT.tail != Tail::Bytes || i >= self.len(m) {
            return None;
        }
        let at = self.off() + T::LAYOUT.tail_offset() + u64::from(i);
        let word = m.state.cage.load32(at & !3);
        u8::try_from((word >> ((at & 3) * 8)) & 0xFF).ok()
    }

    /// Writes tail byte `i`; ignored past the end.
    pub fn set_byte(self, m: &Mutator<'h>, i: u32, b: u8) {
        if T::LAYOUT.tail != Tail::Bytes || i >= self.len(m) {
            return;
        }
        let at = self.off() + T::LAYOUT.tail_offset() + u64::from(i);
        let shift = (at & 3) * 8;
        let word = m.state.cage.load32(at & !3);
        let new = (word & !(0xFF << shift)) | (u32::from(b) << shift);
        m.state.cage.store32(at & !3, new);
    }
}

fn slot_value<'h, 'm>(s: Slot) -> Value<'h, 'm> {
    match s.as_small() {
        Some(i) => Value::Small(i),
        None => Value::Ref(Gc {
            slot: s,
            _borrow: PhantomData,
            _t: PhantomData,
        }),
    }
}

fn value_slot(v: Value<'_, '_>) -> Slot {
    match v {
        Value::Small(i) => Slot::small(i).unwrap_or(Slot::ZERO),
        Value::Ref(g) => g.slot,
    }
}

impl State {
    /// The nodes reachable from the node roots and from the node slots of
    /// reachable cage objects: the same closure a cycle computes, marking
    /// and moving nothing.
    ///
    /// [`Mutator::digest`] needs it. A node a canonical cycle found dead
    /// keeps its epoch until the embedder releases it, and that cycle did
    /// not trace it, so its external traced slots were not updated when
    /// the cage moved under them. Hashing such a node would hash a stale
    /// offset, which names different memory in each collector mode — so
    /// the digest hashes only the reachable ones.
    pub(crate) fn reachable_nodes(&self) -> std::collections::BTreeSet<u32> {
        let mut live = std::collections::BTreeSet::new();
        let Some(space) = self.node_space.as_deref() else {
            return live;
        };
        let mut nq: Vec<u32> = Vec::new();
        for e in self.node_roots.borrow().iter() {
            if live_node(space, e.node) && live.insert(e.node.id) {
                nq.push(e.node.id);
            }
        }
        let mut cq: Vec<u64> = Vec::new();
        let mut seen: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
        for s in self
            .roots
            .borrow()
            .iter()
            .flatten()
            .chain(self.kept.borrow().iter())
            .chain(self.cleared.borrow().iter())
        {
            if let Some(off) = s.as_reference()
                && self.space.is_object(off)
                && seen.insert(off)
            {
                cq.push(off);
            }
        }
        struct R<'a> {
            space: &'a dyn NodeSpace,
            live: &'a mut std::collections::BTreeSet<u32>,
            nq: &'a mut Vec<u32>,
            cq: &'a mut Vec<u64>,
            seen: &'a mut std::collections::BTreeSet<u64>,
            st: &'a State,
        }
        impl NodeVisitor for R<'_> {
            fn node(&mut self, id: u32) {
                if id != 0 && self.space.epoch(id).is_some() && self.live.insert(id) {
                    self.nq.push(id);
                }
            }
            fn external(&mut self, cell: &ExternalCell) {
                if let Some(off) = cell.load().as_reference()
                    && self.st.space.is_object(off)
                    && self.seen.insert(off)
                {
                    self.cq.push(off);
                }
            }
        }
        loop {
            while let Some(off) = cq.pop() {
                let Some((layout, size)) = self.shape(off) else {
                    continue;
                };
                let mut at_slots: Vec<u64> = (0..layout.traced.saturating_add(layout.weak))
                    .map(|i| off + Layout::slot_offset(i))
                    .collect();
                if layout.tail == Tail::Slots {
                    let mut at = off + layout.tail_offset();
                    while at + 4 <= off + size {
                        at_slots.push(at);
                        at += 4;
                    }
                }
                for at in at_slots {
                    if at + 4 > off + size {
                        continue;
                    }
                    if let Some(t) = Slot::from_raw(self.cage.load32(at)).as_reference()
                        && self.space.is_object(t)
                        && seen.insert(t)
                    {
                        cq.push(t);
                    }
                }
                for i in 0..layout.nodes {
                    let at = off + layout.node_offset(i);
                    if at + 8 > off + size {
                        continue;
                    }
                    let r = NodeRef::from_bits(self.cage.load64(at));
                    if live_node(space, r) && live.insert(r.id) {
                        nq.push(r.id);
                    }
                }
            }
            let Some(id) = nq.pop() else { break };
            let mut v = R {
                space,
                live: &mut live,
                nq: &mut nq,
                cq: &mut cq,
                seen: &mut seen,
                st: self,
            };
            space.trace(id, &mut v);
        }
        live
    }
}

/// Whether `r` names a node the space still holds, at the epoch `r` was
/// taken at. A reference read out of the cage is untrusted (H-2), so the
/// index is bounds-checked against the space's slot count and the epoch
/// is compared with the space's own; a mismatch means a stale or forged
/// reference, and it is not followed. In a well-formed heap a strong node
/// reference can never be stale, because the edge keeps its target alive
/// (decision 2), so a mismatch means corruption.
pub(crate) fn live_node(space: &dyn NodeSpace, r: NodeRef) -> bool {
    !r.is_none() && r.id < space.slots() && space.epoch(r.id) == Some(r.epoch)
}

/// Traces one node's edges, into the node worklist for its links and into
/// the cage worklist for its external traced slots.
struct NodeTracer<'a> {
    st: &'a mut State,
    evacuating: &'a mut [bool],
    forward: &'a mut BTreeMap<u64, u64>,
    work: &'a mut Vec<u64>,
    marks: &'a mut NodeMarks,
    node_work: &'a mut Vec<u32>,
    space: &'a dyn NodeSpace,
}

impl NodeVisitor for NodeTracer<'_> {
    fn node(&mut self, id: u32) {
        // A link comes from the space's own columns, so it carries no
        // epoch; the index is still bounds-checked and a slot holding no
        // live node is never followed.
        if id == 0 || id >= self.space.slots() || self.space.epoch(id).is_none() {
            return;
        }
        if !self.marks.mark(id) {
            self.node_work.push(id);
        }
    }

    fn external(&mut self, cell: &ExternalCell) {
        // The one place an external traced slot is followed. The node
        // space never moves, but the cage object it names can, so the
        // slot is written back wherever the target now lives.
        let s = cell.load();
        let n = self.st.visit(s, self.evacuating, self.forward, self.work);
        if n != s {
            cell.store(n);
        }
        // Stamped with the cycle in progress, which is one past the
        // completed count: the counters are raised at the end of the
        // cycle, so a slot this loop reaches ends up ahead of the count a
        // later read compares it with, and a slot it never reaches does
        // not (see `ExternalSlot`).
        cell.stamp(self.st.cycles_in_progress());
    }
}

// --- The collector ----------------------------------------------------------

impl State {
    fn type_of<T: Object>(&mut self) -> u32 {
        let id = TypeId::of::<T>();
        if let Some(&i) = self.type_index.get(&id) {
            return i;
        }
        let i = u32::try_from(self.types.len()).unwrap_or(u32::MAX);
        self.types.push(T::LAYOUT);
        self.type_index.insert(id, i);
        i
    }

    /// The layout and size of the object at `off`, with the size clamped
    /// to the object's block (H-2: the header and length are cage data).
    pub(crate) fn shape(&self, off: u64) -> Option<(Layout, u64)> {
        let ty = usize::try_from(self.cage.load32(off)).ok()?;
        let layout = *self.types.get(ty)?;
        let len = self.cage.load32(off + 4);
        let size = layout.size(len)?.min(self.space.room(off));
        Some((layout, size))
    }

    /// The number of the cycle now running: one past the cycles already
    /// completed, since the counters are raised when a cycle ends.
    fn cycles_in_progress(&self) -> u32 {
        u32::try_from(
            self.stats
                .canonical_cycles
                .saturating_add(self.stats.other_cycles)
                .saturating_add(1),
        )
        .unwrap_or(u32::MAX)
    }

    /// Runs a cycle. A canonical cycle clears dead weak targets and is a
    /// decision point (H-9); any other cycle treats weak slots as strong
    /// and moves every object, so it can change nothing observable.
    fn collect(&mut self, canonical: bool) {
        let mut evacuating = self.space.candidates(!canonical);
        self.space.start_cycle();
        let mut forward: BTreeMap<u64, u64> = BTreeMap::new();
        let mut work: Vec<u64> = Vec::new();

        // The node space's per-cycle state: the side mark bitmap of JS
        // spec 6.2, one bit per slot, and its own worklist. Both are
        // locals, because both are per-cycle; the space itself is an `Rc`
        // cloned out so the tracer below can hold `&mut State` as well.
        let ns = self.node_space.clone();
        let mut node_marks = ns
            .as_ref()
            .map_or_else(NodeMarks::default, |s| NodeMarks::for_slots(s.slots()));
        let mut node_work: Vec<u32> = Vec::new();
        if let Some(space) = ns.as_deref() {
            for e in self.node_roots.borrow().iter() {
                if live_node(space, e.node) && !node_marks.mark(e.node.id) {
                    node_work.push(e.node.id);
                }
            }
        }

        // Roots: the scopes' slots and the kept-alive list.
        let mut roots = core::mem::take(self.roots.get_mut());
        let mut kept = core::mem::take(self.kept.get_mut());
        let mut pending = core::mem::take(self.cleared.get_mut());
        for s in roots
            .iter_mut()
            .flatten()
            .chain(kept.iter_mut())
            .chain(pending.iter_mut())
        {
            *s = self.visit(*s, &mut evacuating, &mut forward, &mut work);
        }
        *self.roots.get_mut() = roots;
        *self.kept.get_mut() = kept;
        *self.cleared.get_mut() = pending;

        // Trace. The two worklists are one closure: a cage object's node
        // slot makes a node live, and a node's external traced slot makes
        // a cage object live, so neither can be finished before the
        // other. Cycles among JS, DOM and Wasm objects are ordinary
        // cycles (JS spec 6.5).
        loop {
            self.trace_cage(
                canonical,
                &mut evacuating,
                &mut forward,
                &mut work,
                ns.as_deref(),
                &mut node_marks,
                &mut node_work,
            );
            let Some(space) = ns.as_deref() else { break };
            if node_work.is_empty() {
                break;
            }
            while let Some(id) = node_work.pop() {
                let mut found: Vec<u32> = Vec::new();
                {
                    let mut t = NodeTracer {
                        st: self,
                        evacuating: &mut evacuating,
                        forward: &mut forward,
                        work: &mut work,
                        marks: &mut node_marks,
                        node_work: &mut found,
                        space,
                    };
                    space.trace(id, &mut t);
                }
                node_work.append(&mut found);
            }
        }

        // Dead node slots go on the release list, which the embedder
        // drains at an event-loop point (JS spec 6.5, release-by-ID).
        // Only a canonical cycle releases: a stress cycle traces weak
        // slots as strong, so a node reachable only through a weak
        // wrapper lives there and would die here, which would make the
        // collector's mode observable.
        if canonical && let Some(space) = ns.as_deref() {
            let mut pending = self.node_release_pending.borrow_mut();
            let mut list = self.node_release.borrow_mut();
            for id in 1..space.slots() {
                if let Some(epoch) = space.epoch(id)
                    && !node_marks.is_marked(id)
                    && pending.insert((id, epoch))
                {
                    list.push(NodeRef::new(id, epoch));
                }
            }
        }
        self.finish_cycle(canonical, &evacuating, &forward);
    }

    /// Drains the cage worklist, pushing any node a node slot names.
    #[expect(
        clippy::too_many_arguments,
        reason = "one cycle's state, threaded rather than put in State: it is per-cycle"
    )]
    fn trace_cage(
        &mut self,
        canonical: bool,
        evacuating: &mut [bool],
        forward: &mut BTreeMap<u64, u64>,
        work: &mut Vec<u64>,
        space: Option<&dyn NodeSpace>,
        node_marks: &mut NodeMarks,
        node_work: &mut Vec<u32>,
    ) {
        while let Some(off) = work.pop() {
            let Some((layout, size)) = self.shape(off) else {
                continue;
            };
            let strong = if canonical {
                layout.traced
            } else {
                layout.traced.saturating_add(layout.weak)
            };
            let mut slots: Vec<u64> = (0..strong).map(|i| off + Layout::slot_offset(i)).collect();
            if layout.tail == Tail::Slots {
                let start = off + layout.tail_offset();
                let end = off + size;
                let mut at = start;
                while at + 4 <= end {
                    slots.push(at);
                    at += 4;
                }
            }
            for at in slots {
                if at + 4 > off + size {
                    continue;
                }
                let s = Slot::from_raw(self.cage.load32(at));
                let n = self.visit(s, evacuating, forward, work);
                if n != s {
                    self.cage.store32(at, n.raw());
                }
            }
            // Node slots: a strong edge from the cage into the node
            // space. This is the half of "the wrapper holds the node"
            // that needs the collector (decision 2).
            if let Some(space) = space {
                for i in 0..layout.nodes {
                    let at = off + layout.node_offset(i);
                    if at + 8 > off + size {
                        continue;
                    }
                    let r = NodeRef::from_bits(self.cage.load64(at));
                    if live_node(space, r) && !node_marks.mark(r.id) {
                        node_work.push(r.id);
                    }
                }
            }
        }
    }

    /// The weak pass, the sweep and the cycle's bookkeeping.
    fn finish_cycle(&mut self, canonical: bool, evacuating: &[bool], forward: &BTreeMap<u64, u64>) {
        // Weak slots (canonical cycles clear dead targets; others moved
        // them as strong edges above).
        let mut cleared = 0u64;
        let holders = core::mem::take(&mut self.weak_holders);
        let mut live_holders = Vec::with_capacity(holders.len());
        for h in holders {
            let Some(now) = self.live_at(h, forward) else {
                continue;
            };
            live_holders.push(now);
            if !canonical {
                continue;
            }
            let Some((layout, _)) = self.shape(now) else {
                continue;
            };
            let before = cleared;
            for i in layout.traced..layout.traced.saturating_add(layout.weak) {
                let at = now + Layout::slot_offset(i);
                let s = Slot::from_raw(self.cage.load32(at));
                let Some(t) = s.as_reference() else { continue };
                match self.live_at(t, forward) {
                    Some(nt) if nt != t => {
                        if let Some(ns) = Slot::reference(nt) {
                            self.cage.store32(at, ns.raw());
                        }
                    }
                    Some(_) => {}
                    None => {
                        self.cage.store32(at, Slot::ZERO.raw());
                        cleared += 1;
                    }
                }
            }
            if cleared != before
                && let Some(slot) = Slot::reference(now)
            {
                self.cleared.get_mut().push(slot);
            }
        }
        self.weak_holders = live_holders;

        self.space.sweep(&self.cage, evacuating, canonical, POISON);
        self.stats.moved += forward.len() as u64;
        let (blocks, used) = self.space.census();
        self.stats.blocks = blocks as u64;
        self.stats.blocks_used = used as u64;
        if canonical {
            self.debt = 0;
            self.stats.canonical_cycles += 1;
            self.stats.cleared += cleared;
            let at = Position {
                agent: self.config.agent,
                progress: self.progress,
            };
            self.decisions
                .record(DecisionKind::GC_COMPLETE, at, cleared);
        } else {
            self.stats.other_cycles += 1;
        }
        self.allocated_since_cycle = false;
        self.run_verifier(canonical);
    }

    /// Runs the heap verifier, if this heap has it on. A violation is a
    /// collector bug, so it panics: a heap that has lost an object must
    /// not carry on and produce a plausible wrong answer.
    fn run_verifier(&self, canonical: bool) {
        if !self.verify {
            return;
        }
        if let Err(v) = crate::verify::verify(self) {
            panic!(
                "hg-heap: the heap verifier failed after a {} cycle: {v}",
                if canonical {
                    "canonical"
                } else {
                    "non-canonical"
                }
            );
        }
    }

    /// Where the object at `off` lives after this cycle's marking, if it
    /// is live.
    fn live_at(&self, off: u64, forward: &BTreeMap<u64, u64>) -> Option<u64> {
        if let Some(&n) = forward.get(&off) {
            return Some(n);
        }
        (self.space.is_object(off) && self.space.is_marked(off)).then_some(off)
    }

    /// Follows slot `s`: marks (and, in an evacuated block, copies) its
    /// target, queues it for tracing, and returns the slot to store back.
    /// When there is no room for a copy, the object is marked where it is
    /// and its block leaves `evacuating`, so the sweep keeps the block and
    /// frees only its unmarked lines. Objects already copied out of it
    /// stay forwarded; their marked old copies go at the next cycle.
    fn visit(
        &mut self,
        s: Slot,
        evacuating: &mut [bool],
        forward: &mut BTreeMap<u64, u64>,
        work: &mut Vec<u64>,
    ) -> Slot {
        let Some(off) = s.as_reference() else {
            return s;
        };
        if let Some(&n) = forward.get(&off) {
            return Slot::reference(n).unwrap_or(s);
        }
        if !self.space.is_object(off) {
            return s; // not an object: never followed (H-2)
        }
        let Some((_, size)) = self.shape(off) else {
            return s;
        };
        let block = self
            .space
            .block_index(off)
            .filter(|&b| evacuating.get(b).copied().unwrap_or(false))
            .filter(|_| self.space.is_small(off));
        if let Some(b) = block
            && !self.space.is_marked(off)
        {
            match self.space.alloc_copy(&self.cage, size, evacuating) {
                Ok(to) => {
                    self.cage.copy(off, to, size);
                    forward.insert(off, to);
                    self.space.mark(to, size);
                    // Mark the old copy too, so a second path finds `forward`.
                    self.space.mark(off, size);
                    work.push(to);
                    return Slot::reference(to).unwrap_or(s);
                }
                Err(_) => {
                    if let Some(e) = evacuating.get_mut(b) {
                        *e = false;
                    }
                }
            }
        }
        if !self.space.mark(off, size) {
            work.push(off);
        }
        s
    }
}
