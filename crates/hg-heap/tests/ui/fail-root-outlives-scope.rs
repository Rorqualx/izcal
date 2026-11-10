//! A `Root` carried out of its scope. The scope truncates the root list
//! when it returns, so the root's slot is gone; `'s` is fresh and
//! invariant per `Mutator::scope` to make that unsayable (JS-D18,
//! soundness step 3).
//!
//! EXPECT: does not compile — the scope brand cannot escape.

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
        let escaped = m.scope(|m, s| m.alloc::<Cell>(s, 0).expect("alloc"));
        m.scope(|m, _s| {
            let _ = escaped.get(m);
        });
    });
}
