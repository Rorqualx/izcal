//! A `Gc` kept across a GC point. The collector may move the object, so
//! the reference would be stale; only a `Root` survives.
//!
//! EXPECT: does not compile — the mutator is borrowed.

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
            let g = x.get(m);
            // A GC point with `g` still live.
            m.collect();
            let _ = g.len(m);
        });
    });
}
