//! Hash maps keyed once per process (coding standard CS-AS-2).
//!
//! Every map in a process uses the same SipHash-2-4 key. The substrate
//! sets it once at startup: from OS randomness in production
//! (`hg-substrate-os`, keeping HashDoS resistance), from the seed in
//! simulation (`hg-substrate-sim`, keeping runs reproducible). A key per
//! map would make results depend on map-creation order, which differs
//! across fork/join workers.
//!
//! Before the key is set, hashing uses the all-zero key. That keeps unit
//! tests deterministic; a shipping binary sets the key before it creates
//! any map, which [`process_hash_key_is_set`] lets the startup path
//! check.
//!
//! SipHash-2-4 is implemented from its specification: J.-P. Aumasson and
//! D. J. Bernstein, "SipHash: a fast short-input PRF" (INDOCRYPT 2012).

use core::hash::{BuildHasher, Hasher};
use std::sync::OnceLock;

/// The process's hash key, set at most once.
static PROCESS_KEY: OnceLock<(u64, u64)> = OnceLock::new();

/// A std `HashMap` with the process-keyed hasher.
#[expect(clippy::disallowed_types, reason = "CS-AS-2: the alias definitions")]
pub type HashMap<K, V> = std::collections::HashMap<K, V, ProcessHasherBuilder>;

/// A std `HashSet` with the process-keyed hasher.
#[expect(clippy::disallowed_types, reason = "CS-AS-2: the alias definitions")]
pub type HashSet<T> = std::collections::HashSet<T, ProcessHasherBuilder>;

/// The key was already set; the first key stays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HashKeyAlreadySet;

impl core::fmt::Display for HashKeyAlreadySet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("the process hash key is already set")
    }
}

impl std::error::Error for HashKeyAlreadySet {}

/// Sets the process's hash key. Called once, by the substrate, before
/// the process creates any map.
///
/// # Errors
/// [`HashKeyAlreadySet`] if a key was set before; the first key stays.
pub fn set_process_hash_key(k0: u64, k1: u64) -> Result<(), HashKeyAlreadySet> {
    PROCESS_KEY
        .set((k0, k1))
        .map_err(|_already| HashKeyAlreadySet)
}

/// Whether the process's hash key has been set.
pub fn process_hash_key_is_set() -> bool {
    PROCESS_KEY.get().is_some()
}

/// Builds [`ProcessHasher`]s with the process's key.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessHasherBuilder;

impl BuildHasher for ProcessHasherBuilder {
    type Hasher = ProcessHasher;

    fn build_hasher(&self) -> ProcessHasher {
        let (k0, k1) = PROCESS_KEY.get().copied().unwrap_or((0, 0));
        ProcessHasher::with_keys(k0, k1)
    }
}

/// SipHash-2-4 over a stream of writes.
#[derive(Debug, Clone)]
pub struct ProcessHasher {
    v0: u64,
    v1: u64,
    v2: u64,
    v3: u64,
    /// Bytes not yet forming a full 8-byte word.
    buf: [u8; 8],
    /// How many bytes of `buf` are filled (0..8).
    nbuf: usize,
    /// The low byte of the total length written (the spec uses the length
    /// modulo 256).
    len_mod_256: u8,
}

impl ProcessHasher {
    /// A hasher with the 128-bit key `(k0, k1)`.
    pub const fn with_keys(k0: u64, k1: u64) -> Self {
        Self {
            v0: k0 ^ 0x736f_6d65_7073_6575,
            v1: k1 ^ 0x646f_7261_6e64_6f6d,
            v2: k0 ^ 0x6c79_6765_6e65_7261,
            v3: k1 ^ 0x7465_6462_7974_6573,
            buf: [0; 8],
            nbuf: 0,
            len_mod_256: 0,
        }
    }

    // cs-allow: CS-TY-12 — SipHash is defined over arithmetic modulo 2^64,
    // so its additions wrap by specification.
    fn round(&mut self) {
        self.v0 = self.v0.wrapping_add(self.v1);
        self.v1 = self.v1.rotate_left(13) ^ self.v0;
        self.v0 = self.v0.rotate_left(32);
        self.v2 = self.v2.wrapping_add(self.v3);
        self.v3 = self.v3.rotate_left(16) ^ self.v2;
        self.v0 = self.v0.wrapping_add(self.v3);
        self.v3 = self.v3.rotate_left(21) ^ self.v0;
        self.v2 = self.v2.wrapping_add(self.v1);
        self.v1 = self.v1.rotate_left(17) ^ self.v2;
        self.v2 = self.v2.rotate_left(32);
    }

    /// One compression step on a full 8-byte word (two rounds: the "2"
    /// of SipHash-2-4).
    fn compress(&mut self, word: [u8; 8]) {
        let m = u64::from_le_bytes(word);
        self.v3 ^= m;
        self.round();
        self.round();
        self.v0 ^= m;
    }

    /// Appends one byte to the partial word, compressing it when full.
    fn push_byte(&mut self, b: u8) {
        if let Some(slot) = self.buf.get_mut(self.nbuf) {
            *slot = b;
        }
        // nbuf < 8 here, so the increment cannot overflow.
        self.nbuf = self.nbuf.saturating_add(1);
        if self.nbuf == 8 {
            let word = self.buf;
            self.compress(word);
            self.buf = [0; 8];
            self.nbuf = 0;
        }
    }
}

impl Hasher for ProcessHasher {
    fn write(&mut self, bytes: &[u8]) {
        // The spec takes the total length modulo 256; the low byte of a
        // sum is the sum of the low bytes, modulo 256.
        // cs-allow: CS-TY-12 — modulo-256 arithmetic by specification.
        self.len_mod_256 = self.len_mod_256.wrapping_add(bytes.len().to_le_bytes()[0]);
        let mut rest = bytes;
        // Finish a partial word first, a byte at a time.
        while self.nbuf != 0 {
            let Some((&b, more)) = rest.split_first() else {
                return;
            };
            self.push_byte(b);
            rest = more;
        }
        let (words, tail) = rest.as_chunks::<8>();
        for &word in words {
            self.compress(word);
        }
        for &b in tail {
            self.push_byte(b);
        }
    }

    fn finish(&self) -> u64 {
        let mut s = self.clone();
        let mut last = s.buf;
        last[7] = s.len_mod_256;
        s.compress(last);
        s.v2 ^= 0xff;
        // Four finalization rounds: the "4" of SipHash-2-4.
        s.round();
        s.round();
        s.round();
        s.round();
        s.v0 ^ s.v1 ^ s.v2 ^ s.v3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The SipHash paper's test vector (its Appendix A): key 00 01 .. 0f,
    /// message 00 01 .. 0e (15 bytes), output 0xa129ca6149be45e5.
    #[test]
    fn paper_test_vector() {
        let k0 = u64::from_le_bytes([0, 1, 2, 3, 4, 5, 6, 7]);
        let k1 = u64::from_le_bytes([8, 9, 10, 11, 12, 13, 14, 15]);
        let msg: Vec<u8> = (0u8..15).collect();
        let mut h = ProcessHasher::with_keys(k0, k1);
        h.write(&msg);
        assert_eq!(h.finish(), 0xa129_ca61_49be_45e5);
    }

    /// Splitting the input across writes does not change the hash.
    #[test]
    fn split_writes_match_one_write() {
        let msg: Vec<u8> = (0u8..=40).collect();
        let mut one = ProcessHasher::with_keys(7, 9);
        one.write(&msg);
        for split in 0..msg.len() {
            let (a, b) = msg.split_at(split);
            let mut two = ProcessHasher::with_keys(7, 9);
            two.write(a);
            two.write(b);
            assert_eq!(two.finish(), one.finish(), "split at {split}");
        }
    }

    /// Different keys give different hashes for the same input.
    #[test]
    fn key_changes_hash() {
        let mut a = ProcessHasher::with_keys(1, 2);
        let mut b = ProcessHasher::with_keys(3, 4);
        a.write(b"izcal");
        b.write(b"izcal");
        assert_ne!(a.finish(), b.finish());
    }

    /// The aliases work as ordinary maps.
    #[test]
    fn aliases_are_usable() {
        let mut m: HashMap<&str, u32> = HashMap::default();
        m.insert("a", 1);
        let mut s: HashSet<u32> = HashSet::default();
        s.insert(5);
        assert_eq!(m.get("a"), Some(&1));
        assert!(s.contains(&5));
    }
}
