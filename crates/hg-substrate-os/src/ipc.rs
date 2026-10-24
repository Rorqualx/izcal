//! The OS IPC transport. Until M1 phases D3 to D5 add the backends
//! (Linux sockets with `SCM_RIGHTS`, macOS Mach ports, Windows named
//! pipes brokered with `DuplicateHandle`; Stage 1 4.2), every call
//! reports [`IpcError::Unsupported`]: nothing pretends to send.

use hg_substrate::SequenceHandle;
use hg_substrate::ipc::{
    Bootstrap, ChannelReceiver, ChannelSender, Handle, Ipc, IpcError, Listening, Sink, Transfer,
};

/// The OS transport (no backend yet).
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

    fn bootstrap(&self) -> Option<Bootstrap> {
        None
    }
}
