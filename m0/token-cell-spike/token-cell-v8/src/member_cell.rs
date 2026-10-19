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
//! Fork fix (was the gap found by the spike): the fork's `Member::set` took
//! `&mut self`, so `set` formed a `&mut Member` the marker may read, and
//! `clear` was a plain store. Both were data races in Rust's model. The fork
//! branch `izcal/member-shared-assign` keeps the slot bytes in an `UnsafeCell`
//! and adds `Member::assign_shared(&self, ..)` and `clear_shared(&self)`, which
//! go through C++ `operator=(T*)`: an atomic store plus the write barrier.
//!
//! Generator rule (spike form in [`crate::gc`]): a `#[gc]` struct's
//! `TokenCell<T>` fields must have `T: NoTraced` (no `Member`, `WeakMember`,
//! `TracedReference`, GC handles); traced fields must be `MemberCell` and
//! appear in `trace`.

use token_cell::dom::NodeData;
use v8::cppgc::{GarbageCollected, GetRustObj, Member, Traced, UnsafePtr, Visitor};

/// `!Sync` (the fork's `Member` holds an `UnsafeCell`), so only the mutator
/// thread writes it; the marker reads it through C++ atomics.
pub struct MemberCell<T: GarbageCollected>(Member<T>);

impl<T: GarbageCollected> MemberCell<T> {
    pub fn empty() -> Self {
        MemberCell(Member::empty())
    }

    /// No token needed: no reference into the slot ever escapes.
    pub fn set(&self, v: &impl GetRustObj<T>) {
        // SAFETY: `get` never lends a `&T` (only a stack `UnsafePtr`), and
        // `MemberCell` is `!Sync`, so this runs on the mutator thread.
        unsafe { self.0.assign_shared(v) }
    }

    pub fn clear(&self) {
        // SAFETY: as in `set`.
        unsafe { self.0.clear_shared() }
    }

    /// A stack pointer to the pointee (`UnsafePtr` must stay on the stack).
    pub fn get(&self) -> Option<UnsafePtr<T>> {
        // SAFETY: the slot is traced by its owner (`DomNode::trace`), so a
        // non-null pointer refers to a live object.
        unsafe { UnsafePtr::new(&self.0) }
    }
}

impl<T: GarbageCollected> Traced for MemberCell<T> {
    fn trace(&self, v: &mut Visitor) {
        v.trace(&self.0)
    }
}

crate::gc_object! {
    /// A cppgc DOM node: links traced and outside TokenCell; script-reachable
    /// state in TokenCell and not traced.
    pub struct DomNode {
        traced {
            pub parent: DomNode,
            pub first_child: DomNode,
            pub next_sibling: DomNode,
            pub prev_sibling: DomNode,
        }
        cells {
            pub data: NodeData,
        }
        plain {
            pub drops: &'static std::sync::atomic::AtomicUsize,
        }
    }
    name = c"DomNode";
}

impl Drop for DomNode {
    // Allowed under CS-V8-9: touches no other GC object.
    fn drop(&mut self) {
        self.drops.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
