//! The shape real code gets wrong: a batch of references gathered into a
//! `Vec` and used after a GC point. Each one is a `Gc`, so the borrow of
//! the mutator is in the `Vec`, and the cycle cannot run while it lives.
//! The fix is a `Vec<Root>`, or a re-read from the roots afterwards.
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
            let mut batch = Vec::new();
            for _ in 0..4 {
                batch.push(m.alloc::<Cell>(s, 0).expect("alloc").get(m));
            }
            m.collect();
            for g in &batch {
                let _ = g.len(m);
            }
        });
    });
}
