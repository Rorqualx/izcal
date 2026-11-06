//! Number::toString(x, 10): shortest round-trip digits, closest to x,
//! ties to the even digit.
//!
//! spec: https://tc39.es/ecma262/#sec-numeric-types-number-tostring
//!
//! The digits come from the free-format method described in prose by
//! Steele & White and by Burger & Dybvig: with exact integer arithmetic,
//! x and the half-way points to its neighbours are written as fractions
//! over a common denominator, the value is scaled by a power of ten so the
//! first digit is the leading digit, and digits are produced one at a time
//! until the digits so far (or the digits so far with the last one raised
//! by one) name a value inside x's rounding interval.

use crate::big::Big;
use std::cmp::Ordering;

/// Number::toString(x, 10).
pub(crate) fn number_to_string(x: f64) -> String {
    // spec: #sec-numeric-types-number-tostring steps 1–4
    if x.is_nan() {
        return String::from("NaN");
    }
    if x == 0.0 {
        return String::from("0");
    }
    if x < 0.0 {
        let mut out = String::from("-");
        out.push_str(&number_to_string(-x));
        return out;
    }
    if x == f64::INFINITY {
        return String::from("Infinity");
    }
    // spec: step 5 with Note 2's refinement
    let (digits, n) = shortest_digits(x);
    format_digits(&digits, n)
}

/// The digits of s (most significant first, no trailing zero) and the
/// exponent n of spec step 5, for a positive finite `x`: x is the
/// binary64 nearest to 0.d1d2…dk × 10^n.
pub(crate) fn shortest_digits(x: f64) -> (Vec<u8>, i64) {
    let bits = x.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i64;
    let fraction = bits & ((1u64 << 52) - 1);
    // x = f × 2^e exactly
    let (f, e) = if biased == 0 {
        (fraction, -1074i64)
    } else {
        (fraction | (1u64 << 52), biased - 1075)
    };
    // Round-half-even on input: when f is even, a decimal exactly at the
    // half-way point to a neighbour still reads back as x.
    let inclusive = f & 1 == 0;
    // When f is a power of two (and x is not the smallest normal), the
    // neighbour below is half as far away as the neighbour above.
    let asymmetric = f == 1u64 << 52 && biased > 1;

    // value = r/s, upper bound (r + m_plus)/s, lower bound (r - m_minus)/s
    let (mut r, mut s, mut m_plus, mut m_minus);
    if e >= 0 {
        let e = e as u64;
        if asymmetric {
            r = Big::from_u64(f);
            r.shl(e + 2);
            s = Big::from_u64(4);
            m_plus = Big::pow2(e + 1);
            m_minus = Big::pow2(e);
        } else {
            r = Big::from_u64(f);
            r.shl(e + 1);
            s = Big::from_u64(2);
            m_plus = Big::pow2(e);
            m_minus = Big::pow2(e);
        }
    } else {
        let ne = e.unsigned_abs();
        if asymmetric {
            r = Big::from_u64(f);
            r.shl(2);
            s = Big::pow2(ne + 2);
            m_plus = Big::from_u64(2);
            m_minus = Big::from_u64(1);
        } else {
            r = Big::from_u64(f);
            r.shl(1);
            s = Big::pow2(ne + 1);
            m_plus = Big::from_u64(1);
            m_minus = Big::from_u64(1);
        }
    }

    // Estimate k ≈ log10(x) from the bit length (x < 2^b), using
    // 78913 / 2^18 ≈ log10(2); the loops below correct it exactly.
    let b = 64 - i64::from(f.leading_zeros()) + e;
    let mut k = (b * 78913) >> 18;
    if k >= 0 {
        s.mul_pow10(k as u64);
    } else {
        let p = k.unsigned_abs();
        r.mul_pow10(p);
        m_plus.mul_pow10(p);
        m_minus.mul_pow10(p);
    }
    // k is right when the upper bound is below 10^k (at or below when the
    // bound itself is excluded) and would not be for k - 1.
    let high_fits = |high: &Big, s: &Big| {
        if inclusive { *high < *s } else { *high <= *s }
    };
    while !high_fits(&r.plus(&m_plus), &s) {
        s.mul_small(10);
        k += 1;
    }
    loop {
        let mut high10 = r.plus(&m_plus);
        high10.mul_small(10);
        if !high_fits(&high10, &s) {
            break;
        }
        r.mul_small(10);
        m_plus.mul_small(10);
        m_minus.mul_small(10);
        k -= 1;
    }

    // Digit generation.
    let mut digits = Vec::with_capacity(17);
    loop {
        r.mul_small(10);
        m_plus.mul_small(10);
        m_minus.mul_small(10);
        let mut d = 0u8;
        while r >= s {
            r.sub(&s);
            d += 1;
        }
        // low: stopping here (digit d) stays inside the interval
        let low = if inclusive { r <= m_minus } else { r < m_minus };
        // high: raising the digit to d + 1 stays inside the interval
        let high = {
            let up = r.plus(&m_plus);
            if inclusive { up >= s } else { up > s }
        };
        if !low && !high {
            digits.push(d);
            continue;
        }
        let last = if low && !high {
            d
        } else if high && !low {
            d + 1
        } else {
            // Both are shortest; take the closer one (Note 2), ties to
            // the even digit.
            let mut twice = r.clone();
            twice.shl(1);
            match twice.cmp(&s) {
                Ordering::Less => d,
                Ordering::Greater => d + 1,
                Ordering::Equal => {
                    if d.is_multiple_of(2) {
                        d
                    } else {
                        d + 1
                    }
                }
            }
        };
        digits.push(last);
        break;
    }
    // The method never produces a digit of 10 (the interval check at the
    // previous position would have stopped earlier), but carry anyway so
    // the function stays total.
    let mut i = digits.len();
    while i > 0 && digits[i - 1] == 10 {
        digits[i - 1] = 0;
        if i == 1 {
            digits.insert(0, 1);
            k += 1;
        } else {
            digits[i - 2] += 1;
        }
        i -= 1;
    }
    while digits.len() > 1 && digits.last() == Some(&0) {
        digits.pop();
    }
    (digits, k)
}

/// Formats digits d1…dk with exponent n per spec steps 6–12.
fn format_digits(digits: &[u8], n: i64) -> String {
    let k = digits.len() as i64;
    let mut out = String::with_capacity(digits.len() + 8);
    let push_digits = |out: &mut String, ds: &[u8]| {
        for &d in ds {
            out.push(char::from(b'0' + d));
        }
    };
    // spec: step 6 (n in the inclusive interval from -5 to 21)
    if (-5..=21).contains(&n) {
        if n >= k {
            // step 6.a: the digits then n - k zeros
            push_digits(&mut out, digits);
            for _ in 0..(n - k) {
                out.push('0');
            }
        } else if n > 0 {
            // step 6.b: n digits, a point, the remaining k - n digits
            let (int_part, frac_part) = digits.split_at(n as usize);
            push_digits(&mut out, int_part);
            out.push('.');
            push_digits(&mut out, frac_part);
        } else {
            // step 6.c–d: "0.", -n zeros, the digits
            out.push_str("0.");
            for _ in 0..(-n) {
                out.push('0');
            }
            push_digits(&mut out, digits);
        }
        return out;
    }
    // spec: steps 7–12, exponential notation
    let sign = if n < 0 { '-' } else { '+' };
    let (first, rest) = digits.split_at(1);
    push_digits(&mut out, first);
    if !rest.is_empty() {
        out.push('.');
        push_digits(&mut out, rest);
    }
    out.push('e');
    out.push(sign);
    out.push_str(&(n - 1).unsigned_abs().to_string());
    out
}
