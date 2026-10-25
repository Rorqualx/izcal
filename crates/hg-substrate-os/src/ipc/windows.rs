//! The Windows IPC transport (Stage 1 4.2; M1 phase D5): channels are
//! one-instance, duplex, message-mode named pipes; handles move only
//! through the broker. Written from Microsoft Learn's reference pages
//! and the D5 probes (`research_notes/M1 D5 Windows IPC/`); no other
//! project's code is a source.
//!
//! **Shape.** As on Linux: a channel's data flows from the sender end
//! (the pipe's client end) to the receiver end (its server end), and the
//! reverse direction carries only credit records, so `Full` is exact at
//! the channel's capacity and is never inferred from the pipe's buffer.
//! A message that cannot be written yet waits in the sender end's
//! outbox; `send` never blocks. One IO thread per process starts every
//! read and write (`ReadFileEx`/`WriteFileEx`) and runs their completion
//! routines in an alertable wait with no timeout (no idle wakeups,
//! FP-18); each listening channel keeps at most one delivery task on
//! its sequence.
//!
//! **Handles move only through the broker** (IPC-R1; design review B1).
//! A sandboxed process cannot duplicate a handle into another process,
//! so a record that carries handles goes in three steps:
//! 1. the sender's transport waits until the carried ends are quiet (no
//!    read or write in flight: a read left pending on a moved end would
//!    steal the next message, probe W3), then asks its broker over the
//!    control link to *relay* them, naming each by its value;
//! 2. the broker pulls each into its own table, checks it against the
//!    kernel's facts and its ledger (`ipc::win::accept`,
//!    `ipc::win::ledger`) and answers with a random 128-bit ticket; the
//!    sender writes the record with that ticket in place of handles;
//! 3. the receiver's transport reads the record and *redeems* the
//!    ticket over its own control link; the broker checks that the
//!    redeemer holds the receiving end of the channel the ticket was
//!    issued for, and duplicates the handles in with a narrowed access
//!    mask ([`sys::MOVE_ACCESS`]).
//!
//! Handle values never travel in a channel. The broker (the process with
//! no control link: the browser, or a test) does the same steps locally
//! for its own sends and receives.
//!
//! **What Windows attests** (security scope IPC-R1, R2, R5; SEC-4):
//! - every handle that moves is a one-instance message pipe end of an
//!   Izcal channel, of the end its claim says, of which no other handle
//!   exists anywhere once the broker holds it (`HandleCount`, probe
//!   W1); pseudo-handles, values with the tag bits set, and every other
//!   object are refused, and so is the whole relay;
//! - who holds each channel end: the broker sees every move and refuses
//!   any that leaves a copy, so its ledger names the one process that
//!   can write to a channel. A channel's creator is attested by the
//!   pipe's name: a child can make pipes only in its own AppContainer's
//!   namespace (probes P1, P2), and each child has its own container;
//! - a bound sender moves once, from its creator; a receiver listened
//!   to with `listen_bound` is reported broken, as an Identity fault
//!   naming the bound peer, as soon as its channel's sender end comes to
//!   rest with any other process.
//!
//! **What it does not attest:** a pipe names no sender per message
//! (probe P13), so a record cannot be traced to a process by the
//! receiver; and a former holder of a moved sender end may have queued
//! records before the move. Every violation a receiver detects in a
//! record is therefore reported unattributed (`Fault::sender` `None`;
//! design review B3). `OsPeer` generations are 0: Windows process IDs
//! carry none.
//!
//! **`Closed` is eventual**, as on Linux: a write accepted while any
//! handle to the reading end exists (one in flight in a ticket, or a
//! write already pending when the reader closed, probe P10) can be
//! lost, and only a later send reports `Closed`.
//!
//! **Lock discipline.** One mutex guards the transport's state. Nothing
//! that may own a handle object is dropped with it held, and no sink
//! runs with it held. Completion routines run only in the IO thread's
//! alertable wait, never with the lock held.

pub(crate) mod sys;

#[cfg(test)]
mod tests;

use core::any::Any;
use std::collections::{BTreeMap, VecDeque};
use std::os::windows::io::OwnedHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::{Duration, Instant as StdInstant};

use hg_substrate::ipc::{
    Bootstrap, ChannelReceiver, ChannelSender, Delivery, Fault, FaultKind, Flow, Handle,
    HandleKind, HandleObject, Ipc, IpcError, Listening, MAX_MESSAGE_BYTES, MAX_MESSAGE_HANDLES,
    OsPeer, Sink, Transfer,
};
use hg_substrate::{SequenceHandle, Task};

use crate::executor::OsExecutor;
use crate::ipc::common::codec::{
    CREDIT_LEN, HandleDesc, WireKind, decode_credit, encode_credit, encode_data,
};
use crate::ipc::common::flow::{PendingCredit, ReceiverFlow, SenderFlow};
use crate::ipc::win::accept::{
    End as PipeEnd, Facts, PipeKey, Proc, Refusal, accept, check_value, distinct, parse_name,
};
use crate::ipc::win::decide::{Decision, Reason, check_redeemed, decide};
use crate::ipc::win::frame::{
    Ctl, Item as CtlItem, Items, MAX_CTL_LEN, MAX_PIPE_DATA, NO_TICKET, Ticket, decode_ctl,
    encode_ctl, join_data,
};
use crate::ipc::win::ledger::{BindEvent, Held, Ledger};

/// The capacity of a process's launch channels (Stage 1 4.1): the
/// process-host interfaces' declared capacity.
pub const BOOTSTRAP_CAPACITY: u32 = 64;

/// How long a message carrying a sender end waits for that end's own
/// queued messages before the transport gives up on them.
const TRANSFER_DEADLINE: Duration = Duration::from_secs(5);

/// How long a child waits for its broker's `Hello`.
const HELLO_TIMEOUT_MS: u32 = 10_000;

/// The read buffer of a sender end: credits are [`CREDIT_LEN`] bytes, and
/// anything longer is a broken peer (`ERROR_MORE_DATA`).
const CREDIT_READ: usize = CREDIT_LEN;

/// The broker's own process number in its ledger.
const BROKER: Proc = 0;

/// One item for a listener.
enum Item {
    Message(Vec<u8>, Vec<Handle>),
    /// A message whose handles are being redeemed; nothing behind it is
    /// delivered first.
    Pending {
        req: u64,
        payload: Vec<u8>,
        descs: Vec<HandleDesc>,
    },
    Closed,
    Violation(Fault),
}

/// A message waiting in a sender end's outbox.
enum Out {
    /// A whole pipe message.
    Ready(Vec<u8>),
    /// A record that carries handles, on its way through the broker.
    Carry(Carry),
}

struct Carry {
    seq: u64,
    payload: Vec<u8>,
    /// The carried ends (still in this process's table until relayed)
    /// and the kinds they travel as.
    ends: Vec<(u64, WireKind)>,
    since: StdInstant,
    /// The encoded data record, once the carried ends are quiet.
    record: Option<Vec<u8>>,
    /// The relay request awaiting the broker's ticket.
    relaying: Option<u64>,
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
    credit: PendingCredit,
}

/// What an end is.
enum Role {
    Sender {
        flow: SenderFlow,
        bound: bool,
        made_here: bool,
        granted: bool,
        outbox: VecDeque<Out>,
        dropped: bool,
        /// Inside a record on its way to another process.
        in_transit: bool,
        broken: bool,
    },
    Receiver {
        flow: ReceiverFlow,
        bound: bool,
        listener: Option<Listener>,
        in_transit: bool,
    },
    /// A control link: in a child, to its broker; in the broker, to
    /// child `proc`.
    Link { proc: Proc, out: VecDeque<Vec<u8>> },
}

struct End {
    h: Arc<OwnedHandle>,
    role: Role,
    /// A read is in flight.
    reading: bool,
    /// A write is in flight.
    writing: bool,
}

/// A child, as its broker knows it.
struct ChildEntry {
    process: sys::Process,
    /// Its AppContainer's SID, which names the namespace its pipes are
    /// created in.
    sid: Option<String>,
    link: u64,
}

/// The broker's state: the ledger, the handles held under tickets, and
/// its children.
struct Broker {
    ledger: Ledger,
    held: BTreeMap<Ticket, Vec<OwnedHandle>>,
    children: BTreeMap<Proc, ChildEntry>,
    #[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher reads it"))]
    next_proc: Proc,
}

#[derive(Default)]
struct State {
    ends: BTreeMap<u64, End>,
    next: u64,
    next_req: u64,
    stopping: bool,
    /// Relay requests: request to carrier end.
    relays: BTreeMap<u64, u64>,
    /// Redemptions: request to receiver end.
    redeems: BTreeMap<u64, u64>,
    broker: Option<Broker>,
    /// This process's link to its broker (a child).
    link: Option<u64>,
    boot: Option<(OwnedHandle, OwnedHandle, u32)>,
}

impl State {
    fn req(&mut self) -> u64 {
        self.next_req = self.next_req.wrapping_add(1);
        self.next_req
    }
}

/// The transport's shared core.
pub(crate) struct Inner {
    exec: Arc<OsExecutor>,
    state: Mutex<State>,
    /// The IO thread, once it runs.
    thread: OnceLock<sys::ThreadRef>,
    wake_pending: AtomicBool,
    /// This process's AppContainer SID, if it runs in one.
    container: Option<String>,
}

impl Inner {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Wakes the IO thread (coalesced).
    fn wake(&self) {
        if !self.wake_pending.swap(true, Ordering::AcqRel)
            && let Some(t) = self.thread.get()
        {
            let _ = sys::wake(t);
        }
    }

    fn insert(st: &mut State, h: OwnedHandle, role: Role) -> Option<u64> {
        st.next = st.next.checked_add(1)?;
        let id = st.next;
        st.ends.insert(
            id,
            End {
                h: Arc::new(h),
                role,
                reading: false,
                writing: false,
            },
        );
        Some(id)
    }

    /// Removes an end; its I/O is cancelled and its handle closes when
    /// the last operation completes. Returns what to drop after the lock
    /// is released.
    fn remove(st: &mut State, id: u64) -> Option<End> {
        let end = st.ends.remove(&id)?;
        sys::cancel(&end.h);
        Some(end)
    }

    /// A new random id for a pipe name or a ticket.
    fn random16() -> Option<[u8; 16]> {
        let mut b = [0u8; 16];
        crate::os::random::fill(&mut b).ok()?;
        Some(b)
    }

    /// Posts `id`'s delivery task if its next item is deliverable.
    fn kick(self: &Arc<Self>, st: &mut State, id: u64) {
        let Some(End {
            role: Role::Receiver {
                listener: Some(l), ..
            },
            ..
        }) = st.ends.get_mut(&id)
        else {
            return;
        };
        if l.draining
            || l.queue
                .front()
                .is_none_or(|i| matches!(i, Item::Pending { .. }))
        {
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
        let Some(End {
            role:
                Role::Receiver {
                    flow,
                    listener: Some(l),
                    ..
                },
            ..
        }) = st.ends.get_mut(&id)
        else {
            return;
        };
        l.draining = false;
        if l.queue
            .front()
            .is_some_and(|i| matches!(i, Item::Pending { .. }))
        {
            return;
        }
        let Some(item) = l.queue.pop_front() else {
            return;
        };
        let sink = Arc::clone(&l.sink);
        let mut final_item = false;
        let delivery = match item {
            Item::Message(bytes, handles) => {
                // The credit goes back before the sink runs (as on Linux).
                if let Ok(c) = flow.on_deliver() {
                    l.credit.note(c);
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
            Item::Pending { .. } => return,
        };
        drop(st);
        self.wake();
        let flow_after = sink.lock().unwrap_or_else(PoisonError::into_inner)(delivery);
        let mut st = self.lock();
        let gone = if flow_after == Flow::Close || final_item {
            Self::remove(&mut st, id)
        } else {
            self.kick(&mut st, id);
            None
        };
        drop(st);
        drop(gone);
        self.wake();
    }

    /// Releases an end whose handle object was dropped.
    fn release(&self, id: u64) {
        let mut st = self.lock();
        let keep = match st.ends.get_mut(&id).map(|e| &mut e.role) {
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
        let gone = if keep {
            None
        } else {
            Self::remove(&mut st, id)
        };
        drop(st);
        drop(gone);
        self.wake();
    }

    /// Stops reading `id` and queues its last item.
    fn end_listener(self: &Arc<Self>, st: &mut State, id: u64, last: Item) {
        if let Some(End {
            h,
            role: Role::Receiver {
                listener: Some(l), ..
            },
            ..
        }) = st.ends.get_mut(&id)
            && !l.ended
        {
            l.ended = true;
            sys::cancel(h);
            // Pending redemptions will never complete into a message.
            l.queue.retain(|i| !matches!(i, Item::Pending { .. }));
            l.queue.push_back(last);
        }
        self.kick(st, id);
    }

    /// The pipe key of a handle, as this process's broker sees it.
    fn key_of(b: &Broker, f: &sys::RawFacts) -> Option<PipeKey> {
        let (sid, id) = parse_name(f.name.as_deref()?)?;
        let creator = match sid {
            None => BROKER,
            Some(s) => {
                *b.children
                    .iter()
                    .find(|(_, c)| c.sid.as_deref() == Some(s))?
                    .0
            }
        };
        Some(PipeKey { creator, id })
    }

    fn facts_for(raw: u64, pulled: Option<&OwnedHandle>, b: &Broker) -> Facts {
        let Some(h) = pulled else {
            return Facts {
                raw,
                pulled: false,
                is_pipe: false,
                server_end: false,
                message: false,
                max_instances: 0,
                pipe: None,
                handle_count: 0,
            };
        };
        let f = sys::facts(h);
        Facts {
            raw,
            pulled: true,
            is_pipe: f.is_pipe,
            server_end: f.server_end,
            message: f.message,
            max_instances: f.max_instances,
            pipe: Self::key_of(b, &f),
            handle_count: f.handle_count,
        }
    }

    // ---- the broker's side of the control protocol --------------------

    /// A relay: takes `items` (values in process `from`, or this
    /// process's own handles when `from` is the broker) into a ticket
    /// for a record on `carrier`.
    fn broker_relay(
        b: &mut Broker,
        from: Proc,
        carrier: PipeKey,
        items: Vec<(u64, WireKind, Option<OwnedHandle>)>,
    ) -> Result<Ticket, Refusal> {
        let mut pulled = Vec::with_capacity(items.len());
        let mut first_err = None;
        // Every value is pulled, even after one fails, so a refused relay
        // leaves nothing behind in the child (design review, minor).
        for (raw, kind, own) in items {
            let h = match own {
                Some(h) => Some(h),
                None => match check_value(raw) {
                    Err(r) => {
                        first_err.get_or_insert(r);
                        None
                    }
                    Ok(()) => b
                        .children
                        .get(&from)
                        .and_then(|c| sys::pull(&c.process, raw, true).ok()),
                },
            };
            pulled.push((raw, kind, h));
        }
        let mut ends = Vec::with_capacity(pulled.len());
        let mut handles = Vec::with_capacity(pulled.len());
        for (raw, kind, h) in pulled {
            let facts = Self::facts_for(raw, h.as_ref(), b);
            let view = facts
                .pipe
                .and_then(|p| b.ledger.view(p, crate::ipc::win::accept::end_for(kind)));
            let ctx = crate::ipc::win::accept::Ctx {
                relayer: from,
                carrier,
                view,
            };
            match accept(kind, &facts, &ctx) {
                Ok((pipe, end)) => ends.push((pipe, end, kind)),
                Err(r) => {
                    first_err.get_or_insert(r);
                }
            }
            if let Some(h) = h {
                handles.push(h);
            }
        }
        if let Some(r) = first_err {
            return Err(r);
        }
        let keys: Vec<(PipeKey, PipeEnd)> = ends.iter().map(|(p, e, _)| (*p, *e)).collect();
        distinct(&keys)?;
        // A free ticket: 128 random bits all but never collide, so this
        // loops at most a few times; without randomness, no relay.
        let ticket = loop {
            let t = Self::random16().ok_or(Refusal::NoTicket)?;
            if t != NO_TICKET && b.ledger.ticket_free(&t) {
                break t;
            }
        };
        b.ledger
            .relay(
                ticket,
                Held {
                    carrier,
                    from,
                    ends,
                },
            )
            .map_err(|_e| Refusal::Quota)?;
        b.held.insert(ticket, handles);
        Ok(ticket)
    }

    /// The key of a pipe end a child names by value, without taking it.
    fn peek_key(b: &Broker, from: Proc, raw: u64, server_end: bool) -> Option<PipeKey> {
        check_value(raw).ok()?;
        let c = b.children.get(&from)?;
        let h = sys::pull(&c.process, raw, false).ok()?;
        let f = sys::facts(&h);
        (f.is_pipe && f.server_end == server_end)
            .then(|| Self::key_of(b, &f))
            .flatten()
    }

    /// Handles one control record from child `from`.
    fn broker_ctl(
        self: &Arc<Self>,
        st: &mut State,
        from: Proc,
        c: Ctl,
    ) -> Vec<Box<dyn Any + Send>> {
        let Some(b) = st.broker.as_mut() else {
            return Vec::new();
        };
        let mut events = Vec::new();
        let reply = match c {
            Ctl::Relay {
                req,
                carrier,
                items,
            } => {
                let carrier_key = Self::peek_key(b, from, carrier, false);
                let items: Vec<(u64, WireKind, Option<OwnedHandle>)> = items
                    .as_slice()
                    .iter()
                    .map(|i| (i.value, i.kind, None))
                    .collect();
                let r = match carrier_key {
                    Some(k) => Self::broker_relay(b, from, k, items),
                    None => {
                        // Pull and close the items anyway: a refused relay
                        // leaves nothing in the child.
                        let _ = Self::broker_relay(
                            b,
                            from,
                            PipeKey {
                                creator: u32::MAX,
                                id: [0xff; 16],
                            },
                            items,
                        );
                        Err(Refusal::Carrier)
                    }
                };
                Some(match r {
                    Ok(ticket) => Ctl::RelayOk { req, ticket },
                    Err(r) => Ctl::RelayRefused {
                        req,
                        reason: r.code(),
                    },
                })
            }
            Ctl::Redeem { req, ticket, rx } => {
                let r = Self::peek_key(b, from, rx, true)
                    .ok_or(crate::ipc::win::ledger::LedgerError::NotHolder)
                    .and_then(|k| b.ledger.redeem(from, &ticket, k));
                Some(match r {
                    Ok((held, ev)) => {
                        events = ev;
                        let handles = b.held.remove(&ticket).unwrap_or_default();
                        let mut items = Vec::with_capacity(handles.len());
                        let mut ok = handles.len() == held.ends.len();
                        if let Some(c) = b.children.get(&from) {
                            for (h, (_, _, kind)) in handles.iter().zip(&held.ends) {
                                match sys::give(&c.process, h) {
                                    Ok(v) => items.push(CtlItem {
                                        value: v,
                                        kind: *kind,
                                    }),
                                    Err(_) => ok = false,
                                }
                            }
                        } else {
                            ok = false;
                        }
                        drop(handles);
                        match Items::from_slice(&items) {
                            Some(items) if ok => Ctl::Redeemed { req, items },
                            _ => Ctl::RedeemRefused { req, reason: 0 },
                        }
                    }
                    Err(e) => Ctl::RedeemRefused {
                        req,
                        reason: e.code(),
                    },
                })
            }
            Ctl::Bind { rx, peer } => {
                if let Some(k) = Self::peek_key(b, from, rx, true)
                    && let Ok(Some(ev)) = b.ledger.bind(from, k, rx, peer)
                {
                    events.push(ev);
                }
                None
            }
            // Records only a broker sends: a child that sends one has
            // broken the protocol; its link is closed.
            Ctl::Hello { .. }
            | Ctl::RelayOk { .. }
            | Ctl::RelayRefused { .. }
            | Ctl::Redeemed { .. }
            | Ctl::RedeemRefused { .. }
            | Ctl::BindBroken { .. } => {
                return self.child_gone(st, from);
            }
        };
        if let Some(r) = reply {
            self.send_ctl(st, Some(from), &r);
        }
        self.dispatch(st, events);
        Vec::new()
    }

    /// Tells binders their receivers broke.
    fn dispatch(self: &Arc<Self>, st: &mut State, events: Vec<BindEvent>) {
        for ev in events {
            if ev.binder == BROKER {
                self.bind_broken(st, ev.rx, ev.peer_pid);
            } else {
                self.send_ctl(
                    st,
                    Some(ev.binder),
                    &Ctl::BindBroken {
                        rx: ev.rx,
                        peer: ev.peer_pid,
                    },
                );
            }
        }
    }

    /// Ends the listener on the receiver whose handle value is `rx`,
    /// with an Identity fault naming `peer`.
    fn bind_broken(self: &Arc<Self>, st: &mut State, rx: u64, peer: u32) {
        let id = st
            .ends
            .iter()
            .find(|(_, e)| {
                matches!(
                    e.role,
                    Role::Receiver {
                        listener: Some(_),
                        ..
                    }
                ) && sys::value(&e.h) == rx
            })
            .map(|(id, _)| *id);
        if let Some(id) = id {
            let fault = Fault {
                kind: FaultKind::Identity,
                sender: Some(peer),
            };
            self.end_listener(st, id, Item::Violation(fault));
        }
    }

    /// Queues a control record: to child `to` (broker), or to the broker
    /// (`None`, a child).
    fn send_ctl(&self, st: &mut State, to: Option<Proc>, c: &Ctl) {
        let link = match to {
            Some(p) => st
                .broker
                .as_ref()
                .and_then(|b| b.children.get(&p))
                .map(|c| c.link),
            None => st.link,
        };
        if let Some(Some(End {
            role: Role::Link { out, .. },
            ..
        })) = link.map(|l| st.ends.get_mut(&l))
        {
            out.push_back(encode_ctl(c));
        }
    }

    /// Child `p` ended or broke its link: its ends and tickets are gone.
    fn child_gone(&self, st: &mut State, p: Proc) -> Vec<Box<dyn Any + Send>> {
        let mut out: Vec<Box<dyn Any + Send>> = Vec::new();
        let Some(b) = st.broker.as_mut() else {
            return out;
        };
        let Some(c) = b.children.remove(&p) else {
            return out;
        };
        for t in b.ledger.gone(p) {
            if let Some(hs) = b.held.remove(&t) {
                out.push(Box::new(hs));
            }
        }
        b.ledger.sweep();
        out.push(Box::new(c.process));
        if let Some(e) = Self::remove(st, c.link) {
            out.push(Box::new(e.h));
        }
        out
    }

    /// Undoes a launch that failed after the child was recorded: forgets
    /// the child as if it had ended, and removes its launch ends. What it
    /// returns is dropped outside the lock.
    fn abandon(&self, st: &mut State, p: Proc, ends: [Option<u64>; 2]) -> Vec<Box<dyn Any + Send>> {
        let mut out = self.child_gone(st, p);
        for id in ends.into_iter().flatten() {
            if let Some(e) = Self::remove(st, id) {
                out.push(Box::new(e.h));
            }
        }
        out
    }

    // ---- a child's side ------------------------------------------------

    /// Handles one control record from the broker.
    fn child_ctl(self: &Arc<Self>, st: &mut State, c: Ctl) -> Vec<Box<dyn Any + Send>> {
        let mut out: Vec<Box<dyn Any + Send>> = Vec::new();
        match c {
            Ctl::RelayOk { req, ticket } => {
                if let Some(carrier) = st.relays.remove(&req) {
                    Self::relay_done(st, carrier, req, Some(ticket));
                }
            }
            Ctl::RelayRefused { req, .. } => {
                if let Some(carrier) = st.relays.remove(&req) {
                    Self::relay_done(st, carrier, req, None);
                }
            }
            Ctl::Redeemed { req, items } => {
                if let Some(rx) = st.redeems.remove(&req) {
                    // A value naming an end this process already holds is
                    // refused, never adopted twice.
                    let held: Vec<u64> = st.ends.values().map(|e| sys::value(&e.h)).collect();
                    let got: Vec<(Option<OwnedHandle>, WireKind)> = items
                        .as_slice()
                        .iter()
                        .map(|i| {
                            let h = (!held.contains(&i.value))
                                .then(|| sys::adopt_pipe(i.value, i.kind == WireKind::Receiver))
                                .flatten();
                            (h, i.kind)
                        })
                        .collect();
                    out.extend(self.redeemed(st, rx, req, Some(got)));
                }
            }
            Ctl::RedeemRefused { req, .. } => {
                if let Some(rx) = st.redeems.remove(&req) {
                    out.extend(self.redeemed(st, rx, req, None));
                }
            }
            Ctl::BindBroken { rx, peer } => self.bind_broken(st, rx, peer),
            // Records only a child sends, or a second Hello: ignored.
            Ctl::Hello { .. } | Ctl::Relay { .. } | Ctl::Redeem { .. } | Ctl::Bind { .. } => {}
        }
        out
    }

    /// The broker answered relay `req` for carrier end `carrier`.
    fn relay_done(st: &mut State, carrier: u64, req: u64, ticket: Option<Ticket>) {
        let Some(End {
            role: Role::Sender { outbox, broken, .. },
            ..
        }) = st.ends.get_mut(&carrier)
        else {
            return;
        };
        let pos = outbox
            .iter()
            .position(|o| matches!(o, Out::Carry(c) if c.relaying == Some(req)));
        let Some(pos) = pos else { return };
        match ticket {
            Some(t) => {
                if let Some(Out::Carry(c)) = outbox.get_mut(pos) {
                    let rec = c.record.take().unwrap_or_default();
                    if let Some(slot) = outbox.get_mut(pos) {
                        *slot = Out::Ready(join_data(&t, &rec));
                    }
                }
            }
            None => {
                // A refused relay closes the carrying channel for this
                // sender (design review M11): nothing after it can keep
                // its sequence.
                outbox.clear();
                *broken = true;
            }
        }
    }

    /// Completes redemption `req` on receiver `rx`: `got` is what the
    /// broker handed over, or `None` if it refused.
    fn redeemed(
        self: &Arc<Self>,
        st: &mut State,
        rx: u64,
        req: u64,
        got: Option<Vec<(Option<OwnedHandle>, WireKind)>>,
    ) -> Vec<Box<dyn Any + Send>> {
        let mut out: Vec<Box<dyn Any + Send>> = Vec::new();
        let Some(End {
            role: Role::Receiver {
                listener: Some(l), ..
            },
            ..
        }) = st.ends.get_mut(&rx)
        else {
            if let Some(g) = got {
                out.push(Box::new(g));
            }
            return out;
        };
        let Some(pos) = l
            .queue
            .iter()
            .position(|i| matches!(i, Item::Pending { req: r, .. } if *r == req))
        else {
            if let Some(g) = got {
                out.push(Box::new(g));
            }
            return out;
        };
        let Some(Item::Pending { payload, descs, .. }) = l.queue.remove(pos) else {
            return out;
        };
        let checked = got.as_ref().map_or(Err(Reason::Redeem), |g| {
            let kinds: Vec<WireKind> = g.iter().map(|(_, k)| *k).collect();
            if g.iter().any(|(h, _)| h.is_none()) {
                return Err(Reason::Redeem);
            }
            check_redeemed(&descs, &kinds)
        });
        match (checked, got) {
            (Ok(kinds), Some(got)) => {
                let mut handles = Vec::with_capacity(got.len());
                for (((h, _), kind), desc) in got.into_iter().zip(kinds).zip(&descs) {
                    let Some(h) = h else { continue };
                    let role = match kind {
                        HandleKind::ChannelSender | HandleKind::BoundChannelSender => {
                            let Ok(flow) = SenderFlow::from_transfer(desc) else {
                                continue;
                            };
                            Role::Sender {
                                flow,
                                bound: kind == HandleKind::BoundChannelSender,
                                made_here: false,
                                granted: false,
                                outbox: VecDeque::new(),
                                dropped: false,
                                in_transit: false,
                                broken: false,
                            }
                        }
                        HandleKind::ChannelReceiver => {
                            let Ok(flow) = ReceiverFlow::from_transfer(desc) else {
                                continue;
                            };
                            Role::Receiver {
                                flow,
                                bound: false,
                                listener: None,
                                in_transit: false,
                            }
                        }
                    };
                    if let Some(id) = Self::insert(st, h, role) {
                        handles.push(Handle::new(Box::new(WinEnd {
                            inner: Arc::clone(self),
                            id,
                            kind,
                            armed: true,
                        })));
                    }
                }
                if let Some(End {
                    role:
                        Role::Receiver {
                            listener: Some(l), ..
                        },
                    ..
                }) = st.ends.get_mut(&rx)
                {
                    l.queue.insert(pos, Item::Message(payload, handles));
                }
                self.kick(st, rx);
            }
            (Err(r), got) => {
                if let Some(g) = got {
                    out.push(Box::new(g));
                }
                let fault = Fault {
                    kind: r.fault_kind(),
                    sender: None,
                };
                self.end_listener(st, rx, Item::Violation(fault));
            }
            (Ok(_), None) => {}
        }
        out
    }

    // ---- the IO thread ---------------------------------------------------

    /// Starts every read and write that should be in flight, and moves
    /// records that carry handles along. Returns the earliest transfer
    /// deadline still pending.
    fn pump(self: &Arc<Self>, st: &mut State) -> (Option<StdInstant>, Vec<Box<dyn Any + Send>>) {
        let mut later: Vec<Box<dyn Any + Send>> = Vec::new();
        let mut deadline: Option<StdInstant> = None;
        let ids: Vec<u64> = st.ends.keys().copied().collect();
        for &id in &ids {
            self.advance_carry(st, id, &mut deadline, &mut later);
        }
        let ids: Vec<u64> = st.ends.keys().copied().collect();
        let mut lost_links: Vec<Proc> = Vec::new();
        for id in ids {
            let Some(end) = st.ends.get_mut(&id) else {
                continue;
            };
            let mut credit = None;
            let (read, write): (Option<usize>, Option<Vec<u8>>) = match &mut end.role {
                Role::Sender {
                    outbox,
                    broken,
                    in_transit,
                    ..
                } => {
                    let read = (!*broken && !*in_transit).then_some(CREDIT_READ);
                    let write = if end.writing || *broken {
                        None
                    } else if matches!(outbox.front(), Some(Out::Ready(_))) {
                        match outbox.pop_front() {
                            Some(Out::Ready(b)) => Some(b),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    (read, write)
                }
                Role::Receiver {
                    flow,
                    listener: Some(l),
                    in_transit: false,
                    ..
                } => {
                    let read = (!l.ended && !flow.paused()).then_some(MAX_PIPE_DATA);
                    let write = if end.writing || !l.credit.is_pending() {
                        None
                    } else {
                        credit = l.credit.take();
                        credit.map(|c| encode_credit(c).to_vec())
                    };
                    (read, write)
                }
                Role::Receiver { .. } => (None, None),
                Role::Link { out, .. } => {
                    let write = if end.writing { None } else { out.pop_front() };
                    (Some(MAX_CTL_LEN), write)
                }
            };
            if let Some(cap) = read
                && !end.reading
            {
                end.reading = sys::start_read(&end.h, cap, id).is_ok();
            }
            if let Some(msg) = write {
                match sys::start_write(&end.h, msg, id) {
                    Ok(()) => end.writing = true,
                    Err(_) => match &mut end.role {
                        Role::Sender { broken, outbox, .. } => {
                            *broken = true;
                            outbox.clear();
                        }
                        Role::Receiver {
                            listener: Some(l), ..
                        } => {
                            // Retried on the next pass.
                            if let Some(c) = credit {
                                l.credit.restore(c);
                            }
                        }
                        // The link is broken, as when a read on it fails.
                        Role::Link { proc, .. } => lost_links.push(*proc),
                        Role::Receiver { listener: None, .. } => {}
                    },
                }
            }
        }
        if st.broker.is_some() {
            for p in lost_links {
                later.extend(self.child_gone(st, p));
            }
        }
        // Dropped senders whose outbox is done.
        let done: Vec<u64> = st
            .ends
            .iter()
            .filter(|(_, e)| {
                matches!(&e.role, Role::Sender { dropped: true, outbox, broken, .. }
                    if (outbox.is_empty() && !e.writing) || *broken)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in done {
            if let Some(e) = Self::remove(st, id) {
                later.push(Box::new(e.h));
            }
        }
        (deadline, later)
    }

    /// Moves the record at the front of sender `id`'s outbox along, if
    /// it carries handles: once the carried ends are quiet, encodes it
    /// with their final counters and relays them (to the broker, or
    /// locally in the broker).
    fn advance_carry(
        self: &Arc<Self>,
        st: &mut State,
        id: u64,
        deadline: &mut Option<StdInstant>,
        later: &mut Vec<Box<dyn Any + Send>>,
    ) {
        let (carried, since) = match st.ends.get(&id).map(|e| &e.role) {
            Some(Role::Sender {
                outbox,
                broken: false,
                ..
            }) => match outbox.front() {
                Some(Out::Carry(c)) if c.relaying.is_none() && c.record.is_none() => {
                    (c.ends.clone(), c.since)
                }
                _ => return,
            },
            _ => return,
        };
        // Quiet: no I/O in flight on any carried end, and every carried
        // sender's own outbox sent (order across a transfer), or given up
        // at the deadline.
        let due = since.checked_add(TRANSFER_DEADLINE).unwrap_or(since);
        let late = StdInstant::now() >= due;
        let mut quiet = true;
        for (cid, _) in &carried {
            let Some(e) = st.ends.get_mut(cid) else {
                continue;
            };
            if e.reading || e.writing {
                quiet = false;
            }
            if let Role::Sender { outbox, broken, .. } = &mut e.role
                && !outbox.is_empty()
                && !*broken
            {
                if late {
                    outbox.clear();
                    *broken = true;
                } else {
                    quiet = false;
                }
            }
        }
        if !quiet {
            *deadline = Some(deadline.map_or(due, |d| d.min(due)));
            return;
        }
        // Encode the record now, with the carried ends' final counters.
        let mut descs = Vec::with_capacity(carried.len());
        for (cid, kind) in &carried {
            match st.ends.get(cid).map(|e| &e.role) {
                Some(Role::Sender { flow, .. }) => descs.push(flow.to_desc(*kind)),
                Some(Role::Receiver { flow, .. }) => descs.push(flow.to_desc()),
                Some(Role::Link { .. }) | None => {
                    Self::break_sender(st, id);
                    return;
                }
            }
        }
        let record = match st.ends.get(&id).map(|e| &e.role) {
            Some(Role::Sender { outbox, .. }) => match outbox.front() {
                Some(Out::Carry(c)) => encode_data(c.seq, &descs, &c.payload).ok(),
                _ => None,
            },
            _ => None,
        };
        let Some(record) = record else {
            Self::break_sender(st, id);
            return;
        };
        // Take the carried ends out of this process's table.
        let mut taken = Vec::with_capacity(carried.len());
        for (cid, kind) in &carried {
            let Some(e) = st.ends.remove(cid) else {
                continue;
            };
            match Arc::try_unwrap(e.h) {
                Ok(h) => taken.push((h, *kind)),
                Err(shared) => {
                    // Still referenced by an operation: cannot move.
                    later.push(Box::new(shared));
                    Self::break_sender(st, id);
                    return;
                }
            }
        }
        let carrier = st
            .ends
            .get(&id)
            .map(|e| (sys::value(&e.h), sys::facts(&e.h)));
        let Some((carrier_value, carrier_facts)) = carrier else {
            later.push(Box::new(taken));
            return;
        };
        if let Some(b) = st.broker.as_mut() {
            // The broker relays its own handles locally.
            let items: Vec<(u64, WireKind, Option<OwnedHandle>)> = taken
                .into_iter()
                .map(|(h, k)| (sys::value(&h), k, Some(h)))
                .collect();
            let r = Self::key_of(b, &carrier_facts).map_or(Err(Refusal::Carrier), |k| {
                Self::broker_relay(b, BROKER, k, items)
            });
            match r {
                Ok(t) => {
                    if let Some(End {
                        role: Role::Sender { outbox, .. },
                        ..
                    }) = st.ends.get_mut(&id)
                        && let Some(front) = outbox.front_mut()
                    {
                        *front = Out::Ready(join_data(&t, &record));
                    }
                }
                Err(_) => Self::break_sender(st, id),
            }
            return;
        }
        // A child: ask the broker.
        let items: Vec<CtlItem> = taken
            .iter()
            .map(|(h, k)| CtlItem {
                value: sys::value(h),
                kind: *k,
            })
            .collect();
        let Some(items) = Items::from_slice(&items) else {
            later.push(Box::new(taken));
            Self::break_sender(st, id);
            return;
        };
        let req = st.req();
        if let Some(End {
            role: Role::Sender { outbox, .. },
            ..
        }) = st.ends.get_mut(&id)
            && let Some(Out::Carry(c)) = outbox.front_mut()
        {
            c.record = Some(record);
            c.relaying = Some(req);
        }
        st.relays.insert(req, id);
        self.send_ctl(
            st,
            None,
            &Ctl::Relay {
                req,
                carrier: carrier_value,
                items,
            },
        );
        // The broker pulls these values with DUPLICATE_CLOSE_SOURCE:
        // they are no longer this process's to close.
        for (h, _) in taken {
            sys::forget(h);
        }
    }

    fn break_sender(st: &mut State, id: u64) {
        if let Some(End {
            role: Role::Sender { outbox, broken, .. },
            ..
        }) = st.ends.get_mut(&id)
        {
            outbox.clear();
            *broken = true;
        }
    }

    /// Handles one finished operation.
    fn complete(self: &Arc<Self>, st: &mut State, c: sys::Completion) -> Vec<Box<dyn Any + Send>> {
        /// What to do once the end's borrow ends.
        enum Next {
            Nothing,
            Kick,
            EndWith(Item),
            Redeem(Ticket, Vec<u8>, Vec<HandleDesc>, u64),
            LinkLost(Proc),
            BrokerCtl(Proc, Ctl),
            ChildCtl(Ctl),
        }
        let mut later: Vec<Box<dyn Any + Send>> = Vec::new();
        let id = c.token;
        let is_broker = st.broker.is_some();
        let Some(end) = st.ends.get_mut(&id) else {
            later.push(Box::new(c.buf));
            return later;
        };
        if c.write {
            end.writing = false;
            if c.error != 0
                && let Role::Sender { broken, outbox, .. } = &mut end.role
            {
                *broken = true;
                outbox.clear();
            }
            return later;
        }
        end.reading = false;
        let rx_value = sys::value(&end.h);
        let next = match &mut end.role {
            Role::Sender { flow, broken, .. } => {
                match c.error {
                    0 => match decode_credit(&c.buf) {
                        Ok(credit) if flow.on_credit(credit).is_ok() => {}
                        // A garbage credit, or a data record sent the
                        // wrong way: the end is broken.
                        _ => *broken = true,
                    },
                    sys::ERROR_OPERATION_ABORTED => {}
                    _ => *broken = true,
                }
                Next::Nothing
            }
            Role::Receiver {
                flow,
                listener: Some(l),
                ..
            } => {
                if l.ended {
                    Next::Nothing
                } else {
                    match c.error {
                        0 | sys::ERROR_MORE_DATA => {
                            match decide(&c.buf, c.error == sys::ERROR_MORE_DATA, flow) {
                                Decision::Accept { view, ticket } if ticket == NO_TICKET => {
                                    l.queue.push_back(Item::Message(
                                        view.payload.to_vec(),
                                        Vec::new(),
                                    ));
                                    Next::Kick
                                }
                                Decision::Accept { view, ticket } => Next::Redeem(
                                    ticket,
                                    view.payload.to_vec(),
                                    view.descs().to_vec(),
                                    rx_value,
                                ),
                                Decision::Violation(r) => Next::EndWith(Item::Violation(Fault {
                                    kind: r.fault_kind(),
                                    sender: None,
                                })),
                            }
                        }
                        sys::ERROR_OPERATION_ABORTED => Next::Nothing,
                        // End of the channel: every queued message was
                        // read (probe P9); or any other read failure.
                        _ => Next::EndWith(Item::Closed),
                    }
                }
            }
            Role::Receiver { listener: None, .. } => Next::Nothing,
            Role::Link { proc, .. } => {
                let p = *proc;
                if c.error != 0 {
                    Next::LinkLost(p)
                } else {
                    match decode_ctl(&c.buf) {
                        Ok(ctl) if is_broker => Next::BrokerCtl(p, ctl),
                        Ok(ctl) => Next::ChildCtl(ctl),
                        Err(_) => Next::LinkLost(p),
                    }
                }
            }
        };
        later.push(Box::new(c.buf));
        match next {
            Next::Nothing => {}
            Next::Kick => self.kick(st, id),
            Next::EndWith(item) => self.end_listener(st, id, item),
            Next::Redeem(ticket, payload, descs, rx_value) => {
                let req = st.req();
                if let Some(End {
                    role:
                        Role::Receiver {
                            listener: Some(l), ..
                        },
                    ..
                }) = st.ends.get_mut(&id)
                {
                    l.queue.push_back(Item::Pending {
                        req,
                        payload,
                        descs,
                    });
                }
                later.extend(self.redeem(st, id, req, ticket, rx_value));
            }
            // A child whose broker is gone keeps running; its relays and
            // redemptions never complete.
            Next::LinkLost(p) => {
                if is_broker {
                    later.extend(self.child_gone(st, p));
                }
            }
            Next::BrokerCtl(p, ctl) => later.extend(self.broker_ctl(st, p, ctl)),
            Next::ChildCtl(ctl) => later.extend(self.child_ctl(st, ctl)),
        }
        later
    }

    /// Redeems `ticket`, read on receiver `rx`.
    fn redeem(
        self: &Arc<Self>,
        st: &mut State,
        rx: u64,
        req: u64,
        ticket: Ticket,
        rx_value: u64,
    ) -> Vec<Box<dyn Any + Send>> {
        if let Some(b) = st.broker.as_mut() {
            let key = st
                .ends
                .get(&rx)
                .and_then(|e| Self::key_of(b, &sys::facts(&e.h)));
            let r = key
                .ok_or(crate::ipc::win::ledger::LedgerError::NotHolder)
                .and_then(|k| b.ledger.redeem(BROKER, &ticket, k));
            let (got, events) = match r {
                Ok((held, events)) => {
                    let hs = b.held.remove(&ticket).unwrap_or_default();
                    let got: Vec<(Option<OwnedHandle>, WireKind)> = hs
                        .into_iter()
                        .zip(held.ends.iter().map(|(_, _, k)| *k))
                        .map(|(h, k)| (Some(h), k))
                        .collect();
                    (Some(got), events)
                }
                Err(_) => (None, Vec::new()),
            };
            let out = self.redeemed(st, rx, req, got);
            self.dispatch(st, events);
            return out;
        }
        st.redeems.insert(req, rx);
        self.send_ctl(
            st,
            None,
            &Ctl::Redeem {
                req,
                ticket,
                rx: rx_value,
            },
        );
        Vec::new()
    }

    /// The IO thread.
    fn io_loop(self: &Arc<Self>) {
        loop {
            self.wake_pending.store(false, Ordering::Release);
            let mut st = self.lock();
            if st.stopping {
                return;
            }
            let (deadline, later) = self.pump(&mut st);
            drop(st);
            drop(later);
            let timeout = deadline.map(|d| {
                let left = d.saturating_duration_since(StdInstant::now());
                u32::try_from(left.as_millis()).unwrap_or(u32::MAX).max(1)
            });
            sys::wait(timeout);
            let done = sys::take_completions();
            if done.is_empty() {
                continue;
            }
            let mut st = self.lock();
            let mut later = Vec::new();
            for c in done {
                later.extend(self.complete(&mut st, c));
            }
            drop(st);
            drop(later);
        }
    }
}

/// A channel end this transport made or received.
struct WinEnd {
    inner: Arc<Inner>,
    id: u64,
    kind: HandleKind,
    /// False once the end became a listener or was sent.
    armed: bool,
}

impl core::fmt::Debug for WinEnd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WinEnd")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .finish()
    }
}

impl HandleObject for WinEnd {
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

impl Drop for WinEnd {
    fn drop(&mut self) {
        if self.armed {
            self.inner.release(self.id);
        }
    }
}

/// A listener; dropping it closes the receiver.
struct WinListening {
    inner: Arc<Inner>,
    id: u64,
}

impl core::fmt::Debug for WinListening {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WinListening")
            .field("id", &self.id)
            .finish()
    }
}

impl HandleObject for WinListening {
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

impl Drop for WinListening {
    fn drop(&mut self) {
        let mut st = self.inner.lock();
        let gone = Inner::remove(&mut st, self.id);
        drop(st);
        drop(gone);
        self.inner.wake();
    }
}

/// The Windows transport of one process.
pub(crate) struct WindowsIpc {
    inner: Arc<Inner>,
}

impl core::fmt::Debug for WindowsIpc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WindowsIpc").finish_non_exhaustive()
    }
}

/// Why the transport could not start.
#[derive(Debug)]
pub enum StartError {
    /// The IO thread could not start.
    Os(std::io::Error),
    /// The launch link carried no valid `Hello`.
    Bootstrap,
}

impl WindowsIpc {
    /// Starts the transport and its IO thread. With `link`, this process
    /// is a child of the broker at the link's other end: its `Hello`
    /// (the launch channels) is read first. Without, this process is a
    /// broker.
    pub(crate) fn start(
        exec: Arc<OsExecutor>,
        link: Option<OwnedHandle>,
    ) -> Result<Self, StartError> {
        let container = sys::app_container_sid();
        let mut state = State::default();
        if let Some(link) = link {
            let hello = sys::read_blocking(&link, MAX_CTL_LEN, HELLO_TIMEOUT_MS)
                .map_err(|_e| StartError::Bootstrap)?;
            let Ok(Ctl::Hello {
                to_parent,
                from_parent,
                parent_pid,
            }) = decode_ctl(&hello)
            else {
                return Err(StartError::Bootstrap);
            };
            if to_parent == from_parent || [to_parent, from_parent].contains(&sys::value(&link)) {
                return Err(StartError::Bootstrap);
            }
            let (Some(up), Some(down)) = (
                sys::adopt_pipe(to_parent, false),
                sys::adopt_pipe(from_parent, true),
            ) else {
                return Err(StartError::Bootstrap);
            };
            state.boot = Some((up, down, parent_pid));
            state.link = Inner::insert(
                &mut state,
                link,
                Role::Link {
                    proc: BROKER,
                    out: VecDeque::new(),
                },
            );
        } else {
            let mut ledger = Ledger::default();
            ledger.add_proc(BROKER, sys::process_id());
            state.broker = Some(Broker {
                ledger,
                held: BTreeMap::new(),
                children: BTreeMap::new(),
                next_proc: 1,
            });
        }
        let inner = Arc::new(Inner {
            exec,
            state: Mutex::new(state),
            thread: OnceLock::new(),
            wake_pending: AtomicBool::new(false),
            container,
        });
        let io = Arc::clone(&inner);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<()>(1);
        crate::thread::spawn("hg-ipc-io", move || {
            if let Ok(t) = sys::current_thread() {
                let _ = io.thread.set(t);
            }
            let _ = ready_tx.send(());
            io.io_loop();
        })
        .map_err(StartError::Os)?;
        let _ = ready_rx.recv();
        Ok(Self { inner })
    }

    /// The end `h` names, if it is this transport's and held here.
    fn held_end(&self, h: &Handle) -> Option<u64> {
        let e = h.object().downcast_ref::<WinEnd>()?;
        if !Arc::ptr_eq(&e.inner, &self.inner) {
            return None;
        }
        self.inner.lock().ends.contains_key(&e.id).then_some(e.id)
    }

    fn handle(&self, id: u64, kind: HandleKind) -> Handle {
        Handle::new(Box::new(WinEnd {
            inner: Arc::clone(&self.inner),
            id,
            kind,
            armed: true,
        }))
    }

    fn start_listening(
        &self,
        from: ChannelReceiver,
        peer: Option<u32>,
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
        let Some(End {
            h,
            role: Role::Receiver {
                bound, listener, ..
            },
            ..
        }) = st.ends.get_mut(&id)
        else {
            return Err(IpcError::Refused);
        };
        if listener.is_some() || (*bound && peer.is_none()) {
            return Err(IpcError::Refused);
        }
        let rx_value = sys::value(h);
        let rx_facts = peer.map(|_| sys::facts(h));
        *listener = Some(Listener {
            seq,
            sink: Arc::new(Mutex::new(sink)),
            queue: VecDeque::new(),
            draining: false,
            ended: false,
            credit: PendingCredit::default(),
        });
        if let Some(p) = peer {
            if let Some(b) = st.broker.as_mut() {
                let key = rx_facts.and_then(|f| Inner::key_of(b, &f));
                let ev = key.and_then(|k| b.ledger.bind(BROKER, k, rx_value, p).ok().flatten());
                if let Some(ev) = ev {
                    self.inner.dispatch(&mut st, vec![ev]);
                }
            } else {
                self.inner.send_ctl(
                    &mut st,
                    None,
                    &Ctl::Bind {
                        rx: rx_value,
                        peer: p,
                    },
                );
            }
        }
        drop(st);
        disarm(from.into_handle());
        self.inner.wake();
        Ok(Listening::new(Handle::new(Box::new(WinListening {
            inner: Arc::clone(&self.inner),
            id,
        }))))
    }
}

impl Drop for WindowsIpc {
    fn drop(&mut self) {
        self.inner.lock().stopping = true;
        self.inner.wake();
    }
}

/// Disarms a handle object that became something else (sent, listened).
fn disarm(h: Handle) {
    if let Ok(mut e) = h.into_object().downcast::<WinEnd>() {
        e.armed = false;
    }
}

impl Ipc for WindowsIpc {
    fn channel(
        &self,
        capacity: u32,
        transfer: Transfer,
    ) -> Result<(ChannelSender, ChannelReceiver), IpcError> {
        let name = Inner::random16().ok_or(IpcError::Closed)?;
        let (server, client) = sys::pipe_pair(self.inner.container.as_deref(), &name)
            .map_err(|_e| IpcError::Closed)?;
        let bound = transfer == Transfer::Bound;
        let mut st = self.inner.lock();
        let tx = Inner::insert(
            &mut st,
            client,
            Role::Sender {
                flow: SenderFlow::new(capacity),
                bound,
                made_here: true,
                granted: false,
                outbox: VecDeque::new(),
                dropped: false,
                in_transit: false,
                broken: false,
            },
        );
        let rx = Inner::insert(
            &mut st,
            server,
            Role::Receiver {
                flow: ReceiverFlow::new(capacity),
                bound,
                listener: None,
                in_transit: false,
            },
        );
        drop(st);
        self.inner.wake();
        let (Some(tx), Some(rx)) = (tx, rx) else {
            return Err(IpcError::Closed);
        };
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
        match st.ends.get(&to_id).map(|e| &e.role) {
            Some(Role::Sender {
                broken: false,
                in_transit: false,
                ..
            }) => {}
            Some(Role::Sender { .. }) => return Err(IpcError::Closed),
            _ => return Err(IpcError::Refused),
        }
        // Every carried handle: a bound or listening receiver never
        // moves; a bound sender moves once, from its maker (IPC-R2 (a)).
        let mut carried = Vec::with_capacity(ids.len());
        for (&id, h) in ids.iter().zip(&handles) {
            let Some(end) = st.ends.get(&id) else {
                return Err(IpcError::Refused);
            };
            let kind = match &end.role {
                Role::Receiver { bound: true, .. }
                | Role::Receiver {
                    listener: Some(_), ..
                }
                | Role::Receiver {
                    in_transit: true, ..
                } => return Err(IpcError::Refused),
                Role::Receiver { .. } => WireKind::Receiver,
                Role::Sender {
                    bound,
                    made_here,
                    granted,
                    broken,
                    in_transit,
                    ..
                } => {
                    if *bound && (!*made_here || *granted) || *in_transit {
                        return Err(IpcError::Refused);
                    }
                    if *broken {
                        return Err(IpcError::Closed);
                    }
                    if h.kind() == HandleKind::BoundChannelSender {
                        WireKind::BoundSender
                    } else {
                        WireKind::Sender
                    }
                }
                Role::Link { .. } => return Err(IpcError::Refused),
            };
            carried.push((id, kind));
        }
        let Some(End {
            role: Role::Sender { flow, outbox, .. },
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
        if carried.is_empty() {
            let Ok(record) = encode_data(seq, &[], &bytes) else {
                return Err(IpcError::TooLarge);
            };
            outbox.push_back(Out::Ready(join_data(&NO_TICKET, &record)));
        } else {
            outbox.push_back(Out::Carry(Carry {
                seq,
                payload: bytes,
                ends: carried.clone(),
                since: StdInstant::now(),
                record: None,
                relaying: None,
            }));
        }
        *flow = next;
        // The carried ends stop their own I/O now (a read left pending on
        // a moved end would steal the next message, probe W3).
        for (id, _) in &carried {
            if let Some(e) = st.ends.get_mut(id) {
                match &mut e.role {
                    Role::Sender {
                        in_transit,
                        granted,
                        ..
                    } => {
                        *in_transit = true;
                        *granted = true;
                    }
                    Role::Receiver { in_transit, .. } => *in_transit = true,
                    Role::Link { .. } => {}
                }
                sys::cancel(&e.h);
            }
        }
        drop(st);
        for h in handles {
            disarm(h);
        }
        self.inner.wake();
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
        if peer.pid() == 0 {
            return Err(IpcError::Refused);
        }
        self.start_listening(from, Some(peer.pid()), seq, sink)
    }

    fn bootstrap(&self) -> Option<Bootstrap> {
        let (up, down, parent_pid) = self.inner.lock().boot.take()?;
        let mut st = self.inner.lock();
        let tx = Inner::insert(
            &mut st,
            up,
            Role::Sender {
                flow: SenderFlow::new(BOOTSTRAP_CAPACITY),
                bound: false,
                made_here: false,
                granted: false,
                outbox: VecDeque::new(),
                dropped: false,
                in_transit: false,
                broken: false,
            },
        );
        let rx = Inner::insert(
            &mut st,
            down,
            Role::Receiver {
                flow: ReceiverFlow::new(BOOTSTRAP_CAPACITY),
                bound: false,
                listener: None,
                in_transit: false,
            },
        );
        drop(st);
        self.inner.wake();
        let to_parent =
            ChannelSender::from_handle(self.handle(tx?, HandleKind::ChannelSender)).ok()?;
        let from_parent =
            ChannelReceiver::from_handle(self.handle(rx?, HandleKind::ChannelReceiver)).ok()?;
        Some(Bootstrap {
            to_parent,
            from_parent,
            parent: OsPeer::new(parent_pid, 0),
        })
    }
}

/// A child the broker started: its process, its launch channel ends on
/// the broker's side, and its identity.
#[derive(Debug)]
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
pub(crate) struct Launched {
    /// The child process (to wait for or end it).
    pub(crate) child: sys::spawn::Child,
    /// The channel from the child.
    pub(crate) from_child: ChannelReceiver,
    /// The channel to the child.
    pub(crate) to_child: ChannelSender,
    /// The child as the OS identifies it.
    pub(crate) peer: OsPeer,
}

/// What to start a child with (for the launcher, phase E, and tests).
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
pub(crate) struct LaunchSpec<'a> {
    /// The executable.
    pub(crate) exe: &'a std::path::Path,
    /// Its command line after the path.
    pub(crate) args: &'a str,
    /// Environment variables added for it; the link handle's value is
    /// added as `IZCAL_IPC_LINK`.
    pub(crate) env: Vec<(String, String)>,
    /// Sandboxed: its own AppContainer (with that container's SID), the
    /// lockdown token and its job. `None` starts it plainly.
    pub(crate) sandbox: Option<(
        &'a crate::sandbox::windows::AppContainerSid,
        &'a crate::sandbox::windows::OwnedHandle,
        &'a crate::sandbox::windows::OwnedHandle,
    )>,
}

/// The environment variable that carries a child's link handle value.
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
pub(crate) const LINK_VAR: &str = "IZCAL_IPC_LINK";

impl WindowsIpc {
    /// Starts a child with its control link and launch channels; this
    /// process must be a broker. The child reads its `Hello` when its
    /// transport starts.
    #[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
    pub(crate) fn launch(&self, spec: LaunchSpec<'_>) -> std::io::Result<Launched> {
        let name = || {
            Inner::random16().ok_or_else(|| std::io::Error::other("no randomness for a pipe name"))
        };
        let (link_server, link_client) = sys::link_pair(&name()?)?;
        let (up_rx, up_tx) = sys::pipe_pair(None, &name()?)?;
        let (down_rx, down_tx) = sys::pipe_pair(None, &name()?)?;
        let inherit = sys::inheritable(&link_client)?;
        let mut env = spec.env;
        env.push((LINK_VAR.to_owned(), sys::value(&inherit).to_string()));
        let sid = spec.sandbox.and_then(|(c, _, _)| sys::sid_string(c));
        let child = sys::spawn::start(&sys::spawn::Spec {
            exe: spec.exe,
            args: spec.args,
            env: &env,
            inherit: &inherit,
            sandbox: spec.sandbox,
        })?;
        drop(inherit);
        drop(link_client);
        let started = (|| -> std::io::Result<(u64, u64)> {
            let v_up = sys::give(&child.process, &up_tx)
                .map_err(|e| std::io::Error::from_raw_os_error(i32::try_from(e).unwrap_or(0)))?;
            let v_down = sys::give(&child.process, &down_rx)
                .map_err(|e| std::io::Error::from_raw_os_error(i32::try_from(e).unwrap_or(0)))?;
            Ok((v_up, v_down))
        })();
        let (v_up, v_down) = match started {
            Ok(v) => v,
            Err(e) => {
                sys::spawn::kill(&child);
                return Err(e);
            }
        };
        let up_key = sys::facts(&up_rx);
        let down_key = sys::facts(&down_tx);
        drop((up_tx, down_rx));
        let process = match sys::spawn::process_handle(&child) {
            Ok(p) => p,
            Err(e) => {
                sys::spawn::kill(&child);
                return Err(e);
            }
        };
        let mut st = self.inner.lock();
        let Some(b) = st.broker.as_mut() else {
            sys::spawn::kill(&child);
            return Err(std::io::Error::other("only a broker launches"));
        };
        let p = b.next_proc;
        b.next_proc = b.next_proc.saturating_add(1);
        b.ledger.add_proc(p, child.pid);
        for (f, end) in [(&up_key, PipeEnd::Sender), (&down_key, PipeEnd::Receiver)] {
            if let Some(k) = Inner::key_of(b, f) {
                let _ = b.ledger.place(k, end, p);
            }
        }
        let link = Inner::insert(
            &mut st,
            link_server,
            Role::Link {
                proc: p,
                out: VecDeque::new(),
            },
        );
        let Some(link) = link else {
            if let Some(b) = st.broker.as_mut() {
                let _ = b.ledger.gone(p);
                b.ledger.sweep();
            }
            drop(st);
            sys::spawn::kill(&child);
            return Err(std::io::Error::other("no room for the link"));
        };
        if let Some(b) = st.broker.as_mut() {
            b.children.insert(p, ChildEntry { process, sid, link });
        }
        let hello = Ctl::Hello {
            to_parent: v_up,
            from_parent: v_down,
            parent_pid: sys::process_id(),
        };
        self.inner.send_ctl(&mut st, Some(p), &hello);
        let rx = Inner::insert(
            &mut st,
            up_rx,
            Role::Receiver {
                flow: ReceiverFlow::new(BOOTSTRAP_CAPACITY),
                bound: false,
                listener: None,
                in_transit: false,
            },
        );
        let tx = Inner::insert(
            &mut st,
            down_tx,
            Role::Sender {
                flow: SenderFlow::new(BOOTSTRAP_CAPACITY),
                bound: false,
                made_here: true,
                granted: false,
                outbox: VecDeque::new(),
                dropped: false,
                in_transit: false,
                broken: false,
            },
        );
        let (Some(rx), Some(tx)) = (rx, tx) else {
            let dropped = self.inner.abandon(&mut st, p, [rx, tx]);
            drop(st);
            drop(dropped);
            sys::spawn::kill(&child);
            return Err(std::io::Error::other("no room for the launch ends"));
        };
        drop(st);
        self.inner.wake();
        if let Err(e) = sys::spawn::resume(&child) {
            let mut st = self.inner.lock();
            let dropped = self.inner.abandon(&mut st, p, [Some(rx), Some(tx)]);
            drop(st);
            drop(dropped);
            sys::spawn::kill(&child);
            return Err(e);
        }
        let from_child = ChannelReceiver::from_handle(self.handle(rx, HandleKind::ChannelReceiver))
            .map_err(|_e| std::io::Error::other("kind"))?;
        let to_child = ChannelSender::from_handle(self.handle(tx, HandleKind::ChannelSender))
            .map_err(|_e| std::io::Error::other("kind"))?;
        let peer = OsPeer::new(child.pid, 0);
        Ok(Launched {
            child,
            from_child,
            to_child,
            peer,
        })
    }
}
