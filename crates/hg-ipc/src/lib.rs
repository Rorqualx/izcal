//! Izcal's IPC runtime (Stage 1 design 4; HGBA-3 B2, VER-1; security
//! scope 6.6). Tier P: no panic on any input (CS-ER-3).
//!
//! Interfaces are declared in `.hgidl` files and compiled by
//! `hg-ipc-codegen` into `hg-ipc-gen`. The generated code builds on this
//! crate: the [`wire`] format, the [`bounded`] containers, and the
//! [`InterfaceInfo`] each interface carries.
//!
//! M1 phase D1 added the wire format and descriptors; D2 adds typed
//! endpoints over the substrate's IPC transport ([`endpoint`]) and the
//! caller identity ([`identity`]).

pub mod bounded;
pub mod endpoint;
pub mod identity;
mod utf8;
pub mod wire;

pub use bounded::{BoundedBytes, BoundedString, BoundedVec, OverBound};
pub use endpoint::{
    BoundInterface, EndReason, Endpoint, FreeInterface, Grant, Interface, Remote, SendError,
    Serving, bound_channel, channel,
};
pub use hg_substrate::ipc::{Handle, HandleKind, Ipc, MAX_MESSAGE_HANDLES};
pub use identity::{CallerIdentity, PeerBinding, PeerId};
pub use wire::{BadMessage, Reader, Wire, Writer, decode_exact, encode_to_vec, validate_exact};

/// The largest message any interface may declare: 64 KiB†. Bulk data
/// crosses as read-only shared memory, never as a large message (Stage 1
/// 4.1); `hg-ipc-codegen` rejects a message whose largest encoding is
/// over this.
pub const MAX_MESSAGE_LEN: usize = hg_substrate::ipc::MAX_MESSAGE_BYTES;

/// Whether an endpoint carries identity (IPC-R2 (a)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    /// Identity-bearing and non-transferable: the broker binds the
    /// caller's identity at creation, and no message can carry it.
    Bound,
    /// A data channel with no identity; it may be sent to another process.
    Free,
}

/// What a sender does when the peer's queue is full (CS-AS-6 (b)). A
/// sender never blocks on a peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    /// Drop the message and count it.
    Drop,
    /// Declare the peer unresponsive and ask the broker to terminate it.
    Terminate,
}

/// What `.hgidl` declared about one interface (one direction of an
/// endpoint).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterfaceInfo {
    /// `protocol.Interface`.
    pub name: &'static str,
    /// Bound or free.
    pub binding: Binding,
    /// Messages the receiving queue holds before [`Overflow`] applies.
    pub capacity: u32,
    /// The overflow policy.
    pub overflow: Overflow,
    /// The longest encoding of any of its messages.
    pub max_message_len: usize,
}
