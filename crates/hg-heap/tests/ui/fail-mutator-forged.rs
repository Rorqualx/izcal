//! A mutator built by naming its fields instead of entering a heap.
//! Criterion 1 asks that the token be derivable only from the heap; a
//! struct literal would be a second way to mint one, with no brand and
//! no heap behind it.
//!
//! EXPECT: does not compile — the fields are private.

// EXPECT-ERROR: E0451
use hg_heap::Mutator;

pub fn case() {
    let _m: Mutator<'static> = Mutator {
        state: todo!(),
        _brand: todo!(),
        _not_send: todo!(),
    };
}
