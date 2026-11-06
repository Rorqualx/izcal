//! StringToNumber and the JSON number grammar, both correctly rounded.
//!
//! spec: https://tc39.es/ecma262/#sec-stringtonumber
//! spec: https://tc39.es/ecma262/#sec-tonumber-applied-to-the-string-type
//!
//! RoundMVResult (spec: #sec-roundmvresult) lets an implementation
//! replace the digits after the 20th significant one by zeros, optionally
//! incrementing the 20th; the two options differ by far less than one
//! binary64 step, so the correctly rounded value of the full literal is
//! always 𝔽 of one of them, and this crate returns it.

use crate::big::Big;
use crate::round::{Decimal, ratio_to_f64};

/// Saturation bound for an exponent part: far beyond any value that
/// could change a finite result, and safe to add to a digit count.
const EXP_LIMIT: i64 = 1 << 40;

/// Whether `u` is a StrWhiteSpaceChar: WhiteSpace or LineTerminator.
/// spec: https://tc39.es/ecma262/#sec-white-space (TAB, VT, FF, ZWNBSP,
/// any Space_Separator code point) and
/// https://tc39.es/ecma262/#sec-line-terminators (LF, CR, LS, PS).
/// Every one of them is in the BMP, so one code unit decides.
fn is_str_white_space(u: u16) -> bool {
    matches!(
        u,
        0x0009 | 0x000a | 0x000b | 0x000c | 0x000d | 0x0020 | 0x00a0 | 0x1680 | 0x2000
            ..=0x200a | 0x2028 | 0x2029 | 0x202f | 0x205f | 0x3000 | 0xfeff
    )
}

fn ascii_digit(u: u16) -> Option<u8> {
    if (0x30..=0x39).contains(&u) {
        Some((u - 0x30) as u8)
    } else {
        None
    }
}

/// StringToNumber over UTF-16 code units.
pub(crate) fn string_to_number(units: &[u16]) -> f64 {
    // spec: #sec-stringtonumber step 1, StringNumericLiteral: optional
    // StrWhiteSpace around an optional StrNumericLiteral.
    let start = units
        .iter()
        .position(|&u| !is_str_white_space(u))
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|&u| !is_str_white_space(u))
        .map_or(start, |i| i + 1);
    let text = units.get(start..end).unwrap_or(&[]);
    if text.is_empty() {
        // spec: StringNumericValue, StringNumericLiteral ::: StrWhiteSpace_opt
        return 0.0;
    }
    // NonDecimalIntegerLiteral[~Sep]: 0x / 0o / 0b, no sign.
    if let [0x30, marker, rest @ ..] = text {
        let bits_per_digit = match marker {
            0x78 | 0x58 => Some(4), // x X
            0x6f | 0x4f => Some(3), // o O
            0x62 | 0x42 => Some(1), // b B
            _ => None,
        };
        if let Some(bits) = bits_per_digit {
            return non_decimal(rest, bits).unwrap_or(f64::NAN);
        }
    }
    // StrDecimalLiteral: optional sign then StrUnsignedDecimalLiteral.
    let (negative, body) = match text {
        [0x2d, rest @ ..] => (true, rest),
        [0x2b, rest @ ..] => (false, rest),
        _ => (false, text),
    };
    let magnitude = match unsigned_decimal(body) {
        Some(v) => v,
        None => return f64::NAN,
    };
    // spec: StrDecimalLiteral ::: - StrUnsignedDecimalLiteral (+0 → -0)
    if negative { -magnitude } else { magnitude }
}

/// The MV of a HexDigits / OctalDigits / BinaryDigits sequence rounded
/// to the nearest binary64; None when the sequence is empty or holds a
/// digit outside the radix.
fn non_decimal(digits: &[u16], bits_per_digit: u32) -> Option<f64> {
    if digits.is_empty() {
        return None;
    }
    let radix = 1u32 << bits_per_digit;
    // Keep the leading significant bits exactly in a u128 (at least 116
    // of them, well past the 54 that decide the rounding); later digits
    // only scale the value and set a sticky bit.
    let mut acc: u128 = 0;
    let mut extra_bits: i64 = 0;
    let mut sticky = false;
    for &u in digits {
        let v = match u {
            0x30..=0x39 => u32::from(u - 0x30),
            0x61..=0x66 => u32::from(u - 0x61 + 10),
            0x41..=0x46 => u32::from(u - 0x41 + 10),
            _ => return None,
        };
        if v >= radix {
            return None;
        }
        if acc >> 116 == 0 {
            acc = (acc << bits_per_digit) | u128::from(v);
        } else {
            extra_bits += i64::from(bits_per_digit);
            sticky |= v != 0;
        }
    }
    let mut n = Big::from_u128(acc);
    let mut pow2 = extra_bits;
    if sticky {
        n.mul_add_small(2, 1);
        pow2 -= 1;
    }
    Some(ratio_to_f64(&n, &Big::from_u64(1), pow2))
}

/// StrUnsignedDecimalLiteral: "Infinity", or DecimalDigits with an
/// optional point and fraction (".5" and "5." both allowed, "." not) and
/// an optional ExponentPart.
fn unsigned_decimal(body: &[u16]) -> Option<f64> {
    const INFINITY: [u16; 8] = [0x49, 0x6e, 0x66, 0x69, 0x6e, 0x69, 0x74, 0x79];
    if body == INFINITY {
        return Some(f64::INFINITY);
    }
    let mut dec = Decimal::new();
    let mut i = 0;
    let mut int_digits = 0usize;
    while let Some(d) = body.get(i).copied().and_then(ascii_digit) {
        dec.push(d, false);
        int_digits += 1;
        i += 1;
    }
    let mut frac_digits = 0usize;
    if body.get(i) == Some(&0x2e) {
        i += 1;
        while let Some(d) = body.get(i).copied().and_then(ascii_digit) {
            dec.push(d, true);
            frac_digits += 1;
            i += 1;
        }
    }
    if int_digits == 0 && frac_digits == 0 {
        return None;
    }
    if matches!(body.get(i), Some(&0x65) | Some(&0x45)) {
        i += 1;
        let e = exponent(body.get(i..).unwrap_or(&[]))?;
        dec.add_exponent(e);
        i = body.len();
    }
    if i != body.len() {
        return None;
    }
    Some(dec.to_f64())
}

/// ExponentPart after the e/E: SignedInteger, the whole rest of the
/// input; saturated at ±EXP_LIMIT.
fn exponent(rest: &[u16]) -> Option<i64> {
    let (negative, digits) = match rest {
        [0x2d, r @ ..] => (true, r),
        [0x2b, r @ ..] => (false, r),
        _ => (false, rest),
    };
    if digits.is_empty() {
        return None;
    }
    let mut e: i64 = 0;
    for &u in digits {
        let d = ascii_digit(u)?;
        e = (e * 10 + i64::from(d)).min(EXP_LIMIT);
    }
    Some(if negative { -e } else { e })
}

/// A JSON number (ECMA-404 / RFC 8259):
/// `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`, rounded to nearest.
pub(crate) fn parse_json_number(text: &str) -> Option<f64> {
    let b = text.as_bytes();
    let mut i = 0;
    let negative = b.first() == Some(&b'-');
    if negative {
        i += 1;
    }
    let mut dec = Decimal::new();
    match b.get(i) {
        Some(b'0') => {
            i += 1;
        }
        Some(c) if c.is_ascii_digit() => {
            while let Some(c) = b.get(i).filter(|c| c.is_ascii_digit()) {
                dec.push(c - b'0', false);
                i += 1;
            }
        }
        _ => return None,
    }
    if b.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while let Some(c) = b.get(i).filter(|c| c.is_ascii_digit()) {
            dec.push(c - b'0', true);
            i += 1;
        }
        if i == start {
            return None;
        }
    }
    if matches!(b.get(i), Some(b'e') | Some(b'E')) {
        i += 1;
        let mut exp_negative = false;
        match b.get(i) {
            Some(b'-') => {
                exp_negative = true;
                i += 1;
            }
            Some(b'+') => i += 1,
            _ => {}
        }
        let start = i;
        let mut e: i64 = 0;
        while let Some(c) = b.get(i).filter(|c| c.is_ascii_digit()) {
            e = (e * 10 + i64::from(c - b'0')).min(EXP_LIMIT);
            i += 1;
        }
        if i == start {
            return None;
        }
        dec.add_exponent(if exp_negative { -e } else { e });
    }
    if i != b.len() {
        return None;
    }
    let v = dec.to_f64();
    Some(if negative { -v } else { v })
}
