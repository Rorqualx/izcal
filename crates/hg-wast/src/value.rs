//! The values a script passes and the patterns it expects back.
//!
//! An action carries constants and an `assert_return` carries patterns.
//! Both are written in the text format's number grammar, which is not
//! the one a host language parses for you: digits may be separated by
//! `_`, floats come in a decimal and a hexadecimal form, and `inf`,
//! `nan` and `nan:0x…` are literals. A result may also be a *pattern*
//! rather than a value — `nan:canonical`, `nan:arithmetic`, a reference
//! of some kind with no payload, or `either` over alternatives — because
//! the specification leaves those results underdetermined.
//!
//! # Exactness
//!
//! A conformance runner that decodes `0x1.fffffffffffffp+1023` to the
//! wrong double does not fail: it passes the wrong test, and reports a
//! conforming engine as broken or a broken one as conforming. So the
//! decimal forms go through the host's correctly rounded parser, and the
//! hexadecimal forms are decoded here, by assembling the significand and
//! rounding once, to nearest, ties to even — the same rounding the
//! format specifies — with a sticky bit so that digits dropped from a
//! long mantissa still decide a tie.

use crate::lex::{Error, Span};

/// The four number types a constant can have.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumType {
    /// `i32`
    I32,
    /// `i64`
    I64,
    /// `f32`
    F32,
    /// `f64`
    F64,
}

impl NumType {
    /// The type named by `name`, if it names one.
    #[must_use]
    pub fn from_name(name: &str) -> Option<NumType> {
        match name {
            "i32" => Some(NumType::I32),
            "i64" => Some(NumType::I64),
            "f32" => Some(NumType::F32),
            "f64" => Some(NumType::F64),
            _ => None,
        }
    }

    /// Its name, as written.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            NumType::I32 => "i32",
            NumType::I64 => "i64",
            NumType::F32 => "f32",
            NumType::F64 => "f64",
        }
    }

    /// True for `f32` and `f64`.
    #[must_use]
    pub fn is_float(self) -> bool {
        matches!(self, NumType::F32 | NumType::F64)
    }
}

/// A `v128`'s lane shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// `i8x16`
    I8x16,
    /// `i16x8`
    I16x8,
    /// `i32x4`
    I32x4,
    /// `i64x2`
    I64x2,
    /// `f32x4`
    F32x4,
    /// `f64x2`
    F64x2,
}

impl Shape {
    /// The shape named by `name`, if it names one.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Shape> {
        match name {
            "i8x16" => Some(Shape::I8x16),
            "i16x8" => Some(Shape::I16x8),
            "i32x4" => Some(Shape::I32x4),
            "i64x2" => Some(Shape::I64x2),
            "f32x4" => Some(Shape::F32x4),
            "f64x2" => Some(Shape::F64x2),
            _ => None,
        }
    }

    /// Its name, as written.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Shape::I8x16 => "i8x16",
            Shape::I16x8 => "i16x8",
            Shape::I32x4 => "i32x4",
            Shape::I64x2 => "i64x2",
            Shape::F32x4 => "f32x4",
            Shape::F64x2 => "f64x2",
        }
    }

    /// How many lanes the shape has.
    #[must_use]
    pub fn lanes(self) -> usize {
        match self {
            Shape::I8x16 => 16,
            Shape::I16x8 => 8,
            Shape::I32x4 | Shape::F32x4 => 4,
            Shape::I64x2 | Shape::F64x2 => 2,
        }
    }

    /// How wide one lane is, in bits.
    #[must_use]
    pub fn lane_bits(self) -> u32 {
        match self {
            Shape::I8x16 => 8,
            Shape::I16x8 => 16,
            Shape::I32x4 | Shape::F32x4 => 32,
            Shape::I64x2 | Shape::F64x2 => 64,
        }
    }

    /// True for `f32x4` and `f64x2`.
    #[must_use]
    pub fn is_float(self) -> bool {
        matches!(self, Shape::F32x4 | Shape::F64x2)
    }
}

/// One value a script passes to an engine, or expects back exactly.
///
/// Numbers are kept as their bit patterns, not as `f32`/`f64`: a script
/// distinguishes NaNs by payload, and a host float would quietly lose
/// that. The payload is the point of half the float tests here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// `(i32.const n)`
    I32(u32),
    /// `(i64.const n)`
    I64(u64),
    /// `(f32.const x)`, as bits.
    F32(u32),
    /// `(f64.const x)`, as bits.
    F64(u64),
    /// `(v128.const …)`, as sixteen bytes in little-endian lane order.
    V128([u8; 16]),
    /// `(ref.null <heap type>)`, the type as written.
    RefNull(String),
    /// `(ref.extern n)`
    RefExtern(u64),
    /// `(ref.host n)`
    RefHost(u64),
}

/// What one result of an `assert_return` must be.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pattern {
    /// Exactly this value.
    Exact(Value),
    /// `nan:canonical` of this type.
    NanCanonical(NumType),
    /// `nan:arithmetic` of this type.
    NanArithmetic(NumType),
    /// A `v128` whose lanes are matched one at a time, so that a lane
    /// may itself be a NaN pattern.
    V128(Shape, Vec<Pattern>),
    /// A reference of the named kind, with no further constraint:
    /// `(ref.extern)`, `(ref.func)`, `(ref.null)`, `(ref.array)`.
    RefOfKind(String),
    /// `(either p …)`: any one of the alternatives.
    Either(Vec<Pattern>),
}

/// The bytes a string literal stands for.
///
/// `span` covers the literal with its quotation marks. Escapes are
/// decoded, `\u{…}` to the character's UTF-8. The result is bytes and
/// not text on purpose: a `(module binary "\00asm…")` is not UTF-8, and
/// the whole point of that form is to hand an engine bytes that may be
/// malformed.
pub fn string_bytes(src: &str, span: Span) -> Result<Vec<u8>, Error> {
    let text = span.text(src);
    let inner = text
        .strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .ok_or_else(|| Error {
            offset: span.start,
            message: "not a string literal".to_string(),
        })?;
    let base = span.start + 1;
    let b = inner.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'\\' {
            out.push(b[i]);
            i += 1;
            continue;
        }
        let at = base + i;
        i += 1;
        let Some(&e) = b.get(i) else {
            return Err(Error {
                offset: at,
                message: "a string ends in a backslash".to_string(),
            });
        };
        match e {
            b't' => {
                out.push(b'\t');
                i += 1;
            }
            b'n' => {
                out.push(b'\n');
                i += 1;
            }
            b'r' => {
                out.push(b'\r');
                i += 1;
            }
            b'"' => {
                out.push(b'"');
                i += 1;
            }
            b'\'' => {
                out.push(b'\'');
                i += 1;
            }
            b'\\' => {
                out.push(b'\\');
                i += 1;
            }
            b'u' => {
                let close = b[i..]
                    .iter()
                    .position(|&c| c == b'}')
                    .ok_or_else(|| Error {
                        offset: at,
                        message: "\\u{ is never closed".to_string(),
                    })?;
                let digits = std::str::from_utf8(&b[i + 2..i + close]).map_err(|_| Error {
                    offset: at,
                    message: "\\u{…} holds something that is not a hexadecimal number".to_string(),
                })?;
                let n = u32::from_str_radix(&digits.replace('_', ""), 16).map_err(|_| Error {
                    offset: at,
                    message: format!("\\u{{{digits}}} is not a hexadecimal number"),
                })?;
                let c = char::from_u32(n).ok_or_else(|| Error {
                    offset: at,
                    message: format!("U+{n:X} is not a character"),
                })?;
                let mut buf = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                i += close + 1;
            }
            _ => {
                let hi = hex_val(e).ok_or_else(|| Error {
                    offset: at,
                    message: "not an escape".to_string(),
                })?;
                let lo = b
                    .get(i + 1)
                    .copied()
                    .and_then(hex_val)
                    .ok_or_else(|| Error {
                        offset: at,
                        message: "an escaped byte needs two hexadecimal digits".to_string(),
                    })?;
                out.push(hi * 16 + lo);
                i += 2;
            }
        }
    }
    Ok(out)
}

/// The text a string literal stands for, for a name that must be text.
pub fn string_text(src: &str, span: Span) -> Result<String, Error> {
    let bytes = string_bytes(src, span)?;
    String::from_utf8(bytes).map_err(|_| Error {
        offset: span.start,
        message: "this name is not valid UTF-8".to_string(),
    })
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// An integer literal's bit pattern, in `bits` bits.
///
/// The format admits both the signed and the unsigned range for the same
/// type (`i32.const -1` and `i32.const 0xffffffff` are the same value),
/// so the check is against whichever the sign selects, and the result is
/// two's complement either way.
pub fn int(text: &str, bits: u32) -> Result<u64, String> {
    let (neg, rest) = match text.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let magnitude = unsigned(rest)?;
    let span = if bits == 64 {
        u128::from(u64::MAX) + 1
    } else {
        1u128 << bits
    };
    if neg {
        if magnitude > span / 2 {
            return Err(format!("{text} is too small for {bits} bits"));
        }
        Ok(((span - magnitude) % span) as u64)
    } else {
        if magnitude >= span {
            return Err(format!("{text} is too large for {bits} bits"));
        }
        Ok(magnitude as u64)
    }
}

/// An unsigned literal's value: decimal, or hexadecimal after `0x`,
/// with `_` allowed between digits.
fn unsigned(text: &str) -> Result<u128, String> {
    let (radix, digits) = match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        Some(d) => (16u128, d),
        None => (10u128, text),
    };
    let mut seen = false;
    let mut v: u128 = 0;
    let mut last_was_digit = false;
    for c in digits.chars() {
        if c == '_' {
            if !last_was_digit {
                return Err(format!("{text}: `_` must come between digits"));
            }
            last_was_digit = false;
            continue;
        }
        let d = c
            .to_digit(radix as u32)
            .ok_or_else(|| format!("{text} is not a number"))?;
        v = v
            .checked_mul(radix)
            .and_then(|v| v.checked_add(u128::from(d)))
            .ok_or_else(|| format!("{text} does not fit in 128 bits"))?;
        seen = true;
        last_was_digit = true;
    }
    if !seen || !last_was_digit {
        return Err(format!("{text} is not a number"));
    }
    Ok(v)
}

/// An `f32` literal's bits.
pub fn float32(text: &str) -> Result<u32, String> {
    float(text, 23, 8).map(|b| b as u32)
}

/// An `f64` literal's bits.
pub fn float64(text: &str) -> Result<u64, String> {
    float(text, 52, 11)
}

/// A float literal's bits, for a format with `mbits` stored mantissa
/// bits and `ebits` exponent bits.
fn float(text: &str, mbits: u32, ebits: u32) -> Result<u64, String> {
    let (neg, rest) = match text.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let sign = u64::from(neg) << (mbits + ebits);
    let exp_all = (1u64 << ebits) - 1;

    if rest == "inf" {
        return Ok(sign | (exp_all << mbits));
    }
    if rest == "nan" {
        // The canonical NaN: the payload's most significant bit alone.
        return Ok(sign | (exp_all << mbits) | (1u64 << (mbits - 1)));
    }
    if let Some(p) = rest.strip_prefix("nan:") {
        let payload = unsigned(p).map_err(|e| format!("{text}: {e}"))?;
        if payload == 0 || payload >= (1u128 << mbits) {
            return Err(format!("{text}: the payload does not fit, or is zero"));
        }
        return Ok(sign | (exp_all << mbits) | (payload as u64));
    }

    if rest.starts_with("0x") || rest.starts_with("0X") {
        return hex_float(text, rest, neg, mbits, ebits);
    }

    // A decimal float goes through the host's parser, which is
    // correctly rounded, and then has its sign re-applied: `-0` must
    // not be lost, and parsing the magnitude keeps the two apart.
    let cleaned = strip_separators(rest).ok_or_else(|| format!("{text} is not a number"))?;
    if mbits == 23 {
        let v: f32 = cleaned
            .parse()
            .map_err(|_| format!("{text} is not a number"))?;
        // Overflow to infinity is the specified result for a decimal
        // literal too large to represent, and that is what the host's
        // parser gives, so there is nothing to check here.
        Ok(sign | u64::from(v.abs().to_bits()))
    } else {
        let v: f64 = cleaned
            .parse()
            .map_err(|_| format!("{text} is not a number"))?;
        Ok(sign | v.abs().to_bits())
    }
}

/// Removes `_` separators, refusing any that is not between digits.
fn strip_separators(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let b = text.as_bytes();
    for (i, &c) in b.iter().enumerate() {
        if c == b'_' {
            let before = i
                .checked_sub(1)
                .map(|j| b[j])
                .is_some_and(|c| c.is_ascii_alphanumeric());
            let after = b.get(i + 1).is_some_and(u8::is_ascii_alphanumeric);
            if !before || !after {
                return None;
            }
            continue;
        }
        out.push(char::from(c));
    }
    Some(out)
}

/// A hexadecimal float, decoded exactly and rounded once.
fn hex_float(text: &str, rest: &str, neg: bool, mbits: u32, ebits: u32) -> Result<u64, String> {
    let body = strip_separators(&rest[2..]).ok_or_else(|| format!("{text} is not a number"))?;
    let (mantissa, exponent) = match body.find(['p', 'P']) {
        Some(i) => {
            let e = &body[i + 1..];
            let (eneg, digits) = match e.strip_prefix('-') {
                Some(d) => (true, d),
                None => (false, e.strip_prefix('+').unwrap_or(e)),
            };
            let mag: i64 = digits
                .parse()
                .map_err(|_| format!("{text}: the exponent is not a number"))?;
            (&body[..i], if eneg { -mag } else { mag })
        }
        None => (&body[..], 0i64),
    };
    let (whole, frac) = match mantissa.find('.') {
        Some(i) => (&mantissa[..i], &mantissa[i + 1..]),
        None => (mantissa, ""),
    };
    if whole.is_empty() && frac.is_empty() {
        return Err(format!("{text} has no digits"));
    }

    // The significand as an integer, with a sticky bit for anything
    // that did not fit: 128 bits hold 32 hexadecimal digits, and a
    // longer literal can still only break a tie.
    let mut sig: u128 = 0;
    let mut sticky = false;
    let mut exp = exponent;
    for c in whole.chars() {
        let d = c
            .to_digit(16)
            .ok_or_else(|| format!("{text} is not a number"))?;
        push_digit(&mut sig, &mut sticky, &mut exp, d, 4);
    }
    for c in frac.chars() {
        let d = c
            .to_digit(16)
            .ok_or_else(|| format!("{text} is not a number"))?;
        push_digit(&mut sig, &mut sticky, &mut exp, d, 4);
        exp -= 4;
    }
    Ok(assemble(neg, sig, sticky, exp, mbits, ebits))
}

/// Appends one `width`-bit digit to the significand, pushing the lowest
/// bits into the sticky flag once the significand is full.
fn push_digit(sig: &mut u128, sticky: &mut bool, exp: &mut i64, digit: u32, width: u32) {
    if *sig >> (128 - width) != 0 {
        // No room: the digit's bits are all below what is kept.
        *sticky = *sticky || digit != 0;
        *exp += i64::from(width);
        return;
    }
    *sig = (*sig << width) | u128::from(digit);
}

/// Builds a float's bits from `sig × 2^exp`, rounding to nearest, ties
/// to even, with `sticky` saying whether anything was dropped below.
fn assemble(neg: bool, sig: u128, sticky: bool, exp: i64, mbits: u32, ebits: u32) -> u64 {
    let sign = u64::from(neg) << (mbits + ebits);
    let exp_all = (1u64 << ebits) - 1;
    if sig == 0 {
        return sign;
    }
    let precision = i64::from(mbits) + 1;
    let bias = (1i64 << (ebits - 1)) - 1;

    let mut sig = sig;
    let mut exp = exp;
    let mut round = false;
    let mut sticky = sticky;

    // Normalise to exactly `precision` significant bits.
    let msb = 128 - i64::from(sig.leading_zeros());
    if msb > precision {
        let k = (msb - precision) as u32;
        shift_right(&mut sig, k, &mut round, &mut sticky);
        exp += i64::from(k);
    } else {
        let k = (precision - msb) as u32;
        sig <<= k;
        exp -= i64::from(k);
    }

    // The unbiased exponent of the leading bit.
    let mut e = exp + precision - 1;
    let emin = 1 - bias;
    if e < emin {
        // Subnormal: shift until the exponent is the smallest there is.
        let k = (emin - e).min(130) as u32;
        shift_right(&mut sig, k, &mut round, &mut sticky);
        e = emin;
    }

    // Round to nearest, ties to even.
    if round && (sticky || (sig & 1) == 1) {
        sig += 1;
        if sig >> precision != 0 {
            sig >>= 1;
            e += 1;
        }
    }

    if sig == 0 {
        return sign;
    }
    if e > bias {
        return sign | (exp_all << mbits);
    }
    if sig >> mbits == 0 {
        // Still subnormal after rounding: a zero exponent field.
        return sign | (sig as u64);
    }
    let biased = (e + bias) as u64;
    sign | (biased << mbits) | ((sig as u64) & ((1u64 << mbits) - 1))
}

/// Shifts `sig` right by `k`, keeping the bit that falls off in `round`
/// and everything below it in `sticky`.
fn shift_right(sig: &mut u128, k: u32, round: &mut bool, sticky: &mut bool) {
    if k == 0 {
        return;
    }
    if k >= 128 {
        *sticky = *sticky || *round || *sig != 0;
        *round = false;
        *sig = 0;
        return;
    }
    let dropped = *sig & ((1u128 << k) - 1);
    *sticky = *sticky || *round || (dropped & ((1u128 << (k - 1)) - 1)) != 0;
    *round = (dropped >> (k - 1)) & 1 == 1;
    *sig >>= k;
}

#[cfg(test)]
mod tests;
