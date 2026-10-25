//! The OS IPC transports (Stage 1 4.2). Linux has its backend
//! ([`linux`], M1 phase D3: Unix seqpacket sockets with `SCM_RIGHTS`),
//! and so has macOS (`macos`, M1 phase D4: Mach ports). Until D5 adds
//! Windows (named pipes with brokered `DuplicateHandle`), [`OsIpc`]
//! reports [`IpcError::Unsupported`] there: nothing pretends to send.

/// The libc-free parts of the Unix transport: record codec, control-data
/// parser, handle acceptance, flow control and the per-message decision.
/// Pure code, so it compiles and is tested on every OS and is what the
/// Kani harnesses prove (security scope SEC-4 (b)).
pub(crate) mod unix;

/// The record codec and flow counters every backend shares.
pub(crate) mod common;

/// The libc-free parts of the macOS transport: message layout and
/// parser, handle acceptance, the per-message decision and notification
/// classification. Pure code, tested on every OS and proved by Kani.
pub(crate) mod mach;

/// The Linux transport.
#[cfg(target_os = "linux")]
pub mod linux;

/// The macOS transport.
#[cfg(target_os = "macos")]
pub mod macos;

use hg_substrate::SequenceHandle;
use hg_substrate::ipc::{
    Bootstrap, ChannelReceiver, ChannelSender, Handle, Ipc, IpcError, Listening, OsPeer, Sink,
    Transfer,
};

/// The OS transport where no backend exists yet (Windows until D5).
#[derive(Debug, Default)]
pub struct OsIpc;

impl Ipc for OsIpc {
    fn channel(
        &self,
        _capacity: u32,
        _transfer: Transfer,
    ) -> Result<(ChannelSender, ChannelReceiver), IpcError> {
        Err(IpcError::Unsupported)
    }

    fn send(
        &self,
        _to: &ChannelSender,
        _bytes: Vec<u8>,
        _handles: Vec<Handle>,
    ) -> Result<(), IpcError> {
        Err(IpcError::Unsupported)
    }

    fn listen(
        &self,
        _from: ChannelReceiver,
        _seq: SequenceHandle,
        _sink: Sink,
    ) -> Result<Listening, IpcError> {
        Err(IpcError::Unsupported)
    }

    fn listen_bound(
        &self,
        _from: ChannelReceiver,
        _peer: OsPeer,
        _seq: SequenceHandle,
        _sink: Sink,
    ) -> Result<Listening, IpcError> {
        Err(IpcError::Unsupported)
    }

    fn bootstrap(&self) -> Option<Bootstrap> {
        None
    }
}
