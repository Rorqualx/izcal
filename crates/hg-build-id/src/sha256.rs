//! SHA-256, from FIPS 180-4 (Secure Hash Standard, August 2015):
//! Sections 4.1.2 (functions), 4.2.2 (constants), 5.1.1 (padding),
//! 5.3.3 (initial value) and 6.2.2 (computation).
//!
//! The constants are not typed in. They are computed at compile time
//! from their definition in the standard: the initial value is "the
//! first thirty-two bits of the fractional parts of the square roots of
//! the first eight prime numbers" (5.3.3) and the round constants "the
//! first thirty-two bits of the fractional parts of the cube roots of the
//! first sixty-four prime numbers" (4.2.2). For a prime `p`, those bits
//! are the integer root of `p * 2^64` (square) or `p * 2^96` (cube),
//! modulo 2^32.
//!
//! A build tool's hash, not a security primitive: `hg-crypto` owns
//! SHA-256 for the engine. The tests check NIST's published examples.

/// The first `N` primes.
const fn primes<const N: usize>() -> [u128; N] {
    let mut out = [0u128; N];
    let mut found = 0;
    let mut n = 2u128;
    while found < N {
        let mut d = 2u128;
        let mut prime = true;
        while d * d <= n {
            if n.is_multiple_of(d) {
                prime = false;
                break;
            }
            d += 1;
        }
        if prime {
            out[found] = n;
            found += 1;
        }
        n += 1;
    }
    out
}

/// The largest `r` with `r^k <= x`, for k = 2 or 3, by bisection. Every
/// `x` here is below 2^105 (the 64th prime is 311), so roots are below
/// 2^40 and `mid^3` stays below 2^120.
const fn iroot(x: u128, k: u32) -> u128 {
    let (mut lo, mut hi) = (0u128, 1u128 << 40);
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        let p = if k == 2 { mid * mid } else { mid * mid * mid };
        if p <= x {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}

const fn fraction_bits<const N: usize>(k: u32) -> [u32; N] {
    let ps = primes::<N>();
    let shift = if k == 2 { 64 } else { 96 };
    let mut out = [0u32; N];
    let mut i = 0;
    while i < N {
        out[i] = (iroot(ps[i] << shift, k) & 0xffff_ffff) as u32;
        i += 1;
    }
    out
}

/// H(0), 5.3.3.
pub const H0: [u32; 8] = fraction_bits::<8>(2);
/// K, 4.2.2.
pub const K: [u32; 64] = fraction_bits::<64>(3);

/// The SHA-256 digest of `data`.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    // 5.1.1: append 1, then zeros to 448 mod 512 bits, then the length
    // in bits as a 64-bit big-endian integer.
    let mut msg = data.to_vec();
    let bits = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bits.to_be_bytes());

    let mut h = H0;
    let (blocks, _) = msg.as_chunks::<64>();
    for block in blocks {
        // 6.2.2 step 1: the message schedule.
        let mut w = [0u32; 64];
        let (words, _) = block.as_chunks::<4>();
        for (t, word) in words.iter().enumerate() {
            w[t] = u32::from_be_bytes(*word);
        }
        for t in 16..64 {
            let s0 = w[t - 15].rotate_right(7) ^ w[t - 15].rotate_right(18) ^ (w[t - 15] >> 3);
            let s1 = w[t - 2].rotate_right(17) ^ w[t - 2].rotate_right(19) ^ (w[t - 2] >> 10);
            w[t] = s1
                .wrapping_add(w[t - 7])
                .wrapping_add(s0)
                .wrapping_add(w[t - 16]);
        }
        // Steps 2 to 4.
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for t in 0..64 {
            let big_s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(big_s1)
                .wrapping_add(ch)
                .wrapping_add(K[t])
                .wrapping_add(w[t]);
            let big_s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = big_s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(y);
        }
    }
    let mut out = [0u8; 32];
    let (chunks, _) = out.as_chunks_mut::<4>();
    for (chunk, word) in chunks.iter_mut().zip(h) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Lower-case hex.
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::{H0, K, hex, sha256};

    // NIST's SHA-256 examples (FIPS 180-4's example set: one-block "abc",
    // the empty message, the two-block 448-bit message, and one million
    // repetitions of "a").
    #[test]
    fn nist_examples() {
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(&sha256(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            hex(&sha256(&vec![b'a'; 1_000_000])),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn padding_boundaries_differ() {
        // Lengths around the 55/56/64-byte padding edges all hash, and
        // differently.
        let mut seen = std::collections::BTreeSet::new();
        for n in 50..=130 {
            assert!(seen.insert(sha256(&vec![0x61; n])));
        }
    }

    #[test]
    fn constants_have_the_standard_shape() {
        // 5.3.3's first word and 4.2.2's first and last, as the standard
        // prints them; the rest come from the same computation.
        assert_eq!(H0[0], 0x6a09_e667);
        assert_eq!(K[0], 0x428a_2f98);
        assert_eq!(K[63], 0xc671_78f2);
    }
}
