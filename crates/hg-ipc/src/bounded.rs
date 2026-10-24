//! Bounded containers: the only variable-length types on the wire
//! (IPC-R4: explicit maximum lengths; CS-TY-3: constructors validate;
//! CS-ER-9: fallible allocation sized by the peer).
//!
//! Each type's `N` is a maximum in bytes (`BoundedBytes`, `BoundedString`)
//! or elements (`BoundedVec`). A value over `N` cannot be built, so it
//! cannot be sent; one over `N` on the wire is a bad message.

use crate::wire::{BadMessage, Reader, Wire, Writer, len_add, len_mul, put_count};

/// A value was longer than its type's maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverBound {
    /// The value's length.
    pub len: usize,
    /// The type's maximum.
    pub max: usize,
}

impl core::fmt::Display for OverBound {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "length {} is over the maximum {}", self.len, self.max)
    }
}

impl std::error::Error for OverBound {}

const fn check(len: usize, max: usize) -> Result<(), OverBound> {
    if len > max {
        Err(OverBound { len, max })
    } else {
        Ok(())
    }
}

fn reserve<T>(n: usize) -> Result<Vec<T>, BadMessage> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_alloc| BadMessage::OutOfMemory)?;
    Ok(v)
}

/// At most `N` bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BoundedBytes<const N: usize>(Vec<u8>);

impl<const N: usize> BoundedBytes<N> {
    /// `bytes`, if at most `N` long.
    ///
    /// # Errors
    /// [`OverBound`] if longer.
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, OverBound> {
        check(bytes.len(), N)?;
        Ok(Self(bytes))
    }

    /// The bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// The bytes, owned.
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl<const N: usize> Wire for BoundedBytes<N> {
    const MIN_LEN: usize = 4;
    const MAX_LEN: usize = len_add(4, N);
    fn encode(&self, w: &mut Writer) {
        put_count(w, self.0.len());
        w.put(&self.0);
    }
    fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {
        let n = r.count(N, 1)?;
        let src = r.take(n)?;
        let mut v = reserve(n)?;
        v.extend_from_slice(src);
        Ok(Self(v))
    }
    fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {
        let n = r.count(N, 1)?;
        r.take(n).map(|_bytes| ())
    }
}

/// At most `N` bytes of UTF-8. Data only: privileged code never parses
/// one as a URL, path or expression (IPC-R3); those have their own types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BoundedString<const N: usize>(String);

impl<const N: usize> BoundedString<N> {
    /// `s`, if at most `N` bytes long.
    ///
    /// # Errors
    /// [`OverBound`] if longer.
    pub fn try_new(s: String) -> Result<Self, OverBound> {
        check(s.len(), N)?;
        Ok(Self(s))
    }

    /// The text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The text, owned.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<const N: usize> Wire for BoundedString<N> {
    const MIN_LEN: usize = 4;
    const MAX_LEN: usize = len_add(4, N);
    fn encode(&self, w: &mut Writer) {
        put_count(w, self.0.len());
        w.put(self.0.as_bytes());
    }
    fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {
        let n = r.count(N, 1)?;
        Ok(Self(crate::utf8::to_string::<N>(r.take(n)?)?))
    }
    fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {
        let n = r.count(N, 1)?;
        crate::utf8::validate::<N>(r.take(n)?)
    }
}

/// At most `N` elements.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoundedVec<T, const N: usize>(Vec<T>);

impl<T, const N: usize> Default for BoundedVec<T, N> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T, const N: usize> BoundedVec<T, N> {
    /// `items`, if at most `N` of them.
    ///
    /// # Errors
    /// [`OverBound`] if more.
    pub fn try_new(items: Vec<T>) -> Result<Self, OverBound> {
        check(items.len(), N)?;
        Ok(Self(items))
    }

    /// The elements.
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// The elements, owned.
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T: Wire, const N: usize> Wire for BoundedVec<T, N> {
    const MIN_LEN: usize = 4;
    const MAX_LEN: usize = len_add(4, len_mul(N, T::MAX_LEN));
    // Both loops are bounded by the constant `N`, not by the length, so
    // Kani unrolls each to the type's maximum (VER-1 proof cost; a loop
    // bounded only by a decoded count is unrolled to the harness limit).
    fn encode(&self, w: &mut Writer) {
        put_count(w, self.0.len());
        for i in 0..N {
            let Some(item) = self.0.get(i) else { break };
            item.encode(w);
        }
    }
    fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {
        let n = r.count(N, T::MIN_LEN)?;
        let mut v = reserve(n)?;
        for _ in 0..N {
            if v.len() >= n {
                break;
            }
            v.push(T::decode(r)?);
        }
        Ok(Self(v))
    }
    fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {
        let n = r.count(N, T::MIN_LEN)?;
        for i in 0..N {
            if i >= n {
                break;
            }
            T::validate(r)?;
        }
        Ok(())
    }
}

/// VER-1 for the containers (Stage 1 4.1; coding standard CS-ER-3,
/// CS-ER-9). Bounds are stated per harness.
///
/// `Option<T>` and `BoundedVec<T, N>` reach `T` only through [`Wire`],
/// so their validator proofs use [`Opaque`], an element that may accept
/// any allowed number of bytes or fail at any point: every element type
/// behaves like one of its cases. The round trips are proved for leaf
/// types and for a vector of scalars; a vector of heap-holding elements
/// is out of the solver's reach (copies out of nested heap memory), so
/// its round trip is covered by the generated property tests, and its
/// parsing by the validator proofs.
#[cfg(kani)]
mod proofs {
    use super::*;
    use crate::wire::{decode_exact, encode_to_vec, validate_exact};

    /// Any element: 1 to `M` bytes, or a failure.
    #[derive(Debug)]
    struct Opaque<const M: usize> {
        bytes: [u8; M],
        len: usize,
    }

    impl<const M: usize> Wire for Opaque<M> {
        const MIN_LEN: usize = 1;
        const MAX_LEN: usize = M;
        fn encode(&self, w: &mut Writer) {
            if let Some(b) = self.bytes.get(..self.len) {
                w.put(b);
            }
        }
        fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {
            if kani::any() {
                return Err(BadMessage::BadBool);
            }
            let len: usize = kani::any_where(|&n: &usize| (1..=M).contains(&n));
            let src = r.take(len)?;
            let mut bytes = [0u8; M];
            if let Some(dst) = bytes.get_mut(..len) {
                dst.copy_from_slice(src);
            }
            Ok(Self { bytes, len })
        }
        fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {
            if kani::any() {
                return Err(BadMessage::BadBool);
            }
            let len: usize = kani::any_where(|&n: &usize| (1..=M).contains(&n));
            r.take(len).map(|_bytes| ())
        }
    }

    /// Decoding any input of at most `K` bytes never panics, and an
    /// accepted one re-encodes to the same bytes.
    fn round_trip<T: Wire, const K: usize>() {
        let bytes: [u8; K] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= K);
        let Some(input) = bytes.get(..len) else {
            return;
        };
        let decoded = decode_exact::<T>(input);
        kani::cover!(decoded.is_ok(), "some input is accepted");
        if let Ok(v) = decoded {
            let out = encode_to_vec(&v);
            assert!(out.len() == len);
            for (a, b) in out.iter().zip(bytes.iter()) {
                assert!(a == b);
            }
        }
    }

    /// Validating any input of at most `K` bytes never panics.
    fn validates<T: Wire, const K: usize>() {
        let bytes: [u8; K] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= K);
        let Some(input) = bytes.get(..len) else {
            return;
        };
        let checked = validate_exact::<T>(input);
        kani::cover!(checked.is_ok(), "some input is accepted");
    }

    /// Bounds: 0 to 20 bytes.
    #[kani::proof]
    #[kani::unwind(22)]
    fn bytes_round_trip() {
        round_trip::<BoundedBytes<16>, 20>();
    }

    /// Bounds: 0 to 16 bytes.
    #[kani::proof]
    #[kani::unwind(18)]
    fn string_round_trip() {
        round_trip::<BoundedString<12>, 16>();
    }

    /// Bounds: 0 to 16 bytes, 3 elements.
    #[kani::proof]
    #[kani::unwind(18)]
    fn vec_of_scalars_round_trip() {
        round_trip::<BoundedVec<u32, 3>, 16>();
    }

    /// Bounds: 0 to 5 bytes.
    #[kani::proof]
    #[kani::unwind(7)]
    fn option_of_any_element_round_trip() {
        round_trip::<Option<Opaque<4>>, 5>();
    }

    /// Bounds: 0 to 64 bytes, 8 elements of 1 to 8 bytes.
    #[kani::proof]
    #[kani::unwind(10)]
    fn vec_of_any_element_validates() {
        validates::<BoundedVec<Opaque<8>, 8>, 64>();
    }

    /// Bounds: 0 to 64 bytes.
    #[kani::proof]
    #[kani::unwind(66)]
    fn string_validates() {
        validates::<BoundedString<60>, 64>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{decode_exact, encode_to_vec};

    #[test]
    fn bounds_hold_at_construction_and_on_the_wire() {
        assert!(BoundedBytes::<3>::try_new(vec![1, 2, 3]).is_ok());
        assert_eq!(
            BoundedBytes::<3>::try_new(vec![0; 4]),
            Err(OverBound { len: 4, max: 3 })
        );
        let four = encode_to_vec(&BoundedBytes::<4>::try_new(vec![0; 4]).unwrap());
        assert_eq!(
            decode_exact::<BoundedBytes<3>>(&four),
            Err(BadMessage::OverBound)
        );

        let s = BoundedString::<8>::try_new("héllo".into()).unwrap();
        let bytes = encode_to_vec(&s);
        assert_eq!(decode_exact::<BoundedString<8>>(&bytes), Ok(s));
        assert_eq!(
            decode_exact::<BoundedString<8>>(&[1, 0, 0, 0, 0xff]),
            Err(BadMessage::BadUtf8)
        );

        let v = BoundedVec::<u16, 4>::try_new(vec![1, 2, 3]).unwrap();
        assert_eq!(encode_to_vec(&v), [3, 0, 0, 0, 1, 0, 2, 0, 3, 0]);
        assert_eq!(decode_exact(&encode_to_vec(&v)), Ok(v));
        // A huge claimed count with no bytes behind it never allocates.
        assert_eq!(
            decode_exact::<BoundedVec<u64, 1_000_000>>(&[0x40, 0x42, 0x0f, 0]),
            Err(BadMessage::Truncated)
        );
    }

    #[test]
    fn size_bounds_compose() {
        assert_eq!(
            <BoundedVec<BoundedString<10>, 3> as Wire>::MAX_LEN,
            4 + 3 * 14
        );
        assert_eq!(<Option<u64> as Wire>::MAX_LEN, 9);
    }
}
