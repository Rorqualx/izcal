//! Exact rounding of a rational value to the nearest binary64, ties to
//! even (IEEE 754 roundTiesToEven, which is what ECMA-262's 𝔽(x) means:
//! spec: https://tc39.es/ecma262/#sec-ecmascript-language-types-number-type
//! "the Number value for x").

use crate::big::Big;

/// Smallest binary exponent of a binary64 quantum (the subnormal step).
const MIN_EXP: i64 = -1074;

/// The binary64 nearest to `num / den × 2^pow2` (ties to even), for a
/// non-zero `den`. Overflow gives +∞, underflow +0.
pub(crate) fn ratio_to_f64(num: &Big, den: &Big, pow2: i64) -> f64 {
    if num.is_zero() {
        return 0.0;
    }
    // value lies in [2^(bl-1), 2^(bl+1)).
    let bl = num.bit_len() as i64 - den.bit_len() as i64 + pow2;
    if bl > 1024 {
        return f64::INFINITY;
    }
    if bl + 1 < MIN_EXP - 1 {
        // below 2^-1076, under half the smallest subnormal
        return 0.0;
    }
    // Choose the quantum exponent e so the quotient q = floor(value / 2^e)
    // has 53 bits (or fewer when e is clamped at the subnormal quantum).
    let mut e = (bl - 53).max(MIN_EXP);
    loop {
        let shift = pow2 - e;
        let (mut a, mut b) = (num.clone(), den.clone());
        if shift >= 0 {
            a.shl(shift as u64);
        } else {
            b.shl(shift.unsigned_abs());
        }
        let mut q = a.div_rem_small_quotient(&b);
        if q >= 1 << 53 {
            e += 1;
            continue;
        }
        // `a` now holds the remainder; compare 2·rem with the divisor.
        a.shl(1);
        let round_up = match a.cmp(&b) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Equal => q & 1 == 1,
            std::cmp::Ordering::Less => false,
        };
        if round_up {
            q += 1;
        }
        // q·2^e with q < 2^53 (or = 2^53 after rounding, which carries
        // into the exponent field), e ≥ -1074, and q < 2^52 only at
        // e = -1074: the bit pattern is ((e + 1074) << 52) + q.
        let bits = (((e - MIN_EXP) as u64) << 52) + q;
        if bits >= 0x7ff0_0000_0000_0000 {
            return f64::INFINITY;
        }
        return f64::from_bits(bits);
    }
}

/// How many significant decimal digits a decimal string keeps exactly.
/// Every binary64 value and every midpoint between two adjacent ones has
/// a terminating decimal expansion of at most 767 significant digits, so
/// truncating after 800 digits and remembering whether anything non-zero
/// was dropped (a "sticky" digit) cannot change the rounding.
pub(crate) const MAX_DIGITS: usize = 800;

/// A decimal value `digits × 10^exp10` being accumulated from text.
#[derive(Debug)]
pub(crate) struct Decimal {
    digits: Vec<u8>,
    exp10: i64,
    sticky: bool,
}

impl Decimal {
    /// An empty accumulator (value zero).
    pub(crate) fn new() -> Decimal {
        Decimal {
            digits: Vec::new(),
            exp10: 0,
            sticky: false,
        }
    }

    /// Appends one digit (0–9); `fraction` says whether it comes after
    /// the decimal point.
    pub(crate) fn push(&mut self, d: u8, fraction: bool) {
        if self.digits.is_empty() && d == 0 {
            if fraction {
                self.exp10 -= 1;
            }
            return;
        }
        if self.digits.len() < MAX_DIGITS {
            self.digits.push(d);
            if fraction {
                self.exp10 -= 1;
            }
        } else {
            if !fraction {
                self.exp10 += 1;
            }
            if d != 0 {
                self.sticky = true;
            }
        }
    }

    /// Adds the value of an exponent part (already saturated by the
    /// caller to a range far beyond any finite result).
    pub(crate) fn add_exponent(&mut self, e: i64) {
        self.exp10 = self.exp10.saturating_add(e);
    }

    /// The nearest binary64 (ties to even) to the accumulated value.
    pub(crate) fn to_f64(&self) -> f64 {
        if self.digits.is_empty() {
            return 0.0;
        }
        let n = self.digits.len() as i64;
        let mut exp10 = self.exp10;
        // value in [10^(n-1+exp10), 10^(n+exp10))
        if n - 1 + exp10 >= 309 {
            return f64::INFINITY;
        }
        if n + exp10 <= -324 {
            // below 10^-324, less than half of 2^-1074 ≈ 4.94e-324
            return 0.0;
        }
        let mut mantissa = Big::zero();
        for chunk in self.digits.chunks(9) {
            let mut v = 0u32;
            let mut scale = 1u32;
            for &d in chunk {
                v = v * 10 + u32::from(d);
                scale *= 10;
            }
            mantissa.mul_add_small(scale, v);
        }
        if self.sticky {
            mantissa.mul_add_small(10, 1);
            exp10 -= 1;
        }
        if exp10 >= 0 {
            mantissa.mul_pow10(exp10 as u64);
            ratio_to_f64(&mantissa, &Big::from_u64(1), 0)
        } else {
            // 10^-k = 5^-k × 2^-k: keep the power of two out of the divisor.
            let mut den = Big::from_u64(1);
            den.mul_pow5(exp10.unsigned_abs());
            ratio_to_f64(&mantissa, &den, exp10)
        }
    }
}
