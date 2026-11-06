//! A small arbitrary-precision unsigned integer: just the operations the
//! exact conversions need (multiply by a small factor, add, subtract,
//! shift, compare, and a quotient known to fit in 64 bits).
//!
//! Limbs are `u32`, least significant first, with no high zero limbs, so
//! zero is the empty vector and two equal values have equal limb vectors.

use std::cmp::Ordering;

/// An unsigned integer of any size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Big {
    limbs: Vec<u32>,
}

/// 5^13, the largest power of five that fits in a `u32`.
const FIVE_POW_13: u32 = 1_220_703_125;

impl Big {
    /// Zero.
    pub(crate) fn zero() -> Big {
        Big { limbs: Vec::new() }
    }

    /// The value `v`.
    pub(crate) fn from_u64(v: u64) -> Big {
        Big::from_u128(u128::from(v))
    }

    /// The value `v`.
    pub(crate) fn from_u128(mut v: u128) -> Big {
        let mut limbs = Vec::new();
        while v != 0 {
            limbs.push((v & 0xffff_ffff) as u32);
            v >>= 32;
        }
        Big { limbs }
    }

    /// Two to the power `n`.
    pub(crate) fn pow2(n: u64) -> Big {
        let mut b = Big::from_u64(1);
        b.shl(n);
        b
    }

    /// Whether the value is zero.
    pub(crate) fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    /// The number of significant bits (0 for zero).
    pub(crate) fn bit_len(&self) -> u64 {
        match self.limbs.last() {
            None => 0,
            Some(&top) => {
                (self.limbs.len() as u64 - 1) * 32 + (32 - u64::from(top.leading_zeros()))
            }
        }
    }

    fn trim(&mut self) {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
    }

    /// `self = self * m + a`.
    pub(crate) fn mul_add_small(&mut self, m: u32, a: u32) {
        let mut carry = u64::from(a);
        for limb in &mut self.limbs {
            let t = u64::from(*limb) * u64::from(m) + carry;
            *limb = (t & 0xffff_ffff) as u32;
            carry = t >> 32;
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
        self.trim();
    }

    /// `self *= m`.
    pub(crate) fn mul_small(&mut self, m: u32) {
        self.mul_add_small(m, 0);
    }

    /// `self *= 5^n`.
    pub(crate) fn mul_pow5(&mut self, mut n: u64) {
        while n >= 13 {
            self.mul_small(FIVE_POW_13);
            n -= 13;
        }
        let mut rest = 1u32;
        for _ in 0..n {
            rest *= 5;
        }
        self.mul_small(rest);
    }

    /// `self *= 10^n`.
    pub(crate) fn mul_pow10(&mut self, n: u64) {
        self.mul_pow5(n);
        self.shl(n);
    }

    /// `self += other`.
    pub(crate) fn add(&mut self, other: &Big) {
        if self.limbs.len() < other.limbs.len() {
            self.limbs.resize(other.limbs.len(), 0);
        }
        let mut carry = 0u64;
        for (i, limb) in self.limbs.iter_mut().enumerate() {
            let o = other.limbs.get(i).copied().unwrap_or(0);
            let t = u64::from(*limb) + u64::from(o) + carry;
            *limb = (t & 0xffff_ffff) as u32;
            carry = t >> 32;
            if carry == 0 && i >= other.limbs.len() {
                break;
            }
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
    }

    /// `self + other`, as a new value.
    pub(crate) fn plus(&self, other: &Big) -> Big {
        let mut t = self.clone();
        t.add(other);
        t
    }

    /// `self -= other`; the caller guarantees `self >= other`.
    pub(crate) fn sub(&mut self, other: &Big) {
        debug_assert!(*self >= *other, "invariant: Big::sub would go negative");
        let mut borrow = 0i64;
        for (i, limb) in self.limbs.iter_mut().enumerate() {
            let o = other.limbs.get(i).copied().unwrap_or(0);
            let mut t = i64::from(*limb) - i64::from(o) - borrow;
            if t < 0 {
                t += 1 << 32;
                borrow = 1;
            } else {
                borrow = 0;
            }
            *limb = t as u32;
            if borrow == 0 && i >= other.limbs.len() {
                break;
            }
        }
        self.trim();
    }

    /// `self <<= bits`.
    pub(crate) fn shl(&mut self, bits: u64) {
        if self.is_zero() || bits == 0 {
            return;
        }
        let limb_shift = (bits / 32) as usize;
        let bit_shift = (bits % 32) as u32;
        if bit_shift != 0 {
            let mut carry = 0u32;
            for limb in &mut self.limbs {
                let next = *limb >> (32 - bit_shift);
                *limb = (*limb << bit_shift) | carry;
                carry = next;
            }
            if carry != 0 {
                self.limbs.push(carry);
            }
        }
        if limb_shift != 0 {
            let mut v = vec![0u32; limb_shift];
            v.extend_from_slice(&self.limbs);
            self.limbs = v;
        }
    }

    /// `self >>= 1`.
    pub(crate) fn shr1(&mut self) {
        let mut carry = 0u32;
        for limb in self.limbs.iter_mut().rev() {
            let next = *limb & 1;
            *limb = (*limb >> 1) | (carry << 31);
            carry = next;
        }
        self.trim();
    }

    /// Divides `self` by `d` in place, leaving the remainder in `self`
    /// and returning the quotient. The caller guarantees the quotient is
    /// below 2^64 and `d` is not zero (binary long division).
    pub(crate) fn div_rem_small_quotient(&mut self, d: &Big) -> u64 {
        let (a_len, d_len) = (self.bit_len(), d.bit_len());
        if a_len < d_len {
            return 0;
        }
        let k = a_len - d_len;
        debug_assert!(k < 64, "invariant: quotient fits in 64 bits");
        let mut shifted = d.clone();
        shifted.shl(k);
        let mut q = 0u64;
        for i in (0..=k).rev() {
            if *self >= shifted {
                self.sub(&shifted);
                q |= 1u64 << i;
            }
            shifted.shr1();
        }
        q
    }
}

#[cfg(test)]
impl Big {
    /// `self /= d`, returning the remainder (tests only: decimal output).
    pub(crate) fn div_small(&mut self, d: u32) -> u32 {
        let mut rem = 0u64;
        for limb in self.limbs.iter_mut().rev() {
            let cur = (rem << 32) | u64::from(*limb);
            *limb = (cur / u64::from(d)) as u32;
            rem = cur % u64::from(d);
        }
        self.trim();
        rem as u32
    }

    /// The decimal digits of the value (tests only).
    pub(crate) fn to_decimal(&self) -> String {
        if self.is_zero() {
            return String::from("0");
        }
        let mut t = self.clone();
        let mut out = Vec::new();
        while !t.is_zero() {
            out.push(b'0' + t.div_small(10) as u8);
        }
        out.reverse();
        String::from_utf8(out).unwrap()
    }
}

impl Ord for Big {
    fn cmp(&self, other: &Big) -> Ordering {
        self.limbs
            .len()
            .cmp(&other.limbs.len())
            .then_with(|| self.limbs.iter().rev().cmp(other.limbs.iter().rev()))
    }
}

impl PartialOrd for Big {
    fn partial_cmp(&self, other: &Big) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
