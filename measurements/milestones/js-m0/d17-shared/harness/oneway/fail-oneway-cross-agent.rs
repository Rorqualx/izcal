//! Two agent heaps of one cluster, in one cage. A reference of the first
//! stored into an object of the second: the slots decode off the same
//! base, so nothing in the encoding stops it. The brands do.
//!
//! EXPECT: does not compile — the two `'h` brands do not unify.

// EXPECT-ERROR: E0521
// The two agent brands do not unify. The runner checks that rustc
// reported this code: without that check a crate-version mismatch, a
// missing crate or a typo would refuse the case and read as the rule
// holding.
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
