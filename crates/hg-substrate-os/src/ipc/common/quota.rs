//! The broker's region ledger (M1 phase D6, review M4): each owner is
//! charged a region's full size at allocation, against a count and byte
//! quota per owner and a byte cap for the whole browser, until the owner
//! exits. No OS tells the broker when a region's last holder lets go
//! (facts L12, M9, S10), so the charge is the owner's for its life.

use std::collections::BTreeMap;

use hg_substrate::ipc::{
    MAX_REGION_BYTES_PER_OWNER, MAX_REGION_BYTES_TOTAL, MAX_REGIONS_PER_OWNER, OsPeer,
};

/// Whether one more region of `len` bytes fits an owner holding `count`
/// regions of `bytes` bytes, in a browser holding `total` bytes.
pub(crate) const fn fits(count: usize, bytes: u64, total: u64, len: u64) -> bool {
    if count >= MAX_REGIONS_PER_OWNER {
        return false;
    }
    match (bytes.checked_add(len), total.checked_add(len)) {
        (Some(b), Some(t)) => b <= MAX_REGION_BYTES_PER_OWNER && t <= MAX_REGION_BYTES_TOTAL,
        _ => false,
    }
}

/// The charges, by owner.
#[derive(Debug, Default)]
pub(crate) struct Ledger {
    owners: BTreeMap<OsPeer, (usize, u64)>,
    total: u64,
}

impl Ledger {
    /// Charges `owner` for `len` bytes, or refuses.
    pub(crate) fn charge(&mut self, owner: OsPeer, len: u64) -> bool {
        let (count, bytes) = self.owners.get(&owner).copied().unwrap_or((0, 0));
        if !fits(count, bytes, self.total, len) {
            return false;
        }
        // `fits` checked both sums.
        let (Some(c), Some(b), Some(t)) = (
            count.checked_add(1),
            bytes.checked_add(len),
            self.total.checked_add(len),
        ) else {
            return false;
        };
        self.owners.insert(owner, (c, b));
        self.total = t;
        true
    }

    /// Takes back one charge of `len` bytes that was not used (the
    /// region could not be made after it was charged).
    pub(crate) fn refund(&mut self, owner: OsPeer, len: u64) {
        let Some(&(count, bytes)) = self.owners.get(&owner) else {
            return;
        };
        if count <= 1 {
            self.owners.remove(&owner);
            self.total = self.total.saturating_sub(bytes);
        } else {
            self.owners
                .insert(owner, (count.saturating_sub(1), bytes.saturating_sub(len)));
            self.total = self.total.saturating_sub(len.min(bytes));
        }
    }

    /// Releases every charge of `owner`.
    pub(crate) fn release(&mut self, owner: OsPeer) {
        if let Some((_, bytes)) = self.owners.remove(&owner) {
            self.total = self.total.saturating_sub(bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_owner_is_capped_and_released() {
        let mut l = Ledger::default();
        let a = OsPeer::new(10, 0);
        for _ in 0..MAX_REGIONS_PER_OWNER {
            assert!(l.charge(a, 4096));
        }
        assert!(!l.charge(a, 4096), "count cap");
        assert!(l.charge(OsPeer::new(11, 0), 4096), "another owner");
        l.release(a);
        assert!(l.charge(a, 4096));
        assert!(!l.charge(OsPeer::new(12, 0), MAX_REGION_BYTES_PER_OWNER + 1));
    }

    #[test]
    fn a_refund_takes_back_exactly_one_charge() {
        let mut l = Ledger::default();
        let a = OsPeer::new(10, 0);
        assert!(l.charge(a, MAX_REGION_BYTES_PER_OWNER));
        assert!(!l.charge(a, 1));
        l.refund(a, MAX_REGION_BYTES_PER_OWNER);
        assert!(l.charge(a, MAX_REGION_BYTES_PER_OWNER));
        assert!(l.charge(OsPeer::new(11, 0), 4096));
        l.refund(OsPeer::new(11, 0), 4096);
        assert_eq!(l.total, MAX_REGION_BYTES_PER_OWNER);
    }
}

#[cfg(kani)]
mod proofs {
    use super::*;

    /// One more charge never takes an owner past its count or bytes, nor
    /// the browser past its cap (D6 review M4).
    #[kani::proof]
    fn fits_keeps_every_quota() {
        let (count, bytes, total, len): (usize, u64, u64, u64) =
            (kani::any(), kani::any(), kani::any(), kani::any());
        if fits(count, bytes, total, len) {
            assert!(count < MAX_REGIONS_PER_OWNER);
            assert!(bytes + len <= MAX_REGION_BYTES_PER_OWNER);
            assert!(total + len <= MAX_REGION_BYTES_TOTAL);
        }
        kani::cover!(fits(count, bytes, total, len) && len > 0);
    }
}
