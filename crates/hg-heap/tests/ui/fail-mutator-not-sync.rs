//! A mutator shared with another thread. Even a shared borrow lets the
//! other thread read the cage while this one allocates into it;
//! `Mutator` is `!Sync`.
//!
//! EXPECT: does not compile — `Mutator` cannot be shared between threads.

// EXPECT-ERROR: E0277
use hg_heap::Heap;

pub fn case(a: &mut Heap) {
    a.enter(|m| {
        let shared: &hg_heap::Mutator<'_> = m;
        std::thread::scope(|sc| {
            sc.spawn(move || {
                let _ = shared.progress();
            });
        });
    });
}
