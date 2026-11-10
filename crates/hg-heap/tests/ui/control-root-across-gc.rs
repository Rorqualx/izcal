//! Positive control: a `Root` is the way to hold a reference across a GC
//! point. Every `fail-gc-across-*` case becomes this when the reference
//! is rooted and re-read afterwards.
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
        m.scope(|m, s| {
            let x = m.alloc::<Cell>(s, 0).expect("alloc");
            m.collect();
            m.charge(1);
            m.safepoint();
            let _ = m.alloc::<Cell>(s, 0).expect("alloc");
            // The root survived all four GC points; the reference is
            // read again from it, never kept across one.
            let _ = x.get(m).len(m);
        });
    });
}
