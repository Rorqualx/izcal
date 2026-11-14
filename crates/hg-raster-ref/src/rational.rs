//! An exact rational, in `i128`.
//!
//! Small and total: every operation is checked and returns `None`
//! rather than wrapping, because an oracle that wrapped would agree
//! with the thing it is checking for the wrong reason. The numbers
//! involved are a pixel's worth of area over denominators built from
//! a few edge slopes, so the range is comfortable — but "comfortable"
//! is not a proof, and the `None` is what says so.

use core::cmp::Ordering;

/// An exact rational `n / d`, always reduced, with `d > 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    n: i128,
    d: i128,
}

impl Rational {
    /// Zero.
    pub const ZERO: Self = Self { n: 0, d: 1 };
    /// One.
    pub const ONE: Self = Self { n: 1, d: 1 };

    /// `n / d`, reduced. `None` when `d` is zero.
    #[must_use]
    pub fn new(n: i128, d: i128) -> Option<Self> {
        if d == 0 {
            return None;
        }
        let (n, d) = if d < 0 {
            (n.checked_neg()?, d.checked_neg()?)
        } else {
            (n, d)
        };
        let g = gcd(n.unsigned_abs(), d.unsigned_abs());
        let g = i128::try_from(g.max(1)).ok()?;
        Some(Self {
            n: n.checked_div(g)?,
            d: d.checked_div(g)?,
        })
    }

    /// An integer.
    #[must_use]
    pub const fn whole(n: i128) -> Self {
        Self { n, d: 1 }
    }

    /// Its numerator.
    #[must_use]
    pub const fn numerator(self) -> i128 {
        self.n
    }

    /// Its denominator, always positive.
    #[must_use]
    pub const fn denominator(self) -> i128 {
        self.d
    }

    /// `self + other`.
    #[must_use]
    pub fn plus(self, other: Self) -> Option<Self> {
        let n = self
            .n
            .checked_mul(other.d)?
            .checked_add(other.n.checked_mul(self.d)?)?;
        Self::new(n, self.d.checked_mul(other.d)?)
    }

    /// `self - other`.
    #[must_use]
    pub fn minus(self, other: Self) -> Option<Self> {
        self.plus(Self {
            n: other.n.checked_neg()?,
            d: other.d,
        })
    }

    /// `self * other`.
    #[must_use]
    pub fn times(self, other: Self) -> Option<Self> {
        Self::new(self.n.checked_mul(other.n)?, self.d.checked_mul(other.d)?)
    }

    /// `self / other`.
    #[must_use]
    pub fn over(self, other: Self) -> Option<Self> {
        if other.n == 0 {
            return None;
        }
        Self::new(self.n.checked_mul(other.d)?, self.d.checked_mul(other.n)?)
    }

    /// `|self|`.
    #[must_use]
    pub fn abs(self) -> Option<Self> {
        Some(Self {
            n: self.n.checked_abs()?,
            d: self.d,
        })
    }

    /// `round(self * 255)`, clamped to 0..=255: the one rounding a
    /// coverage carries.
    #[must_use]
    pub fn to_alpha(self) -> Option<u8> {
        let scaled = self.n.checked_mul(255)?;
        let half = self.d.checked_div(2)?;
        let q = if scaled >= 0 {
            scaled.checked_add(half)?.checked_div(self.d)?
        } else {
            scaled.checked_sub(half)?.checked_div(self.d)?
        };
        u8::try_from(q.clamp(0, 255)).ok()
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        // Denominators are positive, so cross-multiplication keeps the
        // order. Saturating rather than checked because an ordering
        // has no way to say "I do not know", and the inputs here are
        // far from the range where it could matter.
        let a = self.n.saturating_mul(other.d);
        let b = other.n.saturating_mul(self.d);
        a.cmp(&b)
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a.checked_rem(b).unwrap_or(0);
        a = b;
        b = t;
    }
    a
}
