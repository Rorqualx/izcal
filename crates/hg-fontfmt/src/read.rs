//! Reading big-endian scalars out of a byte slice.
//!
//! The only hand-written code in this crate. Every reader is total: it
//! returns `None` when the bytes are not there, and never indexes,
//! slices unchecked, casts with `as` or panics. Byte-order reads need no
//! `unsafe` (fonts and text 6.2).

/// `uint8` at `off`.
#[must_use]
pub fn u8_at(data: &[u8], off: usize) -> Option<u8> {
    data.get(off).copied()
}

/// `int8` at `off`.
#[must_use]
pub fn i8_at(data: &[u8], off: usize) -> Option<i8> {
    u8_at(data, off).map(|b| i8::from_be_bytes([b]))
}

/// The `N` bytes at `off`, or `None`.
fn bytes<const N: usize>(data: &[u8], off: usize) -> Option<[u8; N]> {
    let end = off.checked_add(N)?;
    data.get(off..end)?.try_into().ok()
}

/// `uint16` at `off`.
#[must_use]
pub fn u16_at(data: &[u8], off: usize) -> Option<u16> {
    bytes::<2>(data, off).map(u16::from_be_bytes)
}

/// `int16` (also `FWORD`) at `off`.
#[must_use]
pub fn i16_at(data: &[u8], off: usize) -> Option<i16> {
    bytes::<2>(data, off).map(i16::from_be_bytes)
}

/// `uint24` at `off`, widened to `u32`.
#[must_use]
pub fn u24_at(data: &[u8], off: usize) -> Option<u32> {
    let [a, b, c] = bytes::<3>(data, off)?;
    Some(u32::from_be_bytes([0, a, b, c]))
}

/// `uint32` at `off`.
#[must_use]
pub fn u32_at(data: &[u8], off: usize) -> Option<u32> {
    bytes::<4>(data, off).map(u32::from_be_bytes)
}

/// `int32` at `off`.
#[must_use]
pub fn i32_at(data: &[u8], off: usize) -> Option<i32> {
    bytes::<4>(data, off).map(i32::from_be_bytes)
}

/// `LONGDATETIME` at `off`.
#[must_use]
pub fn i64_at(data: &[u8], off: usize) -> Option<i64> {
    bytes::<8>(data, off).map(i64::from_be_bytes)
}

/// A `Tag` at `off`: four bytes, unexamined.
#[must_use]
pub fn tag_at(data: &[u8], off: usize) -> Option<[u8; 4]> {
    bytes::<4>(data, off)
}
