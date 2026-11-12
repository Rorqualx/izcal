//! A `Gc` kept across a GC point: an explicit safepoint, where the stress mode moves everything. The collector may move the
//! object, so the reference would name where it used to be; only a
//! `Root` survives (JS-D18, soundness step 2).
//!
//! EXPECT: does not compile — the mutator is already borrowed.

// EXPECT-ERROR: E0502
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
        m.scope(|m, s| {
            let x = m.alloc::<Cell>(s, 0).expect("alloc");
            let g = x.get(m);
            m.safepoint();
            let _ = g.len(m);
        });
    });
}
