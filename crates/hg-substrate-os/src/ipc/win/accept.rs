//! Which handles the broker takes into a relay (IPC-R1, IPC-R2 (a);
//! security scope SEC-4).
//!
//! A sandboxed process cannot put a handle into another process: it has
//! no handle to any other process with the right to duplicate into it.
//! So every handle that moves goes through the broker. The child names
//! the handle by its value in its own table; the broker pulls it into its
//! own table (`DuplicateHandle` with `DUPLICATE_CLOSE_SOURCE`) and asks
//! the kernel what it got. Nothing the child says about the handle is
//! believed: these are the kernel's answers ([`Facts`]), and
//! [`accept`] takes a handle only when every one of them fits.
//!
//! The value comes first. Every value at or below 0 is refused before the
//! broker touches it: −1 names the calling process and −2 its thread, so
//! a pull of −2 from the child's table hands the broker a handle to its
//! *own* thread (probe P14, the CVE-2025-2783 shape). Windows ignores the
//! two low bits of a value (probe W2), so a value with them set would
//! name an end already named in the same relay; it is refused too.
//!
//! The object then: a message-mode pipe end of one instance, named as an
//! Izcal channel, of the end its claim says (a sender is a pipe's client
//! end, a receiver its server end), and with no other handle to it
//! anywhere once the broker holds it (`HandleCount` is counted across
//! every process, probe W1): a kept copy would let the sender keep
//! writing as whoever receives the end.
//!
//! The ledger last ([`super::ledger`]): the relaying process must be the
//! end's holder, a bound sender moves once and only from the process that
//! created the channel, a sender of a channel known to be bound may not
//! travel relabelled as free, and a receiver bound to a peer never moves.
//! The creator is attested by the pipe's name: a child can create pipes
//! only in its own AppContainer's namespace, whose path names the
//! container (probes P1, P2), and every child has its own container.

use crate::ipc::common::codec::WireKind;

/// Which end of a channel's pipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum End {
    /// The client end: a channel's sender.
    Sender,
    /// The server end: a channel's receiver.
    Receiver,
}

/// A process as the broker numbers it: 0 is the broker itself, each
/// child it starts the next number.
pub(crate) type Proc = u32;

/// A channel, as the broker knows it: the process whose namespace holds
/// the pipe (so its creator) and the 128 random bits of its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct PipeKey {
    /// The creating process.
    pub(crate) creator: Proc,
    /// The random part of the pipe's name.
    pub(crate) id: [u8; 16],
}

/// What the kernel says about one pulled handle. The sys layer fills it;
/// a query that fails leaves a value no rule accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Facts {
    /// The value the child named, as sent.
    pub(crate) raw: u64,
    /// The pull succeeded.
    pub(crate) pulled: bool,
    /// `GetFileType` is `FILE_TYPE_PIPE`.
    pub(crate) is_pipe: bool,
    /// `GetNamedPipeInfo`: the server end.
    pub(crate) server_end: bool,
    /// `GetNamedPipeInfo`: a message-type pipe.
    pub(crate) message: bool,
    /// `GetNamedPipeInfo`: the most instances the pipe allows.
    pub(crate) max_instances: u32,
    /// The channel its name names; `None` for a name that is not an
    /// Izcal channel's, or a namespace the broker did not give a child.
    pub(crate) pipe: Option<PipeKey>,
    /// `NtQueryObject` basic information: handles to the object in all
    /// processes, counted after the pull.
    pub(crate) handle_count: u32,
}

/// What the ledger knows about a channel, for one decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct View {
    /// The process holding the end in question, if one does.
    pub(crate) holder: Option<Proc>,
    /// The channel's sender was granted as a bound sender.
    pub(crate) bound: bool,
    /// The receiver is bound to one peer (`listen_bound`).
    pub(crate) bound_receiver: bool,
}

/// What a decision needs besides the handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ctx {
    /// The process relaying.
    pub(crate) relayer: Proc,
    /// The channel the record will travel on.
    pub(crate) carrier: PipeKey,
    /// The ledger's view of the handle's channel, if it has one. A
    /// channel the ledger has never seen is held, both ends, by its
    /// creator.
    pub(crate) view: Option<View>,
}

/// Why a handle was refused. Each is the relaying process's own doing:
/// it chose the handle and its claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Refusal {
    /// A value at or below 0: a pseudo-handle or null.
    NotPositive,
    /// A value with either of its two low bits set.
    TagBits,
    /// The value names nothing in the child's table.
    PullFailed,
    /// Not a pipe.
    NotPipe,
    /// A byte-mode pipe.
    NotMessage,
    /// A pipe that allows more than one instance.
    Instances,
    /// Not an Izcal channel's pipe.
    Foreign,
    /// The other end than the claim says.
    WrongEnd,
    /// Another handle to the end still exists somewhere.
    KeptCopy,
    /// An end of the very channel the record travels on.
    Carrier,
    /// The relaying process is not the end's holder.
    NotHolder,
    /// A bound sender from a process other than the channel's creator.
    NotByCreator,
    /// A bound sender's channel's sender, relabelled as free.
    Relabelled,
    /// The receiver of a bound channel, or of one bound to a peer.
    BoundReceiver,
    /// The same end twice in one relay.
    Duplicate,
    /// The relayer is over its ledger quota (tickets or channels).
    Quota,
    /// The broker had no randomness to make a ticket.
    NoTicket,
}

impl Refusal {
    /// The code a `RelayRefused` record carries.
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::NotPositive => 1,
            Self::TagBits => 2,
            Self::PullFailed => 3,
            Self::NotPipe => 4,
            Self::NotMessage => 5,
            Self::Instances => 6,
            Self::Foreign => 7,
            Self::WrongEnd => 8,
            Self::KeptCopy => 9,
            Self::Carrier => 10,
            Self::NotHolder => 11,
            Self::NotByCreator => 12,
            Self::Relabelled => 13,
            Self::BoundReceiver => 14,
            Self::Duplicate => 15,
            Self::Quota => 16,
            Self::NoTicket => 17,
        }
    }
}

/// The value's own checks, made before the broker pulls it.
///
/// # Errors
/// [`Refusal::NotPositive`] or [`Refusal::TagBits`].
pub(crate) const fn check_value(raw: u64) -> Result<(), Refusal> {
    let signed = i64::from_ne_bytes(raw.to_ne_bytes());
    if signed <= 0 {
        return Err(Refusal::NotPositive);
    }
    if raw & 3 != 0 {
        return Err(Refusal::TagBits);
    }
    Ok(())
}

/// The end a claim needs.
pub(crate) const fn end_for(claimed: WireKind) -> End {
    match claimed {
        WireKind::Sender | WireKind::BoundSender => End::Sender,
        WireKind::Receiver => End::Receiver,
    }
}

/// Decides one pulled handle. The checks run in a fixed order, so the
/// reason names the first rule broken.
///
/// # Errors
/// The [`Refusal`]; the broker then refuses the whole relay.
pub(crate) fn accept(claimed: WireKind, f: &Facts, ctx: &Ctx) -> Result<(PipeKey, End), Refusal> {
    check_value(f.raw)?;
    if !f.pulled {
        return Err(Refusal::PullFailed);
    }
    if !f.is_pipe {
        return Err(Refusal::NotPipe);
    }
    if !f.message {
        return Err(Refusal::NotMessage);
    }
    if f.max_instances != 1 {
        return Err(Refusal::Instances);
    }
    let Some(pipe) = f.pipe else {
        return Err(Refusal::Foreign);
    };
    let end = end_for(claimed);
    if f.server_end != (end == End::Receiver) {
        return Err(Refusal::WrongEnd);
    }
    if f.handle_count != 1 {
        return Err(Refusal::KeptCopy);
    }
    if pipe == ctx.carrier {
        return Err(Refusal::Carrier);
    }
    let view = ctx.view.unwrap_or(View {
        holder: Some(pipe.creator),
        bound: false,
        bound_receiver: false,
    });
    if view.holder != Some(ctx.relayer) {
        return Err(Refusal::NotHolder);
    }
    match claimed {
        WireKind::BoundSender => {
            if view.bound || pipe.creator != ctx.relayer {
                return Err(Refusal::NotByCreator);
            }
        }
        WireKind::Sender => {
            if view.bound {
                return Err(Refusal::Relabelled);
            }
        }
        WireKind::Receiver => {
            if view.bound || view.bound_receiver {
                return Err(Refusal::BoundReceiver);
            }
        }
    }
    Ok((pipe, end))
}

/// The prefix of a channel pipe's name; the rest is 32 lowercase hex
/// digits. Control links use another name, so a link is never taken for
/// a channel.
pub(crate) const CHANNEL_PREFIX: &str = "izcal-";

/// Parses a channel pipe's name as `GetFileInformationByHandleEx`
/// reports it, relative to the pipe device: `\izcal-<hex>` for a pipe
/// in the global namespace (the broker's), or
/// `\Sessions\<n>\AppContainerNamedObjects\<SID>\izcal-<hex>` for one in an
/// AppContainer's (probe P2). Returns the container's SID, if any, and
/// the name's random bits; `None` for any other name.
pub(crate) fn parse_name(name: &str) -> Option<(Option<&str>, [u8; 16])> {
    let rest = name.strip_prefix('\\')?;
    let mut parts = rest.split('\\');
    let first = parts.next()?;
    let (sid, last) = if first == "Sessions" {
        let session = parts.next()?;
        if session.is_empty() || !session.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        if parts.next()? != "AppContainerNamedObjects" {
            return None;
        }
        let sid = parts.next()?;
        if !sid.starts_with("S-1-15-2-") {
            return None;
        }
        (Some(sid), parts.next()?)
    } else {
        (None, first)
    };
    if parts.next().is_some() {
        return None;
    }
    let hex = last.strip_prefix(CHANNEL_PREFIX)?;
    if hex.len() != 32 {
        return None;
    }
    let mut id = [0u8; 16];
    let (pairs, _none) = hex.as_bytes().as_chunks::<2>();
    for (slot, pair) in id.iter_mut().zip(pairs) {
        let digit = |c: u8| match c {
            b'0'..=b'9' => Some(c.wrapping_sub(b'0')),
            b'a'..=b'f' => Some(c.wrapping_sub(b'a').wrapping_add(10)),
            _ => None,
        };
        let [hi, lo] = pair;
        *slot = digit(*hi)?.wrapping_shl(4) | digit(*lo)?;
    }
    Some((sid, id))
}

/// Refuses a relay that names one end twice.
///
/// # Errors
/// [`Refusal::Duplicate`].
pub(crate) fn distinct(ends: &[(PipeKey, End)]) -> Result<(), Refusal> {
    for (i, a) in ends.iter().enumerate() {
        for b in ends.iter().take(i) {
            if a == b {
                return Err(Refusal::Duplicate);
            }
        }
    }
    Ok(())
}

/// VER-1 for Windows handle acceptance (SEC-4 (b): "a per-OS Kani proof
/// covers handle acceptance").
#[cfg(kani)]
mod proofs {
    use super::*;

    fn any_kind() -> WireKind {
        match kani::any::<u8>() % 3 {
            0 => WireKind::Sender,
            1 => WireKind::BoundSender,
            _ => WireKind::Receiver,
        }
    }

    fn any_pipe() -> PipeKey {
        PipeKey {
            creator: kani::any_where(|&c: &u32| c < 4),
            id: [kani::any::<u8>() % 2; 16],
        }
    }

    fn any_facts() -> Facts {
        Facts {
            raw: kani::any(),
            pulled: kani::any(),
            is_pipe: kani::any(),
            server_end: kani::any(),
            message: kani::any(),
            max_instances: kani::any(),
            pipe: if kani::any() { Some(any_pipe()) } else { None },
            handle_count: kani::any(),
        }
    }

    fn any_ctx() -> Ctx {
        Ctx {
            relayer: kani::any_where(|&c: &u32| c < 4),
            carrier: any_pipe(),
            view: if kani::any() {
                Some(View {
                    holder: if kani::any() {
                        Some(kani::any_where(|&c: &u32| c < 4))
                    } else {
                        None
                    },
                    bound: kani::any(),
                    bound_receiver: kani::any(),
                })
            } else {
                None
            },
        }
    }

    /// Exactly the stated handles pass: for any facts, claim and ledger
    /// view, an accepted handle is a positive, untagged value naming a
    /// one-instance message pipe of the claimed end, which no one else
    /// holds a handle to, of another channel than the carrier, held by
    /// the relayer; a bound sender comes from its creator, once; a free
    /// sender is not a bound channel's; a receiver is not bound.
    #[kani::proof]
    fn accepts_exactly_the_stated_handles() {
        let claimed = any_kind();
        let f = any_facts();
        let ctx = any_ctx();
        let r = accept(claimed, &f, &ctx);
        if let Ok((pipe, end)) = r {
            let signed = i64::from_ne_bytes(f.raw.to_ne_bytes());
            assert!(signed > 0 && f.raw % 4 == 0);
            assert!(f.pulled && f.is_pipe && f.message && f.max_instances == 1);
            assert!(f.pipe == Some(pipe));
            assert!(f.handle_count == 1);
            assert!(pipe != ctx.carrier);
            assert!(end == end_for(claimed));
            assert!(f.server_end == (end == End::Receiver));
            let holder = ctx.view.map_or(Some(pipe.creator), |v| v.holder);
            assert!(holder == Some(ctx.relayer));
            let bound = ctx.view.is_some_and(|v| v.bound);
            let bound_rx = ctx.view.is_some_and(|v| v.bound_receiver);
            match claimed {
                WireKind::BoundSender => assert!(!bound && pipe.creator == ctx.relayer),
                WireKind::Sender => assert!(!bound),
                WireKind::Receiver => assert!(!bound && !bound_rx),
            }
        }
        kani::cover!(r.is_ok(), "a handle is accepted");
        kani::cover!(
            r == Err(Refusal::NotPositive) && f.raw == u64::MAX - 1,
            "the current-thread pseudo-handle (-2) is refused"
        );
        kani::cover!(r == Err(Refusal::TagBits), "a tagged value is refused");
        kani::cover!(r == Err(Refusal::KeptCopy), "a kept copy is refused");
        kani::cover!(
            r == Err(Refusal::NotByCreator),
            "a forwarded bound sender is refused"
        );
        kani::cover!(
            r == Err(Refusal::Relabelled),
            "a relabelled bound sender is refused"
        );
        kani::cover!(
            r == Err(Refusal::BoundReceiver),
            "a bound receiver is refused"
        );
    }

    /// No value at or below 0 is ever pulled.
    #[kani::proof]
    fn no_pseudo_handle_passes() {
        let raw: u64 = kani::any();
        let signed = i64::from_ne_bytes(raw.to_ne_bytes());
        if signed <= 0 {
            assert!(check_value(raw) == Err(Refusal::NotPositive));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BROKER: Proc = 0;
    const CHILD: Proc = 1;

    fn pipe(creator: Proc) -> PipeKey {
        PipeKey {
            creator,
            id: [7; 16],
        }
    }

    fn good(end: End) -> Facts {
        Facts {
            raw: 0x1a4,
            pulled: true,
            is_pipe: true,
            server_end: end == End::Receiver,
            message: true,
            max_instances: 1,
            pipe: Some(pipe(CHILD)),
            handle_count: 1,
        }
    }

    fn ctx() -> Ctx {
        Ctx {
            relayer: CHILD,
            carrier: PipeKey {
                creator: BROKER,
                id: [1; 16],
            },
            view: None,
        }
    }

    #[test]
    fn a_creators_own_ends_pass() {
        assert_eq!(
            accept(WireKind::Sender, &good(End::Sender), &ctx()),
            Ok((pipe(CHILD), End::Sender))
        );
        assert_eq!(
            accept(WireKind::BoundSender, &good(End::Sender), &ctx()),
            Ok((pipe(CHILD), End::Sender))
        );
        assert_eq!(
            accept(WireKind::Receiver, &good(End::Receiver), &ctx()),
            Ok((pipe(CHILD), End::Receiver))
        );
    }

    #[test]
    fn every_rule_refuses() {
        let c = ctx();
        let k = WireKind::Sender;
        let with = |m: fn(&mut Facts)| {
            let mut f = good(End::Sender);
            m(&mut f);
            accept(k, &f, &c)
        };
        assert_eq!(with(|f| f.raw = u64::MAX), Err(Refusal::NotPositive));
        assert_eq!(with(|f| f.raw = u64::MAX - 1), Err(Refusal::NotPositive));
        assert_eq!(with(|f| f.raw = 0), Err(Refusal::NotPositive));
        assert_eq!(with(|f| f.raw = 0x1a5), Err(Refusal::TagBits));
        assert_eq!(with(|f| f.raw = 0x1a6), Err(Refusal::TagBits));
        assert_eq!(with(|f| f.pulled = false), Err(Refusal::PullFailed));
        assert_eq!(with(|f| f.is_pipe = false), Err(Refusal::NotPipe));
        assert_eq!(with(|f| f.message = false), Err(Refusal::NotMessage));
        assert_eq!(with(|f| f.max_instances = 2), Err(Refusal::Instances));
        assert_eq!(with(|f| f.pipe = None), Err(Refusal::Foreign));
        assert_eq!(with(|f| f.server_end = true), Err(Refusal::WrongEnd));
        assert_eq!(with(|f| f.handle_count = 2), Err(Refusal::KeptCopy));
        let mut on_carrier = c;
        on_carrier.carrier = pipe(CHILD);
        assert_eq!(
            accept(k, &good(End::Sender), &on_carrier),
            Err(Refusal::Carrier)
        );
        // Another child's channel the ledger has never seen: its creator
        // holds it.
        let mut f = good(End::Sender);
        f.pipe = Some(pipe(2));
        assert_eq!(accept(k, &f, &c), Err(Refusal::NotHolder));
    }

    #[test]
    fn bound_rules() {
        let held = |bound, bound_receiver, holder| Ctx {
            view: Some(View {
                holder: Some(holder),
                bound,
                bound_receiver,
            }),
            ..ctx()
        };
        // Granted once already.
        assert_eq!(
            accept(
                WireKind::BoundSender,
                &good(End::Sender),
                &held(true, false, CHILD)
            ),
            Err(Refusal::NotByCreator)
        );
        // From its grantee, not its creator.
        let mut f = good(End::Sender);
        f.pipe = Some(pipe(BROKER));
        assert_eq!(
            accept(WireKind::BoundSender, &f, &held(false, false, CHILD)),
            Err(Refusal::NotByCreator)
        );
        // Relabelled as free.
        assert_eq!(
            accept(WireKind::Sender, &f, &held(true, false, CHILD)),
            Err(Refusal::Relabelled)
        );
        // A bound receiver never moves.
        assert_eq!(
            accept(
                WireKind::Receiver,
                &good(End::Receiver),
                &held(false, true, CHILD)
            ),
            Err(Refusal::BoundReceiver)
        );
    }

    #[test]
    fn channel_names_parse() {
        let hex = "00112233445566778899aabbccddeeff";
        let id = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        assert_eq!(parse_name(&format!("\\izcal-{hex}")), Some((None, id)));
        let sid = "S-1-15-2-1-2-3-4-5-6-7";
        assert_eq!(
            parse_name(&format!(
                "\\Sessions\\1\\AppContainerNamedObjects\\{sid}\\izcal-{hex}"
            )),
            Some((Some(sid), id))
        );
        for bad in [
            format!("izcal-{hex}"),
            format!("\\izcal-link-{hex}"),
            format!("\\izcal-{}", hex.get(1..).unwrap()),
            format!("\\izcal-{hex}0"),
            format!("\\izcal-{}", hex.to_uppercase()),
            format!("\\other-{hex}"),
            format!("\\Sessions\\x\\AppContainerNamedObjects\\{sid}\\izcal-{hex}"),
            format!("\\Sessions\\1\\BaseNamedObjects\\{sid}\\izcal-{hex}"),
            format!("\\Sessions\\1\\AppContainerNamedObjects\\S-1-5-21-1\\izcal-{hex}"),
            format!("\\Sessions\\1\\AppContainerNamedObjects\\{sid}\\izcal-{hex}\\x"),
        ] {
            assert_eq!(parse_name(&bad), None, "{bad}");
        }
    }

    #[test]
    fn a_handle_value_of_minus_one_is_refused() {
        assert_eq!(check_value(u64::MAX), Err(Refusal::NotPositive));
    }

    #[test]
    fn a_handle_value_of_minus_two_is_refused() {
        assert_eq!(check_value(u64::MAX - 1), Err(Refusal::NotPositive));
    }

    #[test]
    fn a_handle_value_of_zero_is_refused() {
        assert_eq!(check_value(0), Err(Refusal::NotPositive));
    }

    #[test]
    fn values_with_tag_bits_set_are_refused() {
        for v in [0x101, 0x102, 0x103] {
            assert_eq!(check_value(v), Err(Refusal::TagBits));
        }
        assert_eq!(check_value(0x100), Ok(()));
    }

    #[test]
    fn a_kept_copy_is_refused() {
        let mut f = good(End::Sender);
        f.handle_count = 2;
        assert_eq!(accept(WireKind::Sender, &f, &ctx()), Err(Refusal::KeptCopy));
    }

    #[test]
    fn a_handle_of_the_wrong_kind_is_refused() {
        let mut f = good(End::Sender);
        f.is_pipe = false;
        assert_eq!(accept(WireKind::Sender, &f, &ctx()), Err(Refusal::NotPipe));
        assert_eq!(
            accept(WireKind::Receiver, &good(End::Sender), &ctx()),
            Err(Refusal::WrongEnd)
        );
    }

    #[test]
    fn refusals_have_distinct_nonzero_codes() {
        let all = [
            Refusal::NotPositive,
            Refusal::TagBits,
            Refusal::PullFailed,
            Refusal::NotPipe,
            Refusal::NotMessage,
            Refusal::Instances,
            Refusal::Foreign,
            Refusal::WrongEnd,
            Refusal::KeptCopy,
            Refusal::Carrier,
            Refusal::NotHolder,
            Refusal::NotByCreator,
            Refusal::Relabelled,
            Refusal::BoundReceiver,
            Refusal::Duplicate,
            Refusal::Quota,
            Refusal::NoTicket,
        ];
        for (i, a) in all.iter().enumerate() {
            assert_ne!(a.code(), 0);
            for b in all.iter().skip(i + 1) {
                assert_ne!(a.code(), b.code());
            }
        }
    }

    #[test]
    fn duplicates_are_refused() {
        let a = (pipe(CHILD), End::Sender);
        let b = (pipe(CHILD), End::Receiver);
        assert_eq!(distinct(&[a, b]), Ok(()));
        assert_eq!(distinct(&[a, b, a]), Err(Refusal::Duplicate));
    }
}
