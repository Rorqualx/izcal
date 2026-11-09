//! Two agent heaps of one cluster, in one cage. A reference of the first
//! stored into an object of the second: the slots decode off the same
//! base, so nothing in the encoding stops it. The brands do.
//!
//! EXPECT: does not compile — the two `'h` brands do not unify.

mod shared_brand {
    include!("shared-brand.rs");
}

use shared_brand::*;

fn main() {
    cluster(|heap| {
        heap.agent(|a| {
            let theirs = a.alloc();
            heap.agent(|b| {
                let mine = b.alloc();
                b.set_own(mine, theirs);
            });
        });
    });
}
