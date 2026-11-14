//! QUIC packet numbers: how many bytes to send, and what a truncated
//! one means (RFC 9000 §17.1, Appendix A.2 and A.3).
//!
//! A packet carries the low 8, 16, 24 or 32 bits of its number, and
//! the receiver reconstructs the rest from the largest number it has
//! already processed. The reconstruction is the interesting part: the
//! standard's pseudocode subtracts half a window from an expected
//! value and adds a whole window to a candidate, and **both steps go
//! out of range at the edges of the 62-bit space** — at the start of a
//! connection, where `expected` is smaller than half a window, and at
//! the top, where a candidate plus a window would not fit.
//!
//! The standard says so: Appendix A.3's conditions carry the guards
//! `candidate_pn < (1 << 62) - pn_win` and `candidate_pn >= pn_win`
//! for exactly this reason. An implementation that drops them has a
//! packet number that wraps, which is an attacker-chosen decryption
//! nonce. So the guards are written here as checked arithmetic, and
//! Kani proves the whole function total for any `largest`, any
//! truncated value and any of the four lengths (G33).

/// Why a packet number could not be handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PacketNumberError {
    /// The encoded length is not 1, 2, 3 or 4 bytes.
    BadLength(usize),
    /// The number is above the 2^62 − 1 the space allows
    /// (RFC 9000 §17.1: a sender must not exceed it).
    OutOfRange(u64),
    /// The truncated number carries bits the encoded length does not,
    /// so it is not something that came off the wire in `len` bytes.
    ///
    /// Found by the Kani harness `packet_number_decode_keeps_the_wire_bits`
    /// rather than by a test: the first version took a `u32` and OR-ed
    /// all 32 bits into the candidate whatever the length, so a
    /// one-byte packet number given 0x1234_5678 produced a number that
    /// did not have 0x78 in its low byte. A real parser reading one
    /// byte never does that, which is exactly why no test would have
    /// caught it and why this is refused rather than masked.
    TruncatedTooLarge {
        /// What was given.
        value: u32,
        /// The encoded length, in bytes.
        len: usize,
    },
}

/// One past the largest packet number: 2^62.
const SPACE: u64 = 1_u64 << 62;

/// The largest packet number, 2^62 − 1 (RFC 9000 §17.1).
pub const MAX: u64 = SPACE - 1;

/// How many bytes `full` needs, given the largest number the peer has
/// acknowledged (`None` before anything is acknowledged).
///
/// Appendix A.2: the encoding must be unambiguous over twice the
/// number of packets in flight, so the window is `2 × range`, and the
/// length is the smallest that spans it.
///
/// # Errors
/// [`PacketNumberError::OutOfRange`] when `full` is above [`MAX`].
pub const fn encoded_len(
    full: u64,
    largest_acked: Option<u64>,
) -> Result<usize, PacketNumberError> {
    if full > MAX {
        return Err(PacketNumberError::OutOfRange(full));
    }
    // A.2's range: the packets that could still be in flight. When
    // `largest_acked` is ahead of `full` the peer has acknowledged a
    // number we have not sent; one byte is then as unambiguous as any,
    // and the caller's own check — not this one — is where that is a
    // protocol error.
    let range = match largest_acked {
        None => full.saturating_add(1),
        Some(l) => full.saturating_sub(l),
    };
    // Twice the range, saturating: a range that large already needs
    // the longest encoding.
    let window = range.saturating_mul(2);
    if window <= 0x100 {
        Ok(1)
    } else if window <= 0x1_0000 {
        Ok(2)
    } else if window <= 0x100_0000 {
        Ok(3)
    } else {
        Ok(4)
    }
}

/// The low `len` bytes of `full`, as they go on the wire.
///
/// # Errors
/// [`PacketNumberError::BadLength`] for a length outside 1..=4, and
/// [`PacketNumberError::OutOfRange`] for a number above [`MAX`].
pub fn truncate(full: u64, len: usize) -> Result<u32, PacketNumberError> {
    if full > MAX {
        return Err(PacketNumberError::OutOfRange(full));
    }
    let low = full & mask_of(len)?;
    match u32::try_from(low) {
        Ok(v) => Ok(v),
        // Unreachable: every mask is at most 0xFFFF_FFFF, so `low`
        // fits. Reported rather than asserted, because tier P has no
        // panic and an unreachable branch that panics is still a panic.
        Err(_) => Err(PacketNumberError::OutOfRange(full)),
    }
}

/// The low-bit mask for an encoded length. Written out rather than
/// shifted, because a shift is arithmetic the tier's lints will not
/// take on trust and these four values are the whole of it.
const fn mask_of(len: usize) -> Result<u64, PacketNumberError> {
    match len {
        1 => Ok(0xFF),
        2 => Ok(0xFFFF),
        3 => Ok(0x00FF_FFFF),
        4 => Ok(0xFFFF_FFFF),
        _ => Err(PacketNumberError::BadLength(len)),
    }
}

/// The window for an encoded length: one more than its mask.
const fn window_of(len: usize) -> Result<u64, PacketNumberError> {
    match len {
        1 => Ok(0x100),
        2 => Ok(0x1_0000),
        3 => Ok(0x0100_0000),
        4 => Ok(0x1_0000_0000),
        _ => Err(PacketNumberError::BadLength(len)),
    }
}

/// Reconstructs a full packet number from the `len` bytes on the wire
/// and the largest number already processed in that space.
///
/// This is Appendix A.3, with its two range guards written as checked
/// arithmetic. `largest` is `None` before any packet has been
/// processed, where the standard's `expected_pn = largest_pn + 1`
/// starts at 0.
///
/// # Errors
/// [`PacketNumberError::BadLength`] for a length outside 1..=4, and
/// [`PacketNumberError::OutOfRange`] when `largest` is itself above
/// [`MAX`] — which no packet we processed could have been, so it is a
/// bug in the caller rather than a peer's doing. Refusing it is what
/// makes "the result is a valid packet number" unconditional.
pub fn decode(truncated: u32, len: usize, largest: Option<u64>) -> Result<u64, PacketNumberError> {
    if let Some(l) = largest
        && l > MAX
    {
        return Err(PacketNumberError::OutOfRange(l));
    }
    let mask = mask_of(len)?;
    let window = window_of(len)?;
    if u64::from(truncated) > mask {
        return Err(PacketNumberError::TruncatedTooLarge {
            value: truncated,
            len,
        });
    }
    // `window` is at most 2^32, so half of it is exact and the
    // division has a non-zero literal divisor.
    let half: u64 = window / 2;

    // A.3's `expected_pn`. Saturating at MAX rather than wrapping: a
    // `largest` of MAX means the space is exhausted, which RFC 9000
    // §17.1 makes a connection error at the sender, and reconstructing
    // into a wrapped number would be worse than reconstructing into
    // the same one.
    let expected: u64 = match largest {
        None => 0,
        // Held at MAX rather than allowed past it. `largest == MAX`
        // means the space is exhausted, which RFC 9000 §12.3 makes the
        // sender's error to avoid; letting `expected` leave the space
        // here would hand back a "packet number" that is not one.
        Some(l) if l >= MAX => MAX,
        Some(l) => l.saturating_add(1),
    };

    // `expected & !mask` keeps the high bits; `| truncated` puts the
    // wire's low bits back.
    let candidate: u64 = (expected & !mask) | u64::from(truncated);

    // Below the window: the number wrapped upward since `expected`.
    // A.3's own guards, written as checked arithmetic. `expected -
    // half` underflows at the start of a connection and `SPACE -
    // window` is what stops the addition leaving the 62-bit space;
    // both are the standard's conditions, not extra caution.
    let floor = expected.checked_sub(half);
    let ceiling = SPACE.checked_sub(window);
    if let (Some(floor), Some(ceiling)) = (floor, ceiling)
        && candidate <= floor
        && candidate < ceiling
        && let Some(up) = candidate.checked_add(window)
    {
        return Ok(up);
    }
    // Above the window: the number is older than `expected`.
    if candidate > expected.saturating_add(half)
        && let Some(down) = candidate.checked_sub(window)
    {
        return Ok(down);
    }
    Ok(candidate)
}
