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
//! A bound sender is different: it may be sent only by the process that
//! made it (IPC-R2 (a)). `SO_PEERCRED` on a socket-pair end names the
//! process that called `socketpair(2)`, and no later holder can change
//! it; the record's `SCM_CREDENTIALS` name the process that sent it. A
//! bound sender passes only when the two agree, so the receiving side
//! checks "sent by its creator" itself.

use hg_substrate::ipc::{HandleKind, MAX_MESSAGE_HANDLES};

use crate::ipc::common::codec::WireKind;

/// `AF_UNIX`, the Unix-domain address family (unix(7)).
pub(crate) const AF_UNIX: i32 = 1;

/// `SOCK_SEQPACKET`, the socket type of every channel (socket(2)).
pub(crate) const SOCK_SEQPACKET: i32 = 5;

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
}

/// The handle kind a claim maps to.
pub(crate) const fn handle_kind(claimed: WireKind) -> HandleKind {
    match claimed {
        WireKind::Sender => HandleKind::ChannelSender,
        WireKind::BoundSender => HandleKind::BoundChannelSender,
        WireKind::Receiver => HandleKind::ChannelReceiver,
    }
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
    if claimed == WireKind::BoundSender
        && !(facts.peercred_pid > 0 && ctx.cred_pid == Some(facts.peercred_pid))
    {
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
        }
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
        let answer = move |_dev: u64, _ino: u64| held;
        let ctx = AcceptCtx {
            cred_pid,
            held: &answer,
        };
        let got = accept(claimed, &f, &ctx);
        let spec = f.fd > 2
            && f.is_socket
            && f.domain == AF_UNIX
            && f.sock_type == SOCK_SEQPACKET
            && f.protocol == 0
            && !held
            && (claimed != WireKind::BoundSender
                || (f.peercred_pid > 0 && cred_pid == Some(f.peercred_pid)));
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
    };

    fn none_held(_dev: u64, _ino: u64) -> bool {
        false
    }

    fn ctx(cred_pid: Option<i32>) -> AcceptCtx<'static, fn(u64, u64) -> bool> {
        const NONE: fn(u64, u64) -> bool = none_held;
        AcceptCtx {
            cred_pid,
            held: &NONE,
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
