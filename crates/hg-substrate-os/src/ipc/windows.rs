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
//! **A fresh pipe per move (W3).** A process can ask the broker to relay
//! an end while it still has reads pending on it. The broker's
//! `DUPLICATE_CLOSE_SOURCE` pull does not cancel them, the broker cannot
//! cancel another process's reads (probes W8b, W13d), and pending reads
//! on one end complete in the order they were issued, across processes
//! (W10). So after every move the moved end and its peer are on a new
//! pipe the broker makes, and nothing is written to the old pipe after
//! the switch:
//! - the broker peeks the moving end (no close-source), claims its
//!   channel (one relay per channel at a time), and switches the peer
//!   first: in the broker or in a ticket at once; with a child by
//!   `RebindValue`/`RebindId`, waiting for its `Rebound` at most
//!   [`REBIND_DEADLINE`]. Only then does it pull the moving end and
//!   check, with `CompareObjectHandles`, that it is the object peeked;
//! - a receiver reads each old pipe to its end (109) before its current
//!   one, so records keep their order and `seq`. A moving receiver's old
//!   pipe stays with the broker until its next holder listens and asks
//!   for it (`TakeDrains`); a receiver already listening keeps its old
//!   pipe as a local drain. At most 16† old pipes are queued or owed per
//!   channel ([`crate::ipc::win::ledger::MAX_OLD_PIPES`]); the move that
//!   would make the 17th is refused and the channel ends in `Closed`;
//! - every data message ends with its own length (a trailer), so a stale
//!   read that takes only the head of a record leaves a suffix the reader
//!   refuses.
//!
//! The bound: a former holder's pending reads can take records written
//! before the switch, any number, never one after; a whole record taken
//! gives `Violation(Sequence)`, a split one `Malformed`; both
//! unattributed (probe W13). († set from first principles, not
//! measured.)
//!
//! **No relay holds a channel across a query that can wait.** The
//! kernel's queries on a synchronous file object wait behind a pending
//! read (W8), and a child chooses its own pipes' flags. Every query that
//! can wait runs at the peek, on the child's own worker, before the
//! channel is claimed, and an end whose file object does synchronous I/O
//! is refused there (`Refusal::Synchronous`). After the pull only the
//! object peeked is queried again: it is overlapped, its mode cannot be
//! changed, and an overlapped end's queries return at once (W14). A
//! pulled value that names another object is refused unqueried.
//!
//! **Kernel memory.** Each pending read pins about 430 bytes of nonpaged
//! pool, and a job's memory limits do not bound it (W13e: 113 MB from
//! one process). A sandboxed child can do this on any pipe end it holds;
//! there is no Izcal-side cap (a residual, m1-d5 README).
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
//!
//! **Threads.** Every process runs one transport thread, its IO thread.
//! A broker also runs one worker per live child (M8), started when the
//! child is launched and ended when the broker forgets it, and one worker
//! for its own relays, which may wait for a child's `Rebound` (W3): a
//! broker with *n* children runs *n* + 2 transport threads, each with the
//! platform's default stack. A relay that waits for a `Rebound` holds its
//! channels' claims for at most [`REBIND_DEADLINE`]; the holder that is
//! asked answers on its IO thread. Every one of them waits with no timeout when idle, so
//! idle threads cost memory, not wakeups (FP-18). A child's worker
//! handles its relays, redemptions, binds and unbinds one at a time, in
//! the order the child sent them: the pulls and kernel queries run there,
//! off the IO thread and the lock, and the decisions under the lock.
//! Those queries can wait without end on a pipe end that has a
//! synchronous read pending (probe W8). Starting such a read takes a
//! handle to that end, and only the process that holds the end has one
//! (a moved end leaves none behind, `HandleCount`, probe W1), so the
//! request that waits is one about the child's own end, and only that
//! child's later requests wait behind it. A child with more than
//! `MAX_CHILD_REQUESTS` queued has its link dropped.

pub(crate) mod sys;

#[cfg(test)]
mod tests;

use core::any::Any;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::os::windows::io::OwnedHandle;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock, PoisonError, Weak, mpsc};
use std::time::{Duration, Instant as StdInstant};

use hg_substrate::ipc::{
    Bootstrap, ChannelReceiver, ChannelSender, ConsumerRegion, Delivery, Fault, FaultKind, Flow,
    Handle, HandleKind, HandleObject, Ipc, IpcError, Listening, MAX_MESSAGE_BYTES,
    MAX_MESSAGE_HANDLES, Mapping, MappingObject, OsPeer, PeerAttest, ProducerRegion,
    ReadOnlyRegion, RegionObject, RegionRefused, RegionWriter, Sink, Transfer,
};
use hg_substrate::{SequenceHandle, Task};

use crate::executor::OsExecutor;
use crate::ipc::common::attest;
use crate::ipc::common::codec::{
    CREDIT_LEN, Credit, HandleDesc, REGION_FILLER_TRUSTED, REGION_IMMUTABLE, WireKind,
    decode_credit, encode_credit, encode_data,
};
use crate::ipc::common::flow::{PendingCredit, ReceiverFlow, SenderFlow};
use crate::ipc::common::quota::Ledger as Quota;
use crate::ipc::win::accept::{
    End as PipeEnd, Facts, PipeKey, Proc, Refusal, accept, check_value, distinct, end_for,
    parse_name,
};
use crate::ipc::win::decide::{Decision, Reason, check_redeemed, decide};
use crate::ipc::win::frame::{
    Ctl, Item as CtlItem, Items, MAX_CTL_LEN, MAX_PIPE_DATA, NO_TICKET, Ticket, decode_ctl,
    encode_ctl, join_data,
};
use crate::ipc::win::ledger::{BindEvent, Held, Ledger, Where};
use crate::ipc::win::regions::{Grant, Pulled as RegionPulled, State as RState, relay_rule};

/// The capacity of a process's launch channels (Stage 1 4.1): the
/// process-host interfaces' declared capacity.
pub const BOOTSTRAP_CAPACITY: u32 = 64;

/// How long a message carrying a sender end waits for that end's own
/// queued messages before the transport gives up on them.
const TRANSFER_DEADLINE: Duration = Duration::from_secs(5);

/// How long the broker waits for the holder of a moving end's peer to
/// answer a `Rebind` before it refuses the move (W3). The peer's holder
/// is the only party that can stall it, and it can already stall its own
/// channel.
const REBIND_DEADLINE: Duration = Duration::from_secs(5);

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

/// The most requests (relays, redemptions, binds, unbinds) one child may
/// have queued for its worker (M8). Its worker handles one at a time, so
/// the queue grows only while a request of that child's is slow or
/// blocked (a synchronous pipe, probe W8); a child past the cap has its
/// link dropped, as a child past [`MAX_LINK_BACKLOG`] does, so it cannot
/// make the broker buffer without bound.
const MAX_CHILD_REQUESTS: usize = 1024;

/// How often a relay waiting for another relay's copy of the same
/// section (review Q1) looks again. It waits on its own child's worker,
/// with no lock held.
const COPY_WAIT_POLL: Duration = Duration::from_millis(10);

/// The longest a relay waits for another relay's copy before it is
/// refused. A copy is registered as soon as it is made, so this bounds
/// the copy itself: eight 64 MiB copies took about 20 s on rig253.
const COPY_WAIT_MAX: Duration = Duration::from_secs(300);

/// A handle a redemption handed over (`None` if it could not be
/// adopted), its item as the broker sent it (the kind it travels as, a
/// region's length and flags, and whether the broker holds old pipes of
/// its channel for it, W3), and, in the broker, its channel end.
type Got = (Option<OwnedHandle>, CtlItem, Option<(PipeKey, PipeEnd)>);

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
    /// In the broker: the copies this record's regions need (review
    /// Q1) are being made off the lock under this token.
    copying: Option<u64>,
    /// In the broker: those copies, made and waiting for the relay.
    prepared: Option<Vec<Prepared>>,
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
    /// A `TakeDrains` request is out (W3): nothing is read until the old
    /// pipes arrive, so they are read first.
    taking: Option<u64>,
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
        /// W3: old pipes this listener reads to their end, oldest first,
        /// before its current pipe `End::h`.
        drains: VecDeque<Arc<OwnedHandle>>,
        /// W3: the broker holds old pipes of this receiver's channel;
        /// listening takes them first.
        old_queued: bool,
    },
    /// A control link: in a child, to its broker; in the broker, to
    /// child `proc`.
    Link { proc: Proc, out: VecDeque<Vec<u8>> },
    /// A shared-memory region (D6): a section handle with its kind's
    /// access. No I/O ever runs on it.
    Region {
        kind: HandleKind,
        /// The exact length (the broker's registered size).
        len: u32,
        /// Carry it with the `immutable` flag (the IDL attribute).
        want: bool,
        /// Live mappings made from it here (a seal waits for none).
        live: Arc<AtomicUsize>,
    },
}

/// Why copies could not be planned now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Plan {
    /// A charge does not fit: the relay is refused.
    Quota,
    /// Another relay is making a copy this one needs: plan again once it
    /// is done.
    Wait,
}

/// A copy the broker is to make for a relay (review Q1): planned and
/// charged to the owner under the lock, made off it (D5 review M8).
struct CopyJob {
    /// The registry id of the section copied.
    src: u64,
    len: u32,
    owner: OsPeer,
    /// The broker's own handle to the source, to read it off the lock.
    from: OwnedHandle,
}

/// A copy made off the lock, already charged to `owner`, waiting for the
/// relay that asked for it. Unused, it is refunded
/// ([`Inner::release_prepared`]).
struct Prepared {
    src: u64,
    len: u32,
    owner: OsPeer,
    /// The copy's full-access handle (its registry identity).
    master: OwnedHandle,
    /// The read-only handle the receiver gets.
    ro: OwnedHandle,
}

/// A region item's claim and facts, for one decision.
#[derive(Clone, Copy)]
struct RegionFacts {
    kind: WireKind,
    len: u32,
    flags: u8,
    section: sys::SectionFacts,
}

/// What [`Inner::region_rollback`] undoes.
#[derive(Default)]
struct RegionUndo {
    /// Sections whose state moved, with the state before.
    states: Vec<(u64, RState)>,
    /// Copies made.
    /// Copies made: (source, copy).
    copies: Vec<(u64, u64)>,
}

/// A registered section, as the broker knows it (D6 review B1).
struct RegEntry {
    size: u32,
    owner: OsPeer,
    state: RState,
    /// The broker's own full-access handle: identifies the section when
    /// it comes back (fact S3), and is the source of a copy.
    master: OwnedHandle,
    /// The broker's live mappings of the section, whatever handle they
    /// were made from (review finding 3): a writer that comes back to the
    /// broker through a relay is a new table entry, but the same section.
    live: Arc<AtomicUsize>,
    /// Its one `immutable` copy (review Q1), made once, charged once and
    /// reused by every later relay that asks (review finding 1).
    copy: Option<u64>,
    /// That copy is being made, off the lock.
    copying: bool,
}

struct End {
    /// The end's current pipe.
    h: Arc<OwnedHandle>,
    role: Role,
    /// A read is in flight: on `h`, or on a receiver's first drain.
    reading: bool,
    /// A write is in flight on `h`.
    writing: bool,
    /// W3: a sender's previous pipes, each kept only while an operation
    /// on it is still running; each closes once both are done, so its
    /// reader sees the pipe end.
    old: Vec<OldPipe>,
    /// The id in the name of the pipe this process made the channel with
    /// (both ends), so a `RebindId` can name the end. `None` for an end
    /// that came from elsewhere.
    pipe_id: Option<[u8; 16]>,
}

/// A sender's previous pipe after a switch (W3).
struct OldPipe {
    h: Arc<OwnedHandle>,
    /// Its last write is still pending; it completes when the reader
    /// drains the pipe, and is never cancelled (no record is lost).
    writing: bool,
    /// Its credit read is being cancelled.
    reading: bool,
}

/// A child, as its broker knows it.
struct ChildEntry {
    process: sys::Process,
    /// Its AppContainer's SID, which names the namespace its pipes are
    /// created in.
    sid: Option<String>,
    link: u64,
    /// It may hold regions: in an AppContainer or below the broker's
    /// integrity (D6 review finding 5, fact S12); measured at launch.
    regions: bool,
    /// The queue to this child's worker (M8), which handles its relays,
    /// redemptions, binds and unbinds in the order the child sent them.
    /// Dropping it (the child is forgotten) ends the worker once the
    /// queue is drained.
    jobs: mpsc::SyncSender<Job>,
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
    /// Channel ends the broker gave each process, by the end's handle
    /// value there, so a process's `Closed` releases the ledger entry
    /// (items 2, 3). A handle value is reused after its end is closed, so
    /// each value holds a FIFO of the grants made at it, oldest first: a
    /// redeem pushes, a `Closed` pops the front. The control link delivers
    /// a process's records in order, and a grant can only reuse a value
    /// whose end the process already closed, so the front is always the
    /// grant the next `Closed` for that value refers to.
    given: BTreeMap<(Proc, u64), Vec<(PipeKey, PipeEnd)>>,
    /// The broker's own ledgered ends, by their transport id, so it can
    /// release one it closes without re-deriving the channel from the
    /// handle (a client end reports no queryable name, items 2, 3).
    own: BTreeMap<u64, (PipeKey, PipeEnd)>,
    /// Each sandboxed child's container SID to its process number. Kept
    /// after the child dies for as long as the ledger has a channel it
    /// created, so those channels can still be keyed and redeemed by the
    /// live processes that hold their ends (item 5).
    creator_sids: BTreeMap<String, Proc>,
    next_proc: Proc,
    /// Every section the broker made, by registry id (D6).
    regions: BTreeMap<u64, RegEntry>,
    next_region: u64,
    /// Region charges per owner (D6 review M4).
    quota: Quota,
    /// W3: the server ends of old pipes queued for their channels'
    /// receivers, by pipe key. Which channel and in what order is the
    /// ledger's; this holds the handles.
    old: BTreeMap<PipeKey, OwnedHandle>,
    /// W3: replacement ends for channel ends that were in transit when
    /// their peer moved, applied when their own relay pulls them.
    pending: BTreeMap<(PipeKey, PipeEnd), Pending>,
    /// W3: channels with a relay in progress; one relay per channel at a
    /// time, so two ends moving at once settle in turn.
    busy: BTreeSet<PipeKey>,
    /// W3: rebind requests awaiting a `Rebound`: the holder asked, and
    /// where its answer goes.
    waiters: BTreeMap<u64, (Proc, mpsc::SyncSender<(u64, Rebind)>)>,
    next_rebind: u64,
    /// The broker's own relays run on a worker of their own, like a
    /// child's, because a relay may wait for another process's
    /// `Rebound` (W3).
    own_jobs: Option<mpsc::SyncSender<Job>>,
    /// Relays refused, by reason code ([`Refusal::code`]): the broker's
    /// record of why moves failed (a child also gets the code in its
    /// `RelayRefused`).
    refused: BTreeMap<u8, u64>,
}

/// A replacement end the broker made for an end in transit (W3).
struct Pending {
    /// The end on the channel's new pipe, for whoever redeems the end.
    handle: OwnedHandle,
    /// The pipe the end in transit is on; its alias is kept until then.
    old_key: PipeKey,
}

/// What a switch to a new pipe did to an end (W3).
enum Switch {
    /// Not switched: no such end here, or it is on its way elsewhere.
    No,
    /// Switched.
    Done,
    /// Switched an unlistened receiver: its old pipe's server end, for
    /// the broker to keep until the receiver is listened to.
    Park(OwnedHandle),
}

/// A holder's answer to a rebind (W3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rebind {
    /// Its end is on the new pipe; with `park`, its old pipe follows.
    Switched { park: bool },
    /// It no longer holds the end, or the end is on its way elsewhere.
    NotSwitched,
    /// It ended.
    Gone,
}

impl Broker {
    /// A broker with `ledger`, whose next child is process `next_proc`.
    fn new(ledger: Ledger, next_proc: Proc) -> Self {
        Self {
            ledger,
            held: BTreeMap::new(),
            children: BTreeMap::new(),
            reserved_sids: BTreeSet::new(),
            creator_sids: BTreeMap::new(),
            given: BTreeMap::new(),
            own: BTreeMap::new(),
            next_proc,
            old: BTreeMap::new(),
            pending: BTreeMap::new(),
            busy: BTreeSet::new(),
            waiters: BTreeMap::new(),
            next_rebind: 0,
            own_jobs: None,
            refused: BTreeMap::new(),
            regions: BTreeMap::new(),
            next_region: 0,
            quota: Quota::default(),
        }
    }

    /// Records that `who` was given `end` of `pipe` at handle value
    /// `value`, appending to that value's FIFO so a later reuse of the
    /// value queues behind an end still awaiting its `Closed` (items 2,
    /// 3).
    fn grant(&mut self, who: Proc, value: u64, pipe: PipeKey, end: PipeEnd) {
        self.given
            .entry((who, value))
            .or_default()
            .push((pipe, end));
    }

    /// Counts a refused relay by its reason.
    fn count(&mut self, r: Refusal) {
        let n = self.refused.entry(r.code()).or_insert(0);
        *n = n.saturating_add(1);
    }

    /// Removes and returns the oldest grant `who` holds at handle value
    /// `value`, which a `Closed` for that value refers to, or `None` if
    /// the broker gave it none.
    fn take_grant(&mut self, who: Proc, value: u64) -> Option<(PipeKey, PipeEnd)> {
        let grants = self.given.get_mut(&(who, value))?;
        let g = grants.remove(0);
        if grants.is_empty() {
            self.given.remove(&(who, value));
        }
        Some(g)
    }

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
        // W3: old pipes released (their receiver ended, or their holder or
        // creator died) are closed, and so are replacements for ends that
        // are gone.
        for k in self.ledger.take_released() {
            self.old.remove(&k);
        }
        let ledger = &self.ledger;
        let gone: Vec<(PipeKey, PipeEnd)> = self
            .pending
            .keys()
            .filter(|(c, e)| !ledger.knows(*c) || ledger.where_is(*c, *e) == Where::Gone)
            .copied()
            .collect();
        for k in gone {
            if let Some(p) = self.pending.remove(&k) {
                self.ledger.let_go(p.old_key);
            }
        }
    }
}

/// A child's request, handled by that child's worker (M8): the pull and
/// the kernel queries run there, off the IO thread and the lock.
enum Job {
    /// `Ctl::Relay`.
    Relay {
        req: u64,
        carrier: u64,
        /// Boxed, so every queued job is small.
        items: Box<Items>,
    },
    /// `Ctl::Redeem`.
    Redeem { req: u64, ticket: Ticket, rx: u64 },
    /// `Ctl::Bind`.
    Bind { rx: u64, rx_id: u64, peer: u32 },
    /// `Ctl::Unbind`: queued behind the child's earlier Binds, so an
    /// Unbind is never handled before the Bind it undoes.
    Unbind { rx_id: u64 },
    /// `Ctl::Park`: an unlistened receiver's old pipe, handed back (W3).
    Park { old: u64 },
    /// `Ctl::TakeDrains` (W3).
    TakeDrains { req: u64, rx: u64 },
    /// A relay of the broker's own ends, on its own worker (W3): the
    /// carrier end's id and facts, and the handles taken out of its
    /// table, each with its kind and, for a region (D6), its declared
    /// length and flags.
    OwnRelay {
        req: u64,
        carrier: u64,
        carrier_facts: Box<sys::RawFacts>,
        taken: Vec<(OwnedHandle, WireKind, u32, u8)>,
    },
}

/// One end in a relay, from the peek to the ticket (W3). The pulls and
/// the kernel queries run off the IO thread (M8) and never under the
/// lock.
struct Cand {
    /// The value the relayer named (the broker's own: its value here).
    value: u64,
    /// The kind it claims.
    kind: WireKind,
    /// A child's end: the broker's peek (a copy, no close-source) until
    /// the pull, then the pulled end. The broker's own end: the end.
    held: Option<OwnedHandle>,
    /// The broker's own end: nothing to pull.
    own: bool,
    /// What the kernel says about it: from the peek, then from the pull.
    facts: sys::RawFacts,
    /// Its channel, and the pipe it is on, by the peek's name.
    chan: Option<PipeKey>,
    pipe: Option<PipeKey>,
    /// What the ticket holds in its place: the end of the channel's new
    /// pipe, or a replacement made earlier. `None`: the end itself.
    ticket_end: Option<OwnedHandle>,
    /// Its pipe's server end joins the channel's old pipes.
    queue_old: bool,
    /// A pipe alias kept for its replacement, let go once decided.
    let_go: Option<PipeKey>,
    /// The pulled end is not the object peeked.
    changed: bool,
    /// For a region (D6): its declared length and flags, and the
    /// kernel's facts on it as a section. A region has no peer to move
    /// first, so a child's is pulled with close-source at once.
    len: u32,
    flags: u8,
    section: sys::SectionFacts,
}

/// A `Rebind` sent for a relay, awaiting its `Rebound` (W3).
struct RebindWait {
    req: u64,
    /// The process asked.
    holder: Proc,
    chan: PipeKey,
    /// The end it holds.
    f: PipeEnd,
    /// The pipe that end was on.
    prev: PipeKey,
    /// The new end's value in the holder.
    value: u64,
    /// The value the broker gave the end at, if it did.
    by_value: Option<u64>,
    /// The broker's copy of the new end.
    mine: OwnedHandle,
    /// The holder's process, to take the new end back if it does not
    /// switch.
    process: Option<sys::Process>,
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
    /// `TakeDrains` requests: request to receiver end (W3).
    takes: BTreeMap<u64, u64>,
    broker: Option<Broker>,
    /// This process's link to its broker (a child).
    link: Option<u64>,
    boot: Option<(OwnedHandle, OwnedHandle, u32)>,
    /// A test's child that ignores rebinds, for the rebind deadline.
    #[cfg(test)]
    deaf: bool,
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
    /// Signalled when a relay releases its channels (W3).
    settled: Condvar,
    /// The IO thread, once it runs.
    thread: OnceLock<sys::ThreadRef>,
    wake_pending: AtomicBool,
    /// This process's AppContainer SID, if it runs in one.
    container: Option<String>,
    /// Tests only: the next region handle cannot be made.
    #[cfg(test)]
    fail_region_handle: AtomicBool,
    /// Tests only: the next copy thread fails to start.
    #[cfg(test)]
    fail_copy_spawn: AtomicBool,
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
                old: Vec::new(),
                pipe_id: None,
            },
        );
        Some(id)
    }

    /// Removes an end; its I/O is cancelled and its handle closes when
    /// the last operation completes. Returns what to drop after the lock
    /// is released.
    fn remove(st: &mut State, id: u64) -> Option<End> {
        let end = st.ends.remove(&id)?;
        if let Some(b) = st.broker.as_mut() {
            b.own.remove(&id);
        }
        Self::cancel_all(&end);
        Some(end)
    }

    /// Cancels every operation of an end: on its pipe, its old pipe's
    /// credit read (a pending write there is left to finish, so no record
    /// is lost), and its drains.
    fn cancel_all(end: &End) {
        sys::cancel(&end.h);
        for o in end.old.iter().filter(|o| !o.writing) {
            sys::cancel(&o.h);
        }
        if let Role::Receiver { drains, .. } = &end.role {
            for d in drains {
                sys::cancel(d);
            }
        }
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
    /// Releases the ledger entry for a channel end being closed normally
    /// (items 2, 3). A child sends its broker the end's handle value,
    /// which the broker resolves through its give map; the broker
    /// resolves its own ends itself (its own pipes are overlapped, so
    /// the kernel queries do not block). The value is read while the
    /// handle is still open.
    fn note_close(&self, st: &mut State, id: u64) {
        let Some(e) = st.ends.get(&id) else {
            return;
        };
        let which = match &e.role {
            Role::Sender { .. } => PipeEnd::Sender,
            Role::Receiver { .. } => PipeEnd::Receiver,
            // A region has no ledger entry: its charge is its owner's.
            Role::Link { .. } | Role::Region { .. } => return,
        };
        let value = sys::value(&e.h);
        let handle = Arc::clone(&e.h);
        if let Some(b) = st.broker.as_mut() {
            let found = b
                .own
                .get(&id)
                .copied()
                .or_else(|| Self::key_of(b, &sys::facts(&handle)).map(|k| (k, which)));
            if let Some((pipe, end)) = found {
                b.ledger.close_end(pipe, end, BROKER);
                b.sweep();
            }
        } else {
            self.send_ctl(st, None, &Ctl::Closed { end: value });
        }
    }

    fn release(&self, id: u64) {
        let mut st = self.lock();
        self.note_close(&mut st, id);
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
            role:
                Role::Receiver {
                    listener: Some(l),
                    drains,
                    ..
                },
            ..
        }) = st.ends.get_mut(&id)
            && !l.ended
        {
            l.ended = true;
            sys::cancel(h);
            for d in drains.iter() {
                sys::cancel(d);
            }
            // Pending redemptions will never complete into a message.
            l.queue.retain(|i| !matches!(i, Item::Pending { .. }));
            l.queue.push_back(last);
        }
        self.kick(st, id);
    }

    /// The channel of a handle, as this process's broker sees it: its
    /// pipe's key, resolved to the channel's own key if the pipe is a
    /// later one (W3).
    fn key_of(b: &Broker, f: &sys::RawFacts) -> Option<PipeKey> {
        Self::pipe_key_of(b, f).map(|k| b.ledger.canonical(k))
    }

    /// The key of the pipe a handle is an end of, by its name.
    fn pipe_key_of(b: &Broker, f: &sys::RawFacts) -> Option<PipeKey> {
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

    // ---- relays: a fresh pipe per move (W3) ------------------------------

    /// A relay: carries `cands` (a child's values, peeked; or the broker's
    /// own ends) into a ticket for a record on the carrier.
    ///
    /// Each moving end's channel gets a new pipe the broker makes, and the
    /// peer end moves to it *before* the moving end is pulled, so nothing
    /// written after the move reaches a pipe its former holder had (probes
    /// W10, W13). In order:
    /// 1. under the lock, the channels are resolved and claimed (one relay
    ///    per channel at a time, so two ends moving at once settle in
    ///    turn);
    /// 2. for each moving end, the peer is found and switched: in a ticket
    ///    or in the broker, at once; with a child, by `Rebind`, whose
    ///    `Rebound` the IO thread routes here; on its way elsewhere, by a
    ///    replacement applied when its own relay pulls it; gone, not at
    ///    all;
    /// 3. off the lock, this worker waits for the `Rebound`s (at most
    ///    [`REBIND_DEADLINE`]), then pulls the moving ends with
    ///    close-source and checks each is the object it peeked;
    /// 4. under the lock, `accept` and the ledger decide, and the ticket
    ///    holds the new pipe's ends. A moving receiver's old pipe is
    ///    queued for whoever listens to it next; a moving sender's is
    ///    closed, so its reader drains it to its end.
    ///
    /// Runs on the relayer's worker (a child's, or the broker's own),
    /// never on the IO thread: it waits for other processes and, for a
    /// child's ends, for kernel queries that can block (probe W8).
    fn relay(
        self: &Arc<Self>,
        from: Proc,
        process: Option<&sys::Process>,
        carrier_facts: &sys::RawFacts,
        mut cands: Vec<Cand>,
    ) -> Result<Ticket, Refusal> {
        let mut later: Vec<Box<dyn Any + Send>> = Vec::new();
        // Every kernel query that can wait (probe W8) runs before a
        // channel is claimed, so a child can stall only its own worker,
        // never another relay of the same channel. Its peeked ends were
        // queried already (on its own worker); one whose file object does
        // synchronous I/O is refused here. After the pull only the object
        // peeked is queried again: it is overlapped, and its mode cannot
        // change (probe W14). A pulled end that is another object is
        // refused unqueried.
        if process.is_some() && cands.iter().any(|c| !c.own && c.facts.synchronous) {
            Self::discard(process, cands);
            if let Some(b) = self.lock().broker.as_mut() {
                b.count(Refusal::Synchronous);
            }
            return Err(Refusal::Synchronous);
        }
        // 0. The copies its regions need (D6 review Q1), made before any
        // channel is claimed: a copy, or a wait for another relay's copy
        // of the same section, stalls only this worker.
        if let Err(r) = self.region_copies(from, &cands, &mut later) {
            Self::discard(process, cands);
            if let Some(b) = self.lock().broker.as_mut() {
                b.count(r);
            }
            return Err(r);
        }
        // 1. Resolve and claim the channels.
        let mut st = self.lock();
        let (carrier, chans) = {
            let Some(b) = st.broker.as_mut() else {
                drop(st);
                Self::discard(process, cands);
                return Err(Refusal::Carrier);
            };
            let Some(carrier) = Self::key_of(b, carrier_facts) else {
                drop(st);
                Self::discard(process, cands);
                return Err(Refusal::Carrier);
            };
            for c in &mut cands {
                c.pipe = Self::pipe_key_of(b, &c.facts);
                c.chan = c.pipe.map(|k| b.ledger.canonical(k));
                // The moving end's pipe stays an alias of its channel
                // until the relay is decided: the switch below makes it a
                // former pipe, and the pulled end is still on it.
                if let (Some(chan), Some(pipe)) = (c.chan, c.pipe) {
                    b.ledger.keep(chan, pipe);
                }
            }
            let chans: BTreeSet<PipeKey> = cands.iter().filter_map(|c| c.chan).collect();
            (carrier, chans)
        };
        loop {
            match st.broker.as_ref() {
                Some(b) if chans.iter().any(|k| b.busy.contains(k)) => {
                    st = self
                        .settled
                        .wait(st)
                        .unwrap_or_else(PoisonError::into_inner);
                }
                _ => break,
            }
        }
        if let Some(b) = st.broker.as_mut() {
            b.busy.extend(chans.iter().copied());
        }
        drop(st);
        // The new pipes, made off the lock: one per moving end; a peer
        // that turns out to be gone needs none.
        let mut fresh: Vec<Option<([u8; 16], OwnedHandle, OwnedHandle)>> = cands
            .iter()
            .map(|c| {
                c.chan.and_then(|_| {
                    let id = Self::random16()?;
                    let (server, client) = sys::pipe_pair(None, &id).ok()?;
                    Some((id, server, client))
                })
            })
            .collect();
        // 2. Switch each moving end's peer.
        // At most one rebind per carried end.
        let (tx, rx) = mpsc::sync_channel::<(u64, Rebind)>(MAX_MESSAGE_HANDLES);
        let mut waits: Vec<RebindWait> = Vec::new();
        let mut refusal: Option<Refusal> = None;
        let mut st = self.lock();
        let mut paired: BTreeSet<PipeKey> = BTreeSet::new();
        for i in 0..cands.len() {
            if refusal.is_some() {
                break;
            }
            let Some((chan, pipe, kind)) =
                cands.get(i).and_then(|c| Some((c.chan?, c.pipe?, c.kind)))
            else {
                continue;
            };
            if paired.contains(&chan) {
                continue;
            }
            let e = end_for(kind);
            let f = Self::other(e);
            let r = self.plan_one(
                &mut st,
                i,
                chan,
                pipe,
                e,
                f,
                &mut cands,
                &mut fresh,
                &tx,
                &mut waits,
                &mut paired,
                &mut later,
            );
            if let Err(r) = r {
                refusal = Some(r);
            }
        }
        drop(st);
        drop(fresh);
        // The rebinds are queued on the holders' links: the IO thread
        // writes them, and must be woken to.
        self.wake();
        // 3. Wait for the Rebounds, then pull.
        let due = StdInstant::now()
            .checked_add(REBIND_DEADLINE)
            .unwrap_or_else(StdInstant::now);
        let mut answers: BTreeMap<u64, Rebind> = BTreeMap::new();
        while answers.len() < waits.len() {
            let left = due.saturating_duration_since(StdInstant::now());
            match rx.recv_timeout(left) {
                Ok((req, a)) => {
                    answers.insert(req, a);
                }
                Err(_) => break,
            }
        }
        for w in &waits {
            // A holder that did not switch must not keep the new end: it
            // is taken back (close-source) and checked.
            if answers.get(&w.req) == Some(&Rebind::NotSwitched)
                && let Some(p) = &w.process
                && let Ok(back) = sys::pull(p, w.value, true)
            {
                later.push(Box::new(back));
            }
        }
        for c in &mut cands {
            // A child's regions were pulled with close-source already.
            if let (Some(p), false, false) = (process, c.own, c.kind.is_region()) {
                let pulled = check_value(c.value)
                    .ok()
                    .and_then(|()| sys::pull(p, c.value, true).ok());
                let peek = c.held.take();
                // A value that names another object than the peek did (or
                // one where the peek found nothing) is not queried at all:
                // it may be a synchronous end with a read pending.
                c.changed = match (&pulled, &peek) {
                    (Some(a), Some(b)) => !sys::same_object(a, b),
                    (Some(_), None) => true,
                    (None, _) => false,
                };
                // The peek's copy goes before the count is taken.
                drop(peek);
                c.facts = match (&pulled, c.changed) {
                    (Some(h), false) => sys::facts(h),
                    _ => sys::RawFacts::default(),
                };
                c.held = pulled;
            }
        }
        // 4. Decide.
        let mut st = self.lock();
        let Some(b) = st.broker.as_mut() else {
            return Err(Refusal::Carrier);
        };
        for k in &chans {
            b.busy.remove(k);
        }
        self.settled.notify_all();
        for w in waits {
            let a = answers.get(&w.req).copied();
            if a.is_none() {
                b.waiters.remove(&w.req);
                refusal.get_or_insert(Refusal::Rebind);
            }
            if w.f == PipeEnd::Receiver && a != Some(Rebind::Switched { park: true }) {
                b.ledger.cancel_park(w.chan, w.holder, w.prev);
            }
            match a {
                Some(Rebind::Switched { .. }) => {
                    if let Some(v) = w.by_value
                        && let Some(g) = b.given.get_mut(&(w.holder, v))
                        && let Some(pos) = g.iter().position(|x| *x == (w.chan, w.f))
                    {
                        g.remove(pos);
                        if g.is_empty() {
                            b.given.remove(&(w.holder, v));
                        }
                    }
                    b.grant(w.holder, w.value, w.chan, w.f);
                    later.push(Box::new(w.mine));
                }
                Some(Rebind::NotSwitched)
                    if b.ledger.knows(w.chan) && b.ledger.where_is(w.chan, w.f) != Where::Gone =>
                {
                    b.ledger.keep(w.chan, w.prev);
                    if let Some(old) = b.pending.insert(
                        (w.chan, w.f),
                        Pending {
                            handle: w.mine,
                            old_key: w.prev,
                        },
                    ) {
                        b.ledger.let_go(old.old_key);
                        later.push(Box::new(old.handle));
                    }
                }
                _ => later.push(Box::new(w.mine)),
            }
        }
        let result = Self::commit(b, from, carrier, &mut cands, refusal, &mut later);
        if let Err(r) = result {
            b.count(r);
        }
        for c in &mut cands {
            if let Some(k) = c.let_go.take() {
                b.ledger.let_go(k);
            }
            if let (Some(_), Some(pipe)) = (c.chan, c.pipe) {
                b.ledger.let_go(pipe);
            }
        }
        b.sweep();
        drop(st);
        drop(cands);
        drop(later);
        result
    }

    /// Step 0 of [`Inner::relay`]: the copies a relay's regions need
    /// (review Q1), planned and charged under the lock, made here with no
    /// lock held (D5 review M8), and registered as their sources' one
    /// copy, which the decision then reuses. A copy another relay is
    /// making is waited for, then reused. The broker's own records had
    /// theirs made by `copies_ready`, so for them this finds them made.
    ///
    /// # Errors
    /// [`Refusal::Quota`] if a charge does not fit, a copy fails, the wait
    /// exceeds [`COPY_WAIT_MAX`], or the relayer died meanwhile (its
    /// copies are then refunded).
    fn region_copies(
        self: &Arc<Self>,
        from: Proc,
        cands: &[Cand],
        later: &mut Vec<Box<dyn Any + Send>>,
    ) -> Result<(), Refusal> {
        if !cands.iter().any(|c| c.kind.is_region()) {
            return Ok(());
        }
        let items = || {
            cands.iter().filter(|c| c.kind.is_region()).filter_map(|c| {
                c.held
                    .as_ref()
                    .map(|h| (h, c.kind, c.len, c.flags, c.section))
            })
        };
        let alive = |b: &Broker| from == BROKER || b.children.contains_key(&from);
        let deadline = StdInstant::now().checked_add(COPY_WAIT_MAX);
        let plan = loop {
            let p = {
                let mut st = self.lock();
                match st.broker.as_mut() {
                    Some(b) if alive(b) => Self::plan_copies(b, from, items()),
                    _ => Ok(Vec::new()),
                }
            };
            if !matches!(p, Err(Plan::Wait)) {
                break p;
            }
            // A copy that takes longer than this refuses the relay rather
            // than pin this worker.
            if deadline.is_none_or(|d| StdInstant::now() >= d) {
                break Err(Plan::Quota);
            }
            std::thread::sleep(COPY_WAIT_POLL);
        };
        let jobs = plan.map_err(|_p| Refusal::Quota)?;
        if jobs.is_empty() {
            return Ok(());
        }
        let (made, failed) = Self::make_copies(jobs);
        let mut st = self.lock();
        let Some(b) = st.broker.as_mut() else {
            later.push(Box::new((made, failed)));
            return Err(Refusal::Quota);
        };
        let copies_ok = failed.is_empty();
        Self::release_jobs(b, failed);
        if !alive(b) {
            // The relayer died while the copies were made.
            later.push(Box::new(Self::release_prepared(b, made)));
            return Err(Refusal::Quota);
        }
        // Registered now, so the in-progress mark lasts no longer than
        // the copy.
        later.push(Box::new(Self::register_made(b, made)));
        if copies_ok {
            Ok(())
        } else {
            Err(Refusal::Quota)
        }
    }

    /// Plans the move of `cands[i]`, end `e` of `chan` (now on `pipe`),
    /// whose peer is end `f`: see [`Inner::relay`], step 2.
    #[expect(
        clippy::too_many_arguments,
        reason = "one step of `relay`, split out for reading"
    )]
    fn plan_one(
        &self,
        st: &mut State,
        i: usize,
        chan: PipeKey,
        pipe: PipeKey,
        e: PipeEnd,
        f: PipeEnd,
        cands: &mut [Cand],
        fresh: &mut [Option<([u8; 16], OwnedHandle, OwnedHandle)>],
        tx: &mpsc::SyncSender<(u64, Rebind)>,
        waits: &mut Vec<RebindWait>,
        paired: &mut BTreeSet<PipeKey>,
        later: &mut Vec<Box<dyn Any + Send>>,
    ) -> Result<(), Refusal> {
        let Some(b) = st.broker.as_mut() else {
            return Err(Refusal::Carrier);
        };
        b.ledger.touch(chan).map_err(|_e| Refusal::Quota)?;
        let old_room = |r: crate::ipc::win::ledger::LedgerError| match r {
            crate::ipc::win::ledger::LedgerError::OldPipes => Refusal::OldPipes,
            _ => Refusal::Quota,
        };
        // An end whose peer moved while it was in transit already has its
        // replacement: a broker-made end no process ever held.
        if let Some(p) = b.pending.remove(&(chan, e)) {
            if e == PipeEnd::Receiver
                && let Err(r) = b.ledger.queue_old(chan, pipe)
            {
                b.pending.insert((chan, e), p);
                return Err(old_room(r));
            }
            if let Some(c) = cands.get_mut(i) {
                c.ticket_end = Some(p.handle);
                c.queue_old = e == PipeEnd::Receiver;
                c.let_go = Some(p.old_key);
            }
            return Ok(());
        }
        let partner = cands
            .iter()
            .position(|c| c.chan == Some(chan) && end_for(c.kind) == f);
        let peer = if partner.is_some() {
            None
        } else {
            Some(b.ledger.where_is(chan, f))
        };
        if peer == Some(Where::Gone) {
            // Nothing can write into or read from the old pipe any more:
            // the end moves as it is.
            return Ok(());
        }
        let Some((id, server, client)) = fresh.get_mut(i).and_then(Option::take) else {
            return Err(Refusal::NoTicket);
        };
        let (mine_e, mine_f) = if e == PipeEnd::Receiver {
            (server, client)
        } else {
            (client, server)
        };
        // The moving receiver's pipe is queued for whoever listens next.
        if e == PipeEnd::Receiver {
            b.ledger.queue_old(chan, pipe).map_err(old_room)?;
        }
        let prev = b.ledger.switch_pipe(
            chan,
            PipeKey {
                creator: BROKER,
                id,
            },
        );
        if let Some(c) = cands.get_mut(i) {
            c.ticket_end = Some(mine_e);
            c.queue_old = e == PipeEnd::Receiver;
        }
        match peer {
            // Both ends in this relay: the new holder gets both new ends.
            None => {
                if let Some(c) = partner.and_then(|j| cands.get_mut(j)) {
                    c.ticket_end = Some(mine_f);
                    c.queue_old = f == PipeEnd::Receiver;
                    if f == PipeEnd::Receiver {
                        b.ledger.queue_old(chan, pipe).map_err(old_room)?;
                    }
                }
                paired.insert(chan);
            }
            Some(Where::Gone) => later.push(Box::new(mine_f)),
            Some(Where::Ticket(t)) => {
                let slot = b
                    .ledger
                    .ticket_index(&t, chan, f)
                    .and_then(|n| b.held.get_mut(&t).and_then(|hs| hs.get_mut(n)));
                let Some(slot) = slot else {
                    later.push(Box::new(mine_f));
                    return Err(Refusal::Rebind);
                };
                let old = core::mem::replace(slot, mine_f);
                Self::keep_or_close(b, chan, f, prev, old, later).map_err(old_room)?;
            }
            Some(Where::Proc(BROKER)) => {
                let own = b
                    .own
                    .iter()
                    .find(|(_, v)| **v == (chan, f))
                    .map(|(id, _)| *id)
                    .filter(|id| st.ends.get(id).is_some_and(Self::movable));
                match own {
                    Some(id) => match Self::switch_end(st, id, mine_f, later) {
                        Switch::Park(h) => {
                            let Some(b) = st.broker.as_mut() else {
                                return Err(Refusal::Carrier);
                            };
                            Self::keep_or_close(b, chan, f, prev, h, later).map_err(old_room)?;
                            if let Some(End {
                                role: Role::Receiver { old_queued, .. },
                                ..
                            }) = st.ends.get_mut(&id)
                            {
                                *old_queued = true;
                            }
                        }
                        Switch::Done | Switch::No => {}
                    },
                    // On its way through the broker's own relay.
                    None => {
                        b.ledger.keep(chan, prev);
                        b.pending.insert(
                            (chan, f),
                            Pending {
                                handle: mine_f,
                                old_key: prev,
                            },
                        );
                    }
                }
            }
            Some(Where::Proc(h)) => {
                let by_value = b
                    .given
                    .iter()
                    .find(|((p, _), g)| *p == h && g.contains(&(chan, f)))
                    .map(|((_, v), _)| *v);
                let target = match by_value {
                    Some(v) => Ok(v),
                    None if chan.creator == h && prev == chan => Err(chan.id),
                    None => {
                        later.push(Box::new(mine_f));
                        return Err(Refusal::Rebind);
                    }
                };
                let Some(child) = b.children.get(&h) else {
                    later.push(Box::new(mine_f));
                    return Err(Refusal::Rebind);
                };
                let process = sys::duplicate(&child.process.0).ok().map(sys::Process);
                if f == PipeEnd::Receiver
                    && let Err(r) = b.ledger.expect_park(chan, h, prev)
                {
                    later.push(Box::new(mine_f));
                    return Err(old_room(r));
                }
                let value = match sys::give(&child.process, &mine_f) {
                    Ok(v) => v,
                    Err(_) => {
                        if f == PipeEnd::Receiver {
                            b.ledger.cancel_park(chan, h, prev);
                        }
                        later.push(Box::new(mine_f));
                        return Err(Refusal::Rebind);
                    }
                };
                b.next_rebind = b.next_rebind.wrapping_add(1);
                let req = b.next_rebind;
                b.waiters.insert(req, (h, tx.clone()));
                let ctl = match target {
                    Ok(end) => Ctl::RebindValue {
                        req,
                        end,
                        new: value,
                    },
                    Err(id) => Ctl::RebindId {
                        req,
                        id,
                        new: value,
                    },
                };
                self.send_ctl(st, Some(h), &ctl);
                waits.push(RebindWait {
                    req,
                    holder: h,
                    chan,
                    f,
                    prev,
                    value,
                    by_value,
                    mine: mine_f,
                    process,
                });
            }
        }
        Ok(())
    }

    /// The other end of a channel.
    const fn other(e: PipeEnd) -> PipeEnd {
        match e {
            PipeEnd::Sender => PipeEnd::Receiver,
            PipeEnd::Receiver => PipeEnd::Sender,
        }
    }

    /// An old pipe end the broker took from a peer that switched: a
    /// receiver's server end is queued for its next listener, a sender's
    /// client end closed.
    fn keep_or_close(
        b: &mut Broker,
        chan: PipeKey,
        end: PipeEnd,
        key: PipeKey,
        h: OwnedHandle,
        later: &mut Vec<Box<dyn Any + Send>>,
    ) -> Result<(), crate::ipc::win::ledger::LedgerError> {
        if end == PipeEnd::Receiver {
            match b.ledger.queue_old(chan, key) {
                Ok(()) => {
                    b.old.insert(key, h);
                }
                Err(r) => {
                    later.push(Box::new(h));
                    return Err(r);
                }
            }
        } else {
            later.push(Box::new(h));
        }
        Ok(())
    }

    /// Step 4 of [`Inner::relay`]: accepts each pulled end against the
    /// kernel's facts and the ledger, and on success holds the ticket's
    /// ends (the new pipe's where there is one) and queues the moving
    /// receivers' old pipes. On refusal every pulled end is closed and
    /// counted gone.
    fn commit(
        b: &mut Broker,
        from: Proc,
        carrier: PipeKey,
        cands: &mut [Cand],
        refusal: Option<Refusal>,
        later: &mut Vec<Box<dyn Any + Send>>,
    ) -> Result<Ticket, Refusal> {
        let mut first_err = refusal;
        let mut ends = Vec::with_capacity(cands.len());
        // Regions (D6): (index, registry id, outcome, kind, len).
        let mut regions = Vec::new();
        for (index, c) in cands.iter().enumerate() {
            if c.changed {
                first_err.get_or_insert(Refusal::Changed);
                continue;
            }
            if c.kind.is_region() {
                match Self::region_decide(b, from, c) {
                    Ok((id, o)) => regions.push((index, id, o, c.kind, c.len)),
                    Err(r) => {
                        first_err.get_or_insert(r);
                    }
                }
                continue;
            }
            let facts = Facts {
                raw: c.value,
                pulled: c.held.is_some(),
                is_pipe: c.facts.is_pipe,
                server_end: c.facts.server_end,
                message: c.facts.message,
                max_instances: c.facts.max_instances,
                read_message: c.facts.read_message,
                wait: c.facts.wait,
                pipe: Self::key_of(b, &c.facts),
                handle_count: c.facts.handle_count,
            };
            let view = facts.pipe.and_then(|k| b.ledger.view(k, end_for(c.kind)));
            let ctx = crate::ipc::win::accept::Ctx {
                relayer: from,
                carrier,
                view,
            };
            match accept(c.kind, &facts, &ctx) {
                Ok((pipe, end)) if c.changed || Some(pipe) != c.chan => {
                    first_err.get_or_insert(Refusal::Changed);
                    ends.push((pipe, end, c.kind));
                }
                Ok((pipe, end)) => ends.push((pipe, end, c.kind)),
                Err(r) => {
                    first_err.get_or_insert(r);
                }
            }
            // A replacement end must be the broker's only handle to it: a
            // holder that kept a copy of what it was given is refused.
            if c.let_go.is_some()
                && let Some(h) = &c.ticket_end
                && sys::count_handles(h) != 1
            {
                first_err.get_or_insert(Refusal::KeptCopy);
            }
        }
        let keys: Vec<(PipeKey, PipeEnd)> = ends.iter().map(|(p, e, _)| (*p, *e)).collect();
        // A section named twice in one relay.
        let mut ids: Vec<u64> = regions.iter().map(|r| r.1).collect();
        ids.sort_unstable();
        if ids.windows(2).any(|w| w.first() == w.get(1)) {
            first_err.get_or_insert(Refusal::Duplicate);
        }
        let result = match first_err {
            Some(r) => Err(r),
            None => distinct(&keys).and_then(|()| {
                let ticket = loop {
                    let t = Self::random16().ok_or(Refusal::NoTicket)?;
                    if t != NO_TICKET && b.ledger.ticket_free(&t) {
                        break t;
                    }
                };
                // The registry moves on last, and is put back if the
                // ledger then refuses the relay. The copies were made and
                // registered before the relay claimed its channels, so
                // none is made here.
                let (grants, undo) =
                    Self::region_commit(b, &regions, cands, later, &mut Vec::new())?;
                let held = Held {
                    carrier,
                    from,
                    ends: ends.clone(),
                    regions: grants,
                };
                if b.ledger.relay(ticket, held).is_err() {
                    Self::region_rollback(b, undo);
                    return Err(Refusal::Quota);
                }
                Ok(ticket)
            }),
        };
        match result {
            Ok(ticket) => {
                let mut handles = Vec::with_capacity(cands.len());
                for c in cands.iter_mut() {
                    let pulled = c.held.take();
                    match c.ticket_end.take() {
                        Some(new) => {
                            handles.push(new);
                            if let (true, Some(old), Some(pipe)) = (c.queue_old, pulled, c.pipe) {
                                b.old.insert(pipe, old);
                            }
                        }
                        None => handles.extend(pulled),
                    }
                }
                b.held.insert(ticket, handles);
                Ok(ticket)
            }
            Err(r) => {
                // The pulled ends are gone; so is each moving end's
                // channel side (its peer, if it switched, sees Closed).
                let lost: Vec<(PipeKey, PipeEnd, WireKind)> = cands
                    .iter()
                    .filter_map(|c| Some((c.chan?, end_for(c.kind), c.kind)))
                    .collect();
                b.ledger.lost(&lost);
                Err(r)
            }
        }
    }

    /// Decides one pulled region against the registry (D6 review B1):
    /// its registry id and what the relay does to it.
    fn region_decide(
        b: &Broker,
        from: Proc,
        c: &Cand,
    ) -> Result<(u64, crate::ipc::win::regions::Outcome), Refusal> {
        let h = c.held.as_ref().ok_or(Refusal::PullFailed)?;
        Self::region_decide_parts(b, from, h, c.kind, c.len, c.flags, c.section)
    }

    /// [`Inner::region_decide`] on a borrowed handle and its facts.
    fn region_decide_parts(
        b: &Broker,
        from: Proc,
        h: &OwnedHandle,
        kind: WireKind,
        len: u32,
        flags: u8,
        section: sys::SectionFacts,
    ) -> Result<(u64, crate::ipc::win::regions::Outcome), Refusal> {
        // Nor come from one (D6 review finding 5, fact S12).
        if from != BROKER && !b.children.get(&from).is_some_and(|c| c.regions) {
            return Err(Refusal::Region);
        }
        let p = RegionFacts {
            kind,
            len,
            flags,
            section,
        };
        let found = p
            .section
            .is_section
            .then(|| {
                b.regions
                    .iter()
                    .find(|(_, e)| sys::same_object(&e.master, h))
            })
            .flatten();
        let facts = RegionPulled {
            is_section: p.section.is_section,
            ours: found.is_some(),
            access: p.section.access,
            handle_count: p.section.handle_count,
        };
        let (id, size, state) = found.map_or((0, 0, RState::SealedByBroker), |(id, e)| {
            (*id, e.size, e.state)
        });
        let want = u64::from(p.flags) & REGION_IMMUTABLE != 0;
        relay_rule(p.kind, p.len, want, from, &facts, size, state)
            .map(|o| (id, o))
            .map_err(|_r| Refusal::Region)
    }

    /// Applies a relay's region outcomes once every item passed: moves
    /// each section's state on and makes the copies the outcomes ask for
    /// (review Q1), putting each copy's read-only handle in place of the
    /// pulled one (the pulled one is closed later, off the lock). Returns
    /// the grants for the ticket.
    fn region_commit(
        b: &mut Broker,
        regions: &[(usize, u64, crate::ipc::win::regions::Outcome, WireKind, u32)],
        cands: &mut [Cand],
        later: &mut Vec<Box<dyn Any + Send>>,
        prepared: &mut Vec<Prepared>,
    ) -> Result<(Vec<(usize, Grant)>, RegionUndo), Refusal> {
        let mut grants = Vec::with_capacity(regions.len());
        let mut undo = RegionUndo::default();
        for &(index, id, o, kind, len) in regions {
            let mut grant = Grant {
                id,
                kind,
                len,
                filler_trusted: o.filler_trusted,
            };
            if o.copy {
                // The section's one copy: reused if it exists, else the
                // one made off the lock for this relay (D5 review M8). A
                // copy neither made nor prepared fails the relay closed.
                let existing = b
                    .regions
                    .get(&id)
                    .and_then(|e| e.copy)
                    .and_then(|c| b.regions.get(&c).map(|ce| (c, ce)))
                    .map(|(c, ce)| (c, sys::narrow(&ce.master, sys::SECTION_READ)));
                let (copy_id, h) = match existing {
                    Some((c, Ok(h))) => (c, h),
                    Some((_, Err(_))) => {
                        Self::region_rollback(b, undo);
                        return Err(Refusal::Region);
                    }
                    None => {
                        let Some(at) = prepared.iter().position(|c| c.src == id && c.len == len)
                        else {
                            Self::region_rollback(b, undo);
                            return Err(Refusal::Region);
                        };
                        let c = prepared.swap_remove(at);
                        let (copy_id, h) = Self::register_copy(b, c);
                        if let Some(e) = b.regions.get_mut(&id) {
                            e.copy = Some(copy_id);
                            e.copying = false;
                        }
                        undo.copies.push((id, copy_id));
                        (copy_id, h)
                    }
                };
                if let Some(c) = cands.get_mut(index)
                    && let Some(pulled) = c.held.replace(h)
                {
                    later.push(Box::new(pulled));
                }
                grant.id = copy_id;
                grant.filler_trusted = false;
            }
            if let Some(e) = b.regions.get_mut(&id) {
                undo.states.push((id, e.state));
                e.state = o.next;
            }
            grants.push((index, grant));
        }
        Ok((grants, undo))
    }

    /// Puts the registry back as it was before a relay that failed after
    /// [`Inner::region_commit`]: the states restored, the copies forgotten
    /// and refunded.
    fn region_rollback(b: &mut Broker, undo: RegionUndo) {
        for (id, state) in undo.states.into_iter().rev() {
            if let Some(e) = b.regions.get_mut(&id) {
                e.state = state;
            }
        }
        for (src, id) in undo.copies {
            if let Some(e) = b.regions.remove(&id) {
                b.quota.refund(e.owner, u64::from(e.size));
            }
            if let Some(e) = b.regions.get_mut(&src) {
                e.copy = None;
            }
        }
    }

    /// Registers a new section made by the broker, charged to `owner`:
    /// its id and the broker's full-access handle's twin with `access`.
    fn register(
        b: &mut Broker,
        len: u32,
        owner: OsPeer,
        state: RState,
        access: u32,
    ) -> Result<(u64, OwnedHandle), IpcError> {
        if !b.quota.charge(owner, u64::from(len)) {
            return Err(IpcError::Quota);
        }
        let made = sys::create_section(len).and_then(|m| sys::narrow(&m, access).map(|h| (m, h)));
        let Ok((master, h)) = made else {
            b.quota.refund(owner, u64::from(len));
            return Err(IpcError::Refused);
        };
        b.next_region = b.next_region.wrapping_add(1);
        let id = b.next_region;
        b.regions.insert(
            id,
            RegEntry {
                size: len,
                owner,
                state,
                master,
                live: Arc::new(AtomicUsize::new(0)),
                copy: None,
                copying: false,
            },
        );
        Ok((id, h))
    }

    /// Registers a copy made off the lock as `SealedByBroker` (its charge
    /// was taken when it was planned): its id and read-only handle.
    fn register_copy(b: &mut Broker, c: Prepared) -> (u64, OwnedHandle) {
        b.next_region = b.next_region.wrapping_add(1);
        let id = b.next_region;
        b.regions.insert(
            id,
            RegEntry {
                size: c.len,
                owner: c.owner,
                state: RState::SealedByBroker,
                master: c.master,
                live: Arc::new(AtomicUsize::new(0)),
                copy: None,
                copying: false,
            },
        );
        (id, c.ro)
    }

    /// Plans the copies a relay's regions need (review Q1), under the
    /// lock: each region whose decision asks for a copy is charged now to
    /// its owner, against the owner's quota and the browser's cap, and
    /// gets a job holding the broker's own handle to the source. The
    /// slow part, [`Inner::make_copies`], runs off the lock.
    ///
    /// # Errors
    /// [`Plan::Quota`] if a charge does not fit, [`Plan::Wait`] if another
    /// relay is still making a copy this one needs (it is reused once
    /// made); nothing stays charged.
    fn plan_copies<'a>(
        b: &mut Broker,
        from: Proc,
        items: impl Iterator<Item = (&'a OwnedHandle, WireKind, u32, u8, sys::SectionFacts)>,
    ) -> Result<Vec<CopyJob>, Plan> {
        let mut jobs: Vec<CopyJob> = Vec::new();
        for (h, kind, len, flags, section) in items {
            if !kind.is_region() {
                continue;
            }
            let Ok((src, o)) = Self::region_decide_parts(b, from, h, kind, len, flags, section)
            else {
                // Refused anyway when the relay is decided.
                continue;
            };
            if !o.copy {
                continue;
            }
            let Some(e) = b.regions.get(&src) else {
                continue;
            };
            // One copy per section: an existing one is reused, and one
            // being made (or planned twice in this relay) is not made
            // again.
            let made = e.copy.is_some_and(|c| b.regions.contains_key(&c));
            if made || jobs.iter().any(|j| j.src == src) {
                continue;
            }
            if e.copying {
                Self::release_jobs(b, jobs);
                return Err(Plan::Wait);
            }
            let (owner, size) = (e.owner, e.size);
            let from_h = sys::duplicate(&e.master).ok();
            let charged = from_h.is_some() && b.quota.charge(owner, u64::from(size));
            let (true, Some(from_h)) = (charged, from_h) else {
                Self::release_jobs(b, jobs);
                return Err(Plan::Quota);
            };
            if let Some(e) = b.regions.get_mut(&src) {
                e.copying = true;
            }
            jobs.push(CopyJob {
                src,
                len: size,
                owner,
                from: from_h,
            });
        }
        Ok(jobs)
    }

    /// Makes planned copies, off the lock: each a new section the broker
    /// fills from the source. Returns the copies made, and the jobs whose
    /// copy failed (still charged: the caller releases both under the
    /// lock when it gives up).
    fn make_copies(jobs: Vec<CopyJob>) -> (Vec<Prepared>, Vec<CopyJob>) {
        let mut made = Vec::with_capacity(jobs.len());
        let mut failed = Vec::new();
        for j in jobs {
            match Self::copy_one(&j) {
                Some((master, ro)) => made.push(Prepared {
                    src: j.src,
                    len: j.len,
                    owner: j.owner,
                    master,
                    ro,
                }),
                None => failed.push(j),
            }
        }
        (made, failed)
    }

    /// One copy: a new section of the job's length, filled from the
    /// source in 1 MiB steps. No lock is held.
    fn copy_one(j: &CopyJob) -> Option<(OwnedHandle, OwnedHandle)> {
        let n = usize::try_from(j.len).ok()?;
        let master = sys::create_section(j.len).ok()?;
        let ro = sys::narrow(&master, sys::SECTION_READ).ok()?;
        let src = sys::map_section(&j.from, n, false, None).ok()?;
        let dst = sys::map_section(&master, n, true, None).ok()?;
        let mut buf = vec![0u8; 1 << 20];
        let mut off = 0usize;
        while off < n {
            let k = buf.len().min(n - off);
            let chunk = buf.get_mut(..k)?;
            src.read(off, chunk).ok()?;
            dst.write(off, chunk).ok()?;
            off += k;
        }
        drop((src, dst));
        Some((master, ro))
    }

    /// Refunds planned copies that will not be made or used.
    fn release_jobs(b: &mut Broker, jobs: Vec<CopyJob>) {
        for j in jobs {
            b.quota.refund(j.owner, u64::from(j.len));
            Self::not_copying(b, j.src);
        }
    }

    /// The copy of `src` is no longer being made.
    fn not_copying(b: &mut Broker, src: u64) {
        if let Some(e) = b.regions.get_mut(&src) {
            e.copying = false;
        }
    }

    /// Registers copies just made as their sources' one copy (charged
    /// when planned), and clears the in-progress marks; a copy whose
    /// source is gone (its owner released) is refunded. Returns the
    /// handles to drop outside the lock: each relay gets its own
    /// read-only handle from the copy when it is decided.
    fn register_made(b: &mut Broker, made: Vec<Prepared>) -> Vec<OwnedHandle> {
        let mut out = Vec::with_capacity(made.len());
        for c in made {
            if b.regions.contains_key(&c.src) {
                let src = c.src;
                let (copy_id, ro) = Self::register_copy(b, c);
                if let Some(e) = b.regions.get_mut(&src) {
                    e.copy = Some(copy_id);
                    e.copying = false;
                }
                out.push(ro);
            } else {
                out.extend(Self::release_prepared(b, vec![c]));
            }
        }
        out
    }

    /// Refunds and drops copies a relay did not use: it was refused, its
    /// child died, or the record carrying it was given up while the copy
    /// was made.
    fn release_prepared(b: &mut Broker, prepared: Vec<Prepared>) -> Vec<OwnedHandle> {
        let mut out = Vec::with_capacity(prepared.len().saturating_mul(2));
        for c in prepared {
            b.quota.refund(c.owner, u64::from(c.len));
            Self::not_copying(b, c.src);
            out.push(c.master);
            out.push(c.ro);
        }
        out
    }

    /// The registry id of the section `h` names, in the broker.
    fn region_id(b: &Broker, h: &OwnedHandle) -> Option<u64> {
        b.regions
            .iter()
            .find(|(_, e)| sys::same_object(&e.master, h))
            .map(|(id, _)| *id)
    }

    /// A refused relay's values: each is still pulled with close-source
    /// (the relayer gave them up) and closed. A child's regions were
    /// pulled already and close with `cands`.
    fn discard(process: Option<&sys::Process>, cands: Vec<Cand>) {
        if let Some(p) = process {
            for c in &cands {
                if !c.own
                    && !c.kind.is_region()
                    && check_value(c.value).is_ok()
                    && let Ok(h) = sys::pull(p, c.value, true)
                {
                    drop(h);
                }
            }
        }
        drop(cands);
    }

    /// Answers child `proc`'s `TakeDrains` for the receiver it peeked as
    /// `raw` (W3): the channel's old pipes, given into the child in order,
    /// if it holds that receiver and none were lost.
    fn give_drains(
        b: &mut Broker,
        proc: Proc,
        req: u64,
        raw: &sys::RawFacts,
        later: &mut Vec<Box<dyn Any + Send>>,
    ) -> Ctl {
        let chan = Self::receiver_key(b, raw)
            .filter(|k| b.ledger.where_is(*k, PipeEnd::Receiver) == Where::Proc(proc));
        let hs: Option<Vec<OwnedHandle>> = chan
            .and_then(|k| b.ledger.take_old(k))
            .and_then(|keys| keys.into_iter().map(|k| b.old.remove(&k)).collect());
        b.sweep();
        let (Some(hs), Some(c)) = (hs, b.children.get(&proc)) else {
            return Ctl::DrainsRefused { req };
        };
        let items: Vec<CtlItem> = hs
            .iter()
            .filter_map(|h| sys::give(&c.process, h).ok())
            .map(|value| CtlItem::end(value, WireKind::Receiver))
            .collect();
        let all = items.len() == hs.len();
        later.push(Box::new(hs));
        match Items::from_slice(&items) {
            Some(items) if all => Ctl::Drains { req, items },
            _ => Ctl::DrainsRefused { req },
        }
    }

    /// The broker listens on its own receiver `id`: it takes the old pipes
    /// it holds for that channel itself (W3). A receiver whose old pipes
    /// were lost ends in `Closed`.
    fn take_own_drains(self: &Arc<Self>, st: &mut State, id: u64, facts: Option<&sys::RawFacts>) {
        let Some(b) = st.broker.as_mut() else {
            return;
        };
        let chan = b
            .own
            .get(&id)
            .map(|(k, _)| *k)
            .or_else(|| facts.and_then(|f| Self::key_of(b, f)));
        let hs: Option<Vec<OwnedHandle>> = chan
            .and_then(|k| b.ledger.take_old(k))
            .and_then(|keys| keys.into_iter().map(|k| b.old.remove(&k)).collect());
        b.sweep();
        match (hs, st.ends.get_mut(&id)) {
            (
                Some(hs),
                Some(End {
                    role: Role::Receiver { drains, .. },
                    ..
                }),
            ) => {
                for h in hs.into_iter().rev() {
                    drains.push_front(Arc::new(h));
                }
            }
            _ => self.end_listener(st, id, Item::Closed),
        }
    }

    /// What the kernel says about a receiver end a child names by value,
    /// without taking it: a peek (no close-source) whose copy is dropped.
    /// Can wait on a synchronous end with a read pending (W8), so it runs
    /// on the child's worker, never on the IO thread (M8).
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
        let mut noted = Vec::with_capacity(handles.len());
        let mut writers = Vec::new();
        let mut ok = handles.len() == held.ends.len() + held.regions.len();
        // Regions never go to a child that could widen them (D6 review
        // finding 5, fact S12).
        if !held.regions.is_empty() && !b.children.get(&from).is_some_and(|c| c.regions) {
            ok = false;
        }
        if let Some(c) = b.children.get(&from).filter(|_| ok) {
            let mut ends = held.ends.iter();
            for (index, h) in handles.iter().enumerate() {
                let region = held.regions.iter().find(|(i, _)| *i == index);
                if let Some((_, g)) = region {
                    // A region goes in with exactly its kind's access.
                    let access = if g.kind == WireKind::RegionWriter {
                        sys::SECTION_RW
                    } else {
                        sys::SECTION_READ
                    };
                    match sys::give_with(&c.process, h, access) {
                        Ok(v) => {
                            items.push(region_item(v, g));
                            if g.kind == WireKind::RegionWriter {
                                writers.push(g.id);
                            }
                        }
                        Err(_) => ok = false,
                    }
                    continue;
                }
                let Some((pipe, end, kind)) = ends.next() else {
                    ok = false;
                    continue;
                };
                match sys::give(&c.process, h) {
                    Ok(v) => {
                        items.push(CtlItem {
                            drains: *end == PipeEnd::Receiver && b.ledger.has_old(*pipe),
                            ..CtlItem::end(v, *kind)
                        });
                        noted.push((v, *pipe, *end));
                    }
                    Err(_) => ok = false,
                }
            }
        } else {
            ok = false;
        }
        // A writer now rests with the redeemer (review B1).
        if ok {
            for id in writers {
                if let Some(e) = b.regions.get_mut(&id) {
                    e.state = RState::WriterOut(Some(from));
                }
            }
        }
        later.push(Box::new(handles));
        match Items::from_slice(&items) {
            Some(items) if ok => {
                for (v, pipe, end) in noted {
                    b.grant(from, v, pipe, end);
                }
                Ctl::Redeemed { req, items }
            }
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

    /// A child's worker (M8): handles the child's relays, redemptions,
    /// binds and unbinds one at a time, in the order the child sent
    /// them, so a request whose kernel queries block (a synchronous pipe,
    /// probe W8) stalls only this child's later requests, never the IO
    /// thread or another child. It blocks in `recv` with no timeout, so
    /// an idle worker never wakes (FP-18), and it ends once the broker
    /// forgets the child (dropping the queue's sender) and the queue is
    /// drained. It holds the transport weakly, so it never keeps it
    /// alive.
    fn worker(
        inner: &Weak<Self>,
        proc: Proc,
        process: Option<&sys::Process>,
        jobs: &mpsc::Receiver<Job>,
    ) {
        while let Ok(job) = jobs.recv() {
            let Some(inner) = inner.upgrade() else {
                return;
            };
            inner.run_job(proc, process, job);
        }
    }

    /// Handles one of child `proc`'s requests on its worker: the pull and
    /// the kernel queries first, off the lock; then the decision under
    /// it. A request whose child is gone by then is dropped unanswered,
    /// and whatever it pulled is closed outside the lock.
    fn run_job(self: &Arc<Self>, proc: Proc, process: Option<&sys::Process>, job: Job) {
        let mut later: Vec<Box<dyn Any + Send>> = Vec::new();
        let mut events = Vec::new();
        let mut reply = None;
        // The broker's own relays have no process; a child's job always
        // has one.
        let process = match (job, process) {
            (
                Job::OwnRelay {
                    req,
                    carrier,
                    carrier_facts,
                    taken,
                },
                _,
            ) => {
                let cands = taken
                    .into_iter()
                    .map(|(h, kind, len, flags)| {
                        let region = kind.is_region();
                        Cand {
                            value: sys::value(&h),
                            kind,
                            facts: if region {
                                sys::RawFacts::default()
                            } else {
                                sys::facts(&h)
                            },
                            section: if region {
                                sys::section_facts(&h)
                            } else {
                                sys::SectionFacts::default()
                            },
                            held: Some(h),
                            own: true,
                            chan: None,
                            pipe: None,
                            ticket_end: None,
                            queue_old: false,
                            let_go: None,
                            changed: false,
                            len,
                            flags,
                        }
                    })
                    .collect();
                let res = self.relay(BROKER, None, &carrier_facts, cands);
                let mut st = self.lock();
                Self::relay_done(&mut st, carrier, req, res.ok());
                drop(st);
                self.wake();
                return;
            }
            (job, Some(p)) => (job, p),
            (_, None) => return,
        };
        let (job, process) = process;
        let mut st = match job {
            Job::OwnRelay { .. } => return,
            Job::Relay {
                req,
                carrier,
                items,
            } => {
                let carrier_raw = Self::peek_raw(process, carrier);
                let cands = items
                    .as_slice()
                    .iter()
                    .map(|i| {
                        // A channel end is peeked: it is pulled only after
                        // its peer has moved to a new pipe (W3). A region
                        // has no peer, so it is pulled with close-source at
                        // once (D6).
                        let region = i.kind.is_region();
                        let held = check_value(i.value)
                            .ok()
                            .and_then(|()| sys::pull(process, i.value, region).ok());
                        let (facts, section) = if region {
                            let s = held.as_ref().map(sys::section_facts).unwrap_or_default();
                            (sys::RawFacts::default(), s)
                        } else {
                            let f = held.as_ref().map(sys::facts).unwrap_or_default();
                            (f, sys::SectionFacts::default())
                        };
                        Cand {
                            value: i.value,
                            kind: i.kind,
                            facts,
                            held,
                            own: false,
                            chan: None,
                            pipe: None,
                            ticket_end: None,
                            queue_old: false,
                            let_go: None,
                            changed: false,
                            len: i.len,
                            flags: i.flags,
                            section,
                        }
                    })
                    .collect();
                let res = self.relay(proc, Some(process), &carrier_raw, cands);
                let st = self.lock();
                if st
                    .broker
                    .as_ref()
                    .is_some_and(|b| b.children.contains_key(&proc))
                {
                    reply = Some(match res {
                        Ok(ticket) => Ctl::RelayOk { req, ticket },
                        Err(r) => Ctl::RelayRefused {
                            req,
                            reason: r.code(),
                        },
                    });
                }
                st
            }
            Job::Park { old } => {
                let pulled = check_value(old)
                    .ok()
                    .and_then(|()| sys::pull(process, old, true).ok());
                let f = pulled.as_ref().map(sys::facts).unwrap_or_default();
                let mut st = self.lock();
                if let Some(b) = st.broker.as_mut()
                    && b.children.contains_key(&proc)
                {
                    // Only an old pipe's sole server end this holder was
                    // expected to hand back joins its channel's queue.
                    let ok = f.is_pipe
                        && f.server_end
                        && f.message
                        && f.read_message
                        && f.wait
                        && f.max_instances == 1
                        && f.handle_count == 1;
                    let key = Self::pipe_key_of(b, &f);
                    match (ok, key, pulled) {
                        (true, Some(k), Some(h)) if b.ledger.park(proc, k).is_ok() => {
                            b.old.insert(k, h);
                        }
                        (_, _, h) => later.push(Box::new(h)),
                    }
                } else {
                    later.push(Box::new(pulled));
                }
                st
            }
            Job::TakeDrains { req, rx } => {
                let raw = Self::peek_raw(process, rx);
                let mut st = self.lock();
                if let Some(b) = st.broker.as_mut()
                    && b.children.contains_key(&proc)
                {
                    reply = Some(Self::give_drains(b, proc, req, &raw, &mut later));
                }
                st
            }
            Job::Redeem { req, ticket, rx } => {
                let raw = Self::peek_raw(process, rx);
                let mut st = self.lock();
                if let Some(b) = st.broker.as_mut()
                    && b.children.contains_key(&proc)
                {
                    let key = Self::receiver_key(b, &raw);
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
                st
            }
            Job::Bind { rx, rx_id, peer } => {
                let raw = Self::peek_raw(process, rx);
                let mut st = self.lock();
                if let Some(b) = st.broker.as_mut()
                    && b.children.contains_key(&proc)
                {
                    match Self::receiver_key(b, &raw) {
                        Some(k) => {
                            if let Ok(Some(ev)) = b.ledger.bind(proc, k, rx_id, peer) {
                                events.push(ev);
                            }
                        }
                        // The end could not be classified: fail closed,
                        // the listener ends with an Identity fault.
                        None => reply = Some(Ctl::BindBroken { rx_id, peer }),
                    }
                }
                st
            }
            Job::Unbind { rx_id } => {
                let mut st = self.lock();
                if let Some(b) = st.broker.as_mut() {
                    b.ledger.unbind(proc, rx_id);
                }
                st
            }
        };
        if let Some(r) = reply {
            self.send_ctl(&mut st, Some(proc), &r);
        }
        self.dispatch(&mut st, events);
        drop(st);
        drop(later);
        self.wake();
    }

    /// Queues one of child `from`'s requests for its worker (M8). False
    /// if the child is unknown, its queue is full or its worker is gone;
    /// the caller then drops the child's link.
    fn enqueue(b: &Broker, from: Proc, job: Job) -> bool {
        b.children
            .get(&from)
            .is_some_and(|c| c.jobs.try_send(job).is_ok())
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
            // Relays, redemptions, binds and unbinds go to the child's
            // worker, which handles them in the order sent, off the IO
            // thread (M8). A child whose queue is full has its link
            // dropped.
            Ctl::Relay {
                req,
                carrier,
                items,
            } => {
                if !Self::enqueue(
                    b,
                    from,
                    Job::Relay {
                        req,
                        carrier,
                        items: Box::new(items),
                    },
                ) {
                    return self.child_gone(st, from);
                }
                None
            }
            Ctl::Redeem { req, ticket, rx } => {
                if !Self::enqueue(b, from, Job::Redeem { req, ticket, rx }) {
                    return self.child_gone(st, from);
                }
                None
            }
            Ctl::Bind { rx, rx_id, peer } => {
                if !Self::enqueue(b, from, Job::Bind { rx, rx_id, peer }) {
                    return self.child_gone(st, from);
                }
                None
            }
            Ctl::Unbind { rx_id } => {
                if !Self::enqueue(b, from, Job::Unbind { rx_id }) {
                    return self.child_gone(st, from);
                }
                None
            }
            // Answered inline, on the IO thread: the relayer's worker is
            // waiting for it (W3).
            Ctl::Rebound {
                req,
                switched,
                park,
            } => {
                if b.waiters.get(&req).is_some_and(|(h, _)| *h == from)
                    && let Some((_, tx)) = b.waiters.remove(&req)
                {
                    let a = if switched {
                        Rebind::Switched { park }
                    } else {
                        Rebind::NotSwitched
                    };
                    let _ = tx.try_send((req, a));
                }
                None
            }
            Ctl::Park { old } => {
                if !Self::enqueue(b, from, Job::Park { old }) {
                    return self.child_gone(st, from);
                }
                None
            }
            Ctl::TakeDrains { req, rx } => {
                if !Self::enqueue(b, from, Job::TakeDrains { req, rx }) {
                    return self.child_gone(st, from);
                }
                None
            }
            Ctl::Closed { end } => {
                // Resolved from the give map, so no handle is pulled and
                // no kernel query blocks (items 2, 3). The oldest grant
                // made at this handle value is the one being closed; a
                // value reused by a later redeem sits behind it, so taking
                // the front never closes the wrong channel's end.
                if let Some((pipe, which)) = b.take_grant(from, end) {
                    b.ledger.close_end(pipe, which, from);
                    b.sweep();
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
            | Ctl::BindBroken { .. }
            | Ctl::RebindValue { .. }
            | Ctl::RebindId { .. }
            | Ctl::Drains { .. }
            | Ctl::DrainsRefused { .. } => {
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
        // A relay waiting for this child's Rebound stops waiting (W3).
        let asked: Vec<u64> = b
            .waiters
            .iter()
            .filter(|(_, (h, _))| *h == p)
            .map(|(r, _)| *r)
            .collect();
        for r in asked {
            if let Some((_, tx)) = b.waiters.remove(&r) {
                let _ = tx.try_send((r, Rebind::Gone));
            }
        }
        for t in b.ledger.gone(p) {
            if let Some(hs) = b.held.remove(&t) {
                out.push(Box::new(hs));
            }
        }
        b.given.retain(|(owner, _), _| *owner != p);
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

    /// Moves end `id` to the new pipe end `new` (W3): its peer moved, so
    /// both ends go to a pipe no former holder ever had. A sender writes
    /// on the new pipe from now on; a write still pending on the old one
    /// finishes when the reader drains it (it is never cancelled, so no
    /// record is lost), then the old pipe's credit read is cancelled and
    /// the old pipe closes. A listening receiver keeps its old pipe as a
    /// drain, reads it to its end, then the new one, and sends its
    /// current delivered count as its first credit on the new pipe (a
    /// credit left in the old pipe is superseded). An unlistened
    /// receiver gives its old pipe back to the broker (`Park`). An end on
    /// its way to another process is not switched; the broker applies the
    /// switch when it pulls that end.
    fn switch_end(
        st: &mut State,
        id: u64,
        new: OwnedHandle,
        later: &mut Vec<Box<dyn Any + Send>>,
    ) -> Switch {
        let Some(end) = st.ends.get_mut(&id).filter(|e| Self::movable(e)) else {
            later.push(Box::new(new));
            return Switch::No;
        };
        let old = core::mem::replace(&mut end.h, Arc::new(new));
        let (writing, reading) = (end.writing, end.reading);
        match &mut end.role {
            Role::Sender { .. } => {
                end.writing = false;
                end.reading = false;
                if writing {
                    // The credit read stops once the write is done.
                    end.old.push(OldPipe {
                        h: old,
                        writing: true,
                        reading,
                    });
                } else if reading {
                    sys::cancel(&old);
                    end.old.push(OldPipe {
                        h: old,
                        writing: false,
                        reading: true,
                    });
                } else {
                    later.push(Box::new(old));
                }
                Switch::Done
            }
            Role::Receiver {
                flow,
                listener: Some(l),
                drains,
                ..
            } => {
                // A read in flight is on the oldest drain or, with none,
                // on the old pipe, which is now the oldest drain.
                drains.push_back(old);
                end.writing = false;
                l.credit.note(Credit {
                    capacity: flow.capacity,
                    delivered: flow.delivered,
                });
                Switch::Done
            }
            Role::Receiver {
                listener: None,
                drains,
                ..
            } => match Arc::try_unwrap(old) {
                Ok(h) => Switch::Park(h),
                Err(shared) => {
                    // Still referenced (cannot happen without a listener):
                    // keep it to read first instead.
                    drains.push_back(shared);
                    Switch::Done
                }
            },
            // Not movable, so never reached.
            Role::Link { .. } | Role::Region { .. } => Switch::No,
        }
    }

    /// Whether a rebind can switch this end: a channel end not on its way
    /// to another process.
    fn movable(e: &End) -> bool {
        match &e.role {
            Role::Sender { in_transit, .. } | Role::Receiver { in_transit, .. } => !*in_transit,
            Role::Link { .. } | Role::Region { .. } => false,
        }
    }

    /// A child's answer to `RebindValue` or `RebindId`: finds the end,
    /// adopts the new pipe end and switches (W3). A value it does not
    /// adopt is left in its table, for the broker to take back.
    fn child_rebind(
        &self,
        st: &mut State,
        req: u64,
        target: Result<u64, [u8; 16]>,
        new: u64,
        later: &mut Vec<Box<dyn Any + Send>>,
    ) {
        let held = |st: &State, v: u64| st.ends.values().any(|e| sys::value(&e.h) == v);
        let found = if held(st, new) {
            None
        } else {
            match target {
                Ok(value) => st
                    .ends
                    .iter()
                    .find(|(_, e)| sys::value(&e.h) == value && Self::movable(e))
                    .map(|(id, e)| (*id, matches!(e.role, Role::Receiver { .. })))
                    .and_then(|(id, server)| sys::adopt_pipe(new, server).map(|h| (id, h))),
                Err(pipe_id) => sys::adopt_pipe_end(new).and_then(|(h, server)| {
                    let id = st
                        .ends
                        .iter()
                        .find(|(_, e)| {
                            e.pipe_id == Some(pipe_id)
                                && Self::movable(e)
                                && matches!(
                                    (&e.role, server),
                                    (Role::Receiver { .. }, true) | (Role::Sender { .. }, false)
                                )
                        })
                        .map(|(id, _)| *id);
                    match id {
                        Some(id) => Some((id, h)),
                        None => {
                            sys::forget(h);
                            None
                        }
                    }
                }),
            }
        };
        let target_id = found.as_ref().map(|(id, _)| *id);
        let outcome = match found {
            // Found movable above, under the same lock: it switches.
            Some((id, h)) => Self::switch_end(st, id, h, later),
            None => Switch::No,
        };
        let (switched, park) = match &outcome {
            Switch::No => (false, false),
            Switch::Done => (true, false),
            Switch::Park(_) => (true, true),
        };
        self.send_ctl(
            st,
            None,
            &Ctl::Rebound {
                req,
                switched,
                park,
            },
        );
        if let Switch::Park(h) = outcome {
            self.send_ctl(
                st,
                None,
                &Ctl::Park {
                    old: sys::value(&h),
                },
            );
            // The broker pulls it with DUPLICATE_CLOSE_SOURCE.
            sys::forget(h);
            if let Some(id) = target_id
                && let Some(End {
                    role: Role::Receiver { old_queued, .. },
                    ..
                }) = st.ends.get_mut(&id)
            {
                *old_queued = true;
            }
        }
    }

    /// The broker answered `TakeDrains` `req`: `got` are the old pipes'
    /// server ends in order, or `None` if it refused. A receiver whose
    /// old pipes cannot all be read ends in `Closed`: records were lost
    /// with them, and reading on would hide that.
    fn drains_arrived(
        self: &Arc<Self>,
        st: &mut State,
        req: u64,
        got: Option<Vec<Option<OwnedHandle>>>,
        later: &mut Vec<Box<dyn Any + Send>>,
    ) {
        let Some(rx) = st.takes.remove(&req) else {
            later.push(Box::new(got));
            return;
        };
        let all: Option<Vec<OwnedHandle>> = got.and_then(|g| g.into_iter().collect());
        match (all, st.ends.get_mut(&rx)) {
            (
                Some(hs),
                Some(End {
                    role:
                        Role::Receiver {
                            listener: Some(l),
                            drains,
                            ..
                        },
                    ..
                }),
            ) if l.taking == Some(req) => {
                // Older than any drain kept here since: read first.
                for h in hs.into_iter().rev() {
                    drains.push_front(Arc::new(h));
                }
                l.taking = None;
            }
            (hs, _) => {
                later.push(Box::new(hs));
                self.end_listener(st, rx, Item::Closed);
            }
        }
    }

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
                    let got: Vec<Got> = items
                        .as_slice()
                        .iter()
                        .map(|i| {
                            let adopt = || match i.kind {
                                WireKind::RegionWriter => {
                                    sys::adopt_section(i.value, sys::SECTION_RW)
                                }
                                WireKind::ReadOnlyRegion | WireKind::ConsumerRegion => {
                                    sys::adopt_section(i.value, sys::SECTION_READ)
                                }
                                WireKind::Sender | WireKind::BoundSender | WireKind::Receiver => {
                                    sys::adopt_pipe(i.value, i.kind == WireKind::Receiver)
                                }
                            };
                            let h = (!held.contains(&i.value)).then(adopt).flatten();
                            (h, *i, None)
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
            #[cfg(test)]
            Ctl::RebindValue { .. } | Ctl::RebindId { .. } if st.deaf => {}
            Ctl::RebindValue { req, end, new } => {
                self.child_rebind(st, req, Ok(end), new, &mut out)
            }
            Ctl::RebindId { req, id, new } => self.child_rebind(st, req, Err(id), new, &mut out),
            Ctl::Drains { req, items } => {
                let held: Vec<u64> = st.ends.values().map(|e| sys::value(&e.h)).collect();
                let got: Vec<Option<OwnedHandle>> = items
                    .as_slice()
                    .iter()
                    .map(|i| {
                        (!held.contains(&i.value) && i.kind == WireKind::Receiver)
                            .then(|| sys::adopt_pipe(i.value, true))
                            .flatten()
                    })
                    .collect();
                self.drains_arrived(st, req, Some(got), &mut out);
            }
            Ctl::DrainsRefused { req } => self.drains_arrived(st, req, None, &mut out),
            // Records only a child sends, or a second Hello: ignored.
            Ctl::Hello { .. }
            | Ctl::Relay { .. }
            | Ctl::Redeem { .. }
            | Ctl::Bind { .. }
            | Ctl::Unbind { .. }
            | Ctl::Closed { .. }
            | Ctl::Rebound { .. }
            | Ctl::Park { .. }
            | Ctl::TakeDrains { .. } => {}
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
        got: Option<Vec<Got>>,
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
            let items: Vec<CtlItem> = got.iter().map(|(_, i, _)| *i).collect();
            if got.iter().any(|(h, _, _)| h.is_none()) {
                Err(Reason::Redeem)
            } else {
                check_redeemed(&descs, &items)
            }
        };
        match checked {
            Ok(kinds) => {
                let mut handles = Vec::with_capacity(got.len());
                for (((h, item, key), (kind, immutable)), desc) in
                    got.into_iter().zip(kinds).zip(&descs)
                {
                    let has_old = item.drains;
                    let Some(h) = h else { continue };
                    if desc.kind.is_region() {
                        let role = Role::Region {
                            kind,
                            len: desc.capacity,
                            want: false,
                            live: Arc::new(AtomicUsize::new(0)),
                        };
                        if let Some(id) = Self::insert(st, h, role) {
                            handles.push(Handle::new(Box::new(WinEnd {
                                inner: Arc::clone(self),
                                id,
                                kind,
                                armed: true,
                                region: Some(RegionInfo {
                                    len: usize::try_from(desc.capacity).unwrap_or(0),
                                    immutable,
                                    want: AtomicBool::new(false),
                                }),
                            })));
                        }
                        continue;
                    }
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
                        // Regions were made above; nothing else is redeemed.
                        HandleKind::RegionWriter
                        | HandleKind::ReadOnlyRegion
                        | HandleKind::ConsumerRegion
                        | HandleKind::ProducerRegion
                        | HandleKind::GpuIoSurface
                        | HandleKind::GpuDmaBuf
                        | HandleKind::GpuDxgiShared => continue,
                        HandleKind::ChannelReceiver => {
                            let Ok(flow) = ReceiverFlow::from_transfer(desc) else {
                                continue;
                            };
                            Role::Receiver {
                                flow,
                                bound: false,
                                listener: None,
                                in_transit: false,
                                drains: VecDeque::new(),
                                old_queued: has_old,
                            }
                        }
                    };
                    if let Some(id) = Self::insert(st, h, role) {
                        // The broker keeps its own ends' channels, so a
                        // close or a peer's move finds them (W3).
                        if let (Some(b), Some(k)) = (st.broker.as_mut(), key) {
                            b.own.insert(id, k);
                        }
                        handles.push(Handle::new(Box::new(WinEnd {
                            inner: Arc::clone(self),
                            id,
                            kind,
                            armed: true,
                            region: None,
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
        let mut ended: Vec<u64> = Vec::new();
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
                    let read =
                        (!l.ended && !flow.paused() && l.taking.is_none()).then_some(MAX_PIPE_DATA);
                    let write = if end.writing || !l.credit.is_pending() {
                        None
                    } else {
                        credit = l.credit.take();
                        credit.map(|c| encode_credit(c).to_vec())
                    };
                    (read, write)
                }
                Role::Receiver { .. } | Role::Region { .. } => (None, None),
                Role::Link { out, .. } => {
                    let write = if end.writing { None } else { out.pop_front() };
                    (Some(MAX_CTL_LEN), write)
                }
            };
            if let Some(cap) = read
                && !end.reading
            {
                // A receiver reads its oldest drain, if it has one, before
                // its current pipe (W3). A read on a pipe whose writer has
                // closed with nothing left in it fails at once (Win32 error
                // 109, `ERROR_BROKEN_PIPE`) instead of completing with it:
                // on a drain that is its end, and the next pipe is read;
                // on the current pipe it is the channel's end, as the same
                // error from a completion is.
                loop {
                    let drain = match &end.role {
                        Role::Receiver { drains, .. } => drains.front().cloned(),
                        _ => None,
                    };
                    let on = drain.as_ref().unwrap_or(&end.h);
                    match (sys::start_read(on, cap, id), &mut end.role) {
                        (Ok(()), _) => end.reading = true,
                        (Err(sys::ERROR_BROKEN_PIPE), Role::Receiver { drains, .. })
                            if drain.is_some() =>
                        {
                            later.push(Box::new(drains.pop_front()));
                            continue;
                        }
                        (Err(sys::ERROR_BROKEN_PIPE), Role::Receiver { .. }) => ended.push(id),
                        // Anything else is tried again on the next pass.
                        (Err(_), _) => {}
                    }
                    break;
                }
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
                        Role::Receiver { listener: None, .. } | Role::Region { .. } => {}
                    },
                }
            }
        }
        for id in ended {
            self.end_listener(st, id, Item::Closed);
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
                Some(Out::Carry(c))
                    if c.relaying.is_none() && c.record.is_none() && c.copying.is_none() =>
                {
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
            // A write still pending on an old pipe (W3) counts too: the
            // moved end's records must all be in before it moves.
            let draining = matches!(
                &e.role,
                Role::Sender { outbox, broken, .. } if !outbox.is_empty() && !*broken
            ) || !e.old.is_empty();
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
        // Encode the record now, with the carried ends' final counters
        // and the regions' lengths and flags.
        let mut descs = Vec::with_capacity(carried.len());
        for (cid, kind) in &carried {
            match st.ends.get(cid).map(|e| &e.role) {
                Some(Role::Sender { flow, .. }) => descs.push(flow.to_desc(*kind)),
                Some(Role::Receiver { flow, .. }) => descs.push(flow.to_desc()),
                Some(Role::Region { len, want, .. }) => {
                    let flags = if *want { REGION_IMMUTABLE } else { 0 };
                    descs.push(HandleDesc::region(*kind, *len, flags));
                }
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
        // In the broker, the copies its own regions need (review Q1) are
        // made first, off the lock on a thread of their own (D5 review
        // M8); this record waits for them.
        if st.broker.is_some() && !self.copies_ready(st, id, &carried, &descs, later) {
            return;
        }
        // Take the carried ends out of this process's table.
        let mut taken = Vec::with_capacity(carried.len());
        for (cid, kind) in &carried {
            let Some(e) = st.ends.remove(cid) else {
                continue;
            };
            if let Some(b) = st.broker.as_mut() {
                b.own.remove(cid);
            }
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
            // The broker relays its own handles on its own worker: the
            // relay may wait for another process's Rebound (W3). Its
            // ends are its own overlapped pipes, so the kernel queries do
            // not block (unlike a child's possibly-crafted handle, M8).
            // The copies its regions need were made first
            // (`copies_ready`), so the worker never makes one.
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
            let taken = taken
                .into_iter()
                .zip(&descs)
                .map(|((h, kind), d)| {
                    if kind.is_region() {
                        (h, kind, d.capacity, item_flags(d.sent))
                    } else {
                        (h, kind, 0, 0)
                    }
                })
                .collect();
            let job = Job::OwnRelay {
                req,
                carrier: id,
                carrier_facts: Box::new(carrier_facts),
                taken,
            };
            let queued = match st.broker.as_ref().and_then(|b| b.own_jobs.as_ref()) {
                Some(q) => q.try_send(job).map_err(|e| later.push(Box::new(e))).is_ok(),
                None => {
                    later.push(Box::new(job));
                    false
                }
            };
            if !queued {
                Self::break_sender(st, id);
            }
            return;
        }
        // A child: ask the broker.
        let items: Vec<CtlItem> = taken
            .iter()
            .zip(&descs)
            .map(|((h, k), d)| {
                if k.is_region() {
                    CtlItem {
                        value: sys::value(h),
                        kind: *k,
                        len: d.capacity,
                        flags: item_flags(d.sent),
                        drains: false,
                    }
                } else {
                    CtlItem::end(sys::value(h), *k)
                }
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

    /// For a record the broker sends: whether the copies its regions need
    /// are ready. If none are needed they are (an empty set); if some are,
    /// they are planned and charged now and made on a new thread, which
    /// stores them in the record and wakes the IO thread, and this
    /// returns false until then. A record given up meanwhile (its sender
    /// broken or gone) has its copies refunded by that thread.
    fn copies_ready(
        self: &Arc<Self>,
        st: &mut State,
        id: u64,
        carried: &[(u64, WireKind)],
        descs: &[HandleDesc],
        later: &mut Vec<Box<dyn Any + Send>>,
    ) -> bool {
        let ready = matches!(
            st.ends.get(&id).map(|e| &e.role),
            Some(Role::Sender { outbox, .. })
                if matches!(outbox.front(), Some(Out::Carry(c)) if c.prepared.is_some())
        );
        if ready {
            return true;
        }
        let items: Vec<(Arc<OwnedHandle>, WireKind, u32, u8, sys::SectionFacts)> = carried
            .iter()
            .zip(descs)
            .filter(|((_, k), _)| k.is_region())
            .filter_map(|((cid, k), d)| {
                let h = Arc::clone(&st.ends.get(cid)?.h);
                let f = sys::section_facts(&h);
                Some((h, *k, d.capacity, item_flags(d.sent), f))
            })
            .collect();
        let Some(b) = st.broker.as_mut() else {
            return false;
        };
        let plan = Self::plan_copies(
            b,
            BROKER,
            items.iter().map(|(h, k, l, f, s)| (&**h, *k, *l, *f, *s)),
        );
        let jobs = match plan {
            Ok(jobs) => jobs,
            // Another relay's copy: this record waits; the IO thread is
            // woken when that relay ends, and plans again.
            Err(Plan::Wait) => return false,
            Err(Plan::Quota) => {
                Self::break_sender(st, id);
                return false;
            }
        };
        let token = st.req();
        let set = |st: &mut State, prepared: Option<Vec<Prepared>>, copying: Option<u64>| {
            if let Some(End {
                role: Role::Sender { outbox, .. },
                ..
            }) = st.ends.get_mut(&id)
                && let Some(Out::Carry(c)) = outbox.front_mut()
            {
                c.prepared = prepared;
                c.copying = copying;
            }
        };
        if jobs.is_empty() {
            set(st, Some(Vec::new()), None);
            return true;
        }
        set(st, None, Some(token));
        // What to take back if the copy thread cannot start: each charge,
        // and each source's in-progress mark (else every later relay
        // needing that copy waits COPY_WAIT_MAX and is refused).
        let planned: Vec<(u64, OsPeer, u32)> =
            jobs.iter().map(|j| (j.src, j.owner, j.len)).collect();
        let inner = Arc::clone(self);
        let work = move || {
            let (made, failed) = Self::make_copies(jobs);
            let mut st = inner.lock();
            let waiting = matches!(
                st.ends.get(&id).map(|e| &e.role),
                Some(Role::Sender { outbox, broken: false, .. })
                    if matches!(outbox.front(), Some(Out::Carry(c)) if c.copying == Some(token))
            );
            let mut drop_later = Vec::new();
            if waiting && failed.is_empty() {
                if let Some(b) = st.broker.as_mut() {
                    drop_later = Self::register_made(b, made);
                }
                if let Some(End {
                    role: Role::Sender { outbox, .. },
                    ..
                }) = st.ends.get_mut(&id)
                    && let Some(Out::Carry(c)) = outbox.front_mut()
                {
                    c.prepared = Some(Vec::new());
                    c.copying = None;
                }
            } else {
                if waiting {
                    Self::break_sender(&mut st, id);
                }
                if let Some(b) = st.broker.as_mut() {
                    Self::release_jobs(b, failed);
                    drop_later = Self::release_prepared(b, made);
                }
            }
            drop(st);
            drop(drop_later);
            inner.wake();
        };
        #[cfg(test)]
        let spawned = if self.fail_copy_spawn.swap(false, Ordering::AcqRel) {
            drop(work);
            Err(std::io::Error::other("injected spawn failure"))
        } else {
            crate::thread::spawn("hg-ipc-copy", work)
        };
        #[cfg(not(test))]
        let spawned = crate::thread::spawn("hg-ipc-copy", work);
        if spawned.is_err() {
            if let Some(b) = st.broker.as_mut() {
                for (src, owner, len) in planned {
                    b.quota.refund(owner, u64::from(len));
                    Self::not_copying(b, src);
                }
            }
            Self::break_sender(st, id);
            // Records waiting for these copies plan again.
            self.wake();
        }
        later.push(Box::new(items));
        false
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
            Drained,
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
            later.push(Box::new((c.buf, c.handle)));
            return later;
        };
        // After a switch to a new pipe (W3) an operation may still finish
        // on the end's old pipe or on a drain.
        let current = Arc::ptr_eq(&c.handle, &end.h);
        if c.write {
            if current {
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
            } else if let Some(i) = end.old.iter().position(|o| Arc::ptr_eq(&o.h, &c.handle)) {
                // The last record on the old pipe is written: stop its
                // credit read; the pipe closes once that is done, and its
                // reader then moves on to the new one.
                if let Some(o) = end.old.get_mut(i) {
                    o.writing = false;
                    if o.reading {
                        sys::cancel(&o.h);
                    } else {
                        later.push(Box::new(end.old.remove(i)));
                    }
                }
            }
            // Otherwise a receiver's credit write on a pipe it left.
            later.push(Box::new(c.handle));
            return later;
        }
        let mut on_drain = false;
        if !current {
            match &mut end.role {
                Role::Receiver { drains, .. }
                    if drains.front().is_some_and(|d| Arc::ptr_eq(d, &c.handle)) =>
                {
                    on_drain = true;
                }
                _ => {
                    // A sender's credit read on its old pipe, cancelled
                    // at the switch: its credits are superseded by the
                    // receiver's first credit on the new pipe.
                    if let Some(i) = end.old.iter().position(|o| Arc::ptr_eq(&o.h, &c.handle)) {
                        if let Some(o) = end.old.get_mut(i) {
                            o.reading = false;
                            if !o.writing {
                                later.push(Box::new(end.old.remove(i)));
                            }
                        }
                    } else {
                        end.reading = false;
                    }
                    later.push(Box::new((c.buf, c.handle)));
                    return later;
                }
            }
        }
        end.reading = false;
        later.push(Box::new(c.handle));
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
                        // The end of a drain: its writer closed it after
                        // its last record (W3). Read the next pipe.
                        _ if on_drain => Next::Drained,
                        // End of the channel: every queued message was
                        // read (probe P9); or any other read failure.
                        _ => Next::EndWith(Item::Closed),
                    }
                }
            }
            Role::Receiver { listener: None, .. } | Role::Region { .. } => Next::Nothing,
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
            Next::Drained => {
                if let Some(End {
                    role: Role::Receiver { drains, .. },
                    ..
                }) = st.ends.get_mut(&id)
                {
                    later.push(Box::new(drains.pop_front()));
                }
            }
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
                    let mut ends = held.ends.iter();
                    let mut got: Vec<Got> = Vec::with_capacity(hs.len());
                    for (index, h) in hs.into_iter().enumerate() {
                        if let Some((_, g)) = held.regions.iter().find(|(i, _)| *i == index) {
                            if g.kind == WireKind::RegionWriter
                                && let Some(e) = b.regions.get_mut(&g.id)
                            {
                                e.state = RState::WriterOut(Some(BROKER));
                            }
                            got.push((Some(h), region_item(0, g), None));
                        } else if let Some((chan, end, k)) = ends.next() {
                            let has_old = *end == PipeEnd::Receiver && b.ledger.has_old(*chan);
                            let item = CtlItem {
                                drains: has_old,
                                ..CtlItem::end(0, *k)
                            };
                            got.push((Some(h), item, Some((*chan, *end))));
                        }
                    }
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

/// A region's control item as the broker hands it over: its exact
/// length, and filler-trust when the broker cannot attest it.
fn region_item(value: u64, g: &Grant) -> CtlItem {
    CtlItem {
        value,
        kind: g.kind,
        drains: false,
        len: g.len,
        flags: if g.filler_trusted {
            item_flags(REGION_FILLER_TRUSTED)
        } else {
            0
        },
    }
}

/// Region flags as a control item carries them (two bits).
fn item_flags(sent: u64) -> u8 {
    u8::try_from(sent & (REGION_IMMUTABLE | REGION_FILLER_TRUSTED)).unwrap_or(0)
}

/// What a region handle knows beyond its kind.
#[derive(Debug)]
struct RegionInfo {
    len: usize,
    immutable: bool,
    want: AtomicBool,
}

/// A channel end this transport made or received.
struct WinEnd {
    inner: Arc<Inner>,
    id: u64,
    kind: HandleKind,
    /// False once the end became a listener or was sent.
    armed: bool,
    /// For a region (D6).
    region: Option<RegionInfo>,
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
    fn region(&self) -> Option<&dyn RegionObject> {
        self.region.is_some().then_some(self as &dyn RegionObject)
    }
}

impl RegionObject for WinEnd {
    fn len(&self) -> usize {
        self.region.as_ref().map_or(0, |r| r.len)
    }
    fn immutable(&self) -> bool {
        self.region.as_ref().is_some_and(|r| r.immutable)
    }
    fn want_immutable(&self) {
        if let Some(r) = &self.region {
            r.want.store(true, Ordering::Relaxed);
        }
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
        self.inner.note_close(&mut st, self.id);
        let gone = Inner::remove(&mut st, self.id);
        drop(st);
        drop(gone);
        self.inner.wake();
    }
}

/// The Windows transport of one process.
pub(crate) struct WindowsIpc {
    inner: Arc<Inner>,
    /// The bootstrap receiver's end ID once taken, and the launcher's
    /// pid: the one receiver `PeerAttest::Inherited` binds (review M7).
    boot_rx: Mutex<Option<(u64, u32)>>,
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
            #[cfg(feature = "measure-launch")]
            crate::launch::windows::timing::mark("ipc_sid");
            let hello = sys::read_blocking(&link, MAX_CTL_LEN, HELLO_TIMEOUT_MS)
                .map_err(|_e| StartError::Bootstrap)?;
            #[cfg(feature = "measure-launch")]
            crate::launch::windows::timing::mark("ipc_hello");
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
            state.broker = Some(Broker::new(ledger, 1));
        }
        let inner = Arc::new(Inner {
            exec,
            state: Mutex::new(state),
            settled: Condvar::new(),
            thread: OnceLock::new(),
            wake_pending: AtomicBool::new(false),
            container,
            #[cfg(test)]
            fail_region_handle: AtomicBool::new(false),
            #[cfg(test)]
            fail_copy_spawn: AtomicBool::new(false),
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
        #[cfg(feature = "measure-launch")]
        crate::launch::windows::timing::mark("ipc_io_thread");
        if inner.lock().broker.is_some() {
            // The worker for the broker's own relays (W3): like a child's,
            // it ends once the transport is gone.
            let (jobs, queue) = mpsc::sync_channel::<Job>(MAX_CHILD_REQUESTS);
            let weak = Arc::downgrade(&inner);
            crate::thread::spawn("hg-ipc-own", move || {
                Inner::worker(&weak, BROKER, None, &queue);
            })
            .map_err(StartError::Os)?;
            if let Some(b) = inner.lock().broker.as_mut() {
                b.own_jobs = Some(jobs);
            }
        }
        Ok(Self {
            inner,
            boot_rx: Mutex::new(None),
        })
    }

    /// The end `h` names, if it is this transport's and held here.
    fn held_end(&self, h: &Handle) -> Option<u64> {
        let e = h.object().downcast_ref::<WinEnd>()?;
        if !Arc::ptr_eq(&e.inner, &self.inner) {
            return None;
        }
        self.inner.lock().ends.contains_key(&e.id).then_some(e.id)
    }

    /// Puts a section handle in the table as a region of `kind` and
    /// returns its handle object.
    fn region_handle(
        &self,
        st: &mut State,
        h: OwnedHandle,
        kind: HandleKind,
        len: u32,
        immutable: bool,
    ) -> Result<Handle, IpcError> {
        #[cfg(test)]
        if self.inner.fail_region_handle.swap(false, Ordering::AcqRel) {
            drop(h);
            return Err(IpcError::Refused);
        }
        let role = Role::Region {
            kind,
            len,
            want: false,
            live: Arc::new(AtomicUsize::new(0)),
        };
        let id = Inner::insert(st, h, role).ok_or(IpcError::Refused)?;
        Ok(Handle::new(Box::new(WinEnd {
            inner: Arc::clone(&self.inner),
            id,
            kind,
            armed: true,
            region: Some(RegionInfo {
                len: usize::try_from(len).unwrap_or(0),
                immutable,
                want: AtomicBool::new(false),
            }),
        })))
    }

    fn handle(&self, id: u64, kind: HandleKind) -> Handle {
        Handle::new(Box::new(WinEnd {
            inner: Arc::clone(&self.inner),
            id,
            kind,
            armed: true,
            region: None,
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
        let take_req = st.req();
        let is_broker = st.broker.is_some();
        let Some(End {
            h,
            role:
                Role::Receiver {
                    flow,
                    bound,
                    listener,
                    old_queued,
                    ..
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
        let rx_facts = (peer.is_some() || *old_queued).then(|| sys::facts(h));
        // W3: the broker holds old pipes of this channel, to read first. A
        // child asks for them and reads nothing until they come.
        let take = core::mem::take(old_queued);
        let mut credit = PendingCredit::default();
        if flow.delivered > 0 {
            // A credit a former holder or an old pipe left stranded is
            // superseded by this one.
            credit.note(Credit {
                capacity: flow.capacity,
                delivered: flow.delivered,
            });
        }
        *listener = Some(Listener {
            seq,
            sink: Arc::new(Mutex::new(sink)),
            queue: VecDeque::new(),
            draining: false,
            ended: false,
            credit,
            bound_peer: peer,
            taking: (take && !is_broker).then_some(take_req),
        });
        if take {
            if is_broker {
                self.inner.take_own_drains(&mut st, id, rx_facts.as_ref());
            } else {
                st.takes.insert(take_req, id);
                self.inner.send_ctl(
                    &mut st,
                    None,
                    &Ctl::TakeDrains {
                        req: take_req,
                        rx: rx_value,
                    },
                );
            }
        }
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
                drains: VecDeque::new(),
                old_queued: false,
            },
        );
        for id in [tx, rx].into_iter().flatten() {
            if let Some(e) = st.ends.get_mut(&id) {
                e.pipe_id = Some(name);
            }
        }
        if let (Some(b), Some(tx), Some(rx)) = (st.broker.as_mut(), tx, rx) {
            let key = PipeKey {
                creator: BROKER,
                id: name,
            };
            b.own.insert(tx, (key, PipeEnd::Sender));
            b.own.insert(rx, (key, PipeEnd::Receiver));
        }
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
        // moves; a bound sender moves once, from its maker (IPC-R2 (a));
        // a writer only from the broker and a producer never (D6).
        let is_broker = st.broker.is_some();
        let mut carried = Vec::with_capacity(ids.len());
        for (&id, h) in ids.iter().zip(&handles) {
            let Some(end) = st.ends.get_mut(&id) else {
                return Err(IpcError::Refused);
            };
            let kind = match &mut end.role {
                Role::Region { kind, want, .. } => {
                    let wire = match kind {
                        HandleKind::RegionWriter if is_broker => WireKind::RegionWriter,
                        HandleKind::ReadOnlyRegion => WireKind::ReadOnlyRegion,
                        HandleKind::ConsumerRegion => WireKind::ConsumerRegion,
                        _ => return Err(IpcError::Refused),
                    };
                    *want = h
                        .object()
                        .downcast_ref::<WinEnd>()
                        .and_then(|e| e.region.as_ref())
                        .is_some_and(|i| i.want.load(Ordering::Relaxed));
                    wire
                }
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
                copying: None,
                prepared: None,
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
                    Role::Link { .. } | Role::Region { .. } => {}
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

    fn allocate_region(&self, len: usize, owner: OsPeer) -> Result<RegionWriter, IpcError> {
        let len = u32::try_from(len).map_err(|_e| IpcError::Refused)?;
        if len == 0 {
            return Err(IpcError::Refused);
        }
        let mut st = self.inner.lock();
        let Some(b) = st.broker.as_mut() else {
            return Err(IpcError::Unsupported);
        };
        let (rid, h) = Inner::register(
            b,
            len,
            owner,
            RState::WriterOut(Some(BROKER)),
            sys::SECTION_RW,
        )?;
        let h = match self.region_handle(&mut st, h, HandleKind::RegionWriter, len, false) {
            Ok(h) => h,
            Err(e) => {
                // No region was given out: forget it and refund it.
                let gone = st.broker.as_mut().and_then(|b| {
                    let e = b.regions.remove(&rid)?;
                    b.quota.refund(owner, u64::from(len));
                    Some(e)
                });
                drop(st);
                drop(gone);
                return Err(e);
            }
        };
        drop(st);
        RegionWriter::from_handle(h).map_err(|_h| IpcError::Refused)
    }

    fn release_owner(&self, owner: OsPeer) {
        let mut st = self.inner.lock();
        let mut gone = Vec::new();
        if let Some(b) = st.broker.as_mut() {
            b.quota.release(owner);
            // The broker forgets the owner's sections; each lives on while
            // a process holds a handle or a view (fact S10).
            let ids: Vec<u64> = b
                .regions
                .iter()
                .filter(|(_, e)| e.owner == owner)
                .map(|(id, _)| *id)
                .collect();
            for id in ids {
                gone.extend(b.regions.remove(&id));
            }
        }
        drop(st);
        drop(gone);
    }

    fn map_region(&self, region: &Handle) -> Result<Mapping, IpcError> {
        let id = self.held_end(region).ok_or(IpcError::Refused)?;
        let st = self.inner.lock();
        let Some(End {
            h,
            role: Role::Region {
                kind, len, live, ..
            },
            ..
        }) = st.ends.get(&id)
        else {
            return Err(IpcError::Refused);
        };
        let writable = match kind {
            HandleKind::RegionWriter => true,
            HandleKind::ReadOnlyRegion | HandleKind::ConsumerRegion => false,
            // A producer holds its one mapping.
            _ => return Err(IpcError::Refused),
        };
        let n = usize::try_from(*len).map_err(|_e| IpcError::Refused)?;
        // In the broker the count is the section's, whichever handle maps.
        let live = st
            .broker
            .as_ref()
            .and_then(|b| Inner::region_id(b, h).and_then(|i| b.regions.get(&i)))
            .map_or_else(|| Arc::clone(live), |e| Arc::clone(&e.live));
        sys::map_section(h, n, writable, Some(live))
            .map(|m| Mapping::new(Box::new(m)))
            .map_err(|_e| IpcError::Refused)
    }

    fn seal_region(&self, writer: RegionWriter) -> Result<ReadOnlyRegion, RegionRefused> {
        let refuse = |w: RegionWriter, e| Err(RegionRefused::new(w.into_handle(), e));
        let Some(id) = self.held_end(writer.handle()) else {
            return refuse(writer, IpcError::Refused);
        };
        let mut st = self.inner.lock();
        let (narrowed, len) = match st.ends.get(&id) {
            Some(End {
                h,
                role:
                    Role::Region {
                        kind: HandleKind::RegionWriter,
                        len,
                        live,
                        ..
                    },
                ..
            }) => {
                let section_live = st
                    .broker
                    .as_ref()
                    .and_then(|b| Inner::region_id(b, h).and_then(|i| b.regions.get(&i)))
                    .is_some_and(|e| e.live.load(Ordering::Acquire) != 0);
                if live.load(Ordering::Acquire) != 0 || section_live {
                    drop(st);
                    return refuse(writer, IpcError::Busy);
                }
                (sys::narrow(h, sys::SECTION_READ), *len)
            }
            _ => {
                drop(st);
                return refuse(writer, IpcError::Refused);
            }
        };
        let Ok(ro) = narrowed else {
            drop(st);
            return refuse(writer, IpcError::Refused);
        };
        // In the broker, which holds the writer and no view of it now,
        // the section is sealed for good: attested immutable.
        let mut sealed = None;
        let sm: &mut State = &mut st;
        if let (Some(b), Some(e)) = (sm.broker.as_mut(), sm.ends.get(&id)) {
            let hid = Inner::region_id(b, &e.h);
            if let Some((i, e)) = hid.and_then(|i| b.regions.get_mut(&i).map(|e| (i, e)))
                && e.state == RState::WriterOut(Some(BROKER))
            {
                e.state = RState::SealedByBroker;
                sealed = Some(i);
            }
        }
        let h = match self.region_handle(
            &mut st,
            ro,
            HandleKind::ReadOnlyRegion,
            len,
            sealed.is_some(),
        ) {
            Ok(h) => h,
            Err(e) => {
                // The writer comes back unsealed: so does the registry.
                if let (Some(b), Some(i)) = (st.broker.as_mut(), sealed)
                    && let Some(e) = b.regions.get_mut(&i)
                {
                    e.state = RState::WriterOut(Some(BROKER));
                }
                drop(st);
                return refuse(writer, e);
            }
        };
        drop(st);
        drop(writer);
        ReadOnlyRegion::from_handle(h).map_err(|h| RegionRefused::new(h, IpcError::Refused))
    }

    fn share_region(&self, writer: RegionWriter) -> Result<ProducerRegion, RegionRefused> {
        let refuse = |w: RegionWriter, e| Err(RegionRefused::new(w.into_handle(), e));
        let Some(id) = self.held_end(writer.handle()) else {
            return refuse(writer, IpcError::Refused);
        };
        let mut st = self.inner.lock();
        let (mapping, dup, len) = match st.ends.get(&id) {
            Some(End {
                h,
                role:
                    Role::Region {
                        kind: HandleKind::RegionWriter,
                        len,
                        ..
                    },
                ..
            }) => {
                let n = usize::try_from(*len).unwrap_or(0);
                (
                    sys::map_section(h, n, true, None),
                    sys::narrow(h, sys::SECTION_RW),
                    *len,
                )
            }
            _ => {
                drop(st);
                return refuse(writer, IpcError::Refused);
            }
        };
        let (Ok(mapping), Ok(dup)) = (mapping, dup) else {
            drop(st);
            return refuse(writer, IpcError::Refused);
        };
        let sm: &mut State = &mut st;
        if let (Some(b), Some(e)) = (sm.broker.as_mut(), sm.ends.get(&id))
            && let Some(e) = Inner::region_id(b, &e.h).and_then(|i| b.regions.get_mut(&i))
            && e.state == RState::WriterOut(Some(BROKER))
        {
            e.state = RState::Producer(BROKER);
        }
        let h = match self.region_handle(&mut st, dup, HandleKind::ProducerRegion, len, false) {
            Ok(h) => h,
            Err(e) => {
                drop(st);
                return refuse(writer, e);
            }
        };
        drop(st);
        drop(writer);
        ProducerRegion::new(h, Mapping::new(Box::new(mapping)))
            .map_err(|(h, _m)| RegionRefused::new(h, IpcError::Refused))
    }

    fn consumer_of(&self, producer: &ProducerRegion) -> Result<ConsumerRegion, IpcError> {
        let id = self.held_end(producer.handle()).ok_or(IpcError::Refused)?;
        let mut st = self.inner.lock();
        let (ro, len) = match st.ends.get(&id) {
            Some(End {
                h,
                role:
                    Role::Region {
                        kind: HandleKind::ProducerRegion,
                        len,
                        ..
                    },
                ..
            }) => (
                sys::narrow(h, sys::SECTION_READ).map_err(|_e| IpcError::Refused)?,
                *len,
            ),
            _ => return Err(IpcError::Refused),
        };
        let h = self.region_handle(&mut st, ro, HandleKind::ConsumerRegion, len, false)?;
        drop(st);
        ConsumerRegion::from_handle(h).map_err(|_h| IpcError::Refused)
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
        peer: PeerAttest,
        seq: SequenceHandle,
        sink: Sink,
    ) -> Result<Listening, IpcError> {
        let boot = *self.boot_rx.lock().unwrap_or_else(PoisonError::into_inner);
        let is_boot = boot.is_some_and(|(id, _p)| self.held_end(from.handle()) == Some(id));
        let pid = match attest::resolve(peer, is_boot) {
            None => return Err(IpcError::Refused),
            // The broker's link attests the launcher: the inherited binding
            // checks it as well (no PID namespace on Windows).
            Some(attest::Bound::Inherited) => match boot {
                Some((_id, p)) => p,
                None => return Err(IpcError::Refused),
            },
            Some(attest::Bound::Peer(p)) => p.pid(),
        };
        if pid == 0 {
            return Err(IpcError::Refused);
        }
        self.start_listening(from, Some(pid), seq, sink)
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
                drains: VecDeque::new(),
                old_queued: false,
            },
        );
        drop(st);
        self.inner.wake();
        let to_parent =
            ChannelSender::from_handle(self.handle(tx?, HandleKind::ChannelSender)).ok()?;
        let from_parent =
            ChannelReceiver::from_handle(self.handle(rx?, HandleKind::ChannelReceiver)).ok()?;
        if let Some(id) = rx {
            *self.boot_rx.lock().unwrap_or_else(PoisonError::into_inner) = Some((id, parent_pid));
        }
        Some(Bootstrap {
            to_parent,
            from_parent,
            parent: PeerAttest::Os(OsPeer::new(parent_pid, 0)),
        })
    }
}

/// A child the broker started: its process, its launch channel ends on
/// the broker's side, and its identity.
#[derive(Debug)]
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
pub(crate) struct LaunchSpec<'a> {
    /// The executable.
    pub(crate) exe: &'a std::path::Path,
    /// Its command line after the path.
    pub(crate) args: &'a str,
    /// Environment variables added for it; the link handle's value is
    /// added as `IZCAL_IPC_LINK`.
    pub(crate) env: Vec<(String, String)>,
    /// Its sandbox (review M9: there is no unsandboxed launch outside the
    /// transport's own tests).
    pub(crate) sandbox: Sandbox<'a>,
}

/// How a child is confined.
pub(crate) enum Sandbox<'a> {
    /// Its own AppContainer (with that container's SID), the lockdown
    /// token, and its job (assigned while it is suspended).
    Contained {
        /// The container.
        container: &'a crate::sandbox::windows::AppContainerSid,
        /// The lockdown token.
        token: &'a crate::sandbox::windows::OwnedHandle,
        /// The job.
        job: &'a crate::sandbox::windows::OwnedHandle,
    },
    /// Started plainly: the transport's own tests only (review M9).
    #[cfg(test)]
    Plain,
}

/// The parts of a sandbox, as the process start takes them.
type SandboxParts<'a> = Option<(
    &'a crate::sandbox::windows::AppContainerSid,
    &'a crate::sandbox::windows::OwnedHandle,
    &'a crate::sandbox::windows::OwnedHandle,
)>;

impl<'a> Sandbox<'a> {
    const fn parts(&self) -> SandboxParts<'a> {
        match *self {
            Self::Contained {
                container,
                token,
                job,
            } => Some((container, token, job)),
            #[cfg(test)]
            Self::Plain => None,
        }
    }
}

/// The environment variable that carries a child's link handle value.
pub(crate) const LINK_VAR: &str = "IZCAL_IPC_LINK";

impl WindowsIpc {
    /// Starts a child with its control link and launch channels; this
    /// process must be a broker. The child reads its `Hello` when its
    /// transport starts.
    pub(crate) fn launch(&self, spec: LaunchSpec<'_>) -> std::io::Result<Launched> {
        let name = || {
            Inner::random16().ok_or_else(|| std::io::Error::other("no randomness for a pipe name"))
        };
        let (link_server, link_client) = sys::link_pair(&name()?)?;
        let (up_id, down_id) = (name()?, name()?);
        let (up_rx, up_tx) = sys::pipe_pair(None, &up_id)?;
        let (down_rx, down_tx) = sys::pipe_pair(None, &down_id)?;
        #[cfg(feature = "measure-launch")]
        crate::launch::windows::timing::mark("pipes");
        // The launch channels are the broker's: keyed by their names.
        let up_key = PipeKey {
            creator: BROKER,
            id: up_id,
        };
        let down_key = PipeKey {
            creator: BROKER,
            id: down_id,
        };
        let inherit = sys::inheritable(&link_client)?;
        let mut env = spec.env;
        env.push((LINK_VAR.to_owned(), sys::value(&inherit).to_string()));
        // A sandboxed launch must have a SID to key its channels and to
        // enforce one container per child; without one, refuse before
        // spawning (fail closed).
        let parts = spec.sandbox.parts();
        let sid = match parts {
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
            sandbox: parts,
        }) {
            Ok(c) => c,
            Err(e) => {
                release(&self.inner);
                return Err(e);
            }
        };
        drop(inherit);
        drop(link_client);
        #[cfg(feature = "measure-launch")]
        crate::launch::windows::timing::mark("spawned");
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
        drop((up_tx, down_rx));
        #[cfg(feature = "measure-launch")]
        crate::launch::windows::timing::mark("given");
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
        // The child's launch ends are given like any other, so its Closed
        // releases them and a move of their peers can name them (W3).
        for (k, end, v) in [
            (up_key, PipeEnd::Sender, v_up),
            (down_key, PipeEnd::Receiver, v_down),
        ] {
            let _ = b.ledger.place(k, end, p);
            b.grant(p, v, k, end);
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
        // The child's worker (M8), started before the child can send a
        // request. It has its own copy of the process handle.
        let regions = sys::own_integrity().is_some_and(|own| sys::may_hold_regions(&process, own));
        let (jobs, queue) = mpsc::sync_channel(MAX_CHILD_REQUESTS);
        let worker = sys::duplicate(&process.0)
            .ok()
            .map(sys::Process)
            .and_then(|wp| {
                let weak = Arc::downgrade(&self.inner);
                crate::thread::spawn("hg-ipc-child", move || {
                    Inner::worker(&weak, p, Some(&wp), &queue);
                })
                .ok()
            });
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
                    regions,
                    process,
                    sid,
                    link,
                    jobs,
                },
            );
        }
        if worker.is_none() {
            let dropped = self.inner.abandon(&mut st, p, [None, None]);
            drop(st);
            drop(dropped);
            sys::spawn::kill(&child);
            return Err(std::io::Error::other("no worker thread for the child"));
        }
        #[cfg(feature = "measure-launch")]
        crate::launch::windows::timing::mark("worker");
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
                drains: VecDeque::new(),
                old_queued: false,
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
        if let Some(b) = st.broker.as_mut() {
            b.own.insert(rx, (up_key, PipeEnd::Receiver));
            b.own.insert(tx, (down_key, PipeEnd::Sender));
        }
        drop(st);
        self.inner.wake();
        #[cfg(feature = "measure-launch")]
        crate::launch::windows::timing::mark("hello");
        if let Err(e) = sys::spawn::resume(&child) {
            let mut st = self.inner.lock();
            let dropped = self.inner.abandon(&mut st, p, [Some(rx), Some(tx)]);
            drop(st);
            drop(dropped);
            sys::spawn::kill(&child);
            return Err(e);
        }
        #[cfg(feature = "measure-launch")]
        crate::launch::windows::timing::mark("resumed");
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
