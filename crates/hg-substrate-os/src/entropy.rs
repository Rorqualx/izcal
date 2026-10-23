//! OS randomness (Stage 1 design 3.1; Q-CS-1). The cryptographic source
//! is the OS CSPRNG and nothing else: `getentropy` on macOS, the
//! `getrandom` system call on Linux, `BCryptGenRandom` with the system
//! preferred generator on Windows. Named streams are xoshiro256** seeded
//! from it (`hg_substrate::prng`).

use hg_substrate::prng::Xoshiro256StarStar;
use hg_substrate::{CryptoRandom, Entropy, EntropyError, EntropyPurpose, RandomStream};

use crate::os;

/// The OS cryptographic random source.
#[derive(Debug, Default)]
pub struct OsCrypto;

impl CryptoRandom for OsCrypto {
    fn fill(&self, dest: &mut [u8]) -> Result<(), EntropyError> {
        os::random::fill(dest).map_err(|_os_error| EntropyError::Unavailable)
    }
}

/// OS-seeded randomness.
#[derive(Debug, Default)]
pub struct OsEntropy {
    crypto: OsCrypto,
}

impl OsEntropy {
    /// The OS entropy source.
    pub const fn new() -> Self {
        Self { crypto: OsCrypto }
    }
}

/// A stream that could not be seeded: it yields nothing usable, so it
/// stops the process rather than return predictable numbers.
fn unseedable() -> ! {
    hg_types::invariant_failed("the OS random source failed while seeding a stream")
}

impl Entropy for OsEntropy {
    fn stream(&self, _purpose: EntropyPurpose) -> Box<dyn RandomStream> {
        // Each stream gets its own OS seed, so streams are independent
        // and the purpose needs no part in the seed.
        let mut seed = [0u8; 8];
        if self.crypto.fill(&mut seed).is_err() {
            unseedable();
        }
        Box::new(Xoshiro256StarStar::from_seed(u64::from_le_bytes(seed)))
    }

    fn crypto(&self) -> &dyn CryptoRandom {
        &self.crypto
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_fills_and_differs() {
        let e = OsEntropy::new();
        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        e.crypto().fill(&mut a).unwrap();
        e.crypto().fill(&mut b).unwrap();
        assert_ne!(a, b);
        assert_ne!(a, [0u8; 64]);
        // More than one OS call's worth (getentropy caps at 256 bytes).
        let mut big = vec![0u8; 1000];
        e.crypto().fill(&mut big).unwrap();
        assert!(big[900..].iter().any(|&x| x != 0));
    }

    #[test]
    fn streams_are_independent() {
        let e = OsEntropy::new();
        let mut a = e.stream(EntropyPurpose::Random);
        let mut b = e.stream(EntropyPurpose::Random);
        assert_ne!(a.next_u64(), b.next_u64());
    }
}
