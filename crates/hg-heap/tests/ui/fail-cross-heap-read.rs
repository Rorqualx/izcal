//! Cross-heap case 2: a reference of heap `'y` read through heap `'x`'s
//! mutator. The offset would be decoded against the right cage, but
//! interpreted with the wrong heap's type table and block ownership, so
//! the header it read would be another heap's object.
//!
//! EXPECT: does not compile — the two heaps' brands do not unify.

// EXPECT-ERROR: LIFETIME
// EXPECT-NOTE: the struct `Mutator<'h>` is invariant over the parameter `'h`
use hg_heap::{Gc, Mutator};

hg_heap::object! {
    /// A list cell.
    pub Cell {
        traced NEXT: Cell,
        tail none,
    }
}

pub fn case<'x, 'y>(ma: &Mutator<'x>, theirs: Gc<'y, '_, Cell>) {
    let _ = theirs.len(ma);
}
