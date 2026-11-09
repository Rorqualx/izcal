// The one-way shared-heap brand, as a type-system prototype.
//
// The JS spec's shared-structs paragraph says that because all agent
// heaps of a process share one cage (JS-D17), a shared heap is "another
// set of blocks in the same cage ... and a reference from an agent heap
// into it is an ordinary 32-bit compressed reference", and that "the
// brand discipline extends with a shared-heap brand that agent heaps may
// reference one way".
//
// `hg-heap` has no shared heap yet, so this is not a test of shipped
// code. It is a test of whether the rule the spec states is expressible
// in the discipline `hg-heap` already uses — an invariant lifetime brand
// per heap, carried by the mutator — and whether the compiler then
// enforces it. Three brands:
//
//   'g   the agent cluster's shared heap
//   'h   one agent heap of that cluster
//
// and two reference types, `AgentRef<'h>` and `SharedRef<'g>`, each four
// bytes: one 32-bit compressed slot, with no base and no heap identity
// in it, exactly as under JS-D17 (a). The one-way rule is then a
// property of the two mutators' signatures:
//
//   * an agent heap's object may hold `AgentRef<'h>` (its own) and
//     `SharedRef<'g>` (its cluster's shared heap): both setters exist;
//   * a shared heap's object may hold `SharedRef<'g>` only: the shared
//     mutator has no setter that takes an `AgentRef`, so the other
//     direction is not merely rejected but unrepresentable;
//   * an agent heap's object may not hold another agent heap's
//     reference: the brands are invariant and fresh per `enter`, so they
//     do not unify.

use std::marker::PhantomData;

/// An invariant lifetime marker, as `hg-heap`'s brands use.
type Inv<'a> = PhantomData<fn(&'a ()) -> &'a ()>;

/// A reference into agent heap `'h`: one 32-bit compressed slot.
#[derive(Clone, Copy)]
pub struct AgentRef<'h> {
    slot: u32,
    _h: Inv<'h>,
}

/// A reference into the cluster's shared heap `'g`: the same 32-bit
/// compressed slot, off the same cage base.
#[derive(Clone, Copy)]
pub struct SharedRef<'g> {
    slot: u32,
    _g: Inv<'g>,
}

/// The cluster's shared heap.
pub struct SharedHeap<'g> {
    next: std::cell::Cell<u32>,
    _g: Inv<'g>,
}

/// The only way to allocate in or write to agent heap `'h`, which
/// belongs to cluster `'g`.
pub struct AgentMutator<'h, 'g> {
    next: std::cell::Cell<u32>,
    _h: Inv<'h>,
    _g: Inv<'g>,
}

/// The only way to allocate in or write to the shared heap `'g`.
pub struct SharedMutator<'g> {
    _g: Inv<'g>,
}

/// Runs `f` with a fresh shared-heap brand.
pub fn cluster<R>(f: impl for<'g> FnOnce(&SharedHeap<'g>) -> R) -> R {
    f(&SharedHeap {
        next: std::cell::Cell::new(9),
        _g: PhantomData,
    })
}

impl<'g> SharedHeap<'g> {
    /// Runs `f` with a fresh agent-heap brand inside this cluster.
    pub fn agent<R>(&self, f: impl for<'h> FnOnce(&AgentMutator<'h, 'g>) -> R) -> R {
        f(&AgentMutator {
            next: std::cell::Cell::new(1),
            _h: PhantomData,
            _g: PhantomData,
        })
    }

    /// Runs `f` with the shared heap's own mutator.
    pub fn shared<R>(&self, f: impl FnOnce(&SharedMutator<'g>, SharedRef<'g>) -> R) -> R {
        let root = SharedRef {
            slot: self.next.get(),
            _g: PhantomData,
        };
        self.next.set(self.next.get() + 2);
        f(&SharedMutator { _g: PhantomData }, root)
    }
}

impl<'h, 'g> AgentMutator<'h, 'g> {
    /// Allocates in this agent heap's own blocks.
    pub fn alloc(&self) -> AgentRef<'h> {
        let slot = self.next.get();
        self.next.set(slot + 2);
        AgentRef {
            slot,
            _h: PhantomData,
        }
    }

    /// Stores this agent heap's own reference in one of its objects.
    pub fn set_own(&self, _at: AgentRef<'h>, _v: AgentRef<'h>) {}

    /// Stores a reference into the cluster's shared heap in one of this
    /// agent heap's objects: the one direction the rule allows.
    pub fn set_shared(&self, _at: AgentRef<'h>, _v: SharedRef<'g>) {}

    /// The slot, to show it is an ordinary 32-bit value either way.
    pub fn bits(&self, r: AgentRef<'h>) -> u32 {
        r.slot
    }

    /// The same for a shared-heap reference: the same width, the same
    /// decoding, no second base.
    pub fn shared_bits(&self, r: SharedRef<'g>) -> u32 {
        r.slot
    }
}

impl<'g> SharedMutator<'g> {
    /// Allocates in the shared heap's blocks.
    pub fn alloc(&self) -> SharedRef<'g> {
        SharedRef {
            slot: 101,
            _g: PhantomData,
        }
    }

    /// Stores a shared-heap reference in a shared-heap object. There is
    /// deliberately no setter that takes an `AgentRef`.
    pub fn set_shared(&self, _at: SharedRef<'g>, _v: SharedRef<'g>) {}
}
