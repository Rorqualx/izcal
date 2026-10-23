//! The stream generator shared by both substrates: xoshiro256**, seeded
//! through SplitMix64. Production seeds it from the OS's cryptographic
//! source; the simulator seeds it from the run's seed, so a simulated
//! stream is the same sequence on every OS and ISA.
//!
//! Written from the published descriptions: D. Blackman and S. Vigna,
//! "Scrambled linear pseudorandom number generators" (ACM TOMS, 2021),
//! for xoshiro256**; G. Steele, D. Lea and C. Flood, "Fast splittable
//! pseudorandom number generators" (OOPSLA 2014), for SplitMix64.
//! Not cryptographic: secrets come from `Entropy::crypto`.

use crate::RandomStream;

/// One SplitMix64 step: advances `state` and returns the next output.
/// Used to expand a 64-bit seed into xoshiro's 256-bit state.
// cs-allow: CS-TY-12 — both generators are defined over arithmetic
// modulo 2^64, so their additions and multiplications wrap by
// specification.
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// xoshiro256**: 256 bits of state, 64-bit outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Xoshiro256StarStar {
    s: [u64; 4],
}

impl Xoshiro256StarStar {
    /// A generator whose state is four SplitMix64 outputs from `seed`
    /// (the seeding the paper recommends; never the all-zero state).
    pub fn from_seed(seed: u64) -> Self {
        let mut sm = seed;
        Self {
            s: [
                splitmix64(&mut sm),
                splitmix64(&mut sm),
                splitmix64(&mut sm),
                splitmix64(&mut sm),
            ],
        }
    }
}

impl RandomStream for Xoshiro256StarStar {
    // cs-allow: CS-TY-12 — modulo-2^64 arithmetic by specification.
    fn next_u64(&mut self) -> u64 {
        let [s0, s1, s2, s3] = &mut self.s;
        let result = s1.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = *s1 << 17;
        *s2 ^= *s0;
        *s3 ^= *s1;
        *s1 ^= *s2;
        *s0 ^= *s3;
        *s2 ^= t;
        *s3 = s3.rotate_left(45);
        result
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let (words, tail) = dest.as_chunks_mut::<8>();
        for w in words {
            *w = self.next_u64().to_le_bytes();
        }
        let last = self.next_u64().to_le_bytes();
        for (d, s) in tail.iter_mut().zip(last) {
            *d = s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SplitMix64's first output from state 0, a widely published value.
    #[test]
    fn splitmix64_from_zero() {
        let mut s = 0;
        assert_eq!(splitmix64(&mut s), 0xe220_a839_7b1d_cdaf);
    }

    #[test]
    fn same_seed_same_stream_different_seed_different_stream() {
        let mut a = Xoshiro256StarStar::from_seed(42);
        let mut b = Xoshiro256StarStar::from_seed(42);
        let mut c = Xoshiro256StarStar::from_seed(43);
        let va: Vec<u64> = (0..16).map(|_| a.next_u64()).collect();
        let vb: Vec<u64> = (0..16).map(|_| b.next_u64()).collect();
        let vc: Vec<u64> = (0..16).map(|_| c.next_u64()).collect();
        assert_eq!(va, vb);
        assert_ne!(va, vc);
    }

    #[test]
    fn fill_bytes_covers_partial_words() {
        let mut g = Xoshiro256StarStar::from_seed(7);
        let mut buf = [0u8; 13];
        g.fill_bytes(&mut buf);
        assert!(buf.iter().any(|&b| b != 0));
    }
}
