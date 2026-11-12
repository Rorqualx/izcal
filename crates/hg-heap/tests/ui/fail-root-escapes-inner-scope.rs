//! A `Root` of an inner scope used after that scope returned. The inner
//! scope's roots die with it, so this would read a slot the next scope
//! has reused. The legitimate form is `control-nested-scopes`, which
//! uses the outer scope's root inside the inner one.
//!
//! EXPECT: does not compile — the inner scope's brand cannot escape.

// EXPECT-ERROR: LIFETIME
// EXPECT-NOTE: the struct `Root<'h, 's, T>` is invariant over the parameter `'h`
use hg_heap::Heap;

hg_heap::object! {
    /// A list cell.
    pub Cell {
        traced NEXT: Cell,
        tail none,
    }
}

pub fn case(a: &mut Heap) {
    a.enter(|m| {
        m.scope(|m, _outer| {
            let from_inner = m.scope(|m, inner| m.alloc::<Cell>(inner, 0).expect("alloc"));
            let _ = from_inner.get(m);
        });
    });
}
