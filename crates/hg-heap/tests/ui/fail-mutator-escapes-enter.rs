//! A mutator carried out of its `Heap::enter`. The brand `'h` is fresh
//! and invariant per enter, so nothing branded with it can be named
//! outside (JS-D18, soundness step 1).
//!
//! EXPECT: does not compile — the brand cannot escape.

// EXPECT-ERROR: LIFETIME
// EXPECT-NOTE: mutable references are invariant over their type parameter
use hg_heap::{Heap, Mutator};

pub fn case<'any>(a: &'any mut Heap) -> &'any Mutator<'any> {
    a.enter(|m| m)
}
