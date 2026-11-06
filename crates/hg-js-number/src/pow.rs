//! Number::exponentiate: the spec's special cases exactly, then a
//! deterministic approximation from IEEE basic operations only.
//!
//! spec: https://tc39.es/ecma262/#sec-numeric-types-number-exponentiate
//!
//! The spec leaves step 13 implementation-approximated. This is the
//! interim deterministic version: integral exponents below 2^53 use
//! left-to-right binary exponentiation in double-double with the binary
//! exponent kept apart (so no intermediate overflows or underflows), and
//! the rest use exp(y · ln x) in double-double. Either way the final
//! double-double value is rounded once, exactly, to binary64. The error
//! is far below one ulp (the result is correctly rounded except in
//! extraordinarily close cases), and exact whenever an integral power is
//! representable. The correctly rounded version belongs to hg-math (JS
//! spec 6.9); this one only promises determinism and ≤ 1 ulp.

use crate::big::Big;
use crate::dd::{Dd, pow2};
use crate::round::ratio_to_f64;

/// ln 2 as a double-double (checked against a series in the tests).
pub(crate) const LN2: Dd = Dd::from_bits(0x3fe6_2e42_fefa_39ef, 0x3c7a_bc9e_3b39_803f);

/// 1 / ln 2 (= log2 e), rounded; only used to pick a reduction integer.
const INV_LN2: f64 = std::f64::consts::LOG2_E;

/// √2 rounded to a double: the split point of the log reduction (any
/// point near √2 works; it only bounds the series argument).
const SQRT2: f64 = std::f64::consts::SQRT_2;

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

/// Whether finite `y` is an integral Number.
fn is_integral(y: f64) -> bool {
    let (f, e) = decompose(y.abs());
    if e >= 0 || f == 0 {
        return true;
    }
    let shift = e.unsigned_abs();
    shift < 64 && f & ((1u64 << shift) - 1) == 0
}

/// Whether finite `y` is an odd integral Number.
fn is_odd_integral(y: f64) -> bool {
    if !is_integral(y) {
        return false;
    }
    let (f, e) = decompose(y.abs());
    match e {
        0 => f & 1 == 1,
        e if e < 0 && e > -64 => (f >> e.unsigned_abs()) & 1 == 1,
        _ => false,
    }
}

/// A positive finite value m × 2^e with m a double-double near [1, 2).
#[derive(Clone, Copy, Debug)]
struct Scaled {
    m: Dd,
    e: i64,
}

impl Scaled {
    /// Moves the binary exponent of m.hi into e (m.hi normal, positive).
    fn normalize(self) -> Scaled {
        let biased = ((self.m.hi.to_bits() >> 52) & 0x7ff) as i32;
        let k = biased - 1023;
        Scaled {
            m: self.m.scale_pow2(-k),
            e: self.e + i64::from(k),
        }
    }

    /// The binary64 nearest to m × 2^e, rounded once and exactly.
    fn to_f64(self) -> f64 {
        let (fh, xh) = decompose(self.m.hi);
        if self.m.lo == 0.0 {
            return ratio_to_f64(&Big::from_u64(fh), &Big::from_u64(1), xh + self.e);
        }
        let (fl, xl) = decompose(self.m.lo.abs());
        let base = xh.min(xl);
        let mut n = Big::from_u64(fh);
        n.shl((xh - base) as u64);
        let mut l = Big::from_u64(fl);
        l.shl((xl - base) as u64);
        if self.m.lo > 0.0 {
            n.add(&l);
        } else {
            n.sub(&l);
        }
        ratio_to_f64(&n, &Big::from_u64(1), base + self.e)
    }
}

/// Beyond this binary exponent magnitude the result is certainly ±∞ or 0.
const SATURATE: i64 = 2200;

/// Number::exponentiate(base, exponent).
pub(crate) fn exponentiate(base: f64, exponent: f64) -> f64 {
    // spec: #sec-numeric-types-number-exponentiate steps 1–3
    if exponent.is_nan() {
        return f64::NAN;
    }
    if exponent == 0.0 {
        return 1.0;
    }
    if base.is_nan() {
        return f64::NAN;
    }
    // step 4: base +∞
    if base == f64::INFINITY {
        return if exponent > 0.0 { f64::INFINITY } else { 0.0 };
    }
    // step 5: base −∞
    if base == f64::NEG_INFINITY {
        let odd = exponent.is_finite() && is_odd_integral(exponent);
        return match (exponent > 0.0, odd) {
            (true, true) => f64::NEG_INFINITY,
            (true, false) => f64::INFINITY,
            (false, true) => -0.0,
            (false, false) => 0.0,
        };
    }
    // steps 6–7: base ±0
    if base == 0.0 {
        let odd = exponent.is_finite() && is_odd_integral(exponent);
        let negative_zero = base.to_bits() >> 63 == 1;
        return match (negative_zero, exponent > 0.0, odd) {
            (false, true, _) => 0.0,
            (false, false, _) => f64::INFINITY,
            (true, true, true) => -0.0,
            (true, true, false) => 0.0,
            (true, false, true) => f64::NEG_INFINITY,
            (true, false, false) => f64::INFINITY,
        };
    }
    // step 8: base is finite and non-zero. Steps 9–10: exponent ±∞.
    let ax = base.abs();
    if exponent.is_infinite() {
        let grows = exponent > 0.0;
        return if ax == 1.0 {
            f64::NAN
        } else if (ax > 1.0) == grows {
            f64::INFINITY
        } else {
            0.0
        };
    }
    // step 11: exponent finite non-zero. Step 12.
    let integral = is_integral(exponent);
    if base < 0.0 && !integral {
        return f64::NAN;
    }
    // step 13: implementation-approximated ℝ(base)^ℝ(exponent).
    let magnitude = if integral && exponent.abs() < 9_007_199_254_740_992.0 {
        integer_power(ax, exponent)
    } else {
        exp_log_power(ax, exponent)
    };
    if base < 0.0 && is_odd_integral(exponent) {
        -magnitude
    } else {
        magnitude
    }
}

/// x^y for x > 0 finite and y integral with 0 < |y| < 2^53.
fn integer_power(x: f64, y: f64) -> f64 {
    let (f, e) = decompose(y.abs());
    let n = if e >= 0 {
        f << e
    } else {
        f >> e.unsigned_abs()
    };
    let xs = Scaled {
        m: Dd::from(x),
        e: 0,
    };
    let xs = normalize_input(xs);
    let mut acc = Scaled {
        m: Dd::from(1.0),
        e: 0,
    };
    let top = 63 - n.leading_zeros();
    let mut saturated = false;
    for bit in (0..=top).rev() {
        acc = Scaled {
            m: acc.m.mul(acc.m),
            e: acc.e * 2,
        }
        .normalize();
        if (n >> bit) & 1 == 1 {
            acc = Scaled {
                m: acc.m.mul(xs.m),
                e: acc.e + xs.e,
            }
            .normalize();
        }
        // Every later step only moves log2 of acc further from 0 in the
        // same direction, so past ±SATURATE the result is decided.
        if acc.e.abs() > SATURATE {
            saturated = true;
            break;
        }
    }
    let positive = y > 0.0;
    if saturated {
        return if (acc.e > 0) == positive {
            f64::INFINITY
        } else {
            0.0
        };
    }
    if !positive {
        acc = Scaled {
            m: Dd::from(1.0).div(acc.m),
            e: -acc.e,
        }
        .normalize();
    }
    acc.to_f64()
}

/// Brings a positive finite double (possibly subnormal) to m in [1, 2).
fn normalize_input(s: Scaled) -> Scaled {
    if s.m.hi < f64::MIN_POSITIVE {
        // subnormal: scale by 2^64 first so the leading bit is normal
        Scaled {
            m: s.m.scale_pow2(64),
            e: s.e - 64,
        }
        .normalize()
    } else {
        s.normalize()
    }
}

/// ln x in double-double for x > 0 finite.
fn ln_dd(x: f64) -> Dd {
    let s = normalize_input(Scaled {
        m: Dd::from(x),
        e: 0,
    });
    let (mut m, mut k) = (s.m.hi, s.e);
    if m > SQRT2 {
        m *= 0.5;
        k += 1;
    }
    // ln m = 2 atanh(t), t = (m − 1)/(m + 1), |t| ≤ 0.1716
    let num = Dd::from(m - 1.0); // exact (Sterbenz)
    let den = Dd::from(m).add(Dd::from(1.0));
    let t = num.div(den);
    let t2 = t.mul(t);
    let mut sum = t;
    let mut power = t;
    let mut i = 1.0f64;
    while i < 80.0 {
        power = power.mul(t2);
        i += 2.0;
        let term = power.div(Dd::from(i));
        sum = sum.add(term);
        if term.hi.abs() <= sum.hi.abs() * pow2(-112) {
            break;
        }
    }
    LN2.mul_f64(k as f64).add(sum.mul_f64(2.0))
}

/// x^y = exp(y · ln x) for x > 0 finite, y finite non-zero.
fn exp_log_power(x: f64, y: f64) -> f64 {
    let ln = ln_dd(x);
    if ln.hi == 0.0 {
        return 1.0; // x = 1
    }
    let rough = ln.hi * y;
    if rough > 710.0 {
        return f64::INFINITY;
    }
    if rough < -746.0 {
        return 0.0;
    }
    let p = ln.mul_f64(y);
    exp_dd(p)
}

/// e^p rounded to binary64 for |p| ≤ 746 (p in double-double).
fn exp_dd(p: Dd) -> f64 {
    // p = k ln 2 + r, |r| ≲ ln 2 / 2
    let guess = p.hi * INV_LN2;
    let k = if guess < 0.0 {
        (guess - 0.5) as i64
    } else {
        (guess + 0.5) as i64
    };
    let r = p.sub(LN2.mul_f64(k as f64));
    // e^r = (e^(r/2^10))^(2^10); work with expm1 so squaring keeps digits:
    // (1 + s)^2 − 1 = 2s + s²
    let r = r.scale_pow2(-10);
    let mut term = r;
    let mut s = r;
    let mut i = 1.0f64;
    while i < 30.0 {
        i += 1.0;
        term = term.mul(r).div(Dd::from(i));
        s = s.add(term);
        if term.hi.abs() <= s.hi.abs() * pow2(-112) {
            break;
        }
    }
    for _ in 0..10 {
        s = s.mul_f64(2.0).add(s.mul(s));
    }
    Scaled {
        m: Dd::from(1.0).add(s),
        e: k,
    }
    .normalize()
    .to_f64()
}
