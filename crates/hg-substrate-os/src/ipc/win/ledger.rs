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
//!
//! **A fresh pipe per move (W3).** After every move a channel's ends are
//! on a new pipe the broker made, so the channel outlives its first pipe.
//! The ledger keeps one key per channel, its first pipe's, and every
//! later pipe's key is an *alias* of it ([`Ledger::canonical`]). An alias
//! is counted: it lives while the pipe is the channel's current one, an
//! old pipe queued for the receiver, an old pipe a holder still owes the
//! broker (a *park*), the pipe of an end whose replacement waits in
//! the broker, or the pipe of an end in a relay not yet decided. A receiver's old pipes wait in a per-channel FIFO until
//! its holder listens; at most [`MAX_OLD_PIPES`] are queued or owed at
//! once. Each is charged to the channel's creator, like a channel, and
//! all are released when the receiver ends, when the creator or the
//! holder dies, or when the channel is swept.

use std::collections::{BTreeMap, VecDeque};

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

/// The most channels the ledger tracks for one creator. A channel's old
/// pipes held for its receiver (W3) count against it too.
pub(crate) const MAX_PIPES_PER_PROC: usize = 16_384;

/// The most old pipes one channel may have queued at the broker or owed
/// to it (W3, option 2). Set from first principles, not measured (†):
/// each costs the broker one handle and the receiver one extra read
/// pass, and only a receiver moved again and again without anyone
/// listening, or an unlistened receiver whose sender keeps moving, owes
/// more than one or two. A move past the limit is refused, and the
/// channel ends in `Closed`.
pub(crate) const MAX_OLD_PIPES: usize = 16;

/// A channel's pipes after its first move (W3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Pipes {
    /// The pipe the channel's ends are on now; `None` for its first.
    current: Option<PipeKey>,
    /// Old pipes the broker holds for the receiver, oldest first.
    queued: VecDeque<PipeKey>,
    /// Old pipes a holder of the unlistened receiver said it will hand
    /// back (`Park`), by holder.
    parks: Vec<(Proc, PipeKey)>,
    /// Old pipes were released before the receiver took them (its
    /// creator died): the receiver must end in `Closed`, not read on as
    /// if nothing were missing.
    lost: bool,
}

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

/// Where a channel end is, for a move of its peer (W3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Where {
    /// With a process (the broker is process 0).
    Proc(Proc),
    /// In the broker, under a ticket.
    Ticket(Ticket),
    /// Its holder closed it or ended.
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
    /// The regions (D6), each with its index in the record's descriptor
    /// order; the ends fill the other indices, in order.
    pub(crate) regions: Vec<(usize, super::regions::Grant)>,
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
    /// The channel already has [`MAX_OLD_PIPES`] old pipes queued or
    /// owed (W3).
    OldPipes,
    /// No such park was expected (W3).
    NoPark,
}

impl LedgerError {
    /// The code a refusal record carries.
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::NoTicket => 1,
            Self::WrongChannel => 2,
            Self::NotHolder => 3,
            Self::Quota => 4,
            Self::OldPipes => 5,
            Self::NoPark => 6,
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
    /// W3: a later pipe's key to its channel's key, and how many uses
    /// keep the alias (see the module docs).
    aliases: BTreeMap<PipeKey, (PipeKey, u32)>,
    /// W3: each moved channel's pipes, by channel key.
    pipes: BTreeMap<PipeKey, Pipes>,
    /// W3: old pipes released since the broker last asked, whose handles
    /// it must close ([`Ledger::take_released`]).
    released: Vec<PipeKey>,
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
        // W3: old pipes go with the receiver's holder, and with the
        // creator they are charged to (then the receiver must close).
        let mut release: Vec<(PipeKey, bool)> = Vec::new();
        for (k, e) in &mut self.entries {
            if lost(e.sender) {
                e.sender = Holder::Gone;
            }
            if lost(e.receiver) {
                e.receiver = Holder::Gone;
                release.push((*k, false));
            } else if k.creator == p {
                release.push((*k, true));
            }
            if e.binding.is_some_and(|b| b.binder == p) {
                e.binding = None;
            }
        }
        self.drop_parks_of(p);
        for (k, lost_pipes) in release {
            self.release_old(k, lost_pipes);
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
            if end == End::Receiver {
                self.release_old(pipe, false);
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
                if end == End::Receiver {
                    self.release_old(pipe, false);
                }
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
            self.forget_pipes(k);
            // Reported once, when this channel's slot was its last.
            let had = self.per_creator.contains_key(&k.creator);
            self.uncharge(k.creator);
            if had && !self.per_creator.contains_key(&k.creator) {
                emptied.push(k.creator);
            }
        }
        emptied
    }

    // ---- a fresh pipe per move (W3) ------------------------------------

    /// The channel a pipe's key belongs to: the key itself, or the
    /// channel's key if it is a live alias.
    pub(crate) fn canonical(&self, key: PipeKey) -> PipeKey {
        self.aliases.get(&key).map_or(key, |&(chan, _)| chan)
    }

    /// One more use of `key` as a pipe of `chan`. A no-op for the
    /// channel's own key, which always names it.
    fn hold(&mut self, chan: PipeKey, key: PipeKey) {
        if key == chan {
            return;
        }
        let e = self.aliases.entry(key).or_insert((chan, 0));
        e.1 = e.1.saturating_add(1);
    }

    /// One use of `key` fewer; the alias goes with its last use.
    fn unhold(&mut self, key: PipeKey) {
        if let Some(e) = self.aliases.get_mut(&key) {
            e.1 = e.1.saturating_sub(1);
            if e.1 == 0 {
                self.aliases.remove(&key);
            }
        }
    }

    /// Keeps `key` resolvable to `chan` while an end on that pipe waits
    /// for its replacement in the broker; undone by [`Ledger::let_go`].
    pub(crate) fn keep(&mut self, chan: PipeKey, key: PipeKey) {
        self.hold(chan, key);
    }

    /// Ends a use taken with [`Ledger::keep`].
    pub(crate) fn let_go(&mut self, key: PipeKey) {
        self.unhold(key);
    }

    /// Channel `chan` moved to the broker-made pipe `new`: `new` becomes
    /// its current pipe. Returns the pipe it was on.
    pub(crate) fn switch_pipe(&mut self, chan: PipeKey, new: PipeKey) -> PipeKey {
        self.hold(chan, new);
        let p = self.pipes.entry(chan).or_default();
        let prev = p.current.replace(new).unwrap_or(chan);
        self.unhold(prev);
        prev
    }

    /// Makes sure `chan` has a ledger entry (charged to its creator), so a
    /// channel that changes pipes is swept like any other.
    ///
    /// # Errors
    /// [`LedgerError::Quota`].
    pub(crate) fn touch(&mut self, chan: PipeKey) -> Result<(), LedgerError> {
        let e = self.entry(chan);
        self.put(chan, e)
    }

    /// Where `end` of `chan` is: with the channel's creator if the ledger
    /// has never seen it move.
    pub(crate) fn where_is(&self, chan: PipeKey, end: End) -> Where {
        let e = self.entry(chan);
        match if end == End::Sender {
            e.sender
        } else {
            e.receiver
        } {
            Holder::Proc(p) => Where::Proc(p),
            Holder::Ticket(t) => Where::Ticket(t),
            Holder::Gone => Where::Gone,
        }
    }

    /// The position of `end` of `chan`'s handle in the broker's store for
    /// `ticket`: the store is in the record's descriptor order, regions
    /// (D6) at their own indices and the ends filling the others in
    /// order.
    pub(crate) fn ticket_index(&self, ticket: &Ticket, chan: PipeKey, end: End) -> Option<usize> {
        let held = self.tickets.get(ticket)?;
        let n = held
            .ends
            .iter()
            .position(|&(k, e, _)| k == chan && e == end)?;
        (0..held.ends.len().saturating_add(held.regions.len()))
            .filter(|i| !held.regions.iter().any(|(r, _)| r == i))
            .nth(n)
    }

    /// Withdraws a park [`Ledger::expect_park`] reserved, when the holder
    /// answered that it will not park (it was listening, or did not
    /// switch).
    pub(crate) fn cancel_park(&mut self, chan: PipeKey, holder: Proc, key: PipeKey) {
        let Some(p) = self.pipes.get_mut(&chan) else {
            return;
        };
        let Some(i) = p.parks.iter().position(|&(h, k)| h == holder && k == key) else {
            return;
        };
        p.parks.remove(i);
        self.unhold(key);
        self.uncharge(chan.creator);
    }

    /// Whether the ledger tracks `chan`.
    pub(crate) fn knows(&self, chan: PipeKey) -> bool {
        self.entries.contains_key(&chan)
    }

    /// The pipe channel `chan` is on now.
    #[cfg(test)]
    pub(crate) fn current_pipe(&self, chan: PipeKey) -> PipeKey {
        self.pipes
            .get(&chan)
            .and_then(|p| p.current)
            .unwrap_or(chan)
    }

    /// Whether `chan` may queue or owe one more old pipe: under
    /// [`MAX_OLD_PIPES`], and its creator under its pipe quota.
    pub(crate) fn old_pipe_room(&self, chan: PipeKey) -> Result<(), LedgerError> {
        let owed = self
            .pipes
            .get(&chan)
            .map_or(0, |p| p.queued.len().saturating_add(p.parks.len()));
        if owed >= MAX_OLD_PIPES {
            return Err(LedgerError::OldPipes);
        }
        if self.per_creator.get(&chan.creator).copied().unwrap_or(0) >= MAX_PIPES_PER_PROC {
            return Err(LedgerError::Quota);
        }
        Ok(())
    }

    fn charge(&mut self, creator: Proc) {
        let n = self.per_creator.entry(creator).or_insert(0);
        *n = n.saturating_add(1);
    }

    fn uncharge(&mut self, creator: Proc) {
        if let Some(n) = self.per_creator.get_mut(&creator) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                self.per_creator.remove(&creator);
            }
        }
    }

    /// Queues old pipe `key` for `chan`'s receiver, behind any already
    /// queued (the broker keeps its server end).
    ///
    /// # Errors
    /// [`LedgerError::OldPipes`] or [`LedgerError::Quota`]; nothing
    /// changes.
    pub(crate) fn queue_old(&mut self, chan: PipeKey, key: PipeKey) -> Result<(), LedgerError> {
        self.old_pipe_room(chan)?;
        self.hold(chan, key);
        self.charge(chan.creator);
        self.pipes.entry(chan).or_default().queued.push_back(key);
        Ok(())
    }

    /// Process `holder`, whose unlistened receiver of `chan` was just
    /// moved off pipe `key`, will hand that pipe's server end back
    /// (`Park`). Counted against the limit until it does.
    ///
    /// # Errors
    /// As [`Ledger::queue_old`].
    pub(crate) fn expect_park(
        &mut self,
        chan: PipeKey,
        holder: Proc,
        key: PipeKey,
    ) -> Result<(), LedgerError> {
        self.old_pipe_room(chan)?;
        self.hold(chan, key);
        self.charge(chan.creator);
        self.pipes
            .entry(chan)
            .or_default()
            .parks
            .push((holder, key));
        Ok(())
    }

    /// `holder` handed back old pipe `key`: it joins its channel's queue.
    /// Returns the channel.
    ///
    /// # Errors
    /// [`LedgerError::NoPark`] when no such park was expected.
    pub(crate) fn park(&mut self, holder: Proc, key: PipeKey) -> Result<PipeKey, LedgerError> {
        let chan = self.canonical(key);
        let p = self.pipes.get_mut(&chan).ok_or(LedgerError::NoPark)?;
        let i = p
            .parks
            .iter()
            .position(|&(h, k)| h == holder && k == key)
            .ok_or(LedgerError::NoPark)?;
        p.parks.remove(i);
        // The use and the charge carry over from the park.
        p.queued.push_back(key);
        Ok(chan)
    }

    /// Whether `chan`'s receiver has old pipes to take, or lost some
    /// (then its holder must end it in `Closed`).
    pub(crate) fn has_old(&self, chan: PipeKey) -> bool {
        self.pipes
            .get(&chan)
            .is_some_and(|p| !p.queued.is_empty() || p.lost)
    }

    /// Hands `chan`'s queued old pipes to its receiver's holder, oldest
    /// first, releasing them here (none for a channel that never moved).
    /// `None` only if some were lost.
    pub(crate) fn take_old(&mut self, chan: PipeKey) -> Option<Vec<PipeKey>> {
        let Some(p) = self.pipes.get_mut(&chan) else {
            return Some(Vec::new());
        };
        if p.lost {
            return None;
        }
        let keys: Vec<PipeKey> = p.queued.drain(..).collect();
        for &k in &keys {
            self.unhold(k);
            self.uncharge(chan.creator);
        }
        Some(keys)
    }

    /// Releases every old pipe `chan` has queued or owed; the queued ones'
    /// handles are listed for the broker to close. With `lost`, a later
    /// `take_old` answers `None`.
    fn release_old(&mut self, chan: PipeKey, lost: bool) {
        let Some(p) = self.pipes.get_mut(&chan) else {
            return;
        };
        let queued: Vec<PipeKey> = p.queued.drain(..).collect();
        let parks: Vec<PipeKey> = p.parks.drain(..).map(|(_, k)| k).collect();
        if lost && !(queued.is_empty() && parks.is_empty()) {
            p.lost = true;
        }
        for &k in queued.iter().chain(&parks) {
            self.unhold(k);
            self.uncharge(chan.creator);
        }
        self.released.extend(queued);
    }

    /// Process `p` will hand back no more pipes: its parks are dropped.
    fn drop_parks_of(&mut self, p: Proc) {
        let mut dropped = Vec::new();
        for (chan, pipes) in &mut self.pipes {
            pipes.parks.retain(|&(h, k)| {
                let keep = h != p;
                if !keep {
                    dropped.push((*chan, k));
                }
                keep
            });
        }
        for (chan, k) in dropped {
            self.unhold(k);
            self.uncharge(chan.creator);
        }
    }

    /// Old pipes released since the last call, whose server ends the
    /// broker holds and must now close.
    pub(crate) fn take_released(&mut self) -> Vec<PipeKey> {
        core::mem::take(&mut self.released)
    }

    /// Forgets a swept channel's pipes and every alias of it.
    fn forget_pipes(&mut self, chan: PipeKey) {
        self.release_old(chan, false);
        self.pipes.remove(&chan);
        self.aliases.retain(|_, &mut (c, _)| c != chan);
    }

    /// Aliases the ledger keeps (for the tests: none may leak).
    #[cfg(test)]
    pub(crate) fn alias_count(&self) -> usize {
        self.aliases.len()
    }

    /// Old pipes queued or owed for `chan` (for the tests).
    #[cfg(test)]
    pub(crate) fn old_pipes(&self, chan: PipeKey) -> usize {
        self.pipes
            .get(&chan)
            .map_or(0, |p| p.queued.len().saturating_add(p.parks.len()))
    }

    /// Slots charged to `creator` (for the tests).
    #[cfg(test)]
    pub(crate) fn charged(&self, creator: Proc) -> usize {
        self.per_creator.get(&creator).copied().unwrap_or(0)
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
                regions: Vec::new(),
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
                regions: Vec::new(),
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
                regions: Vec::new(),
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
                regions: Vec::new(),
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
                regions: Vec::new(),
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
                regions: Vec::new(),
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
                regions: Vec::new(),
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
            regions: Vec::new(),
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
                regions: Vec::new(),
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
            LedgerError::OldPipes.code(),
            LedgerError::NoPark.code(),
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
                    regions: Vec::new(),
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
                    regions: Vec::new(),
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
                    regions: Vec::new(),
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
                        regions: Vec::new(),
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
                    regions: Vec::new(),
                },
            ),
            Err(LedgerError::Quota)
        );
    }

    // ---- W3: a fresh pipe per move ----------------------------------------

    #[test]
    fn a_later_pipe_resolves_to_its_channel_while_it_is_used() {
        let mut l = ledger();
        let ch = pipe(A, 1);
        let (p1, p2) = (pipe(BROKER, 21), pipe(BROKER, 22));
        assert_eq!(l.canonical(p1), p1, "an unknown key names itself");
        assert_eq!(l.switch_pipe(ch, p1), ch);
        assert_eq!(l.canonical(p1), ch);
        assert_eq!(l.current_pipe(ch), p1);
        // The receiver's old pipe p1 is queued as the channel moves on.
        l.queue_old(ch, p1).unwrap();
        assert_eq!(l.switch_pipe(ch, p2), p1);
        assert_eq!(l.canonical(p1), ch, "still queued, still resolvable");
        assert_eq!(l.canonical(p2), ch);
        // Taken: p1 is no longer the channel's, p2 still is.
        assert_eq!(l.take_old(ch), Some(vec![p1]));
        assert_eq!(l.canonical(p1), p1);
        assert_eq!(l.canonical(p2), ch);
        // A kept end keeps its pipe resolvable until it is let go.
        l.keep(ch, p2);
        let p3 = pipe(BROKER, 23);
        l.switch_pipe(ch, p3);
        assert_eq!(l.canonical(p2), ch);
        l.let_go(p2);
        assert_eq!(l.canonical(p2), p2);
        assert_eq!(l.alias_count(), 1, "only the current pipe's alias");
    }

    #[test]
    fn old_pipes_are_taken_in_order_and_once() {
        let mut l = ledger();
        let ch = pipe(A, 1);
        l.touch(ch).unwrap();
        for n in 30..33 {
            l.queue_old(ch, pipe(BROKER, n)).unwrap();
        }
        assert!(l.has_old(ch));
        assert_eq!(
            l.take_old(ch),
            Some(vec![pipe(BROKER, 30), pipe(BROKER, 31), pipe(BROKER, 32)])
        );
        assert!(!l.has_old(ch));
        assert_eq!(l.take_old(ch), Some(vec![]));
        assert_eq!(l.charged(A), 1, "only the channel itself");
        assert_eq!(l.take_old(pipe(A, 7)), Some(vec![]), "never moved");
    }

    #[test]
    fn the_seventeenth_old_pipe_is_refused() {
        let mut l = ledger();
        let ch = pipe(A, 1);
        l.touch(ch).unwrap();
        for n in 0..MAX_OLD_PIPES {
            let n = u8::try_from(n).unwrap();
            if n % 2 == 0 {
                l.queue_old(ch, pipe(BROKER, n)).unwrap();
            } else {
                l.expect_park(ch, B, pipe(BROKER, n)).unwrap();
            }
        }
        assert_eq!(l.old_pipes(ch), MAX_OLD_PIPES);
        // Queued and owed count alike: one more of either is refused.
        assert_eq!(
            l.queue_old(ch, pipe(BROKER, 99)),
            Err(LedgerError::OldPipes)
        );
        assert_eq!(
            l.expect_park(ch, B, pipe(BROKER, 99)),
            Err(LedgerError::OldPipes)
        );
        assert_eq!(l.old_pipe_room(ch), Err(LedgerError::OldPipes));
        assert_eq!(l.old_pipes(ch), MAX_OLD_PIPES, "a refusal changes nothing");
        // Another channel is not affected.
        assert_eq!(l.old_pipe_room(pipe(A, 2)), Ok(()));
    }

    #[test]
    fn a_park_joins_the_queue_only_from_its_holder() {
        let mut l = ledger();
        let ch = pipe(A, 1);
        l.touch(ch).unwrap();
        let (old, owed) = (pipe(BROKER, 40), pipe(BROKER, 41));
        l.queue_old(ch, old).unwrap();
        l.expect_park(ch, B, owed).unwrap();
        assert_eq!(l.park(A, owed), Err(LedgerError::NoPark));
        assert_eq!(l.park(B, pipe(BROKER, 42)), Err(LedgerError::NoPark));
        assert_eq!(l.park(B, owed), Ok(ch));
        assert_eq!(l.park(B, owed), Err(LedgerError::NoPark), "only once");
        assert_eq!(l.take_old(ch), Some(vec![old, owed]));
        assert_eq!(l.charged(A), 1);
        assert_eq!(l.alias_count(), 0);
    }

    #[test]
    fn a_withdrawn_park_frees_its_slot() {
        let mut l = ledger();
        let ch = pipe(A, 1);
        l.touch(ch).unwrap();
        l.expect_park(ch, B, pipe(BROKER, 70)).unwrap();
        l.cancel_park(ch, B, pipe(BROKER, 70));
        assert_eq!(l.old_pipes(ch), 0);
        assert_eq!(l.charged(A), 1);
        assert_eq!(l.alias_count(), 0);
        assert_eq!(l.park(B, pipe(BROKER, 70)), Err(LedgerError::NoPark));
    }

    /// A ticket's handle store has its regions at their own indices (D6),
    /// so an end's slot skips them (W3's rebind swaps the slot in place).
    #[test]
    fn a_ticket_index_skips_the_regions() {
        let mut l = ledger();
        let (a, b) = (pipe(A, 1), pipe(A, 2));
        let grant = super::super::regions::Grant {
            id: 1,
            kind: WireKind::ReadOnlyRegion,
            len: 4096,
            filler_trusted: false,
        };
        l.relay(
            [9; 16],
            Held {
                carrier: pipe(BROKER, 9),
                from: A,
                ends: vec![
                    (a, End::Sender, WireKind::Sender),
                    (b, End::Receiver, WireKind::Receiver),
                ],
                regions: vec![(0, grant), (2, grant)],
            },
        )
        .unwrap();
        assert_eq!(l.ticket_index(&[9; 16], a, End::Sender), Some(1));
        assert_eq!(l.ticket_index(&[9; 16], b, End::Receiver), Some(3));
        assert_eq!(l.ticket_index(&[9; 16], a, End::Receiver), None);
    }

    #[test]
    fn where_an_end_is() {
        let mut l = ledger();
        let ch = pipe(A, 1);
        assert_eq!(l.where_is(ch, End::Sender), Where::Proc(A), "unseen");
        assert!(!l.knows(ch));
        l.relay(
            [9; 16],
            Held {
                carrier: pipe(BROKER, 9),
                from: A,
                ends: vec![(ch, End::Receiver, WireKind::Receiver)],
                regions: Vec::new(),
            },
        )
        .unwrap();
        assert_eq!(l.where_is(ch, End::Receiver), Where::Ticket([9; 16]));
        assert_eq!(l.ticket_index(&[9; 16], ch, End::Receiver), Some(0));
        assert_eq!(l.ticket_index(&[9; 16], ch, End::Sender), None);
        assert!(l.knows(ch));
        l.close_end(ch, End::Sender, A);
        assert_eq!(l.where_is(ch, End::Sender), Where::Gone);
    }

    #[test]
    fn old_pipes_are_charged_to_the_creator() {
        let mut l = ledger();
        let ch = pipe(A, 1);
        l.touch(ch).unwrap();
        l.queue_old(ch, pipe(BROKER, 50)).unwrap();
        l.expect_park(ch, B, pipe(BROKER, 51)).unwrap();
        assert_eq!(l.charged(A), 3, "the channel and two old pipes");
        assert_eq!(l.charged(BROKER), 0, "not the broker that made them");
    }

    /// Queued old pipes for a channel whose receiver B holds, with one
    /// more owed by B.
    fn with_old_pipes() -> (Ledger, PipeKey) {
        let mut l = ledger();
        let ch = pipe(A, 1);
        l.place(ch, End::Sender, A).unwrap();
        l.place(ch, End::Receiver, B).unwrap();
        l.switch_pipe(ch, pipe(BROKER, 60));
        l.queue_old(ch, pipe(A, 1)).unwrap();
        l.switch_pipe(ch, pipe(BROKER, 61));
        l.queue_old(ch, pipe(BROKER, 60)).unwrap();
        l.switch_pipe(ch, pipe(BROKER, 62));
        l.expect_park(ch, B, pipe(BROKER, 61)).unwrap();
        assert_eq!(l.charged(A), 4);
        (l, ch)
    }

    #[test]
    fn old_pipes_are_released_when_the_receiver_closes() {
        let (mut l, ch) = with_old_pipes();
        l.close_end(ch, End::Receiver, B);
        assert_eq!(l.old_pipes(ch), 0);
        assert_eq!(l.charged(A), 1);
        assert_eq!(l.take_released(), vec![pipe(A, 1), pipe(BROKER, 60)]);
        assert_eq!(l.take_released(), vec![]);
    }

    #[test]
    fn old_pipes_are_released_when_their_holder_dies() {
        let (mut l, ch) = with_old_pipes();
        let _ = l.gone(B);
        assert_eq!(l.old_pipes(ch), 0);
        assert_eq!(l.charged(A), 1);
        assert_eq!(l.take_released().len(), 2);
    }

    #[test]
    fn old_pipes_are_released_when_their_creator_dies_and_the_receiver_closes() {
        let (mut l, ch) = with_old_pipes();
        let _ = l.gone(A);
        assert_eq!(l.old_pipes(ch), 0);
        assert_eq!(l.take_released().len(), 2);
        // The receiver lives on but has lost records: it must close.
        assert!(l.has_old(ch));
        assert_eq!(l.take_old(ch), None);
        // B's late park is no longer expected.
        assert_eq!(l.park(B, pipe(BROKER, 61)), Err(LedgerError::NoPark));
    }

    #[test]
    fn a_swept_channel_leaves_no_alias_or_charge() {
        let (mut l, ch) = with_old_pipes();
        l.keep(ch, pipe(BROKER, 61));
        l.close_end(ch, End::Sender, A);
        l.close_end(ch, End::Receiver, B);
        assert_eq!(l.sweep(), vec![A]);
        assert_eq!(l.alias_count(), 0);
        assert_eq!(l.charged(A), 0);
        assert!(!l.has_creator(A));
        assert_eq!(l.canonical(pipe(BROKER, 62)), pipe(BROKER, 62));
    }
}
