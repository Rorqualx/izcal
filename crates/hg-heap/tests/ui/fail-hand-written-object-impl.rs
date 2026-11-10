//! JS-10's planted-bug class "a hand-written `Trace`", in the spelling
//! that escaped the lint: the object traits reached through a renamed
//! import, so the trait's own name never appears where a rule that
//! reads the source would look for it.
//!
//! H-3 cannot seal the trait -- `hg_heap::object!` expands in the
//! calling crate, so a private supertrait would not be nameable there
//! and would seal nothing. What it can do is make the impl name
//! something: `Object::LAYOUT_WITNESS` has one value, from
//! `hg_heap::layout_witness()`, and an impl that leaves it out does not
//! compile. That is this case. An impl that writes it does compile, and
//! is caught by H-3's `heap-object-witness` rule and by clippy's
//! `disallowed-methods`, which resolves the path instead of reading it.
//!
//! EXPECT: does not compile -- the witness is missing from the impl.
// EXPECT-ERROR: E0046
// EXPECT-NOTE: missing `LAYOUT_WITNESS` in implementation
use hg_heap::{Layout, Object as Obj, Referent as Ref, Tail};

pub enum Forged {}

impl Ref for Forged {}

impl Obj for Forged {
    // A layout that claims eight traced slots on an object that has
    // none: under H-2 the worst it can do is make the tracer read the
    // wrong words of every object of this type, which is an in-cage
    // wrong answer. The rule is still that the macro writes layouts.
    const LAYOUT: Layout = Layout {
        name: "Forged",
        traced: 8,
        weak: 0,
        nodes: 0,
        raw_words: 0,
        tail: Tail::None,
    };
}
