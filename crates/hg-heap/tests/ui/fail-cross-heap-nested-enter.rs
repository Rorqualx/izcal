//! The shape the four cross-heap cases take in real code: heap `b`
//! entered inside heap `a`'s enter, and `a`'s mutator used in there.
//!
//! This is refused for two independent reasons, which is why the brand
//! has its own cases beside this one. `Heap::enter` takes a
//! `for<'h> FnOnce(&mut Mutator<'h>) -> R` whose result cannot mention
//! `'h`, so the outer mutator cannot even be named inside another heap's
//! enter; and where a reference does cross, the brands do not unify.
//! Either way, no code reaches the point of mixing two heaps.
//!
//! EXPECT: does not compile — the outer mutator cannot cross the inner
//! enter.

// EXPECT-ERROR: E0521
// EXPECT-NOTE: escapes the closure body here
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
            b.enter(|mb| {
                mb.scope(|mb, sb| {
                    let theirs = mb.alloc::<Cell>(sb, 0).expect("alloc");
                    mine.get(ma).set(ma, Cell::NEXT, Some(theirs.get(mb)));
                });
            });
        });
    });
}
