//! Cross-heap case 1: a reference of heap `'y` stored into an object of
//! heap `'x`. Both heaps are in the one process cage, so the slot would
//! decode to a real object and heap `'x`'s collector would follow it
//! into blocks it does not own (H-12).
//!
//! The two mutators arrive as parameters with separate brands, which is
//! what two nested `Heap::enter` calls give. Written as the nesting it
//! would be in real code the case is refused twice over — see
//! `fail-cross-heap-nested-enter` — and only one of those refusals is
//! the brand, so the brand gets a case of its own here.
//!
//! EXPECT: does not compile — the two heaps' brands do not unify.

// EXPECT-ERROR: LIFETIME
// EXPECT-NOTE: the struct `Mutator<'h>` is invariant over the parameter `'h`
use hg_heap::{Gc, Mutator, Scope};

hg_heap::object! {
    /// A list cell.
    pub Cell {
        traced NEXT: Cell,
        tail none,
    }
}

pub fn case<'x, 'y>(ma: &mut Mutator<'x>, sa: &Scope<'x, '_>, theirs: Gc<'y, '_, Cell>) {
    let mine = ma.alloc::<Cell>(sa, 0).expect("alloc");
    mine.get(ma).set(ma, Cell::NEXT, Some(theirs));
}
