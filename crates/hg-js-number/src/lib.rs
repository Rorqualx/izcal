//! `hg-js-number`: the conversions and operators of the JavaScript Number
//! type that need more than one IEEE 754 operation. Layer L2e, tier R.
//!
//! - [`number_to_string`]: Number::toString(x, 10) with the shortest
//!   digits that read back as x, the closest of them, ties to even.
//! - [`string_to_number`]: StringToNumber over UTF-16, correctly rounded.
//! - [`parse_json_number`]: the JSON number grammar, correctly rounded.
//! - [`remainder`]: Number::remainder, exact.
//! - [`to_int32`] / [`to_uint32`]: ToInt32 / ToUint32 on a Number.
//! - [`exponentiate`]: Number::exponentiate, deterministic on every OS.
//!
//! This is Izcal's own implementation, written from the ECMA-262 text
//! (<https://tc39.es/ecma262/>), IEEE 754 semantics and published prose
//! descriptions of the algorithms; no library and no other engine's code.
//! Only IEEE basic operations (+ − × ÷, comparisons, bit casts, integer
//! conversions) touch floating-point values, so every platform produces
//! the same bits: no platform libm, no FMA, no std float formatting or
//! parsing. Exact steps run on a small arbitrary-precision integer.
//!
//! G13: no OS access of any kind. Threading: every function is pure and
//! may be called from any thread.

#![forbid(unsafe_code)]

mod big;
mod dd;
mod parse;
mod pow;
mod round;
mod to_string;

#[cfg(test)]
mod tests;

/// ECMA-262 Number::toString(x, 10)
/// (<https://tc39.es/ecma262/#sec-numeric-types-number-tostring>), with
/// Note 2's refinement: the fewest significant digits that convert back
/// to `x`, the closest such digits to `x`, the even one on a tie.
pub fn number_to_string(x: f64) -> String {
    to_string::number_to_string(x)
}

/// ECMA-262 StringToNumber
/// (<https://tc39.es/ecma262/#sec-stringtonumber>) over UTF-16 code
/// units.
///
/// StrWhiteSpace (WhiteSpace and LineTerminator) is trimmed from both
/// ends; empty or all-white-space text is +0. A StrDecimalLiteral may
/// carry a sign, be `Infinity`, use the point forms `.5` and `5.` and an
/// exponent `e`/`E` with an optional sign. A NonDecimalIntegerLiteral
/// (`0x`, `0o`, `0b`, either case) has no sign. Numeric separators are
/// not allowed. Anything else is NaN. The result is the correctly
/// rounded value of the literal, which RoundMVResult permits.
pub fn string_to_number(units: &[u16]) -> f64 {
    parse::string_to_number(units)
}

/// Parses a JSON number (RFC 8259 / ECMA-404 grammar:
/// `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`) to the nearest
/// binary64 (round half to even); `None` if the text is not exactly a
/// JSON number (no white space, no leading `+`, no leading zeros).
pub fn parse_json_number(text: &str) -> Option<f64> {
    parse::parse_json_number(text)
}

/// ECMA-262 Number::remainder
/// (<https://tc39.es/ecma262/#sec-numeric-types-number-remainder>): the
/// remainder of a truncating division, with the sign of the dividend.
/// The result is always representable, so it is computed exactly.
pub fn remainder(n: f64, d: f64) -> f64 {
    // spec: #sec-numeric-types-number-remainder steps 1–5
    if n.is_nan() || d.is_nan() || n.is_infinite() {
        return f64::NAN;
    }
    if d.is_infinite() {
        return n;
    }
    if d == 0.0 {
        return f64::NAN;
    }
    if n == 0.0 {
        return n;
    }
    // steps 6–11: r = n − d × truncate(n / d), exactly; -0 when r = 0
    // and n < 0, so the sign of n always carries over.
    let (an, ad) = (n.abs(), d.abs());
    if an < ad {
        return n;
    }
    let (fn_, en) = decompose(an);
    let (fd, ed) = decompose(ad);
    let (rem, exp) = if en >= ed {
        // (fn × 2^(en − ed)) mod fd, feeding the shift in 64-bit chunks
        let mut rem = fn_ % fd;
        let mut shift = (en - ed) as u64;
        while shift > 0 {
            let c = shift.min(64);
            rem = ((u128::from(rem) << c) % u128::from(fd)) as u64;
            shift -= c;
        }
        (rem, ed)
    } else {
        // |n| ≥ |d| with en < ed: fd × 2^(ed − en) ≤ fn < 2^53
        (fn_ % (fd << (ed - en)), en)
    };
    let magnitude = compose(rem, exp);
    if n < 0.0 { -magnitude } else { magnitude }
}

/// x = f × 2^e exactly, for finite x ≥ 0.
fn decompose(x: f64) -> (u64, i64) {
    let bits = x.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i64;
    let fraction = bits & ((1u64 << 52) - 1);
    if biased == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1u64 << 52), biased - 1075)
    }
}

/// m × 2^e for m < 2^53 and e ≥ −1074 whose value is representable.
fn compose(m: u64, e: i64) -> f64 {
    if m == 0 {
        return 0.0;
    }
    // shift m up to 53 bits, but not below the subnormal quantum
    let want = i64::from(m.leading_zeros()) - 11;
    let shift = want.min(e + 1074).max(0);
    let (m, e) = (m << shift, e - shift);
    if m < 1u64 << 52 {
        f64::from_bits(m)
    } else {
        f64::from_bits((((e + 1075) as u64) << 52) | (m - (1u64 << 52)))
    }
}

/// ECMA-262 ToInt32 applied to a Number
/// (<https://tc39.es/ecma262/#sec-toint32>): NaN, ±0 and ±∞ give 0;
/// otherwise truncate toward zero and reduce modulo 2^32 into
/// [−2^31, 2^31).
pub fn to_int32(x: f64) -> i32 {
    to_uint32(x) as i32
}

/// ECMA-262 ToUint32 applied to a Number
/// (<https://tc39.es/ecma262/#sec-touint32>): NaN, ±0 and ±∞ give 0;
/// otherwise truncate toward zero and reduce modulo 2^32.
pub fn to_uint32(x: f64) -> u32 {
    // spec: ToIntegerOrInfinity then ToFixedSizeInteger(int, unsigned, 32)
    if !x.is_finite() || x == 0.0 {
        return 0;
    }
    let (f, e) = decompose(x.abs());
    let low = if e >= 0 {
        if e >= 32 {
            0
        } else {
            ((u128::from(f) << e) & 0xffff_ffff) as u32
        }
    } else if e <= -64 {
        0
    } else {
        ((f >> e.unsigned_abs()) & 0xffff_ffff) as u32
    };
    if x < 0.0 { low.wrapping_neg() } else { low }
}

/// ECMA-262 Number::exponentiate
/// (<https://tc39.es/ecma262/#sec-numeric-types-number-exponentiate>).
///
/// Every special case of the spec's steps is exact: a NaN exponent gives
/// NaN, a ±0 exponent gives 1, then a NaN base gives NaN, the ±∞ and ±0
/// bases follow the sign of the exponent and whether it is an odd
/// integral Number, |base| = 1 with a ±∞ exponent gives NaN, and a
/// negative finite base with a non-integral finite exponent gives NaN.
///
/// The general case is implementation-approximated. This is the
/// deterministic interim: computed with IEEE basic operations only (no
/// FMA, no libm), by binary exponentiation in double-double for integral
/// exponents below 2^53 in magnitude and by exp(y · ln x) in
/// double-double otherwise, then rounded once to binary64. The error is
/// at most 1 ulp, and integral powers whose exact value is representable
/// (`2 ** 10`, `10 ** 22`, `2 ** -1074`, `(-2) ** 3`) are exact. The
/// correctly rounded version belongs to hg-math (JS spec 6.9).
pub fn exponentiate(base: f64, exponent: f64) -> f64 {
    pow::exponentiate(base, exponent)
}
