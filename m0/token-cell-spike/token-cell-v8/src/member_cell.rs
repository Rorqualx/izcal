//! Criterion (3) sketch: traced links stay out of `TokenCell`.
//!
//! cppgc's concurrent marker reads `Member` slots from another thread without
//! any token, so a `Member` inside a `TokenCell` would be read while the
//! mutator holds `&mut` to it. Instead, each traced link is its own
//! `MemberCell`: a slot that is written through `&self` (the C++ `Member`
//! assignment is an atomic store plus write barrier, `member.h`
//! `SetRawAtomic` + `AssigningWriteBarrier`) and never lends a reference into
//! itself: `get` returns a stack pointer to the pointee.
//!
//! Gap found: the fork's `Member::set` takes `&mut self`, so `MemberCell::set`
//! has to form a `&mut Member` that the marker may read concurrently. The
//! bytes are only touched by C++ atomics, but in Rust's model it is a `&mut`
//! aliased by another thread. M2 should add a fork API that assigns through a
//! raw pointer (or `&self`), e.g. `Member::assign_shared(&self, ..)`.
//!
//! Generator rule (not built here): a `#[gc]` struct's `TokenCell<T>` fields
//! must have `T: NoTraced` (no `Member`, `WeakMember`, `TracedReference`, GC
//! handles); traced fields must be `MemberCell` and appear in `trace`.

use std::cell::UnsafeCell;
use token_cell::TokenCell;
use token_cell::dom::NodeData;
use v8::cppgc::{GarbageCollected, GetRustObj, Member, Traced, UnsafePtr, Visitor};

pub struct MemberCell<T: GarbageCollected>(UnsafeCell<Member<T>>);

impl<T: GarbageCollected> MemberCell<T> {
    pub fn empty() -> Self {
        MemberCell(UnsafeCell::new(Member::empty()))
    }

    /// No token needed: no reference into the slot ever escapes.
    pub fn set(&self, v: &impl GetRustObj<T>) {
        // SAFETY: single mutator thread (`MemberCell` is `!Sync`); no `&Member`
        // from this slot is alive (none is ever handed out). See the module
        // comment for the concurrent-marker caveat.
        unsafe { (*self.0.get()).set(v) }
    }

    pub fn clear(&self) {
        // SAFETY: as in `set`.
        unsafe { *self.0.get() = Member::empty() }
    }

    /// A stack pointer to the pointee (`UnsafePtr` must stay on the stack).
    pub fn get(&self) -> Option<UnsafePtr<T>> {
        // SAFETY: the slot is traced by its owner (`DomNode::trace`), so a
        // non-null pointer refers to a live object; the `&Member` is
        // temporary and dropped before return.
        unsafe { UnsafePtr::new(&*self.0.get()) }
    }
}

impl<T: GarbageCollected> Traced for MemberCell<T> {
    fn trace(&self, v: &mut Visitor) {
        // SAFETY: called by the GC; reads the slot through the C++ visitor.
        v.trace(unsafe { &*self.0.get() })
    }
}

/// A cppgc DOM node: links traced and outside TokenCell; script-reachable
/// state in TokenCell and not traced.
pub struct DomNode {
    pub parent: MemberCell<DomNode>,
    pub first_child: MemberCell<DomNode>,
    pub next_sibling: MemberCell<DomNode>,
    pub prev_sibling: MemberCell<DomNode>,
    pub data: TokenCell<NodeData>,
    pub drops: &'static std::sync::atomic::AtomicUsize,
}

// SAFETY: every `Member` reachable from `DomNode` is in a `MemberCell` field
// and traced below; `data` holds no GC pointer.
unsafe impl GarbageCollected for DomNode {
    fn trace(&self, v: &mut Visitor) {
        v.trace(&self.parent);
        v.trace(&self.first_child);
        v.trace(&self.next_sibling);
        v.trace(&self.prev_sibling);
    }
    fn get_name(&self) -> &'static std::ffi::CStr {
        c"DomNode"
    }
}

impl Drop for DomNode {
    // Allowed under CS-V8-9: touches no other GC object.
    fn drop(&mut self) {
        self.drops.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
