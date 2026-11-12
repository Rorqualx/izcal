//! Cross-heap case 3: a reference of heap `'y` rooted in a scope of heap
//! `'x`. Heap `'x`'s root list would keep alive, and update on
//! evacuation, an offset in another heap's blocks — the one way a root
//! could corrupt a heap it does not belong to.
//!
//! EXPECT: does not compile — the two heaps' brands do not unify.

// EXPECT-ERROR: LIFETIME
// EXPECT-NOTE: the struct `Gc<'h, 'm, T>` is invariant over the parameter `'h`
use hg_heap::{Gc, Mutator, Scope};

hg_heap::object! {
    /// A list cell.
    pub Cell {
        traced NEXT: Cell,
        tail none,
    }
}

pub fn case<'x, 'y>(ma: &Mutator<'x>, sa: &Scope<'x, '_>, theirs: Gc<'y, '_, Cell>) {
    let _ = sa.root(ma, theirs);
}
