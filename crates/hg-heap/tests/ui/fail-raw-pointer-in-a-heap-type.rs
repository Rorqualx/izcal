//! JS-10's planted-bug class "a raw pointer in a heap type". H-1 says no
//! raw pointer can be stored in the cage; the premise H-2 rests on is
//! that corrupt cage contents can do nothing worse than name another
//! part of the cage, and a host address sitting in a raw word is the one
//! value for which that is untrue.
//!
//! This case compiled until `RawValue` was sealed. Nothing had gone
//! wrong — no crate had done it — but the rule was stated as closed by
//! construction and was not, so the first planted bug of this class
//! escaped. The seal is a private supertrait, so the five raw types
//! `hg-heap` provides are the only ones there are.
//!
//! EXPECT: does not compile — `RawValue` is sealed.

// EXPECT-ERROR: E0277
// EXPECT-NOTE: RawWord` is not implemented for `Smuggled
use hg_heap::{Heap, RawValue};

/// A raw pointer wearing a `RawValue` costume.
#[derive(Clone, Copy)]
pub struct Smuggled(*const u8);

impl RawValue for Smuggled {
    fn to_bits(self) -> u64 {
        self.0 as u64
    }
    fn from_bits(bits: u64) -> Self {
        Self(bits as *const u8)
    }
}

hg_heap::object! {
    /// An object whose raw word is a host pointer.
    pub Leaky {
        raw PTR: Smuggled,
        tail none,
    }
}

pub fn case(a: &mut Heap, target: &u8) {
    a.enter(|m| {
        m.scope(|m, s| {
            let o = m.alloc::<Leaky>(s, 0).expect("alloc");
            o.get(m)
                .set_raw(m, Leaky::PTR, Smuggled(std::ptr::from_ref(target)));
        });
    });
}
