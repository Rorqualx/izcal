//! Positive control for the one-way rule: an agent heap may hold a
//! reference into its cluster's shared heap, and both references are one
//! 32-bit slot wide.
//!
//! EXPECT: compiles.

mod shared_brand {
    include!("shared-brand.rs");
}

use shared_brand::*;

fn main() {
    cluster(|heap| {
        let root = heap.shared(|sm, root| {
            let other = sm.alloc();
            sm.set_shared(root, other);
            root
        });
        heap.agent(|am| {
            let mine = am.alloc();
            let also = am.alloc();
            // Its own blocks: allowed.
            am.set_own(mine, also);
            // Into the shared heap: the one direction the rule allows.
            am.set_shared(mine, root);
            assert_eq!(std::mem::size_of_val(&mine), 4);
            assert_eq!(std::mem::size_of_val(&root), 4);
            assert!(am.bits(mine) > 0 && am.shared_bits(root) > 0);
        });
    });
}
