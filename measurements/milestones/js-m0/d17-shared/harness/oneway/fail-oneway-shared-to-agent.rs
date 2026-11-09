//! The direction the rule forbids: a shared-heap object holding a
//! reference into an agent heap. A shared heap is read by every agent of
//! the cluster, so such a reference would let one agent reach another's
//! blocks, and would make the shared heap's collection depend on every
//! agent's roots.
//!
//! EXPECT: does not compile — `SharedMutator::set_shared` takes a
//! `SharedRef`, and there is no setter that takes an `AgentRef`.

mod shared_brand {
    include!("shared-brand.rs");
}

use shared_brand::*;

fn main() {
    cluster(|heap| {
        heap.agent(|am| {
            let mine = am.alloc();
            heap.shared(|sm, root| {
                // The forbidden direction.
                sm.set_shared(root, mine);
            });
        });
    });
}
