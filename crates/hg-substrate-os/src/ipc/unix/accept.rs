//! Which received descriptors become handles (IPC-R1, IPC-R2 (a),
//! security scope SEC-4).
//!
//! A descriptor arrives already installed, with a kind its sender
//! *claims*. The transport believes nothing the sender says about it:
//! the sys module asks the kernel (`fstat(2)` for the file type and
//! `(st_dev, st_ino)`, and `getsockopt(2)` for `SO_DOMAIN`, `SO_TYPE`,
//! `SO_PROTOCOL` and `SO_PEERCRED`, socket(7)) and hands the answers here
//! as [`FdFacts`]. Only the ends of our own channels pass: Unix-domain
//! `SOCK_SEQPACKET` sockets with protocol 0. Files, pipes, memory files,
//! other sockets and the standard descriptors are refused, and so is the
//! whole record.
//!
//! The kernel cannot say which end of a socket pair a descriptor is, so a
//! sender end claimed as a receiver (or the reverse) passes. That is
//! harmless: it lets an attacker talk to its victim only over a channel
//! the attacker controls at both ends, and credit reads use no control
//! buffer, so the wrong end's traffic never installs a descriptor.
//!
//! Shared-memory regions (M1 phase D6) are memfds, checked by their seals
//! ([`accept_region`]): seals are a property of the file, so a holder
//! that reopens its descriptor read-write through `/proc` gains nothing,
//! and only a memfd made with `MFD_ALLOW_SEALING` can carry a write seal
//! (facts L4, L5, L7 in `research_notes/M1 D6 shared memory`).
//!
//! A bound sender is different: it may be sent only by the process that
//! made it (IPC-R2 (a)). `SO_PEERCRED` on a socket-pair end names the
//! process that called `socketpair(2)`, and no later holder can change
//! it; the record's `SCM_CREDENTIALS` name the process that sent it. A
//! bound sender passes only when the two agree, so the receiving side
//! checks "sent by its creator" itself.

use hg_substrate::ipc::{HandleKind, MAX_MESSAGE_HANDLES};

#[cfg(test)]
use crate::ipc::common::codec::REGION_IMMUTABLE;
use crate::ipc::common::codec::{HandleDesc, REGION_FILLER_TRUSTED, WireKind};

/// `AF_UNIX`, the Unix-domain address family (unix(7)).
pub(crate) const AF_UNIX: i32 = 1;

/// `SOCK_SEQPACKET`, the socket type of every channel (socket(2)).
pub(crate) const SOCK_SEQPACKET: i32 = 5;

/// `F_SEAL_SEAL`, `F_SEAL_SHRINK`, `F_SEAL_GROW`, `F_SEAL_WRITE`,
/// `F_SEAL_FUTURE_WRITE` (fcntl(2); values as the `libc` crate compiles
/// them, fact L0) and the exec seal `MFD_NOEXEC_SEAL` sets (0x20, read
/// back black-box, fact L13).
pub(crate) const SEAL_SEAL: i32 = 0x1;
pub(crate) const SEAL_SHRINK: i32 = 0x2;
pub(crate) const SEAL_GROW: i32 = 0x4;
pub(crate) const SEAL_WRITE: i32 = 0x8;
pub(crate) const SEAL_FUTURE_WRITE: i32 = 0x10;
pub(crate) const SEAL_EXEC: i32 = 0x20;

/// A writer's seals, exactly: its size fixed, not executable, nothing
/// else yet (review M5).
pub(crate) const WRITER_SEALS: i32 = SEAL_GROW | SEAL_SHRINK | SEAL_EXEC;
/// What a read-only region's seals must include.
pub(crate) const READ_ONLY_SEALS: i32 = WRITER_SEALS | SEAL_WRITE | SEAL_SEAL;
/// What a consumer region's seals must include.
pub(crate) const CONSUMER_SEALS: i32 = WRITER_SEALS | SEAL_FUTURE_WRITE | SEAL_SEAL;

/// The highest standard descriptor (stderr). Descriptors 0 to 2 are
/// never handles: a process with a closed stdio slot could otherwise be
/// handed a channel there, and later stdio writes would go into it.
pub(crate) const LAST_STDIO_FD: i32 = 2;

/// What the kernel says about one received descriptor. The sys module
/// fills it; a query that fails leaves a value no rule accepts (`false`,
/// or -1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct FdFacts {
    /// The descriptor number.
    pub(crate) fd: i32,
    /// `fstat(2)`: `S_ISSOCK(st_mode)`.
    pub(crate) is_socket: bool,
    /// `SO_DOMAIN`.
    pub(crate) domain: i32,
    /// `SO_TYPE`.
    pub(crate) sock_type: i32,
    /// `SO_PROTOCOL`.
    pub(crate) protocol: i32,
    /// `st_dev`.
    pub(crate) dev: u64,
    /// `st_ino`.
    pub(crate) ino: u64,
    /// `SO_PEERCRED`'s pid: for a socket-pair end, the process that made
    /// the pair.
    pub(crate) peercred_pid: i32,
    /// `fstat(2)`: `S_ISREG(st_mode)` (a memfd is a regular file).
    pub(crate) is_regular: bool,
    /// `st_size`.
    pub(crate) size: i64,
    /// `F_GET_SEALS`, or -1 for a file that cannot be sealed.
    pub(crate) seals: i32,
}

/// Which sockets this process already holds, by `(st_dev, st_ino)`.
/// A received end this process holds is its own channel coming back (or
/// a dup of it), which no honest peer sends.
pub(crate) trait Held {
    /// Whether this process already holds the socket `(dev, ino)`.
    fn already_held(&self, dev: u64, ino: u64) -> bool;
}

impl<F: Fn(u64, u64) -> bool> Held for F {
    fn already_held(&self, dev: u64, ino: u64) -> bool {
        self(dev, ino)
    }
}

/// What acceptance needs beyond the descriptor itself.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AcceptCtx<'a, H: Held> {
    /// The carrying record's `SCM_CREDENTIALS` pid, if it had any.
    pub(crate) cred_pid: Option<i32>,
    /// This process's held sockets.
    pub(crate) held: &'a H,
    /// The broker's pid, the only process that may send a region writer:
    /// the process that launched this one. `None` in the broker itself,
    /// which receives no writer (D6 review M1).
    pub(crate) broker_pid: Option<i32>,
    /// The record arrived on the inherited bootstrap channel, which only
    /// the broker can send on (design review M7, M8): a child in a PID
    /// namespace reads the broker as pid 0, so this is how it knows a
    /// writer or a bound sender came from the broker.
    pub(crate) broker_channel: bool,
}

impl<H: Held> AcceptCtx<'_, H> {
    /// Whether the record is the broker's: on the inherited channel, or
    /// with the broker's attested pid.
    const fn is_from_broker(&self) -> bool {
        self.broker_channel
            || matches!((self.cred_pid, self.broker_pid), (Some(c), Some(b)) if c == b && b > 0)
    }
}

/// Why a descriptor was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Refusal {
    /// A standard descriptor (0 to 2) or a negative number.
    Stdio,
    /// Not a socket.
    NotSocket,
    /// A socket of another family.
    Domain,
    /// A socket of another type.
    Type,
    /// A nonzero protocol.
    Protocol,
    /// A socket this process already holds.
    AlreadyHeld,
    /// The same socket twice in one record.
    Duplicate,
    /// A bound sender not sent by the process that made it.
    NotByCreator,
    /// Claimed as a region but not a sealable regular file.
    NotRegion,
    /// A region whose seals are not its kind's.
    Seals,
    /// A region whose size is not its declared length.
    Size,
    /// A region flag no Linux sender sets (`REGION_FILLER_TRUSTED`).
    Flags,
    /// A region writer from a process other than the broker.
    WriterNotFromBroker,
}

/// The handle kind a claim maps to.
pub(crate) const fn handle_kind(claimed: WireKind) -> HandleKind {
    match claimed {
        WireKind::Sender => HandleKind::ChannelSender,
        WireKind::BoundSender => HandleKind::BoundChannelSender,
        WireKind::Receiver => HandleKind::ChannelReceiver,
        WireKind::RegionWriter => HandleKind::RegionWriter,
        WireKind::ReadOnlyRegion => HandleKind::ReadOnlyRegion,
        WireKind::ConsumerRegion => HandleKind::ConsumerRegion,
    }
}

/// Decides one descriptor claimed as a region (D6). In order:
/// - not a standard descriptor;
/// - a sealable regular file (only a memfd made with `MFD_ALLOW_SEALING`
///   can carry the seals below, fact L7);
/// - seals: a writer's exactly [`WRITER_SEALS`]; a read-only region's
///   at least [`READ_ONLY_SEALS`]; a consumer's at least
///   [`CONSUMER_SEALS`] (a consumer region claimed as read-only lacks
///   `F_SEAL_WRITE` and is refused, so a changing region never passes as
///   an unchanging one);
/// - the size exactly the declared length (`desc.capacity`);
/// - no flag but `REGION_IMMUTABLE`;
/// - a writer only from the broker, as the record's credentials attest.
///
/// # Errors
/// The [`Refusal`]; the caller then refuses the whole record.
pub(crate) fn accept_region<H: Held>(
    desc: &HandleDesc,
    facts: &FdFacts,
    ctx: &AcceptCtx<'_, H>,
) -> Result<HandleKind, Refusal> {
    if facts.fd <= LAST_STDIO_FD {
        return Err(Refusal::Stdio);
    }
    if !facts.is_regular || facts.seals < 0 {
        return Err(Refusal::NotRegion);
    }
    let seals_ok = match desc.kind {
        WireKind::RegionWriter => facts.seals == WRITER_SEALS,
        WireKind::ReadOnlyRegion => facts.seals & READ_ONLY_SEALS == READ_ONLY_SEALS,
        WireKind::ConsumerRegion => facts.seals & CONSUMER_SEALS == CONSUMER_SEALS,
        WireKind::Sender | WireKind::BoundSender | WireKind::Receiver => false,
    };
    if !seals_ok {
        return Err(Refusal::Seals);
    }
    if i64::from(desc.capacity) != facts.size {
        return Err(Refusal::Size);
    }
    if desc.sent & REGION_FILLER_TRUSTED != 0 {
        return Err(Refusal::Flags);
    }
    if desc.kind == WireKind::RegionWriter && !ctx.is_from_broker() {
        return Err(Refusal::WriterNotFromBroker);
    }
    Ok(handle_kind(desc.kind))
}

/// Decides one descriptor. The checks run in a fixed order, so the
/// reason names the first rule broken.
///
/// # Errors
/// The [`Refusal`]; the caller then refuses the whole record.
pub(crate) fn accept<H: Held>(
    claimed: WireKind,
    facts: &FdFacts,
    ctx: &AcceptCtx<'_, H>,
) -> Result<HandleKind, Refusal> {
    if facts.fd <= LAST_STDIO_FD {
        return Err(Refusal::Stdio);
    }
    // A region claim is decided by `accept_region`, never here.
    if claimed.is_region() {
        return Err(Refusal::NotRegion);
    }
    if !facts.is_socket {
        return Err(Refusal::NotSocket);
    }
    if facts.domain != AF_UNIX {
        return Err(Refusal::Domain);
    }
    if facts.sock_type != SOCK_SEQPACKET {
        return Err(Refusal::Type);
    }
    if facts.protocol != 0 {
        return Err(Refusal::Protocol);
    }
    if ctx.held.already_held(facts.dev, facts.ino) {
        return Err(Refusal::AlreadyHeld);
    }
    // A bound sender only from its creator: its creator's attested pid,
    // or on the inherited channel a creator outside this PID namespace
    // (pid 0), which can only be the broker, since the broker sends only
    // bound senders it made.
    let by_creator = (facts.peercred_pid > 0 && ctx.cred_pid == Some(facts.peercred_pid))
        || (ctx.broker_channel && facts.peercred_pid == 0);
    if claimed == WireKind::BoundSender && !by_creator {
        return Err(Refusal::NotByCreator);
    }
    Ok(handle_kind(claimed))
}

/// Refuses a record that carries one socket twice (the same
/// `(st_dev, st_ino)` under two descriptor numbers): the two would become
/// two handles to one end.
///
/// # Errors
/// [`Refusal::Duplicate`].
pub(crate) fn distinct(facts: &[FdFacts]) -> Result<(), Refusal> {
    // Both loops are bounded by the constant (VER-1 cost); a record has
    // at most MAX_MESSAGE_HANDLES descriptors, and any beyond are the
    // count check's to refuse.
    for (i, a) in facts.iter().enumerate().take(MAX_MESSAGE_HANDLES) {
        for b in facts.iter().take(i) {
            if (a.dev, a.ino) == (b.dev, b.ino) {
                return Err(Refusal::Duplicate);
            }
        }
    }
    Ok(())
}

/// VER-1 for handle acceptance (SEC-4 (b): "a per-OS Kani proof covers
/// handle acceptance").
#[cfg(kani)]
pub(crate) mod proofs {
    use super::*;

    /// Any claimed kind.
    pub(crate) fn any_kind() -> WireKind {
        match kani::any::<u8>() {
            0 => WireKind::Sender,
            1 => WireKind::BoundSender,
            _ => WireKind::Receiver,
        }
    }

    /// Any kernel facts.
    pub(crate) fn any_facts() -> FdFacts {
        FdFacts {
            fd: kani::any(),
            is_socket: kani::any(),
            domain: kani::any(),
            sock_type: kani::any(),
            protocol: kani::any(),
            dev: kani::any(),
            ino: kani::any(),
            peercred_pid: kani::any(),
            is_regular: kani::any(),
            size: kani::any(),
            seals: kani::any(),
        }
    }

    /// Any region claim, with a descriptor the codec accepts.
    fn any_region_desc() -> HandleDesc {
        let kind = match kani::any::<u8>() % 3 {
            0 => WireKind::RegionWriter,
            1 => WireKind::ReadOnlyRegion,
            _ => WireKind::ConsumerRegion,
        };
        let len: u32 = kani::any();
        let flags: u64 = kani::any();
        kani::assume(len > 0 && flags <= 3);
        HandleDesc::region(kind, len, flags)
    }

    /// D6: a region claim is accepted exactly when its stated rules hold
    /// (review minor 3: a writer only from the broker; a consumer never
    /// as read-only), for every fact, credential and broker.
    #[kani::proof]
    fn regions_accept_exactly_the_stated_set() {
        let d = any_region_desc();
        let f = any_facts();
        let (cred_pid, broker_pid): (Option<i32>, Option<i32>) = (kani::any(), kani::any());
        let broker_channel: bool = kani::any();
        let answer = |_dev: u64, _ino: u64| false;
        let ctx = AcceptCtx {
            cred_pid,
            held: &answer,
            broker_pid,
            broker_channel,
        };
        let got = accept_region(&d, &f, &ctx);
        // The spec, written from fcntl(2)'s seal bits by position (SEAL 0,
        // SHRINK 1, GROW 2, WRITE 3, FUTURE_WRITE 4, EXEC 5) and the
        // design's table, not from the constants the code uses.
        let bit = |n: u32| f.seals >= 0 && (f.seals >> n) & 1 == 1;
        let base = bit(1) && bit(2) && bit(5);
        let seals_ok = match d.kind {
            // Exactly grow, shrink and exec: nothing else, no unknown bit.
            WireKind::RegionWriter => base && !bit(0) && !bit(3) && !bit(4) && f.seals >> 6 == 0,
            WireKind::ReadOnlyRegion => base && bit(3) && bit(0),
            _ => base && bit(4) && bit(0),
        };
        let from_broker = broker_channel
            || (cred_pid.is_some() && cred_pid == broker_pid && broker_pid > Some(0));
        let size_ok = i128::from(f.size) == i128::from(d.capacity);
        let spec = f.fd >= 3
            && f.is_regular
            && seals_ok
            && size_ok
            && d.sent & 2 == 0
            && (d.kind != WireKind::RegionWriter || from_broker);
        assert!(got.is_ok() == spec);
        if let Ok(k) = got {
            assert!(k == handle_kind(d.kind));
        }
        // A consumer region (no write seal) never passes as read-only.
        if d.kind == WireKind::ReadOnlyRegion && f.seals & SEAL_WRITE == 0 {
            assert!(got.is_err());
        }
        kani::cover!(
            got.is_ok() && d.kind == WireKind::RegionWriter,
            "a writer from the broker"
        );
        kani::cover!(
            got == Err(Refusal::WriterNotFromBroker),
            "a writer from anyone else"
        );
        kani::cover!(
            got.is_ok() && d.kind == WireKind::ConsumerRegion,
            "a consumer"
        );
    }

    /// For every claim, every combination of kernel facts (every mode,
    /// family, type, protocol, sentinel and descriptor number), every
    /// credential and every answer to "already held": accepted exactly
    /// when the stated rules all hold, and then as the claimed kind.
    #[kani::proof]
    fn accepts_exactly_the_stated_set() {
        let claimed = any_kind();
        let f = any_facts();
        let cred_pid: Option<i32> = kani::any();
        let held: bool = kani::any();
        let broker_channel: bool = kani::any();
        let answer = move |_dev: u64, _ino: u64| held;
        let ctx = AcceptCtx {
            cred_pid,
            held: &answer,
            broker_pid: kani::any(),
            broker_channel,
        };
        let got = accept(claimed, &f, &ctx);
        let spec = f.fd > 2
            && f.is_socket
            && f.domain == AF_UNIX
            && f.sock_type == SOCK_SEQPACKET
            && f.protocol == 0
            && !held
            && (claimed != WireKind::BoundSender
                || (f.peercred_pid > 0 && cred_pid == Some(f.peercred_pid))
                || (broker_channel && f.peercred_pid == 0));
        kani::cover!(
            got.is_ok() && claimed == WireKind::BoundSender,
            "a bound sender is accepted"
        );
        assert!(got.is_ok() == spec);
        if let Ok(k) = got {
            assert!(k == handle_kind(claimed));
            assert!(f.fd > 2);
        }
        // The kind map is one to one and onto the three kinds.
        assert!(handle_kind(WireKind::Sender) == HandleKind::ChannelSender);
        assert!(handle_kind(WireKind::BoundSender) == HandleKind::BoundChannelSender);
        assert!(handle_kind(WireKind::Receiver) == HandleKind::ChannelReceiver);
    }

    /// Up to 3 descriptors: refused exactly when two share
    /// `(dev, ino)`.
    #[kani::proof]
    #[kani::unwind(5)]
    fn duplicates_are_refused() {
        let all = [any_facts(), any_facts(), any_facts()];
        let n: usize = kani::any_where(|&n: &usize| n <= 3);
        let Some(fs) = all.get(..n) else { return };
        let mut dup = false;
        for i in 0..3 {
            for j in 0..3 {
                if i < j && j < n && (all[i].dev, all[i].ino) == (all[j].dev, all[j].ino) {
                    dup = true;
                }
            }
        }
        kani::cover!(n == 3 && !dup, "three distinct sockets");
        assert!(distinct(fs).is_err() == dup);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: FdFacts = FdFacts {
        fd: 7,
        is_socket: true,
        domain: AF_UNIX,
        sock_type: SOCK_SEQPACKET,
        protocol: 0,
        dev: 1,
        ino: 99,
        peercred_pid: 500,
        is_regular: false,
        size: 0,
        seals: -1,
    };

    fn none_held(_dev: u64, _ino: u64) -> bool {
        false
    }

    const BROKER: i32 = 300;

    /// A sealed region's facts: a memfd of 4096 bytes.
    const REGION: FdFacts = FdFacts {
        fd: 8,
        is_socket: false,
        domain: -1,
        sock_type: -1,
        protocol: -1,
        dev: 2,
        ino: 7,
        peercred_pid: -1,
        is_regular: true,
        size: 4096,
        seals: READ_ONLY_SEALS,
    };

    fn region(kind: WireKind, flags: u64) -> HandleDesc {
        HandleDesc::region(kind, 4096, flags)
    }

    #[test]
    fn each_region_kind_is_accepted_with_its_seals() {
        let c = ctx(Some(BROKER));
        let ro = region(WireKind::ReadOnlyRegion, REGION_IMMUTABLE);
        assert_eq!(
            accept_region(&ro, &REGION, &c),
            Ok(HandleKind::ReadOnlyRegion)
        );
        let mut consumer = REGION;
        consumer.seals = CONSUMER_SEALS;
        let cd = region(WireKind::ConsumerRegion, 0);
        assert_eq!(
            accept_region(&cd, &consumer, &c),
            Ok(HandleKind::ConsumerRegion)
        );
        let mut writer = REGION;
        writer.seals = WRITER_SEALS;
        let wd = region(WireKind::RegionWriter, 0);
        assert_eq!(
            accept_region(&wd, &writer, &c),
            Ok(HandleKind::RegionWriter)
        );
    }

    #[test]
    fn a_consumer_region_sent_as_read_only_is_refused() {
        let c = ctx(Some(BROKER));
        let mut consumer = REGION;
        consumer.seals = CONSUMER_SEALS;
        let ro = region(WireKind::ReadOnlyRegion, 0);
        assert_eq!(accept_region(&ro, &consumer, &c), Err(Refusal::Seals));
    }

    #[test]
    fn regions_with_wrong_facts_are_refused() {
        let c = ctx(Some(BROKER));
        let ro = region(WireKind::ReadOnlyRegion, 0);
        let with = |f: fn(&mut FdFacts)| {
            let mut x = REGION;
            f(&mut x);
            accept_region(&ro, &x, &c)
        };
        assert_eq!(with(|f| f.fd = 2), Err(Refusal::Stdio));
        assert_eq!(with(|f| f.is_regular = false), Err(Refusal::NotRegion));
        // Not sealable: F_GET_SEALS failed.
        assert_eq!(with(|f| f.seals = -1), Err(Refusal::NotRegion));
        assert_eq!(with(|f| f.seals = WRITER_SEALS), Err(Refusal::Seals));
        assert_eq!(with(|f| f.size = 4095), Err(Refusal::Size));
        assert_eq!(with(|f| f.size = 8192), Err(Refusal::Size));
        // A writer must carry exactly its seals: one with WRITE is not one.
        let wd = region(WireKind::RegionWriter, 0);
        assert_eq!(accept_region(&wd, &REGION, &c), Err(Refusal::Seals));
        // Filler-trust comes only from the Windows broker relay.
        let trusted = region(WireKind::ReadOnlyRegion, REGION_FILLER_TRUSTED);
        assert_eq!(accept_region(&trusted, &REGION, &c), Err(Refusal::Flags));
    }

    #[test]
    fn a_writer_comes_only_from_the_broker() {
        let mut writer = REGION;
        writer.seals = WRITER_SEALS;
        let wd = region(WireKind::RegionWriter, 0);
        for cred in [Some(BROKER + 1), None] {
            assert_eq!(
                accept_region(&wd, &writer, &ctx(cred)),
                Err(Refusal::WriterNotFromBroker)
            );
        }
        // In the broker itself there is no broker to receive from.
        let in_broker = AcceptCtx {
            cred_pid: Some(BROKER),
            held: &none_held,
            broker_pid: None,
            broker_channel: false,
        };
        assert_eq!(
            accept_region(&wd, &writer, &in_broker),
            Err(Refusal::WriterNotFromBroker)
        );
    }

    /// Design review M7, M8: a child in a PID namespace reads the broker
    /// as pid 0 (launch facts E12, E13). On its inherited bootstrap
    /// channel, which only the broker can send on, a writer and a bound
    /// sender made outside the namespace are the broker's; on any other
    /// channel they are refused.
    #[test]
    fn the_inherited_channel_stands_for_the_broker() {
        const NONE_HELD: fn(u64, u64) -> bool = none_held;
        let mut writer = REGION;
        writer.seals = WRITER_SEALS;
        let wd = region(WireKind::RegionWriter, 0);
        let mut bound = GOOD;
        bound.peercred_pid = 0;
        for broker_channel in [true, false] {
            let c = AcceptCtx {
                cred_pid: None,
                held: &NONE_HELD,
                broker_pid: None,
                broker_channel,
            };
            let (w, b) = if broker_channel {
                (
                    Ok(HandleKind::RegionWriter),
                    Ok(HandleKind::BoundChannelSender),
                )
            } else {
                (
                    Err(Refusal::WriterNotFromBroker),
                    Err(Refusal::NotByCreator),
                )
            };
            assert_eq!(accept_region(&wd, &writer, &c), w);
            assert_eq!(accept(WireKind::BoundSender, &bound, &c), b);
        }
        // A creator this process can see is never excused by the channel.
        let mut seen = GOOD;
        seen.peercred_pid = 4242;
        let c = AcceptCtx {
            cred_pid: None,
            held: &NONE_HELD,
            broker_pid: None,
            broker_channel: true,
        };
        assert_eq!(
            accept(WireKind::BoundSender, &seen, &c),
            Err(Refusal::NotByCreator)
        );
    }

    #[test]
    fn a_region_claim_on_the_channel_path_is_refused() {
        assert_eq!(
            accept(WireKind::ReadOnlyRegion, &REGION, &ctx(Some(BROKER))),
            Err(Refusal::NotRegion)
        );
    }

    fn ctx(cred_pid: Option<i32>) -> AcceptCtx<'static, fn(u64, u64) -> bool> {
        const NONE: fn(u64, u64) -> bool = none_held;
        AcceptCtx {
            cred_pid,
            held: &NONE,
            broker_pid: Some(BROKER),
            broker_channel: false,
        }
    }

    #[test]
    fn our_channel_ends_are_accepted_as_claimed() {
        let c = ctx(Some(500));
        assert_eq!(
            accept(WireKind::Sender, &GOOD, &c),
            Ok(HandleKind::ChannelSender)
        );
        assert_eq!(
            accept(WireKind::Receiver, &GOOD, &c),
            Ok(HandleKind::ChannelReceiver)
        );
        assert_eq!(
            accept(WireKind::BoundSender, &GOOD, &c),
            Ok(HandleKind::BoundChannelSender)
        );
    }

    #[test]
    fn every_other_descriptor_is_refused() {
        let c = ctx(Some(500));
        let with = |f: fn(&mut FdFacts)| {
            let mut x = GOOD;
            f(&mut x);
            accept(WireKind::Sender, &x, &c)
        };
        for fd in [-1, 0, 1, 2] {
            let mut x = GOOD;
            x.fd = fd;
            assert_eq!(accept(WireKind::Sender, &x, &c), Err(Refusal::Stdio));
        }
        assert_eq!(with(|f| f.is_socket = false), Err(Refusal::NotSocket));
        // AF_INET (2), AF_INET6 (10), AF_NETLINK (16), a failed query.
        for d in [2, 10, 16, -1] {
            let mut x = GOOD;
            x.domain = d;
            assert_eq!(accept(WireKind::Sender, &x, &c), Err(Refusal::Domain));
        }
        // SOCK_STREAM (1), SOCK_DGRAM (2), SOCK_RAW (3), a failed query.
        for t in [1, 2, 3, -1] {
            let mut x = GOOD;
            x.sock_type = t;
            assert_eq!(accept(WireKind::Sender, &x, &c), Err(Refusal::Type));
        }
        assert_eq!(with(|f| f.protocol = 1), Err(Refusal::Protocol));
        assert_eq!(with(|f| f.protocol = -1), Err(Refusal::Protocol));
        let ours = |dev: u64, ino: u64| (dev, ino) == (1, 99);
        let held = AcceptCtx {
            cred_pid: None,
            held: &ours,
            broker_pid: None,
            broker_channel: false,
        };
        assert_eq!(
            accept(WireKind::Receiver, &GOOD, &held),
            Err(Refusal::AlreadyHeld)
        );
    }

    #[test]
    fn a_bound_sender_needs_its_creator_as_sender() {
        let forwarded = accept(WireKind::BoundSender, &GOOD, &ctx(Some(501)));
        assert_eq!(forwarded, Err(Refusal::NotByCreator));
        let no_cred = accept(WireKind::BoundSender, &GOOD, &ctx(None));
        assert_eq!(no_cred, Err(Refusal::NotByCreator));
        for pid in [0, -1] {
            let mut x = GOOD;
            x.peercred_pid = pid;
            assert_eq!(
                accept(WireKind::BoundSender, &x, &ctx(Some(pid))),
                Err(Refusal::NotByCreator)
            );
        }
        // A free sender may be forwarded by anyone.
        assert!(accept(WireKind::Sender, &GOOD, &ctx(Some(501))).is_ok());
    }

    #[test]
    fn duplicates_within_a_record() {
        let mut b = GOOD;
        b.fd = 8;
        b.ino = 100;
        assert_eq!(distinct(&[]), Ok(()));
        assert_eq!(distinct(&[GOOD, b]), Ok(()));
        let mut dup = GOOD;
        dup.fd = 9;
        assert_eq!(distinct(&[GOOD, b, dup]), Err(Refusal::Duplicate));
    }
}
