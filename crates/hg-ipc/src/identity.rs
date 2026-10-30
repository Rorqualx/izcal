//! Who is calling (security scope IPC-R2): identity comes from the
//! endpoint the broker bound, never from a message.
//!
//! The broker serves each bound endpoint with a [`PeerBinding`] naming the
//! peer it made the endpoint for. Every message on that endpoint reaches
//! its handler with a [`CallerIdentity`] built from that binding.
//! `CallerIdentity` has no public constructor, so no other code can make
//! one, and nothing in a message can change it. Free endpoints carry no
//! identity: their handlers get none.
//!
//! M1 identifies the peer process, and the binding names its OS process
//! too, so the transport itself refuses records from any other process. Site, origin, container, frame and
//! worker identity join as those types arrive (IPC-R2 (b)); the types are
//! `#[non_exhaustive]` for that.

use hg_substrate::ipc::OsPeer;

/// A peer process as the broker numbers it: an index and a generation,
/// so an identifier is never reused for a later process (IPC-R8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PeerId {
    index: u32,
    generation: u32,
}

impl PeerId {
    /// Peer `index`, incarnation `generation`. The broker's table assigns
    /// these; nothing else should.
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// The broker's index.
    pub const fn index(self) -> u32 {
        self.index
    }

    /// The incarnation.
    pub const fn generation(self) -> u32 {
        self.generation
    }
}

/// What the broker binds to an endpoint when it serves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct PeerBinding {
    /// The peer the endpoint was made for.
    pub peer: PeerId,
    /// That peer's process as the OS identifies it: the receiving
    /// transport accepts records from this process only
    /// ([`hg_substrate::ipc::Ipc::listen_bound`]).
    pub os: OsPeer,
}

impl PeerBinding {
    /// A binding to `peer`, whose process is `os`.
    pub const fn new(peer: PeerId, os: OsPeer) -> Self {
        Self { peer, os }
    }
}

/// The identity of the process at the other end of a bound endpoint, as
/// the broker bound it. Only `hg-ipc` constructs one.
///
/// ```compile_fail
/// // Not constructible outside hg-ipc (IPC-R2 (c)):
/// let forged = hg_ipc::CallerIdentity { peer: hg_ipc::PeerId::new(0, 0) };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct CallerIdentity {
    peer: PeerId,
}

impl CallerIdentity {
    pub(crate) const fn from_binding(b: PeerBinding) -> Self {
        Self { peer: b.peer }
    }

    /// The peer the endpoint was bound to.
    pub const fn peer(&self) -> PeerId {
        self.peer
    }
}
