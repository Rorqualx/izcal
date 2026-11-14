//! QUIC ACK ranges (RFC 9000 §19.3).
//!
//! An ACK frame names the largest packet it acknowledges and then
//! walks *downwards*: a first range below the largest, then pairs of
//! (gap, length) each describing a hole and a run below the previous
//! one. Every step is a subtraction of a peer-supplied number from a
//! number the peer also supplied.
//!
//! The standard is explicit about what that means: §19.3.1 says "if
//! any computed packet number is negative, an endpoint MUST generate a
//! connection error of type `FRAME_ENCODING_ERROR`". So the arithmetic
//! is not merely checked for our safety — **the check is the
//! protocol**, and a kernel that saturated instead of failing would
//! accept frames the standard says to reject.
//!
//! The walk is a cursor rather than a collection, so a frame with a
//! hostile range count costs the budget ([`crate::limits::ACK_RANGES`])
//! and not an allocation.

/// Why an ACK frame's ranges do not describe packet numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AckError {
    /// A computed packet number would be negative
    /// (RFC 9000 §19.3.1: `FRAME_ENCODING_ERROR`).
    Negative,
    /// The largest acknowledged number is above the 2^62 − 1 the
    /// packet-number space allows.
    OutOfRange(u64),
}

/// One acknowledged run of packet numbers, inclusive at both ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// The highest number in the run.
    pub largest: u64,
    /// The lowest number in the run.
    pub smallest: u64,
}

impl Range {
    /// How many packet numbers the run covers. At least 1 — a range
    /// is never empty, which is why this is not called `len`.
    #[must_use]
    pub const fn count(&self) -> u64 {
        // `smallest <= largest` is this type's invariant: every
        // constructor in this module checks it.
        self.largest.saturating_sub(self.smallest).saturating_add(1)
    }

    /// Whether the run covers `pn`.
    #[must_use]
    pub const fn contains(&self, pn: u64) -> bool {
        self.smallest <= pn && pn <= self.largest
    }
}

/// Where the walk down an ACK frame's ranges has got to.
///
/// Created by [`first`] and advanced by [`Cursor::next_range`]; it
/// holds only the smallest number seen so far, which is all the next
/// step needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    smallest: u64,
}

/// The first range: `largest` down to `largest - first_range`.
///
/// # Errors
/// [`AckError::OutOfRange`] when `largest` is above the packet-number
/// space, and [`AckError::Negative`] when `first_range` reaches below
/// zero.
pub const fn first(largest: u64, first_range: u64) -> Result<(Range, Cursor), AckError> {
    if largest > crate::packet_number::MAX {
        return Err(AckError::OutOfRange(largest));
    }
    let smallest = match largest.checked_sub(first_range) {
        Some(s) => s,
        None => return Err(AckError::Negative),
    };
    Ok((Range { largest, smallest }, Cursor { smallest }))
}

impl Cursor {
    /// The next range down: a `gap` of unacknowledged numbers, then a
    /// run of `len + 1`.
    ///
    /// §19.3.1 gives the arithmetic: the next largest is
    /// `previous_smallest - gap - 2`, because the gap field encodes
    /// one less than the number of missing packets and the previous
    /// smallest is itself acknowledged. Each subtraction is checked
    /// because each is the one the standard says may not go negative.
    ///
    /// # Errors
    /// [`AckError::Negative`] when any computed number would be below
    /// zero.
    pub const fn next_range(self, gap: u64, len: u64) -> Result<(Range, Self), AckError> {
        let above_gap = match self.smallest.checked_sub(gap) {
            Some(v) => v,
            None => return Err(AckError::Negative),
        };
        let largest = match above_gap.checked_sub(2) {
            Some(v) => v,
            None => return Err(AckError::Negative),
        };
        let smallest = match largest.checked_sub(len) {
            Some(v) => v,
            None => return Err(AckError::Negative),
        };
        Ok((Range { largest, smallest }, Self { smallest }))
    }

    /// The smallest number acknowledged so far.
    #[must_use]
    pub const fn smallest(&self) -> u64 {
        self.smallest
    }
}
