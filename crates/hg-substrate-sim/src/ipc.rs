//! The simulated IPC transport (Stage 1 design 4.2, "in-memory queues
//! delivered by the seeded scheduler").
//!
//! Channels live in the world's state. A send queues the message and
//! puts one delivery task on the receiving sequence, so delivery order
//! across processes is the scheduler's seeded choice and every send,
//! delivery and close is in the trace. The transport keeps each end's
//! owner, (process, incarnation), and enforces what the OS backends
//! will:
//! - a process sends only handles it holds, and only this transport's
//!   (IPC-R1);
//! - a bound sender is sent at most once, by the process that made the
//!   channel (IPC-R2 (a));
//! - a crash closes every end the process held, so its peers see
//!   [`Delivery::Closed`] (IPC-R8's closure fault).
//!
//! Handle objects lock the world when dropped, so nothing that may hold
//! one (a queued message, a sink) is dropped with the lock held: it is
//! moved out and dropped after.

use core::any::Any;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};

use hg_substrate::ipc::{
    Bootstrap, ChannelReceiver, ChannelSender, Delivery, Flow, Handle, HandleKind, HandleObject,
    Ipc, IpcError, Listening, MAX_MESSAGE_BYTES, MAX_MESSAGE_HANDLES, Sink, Transfer,
};
use hg_substrate::{SequenceHandle, Task};

use crate::world::{ProcessId, SEQUENCE_CAPACITY, Shared, State};

/// Which end of a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    Sender,
    Receiver,
}

/// Who holds an end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Owner {
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
}

struct Chan {
    capacity: u32,
    transfer: Transfer,
    creator: ProcessId,
    granted: bool,
    sender: End,
    receiver: End,
    queue: VecDeque<(Vec<u8>, Vec<Handle>)>,
    listen: Option<Listen>,
    closed_delivered: bool,
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
    boot: BTreeMap<ProcessId, (u64, u64)>,
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
                listen: None,
                closed_delivered: false,
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
    }

    /// Everything, for the world's teardown.
    pub(crate) fn drain(&mut self, out: &mut Dropped) {
        for (_, mut c) in std::mem::take(&mut self.chans) {
            for (_, handles) in c.queue.drain(..) {
                out.push(Box::new(handles));
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
        Side::Sender => {
            if let Some(c) = st.ipc.chans.get(&id)
                && let Some(l) = &c.listen
            {
                let (pid, generation, seq) = (l.pid, l.generation, l.seq);
                // The closing notice may exceed the sequence's capacity by
                // one: it must not be lost.
                let _queued = queue_delivery(st, id, pid, generation, seq, true);
            }
        }
        Side::Receiver => {
            if let Some(c) = st.ipc.chans.get_mut(&id) {
                for (_, handles) in c.queue.drain(..) {
                    out.push(Box::new(handles));
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
    {
        st.ipc.chans.remove(&id);
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

/// One delivery task: the next message, or the closing notice.
fn deliver(shared: &Arc<Shared>, id: u64) {
    let mut st = shared.lock();
    let Some(c) = st.ipc.chans.get_mut(&id) else {
        return;
    };
    let Some(l) = &c.listen else { return };
    let (pid, generation, sink) = (l.pid, l.generation, Arc::clone(&l.sink));
    let delivery = if let Some((bytes, handles)) = c.queue.pop_front() {
        Delivery::Message { bytes, handles }
    } else if !c.sender.open && !c.closed_delivered {
        c.closed_delivered = true;
        Delivery::Closed
    } else {
        return;
    };
    let what = match &delivery {
        Delivery::Message { bytes, handles } => {
            for h in handles {
                if let Some(e) = h.object().downcast_ref::<SimEnd>()
                    && let Some(hc) = st.ipc.chans.get_mut(&e.chan)
                {
                    hc.end_mut(e.side).owner = Owner::Proc(pid, generation);
                }
            }
            format!("{} bytes {} handles", bytes.len(), handles.len())
        }
        Delivery::Closed => "closed".to_owned(),
    };
    st.trace.record(format_args!("ipc deliver #{id} {what}"));
    drop(st);
    let flow = sink.lock().unwrap_or_else(PoisonError::into_inner)(delivery);
    if flow == Flow::Close {
        let mut out = Dropped::new();
        let mut st = shared.lock();
        close(&mut st, id, Side::Receiver, &mut out);
        drop(st);
        drop(out);
    }
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
    fn me(&self) -> Owner {
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
        for h in &handles {
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
        if !c.receiver.open {
            drop(st);
            return Err(IpcError::Closed);
        }
        if c.queue.len() >= usize::try_from(c.capacity).unwrap_or(usize::MAX) {
            drop(st);
            return Err(IpcError::Full);
        }
        let listen = c.listen.as_ref().map(|l| (l.pid, l.generation, l.seq));
        if let Some((pid, generation, seq)) = listen
            && !queue_delivery(&mut st, id, pid, generation, seq, false)
        {
            drop(st);
            return Err(IpcError::Full);
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
            c.queue.push_back((bytes, handles));
        }
        Ok(())
    }

    fn listen(
        &self,
        from: ChannelReceiver,
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
        c.listen = Some(Listen {
            pid: self.pid,
            generation: self.generation,
            seq: si,
            sink: Arc::new(Mutex::new(sink)),
        });
        let pending = c.queue.len();
        let closed = !c.sender.open;
        st.trace
            .record(format_args!("ipc listen #{id} pending={pending}"));
        for _ in 0..pending {
            let _queued = queue_delivery(&mut st, id, self.pid, self.generation, si, true);
        }
        if closed {
            let _queued = queue_delivery(&mut st, id, self.pid, self.generation, si, true);
        }
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

    fn bootstrap(&self) -> Option<Bootstrap> {
        let mut st = self.shared.lock();
        let (up, down) = *st.ipc.boot.get(&self.pid)?;
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
        })
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
    let (p, c) = (Owner::Proc(parent, pgen), Owner::Proc(child, cgen));
    let up = st.ipc.new_chan(capacity, Transfer::Free, c, p, parent);
    let down = st.ipc.new_chan(capacity, Transfer::Free, p, c, parent);
    st.ipc.boot.insert(child, (up, down));
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
