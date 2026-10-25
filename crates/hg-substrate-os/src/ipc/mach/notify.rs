//! Which received messages are the kernel's own notifications, and what
//! each means to the transport.
//!
//! The kernel's notifications carry an all-zero audit token, which no
//! process can produce (P2, P4). A peer cannot forge one; it could only
//! make the kernel send one for it, by naming a send-once right to our
//! port as a request's notify port. The transport gives no peer such a
//! right: the only send-once rights it makes are its credit ports'
//! reply rights, which go to the channel's receiver, and the no-senders
//! request each data port holds on itself, which stays with the kernel
//! and is cancelled before the port moves. A received data port with
//! any send-once right out is refused (`accept`, R8), so no former
//! holder keeps one. As a second fence, a no-senders notification
//! counts only with its exact size, and the IO layer acts on it only
//! for a port whose request it armed itself.
//!
//! What each means:
//! - **No senders** (70) on a data port: every send right to it is gone.
//!   The request is aimed at the data port itself, so the notification
//!   is queued behind every message already sent (P20), and the channel
//!   reports `Closed` after them.
//! - **Send-once** (71): a send-once right to the port was destroyed
//!   unused (P36): a receiver dropped a credit reply right, or this
//!   process cancelled a request. Nothing to do.
//! - **Send possible** (66), on the process's notify port: a queue that
//!   was full has room (P19); the outboxes are flushed.
//! - **Port destroyed** (69): carries a receive right back (P10); it is
//!   released.
//! - **Dead name** (72) and **port deleted** (65): a send-possible
//!   request's port went away, or the name did (P19); nothing to do but
//!   release what it carries.
//!
//! Any other message with the kernel's token is released and ignored.

use super::wire::{
    Header, NO_SENDERS_SIZE, NOTIFY_DEAD_NAME, NOTIFY_NO_SENDERS, NOTIFY_PORT_DELETED,
    NOTIFY_PORT_DESTROYED, NOTIFY_SEND_ONCE, NOTIFY_SEND_POSSIBLE, Token, WireError,
};

/// What a received message is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// From a process: a data or credit record for the transport's rules.
    Peer(Token),
    /// The port's last send right went away.
    NoSenders,
    /// A queue that was full has room.
    SendPossible,
    /// A kernel message with nothing to do but release what it carries.
    Ignore,
    /// No token at all: the kernel always writes one when asked (P1), so
    /// this is the receiver's own mistake; the channel ends unblamed.
    NoToken,
}

/// Classifies a message by its token and header.
pub(crate) fn classify(header: &Header, token: Result<Token, WireError>) -> Kind {
    let Ok(t) = token else {
        return Kind::NoToken;
    };
    if !t.is_kernel() {
        return Kind::Peer(t);
    }
    match header.id {
        NOTIFY_NO_SENDERS if !header.complex() && header.size == NO_SENDERS_SIZE => Kind::NoSenders,
        NOTIFY_SEND_POSSIBLE if !header.complex() => Kind::SendPossible,
        NOTIFY_SEND_ONCE | NOTIFY_PORT_DESTROYED | NOTIFY_DEAD_NAME | NOTIFY_PORT_DELETED => {
            Kind::Ignore
        }
        _ => Kind::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::mach::wire::{BITS_COMPLEX, ID_DATA};

    fn h(id: u32, complex: bool) -> Header {
        Header {
            bits: if complex { BITS_COMPLEX } else { 0 },
            id,
            size: if id == 70 { 36 } else { 24 },
            ..Header::default()
        }
    }

    const KERNEL: Token = Token { pid: 0, version: 0 };

    #[test]
    fn only_the_kernel_s_token_makes_a_notification() {
        let peer = Token {
            pid: 55,
            version: 3,
        };
        assert_eq!(classify(&h(70, false), Ok(peer)), Kind::Peer(peer));
        assert_eq!(classify(&h(70, false), Ok(KERNEL)), Kind::NoSenders);
        assert_eq!(classify(&h(70, true), Ok(KERNEL)), Kind::Ignore);
        // A no-senders id of any other size is not one.
        for size in [24, 32, 40, 1000] {
            let mut x = h(70, false);
            x.size = size;
            assert_eq!(classify(&x, Ok(KERNEL)), Kind::Ignore);
        }
        assert_eq!(classify(&h(66, false), Ok(KERNEL)), Kind::SendPossible);
        assert_eq!(classify(&h(71, false), Ok(KERNEL)), Kind::Ignore);
        assert_eq!(classify(&h(69, true), Ok(KERNEL)), Kind::Ignore);
        assert_eq!(classify(&h(ID_DATA, false), Ok(KERNEL)), Kind::Ignore);
        // Pid 0 with a nonzero version is no kernel message.
        let odd = Token { pid: 0, version: 1 };
        assert_eq!(classify(&h(70, false), Ok(odd)), Kind::Peer(odd));
        assert_eq!(
            classify(&h(70, false), Err(WireError::NoTrailer)),
            Kind::NoToken
        );
    }
}

#[cfg(kani)]
mod proofs {
    use super::*;

    /// A peer's token is always a peer's message; only the kernel's
    /// all-zero token makes a notification, and no-senders needs its id,
    /// its exact size and no descriptors; send-possible its id and no
    /// descriptors.
    #[kani::proof]
    fn classify_trusts_only_the_kernel_s_exact_notifications() {
        let header = Header {
            bits: kani::any(),
            size: kani::any(),
            remote: kani::any(),
            local: kani::any(),
            voucher: kani::any(),
            id: kani::any(),
        };
        let token: Result<Token, WireError> = if kani::any() {
            Ok(Token {
                pid: kani::any(),
                version: kani::any(),
            })
        } else {
            Err(WireError::NoTrailer)
        };
        let k = classify(&header, token);
        match token {
            Err(_) => assert!(k == Kind::NoToken),
            Ok(t) if !t.is_kernel() => assert!(k == Kind::Peer(t)),
            Ok(_) => {
                assert!(!matches!(k, Kind::Peer(_) | Kind::NoToken));
                if k == Kind::NoSenders {
                    assert!(header.id == NOTIFY_NO_SENDERS);
                    assert!(header.size == NO_SENDERS_SIZE);
                    assert!(!header.complex());
                }
                if k == Kind::SendPossible {
                    assert!(header.id == NOTIFY_SEND_POSSIBLE);
                    assert!(!header.complex());
                }
            }
        }
        kani::cover!(k == Kind::NoSenders, "a no-senders notification");
        kani::cover!(k == Kind::SendPossible, "a send-possible notification");
        kani::cover!(k == Kind::Ignore, "an ignored kernel message");
    }
}
