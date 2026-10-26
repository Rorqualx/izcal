//! The simulated IPC transport (Stage 1 design 4.2, "in-memory queues
//! delivered by the seeded scheduler").
//!
//! Channels live in the world's state. A send queues the message; each
//! listening channel keeps at most one delivery task on its sequence,
//! which delivers the next item and posts itself again while more are
//! queued (the drain-task model the OS transports use, so a sender can
//! never be refused because the receiver's sequence is busy). Delivery
//! order across processes is the scheduler's seeded choice and every
//! send, delivery and close is in the trace. The transport keeps each end's
//! owner, (process, incarnation), and enforces what the OS backends
//! will:
//! - a process sends only handles it holds, and only this transport's
//!   (IPC-R1);
//! - a bound sender is sent at most once, by the process that made the
//!   channel (IPC-R2 (a));
//! - a receiver listened to with [`Ipc::listen_bound`] takes messages
//!   from the process it names only, and a bound receiver listens no
//!   other way (IPC-R2);
//! - a crash closes every end the process held, so its peers see
//!   [`Delivery::Closed`] (IPC-R8's closure fault);
//! - a process marked with [`crate::World::corrupt_next_send`] sends one
//!   record the receiving transport refuses, so its peer sees
//!   [`Delivery::Violation`] (the adversarial-peer fault).
//!
//! The fault plan's IPC faults (phase E rule E-R8), each from its own
//! stream: a **delay** holds a message back in virtual time, and the
//! messages after it on its channel wait behind it; a **reorder** makes a
//! channel's next delivery step back behind the work already queued on
//! its receiver's sequence, so other channels' deliveries overtake it;
//! a **drop** loses a message and closes its channel, so the receiver
//! sees `Closed` after what came before. Only a channel's head is ever
//! delivered, so no fault reorders messages within one channel: per-
//! channel FIFO is every backend's contract, and a fault that broke it
//! would test a guarantee none makes.
//!
//! Handle objects lock the world when dropped, so nothing that may hold
//! one (a queued message, a sink) is dropped with the lock held: it is
//! moved out and dropped after.

use core::any::Any;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};

use hg_substrate::ipc::{
    Bootstrap, ChannelReceiver, ChannelSender, ConsumerRegion, Delivery, Fault, FaultKind, Flow,
    Handle, HandleKind, HandleObject, Ipc, IpcError, Listening, MAX_MESSAGE_BYTES,
    MAX_MESSAGE_HANDLES, Mapping, OsPeer, ProducerRegion, ReadOnlyRegion, RegionRefused,
    RegionWriter, Sink, Transfer,
};
use hg_substrate::{RandomStream as _, SequenceHandle, Task};

use crate::region::Regions;
use crate::world::{ProcessId, SEQUENCE_CAPACITY, Shared, State};

/// Which end of a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    Sender,
    Receiver,
}

/// A queued message.
struct Msg {
    bytes: Vec<u8>,
    handles: Vec<Handle>,
    /// The sending end's owner when it was sent.
    from: Owner,
    /// Virtual time (ns) before which it is not delivered (E-R8 delay).
    ready_at: u64,
    /// It already stepped back once (E-R8 reorder): it does not again.
    yielded: bool,
}

/// Who holds an end or a region handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Owner {
    Proc(ProcessId, u32),
    /// Inside a queued message.
    Transit,
}

#[derive(Debug, Clone, Copy)]
struct End {
    owner: Owner,
    open: bool,
}

struct Listen {
    pid: ProcessId,
    generation: u32,
    seq: usize,
    sink: Arc<Mutex<Sink>>,
    /// For a bound receiver, the one process whose messages it accepts
    /// ([`Ipc::listen_bound`]).
    peer: Option<Owner>,
}

struct Chan {
    capacity: u32,
    transfer: Transfer,
    creator: ProcessId,
    granted: bool,
    sender: End,
    receiver: End,
    /// Undelivered messages, in send order.
    queue: VecDeque<Msg>,
    /// A refused record, delivered after the queued messages; nothing
    /// follows it.
    fault: Option<Fault>,
    listen: Option<Listen>,
    /// A delivery task for this channel is on its sequence.
    draining: bool,
    closed_delivered: bool,
    /// An injected drop lost a message: nothing more is sent, and the
    /// receiver sees `Closed` after what was queued (E-R8).
    severed: bool,
    /// A wake is pending for a held-back head message.
    wake_pending: bool,
}

impl Chan {
    fn end_mut(&mut self, side: Side) -> &mut End {
        match side {
            Side::Sender => &mut self.sender,
            Side::Receiver => &mut self.receiver,
        }
    }
}

/// Things to drop once the world's lock is released.
pub(crate) type Dropped = Vec<Box<dyn Any + Send>>;

/// Every channel in the world.
#[derive(Default)]
pub(crate) struct Channels {
    chans: BTreeMap<u64, Chan>,
    next: u64,
    /// A process's launch channels: (to its parent, from its parent).
    boot: BTreeMap<ProcessId, (u64, u64, OsPeer)>,
    /// Processes whose next send is refused by the receiving transport.
    corrupt: BTreeMap<ProcessId, FaultKind>,
}

impl Channels {
    fn new_chan(
        &mut self,
        capacity: u32,
        transfer: Transfer,
        a: Owner,
        b: Owner,
        creator: ProcessId,
    ) -> u64 {
        let id = self.next;
        self.next = self.next.saturating_add(1);
        self.chans.insert(
            id,
            Chan {
                capacity,
                transfer,
                creator,
                granted: false,
                sender: End {
                    owner: a,
                    open: true,
                },
                receiver: End {
                    owner: b,
                    open: true,
                },
                queue: VecDeque::new(),
                fault: None,
                listen: None,
                draining: false,
                closed_delivered: false,
                severed: false,
                wake_pending: false,
            },
        );
        id
    }

    /// Closes every end `pid` holds (any incarnation): it crashed.
    pub(crate) fn crash(st: &mut State, pid: ProcessId, out: &mut Dropped) {
        let ids: Vec<u64> = st.ipc.chans.keys().copied().collect();
        for id in ids {
            let held = |e: &End| matches!(e.owner, Owner::Proc(p, _) if p == pid) && e.open;
            let Some(c) = st.ipc.chans.get(&id) else {
                continue;
            };
            let (s, r) = (held(&c.sender), held(&c.receiver));
            if s {
                close(st, id, Side::Sender, out);
            }
            if r {
                close(st, id, Side::Receiver, out);
            }
            // A listener of the dead process goes, whoever owns the end.
            if let Some(c) = st.ipc.chans.get_mut(&id)
                && c.listen.as_ref().is_some_and(|l| l.pid == pid)
                && let Some(l) = c.listen.take()
            {
                out.push(Box::new(l.sink));
            }
        }
        st.ipc.boot.remove(&pid);
        Regions::owner_gone(st, pid);
    }

    /// Everything, for the world's teardown.
    pub(crate) fn drain(&mut self, out: &mut Dropped) {
        for (_, mut c) in std::mem::take(&mut self.chans) {
            for m in c.queue.drain(..) {
                out.push(Box::new(m.handles));
            }
            if let Some(l) = c.listen.take() {
                out.push(Box::new(l.sink));
            }
        }
    }
}

/// Closes one end. Closing the sender queues a final `Closed` delivery
/// for a listener; closing the receiver drops what is queued.
fn close(st: &mut State, id: u64, side: Side, out: &mut Dropped) {
    let Some(c) = st.ipc.chans.get_mut(&id) else {
        return;
    };
    let end = c.end_mut(side);
    if !end.open {
        return;
    }
    end.open = false;
    st.trace.record(format_args!("ipc close #{id} {side:?}"));
    match side {
        Side::Sender => kick(st, id),
        Side::Receiver => {
            if let Some(c) = st.ipc.chans.get_mut(&id) {
                for m in c.queue.drain(..) {
                    out.push(Box::new(m.handles));
                }
                if let Some(l) = c.listen.take() {
                    out.push(Box::new(l.sink));
                }
            }
        }
    }
    if let Some(c) = st.ipc.chans.get(&id)
        && !c.sender.open
        && !c.receiver.open
        && c.queue.is_empty()
        && c.fault.is_none()
    {
        st.ipc.chans.remove(&id);
    }
}

/// Marks `pid`'s next send as refused by the receiver (see
/// [`crate::World::corrupt_next_send`]).
pub(crate) fn corrupt_next(st: &mut State, pid: ProcessId, kind: FaultKind) {
    st.trace
        .record(format_args!("ipc corrupt-next {} {kind:?}", pid.0));
    st.ipc.corrupt.insert(pid, kind);
}

/// Starts channel `id`'s delivery task if it is listened to, has
/// something to deliver, and has no task on its sequence yet. A drain
/// task may exceed the sequence's capacity: there is at most one per
/// listening channel, and a delivery must not be lost.
fn kick(st: &mut State, id: u64) {
    let now = st.now.as_nanos();
    let Some(c) = st.ipc.chans.get_mut(&id) else {
        return;
    };
    let Some(l) = &c.listen else { return };
    let ended = !c.sender.open || c.severed;
    let pending = !c.queue.is_empty() || c.fault.is_some() || (ended && !c.closed_delivered);
    if c.draining || !pending {
        return;
    }
    // A held-back head holds the whole channel (FIFO): wake it when due.
    if let Some(at) = c.queue.front().map(|m| m.ready_at).filter(|&at| at > now) {
        if !c.wake_pending {
            c.wake_pending = true;
            st.add_ipc_wake(id, at);
        }
        return;
    }
    let (pid, generation, seq) = (l.pid, l.generation, l.seq);
    if queue_delivery(st, id, pid, generation, seq, true)
        && let Some(c) = st.ipc.chans.get_mut(&id)
    {
        c.draining = true;
    }
}

/// Puts a delivery task for channel `id` on `(pid, generation)`'s
/// sequence `seq`. Returns false if the sequence is gone or full.
fn queue_delivery(
    st: &mut State,
    id: u64,
    pid: ProcessId,
    generation: u32,
    seq: usize,
    force: bool,
) -> bool {
    let shared = st.ipc_self.clone();
    let Some(p) = st.proc_mut(pid, generation) else {
        return false;
    };
    let Some(s) = p.seqs.get_mut(seq) else {
        return false;
    };
    if !force && s.queue.len().saturating_add(s.delayed) >= SEQUENCE_CAPACITY {
        return false;
    }
    let number = s.posted;
    s.posted = s.posted.saturating_add(1);
    let Some(shared) = shared.upgrade() else {
        return false;
    };
    s.queue
        .push_back((number, Task::new(move || deliver(&shared, id))));
    true
}

/// A held-back message on channel `id` is due.
pub(crate) fn wake(st: &mut State, id: u64) {
    if let Some(c) = st.ipc.chans.get_mut(&id) {
        c.wake_pending = false;
    }
    kick(st, id);
}

/// One delivery task: the next message, or the closing notice.
fn deliver(shared: &Arc<Shared>, id: u64) {
    let mut st = shared.lock();
    let Some(c) = st.ipc.chans.get_mut(&id) else {
        return;
    };
    c.draining = false;
    let Some(l) = &c.listen else { return };
    let (pid, generation, sink, peer) = (l.pid, l.generation, Arc::clone(&l.sink), l.peer);
    let seq = l.seq;
    // E-R8 reorder: the head steps back once behind what the sequence
    // already holds; the channel's own order is kept.
    let rate = st.config.faults.ipc_reorder_per_million;
    if st
        .ipc
        .chans
        .get(&id)
        .is_some_and(|c| c.queue.front().is_some_and(|m| !m.yielded))
        && crate::world::fires(&mut st.faults.ipc_reorder, rate)
    {
        if let Some(m) = st.ipc.chans.get_mut(&id).and_then(|c| c.queue.front_mut()) {
            m.yielded = true;
        }
        st.trace.record(format_args!("ipc fault reorder #{id}"));
        if queue_delivery(&mut st, id, pid, generation, seq, true)
            && let Some(c) = st.ipc.chans.get_mut(&id)
        {
            c.draining = true;
        }
        return;
    }
    let Some(c) = st.ipc.chans.get_mut(&id) else {
        return;
    };
    // Handles of a message refused here, dropped once the lock is released.
    let mut stray = Vec::new();
    let delivery = if let Some(Msg {
        bytes,
        handles,
        from,
        ..
    }) = c.queue.pop_front()
    {
        match peer {
            // A bound receiver's message from another process: the bound
            // peer passed its sender on, so it is the one named.
            Some(bound @ Owner::Proc(p, _)) if from != bound => {
                // Nothing follows a violation. The handles of this message
                // and of every one queued after it close once the lock is
                // released (dropping one locks the world).
                stray = handles;
                for m in c.queue.drain(..) {
                    stray.extend(m.handles);
                }
                c.closed_delivered = true;
                Delivery::Violation(Fault {
                    kind: FaultKind::Identity,
                    sender: Some(p.0),
                })
            }
            // A writer from anyone but the broker (D6 review M1).
            Some(_) | None
                if handles.iter().any(Regions::is_writer)
                    && !matches!(from, Owner::Proc(p, g) if st.proc(p, g).is_some_and(|x| !x.launched)) =>
            {
                stray = handles;
                for m in c_queue_drain(&mut st, id) {
                    stray.extend(m.handles);
                }
                if let Some(c) = st.ipc.chans.get_mut(&id) {
                    c.closed_delivered = true;
                }
                Delivery::Violation(Fault {
                    kind: FaultKind::BadHandle,
                    sender: match from {
                        Owner::Proc(p, _) => Some(p.0),
                        Owner::Transit => None,
                    },
                })
            }
            _ => Delivery::Message { bytes, handles },
        }
    } else if let Some(fault) = c.fault.take() {
        c.closed_delivered = true;
        Delivery::Violation(fault)
    } else if (!c.sender.open || c.severed) && !c.closed_delivered {
        c.closed_delivered = true;
        Delivery::Closed
    } else {
        return;
    };
    // After a violation the transport closes the channel: nothing follows.
    let refused = matches!(delivery, Delivery::Violation(_));
    let what = match &delivery {
        Delivery::Message { bytes, handles } => {
            let os = st.config.region_os;
            for h in handles {
                if let Some(e) = h.object().downcast_ref::<SimEnd>()
                    && let Some(hc) = st.ipc.chans.get_mut(&e.chan)
                {
                    hc.end_mut(e.side).owner = Owner::Proc(pid, generation);
                }
                Regions::arrive(&mut st, h, Owner::Proc(pid, generation), os);
            }
            format!("{} bytes {} handles", bytes.len(), handles.len())
        }
        Delivery::Closed => "closed".to_owned(),
        Delivery::Violation(f) => format!("violation {:?}", f.kind),
    };
    st.trace.record(format_args!("ipc deliver #{id} {what}"));
    drop(st);
    drop(stray);
    let flow = sink.lock().unwrap_or_else(PoisonError::into_inner)(delivery);
    let mut out = Dropped::new();
    let mut st = shared.lock();
    if flow == Flow::Close || refused {
        close(&mut st, id, Side::Receiver, &mut out);
    } else {
        kick(&mut st, id);
    }
    drop(st);
    drop(out);
}

/// Takes every message still queued on channel `id`.
fn c_queue_drain(st: &mut State, id: u64) -> Vec<Msg> {
    st.ipc
        .chans
        .get_mut(&id)
        .map(|c| c.queue.drain(..).collect())
        .unwrap_or_default()
}

/// A channel end this transport made.
pub(crate) struct SimEnd {
    shared: Arc<Shared>,
    chan: u64,
    side: Side,
    /// A bound channel's sender is its own kind (IPC-R1, R2).
    bound: bool,
    /// False once the end has been turned into something else (a
    /// listener), so this object's drop closes nothing.
    armed: bool,
}

impl core::fmt::Debug for SimEnd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SimEnd")
            .field("chan", &self.chan)
            .field("side", &self.side)
            .finish()
    }
}

impl HandleObject for SimEnd {
    fn kind(&self) -> HandleKind {
        match (self.side, self.bound) {
            (Side::Sender, false) => HandleKind::ChannelSender,
            (Side::Sender, true) => HandleKind::BoundChannelSender,
            (Side::Receiver, _) => HandleKind::ChannelReceiver,
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

impl Drop for SimEnd {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut out = Dropped::new();
        let mut st = self.shared.lock();
        close(&mut st, self.chan, self.side, &mut out);
        drop(st);
        drop(out);
    }
}

fn end_handle(shared: &Arc<Shared>, chan: u64, side: Side, transfer: Transfer) -> Handle {
    Handle::new(Box::new(SimEnd {
        shared: Arc::clone(shared),
        chan,
        side,
        bound: transfer == Transfer::Bound,
        armed: true,
    }))
}

/// One simulated process's transport.
pub(crate) struct SimIpc {
    pub(crate) shared: Arc<Shared>,
    pub(crate) pid: ProcessId,
    pub(crate) generation: u32,
}

impl SimIpc {
    pub(crate) fn me(&self) -> Owner {
        Owner::Proc(self.pid, self.generation)
    }

    /// The end `h` names, if it is this transport's, open, and ours.
    fn held(&self, st: &State, h: &Handle) -> Option<(u64, Side)> {
        let e = h.object().downcast_ref::<SimEnd>()?;
        if !Arc::ptr_eq(&e.shared, &self.shared) {
            return None;
        }
        let c = st.ipc.chans.get(&e.chan)?;
        let end = match e.side {
            Side::Sender => c.sender,
            Side::Receiver => c.receiver,
        };
        (end.open && end.owner == self.me()).then_some((e.chan, e.side))
    }
}

impl Ipc for SimIpc {
    fn channel(
        &self,
        capacity: u32,
        transfer: Transfer,
    ) -> Result<(ChannelSender, ChannelReceiver), IpcError> {
        let mut st = self.shared.lock();
        let Some(pname) = st.proc(self.pid, self.generation).map(|p| p.name) else {
            return Err(IpcError::Closed);
        };
        let id = st
            .ipc
            .new_chan(capacity, transfer, self.me(), self.me(), self.pid);
        st.trace.record(format_args!(
            "ipc channel #{id} {pname}.{} cap={capacity} {transfer:?}",
            self.generation
        ));
        drop(st);
        let tx = ChannelSender::from_handle(end_handle(&self.shared, id, Side::Sender, transfer));
        let rx =
            ChannelReceiver::from_handle(end_handle(&self.shared, id, Side::Receiver, transfer));
        match (tx, rx) {
            (Ok(tx), Ok(rx)) => Ok((tx, rx)),
            // Unreachable: the kinds are the ones just made.
            _ => Err(IpcError::Refused),
        }
    }

    fn send(
        &self,
        to: &ChannelSender,
        bytes: Vec<u8>,
        handles: Vec<Handle>,
    ) -> Result<(), IpcError> {
        if bytes.len() > MAX_MESSAGE_BYTES || handles.len() > MAX_MESSAGE_HANDLES {
            return Err(IpcError::TooLarge);
        }
        let mut st = self.shared.lock();
        let Some((id, Side::Sender)) = self.held(&st, to.handle()) else {
            drop(st);
            return Err(IpcError::Refused);
        };
        // Every handle: ours, held, and a bound sender only from its maker,
        // once.
        let mut moved = Vec::with_capacity(handles.len());
        let broker = self.is_broker(&st);
        for h in &handles {
            if h.region().is_some() {
                // A region: held, a writer only from the broker, a
                // producer never (D6).
                if !Regions::may_send(&st, h, self.me(), broker) {
                    drop(st);
                    return Err(IpcError::Refused);
                }
                continue;
            }
            let Some((hid, side)) = self.held(&st, h) else {
                drop(st);
                return Err(IpcError::Refused);
            };
            if hid == id {
                // A channel's own end inside its own message.
                drop(st);
                return Err(IpcError::Refused);
            }
            // A bound channel's receiver never moves; its sender moves
            // once, from its maker (IPC-R2 (a)).
            if let Some(hc) = st.ipc.chans.get(&hid)
                && hc.transfer == Transfer::Bound
                && (side == Side::Receiver || hc.creator != self.pid || hc.granted)
            {
                drop(st);
                return Err(IpcError::Refused);
            }
            moved.push((hid, side));
        }
        let Some(c) = st.ipc.chans.get(&id) else {
            drop(st);
            return Err(IpcError::Closed);
        };
        if !c.receiver.open || c.fault.is_some() || c.severed {
            drop(st);
            return Err(IpcError::Closed);
        }
        if c.queue.len() >= usize::try_from(c.capacity).unwrap_or(usize::MAX) {
            drop(st);
            return Err(IpcError::Full);
        }
        if let Some(kind) = st.ipc.corrupt.remove(&self.pid) {
            // The adversarial-peer fault: the receiving transport refuses
            // this record. Its handles stay here and close when `handles`
            // drops, after the lock is released, as a refused record's
            // handles are closed by the receiving OS transport.
            st.trace
                .record(format_args!("ipc send #{id} refused {kind:?}"));
            if let Some(c) = st.ipc.chans.get_mut(&id) {
                c.fault = Some(Fault {
                    kind,
                    sender: Some(self.pid.0),
                });
            }
            kick(&mut st, id);
            drop(st);
            drop(handles);
            return Ok(());
        }
        let plan = st.config.faults;
        if crate::world::fires(&mut st.faults.ipc_drop, plan.ipc_drop_per_million) {
            // E-R8 drop: the message is lost and the channel closes. Its
            // handles close with it, once the lock is released.
            st.trace.record(format_args!(
                "ipc fault drop #{id} {} bytes {} handles",
                bytes.len(),
                handles.len()
            ));
            if let Some(c) = st.ipc.chans.get_mut(&id) {
                c.severed = true;
            }
            kick(&mut st, id);
            drop(st);
            drop(handles);
            return Ok(());
        }
        let now = st.now.as_nanos();
        let ready_at = if crate::world::fires(&mut st.faults.ipc_delay, plan.ipc_delay_per_million)
        {
            let max = u64::try_from(plan.ipc_delay_max.as_nanos()).unwrap_or(u64::MAX);
            let extra = match max.checked_add(1) {
                Some(width) => crate::world::below(&mut st.faults.ipc_delay, width),
                None => st.faults.ipc_delay.next_u64(),
            };
            st.trace
                .record(format_args!("ipc fault delay #{id} +{extra}"));
            now.saturating_add(extra)
        } else {
            now
        };
        for h in handles.iter().filter(|h| h.region().is_some()) {
            Regions::send(&mut st, h);
        }
        for (hid, side) in moved {
            if let Some(hc) = st.ipc.chans.get_mut(&hid) {
                hc.end_mut(side).owner = Owner::Transit;
                if side == Side::Sender && hc.transfer == Transfer::Bound {
                    hc.granted = true;
                }
            }
        }
        st.trace.record(format_args!(
            "ipc send #{id} {} bytes {} handles",
            bytes.len(),
            handles.len()
        ));
        if let Some(c) = st.ipc.chans.get_mut(&id) {
            c.queue.push_back(Msg {
                bytes,
                handles,
                from: self.me(),
                ready_at,
                yielded: false,
            });
        }
        kick(&mut st, id);
        Ok(())
    }

    fn listen(
        &self,
        from: ChannelReceiver,
        seq: SequenceHandle,
        sink: Sink,
    ) -> Result<Listening, IpcError> {
        self.start_listening(from, None, seq, sink)
    }

    fn listen_bound(
        &self,
        from: ChannelReceiver,
        peer: OsPeer,
        seq: SequenceHandle,
        sink: Sink,
    ) -> Result<Listening, IpcError> {
        let Ok(generation) = u32::try_from(peer.generation()) else {
            return Err(IpcError::Refused);
        };
        let owner = Owner::Proc(ProcessId(peer.pid()), generation);
        self.start_listening(from, Some(owner), seq, sink)
    }
    fn allocate_region(&self, len: usize, owner: OsPeer) -> Result<RegionWriter, IpcError> {
        self.region_allocate(len, owner)
    }

    fn release_owner(&self, owner: OsPeer) {
        self.region_release_owner(owner);
    }

    fn map_region(&self, region: &Handle) -> Result<Mapping, IpcError> {
        self.region_map(region)
    }

    fn seal_region(&self, writer: RegionWriter) -> Result<ReadOnlyRegion, RegionRefused> {
        self.region_seal(writer)
    }

    fn share_region(&self, writer: RegionWriter) -> Result<ProducerRegion, RegionRefused> {
        self.region_share(writer)
    }

    fn consumer_of(&self, producer: &ProducerRegion) -> Result<ConsumerRegion, IpcError> {
        self.region_consumer(producer)
    }

    fn bootstrap(&self) -> Option<Bootstrap> {
        let mut st = self.shared.lock();
        let (up, down, parent) = *st.ipc.boot.get(&self.pid)?;
        let ours = |e: &End| e.open && e.owner == self.me();
        let ok = st.ipc.chans.get(&up).is_some_and(|c| ours(&c.sender))
            && st.ipc.chans.get(&down).is_some_and(|c| ours(&c.receiver));
        st.ipc.boot.remove(&self.pid);
        drop(st);
        if !ok {
            return None;
        }
        let to_parent =
            ChannelSender::from_handle(end_handle(&self.shared, up, Side::Sender, Transfer::Free))
                .ok()?;
        let from_parent = ChannelReceiver::from_handle(end_handle(
            &self.shared,
            down,
            Side::Receiver,
            Transfer::Free,
        ))
        .ok()?;
        Some(Bootstrap {
            to_parent,
            from_parent,
            parent,
        })
    }
}

impl SimIpc {
    /// [`Ipc::listen`] and [`Ipc::listen_bound`]: `peer` is the bound
    /// peer, `None` for a free receiver.
    fn start_listening(
        &self,
        from: ChannelReceiver,
        peer: Option<Owner>,
        seq: SequenceHandle,
        sink: Sink,
    ) -> Result<Listening, IpcError> {
        let mut st = self.shared.lock();
        let Some((id, Side::Receiver)) = self.held(&st, from.handle()) else {
            drop(st);
            return Err(IpcError::Refused);
        };
        let si = usize::try_from(seq.raw()).unwrap_or(usize::MAX);
        if st
            .proc(self.pid, self.generation)
            .and_then(|p| p.seqs.get(si))
            .is_none()
        {
            drop(st);
            return Err(IpcError::Sequence);
        }
        let Some(c) = st.ipc.chans.get_mut(&id) else {
            drop(st);
            return Err(IpcError::Closed);
        };
        // A bound receiver listens only with its peer named.
        if (c.transfer == Transfer::Bound && peer.is_none()) || c.listen.is_some() {
            drop(st);
            return Err(IpcError::Refused);
        }
        c.listen = Some(Listen {
            pid: self.pid,
            generation: self.generation,
            seq: si,
            sink: Arc::new(Mutex::new(sink)),
            peer,
        });
        let pending = c.queue.len();
        st.trace
            .record(format_args!("ipc listen #{id} pending={pending}"));
        kick(&mut st, id);
        drop(st);
        // The receiver object becomes the listener: disarm it, so only the
        // listener's drop closes the end.
        if let Ok(mut end) = from.into_handle().into_object().downcast::<SimEnd>() {
            end.armed = false;
        }
        Ok(Listening::new(end_handle(
            &self.shared,
            id,
            Side::Receiver,
            Transfer::Free,
        )))
    }
}

/// Makes `child`'s launch channels to `parent` and returns the parent's
/// ends: what launching a process gives its launcher (phase E's
/// launcher calls the OS equivalent).
pub(crate) fn link(
    shared: &Arc<Shared>,
    parent: ProcessId,
    child: ProcessId,
    capacity: u32,
) -> Option<(ChannelReceiver, ChannelSender)> {
    let mut st = shared.lock();
    let pgen = st
        .procs
        .get(usize::try_from(parent.0).ok()?)
        .filter(|p| p.alive)?
        .generation;
    let cgen = st
        .procs
        .get(usize::try_from(child.0).ok()?)
        .filter(|p| p.alive)?
        .generation;
    if let Some(p) = st.procs.get_mut(usize::try_from(child.0).ok()?) {
        // A launched process is never the broker (D6: only the broker
        // allocates regions and sends writers).
        p.launched = true;
    }
    let (p, c) = (Owner::Proc(parent, pgen), Owner::Proc(child, cgen));
    let up = st.ipc.new_chan(capacity, Transfer::Free, c, p, parent);
    let down = st.ipc.new_chan(capacity, Transfer::Free, p, c, parent);
    let parent_peer = OsPeer::new(parent.0, u64::from(pgen));
    st.ipc.boot.insert(child, (up, down, parent_peer));
    st.trace.record(format_args!(
        "ipc link {}->{} #{up} #{down}",
        parent.0, child.0
    ));
    drop(st);
    // Handle objects lock the world when dropped: make them unlocked.
    let from_child =
        ChannelReceiver::from_handle(end_handle(shared, up, Side::Receiver, Transfer::Free));
    let to_child =
        ChannelSender::from_handle(end_handle(shared, down, Side::Sender, Transfer::Free));
    let (Ok(from_child), Ok(to_child)) = (from_child, to_child) else {
        // Unreachable (the kinds are the ones just made); if it were not,
        // leave nothing registered.
        let mut st = shared.lock();
        st.ipc.boot.remove(&child);
        let mut out = Dropped::new();
        for (id, side) in [
            (up, Side::Sender),
            (up, Side::Receiver),
            (down, Side::Sender),
            (down, Side::Receiver),
        ] {
            close(&mut st, id, side, &mut out);
        }
        drop(st);
        drop(out);
        return None;
    };
    Some((from_child, to_child))
}
