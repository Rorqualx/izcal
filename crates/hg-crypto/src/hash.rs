//! SHA-256 (FIPS 180-4), and the hash interface over it.
//!
//! Izcal's own code, written from the published standard. Nothing here
//! is taken from any implementation (M-1 part 3): the constants are
//! the ones the standard prints — the first thirty-two bits of the
//! fractional parts of the square roots and cube roots of the first
//! primes — and the four functions are its §4.1.2 formulas.
//!
//! # Why this one, and why now
//!
//! TLS 1.3's whole key schedule is HKDF over one hash (RFC 8446 §7.1),
//! and HKDF is HMAC, and HMAC is this. So the floor of `hg-tls` is a
//! hash, and a pin row for one would have left the Wycheproof runner
//! with nothing to run at NW-M0.
//!
//! # Tier P, without an exception
//!
//! Tier P denies indexing and unchecked arithmetic. A hash is written
//! in index notation everywhere it is published, so the usual answer
//! is a module-level `expect` saying the indices are loop counters.
//! This module does not take it: the message schedule is computed over
//! a sliding `get(..)` window destructured by pattern, the rounds are
//! a `zip` of the constants with the schedule, and the state is
//! destructured rather than indexed. The result is the same arithmetic
//! and no claim to audit.
//!
//! Every addition in the compression function is `wrapping_add`
//! **because the standard says addition is modulo 2^32** (§2.2.2), not
//! as a way around the lint.

use core::fmt;

/// A hash function, as the rest of Izcal uses one.
///
/// `OUT` is its output length in bytes and `BLOCK` its block length,
/// which HMAC needs (RFC 2104 keys longer than a block are hashed).
pub trait Hash: Default {
    /// Output length in bytes.
    const OUT: usize;
    /// Block length in bytes.
    const BLOCK: usize;
    /// The output type.
    type Output: AsRef<[u8]> + Copy;

    /// Adds `data` to what is being hashed.
    fn update(&mut self, data: &[u8]);
    /// Finishes, consuming the state.
    fn finish(self) -> Self::Output;

    /// The hash of `data`, for the one-shot case.
    #[must_use]
    fn digest(data: &[u8]) -> Self::Output {
        let mut h = Self::default();
        h.update(data);
        h.finish()
    }
}

/// FIPS 180-4 §4.2.2: the first thirty-two bits of the fractional
/// parts of the cube roots of the first sixty-four primes.
const K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// FIPS 180-4 §5.3.3: the first thirty-two bits of the fractional
/// parts of the square roots of the first eight primes.
const INIT: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

const BLOCK_LEN: usize = 64;
const OUT_LEN: usize = 32;

/// A SHA-256 digest: thirty-two bytes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Digest(pub [u8; OUT_LEN]);

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for Digest {
    /// Lower-case hex, because a digest is public and a reader needs
    /// to compare it with a published one.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

/// SHA-256 (FIPS 180-4).
#[derive(Clone)]
pub struct Sha256 {
    state: [u32; 8],
    buffer: [u8; BLOCK_LEN],
    buffered: usize,
    /// Message length in **bits**, as §5.1.1's padding needs it.
    /// Saturating: a message of 2^64 bits is not reachable, and
    /// wrapping here would make two messages hash alike.
    bits: u64,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self {
            state: INIT,
            buffer: [0; BLOCK_LEN],
            buffered: 0,
            bits: 0,
        }
    }
}

impl Sha256 {
    /// A fresh state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// §4.1.2's four functions, and §6.2.2's round.
    fn compress(state: &mut [u32; 8], block: &[u8; BLOCK_LEN]) {
        let mut w = [0_u32; 64];

        // The first sixteen words are the block, big-endian.
        let (words, _) = block.as_chunks::<4>();
        for (slot, bytes) in w.iter_mut().zip(words.iter()) {
            *slot = u32::from_be_bytes(*bytes);
        }

        // §6.2.2 step 1 for t = 16..64. The window is the sixteen
        // words before the one being written, destructured so that
        // w[t-16], w[t-15], w[t-7] and w[t-2] are names rather than
        // indices.
        for start in 0..48_usize {
            let at = start.saturating_add(16);
            let Some(window) = w.get(start..at) else {
                continue;
            };
            let Ok(prev) = <[u32; 16]>::try_from(window) else {
                continue;
            };
            let [m16, m15, _, _, _, _, _, _, _, m7, _, _, _, _, m2, _] = prev;
            let s0 = m15.rotate_right(7) ^ m15.rotate_right(18) ^ (m15 >> 3);
            let s1 = m2.rotate_right(17) ^ m2.rotate_right(19) ^ (m2 >> 10);
            let next = s1.wrapping_add(m7).wrapping_add(s0).wrapping_add(m16);
            if let Some(slot) = w.get_mut(at) {
                *slot = next;
            }
        }

        let [h0, h1, h2, h3, h4, h5, h6, h7] = *state;
        let (mut a, mut b, mut c, mut d) = (h0, h1, h2, h3);
        let (mut e, mut f, mut g, mut h) = (h4, h5, h6, h7);

        for (k, wt) in K.iter().zip(w.iter()) {
            let big_s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(big_s1)
                .wrapping_add(ch)
                .wrapping_add(*k)
                .wrapping_add(*wt);
            let big_s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = big_s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        *state = [
            h0.wrapping_add(a),
            h1.wrapping_add(b),
            h2.wrapping_add(c),
            h3.wrapping_add(d),
            h4.wrapping_add(e),
            h5.wrapping_add(f),
            h6.wrapping_add(g),
            h7.wrapping_add(h),
        ];
    }

    /// Fills the buffer from `data` and compresses whole blocks.
    fn absorb(&mut self, data: &[u8]) {
        let mut rest = data;
        while !rest.is_empty() {
            let room = BLOCK_LEN.saturating_sub(self.buffered);
            let take = room.min(rest.len());
            let Some((head, tail)) = rest.split_at_checked(take) else {
                return;
            };
            let end = self.buffered.saturating_add(take);
            if let Some(slot) = self.buffer.get_mut(self.buffered..end) {
                slot.copy_from_slice(head);
            }
            self.buffered = end;
            rest = tail;
            if self.buffered == BLOCK_LEN {
                let block = self.buffer;
                Self::compress(&mut self.state, &block);
                self.buffered = 0;
            }
        }
    }
}

impl Hash for Sha256 {
    const OUT: usize = OUT_LEN;
    const BLOCK: usize = BLOCK_LEN;
    type Output = Digest;

    fn update(&mut self, data: &[u8]) {
        let added = u64::try_from(data.len())
            .unwrap_or(u64::MAX)
            .saturating_mul(8);
        self.bits = self.bits.saturating_add(added);
        self.absorb(data);
    }

    fn finish(mut self) -> Digest {
        // §5.1.1: append 0x80, then zeros, then the length in bits as
        // a 64-bit big-endian integer, so that the total is a whole
        // number of blocks.
        let bits = self.bits;
        self.absorb(&[0x80]);
        // `bits` must not grow from the padding, so absorb directly
        // rather than through `update`.
        while self.buffered != BLOCK_LEN.saturating_sub(8) {
            self.absorb(&[0]);
        }
        self.absorb(&bits.to_be_bytes());

        let mut out = [0_u8; OUT_LEN];
        let (chunks, _) = out.as_chunks_mut::<4>();
        for (chunk, word) in chunks.iter_mut().zip(self.state.iter()) {
            *chunk = word.to_be_bytes();
        }
        Digest(out)
    }
}
