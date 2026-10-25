//! Flow control: the counters behind `Full` and the credits, as pure
//! state machines.
//!
//! Capacity counts *undelivered* messages, as in the simulator: a
//! message stops counting when its delivery task starts. The receiver
//! then writes a credit record with its cumulative delivered count, and
//! the sender's `Full` is decided from credits alone, never from the
//! kernel's buffer:
//!
//! - sender: `Full` iff `sent - delivered_known >= capacity`;
//! - receiver: paused (stops reading) iff `received - delivered >=
//!   capacity`.
//!
//! Since `delivered_known <= delivered` (credits arrive late, never
//! early) and `received <= sent`, an honest sender never has more than
//! `capacity` records the receiver has not delivered, so a receiver that
//! finds itself over capacity, or a record out of sequence, has a lying
//! peer.
//!
//! A transferred end carries its counters in its descriptor
//! ([`super::codec::HandleDesc`]), so its new holder continues the same
//! sequence (plan section 6).

use super::codec::{Credit, HandleDesc};

/// A counter or credit that an honest peer cannot produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum FlowError {
    /// The channel holds its capacity of undelivered messages.
    Full,
    /// A counter would pass `u64::MAX` (2^64 messages: unreachable for an
    /// honest peer).
    Overflow,
    /// A credit for a different capacity: another channel's, or garbage.
    CapacityMismatch,
    /// A credit for more deliveries than were sent.
    AheadOfSent,
    /// A credit below one already seen: credits are cumulative.
    Regressed,
    /// A record whose `seq` is not the next expected.
    Sequence,
    /// A record beyond the capacity: the sender ignored `Full`.
    Overrun,
    /// A delivery with nothing received.
    NothingInHand,
    /// A transferred end's counters that cannot happen: delivered above
    /// sent, or more in flight than the capacity.
    BadTransfer,
}

/// The sending end's view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SenderFlow {
    /// The channel's capacity.
    pub(crate) capacity: u32,
    /// Messages sent by this end and every former holder.
    pub(crate) sent: u64,
    /// Deliveries known from credits; at most `sent`.
    pub(crate) delivered_known: u64,
}

impl SenderFlow {
    /// A new channel's sender.
    pub(crate) const fn new(capacity: u32) -> Self {
        Self {
            capacity,
            sent: 0,
            delivered_known: 0,
        }
    }

    /// Messages sent and not known delivered.
    pub(crate) const fn in_flight(&self) -> u64 {
        // delivered_known <= sent is kept by every constructor and
        // transition; 0 is never reached otherwise.
        match self.sent.checked_sub(self.delivered_known) {
            Some(n) => n,
            None => 0,
        }
    }

    /// Whether a send now would be refused as `Full`.
    pub(crate) const fn would_be_full(&self) -> bool {
        self.in_flight() >= u64_from(self.capacity)
    }

    /// Counts one send and gives its `seq` (the sent count before it).
    ///
    /// # Errors
    /// [`FlowError::Full`] at capacity, [`FlowError::Overflow`]; the
    /// counters are unchanged.
    pub(crate) fn on_send(&mut self) -> Result<u64, FlowError> {
        if self.would_be_full() {
            return Err(FlowError::Full);
        }
        let seq = self.sent;
        self.sent = seq.checked_add(1).ok_or(FlowError::Overflow)?;
        Ok(seq)
    }

    /// Applies a credit record.
    ///
    /// Equal credits are accepted (a transfer can leave the new holder
    /// reading a credit its former holder already counted only if the
    /// counts agree), lower ones are not.
    ///
    /// # Errors
    /// [`FlowError::CapacityMismatch`], [`FlowError::AheadOfSent`],
    /// [`FlowError::Regressed`]; the counters are unchanged.
    pub(crate) fn on_credit(&mut self, c: Credit) -> Result<(), FlowError> {
        if c.capacity != self.capacity {
            return Err(FlowError::CapacityMismatch);
        }
        if c.delivered > self.sent {
            return Err(FlowError::AheadOfSent);
        }
        if c.delivered < self.delivered_known {
            return Err(FlowError::Regressed);
        }
        self.delivered_known = c.delivered;
        Ok(())
    }

    /// The descriptor that carries this end to another process.
    pub(crate) const fn to_desc(self, kind: super::codec::WireKind) -> HandleDesc {
        HandleDesc {
            kind,
            capacity: self.capacity,
            sent: self.sent,
            delivered: self.delivered_known,
        }
    }

    /// The counters of a received sender end.
    ///
    /// # Errors
    /// [`FlowError::BadTransfer`] for counters no honest holder has.
    pub(crate) fn from_transfer(d: &HandleDesc) -> Result<Self, FlowError> {
        let f = Self {
            capacity: d.capacity,
            sent: d.sent,
            delivered_known: d.delivered,
        };
        let in_flight = d.sent.checked_sub(d.delivered);
        match in_flight {
            Some(n) if n <= u64_from(d.capacity) => Ok(f),
            Some(_) | None => Err(FlowError::BadTransfer),
        }
    }
}

/// The receiving end's view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ReceiverFlow {
    /// The channel's capacity.
    pub(crate) capacity: u32,
    /// Records read, by this end and every former holder.
    pub(crate) received: u64,
    /// Deliveries started; at most `received`.
    pub(crate) delivered: u64,
}

impl ReceiverFlow {
    /// A new channel's receiver.
    pub(crate) const fn new(capacity: u32) -> Self {
        Self {
            capacity,
            received: 0,
            delivered: 0,
        }
    }

    /// Records read and not yet delivered.
    pub(crate) const fn in_hand(&self) -> u64 {
        match self.received.checked_sub(self.delivered) {
            Some(n) => n,
            None => 0,
        }
    }

    /// Whether to stop reading: the capacity is in hand. An honest sender
    /// sends nothing more until a delivery frees a slot.
    pub(crate) const fn paused(&self) -> bool {
        self.in_hand() >= u64_from(self.capacity)
    }

    /// Checks a record's `seq` without counting it.
    ///
    /// # Errors
    /// [`FlowError::Sequence`] unless `seq` is the received count,
    /// [`FlowError::Overrun`] when paused.
    pub(crate) const fn check_record(&self, seq: u64) -> Result<(), FlowError> {
        if seq != self.received {
            return Err(FlowError::Sequence);
        }
        if self.paused() {
            return Err(FlowError::Overrun);
        }
        Ok(())
    }

    /// Counts a record.
    ///
    /// # Errors
    /// As [`Self::check_record`], and [`FlowError::Overflow`]; the
    /// counters are unchanged.
    pub(crate) fn on_record(&mut self, seq: u64) -> Result<(), FlowError> {
        self.check_record(seq)?;
        self.received = self.received.checked_add(1).ok_or(FlowError::Overflow)?;
        Ok(())
    }

    /// Counts the start of a delivery task and gives the credit to send.
    ///
    /// # Errors
    /// [`FlowError::NothingInHand`]; the counters are unchanged.
    pub(crate) fn on_deliver(&mut self) -> Result<Credit, FlowError> {
        if self.in_hand() == 0 {
            return Err(FlowError::NothingInHand);
        }
        self.delivered = self.delivered.checked_add(1).ok_or(FlowError::Overflow)?;
        Ok(Credit {
            capacity: self.capacity,
            delivered: self.delivered,
        })
    }

    /// The descriptor that carries this end to another process.
    pub(crate) const fn to_desc(self) -> HandleDesc {
        HandleDesc {
            kind: super::codec::WireKind::Receiver,
            capacity: self.capacity,
            sent: self.received,
            delivered: self.delivered,
        }
    }

    /// The counters of a received receiver end.
    ///
    /// # Errors
    /// [`FlowError::BadTransfer`] for counters no honest holder has.
    pub(crate) fn from_transfer(d: &HandleDesc) -> Result<Self, FlowError> {
        let f = Self {
            capacity: d.capacity,
            received: d.sent,
            delivered: d.delivered,
        };
        match d.sent.checked_sub(d.delivered) {
            Some(n) if n <= u64_from(d.capacity) => Ok(f),
            Some(_) | None => Err(FlowError::BadTransfer),
        }
    }
}

/// Credits waiting to be written. Credits are cumulative, so only the
/// latest matters: when the reverse direction's buffer is full, newer
/// ones replace older ones and nothing is lost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) struct PendingCredit {
    latest: Option<Credit>,
}

impl PendingCredit {
    /// Notes a credit; keeps the higher of it and any pending one.
    pub(crate) fn note(&mut self, c: Credit) {
        self.latest = match self.latest {
            Some(old) if old.delivered >= c.delivered => Some(old),
            Some(_) | None => Some(c),
        };
    }

    /// The credit to write, if any, leaving nothing pending.
    pub(crate) const fn take(&mut self) -> Option<Credit> {
        self.latest.take()
    }

    /// Puts back a credit that could not be written (`EAGAIN`), unless a
    /// newer one has been noted since.
    pub(crate) fn restore(&mut self, c: Credit) {
        self.note(c);
    }

    /// Whether a credit is waiting.
    pub(crate) const fn is_pending(&self) -> bool {
        self.latest.is_some()
    }
}

/// `u64::from` for a `const fn` (trait methods are not const).
const fn u64_from(n: u32) -> u64 {
    let [a, b, c, d] = n.to_le_bytes();
    u64::from_le_bytes([a, b, c, d, 0, 0, 0, 0])
}

/// VER-1 for flow control. Bounds: capacity 0 to 3, 8 steps.
#[cfg(kani)]
mod proofs {
    use super::*;

    const STEPS: usize = 8;

    /// An honest sender and receiver, with records and credits in transit
    /// in order, under any interleaving of 8 steps: no step is refused as
    /// a violation (the receiver reads whenever a record is there, paused
    /// or not), in-flight never exceeds the capacity, `Full` happens
    /// exactly at the bound, credits are monotonic, and the receiver
    /// never holds more than its capacity.
    #[kani::proof]
    #[kani::unwind(10)]
    fn honest_peers_never_violate() {
        let capacity: u32 = kani::any_where(|&c: &u32| c <= 3);
        let mut s = SenderFlow::new(capacity);
        let mut r = ReceiverFlow::new(capacity);
        // Records in transit: their seqs are a contiguous run starting at
        // `next_seq_to_read`, of length `records`.
        let mut records: u64 = 0;
        let mut next_seq: u64 = 0;
        // Credits in transit, coalesced as the IO layer does.
        let mut pending = PendingCredit::default();
        let mut saw_full = false;
        for _ in 0..STEPS {
            match kani::any::<u8>() % 4 {
                0 => {
                    let full_before = s.would_be_full();
                    let in_flight = s.in_flight();
                    match s.on_send() {
                        Ok(seq) => {
                            assert!(!full_before);
                            assert!(seq == next_seq + records);
                            records += 1;
                        }
                        Err(e) => {
                            assert!(e == FlowError::Full);
                            assert!(full_before && in_flight == u64::from(capacity));
                            saw_full = true;
                        }
                    }
                }
                1 => {
                    // No pause check: even a receiver that reads whenever
                    // a record is there never sees an overrun.
                    if records > 0 {
                        assert!(r.on_record(next_seq).is_ok());
                        next_seq += 1;
                        records -= 1;
                    }
                }
                2 => {
                    if let Ok(c) = r.on_deliver() {
                        pending.note(c);
                    }
                }
                _ => {
                    if let Some(c) = pending.take() {
                        let before = s.delivered_known;
                        assert!(s.on_credit(c).is_ok());
                        assert!(s.delivered_known >= before);
                    }
                }
            }
            assert!(s.in_flight() <= u64::from(capacity));
            assert!(s.would_be_full() == (s.in_flight() == u64::from(capacity)));
            assert!(r.in_hand() <= u64::from(capacity));
            assert!(s.delivered_known <= r.delivered && r.delivered <= r.received);
            assert!(r.received <= s.sent);
        }
        kani::cover!(saw_full && capacity == 2, "the sender reached Full");
    }

    /// Any credit: accepted exactly when its capacity matches and it lies
    /// between the known count and the sent count; a refused credit
    /// changes nothing.
    #[kani::proof]
    fn credits_are_checked() {
        let d = HandleDesc {
            kind: crate::ipc::common::codec::WireKind::Sender,
            capacity: kani::any(),
            sent: kani::any(),
            delivered: kani::any(),
        };
        let Ok(mut s) = SenderFlow::from_transfer(&d) else {
            assert!(d.delivered > d.sent || d.sent - d.delivered > u64::from(d.capacity));
            return;
        };
        let before = s;
        let c = Credit {
            capacity: kani::any(),
            delivered: kani::any(),
        };
        let ok = s.on_credit(c).is_ok();
        kani::cover!(ok, "a credit is accepted");
        assert!(
            ok == (c.capacity == before.capacity
                && before.delivered_known <= c.delivered
                && c.delivered <= before.sent)
        );
        if !ok {
            assert!(s == before);
        }
        assert!(s.in_flight() <= u64::from(s.capacity));
    }

    /// A record is accepted exactly when its `seq` is the next one and
    /// the receiver is below capacity.
    #[kani::proof]
    fn records_are_in_sequence() {
        let d = HandleDesc {
            kind: crate::ipc::common::codec::WireKind::Receiver,
            capacity: kani::any(),
            sent: kani::any(),
            delivered: kani::any(),
        };
        let Ok(mut r) = ReceiverFlow::from_transfer(&d) else {
            return;
        };
        let before = r;
        let seq: u64 = kani::any();
        let ok = r.on_record(seq).is_ok();
        kani::cover!(ok, "a record is accepted");
        assert!(
            ok == (seq == before.received
                && before.in_hand() < u64::from(before.capacity)
                && before.received < u64::MAX)
        );
        if !ok {
            assert!(r == before);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::common::codec::WireKind;

    #[test]
    fn full_is_exact_and_credits_free_slots() {
        let mut s = SenderFlow::new(2);
        let mut r = ReceiverFlow::new(2);
        assert_eq!(s.on_send(), Ok(0));
        assert_eq!(s.on_send(), Ok(1));
        assert!(s.would_be_full());
        assert_eq!(s.on_send(), Err(FlowError::Full));
        assert_eq!(s.sent, 2);

        assert_eq!(r.on_record(0), Ok(()));
        assert_eq!(r.on_record(1), Ok(()));
        assert!(r.paused());
        assert_eq!(r.on_record(2), Err(FlowError::Overrun));
        let c = r.on_deliver().unwrap();
        assert_eq!(
            c,
            Credit {
                capacity: 2,
                delivered: 1
            }
        );
        assert!(!r.paused());
        assert_eq!(s.on_credit(c), Ok(()));
        assert!(!s.would_be_full());
        assert_eq!(s.on_send(), Ok(2));
    }

    #[test]
    fn capacity_zero_is_always_full() {
        let mut s = SenderFlow::new(0);
        assert_eq!(s.on_send(), Err(FlowError::Full));
        assert!(ReceiverFlow::new(0).paused());
    }

    #[test]
    fn bad_credits_are_refused_and_change_nothing() {
        let mut s = SenderFlow::new(4);
        for _ in 0..3 {
            s.on_send().unwrap();
        }
        let credit = |capacity, delivered| Credit {
            capacity,
            delivered,
        };
        s.on_credit(credit(4, 2)).unwrap();
        let before = s;
        assert_eq!(s.on_credit(credit(5, 3)), Err(FlowError::CapacityMismatch));
        assert_eq!(s.on_credit(credit(4, 4)), Err(FlowError::AheadOfSent));
        assert_eq!(s.on_credit(credit(4, 1)), Err(FlowError::Regressed));
        assert_eq!(s, before);
        // A repeat is harmless.
        assert_eq!(s.on_credit(credit(4, 2)), Ok(()));
        assert_eq!(s.on_credit(credit(4, 3)), Ok(()));
        assert_eq!(s.in_flight(), 0);
    }

    #[test]
    fn records_out_of_sequence_are_refused() {
        let mut r = ReceiverFlow::new(8);
        assert_eq!(r.on_record(1), Err(FlowError::Sequence));
        r.on_record(0).unwrap();
        assert_eq!(r.on_record(0), Err(FlowError::Sequence));
        assert_eq!(r.on_deliver().map(|c| c.delivered), Ok(1));
        assert_eq!(r.on_deliver(), Err(FlowError::NothingInHand));
    }

    #[test]
    fn transfers_carry_the_counters() {
        let mut s = SenderFlow::new(3);
        s.on_send().unwrap();
        s.on_send().unwrap();
        let d = s.to_desc(WireKind::Sender);
        let mut moved = SenderFlow::from_transfer(&d).unwrap();
        assert_eq!(moved, s);
        assert_eq!(moved.on_send(), Ok(2));

        let mut r = ReceiverFlow::new(3);
        r.on_record(0).unwrap();
        let back = ReceiverFlow::from_transfer(&r.to_desc()).unwrap();
        assert_eq!(back, r);

        let lie = |sent, delivered, capacity| HandleDesc {
            kind: WireKind::Sender,
            capacity,
            sent,
            delivered,
        };
        assert_eq!(
            SenderFlow::from_transfer(&lie(1, 2, 3)),
            Err(FlowError::BadTransfer)
        );
        assert_eq!(
            SenderFlow::from_transfer(&lie(9, 2, 3)),
            Err(FlowError::BadTransfer)
        );
        assert_eq!(
            ReceiverFlow::from_transfer(&lie(9, 2, 3)),
            Err(FlowError::BadTransfer)
        );
    }

    #[test]
    fn pending_credit_keeps_only_the_latest() {
        let c = |delivered| Credit {
            capacity: 1,
            delivered,
        };
        let mut p = PendingCredit::default();
        assert!(!p.is_pending());
        p.note(c(1));
        p.note(c(3));
        p.note(c(2));
        assert_eq!(p.take(), Some(c(3)));
        assert_eq!(p.take(), None);
        p.note(c(5));
        p.restore(c(4));
        assert_eq!(p.take(), Some(c(5)));
    }
}
