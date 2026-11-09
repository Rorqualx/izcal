//! Positive control: within one agent heap the branded API compiles, so
//! a failure in the cases beside this one is the brand rejecting
//! something and not the harness being broken.
//!
//! EXPECT: compiles.

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
    a.enter(|m| {
        m.scope(|m, s| {
            let x = m.alloc::<Cell>(s, 0).expect("alloc");
            let y = m.alloc::<Cell>(s, 0).expect("alloc");
            // Both references are of heap `a`'s brand, so this compiles.
            x.get(m).set(m, Cell::NEXT, Some(y.get(m)));
        });
    });
}
