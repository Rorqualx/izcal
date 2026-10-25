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

/// The most tickets one process may have outstanding.
pub(crate) const MAX_TICKETS_PER_PROC: usize = 1024;

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
        if n >= MAX_TICKETS_PER_PROC || !self.room_for(&held.ends) {
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
        self.release_count(held.from);
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

    fn release_count(&mut self, from: Proc) {
        if let Some(n) = self.outstanding.get_mut(&from) {
            *n = n.saturating_sub(1);
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
                self.release_count(h.from);
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

    #[test]
    fn tickets_are_limited_per_process() {
        let mut l = ledger();
        for i in 0..MAX_TICKETS_PER_PROC {
            let mut t = [0u8; 16];
            t[..8].copy_from_slice(&u64::try_from(i + 1).unwrap().to_le_bytes());
            l.relay(
                t,
                Held {
                    carrier: pipe(BROKER, 9),
                    from: A,
                    ends: Vec::new(),
                },
            )
            .unwrap();
        }
        assert_eq!(
            l.relay(
                [0xff; 16],
                Held {
                    carrier: pipe(BROKER, 9),
                    from: A,
                    ends: Vec::new(),
                },
            ),
            Err(LedgerError::Quota)
        );
    }
}
