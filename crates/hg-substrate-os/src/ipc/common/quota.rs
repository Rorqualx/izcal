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

/// One owner's charges: (regions, bytes); `None` when it has none.
type Entry = Option<(usize, u64)>;

/// A change to one owner's charges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    /// One more region of this many bytes.
    Charge(u64),
    /// One charge of this many bytes taken back.
    Refund(u64),
    /// Every charge taken back.
    Release,
}

/// What `op` does to one owner's entry `e` and the browser's `total`:
/// the new entry and total, or `None` for a refused charge (nothing
/// changes). Only this owner's bytes move the total, so the total stays
/// the sum of the owners' bytes (proved below).
const fn step(op: Op, e: Entry, total: u64) -> Option<(Entry, u64)> {
    let (count, bytes) = match e {
        Some(cb) => cb,
        None => (0, 0),
    };
    match op {
        Op::Charge(len) => {
            if !fits(count, bytes, total, len) {
                return None;
            }
            // `fits` checked both sums.
            match (
                count.checked_add(1),
                bytes.checked_add(len),
                total.checked_add(len),
            ) {
                (Some(c), Some(b), Some(t)) => Some((Some((c, b)), t)),
                _ => None,
            }
        }
        Op::Refund(len) => {
            if e.is_none() {
                return Some((None, total));
            }
            if count <= 1 {
                Some((None, total.saturating_sub(bytes)))
            } else {
                let len = if len < bytes { len } else { bytes };
                // count > 1 and len <= bytes: neither saturates.
                Some((
                    Some((count.saturating_sub(1), bytes.saturating_sub(len))),
                    total.saturating_sub(len),
                ))
            }
        }
        Op::Release => Some((None, total.saturating_sub(bytes))),
    }
}

/// The charges, by owner.
#[derive(Debug, Default)]
pub(crate) struct Ledger {
    owners: BTreeMap<OsPeer, (usize, u64)>,
    total: u64,
}

impl Ledger {
    fn apply(&mut self, owner: OsPeer, op: Op) -> bool {
        let Some((e, total)) = step(op, self.owners.get(&owner).copied(), self.total) else {
            return false;
        };
        match e {
            Some(cb) => self.owners.insert(owner, cb),
            None => self.owners.remove(&owner),
        };
        self.total = total;
        true
    }

    /// Charges `owner` for `len` bytes, or refuses.
    pub(crate) fn charge(&mut self, owner: OsPeer, len: u64) -> bool {
        self.apply(owner, Op::Charge(len))
    }

    /// Takes back one charge of `len` bytes that was not used (the
    /// region could not be made after it was charged).
    pub(crate) fn refund(&mut self, owner: OsPeer, len: u64) {
        self.apply(owner, Op::Refund(len));
    }

    /// Releases every charge of `owner`.
    pub(crate) fn release(&mut self, owner: OsPeer) {
        self.apply(owner, Op::Release);
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

    /// `fits` is exactly the design's quota (D6 review M4), stated in
    /// 128-bit arithmetic with the design's numbers (256 regions and
    /// 1 GiB per owner, 4 GiB in all), not the code's constants or its
    /// checked sums.
    #[kani::proof]
    fn fits_is_exactly_the_quota() {
        let (count, bytes, total, len): (usize, u64, u64, u64) =
            (kani::any(), kani::any(), kani::any(), kani::any());
        let spec = count < 256
            && u128::from(bytes) + u128::from(len) <= 1u128 << 30
            && u128::from(total) + u128::from(len) <= 4u128 << 30;
        assert!(fits(count, bytes, total, len) == spec);
        kani::cover!(spec && len > 0);
        kani::cover!(!spec && count < 256);
    }

    /// The ledger's own accounting (review finding 8), by induction over
    /// its steps. Each step changes one owner's entry and the total; if
    /// before it that owner is within the quota and the total covers its
    /// bytes and is within the cap, then after it the same holds, nothing
    /// underflows, and the total moved by exactly that owner's bytes. So
    /// from an empty ledger (total 0, no owners) the total is always the
    /// sum of every owner's bytes, for any number of owners and steps.
    #[kani::proof]
    fn the_ledger_total_is_the_sum_of_its_owners() {
        const COUNT: usize = 256;
        const BYTES: u64 = 1 << 30;
        const TOTAL: u64 = 4 << 30;
        let e: Entry = if kani::any() {
            let (c, b): (usize, u64) = (kani::any(), kani::any());
            kani::assume(c >= 1 && c <= COUNT && b <= BYTES);
            Some((c, b))
        } else {
            None
        };
        let total: u64 = kani::any();
        let bytes = e.map_or(0, |(_, b)| b);
        kani::assume(bytes <= total && total <= TOTAL);
        let op = match kani::any::<u8>() % 3 {
            0 => Op::Charge(kani::any()),
            1 => Op::Refund(kani::any()),
            _ => Op::Release,
        };
        let Some((e2, total2)) = step(op, e, total) else {
            assert!(matches!(op, Op::Charge(_)));
            return;
        };
        let bytes2 = e2.map_or(0, |(_, b)| b);
        if let Some((c, b)) = e2 {
            assert!(c >= 1 && c <= COUNT && b <= BYTES);
        }
        assert!(bytes2 <= total2 && total2 <= TOTAL);
        // The rest of the owners' sum is unchanged.
        assert!(i128::from(total2) - i128::from(bytes2) == i128::from(total) - i128::from(bytes));
        // A charge adds one region of exactly its bytes; a refund or a
        // release never adds.
        match op {
            Op::Charge(len) => {
                let c0 = e.map_or(0, |(c, _)| c);
                assert!(
                    e2.is_some_and(|(c, b)| c == c0 + 1
                        && u128::from(b) == u128::from(bytes) + u128::from(len))
                );
            }
            _ => assert!(bytes2 <= bytes),
        }
        kani::cover!(matches!(op, Op::Charge(_)) && e.is_some());
        kani::cover!(matches!(op, Op::Refund(_)) && e2.is_some());
    }
}
