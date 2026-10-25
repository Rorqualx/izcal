//! The OS IPC transports (Stage 1 4.2). Linux has its backend
//! ([`linux`], M1 phase D3: Unix seqpacket sockets with `SCM_RIGHTS`).
//! Until D4 and D5 add macOS (Mach ports) and Windows (named pipes with
//! brokered `DuplicateHandle`), [`OsIpc`] reports
//! [`IpcError::Unsupported`] there: nothing pretends to send.

/// The libc-free parts of the Unix transport: record codec, control-data
/// parser, handle acceptance, flow control and the per-message decision.
/// Pure code, so it compiles and is tested on every OS and is what the
/// Kani harnesses prove (security scope SEC-4 (b)).
pub(crate) mod unix;

/// The record codec and flow counters every backend shares.
pub(crate) mod common;

/// The Linux transport.
#[cfg(target_os = "linux")]
pub mod linux;

use hg_substrate::SequenceHandle;
use hg_substrate::ipc::{
    Bootstrap, ChannelReceiver, ChannelSender, Handle, Ipc, IpcError, Listening, OsPeer, Sink,
    Transfer,
};

/// The OS transport where no backend exists yet (macOS and Windows
/// until D4 and D5).
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
