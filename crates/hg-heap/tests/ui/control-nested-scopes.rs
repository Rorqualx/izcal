//! Positive control: an outer scope's `Root` is usable inside an inner
//! scope. Only the other direction is refused (see
//! `fail-root-escapes-inner-scope`).
//!
//! EXPECT: compiles.
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
        m.scope(|m, outer| {
            let x = m.alloc::<Cell>(outer, 0).expect("alloc");
            m.scope(|m, inner| {
                let y = m.alloc::<Cell>(inner, 0).expect("alloc");
                // The outer root is still live here.
                x.get(m).set(m, Cell::NEXT, Some(y.get(m)));
            });
            let _ = x.get(m).len(m);
        });
    });
}
