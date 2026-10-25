//! Channels between processes (Stage 1 design 4.1, 4.2; HGBA-3 B2;
//! security scope 6.6).
//!
//! A channel is one-way: bytes and handles go from its [`ChannelSender`]
//! to its [`ChannelReceiver`], in order. Each delivery is a task on the
//! sequence the receiver listens on (sequence affinity), so IPC and the
//! event loop share one ordering model, and the simulator controls both.
//! A sender never blocks (CS-AS-6 (b)): a full channel is an error the
//! sender handles.
//!
//! Handles are owned objects. Dropping one closes it. The transport, not
//! the message, decides a handle's kind (IPC-R1): every backend makes its
//! own handle objects and refuses any it did not make. The typed layer on
//! top (endpoints, identities, the per-interface rules) is `hg-ipc`.
//!
//! Implementations: `hg-substrate-sim` (in-memory queues under the seeded
//! scheduler) and, per OS, `hg-substrate-os` (M1 phase D3 to D5).

use core::any::Any;

use crate::executor::SequenceHandle;

/// The most bytes one message may carry: 64 KiB†. Bulk data crosses as
/// shared memory, never as a large message (Stage 1 4.1).
pub const MAX_MESSAGE_BYTES: usize = 64 * 1024;

/// The most handles one message may carry: 16†.
pub const MAX_MESSAGE_HANDLES: usize = 16;

/// What a handle is. The transport reads it from the object (IPC-R1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandleKind {
    /// The sending end of a free channel: the right to send on it.
    ChannelSender,
    /// The sending end of a bound channel (IPC-R2 (a)): a distinct kind,
    /// so a bound sender cannot pass where a free one is declared.
    BoundChannelSender,
    /// The receiving end of a channel: the right to receive from it.
    ChannelReceiver,
}

/// A backend's handle object. Dropping it closes what it names.
pub trait HandleObject: Send + Sync + core::fmt::Debug {
    /// What it is.
    fn kind(&self) -> HandleKind;
    /// The backend's own type, so the backend can recognise its handles.
    fn as_any(&self) -> &dyn Any;
    /// The backend's own type, owned, for a backend taking a handle back.
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send>;
}

/// An owned handle of some kind.
#[derive(Debug)]
pub struct Handle(Box<dyn HandleObject>);

impl Handle {
    /// Wraps a backend's handle object. For backends only.
    pub fn new(object: Box<dyn HandleObject>) -> Self {
        Self(object)
    }

    /// What it is.
    pub fn kind(&self) -> HandleKind {
        self.0.kind()
    }

    /// The backend object. For backends only.
    pub fn object(&self) -> &dyn Any {
        self.0.as_any()
    }

    /// The backend object, owned. For backends only.
    pub fn into_object(self) -> Box<dyn Any + Send> {
        self.0.into_any()
    }
}

/// The sending end of a channel.
#[derive(Debug)]
pub struct ChannelSender(Handle);

/// The receiving end of a channel.
#[derive(Debug)]
pub struct ChannelReceiver(Handle);

macro_rules! typed_handle {
    ($t:ident, $($kind:ident)|+) => {
        impl $t {
            /// Wraps a handle of this kind.
            ///
            /// # Errors
            /// Gives the handle back if it is another kind.
            pub fn from_handle(h: Handle) -> Result<Self, Handle> {
                if matches!(h.kind(), $(HandleKind::$kind)|+) {
                    Ok(Self(h))
                } else {
                    Err(h)
                }
            }

            /// The untyped handle, to send it.
            pub fn into_handle(self) -> Handle {
                self.0
            }

            /// The untyped handle. For backends only.
            pub const fn handle(&self) -> &Handle {
                &self.0
            }
        }
    };
}

typed_handle!(ChannelSender, ChannelSender | BoundChannelSender);
typed_handle!(ChannelReceiver, ChannelReceiver);

/// Whether a channel's sender is identity-bearing (IPC-R2 (a)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transfer {
    /// The sender may be sent onward by whoever holds it.
    Free,
    /// The sender may be sent only by the process that created the
    /// channel, which grants it to one peer; the transport refuses to
    /// carry it from anywhere else, so a holder cannot pass it on. The
    /// receiver is never carried: it stays with the process serving it.
    Bound,
}

/// What a listener is given.
#[derive(Debug)]
pub enum Delivery {
    /// The next message, in send order.
    Message {
        /// Its bytes, at most [`MAX_MESSAGE_BYTES`].
        bytes: Vec<u8>,
        /// Its handles, at most [`MAX_MESSAGE_HANDLES`], in send order;
        /// now owned by this process.
        handles: Vec<Handle>,
    },
    /// The sender is gone (dropped, or its process ended). Nothing
    /// follows.
    Closed,
    /// The transport refused what arrived: a record that breaks the
    /// transport's own rules, before any message could be built. The
    /// channel is closed and nothing follows. Unlike [`Delivery::Closed`]
    /// this is the peer's fault, so the receiver's owner can ask for the
    /// sender to be terminated (IPC-R5).
    Violation(Fault),
}

/// What a peer did wrong at the transport level (IPC-R1, R2, R5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaultKind {
    /// A record that does not parse: bad framing, a truncated record, an
    /// empty record, or ancillary data the transport never sends.
    Malformed,
    /// A handle of a kind or type the transport does not accept, one the
    /// receiver already holds, or a count that disagrees with the record.
    BadHandle,
    /// A record on a bound channel from a process other than the one the
    /// channel is bound to, or a bound sender passed on by a process
    /// that did not create it (IPC-R2).
    Identity,
    /// Flow-control state that cannot happen for an honest sender:
    /// sequence numbers out of order, or credits beyond what was sent.
    Sequence,
}

/// A transport-level violation and, where the OS attests it, who sent
/// the offending record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fault {
    /// What went wrong.
    pub kind: FaultKind,
    /// The OS process ID of the record's sender, when the OS attests it
    /// (Linux `SCM_CREDENTIALS`) and the fault is one only that sender
    /// could cause. `None` when the transport cannot tell who is at
    /// fault, for example when a former holder of a transferred end may
    /// have lied about its counters.
    pub sender: Option<u32>,
}

/// What a listener does after a delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// Keep listening.
    Continue,
    /// Close the receiver now: drop what is queued and refuse new sends.
    Close,
}

/// A listener's callback: runs as a task on its sequence, once per
/// delivery, in order.
pub type Sink = Box<dyn FnMut(Delivery) -> Flow + Send>;

/// Keeps a receiver listening; dropping it closes the receiver.
#[derive(Debug)]
pub struct Listening {
    /// Held only for its `Drop`, which closes the receiver.
    _closes_on_drop: Handle,
}

impl Listening {
    /// Wraps the backend's listening object. For backends only.
    pub const fn new(h: Handle) -> Self {
        Self { _closes_on_drop: h }
    }
}

/// A process's launch channels: one each way to the process that
/// launched it ("renderers start with exactly one endpoint, to the
/// browser process", Stage 1 4.1).
#[derive(Debug)]
pub struct Bootstrap {
    /// To the launching process.
    pub to_parent: ChannelSender,
    /// From the launching process.
    pub from_parent: ChannelReceiver,
}

/// Why an IPC operation failed. Every case is the caller's to handle;
/// none is a panic (CS-ER-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    /// The receiver holds its capacity of undelivered messages. The
    /// sender drops the message with a count or declares the peer
    /// unresponsive, as its interface declares (CS-AS-6 (b)).
    Full,
    /// The other end is gone.
    Closed,
    /// The transport refused a handle: not one of its own, not held by
    /// this process, or a bound sender sent by a process that did not
    /// create it (IPC-R1, IPC-R2).
    Refused,
    /// More than [`MAX_MESSAGE_BYTES`] bytes or [`MAX_MESSAGE_HANDLES`]
    /// handles.
    TooLarge,
    /// The sequence to deliver on cannot take a task.
    Sequence,
    /// This substrate has no transport yet (the OS backends arrive in
    /// M1 phases D3 to D5).
    Unsupported,
}

impl core::fmt::Display for IpcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Full => "the receiver is full",
            Self::Closed => "the other end is closed",
            Self::Refused => "the transport refused a handle",
            Self::TooLarge => "too many bytes or handles for one message",
            Self::Sequence => "the delivery sequence cannot take a task",
            Self::Unsupported => "no IPC transport on this substrate yet",
        })
    }
}

impl std::error::Error for IpcError {}

/// The IPC transport of one process.
pub trait Ipc: Send + Sync {
    /// A new channel holding at most `capacity` undelivered messages,
    /// with both ends owned by this process.
    ///
    /// # Errors
    /// [`IpcError::Unsupported`], or a backend's resource failure.
    fn channel(
        &self,
        capacity: u32,
        transfer: Transfer,
    ) -> Result<(ChannelSender, ChannelReceiver), IpcError>;

    /// Sends `bytes` and `handles` on `to`. Never blocks. On an error the
    /// message is not sent and its handles are dropped (closed).
    ///
    /// # Errors
    /// [`IpcError::Full`], [`IpcError::Closed`], [`IpcError::Refused`],
    /// [`IpcError::TooLarge`].
    fn send(
        &self,
        to: &ChannelSender,
        bytes: Vec<u8>,
        handles: Vec<Handle>,
    ) -> Result<(), IpcError>;

    /// Delivers what arrives on `from`, and finally its closing, to
    /// `sink`, each as a task on `seq`, in order.
    ///
    /// # Errors
    /// [`IpcError::Refused`] for a receiver this process does not hold,
    /// [`IpcError::Sequence`], [`IpcError::Unsupported`].
    fn listen(
        &self,
        from: ChannelReceiver,
        seq: SequenceHandle,
        sink: Sink,
    ) -> Result<Listening, IpcError>;

    /// This process's launch channels. The first call takes them; later
    /// calls, and processes launched without them, get `None`.
    fn bootstrap(&self) -> Option<Bootstrap>;
}
