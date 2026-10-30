//! Typed endpoints (Stage 1 design 4.1; security scope IPC-R1, R2, R5;
//! coding standard CS-AS-6 (b)).
//!
//! - A [`Remote<T>`] is the right to send interface `T`'s messages. It
//!   is a capability: the only ways to get one are to make the channel
//!   ([`channel`]) or to be sent one.
//! - An [`Endpoint<T>`] is the right to receive them. Serving it makes
//!   every message a task on one sequence (sequence affinity), checked
//!   before the handler sees it: handle count and kinds, then the bytes
//!   by the validator, then decoded. A bad message or a handler error
//!   closes the endpoint and reports why; nothing panics (IPC-R5).
//! - A bound interface's sender travels as a [`Grant<T>`], which only
//!   [`bound_channel`] makes, so a `Remote` has no way into a message;
//!   the transport also refuses to carry a bound sender from anyone but
//!   its maker (IPC-R2 (a)).
//! - A sender never blocks: a full peer is a [`SendError`] the caller
//!   handles, as the interface's overflow policy says.
//!
//! The generator writes, per interface, the [`Interface`] impl (its wire
//! form, message by message), a handler trait and a sender trait.

use core::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use hg_substrate::SequenceHandle;
use hg_substrate::ipc::{
    ChannelReceiver, ChannelSender, ConsumerRegion, Delivery, Fault, Flow, Handle, HandleKind, Ipc,
    IpcError, Listening, OsPeer, ReadOnlyRegion, RegionWriter, Sink, Transfer,
};

use crate::identity::{CallerIdentity, PeerBinding};
use crate::wire::BadMessage;
use crate::{InterfaceInfo, Overflow};

/// An interface: its messages and how they cross. Implemented by the
/// generator.
pub trait Interface: Sized + Send + 'static {
    /// What the `.hgidl` declared.
    const INFO: InterfaceInfo;
    /// The message's bytes and handles, handles in declaration order.
    fn into_wire(self) -> (Vec<u8>, Vec<Handle>);
    /// The message `bytes` and `handles` encode. Checks the handles
    /// against the message's declared kinds and validates the bytes
    /// before decoding anything.
    ///
    /// # Errors
    /// [`BadMessage`] if they are not one valid message.
    fn from_wire(
        bytes: &[u8],
        handles: Vec<Handle>,
        ipc: &Arc<dyn Ipc>,
    ) -> Result<Self, BadMessage>;
}

/// A free interface: a data channel with no identity (IPC-R2 (a)).
pub trait FreeInterface: Interface {}

/// A bound interface: identity-bearing, its sender never forwarded.
pub trait BoundInterface: Interface {}

/// Checks that `got` are exactly the kinds `want` declares, in order
/// (IPC-R1: the kind is the transport's reading of the object).
///
/// # Errors
/// [`BadMessage::BadHandles`] on any difference in count or kind.
pub fn expect_kinds(
    got: impl ExactSizeIterator<Item = HandleKind>,
    want: &[HandleKind],
) -> Result<(), BadMessage> {
    if got.len() != want.len() {
        return Err(BadMessage::BadHandles);
    }
    for (g, w) in got.zip(want) {
        if g != *w {
            return Err(BadMessage::BadHandles);
        }
    }
    Ok(())
}

/// Why a send did not happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendError {
    /// The peer was full and the interface drops on overflow: the
    /// message is gone and counted ([`Remote::dropped`]).
    Dropped,
    /// The peer was full and the interface terminates on overflow: the
    /// caller asks the broker to terminate the peer (CS-AS-6 (b)).
    PeerUnresponsive,
    /// The peer is gone.
    Closed,
    /// The transport refused a handle in the message (IPC-R1, R2).
    Refused,
    /// The message is over the transport's size or handle limit.
    TooLarge,
    /// The transport failed otherwise.
    Transport(IpcError),
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Dropped => f.write_str("the peer was full; the message was dropped"),
            Self::PeerUnresponsive => f.write_str("the peer was full; it is unresponsive"),
            Self::Closed => f.write_str("the peer is closed"),
            Self::Refused => f.write_str("the transport refused a handle"),
            Self::TooLarge => f.write_str("the message is too large"),
            Self::Transport(e) => write!(f, "transport: {e}"),
        }
    }
}

impl std::error::Error for SendError {}

/// The right to send `T`'s messages.
pub struct Remote<T: Interface> {
    ipc: Arc<dyn Ipc>,
    tx: ChannelSender,
    dropped: AtomicU64,
    _t: PhantomData<fn(T)>,
}

impl<T: Interface> core::fmt::Debug for Remote<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Remote")
            .field("interface", &T::INFO.name)
            .field("dropped", &self.dropped())
            .finish()
    }
}

impl<T: Interface> Remote<T> {
    /// A typed sender over `tx`: for a process's launch channel, whose
    /// interface the launch protocol fixes. A sender typed wrongly only
    /// produces bad messages at the peer, which ends the conversation.
    pub fn from_sender(ipc: &Arc<dyn Ipc>, tx: ChannelSender) -> Self {
        Self {
            ipc: Arc::clone(ipc),
            tx,
            dropped: AtomicU64::new(0),
            _t: PhantomData,
        }
    }

    /// Sends `msg`. Never blocks.
    ///
    /// # Errors
    /// [`SendError`]; on a full peer, [`SendError::Dropped`] or
    /// [`SendError::PeerUnresponsive`] as `T` declares.
    pub fn send(&self, msg: T) -> Result<(), SendError> {
        let (bytes, handles) = msg.into_wire();
        match self.ipc.send(&self.tx, bytes, handles) {
            Ok(()) => Ok(()),
            Err(IpcError::Full) => match T::INFO.overflow {
                Overflow::Drop => {
                    self.dropped.fetch_add(1, Ordering::Relaxed);
                    Err(SendError::Dropped)
                }
                Overflow::Terminate => Err(SendError::PeerUnresponsive),
            },
            Err(IpcError::Closed) => Err(SendError::Closed),
            Err(IpcError::Refused) => Err(SendError::Refused),
            Err(IpcError::TooLarge) => Err(SendError::TooLarge),
            Err(e) => Err(SendError::Transport(e)),
        }
    }

    /// Messages dropped because the peer was full.
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// The untyped sender, to put in a message. For generated code.
    pub fn into_handle(self) -> Handle {
        self.tx.into_handle()
    }
}

/// A bound interface's sender, granted by the process that made it. The
/// recipient turns it into a [`Remote`] and cannot send it on: a
/// `Remote` cannot become a `Grant`, and the transport refuses a bound
/// sender from anyone but its maker (IPC-R2 (a)).
pub struct Grant<T: BoundInterface> {
    tx: ChannelSender,
    _t: PhantomData<fn(T)>,
}

impl<T: BoundInterface> core::fmt::Debug for Grant<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Grant")
            .field("interface", &T::INFO.name)
            .finish()
    }
}

impl<T: BoundInterface> Grant<T> {
    /// The sender it grants.
    pub fn into_remote(self, ipc: &Arc<dyn Ipc>) -> Remote<T> {
        Remote::from_sender(ipc, self.tx)
    }

    /// The untyped sender, to put in a message. For generated code.
    pub fn into_handle(self) -> Handle {
        self.tx.into_handle()
    }
}

/// The right to receive `T`'s messages.
pub struct Endpoint<T: Interface> {
    ipc: Arc<dyn Ipc>,
    rx: ChannelReceiver,
    _t: PhantomData<fn(T)>,
}

impl<T: Interface> core::fmt::Debug for Endpoint<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Endpoint")
            .field("interface", &T::INFO.name)
            .finish()
    }
}

/// Why a served endpoint stopped. Its owner decides what follows: for a
/// bad message or handler error from a sandboxed peer, the broker
/// terminates the sender (IPC-R5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndReason<E> {
    /// A message failed its checks.
    BadMessage(BadMessage),
    /// The handler returned an error for a message.
    Handler(E),
    /// The sender is gone.
    PeerClosed,
    /// The transport refused a record before any message could be built
    /// (a malformed record, a refused handle, a bound channel used by
    /// another process). [`Fault::sender`] names the process to
    /// terminate when the OS attests it (IPC-R5).
    Violation(Fault),
}

/// Keeps an endpoint served; dropping it closes the endpoint.
#[derive(Debug)]
pub struct Serving {
    /// Held only for its `Drop`, which closes the endpoint.
    _closes_on_drop: Listening,
}

impl<T: Interface> Endpoint<T> {
    /// A typed receiver over `rx`: for a process's launch channel.
    pub fn from_receiver(ipc: &Arc<dyn Ipc>, rx: ChannelReceiver) -> Self {
        Self {
            ipc: Arc::clone(ipc),
            rx,
            _t: PhantomData,
        }
    }

    /// The untyped receiver, to put in a message. For generated code.
    pub fn into_handle(self) -> Handle {
        self.rx.into_handle()
    }

    fn listen<E: Send + 'static>(
        self,
        peer: Option<OsPeer>,
        seq: SequenceHandle,
        mut handle: impl FnMut(T) -> Result<(), E> + Send + 'static,
        on_end: impl FnOnce(EndReason<E>) + Send + 'static,
    ) -> Result<Serving, IpcError> {
        let ipc = Arc::clone(&self.ipc);
        let mut on_end = Some(on_end);
        let mut end = move |why| {
            if let Some(f) = on_end.take() {
                f(why);
            }
            Flow::Close
        };
        let sink: Sink = Box::new(move |d| match d {
            Delivery::Message { bytes, handles } => match T::from_wire(&bytes, handles, &ipc) {
                Ok(msg) => match handle(msg) {
                    Ok(()) => Flow::Continue,
                    Err(e) => end(EndReason::Handler(e)),
                },
                Err(bad) => end(EndReason::BadMessage(bad)),
            },
            Delivery::Closed => end(EndReason::PeerClosed),
            Delivery::Violation(fault) => end(EndReason::Violation(fault)),
        });
        let listening = match peer {
            None => self.ipc.listen(self.rx, seq, sink)?,
            Some(p) => self.ipc.listen_bound(self.rx, p, seq, sink)?,
        };
        Ok(Serving {
            _closes_on_drop: listening,
        })
    }
}

impl<T: FreeInterface> Endpoint<T> {
    /// Serves the endpoint on `seq`: each message, checked, goes to
    /// `handle`, in order. Free endpoints carry no identity.
    ///
    /// # Errors
    /// The transport's, if it cannot listen.
    pub fn serve<E: Send + 'static>(
        self,
        seq: SequenceHandle,
        handle: impl FnMut(T) -> Result<(), E> + Send + 'static,
        on_end: impl FnOnce(EndReason<E>) + Send + 'static,
    ) -> Result<Serving, IpcError> {
        self.listen(None, seq, handle, on_end)
    }
}

impl<T: BoundInterface> Endpoint<T> {
    /// Serves the endpoint on `seq` with the identity `binding`: each
    /// message, checked, goes to `handle` with the caller's identity.
    ///
    /// # Errors
    /// The transport's, if it cannot listen.
    pub fn serve_bound<E: Send + 'static>(
        self,
        seq: SequenceHandle,
        binding: PeerBinding,
        mut handle: impl FnMut(T, &CallerIdentity) -> Result<(), E> + Send + 'static,
        on_end: impl FnOnce(EndReason<E>) + Send + 'static,
    ) -> Result<Serving, IpcError> {
        let who = CallerIdentity::from_binding(binding);
        self.listen(Some(binding.os), seq, move |m| handle(m, &who), on_end)
    }
}

/// A new channel for free interface `T`.
///
/// # Errors
/// The transport's.
pub fn channel<T: FreeInterface>(ipc: &Arc<dyn Ipc>) -> Result<(Remote<T>, Endpoint<T>), IpcError> {
    let (tx, rx) = ipc.channel(T::INFO.capacity, Transfer::Free)?;
    Ok((
        Remote::from_sender(ipc, tx),
        Endpoint::from_receiver(ipc, rx),
    ))
}

/// A new channel for bound interface `T`: the grant to send to the peer
/// it is for, and the endpoint to serve with that peer's binding.
///
/// # Errors
/// The transport's.
pub fn bound_channel<T: BoundInterface>(
    ipc: &Arc<dyn Ipc>,
) -> Result<(Grant<T>, Endpoint<T>), IpcError> {
    let (tx, rx) = ipc.channel(T::INFO.capacity, Transfer::Bound)?;
    Ok((
        Grant {
            tx,
            _t: PhantomData,
        },
        Endpoint::from_receiver(ipc, rx),
    ))
}

/// Takes the next handle as a [`Remote`]. For generated `from_wire`,
/// after [`expect_kinds`].
///
/// # Errors
/// [`BadMessage::BadHandles`] if it is missing or another kind.
pub fn take_remote<T: FreeInterface>(
    h: Option<Handle>,
    ipc: &Arc<dyn Ipc>,
) -> Result<Remote<T>, BadMessage> {
    let h = h.ok_or(BadMessage::BadHandles)?;
    if h.kind() != HandleKind::ChannelSender {
        return Err(BadMessage::BadHandles);
    }
    let tx = ChannelSender::from_handle(h).map_err(|_other| BadMessage::BadHandles)?;
    Ok(Remote::from_sender(ipc, tx))
}

/// Takes the next handle as a [`Grant`]. For generated `from_wire`.
///
/// # Errors
/// [`BadMessage::BadHandles`] if it is missing or another kind.
pub fn take_grant<T: BoundInterface>(h: Option<Handle>) -> Result<Grant<T>, BadMessage> {
    let h = h.ok_or(BadMessage::BadHandles)?;
    if h.kind() != HandleKind::BoundChannelSender {
        return Err(BadMessage::BadHandles);
    }
    let tx = ChannelSender::from_handle(h).map_err(|_other| BadMessage::BadHandles)?;
    Ok(Grant {
        tx,
        _t: PhantomData,
    })
}

/// Takes the next handle as an [`Endpoint`]. For generated `from_wire`.
///
/// # Errors
/// [`BadMessage::BadHandles`] if it is missing or another kind.
pub fn take_endpoint<T: FreeInterface>(
    h: Option<Handle>,
    ipc: &Arc<dyn Ipc>,
) -> Result<Endpoint<T>, BadMessage> {
    let rx = ChannelReceiver::from_handle(h.ok_or(BadMessage::BadHandles)?)
        .map_err(|_other| BadMessage::BadHandles)?;
    Ok(Endpoint::from_receiver(ipc, rx))
}

/// Whether a region handle of `kind` and declared length `len` may fill a
/// field of `want` kind bounded by `bound` bytes that does or does not
/// require `immutable`, given whether this process's transport attests
/// the region `immutable` (D6). The one rule the generated decoders use.
pub fn region_fits(
    kind: HandleKind,
    want: HandleKind,
    len: usize,
    bound: u32,
    attested: bool,
    immutable: bool,
) -> bool {
    kind == want
        && matches!(
            want,
            HandleKind::RegionWriter | HandleKind::ReadOnlyRegion | HandleKind::ConsumerRegion
        )
        && len > 0
        && u32::try_from(len).is_ok_and(|l| l <= bound)
        && (!immutable || attested)
}

fn region_parts(h: Option<Handle>) -> Result<(Handle, usize, bool), BadMessage> {
    let h = h.ok_or(BadMessage::BadHandles)?;
    let (len, attested) = h
        .region()
        .map(|r| (r.len(), r.immutable()))
        .ok_or(BadMessage::BadHandles)?;
    Ok((h, len, attested))
}

/// Takes the next handle as a `region<writer, N>`. For generated
/// `from_wire`.
///
/// # Errors
/// [`BadMessage::BadHandles`] if it is missing, another kind, or longer
/// than `bound`.
pub fn take_region_writer(h: Option<Handle>, bound: u32) -> Result<RegionWriter, BadMessage> {
    let (h, len, attested) = region_parts(h)?;
    if !region_fits(
        h.kind(),
        HandleKind::RegionWriter,
        len,
        bound,
        attested,
        false,
    ) {
        return Err(BadMessage::BadHandles);
    }
    RegionWriter::from_handle(h).map_err(|_other| BadMessage::BadHandles)
}

/// Takes the next handle as a `region<ro, N>`, or with `immutable` a
/// `region<ro, N, immutable>`, which this process's transport must attest
/// immutable. For generated `from_wire`.
///
/// # Errors
/// [`BadMessage::BadHandles`] if it is missing, another kind, longer than
/// `bound`, or not attested immutable where the field requires it.
pub fn take_read_only_region(
    h: Option<Handle>,
    bound: u32,
    immutable: bool,
) -> Result<ReadOnlyRegion, BadMessage> {
    let (h, len, attested) = region_parts(h)?;
    if !region_fits(
        h.kind(),
        HandleKind::ReadOnlyRegion,
        len,
        bound,
        attested,
        immutable,
    ) {
        return Err(BadMessage::BadHandles);
    }
    ReadOnlyRegion::from_handle(h).map_err(|_other| BadMessage::BadHandles)
}

/// Takes the next handle as a `region<consumer, N>`. For generated
/// `from_wire`.
///
/// # Errors
/// [`BadMessage::BadHandles`] if it is missing, another kind, or longer
/// than `bound`.
pub fn take_consumer_region(h: Option<Handle>, bound: u32) -> Result<ConsumerRegion, BadMessage> {
    let (h, len, attested) = region_parts(h)?;
    if !region_fits(
        h.kind(),
        HandleKind::ConsumerRegion,
        len,
        bound,
        attested,
        false,
    ) {
        return Err(BadMessage::BadHandles);
    }
    ConsumerRegion::from_handle(h).map_err(|_other| BadMessage::BadHandles)
}

#[cfg(kani)]
mod proofs {
    use super::*;

    fn kind(b: u8) -> HandleKind {
        match b % 3 {
            0 => HandleKind::ChannelSender,
            1 => HandleKind::BoundChannelSender,
            _ => HandleKind::ChannelReceiver,
        }
    }

    /// IPC-R1: the check accepts exactly the declared kinds, in order,
    /// for every pair of lists of up to 4 handles of the three kinds.
    #[kani::proof]
    #[kani::unwind(6)]
    fn expect_kinds_is_exact() {
        let got: [u8; 4] = kani::any();
        let want: [u8; 4] = kani::any();
        let (gn, wn): (usize, usize) = (kani::any(), kani::any());
        kani::assume(gn <= 4 && wn <= 4);
        let g: [HandleKind; 4] = [kind(got[0]), kind(got[1]), kind(got[2]), kind(got[3])];
        let w: [HandleKind; 4] = [kind(want[0]), kind(want[1]), kind(want[2]), kind(want[3])];
        let (Some(g), Some(w)) = (g.get(..gn), w.get(..wn)) else {
            return;
        };
        let ok = expect_kinds(g.iter().copied(), w).is_ok();
        assert!(ok == (g == w));
    }

    fn any_kind(b: u8) -> HandleKind {
        match b % 6 {
            0 => HandleKind::ChannelSender,
            1 => HandleKind::RegionWriter,
            2 => HandleKind::ReadOnlyRegion,
            3 => HandleKind::ConsumerRegion,
            4 => HandleKind::ProducerRegion,
            _ => HandleKind::GpuDmaBuf,
        }
    }

    /// D6: a region fills a field only if it is that field's region kind
    /// (never a producer or a GPU buffer), non-empty, within the bound,
    /// and attested immutable where the field requires it.
    #[kani::proof]
    fn region_fits_exactly() {
        let (k, w): (u8, u8) = (kani::any(), kani::any());
        let (kind, want) = (any_kind(k), any_kind(w));
        let (len, bound): (usize, u32) = (kani::any(), kani::any());
        let (attested, immutable): (bool, bool) = (kani::any(), kani::any());
        let ok = region_fits(kind, want, len, bound, attested, immutable);
        if ok {
            assert!(kind == want);
            assert!(kind != HandleKind::ProducerRegion && kind != HandleKind::GpuDmaBuf);
            assert!(kind != HandleKind::ChannelSender);
            assert!(len > 0 && len as u64 <= u64::from(bound));
            assert!(attested || !immutable);
        }
        kani::cover!(ok && immutable);
        kani::cover!(!ok && kind == want && len as u64 > u64::from(bound));
    }
}
