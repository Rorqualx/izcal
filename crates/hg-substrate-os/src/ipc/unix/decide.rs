//! The decision on one received data record (IPC-R1, R2, R5; security
//! scope SEC-4).
//!
//! The IO layer reads one record with `recvmsg(2)`, parses its control
//! data ([`super::cmsg::parse`]) and asks the kernel about each received
//! descriptor ([`FdFacts`]). [`decide`] then gives one of three answers:
//!
//! - **Accept:** deliver the message; its descriptors become handles of
//!   the kinds given.
//! - **Violation:** the peer broke the protocol. The receiver reports a
//!   [`Fault`], naming the sending process when the kernel attests it
//!   and the fault is one only that process could cause, so the broker
//!   can end it (IPC-R5).
//! - **Closed:** the channel ends with no one blamed, because the cause
//!   may be this process's own (running out of descriptors).
//!
//! On anything but Accept the caller closes every descriptor the kernel
//! installed ([`Verdict::close`]): the record is refused whole, so no
//! handle from it survives.
//!
//! The rules, in the order applied, each with its reason:
//!
//! 1. `MSG_CTRUNC` → Closed. The control buffer always has room for 16
//!    descriptors and the credentials, so truncation means either a
//!    sender sending more, or the kernel failing to install descriptors
//!    because this process is out of them (recvmsg(2), unix(7)). The two
//!    cannot be told apart, and blaming a peer for our own shortage
//!    would let a busy broker kill honest renderers.
//! 2. A zero-length read: with a hang-up → Closed (the peer is gone:
//!    end of file). Without one → Violation: this transport never sends
//!    an empty record (every data record has a header), and on a
//!    `SOCK_SEQPACKET` socket an empty record is otherwise
//!    indistinguishable from end of file.
//! 3. `MSG_TRUNC` → Violation: the buffer holds the longest legal record,
//!    so only a sender writing a longer one truncates.
//! 4. Control data refused by the parser → Violation: the kernel
//!    writes only what the sender asked for, and we ask for nothing but
//!    descriptors and credentials.
//! 5. A bound receiver (IPC-R2): every record must carry credentials
//!    (`SO_PASSCRED` is set before the end leaves its maker, so their
//!    absence means the channel is not what it claims; no one is
//!    blamed, since no pid is known), and the pid must match the one
//!    latched from the first accepted record (a mismatch is blamed on
//!    the record's attested sender, the intruder).
//! 6. A record that does not decode → Violation.
//! 7. Handles: an `SCM_RIGHTS` with no descriptors, a descriptor count
//!    that differs from the descriptor table's, a duplicate socket, a
//!    refused descriptor ([`accept`]) or a descriptor whose counters no
//!    honest holder has → Violation. The table is written by the
//!    sending transport, so all of these are its own doing.
//! 8. `seq` not the next expected, or a record over capacity →
//!    Violation, *not* attributed: a transferred sender carries its
//!    counters, and a former holder that lied about them makes the next,
//!    honest holder look wrong, so the receiver cannot tell who is at
//!    fault (plan section 2).
//!
//! Attribution needs the record's `SCM_CREDENTIALS`, which the kernel
//! fills in and the sender cannot forge (unix(7)); every rule above that
//! is blamed on the sender is a fault only the immediate sender can
//! cause.

use hg_substrate::ipc::{Fault, FaultKind, HandleKind, MAX_MESSAGE_HANDLES};

use super::accept::{AcceptCtx, FdFacts, Held, Refusal, accept, distinct};
use super::cmsg::{CmsgError, ParsedControl};
use super::codec::{CodecError, DataView, HandleDesc, WireKind, decode_data};
use super::flow::{FlowError, ReceiverFlow, SenderFlow};

/// One record as read.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Record<'a> {
    /// The bytes `recvmsg(2)` returned.
    pub(crate) bytes: &'a [u8],
    /// The socket reported a hang-up (`EPOLLHUP` or `EPOLLRDHUP`).
    pub(crate) hang_up: bool,
    /// `MSG_TRUNC` was set in `msg_flags`.
    pub(crate) truncated: bool,
    /// `MSG_CTRUNC` was set in `msg_flags`.
    pub(crate) ctrl_truncated: bool,
    /// The parsed control data.
    pub(crate) control: &'a ParsedControl,
    /// The kernel's facts on each received descriptor, in the order of
    /// `control.fds()`. Empty when the control data was refused (the
    /// caller need not ask about descriptors it will close).
    pub(crate) facts: &'a [FdFacts],
}

/// The receiving channel's binding (IPC-R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Binding {
    /// The channel is bound: only one process may send on it.
    pub(crate) bound: bool,
    /// The pid latched from the first accepted record, once there is one.
    pub(crate) latched: Option<i32>,
}

/// Why a record was a violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reason {
    /// A zero-length record without a hang-up.
    Empty,
    /// `MSG_TRUNC`: longer than any legal record.
    Truncated,
    /// Control data the parser refused.
    Control(CmsgError),
    /// No credentials on a bound channel.
    NoCredentials,
    /// Credentials of a process other than the latched one.
    WrongPeer,
    /// A record that does not decode.
    Codec(CodecError),
    /// An `SCM_RIGHTS` message with no descriptors.
    EmptyRights,
    /// The descriptor table and the received descriptors disagree.
    CountMismatch,
    /// A descriptor refused.
    Handle(Refusal),
    /// A descriptor's counters that no honest holder has.
    Counters(FlowError),
    /// Out of sequence, or over capacity.
    Sequence(FlowError),
}

impl Reason {
    /// The fault kind reported for it.
    pub(crate) const fn fault_kind(self) -> FaultKind {
        match self {
            Self::Empty | Self::Truncated | Self::Control(_) | Self::Codec(_) => {
                FaultKind::Malformed
            }
            Self::EmptyRights | Self::CountMismatch => FaultKind::BadHandle,
            Self::Handle(r) => match r {
                Refusal::NotByCreator => FaultKind::Identity,
                Refusal::Stdio
                | Refusal::NotSocket
                | Refusal::Domain
                | Refusal::Type
                | Refusal::Protocol
                | Refusal::AlreadyHeld
                | Refusal::Duplicate => FaultKind::BadHandle,
            },
            Self::NoCredentials | Self::WrongPeer => FaultKind::Identity,
            Self::Counters(_) | Self::Sequence(_) => FaultKind::Sequence,
        }
    }
}

/// The answer for one record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(
    clippy::large_enum_variant,
    reason = "one per record, on the stack: the receive path does not allocate for it"
)]
pub(crate) enum Decision<'a> {
    /// Deliver it.
    Accept {
        /// The record.
        view: DataView<'a>,
        /// The handle kind of each received descriptor, in order; slots
        /// at and past `count` are filler.
        kinds: [HandleKind; MAX_MESSAGE_HANDLES],
        /// How many handles.
        count: usize,
    },
    /// The peer broke the protocol.
    Violation {
        /// What it did.
        reason: Reason,
        /// The attested sender, when the fault is one only it could
        /// cause.
        sender: Option<u32>,
    },
    /// The channel ends without blame.
    Closed,
}

impl Decision<'_> {
    /// The fault to report, for a violation.
    pub(crate) const fn fault(&self) -> Option<Fault> {
        match *self {
            Self::Violation { reason, sender } => Some(Fault {
                kind: reason.fault_kind(),
                sender,
            }),
            Self::Accept { .. } | Self::Closed => None,
        }
    }

    /// The accepted handle kinds, in order.
    pub(crate) fn kinds(&self) -> &[HandleKind] {
        match self {
            Self::Accept { kinds, count, .. } => kinds.get(..*count).unwrap_or(&[]),
            Self::Violation { .. } | Self::Closed => &[],
        }
    }
}

/// [`decide`]'s result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Verdict<'a> {
    /// The answer.
    pub(crate) decision: Decision<'a>,
    /// The descriptors to close now: every one the kernel installed,
    /// unless the record is accepted (then none).
    pub(crate) close: &'a [i32],
    /// On a bound channel's first accepted record, the pid to latch.
    pub(crate) latch: Option<i32>,
}

/// The attested sender, if the kernel named one.
fn attested(control: &ParsedControl) -> Option<u32> {
    control
        .cred
        .and_then(|c| u32::try_from(c.pid).ok())
        .filter(|&pid| pid > 0)
}

/// Checks that a received end's counters are possible.
fn counters(d: &HandleDesc) -> Result<(), FlowError> {
    match d.kind {
        WireKind::Sender | WireKind::BoundSender => SenderFlow::from_transfer(d).map(|_f| ()),
        WireKind::Receiver => ReceiverFlow::from_transfer(d).map(|_f| ()),
    }
}

/// Decides one record. On Accept the record is counted in `flow`; on
/// anything else `flow` is unchanged (the channel is ending anyway).
pub(crate) fn decide<'a, H: Held>(
    rec: &Record<'a>,
    binding: Binding,
    flow: &mut ReceiverFlow,
    held: &H,
) -> Verdict<'a> {
    decide_with(rec, binding, flow, held, decode_data)
}

/// [`decide`] with the record decoder given, so the decision's proof can
/// stand for any result the decoder gives (the codec's own proofs cover
/// the decoder). It is called at rule 6, after rules 1 to 5.
fn decide_with<'a, H: Held, D>(
    rec: &Record<'a>,
    binding: Binding,
    flow: &mut ReceiverFlow,
    held: &H,
    decode: D,
) -> Verdict<'a>
where
    D: FnOnce(&'a [u8]) -> Result<DataView<'a>, CodecError>,
{
    let control = rec.control;
    let close = control.close_set();
    let sender = attested(control);
    let refuse = |reason: Reason, blame: bool| Verdict {
        decision: Decision::Violation {
            reason,
            sender: if blame { sender } else { None },
        },
        close,
        latch: None,
    };
    let closed = Verdict {
        decision: Decision::Closed,
        close,
        latch: None,
    };

    // Rules 1 to 4: the read itself.
    if rec.ctrl_truncated {
        return closed;
    }
    if rec.bytes.is_empty() {
        return if rec.hang_up {
            closed
        } else {
            refuse(Reason::Empty, true)
        };
    }
    if rec.truncated {
        return refuse(Reason::Truncated, true);
    }
    if let Err(e) = control.outcome {
        return refuse(Reason::Control(e), true);
    }

    // Rule 5: the binding.
    let pid = control.cred.map(|c| c.pid).filter(|&p| p > 0);
    let mut latch = None;
    if binding.bound {
        let Some(pid) = pid else {
            return refuse(Reason::NoCredentials, false);
        };
        match binding.latched {
            Some(l) if l != pid => return refuse(Reason::WrongPeer, true),
            Some(_) => {}
            None => latch = Some(pid),
        }
    }

    // Rule 6: the record.
    let view = match decode(rec.bytes) {
        Ok(v) => v,
        Err(e) => return refuse(Reason::Codec(e), true),
    };

    // Rule 7: the handles.
    let fds = control.fds();
    if control.rights_msgs > 0 && fds.is_empty() {
        return refuse(Reason::EmptyRights, true);
    }
    let descs = view.descs();
    if descs.len() != fds.len() {
        return refuse(Reason::CountMismatch, true);
    }
    // The caller's facts must describe exactly the received descriptors;
    // anything else is this process's own bug, so no one is blamed.
    if rec.facts.len() != fds.len() || rec.facts.iter().zip(fds).any(|(f, &fd)| f.fd != fd) {
        return closed;
    }
    if let Err(r) = distinct(rec.facts) {
        return refuse(Reason::Handle(r), true);
    }
    let ctx = AcceptCtx {
        cred_pid: pid,
        held,
    };
    let mut kinds = [HandleKind::ChannelReceiver; MAX_MESSAGE_HANDLES];
    for ((slot, d), f) in kinds.iter_mut().zip(descs).zip(rec.facts) {
        match accept(d.kind, f, &ctx) {
            Ok(k) => *slot = k,
            Err(r) => return refuse(Reason::Handle(r), true),
        }
        if let Err(e) = counters(d) {
            return refuse(Reason::Counters(e), true);
        }
    }

    // Rule 8: the sequence.
    if let Err(e) = flow.on_record(view.seq) {
        return refuse(Reason::Sequence(e), false);
    }
    Verdict {
        decision: Decision::Accept {
            view,
            kinds,
            count: descs.len(),
        },
        close: &[],
        latch,
    }
}

/// VER-1 for the whole-message decision. Bounds: up to 2 descriptors
/// received and 1 more seen only in the close set, up to 2 descriptor
/// table entries, and any result of the record decoder (whose own proofs
/// are the codec's).
#[cfg(kani)]
mod proofs {
    use super::*;
    use crate::ipc::unix::accept::proofs::any_facts;
    use crate::ipc::unix::cmsg::Ucred;
    use crate::ipc::unix::codec::proofs::any_desc;

    /// Which part of the decision a harness covers.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Part {
        /// Rules 1 to 5 refuse the record, before it is decoded. They
        /// read only the read's flags, the control outcome, the
        /// credentials and the binding, never the descriptors, so one
        /// descriptor count stands for all.
        Early,
        /// Rules 1 to 5 pass; rules 6 to 8 decide.
        Late,
    }

    /// Any inputs in the bounds, with `NF` descriptors received and `ND`
    /// descriptor-table entries: an Accept (so never one where the counts
    /// disagree) has exactly as many handles as descriptors received,
    /// kernel facts given and descriptor-table entries, each accepted
    /// by [`accept`] as its claimed kind, closes nothing and counts the
    /// record; anything else closes every installed descriptor, changes
    /// no counter, and blames only the attested sender. The counts are
    /// fixed per harness and the rest symbolic (the D1 lesson); together
    /// the harnesses below cover every count up to 2 on each side.
    /// What a harness's decision was, for its covers.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Seen {
        Accepted,
        Blamed,
        Unblamed,
        Closed,
    }

    fn decisions_are_sound<const NF: usize, const ND: usize>(part: Part) -> Seen {
        let fds: [i32; 2] = kani::any();
        let n_fds = NF;
        let extra: [i32; 1] = kani::any();
        let n_extra: usize = kani::any_where(|&n: &usize| n <= 1);
        let cred: Option<Ucred> = if kani::any() {
            Some(Ucred {
                pid: kani::any(),
                uid: 0,
                gid: 0,
            })
        } else {
            None
        };
        let outcome = if kani::any() {
            Ok(())
        } else {
            Err(CmsgError::TwoRights)
        };
        let rights_msgs: usize = kani::any_where(|&n: &usize| n <= 2);
        let control =
            ParsedControl::from_parts(&fds[..n_fds], &extra[..n_extra], rights_msgs, cred, outcome);

        // The record: empty or not (rules 2 and 6), and any result the
        // decoder can give for it: a refusal, or a view of up to two
        // descriptors.
        let raw = [0u8; 1];
        let bytes: &[u8] = if kani::any() { &raw } else { &raw[..0] };
        let mut table = [HandleDesc::EMPTY; MAX_MESSAGE_HANDLES];
        table[0] = any_desc();
        table[1] = any_desc();
        let n_descs = ND;
        let decoded: Result<DataView<'_>, CodecError> = if kani::any() {
            Ok(DataView::for_proofs(kani::any(), table, n_descs, &raw[..0]))
        } else {
            Err(match kani::any::<u8>() % 4 {
                0 => CodecError::Short,
                1 => CodecError::BadMagic,
                2 => CodecError::BadKind,
                _ => CodecError::TooManyHandles,
            })
        };

        let mut facts = [any_facts(), any_facts()];
        let n_facts: usize = kani::any_where(|&n: &usize| n <= 2);
        if kani::any() {
            facts[0].fd = fds[0];
            facts[1].fd = fds[1];
        }
        let rec = Record {
            bytes,
            hang_up: kani::any(),
            truncated: kani::any(),
            ctrl_truncated: kani::any(),
            control: &control,
            facts: &facts[..n_facts],
        };
        let binding = Binding {
            bound: kani::any(),
            latched: kani::any(),
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
        let held_answer: bool = kani::any();
        let held = move |_d: u64, _i: u64| held_answer;

        let pid = cred.map(|c| c.pid).filter(|&p| p > 0);
        let binding_ok =
            !binding.bound || pid.is_some_and(|p| binding.latched.is_none_or(|l| l == p));
        let early_ok = !rec.ctrl_truncated
            && !rec.bytes.is_empty()
            && !rec.truncated
            && control.outcome.is_ok()
            && binding_ok;
        kani::assume(early_ok == (part == Part::Late));

        let v = decide_with(&rec, binding, &mut flow, &held, |_b| decoded);
        if part == Part::Early {
            assert!(!matches!(v.decision, Decision::Accept { .. }));
        }
        match v.decision {
            Decision::Accept { view, kinds, count } => {
                assert!(count == NF);
                assert!(count == control.fds().len());
                assert!(count == view.descs().len());
                assert!(count == n_facts);
                assert!(v.close.is_empty());
                assert!(control.outcome.is_ok() && !rec.ctrl_truncated && !rec.truncated);
                let ctx = AcceptCtx {
                    cred_pid: cred.map(|c| c.pid).filter(|&p| p > 0),
                    held: &held,
                };
                for i in 0..2 {
                    if i < count {
                        assert!(accept(view.descs()[i].kind, &facts[i], &ctx) == Ok(kinds[i]));
                    }
                }
                assert!(flow.received == before.received + 1);
                if let Some(p) = v.latch {
                    assert!(binding.bound && binding.latched.is_none());
                    assert!(cred.map(|c| c.pid) == Some(p));
                }
                if binding.bound {
                    assert!(v.latch.is_some() || binding.latched == cred.map(|c| c.pid));
                }
                Seen::Accepted
            }
            Decision::Violation { sender, .. } => {
                assert!(core::ptr::eq(v.close, control.close_set()));
                assert!(flow == before);
                assert!(v.latch.is_none());
                if let Some(s) = sender {
                    assert!(cred.is_some_and(|c| i64::from(c.pid) == i64::from(s)));
                    Seen::Blamed
                } else {
                    Seen::Unblamed
                }
            }
            Decision::Closed => {
                assert!(core::ptr::eq(v.close, control.close_set()));
                assert!(flow == before);
                Seen::Closed
            }
        }
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_early() {
        let seen = decisions_are_sound::<1, 1>(Part::Early);
        kani::cover!(seen == Seen::Blamed, "an early refusal is blamed");
        kani::cover!(seen == Seen::Unblamed, "an early refusal is not blamed");
        kani::cover!(seen == Seen::Closed, "an early close");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_0_0() {
        let seen = decisions_are_sound::<0, 0>(Part::Late);
        kani::cover!(
            seen == Seen::Accepted,
            "a record with 0 handles is accepted"
        );
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_0_1() {
        let seen = decisions_are_sound::<0, 1>(Part::Late);
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_0_2() {
        let seen = decisions_are_sound::<0, 2>(Part::Late);
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_1_0() {
        let seen = decisions_are_sound::<1, 0>(Part::Late);
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_1_1() {
        let seen = decisions_are_sound::<1, 1>(Part::Late);
        kani::cover!(
            seen == Seen::Accepted,
            "a record with 1 handles is accepted"
        );
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
        kani::cover!(seen == Seen::Unblamed, "a sequence fault is not attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_1_2() {
        let seen = decisions_are_sound::<1, 2>(Part::Late);
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_2_0() {
        let seen = decisions_are_sound::<2, 0>(Part::Late);
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_2_1() {
        let seen = decisions_are_sound::<2, 1>(Part::Late);
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn decisions_are_sound_2_2() {
        let seen = decisions_are_sound::<2, 2>(Part::Late);
        kani::cover!(
            seen == Seen::Accepted,
            "a record with 2 handles is accepted"
        );
        kani::cover!(seen == Seen::Blamed, "a violation is attributed");
        kani::cover!(seen == Seen::Unblamed, "a sequence fault is not attributed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::unix::accept::{AF_UNIX, SOCK_SEQPACKET};
    use crate::ipc::unix::cmsg::{CmsgError, Ucred};
    use crate::ipc::unix::codec::encode_data;

    const PEER: i32 = 4242;

    fn cred(pid: i32) -> Option<Ucred> {
        Some(Ucred {
            pid,
            uid: 1000,
            gid: 1000,
        })
    }

    fn facts(fd: i32, ino: u64) -> FdFacts {
        FdFacts {
            fd,
            is_socket: true,
            domain: AF_UNIX,
            sock_type: SOCK_SEQPACKET,
            protocol: 0,
            dev: 1,
            ino,
            peercred_pid: PEER,
        }
    }

    fn desc(kind: WireKind) -> HandleDesc {
        HandleDesc {
            kind,
            capacity: 8,
            sent: 2,
            delivered: 1,
        }
    }

    fn none_held(_d: u64, _i: u64) -> bool {
        false
    }

    const FREE: Binding = Binding {
        bound: false,
        latched: None,
    };

    struct Case {
        bytes: Vec<u8>,
        control: ParsedControl,
        facts: Vec<FdFacts>,
        hang_up: bool,
        truncated: bool,
        ctrl_truncated: bool,
        binding: Binding,
        flow: ReceiverFlow,
    }

    impl Case {
        /// A good record: seq 0, the given descriptors on fds 10, 11, ...
        fn new(kinds: &[WireKind]) -> Self {
            let descs: Vec<HandleDesc> = kinds.iter().map(|&k| desc(k)).collect();
            let fds: Vec<i32> = (10..).take(kinds.len()).collect();
            let rights = usize::from(!fds.is_empty());
            Self {
                bytes: encode_data(0, &descs, b"hello").unwrap(),
                control: ParsedControl::from_parts(&fds, &[], rights, cred(PEER), Ok(())),
                facts: fds
                    .iter()
                    .map(|&fd| facts(fd, u64::try_from(fd).unwrap()))
                    .collect(),
                hang_up: false,
                truncated: false,
                ctrl_truncated: false,
                binding: FREE,
                flow: ReceiverFlow::new(8),
            }
        }

        fn run(&mut self) -> (Decision<'_>, Vec<i32>, Option<i32>) {
            let rec = Record {
                bytes: &self.bytes,
                hang_up: self.hang_up,
                truncated: self.truncated,
                ctrl_truncated: self.ctrl_truncated,
                control: &self.control,
                facts: &self.facts,
            };
            let v = decide(&rec, self.binding, &mut self.flow, &none_held);
            (v.decision, v.close.to_vec(), v.latch)
        }

        fn violation(&mut self) -> (Reason, Option<u32>, Vec<i32>) {
            let before = self.flow;
            let (d, close, latch) = self.run();
            assert_eq!(latch, None);
            let Decision::Violation { reason, sender } = d else {
                panic!("expected a violation, got {d:?}");
            };
            assert_eq!(self.flow, before);
            (reason, sender, close)
        }
    }

    const ATTESTED: Option<u32> = Some(4242);

    #[test]
    fn a_good_record_is_accepted() {
        let mut c = Case::new(&[WireKind::Sender, WireKind::Receiver]);
        let (d, close, latch) = c.run();
        assert_eq!(
            d.kinds(),
            &[HandleKind::ChannelSender, HandleKind::ChannelReceiver]
        );
        let Decision::Accept { view, .. } = d else {
            panic!()
        };
        assert_eq!(view.payload, b"hello");
        assert!(close.is_empty());
        assert_eq!(latch, None);
        assert_eq!(d.fault(), None);
        assert_eq!(c.flow.received, 1);
    }

    #[test]
    fn ctrunc_and_hang_up_close_without_blame() {
        let mut c = Case::new(&[WireKind::Sender]);
        c.ctrl_truncated = true;
        assert_eq!(c.run(), (Decision::Closed, vec![10], None));

        let mut c = Case::new(&[]);
        c.bytes.clear();
        c.hang_up = true;
        assert_eq!(c.run().0, Decision::Closed);
    }

    #[test]
    fn read_level_violations_are_blamed_on_the_sender() {
        let mut c = Case::new(&[WireKind::Sender]);
        c.bytes.clear();
        assert_eq!(c.violation(), (Reason::Empty, ATTESTED, vec![10]));

        let mut c = Case::new(&[WireKind::Sender]);
        c.truncated = true;
        assert_eq!(c.violation(), (Reason::Truncated, ATTESTED, vec![10]));

        let mut c = Case::new(&[]);
        c.control = ParsedControl::from_parts(&[5], &[6], 2, cred(PEER), Err(CmsgError::TwoRights));
        let (reason, sender, close) = c.violation();
        assert_eq!(reason, Reason::Control(CmsgError::TwoRights));
        assert_eq!((sender, close), (ATTESTED, vec![5, 6]));

        let mut c = Case::new(&[]);
        c.bytes[0] ^= 1;
        let (reason, _, _) = c.violation();
        assert_eq!(reason, Reason::Codec(CodecError::BadMagic));
        assert_eq!(reason.fault_kind(), FaultKind::Malformed);
    }

    #[test]
    fn without_credentials_nothing_is_attributed() {
        let mut c = Case::new(&[]);
        c.control = ParsedControl::from_parts(&[], &[], 0, None, Ok(()));
        c.truncated = true;
        assert_eq!(c.violation().1, None);
        // A pid of 0 (a sender outside our pid namespace) names no one.
        c.control = ParsedControl::from_parts(&[], &[], 0, cred(0), Ok(()));
        assert_eq!(c.violation().1, None);
    }

    #[test]
    fn handle_rules() {
        // A descriptor table longer than the descriptors.
        let mut c = Case::new(&[WireKind::Sender]);
        c.control = ParsedControl::from_parts(&[], &[], 0, cred(PEER), Ok(()));
        c.facts.clear();
        assert_eq!(c.violation().0, Reason::CountMismatch);

        // Descriptors with no table entry.
        let mut c = Case::new(&[]);
        c.control = ParsedControl::from_parts(&[10], &[], 1, cred(PEER), Ok(()));
        c.facts = vec![facts(10, 10)];
        assert_eq!(c.violation(), (Reason::CountMismatch, ATTESTED, vec![10]));

        // An SCM_RIGHTS with no descriptors.
        let mut c = Case::new(&[]);
        c.control = ParsedControl::from_parts(&[], &[], 1, cred(PEER), Ok(()));
        assert_eq!(c.violation().0, Reason::EmptyRights);

        // A pipe claimed as a sender.
        let mut c = Case::new(&[WireKind::Sender, WireKind::Sender]);
        c.facts[1].is_socket = false;
        let (reason, sender, close) = c.violation();
        assert_eq!(reason, Reason::Handle(Refusal::NotSocket));
        assert_eq!(reason.fault_kind(), FaultKind::BadHandle);
        assert_eq!((sender, close), (ATTESTED, vec![10, 11]));

        // One socket twice.
        let mut c = Case::new(&[WireKind::Sender, WireKind::Receiver]);
        c.facts[1].ino = c.facts[0].ino;
        assert_eq!(c.violation().0, Reason::Handle(Refusal::Duplicate));

        // A forwarded bound sender.
        let mut c = Case::new(&[WireKind::BoundSender]);
        c.facts[0].peercred_pid = PEER + 1;
        let (reason, sender, _) = c.violation();
        assert_eq!(reason, Reason::Handle(Refusal::NotByCreator));
        assert_eq!(reason.fault_kind(), FaultKind::Identity);
        assert_eq!(sender, ATTESTED);

        // Impossible counters on a transferred sender.
        let mut c = Case::new(&[WireKind::Sender]);
        let mut d = desc(WireKind::Sender);
        d.delivered = 9;
        c.bytes = encode_data(0, &[d], &[]).unwrap();
        let (reason, _, _) = c.violation();
        assert_eq!(reason, Reason::Counters(FlowError::BadTransfer));
        assert_eq!(reason.fault_kind(), FaultKind::Sequence);
    }

    #[test]
    fn facts_out_of_step_with_the_descriptors_close_without_blame() {
        let mut c = Case::new(&[WireKind::Sender]);
        c.facts[0].fd = 99;
        assert_eq!(c.run(), (Decision::Closed, vec![10], None));
        let mut c = Case::new(&[WireKind::Sender]);
        c.facts.clear();
        assert_eq!(c.run().0, Decision::Closed);
    }

    #[test]
    fn sequence_faults_are_not_attributed() {
        let mut c = Case::new(&[WireKind::Sender]);
        c.flow.received = 1;
        c.flow.delivered = 1;
        let (reason, sender, close) = c.violation();
        assert_eq!(reason, Reason::Sequence(FlowError::Sequence));
        assert_eq!((sender, close), (None, vec![10]));
        let fault = Decision::Violation { reason, sender }.fault();
        assert_eq!(
            fault,
            Some(Fault {
                kind: FaultKind::Sequence,
                sender: None
            })
        );

        let mut c = Case::new(&[]);
        c.flow = ReceiverFlow::new(0);
        assert_eq!(c.violation().0, Reason::Sequence(FlowError::Overrun));
    }

    #[test]
    fn a_bound_receiver_latches_its_first_sender() {
        let mut c = Case::new(&[]);
        c.binding = Binding {
            bound: true,
            latched: None,
        };
        let (d, _, latch) = c.run();
        assert!(matches!(d, Decision::Accept { .. }));
        assert_eq!(latch, Some(PEER));

        let mut c = Case::new(&[]);
        c.binding = Binding {
            bound: true,
            latched: Some(PEER),
        };
        let (d, _, latch) = c.run();
        assert!(matches!(d, Decision::Accept { .. }));
        assert_eq!(latch, None);

        let mut c = Case::new(&[WireKind::Sender]);
        c.binding = Binding {
            bound: true,
            latched: Some(PEER + 1),
        };
        let (reason, sender, close) = c.violation();
        assert_eq!(
            (reason, sender, close),
            (Reason::WrongPeer, ATTESTED, vec![10])
        );
        assert_eq!(reason.fault_kind(), FaultKind::Identity);

        let mut c = Case::new(&[]);
        c.binding = Binding {
            bound: true,
            latched: None,
        };
        c.control = ParsedControl::from_parts(&[], &[], 0, None, Ok(()));
        assert_eq!(c.violation(), (Reason::NoCredentials, None, vec![]));
    }
}
