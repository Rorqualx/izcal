//! `TokenCell<T>`: interior mutability whose borrows are tied to the thread's
//! `ScriptToken` (GhostCell without the brand: the token is unique per thread
//! and the cell is `!Sync`, so a per-thread token is enough).
//!
//! - `borrow(&'a self, &'a ScriptToken) -> &'a T`
//! - `borrow_mut(&'a self, &'a mut ScriptToken) -> &'a mut T`
//!
//! Anything that can run script takes `&mut ScriptToken`, so a borrow cannot be
//! alive across it: E0499 / E0502 at compile time, no runtime flag.
//!
//! `TokenCell` must never hold traced GC fields (`Member`): the concurrent
//! marker reads those without the token (criterion 3; see the V8 crate's
//! `member_cell` sketch).

use crate::token::ScriptToken;
use std::cell::UnsafeCell;

#[repr(transparent)]
pub struct TokenCell<T: ?Sized> {
    value: UnsafeCell<T>,
}

// `UnsafeCell<T>` makes `TokenCell<T>` `!Sync` and `Send` iff `T: Send`, the
// same auto traits as `RefCell`. No manual impls.

impl<T> TokenCell<T> {
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        TokenCell {
            value: UnsafeCell::new(value),
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }

    #[inline(always)]
    pub fn replace(&self, value: T, tok: &mut ScriptToken) -> T {
        std::mem::replace(self.borrow_mut(tok), value)
    }

    #[inline(always)]
    pub fn set(&self, value: T, tok: &mut ScriptToken) {
        *self.borrow_mut(tok) = value;
    }

    #[inline(always)]
    pub fn get(&self, tok: &ScriptToken) -> T
    where
        T: Copy,
    {
        *self.borrow(tok)
    }
}

impl<T: ?Sized> TokenCell<T> {
    #[inline(always)]
    pub fn borrow<'a>(&'a self, tok: &'a ScriptToken) -> &'a T {
        let _ = tok;
        // SAFETY: `&'a ScriptToken` means no `&mut ScriptToken` (unique per thread,
        // token.rs) and so no `&mut T` from `borrow_mut` lives for `'a`; `!Sync`.
        unsafe { &*self.value.get() }
    }

    #[inline(always)]
    pub fn borrow_mut<'a>(&'a self, tok: &'a mut ScriptToken) -> &'a mut T {
        let _ = tok;
        // SAFETY: `tok`, the thread's only token, is exclusively borrowed for `'a`,
        // so no other `&T`/`&mut T` from any `TokenCell` lives for `'a`; `!Sync`.
        unsafe { &mut *self.value.get() }
    }

    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
}

impl<T: Default> Default for TokenCell<T> {
    fn default() -> Self {
        TokenCell::new(T::default())
    }
}

#[inline(always)]
fn byte_range<T: ?Sized>(c: &TokenCell<T>) -> (usize, usize) {
    let start = c.value.get() as *const u8 as usize;
    (start, start + std::mem::size_of_val(c))
}

/// Two cells overlap if their byte ranges intersect, or they start at the same
/// address (covers zero-sized cells and a cell nested at offset 0 of another).
#[inline(always)]
fn overlap<A: ?Sized, B: ?Sized>(a: &TokenCell<A>, b: &TokenCell<B>) -> bool {
    let (a0, a1) = byte_range(a);
    let (b0, b1) = byte_range(b);
    a0 == b0 || (a0 < b1 && b0 < a1)
}

/// Mutable borrows of two distinct cells under one token (the "scoped
/// multi-borrow" pattern). Panics if the cells overlap.
#[inline]
pub fn borrow_mut_2<'a, A: ?Sized, B: ?Sized>(
    a: &'a TokenCell<A>,
    b: &'a TokenCell<B>,
    tok: &'a mut ScriptToken,
) -> (&'a mut A, &'a mut B) {
    let _ = tok;
    assert!(!overlap(a, b), "borrow_mut_2: cells overlap");
    // SAFETY: as in `borrow_mut`, the token proves no other borrow of any
    // `TokenCell` on this thread is alive for `'a`; the two cells do not
    // overlap (checked above), so the two `&mut` do not alias.
    unsafe { (&mut *a.value.get(), &mut *b.value.get()) }
}

/// Three-cell form of [`borrow_mut_2`].
#[inline]
pub fn borrow_mut_3<'a, A: ?Sized, B: ?Sized, C: ?Sized>(
    a: &'a TokenCell<A>,
    b: &'a TokenCell<B>,
    c: &'a TokenCell<C>,
    tok: &'a mut ScriptToken,
) -> (&'a mut A, &'a mut B, &'a mut C) {
    let _ = tok;
    assert!(
        !overlap(a, b) && !overlap(a, c) && !overlap(b, c),
        "borrow_mut_3: cells overlap"
    );
    // SAFETY: as in `borrow_mut_2`, for three pairwise non-overlapping cells.
    unsafe {
        (
            &mut *a.value.get(),
            &mut *b.value.get(),
            &mut *c.value.get(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{enter, lend, reenter};
    use std::rc::Rc;

    #[test]
    fn borrow_and_mutate() {
        let c = TokenCell::new(vec![1, 2, 3]);
        enter(|t| {
            c.borrow_mut(t).push(4);
            let a = c.borrow(t);
            let b = c.borrow(t); // many shared borrows are fine
            assert_eq!(a.len() + b.len(), 8);
        });
        assert_eq!(c.into_inner(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn reentrant_mutation_is_seen_after_lend() {
        let c = Rc::new(TokenCell::new(String::from("before")));
        enter(|t| {
            let c2 = c.clone();
            lend(t, move || {
                reenter(|t2| c2.borrow_mut(t2).push_str("+script"))
            });
            assert_eq!(c.borrow(t), "before+script");
        });
    }

    #[test]
    fn multi_borrow_distinct() {
        let a = TokenCell::new(1u64);
        let b = TokenCell::new(String::from("x"));
        let c = TokenCell::new([0u8; 3]);
        enter(|t| {
            let (x, y, z) = borrow_mut_3(&a, &b, &c, t);
            *x += 1;
            y.push('y');
            z[0] = 9;
            let (x, z) = borrow_mut_2(&a, &c, t);
            *x += u64::from(z[0]);
        });
        assert_eq!(a.into_inner(), 11);
        assert_eq!(b.into_inner(), "xy");
    }

    #[test]
    #[should_panic(expected = "overlap")]
    fn multi_borrow_same_cell_panics() {
        let a = TokenCell::new(1u32);
        enter(|t| {
            let _ = borrow_mut_2(&a, &a, t);
        });
    }

    #[test]
    #[should_panic(expected = "overlap")]
    fn multi_borrow_nested_cell_panics() {
        // A cell inside another cell: the outer `&mut` would cover the inner.
        let outer = TokenCell::new((TokenCell::new(5u32), 6u32));
        enter(|t| {
            // Obtain `&TokenCell<u32>` for the inner cell without a borrow of
            // the outer one being alive afterwards: go through a raw-free path,
            // `borrow` then re-borrow the field.
            let inner: *const TokenCell<u32> = &outer.borrow(t).0;
            // SAFETY: test-only; `outer` lives for the whole closure and is not
            // moved, so the pointer is valid. This mimics code that reached the
            // inner cell through another path.
            let inner = unsafe { &*inner };
            let _ = borrow_mut_2(&outer, inner, t);
        });
    }

    #[test]
    #[should_panic(expected = "overlap")]
    fn multi_borrow_zst_same_cell_panics() {
        let z = TokenCell::new(());
        enter(|t| {
            let _ = borrow_mut_2(&z, &z, t);
        });
    }

    #[test]
    fn zst_distinct_cells() {
        let z1 = TokenCell::new(());
        let z2 = TokenCell::new(());
        // Distinct ZST locals may share an address; then the check is
        // conservative and panics, which is acceptable. Only assert no UB.
        enter(|t| {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = borrow_mut_2(&z1, &z2, t);
            }));
        });
    }

    #[test]
    fn drop_and_replace() {
        let c = TokenCell::new(String::from("a"));
        enter(|t| {
            let old = c.replace(String::from("b"), t);
            assert_eq!(old, "a");
            c.set(String::from("c"), t);
            assert_eq!(c.borrow(t), "c");
        });
    }

    #[test]
    fn get_mut_without_token() {
        let mut c = TokenCell::new(3);
        *c.get_mut() += 1;
        enter(|t| assert_eq!(c.get(t), 4));
    }

    #[test]
    fn slices_unsized() {
        let c: Box<TokenCell<[u32]>> = {
            let b: Box<TokenCell<[u32; 4]>> = Box::new(TokenCell::new([1, 2, 3, 4]));
            b
        };
        enter(|t| {
            c.borrow_mut(t)[2] = 30;
            assert_eq!(c.borrow(t).iter().sum::<u32>(), 37);
        });
    }
}
