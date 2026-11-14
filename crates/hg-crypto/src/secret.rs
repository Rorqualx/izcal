//! Secret bytes that do not leave by accident (NW-D23).
//!
//! The design has one idea in it: **there is no accessor that returns
//! the bytes.** [`Secret::expose_bytes`] hands them to a closure and
//! takes them back, so a secret cannot be stored somewhere else
//! without the storing being written out at the call site. A `fn
//! as_bytes(&self) -> &[u8]` would make every leak a one-line change
//! that reviews as nothing.
//!
//! What follows from that:
//!
//! - `Debug` prints the length and nothing else, so a secret in a log
//!   line or a panic message is not a secret in a log line.
//! - Comparison is [`crate::ct::eq`], never `==` on the bytes, so
//!   comparing two secrets takes the same time whatever they are.
//! - `Drop` overwrites the bytes.
//!
//! # What the overwrite is and is not
//!
//! It writes zeros and then passes the buffer through
//! [`core::hint::black_box`], which is what keeps the compiler from
//! removing a write to memory it can see is dead. That is the best a
//! crate with `unsafe_code = "forbid"` can do, and it is **not** a
//! guarantee that no copy of the secret remains: the value may have
//! been moved, spilled to the stack or paged out before this runs.
//! Removing those copies needs locked, non-dumpable memory from the
//! substrate, which is NW-M1's work and is named here so the gap is
//! not mistaken for a solved problem.

use core::fmt;

/// `N` secret bytes.
///
/// The length is in the type, because every secret this crate handles
/// has a length fixed by its algorithm, and a runtime length is one
/// more thing that can be wrong at a comparison.
pub struct Secret<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> Secret<N> {
    /// Takes ownership of `bytes`.
    #[must_use]
    pub const fn new(bytes: [u8; N]) -> Self {
        Self { bytes }
    }

    /// `N` zero bytes.
    #[must_use]
    pub const fn zero() -> Self {
        Self { bytes: [0; N] }
    }

    /// How many bytes it holds. Not secret: every algorithm's output
    /// length is public.
    #[must_use]
    pub const fn len(&self) -> usize {
        N
    }

    /// Whether it holds no bytes at all, which only `Secret<0>` does.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        N == 0
    }

    /// Runs `f` on the bytes and returns what it returns.
    ///
    /// The one way to read a secret. Whatever `f` returns leaves this
    /// call, so a caller that wants to copy the bytes out can — but it
    /// has to write that down.
    pub fn expose_bytes<R>(&self, f: impl FnOnce(&[u8; N]) -> R) -> R {
        f(&self.bytes)
    }

    /// Runs `f` on the bytes, which it may change.
    ///
    /// For a primitive that fills a secret in place rather than
    /// returning one.
    pub fn expose_bytes_mut<R>(&mut self, f: impl FnOnce(&mut [u8; N]) -> R) -> R {
        f(&mut self.bytes)
    }

    /// Whether two secrets hold the same bytes, in time that does not
    /// depend on where they differ.
    #[must_use]
    pub fn ct_eq(&self, other: &Self) -> bool {
        crate::ct::eq(&self.bytes, &other.bytes)
    }
}

impl<const N: usize> fmt::Debug for Secret<N> {
    /// The length, and nothing else.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret<{N}>(..)")
    }
}

impl<const N: usize> Drop for Secret<N> {
    fn drop(&mut self) {
        self.bytes.fill(0);
        // Without this the write above is dead on every path and the
        // optimizer may remove it. `black_box` is the stable way to
        // say "this memory is observed" with no `unsafe`.
        let _ = core::hint::black_box(&self.bytes);
    }
}

impl<const N: usize> Clone for Secret<N> {
    fn clone(&self) -> Self {
        Self { bytes: self.bytes }
    }
}

impl<const N: usize> From<[u8; N]> for Secret<N> {
    fn from(bytes: [u8; N]) -> Self {
        Self::new(bytes)
    }
}
