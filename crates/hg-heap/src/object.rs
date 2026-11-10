//! How the collector sees a type (heap design H-3, H-4). An object in the
//! cage is a 4-byte header (its type index), a 4-byte length word, then
//! its body: the strong slots, the weak slots, the raw words, and an
//! optional variable tail of slots or bytes. Nothing here is a Rust
//! struct in cage memory; fields are reached through typed descriptors
//! ([`Traced`], [`Weak`], [`Raw`]) that [`object!`](crate::object) makes.

use core::marker::PhantomData;

/// The object header's size: the type index and the length word.
pub(crate) const HEADER_BYTES: u64 = 8;

/// An object's variable tail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tail {
    /// No tail: the object's size is fixed.
    None,
    /// A tail of traced (strong) slots, as many as the length word says.
    Slots,
    /// A tail of raw bytes, as many as the length word says.
    Bytes,
}

/// One type's layout, as the collector reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    /// The type's name, for diagnostics.
    pub name: &'static str,
    /// Strong slots at the start of the body.
    pub traced: u32,
    /// Weak slots after the strong ones (heap design H-10).
    pub weak: u32,
    /// Node slots after the weak ones: 8 bytes each, 8-aligned, each
    /// naming a node of the heap's node space (heap design H-4 as
    /// amended by decision 1; JS-D4 (a)). They come after the 32-bit
    /// slots so that [`Layout::slot_offset`] is unchanged for the traced
    /// and weak ones.
    pub nodes: u32,
    /// Raw 64-bit words after the slots.
    pub raw_words: u32,
    /// The variable tail.
    pub tail: Tail,
}

impl Layout {
    /// The bytes of the fixed body: the 32-bit slots padded to 8, then
    /// the node slots, then the raw words.
    pub(crate) const fn fixed_body(&self) -> u64 {
        self.nodes_end() - HEADER_BYTES + self.raw_words as u64 * 8
    }

    /// Where the 32-bit slots end, padded to 8 so the node slots and the
    /// raw words are aligned.
    const fn slots_end(&self) -> u64 {
        let slots = (self.traced as u64 + self.weak as u64) * 4;
        HEADER_BYTES + slots.next_multiple_of(8)
    }

    /// Where the node slots end.
    const fn nodes_end(&self) -> u64 {
        self.slots_end() + self.nodes as u64 * 8
    }

    /// The byte offset of node slot `i` from the object's start.
    pub(crate) const fn node_offset(&self, i: u32) -> u64 {
        self.slots_end() + i as u64 * 8
    }

    /// The byte offset of slot `i` from the object's start.
    pub(crate) const fn slot_offset(i: u32) -> u64 {
        HEADER_BYTES + i as u64 * 4
    }

    /// The byte offset of raw word `i` from the object's start.
    pub(crate) const fn raw_offset(&self, i: u32) -> u64 {
        self.nodes_end() + i as u64 * 8
    }

    /// The byte offset of the tail from the object's start.
    pub(crate) const fn tail_offset(&self) -> u64 {
        HEADER_BYTES + self.fixed_body()
    }

    /// The object's total size, in bytes (a multiple of 8), for a tail of
    /// `len` elements; `None` if it overflows.
    pub(crate) const fn size(&self, len: u32) -> Option<u64> {
        let tail = match self.tail {
            Tail::None => 0,
            Tail::Slots => len as u64 * 4,
            Tail::Bytes => len as u64,
        };
        let total = self.tail_offset() + tail;
        Some(total.next_multiple_of(8))
    }
}

/// A type whose objects live in the heap. Implemented only by
/// [`object!`](crate::object); a hand-written impl is a lint error (heap
/// design H-3). The layout decides what the collector traces; a wrong one
/// gives wrong answers inside the cage, never an access outside it.
pub trait Object: Referent {
    /// The type's layout.
    const LAYOUT: Layout;

    /// Proof that [`object!`](crate::object) wrote this impl.
    ///
    /// The trait cannot be sealed: the macro expands in the calling
    /// crate, so a private supertrait would not be nameable there and
    /// would seal nothing. What this const does instead is make the
    /// impl *say a word*. A hand-written impl cannot leave it out —
    /// that is `E0046`, a compile error — and cannot write it without
    /// naming `LAYOUT_WITNESS` and some source for its value. Those
    /// names belong to this mechanism and to nothing else in the tree,
    /// so the H-3 rules can own them: `heap-object-witness` reports any
    /// crate but `hg-heap` that names one, and clippy's
    /// `disallowed-methods` carries [`layout_witness`] as a second
    /// layer that resolves paths rather than reading them, so a renamed
    /// import cannot spell its way past it.
    ///
    /// Measured 10 November 2026: the three text rules on the trait's
    /// own name were all escaped by a macro that takes the trait as an
    /// argument (`plant!(hg_heap::Object, Evil)` compiled, reported by
    /// nothing), which is what put an unavoidable name in the impl
    /// instead of another rule on an avoidable one.
    #[doc(hidden)]
    const LAYOUT_WITNESS: LayoutWitness;
}

/// Proof that [`object!`](crate::object) wrote an [`Object`] impl. It
/// has one constructor, [`layout_witness`], and that function's name is
/// in clippy's `disallowed-methods` and in H-3's lint.
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct LayoutWitness(());

/// The only way to make a [`LayoutWitness`]. Called by
/// [`object!`](crate::object) under an `#[expect]` naming the clippy
/// rule, so a legitimate object type costs its author nothing and every
/// other caller is a finding.
#[doc(hidden)]
#[must_use]
pub const fn layout_witness() -> LayoutWitness {
    LayoutWitness(())
}

/// Something a slot can reference: an [`Object`] type or [`Any`].
pub trait Referent: 'static {}

/// A reference of unknown type: what a JS value field holds. A checked
/// cast ([`Gc::cast`](crate::Gc::cast)) turns it into a typed reference.
#[derive(Debug)]
pub enum Any {}

impl Referent for Any {}

/// A strong slot of `T` that references `U`.
#[derive(Debug)]
pub struct Traced<T, U> {
    index: u32,
    _p: PhantomData<fn() -> (T, U)>,
}

/// A weak slot of `T` that references `U`: not traced, and cleared when
/// its target dies (heap design H-10).
#[derive(Debug)]
pub struct Weak<T, U> {
    index: u32,
    _p: PhantomData<fn() -> (T, U)>,
}

/// A node slot of `T`: a strong reference to a node of the heap's node
/// space, 8 bytes holding its slot index and epoch (JS-D4 (a)). The
/// lazily created wrapper of a node holds one of these, and because it is
/// traced strongly the node cannot die while the wrapper lives — which
/// is the half of "held strongly once created" that needs the collector's
/// help (decision 2).
#[derive(Debug)]
pub struct Node<T> {
    index: u32,
    _p: PhantomData<fn() -> T>,
}

impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Node<T> {}

impl<T> Node<T> {
    /// The descriptor for index `index`. For [`object!`](crate::object).
    #[doc(hidden)]
    pub const fn new(index: u32) -> Self {
        Self {
            index,
            _p: PhantomData,
        }
    }

    /// The field's index.
    pub const fn index(self) -> u32 {
        self.index
    }
}

/// A raw 64-bit word of `T` that holds a `V`.
#[derive(Debug)]
pub struct Raw<T, V> {
    index: u32,
    _p: PhantomData<fn() -> (T, V)>,
}

macro_rules! descriptor_impls {
    ($t:ident) => {
        impl<T, U> Clone for $t<T, U> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<T, U> Copy for $t<T, U> {}
        impl<T, U> $t<T, U> {
            /// The descriptor for index `index`. For [`object!`](crate::object).
            #[doc(hidden)]
            pub const fn new(index: u32) -> Self {
                Self {
                    index,
                    _p: PhantomData,
                }
            }

            /// The field's index.
            pub const fn index(self) -> u32 {
                self.index
            }
        }
    };
}
descriptor_impls!(Traced);
descriptor_impls!(Weak);
descriptor_impls!(Raw);

/// Sealing supertraits. The module is private, so nothing outside
/// `hg-heap` can name what is in it, and so nothing outside can
/// implement a trait that requires it.
mod sealed {
    /// The seal on [`RawValue`](super::RawValue).
    pub trait RawWord {}
}

/// A value a raw word can hold.
///
/// **Sealed.** The implementations are the five below and no crate can
/// add another, which is how H-1 — "no raw pointer can be stored in the
/// cage" — is closed by construction rather than by a rule somebody has
/// to remember. Before it was sealed, a crate could define
/// `struct P(*const u8)` with `to_bits` returning `self.0 as u64` and
/// store a host address in a raw word: that planted bug compiled, which
/// is how the seal came to be written (JS-10's heap classes, JS-M0 plan
/// Phase 2 item 8). The refusal is now a fixture.
///
/// A new raw type belongs here, beside these, where the question "is it
/// a pointer?" is asked once.
pub trait RawValue: Copy + sealed::RawWord {
    /// The value as 64 bits.
    fn to_bits(self) -> u64;
    /// The value from 64 bits read from the cage (any bits: H-2).
    fn from_bits(bits: u64) -> Self;
}

impl sealed::RawWord for u64 {}
impl RawValue for u64 {
    fn to_bits(self) -> u64 {
        self
    }
    fn from_bits(bits: u64) -> Self {
        bits
    }
}

impl sealed::RawWord for u32 {}
impl RawValue for u32 {
    fn to_bits(self) -> u64 {
        u64::from(self)
    }
    fn from_bits(bits: u64) -> Self {
        // The low 32 bits; a corrupted high half is ignored.
        u32::try_from(bits & 0xFFFF_FFFF).unwrap_or(0)
    }
}

impl sealed::RawWord for i32 {}
impl RawValue for i32 {
    fn to_bits(self) -> u64 {
        u64::from(self.cast_unsigned())
    }
    fn from_bits(bits: u64) -> Self {
        u32::from_bits(bits).cast_signed()
    }
}

impl sealed::RawWord for f64 {}
impl RawValue for f64 {
    fn to_bits(self) -> u64 {
        f64::to_bits(self)
    }
    fn from_bits(bits: u64) -> Self {
        f64::from_bits(bits)
    }
}

impl sealed::RawWord for bool {}
impl RawValue for bool {
    fn to_bits(self) -> u64 {
        u64::from(self)
    }
    fn from_bits(bits: u64) -> Self {
        bits & 1 != 0
    }
}

/// Declares a heap object type: a marker type, its [`Object`] layout, and
/// a descriptor constant per field. Field names are the constants' names.
///
/// ```
/// hg_heap::object! {
///     /// A list cell.
///     pub Cell {
///         traced NEXT: Cell,
///         traced VALUE: hg_heap::Any,
///         weak OBSERVED: hg_heap::Any,
///         node OWNER_NODE,
///         raw COUNT: u32,
///         tail none,
///     }
/// }
/// ```
///
/// Every section must be present, in this order (`traced`, `weak`,
/// `node`, `raw`), each with zero or more fields, and `tail` is `none`,
/// `slots` or `bytes`. A `node` field names no type: it references a node
/// of the heap's node space, not another heap object.
#[macro_export]
macro_rules! object {
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident {
            $(traced $t:ident : $tt:ty,)*
            $(weak $w:ident : $wt:ty,)*
            $(node $n:ident,)*
            $(raw $r:ident : $rt:ty,)*
            tail $tail:ident,
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        $vis enum $name {}

        impl $crate::Referent for $name {}

        #[expect(
            clippy::disallowed_methods,
            reason = "H-3: object! is the only author of a layout, and the witness is what says so"
        )]
        impl $crate::Object for $name {
            const LAYOUT_WITNESS: $crate::LayoutWitness = $crate::layout_witness();

            const LAYOUT: $crate::Layout = $crate::Layout {
                name: stringify!($name),
                traced: $crate::object!(@count $($t)*),
                weak: $crate::object!(@count $($w)*),
                nodes: $crate::object!(@count $($n)*),
                raw_words: $crate::object!(@count $($r)*),
                tail: $crate::object!(@tail $tail),
            };
        }

        impl $name {
            $crate::object!(@fields $name, $vis, Traced, 0u32, $($t : $tt,)*);
            $crate::object!(@fields $name, $vis, Weak, $crate::object!(@count $($t)*), $($w : $wt,)*);
            $crate::object!(@nodes $name, $vis, 0u32, $($n,)*);
            $crate::object!(@fields $name, $vis, Raw, 0u32, $($r : $rt,)*);
        }
    };
    (@count) => { 0u32 };
    (@count $x:ident $($rest:ident)*) => { 1u32 + $crate::object!(@count $($rest)*) };
    (@fields $name:ident, $vis:vis, $kind:ident, $idx:expr, ) => {};
    (@fields $name:ident, $vis:vis, $kind:ident, $idx:expr, $f:ident : $ft:ty, $($rest:tt)*) => {
        #[doc = concat!("The field `", stringify!($f), "`.")]
        $vis const $f: $crate::$kind<$name, $ft> = $crate::$kind::new($idx);
        $crate::object!(@fields $name, $vis, $kind, $idx + 1u32, $($rest)*);
    };
    (@nodes $name:ident, $vis:vis, $idx:expr, ) => {};
    (@nodes $name:ident, $vis:vis, $idx:expr, $f:ident, $($rest:tt)*) => {
        #[doc = concat!("The node field `", stringify!($f), "`.")]
        $vis const $f: $crate::Node<$name> = $crate::Node::new($idx);
        $crate::object!(@nodes $name, $vis, $idx + 1u32, $($rest)*);
    };
    (@tail none) => { $crate::Tail::None };
    (@tail slots) => { $crate::Tail::Slots };
    (@tail bytes) => { $crate::Tail::Bytes };
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::object! {
        /// A test type.
        pub(crate) Pair {
            traced LEFT: Pair,
            traced RIGHT: Any,
            weak WATCH: Any,
            raw COUNT: u32,
            raw WEIGHT: f64,
            tail none,
        }
    }

    crate::object! {
        /// A test array.
        pub(crate) Array {
            traced OWNER: Any,
            tail slots,
        }
    }

    #[test]
    fn the_macro_numbers_fields_and_sizes_the_layout() {
        let l = Pair::LAYOUT;
        assert_eq!(
            (l.traced, l.weak, l.raw_words, l.tail),
            (2, 1, 2, Tail::None)
        );
        assert_eq!(Pair::LEFT.index(), 0);
        assert_eq!(Pair::RIGHT.index(), 1);
        assert_eq!(Pair::WATCH.index(), 2);
        assert_eq!(Pair::COUNT.index(), 0);
        assert_eq!(Pair::WEIGHT.index(), 1);
        // 3 slots = 12 bytes, padded to 16; two raw words.
        assert_eq!(l.fixed_body(), 32);
        assert_eq!(l.raw_offset(1), HEADER_BYTES + 16 + 8);
        assert_eq!(l.size(0), Some(40));

        let a = Array::LAYOUT;
        assert_eq!(a.tail, Tail::Slots);
        assert_eq!(Array::OWNER.index(), 0);
        assert_eq!(a.tail_offset(), HEADER_BYTES + 8);
        assert_eq!(a.size(3), Some(32)); // 8 + 8 + 12, rounded up to 8
    }

    #[test]
    fn raw_values_take_any_bits() {
        assert_eq!(<u32 as RawValue>::from_bits(u64::MAX), u32::MAX);
        assert_eq!(<i32 as RawValue>::from_bits((-5i32).to_bits()), -5);
        assert!(<bool as RawValue>::from_bits(3));
        assert_eq!(<f64 as RawValue>::from_bits(1.5f64.to_bits()), 1.5);
    }
}
