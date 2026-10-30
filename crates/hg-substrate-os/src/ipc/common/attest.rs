//! How a bound receiver's attestation resolves (design review M7; launch
//! design E-R6, Kani harness 5): one rule every backend applies before it
//! starts a bound listener.
//!
//! [`PeerAttest::Inherited`] binds a receiver by the channel itself, and
//! only the process's own bootstrap receiver has a channel that attests:
//! it came at birth, and only the launcher holds its sender. Any other
//! receiver asked to listen that way is refused.

use hg_substrate::ipc::{OsPeer, PeerAttest};

/// What a bound listener checks on each record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Bound {
    /// The OS-attested sender must be this process.
    Peer(OsPeer),
    /// The channel attests; no sender check (the bootstrap receiver).
    Inherited,
}

/// Resolves `attest` for a receiver; `is_bootstrap` says whether it is
/// the receiver `Ipc::bootstrap` returned. `None`: refused.
pub(crate) const fn resolve(attest: PeerAttest, is_bootstrap: bool) -> Option<Bound> {
    match attest {
        PeerAttest::Os(p) => Some(Bound::Peer(p)),
        PeerAttest::Inherited if is_bootstrap => Some(Bound::Inherited),
        PeerAttest::Inherited => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherited_binds_only_the_bootstrap_receiver() {
        assert_eq!(resolve(PeerAttest::Inherited, true), Some(Bound::Inherited));
        assert_eq!(resolve(PeerAttest::Inherited, false), None);
        let p = OsPeer::new(7, 1);
        assert_eq!(resolve(PeerAttest::Os(p), false), Some(Bound::Peer(p)));
        assert_eq!(resolve(PeerAttest::Os(p), true), Some(Bound::Peer(p)));
    }
}

#[cfg(kani)]
mod proofs {
    use super::{Bound, PeerAttest, resolve};
    use hg_substrate::ipc::OsPeer;

    /// Harness 5 (E-R11): an inherited binding exists only on the
    /// bootstrap receiver, and an OS binding is never weakened to one.
    #[kani::proof]
    fn inherited_is_accepted_only_on_the_bootstrap_receiver() {
        let inherited: bool = kani::any();
        let peer = OsPeer::new(kani::any(), kani::any());
        let attest = if inherited {
            PeerAttest::Inherited
        } else {
            PeerAttest::Os(peer)
        };
        let is_bootstrap: bool = kani::any();
        let r = resolve(attest, is_bootstrap);
        if r == Some(Bound::Inherited) {
            assert!(inherited && is_bootstrap);
        }
        if !inherited {
            assert!(r == Some(Bound::Peer(peer)));
        }
        if inherited && !is_bootstrap {
            assert!(r.is_none());
        }
        kani::cover!(r == Some(Bound::Inherited), "the bootstrap receiver binds");
    }
}
