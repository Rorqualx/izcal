//! Double-double arithmetic: a value hi + lo held as two binary64 numbers
//! with |lo| ≤ half an ulp of hi, about 106 significant bits.
//!
//! Only IEEE 754 basic operations are used (+ − × ÷, each correctly
//! rounded, round to nearest even). The exact product of two doubles
//! comes from Veltkamp splitting and Dekker's product, not from a fused
//! multiply-add, so every platform computes the same bits. Rust never
//! contracts `a * b + c` into an FMA on its own.

/// A double-double value hi + lo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Dd {
    /// The leading part.
    pub(crate) hi: f64,
    /// The trailing part, |lo| ≤ ulp(hi) / 2.
    pub(crate) lo: f64,
}

/// s + e = a + b exactly (Knuth's two-sum, any magnitudes).
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    let e = (a - (s - bb)) + (b - bb);
    (s, e)
}

/// s + e = a + b exactly, given |a| ≥ |b| (or a = 0).
fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let e = b - (s - a);
    (s, e)
}

/// Veltkamp split: a = hi + lo with each half fitting in 26 bits.
/// Valid while |a| < 2^995 (callers keep values far smaller).
fn split(a: f64) -> (f64, f64) {
    const SPLITTER: f64 = 134_217_729.0; // 2^27 + 1
    let t = SPLITTER * a;
    let hi = t - (t - a);
    (hi, a - hi)
}

/// p + e = a × b exactly (Dekker's product), barring underflow.
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let (ah, al) = split(a);
    let (bh, bl) = split(b);
    let e = ((ah * bh - p) + ah * bl + al * bh) + al * bl;
    (p, e)
}

impl Dd {
    /// The double `x` as a double-double.
    pub(crate) fn from(x: f64) -> Dd {
        Dd { hi: x, lo: 0.0 }
    }

    /// From bit patterns, for constants.
    pub(crate) const fn from_bits(hi: u64, lo: u64) -> Dd {
        Dd {
            hi: f64::from_bits(hi),
            lo: f64::from_bits(lo),
        }
    }

    /// self + other.
    pub(crate) fn add(self, other: Dd) -> Dd {
        let (s, e) = two_sum(self.hi, other.hi);
        let (t, f) = two_sum(self.lo, other.lo);
        let (s, e) = quick_two_sum(s, e + t);
        let (hi, lo) = quick_two_sum(s, e + f);
        Dd { hi, lo }
    }

    /// −self.
    pub(crate) fn neg(self) -> Dd {
        Dd {
            hi: -self.hi,
            lo: -self.lo,
        }
    }

    /// self − other.
    pub(crate) fn sub(self, other: Dd) -> Dd {
        self.add(other.neg())
    }

    /// self × other.
    pub(crate) fn mul(self, other: Dd) -> Dd {
        let (p, e) = two_prod(self.hi, other.hi);
        let e = e + (self.hi * other.lo + self.lo * other.hi);
        let (hi, lo) = quick_two_sum(p, e);
        Dd { hi, lo }
    }

    /// self × b for a double b.
    pub(crate) fn mul_f64(self, b: f64) -> Dd {
        let (p, e) = two_prod(self.hi, b);
        let e = e + self.lo * b;
        let (hi, lo) = quick_two_sum(p, e);
        Dd { hi, lo }
    }

    /// self ÷ other (long division in three double-precision steps).
    pub(crate) fn div(self, other: Dd) -> Dd {
        let q1 = self.hi / other.hi;
        let r = self.sub(other.mul_f64(q1));
        let q2 = r.hi / other.hi;
        let r = r.sub(other.mul_f64(q2));
        let q3 = r.hi / other.hi;
        let (hi, lo) = quick_two_sum(q1, q2);
        Dd { hi, lo }.add(Dd::from(q3))
    }

    /// self × 2^k, exact while both parts stay normal (applied in steps
    /// so any k in the binary64 exponent range works).
    pub(crate) fn scale_pow2(self, k: i32) -> Dd {
        let mut out = self;
        let mut k = k;
        while k != 0 {
            let step = k.clamp(-1000, 1000);
            let f = pow2(step);
            out = Dd {
                hi: out.hi * f,
                lo: out.lo * f,
            };
            k -= step;
        }
        out
    }
}

/// 2^k for -1022 ≤ k ≤ 1023, built from its bit pattern.
pub(crate) fn pow2(k: i32) -> f64 {
    debug_assert!(
        (-1022..=1023).contains(&k),
        "invariant: normal power of two"
    );
    f64::from_bits(((i64::from(k) + 1023) as u64) << 52)
}
