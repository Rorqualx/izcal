//! Positive control: inside one heap the branded API compiles, so a
//! refusal in the cases beside this one is the brand rejecting something
//! and not the harness being broken.
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
            let y = m.alloc::<Cell>(s, 0).expect("alloc");
            // Both references carry this heap's brand, so this compiles.
            x.get(m).set(m, Cell::NEXT, Some(y.get(m)));
        });
    });
}
