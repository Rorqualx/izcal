//! The wire format (Stage 1 design 4.1, VER-1; security scope IPC-R4,
//! IPC-R6; coding standard CS-ER-9).
//!
//! Fixed, little-endian, length-prefixed, with no self-describing
//! schema: both ends know the type from the endpoint, so the bytes carry
//! only values.
//!
//! | Type | Encoding |
//! |---|---|
//! | `bool` | one byte, 0 or 1; anything else is a bad message |
//! | `u8` … `u64`, `i32`, `i64` | little-endian, fixed width |
//! | `f32`, `f64` | the IEEE 754 bit pattern, little-endian (so NaN payloads survive the round trip) |
//! | `option<T>` | one byte, 0 (none) or 1 (some), then `T` |
//! | `bytes<N>`, `string<N>` | `u32` byte length ≤ N, then the bytes; a string must be UTF-8 |
//! | `vec<T, N>` | `u32` count ≤ N, then the elements |
//! | struct | its fields, in declaration order |
//! | enum, union | `u32` tag (declaration order from 0), then the variant's fields |
//! | message | `u16` ordinal (declaration order from 0), then its fields |
//!
//! A decoder accepts exactly the bytes its encoder produces: every
//! accepted message re-encodes to the identical bytes (IPC-R6), and
//! bytes left over after a message are a bad message. Before allocating
//! for a count, the decoder checks that the remaining input could hold
//! that many elements at their minimum size, so an allocation is never
//! larger than the input that asked for it (bounded allocation).
//!
//! # Under Kani: the writer's buffer
//!
//! Under `cfg(kani)`, [`Writer`] appends to a fixed buffer of
//! [`KANI_WRITER_CAPACITY`] bytes instead of a `Vec`, with the same API
//! and the same append semantics; `into_bytes` makes one `Vec` at the
//! end. The Kani proofs (VER-1) are about this crate's and the generated
//! encoders and decoders; they are proved against that buffer. What they
//! trust std for is `Vec::extend_from_slice` appending a slice to a
//! vector, which the production writer uses for every `put`. CBMC's cost
//! for that call's growth code was the measured bottleneck: on a fixed
//! 21-byte message with no strings, 426 k steps and out of memory at
//! 10 GiB with the `Vec`, 90 k steps and a pass with the buffer
//! (measurements/milestones/m1-kani). A put past the capacity fails an
//! assertion; it never truncates. The `the_kani_buffer_appends_as_a_vec`
//! test runs both on the same random puts in every build.

/// Why bytes were not a valid message. The receiver closes the endpoint
/// and asks the broker to terminate the sender (IPC-R5, CS-ER-6); none of
/// this crosses the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadMessage {
    /// The input ended inside a value.
    Truncated,
    /// Bytes were left over after the message.
    TrailingBytes,
    /// A `bool` or `option` byte was neither 0 nor 1.
    BadBool,
    /// An enum or union tag names no variant.
    UnknownTag,
    /// A message ordinal names no message of the interface.
    UnknownOrdinal,
    /// A length or count is over the type's declared maximum.
    OverBound,
    /// A string's bytes are not UTF-8.
    BadUtf8,
    /// Memory for a bounded value could not be reserved (CS-ER-9).
    OutOfMemory,
    /// The message's handles are not exactly the kinds it declares, in
    /// order (IPC-R1).
    BadHandles,
}

impl core::fmt::Display for BadMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Truncated => "the message ends inside a value",
            Self::TrailingBytes => "bytes follow the message",
            Self::BadBool => "a boolean or option byte is not 0 or 1",
            Self::UnknownTag => "an enum or union tag names no variant",
            Self::UnknownOrdinal => "the message ordinal names no message",
            Self::OverBound => "a length is over its declared maximum",
            Self::BadUtf8 => "a string is not UTF-8",
            Self::OutOfMemory => "memory for the message could not be reserved",
            Self::BadHandles => "the message's handles are not the declared kinds",
        })
    }
}

impl std::error::Error for BadMessage {}

/// The bytes a [`Writer`] holds under Kani. Every generated harness
/// checks at compile time that its largest message fits.
pub const KANI_WRITER_CAPACITY: usize = 64;

/// The Kani writer's buffer: a fixed array and a length (see the module
/// docs). Compiled for tests too, which compare it with a `Vec`.
#[cfg(any(kani, test))]
#[derive(Debug)]
struct FixedBuf {
    bytes: [u8; KANI_WRITER_CAPACITY],
    len: usize,
}

#[cfg(any(kani, test))]
impl FixedBuf {
    const fn new() -> Self {
        Self {
            bytes: [0; KANI_WRITER_CAPACITY],
            len: 0,
        }
    }

    /// Appends `src`. Past the capacity this fails an assertion: the
    /// harness's message did not fit, which its compile-time check rules
    /// out.
    fn put(&mut self, src: &[u8]) {
        let end = self.len.checked_add(src.len());
        let dst = end.and_then(|e| self.bytes.get_mut(self.len..e));
        assert!(dst.is_some(), "the Kani writer's buffer is full");
        if let (Some(dst), Some(e)) = (dst, end) {
            dst.copy_from_slice(src);
            self.len = e;
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bytes
            .get(..self.len)
            .map(<[u8]>::to_vec)
            .unwrap_or_default()
    }
}

/// Builds a message's bytes.
#[derive(Debug)]
pub struct Writer {
    #[cfg(not(kani))]
    buf: Vec<u8>,
    #[cfg(kani)]
    buf: FixedBuf,
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(kani))]
impl Writer {
    /// An empty writer.
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Appends raw bytes.
    pub fn put(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// The bytes written.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

/// Under Kani: the same API, on the fixed buffer (see the module docs).
#[cfg(kani)]
impl Writer {
    /// An empty writer.
    pub const fn new() -> Self {
        Self {
            buf: FixedBuf::new(),
        }
    }

    /// Appends raw bytes.
    pub fn put(&mut self, bytes: &[u8]) {
        self.buf.put(bytes);
    }

    /// The bytes written.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf.into_bytes()
    }
}

/// Reads a message's bytes, front to back.
#[derive(Debug)]
pub struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    /// A reader over `bytes`.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    /// Bytes not yet read.
    pub const fn remaining(&self) -> usize {
        self.rest.len()
    }

    /// Takes the next `n` bytes.
    ///
    /// # Errors
    /// [`BadMessage::Truncated`] if fewer than `n` remain.
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], BadMessage> {
        let (head, tail) = self.rest.split_at_checked(n).ok_or(BadMessage::Truncated)?;
        self.rest = tail;
        Ok(head)
    }

    /// Takes the next `N` bytes as an array.
    ///
    /// # Errors
    /// [`BadMessage::Truncated`] if fewer than `N` remain.
    pub fn take_array<const N: usize>(&mut self) -> Result<[u8; N], BadMessage> {
        let (head, tail) = self
            .rest
            .split_first_chunk::<N>()
            .ok_or(BadMessage::Truncated)?;
        self.rest = tail;
        Ok(*head)
    }

    /// Checks that the input is used up.
    ///
    /// # Errors
    /// [`BadMessage::TrailingBytes`] if bytes remain.
    pub const fn finish(self) -> Result<(), BadMessage> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(BadMessage::TrailingBytes)
        }
    }

    /// Reads a `u32` count and checks it against `max` and against the
    /// input left, at `min_len` bytes per element. The caller may then
    /// reserve `count` elements: they fit in the input.
    ///
    /// # Errors
    /// [`BadMessage::OverBound`] over `max`; [`BadMessage::Truncated`] if
    /// the input cannot hold that many elements.
    pub fn count(&mut self, max: usize, min_len: usize) -> Result<usize, BadMessage> {
        let n = usize::try_from(u32::decode(self)?).map_err(|_overflow| BadMessage::OverBound)?;
        if n > max {
            return Err(BadMessage::OverBound);
        }
        let need = n.checked_mul(min_len).ok_or(BadMessage::Truncated)?;
        if need > self.remaining() {
            return Err(BadMessage::Truncated);
        }
        Ok(n)
    }
}

/// A value with a wire encoding.
pub trait Wire: Sized {
    /// The fewest bytes any value encodes to.
    const MIN_LEN: usize;
    /// The most bytes any value encodes to (saturating at `usize::MAX`;
    /// `hg-ipc-codegen` rejects messages over the transport's cap).
    const MAX_LEN: usize;
    /// Appends the encoding.
    fn encode(&self, w: &mut Writer);
    /// Reads one value.
    ///
    /// # Errors
    /// [`BadMessage`] if the bytes are not a valid encoding.
    fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage>;
    /// Checks that the next bytes are one valid encoding and moves past
    /// them, allocating nothing. [`decode_exact`] runs this over the
    /// whole message before decoding, so every decision about untrusted
    /// bytes is made here first, in code with no heap, which Kani can
    /// prove for nested messages too (VER-1).
    ///
    /// # Errors
    /// [`BadMessage`] exactly when `decode` would fail on these bytes,
    /// other than for lack of memory.
    fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage>;
}

/// `a + b`, saturating, for compile-time size bounds only.
pub const fn len_add(a: usize, b: usize) -> usize {
    match a.checked_add(b) {
        Some(n) => n,
        // A bound past usize::MAX is "too large"; the generator rejects it.
        None => usize::MAX,
    }
}

/// `a * b`, saturating, for compile-time size bounds only.
pub const fn len_mul(a: usize, b: usize) -> usize {
    match a.checked_mul(b) {
        Some(n) => n,
        None => usize::MAX,
    }
}

/// The larger of `a` and `b`, for compile-time size bounds.
pub const fn len_max(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}

/// The smaller of `a` and `b`, for compile-time size bounds.
pub const fn len_min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

macro_rules! fixed {
    ($($t:ty),*) => {$(
        impl Wire for $t {
            const MIN_LEN: usize = core::mem::size_of::<$t>();
            const MAX_LEN: usize = core::mem::size_of::<$t>();
            fn encode(&self, w: &mut Writer) {
                w.put(&self.to_le_bytes());
            }
            fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {
                Ok(<$t>::from_le_bytes(r.take_array()?))
            }
            fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {
                r.take(core::mem::size_of::<$t>()).map(|_bytes| ())
            }
        }
    )*};
}

fixed!(u8, u16, u32, u64, i32, i64);

impl Wire for f32 {
    const MIN_LEN: usize = 4;
    const MAX_LEN: usize = 4;
    fn encode(&self, w: &mut Writer) {
        self.to_bits().encode(w);
    }
    fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {
        Ok(Self::from_bits(u32::decode(r)?))
    }
    fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {
        u32::validate(r)
    }
}

impl Wire for f64 {
    const MIN_LEN: usize = 8;
    const MAX_LEN: usize = 8;
    fn encode(&self, w: &mut Writer) {
        self.to_bits().encode(w);
    }
    fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {
        Ok(Self::from_bits(u64::decode(r)?))
    }
    fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {
        u64::validate(r)
    }
}

impl Wire for bool {
    const MIN_LEN: usize = 1;
    const MAX_LEN: usize = 1;
    fn encode(&self, w: &mut Writer) {
        u8::from(*self).encode(w);
    }
    fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {
        match u8::decode(r)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(BadMessage::BadBool),
        }
    }
    fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {
        Self::decode(r).map(|_value| ())
    }
}

impl<T: Wire> Wire for Option<T> {
    const MIN_LEN: usize = 1;
    const MAX_LEN: usize = len_add(1, T::MAX_LEN);
    fn encode(&self, w: &mut Writer) {
        match self {
            None => false.encode(w),
            Some(v) => {
                true.encode(w);
                v.encode(w);
            }
        }
    }
    fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {
        if bool::decode(r)? {
            Ok(Some(T::decode(r)?))
        } else {
            Ok(None)
        }
    }
    fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {
        if bool::decode(r)? {
            T::validate(r)
        } else {
            Ok(())
        }
    }
}

/// Encodes `value` as one whole message.
pub fn encode_to_vec<T: Wire>(value: &T) -> Vec<u8> {
    let mut w = Writer::new();
    value.encode(&mut w);
    w.into_bytes()
}

/// Decodes exactly one message from `bytes`.
///
/// # Errors
/// [`BadMessage`] if `bytes` are not exactly one valid encoding.
pub fn decode_exact<T: Wire>(bytes: &[u8]) -> Result<T, BadMessage> {
    validate_exact::<T>(bytes)?;
    let mut r = Reader::new(bytes);
    let v = T::decode(&mut r)?;
    r.finish()?;
    Ok(v)
}

/// Checks that `bytes` are exactly one valid encoding, allocating nothing.
///
/// # Errors
/// [`BadMessage`] if they are not.
pub fn validate_exact<T: Wire>(bytes: &[u8]) -> Result<(), BadMessage> {
    let mut r = Reader::new(bytes);
    T::validate(&mut r)?;
    r.finish()
}

/// Writes a length or count. Bounded types keep it at most their `N`,
/// which the generator keeps within `u32`.
pub fn put_count(w: &mut Writer, n: usize) {
    // A count over u32 cannot come from a bounded type (N ≤ u32::MAX is
    // checked by the generator); saturating keeps this total anyway.
    u32::try_from(n).unwrap_or(u32::MAX).encode(w);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Kani writer's buffer appends exactly as a `Vec` does: random
    /// puts, empty ones included, up to the capacity, give the same bytes.
    #[test]
    fn the_kani_buffer_appends_as_a_vec() {
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = |n: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state.checked_rem(n).unwrap_or(0)
        };
        for _ in 0..20_000 {
            let mut model = FixedBuf::new();
            let mut real = Vec::new();
            loop {
                let room = KANI_WRITER_CAPACITY.saturating_sub(real.len());
                let room = u64::try_from(room).unwrap_or(0);
                // Empty puts about one time in eight; sometimes all the room.
                let n = match next(8) {
                    0 => 0,
                    1 => room,
                    _ => next(room.min(9).saturating_add(1)),
                };
                let n = usize::try_from(n).unwrap_or(0);
                let src: Vec<u8> = (0..n)
                    .map(|_| u8::try_from(next(256)).unwrap_or(0))
                    .collect();
                model.put(&src);
                real.extend_from_slice(&src);
                if real.len() == KANI_WRITER_CAPACITY || next(16) == 0 {
                    break;
                }
            }
            assert_eq!(model.into_bytes(), real);
        }
    }

    #[test]
    #[should_panic(expected = "the Kani writer's buffer is full")]
    fn the_kani_buffer_refuses_a_put_past_its_capacity() {
        let mut model = FixedBuf::new();
        model.put(&[0; KANI_WRITER_CAPACITY]);
        model.put(&[1]);
    }

    #[test]
    fn primitives_round_trip_and_reject_bad_bytes() {
        let v = (7u16, -3i64, f64::from_bits(0x7ff8_0000_0000_0001), true);
        let mut w = Writer::new();
        v.0.encode(&mut w);
        v.1.encode(&mut w);
        v.2.encode(&mut w);
        v.3.encode(&mut w);
        Some(9u8).encode(&mut w);
        let bytes = w.into_bytes();
        assert_eq!(bytes.len(), 2 + 8 + 8 + 1 + 2);
        let mut r = Reader::new(&bytes);
        assert_eq!(u16::decode(&mut r), Ok(7));
        assert_eq!(i64::decode(&mut r), Ok(-3));
        assert_eq!(
            f64::decode(&mut r).map(f64::to_bits),
            Ok(0x7ff8_0000_0000_0001)
        );
        assert_eq!(bool::decode(&mut r), Ok(true));
        assert_eq!(Option::<u8>::decode(&mut r), Ok(Some(9)));
        assert_eq!(r.finish(), Ok(()));

        assert_eq!(decode_exact::<bool>(&[2]), Err(BadMessage::BadBool));
        assert_eq!(decode_exact::<u32>(&[1, 2, 3]), Err(BadMessage::Truncated));
        assert_eq!(decode_exact::<u8>(&[1, 2]), Err(BadMessage::TrailingBytes));
    }

    #[test]
    fn counts_are_checked_against_the_bound_and_the_input() {
        let mut w = Writer::new();
        put_count(&mut w, 3);
        w.put(&[0; 12]);
        let bytes = w.into_bytes();
        assert_eq!(Reader::new(&bytes).count(3, 4), Ok(3));
        assert_eq!(Reader::new(&bytes).count(2, 4), Err(BadMessage::OverBound));
        // Three elements of 8 bytes do not fit in 12.
        assert_eq!(Reader::new(&bytes).count(3, 8), Err(BadMessage::Truncated));
    }
}
