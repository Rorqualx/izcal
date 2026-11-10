//! A reference carried out of its `Heap::enter`. Outside the enter there
//! is no mutator, so there is no way to say when the next GC point is.
//!
//! EXPECT: does not compile — the brand cannot escape.

// EXPECT-ERROR: LIFETIME
// EXPECT-NOTE: the struct `Gc<'h, 'm, T>` is invariant over the parameter `'h`
use hg_heap::{Gc, Heap};

hg_heap::object! {
    /// A list cell.
    pub Cell {
        traced NEXT: Cell,
        tail none,
    }
}

pub fn case<'any>(a: &'any mut Heap) -> Gc<'any, 'any, Cell> {
    a.enter(|m| m.scope(|m, s| m.alloc::<Cell>(s, 0).expect("alloc").get(m)))
}
