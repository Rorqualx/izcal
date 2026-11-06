//! The 32-bit heap slot (JS spec 6.1, JS-D1 (a)). The low bit tags it:
//! a **small integer** has tag 0 and carries 31 bits; a **reference** has
//! tag 1 and carries a 31-bit offset in units of 8 bytes, so a cage spans
//! 16 GiB. The stored value is `v = (offset / 8) << 1 | 1`, so the
//! address is `base + 4·v − 4`.

/// The largest cage a slot can address: 2^31 units of 8 bytes, 16 GiB.
pub const MAX_CAGE_BYTES: u64 = 1 << 34;

/// One 32-bit heap slot: a small integer or a reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slot(u32);

impl Slot {
    /// The small integer 0. It is also what an empty reference field holds.
    pub const ZERO: Self = Self(0);

    /// The slot holding the small integer `i`, if it fits in 31 bits.
    pub const fn small(i: i32) -> Option<Self> {
        if i < -(1 << 30) || i >= (1 << 30) {
            return None;
        }
        // In range, so the shift drops only the sign copy in bit 31.
        Some(Self((i as u32) << 1))
    }

    /// The small integer this slot holds, if it holds one.
    pub const fn as_small(self) -> Option<i32> {
        if self.0 & 1 != 0 {
            return None;
        }
        // Arithmetic shift restores the sign.
        Some((self.0 as i32) >> 1)
    }

    /// The reference to cage byte offset `offset`: a multiple of 8 below
    /// [`MAX_CAGE_BYTES`].
    pub(crate) const fn reference(offset: u64) -> Option<Self> {
        if !offset.is_multiple_of(8) || offset >= MAX_CAGE_BYTES {
            return None;
        }
        // offset / 8 < 2^31, so the shifted value fits in 32 bits.
        Some(Self((((offset / 8) as u32) << 1) | 1))
    }

    /// The cage byte offset this slot references, if it is a reference.
    /// Every result is a multiple of 8 below [`MAX_CAGE_BYTES`], whatever
    /// the slot holds (H-2: a slot read from the cage is untrusted).
    pub(crate) const fn as_reference(self) -> Option<u64> {
        if self.0 & 1 == 0 {
            return None;
        }
        // 4·v − 4 = 8 · (v >> 1).
        Some(((self.0 >> 1) as u64) * 8)
    }

    /// Whether the slot holds a reference.
    pub const fn is_reference(self) -> bool {
        self.0 & 1 != 0
    }

    /// The slot's raw 32 bits.
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// The slot with raw bits `bits`, as read from the cage.
    pub const fn from_raw(bits: u32) -> Self {
        Self(bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_integers_round_trip_at_the_edges() {
        for i in [0, 1, -1, (1 << 30) - 1, -(1 << 30)] {
            let s = Slot::small(i).unwrap();
            assert!(!s.is_reference());
            assert_eq!(s.as_small(), Some(i));
            assert_eq!(s.as_reference(), None);
        }
        assert_eq!(Slot::small(1 << 30), None);
        assert_eq!(Slot::small(-(1 << 30) - 1), None);
    }

    #[test]
    fn references_round_trip_and_decode_to_address_offsets() {
        for off in [0, 8, 4096, MAX_CAGE_BYTES - 8] {
            let s = Slot::reference(off).unwrap();
            assert!(s.is_reference());
            assert_eq!(s.as_reference(), Some(off));
            // The spec's formula: address = base + 4·v − 4.
            assert_eq!(4 * u64::from(s.raw()) - 4, off);
        }
        assert_eq!(Slot::reference(4), None);
        assert_eq!(Slot::reference(MAX_CAGE_BYTES), None);
        assert_eq!(
            Slot::from_raw(u32::MAX).as_reference(),
            Some(MAX_CAGE_BYTES - 8)
        );
    }
}

#[cfg(kani)]
mod proofs {
    use super::*;

    /// Every slot value, as an attacker might write it, decodes to an
    /// offset inside the largest cage, aligned to 8.
    #[kani::proof]
    fn any_slot_decodes_inside_the_cage() {
        let s = Slot::from_raw(kani::any());
        if let Some(off) = s.as_reference() {
            assert!(off < MAX_CAGE_BYTES);
            assert!(off % 8 == 0);
        }
    }

    /// Encoding then decoding is the identity, for references and small
    /// integers.
    #[kani::proof]
    fn encode_decode_round_trips() {
        let off: u64 = kani::any();
        if let Some(s) = Slot::reference(off) {
            assert!(s.as_reference() == Some(off));
            assert!(s.as_small().is_none());
        }
        let i: i32 = kani::any();
        if let Some(s) = Slot::small(i) {
            assert!(s.as_small() == Some(i));
            assert!(s.as_reference().is_none());
        }
    }
}
