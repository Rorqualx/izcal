//! QUIC variable-length integers (RFC 9000 §16).
//!
//! The first two bits of the first byte give the length — 1, 2, 4 or 8
//! bytes — and the remaining 62 bits carry the value. Every length and
//! every byte string decodes, so the only failures are "the bytes are
//! not there" and "the value does not fit 62 bits".
//!
//! # Minimal and non-minimal encodings
//!
//! RFC 9000 §16 does not require the minimum number of bytes, so a
//! decoder that refused a long encoding would reject conforming
//! peers. But §12.4 lets an endpoint treat a frame *type* encoded
//! longer than necessary as a `PROTOCOL_VIOLATION`, and the transport
//! parameters have their own rule. The two needs are different, so
//! [`decode`] accepts what the standard accepts and [`is_minimal`]
//! answers the separate question, at the one call site that has it.
//!
//! Nothing here reads beyond what the length prefix claims, and
//! nothing indexes: the kernel is small enough that Kani proves it
//! total for every input (G33).

/// The largest value a variable-length integer can carry: 2^62 − 1.
pub const MAX: u64 = (1_u64 << 62) - 1;

/// How many bytes the minimal encoding of `v` takes, or `None` when
/// `v` is above [`MAX`].
#[must_use]
pub const fn encoded_len(v: u64) -> Option<usize> {
    if v <= 63 {
        Some(1)
    } else if v <= 16_383 {
        Some(2)
    } else if v <= 1_073_741_823 {
        Some(4)
    } else if v <= MAX {
        Some(8)
    } else {
        None
    }
}

/// Whether `len` bytes is the minimal encoding of `v`.
///
/// `false` for a `len` that is not 1, 2, 4 or 8, and for a `v` above
/// [`MAX`], neither of which any encoding of `v` could be.
#[must_use]
pub const fn is_minimal(v: u64, len: usize) -> bool {
    match encoded_len(v) {
        Some(n) => n == len,
        None => false,
    }
}

/// Writes the minimal encoding of `v` at the start of `out`.
///
/// Returns how many bytes were written, or `None` when `v` is above
/// [`MAX`] or `out` is too short. Nothing is written when it returns
/// `None`, so a short buffer leaves the caller's bytes alone.
///
/// One arm per length, each a fixed-size write. The first version
/// computed a slice of the length and copied into it, which is the
/// same bytes and reads more neatly — and left the copy's size
/// symbolic, so Kani could not fold it and the eight-byte round-trip
/// proof did not finish in half an hour. Four constant-size writes are
/// the same work at run time, a little clearer to read against the
/// standard's table, and they let the proof close in seconds.
#[must_use]
pub fn encode(v: u64, out: &mut [u8]) -> Option<usize> {
    let len = encoded_len(v)?;
    // Destructured rather than indexed: the bytes have names, and tier
    // P does not have to be told that `be[7]` is in bounds.
    let [b0, b1, b2, b3, b4, b5, b6, b7] = v.to_be_bytes();
    match len {
        1 => *out.first_chunk_mut::<1>()? = [b7],
        2 => *out.first_chunk_mut::<2>()? = [b6 | 0b0100_0000, b7],
        4 => *out.first_chunk_mut::<4>()? = [b4 | 0b1000_0000, b5, b6, b7],
        _ => {
            *out.first_chunk_mut::<8>()? = [b0 | 0b1100_0000, b1, b2, b3, b4, b5, b6, b7];
        }
    }
    Some(len)
}

/// Reads a variable-length integer from the start of `input`.
///
/// Returns the value and how many bytes it took, or `None` when
/// `input` is empty or holds fewer bytes than the length prefix
/// claims.
///
/// One arm per length, as [`encode`] has, and for the same reason: an
/// accumulating loop over a symbolic length is seven multiply-adds the
/// solver has to invert, and four fixed-width reads are none.
#[must_use]
pub fn decode(input: &[u8]) -> Option<(u64, usize)> {
    const VALUE: u8 = 0b0011_1111;
    let first = *input.first()?;
    match first >> 6 {
        0 => Some((u64::from(first & VALUE), 1)),
        1 => {
            let [a, b] = *input.first_chunk::<2>()?;
            Some((u64::from(u16::from_be_bytes([a & VALUE, b])), 2))
        }
        2 => {
            let [a, b, c, d] = *input.first_chunk::<4>()?;
            Some((u64::from(u32::from_be_bytes([a & VALUE, b, c, d])), 4))
        }
        _ => {
            let [a, b, c, d, e, f, g, h] = *input.first_chunk::<8>()?;
            Some((u64::from_be_bytes([a & VALUE, b, c, d, e, f, g, h]), 8))
        }
    }
}
