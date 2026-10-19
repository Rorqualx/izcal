//! Criterion (3) generator check, spike form: `gc_object!` stands in for the
//! binding generator's `#[gc]` structs.
//!
//! A GC object has three kinds of field, and the macro decides the type of
//! each, so a struct cannot get them wrong:
//! - `traced`: written as the pointee type, emitted as `MemberCell<T>` and
//!   traced, so every traced link is outside `TokenCell` and in `trace`;
//! - `cells`: emitted as `TokenCell<T>`, with `T: NoTraced` required;
//! - `plain`: emitted as written, with `NoTraced` required (an untraced
//!   `Member` would be a dangling pointer after GC).

/// Declares a cppgc object (see the module docs).
#[macro_export]
macro_rules! gc_object {
    (
        $(#[$m:meta])*
        $vis:vis struct $name:ident {
            traced { $( $tvis:vis $tf:ident : $tt:ty ),* $(,)? }
            cells { $( $cvis:vis $cf:ident : $ct:ty ),* $(,)? }
            plain { $( $pvis:vis $pf:ident : $pt:ty ),* $(,)? }
        }
        name = $cname:literal;
    ) => {
        $(#[$m])*
        $vis struct $name {
            $( $tvis $tf: $crate::member_cell::MemberCell<$tt>, )*
            $( $cvis $cf: $crate::token_cell::TokenCell<$ct>, )*
            $( $pvis $pf: $pt, )*
        }

        #[allow(dead_code)] // type-checked, never called
        const _: () = {
            fn assert_no_traced<T: $crate::token_cell::NoTraced + ?Sized>() {}
            fn check() {
                $( assert_no_traced::<$ct>(); )*
                $( assert_no_traced::<$pt>(); )*
            }
        };

        // SAFETY: every traced slot is a `MemberCell` field traced below; the
        // other fields are `NoTraced` (checked above), so hold no GC pointer.
        unsafe impl $crate::v8::cppgc::GarbageCollected for $name {
            fn trace(&self, _v: &mut $crate::v8::cppgc::Visitor) {
                $( _v.trace(&self.$tf); )*
            }
            fn get_name(&self) -> &'static ::std::ffi::CStr {
                $cname
            }
        }
    };
}
