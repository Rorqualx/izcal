//! The agent heap, its branded mutator, roots and the collector (JS spec
//! 6.5; heap design H-7 to H-11).

use core::cell::RefCell;
use core::marker::PhantomData;
use std::any::TypeId;
use std::collections::BTreeMap;

use hg_types::HashMap;
use std::sync::Arc;

use hg_substrate::{DecisionKind, DecisionLog, Memory, MemoryKind, Position};

use crate::cage::{Cage, CageError};
use crate::object::{Any, Layout, Object, Raw, RawValue, Referent, Tail, Traced, Weak};
use crate::slot::Slot;
use crate::space::{AllocError, Space};

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
    pub(crate) cage: Cage,
    space: Space,
    /// The type table: index → layout (outside the cage, H-4).
    types: Vec<Layout>,
    type_index: HashMap<TypeId, u32>,
    /// Root slots, by scope order; a scope truncates on exit.
    roots: RefCell<Vec<Slot>>,
    /// The agent's [[KeptAlive]] list (ECMA-262 9.10).
    kept: RefCell<Vec<Slot>>,
    /// Objects that had a weak slot cleared by a canonical cycle and that
    /// the embedder has not taken yet ([`Mutator::take_cleared`]). Traced
    /// as roots until taken, in the order the cycle cleared them.
    cleared: RefCell<Vec<Slot>>,
    /// Objects whose layout has weak slots, to clear after a cycle.
    weak_holders: Vec<u64>,
    debt: u64,
    config: HeapConfig,
    decisions: Arc<dyn DecisionLog>,
    progress: u64,
    stats: HeapStats,
}

/// One agent heap. It is entered for each turn of its agent's work.
pub struct Heap {
    state: State,
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
    _h: Invariant<'h>,
    _s: Invariant<'s>,
}

/// A rooted reference: survives GC points, and cannot outlive scope `'s`.
pub struct Root<'h, 's, T: Referent> {
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
    /// A heap in a fresh cage reserved through `mem`, logging its
    /// decisions to `decisions`.
    ///
    /// # Errors
    /// [`HeapError::Cage`] if the cage cannot be reserved.
    pub fn new(
        mem: Arc<dyn Memory>,
        decisions: Arc<dyn DecisionLog>,
        config: HeapConfig,
    ) -> Result<Self, HeapError> {
        let cage = Cage::new(mem, config.cage_bytes).map_err(HeapError::Cage)?;
        Ok(Self {
            state: State {
                cage,
                space: Space::new(MemoryKind::PerAgent),
                types: Vec::new(),
                type_index: HashMap::default(),
                roots: RefCell::new(Vec::new()),
                kept: RefCell::new(Vec::new()),
                cleared: RefCell::new(Vec::new()),
                weak_holders: Vec::new(),
                debt: 0,
                config,
                decisions,
                progress: 0,
                stats: HeapStats::default(),
            },
        })
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
}

impl<'h> Mutator<'h> {
    /// Runs `f` with a fresh root scope; its roots die when it returns.
    pub fn scope<R>(&mut self, f: impl for<'s> FnOnce(&mut Self, &Scope<'h, 's>) -> R) -> R {
        let depth = self.state.roots.borrow().len();
        let s = Scope {
            _h: PhantomData,
            _s: PhantomData,
        };
        let r = f(self, &s);
        self.state.roots.borrow_mut().truncate(depth);
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
        let _ = s;
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
        let slot = Slot::reference(at).ok_or(AllocError::OutOfMemory)?;
        let mut roots = st.roots.borrow_mut();
        roots.push(slot);
        Ok(Root {
            index: roots.len() - 1,
            _h: PhantomData,
            _s: PhantomData,
            _t: PhantomData,
        })
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
        let _ = s;
        let taken = core::mem::take(&mut *self.state.cleared.borrow_mut());
        let mut roots = self.state.roots.borrow_mut();
        taken
            .into_iter()
            .map(|slot| {
                roots.push(slot);
                Root {
                    index: roots.len() - 1,
                    _h: PhantomData,
                    _s: PhantomData,
                    _t: PhantomData,
                }
            })
            .collect()
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
            .chain(st.kept.borrow().iter())
            .chain(st.cleared.borrow().iter())
            .copied()
            .collect();
        h.write_u64(roots.len() as u64);
        for s in roots {
            h.write_u64(slot_word(s, &mut ids, &mut queue));
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
        let mut roots = m.state.roots.borrow_mut();
        roots.push(slot);
        Root {
            index: roots.len() - 1,
            _h: PhantomData,
            _s: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<'h, T: Referent> Root<'h, '_, T> {
    /// The rooted reference, for as long as the mutator is borrowed.
    pub fn get<'m>(&self, m: &'m Mutator<'h>) -> Gc<'h, 'm, T> {
        let slot = m
            .state
            .roots
            .borrow()
            .get(self.index)
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
        if let Some(s) = m.state.roots.borrow_mut().get_mut(self.index) {
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
            .get(self.index)
            .copied()
            .unwrap_or(Slot::ZERO);
        slot_value(slot)
    }

    /// Makes this root hold the JS value `v` instead.
    pub fn set_value(&self, m: &Mutator<'h>, v: Value<'h, '_>) {
        if let Some(s) = m.state.roots.borrow_mut().get_mut(self.index) {
            *s = value_slot(v);
        }
    }
}

impl<'h, 'm, T: Referent> Gc<'h, 'm, T> {
    fn off(self) -> u64 {
        self.slot.as_reference().unwrap_or(0)
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
    fn shape(&self, off: u64) -> Option<(Layout, u64)> {
        let ty = usize::try_from(self.cage.load32(off)).ok()?;
        let layout = *self.types.get(ty)?;
        let len = self.cage.load32(off + 4);
        let size = layout.size(len)?.min(self.space.room(off));
        Some((layout, size))
    }

    /// Runs a cycle. A canonical cycle clears dead weak targets and is a
    /// decision point (H-9); any other cycle treats weak slots as strong
    /// and moves every object, so it can change nothing observable.
    fn collect(&mut self, canonical: bool) {
        let evacuating = self.space.candidates(!canonical);
        self.space.start_cycle();
        let mut forward: BTreeMap<u64, u64> = BTreeMap::new();
        let mut work: Vec<u64> = Vec::new();

        // Roots: the scopes' slots and the kept-alive list.
        let mut roots = core::mem::take(self.roots.get_mut());
        let mut kept = core::mem::take(self.kept.get_mut());
        let mut pending = core::mem::take(self.cleared.get_mut());
        for s in roots
            .iter_mut()
            .chain(kept.iter_mut())
            .chain(pending.iter_mut())
        {
            *s = self.visit(*s, &evacuating, &mut forward, &mut work);
        }
        *self.roots.get_mut() = roots;
        *self.kept.get_mut() = kept;
        *self.cleared.get_mut() = pending;

        // Trace.
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
                let n = self.visit(s, &evacuating, &mut forward, &mut work);
                if n != s {
                    self.cage.store32(at, n.raw());
                }
            }
        }

        // Weak slots (canonical cycles clear dead targets; others moved
        // them as strong edges above).
        let mut cleared = 0u64;
        let holders = core::mem::take(&mut self.weak_holders);
        let mut live_holders = Vec::with_capacity(holders.len());
        for h in holders {
            let Some(now) = self.live_at(h, &forward) else {
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
                match self.live_at(t, &forward) {
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

        self.space.sweep(&self.cage, &evacuating, canonical, POISON);
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
    fn visit(
        &mut self,
        s: Slot,
        evacuating: &[bool],
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
        let block = usize::try_from(off / crate::space::BLOCK).unwrap_or(usize::MAX);
        let evacuate = evacuating.get(block).copied().unwrap_or(false) && self.space.is_small(off);
        if evacuate
            && !self.space.is_marked(off)
            && let Ok(to) = self.space.alloc_copy(&self.cage, size, evacuating)
        {
            self.cage.copy(off, to, size);
            forward.insert(off, to);
            self.space.mark(to, size);
            // Mark the old copy too, so a second path finds `forward`.
            self.space.mark(off, size);
            work.push(to);
            return Slot::reference(to).unwrap_or(s);
        }
        if !self.space.mark(off, size) {
            work.push(off);
        }
        s
    }
}
