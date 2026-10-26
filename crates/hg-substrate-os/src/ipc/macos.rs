//! The macOS IPC transport (Stage 1 4.2; M1 phase D4): channels are Mach
//! ports, handles cross as port rights. Written from Apple's published
//! Mach documentation and the black-box probes in `research_notes/M1 D4
//! macOS IPC/mach_facts.md`; no header, no kernel source and no other
//! project's code is a source.
//!
//! **Shape.** A channel is two ports. The data port's receive right is
//! the receiver end; its send right, with the credit port's receive
//! right, is the sender end. Every data message carries a send-once
//! reply right made from the sender's credit port. Credits are
//! cumulative (the count delivered so far), so the receiver keeps only
//! the newest reply right, releasing the older (rule M3: at most one per
//! channel), and when a delivery task starts with no further message
//! queued it sends the credit back on it. A sender so knows exactly how
//! many of its messages are undelivered by the end of each batch
//! (`Full` at the channel's capacity, as in the simulator), and the
//! credit always reaches whoever holds the credit port now, wherever the
//! sender end has moved. The receiver end holds no right to the credit
//! port at all. Each data port's queue limit is the channel's capacity
//! (at most 1024, P16), so an honest sender never finds it full; a
//! message that does (a capacity above 1024, or a former holder's queue)
//! waits in the sender end's outbox, sent again as the kernel handed it
//! back (P17) when the port's send-possible notification says there is
//! room (P19). `send` never blocks (timeout 0, P17) and never reports
//! `Full` early. One IO thread per process waits on a port set with no
//! timeout (no idle wakeups, P23), reads every listening data port's
//! messages, has the pure core (`ipc::mach::decide`) judge each one, and
//! queues what passes; each listening channel keeps at most one delivery
//! task on its sequence. It keeps reading a receiver past its capacity,
//! so a peer that overruns it is refused, not left to fill the kernel.
//!
//! **What the kernel attests and what the transport checks**
//! (security scope IPC-R1, R2, R5; SEC-4):
//! - every message carries the sender's audit token (pid and pid
//!   version, P1, P3), which no sender can forge (P2); a violation names
//!   that process;
//! - every received right is checked against what the kernel says it
//!   is (`ipc::mach::accept`): plain ports only, never a task, thread,
//!   host, memory-entry or IOKit port (P8), never a right this process
//!   already held, never one name twice; anything else refuses the
//!   message and every right in it is released;
//! - a receiver listened to with `listen_bound` accepts messages from
//!   the one process (pid and pid version) it names; a message from
//!   another is blamed on that peer, the only process that could have
//!   passed its sender on;
//! - the kernel's own notifications are recognised by their all-zero
//!   token (P4) and only ever arrive on ports whose requests this
//!   transport armed, aimed at the data port itself: no peer holds a
//!   send-once right to any port of ours.
//!
//! **Closed.** A data port's no-senders request is aimed at the port
//! itself, so the notification queues behind every message already sent
//! (P20) and the receiver reports `Closed` after them. A sender learns
//! its receiver is gone when a send fails with an invalid destination
//! (P21). Rights are never inherited by a child (P28), so unlike Linux
//! there is no spawn window: `Closed` is prompt.
//!
//! **Lock discipline.** As on Linux: one mutex guards the transport's
//! state; nothing that may own a handle is dropped, and no sink runs,
//! with it held.

pub(crate) mod sys;

use core::any::Any;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant as StdInstant};

use hg_substrate::ipc::{
    Bootstrap, ChannelReceiver, ChannelSender, Delivery, Fault, Flow, Handle, HandleKind,
    HandleObject, Ipc, IpcError, Listening, MAX_MESSAGE_BYTES, MAX_MESSAGE_HANDLES, OsPeer, Sink,
    Transfer,
};
use hg_substrate::{SequenceHandle, Task};

use crate::executor::OsExecutor;
use crate::ipc::common::codec::{
    Credit, HandleDesc, WireKind, decode_credit, encode_credit, encode_data,
};
use crate::ipc::common::flow::{ReceiverFlow, SenderFlow};
use crate::ipc::mach::accept::{Held, SlotFacts};
use crate::ipc::mach::decide::{Arrival, Binding, Decision, decide, decide_launch};
use crate::ipc::mach::notify::{Kind, classify};
use crate::ipc::mach::wire::{
    self, COPY_SEND, ID_CREDIT, ID_DATA, ID_HELLO, ID_HELLO_REPLY, MAKE_SEND, MAKE_SEND_ONCE,
    MAX_SLOTS, MOVE_RECEIVE, MOVE_SEND, MOVE_SEND_ONCE, NAME_DEAD, NAME_NULL, Outgoing as WireMsg,
    RCV_TOO_LARGE, RECEIVE_BUFFER, SEND_INVALID_DEST, SEND_TIMED_OUT, STATUS_MSGCOUNT, Token,
};
use sys::{Buf, Kr};

/// The capacity of a process's launch channels (Stage 1 4.1).
pub const BOOTSTRAP_CAPACITY: u32 = 64;

/// How long a message carrying a sender end waits for that end's own
/// queued messages before the transport gives up on them (the carried
/// end then reaches its new holder closed).
const TRANSFER_DEADLINE: Duration = Duration::from_secs(5);

/// How long a launched process waits for its launcher's answer, and a
/// launcher for a child's hello.
const LAUNCH_WAIT_MS: u32 = 10_000;

/// Bytes of a credit message with its trailer.
const CREDIT_BUFFER: usize = 256;

/// One item for a listener.
enum Item {
    Message(Vec<u8>, Vec<Handle>),
    Closed,
    Violation(Fault),
}

/// A message waiting in a sender end's outbox: the bytes exactly as the
/// kernel last left them (P17).
struct Outgoing {
    buf: Buf,
    size: usize,
    /// Sender ends inside this message whose own outbox must reach the
    /// kernel first (order across a transfer).
    after: Vec<u64>,
    /// When it started waiting on `after`.
    since: StdInstant,
}

/// A listener's state.
struct Listener {
    seq: SequenceHandle,
    sink: Arc<Mutex<Sink>>,
    queue: VecDeque<Item>,
    /// A delivery task is on the sequence.
    draining: bool,
    /// Nothing more will be read.
    ended: bool,
    /// For a bound receiver, the one process whose messages it accepts.
    peer: Option<Token>,
    /// The reply right of the newest message received, if no credit has
    /// gone back on it yet (rule M3): at most one per channel. Credits
    /// are cumulative, so one right is enough; an older one is released
    /// when a newer arrives.
    reply: Option<u32>,
}

/// What an end is.
enum Role {
    Sender {
        /// The data port's send right.
        data: u32,
        /// The credit port's receive right.
        credit: u32,
        flow: SenderFlow,
        bound: bool,
        /// This process created the channel.
        made_here: bool,
        /// A bound sender was already sent once.
        granted: bool,
        outbox: VecDeque<Outgoing>,
        /// Its handle was dropped; it stays until its outbox drains.
        dropped: bool,
        /// Sends failed or the receiver is gone.
        broken: bool,
        /// Its send-possible request is armed.
        armed: bool,
        /// It is carried in a message waiting in another end's outbox:
        /// its rights belong to that message, so when it leaves the table
        /// they are not released.
        carried: bool,
    },
    Receiver {
        /// The data port's receive right.
        data: u32,
        flow: ReceiverFlow,
        bound: bool,
        listener: Option<Listener>,
        /// This process's closer port for this end: the data port's
        /// no-senders request notifies it (R8: never the data port itself,
        /// so the data port has no send-once right out when it moves).
        closer: Option<u32>,
        /// The no-senders notification came: no sender is left, so once
        /// the data port's queue is empty the channel is `Closed`.
        peer_gone: bool,
    },
}

#[derive(Default)]
struct State {
    ends: BTreeMap<u64, Role>,
    next: u64,
    /// Listening receivers by data port name.
    listening: BTreeMap<u32, u64>,
    /// Receivers by closer port name.
    closers: BTreeMap<u32, u64>,
    /// Send references this process's sender ends hold, by name.
    send_refs: BTreeMap<u32, u32>,
    /// Data port names of this process's receiver ends.
    receivers: BTreeMap<u32, u64>,
    /// Stand-in data ports that close at their first message.
    sinks: std::collections::BTreeSet<u32>,
    stopping: bool,
}

impl Held for State {
    fn send_refs(&self, name: u32) -> u32 {
        self.send_refs.get(&name).copied().unwrap_or(0)
    }
    fn receiver(&self, name: u32) -> bool {
        self.receivers.contains_key(&name) || self.sinks.contains(&name)
    }
}

impl State {
    /// Registers an end under a new id. Ids are never reused: one per
    /// nanosecond would take 584 years to run out of 64 bits.
    fn insert(&mut self, role: Role) -> u64 {
        self.next = self.next.wrapping_add(1);
        let id = self.next;
        match &role {
            Role::Sender { data, .. } => {
                let r = self.send_refs.entry(*data).or_insert(0);
                *r = r.saturating_add(1);
            }
            Role::Receiver { data, closer, .. } => {
                self.receivers.insert(*data, id);
                if let Some(c) = closer {
                    self.closers.insert(*c, id);
                }
            }
        }
        self.ends.insert(id, role);
        id
    }

    /// Takes an end out of the registry. The caller releases its rights
    /// (or they left in a message).
    fn take(&mut self, id: u64) -> Option<Role> {
        let role = self.ends.remove(&id)?;
        match &role {
            Role::Sender { data, .. } => {
                if let Some(r) = self.send_refs.get_mut(data) {
                    *r = r.saturating_sub(1);
                    if *r == 0 {
                        self.send_refs.remove(data);
                    }
                }
            }
            Role::Receiver { data, closer, .. } => {
                self.receivers.remove(data);
                self.listening.remove(data);
                if let Some(c) = closer {
                    self.closers.remove(c);
                }
            }
        }
        Some(role)
    }
}

/// Takes this process's own notification requests off end `id`'s port
/// before its rights move: a receiver's no-senders request, and a
/// sender's send-possible request unless its own outbox still needs it
/// (then it comes off when that outbox drains, before the message that
/// carries it is sent).
fn unrequest(st: &mut State, id: u64, set: u32) {
    match st.ends.get_mut(&id) {
        Some(Role::Receiver { data, closer, .. }) => {
            sys::cancel_no_senders(*data);
            if let Some(c) = closer.take() {
                drop_closer(c, set);
                st.closers.remove(&c);
            }
        }
        Some(Role::Sender {
            data,
            armed,
            outbox,
            ..
        }) if *armed && outbox.is_empty() => {
            sys::cancel_send_possible(*data);
            *armed = false;
        }
        Some(Role::Sender { .. }) | None => {}
    }
}

/// The ends carried in the head of sender `id`'s outbox that waited for
/// their own queues.
fn outbox_head_after(st: &State, id: u64) -> Vec<u64> {
    match st.ends.get(&id) {
        Some(Role::Sender { outbox, .. }) => {
            outbox.front().map(|h| h.after.clone()).unwrap_or_default()
        }
        Some(Role::Receiver { .. }) | None => Vec::new(),
    }
}

/// Re-arms what [`unrequest`] took off, when the rights did not move.
fn rerequest(st: &mut State, id: u64, set: u32) {
    let Some(Role::Receiver { data, closer, .. }) = st.ends.get_mut(&id) else {
        return;
    };
    *closer = arm_closer(*data, 1, set);
    if let Some(c) = *closer {
        st.closers.insert(c, id);
    }
}

/// A closer port for receiver data port `data`, in the IO thread's set,
/// that the data port's no-senders request notifies once its make-send
/// count reaches `sync` and no send right is left (P20). `None` if it
/// cannot be made: the end then never reports `Closed` on its own.
fn arm_closer(data: u32, sync: u32, set: u32) -> Option<u32> {
    let c = sys::allocate_receive().ok()?;
    if sys::insert_member(c, set).is_err() {
        sys::destroy_receive(c);
        return None;
    }
    if sys::request_no_senders(data, sync, c).is_err() {
        drop_closer(c, set);
        return None;
    }
    Some(c)
}

/// Destroys closer port `c` (its request was cancelled or its data port
/// destroyed).
fn drop_closer(c: u32, set: u32) {
    sys::extract_member(c, set);
    sys::destroy_receive(c);
}

/// Releases an end's rights and whatever it still holds.
fn release_role(role: Role, set: u32) -> Vec<Item> {
    match role {
        Role::Sender {
            data,
            credit,
            outbox,
            ..
        } => {
            for o in outbox {
                release_unsent(&o);
            }
            sys::deallocate(data);
            sys::destroy_receive(credit);
            Vec::new()
        }
        Role::Receiver {
            data,
            listener,
            closer,
            ..
        } => {
            sys::extract_member(data, set);
            sys::destroy_receive(data);
            if let Some(c) = closer {
                drop_closer(c, set);
            }
            let Some(l) = listener else {
                return Vec::new();
            };
            if let Some(r) = l.reply {
                sys::deallocate(r);
            }
            l.queue.into_iter().collect()
        }
    }
}

/// Releases the rights in an outbox message never sent.
fn release_unsent(o: &Outgoing) {
    sys::release_outgoing(o.buf.bytes().get(..o.size).unwrap_or(&[]));
}

/// The transport's shared core.
pub(crate) struct Inner {
    exec: Arc<OsExecutor>,
    /// The port set the IO thread waits on.
    set: u32,
    /// A port in the set, to wake the IO thread (qlimit 1).
    wake: u32,
    /// A port in the set for send-possible notifications.
    notify: u32,
    state: Mutex<State>,
    token: Token,
    boot: Mutex<Option<Boot>>,
    /// Tests only: the transfer deadline's stand-in cannot be built.
    #[cfg(test)]
    fail_stand_in: std::sync::atomic::AtomicBool,
}

impl Drop for Inner {
    fn drop(&mut self) {
        sys::destroy_receive(self.wake);
        sys::destroy_receive(self.notify);
        sys::destroy_set(self.set);
    }
}

/// A launched process's launch channel ends, as received.
struct Boot {
    up_data: u32,
    up_credit: u32,
    up_flow: SenderFlow,
    down_data: u32,
    down_flow: ReceiverFlow,
    parent: Token,
}

impl Inner {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn wake_io(&self) {
        let mut buf = Buf::new(64);
        let m = WireMsg {
            dest: self.wake,
            dest_disposition: MAKE_SEND,
            reply: NAME_NULL,
            reply_disposition: 0,
            id: 0,
            ports: &[],
            body: &[],
        };
        if let Some(size) = wire::encode(&m, buf.bytes_mut()) {
            // A full wake port already wakes the thread (P38).
            let _ = sys::send(&mut buf, size);
        }
    }

    /// Posts `id`'s delivery task if it has items and none is posted.
    fn kick(self: &Arc<Self>, st: &mut State, id: u64) {
        let Some(Role::Receiver {
            listener: Some(l), ..
        }) = st.ends.get_mut(&id)
        else {
            return;
        };
        if l.draining || l.queue.is_empty() {
            return;
        }
        let me = Arc::clone(self);
        if self
            .exec
            .post_reserved(l.seq, Task::new(move || me.deliver(id)))
            .is_ok()
        {
            l.draining = true;
        }
    }

    /// One delivery task: the next item of listener `id`.
    fn deliver(self: &Arc<Self>, id: u64) {
        let mut st = self.lock();
        let Some(Role::Receiver {
            flow,
            listener: Some(l),
            ..
        }) = st.ends.get_mut(&id)
        else {
            return;
        };
        l.draining = false;
        let Some(item) = l.queue.pop_front() else {
            return;
        };
        let sink = Arc::clone(&l.sink);
        let mut final_item = false;
        let delivery = match item {
            Item::Message(bytes, handles) => {
                // The credit goes back before the sink runs, on the one
                // reply right held, once no further message is queued: it
                // counts every delivery so far, so Full stays exact.
                if let Ok(c) = flow.on_deliver()
                    && !l.queue.iter().any(|i| matches!(i, Item::Message(..)))
                    && let Some(reply) = l.reply.take()
                {
                    send_credit(reply, c);
                }
                Delivery::Message { bytes, handles }
            }
            Item::Closed => {
                final_item = true;
                Delivery::Closed
            }
            Item::Violation(f) => {
                final_item = true;
                Delivery::Violation(f)
            }
        };
        drop(st);
        let flow_after = sink.lock().unwrap_or_else(PoisonError::into_inner)(delivery);
        let mut st = self.lock();
        let gone = if flow_after == Flow::Close || final_item {
            st.take(id)
        } else {
            self.kick(&mut st, id);
            None
        };
        drop(st);
        let leftovers = gone.map(|r| release_role(r, self.set));
        drop(leftovers);
    }

    /// Releases an end whose handle was dropped.
    fn release(&self, id: u64) {
        let mut st = self.lock();
        let keep = match st.ends.get_mut(&id) {
            Some(Role::Sender {
                outbox,
                dropped,
                broken,
                ..
            }) if !outbox.is_empty() && !*broken => {
                *dropped = true;
                true
            }
            _ => false,
        };
        let gone = if keep { None } else { st.take(id) };
        drop(st);
        let leftovers = gone.map(|r| release_role(r, self.set));
        drop(leftovers);
    }

    /// Stops reading `id` and queues its last item.
    fn end_listener(self: &Arc<Self>, st: &mut State, id: u64, last: Item) {
        if let Some(Role::Receiver {
            data,
            listener: Some(l),
            ..
        }) = st.ends.get_mut(&id)
            && !l.ended
        {
            l.ended = true;
            sys::extract_member(*data, self.set);
            let d = *data;
            st.listening.remove(&d);
            if let Some(Role::Receiver {
                listener: Some(l), ..
            }) = st.ends.get_mut(&id)
            {
                l.queue.push_back(last);
            }
        }
        self.kick(st, id);
    }

    /// Judges one message received on listening receiver `id`.
    fn on_message(self: &Arc<Self>, id: u64, buf: &Buf) {
        let bytes = buf.bytes();
        let Ok(l) = wire::layout(bytes) else {
            return;
        };
        let kind = classify(&l.header, l.token);
        match kind {
            Kind::Ignore | Kind::SendPossible => {
                sys::release_received(buf);
                return;
            }
            Kind::NoSenders => {
                // This process aims no request at a data port (its closer
                // port takes them), so this one is not its own: ignored.
                sys::release_received(buf);
                return;
            }
            Kind::NoToken => {
                sys::release_received(buf);
                let mut st = self.lock();
                self.end_listener(&mut st, id, Item::Closed);
                return;
            }
            Kind::Peer(_) => {}
        }
        // The kernel's facts on each right, as far as any message could
        // need them (one more than the most a record takes, to refuse).
        let facts: Vec<SlotFacts> = wire::descs(bytes, &l)
            .take(MAX_SLOTS.saturating_add(1))
            .map(sys::slot_facts)
            .collect();
        let body = wire::unframe(bytes.get(l.body_at..l.end).unwrap_or(&[])).unwrap_or(&[]);
        let arrival = Arrival {
            header: l.header,
            walk: l.walk,
            token: l.token,
            count: l.count,
            slots: &facts,
            body,
        };
        let mut st = self.lock();
        let Some(Role::Receiver {
            flow,
            listener: Some(lis),
            ..
        }) = st.ends.get_mut(&id)
        else {
            drop(st);
            sys::release_received(buf);
            return;
        };
        if lis.ended {
            drop(st);
            sys::release_received(buf);
            return;
        }
        let binding = Binding { peer: lis.peer };
        let mut flow_now = *flow;
        let decision = {
            let held: &State = &st;
            decide(&arrival, binding, &mut flow_now, held)
        };
        match decision {
            Decision::Accept { view, .. } => {
                let kinds: Vec<HandleKind> = decision.kinds().to_vec();
                let descs: Vec<HandleDesc> = view.descs().to_vec();
                let payload = view.payload.to_vec();
                let mut handles = Vec::with_capacity(kinds.len());
                let mut slot = facts.iter();
                for (kind, desc) in kinds.iter().zip(&descs) {
                    let role = match kind {
                        HandleKind::ChannelSender | HandleKind::BoundChannelSender => {
                            let Ok(flow) = SenderFlow::from_transfer(desc) else {
                                break;
                            };
                            let (d, c) = (slot.next(), slot.next());
                            let (Some(d), Some(c)) = (d, c) else {
                                for f in d.into_iter().chain(c) {
                                    sys::release_right(f.name, f.disposition);
                                }
                                break;
                            };
                            Role::Sender {
                                data: d.name,
                                credit: c.name,
                                flow,
                                bound: *kind == HandleKind::BoundChannelSender,
                                made_here: false,
                                granted: false,
                                outbox: VecDeque::new(),
                                dropped: false,
                                broken: false,
                                armed: false,
                                carried: false,
                            }
                        }
                        HandleKind::ChannelReceiver => {
                            let Ok(flow) = ReceiverFlow::from_transfer(desc) else {
                                break;
                            };
                            let Some(d) = slot.next() else {
                                break;
                            };
                            // A moved receive right keeps its queue limit
                            // and its no-senders request (P10, P37):
                            // reset the one, re-arm the other (its
                            // make-send count restarted at 0, P10).
                            let _ = sys::set_qlimit(d.name, desc.capacity);
                            Role::Receiver {
                                data: d.name,
                                flow,
                                bound: false,
                                listener: None,
                                closer: arm_closer(d.name, 0, self.set),
                                peer_gone: false,
                            }
                        }
                    };
                    let hid = st.insert(role);
                    handles.push(Handle::new(Box::new(MacEnd {
                        inner: Arc::clone(self),
                        id: hid,
                        kind: *kind,
                        armed: true,
                    })));
                }
                // `decide` checked every slot, so none is left; if one
                // were, its right would still be ours.
                for f in slot {
                    sys::release_right(f.name, f.disposition);
                }
                if let Some(Role::Receiver {
                    flow,
                    listener: Some(lis),
                    ..
                }) = st.ends.get_mut(&id)
                {
                    *flow = flow_now;
                    if let Some(older) = lis.reply.replace(l.header.remote) {
                        sys::deallocate(older);
                    }
                    lis.queue.push_back(Item::Message(payload, handles));
                }
                self.kick(&mut st, id);
                // Its rights are handles and a kept reply right now.
                buf.adopted();
                sys::deallocate(l.header.voucher);
            }
            Decision::Violation { .. } => {
                let fault = decision.fault();
                self.end_listener(&mut st, id, fault.map_or(Item::Closed, Item::Violation));
                drop(st);
                sys::release_received(buf);
            }
            Decision::Closed => {
                self.end_listener(&mut st, id, Item::Closed);
                drop(st);
                sys::release_received(buf);
            }
        }
    }

    /// Sends what sender ends' outboxes hold, in order, as far as the
    /// kernel and transfer order allow. Returns the earliest deadline of
    /// a message still waiting on a transfer.
    fn flush_all(&self) -> Option<StdInstant> {
        let mut st = self.lock();
        let ids: Vec<u64> = st
            .ends
            .iter()
            .filter(|(_, e)| matches!(e, Role::Sender { outbox, .. } if !outbox.is_empty()))
            .map(|(id, _)| *id)
            .collect();
        let mut deadline: Option<StdInstant> = None;
        let mut gone = Vec::new();
        for _ in 0..=ids.len() {
            let mut progressed = false;
            for &id in &ids {
                progressed |= self.flush_one(&mut st, id, &mut deadline);
            }
            if !progressed {
                break;
            }
        }
        for &id in &ids {
            let done = matches!(
                st.ends.get(&id),
                Some(Role::Sender { outbox, dropped: true, .. }) if outbox.is_empty()
            ) || matches!(
                st.ends.get(&id),
                Some(Role::Sender {
                    dropped: true,
                    broken: true,
                    ..
                })
            );
            if done && let Some(r) = st.take(id) {
                // A carried end's rights went with (or wait in) the
                // message that carries it.
                if !matches!(r, Role::Sender { carried: true, .. }) {
                    gone.push(r);
                }
            }
        }
        drop(st);
        let leftovers: Vec<Vec<Item>> = gone
            .into_iter()
            .map(|r| release_role(r, self.set))
            .collect();
        drop(leftovers);
        deadline
    }

    /// Flushes one outbox; returns whether anything was sent.
    fn flush_one(&self, st: &mut State, id: u64, deadline: &mut Option<StdInstant>) -> bool {
        let mut progressed = false;
        loop {
            let Some(Role::Sender {
                outbox,
                broken: false,
                ..
            }) = st.ends.get(&id)
            else {
                return progressed;
            };
            let Some(head) = outbox.front() else {
                return progressed;
            };
            let since = head.since;
            let waiting: Vec<u64> = head
                .after
                .iter()
                .copied()
                .filter(|dep| {
                    matches!(st.ends.get(dep),
                        Some(Role::Sender { outbox, broken: false, .. }) if !outbox.is_empty())
                })
                .collect();
            if !waiting.is_empty() {
                let due = since.checked_add(TRANSFER_DEADLINE).unwrap_or(since);
                if StdInstant::now() < due {
                    *deadline = Some(deadline.map_or(due, |d| d.min(due)));
                    return progressed;
                }
                // Deadline: the carried ends' queued messages are given up
                // and the carried ends replaced, in the waiting message, by
                // ends of a channel whose receiver is destroyed once it is
                // sent: their new holder sees them closed.
                for dep in waiting {
                    self.give_up(st, id, dep);
                }
            }
            // The carried senders' queues are done: their send-possible
            // requests come off before their rights move (R9).
            let carried: Vec<u64> = outbox_head_after(st, id);
            for dep in carried {
                if let Some(Role::Sender { data, armed, .. }) = st.ends.get_mut(&dep)
                    && *armed
                {
                    sys::cancel_send_possible(*data);
                    *armed = false;
                }
            }
            let Some(Role::Sender { outbox, .. }) = st.ends.get_mut(&id) else {
                return progressed;
            };
            let Some(head) = outbox.front_mut() else {
                return progressed;
            };
            let size = head.size;
            match sys::send(&mut head.buf, size) {
                Ok(()) => {
                    let _sent = outbox.pop_front();
                    progressed = true;
                }
                Err(Kr(SEND_TIMED_OUT)) => {
                    // The notification fires for a send that fails after
                    // the request is armed (P19): arm, then try once more.
                    if self.arm_possible(st, id) {
                        continue;
                    }
                    return progressed;
                }
                Err(_) => {
                    if let Some(Role::Sender { outbox, broken, .. }) = st.ends.get_mut(&id) {
                        for o in outbox.drain(..) {
                            release_unsent(&o);
                        }
                        *broken = true;
                    }
                    return progressed;
                }
            }
        }
    }

    /// Arms sender `id`'s send-possible request, once per full queue.
    /// Returns whether it armed it now.
    fn arm_possible(&self, st: &mut State, id: u64) -> bool {
        if let Some(Role::Sender { data, armed, .. }) = st.ends.get_mut(&id)
            && !*armed
        {
            sys::request_send_possible(*data, self.notify);
            *armed = true;
            return true;
        }
        false
    }

    /// The transfer deadline for `dep`, carried in the head of `id`'s
    /// outbox: its own queued messages are dropped, and its rights in the
    /// waiting message are swapped for a stand-in channel's.
    fn give_up(&self, st: &mut State, id: u64, dep: u64) {
        let Some(Role::Sender {
            data,
            credit,
            outbox: dep_box,
            ..
        }) = st.ends.get_mut(&dep)
        else {
            return;
        };
        // Its queued messages are given up either way.
        for o in dep_box.drain(..) {
            release_unsent(&o);
        }
        let (old_data, old_credit) = (*data, *credit);
        let has_head = matches!(st.ends.get(&id),
            Some(Role::Sender { outbox, .. }) if !outbox.is_empty());
        // Until the stand-in is in the waiting message, the end's real
        // rights belong to that message: everything that can fail comes
        // before anything changes.
        let stand = if has_head { self.stand_in() } else { None };
        let Some((stand_data, stand_credit)) = stand else {
            // No stand-in: the end goes as it is, without its queued
            // messages; its receiver sees the gap as a sequence violation,
            // unblamed. Its send-possible request comes off first (R9).
            if let Some(Role::Sender { data, armed, .. }) = st.ends.get_mut(&dep)
                && *armed
            {
                sys::cancel_send_possible(*data);
                *armed = false;
            }
            if let Some(Role::Sender { outbox, .. }) = st.ends.get_mut(&id)
                && let Some(head) = outbox.front_mut()
            {
                head.after.retain(|d| *d != dep);
            }
            return;
        };
        if let Some(Role::Sender { outbox, .. }) = st.ends.get_mut(&id)
            && let Some(head) = outbox.front_mut()
        {
            let size = head.size;
            swap_ports(
                head.buf.bytes_mut().get_mut(..size).unwrap_or(&mut []),
                &[(old_data, stand_data), (old_credit, stand_credit)],
            );
            head.after.retain(|d| *d != dep);
        }
        st.sinks.insert(stand_data);
        // Its rights are its own again, released when it leaves the
        // table: its receiver then sees no senders, and reports Closed.
        if let Some(Role::Sender {
            broken, carried, ..
        }) = st.ends.get_mut(&dep)
        {
            *broken = true;
            *carried = false;
        }
    }

    /// A stand-in sender end for the transfer deadline: a data port that
    /// is a sink this process keeps (valid when the waiting message
    /// arrives, and destroyed by the IO thread at the first message sent
    /// to it, so its new holder's next send reports Closed, eventually,
    /// as on Linux), and a credit port.
    fn stand_in(&self) -> Option<(u32, u32)> {
        #[cfg(test)]
        if self
            .fail_stand_in
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return None;
        }
        let data = sys::allocate_receive().ok()?;
        let Ok(credit) = sys::allocate_receive() else {
            sys::destroy_receive(data);
            return None;
        };
        if sys::make_send(data).is_err() || sys::insert_member(data, self.set).is_err() {
            sys::destroy_receive(data);
            sys::destroy_receive(credit);
            return None;
        }
        // Its receive right never moves: the request may notify the sink
        // itself, which the IO thread destroys at its first message.
        let _ = sys::request_no_senders(data, 1, data);
        Some((data, credit))
    }

    /// The IO thread.
    fn io_loop(self: &Arc<Self>) {
        let mut buf = Buf::new(RECEIVE_BUFFER);
        let mut deadline: Option<StdInstant> = None;
        let mut failures = 0u32;
        loop {
            let got = if let Some(d) = deadline {
                let left = d.saturating_duration_since(StdInstant::now());
                let ms = u32::try_from(left.as_millis()).unwrap_or(u32::MAX).max(1);
                sys::receive_within(self.set, &mut buf, ms)
            } else {
                sys::receive(self.set, &mut buf)
            };
            let mut flush = deadline.is_some();
            let mut failed = false;
            match got {
                Ok(()) => {
                    if self.dispatch(&buf, &mut flush) {
                        return;
                    }
                }
                Err(Kr(RCV_TOO_LARGE)) => {
                    // The kernel reports only the size, not the port, and
                    // keeps the message at the head of the set (R1): read
                    // it into a buffer of that size (at most the kernel's
                    // 48 MiB, P13) and judge it like any other; a data
                    // message that large decodes as no record, a fault the
                    // audit token attributes.
                    let size = wire::word(buf.bytes(), 4)
                        .and_then(|s| usize::try_from(s).ok())
                        .unwrap_or(0);
                    let mut big = Buf::new(size.saturating_add(wire::TRAILER_MAX));
                    match sys::receive(self.set, &mut big) {
                        Ok(()) => {
                            if self.dispatch(&big, &mut flush) {
                                return;
                            }
                        }
                        Err(_) => failed = true,
                    }
                }
                Err(Kr(wire::RCV_TIMED_OUT)) => {}
                Err(_) => failed = true,
            }
            if failed {
                // The set itself failing is no peer's doing and does not
                // heal; stop rather than spin.
                failures = failures.saturating_add(1);
                if self.lock().stopping || failures > 100 {
                    return;
                }
            } else {
                failures = 0;
            }
            if flush {
                deadline = self.flush_all();
            }
        }
    }

    /// Handles one received message; returns true when the transport is
    /// stopping.
    fn dispatch(self: &Arc<Self>, buf: &Buf, flush: &mut bool) -> bool {
        let local = wire::header(buf.bytes())
            .map(|h| h.local)
            .unwrap_or(NAME_NULL);
        if local == self.wake {
            sys::release_received(buf);
            if self.lock().stopping {
                return true;
            }
            *flush = true;
        } else if local == self.notify {
            // Send possible, or a request's port went away: either way the
            // requests are spent; try the outboxes, which re-arm what is
            // still full.
            sys::release_received(buf);
            for role in self.lock().ends.values_mut() {
                if let Role::Sender { armed, .. } = role {
                    *armed = false;
                }
            }
            *flush = true;
        } else {
            let mut st = self.lock();
            if st.sinks.remove(&local) {
                drop(st);
                sys::release_received(buf);
                sys::extract_member(local, self.set);
                sys::destroy_receive(local);
                return false;
            }
            if let Some(&id) = st.closers.get(&local) {
                drop(st);
                self.on_closer(id, buf);
                return false;
            }
            let id = st.listening.get(&local).copied();
            drop(st);
            match id {
                Some(id) => {
                    self.on_message(id, buf);
                    self.close_if_drained(id);
                }
                None => sys::release_received(buf),
            }
        }
        false
    }

    /// A message on receiver `id`'s closer port: its data port's
    /// no-senders notification (exact size, the kernel's token), or the
    /// send-once notification a cancelled request leaves (ignored).
    fn on_closer(self: &Arc<Self>, id: u64, buf: &Buf) {
        let kind = wire::layout(buf.bytes())
            .map(|l| classify(&l.header, l.token))
            .ok();
        sys::release_received(buf);
        if kind != Some(Kind::NoSenders) {
            return;
        }
        if let Some(Role::Receiver { peer_gone, .. }) = self.lock().ends.get_mut(&id) {
            *peer_gone = true;
        }
        self.close_if_drained(id);
    }

    /// Reports `Closed` on listening receiver `id` once no sender is left
    /// and its data port's queue is empty: no message can arrive after
    /// that (a send right in flight in another message keeps the
    /// notification back, P20), so `Closed` comes after every message.
    fn close_if_drained(self: &Arc<Self>, id: u64) {
        let mut st = self.lock();
        let data = match st.ends.get(&id) {
            Some(Role::Receiver {
                data,
                peer_gone: true,
                listener: Some(l),
                ..
            }) if !l.ended => *data,
            _ => return,
        };
        let queued = sys::receive_status(data).and_then(|w| w.get(STATUS_MSGCOUNT).copied());
        if queued == Some(0) {
            self.end_listener(&mut st, id, Item::Closed);
        }
    }
}

/// Writes `pairs` (old name, new name) over the names of the port
/// descriptors in an outgoing message.
fn swap_ports(bytes: &mut [u8], pairs: &[(u32, u32)]) {
    let Ok(l) = wire::layout(bytes) else {
        return;
    };
    let mut at = l.descs_at;
    for _ in 0..l.count {
        let Ok((d, next)) = wire::desc_at(bytes, at, l.end) else {
            return;
        };
        if let wire::Desc::Port { name, .. } = d
            && let Some(&(_, new)) = pairs.iter().find(|(old, _)| *old == name)
            && let Some(w) = bytes.get_mut(at..at.saturating_add(4))
        {
            w.copy_from_slice(&new.to_le_bytes());
        }
        at = next;
    }
}

/// Sends a credit record on send-once reply right `reply`. A dead reply
/// right (the sender end is gone) is no error.
fn send_credit(reply: u32, c: Credit) {
    if reply == NAME_DEAD {
        return;
    }
    // Framed like every body, so the reader needs one rule.
    let Some(body) = wire::frame(&encode_credit(c)) else {
        sys::deallocate(reply);
        return;
    };
    let m = WireMsg {
        dest: reply,
        dest_disposition: MOVE_SEND_ONCE,
        reply: NAME_NULL,
        reply_disposition: 0,
        id: ID_CREDIT,
        ports: &[],
        body: &body,
    };
    let mut buf = Buf::new(CREDIT_BUFFER);
    if let Some(size) = wire::encode(&m, buf.bytes_mut())
        && sys::send(&mut buf, size).is_err()
    {
        // Not sent: the right is still ours.
        sys::deallocate(reply);
    }
}

/// Reads every credit queued on credit port `credit` into `flow`. An
/// error means the receiving side broke the credit protocol (or a
/// message arrived on the wrong port): the channel is closed, unblamed.
fn drain_credits(credit: u32, flow: &mut SenderFlow) -> Result<(), ()> {
    let mut buf = Buf::new(CREDIT_BUFFER);
    loop {
        match sys::receive_now(credit, &mut buf) {
            Ok(()) => {}
            Err(Kr(wire::RCV_TIMED_OUT)) => return Ok(()),
            Err(_) => return Err(()),
        }
        let bytes = buf.bytes();
        let Ok(l) = wire::layout(bytes) else {
            return Err(());
        };
        match classify(&l.header, l.token) {
            Kind::Peer(_) => {}
            Kind::NoSenders | Kind::SendPossible | Kind::Ignore => {
                // A reply right destroyed unused (P36), and so on.
                sys::release_received(&buf);
                continue;
            }
            Kind::NoToken => {
                sys::release_received(&buf);
                return Err(());
            }
        }
        if l.header.id != ID_CREDIT || l.header.complex() {
            sys::release_received(&buf);
            return Err(());
        }
        let c = wire::unframe(bytes.get(l.body_at..l.end).unwrap_or(&[])).map(decode_credit);
        sys::release_received(&buf);
        let Some(Ok(c)) = c else {
            return Err(());
        };
        flow.on_credit(c).map_err(|_e| ())?;
    }
}

/// A channel end this transport made or received.
struct MacEnd {
    inner: Arc<Inner>,
    id: u64,
    kind: HandleKind,
    /// False once the end became a listener or was sent.
    armed: bool,
}

impl core::fmt::Debug for MacEnd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MacEnd")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .finish()
    }
}

impl HandleObject for MacEnd {
    fn kind(&self) -> HandleKind {
        self.kind
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

impl Drop for MacEnd {
    fn drop(&mut self) {
        if self.armed {
            self.inner.release(self.id);
        }
    }
}

/// A listener; dropping it closes the receiver.
struct MacListening {
    inner: Arc<Inner>,
    id: u64,
}

impl core::fmt::Debug for MacListening {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MacListening")
            .field("id", &self.id)
            .finish()
    }
}

impl HandleObject for MacListening {
    fn kind(&self) -> HandleKind {
        HandleKind::ChannelReceiver
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

impl Drop for MacListening {
    fn drop(&mut self) {
        let mut st = self.inner.lock();
        let gone = st.take(self.id);
        drop(st);
        let leftovers = gone.map(|r| release_role(r, self.inner.set));
        drop(leftovers);
    }
}

/// The macOS transport of one process.
pub(crate) struct MacIpc {
    inner: Arc<Inner>,
}

impl core::fmt::Debug for MacIpc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MacIpc")
            .field("pid", &self.inner.token.pid)
            .finish()
    }
}

/// Why the transport could not start.
#[derive(Debug)]
pub enum StartError {
    /// A port, the port set or the IO thread.
    Os(std::io::Error),
    /// The launcher's answer did not come, or was not two channel ends.
    Bootstrap,
}

fn os_err(e: Kr) -> StartError {
    StartError::Os(std::io::Error::other(format!("mach error {:#x}", e.0)))
}

impl MacIpc {
    /// Starts the transport and its IO thread. With `launched`, first
    /// takes the launch channels from the launcher over the bootstrap
    /// port (the hello exchange).
    pub(crate) fn start(exec: Arc<OsExecutor>, launched: bool) -> Result<Self, StartError> {
        let token = sys::own_token().ok_or(StartError::Bootstrap)?;
        let boot = if launched {
            Some(hello().ok_or(StartError::Bootstrap)?)
        } else {
            None
        };
        let set = sys::allocate_set().map_err(os_err)?;
        let wake = sys::allocate_receive().map_err(os_err)?;
        sys::set_qlimit(wake, 1).map_err(os_err)?;
        sys::insert_member(wake, set).map_err(os_err)?;
        let notify = sys::allocate_receive().map_err(os_err)?;
        sys::insert_member(notify, set).map_err(os_err)?;
        let inner = Arc::new(Inner {
            exec,
            set,
            wake,
            notify,
            state: Mutex::new(State::default()),
            token,
            boot: Mutex::new(boot),
            #[cfg(test)]
            fail_stand_in: std::sync::atomic::AtomicBool::new(false),
        });
        let io = Arc::clone(&inner);
        crate::thread::spawn("hg-ipc-io", move || io.io_loop()).map_err(StartError::Os)?;
        Ok(Self { inner })
    }

    /// The end `h` names, if it is this transport's and held here.
    fn held_end(&self, h: &Handle) -> Option<u64> {
        let e = h.object().downcast_ref::<MacEnd>()?;
        if !Arc::ptr_eq(&e.inner, &self.inner) {
            return None;
        }
        self.inner.lock().ends.contains_key(&e.id).then_some(e.id)
    }

    fn handle(&self, id: u64, kind: HandleKind) -> Handle {
        Handle::new(Box::new(MacEnd {
            inner: Arc::clone(&self.inner),
            id,
            kind,
            armed: true,
        }))
    }

    /// A channel's ports: the data port, with its queue limit (and,
    /// with `own_send`, a send right beside its receive right), and the
    /// credit port. Its receiver end's closer is armed by whoever keeps
    /// that end ([`arm_closer`]).
    fn new_ports(capacity: u32, own_send: bool) -> Result<(u32, u32), Kr> {
        let data = sys::allocate_receive()?;
        let credit = match sys::allocate_receive() {
            Ok(c) => c,
            Err(e) => {
                sys::destroy_receive(data);
                return Err(e);
            }
        };
        let made = if own_send {
            sys::make_send(data)
        } else {
            Ok(())
        };
        let set_up = made.and_then(|()| sys::set_qlimit(data, capacity));
        if let Err(e) = set_up {
            sys::destroy_receive(data);
            sys::destroy_receive(credit);
            return Err(e);
        }
        Ok((data, credit))
    }

    /// [`Ipc::listen`] and [`Ipc::listen_bound`].
    fn start_listening(
        &self,
        from: ChannelReceiver,
        peer: Option<Token>,
        seq: SequenceHandle,
        sink: Sink,
    ) -> Result<Listening, IpcError> {
        let Some(id) = self.held_end(from.handle()) else {
            return Err(IpcError::Refused);
        };
        if !self.inner.exec.has_sequence(seq) {
            return Err(IpcError::Sequence);
        }
        let mut st = self.inner.lock();
        let Some(Role::Receiver {
            data,
            bound,
            listener,
            ..
        }) = st.ends.get_mut(&id)
        else {
            return Err(IpcError::Refused);
        };
        if listener.is_some() || (*bound && peer.is_none()) {
            return Err(IpcError::Refused);
        }
        let d = *data;
        if sys::insert_member(d, self.inner.set).is_err() {
            return Err(IpcError::Closed);
        }
        *listener = Some(Listener {
            seq,
            sink: Arc::new(Mutex::new(sink)),
            queue: VecDeque::new(),
            draining: false,
            ended: false,
            peer,
            reply: None,
        });
        st.listening.insert(d, id);
        drop(st);
        disarm(from.into_handle());
        // Its sender may already be gone with nothing queued.
        self.inner.close_if_drained(id);
        Ok(Listening::new(Handle::new(Box::new(MacListening {
            inner: Arc::clone(&self.inner),
            id,
        }))))
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
    /// The launcher's side of the hello exchange (phase E's launcher and
    /// the tests): waits on `boot`, the bootstrap port given to a child
    /// started with [`sys::spawn`], for the hello of `expected`
    /// (released unanswered: anything else that arrives first, such as
    /// the child's runtime start-up and sandbox requests, P26, P27),
    /// answers it with the two launch channel ends, and destroys `boot`.
    /// Returns this process's ends: from the child, to the child.
    pub(crate) fn adopt_child(
        &self,
        boot: u32,
        expected: Token,
    ) -> Option<(ChannelReceiver, ChannelSender)> {
        let descs = [
            SenderFlow::new(BOOTSTRAP_CAPACITY).to_desc(WireKind::Sender),
            ReceiverFlow::new(BOOTSTRAP_CAPACITY).to_desc(),
        ];
        let body = encode_data(0, &descs, b"")
            .ok()
            .and_then(|r| wire::frame(&r));
        let deadline =
            StdInstant::now().checked_add(Duration::from_millis(u64::from(LAUNCH_WAIT_MS)));
        let (Some(body), Some(deadline)) = (body, deadline) else {
            sys::destroy_receive(boot);
            return None;
        };
        let mut buf = Buf::new(RECEIVE_BUFFER);
        let reply = loop {
            let left = deadline.saturating_duration_since(StdInstant::now());
            let ms = u32::try_from(left.as_millis()).unwrap_or(0);
            if ms == 0 || sys::receive_within(boot, &mut buf, ms).is_err() {
                sys::destroy_receive(boot);
                return None;
            }
            let Ok(l) = wire::layout(buf.bytes()) else {
                continue;
            };
            let hello = l.header.id == ID_HELLO
                && !l.header.complex()
                && l.token == Ok(expected)
                && l.header.remote_disposition() == wire::PORT_SEND_ONCE;
            if hello {
                // Its reply right is ours to answer on.
                buf.adopted();
                break l.header.remote;
            }
            sys::release_received(&buf);
        };
        let built = self.launch_ends();
        let Some((up_data, up_credit, down_data, ours)) = built else {
            sys::deallocate(reply);
            sys::destroy_receive(boot);
            return None;
        };
        let ports = [
            (up_data, MAKE_SEND),
            (up_credit, MOVE_RECEIVE),
            (down_data, MOVE_RECEIVE),
        ];
        let m = WireMsg {
            dest: reply,
            dest_disposition: MOVE_SEND_ONCE,
            reply: NAME_NULL,
            reply_disposition: 0,
            id: ID_HELLO_REPLY,
            ports: &ports,
            body: &body,
        };
        let mut out = Buf::new(RECEIVE_BUFFER);
        let size = wire::encode(&m, out.bytes_mut());
        let sent = size.map(|n| sys::send(&mut out, n));
        sys::destroy_receive(boot);
        match (size, sent) {
            (_, Some(Ok(()))) => return Some(ours),
            // Not sent: the moved rights are back in this task under the
            // names the kernel wrote into the message (P17, R7).
            (Some(n), _) => sys::release_outgoing(out.bytes().get(..n).unwrap_or(&[])),
            (None, _) => {
                sys::destroy_receive(up_credit);
                sys::destroy_receive(down_data);
            }
        }
        sys::deallocate(reply);
        drop(ours);
        None
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
    /// The ports of a child's two launch channels and this process's
    /// ends of them: (up data, up credit, down data, ours).
    fn launch_ends(&self) -> Option<(u32, u32, u32, (ChannelReceiver, ChannelSender))> {
        // Our receiver of `up` holds the data port's receive right; the
        // child's sender end gets a send right made from it as the answer
        // is sent. Our sender of `down` keeps its send right and credit
        // port; the child gets the data port's receive right.
        let (up_data, up_credit) = Self::new_ports(BOOTSTRAP_CAPACITY, false).ok()?;
        let Ok((down_data, down_credit)) = Self::new_ports(BOOTSTRAP_CAPACITY, true) else {
            sys::destroy_receive(up_data);
            sys::destroy_receive(up_credit);
            return None;
        };
        let mut st = self.inner.lock();
        let rx = st.insert(Role::Receiver {
            data: up_data,
            flow: ReceiverFlow::new(BOOTSTRAP_CAPACITY),
            bound: false,
            listener: None,
            // Fires once the child's send right, made as the answer is
            // sent, is gone.
            closer: arm_closer(up_data, 1, self.inner.set),
            peer_gone: false,
        });
        let tx = st.insert(Role::Sender {
            data: down_data,
            credit: down_credit,
            flow: SenderFlow::new(BOOTSTRAP_CAPACITY),
            bound: false,
            made_here: true,
            granted: false,
            outbox: VecDeque::new(),
            dropped: false,
            broken: false,
            armed: false,
            carried: false,
        });
        drop(st);
        // From here the two ends release their own rights when dropped;
        // the child's two receive rights are the caller's.
        let rx = ChannelReceiver::from_handle(self.handle(rx, HandleKind::ChannelReceiver));
        let tx = ChannelSender::from_handle(self.handle(tx, HandleKind::ChannelSender));
        let (Ok(rx), Ok(tx)) = (rx, tx) else {
            sys::destroy_receive(up_credit);
            sys::destroy_receive(down_data);
            return None;
        };
        Some((up_data, up_credit, down_data, (rx, tx)))
    }
}

/// The launched process's side of the hello exchange: sends a hello on
/// its bootstrap port and takes the launch channel ends from the answer.
fn hello() -> Option<Boot> {
    let boot = sys::bootstrap_port()?;
    let reply = sys::allocate_receive().ok()?;
    let m = WireMsg {
        dest: boot,
        dest_disposition: COPY_SEND,
        reply,
        reply_disposition: MAKE_SEND_ONCE,
        id: ID_HELLO,
        ports: &[],
        body: &[],
    };
    let mut buf = Buf::new(RECEIVE_BUFFER);
    let size = wire::encode(&m, buf.bytes_mut())?;
    if sys::send(&mut buf, size).is_err() {
        sys::destroy_receive(reply);
        return None;
    }
    let got = sys::receive_within(reply, &mut buf, LAUNCH_WAIT_MS);
    sys::destroy_receive(reply);
    got.ok()?;
    let bytes = buf.bytes();
    let l = wire::layout(bytes).ok()?;
    let facts: Vec<SlotFacts> = wire::descs(bytes, &l)
        .take(MAX_SLOTS.saturating_add(1))
        .map(sys::slot_facts)
        .collect();
    let body = wire::unframe(bytes.get(l.body_at..l.end).unwrap_or(&[])).unwrap_or(&[]);
    let arrival = Arrival {
        header: l.header,
        walk: l.walk,
        token: l.token,
        count: l.count,
        slots: &facts,
        body,
    };
    let empty = State::default();
    let Ok((parent, view)) = decide_launch(&arrival, &empty) else {
        sys::release_received(&buf);
        return None;
    };
    // `decide_launch` checked this shape; anything else is released.
    let shape = match (view.descs(), facts.as_slice()) {
        ([up_d, down_d], [a, b, c]) => SenderFlow::from_transfer(up_d)
            .ok()
            .zip(ReceiverFlow::from_transfer(down_d).ok())
            .map(|(up, down)| (up, down, a.name, b.name, c.name)),
        _ => None,
    };
    let Some((up_flow, down_flow, up_data, up_credit, down_data)) = shape else {
        sys::release_received(&buf);
        return None;
    };
    // Its three rights are the launch channels now.
    buf.adopted();
    Some(Boot {
        up_data,
        up_credit,
        up_flow,
        down_data,
        down_flow,
        parent,
    })
}

impl Drop for MacIpc {
    fn drop(&mut self) {
        self.inner.lock().stopping = true;
        self.inner.wake_io();
    }
}

/// Disarms a handle object that became something else (sent, listened).
fn disarm(h: Handle) {
    if let Ok(mut e) = h.into_object().downcast::<MacEnd>() {
        e.armed = false;
    }
}

impl Ipc for MacIpc {
    fn channel(
        &self,
        capacity: u32,
        transfer: Transfer,
    ) -> Result<(ChannelSender, ChannelReceiver), IpcError> {
        let (data, credit) = Self::new_ports(capacity, true).map_err(|_e| IpcError::Closed)?;
        let bound = transfer == Transfer::Bound;
        let mut st = self.inner.lock();
        let tx = st.insert(Role::Sender {
            data,
            credit,
            flow: SenderFlow::new(capacity),
            bound,
            made_here: true,
            granted: false,
            outbox: VecDeque::new(),
            dropped: false,
            broken: false,
            armed: false,
            carried: false,
        });
        let rx = st.insert(Role::Receiver {
            data,
            flow: ReceiverFlow::new(capacity),
            bound,
            listener: None,
            closer: arm_closer(data, 1, self.inner.set),
            peer_gone: false,
        });
        drop(st);
        let tk = if bound {
            HandleKind::BoundChannelSender
        } else {
            HandleKind::ChannelSender
        };
        match (
            ChannelSender::from_handle(self.handle(tx, tk)),
            ChannelReceiver::from_handle(self.handle(rx, HandleKind::ChannelReceiver)),
        ) {
            (Ok(t), Ok(r)) => Ok((t, r)),
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
        let Some(to_id) = self.held_end(to.handle()) else {
            return Err(IpcError::Refused);
        };
        let mut ids = Vec::with_capacity(handles.len());
        for h in &handles {
            let Some(id) = self.held_end(h) else {
                return Err(IpcError::Refused);
            };
            if id == to_id || ids.contains(&id) {
                return Err(IpcError::Refused);
            }
            ids.push(id);
        }
        let mut st = self.inner.lock();
        // Credits first, so Full is exact.
        match st.ends.get_mut(&to_id) {
            Some(Role::Sender {
                credit,
                flow,
                broken,
                ..
            }) => {
                if *broken {
                    return Err(IpcError::Closed);
                }
                if drain_credits(*credit, flow).is_err() {
                    *broken = true;
                    return Err(IpcError::Closed);
                }
            }
            _ => return Err(IpcError::Refused),
        }
        // Every carried handle: a bound receiver, or one already
        // listening, never moves; a bound sender moves once, from the
        // process that made it (IPC-R2 (a)).
        let mut descs = Vec::with_capacity(ids.len());
        let mut ports: Vec<(u32, u8)> = Vec::with_capacity(ids.len().saturating_mul(2));
        let mut after = Vec::new();
        for (&id, h) in ids.iter().zip(&handles) {
            let d = match st.ends.get_mut(&id) {
                Some(
                    Role::Receiver { bound: true, .. }
                    | Role::Receiver {
                        listener: Some(_), ..
                    },
                ) => return Err(IpcError::Refused),
                Some(Role::Receiver { data, flow, .. }) => {
                    ports.push((*data, MOVE_RECEIVE));
                    flow.to_desc()
                }
                Some(Role::Sender {
                    data,
                    credit,
                    bound,
                    made_here,
                    granted,
                    flow,
                    outbox,
                    broken,
                    ..
                }) => {
                    if *bound && (!*made_here || *granted) {
                        return Err(IpcError::Refused);
                    }
                    if *broken {
                        return Err(IpcError::Closed);
                    }
                    // Its credits so far, so the counters it carries are
                    // current; later ones travel queued on its credit port.
                    if drain_credits(*credit, flow).is_err() {
                        return Err(IpcError::Closed);
                    }
                    if !outbox.is_empty() {
                        after.push(id);
                    }
                    ports.push((*data, MOVE_SEND));
                    ports.push((*credit, MOVE_RECEIVE));
                    let kind = if h.kind() == HandleKind::BoundChannelSender {
                        WireKind::BoundSender
                    } else {
                        WireKind::Sender
                    };
                    flow.to_desc(kind)
                }
                None => return Err(IpcError::Refused),
            };
            descs.push(d);
        }
        let Some(Role::Sender {
            data,
            credit,
            flow,
            outbox,
            ..
        }) = st.ends.get_mut(&to_id)
        else {
            return Err(IpcError::Refused);
        };
        if flow.would_be_full() {
            return Err(IpcError::Full);
        }
        let mut next = *flow;
        let Ok(seq) = next.on_send() else {
            return Err(IpcError::Full);
        };
        let Some(record) = encode_data(seq, &descs, &bytes)
            .ok()
            .and_then(|r| wire::frame(&r))
        else {
            return Err(IpcError::TooLarge);
        };
        let m = WireMsg {
            dest: *data,
            dest_disposition: COPY_SEND,
            reply: *credit,
            reply_disposition: MAKE_SEND_ONCE,
            id: ID_DATA,
            ports: &ports,
            body: &record,
        };
        let mut buf = Buf::new(wire::MAX_DATA_MESSAGE);
        let Some(size) = wire::encode(&m, buf.bytes_mut()) else {
            return Err(IpcError::TooLarge);
        };
        let queue_first = !outbox.is_empty() || !after.is_empty();
        // Before any right moves, this process's own requests come off
        // the carried ports (R8): a receiver's no-senders request is a
        // send-once right to its port, which the receiving side refuses.
        for &id in &ids {
            unrequest(&mut st, id, self.inner.set);
        }
        let outcome = if queue_first {
            Err(Kr(SEND_TIMED_OUT))
        } else {
            sys::send(&mut buf, size)
        };
        let queued = match outcome {
            Ok(()) => false,
            Err(Kr(SEND_TIMED_OUT)) => true,
            Err(e) => {
                // Not sent: the carried rights are still ours (R7).
                for &id in &ids {
                    rerequest(&mut st, id, self.inner.set);
                }
                if e == Kr(SEND_INVALID_DEST)
                    && let Some(Role::Sender { broken, .. }) = st.ends.get_mut(&to_id)
                {
                    *broken = true;
                }
                return Err(IpcError::Closed);
            }
        };
        // Committed: count it, mark grants, retire the carried ends (their
        // rights left in the message, or wait in it in the outbox).
        if let Some(Role::Sender { flow, outbox, .. }) = st.ends.get_mut(&to_id) {
            *flow = next;
            if queued {
                outbox.push_back(Outgoing {
                    buf,
                    size,
                    after: after.clone(),
                    since: StdInstant::now(),
                });
            }
        }
        for &id in &ids {
            if after.contains(&id) {
                // Its own outbox still drains first; it leaves the table
                // once that is done (its rights then go with the waiting
                // message).
                if let Some(Role::Sender {
                    dropped,
                    granted,
                    carried,
                    ..
                }) = st.ends.get_mut(&id)
                {
                    *dropped = true;
                    *granted = true;
                    *carried = true;
                }
            } else {
                // Its rights are in the message: forget the end without
                // releasing them.
                let _ = st.take(id);
            }
        }
        drop(st);
        for h in handles {
            disarm(h);
        }
        if queued {
            self.inner.wake_io();
        }
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
        let (Ok(version), true) = (u32::try_from(peer.generation()), peer.pid() > 0) else {
            return Err(IpcError::Refused);
        };
        let token = Token {
            pid: peer.pid(),
            version,
        };
        self.start_listening(from, Some(token), seq, sink)
    }

    fn bootstrap(&self) -> Option<Bootstrap> {
        let b = self
            .inner
            .boot
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()?;
        let mut st = self.inner.lock();
        let tx = st.insert(Role::Sender {
            data: b.up_data,
            credit: b.up_credit,
            flow: b.up_flow,
            bound: false,
            made_here: false,
            granted: false,
            outbox: VecDeque::new(),
            dropped: false,
            broken: false,
            armed: false,
            carried: false,
        });
        let rx = st.insert(Role::Receiver {
            data: b.down_data,
            flow: b.down_flow,
            bound: false,
            listener: None,
            // A moved receive right's make-send count restarts at 0 (P10):
            // sync 0 fires at once if the launcher's sender is gone.
            closer: arm_closer(b.down_data, 0, self.inner.set),
            peer_gone: false,
        });
        drop(st);
        let to_parent =
            ChannelSender::from_handle(self.handle(tx, HandleKind::ChannelSender)).ok()?;
        let from_parent =
            ChannelReceiver::from_handle(self.handle(rx, HandleKind::ChannelReceiver)).ok()?;
        Some(Bootstrap {
            to_parent,
            from_parent,
            parent: OsPeer::new(b.parent.pid, u64::from(b.parent.version)),
        })
    }
}

#[cfg(test)]
mod tests;
