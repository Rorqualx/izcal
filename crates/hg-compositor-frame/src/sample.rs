//! Random values of every generated type, for the round-trip and
//! mutation tests (CS-TE-10's seeded properties; the seed is printed by
//! the test that uses it).

use hg_substrate::RandomStream;
use hg_substrate::prng::Xoshiro256StarStar;

/// A seeded value source.
pub struct Gen {
    rng: Xoshiro256StarStar,
}

impl Gen {
    /// A source for `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Xoshiro256StarStar::from_seed(seed),
        }
    }

    /// The next 64 random bits.
    pub fn bits(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// A value in `0..n` (n > 0).
    pub fn below(&mut self, n: usize) -> usize {
        let n64 = u64::try_from(n).unwrap_or(u64::MAX).max(1);
        usize::try_from(self.bits().checked_rem(n64).unwrap_or(0)).unwrap_or(0)
    }
}

/// Something [`Gen`] can make.
pub trait Sample {
    /// A random value.
    fn sample(g: &mut Gen) -> Self;
}

macro_rules! from_bits {
    ($($t:ty),*) => {$(
        impl Sample for $t {
            fn sample(g: &mut Gen) -> Self {
                let b = g.bits().to_le_bytes();
                <$t>::from_le_bytes(b[..core::mem::size_of::<$t>()].try_into().unwrap())
            }
        }
    )*};
}

from_bits!(u8, u16, u32, u64, i32, i64);

impl Sample for bool {
    fn sample(g: &mut Gen) -> Self {
        g.bits() & 1 == 1
    }
}

impl Sample for f32 {
    fn sample(g: &mut Gen) -> Self {
        f32::from_bits(u32::sample(g))
    }
}

impl Sample for f64 {
    fn sample(g: &mut Gen) -> Self {
        f64::from_bits(g.bits())
    }
}

impl<T: Sample> Sample for Option<T> {
    fn sample(g: &mut Gen) -> Self {
        (g.below(2) == 1).then(|| T::sample(g))
    }
}

/// A length up to `max`, favoring the edges.
fn len(g: &mut Gen, max: usize) -> usize {
    match g.below(4) {
        0 => 0,
        1 => max,
        _ => g.below(max.min(8).saturating_add(1)),
    }
}

impl<const N: usize> Sample for hg_ipc::BoundedBytes<N> {
    fn sample(g: &mut Gen) -> Self {
        let n = len(g, N);
        Self::try_new((0..n).map(|_| u8::sample(g)).collect()).unwrap()
    }
}

impl<const N: usize> Sample for hg_ipc::BoundedString<N> {
    fn sample(g: &mut Gen) -> Self {
        const CHARS: [char; 6] = ['a', 'Z', '0', ' ', 'é', '漢'];
        let budget = len(g, N);
        let mut s = String::new();
        loop {
            let c = CHARS[g.below(CHARS.len())];
            if s.len().saturating_add(c.len_utf8()) > budget {
                break;
            }
            s.push(c);
        }
        Self::try_new(s).unwrap()
    }
}

impl<T: Sample, const N: usize> Sample for hg_ipc::BoundedVec<T, N> {
    fn sample(g: &mut Gen) -> Self {
        let n = len(g, N);
        Self::try_new((0..n).map(|_| T::sample(g)).collect()).unwrap()
    }
}
