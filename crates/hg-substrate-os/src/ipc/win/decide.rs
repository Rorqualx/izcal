//! A receiver's decision on one data message (IPC-R1, R5, R6; security
//! scope SEC-4).
//!
//! The IO layer reads one pipe message; [`decide`] then accepts it,
//! refuses it as a violation, or closes the channel. An accepted message
//! with handles carries a ticket, which the transport redeems with the
//! broker; [`check_redeemed`] then compares what the broker handed over
//! with the record's descriptors.
//!
//! The rules, in order:
//!
//! 1. A message longer than the buffer (`ERROR_MORE_DATA`): Violation.
//!    The buffer holds the longest legal message.
//! 2. An empty message: Violation. Every data message has a ticket and a
//!    header.
//! 3. A message that does not split into a ticket, a record and a
//!    trailer stating the message's own length, or a record that does
//!    not decode: Violation (Malformed). A message whose trailer states
//!    another length is the rest of one whose head a stale read took
//!    (probe W13c).
//! 4. Handles without a ticket, or a ticket without handles: Violation.
//! 5. A descriptor whose counters no honest holder has: Violation.
//! 6. `seq` not the next expected, or a record over capacity: Violation.
//!
//! **No violation here names a sender.** A pipe carries no sender
//! identity (probe P13), and a former holder of a moved sender end may
//! have queued records before the move, so the receiving process cannot
//! say who wrote a given record (design review B3). The one attributed
//! fault on Windows is the broker's: a bound receiver's sender reaching
//! another process ([`super::ledger::BindEvent`]).

use hg_substrate::ipc::{FaultKind, HandleKind, MAX_MESSAGE_HANDLES};

use super::frame::{FrameError, Item, NO_TICKET, Ticket, split_data};
use crate::ipc::common::codec::{
    CodecError, DataView, HandleDesc, REGION_FILLER_TRUSTED, WireKind, decode_data,
};
use crate::ipc::common::flow::{FlowError, ReceiverFlow, SenderFlow};

/// Why a message was a violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reason {
    /// Longer than the longest legal message.
    TooLong,
    /// An empty message.
    Empty,
    /// No room for the ticket, or longer than any data message.
    Frame(FrameError),
    /// A record that does not decode.
    Codec(CodecError),
    /// Handles without a ticket, or a ticket without handles.
    Ticket,
    /// A descriptor's counters that no honest holder has.
    Counters(FlowError),
    /// Out of sequence, or over capacity.
    Sequence(FlowError),
    /// The broker refused the ticket, or handed over other handles than
    /// the record describes.
    Redeem,
}

impl Reason {
    /// The fault kind reported for it.
    pub(crate) const fn fault_kind(self) -> FaultKind {
        match self {
            Self::TooLong | Self::Empty | Self::Frame(_) | Self::Codec(_) => FaultKind::Malformed,
            Self::Ticket | Self::Counters(_) | Self::Redeem => FaultKind::BadHandle,
            Self::Sequence(_) => FaultKind::Sequence,
        }
    }
}

/// [`decide`]'s answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(
    clippy::large_enum_variant,
    reason = "returned by value once per message; boxing would allocate per message"
)]
pub(crate) enum Decision<'a> {
    /// Deliver it, after redeeming `ticket` if it carries handles.
    Accept {
        /// The record.
        view: DataView<'a>,
        /// Its ticket; [`NO_TICKET`] when it has no handles.
        ticket: Ticket,
    },
    /// The peer broke the protocol; the channel ends.
    Violation(Reason),
}

/// Checks that a received end's counters are possible.
fn counters(d: &HandleDesc) -> Result<(), FlowError> {
    match d.kind {
        WireKind::Sender | WireKind::BoundSender => SenderFlow::from_transfer(d).map(|_f| ()),
        WireKind::Receiver => ReceiverFlow::from_transfer(d).map(|_f| ()),
        // A region carries no counters; filler-trust is the broker's to
        // say, in what it hands over, never the sender's.
        WireKind::RegionWriter | WireKind::ReadOnlyRegion | WireKind::ConsumerRegion => {
            if d.sent & REGION_FILLER_TRUSTED == 0 {
                Ok(())
            } else {
                Err(FlowError::BadTransfer)
            }
        }
    }
}

/// Decides one message. On Accept the record is counted in `flow`; on
/// a violation `flow` is unchanged (the channel is ending anyway).
pub(crate) fn decide<'a>(msg: &'a [u8], too_long: bool, flow: &mut ReceiverFlow) -> Decision<'a> {
    decide_with(msg, too_long, flow, decode_data)
}

/// [`decide`] with the record decoder given, so the proof can stand for
/// any result the decoder gives (the codec's own proofs cover it).
fn decide_with<'a, D>(
    msg: &'a [u8],
    too_long: bool,
    flow: &mut ReceiverFlow,
    decode: D,
) -> Decision<'a>
where
    D: FnOnce(&'a [u8]) -> Result<DataView<'a>, CodecError>,
{
    if too_long {
        return Decision::Violation(Reason::TooLong);
    }
    if msg.is_empty() {
        return Decision::Violation(Reason::Empty);
    }
    let (ticket, record) = match split_data(msg) {
        Ok(p) => p,
        Err(e) => return Decision::Violation(Reason::Frame(e)),
    };
    let view = match decode(record) {
        Ok(v) => v,
        Err(e) => return Decision::Violation(Reason::Codec(e)),
    };
    if view.descs().is_empty() != (ticket == NO_TICKET) {
        return Decision::Violation(Reason::Ticket);
    }
    for d in view.descs() {
        if let Err(e) = counters(d) {
            return Decision::Violation(Reason::Counters(e));
        }
    }
    if let Err(e) = flow.on_record(view.seq) {
        return Decision::Violation(Reason::Sequence(e));
    }
    Decision::Accept { view, ticket }
}

/// The handle kind a descriptor claims.
pub(crate) const fn handle_kind(k: WireKind) -> HandleKind {
    match k {
        WireKind::Sender => HandleKind::ChannelSender,
        WireKind::BoundSender => HandleKind::BoundChannelSender,
        WireKind::Receiver => HandleKind::ChannelReceiver,
        WireKind::RegionWriter => HandleKind::RegionWriter,
        WireKind::ReadOnlyRegion => HandleKind::ReadOnlyRegion,
        WireKind::ConsumerRegion => HandleKind::ConsumerRegion,
    }
}

/// One handle as received: its kind, and for a region whether this
/// process attests it immutable.
pub(crate) type Received = (HandleKind, bool);

/// Compares what the broker handed over for a ticket with the record's
/// descriptors: as many, in order, each of the kind described, and each
/// region of exactly the length described (D6 review minor 1: the broker
/// gives the registered size, which the receiver cannot learn itself,
/// fact S2). A read-only region is immutable unless the broker says it
/// is filler-trusted.
///
/// # Errors
/// [`Reason::Redeem`].
pub(crate) fn check_redeemed(
    descs: &[HandleDesc],
    got: &[Item],
) -> Result<[Received; MAX_MESSAGE_HANDLES], Reason> {
    if descs.len() != got.len() || descs.len() > MAX_MESSAGE_HANDLES {
        return Err(Reason::Redeem);
    }
    let mut out = [(HandleKind::ChannelReceiver, false); MAX_MESSAGE_HANDLES];
    for ((slot, d), g) in out.iter_mut().zip(descs).zip(got) {
        if d.kind != g.kind {
            return Err(Reason::Redeem);
        }
        if g.kind.is_region() && d.capacity != g.len {
            return Err(Reason::Redeem);
        }
        let immutable =
            g.kind == WireKind::ReadOnlyRegion && u64::from(g.flags) & REGION_FILLER_TRUSTED == 0;
        *slot = (handle_kind(g.kind), immutable);
    }
    Ok(out)
}

/// VER-1 for the Windows receive decision.
#[cfg(kani)]
mod proofs {
    use super::*;
    use crate::ipc::common::codec::proofs::any_desc;
    use crate::ipc::win::frame::{TICKET_LEN, TRAILER_LEN};

    /// The shortest data message: a ticket and the trailer.
    const MIN_MSG: usize = TICKET_LEN + TRAILER_LEN;

    /// The receiver's counters before a decision: any state within a
    /// small capacity.
    fn any_flow() -> ReceiverFlow {
        let capacity: u32 = kani::any_where(|&c: &u32| c <= 4);
        let received: u64 = kani::any_where(|&r: &u64| r <= 4);
        let delivered: u64 = kani::any_where(|&d: &u64| d <= received);
        ReceiverFlow {
            capacity,
            received,
            delivered,
        }
    }

    /// The rules before the decoder: a message too long, empty or
    /// shorter than a ticket and a trailer is refused and changes
    /// nothing.
    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_early() {
        let bytes = [0u8; MIN_MSG];
        let n: usize = kani::any_where(|&n: &usize| n < MIN_MSG);
        let too_long: bool = kani::any();
        let mut flow = any_flow();
        let before = flow;
        let d = decide_with(&bytes[..n], too_long, &mut flow, |_r| {
            Err(CodecError::Short)
        });
        assert!(matches!(d, Decision::Violation(_)));
        assert!(flow == before);
        kani::cover!(
            d == Decision::Violation(Reason::Frame(FrameError::Short)),
            "a message shorter than a ticket and a trailer is refused"
        );
    }

    /// A message of a symbolic ticket and any decoder result with `ND`
    /// descriptors (fixed per harness, the rest symbolic: the D1
    /// lesson): an accepted message has a ticket exactly when it has
    /// descriptors, only possible counters, and is counted; a violation
    /// changes nothing. Unwind 17: comparing the 16-byte ticket is a
    /// loop in the prover's `memcmp`.
    fn decisions_are_sound<const ND: usize>() -> bool {
        // A symbolic ticket, and the trailer of a message with an empty
        // record (the decoder's answer stands for any record).
        let mut msg: [u8; MIN_MSG] = kani::any();
        let [t0, t1, t2, t3] = u32::try_from(MIN_MSG).unwrap_or(0).to_le_bytes();
        msg[TICKET_LEN] = t0;
        msg[TICKET_LEN + 1] = t1;
        msg[TICKET_LEN + 2] = t2;
        msg[TICKET_LEN + 3] = t3;
        let mut table = [HandleDesc::EMPTY; MAX_MESSAGE_HANDLES];
        table[0] = any_desc();
        table[1] = any_desc();
        let decoded: Result<DataView<'_>, CodecError> = if kani::any() {
            Ok(DataView::for_proofs(kani::any(), table, ND, &msg[..0]))
        } else {
            Err(CodecError::BadMagic)
        };
        let mut flow = any_flow();
        let before = flow;
        let d = decide_with(&msg, false, &mut flow, |_r| decoded);
        match d {
            Decision::Accept { view, ticket } => {
                assert!(view.descs().len() == ND);
                assert!((ND == 0) == (ticket == NO_TICKET));
                for desc in view.descs() {
                    assert!(counters(desc).is_ok());
                }
                assert!(flow.received == before.received + 1);
                true
            }
            Decision::Violation(_) => {
                assert!(flow == before);
                false
            }
        }
    }

    #[kani::proof]
    #[kani::unwind(17)]
    fn decisions_are_sound_0() {
        kani::cover!(decisions_are_sound::<0>(), "a plain message is accepted");
    }

    #[kani::proof]
    #[kani::unwind(17)]
    fn decisions_are_sound_1() {
        kani::cover!(
            decisions_are_sound::<1>(),
            "a message with a handle is accepted"
        );
    }

    #[kani::proof]
    #[kani::unwind(17)]
    fn decisions_are_sound_2() {
        kani::cover!(
            decisions_are_sound::<2>(),
            "a message with two handles is accepted"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::common::codec::encode_data;
    use crate::ipc::win::frame::join_data;

    fn desc(kind: WireKind) -> HandleDesc {
        HandleDesc {
            kind,
            capacity: 4,
            sent: 0,
            delivered: 0,
        }
    }

    #[test]
    fn good_messages_are_accepted() {
        let mut flow = ReceiverFlow::new(4);
        let m = join_data(&NO_TICKET, &encode_data(0, &[], b"hi").unwrap());
        assert!(matches!(
            decide(&m, false, &mut flow),
            Decision::Accept {
                ticket: NO_TICKET,
                ..
            }
        ));
        let m = join_data(
            &[1; 16],
            &encode_data(1, &[desc(WireKind::Sender)], b"h").unwrap(),
        );
        let Decision::Accept { view, ticket } = decide(&m, false, &mut flow) else {
            panic!()
        };
        assert_eq!(ticket, [1; 16]);
        assert_eq!(
            check_redeemed(view.descs(), &[Item::end(4, WireKind::Sender)]).unwrap()[0],
            (HandleKind::ChannelSender, false)
        );
        assert_eq!(
            check_redeemed(view.descs(), &[Item::end(4, WireKind::Receiver)]),
            Err(Reason::Redeem)
        );
        assert_eq!(check_redeemed(view.descs(), &[]), Err(Reason::Redeem));
    }

    /// D6: a region's length is the broker's, exactly; immutability is
    /// the broker's to deny.
    #[test]
    fn redeemed_regions_have_the_described_length() {
        let d = [HandleDesc::region(WireKind::ReadOnlyRegion, 4096, 0)];
        let item = |len, flags| Item {
            value: 8,
            kind: WireKind::ReadOnlyRegion,
            len,
            flags,
            drains: false,
        };
        assert_eq!(
            check_redeemed(&d, &[item(4096, 0)]).unwrap()[0],
            (HandleKind::ReadOnlyRegion, true)
        );
        assert_eq!(
            check_redeemed(&d, &[item(4096, 2)]).unwrap()[0],
            (HandleKind::ReadOnlyRegion, false)
        );
        assert_eq!(check_redeemed(&d, &[item(8192, 0)]), Err(Reason::Redeem));
        // A sender's descriptor never claims filler-trust.
        let trusted = HandleDesc::region(WireKind::ReadOnlyRegion, 4096, 2);
        assert!(counters(&trusted).is_err());
    }

    #[test]
    fn every_rule_refuses() {
        let mut flow = ReceiverFlow::new(4);
        let rec = encode_data(0, &[], b"hi").unwrap();
        let v = |m: &[u8], long: bool, f: &mut ReceiverFlow| match decide(m, long, f) {
            Decision::Violation(r) => r,
            Decision::Accept { .. } => panic!("accepted"),
        };
        assert_eq!(v(&[], true, &mut flow), Reason::TooLong);
        assert_eq!(v(&[], false, &mut flow), Reason::Empty);
        assert_eq!(
            v(&[0; 3], false, &mut flow),
            Reason::Frame(FrameError::Short)
        );
        assert_eq!(
            v(&join_data(&NO_TICKET, b"junk"), false, &mut flow),
            Reason::Codec(CodecError::BadMagic)
        );
        assert_eq!(
            v(&join_data(&[1; 16], &rec), false, &mut flow),
            Reason::Ticket
        );
        let with_handle = encode_data(0, &[desc(WireKind::Sender)], b"").unwrap();
        assert_eq!(
            v(&join_data(&NO_TICKET, &with_handle), false, &mut flow),
            Reason::Ticket
        );
        let bad = HandleDesc {
            kind: WireKind::Sender,
            capacity: 1,
            sent: 1,
            delivered: 2,
        };
        assert!(matches!(
            v(
                &join_data(&[1; 16], &encode_data(0, &[bad], b"").unwrap()),
                false,
                &mut flow
            ),
            Reason::Counters(_)
        ));
        assert!(matches!(
            v(
                &join_data(&NO_TICKET, &encode_data(5, &[], b"").unwrap()),
                false,
                &mut flow
            ),
            Reason::Sequence(_)
        ));
        assert_eq!(flow.received, 0, "violations count nothing");
    }

    #[test]
    fn reasons_map_to_fault_kinds() {
        assert_eq!(Reason::TooLong.fault_kind(), FaultKind::Malformed);
        assert_eq!(Reason::Ticket.fault_kind(), FaultKind::BadHandle);
        assert_eq!(Reason::Redeem.fault_kind(), FaultKind::BadHandle);
        assert_eq!(
            Reason::Sequence(FlowError::Overrun).fault_kind(),
            FaultKind::Sequence
        );
    }
}
