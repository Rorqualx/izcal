//! The decision on one received data message (IPC-R1, R2, R5; security
//! scope SEC-4).
//!
//! The IO layer receives one message into a buffer, reads its layout
//! ([`super::wire::layout`]), sets the kernel's own notifications aside
//! ([`super::notify`]) and asks the kernel about each right the message
//! carries ([`SlotFacts`]). [`decide`] then gives one of three answers:
//!
//! - **Accept:** deliver the message; its rights become handles of the
//!   kinds given, and its reply right is kept to send the credit with.
//! - **Violation:** the peer broke the protocol. The receiver reports a
//!   [`Fault`], naming the sending process when the kernel attests it
//!   (the audit token, P1) and only that process could have caused it,
//!   so the broker can end it (IPC-R5).
//! - **Closed:** the channel ends with no one blamed.
//!
//! On anything but Accept the caller releases every right and region
//! the message carries: it is refused whole.
//!
//! The rules, in the order applied, each with its reason:
//!
//! 1. No audit token → Closed: the kernel always writes one when asked
//!    (P1), so its absence is this process's own mistake.
//! 2. A bound receiver (IPC-R2): the token must be the peer the receiver
//!    was bound to (pid and pid version, P3); a message from any other
//!    process is blamed on the bound peer, the only process that could
//!    have passed its sender on, whatever the message holds. Then a
//!    layout the parser cannot read past the header (a descriptor of an
//!    unknown type, a count past the message) → Violation: the kernel
//!    builds descriptors only from what the sender asked for.
//! 3. The message itself: our data id, sent on a send right (a message
//!    that arrived on a send-once right to a data port was not sent by
//!    this transport: no peer is ever given one), and carrying a
//!    send-once reply right, which the credit goes back on (a dead one is
//!    fine: the sender end was dropped after sending, P22) → else
//!    Violation.
//! 4. Out-of-line memory or port arrays → Violation: this transport
//!    sends neither (an array of ports is also fatal to an entitled
//!    sender, P30).
//! 5. A body that does not decode → Violation.
//! 6. Handles: the rights must be exactly the slots the descriptor table
//!    names ([`super::accept::slots`]), all distinct, each accepted, and
//!    every carried end's counters possible, and a carried sender's
//!    credit port with no more send-once rights than its messages in
//!    flight (R8) → else Violation. The table and the rights are the
//!    sending transport's own doing.
//! 7. `seq` not the next expected, or a message over capacity →
//!    Violation, *not* attributed: a transferred sender carries its
//!    counters, and a former holder that lied about them makes the next,
//!    honest holder look wrong.

use hg_substrate::ipc::{Fault, FaultKind, HandleKind, MAX_MESSAGE_HANDLES};

use super::super::common::codec::{CodecError, DataView, HandleDesc, WireKind, decode_data};
use super::super::common::flow::{FlowError, ReceiverFlow, SenderFlow};
use super::accept::{
    Held, Refusal, SlotFacts, accept_slot, distinct, handle_kind, slots, wire_slots,
};
use super::wire::{
    DESC_PORT, Header, ID_DATA, ID_HELLO_REPLY, NAME_NULL, PORT_SEND, PORT_SEND_ONCE, Token,
    WireError,
};

/// One data message as received.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Arrival<'a> {
    /// Its header.
    pub(crate) header: Header,
    /// Whether the parser read every descriptor.
    pub(crate) walk: Result<(), WireError>,
    /// The audit token.
    pub(crate) token: Result<Token, WireError>,
    /// The descriptor count the message claims.
    pub(crate) count: u32,
    /// The kernel's facts on each descriptor, in order; for a message
    /// with more descriptors than any record takes, the first
    /// `MAX_SLOTS + 1` (enough to refuse it).
    pub(crate) slots: &'a [SlotFacts],
    /// The body.
    pub(crate) body: &'a [u8],
}

/// The receiving channel's binding (IPC-R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Binding {
    /// For a bound channel, the one process that may send on it; `None`
    /// for a free one.
    pub(crate) peer: Option<Token>,
    /// The broker's token, in a launched process: the only sender of a
    /// region writer (D6 review M1). `None` in the broker itself, which
    /// never receives one.
    pub(crate) broker: Option<Token>,
}

/// Why a message was a violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reason {
    /// A layout the parser could not read.
    Layout(WireError),
    /// A process other than the bound peer.
    WrongPeer,
    /// Not our data id.
    WrongId,
    /// Sent on something other than a send right.
    NotSent,
    /// No send-once reply right.
    NoReply,
    /// Out-of-line memory or ports.
    OutOfLine,
    /// A body that does not decode.
    Codec(CodecError),
    /// The descriptor table and the rights disagree.
    CountMismatch,
    /// A right refused.
    Handle(Refusal),
    /// An end's counters that no honest holder has.
    Counters(FlowError),
    /// Out of sequence, or over capacity.
    Sequence(FlowError),
}

impl Reason {
    /// The fault kind reported for it.
    pub(crate) const fn fault_kind(self) -> FaultKind {
        match self {
            Self::Layout(_)
            | Self::WrongId
            | Self::NotSent
            | Self::NoReply
            | Self::OutOfLine
            | Self::Codec(_) => FaultKind::Malformed,
            Self::Handle(Refusal::WriterNotFromBroker) => FaultKind::Identity,
            Self::CountMismatch | Self::Handle(_) => FaultKind::BadHandle,
            Self::WrongPeer => FaultKind::Identity,
            Self::Counters(_) | Self::Sequence(_) => FaultKind::Sequence,
        }
    }
}

/// [`decide`]'s answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(
    clippy::large_enum_variant,
    reason = "one per message, on the stack: the receive path does not allocate for it"
)]
pub(crate) enum Decision<'a> {
    /// Deliver.
    Accept {
        /// The decoded record.
        view: DataView<'a>,
        /// The handle kind of each end, in table order.
        kinds: [HandleKind; MAX_MESSAGE_HANDLES],
        /// How many.
        count: usize,
    },
    /// Refuse and report.
    Violation {
        /// Why.
        reason: Reason,
        /// The process at fault, when the kernel attests it.
        sender: Option<u32>,
    },
    /// Refuse without blame.
    Closed,
}

impl Decision<'_> {
    /// The kinds of an accepted message's handles.
    pub(crate) fn kinds(&self) -> &[HandleKind] {
        match self {
            Self::Accept { kinds, count, .. } => kinds.get(..*count).unwrap_or(&[]),
            Self::Violation { .. } | Self::Closed => &[],
        }
    }

    /// The fault a violation reports.
    pub(crate) const fn fault(&self) -> Option<Fault> {
        match *self {
            Self::Violation { reason, sender } => Some(Fault {
                kind: reason.fault_kind(),
                sender,
            }),
            Self::Accept { .. } | Self::Closed => None,
        }
    }
}

/// Checks that a received end's counters are possible.
fn counters(d: &HandleDesc) -> Result<(), FlowError> {
    match d.kind {
        WireKind::Sender | WireKind::BoundSender => SenderFlow::from_transfer(d).map(|_f| ()),
        WireKind::Receiver => ReceiverFlow::from_transfer(d).map(|_f| ()),
        // A region carries no counters (the codec checked its fields).
        WireKind::RegionWriter | WireKind::ReadOnlyRegion | WireKind::ConsumerRegion => Ok(()),
    }
}

/// Decides one message. On Accept the record is counted in `flow`; on
/// anything else `flow` is unchanged (the channel is ending anyway).
pub(crate) fn decide<'a, H: Held>(
    a: &Arrival<'a>,
    binding: Binding,
    flow: &mut ReceiverFlow,
    held: &H,
) -> Decision<'a> {
    decide_with(a, binding, flow, held, decode_data)
}

/// [`decide`] with the record decoder given, so the decision's proof can
/// stand for any result the decoder gives (the codec's own proofs cover
/// the decoder). It is called at rule 5.
fn decide_with<'a, H: Held, D>(
    a: &Arrival<'a>,
    binding: Binding,
    flow: &mut ReceiverFlow,
    held: &H,
    decode: D,
) -> Decision<'a>
where
    D: FnOnce(&'a [u8]) -> Result<DataView<'a>, CodecError>,
{
    // Rule 1: the layout and the token.
    let Ok(token) = a.token else {
        return Decision::Closed;
    };
    let sender = (token.pid != 0).then_some(token.pid);
    let refuse = |reason: Reason| Decision::Violation { reason, sender };

    // Rule 2: the binding, before anything the sender wrote is read: a
    // message from another process is the bound peer's fault whatever
    // its shape.
    if let Some(peer) = binding.peer
        && token != peer
    {
        return Decision::Violation {
            reason: Reason::WrongPeer,
            sender: (peer.pid != 0).then_some(peer.pid),
        };
    }
    if let Err(e) = a.walk {
        return refuse(Reason::Layout(e));
    }

    // Rule 3: the message itself.
    if a.header.id != ID_DATA {
        return refuse(Reason::WrongId);
    }
    if a.header.local_disposition() != PORT_SEND {
        return refuse(Reason::NotSent);
    }
    // A reply right whose credit port is gone arrives as a dead name
    // (P22): its sender end was dropped after sending, which is honest.
    if a.header.remote_disposition() != PORT_SEND_ONCE || a.header.remote == NAME_NULL {
        return refuse(Reason::NoReply);
    }

    // Rule 4: nothing out of line.
    if a.slots.iter().any(|s| s.desc_type != DESC_PORT) {
        return refuse(Reason::OutOfLine);
    }

    // Rule 5: the record.
    let view = match decode(a.body) {
        Ok(v) => v,
        Err(e) => return refuse(Reason::Codec(e)),
    };

    // Rule 6: the handles.
    let descs = view.descs();
    let kinds_claimed: [WireKind; MAX_MESSAGE_HANDLES] = {
        let mut k = [WireKind::Receiver; MAX_MESSAGE_HANDLES];
        for (slot, d) in k.iter_mut().zip(descs) {
            *slot = d.kind;
        }
        k
    };
    let claimed = kinds_claimed.get(..descs.len()).unwrap_or(&[]);
    let Some((want, n_want)) = wire_slots(claimed) else {
        return refuse(Reason::CountMismatch);
    };
    if usize::try_from(a.count).ok() != Some(n_want) || a.slots.len() != n_want {
        return refuse(Reason::CountMismatch);
    }
    if let Err(r) = distinct(a.slots) {
        return refuse(Reason::Handle(r));
    }
    for (slot, f) in want.iter().zip(a.slots) {
        if let Err(r) = accept_slot(*slot, f, held) {
            return refuse(Reason::Handle(r));
        }
    }
    let mut kinds = [HandleKind::ChannelReceiver; MAX_MESSAGE_HANDLES];
    let mut at = 0usize;
    for (k, d) in kinds.iter_mut().zip(descs) {
        if let Err(e) = counters(d) {
            return refuse(Reason::Counters(e));
        }
        // A writer only from the broker, as the kernel attests it.
        if d.kind == WireKind::RegionWriter && binding.broker.is_none_or(|b| b != token) {
            return refuse(Reason::Handle(Refusal::WriterNotFromBroker));
        }
        // A carried sender's credit port: its send-once rights are the
        // reply rights of its messages not yet credited (R8), no more.
        let (_, n) = slots(d.kind);
        if n == 2
            && let Some(c) = a.slots.get(at.saturating_add(1))
            && u64::from(c.send_once) > d.sent.saturating_sub(d.delivered)
        {
            return refuse(Reason::Handle(Refusal::SendOnce));
        }
        at = at.saturating_add(n);
        *k = handle_kind(d.kind);
    }

    // Rule 7: the sequence.
    if let Err(e) = flow.on_record(view.seq) {
        return Decision::Violation {
            reason: Reason::Sequence(e),
            sender: None,
        };
    }
    Decision::Accept {
        view,
        kinds,
        count: descs.len(),
    }
}

/// The check a launched process makes on its launcher's answer to its
/// hello: our reply id, a token, no out-of-line data, a record whose
/// table is exactly one sender end (to the launcher) and one receiver
/// end (from it), and the three rights those take, distinct and each
/// accepted. The launcher is trusted, but its answer still crosses the
/// kernel: a right that is not what it claims is refused here as
/// anywhere.
///
/// # Errors
/// The [`Reason`].
pub(crate) fn decide_launch<'a, H: Held>(
    a: &Arrival<'a>,
    held: &H,
) -> Result<(Token, DataView<'a>), Reason> {
    let token = a.token.map_err(Reason::Layout)?;
    a.walk.map_err(Reason::Layout)?;
    if a.header.id != ID_HELLO_REPLY {
        return Err(Reason::WrongId);
    }
    if a.slots.iter().any(|s| s.desc_type != DESC_PORT) {
        return Err(Reason::OutOfLine);
    }
    let view = decode_data(a.body).map_err(Reason::Codec)?;
    let kinds: [WireKind; 2] = match view.descs() {
        [x, y] => [x.kind, y.kind],
        _ => return Err(Reason::CountMismatch),
    };
    if kinds != [WireKind::Sender, WireKind::Receiver] {
        return Err(Reason::CountMismatch);
    }
    let Some((want, n)) = wire_slots(&kinds) else {
        return Err(Reason::CountMismatch);
    };
    if usize::try_from(a.count).ok() != Some(n) || a.slots.len() != n {
        return Err(Reason::CountMismatch);
    }
    distinct(a.slots).map_err(Reason::Handle)?;
    for (slot, f) in want.iter().zip(a.slots) {
        accept_slot(*slot, f, held).map_err(Reason::Handle)?;
    }
    for d in view.descs() {
        counters(d).map_err(Reason::Counters)?;
    }
    Ok((token, view))
}

/// VER-1 for the whole-message decision. Bounds: up to 2 handles claimed
/// and up to 3 rights received, any header, token and binding, and any
/// result of the record decoder (whose own proofs are the codec's).
#[cfg(kani)]
mod proofs {
    use super::*;
    use crate::ipc::common::codec::proofs::any_desc;
    use crate::ipc::mach::accept::proofs::{OneHeld, any_facts};

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Seen {
        Accepted,
        Blamed,
        Unblamed,
        Closed,
    }

    fn any_token() -> Result<Token, WireError> {
        if kani::any() {
            Ok(Token {
                pid: kani::any(),
                version: kani::any(),
            })
        } else {
            Err(WireError::NoTrailer)
        }
    }

    /// For `NS` rights received and `ND` table entries: an Accept has
    /// exactly as many rights as its table's slots, every right accepted
    /// in its slot, all distinct, the bound peer's token, our id, a send
    /// right destination and a send-once reply, and counts the record;
    /// anything else changes no counter and blames only the attested
    /// sender, or, for a wrong peer, the bound peer.
    fn decisions_are_sound<const NS: usize, const ND: usize>() -> Seen {
        let facts = [any_facts(), any_facts(), any_facts()];
        let header = Header {
            bits: kani::any(),
            size: kani::any(),
            remote: kani::any(),
            local: kani::any(),
            voucher: kani::any(),
            id: kani::any(),
        };
        let token = any_token();
        let walk = if kani::any() {
            Ok(())
        } else {
            Err(WireError::Truncated)
        };
        let a = Arrival {
            header,
            walk,
            token,
            count: kani::any(),
            slots: &facts[..NS],
            body: &[],
        };
        let mut table = [HandleDesc::EMPTY; MAX_MESSAGE_HANDLES];
        table[0] = any_desc();
        table[1] = any_desc();
        let table_kinds = [table[0].kind, table[1].kind];
        let decoded: Result<DataView<'_>, CodecError> = if kani::any() {
            Ok(DataView::for_proofs(kani::any(), table, ND, &[]))
        } else {
            Err(CodecError::BadMagic)
        };
        let any_peer = || {
            if kani::any() {
                Some(Token {
                    pid: kani::any(),
                    version: kani::any(),
                })
            } else {
                None
            }
        };
        let binding = Binding {
            peer: any_peer(),
            broker: any_peer(),
        };
        let capacity: u32 = kani::any_where(|&c: &u32| c <= 4);
        let received: u64 = kani::any_where(|&r: &u64| r <= 4);
        let delivered: u64 = kani::any_where(|&d: &u64| d <= received);
        let mut flow = ReceiverFlow {
            capacity,
            received,
            delivered,
        };
        let before = flow;
        let held = OneHeld {
            held_name: kani::any(),
            refs: kani::any(),
            receiver: kani::any(),
        };
        let d = decide_with(&a, binding, &mut flow, &held, |_b| decoded);
        // A bound receiver's message from any other process is the bound
        // peer's violation, whatever else is wrong with it.
        if let (Some(p), Ok(t)) = (binding.peer, token)
            && t != p
        {
            assert!(matches!(
                d,
                Decision::Violation {
                    reason: Reason::WrongPeer,
                    sender,
                } if sender == (p.pid != 0).then_some(p.pid)
            ));
        }
        match d {
            Decision::Accept { view, kinds, count } => {
                assert!(count == ND);
                let Some((want, n)) = wire_slots(&table_kinds[..ND]) else {
                    panic!()
                };
                assert!(n == NS);
                assert!(usize::try_from(a.count).ok() == Some(NS));
                for i in 0..NS {
                    assert!(accept_slot(want[i], &facts[i], &held).is_ok());
                    for j in 0..i {
                        assert!(facts[i].name != facts[j].name);
                    }
                }
                let mut at = 0usize;
                for i in 0..ND {
                    assert!(kinds[i] == handle_kind(table_kinds[i]));
                    // A writer only with the broker's token (review M1).
                    if table_kinds[i] == WireKind::RegionWriter {
                        assert!(binding.broker.is_some() && binding.broker == token.ok());
                    }
                    let (_, n) = slots(table_kinds[i]);
                    if n == 2 {
                        assert!(
                            u64::from(facts[at + 1].send_once)
                                <= table[i].sent - table[i].delivered
                        );
                    }
                    at += n;
                }
                let _ = view;
                let Ok(t) = token else { panic!() };
                if let Some(p) = binding.peer {
                    assert!(t == p);
                }
                assert!(header.id == ID_DATA);
                assert!(header.local_disposition() == PORT_SEND);
                assert!(header.remote_disposition() == PORT_SEND_ONCE);
                assert!(walk.is_ok());
                assert!(flow.received == before.received + 1);
                Seen::Accepted
            }
            Decision::Violation { reason, sender } => {
                assert!(flow == before);
                if let Some(s) = sender {
                    if reason == Reason::WrongPeer {
                        assert!(binding.peer.is_some_and(|p| p.pid == s));
                    } else {
                        assert!(token.is_ok_and(|t| t.pid == s));
                    }
                    Seen::Blamed
                } else {
                    Seen::Unblamed
                }
            }
            Decision::Closed => {
                assert!(flow == before);
                assert!(token.is_err());
                Seen::Closed
            }
        }
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_0_0() {
        let seen = decisions_are_sound::<0, 0>();
        kani::cover!(seen == Seen::Accepted, "an empty message is accepted");
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
        kani::cover!(seen == Seen::Unblamed, "a sequence fault is not attributed");
        kani::cover!(seen == Seen::Closed, "no token: closed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_1_1() {
        let seen = decisions_are_sound::<1, 1>();
        kani::cover!(seen == Seen::Accepted, "a receiver end is accepted");
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_2_1() {
        let seen = decisions_are_sound::<2, 1>();
        kani::cover!(seen == Seen::Accepted, "a sender end is accepted");
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_3_2() {
        let seen = decisions_are_sound::<3, 2>();
        kani::cover!(seen == Seen::Accepted, "a sender and a receiver end");
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_2_2() {
        let seen = decisions_are_sound::<2, 2>();
        kani::cover!(seen == Seen::Accepted, "two receiver ends");
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::common::codec::encode_data;
    use crate::ipc::mach::accept::Held;
    use crate::ipc::mach::wire::{
        BITS_COMPLEX, KOBJECT_NONE, PORT_RECEIVE, TYPE_RECEIVE, TYPE_SEND,
    };

    struct Nothing;
    impl Held for Nothing {
        fn send_refs(&self, _n: u32) -> u32 {
            0
        }
        fn receiver(&self, _n: u32) -> bool {
            false
        }
    }

    const PEER: Token = Token {
        pid: 4242,
        version: 9,
    };

    fn send_fact(name: u32) -> SlotFacts {
        SlotFacts {
            desc_type: DESC_PORT,
            name,
            disposition: PORT_SEND,
            type_bits: TYPE_SEND,
            send_refs: 1,
            kobject: KOBJECT_NONE,
            plain_queue: false,
            send_once: 0,
            destroy_request: false,
        }
    }

    fn recv_fact(name: u32) -> SlotFacts {
        SlotFacts {
            desc_type: DESC_PORT,
            name,
            disposition: PORT_RECEIVE,
            type_bits: TYPE_RECEIVE,
            send_refs: 0,
            kobject: 0,
            plain_queue: true,
            send_once: 0,
            destroy_request: false,
        }
    }

    fn desc(kind: WireKind) -> HandleDesc {
        HandleDesc {
            kind,
            capacity: 8,
            sent: 0,
            delivered: 0,
        }
    }

    struct Case {
        body: Vec<u8>,
        header: Header,
        token: Result<Token, WireError>,
        walk: Result<(), WireError>,
        count: u32,
        slots: Vec<SlotFacts>,
        binding: Binding,
        flow: ReceiverFlow,
    }

    impl Case {
        /// A good message from PEER: seq 0, the given ends.
        fn new(kinds: &[WireKind]) -> Self {
            let descs: Vec<HandleDesc> = kinds.iter().map(|&k| desc(k)).collect();
            let mut slots = Vec::new();
            let mut name = 0x1000;
            for k in kinds {
                match k {
                    WireKind::Sender | WireKind::BoundSender => {
                        slots.push(send_fact(name));
                        slots.push(recv_fact(name | 1));
                    }
                    WireKind::Receiver => slots.push(recv_fact(name)),
                    WireKind::RegionWriter
                    | WireKind::ReadOnlyRegion
                    | WireKind::ConsumerRegion => {
                        // No case here sends a region; D6's macOS step adds them.
                        slots.push(send_fact(name));
                    }
                }
                name = name.wrapping_add(0x10);
            }
            Self {
                body: encode_data(0, &descs, b"hello").unwrap(),
                header: Header {
                    bits: u32::from(PORT_SEND_ONCE)
                        | (u32::from(PORT_SEND) << 8)
                        | if slots.is_empty() { 0 } else { BITS_COMPLEX },
                    size: 0,
                    remote: 0x707,
                    local: 0x303,
                    voucher: 0,
                    id: ID_DATA,
                },
                token: Ok(PEER),
                walk: Ok(()),
                count: u32::try_from(slots.len()).unwrap(),
                slots,
                binding: Binding {
                    peer: None,
                    broker: None,
                },
                flow: ReceiverFlow::new(8),
            }
        }

        fn run(&mut self) -> Decision<'_> {
            let a = Arrival {
                header: self.header,
                walk: self.walk,
                token: self.token,
                count: self.count,
                slots: &self.slots,
                body: &self.body,
            };
            decide(&a, self.binding, &mut self.flow, &Nothing)
        }

        fn violation(&mut self) -> (Reason, Option<u32>) {
            let before = self.flow;
            let d = self.run();
            let Decision::Violation { reason, sender } = d else {
                panic!("expected a violation, got {d:?}");
            };
            assert_eq!(self.flow, before);
            (reason, sender)
        }
    }

    /// A carried sender's credit port may hold the reply rights of its
    /// messages in flight, and no more (R8).
    #[test]
    fn a_credit_port_s_send_once_rights_are_its_messages_in_flight() {
        let mut c = Case::new(&[WireKind::Sender]);
        c.slots[1].send_once = 1;
        assert_eq!(
            c.violation(),
            (Reason::Handle(Refusal::SendOnce), Some(4242))
        );
        let mut c = Case::new(&[WireKind::Sender]);
        c.slots[1].send_once = 1;
        let one_out = HandleDesc {
            sent: 1,
            ..desc(WireKind::Sender)
        };
        c.body = encode_data(0, &[one_out], b"hello").unwrap();
        assert!(matches!(c.run(), Decision::Accept { .. }));
        // A receiver end's data port may have none.
        let mut c = Case::new(&[WireKind::Receiver]);
        c.slots[0].send_once = 1;
        assert_eq!(
            c.violation(),
            (Reason::Handle(Refusal::SendOnce), Some(4242))
        );
    }

    #[test]
    fn a_good_message_is_accepted() {
        let mut c = Case::new(&[WireKind::Sender, WireKind::Receiver]);
        let d = c.run();
        assert_eq!(
            d.kinds(),
            &[HandleKind::ChannelSender, HandleKind::ChannelReceiver]
        );
        let Decision::Accept { view, .. } = d else {
            panic!()
        };
        assert_eq!(view.payload, b"hello");
        assert_eq!(c.flow.received, 1);
    }

    /// A sender end dropped after sending destroys its credit port, so
    /// its queued messages' reply rights arrive dead (P22): still honest.
    #[test]
    fn a_dead_reply_right_is_accepted() {
        let mut c = Case::new(&[]);
        c.header.remote = super::super::wire::NAME_DEAD;
        assert!(matches!(c.run(), Decision::Accept { .. }));
    }

    #[test]
    fn malformed_messages_are_blamed_on_their_sender() {
        let mut c = Case::new(&[]);
        c.header.id = 70;
        assert_eq!(c.violation(), (Reason::WrongId, Some(4242)));
        let mut c = Case::new(&[]);
        c.header.bits = u32::from(PORT_SEND_ONCE) | (u32::from(PORT_SEND_ONCE) << 8);
        assert_eq!(c.violation(), (Reason::NotSent, Some(4242)));
        let mut c = Case::new(&[]);
        c.header.remote = 0;
        assert_eq!(c.violation(), (Reason::NoReply, Some(4242)));
        let mut c = Case::new(&[WireKind::Receiver]);
        c.slots[0].desc_type = 1;
        assert_eq!(c.violation(), (Reason::OutOfLine, Some(4242)));
        let mut c = Case::new(&[]);
        c.walk = Err(WireError::UnknownDescriptor);
        assert_eq!(
            c.violation(),
            (Reason::Layout(WireError::UnknownDescriptor), Some(4242))
        );
        let mut c = Case::new(&[]);
        c.body = b"junk".to_vec();
        assert!(matches!(c.violation(), (Reason::Codec(_), Some(4242))));
    }

    #[test]
    fn handles_must_match_the_table() {
        // One right too few for a sender end.
        let mut c = Case::new(&[WireKind::Sender]);
        c.slots.pop();
        c.count = 1;
        assert_eq!(c.violation(), (Reason::CountMismatch, Some(4242)));
        // A claimed count that differs from the rights read.
        let mut c = Case::new(&[WireKind::Receiver]);
        c.count = 5;
        assert_eq!(c.violation(), (Reason::CountMismatch, Some(4242)));
        // A task port where the data send right should be.
        let mut c = Case::new(&[WireKind::Sender]);
        c.slots[0].kobject = 2;
        assert_eq!(
            c.violation(),
            (Reason::Handle(Refusal::KernelObject), Some(4242))
        );
        // The same name twice.
        let mut c = Case::new(&[WireKind::Receiver, WireKind::Receiver]);
        c.slots[1].name = c.slots[0].name;
        assert_eq!(
            c.violation(),
            (Reason::Handle(Refusal::Duplicate), Some(4242))
        );
    }

    #[test]
    fn a_bound_receiver_hears_only_its_peer() {
        let mut c = Case::new(&[]);
        c.binding = Binding {
            peer: Some(PEER),
            broker: None,
        };
        assert!(matches!(c.run(), Decision::Accept { .. }));
        // Another process, or the same pid after exec: blamed on the bound
        // peer.
        for other in [
            Token { pid: 7, version: 1 },
            Token {
                pid: 4242,
                version: 10,
            },
        ] {
            let mut c = Case::new(&[]);
            c.binding = Binding {
                peer: Some(PEER),
                broker: None,
            };
            c.token = Ok(other);
            assert_eq!(c.violation(), (Reason::WrongPeer, Some(4242)));
            // A wrong process's unreadable message is still the bound
            // peer's fault, not the sender's to be blamed for.
            let mut c = Case::new(&[]);
            c.binding = Binding {
                peer: Some(PEER),
                broker: None,
            };
            c.token = Ok(other);
            c.walk = Err(WireError::UnknownDescriptor);
            assert_eq!(c.violation(), (Reason::WrongPeer, Some(4242)));
        }
    }

    #[test]
    fn sequence_faults_are_not_blamed_and_no_token_closes() {
        let mut c = Case::new(&[]);
        c.flow = ReceiverFlow::new(0);
        assert_eq!(c.violation(), (Reason::Sequence(FlowError::Overrun), None));
        let mut c = Case::new(&[]);
        c.token = Err(WireError::NoTrailer);
        assert_eq!(c.run(), Decision::Closed);
    }

    #[test]
    fn a_launch_reply_carries_exactly_two_ends() {
        let mut c = Case::new(&[WireKind::Sender, WireKind::Receiver]);
        c.header.id = ID_HELLO_REPLY;
        let a = Arrival {
            header: c.header,
            walk: c.walk,
            token: c.token,
            count: c.count,
            slots: &c.slots,
            body: &c.body,
        };
        let (t, view) = decide_launch(&a, &Nothing).unwrap();
        assert_eq!(t, PEER);
        assert_eq!(view.descs().len(), 2);
        let swapped = Case::new(&[WireKind::Receiver, WireKind::Sender]);
        let a = Arrival {
            header: Header {
                id: ID_HELLO_REPLY,
                ..swapped.header
            },
            walk: Ok(()),
            token: Ok(PEER),
            count: swapped.count,
            slots: &swapped.slots,
            body: &swapped.body,
        };
        assert_eq!(
            decide_launch(&a, &Nothing).err(),
            Some(Reason::CountMismatch)
        );
    }
}
