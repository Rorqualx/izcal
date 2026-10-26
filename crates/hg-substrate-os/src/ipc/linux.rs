//! The Linux IPC transport (Stage 1 4.2; M1 phase D3): channels are
//! `AF_UNIX` `SOCK_SEQPACKET` socket pairs, handles cross as `SCM_RIGHTS`
//! descriptors. Written from unix(7), cmsg(3), socket(7), sendmsg(2),
//! recvmsg(2), epoll(7) and eventfd(2); no other project's code is a
//! source.
//!
//! **Shape.** A channel's data flows from the sender end to the receiver
//! end; the reverse direction carries only credit records, so a sender
//! knows exactly how many of its messages are undelivered (`Full` at the
//! channel's capacity, as in the simulator). A message that finds the
//! kernel buffer full below that capacity waits in the sender end's
//! outbox and the IO thread sends it when the socket is writable: `send`
//! never blocks and never reports `Full` early. One IO thread per process
//! waits on epoll with no timeout (no idle wakeups, FP-18), reads
//! records, has the pure core (`ipc::unix::decide`) judge each one, and
//! queues what passes; each listening channel keeps at most one delivery
//! task on its sequence, which delivers one item and posts itself again.
//!
//! **What the kernel attests and what the transport checks**
//! (security scope IPC-R1, R2, R5; SEC-4):
//! - every received descriptor must be a Unix seqpacket socket above the
//!   stdio numbers, not one this process already holds, and not twice in
//!   one record (`ipc::unix::accept`); anything else refuses the record;
//! - every record carries the sender's kernel-attested process ID
//!   (`SO_PASSCRED` on every receiving end); a violation names that
//!   process, so its owner can terminate it;
//! - a bound sender is accepted only from the process that created its
//!   socket pair (`SO_PEERCRED`, fixed for the socket's life), and a
//!   receiver listened to with `listen_bound` (every bound channel's,
//!   and the launch channels') accepts records from the one process it
//!   names; a record from another is blamed on that named peer, since
//!   only it could have passed its sender on;
//! - which end of a pair a descriptor is cannot be attested; mislabelling
//!   only lets a peer talk to this process over a channel it controls at
//!   both ends, and the credit path installs no descriptors at all.
//!
//! **Measured on Linux 6.8 before relying on it** (D3 probes, hueyj,
//! 24 October 2026): credentials are attached without the sender's help
//! and name the sending process; `SO_PEERCRED` on a socket pair end held
//! elsewhere names the pair's creator; a 64 KiB + 512 B record fits one
//! send with a 256 KiB† send buffer (the kernel stores 425 984 bytes);
//! 555 credit records fit the reverse direction, so credits coalesce
//! above that; an empty record reads as 0 once, then `EAGAIN`, while end
//! of file reads 0 every time; a close with unread data makes the peer's
//! next read report `ECONNRESET` once, after which every queued record is
//! still read, then end of file; too few free descriptor slots installs
//! some descriptors and sets `MSG_CTRUNC`; more than `RLIMIT_NOFILE`
//! descriptors in flight fails with `ETOOMANYREFS`.
//!
//! **`Closed` is eventual.** A send succeeds while any process holds the
//! receiving socket open. While another thread of this process is
//! starting a child, that child holds a copy of every descriptor until
//! it execs (posix_spawn(3), fork(2)), so an end this process has just
//! dropped can take a message that is then lost, and only the next send
//! reports `Closed`. Measured on hueyj, 25 October 2026: a send right
//! after dropping the receiver was accepted in 6 of 40 runs beside the
//! tests that start children, and 0 of 200 without them. `Ok` from
//! `send` therefore means accepted, never delivered, which is also all
//! it can mean across processes; a process that starts children while
//! dropping ends (the broker) is phase E's design question.
//!
//! **Lock discipline.** One mutex guards the transport's state. Nothing
//! that may own a handle (a queued message, a sink, a handle object) is
//! dropped with it held, and no sink runs with it held: both lock it
//! again. Such values are moved out and dropped after the lock is
//! released.

pub(crate) mod sys;

use core::any::Any;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::os::fd::{AsFd, OwnedFd, RawFd};
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
    CREDIT_LEN, Credit, HandleDesc, MAX_DATA_RECORD, WireKind, decode_credit, encode_credit,
    encode_data,
};
use crate::ipc::common::flow::{PendingCredit, ReceiverFlow, SenderFlow};
use crate::ipc::unix::accept::FdFacts;
use crate::ipc::unix::decide::{Binding, Decision, Record, decide};
use sys::{Interest, RecvOutcome, SendOutcome};

/// The send buffer each end asks for: room for several maximum records
/// (the kernel doubles it, capped at `wmem_max`).
const SNDBUF_BYTES: usize = 256 * 1024;

/// The capacity of a process's launch channels (Stage 1 4.1): the
/// process-host interfaces' declared capacity.
pub const BOOTSTRAP_CAPACITY: u32 = 64;

/// The launch descriptors the launcher places (phase E): the sender to
/// the parent, then the receiver from it.
pub const BOOTSTRAP_FDS: (RawFd, RawFd) = (3, 4);

/// How long a message carrying a sender end waits for that end's own
/// queued messages to reach the kernel before the transport gives up on
/// them (closes the end for writing, so its new holder sees it closed).
const TRANSFER_DEADLINE: Duration = Duration::from_secs(5);

/// Records read for one channel per wake, so one busy channel cannot
/// starve the others.
const READS_PER_WAKE: usize = 64;

/// Descriptor slots kept free before a read that may install up to 16.
const FD_HEADROOM: usize = MAX_MESSAGE_HANDLES + 8;

/// The epoll token of the wake eventfd; channel ends use their ID, which
/// starts at 1 and is never reused.
const WAKER_TOKEN: u64 = 0;

/// One item for a listener.
enum Item {
    Message(Vec<u8>, Vec<Handle>),
    Closed,
    Violation(Fault),
}

/// A message waiting in a sender end's outbox.
struct Outgoing {
    bytes: Vec<u8>,
    /// The ends this record carries, kept open until it is in the kernel.
    carried: Vec<Arc<OwnedFd>>,
    /// Sender ends inside this record whose own outbox must reach the
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
    /// Nothing more will be read (end of file, a violation, an error).
    ended: bool,
    /// Registered with epoll and armed.
    armed: bool,
    /// For a bound receiver, the one process whose records it accepts
    /// ([`Ipc::listen_bound`]); `None` for a free one.
    peer: Option<i32>,
    credit: PendingCredit,
}

/// What an end is.
enum Role {
    Sender {
        flow: SenderFlow,
        bound: bool,
        /// This process created the pair (it may grant a bound sender).
        made_here: bool,
        /// A bound sender was already sent once.
        granted: bool,
        outbox: VecDeque<Outgoing>,
        /// Its handle was dropped; it stays until its outbox drains.
        dropped: bool,
        /// Inside a record waiting in another end's outbox.
        in_transit: bool,
        /// Writes failed or the peer is gone.
        broken: bool,
    },
    Receiver {
        flow: ReceiverFlow,
        bound: bool,
        listener: Option<Listener>,
    },
}

struct End {
    fd: Arc<OwnedFd>,
    dev: u64,
    ino: u64,
    role: Role,
    /// Registered with epoll (only then may it be modified or deleted).
    registered: bool,
}

#[derive(Default)]
struct State {
    ends: BTreeMap<u64, End>,
    next: u64,
    /// `(st_dev, st_ino)` of every socket this process holds as an end.
    held: BTreeSet<(u64, u64)>,
    /// Listeners whose reads wait for free descriptor slots; re-armed
    /// when an end is released.
    fd_waiters: BTreeSet<u64>,
    stopping: bool,
}

/// The transport's shared core, owned by the transport, its handles and
/// its IO thread.
pub(crate) struct Inner {
    exec: Arc<OsExecutor>,
    epoll: sys::Epoll,
    waker: sys::Waker,
    state: Mutex<State>,
    pid: i32,
    /// Open descriptors when the transport started, for the headroom.
    baseline_fds: usize,
    boot: Mutex<Option<(OwnedFd, OwnedFd)>>,
}

impl Inner {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Registers an end; returns its ID.
    fn insert(&self, st: &mut State, fd: OwnedFd, role: Role) -> Option<u64> {
        let facts = sys::fd_facts(fd.as_fd()).ok()?;
        st.next = st.next.checked_add(1)?;
        let id = st.next;
        st.held.insert((facts.dev, facts.ino));
        st.ends.insert(
            id,
            End {
                fd: Arc::new(fd),
                dev: facts.dev,
                ino: facts.ino,
                role,
                registered: false,
            },
        );
        Some(id)
    }

    /// Removes an end: out of epoll first (epoll(7) Q6), then out of the
    /// registry. Returns what must be dropped after the lock is released.
    fn remove(&self, st: &mut State, id: u64) -> Option<End> {
        let end = st.ends.remove(&id)?;
        if end.registered {
            let _ = self.epoll.delete(end.fd.as_fd());
        }
        st.held.remove(&(end.dev, end.ino));
        st.fd_waiters.remove(&id);
        if !st.fd_waiters.is_empty() && self.fd_room(st) {
            let waiting: Vec<u64> = std::mem::take(&mut st.fd_waiters).into_iter().collect();
            for w in waiting {
                self.arm(st, w);
            }
        }
        Some(end)
    }

    /// Arms an end's epoll registration for what it currently waits on.
    fn arm(&self, st: &mut State, id: u64) {
        let Some(end) = st.ends.get_mut(&id) else {
            return;
        };
        let (read, write) = match &mut end.role {
            Role::Sender { outbox, broken, .. } => (false, !outbox.is_empty() && !*broken),
            Role::Receiver {
                flow,
                listener: Some(l),
                ..
            } => {
                let read = !l.ended && !flow.paused();
                l.armed = read;
                (read, l.credit.is_pending())
            }
            Role::Receiver { listener: None, .. } => (false, false),
        };
        if !read && !write {
            return;
        }
        let interest = Interest::end(read, write);
        let fd = end.fd.as_fd();
        let ok = if end.registered {
            self.epoll.modify(fd, interest, id).is_ok()
        } else {
            self.epoll.add(fd, interest, id).is_ok()
        };
        end.registered = end.registered || ok;
    }

    /// Posts `id`'s delivery task if it has items and none is posted.
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

    /// The headroom check before a read that may install descriptors:
    /// the descriptors open when the transport started, plus every end it
    /// holds and every descriptor waiting in an outbox, plus room for one
    /// record's worth, must fit the soft `RLIMIT_NOFILE`. An estimate
    /// (other code in the process opens descriptors too); a read that
    /// still runs short gets `MSG_CTRUNC`, which ends the channel without
    /// blame.
    fn fd_room(&self, st: &State) -> bool {
        let carried: usize = st
            .ends
            .values()
            .map(|e| match &e.role {
                Role::Sender { outbox, .. } => outbox.iter().map(|o| o.carried.len()).sum(),
                Role::Receiver { .. } => 0,
            })
            .sum();
        let used = self
            .baseline_fds
            .saturating_add(st.ends.len())
            .saturating_add(carried)
            .saturating_add(FD_HEADROOM);
        used <= sys::fd_limit()
    }

    /// One delivery task: the next item of listener `id`.
    fn deliver(self: &Arc<Self>, id: u64) {
        let mut st = self.lock();
        let Some(End {
            fd,
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
        let Some(item) = l.queue.pop_front() else {
            return;
        };
        let sink = Arc::clone(&l.sink);
        let was_paused = flow.paused();
        let mut final_item = false;
        let delivery = match item {
            Item::Message(bytes, handles) => {
                // The credit goes back before the sink runs: a send that
                // happens after this delivery started sees the slot free.
                if let Ok(c) = flow.on_deliver()
                    && !send_credit(fd, c)
                {
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
        };
        if was_paused || st.ends.get(&id).is_some_and(|e| matches!(&e.role, Role::Receiver { listener: Some(l), .. } if l.credit.is_pending())) {
            self.arm(&mut st, id);
        }
        drop(st);
        let flow_after = sink.lock().unwrap_or_else(PoisonError::into_inner)(delivery);
        let mut st = self.lock();
        let gone = if flow_after == Flow::Close || final_item {
            self.remove(&mut st, id)
        } else {
            self.kick(&mut st, id);
            None
        };
        drop(st);
        drop(gone);
    }

    /// Releases an end whose handle was dropped.
    fn release(&self, id: u64) {
        let mut st = self.lock();
        let keep = match st.ends.get_mut(&id).map(|e| &mut e.role) {
            Some(Role::Sender {
                outbox,
                dropped,
                broken,
                ..
            }) if !outbox.is_empty() && !*broken => {
                // Its queued messages still go out, as in the simulator.
                *dropped = true;
                true
            }
            _ => false,
        };
        let gone = if keep { None } else { self.remove(&mut st, id) };
        drop(st);
        drop(gone);
    }

    /// Reads what `id` has, judges each record, and queues the results.
    fn read_ready(self: &Arc<Self>, id: u64, buf: &mut [u8]) {
        let mut taken: Vec<Box<dyn Any + Send>> = Vec::new();
        for _ in 0..READS_PER_WAKE {
            let mut st = self.lock();
            let Some(End {
                fd,
                role:
                    Role::Receiver {
                        flow,
                        listener: Some(l),
                        ..
                    },
                ..
            }) = st.ends.get(&id)
            else {
                return;
            };
            if l.ended || flow.paused() {
                break;
            }
            let fd = Arc::clone(fd);
            if !self.fd_room(&st) {
                // Too few descriptor slots to read safely: wait until an
                // end is released (`remove` re-arms the waiters).
                if let Some(End {
                    role:
                        Role::Receiver {
                            listener: Some(l), ..
                        },
                    ..
                }) = st.ends.get_mut(&id)
                {
                    l.armed = false;
                }
                st.fd_waiters.insert(id);
                return;
            }
            drop(st);
            let outcome = sys::recv_with_rights(fd.as_fd(), buf);
            let sys::WithRights {
                rec,
                control: parsed,
                owned,
            } = match outcome {
                RecvOutcome::WouldBlock => break,
                RecvOutcome::Reset => {
                    let _ = sys::take_error(fd.as_fd());
                    continue;
                }
                RecvOutcome::Failed(_) => {
                    self.end_listener(id, Item::Closed);
                    return;
                }
                RecvOutcome::Record(r) => r,
            };
            // `owned` holds every descriptor the kernel installed, so each
            // closes exactly once whatever the verdict.
            let mut owned = owned;
            // An empty record and end of file both read as 0: end of file
            // reads 0 again, an empty record does not.
            let mut hang_up = false;
            if rec.len == 0 && parsed.fd_count() == 0 {
                let mut probe = [0u8; 1];
                hang_up = matches!(
                    sys::recv_record(fd.as_fd(), &mut probe),
                    RecvOutcome::Record(r) if r.len == 0
                );
            }
            let facts: Vec<FdFacts> = parsed
                .fds()
                .iter()
                .filter_map(|raw| owned.get(raw).and_then(|o| sys::fd_facts(o.as_fd()).ok()))
                .collect();
            let bytes = buf.get(..rec.len).unwrap_or(&[]);
            let record = Record {
                bytes,
                hang_up,
                truncated: rec.truncated,
                ctrl_truncated: rec.control_truncated,
                control: &parsed,
                facts: &facts,
            };
            let mut st = self.lock();
            let held = st.held.clone();
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
                drop(st);
                drop(owned);
                return;
            };
            let binding = Binding { peer: l.peer };
            let verdict = decide(&record, binding, flow, &|dev: u64, ino: u64| {
                held.contains(&(dev, ino))
            });
            match verdict.decision {
                Decision::Accept { view, .. } => {
                    let kinds: Vec<HandleKind> = verdict.decision.kinds().to_vec();
                    let payload = view.payload.to_vec();
                    let descs: Vec<HandleDesc> = view.descs().to_vec();
                    let mut handles = Vec::with_capacity(kinds.len());
                    for ((raw, kind), desc) in parsed.fds().iter().zip(&kinds).zip(&descs) {
                        let Some(fd) = owned.remove(raw) else {
                            continue;
                        };
                        let role = match kind {
                            HandleKind::ChannelSender | HandleKind::BoundChannelSender => {
                                let Ok(flow) = SenderFlow::from_transfer(desc) else {
                                    continue;
                                };
                                Role::Sender {
                                    flow,
                                    bound: *kind == HandleKind::BoundChannelSender,
                                    made_here: false,
                                    granted: false,
                                    outbox: VecDeque::new(),
                                    dropped: false,
                                    in_transit: false,
                                    broken: false,
                                }
                            }
                            // The accept step makes no other kind (regions join in
                            // D6's step for this OS).
                            HandleKind::RegionWriter
                            | HandleKind::ReadOnlyRegion
                            | HandleKind::ConsumerRegion
                            | HandleKind::ProducerRegion
                            | HandleKind::GpuIoSurface
                            | HandleKind::GpuDmaBuf
                            | HandleKind::GpuDxgiShared => break,
                            HandleKind::ChannelReceiver => {
                                let Ok(flow) = ReceiverFlow::from_transfer(desc) else {
                                    continue;
                                };
                                // A received receiver keeps its credentials
                                // option on (it was set when made).
                                let _ = sys::set_passcred(fd.as_fd());
                                Role::Receiver {
                                    flow,
                                    bound: false,
                                    listener: None,
                                }
                            }
                        };
                        let _ = sys::set_sndbuf(fd.as_fd(), SNDBUF_BYTES);
                        if let Some(hid) = self.insert(&mut st, fd, role) {
                            handles.push(Handle::new(Box::new(LinuxEnd {
                                inner: Arc::clone(self),
                                id: hid,
                                kind: *kind,
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
                    }) = st.ends.get_mut(&id)
                    {
                        l.queue.push_back(Item::Message(payload, handles));
                    }
                    self.kick(&mut st, id);
                }
                Decision::Violation { .. } => {
                    let fault = verdict.decision.fault();
                    drop(st);
                    self.end_listener(id, fault.map_or(Item::Closed, Item::Violation));
                    taken.push(Box::new(owned));
                    return;
                }
                Decision::Closed => {
                    drop(st);
                    self.end_listener(id, Item::Closed);
                    taken.push(Box::new(owned));
                    return;
                }
            }
            drop(st);
            taken.push(Box::new(owned));
        }
        let mut st = self.lock();
        self.arm(&mut st, id);
        drop(st);
        drop(taken);
    }

    /// Stops reading `id` and queues its last item.
    fn end_listener(self: &Arc<Self>, id: u64, last: Item) {
        let mut st = self.lock();
        if let Some(End {
            fd,
            registered,
            role: Role::Receiver {
                listener: Some(l), ..
            },
            ..
        }) = st.ends.get_mut(&id)
            && !l.ended
        {
            l.ended = true;
            l.armed = false;
            if *registered {
                let _ = self.epoll.delete(fd.as_fd());
                *registered = false;
            }
            l.queue.push_back(last);
        }
        self.kick(&mut st, id);
    }

    /// Retries a listener's pending credit.
    fn write_credit(&self, id: u64) {
        let mut st = self.lock();
        if let Some(End {
            fd,
            role: Role::Receiver {
                listener: Some(l), ..
            },
            ..
        }) = st.ends.get_mut(&id)
            && let Some(c) = l.credit.take()
            && !send_credit(fd, c)
        {
            l.credit.restore(c);
        }
        self.arm(&mut st, id);
    }

    /// Sends what sender ends' outboxes hold, in order, as far as the
    /// kernel and transfer order allow. Returns the earliest deadline of
    /// a record still waiting on a transfer.
    fn flush_all(&self) -> Option<StdInstant> {
        let mut st = self.lock();
        let ids: Vec<u64> = st
            .ends
            .iter()
            .filter(|(_, e)| matches!(&e.role, Role::Sender { outbox, .. } if !outbox.is_empty()))
            .map(|(id, _)| *id)
            .collect();
        let mut deadline: Option<StdInstant> = None;
        let mut dropped_now = Vec::new();
        let mut gone = Vec::new();
        // Each pass can unblock a record in another outbox; the number of
        // outboxes bounds the passes.
        for _ in 0..=ids.len() {
            let mut progressed = false;
            for &id in &ids {
                progressed |= self.flush_one(&mut st, id, &mut deadline, &mut dropped_now);
            }
            if !progressed {
                break;
            }
        }
        for &id in &ids {
            let done = matches!(
                st.ends.get(&id).map(|e| &e.role),
                Some(Role::Sender { outbox, dropped: true, .. }) if outbox.is_empty()
            ) || matches!(
                st.ends.get(&id).map(|e| &e.role),
                Some(Role::Sender {
                    dropped: true,
                    broken: true,
                    ..
                })
            );
            if done {
                gone.push(self.remove(&mut st, id));
            } else {
                self.arm(&mut st, id);
            }
        }
        drop(st);
        drop(dropped_now);
        drop(gone);
        deadline
    }

    /// Flushes one outbox; returns whether anything was sent.
    fn flush_one(
        &self,
        st: &mut State,
        id: u64,
        deadline: &mut Option<StdInstant>,
        out: &mut Vec<Outgoing>,
    ) -> bool {
        let mut progressed = false;
        loop {
            let Some(End {
                fd,
                role: Role::Sender { outbox, broken, .. },
                ..
            }) = st.ends.get(&id)
            else {
                return progressed;
            };
            if *broken {
                return progressed;
            }
            let Some(head) = outbox.front() else {
                return progressed;
            };
            let fd = Arc::clone(fd);
            let since = head.since;
            let after = head.after.clone();
            // Wait for carried senders' own queued records.
            let waiting: Vec<u64> = after
                .iter()
                .copied()
                .filter(|dep| {
                    matches!(st.ends.get(dep).map(|e| &e.role),
                        Some(Role::Sender { outbox, broken: false, .. }) if !outbox.is_empty())
                })
                .collect();
            if !waiting.is_empty() {
                let due = since.checked_add(TRANSFER_DEADLINE).unwrap_or(since);
                if StdInstant::now() < due {
                    *deadline = Some(deadline.map_or(due, |d| d.min(due)));
                    return progressed;
                }
                // Deadline: the carried ends' queued records are dropped
                // and the ends closed for writing; their new holder sees
                // them closed. Recorded through the end's state.
                for dep in waiting {
                    if let Some(End {
                        fd,
                        role: Role::Sender { outbox, broken, .. },
                        ..
                    }) = st.ends.get_mut(&dep)
                    {
                        let _ = sys::shutdown_write(fd.as_fd());
                        out.extend(outbox.drain(..));
                        *broken = true;
                    }
                }
            }
            let Some(End {
                role: Role::Sender { outbox, .. },
                ..
            }) = st.ends.get(&id)
            else {
                return progressed;
            };
            let Some(head) = outbox.front() else {
                return progressed;
            };
            let carried: Vec<std::os::fd::BorrowedFd<'_>> =
                head.carried.iter().map(|c| c.as_fd()).collect();
            let outcome = sys::send_record(fd.as_fd(), &head.bytes, &carried);
            drop(carried);
            match outcome {
                SendOutcome::Sent => {
                    if let Some(End {
                        role: Role::Sender { outbox, .. },
                        ..
                    }) = st.ends.get_mut(&id)
                        && let Some(sent) = outbox.pop_front()
                    {
                        out.push(sent);
                    }
                    progressed = true;
                }
                SendOutcome::WouldBlock => return progressed,
                SendOutcome::PeerGone | SendOutcome::Failed(_) => {
                    if let Some(End {
                        role: Role::Sender { outbox, broken, .. },
                        ..
                    }) = st.ends.get_mut(&id)
                    {
                        out.extend(outbox.drain(..));
                        *broken = true;
                    }
                    return progressed;
                }
            }
        }
    }

    /// The IO thread.
    fn io_loop(self: &Arc<Self>) {
        let mut ready = Vec::new();
        let mut buf = vec![0u8; MAX_DATA_RECORD];
        let mut deadline: Option<StdInstant> = None;
        loop {
            let timeout = deadline.map(|d| {
                let left = d.saturating_duration_since(StdInstant::now());
                u32::try_from(left.as_millis()).unwrap_or(u32::MAX).max(1)
            });
            if self.epoll.wait(&mut ready, 64, timeout).is_err() {
                return;
            }
            let mut flush = deadline.is_some();
            for r in &ready {
                if r.token == WAKER_TOKEN {
                    self.waker.drain();
                    if self.lock().stopping {
                        return;
                    }
                    flush = true;
                    continue;
                }
                let is_receiver = matches!(
                    self.lock().ends.get(&r.token).map(|e| &e.role),
                    Some(Role::Receiver { .. })
                );
                if is_receiver {
                    if r.output {
                        self.write_credit(r.token);
                    }
                    if r.input || r.hangup {
                        self.read_ready(r.token, &mut buf);
                    }
                } else if r.output || r.hangup {
                    flush = true;
                }
            }
            if flush {
                deadline = self.flush_all();
            }
        }
    }
}

/// Sends a credit record on a receiver end. False if it must wait.
fn send_credit(fd: &Arc<OwnedFd>, c: Credit) -> bool {
    let bytes = encode_credit(c);
    !matches!(
        sys::send_record(fd.as_fd(), &bytes, &[]),
        SendOutcome::WouldBlock
    )
}

/// A channel end this transport made or received.
struct LinuxEnd {
    inner: Arc<Inner>,
    id: u64,
    kind: HandleKind,
    /// False once the end became a listener or was sent: then its drop
    /// closes nothing.
    armed: bool,
}

impl core::fmt::Debug for LinuxEnd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LinuxEnd")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .finish()
    }
}

impl HandleObject for LinuxEnd {
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

impl Drop for LinuxEnd {
    fn drop(&mut self) {
        if self.armed {
            self.inner.release(self.id);
        }
    }
}

/// A listener; dropping it closes the receiver.
struct LinuxListening {
    inner: Arc<Inner>,
    id: u64,
}

impl core::fmt::Debug for LinuxListening {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LinuxListening")
            .field("id", &self.id)
            .finish()
    }
}

impl HandleObject for LinuxListening {
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

impl Drop for LinuxListening {
    fn drop(&mut self) {
        let mut st = self.inner.lock();
        let gone = self.inner.remove(&mut st, self.id);
        drop(st);
        drop(gone);
    }
}

/// The Linux transport of one process.
pub(crate) struct LinuxIpc {
    inner: Arc<Inner>,
}

impl core::fmt::Debug for LinuxIpc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LinuxIpc")
            .field("pid", &self.inner.pid)
            .finish()
    }
}

/// Why the transport could not start.
#[derive(Debug)]
pub enum StartError {
    /// epoll, the eventfd or the IO thread.
    Os(std::io::Error),
    /// A launch descriptor is missing or not a channel end.
    Bootstrap,
}

impl LinuxIpc {
    /// Starts the transport and its IO thread. With `bootstrap`, the
    /// process's launch channel ends (to the parent, from the parent).
    pub(crate) fn start(
        exec: Arc<OsExecutor>,
        bootstrap: Option<(OwnedFd, OwnedFd)>,
    ) -> Result<Self, StartError> {
        let epoll = sys::Epoll::new().map_err(StartError::Os)?;
        let waker = sys::Waker::new().map_err(StartError::Os)?;
        epoll
            .add(waker.fd(), Interest::WAKER, WAKER_TOKEN)
            .map_err(StartError::Os)?;
        let limit = sys::fd_limit();
        let inner = Arc::new(Inner {
            exec,
            epoll,
            waker,
            state: Mutex::new(State::default()),
            pid: i32::try_from(std::process::id()).unwrap_or(0),
            baseline_fds: sys::count_open(limit.min(4096)),
            boot: Mutex::new(bootstrap),
        });
        let io = Arc::clone(&inner);
        crate::thread::spawn("hg-ipc-io", move || io.io_loop()).map_err(StartError::Os)?;
        Ok(Self { inner })
    }

    /// The end `h` names, if it is this transport's and held here.
    fn held_end(&self, h: &Handle) -> Option<u64> {
        let e = h.object().downcast_ref::<LinuxEnd>()?;
        if !Arc::ptr_eq(&e.inner, &self.inner) {
            return None;
        }
        self.inner.lock().ends.contains_key(&e.id).then_some(e.id)
    }

    fn handle(&self, id: u64, kind: HandleKind) -> Handle {
        Handle::new(Box::new(LinuxEnd {
            inner: Arc::clone(&self.inner),
            id,
            kind,
            armed: true,
        }))
    }
}

impl Drop for LinuxIpc {
    fn drop(&mut self) {
        self.inner.lock().stopping = true;
        self.inner.waker.wake();
    }
}

/// Disarms a handle object that became something else (sent, listened).
fn disarm(h: Handle) {
    if let Ok(mut e) = h.into_object().downcast::<LinuxEnd>() {
        e.armed = false;
    }
}

impl Ipc for LinuxIpc {
    fn channel(
        &self,
        capacity: u32,
        transfer: Transfer,
    ) -> Result<(ChannelSender, ChannelReceiver), IpcError> {
        let (a, b) = sys::socketpair().map_err(|_e| IpcError::Closed)?;
        // Credentials on the receiving end before either end can leave
        // this process; the send buffers sized for maximum records.
        sys::set_passcred(b.as_fd()).map_err(|_e| IpcError::Closed)?;
        let _ = sys::set_sndbuf(a.as_fd(), SNDBUF_BYTES);
        let _ = sys::set_sndbuf(b.as_fd(), SNDBUF_BYTES);
        let bound = transfer == Transfer::Bound;
        let mut st = self.inner.lock();
        let tx = self.inner.insert(
            &mut st,
            a,
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
        let rx = self.inner.insert(
            &mut st,
            b,
            Role::Receiver {
                flow: ReceiverFlow::new(capacity),
                bound,
                listener: None,
            },
        );
        drop(st);
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
        let fd = match st.ends.get(&to_id) {
            Some(End {
                fd,
                role: Role::Sender { broken: false, .. },
                ..
            }) => Arc::clone(fd),
            Some(End {
                role: Role::Sender { .. },
                ..
            }) => return Err(IpcError::Closed),
            _ => return Err(IpcError::Refused),
        };
        if drain_credits(&mut st, to_id, &fd).is_err() {
            if let Some(End {
                role: Role::Sender { broken, .. },
                ..
            }) = st.ends.get_mut(&to_id)
            {
                *broken = true;
            }
            return Err(IpcError::Closed);
        }
        // Every carried handle: a bound receiver never moves; a bound
        // sender moves once, from the process that made it (IPC-R2 (a)).
        let mut descs = Vec::with_capacity(ids.len());
        let mut after = Vec::new();
        for (&id, h) in ids.iter().zip(&handles) {
            let Some(end) = st.ends.get(&id) else {
                return Err(IpcError::Refused);
            };
            let d = match &end.role {
                Role::Receiver { bound: true, .. }
                | Role::Receiver {
                    listener: Some(_), ..
                } => {
                    return Err(IpcError::Refused);
                }
                Role::Receiver { flow, .. } => flow.to_desc(),
                Role::Sender {
                    bound,
                    made_here,
                    granted,
                    flow,
                    outbox,
                    broken,
                    ..
                } => {
                    if *bound && (!*made_here || *granted) {
                        return Err(IpcError::Refused);
                    }
                    if *broken {
                        return Err(IpcError::Closed);
                    }
                    if !outbox.is_empty() {
                        after.push(id);
                    }
                    let kind = if h.kind() == HandleKind::BoundChannelSender {
                        WireKind::BoundSender
                    } else {
                        WireKind::Sender
                    };
                    flow.to_desc(kind)
                }
            };
            descs.push(d);
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
        let Ok(record) = encode_data(seq, &descs, &bytes) else {
            return Err(IpcError::TooLarge);
        };
        let queue_first = !outbox.is_empty() || !after.is_empty();
        // The carried ends: kept open until the record is in the kernel,
        // then this process's descriptors close (the kernel holds its
        // own references while the record is in flight).
        let carried: Vec<Arc<OwnedFd>> = ids
            .iter()
            .filter_map(|id| st.ends.get(id).map(|e| Arc::clone(&e.fd)))
            .collect();
        let outcome = if queue_first {
            SendOutcome::WouldBlock
        } else {
            let borrowed: Vec<std::os::fd::BorrowedFd<'_>> =
                carried.iter().map(|c| c.as_fd()).collect();
            sys::send_record(fd.as_fd(), &record, &borrowed)
        };
        match outcome {
            SendOutcome::Sent | SendOutcome::WouldBlock => {}
            SendOutcome::PeerGone => return Err(IpcError::Closed),
            // A shrunk buffer or too many descriptors in flight: the
            // channel cannot carry this; recorded as closed.
            SendOutcome::Failed(_) => return Err(IpcError::Closed),
        }
        // Committed: count it, mark grants, retire the carried ends.
        let mut carried_out = Vec::new();
        if let Some(End {
            role: Role::Sender { flow, outbox, .. },
            ..
        }) = st.ends.get_mut(&to_id)
        {
            *flow = next;
            if outcome == SendOutcome::WouldBlock {
                outbox.push_back(Outgoing {
                    bytes: record,
                    carried: carried.clone(),
                    after: after.clone(),
                    since: StdInstant::now(),
                });
            }
        }
        for &id in &ids {
            let waits = after.contains(&id);
            if let Some(End {
                role:
                    Role::Sender {
                        granted,
                        in_transit,
                        ..
                    },
                ..
            }) = st.ends.get_mut(&id)
            {
                *granted = true;
                *in_transit = waits;
            }
            if !waits {
                carried_out.push(self.inner.remove(&mut st, id));
            } else {
                // Its own outbox still drains; it is released after.
                if let Some(End {
                    role: Role::Sender { dropped, .. },
                    ..
                }) = st.ends.get_mut(&id)
                {
                    *dropped = true;
                }
            }
        }
        if outcome == SendOutcome::WouldBlock {
            self.inner.arm(&mut st, to_id);
        }
        drop(st);
        for h in handles {
            disarm(h);
        }
        drop(carried_out);
        drop(carried);
        if outcome == SendOutcome::WouldBlock {
            self.inner.waker.wake();
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
        // A pid the kernel can never attest binds to no one.
        let Some(pid) = i32::try_from(peer.pid()).ok().filter(|&p| p > 0) else {
            return Err(IpcError::Refused);
        };
        self.start_listening(from, Some(pid), seq, sink)
    }

    fn bootstrap(&self) -> Option<Bootstrap> {
        let (up, down) = self
            .inner
            .boot
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()?;
        let mut st = self.inner.lock();
        let tx = self.inner.insert(
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
        let rx = self.inner.insert(
            &mut st,
            down,
            Role::Receiver {
                flow: ReceiverFlow::new(BOOTSTRAP_CAPACITY),
                bound: false,
                listener: None,
            },
        );
        drop(st);
        let to_parent =
            ChannelSender::from_handle(self.handle(tx?, HandleKind::ChannelSender)).ok()?;
        let from_parent =
            ChannelReceiver::from_handle(self.handle(rx?, HandleKind::ChannelReceiver)).ok()?;
        // The launching process. Until phase E's spawner sits between
        // the broker and its children, it is this process's parent.
        let parent = OsPeer::new(std::os::unix::process::parent_id(), 0);
        Some(Bootstrap {
            to_parent,
            from_parent,
            parent,
        })
    }
}

impl LinuxIpc {
    /// [`Ipc::listen`] and [`Ipc::listen_bound`]: `peer` is the bound
    /// peer's pid, `None` for a free receiver.
    fn start_listening(
        &self,
        from: ChannelReceiver,
        peer: Option<i32>,
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
            role: Role::Receiver {
                bound, listener, ..
            },
            ..
        }) = st.ends.get_mut(&id)
        else {
            return Err(IpcError::Refused);
        };
        // A bound receiver listens only with its peer named.
        if listener.is_some() || (*bound && peer.is_none()) {
            return Err(IpcError::Refused);
        }
        *listener = Some(Listener {
            seq,
            sink: Arc::new(Mutex::new(sink)),
            queue: VecDeque::new(),
            draining: false,
            ended: false,
            armed: false,
            peer,
            credit: PendingCredit::default(),
        });
        self.inner.arm(&mut st, id);
        drop(st);
        disarm(from.into_handle());
        Ok(Listening::new(Handle::new(Box::new(LinuxListening {
            inner: Arc::clone(&self.inner),
            id,
        }))))
    }
}

/// Reads every credit queued on sender end `id`. An error means the
/// peer broke the credit protocol, or a record arrived on the wrong end.
fn drain_credits(st: &mut State, id: u64, fd: &Arc<OwnedFd>) -> Result<(), ()> {
    let mut buf = [0u8; CREDIT_LEN + 1];
    loop {
        // No control buffer: a descriptor sent the wrong way is never
        // installed (the kernel discards it and sets MSG_CTRUNC).
        match sys::recv_record(fd.as_fd(), &mut buf) {
            RecvOutcome::WouldBlock => return Ok(()),
            RecvOutcome::Reset => {
                let _ = sys::take_error(fd.as_fd());
            }
            // End of file: the receiver is gone; the send reports it.
            RecvOutcome::Record(r) if r.len == 0 => return Ok(()),
            RecvOutcome::Record(r) => {
                if r.truncated || r.control_truncated {
                    return Err(());
                }
                let c = decode_credit(buf.get(..r.len).unwrap_or(&[])).map_err(|_e| ())?;
                let Some(End {
                    role: Role::Sender { flow, .. },
                    ..
                }) = st.ends.get_mut(&id)
                else {
                    return Err(());
                };
                flow.on_credit(c).map_err(|_e| ())?;
            }
            RecvOutcome::Failed(_) => return Err(()),
        }
    }
}

/// Checks the launch descriptors a process was started with (the ones
/// its launcher placed at [`BOOTSTRAP_FDS`], which the process's entry
/// point takes ownership of): both must be channel ends (Unix seqpacket
/// sockets above the stdio numbers) and stdio must be open. Marks them
/// close-on-exec. Anything else hands both back, untouched.
///
/// # Errors
/// The two descriptors, unchanged, when they are not launch channels.
pub fn launch_ends(up: OwnedFd, down: OwnedFd) -> Result<(OwnedFd, OwnedFd), (OwnedFd, OwnedFd)> {
    let is_end = |fd: &OwnedFd| {
        sys::fd_facts(fd.as_fd()).is_ok_and(|f| {
            f.fd > 2
                && f.is_socket
                && f.domain == libc::AF_UNIX
                && f.sock_type == libc::SOCK_SEQPACKET
                && f.protocol == 0
        })
    };
    if !(0..=2).all(sys::is_open) || !is_end(&up) || !is_end(&down) {
        return Err((up, down));
    }
    if sys::set_cloexec(up.as_fd()).is_err() || sys::set_cloexec(down.as_fd()).is_err() {
        return Err((up, down));
    }
    let _ = sys::set_passcred(down.as_fd());
    Ok((up, down))
}

/// Makes a pair of launch channels for a child: the parent's ends (from
/// the child, to the child) and the child's (to the parent, from the
/// parent), ready to be placed at [`BOOTSTRAP_FDS`]. For the launcher
/// (phase E) and tests.
pub fn launch_pair() -> std::io::Result<((OwnedFd, OwnedFd), (OwnedFd, OwnedFd))> {
    let (up_tx, up_rx) = sys::socketpair()?;
    let (down_tx, down_rx) = sys::socketpair()?;
    sys::set_passcred(up_rx.as_fd())?;
    sys::set_passcred(down_rx.as_fd())?;
    for fd in [&up_tx, &up_rx, &down_tx, &down_rx] {
        let _ = sys::set_sndbuf(fd.as_fd(), SNDBUF_BYTES);
    }
    Ok(((up_rx, down_tx), (up_tx, down_rx)))
}

/// The parent side of [`launch_pair`], as this transport's ends.
#[cfg_attr(not(test), expect(dead_code, reason = "phase E's launcher calls it"))]
pub(crate) fn adopt_parent_ends(
    ipc: &LinuxIpc,
    from_child: OwnedFd,
    to_child: OwnedFd,
) -> Option<(ChannelReceiver, ChannelSender)> {
    let mut st = ipc.inner.lock();
    let rx = ipc.inner.insert(
        &mut st,
        from_child,
        Role::Receiver {
            flow: ReceiverFlow::new(BOOTSTRAP_CAPACITY),
            bound: false,
            listener: None,
        },
    );
    let tx = ipc.inner.insert(
        &mut st,
        to_child,
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
    drop(st);
    let rx = ChannelReceiver::from_handle(ipc.handle(rx?, HandleKind::ChannelReceiver)).ok()?;
    let tx = ChannelSender::from_handle(ipc.handle(tx?, HandleKind::ChannelSender)).ok()?;
    Some((rx, tx))
}

#[cfg(test)]
mod tests;
