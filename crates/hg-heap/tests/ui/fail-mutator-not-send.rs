//! A mutator moved to another thread. Two threads in one agent heap
//! would race the collector and each other's root list; `Mutator` is
//! `!Send` (JS-D18).
//!
//! EXPECT: does not compile — `Mutator` cannot be sent between threads.

// EXPECT-ERROR: E0277
use hg_heap::Heap;

pub fn case(a: &mut Heap) {
    a.enter(|m| {
        std::thread::scope(|sc| {
            sc.spawn(move || {
                m.scope(|_m, _s| {});
            });
        });
    });
}
