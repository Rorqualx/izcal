//! The broker's ledger (IPC-R1, R2; design review B1, B3, M10): which
//! process holds each channel end, the handles held under each ticket,
//! and the receivers bound to one peer.
//!
//! Every handle that moves between processes passes through the broker
//! ([`super::accept`]), and the broker refuses a move that leaves a copy
//! behind, so the ledger knows every end's holder exactly: an end the
//! ledger has never seen is still with its creator. That is what Windows
//! can attest in place of a per-message sender: pipes carry no sender
//! identity (probe P13), but the one process that holds a channel's
//! sender end is the only one that can write to it.
//!
//! A relay moves the handles into a ticket, held by the broker; a
//! redemption moves them to the redeeming process, which must hold the
//! receiver end of the channel the ticket was issued for. A receiver
//! bound to a peer ([`Ledger::bind`]) is watched: whenever its channel's
//! sender end comes to rest with any process but the peer (or its
//! creator, before the grant), the binder is told
//! ([`BindEvent`]), so its transport ends the channel with an Identity
//! fault naming the peer, the only process that could have passed the
//! sender on.
//!
//! Keyed by the pipe's creator and name, never by a handle the broker
//! holds: holding ends would keep them open, and a receiver would never
//! see its channel close (design review M10).

use std::collections::BTreeMap;

use super::accept::{End, PipeKey, Proc, View};
use super::frame::Ticket;
use crate::ipc::common::codec::WireKind;

/// The most tickets outstanding on one carrier channel (design review
/// B1: per channel, not per process, so a channel a peer never reads
/// cannot pin another's relays). Flow control already caps records in
/// flight per channel at the channel's capacity; this is the broker's
/// backstop above that. The broker does not track per-channel capacity,
/// so a fixed bound stands in for B1's capacity x 16.
pub(crate) const MAX_TICKETS_PER_CHANNEL: usize = 1024;

/// The most tickets one process may have outstanding across all its
/// carriers (the per-child total, to bound the broker's memory).
pub(crate) const MAX_TICKETS_PER_PROC: usize = 16_384;

/// The most channels the ledger tracks for one creator.
pub(crate) const MAX_PIPES_PER_PROC: usize = 16_384;

/// Who holds an end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Holder {
    /// A process.
    Proc(Proc),
    /// The broker, under a ticket.
    Ticket(Ticket),
    /// Its holder ended.
    Gone,
}

/// A receiver bound to one peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Binding {
    /// The process holding the receiver.
    binder: Proc,
    /// The binder's never-reused id for the listener, to name it in
    /// events (not a handle value, which can be reused: item 8).
    rx_id: u64,
    /// The peer's process ID.
    peer_pid: u32,
    /// Already reported broken.
    broken: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    sender: Holder,
    receiver: Holder,
    /// Its sender was granted as a bound sender.
    bound: bool,
    binding: Option<Binding>,
}

/// One relay's handles, as the broker holds them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Held {
    /// The channel whose record carries the ticket.
    pub(crate) carrier: PipeKey,
    /// The relaying process.
    pub(crate) from: Proc,
    /// The ends, in the record's descriptor order, with the kind each
    /// travels as.
    pub(crate) ends: Vec<(PipeKey, End, WireKind)>,
}

/// A bound receiver whose sender has left its peer: the binder's
/// transport ends that receiver with an Identity fault naming `peer_pid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BindEvent {
    /// The process to tell.
    pub(crate) binder: Proc,
    /// The binder's never-reused listener id.
    pub(crate) rx_id: u64,
    /// The peer it was bound to.
    pub(crate) peer_pid: u32,
}

/// Why a redemption or binding was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LedgerError {
    /// No such ticket (never issued, already redeemed, or released).
    NoTicket,
    /// The redeemer's receiver is not the ticket's channel's.
    WrongChannel,
    /// The process does not hold the receiver end it names.
    NotHolder,
    /// Too many tickets outstanding, or channels tracked.
    Quota,
}

impl LedgerError {
    /// The code a refusal record carries.
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::NoTicket => 1,
            Self::WrongChannel => 2,
            Self::NotHolder => 3,
            Self::Quota => 4,
        }
    }
}

/// The broker's ledger.
#[derive(Debug, Default)]
pub(crate) struct Ledger {
    entries: BTreeMap<PipeKey, Entry>,
    tickets: BTreeMap<Ticket, Held>,
    pids: BTreeMap<Proc, u32>,
    outstanding: BTreeMap<Proc, usize>,
    by_carrier: BTreeMap<PipeKey, usize>,
    per_creator: BTreeMap<Proc, usize>,
}

impl Ledger {
    /// Records process `p` with OS process ID `pid` (the broker itself
    /// is process 0).
    pub(crate) fn add_proc(&mut self, p: Proc, pid: u32) {
        self.pids.insert(p, pid);
    }

    fn entry(&self, pipe: PipeKey) -> Entry {
        self.entries.get(&pipe).copied().unwrap_or(Entry {
            sender: Holder::Proc(pipe.creator),
            receiver: Holder::Proc(pipe.creator),
            bound: false,
            binding: None,
        })
    }

    fn put(&mut self, pipe: PipeKey, e: Entry) -> Result<(), LedgerError> {
        if !self.entries.contains_key(&pipe) {
            let n = self.per_creator.entry(pipe.creator).or_insert(0);
            if *n >= MAX_PIPES_PER_PROC {
                return Err(LedgerError::Quota);
            }
            *n = n.saturating_add(1);
        }
        self.entries.insert(pipe, e);
        Ok(())
    }

    /// What [`super::accept::accept`] needs to know about `end` of
    /// `pipe`, or `None` for a channel never seen.
    pub(crate) fn view(&self, pipe: PipeKey, end: End) -> Option<View> {
        let e = self.entries.get(&pipe)?;
        let holder = match if end == End::Sender {
            e.sender
        } else {
            e.receiver
        } {
            Holder::Proc(p) => Some(p),
            Holder::Ticket(_) | Holder::Gone => None,
        };
        Some(View {
            holder,
            bound: e.bound,
            bound_receiver: e.binding.is_some(),
        })
    }

    /// Whether `ticket` is free to issue.
    pub(crate) fn ticket_free(&self, ticket: &Ticket) -> bool {
        !self.tickets.contains_key(ticket)
    }

    /// Moves accepted ends into `ticket`.
    ///
    /// # Errors
    /// [`LedgerError::Quota`] when `from` has too many tickets
    /// outstanding; nothing changes.
    pub(crate) fn relay(&mut self, ticket: Ticket, held: Held) -> Result<(), LedgerError> {
        let n = self.outstanding.get(&held.from).copied().unwrap_or(0);
        let per_channel = self.by_carrier.get(&held.carrier).copied().unwrap_or(0);
        if n >= MAX_TICKETS_PER_PROC
            || per_channel >= MAX_TICKETS_PER_CHANNEL
            || !self.room_for(&held.ends)
        {
            return Err(LedgerError::Quota);
        }
        // Folded per channel: a relay may carry both ends of one.
        let mut updates: BTreeMap<PipeKey, Entry> = BTreeMap::new();
        for &(pipe, end, kind) in &held.ends {
            let mut e = updates
                .get(&pipe)
                .copied()
                .unwrap_or_else(|| self.entry(pipe));
            match end {
                End::Sender => e.sender = Holder::Ticket(ticket),
                End::Receiver => e.receiver = Holder::Ticket(ticket),
            }
            if kind == WireKind::BoundSender {
                e.bound = true;
            }
            updates.insert(pipe, e);
        }
        for (pipe, e) in updates {
            self.put(pipe, e)?;
        }
        self.outstanding.insert(held.from, n.saturating_add(1));
        let c = self.by_carrier.entry(held.carrier).or_insert(0);
        *c = c.saturating_add(1);
        self.tickets.insert(ticket, held);
        Ok(())
    }

    /// Whether the channels among `ends` the ledger has not seen fit
    /// their creators' quotas.
    fn room_for(&self, ends: &[(PipeKey, End, WireKind)]) -> bool {
        let mut new: BTreeMap<Proc, usize> = BTreeMap::new();
        for (pipe, _, _) in ends {
            if !self.entries.contains_key(pipe) {
                let n = new.entry(pipe.creator).or_insert(0);
                *n = n.saturating_add(1);
            }
        }
        new.iter().all(|(c, n)| {
            self.per_creator
                .get(c)
                .copied()
                .unwrap_or(0)
                .saturating_add(*n)
                <= MAX_PIPES_PER_PROC
        })
    }

    /// Records that `end` of `pipe` was placed with process `to` without
    /// a relay: a launch channel the broker made and put in a child's
    /// table itself.
    ///
    /// # Errors
    /// [`LedgerError::Quota`].
    pub(crate) fn place(&mut self, pipe: PipeKey, end: End, to: Proc) -> Result<(), LedgerError> {
        let mut e = self.entry(pipe);
        match end {
            End::Sender => e.sender = Holder::Proc(to),
            End::Receiver => e.receiver = Holder::Proc(to),
        }
        self.put(pipe, e)
    }

    /// Redeems `ticket` for process `by`, which read it from its
    /// receiver end of channel `rx`: the ends become `by`'s. Returns them
    /// and the bindings that broke.
    ///
    /// # Errors
    /// [`LedgerError::NoTicket`], [`LedgerError::WrongChannel`],
    /// [`LedgerError::NotHolder`]; the ticket then stays.
    pub(crate) fn redeem(
        &mut self,
        by: Proc,
        ticket: &Ticket,
        rx: PipeKey,
    ) -> Result<(Held, Vec<BindEvent>), LedgerError> {
        let held = self.tickets.get(ticket).ok_or(LedgerError::NoTicket)?;
        if held.carrier != rx {
            return Err(LedgerError::WrongChannel);
        }
        if self.entry(rx).receiver != Holder::Proc(by) {
            return Err(LedgerError::NotHolder);
        }
        let held = self.tickets.remove(ticket).ok_or(LedgerError::NoTicket)?;
        self.release_counts(&held);
        let mut events = Vec::new();
        for &(pipe, end, _) in &held.ends {
            let mut e = self.entry(pipe);
            match end {
                End::Sender => e.sender = Holder::Proc(by),
                End::Receiver => e.receiver = Holder::Proc(by),
            }
            if end == End::Sender
                && let Some(b) = e.binding.as_mut()
                && !b.broken
                && self.pids.get(&by) != Some(&b.peer_pid)
            {
                b.broken = true;
                events.push(BindEvent {
                    binder: b.binder,
                    rx_id: b.rx_id,
                    peer_pid: b.peer_pid,
                });
            }
            // An entry that already existed cannot hit the quota.
            let _ = self.put(pipe, e);
        }
        Ok((held, events))
    }

    fn release_counts(&mut self, held: &Held) {
        if let Some(n) = self.outstanding.get_mut(&held.from) {
            *n = n.saturating_sub(1);
        }
        if let Some(n) = self.by_carrier.get_mut(&held.carrier) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                self.by_carrier.remove(&held.carrier);
            }
        }
    }

    /// Binds the receiver of `pipe`, which process `binder` holds as
    /// handle value `rx`, to the process with ID `peer_pid`. Returns the
    /// event at once if the sender end already rests elsewhere.
    ///
    /// # Errors
    /// [`LedgerError::NotHolder`] if `binder` does not hold the receiver.
    pub(crate) fn bind(
        &mut self,
        binder: Proc,
        pipe: PipeKey,
        rx_id: u64,
        peer_pid: u32,
    ) -> Result<Option<BindEvent>, LedgerError> {
        let mut e = self.entry(pipe);
        if e.receiver != Holder::Proc(binder) {
            return Err(LedgerError::NotHolder);
        }
        let consistent = match e.sender {
            // Still with the binder or the channel's creator (before the
            // grant), in flight, or with the peer.
            Holder::Proc(p) => {
                p == binder || p == pipe.creator || self.pids.get(&p) == Some(&peer_pid)
            }
            Holder::Ticket(_) | Holder::Gone => true,
        };
        e.binding = Some(Binding {
            binder,
            rx_id,
            peer_pid,
            broken: !consistent,
        });
        self.put(pipe, e)?;
        Ok((!consistent).then_some(BindEvent {
            binder,
            rx_id,
            peer_pid,
        }))
    }

    /// Drops the binding `binder` holds as listener `rx_id`, because its
    /// receiver closed (item 8). A no-op if there is no such binding.
    pub(crate) fn unbind(&mut self, binder: Proc, rx_id: u64) {
        for e in self.entries.values_mut() {
            if e.binding
                .is_some_and(|b| b.binder == binder && b.rx_id == rx_id)
            {
                e.binding = None;
            }
        }
    }

    /// Process `p` ended: the ends it held are gone, the tickets only it
    /// could redeem are released (their handles closed by the caller),
    /// and its bindings are dropped. Returns the released tickets.
    pub(crate) fn gone(&mut self, p: Proc) -> Vec<Ticket> {
        // Tickets only `p` could redeem: their channel's receiver is
        // `p`'s (or already gone).
        let dead: Vec<Ticket> = self
            .tickets
            .iter()
            .filter(|(_, h)| {
                matches!(self.entry(h.carrier).receiver, Holder::Gone)
                    || self.entry(h.carrier).receiver == Holder::Proc(p)
            })
            .map(|(t, _)| *t)
            .collect();
        // The broker closes the ends a dropped ticket held, so they are
        // gone too, and their channels can be swept.
        let lost =
            |h: Holder| h == Holder::Proc(p) || matches!(h, Holder::Ticket(t) if dead.contains(&t));
        for e in self.entries.values_mut() {
            if lost(e.sender) {
                e.sender = Holder::Gone;
            }
            if lost(e.receiver) {
                e.receiver = Holder::Gone;
            }
            if e.binding.is_some_and(|b| b.binder == p) {
                e.binding = None;
            }
        }
        for t in &dead {
            if let Some(h) = self.tickets.remove(t) {
                self.release_counts(&h);
            }
        }
        self.pids.remove(&p);
        self.outstanding.remove(&p);
        dead
    }

    /// Ends the broker failed to hand over after a redemption: it took
    /// them back and closed them, so they are gone.
    pub(crate) fn lost(&mut self, ends: &[(PipeKey, End, WireKind)]) {
        for &(pipe, end, _) in ends {
            if let Some(e) = self.entries.get_mut(&pipe) {
                match end {
                    End::Sender => e.sender = Holder::Gone,
                    End::Receiver => e.receiver = Holder::Gone,
                }
            }
        }
    }

    /// Process `by` closed its `end` of `pipe` normally: mark it gone, so
    /// a following [`Ledger::sweep`] can forget the channel and give its
    /// creator a slot back (design review items 2, 3). Only the holder's
    /// close counts; a stale report about an end held by someone else is
    /// ignored.
    pub(crate) fn close_end(&mut self, pipe: PipeKey, end: End, by: Proc) {
        if let Some(e) = self.entries.get_mut(&pipe) {
            let slot = match end {
                End::Sender => &mut e.sender,
                End::Receiver => &mut e.receiver,
            };
            if *slot == Holder::Proc(by) {
                *slot = Holder::Gone;
            }
        }
    }

    /// Forgets channels whose two ends are both gone. Returns the
    /// creators left with no channels, so the broker can drop a dead
    /// creator's name map (item 5).
    pub(crate) fn sweep(&mut self) -> Vec<Proc> {
        let gone: Vec<PipeKey> = self
            .entries
            .iter()
            .filter(|(_, e)| e.sender == Holder::Gone && e.receiver == Holder::Gone)
            .map(|(k, _)| *k)
            .collect();
        let mut emptied = Vec::new();
        for k in gone {
            self.entries.remove(&k);
            if let Some(n) = self.per_creator.get_mut(&k.creator) {
                *n = n.saturating_sub(1);
                if *n == 0 {
                    self.per_creator.remove(&k.creator);
                    emptied.push(k.creator);
                }
            }
        }
        emptied
    }

    /// Whether the ledger still tracks any channel creator `p` made.
    pub(crate) fn has_creator(&self, p: Proc) -> bool {
        self.per_creator.get(&p).is_some_and(|n| *n > 0)
    }

    /// Tickets outstanding.
    #[cfg(test)]
    pub(crate) fn tickets_outstanding(&self) -> usize {
        self.tickets.len()
    }

    /// Channels the ledger still tracks.
    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Bindings process `binder` holds.
    #[cfg(test)]
    pub(crate) fn bindings_by(&self, binder: Proc) -> usize {
        self.entries
            .values()
            .filter(|e| e.binding.is_some_and(|b| b.binder == binder))
            .count()
    }
}

/// A fixed-size model of the ledger, for its Kani proofs (VER-1, D5 item
/// 12).
///
/// Kani cannot check [`Ledger`] itself: CBMC unrolls each loop inside
/// `BTreeMap`'s search, insertion and removal to the proof's full bound,
/// because their trip counts depend on node lengths held in memory.
/// Measured on hueyj: one `place` on a concrete key grew from 0.40 million
/// steps at bound 6 to 2.46 million at bound 14 and did not finish
/// symbolic execution at bound 18; a relay and redemption ran out of
/// 10 GiB. Symbolic keys cost no more than concrete ones; the map's loops
/// and the number of map calls do.
///
/// The model follows the ledger's code step by step over fixed arrays
/// indexed by small numbers: [`PIPES`] channels (creators 0, 1 and 2,
/// two ids each, the domain of the earlier harnesses), [`PROCS`]
/// processes (0 the broker) and [`TICKETS`] tickets, each relay carrying
/// one end. Every loop runs over a constant range, so CBMC unrolls it
/// exactly. Quotas are left out: no sequence in this domain comes near
/// them, and unit tests cover them on the ledger. The test
/// `the_model_agrees_with_the_ledger` runs long random sequences of every
/// operation here on both and compares results and every observable
/// value after each step.
#[cfg(any(test, kani))]
mod model {
    use super::End;

    pub(super) const PIPES: usize = 6;
    pub(super) const PROCS: usize = 3;
    pub(super) const TICKETS: usize = 2;

    /// Channel `i`'s creator.
    pub(super) fn creator(i: usize) -> usize {
        i / 2
    }

    /// Who holds an end, as [`super::Holder`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum H {
        Proc(usize),
        Ticket(usize),
        Gone,
    }

    /// One relay: its carrier, relayer and the one end it carries.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) struct T {
        pub(super) carrier: usize,
        pub(super) from: usize,
        pub(super) pipe: usize,
        pub(super) end: End,
    }

    /// Why a redemption was refused, as [`super::LedgerError`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum E {
        NoTicket,
        WrongChannel,
        NotHolder,
    }

    #[derive(Debug, Clone, Copy)]
    pub(super) struct Model {
        known: [bool; PIPES],
        sender: [H; PIPES],
        receiver: [H; PIPES],
        tickets: [Option<T>; TICKETS],
        outstanding: [usize; PROCS],
        by_carrier: [usize; PIPES],
        per_creator: [usize; PROCS],
    }

    impl Default for Model {
        fn default() -> Self {
            Self {
                known: [false; PIPES],
                sender: [H::Gone; PIPES],
                receiver: [H::Gone; PIPES],
                tickets: [None; TICKETS],
                outstanding: [0; PROCS],
                by_carrier: [0; PIPES],
                per_creator: [0; PROCS],
            }
        }
    }

    fn at<X: Copy>(a: &[X], i: usize, or: X) -> X {
        a.get(i).copied().unwrap_or(or)
    }

    fn set<X>(a: &mut [X], i: usize, v: X) {
        if let Some(x) = a.get_mut(i) {
            *x = v;
        }
    }

    fn inc(a: &mut [usize], i: usize) {
        if let Some(x) = a.get_mut(i) {
            *x = x.saturating_add(1);
        }
    }

    fn dec(a: &mut [usize], i: usize) {
        if let Some(x) = a.get_mut(i) {
            *x = x.saturating_sub(1);
        }
    }

    impl Model {
        /// The ends' holders, as `Ledger::entry`: an unseen channel's
        /// ends are its creator's.
        fn entry(&self, i: usize) -> (H, H) {
            if at(&self.known, i, false) {
                (at(&self.sender, i, H::Gone), at(&self.receiver, i, H::Gone))
            } else {
                (H::Proc(creator(i)), H::Proc(creator(i)))
            }
        }

        /// As `Ledger::put`, without the quota.
        fn put(&mut self, i: usize, (tx, rx): (H, H)) {
            if !at(&self.known, i, false) {
                let c = creator(i);
                inc(&mut self.per_creator, c);
                set(&mut self.known, i, true);
            }
            set(&mut self.sender, i, tx);
            set(&mut self.receiver, i, rx);
        }

        fn with_end((tx, rx): (H, H), end: End, h: H) -> (H, H) {
            match end {
                End::Sender => (h, rx),
                End::Receiver => (tx, h),
            }
        }

        pub(super) fn place(&mut self, i: usize, end: End, to: usize) {
            let e = Self::with_end(self.entry(i), end, H::Proc(to));
            self.put(i, e);
        }

        /// As `Ledger::relay` of one end; the ledger replaces a ticket
        /// already outstanding under the same value, and so does this.
        pub(super) fn relay(&mut self, t: usize, held: T) {
            let e = Self::with_end(self.entry(held.pipe), held.end, H::Ticket(t));
            self.put(held.pipe, e);
            inc(&mut self.outstanding, held.from);
            inc(&mut self.by_carrier, held.carrier);
            set(&mut self.tickets, t, Some(held));
        }

        fn release_counts(&mut self, held: T) {
            dec(&mut self.outstanding, held.from);
            dec(&mut self.by_carrier, held.carrier);
        }

        pub(super) fn redeem(&mut self, by: usize, t: usize, rx: usize) -> Result<T, E> {
            let held = at(&self.tickets, t, None).ok_or(E::NoTicket)?;
            if held.carrier != rx {
                return Err(E::WrongChannel);
            }
            if self.entry(rx).1 != H::Proc(by) {
                return Err(E::NotHolder);
            }
            set(&mut self.tickets, t, None);
            self.release_counts(held);
            let e = Self::with_end(self.entry(held.pipe), held.end, H::Proc(by));
            self.put(held.pipe, e);
            Ok(held)
        }

        /// As `Ledger::gone`; returns which tickets it released.
        pub(super) fn gone(&mut self, p: usize) -> [bool; TICKETS] {
            let mut dead = [false; TICKETS];
            for (t, d) in dead.iter_mut().enumerate() {
                if let Some(h) = at(&self.tickets, t, None) {
                    let rx = self.entry(h.carrier).1;
                    *d = rx == H::Gone || rx == H::Proc(p);
                }
            }
            let lost = |h: H| match h {
                H::Proc(q) => q == p,
                H::Ticket(t) => at(&dead, t, false),
                H::Gone => false,
            };
            for i in 0..PIPES {
                if at(&self.known, i, false) {
                    if lost(at(&self.sender, i, H::Gone)) {
                        set(&mut self.sender, i, H::Gone);
                    }
                    if lost(at(&self.receiver, i, H::Gone)) {
                        set(&mut self.receiver, i, H::Gone);
                    }
                }
            }
            for (t, d) in dead.iter().enumerate() {
                if *d && let Some(h) = at(&self.tickets, t, None) {
                    set(&mut self.tickets, t, None);
                    self.release_counts(h);
                }
            }
            set(&mut self.outstanding, p, 0);
            dead
        }

        pub(super) fn close_end(&mut self, i: usize, end: End, by: usize) {
            if at(&self.known, i, false) {
                let (tx, rx) = self.entry(i);
                let slot = if end == End::Sender { tx } else { rx };
                if slot == H::Proc(by) {
                    let e = Self::with_end((tx, rx), end, H::Gone);
                    self.put(i, e);
                }
            }
        }

        /// As `Ledger::sweep`; returns which creators it emptied.
        pub(super) fn sweep(&mut self) -> [bool; PROCS] {
            let mut emptied = [false; PROCS];
            for i in 0..PIPES {
                if at(&self.known, i, false)
                    && at(&self.sender, i, H::Proc(0)) == H::Gone
                    && at(&self.receiver, i, H::Proc(0)) == H::Gone
                {
                    set(&mut self.known, i, false);
                    let c = creator(i);
                    let n = at(&self.per_creator, c, 0);
                    if n > 0 {
                        set(&mut self.per_creator, c, n.saturating_sub(1));
                        if n == 1 {
                            set(&mut emptied, c, true);
                        }
                    }
                }
            }
            emptied
        }

        /// As `Ledger::view`'s holder: `None` for an unseen channel,
        /// `Some(None)` for an end in a ticket or gone.
        pub(super) fn holder(&self, i: usize, end: End) -> Option<Option<usize>> {
            if !at(&self.known, i, false) {
                return None;
            }
            let (tx, rx) = self.entry(i);
            match if end == End::Sender { tx } else { rx } {
                H::Proc(p) => Some(Some(p)),
                H::Ticket(_) | H::Gone => Some(None),
            }
        }

        pub(super) fn ticket_free(&self, t: usize) -> bool {
            at(&self.tickets, t, None).is_none()
        }

        pub(super) fn tickets_outstanding(&self) -> usize {
            self.tickets.iter().filter(|t| t.is_some()).count()
        }

        pub(super) fn has_creator(&self, c: usize) -> bool {
            at(&self.per_creator, c, 0) > 0
        }

        pub(super) fn by_carrier(&self, i: usize) -> usize {
            at(&self.by_carrier, i, 0)
        }

        pub(super) fn outstanding(&self, p: usize) -> usize {
            at(&self.outstanding, p, 0)
        }
    }
}

/// VER-1 for the broker's ledger (IPC-R1), over [`model`]: a relay and its
/// redemption move an end from the relayer to the redeemer and leave the
/// ticket counters empty, and `gone` plus `sweep` reclaim what a dead
/// process held. The same three properties as before, over the same
/// domain (three processes, six channels); see `model` for why the proofs
/// run on the model and how it is held to the ledger.
#[cfg(kani)]
mod proofs {
    use super::End;
    use super::model::{E, Model, PIPES, PROCS, T, creator};

    fn any_proc() -> usize {
        kani::any_where(|&p: &usize| p < PROCS)
    }

    fn any_pipe() -> usize {
        kani::any_where(|&i: &usize| i < PIPES)
    }

    fn any_end() -> End {
        if kani::any() {
            End::Sender
        } else {
            End::Receiver
        }
    }

    /// A relay of one end then a redemption on its carrier moves the end
    /// to the redeemer and leaves no ticket behind.
    #[kani::proof]
    #[kani::unwind(8)]
    fn relay_then_redeem_moves_one_end() {
        let mut l = Model::default();
        let carrier = any_pipe();
        let moved = any_pipe();
        let from = any_proc();
        let by = any_proc();
        let end = any_end();
        // The redeemer holds the carrier's receiver.
        l.place(carrier, End::Receiver, by);
        kani::assume(moved != carrier);
        l.relay(
            0,
            T {
                carrier,
                from,
                pipe: moved,
                end,
            },
        );
        // While held under the ticket, the moved end has no live holder.
        assert!(l.holder(moved, end) == Some(None));
        assert!(!l.ticket_free(0));
        match l.redeem(by, 0, carrier) {
            Ok(held) => {
                assert!(held.pipe == moved && held.end == end);
                assert!(l.holder(moved, end) == Some(Some(by)));
                assert!(l.ticket_free(0));
                assert!(l.tickets_outstanding() == 0);
                kani::cover!(end == End::Receiver, "a receiver end reaches the redeemer");
                kani::cover!(end == End::Sender, "a sender end reaches the redeemer");
            }
            Err(e) => {
                // Only a refused redemption keeps the ticket.
                assert!(e != E::NoTicket);
                assert!(!l.ticket_free(0));
            }
        }
    }

    /// A relay counts one ticket against its carrier, and redeeming a
    /// fresh ledger's only ticket empties the per-carrier count, so the
    /// carrier has room again.
    #[kani::proof]
    #[kani::unwind(8)]
    fn a_redeemed_ticket_frees_its_carriers_slot() {
        let mut l = Model::default();
        let carrier = any_pipe();
        let by = any_proc();
        l.place(carrier, End::Receiver, by);
        let a = any_pipe();
        kani::assume(a != carrier);
        l.relay(
            1,
            T {
                carrier,
                from: by,
                pipe: a,
                end: End::Sender,
            },
        );
        assert!(l.by_carrier(carrier) == 1);
        assert!(l.redeem(by, 1, carrier).is_ok());
        assert!(l.by_carrier(carrier) == 0);
        assert!(l.outstanding(by) == 0);
        kani::cover!(true, "the ticket is redeemed");
    }

    /// A process ending releases every ticket only it could redeem and,
    /// after a sweep, forgets a channel whose ends are both gone.
    #[kani::proof]
    #[kani::unwind(8)]
    fn gone_then_sweep_reclaims() {
        let mut l = Model::default();
        let p = any_proc();
        let chan = any_pipe();
        l.place(chan, End::Sender, p);
        l.place(chan, End::Receiver, p);
        assert!(l.has_creator(creator(chan)));
        let _ = l.gone(p);
        let emptied = l.sweep();
        assert!(l.holder(chan, End::Sender).is_none());
        let c = creator(chan);
        assert!(emptied.get(c).copied().unwrap_or(false) || l.has_creator(c));
        kani::cover!(
            emptied.get(c).copied().unwrap_or(false),
            "the creator is left with no channels"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BROKER: Proc = 0;
    const A: Proc = 1;
    const B: Proc = 2;

    fn pipe(creator: Proc, n: u8) -> PipeKey {
        PipeKey {
            creator,
            id: [n; 16],
        }
    }

    fn ledger() -> Ledger {
        let mut l = Ledger::default();
        l.add_proc(BROKER, 100);
        l.add_proc(A, 101);
        l.add_proc(B, 102);
        l
    }

    #[test]
    fn both_ends_of_one_channel_in_one_relay_are_both_recorded() {
        let mut l = ledger();
        l.relay(
            [7; 16],
            Held {
                carrier: pipe(BROKER, 9),
                from: A,
                ends: vec![
                    (pipe(A, 1), End::Sender, WireKind::Sender),
                    (pipe(A, 1), End::Receiver, WireKind::Receiver),
                ],
            },
        )
        .unwrap();
        // Both ends are in the ticket: neither is still A's.
        assert_eq!(l.view(pipe(A, 1), End::Sender).unwrap().holder, None);
        assert_eq!(l.view(pipe(A, 1), End::Receiver).unwrap().holder, None);
    }

    #[test]
    fn a_receiver_binds_while_the_sender_is_still_with_its_creator() {
        let mut l = ledger();
        // Channel 1 made by A; its receiver given to B; the sender still A's.
        l.place(pipe(A, 1), End::Receiver, B).unwrap();
        l.place(pipe(A, 1), End::Sender, A).unwrap();
        // B binds to some other peer (pid 555) before A grants the sender.
        assert_eq!(l.bind(B, pipe(A, 1), 0x40, 555), Ok(None));
        assert_eq!(l.bindings_by(B), 1);
        l.unbind(B, 0x40);
        assert_eq!(l.bindings_by(B), 0);
    }

    #[test]
    fn an_unseen_channel_is_its_creators() {
        let l = ledger();
        assert_eq!(l.view(pipe(A, 1), End::Sender), None);
    }

    #[test]
    fn relay_then_redeem_moves_the_ends() {
        let mut l = ledger();
        // A sends its channel 1's receiver to B over broker channel 9.
        let carrier = pipe(BROKER, 9);
        l.relay(
            [5; 16],
            Held {
                carrier,
                from: A,
                ends: vec![(pipe(A, 1), End::Receiver, WireKind::Receiver)],
            },
        )
        .unwrap();
        assert_eq!(l.view(pipe(A, 1), End::Receiver).unwrap().holder, None);
        // Only the holder of channel 9's receiver may redeem: the broker
        // made it, so the broker holds it.
        assert_eq!(
            l.redeem(B, &[5; 16], carrier).err(),
            Some(LedgerError::NotHolder)
        );
        assert_eq!(
            l.redeem(BROKER, &[5; 16], pipe(BROKER, 8)).err(),
            Some(LedgerError::WrongChannel)
        );
        let (held, events) = l.redeem(BROKER, &[5; 16], carrier).unwrap();
        assert_eq!(held.ends.len(), 1);
        assert!(events.is_empty());
        assert_eq!(
            l.view(pipe(A, 1), End::Receiver).unwrap().holder,
            Some(BROKER)
        );
        assert_eq!(
            l.redeem(BROKER, &[5; 16], carrier).err(),
            Some(LedgerError::NoTicket)
        );
    }

    #[test]
    fn a_binding_breaks_when_the_sender_leaves_its_peer() {
        let mut l = ledger();
        // The broker binds its channel 1's receiver to A (pid 101) and
        // grants A the bound sender over broker channel 9, which A holds.
        let ch = pipe(BROKER, 1);
        assert_eq!(l.bind(BROKER, ch, 0x40, 101), Ok(None));
        let to_a = pipe(BROKER, 9);
        l.relay(
            [1; 16],
            Held {
                carrier: to_a,
                from: BROKER,
                ends: vec![(ch, End::Sender, WireKind::BoundSender)],
            },
        )
        .unwrap();
        // Hand channel 9's receiver to A first (as a launch would).
        l.place(to_a, End::Receiver, A).unwrap();
        let (_, events) = l.redeem(A, &[1; 16], to_a).unwrap();
        assert!(events.is_empty(), "the peer itself received it");
        assert!(l.view(ch, End::Sender).unwrap().bound);
        // A relays it on (as free, which accept refuses; here, the
        // ledger alone): when B redeems it the binding breaks, naming A.
        let a_to_b = pipe(A, 3);
        l.place(a_to_b, End::Receiver, B).unwrap();
        l.relay(
            [2; 16],
            Held {
                carrier: a_to_b,
                from: A,
                ends: vec![(ch, End::Sender, WireKind::Sender)],
            },
        )
        .unwrap();
        let (_, events) = l.redeem(B, &[2; 16], a_to_b).unwrap();
        assert_eq!(
            events,
            vec![BindEvent {
                binder: BROKER,
                rx_id: 0x40,
                peer_pid: 101
            }]
        );
    }

    #[test]
    fn a_closed_receivers_binding_is_cleared() {
        let mut l = ledger();
        let ch = pipe(BROKER, 1);
        l.place(ch, End::Receiver, B).unwrap();
        l.place(ch, End::Sender, B).unwrap();
        assert_eq!(l.bind(B, ch, 0x40, 101), Ok(None));
        // B closes its listener 0x40: the binding is gone, so a later
        // move of the sender raises no event.
        l.unbind(B, 0x40);
        let carrier = pipe(BROKER, 9);
        l.place(carrier, End::Receiver, A).unwrap();
        l.relay(
            [3; 16],
            Held {
                carrier,
                from: B,
                ends: vec![(ch, End::Sender, WireKind::Sender)],
            },
        )
        .unwrap();
        let (_, events) = l.redeem(A, &[3; 16], carrier).unwrap();
        assert!(events.is_empty(), "a cleared binding still fired");
    }

    #[test]
    fn binding_a_receiver_whose_sender_is_elsewhere_breaks_at_once() {
        let mut l = ledger();
        let ch = pipe(BROKER, 1);
        l.place(ch, End::Sender, B).unwrap();
        assert_eq!(
            l.bind(BROKER, ch, 0x40, 101),
            Ok(Some(BindEvent {
                binder: BROKER,
                rx_id: 0x40,
                peer_pid: 101
            }))
        );
        assert_eq!(l.bind(A, ch, 0x40, 101), Err(LedgerError::NotHolder));
    }

    #[test]
    fn a_process_ending_releases_what_only_it_could_redeem() {
        let mut l = ledger();
        let carrier = pipe(A, 9);
        l.relay(
            [3; 16],
            Held {
                carrier,
                from: B,
                ends: vec![(pipe(B, 1), End::Sender, WireKind::Sender)],
            },
        )
        .unwrap();
        assert_eq!(l.tickets_outstanding(), 1);
        assert!(!l.ticket_free(&[3; 16]));
        assert_eq!(l.gone(A), vec![[3; 16]]);
        assert_eq!(l.tickets_outstanding(), 0);
        assert!(l.ticket_free(&[3; 16]));
        // Channels with both ends gone are forgotten, and the creator
        // then has none (item 5's SID map relies on this).
        l.place(pipe(A, 5), End::Sender, A).unwrap();
        l.place(pipe(A, 5), End::Receiver, A).unwrap();
        assert!(l.has_creator(A));
        let _ = l.gone(A);
        assert_eq!(l.sweep(), vec![A]);
        assert_eq!(l.view(pipe(A, 5), End::Sender), None);
        assert!(!l.has_creator(A));
    }

    #[test]
    fn an_end_in_a_dropped_ticket_is_gone_and_its_channel_swept() {
        let mut l = ledger();
        let carried = pipe(B, 1);
        l.place(carried, End::Receiver, B).unwrap();
        l.place(pipe(A, 9), End::Receiver, A).unwrap();
        l.relay(
            [4; 16],
            Held {
                carrier: pipe(A, 9),
                from: B,
                ends: vec![(carried, End::Sender, WireKind::Sender)],
            },
        )
        .unwrap();
        // A held the carrier's receiver: the ticket dies with A, and
        // the sender it held with it.
        assert_eq!(l.gone(A), vec![[4; 16]]);
        let _ = l.gone(B);
        l.sweep();
        assert_eq!(l.view(carried, End::Sender), None);
    }

    #[test]
    fn ends_lost_after_a_redemption_are_gone_and_swept() {
        let mut l = ledger();
        let carried = pipe(B, 1);
        l.place(carried, End::Receiver, B).unwrap();
        l.place(pipe(A, 9), End::Receiver, A).unwrap();
        let held = Held {
            carrier: pipe(A, 9),
            from: B,
            ends: vec![(carried, End::Sender, WireKind::Sender)],
        };
        l.relay([5; 16], held).unwrap();
        let (held, _) = l.redeem(A, &[5; 16], pipe(A, 9)).unwrap();
        l.lost(&held.ends);
        let _ = l.gone(B);
        l.sweep();
        assert_eq!(l.view(carried, End::Sender), None);
    }

    #[test]
    fn a_ticket_redeemed_by_a_non_holder_is_refused() {
        let mut l = ledger();
        let carrier = pipe(BROKER, 9);
        l.place(carrier, End::Receiver, A).unwrap();
        l.relay(
            [6; 16],
            Held {
                carrier,
                from: BROKER,
                ends: Vec::new(),
            },
        )
        .unwrap();
        assert_eq!(
            l.redeem(B, &[6; 16], carrier).err(),
            Some(LedgerError::NotHolder)
        );
        assert!(l.redeem(A, &[6; 16], carrier).is_ok());
    }

    #[test]
    fn a_stale_ticket_is_refused() {
        let mut l = ledger();
        assert_eq!(
            l.redeem(BROKER, &[8; 16], pipe(BROKER, 9)).err(),
            Some(LedgerError::NoTicket)
        );
    }

    #[test]
    fn errors_have_distinct_codes() {
        let codes = [
            LedgerError::NoTicket.code(),
            LedgerError::WrongChannel.code(),
            LedgerError::NotHolder.code(),
            LedgerError::Quota.code(),
        ];
        for (i, a) in codes.iter().enumerate() {
            for b in codes.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }

    fn ticket_bytes(i: usize) -> [u8; 16] {
        let mut t = [0u8; 16];
        let n = u64::try_from(i).unwrap_or(0).wrapping_add(1);
        t[..8].copy_from_slice(&n.to_le_bytes());
        t
    }

    /// The Kani proofs run on `model`; this holds the model to the
    /// ledger. Random sequences of every operation the model has, over its
    /// whole domain, give the same results on both, and after each step
    /// the same holder for every end, the same tickets, creators and
    /// per-carrier and per-process counts.
    #[test]
    fn the_model_agrees_with_the_ledger() {
        use super::model::{E, Model, PIPES, PROCS, T, TICKETS, creator};
        let key = |i: usize| PipeKey {
            creator: u32::try_from(creator(i)).unwrap(),
            id: [u8::try_from(i % 2).unwrap(); 16],
        };
        let ticket = |t: usize| [u8::try_from(t).unwrap().wrapping_add(1); 16];
        let proc = |p: usize| u32::try_from(p).unwrap();
        let mut x: u64 = 0x2545_f491_4f6c_dd1d;
        let mut next = |n: usize| {
            x ^= x.wrapping_shl(13);
            x ^= x.wrapping_shr(7);
            x ^= x.wrapping_shl(17);
            usize::try_from(x % u64::try_from(n).unwrap()).unwrap()
        };
        let mut ops = [0usize; 6];
        for _ in 0..3_000 {
            let mut l = Ledger::default();
            let mut m = Model::default();
            for _ in 0..40 {
                let end = if next(2) == 0 {
                    End::Sender
                } else {
                    End::Receiver
                };
                let op = next(6);
                ops[op] += 1;
                match op {
                    0 => {
                        let (i, to) = (next(PIPES), next(PROCS));
                        l.place(key(i), end, proc(to)).unwrap();
                        m.place(i, end, to);
                    }
                    1 => {
                        let (t, c, from, i) =
                            (next(TICKETS), next(PIPES), next(PROCS), next(PIPES));
                        let held = Held {
                            carrier: key(c),
                            from: proc(from),
                            ends: vec![(key(i), end, WireKind::Sender)],
                        };
                        l.relay(ticket(t), held).unwrap();
                        m.relay(
                            t,
                            T {
                                carrier: c,
                                from,
                                pipe: i,
                                end,
                            },
                        );
                    }
                    2 => {
                        let (by, t, rx) = (next(PROCS), next(TICKETS), next(PIPES));
                        let real = l.redeem(proc(by), &ticket(t), key(rx));
                        let got = m.redeem(by, t, rx);
                        match (real, got) {
                            (Ok((held, _)), Ok(h)) => {
                                assert_eq!(held.carrier, key(h.carrier));
                                assert_eq!(held.from, proc(h.from));
                                assert_eq!(held.ends, vec![(key(h.pipe), h.end, WireKind::Sender)]);
                            }
                            (Err(LedgerError::NoTicket), Err(E::NoTicket))
                            | (Err(LedgerError::WrongChannel), Err(E::WrongChannel))
                            | (Err(LedgerError::NotHolder), Err(E::NotHolder)) => {}
                            (a, b) => panic!("redeem: ledger {a:?}, model {b:?}"),
                        }
                    }
                    3 => {
                        let p = next(PROCS);
                        let mut real = l.gone(proc(p));
                        real.sort_unstable();
                        let dead = m.gone(p);
                        let want: Vec<_> = (0..TICKETS).filter(|t| dead[*t]).map(ticket).collect();
                        assert_eq!(real, want);
                    }
                    4 => {
                        let mut real = l.sweep();
                        real.sort_unstable();
                        let emptied = m.sweep();
                        let want: Vec<_> = (0..PROCS).filter(|c| emptied[*c]).map(proc).collect();
                        assert_eq!(real, want);
                    }
                    _ => {
                        let (i, by) = (next(PIPES), next(PROCS));
                        l.close_end(key(i), end, proc(by));
                        m.close_end(i, end, by);
                    }
                }
                for i in 0..PIPES {
                    for e in [End::Sender, End::Receiver] {
                        let real = l
                            .view(key(i), e)
                            .map(|v| v.holder.map(|p| usize::try_from(p).unwrap()));
                        assert_eq!(real, m.holder(i, e), "channel {i} {e:?}");
                    }
                    let real = l.by_carrier.get(&key(i)).copied().unwrap_or(0);
                    assert_eq!(real, m.by_carrier(i));
                }
                for t in 0..TICKETS {
                    assert_eq!(l.ticket_free(&ticket(t)), m.ticket_free(t));
                }
                assert_eq!(l.tickets_outstanding(), m.tickets_outstanding());
                for p in 0..PROCS {
                    assert_eq!(l.has_creator(proc(p)), m.has_creator(p));
                    let real = l.outstanding.get(&proc(p)).copied().unwrap_or(0);
                    assert_eq!(real, m.outstanding(p));
                }
            }
        }
        // Every operation ran many times.
        assert!(ops.iter().all(|n| *n > 10_000), "{ops:?}");
    }

    #[test]
    fn closing_channels_frees_their_creators_slots() {
        // A long-lived process creates far more than its pipe quota of
        // channels, dropping each before the next, and keeps going:
        // closing both ends and sweeping gives the slot back (item 3).
        let mut l = ledger();
        for i in 0..(MAX_PIPES_PER_PROC * 2) {
            let n = u8::try_from(i % 2).unwrap();
            let ch = pipe(A, n);
            l.place(ch, End::Sender, A).unwrap();
            l.place(ch, End::Receiver, A).unwrap();
            l.close_end(ch, End::Sender, A);
            l.close_end(ch, End::Receiver, A);
            l.sweep();
            assert!(!l.has_creator(A));
        }
        // After all that churn A still has room, and the ledger is not
        // holding the churned channels.
        assert!(l.entry_count() <= 1);
        assert!(l.place(pipe(A, 0), End::Sender, A).is_ok());
    }

    #[test]
    fn a_close_by_a_non_holder_is_ignored() {
        let mut l = ledger();
        let ch = pipe(A, 1);
        l.place(ch, End::Sender, A).unwrap();
        // B does not hold the sender: its close changes nothing.
        l.close_end(ch, End::Sender, B);
        assert_eq!(l.view(ch, End::Sender).unwrap().holder, Some(A));
        l.close_end(ch, End::Sender, A);
        assert_eq!(l.view(ch, End::Sender).unwrap().holder, None);
    }

    #[test]
    fn tickets_are_limited_per_channel() {
        let mut l = ledger();
        let carrier = pipe(BROKER, 9);
        for i in 0..MAX_TICKETS_PER_CHANNEL {
            l.relay(
                ticket_bytes(i),
                Held {
                    carrier,
                    from: A,
                    ends: Vec::new(),
                },
            )
            .unwrap();
        }
        // One more on the same carrier is refused.
        assert_eq!(
            l.relay(
                [0xff; 16],
                Held {
                    carrier,
                    from: A,
                    ends: Vec::new(),
                },
            ),
            Err(LedgerError::Quota)
        );
        // But a different carrier (same process) still has room: a
        // channel a peer never reads does not pin another's relays.
        assert!(
            l.relay(
                [0xfe; 16],
                Held {
                    carrier: pipe(BROKER, 10),
                    from: A,
                    ends: Vec::new(),
                },
            )
            .is_ok()
        );
    }

    #[test]
    fn tickets_are_also_limited_per_process_across_channels() {
        let mut l = ledger();
        // Spread tickets over many carriers so the per-channel cap never
        // bites; the per-process total should.
        let mut i = 0usize;
        'outer: for carrier_n in 0..u8::MAX {
            for _ in 0..MAX_TICKETS_PER_CHANNEL {
                if i >= MAX_TICKETS_PER_PROC {
                    break 'outer;
                }
                l.relay(
                    ticket_bytes(i),
                    Held {
                        carrier: pipe(BROKER, carrier_n),
                        from: A,
                        ends: Vec::new(),
                    },
                )
                .unwrap();
                i += 1;
            }
        }
        assert_eq!(i, MAX_TICKETS_PER_PROC);
        assert_eq!(
            l.relay(
                [0xff; 16],
                Held {
                    carrier: pipe(BROKER, u8::MAX),
                    from: A,
                    ends: Vec::new(),
                },
            ),
            Err(LedgerError::Quota)
        );
    }
}
