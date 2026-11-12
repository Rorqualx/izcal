//! Positive control: two heaps of one process, each used with its own
//! mutator. The brand separates them; it does not stop a thread from
//! holding both.
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

pub fn case(a: &mut Heap, b: &mut Heap) {
    a.enter(|ma| {
        ma.scope(|ma, sa| {
            let mine = ma.alloc::<Cell>(sa, 0).expect("alloc");
            let other = ma.alloc::<Cell>(sa, 0).expect("alloc");
            mine.get(ma).set(ma, Cell::NEXT, Some(other.get(ma)));
        });
    });
    b.enter(|mb| {
        mb.scope(|mb, sb| {
            let theirs = mb.alloc::<Cell>(sb, 0).expect("alloc");
            let _ = theirs.get(mb).len(mb);
        });
    });
}
