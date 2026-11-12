//! The same class again, in the spelling that escaped every rule that
//! reads the trait's name: a macro that takes the trait as an argument,
//! so neither the macro's body nor any impl in the source says
//! `Object`. Measured 10 November 2026: with the three text rules in
//! place this compiled and was reported by nothing.
//!
//! It is refused now for the reason the case above is, which is the
//! point of putting the unavoidable name in the impl rather than
//! writing a fourth rule about an avoidable one.
//!
//! EXPECT: does not compile -- the witness is missing from the impl.
// EXPECT-ERROR: E0046
// EXPECT-NOTE: missing `LAYOUT_WITNESS` in implementation

macro_rules! plant {
    ($t:path, $ty:ty) => {
        impl $t for $ty {
            const LAYOUT: hg_heap::Layout = hg_heap::Layout {
                name: "planted",
                traced: 8,
                weak: 0,
                nodes: 0,
                raw_words: 0,
                tail: hg_heap::Tail::None,
            };
        }
    };
}

pub enum Forged {}

impl hg_heap::Referent for Forged {}

plant!(hg_heap::Object, Forged);
