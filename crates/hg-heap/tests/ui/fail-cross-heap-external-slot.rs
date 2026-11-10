//! Cross-heap case 4: one thread holding both heaps, writing heap `'y`'s
//! reference into an external traced slot through heap `'x`'s mutator.
//!
//! An external slot lives outside the cage, in a table that outlives any
//! one `Heap::enter`, so the slot itself cannot carry a per-enter brand.
//! What the brand does cover is the pair of arguments at the call, and
//! that is this case. The other half of the hazard — a slot written from
//! the wrong heap in a later enter, where there is no pair to compare —
//! is outside what the type system can see, and is the debug assertion
//! in `ExternalSlot::set` that the crate's own tests exercise.
//!
//! EXPECT: does not compile — the two heaps' brands do not unify.

// EXPECT-ERROR: LIFETIME
// EXPECT-NOTE: the struct `Mutator<'h>` is invariant over the parameter `'h`
use hg_heap::{ExternalSlot, Gc, Mutator};

hg_heap::object! {
    /// A list cell.
    pub Cell {
        traced NEXT: Cell,
        tail none,
    }
}

pub fn case<'x, 'y>(ma: &Mutator<'x>, slot: &ExternalSlot<Cell>, theirs: Gc<'y, '_, Cell>) {
    slot.set(ma, Some(theirs));
}
