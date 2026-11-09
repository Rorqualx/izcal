//! A mutator moved to another thread. Two threads in one agent heap
//! would race the collector; `Mutator` is `!Send`.
//!
//! EXPECT: does not compile — `Mutator` cannot be sent between threads.

use std::sync::Arc;

use hg_heap::{Heap, HeapConfig, ProcessCage, Stress};
use hg_substrate::NotRecording;
use hg_substrate_os::memory::OsMemory;

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
        std::thread::scope(|sc| {
            sc.spawn(move || {
                m.scope(|_m, _s| {});
            });
        });
    });
}
