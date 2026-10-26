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
//! **A compromised holder can keep reading an end it gives away.** The
//! move protocol has the giving transport cancel the carried end's
//! reads and wait for them to abort before the broker pulls it (M9,
//! probe W3), which a well-behaved process does. A compromised process,
//! though, drives its own transport: it can ask the broker to relay an
//! end while it still has a `ReadFile` pending on that end. A pending
//! read is not cancelled by the broker's `DUPLICATE_CLOSE_SOURCE` pull
//! and keeps completing with messages the new holder should receive
//! (probe W3), and the broker cannot cancel another process's read
//! (probe W8b, `ERROR_NOT_FOUND`). `HandleCount == 1` does not catch
//! this, because a pending read is not a handle. So a process that gives
//! away an end of a channel it was already part of can keep stealing
//! records sent on that channel afterwards. It is confined to channels
//! that process already held; it reaches no other. This limit is
//! accepted for M1 and recorded for the design review (D5 item 6); the
//! m1-d5 measurement README lists the options considered.
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
use std::collections::{BTreeMap, BTreeSet, VecDeque};
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

/// The most control records the broker queues for a child that is not
/// reading its link before the broker gives up on that child (M8): a
/// child cannot make the broker buffer without bound.
const MAX_LINK_BACKLOG: usize = 4096;

/// The most relays one child may have in flight on relay threads at once
/// (M8): the per-child relay quota. A child over it is refused until its
/// earlier relays finish, so it cannot spawn unbounded threads or pin
/// the broker's memory, and a child whose classification blocks (a
/// crafted synchronous pipe, probe W8) stalls only its own relays.
const MAX_RELAYS_INFLIGHT: usize = 256;

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
    /// Set for a `listen_bound` listener: the ledger has a binding to
    /// drop when this receiver closes (item 8).
    bound_peer: Option<u32>,
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
    /// Relays this child has in flight on relay threads (M8): bounded so
    /// a child cannot queue unbounded relay work.
    relays: usize,
}

/// The broker's state: the ledger, the handles held under tickets, and
/// its children.
struct Broker {
    ledger: Ledger,
    held: BTreeMap<Ticket, Vec<OwnedHandle>>,
    children: BTreeMap<Proc, ChildEntry>,
    /// AppContainer SIDs of launches in progress, reserved before the
    /// child is spawned so a duplicate never runs (M10, probe W4).
    reserved_sids: BTreeSet<String>,
    /// Each sandboxed child's container SID to its process number. Kept
    /// after the child dies for as long as the ledger has a channel it
    /// created, so those channels can still be keyed and redeemed by the
    /// live processes that hold their ends (item 5).
    creator_sids: BTreeMap<String, Proc>,
    #[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher reads it"))]
    next_proc: Proc,
}

impl Broker {
    /// Whether a live child or a launch in progress already holds `sid`.
    fn sid_in_use(&self, sid: &str) -> bool {
        self.reserved_sids.contains(sid)
            || self.creator_sids.contains_key(sid)
            || self
                .children
                .values()
                .any(|c| c.sid.as_deref() == Some(sid))
    }

    /// Sweeps dead channels and forgets the SID of any creator that is no
    /// longer a live child and now has no channels (item 5).
    fn sweep(&mut self) {
        for creator in self.ledger.sweep() {
            if !self.children.contains_key(&creator) {
                self.creator_sids.retain(|_, p| *p != creator);
            }
        }
    }
}

/// One pulled handle for a relay: the value the child named, the kind it
/// claims, the handle itself (pulled with close-source, kept on success
/// and dropped on refusal), and what the kernel says about it. The pull
/// and the kernel queries run off the IO thread (M8), so this is the
/// only blocking relay work and it never holds the lock.
/// A child request that names one of its receiver ends by value, decided
/// after that end is classified off the IO thread (M8).
enum PeekJob {
    /// `Ctl::Redeem`.
    Redeem { req: u64, ticket: Ticket },
    /// `Ctl::Bind`.
    Bind { rx_id: u64, peer: u32 },
}

struct Pulled {
    value: u64,
    kind: WireKind,
    handle: Option<OwnedHandle>,
    raw: sys::RawFacts,
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

    /// If end `id` is a bound listener, drops its binding: in a child, a
    /// single `Unbind` to the broker; in the broker, the ledger itself
    /// (item 8). Call before closing a bound receiver.
    fn drop_binding(&self, st: &mut State, id: u64) {
        let peer = match st.ends.get_mut(&id).map(|e| &mut e.role) {
            Some(Role::Receiver {
                listener: Some(l), ..
            }) => l.bound_peer.take(),
            _ => None,
        };
        if peer.is_none() {
            return;
        }
        if let Some(b) = st.broker.as_mut() {
            b.ledger.unbind(BROKER, id);
        } else {
            self.send_ctl(st, None, &Ctl::Unbind { rx_id: id });
        }
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
            self.drop_binding(&mut st, id);
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
            // The map outlives the child, so a channel a dead creator
            // made is still keyed for the live processes holding its ends
            // (item 5).
            Some(s) => *b.creator_sids.get(s)?,
        };
        Some(PipeKey { creator, id })
    }

    // ---- the broker's side of the control protocol --------------------

    /// A relay: takes `items` (values in process `from`, or this
    /// process's own handles when `from` is the broker) into a ticket
    /// for a record on `carrier`.
    /// Pulls a relay's carrier and items out of `process` and asks the
    /// kernel about them, off the lock (M8). The carrier is peeked (no
    /// close-source) and its copy dropped; the items are taken with
    /// close-source. A crafted synchronous handle can make the kernel
    /// queries block here (probe W8); this runs on a relay thread, so
    /// only that child's relays wait.
    fn classify_relay(
        process: &sys::Process,
        carrier: u64,
        items: &Items,
    ) -> (sys::RawFacts, Vec<Pulled>) {
        let carrier_raw = check_value(carrier)
            .ok()
            .and_then(|()| sys::pull(process, carrier, false).ok())
            .map(|h| sys::facts(&h))
            .unwrap_or_default();
        let pulled = items
            .as_slice()
            .iter()
            .map(|i| {
                let handle = check_value(i.value)
                    .ok()
                    .and_then(|()| sys::pull(process, i.value, true).ok());
                let raw = handle.as_ref().map(sys::facts).unwrap_or_default();
                Pulled {
                    value: i.value,
                    kind: i.kind,
                    handle,
                    raw,
                }
            })
            .collect();
        (carrier_raw, pulled)
    }

    /// Decides a relay from already-pulled, already-classified handles,
    /// under the lock (M8): resolves the channel keys, runs
    /// [`accept`] on each, and on success moves the handles into a
    /// ticket. Returns the ticket or the refusal, plus any handles to
    /// drop outside the lock (the pulled copies, kept only on success).
    fn relay_decide(
        b: &mut Broker,
        from: Proc,
        carrier_raw: &sys::RawFacts,
        pulled: Vec<Pulled>,
    ) -> (Result<Ticket, Refusal>, Vec<OwnedHandle>) {
        let mut spare: Vec<OwnedHandle> = Vec::new();
        let carrier = match Self::key_of(b, carrier_raw) {
            Some(k) => k,
            None => {
                // The carrier is not this relayer's channel: refuse, and
                // drop every pulled item (it is gone from the child).
                spare.extend(pulled.into_iter().filter_map(|p| p.handle));
                return (Err(Refusal::Carrier), spare);
            }
        };
        let mut ends = Vec::with_capacity(pulled.len());
        let mut handles = Vec::with_capacity(pulled.len());
        let mut first_err = None;
        for p in pulled {
            let facts = Facts {
                raw: p.value,
                pulled: p.handle.is_some(),
                is_pipe: p.raw.is_pipe,
                server_end: p.raw.server_end,
                message: p.raw.message,
                max_instances: p.raw.max_instances,
                read_message: p.raw.read_message,
                wait: p.raw.wait,
                pipe: Self::key_of(b, &p.raw),
                handle_count: p.raw.handle_count,
            };
            let view = facts
                .pipe
                .and_then(|k| b.ledger.view(k, crate::ipc::win::accept::end_for(p.kind)));
            let ctx = crate::ipc::win::accept::Ctx {
                relayer: from,
                carrier,
                view,
            };
            match accept(p.kind, &facts, &ctx) {
                Ok((pipe, end)) => ends.push((pipe, end, p.kind)),
                Err(r) => {
                    first_err.get_or_insert(r);
                }
            }
            if let Some(h) = p.handle {
                handles.push(h);
            }
        }
        let keys: Vec<(PipeKey, PipeEnd)> = ends.iter().map(|(p, e, _)| (*p, *e)).collect();
        let result = match first_err {
            Some(r) => Err(r),
            None => distinct(&keys).and_then(|()| {
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
                Ok(ticket)
            }),
        };
        match result {
            Ok(ticket) => {
                b.held.insert(ticket, handles);
                (Ok(ticket), spare)
            }
            Err(r) => {
                spare.extend(handles);
                (Err(r), spare)
            }
        }
    }

    /// What the kernel says about a receiver end a child names by value,
    /// without taking it: a peek (no close-source) whose copy is dropped.
    /// Can wait on a synchronous end with a read pending (W8), so it runs
    /// on a request thread, never on the IO thread (M8).
    fn peek_raw(process: &sys::Process, raw: u64) -> sys::RawFacts {
        check_value(raw)
            .ok()
            .and_then(|()| sys::pull(process, raw, false).ok())
            .map(|h| sys::facts(&h))
            .unwrap_or_default()
    }

    /// The channel of a peeked receiver end, if it is one.
    fn receiver_key(b: &Broker, f: &sys::RawFacts) -> Option<PipeKey> {
        (f.is_pipe && f.server_end)
            .then(|| Self::key_of(b, f))
            .flatten()
    }

    /// Decides redemption `req` of `ticket` by child `from`, whose
    /// receiver end resolved to `key`, under the lock. Gives the held
    /// handles into the child and answers; on a partial hand-over takes
    /// back what was given and closes the ends.
    fn redeem_with_key(
        b: &mut Broker,
        from: Proc,
        req: u64,
        ticket: Ticket,
        key: Option<PipeKey>,
        later: &mut Vec<Box<dyn Any + Send>>,
        events: &mut Vec<BindEvent>,
    ) -> Ctl {
        let r = key
            .ok_or(crate::ipc::win::ledger::LedgerError::NotHolder)
            .and_then(|k| b.ledger.redeem(from, &ticket, k));
        let (held, ev) = match r {
            Ok(v) => v,
            Err(e) => {
                return Ctl::RedeemRefused {
                    req,
                    reason: e.code(),
                };
            }
        };
        events.extend(ev);
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
        later.push(Box::new(handles));
        match Items::from_slice(&items) {
            Some(items) if ok => Ctl::Redeemed { req, items },
            _ => {
                // Take back what was given, closing the child's copies,
                // so no end stays open there unreported; the ends are
                // gone.
                if let Some(c) = b.children.get(&from) {
                    for i in &items {
                        if let Ok(h) = sys::pull(&c.process, i.value, true) {
                            later.push(Box::new(h));
                        }
                    }
                }
                b.ledger.lost(&held.ends);
                b.sweep();
                Ctl::RedeemRefused { req, reason: 0 }
            }
        }
    }

    /// Spawns a Redeem or Bind off the IO thread (M8), as
    /// [`Self::spawn_relay`] does for relays, under the same per-child
    /// quota. Returns whether the thread started.
    fn spawn_peek(
        self: &Arc<Self>,
        proc: Proc,
        process: sys::Process,
        rx: u64,
        job: PeekJob,
    ) -> bool {
        let inner = Arc::clone(self);
        crate::thread::spawn("hg-ipc-peek", move || {
            let raw = Self::peek_raw(&process, rx);
            let mut st = inner.lock();
            let mut later: Vec<Box<dyn Any + Send>> = Vec::new();
            let mut events = Vec::new();
            let mut reply = None;
            if let Some(b) = st.broker.as_mut()
                && b.children.contains_key(&proc)
            {
                let key = Self::receiver_key(b, &raw);
                match job {
                    PeekJob::Redeem { req, ticket } => {
                        reply = Some(Self::redeem_with_key(
                            b,
                            proc,
                            req,
                            ticket,
                            key,
                            &mut later,
                            &mut events,
                        ));
                    }
                    PeekJob::Bind { rx_id, peer } => {
                        if let Some(k) = key
                            && let Ok(Some(ev)) = b.ledger.bind(proc, k, rx_id, peer)
                        {
                            events.push(ev);
                        }
                    }
                }
                if let Some(c) = b.children.get_mut(&proc) {
                    c.relays = c.relays.saturating_sub(1);
                }
            }
            if let Some(r) = reply {
                inner.send_ctl(&mut st, Some(proc), &r);
            }
            inner.dispatch(&mut st, events);
            drop(st);
            drop(later);
            inner.wake();
        })
        .is_ok()
    }

    /// Spawns a relay off the IO thread (M8): the pull and the kernel
    /// queries, which can block on a crafted synchronous handle (W8),
    /// run on this thread, so one child's blocked relay never stalls the
    /// broker's IO thread or another child. The per-child relay quota
    /// bounds how many such threads a child can have at once.
    fn spawn_relay(
        self: &Arc<Self>,
        proc: Proc,
        process: sys::Process,
        req: u64,
        carrier: u64,
        items: Items,
    ) -> bool {
        let inner = Arc::clone(self);
        crate::thread::spawn("hg-ipc-relay", move || {
            let (carrier_raw, pulled) = Self::classify_relay(&process, carrier, &items);
            let mut st = inner.lock();
            let alive = st
                .broker
                .as_ref()
                .is_some_and(|b| b.children.contains_key(&proc));
            let (reply, spare): (Option<Ctl>, Vec<OwnedHandle>) = match st.broker.as_mut() {
                Some(b) if alive => {
                    let (res, drop_now) = Self::relay_decide(b, proc, &carrier_raw, pulled);
                    let r = match res {
                        Ok(ticket) => Ctl::RelayOk { req, ticket },
                        Err(r) => Ctl::RelayRefused {
                            req,
                            reason: r.code(),
                        },
                    };
                    (Some(r), drop_now)
                }
                // No broker, or the child died while this relay was in
                // flight: drop the pulled handles, send nothing.
                _ => (None, pulled.into_iter().filter_map(|p| p.handle).collect()),
            };
            if let Some(b) = st.broker.as_mut()
                && let Some(c) = b.children.get_mut(&proc)
            {
                c.relays = c.relays.saturating_sub(1);
            }
            if let Some(r) = reply {
                inner.send_ctl(&mut st, Some(proc), &r);
            }
            drop(st);
            drop(spare);
            inner.wake();
        })
        .is_ok()
    }

    /// Charges child `from`'s request quota and starts a peek job;
    /// false if over quota or the thread could not start (the slot is
    /// given back).
    fn start_peek(self: &Arc<Self>, b: &mut Broker, from: Proc, rx: u64, job: PeekJob) -> bool {
        let Some(c) = b.children.get_mut(&from) else {
            return false;
        };
        if c.relays >= MAX_RELAYS_INFLIGHT {
            return false;
        }
        let Ok(process) = sys::duplicate(&c.process.0) else {
            return false;
        };
        c.relays = c.relays.saturating_add(1);
        if self.spawn_peek(from, sys::Process(process), rx, job) {
            return true;
        }
        if let Some(c) = b.children.get_mut(&from) {
            c.relays = c.relays.saturating_sub(1);
        }
        false
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
        let events: Vec<BindEvent> = Vec::new();
        let later: Vec<Box<dyn Any + Send>> = Vec::new();
        let reply = match c {
            Ctl::Relay {
                req,
                carrier,
                items,
            } => {
                // The relay's pull and kernel queries run off the IO
                // thread (M8): dup the child's process for the relay
                // thread, charge the per-child quota, and hand it over.
                // Over quota, refuse at once.
                let over = b
                    .children
                    .get(&from)
                    .is_none_or(|c| c.relays >= MAX_RELAYS_INFLIGHT);
                let dup = b
                    .children
                    .get(&from)
                    .and_then(|c| sys::duplicate(&c.process.0).ok());
                match (over, dup) {
                    (false, Some(process)) => {
                        if let Some(c) = b.children.get_mut(&from) {
                            c.relays = c.relays.saturating_add(1);
                        }
                        if self.spawn_relay(from, sys::Process(process), req, carrier, items) {
                            None
                        } else {
                            // No relay thread: give the slot back and
                            // refuse, so the child is not left waiting.
                            if let Some(c) = b.children.get_mut(&from) {
                                c.relays = c.relays.saturating_sub(1);
                            }
                            Some(Ctl::RelayRefused {
                                req,
                                reason: Refusal::Quota.code(),
                            })
                        }
                    }
                    _ => Some(Ctl::RelayRefused {
                        req,
                        reason: Refusal::Quota.code(),
                    }),
                }
            }
            Ctl::Redeem { req, ticket, rx } => {
                match self.start_peek(b, from, rx, PeekJob::Redeem { req, ticket }) {
                    true => None,
                    false => Some(Ctl::RedeemRefused {
                        req,
                        reason: crate::ipc::win::ledger::LedgerError::Quota.code(),
                    }),
                }
            }
            Ctl::Bind { rx, rx_id, peer } => {
                // A Bind that cannot be queued is dropped: the receiver
                // then stays unbound, and its listener reports no peer
                // fault it should have (the child is over its quota).
                let _ = self.start_peek(b, from, rx, PeekJob::Bind { rx_id, peer });
                None
            }
            Ctl::Unbind { rx_id } => {
                b.ledger.unbind(from, rx_id);
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
        later
    }

    /// Tells binders their receivers broke.
    fn dispatch(self: &Arc<Self>, st: &mut State, events: Vec<BindEvent>) {
        for ev in events {
            if ev.binder == BROKER {
                self.bind_broken(st, ev.rx_id, ev.peer_pid);
            } else {
                self.send_ctl(
                    st,
                    Some(ev.binder),
                    &Ctl::BindBroken {
                        rx_id: ev.rx_id,
                        peer: ev.peer_pid,
                    },
                );
            }
        }
    }

    /// Ends listener `rx_id` (the holder's own never-reused end id) with
    /// an Identity fault naming `peer`, if it is still a live listener.
    fn bind_broken(self: &Arc<Self>, st: &mut State, rx_id: u64, peer: u32) {
        let live = matches!(
            st.ends.get(&rx_id).map(|e| &e.role),
            Some(Role::Receiver {
                listener: Some(_),
                ..
            })
        );
        if live {
            let fault = Fault {
                kind: FaultKind::Identity,
                sender: Some(peer),
            };
            self.end_listener(st, rx_id, Item::Violation(fault));
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
        b.sweep();
        // A dead child whose channels are all gone (including one that
        // never made any) keeps no SID entry (item 5).
        if !b.ledger.has_creator(p) {
            b.creator_sids.retain(|_, pp| *pp != p);
        }
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
            Ctl::BindBroken { rx_id, peer } => self.bind_broken(st, rx_id, peer),
            // Records only a child sends, or a second Hello: ignored.
            Ctl::Hello { .. }
            | Ctl::Relay { .. }
            | Ctl::Redeem { .. }
            | Ctl::Bind { .. }
            | Ctl::Unbind { .. } => {}
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
        // The broker refused the redemption: the ticket and its handles
        // are gone (e.g. the channel's creator died, item 5). That is a
        // close, not a fault by the current sender, so the channel ends
        // in Closed. A record whose descriptors do not match what the
        // broker handed over is still a BadHandle violation.
        let Some(got) = got else {
            self.end_listener(st, rx, Item::Closed);
            return out;
        };
        let checked = {
            let kinds: Vec<WireKind> = got.iter().map(|(_, k)| *k).collect();
            if got.iter().any(|(h, _)| h.is_none()) {
                Err(Reason::Redeem)
            } else {
                check_redeemed(&descs, &kinds)
            }
        };
        match checked {
            Ok(kinds) => {
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
            Err(r) => {
                out.push(Box::new(got));
                let fault = Fault {
                    kind: r.fault_kind(),
                    sender: None,
                };
                self.end_listener(st, rx, Item::Violation(fault));
            }
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
        let is_broker = st.broker.is_some();
        let mut lost_links: Vec<Proc> = Vec::new();
        for id in ids {
            let Some(end) = st.ends.get_mut(&id) else {
                continue;
            };
            // A child that will not drain its link cannot make the broker
            // buffer without bound: drop the link past the cap (M8). Only
            // the broker enforces this (a child never drops its own link
            // to the broker).
            if is_broker
                && let Role::Link { proc, out } = &end.role
                && out.len() > MAX_LINK_BACKLOG
            {
                lost_links.push(*proc);
                continue;
            }
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
        // A carried end is quiet once its writes have drained (a carried
        // sender's queued messages all written, order across a transfer)
        // and its leftover credit-read is cancelled. Writes are never
        // cancelled, so the move never loses a message or lies about the
        // counters (item 4).
        let due = since.checked_add(TRANSFER_DEADLINE).unwrap_or(since);
        let late = StdInstant::now() >= due;
        let mut quiet = true;
        let mut stuck = false;
        let mut cancel_reads = Vec::new();
        for (cid, _) in &carried {
            let Some(e) = st.ends.get(cid) else {
                continue;
            };
            let draining = matches!(
                &e.role,
                Role::Sender { outbox, broken, .. } if !outbox.is_empty() && !*broken
            );
            if e.writing || draining {
                // Still writing: wait (the deadline bounds the wait).
                quiet = false;
                stuck = true;
            } else if e.reading {
                // Drained: stop the leftover read before the move (W3).
                quiet = false;
                cancel_reads.push(*cid);
            }
        }
        if !quiet {
            // Abandon only when writes are genuinely stuck past the
            // deadline; a transfer whose writes have all drained and
            // whose only leftover is the read it is about to cancel is
            // let through on the next pass, even at the deadline.
            if late && stuck {
                // The carried ends could not quiesce in time. Close them
                // and the carrier deterministically (as Linux's SHUT_WR
                // does), rather than move with messages still unwritten.
                Self::break_sender(st, id);
                for (cid, _) in &carried {
                    if let Some(e) = Self::remove(st, *cid) {
                        later.push(Box::new(e.h));
                    }
                }
                return;
            }
            for cid in cancel_reads {
                if let Some(e) = st.ends.get(&cid) {
                    sys::cancel(&e.h);
                }
            }
            // Keep a future deadline only while writes may still stall;
            // once only the read remains, its completion wakes us.
            if stuck {
                *deadline = Some(deadline.map_or(due, |d| d.min(due)));
            }
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
                    // Still referenced by an operation: cannot move. Every
                    // handle is dropped outside the lock.
                    later.push(Box::new(shared));
                    later.push(Box::new(core::mem::take(&mut taken)));
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
        if st.broker.is_some() {
            // The broker relays its own handles locally. Its ends are its
            // own overlapped pipes, so the kernel queries do not block
            // (unlike a child's possibly-crafted handle, M8), and this
            // stays inline. Classify each owned handle, then decide.
            let pulled: Vec<Pulled> = taken
                .into_iter()
                .map(|(h, kind)| Pulled {
                    value: sys::value(&h),
                    kind,
                    raw: sys::facts(&h),
                    handle: Some(h),
                })
                .collect();
            let b = match st.broker.as_mut() {
                Some(b) => b,
                None => return,
            };
            let (res, spare) = Self::relay_decide(b, BROKER, &carrier_facts, pulled);
            later.push(Box::new(spare));
            match res {
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
            // A cancelled write (the end is being torn down) is not a
            // broken peer; only a real write failure breaks the end
            // (item 4).
            if c.error != 0
                && c.error != sys::ERROR_OPERATION_ABORTED
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
        self.inner.drop_binding(&mut st, self.id);
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
                reserved_sids: BTreeSet::new(),
                creator_sids: BTreeMap::new(),
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
            bound_peer: peer,
        });
        if let Some(p) = peer {
            // `id` is this listener's never-reused key (item 8).
            if let Some(b) = st.broker.as_mut() {
                let key = rx_facts.and_then(|f| Inner::key_of(b, &f));
                let ev = key.and_then(|k| b.ledger.bind(BROKER, k, id, p).ok().flatten());
                if let Some(ev) = ev {
                    self.inner.dispatch(&mut st, vec![ev]);
                }
            } else {
                self.inner.send_ctl(
                    &mut st,
                    None,
                    &Ctl::Bind {
                        rx: rx_value,
                        rx_id: id,
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
        // Mark the carried ends in transit: pump starts no new reads on
        // them, but a carried sender keeps draining its outbox so its
        // queued messages are written before it moves (its leftover
        // credit-read is cancelled in advance_carry once drained, not
        // here, so the move never cancels a pending write: item 4, W3).
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
        // A sandboxed launch must have a SID to key its channels and to
        // enforce one container per child; without one, refuse before
        // spawning (fail closed).
        let sid = match spec.sandbox {
            Some((c, _, _)) => Some(
                sys::sid_string(c)
                    .ok_or_else(|| std::io::Error::other("no AppContainer SID string"))?,
            ),
            None => None,
        };
        // Reserve the SID under the lock before spawning, so a duplicate
        // child never runs even briefly in a shared container.
        if let Some(s) = &sid {
            let mut st = self.inner.lock();
            let Some(b) = st.broker.as_mut() else {
                return Err(std::io::Error::other("only a broker launches"));
            };
            if b.sid_in_use(s) {
                return Err(std::io::Error::other(
                    "the AppContainer SID is already in use",
                ));
            }
            b.reserved_sids.insert(s.clone());
        }
        // From here, every exit must release the reservation.
        let release = |inner: &Inner| {
            if let Some(s) = &sid
                && let Some(b) = inner.lock().broker.as_mut()
            {
                b.reserved_sids.remove(s);
            }
        };
        let child = match sys::spawn::start(&sys::spawn::Spec {
            exe: spec.exe,
            args: spec.args,
            env: &env,
            inherit: &inherit,
            sandbox: spec.sandbox,
        }) {
            Ok(c) => c,
            Err(e) => {
                release(&self.inner);
                return Err(e);
            }
        };
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
                release(&self.inner);
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
                release(&self.inner);
                sys::spawn::kill(&child);
                return Err(e);
            }
        };
        let mut st = self.inner.lock();
        let Some(b) = st.broker.as_mut() else {
            drop(st);
            release(&self.inner);
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
                b.sweep();
                if let Some(s) = &sid {
                    b.reserved_sids.remove(s);
                }
            }
            drop(st);
            sys::spawn::kill(&child);
            return Err(std::io::Error::other("no room for the link"));
        };
        if let Some(b) = st.broker.as_mut() {
            if let Some(s) = &sid {
                b.reserved_sids.remove(s);
            }
            if let Some(s) = &sid {
                b.creator_sids.insert(s.clone(), p);
            }
            b.children.insert(
                p,
                ChildEntry {
                    process,
                    sid,
                    link,
                    relays: 0,
                },
            );
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
