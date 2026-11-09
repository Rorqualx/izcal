//! A reference of one agent heap stored into an object of another. Both
//! heaps are in the same `ProcessCage`, so the slot would decode to a
//! real object and the collector would follow it into blocks it does not
//! own. `Mutator<'h>`'s brand is what stops it.
//!
//! EXPECT: does not compile — lifetime mismatch on the brand `'h`.

// EXPECT-ERROR: E0521
// The two heaps' brands do not unify. The runner checks that rustc
// reported this code: without that check a crate-version mismatch, a
// missing crate or a typo would refuse the case and read as the rule
// holding.
use std::sync::Arc;

use hg_heap::{Heap, HeapConfig, ProcessCage, Stress};
use hg_substrate::NotRecording;
use hg_substrate_os::memory::OsMemory;

hg_heap::object! {
    /// A cell.
    pub Cell {
        traced NEXT: Cell,
        tail none,
    }
}

fn main() {
    let mem = Arc::new(OsMemory::new());
    let config = HeapConfig {
        cage_bytes: 1 << 24,
        trigger: u64::MAX,
        stress: Stress::None,
        agent: 0,
    };
    let cage = ProcessCage::new(mem, config.cage_bytes).expect("cage");
    let mut a = Heap::in_cage(&cage, Arc::new(NotRecording), config);
    let mut b = Heap::in_cage(&cage, Arc::new(NotRecording), config);
    a.enter(|ma| {
        ma.scope(|ma, sa| {
            let mine = ma.alloc::<Cell>(sa, 0).expect("alloc");
            b.enter(|mb| {
                mb.scope(|mb, sb| {
                    let theirs = mb.alloc::<Cell>(sb, 0).expect("alloc");
                    // Heap b's reference into heap a's object.
                    mine.get(ma).set(ma, Cell::NEXT, Some(theirs.get(mb)));
                });
            });
        });
    });
}
