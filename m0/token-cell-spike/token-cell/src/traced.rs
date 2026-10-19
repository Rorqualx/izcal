//! `NoTraced`: "holds no GC pointer the marker must see" (criterion 3).
//!
//! cppgc's concurrent marker reads traced slots (`Member`, `WeakMember`,
//! `TracedReference`) without the token, so none may sit inside a
//! `TokenCell`, whose `&mut` the mutator holds while the marker runs. The
//! binding generator requires `T: NoTraced` for every `TokenCell<T>` field of a
//! GC object (the V8 crate's `gc_object!` stands in for it).
//!
//! Stable Rust has no auto traits, so `NoTraced` is opt-in: implemented here
//! for std types whose contents are themselves `NoTraced`, and for a struct by
//! `no_traced!`, which checks every field. GC handle types never implement it.
//! A hand-written `unsafe impl NoTraced` is an `unsafe` block like any other:
//! it needs a SAFETY comment and an inventory entry (CS-TY-13).

/// # Safety
///
/// The type contains no `Member`, `WeakMember`, `TracedReference` or other
/// slot that the GC traces or updates.
pub unsafe trait NoTraced {}

macro_rules! no_traced_leaf {
    ($($t:ty),* $(,)?) => {
        // SAFETY: these types hold no pointer into the GC heap.
        $(unsafe impl NoTraced for $t {})*
    };
}

no_traced_leaf!(
    (),
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    str,
    String,
    std::sync::atomic::AtomicUsize,
    std::sync::atomic::AtomicU32,
    std::sync::atomic::AtomicBool,
);

macro_rules! no_traced_wrapper {
    ($($w:ident)::+ <T>) => {
        // SAFETY: holds only `T`s, which are `NoTraced`.
        unsafe impl<T: NoTraced + ?Sized> NoTraced for $($w)::+<T> {}
    };
}

no_traced_wrapper!(Box<T>);
no_traced_wrapper!(std::rc::Rc<T>);
no_traced_wrapper!(std::cell::Cell<T>);
no_traced_wrapper!(crate::cell::TokenCell<T>);

// SAFETY: holds only `T`s, which are `NoTraced`.
unsafe impl<T: NoTraced> NoTraced for Vec<T> {}
// SAFETY: as above.
unsafe impl<T: NoTraced> NoTraced for Option<T> {}
// SAFETY: as above.
unsafe impl<T: NoTraced> NoTraced for [T] {}
// SAFETY: as above.
unsafe impl<T: NoTraced, const N: usize> NoTraced for [T; N] {}
// SAFETY: a shared reference to a `NoTraced` value.
unsafe impl<T: NoTraced + ?Sized> NoTraced for &'static T {}
// SAFETY: both halves are `NoTraced`.
unsafe impl<A: NoTraced, B: NoTraced> NoTraced for (A, B) {}

/// Declares a struct and implements `NoTraced` for it after checking that
/// every field type is `NoTraced` (a compile error otherwise). The stand-in for
/// `#[derive(NoTraced)]`; non-generic structs with named fields only.
#[macro_export]
macro_rules! no_traced {
    (
        $(#[$m:meta])*
        $vis:vis struct $name:ident {
            $( $(#[$fm:meta])* $fvis:vis $f:ident : $t:ty ),* $(,)?
        }
    ) => {
        $(#[$m])*
        $vis struct $name { $( $(#[$fm])* $fvis $f : $t ),* }

        #[allow(dead_code)] // type-checked, never called
        const _: () = {
            fn assert_no_traced<T: $crate::traced::NoTraced + ?Sized>() {}
            fn check() { $( assert_no_traced::<$t>(); )* }
        };

        // SAFETY: every field type is `NoTraced`, checked just above.
        unsafe impl $crate::traced::NoTraced for $name {}
    };
}
