//! A `Scope` handle carried out of `Mutator::scope`. A scope that
//! outlived its call could root into a segment of the root list that has
//! been truncated.
//!
//! EXPECT: does not compile — the scope brand cannot escape.

// EXPECT-ERROR: LIFETIME
// EXPECT-NOTE: the struct `hg_heap::Scope<'h, 's>` is invariant over the parameter `'h`
use hg_heap::Heap;

pub fn case(a: &mut Heap) {
    a.enter(|m| {
        let escaped = m.scope(|_m, s| s);
        let _ = escaped;
    });
}
