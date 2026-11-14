//! Appending big-endian scalars to a buffer, and what can go wrong.
//!
//! The only hand-written code in this crate, and the mirror of
//! `hg_fontfmt::read`. Every appender is infallible except [`u24`],
//! which has a value the type cannot hold; none of them indexes, casts
//! with `as` or panics.

use core::fmt;

/// Why a table could not be written.
///
/// Every variant names the table and the field, because a compiler that
/// says only "bad length" sends its author back through the whole
/// description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A field was never set, so there is no value to write.
    Unset {
        /// The table being written.
        table: &'static str,
        /// The field that was never set.
        field: &'static str,
    },
    /// An array holds a different number of elements than the table's
    /// own count fields declare.
    ///
    /// The writer refuses rather than padding or truncating: both would
    /// produce a table a conforming reader declines, and the author
    /// would learn of it from a font that does not load rather than
    /// from the compiler.
    Count {
        /// The table being written.
        table: &'static str,
        /// The array field.
        field: &'static str,
        /// What the table's own fields declare, read the way the
        /// generated reader reads them.
        declared: usize,
        /// How many elements were pushed.
        given: usize,
    },
    /// A count expression overflowed, so the table has no length.
    Overflow {
        /// The table being written.
        table: &'static str,
        /// The array field whose count could not be computed.
        field: &'static str,
    },
    /// A value does not fit the field's width.
    Range {
        /// The table being written.
        table: &'static str,
        /// The field.
        field: &'static str,
        /// The value that does not fit.
        value: u32,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Unset { table, field } => {
                write!(f, "{table}.{field} was never set")
            }
            Self::Count {
                table,
                field,
                declared,
                given,
            } => write!(
                f,
                "{table}.{field}: the table declares {declared} and {given} were given"
            ),
            Self::Overflow { table, field } => {
                write!(f, "{table}.{field}: the count overflowed")
            }
            Self::Range {
                table,
                field,
                value,
            } => write!(f, "{table}.{field}: {value} does not fit the field"),
        }
    }
}

impl std::error::Error for Error {}

/// Appends a `uint8`.
pub fn u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

/// Appends an `int8`.
pub fn i8(out: &mut Vec<u8>, v: i8) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Appends a `uint16`.
pub fn u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Appends an `int16` (also `FWORD`).
pub fn i16(out: &mut Vec<u8>, v: i16) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Appends a `uint24`.
///
/// # Errors
/// [`Error::Range`] when `v` is above 0xFF_FFFF, which the three bytes
/// cannot hold. Truncating it silently would write a different number
/// than the author asked for.
pub fn u24(
    out: &mut Vec<u8>,
    table: &'static str,
    field: &'static str,
    v: u32,
) -> Result<(), Error> {
    let [top, a, b, c] = v.to_be_bytes();
    if top != 0 {
        return Err(Error::Range {
            table,
            field,
            value: v,
        });
    }
    out.extend_from_slice(&[a, b, c]);
    Ok(())
}

/// Appends a `uint32`.
pub fn u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Appends an `int32`.
pub fn i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Appends a `LONGDATETIME`.
pub fn i64(out: &mut Vec<u8>, v: i64) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Appends a `Tag`: four bytes, unexamined.
pub fn tag(out: &mut Vec<u8>, v: [u8; 4]) {
    out.extend_from_slice(&v);
}
