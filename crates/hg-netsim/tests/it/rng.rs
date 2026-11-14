//! A deterministic stream for the tests.
//!
//! Written here rather than taken from `hg-substrate-sim`: this crate
//! is about being a function of its inputs, and a test that needed a
//! whole simulated world to make one number would be testing the
//! world. SplitMix64 is from the published algorithm (Steele, Lea and
//! Flood, 2014); it is not cryptographic and is not meant to be.

use hg_substrate::RandomStream;

pub struct SplitMix(pub u64);

impl RandomStream for SplitMix {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            let bytes = self.next_u64().to_le_bytes();
            let n = chunk.len().min(8);
            if let (Some(d), Some(s)) = (chunk.get_mut(..n), bytes.get(..n)) {
                d.copy_from_slice(s);
            }
        }
    }
}

pub fn stream(seed: u64) -> SplitMix {
    SplitMix(seed)
}
