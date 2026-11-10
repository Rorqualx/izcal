//! A reference minted from a raw slot through the heap's own
//! constructor. It exists, because the collector and the field accessors
//! need it, but it is crate-private: outside `hg-heap` the only sources
//! of a `Gc` are an allocation, a field read and a root.
//!
//! EXPECT: does not compile — the constructor is private.

// EXPECT-ERROR: E0624
use hg_heap::{Any, Gc, Slot};

pub fn case(s: Slot) {
    let _g: Gc<'static, 'static, Any> = Gc::from_slot(s);
}
