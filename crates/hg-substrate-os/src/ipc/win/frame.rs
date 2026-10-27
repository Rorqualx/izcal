//! What crosses a Windows pipe besides the shared record formats
//! (IPC-R6: one fixed little-endian layout, decoded without allocating).
//!
//! **Data messages.** A channel is one duplex message-mode pipe: data
//! messages from the sender end, credit records
//! ([`crate::ipc::common::codec`]) back from the receiver end. Each data
//! message is a [`TICKET_LEN`]-byte ticket, a data record, and a
//! [`TRAILER_LEN`]-byte trailer holding the message's own total length
//! (u32). The ticket is all zeros when the record carries no handles;
//! otherwise it names the handles the broker holds for this record,
//! which the receiver redeems over its control link. Handle values never
//! travel in a channel: a value written by a peer would name whatever
//! this process happens to hold under that number (design review B1).
//!
//! **Why the trailer (probe W13c).** A read left pending by a former
//! holder of a receiver end takes the next message written to that pipe,
//! and a read with a buffer shorter than the message takes only its
//! head: the rest stays in the pipe and the next reader gets it as an
//! ordinary message, with nothing marking it as a remainder. If a
//! payload carried bytes shaped like a ticket and a record, the new
//! holder would accept that suffix as a whole message. A suffix keeps
//! the sender's trailer, which states the whole message's length, and
//! is always shorter than that, so [`split_data`] refuses every proper
//! suffix of a message ([`FrameError::Suffix`]).
//!
//! **Control records** cross the control link, a pipe between a
//! sandboxed process and its broker. An 8-byte header ([`CTL_MAGIC`],
//! [`CTL_VERSION`], the record kind, an item count and a reserved 0)
//! and a body fixed by the kind:
//!
//! | Kind | Direction | Body |
//! |---|---|---|
//! | 1 `Hello` | broker to child | to-parent value u64, from-parent value u64, parent pid u32, reserved u32 |
//! | 2 `Relay` | child to broker | request u64, carrier value u64, then `count` items |
//! | 3 `RelayOk` | broker to child | request u64, ticket (16 bytes) |
//! | 4 `RelayRefused` | broker to child | request u64, reason u8, 7 reserved |
//! | 5 `Redeem` | child to broker | request u64, ticket, receiver value u64 |
//! | 6 `Redeemed` | broker to child | request u64, then `count` items |
//! | 7 `RedeemRefused` | broker to child | request u64, reason u8, 7 reserved |
//! | 8 `Bind` | child to broker | receiver value u64, peer pid u32, reserved u32 |
//! | 9 `BindBroken` | broker to child | receiver value u64, peer pid u32, reserved u32 |
//! | 10 `Unbind` | child to broker | listener id u64, reserved u64 |
//! | 11 `Closed` | child to broker | end value u64, reserved u64 |
//! | 12 `RebindValue` | broker to child | request u64, end value u64, new value u64, reserved u64 |
//! | 13 `RebindId` | broker to child | request u64, pipe id (16 bytes), new value u64 |
//! | 14 `Rebound` | child to broker | request u64, state u8 (0 not switched, 1 switched, 2 switched and a `Park` follows), 7 reserved |
//! | 15 `Park` | child to broker | old pipe's server end value u64, reserved u64 |
//! | 16 `TakeDrains` | child to broker | request u64, receiver value u64 |
//! | 17 `Drains` | broker to child | request u64, then `count` items |
//! | 18 `DrainsRefused` | broker to child | request u64, reserved u64 |
//!
//! Kinds 12 to 18 are a fresh pipe per move (W3): the broker moves the
//! peer of a moving end to a new pipe (`Rebind*`, `Rebound`), an
//! unlistened receiver hands its old pipe back (`Park`), and a receiver
//! about to listen takes the old pipes the broker kept for it
//! (`TakeDrains`, `Drains`).
//!
//! An item is a handle value (u64) in the process that sent or receives
//! it, the claimed [`WireKind`] (u8), a flag byte and 6 reserved bytes.
//! Only `Relay`, `Redeemed` and `Drains` have items, at most
//! [`MAX_MESSAGE_HANDLES`]. The flag is set only in `Redeemed`, on a
//! `Receiver`: the broker holds old pipes of that receiver's channel.

use hg_substrate::ipc::MAX_MESSAGE_HANDLES;

use crate::ipc::common::codec::{MAX_DATA_RECORD, WireKind};

/// A ticket: 128 random bits naming handles the broker holds.
pub(crate) type Ticket = [u8; TICKET_LEN];

/// The length of a ticket.
pub(crate) const TICKET_LEN: usize = 16;

/// The ticket of a record that carries no handles.
pub(crate) const NO_TICKET: Ticket = [0; TICKET_LEN];

/// The length of a data message's trailer: its own total length, u32.
pub(crate) const TRAILER_LEN: usize = 4;

/// The longest legal data message.
pub(crate) const MAX_PIPE_DATA: usize = TICKET_LEN + MAX_DATA_RECORD + TRAILER_LEN;

const _: () = assert!(MAX_PIPE_DATA < (1 << 32), "the trailer is a u32");

/// The control record magic, "IZBL" in little-endian order.
pub(crate) const CTL_MAGIC: u32 = u32::from_le_bytes(*b"IZBL");

/// The control record version.
pub(crate) const CTL_VERSION: u8 = 1;

/// The control record header length.
const CTL_HEADER_LEN: usize = 8;

/// An item's length.
const ITEM_LEN: usize = 16;

/// The longest legal control record (a full `Relay`).
pub(crate) const MAX_CTL_LEN: usize = CTL_HEADER_LEN + 16 + MAX_MESSAGE_HANDLES * ITEM_LEN;

const _: () = assert!(MAX_CTL_LEN == 280);

/// Why a data message or control record was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum FrameError {
    /// Fewer bytes than the fixed parts need.
    Short,
    /// More bytes than the kind allows.
    Long,
    /// Not a control record.
    BadMagic,
    /// A version this transport does not speak.
    BadVersion,
    /// An unknown record kind.
    BadKind,
    /// An item count the kind does not allow.
    BadCount,
    /// A reserved byte is not 0.
    Reserved,
    /// An item's kind byte is not a [`WireKind`].
    BadItemKind,
    /// A data message whose trailer states another length: a remainder
    /// left by a read that took the message's head (probe W13c).
    Suffix,
}

/// Splits a data message into its ticket and its data record, checking
/// its trailer.
///
/// # Errors
/// [`FrameError::Short`] for fewer than [`TICKET_LEN`] +
/// [`TRAILER_LEN`] bytes, [`FrameError::Long`] for more than
/// [`MAX_PIPE_DATA`], [`FrameError::Suffix`] when the trailer states
/// another length.
pub(crate) fn split_data(msg: &[u8]) -> Result<(Ticket, &[u8]), FrameError> {
    if msg.len() > MAX_PIPE_DATA {
        return Err(FrameError::Long);
    }
    let (body, trailer) = msg
        .split_last_chunk::<TRAILER_LEN>()
        .ok_or(FrameError::Short)?;
    let (t, record) = body
        .split_first_chunk::<TICKET_LEN>()
        .ok_or(FrameError::Short)?;
    if usize::try_from(u32::from_le_bytes(*trailer)).ok() != Some(msg.len()) {
        return Err(FrameError::Suffix);
    }
    Ok((*t, record))
}

/// A data message: `ticket`, then `record`, then the trailer. A record
/// is at most [`MAX_DATA_RECORD`] bytes, so the length fits the trailer.
pub(crate) fn join_data(ticket: &Ticket, record: &[u8]) -> Vec<u8> {
    let len = TICKET_LEN
        .saturating_add(record.len())
        .saturating_add(TRAILER_LEN);
    let mut out = Vec::with_capacity(len);
    out.extend_from_slice(ticket);
    out.extend_from_slice(record);
    out.extend_from_slice(&u32::try_from(len).unwrap_or(u32::MAX).to_le_bytes());
    out
}

/// One handle in a control record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Item {
    /// The handle's value in the process the record is to or from.
    pub(crate) value: u64,
    /// What the handle claims to be.
    pub(crate) kind: WireKind,
    /// In `Redeemed` only, on a `Receiver`: the broker holds old pipes of
    /// that receiver's channel (a fresh pipe per move, W3), which its
    /// holder takes with `TakeDrains` before it reads. Always false in a
    /// `Relay` or `Drains`.
    pub(crate) drains: bool,
}

impl Item {
    const EMPTY: Self = Self {
        value: 0,
        kind: WireKind::Sender,
        drains: false,
    };
}

/// Up to [`MAX_MESSAGE_HANDLES`] items, without allocating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Items {
    items: [Item; MAX_MESSAGE_HANDLES],
    count: usize,
}

impl Items {
    /// No items.
    pub(crate) const NONE: Self = Self {
        items: [Item::EMPTY; MAX_MESSAGE_HANDLES],
        count: 0,
    };

    /// The items of `from`, or `None` for more than
    /// [`MAX_MESSAGE_HANDLES`].
    pub(crate) fn from_slice(from: &[Item]) -> Option<Self> {
        let mut out = Self::NONE;
        if from.len() > MAX_MESSAGE_HANDLES {
            return None;
        }
        for (slot, it) in out.items.iter_mut().zip(from) {
            *slot = *it;
        }
        out.count = from.len();
        Some(out)
    }

    /// The items.
    pub(crate) fn as_slice(&self) -> &[Item] {
        self.items.get(..self.count).unwrap_or(&[])
    }
}

/// One control record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ctl {
    /// The broker's first record to a child: its launch channel ends,
    /// already in its handle table, and the broker's pid.
    Hello {
        /// The sender to the parent.
        to_parent: u64,
        /// The receiver from the parent.
        from_parent: u64,
        /// The broker's process ID.
        parent_pid: u32,
    },
    /// Take these handles from the child and hold them for a record on
    /// the channel whose sender end is `carrier`.
    Relay {
        /// Matches the answer to the request.
        req: u64,
        /// The child's sender end the record will be written on.
        carrier: u64,
        /// The handles, in the record's descriptor order.
        items: Items,
    },
    /// The handles are held under `ticket`.
    RelayOk {
        /// The request answered.
        req: u64,
        /// The ticket to write in the record.
        ticket: Ticket,
    },
    /// The broker refused the relay; the handles are closed.
    RelayRefused {
        /// The request answered.
        req: u64,
        /// Why ([`super::accept::Refusal::code`]).
        reason: u8,
    },
    /// Give this child the handles held under `ticket`, read from the
    /// channel whose receiver end is `rx`.
    Redeem {
        /// Matches the answer to the request.
        req: u64,
        /// The ticket read from the record.
        ticket: Ticket,
        /// The child's receiver end the record was read from.
        rx: u64,
    },
    /// The handles, now in the child's table, in descriptor order.
    Redeemed {
        /// The request answered.
        req: u64,
        /// The handles and their kinds as the broker found them.
        items: Items,
    },
    /// The broker refused the redemption.
    RedeemRefused {
        /// The request answered.
        req: u64,
        /// Why.
        reason: u8,
    },
    /// The child's receiver `rx` accepts records from `peer` only
    /// (`Ipc::listen_bound`).
    Bind {
        /// The child's receiver end, by handle value, so the broker can
        /// find its channel.
        rx: u64,
        /// The binder's own never-reused id for that listener, echoed in
        /// `BindBroken` so a closed-and-reused handle value cannot be
        /// mistaken for it (design review item 8).
        rx_id: u64,
        /// The peer's process ID.
        peer: u32,
    },
    /// The sender end of the channel the child bound with `Bind` is no
    /// longer held by `peer` alone.
    BindBroken {
        /// The binder's listener id, as given in `Bind`.
        rx_id: u64,
        /// The peer it was bound to.
        peer: u32,
    },
    /// The child closed the bound receiver `rx_id`: drop its binding.
    Unbind {
        /// The binder's listener id, as given in `Bind`.
        rx_id: u64,
    },
    /// The child closed the channel end at handle value `end`, which the
    /// broker had given it: release the ledger entry (items 2, 3).
    Closed {
        /// The end's handle value in the child.
        end: u64,
    },
    /// The broker to a child: the other end of a channel the child holds
    /// is moving, so move the child's end, at handle value `end`, to the
    /// new pipe whose matching end is already in the child's table at
    /// `new` (a fresh pipe per move, W3).
    RebindValue {
        /// Matches the answer.
        req: u64,
        /// The child's end, by handle value (an end the broker gave it).
        end: u64,
        /// The new pipe's end, in the child's table.
        new: u64,
    },
    /// As `RebindValue`, naming the child's end by its channel's pipe id
    /// (a channel the child made itself, whose value the broker never
    /// learnt).
    RebindId {
        /// Matches the answer.
        req: u64,
        /// The id in the channel's pipe name.
        id: [u8; 16],
        /// The new pipe's end, in the child's table.
        new: u64,
    },
    /// A child's answer to a rebind: whether it moved its end. Not
    /// switched when it no longer holds that end, or the end is already
    /// on its way to another process.
    Rebound {
        /// The request answered.
        req: u64,
        /// The end is on the new pipe now.
        switched: bool,
        /// The end is an unlistened receiver: its old pipe's server end
        /// follows in a `Park`. Only with `switched`.
        park: bool,
    },
    /// The child hands back the old pipe of an unlistened receiver that
    /// was moved to a new pipe, for the broker to keep until the
    /// receiver is listened to.
    Park {
        /// The old pipe's server end, by handle value in the child.
        old: u64,
    },
    /// The child is about to listen on receiver `rx`: give it the old
    /// pipes the broker keeps for that receiver's channel.
    TakeDrains {
        /// Matches the answer.
        req: u64,
        /// The child's receiver end, by handle value.
        rx: u64,
    },
    /// The old pipes' server ends, now in the child's table, oldest
    /// first: to read each to its end, then the receiver's own pipe.
    Drains {
        /// The request answered.
        req: u64,
        /// The old pipes, each a `Receiver` item.
        items: Items,
    },
    /// The broker refused `TakeDrains`: the caller does not hold the
    /// receiver, or old pipes of the channel were lost. The receiver
    /// ends in `Closed`.
    DrainsRefused {
        /// The request answered.
        req: u64,
    },
}

impl Ctl {
    const fn kind(&self) -> u8 {
        match self {
            Self::Hello { .. } => 1,
            Self::Relay { .. } => 2,
            Self::RelayOk { .. } => 3,
            Self::RelayRefused { .. } => 4,
            Self::Redeem { .. } => 5,
            Self::Redeemed { .. } => 6,
            Self::RedeemRefused { .. } => 7,
            Self::Bind { .. } => 8,
            Self::BindBroken { .. } => 9,
            Self::Unbind { .. } => 10,
            Self::Closed { .. } => 11,
            Self::RebindValue { .. } => 12,
            Self::RebindId { .. } => 13,
            Self::Rebound { .. } => 14,
            Self::Park { .. } => 15,
            Self::TakeDrains { .. } => 16,
            Self::Drains { .. } => 17,
            Self::DrainsRefused { .. } => 18,
        }
    }
}

/// Under Kani, the control-record writer's buffer: [`MAX_CTL_LEN`] bytes
/// with the same append semantics as a `Vec` (`Out` uses it instead of
/// one). CBMC's cost for `Vec` growth was the measured bottleneck: the
/// 40-byte round-trip harness took 3.26 M symex steps and ran out of
/// memory at 10 GiB with the `Vec` (measurements/milestones/m1-d5, W3
/// Kani). A put past the capacity fails an assertion; it never
/// truncates. `the_kani_buffer_appends_as_a_vec` runs both on the same
/// puts in every test build.
#[cfg(any(kani, test))]
struct FixedBuf {
    bytes: [u8; MAX_CTL_LEN],
    len: usize,
}

#[cfg(any(kani, test))]
impl FixedBuf {
    const fn new() -> Self {
        Self {
            bytes: [0; MAX_CTL_LEN],
            len: 0,
        }
    }

    /// Appends `src`; past the capacity, an assertion fails.
    fn put(&mut self, src: &[u8]) {
        let end = self.len.checked_add(src.len());
        let dst = end.and_then(|e| self.bytes.get_mut(self.len..e));
        assert!(dst.is_some(), "the Kani writer's buffer is full");
        if let (Some(dst), Some(e)) = (dst, end) {
            dst.copy_from_slice(src);
            self.len = e;
        }
    }

    fn into_vec(self) -> Vec<u8> {
        self.bytes
            .get(..self.len)
            .map(<[u8]>::to_vec)
            .unwrap_or_default()
    }
}

/// Appends little-endian fields to a record: to a `Vec`, or under Kani
/// to a [`FixedBuf`].
struct Out {
    #[cfg(not(kani))]
    buf: Vec<u8>,
    #[cfg(kani)]
    buf: FixedBuf,
}

impl Out {
    fn new() -> Self {
        Self {
            #[cfg(not(kani))]
            buf: Vec::with_capacity(MAX_CTL_LEN),
            #[cfg(kani)]
            buf: FixedBuf::new(),
        }
    }
    fn put(&mut self, v: &[u8]) {
        #[cfg(not(kani))]
        self.buf.extend_from_slice(v);
        #[cfg(kani)]
        self.buf.put(v);
    }
    fn into_vec(self) -> Vec<u8> {
        #[cfg(not(kani))]
        return self.buf;
        #[cfg(kani)]
        return self.buf.into_vec();
    }
    fn u8(&mut self, v: u8) {
        self.put(&[v]);
    }
    fn u32(&mut self, v: u32) {
        self.put(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.put(&v.to_le_bytes());
    }
    fn bytes(&mut self, v: &[u8]) {
        self.put(v);
    }
    fn zeros<const N: usize>(&mut self) {
        self.put(&[0; N]);
    }
    fn items(&mut self, items: &Items) {
        for it in items.as_slice() {
            self.u64(it.value);
            self.u8(it.kind.to_byte());
            self.u8(u8::from(it.drains));
            self.zeros::<6>();
        }
    }
}

/// Encodes a control record.
pub(crate) fn encode_ctl(c: &Ctl) -> Vec<u8> {
    let count = match c {
        Ctl::Relay { items, .. } | Ctl::Redeemed { items, .. } | Ctl::Drains { items, .. } => {
            items.as_slice().len()
        }
        Ctl::Hello { .. }
        | Ctl::RelayOk { .. }
        | Ctl::RelayRefused { .. }
        | Ctl::Redeem { .. }
        | Ctl::RedeemRefused { .. }
        | Ctl::Bind { .. }
        | Ctl::BindBroken { .. }
        | Ctl::Unbind { .. }
        | Ctl::Closed { .. }
        | Ctl::RebindValue { .. }
        | Ctl::RebindId { .. }
        | Ctl::Rebound { .. }
        | Ctl::Park { .. }
        | Ctl::TakeDrains { .. }
        | Ctl::DrainsRefused { .. } => 0,
    };
    let mut o = Out::new();
    o.u32(CTL_MAGIC);
    o.u8(CTL_VERSION);
    o.u8(c.kind());
    // At most MAX_MESSAGE_HANDLES (16): `Items` holds no more.
    o.u8(u8::try_from(count).unwrap_or(u8::MAX));
    o.u8(0);
    match c {
        Ctl::Hello {
            to_parent,
            from_parent,
            parent_pid,
        } => {
            o.u64(*to_parent);
            o.u64(*from_parent);
            o.u32(*parent_pid);
            o.u32(0);
        }
        Ctl::Relay {
            req,
            carrier,
            items,
        } => {
            o.u64(*req);
            o.u64(*carrier);
            o.items(items);
        }
        Ctl::RelayOk { req, ticket } => {
            o.u64(*req);
            o.bytes(ticket);
        }
        Ctl::RelayRefused { req, reason } | Ctl::RedeemRefused { req, reason } => {
            o.u64(*req);
            o.u8(*reason);
            o.zeros::<7>();
        }
        Ctl::Redeem { req, ticket, rx } => {
            o.u64(*req);
            o.bytes(ticket);
            o.u64(*rx);
        }
        Ctl::Redeemed { req, items } => {
            o.u64(*req);
            o.items(items);
        }
        Ctl::Bind { rx, rx_id, peer } => {
            o.u64(*rx);
            o.u64(*rx_id);
            o.u32(*peer);
            o.u32(0);
        }
        Ctl::BindBroken { rx_id, peer } => {
            o.u64(*rx_id);
            o.u32(*peer);
            o.u32(0);
        }
        Ctl::Unbind { rx_id } => {
            o.u64(*rx_id);
            o.zeros::<8>();
        }
        Ctl::Closed { end } => {
            o.u64(*end);
            o.zeros::<8>();
        }
        Ctl::RebindValue { req, end, new } => {
            o.u64(*req);
            o.u64(*end);
            o.u64(*new);
            o.zeros::<8>();
        }
        Ctl::RebindId { req, id, new } => {
            o.u64(*req);
            o.bytes(id);
            o.u64(*new);
        }
        Ctl::Rebound {
            req,
            switched,
            park,
        } => {
            o.u64(*req);
            o.u8(match (*switched, *park) {
                (false, _) => 0,
                (true, false) => 1,
                (true, true) => 2,
            });
            o.zeros::<7>();
        }
        Ctl::Park { old } => {
            o.u64(*old);
            o.zeros::<8>();
        }
        Ctl::TakeDrains { req, rx } => {
            o.u64(*req);
            o.u64(*rx);
        }
        Ctl::Drains { req, items } => {
            o.u64(*req);
            o.items(items);
        }
        Ctl::DrainsRefused { req } => {
            o.u64(*req);
            o.zeros::<8>();
        }
    }
    o.into_vec()
}

/// Reads little-endian fields from a record, front to back.
struct In<'a>(&'a [u8]);

impl In<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], FrameError> {
        let (head, rest) = self.0.split_first_chunk::<N>().ok_or(FrameError::Short)?;
        self.0 = rest;
        Ok(*head)
    }
    fn u8(&mut self) -> Result<u8, FrameError> {
        Ok(self.take::<1>()?[0])
    }
    fn u32(&mut self) -> Result<u32, FrameError> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    fn u64(&mut self) -> Result<u64, FrameError> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn zeros<const N: usize>(&mut self) -> Result<(), FrameError> {
        if self.take::<N>()? == [0; N] {
            Ok(())
        } else {
            Err(FrameError::Reserved)
        }
    }
    /// `count` items; a set flag only where `flag_ok` (a `Redeemed`) and
    /// only on a `Receiver`.
    fn items(&mut self, count: usize, flag_ok: bool) -> Result<Items, FrameError> {
        let mut out = Items::NONE;
        for (i, slot) in out.items.iter_mut().enumerate() {
            if i >= count {
                break;
            }
            let value = self.u64()?;
            let kind = WireKind::from_byte(self.u8()?).ok_or(FrameError::BadItemKind)?;
            let drains = match self.u8()? {
                0 => false,
                1 if flag_ok && kind == WireKind::Receiver => true,
                _ => return Err(FrameError::Reserved),
            };
            self.zeros::<6>()?;
            *slot = Item {
                value,
                kind,
                drains,
            };
        }
        out.count = count;
        Ok(out)
    }
    fn end(&self) -> Result<(), FrameError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(FrameError::Long)
        }
    }
}

/// Decodes a control record: exactly one, whole.
///
/// # Errors
/// Any [`FrameError`].
pub(crate) fn decode_ctl(input: &[u8]) -> Result<Ctl, FrameError> {
    if input.len() > MAX_CTL_LEN {
        return Err(FrameError::Long);
    }
    let mut r = In(input);
    if r.u32()? != CTL_MAGIC {
        return Err(FrameError::BadMagic);
    }
    if r.u8()? != CTL_VERSION {
        return Err(FrameError::BadVersion);
    }
    let kind = r.u8()?;
    let count = usize::from(r.u8()?);
    r.zeros::<1>()?;
    let has_items = matches!(kind, 2 | 6 | 17);
    if count > MAX_MESSAGE_HANDLES || (!has_items && count != 0) {
        return Err(FrameError::BadCount);
    }
    let c = match kind {
        1 => {
            let to_parent = r.u64()?;
            let from_parent = r.u64()?;
            let parent_pid = r.u32()?;
            r.zeros::<4>()?;
            Ctl::Hello {
                to_parent,
                from_parent,
                parent_pid,
            }
        }
        2 => {
            let req = r.u64()?;
            let carrier = r.u64()?;
            let items = r.items(count, false)?;
            Ctl::Relay {
                req,
                carrier,
                items,
            }
        }
        3 => {
            let req = r.u64()?;
            let ticket = r.take::<TICKET_LEN>()?;
            Ctl::RelayOk { req, ticket }
        }
        4 | 7 => {
            let req = r.u64()?;
            let reason = r.u8()?;
            r.zeros::<7>()?;
            if kind == 4 {
                Ctl::RelayRefused { req, reason }
            } else {
                Ctl::RedeemRefused { req, reason }
            }
        }
        5 => {
            let req = r.u64()?;
            let ticket = r.take::<TICKET_LEN>()?;
            let rx = r.u64()?;
            Ctl::Redeem { req, ticket, rx }
        }
        6 => {
            let req = r.u64()?;
            let items = r.items(count, true)?;
            Ctl::Redeemed { req, items }
        }
        8 => {
            let rx = r.u64()?;
            let rx_id = r.u64()?;
            let peer = r.u32()?;
            r.zeros::<4>()?;
            Ctl::Bind { rx, rx_id, peer }
        }
        9 => {
            let rx_id = r.u64()?;
            let peer = r.u32()?;
            r.zeros::<4>()?;
            Ctl::BindBroken { rx_id, peer }
        }
        10 => {
            let rx_id = r.u64()?;
            r.zeros::<8>()?;
            Ctl::Unbind { rx_id }
        }
        11 => {
            let end = r.u64()?;
            r.zeros::<8>()?;
            Ctl::Closed { end }
        }
        12 => {
            let req = r.u64()?;
            let end = r.u64()?;
            let new = r.u64()?;
            r.zeros::<8>()?;
            Ctl::RebindValue { req, end, new }
        }
        13 => {
            let req = r.u64()?;
            let id = r.take::<16>()?;
            let new = r.u64()?;
            Ctl::RebindId { req, id, new }
        }
        14 => {
            let req = r.u64()?;
            let (switched, park) = match r.u8()? {
                0 => (false, false),
                1 => (true, false),
                2 => (true, true),
                _ => return Err(FrameError::Reserved),
            };
            r.zeros::<7>()?;
            Ctl::Rebound {
                req,
                switched,
                park,
            }
        }
        15 => {
            let old = r.u64()?;
            r.zeros::<8>()?;
            Ctl::Park { old }
        }
        16 => {
            let req = r.u64()?;
            let rx = r.u64()?;
            Ctl::TakeDrains { req, rx }
        }
        17 => {
            let req = r.u64()?;
            let items = r.items(count, false)?;
            Ctl::Drains { req, items }
        }
        18 => {
            let req = r.u64()?;
            r.zeros::<8>()?;
            Ctl::DrainsRefused { req }
        }
        _ => return Err(FrameError::BadKind),
    };
    r.end()?;
    Ok(c)
}

/// VER-1 for the control records and data messages (IPC-R6): no input
/// makes a decoder panic or read out of bounds, everything accepted
/// re-encodes to the same bytes, and no proper suffix of a data message
/// is accepted (W13c).
#[cfg(kani)]
mod proofs {
    use super::*;

    /// Any input of `N` bytes: decoding never panics, and an accepted
    /// record encodes back to exactly the input.
    fn ctl_round_trip<const N: usize>() {
        let input: [u8; N] = kani::any();
        if let Ok(c) = decode_ctl(&input) {
            let back = encode_ctl(&c);
            assert!(back.len() == N);
            for i in 0..N {
                assert!(back[i] == input[i]);
            }
        }
    }

    // Every record length with up to two items has a harness: 16 (a
    // zero-item Redeemed or Drains), 24 (RelayRefused, RedeemRefused,
    // BindBroken, Unbind, Closed, Rebound, Park, TakeDrains,
    // DrainsRefused, a zero-item Relay), 32 (Hello, RelayOk, Bind, a
    // one-item Redeemed or Drains), 40 (Redeem, RebindValue, RebindId, a
    // one-item Relay), 48 (a two-item Redeemed or Drains) and 56 (a
    // two-item Relay). Records of three to sixteen items (64 to 280
    // bytes) have no harness of their own: each further item is decoded
    // by the same loop body the two-item cases check. Each harness
    // accepts only inputs that re-encode to their own length, so a
    // decoder that read past or short of a record fails it.
    #[kani::proof]
    #[kani::unwind(18)]
    fn ctl_round_trips_16() {
        ctl_round_trip::<16>();
    }

    #[kani::proof]
    #[kani::unwind(26)]
    fn ctl_round_trips_24() {
        ctl_round_trip::<24>();
    }

    #[kani::proof]
    #[kani::unwind(34)]
    fn ctl_round_trips_32() {
        ctl_round_trip::<32>();
    }

    #[kani::proof]
    #[kani::unwind(42)]
    fn ctl_round_trips_40() {
        ctl_round_trip::<40>();
    }

    #[kani::proof]
    #[kani::unwind(50)]
    fn ctl_round_trips_48() {
        ctl_round_trip::<48>();
    }

    #[kani::proof]
    #[kani::unwind(58)]
    fn ctl_round_trips_56() {
        ctl_round_trip::<56>();
    }

    /// A one-item `Relay` (40 bytes) and a two-item `Relay` (56 bytes)
    /// are reachable, so the length harnesses are not vacuous for items
    /// or for multi-item records.
    #[kani::proof]
    #[kani::unwind(42)]
    fn a_relay_decodes() {
        let input: [u8; 40] = kani::any();
        let r = decode_ctl(&input);
        kani::cover!(matches!(r, Ok(Ctl::Relay { .. })), "a relay with one item");
        kani::cover!(matches!(r, Ok(Ctl::Redeem { .. })), "a redeem");
        kani::cover!(matches!(r, Ok(Ctl::RebindId { .. })), "a rebind by id");
    }

    #[kani::proof]
    #[kani::unwind(58)]
    fn a_two_item_relay_decodes() {
        let input: [u8; 56] = kani::any();
        let r = decode_ctl(&input);
        kani::cover!(
            matches!(r, Ok(Ctl::Relay { items, .. }) if items.as_slice().len() == 2),
            "a relay carrying two items"
        );
    }

    /// The W3 records with items or flags are reachable: a `Redeemed`
    /// whose receiver has old pipes, and `Drains` with two of them.
    #[kani::proof]
    #[kani::unwind(50)]
    fn the_w3_records_decode() {
        let input: [u8; 48] = kani::any();
        let r = decode_ctl(&input);
        kani::cover!(
            matches!(r, Ok(Ctl::Drains { items, .. }) if items.as_slice().len() == 2),
            "two old pipes"
        );
        kani::cover!(
            matches!(r, Ok(Ctl::Redeemed { items, .. })
                if items.as_slice().iter().any(|i| i.drains)),
            "a redeemed receiver with old pipes"
        );
    }

    /// The longest message these harnesses build: a ticket, `MAX_REC`
    /// record bytes and the trailer.
    const MAX_REC: usize = 4;
    const MAX_MSG: usize = TICKET_LEN + MAX_REC + TRAILER_LEN;

    /// Data messages, at every length up to `MAX_MSG`: the split never
    /// panics, and it accepts exactly the messages whose trailer states
    /// their length, giving back the ticket and the bytes between.
    #[kani::proof]
    #[kani::unwind(26)]
    fn data_split_checks_the_trailer() {
        let input: [u8; MAX_MSG] = kani::any();
        let n: usize = kani::any_where(|&n: &usize| n <= MAX_MSG);
        let msg = &input[..n];
        match split_data(msg) {
            Ok((t, rest)) => {
                assert!(n >= TICKET_LEN + TRAILER_LEN);
                let stated =
                    u32::from_le_bytes([input[n - 4], input[n - 3], input[n - 2], input[n - 1]]);
                assert!(stated as usize == n);
                assert!(rest.len() == n - TICKET_LEN - TRAILER_LEN);
                for i in 0..TICKET_LEN {
                    assert!(t[i] == input[i]);
                }
                for i in 0..rest.len() {
                    assert!(rest[i] == input[TICKET_LEN + i]);
                }
            }
            Err(FrameError::Short) => assert!(n < TICKET_LEN + TRAILER_LEN),
            Err(e) => assert!(e == FrameError::Suffix),
        }
    }

    /// Any ticket and record of any length up to `MAX_REC`: the message
    /// [`join_data`] makes splits back to them, and **no proper suffix of
    /// it splits** (W13c: what a read that took the message's head leaves
    /// behind is refused).
    #[kani::proof]
    #[kani::unwind(26)]
    fn a_message_round_trips_and_no_suffix_is_accepted() {
        let ticket: Ticket = kani::any();
        let rec: [u8; MAX_REC] = kani::any();
        let len: usize = kani::any_where(|&l: &usize| l <= MAX_REC);
        let msg = join_data(&ticket, &rec[..len]);
        assert!(msg.len() == TICKET_LEN + len + TRAILER_LEN);
        match split_data(&msg) {
            Ok((t, r)) => {
                assert!(t == ticket);
                assert!(r == &rec[..len]);
            }
            Err(_) => panic!("a whole message is accepted"),
        }
        let cut: usize = kani::any_where(|&c: &usize| c >= 1 && c <= msg.len());
        assert!(split_data(&msg[cut..]).is_err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Kani writer's buffer appends exactly as the `Vec` the real
    /// writer uses: the same puts (random lengths and bytes, up to the
    /// capacity) give the same bytes.
    #[test]
    fn the_kani_buffer_appends_as_a_vec() {
        let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = |m: u64| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed % m
        };
        for _ in 0..500 {
            let (mut real, mut model) = (Vec::new(), FixedBuf::new());
            loop {
                let room = MAX_CTL_LEN.saturating_sub(real.len());
                let n = usize::try_from(next(17)).unwrap().min(room);
                let chunk: Vec<u8> = (0..n).map(|_| u8::try_from(next(256)).unwrap()).collect();
                real.extend_from_slice(&chunk);
                model.put(&chunk);
                if real.len() == MAX_CTL_LEN || next(16) == 0 {
                    break;
                }
            }
            assert_eq!(model.into_vec(), real);
        }
    }

    #[test]
    #[should_panic(expected = "the Kani writer's buffer is full")]
    fn the_kani_buffer_refuses_a_put_past_its_capacity() {
        let mut model = FixedBuf::new();
        model.put(&[0; MAX_CTL_LEN]);
        model.put(&[0]);
    }

    fn items(n: usize) -> Items {
        let v: Vec<Item> = (1..=n)
            .map(|i| Item {
                value: u64::try_from(i).unwrap().checked_mul(4).unwrap(),
                kind: if i % 2 == 0 {
                    WireKind::Sender
                } else {
                    WireKind::Receiver
                },
                drains: false,
            })
            .collect();
        Items::from_slice(&v).unwrap()
    }

    fn every_kind() -> Vec<Ctl> {
        vec![
            Ctl::Hello {
                to_parent: 0x10,
                from_parent: 0x14,
                parent_pid: 4242,
            },
            Ctl::Relay {
                req: 7,
                carrier: 0x20,
                items: items(16),
            },
            Ctl::RelayOk {
                req: 7,
                ticket: [9; 16],
            },
            Ctl::RelayRefused { req: 7, reason: 3 },
            Ctl::Redeem {
                req: 8,
                ticket: [1; 16],
                rx: 0x30,
            },
            Ctl::Redeemed {
                req: 8,
                items: items(3),
            },
            Ctl::RedeemRefused { req: 8, reason: 1 },
            Ctl::Bind {
                rx: 0x30,
                rx_id: 0x44,
                peer: 77,
            },
            Ctl::BindBroken {
                rx_id: 0x44,
                peer: 77,
            },
            Ctl::Unbind { rx_id: 0x44 },
            Ctl::Closed { end: 0x30 },
            Ctl::RebindValue {
                req: 9,
                end: 0x30,
                new: 0x34,
            },
            Ctl::RebindId {
                req: 10,
                id: [7; 16],
                new: 0x38,
            },
            Ctl::Rebound {
                req: 9,
                switched: true,
                park: false,
            },
            Ctl::Rebound {
                req: 9,
                switched: true,
                park: true,
            },
            Ctl::Rebound {
                req: 9,
                switched: false,
                park: false,
            },
            Ctl::Park { old: 0x48 },
            Ctl::TakeDrains { req: 12, rx: 0x4c },
            Ctl::Drains {
                req: 12,
                items: Items::from_slice(
                    &[Item {
                        value: 0x50,
                        kind: WireKind::Receiver,
                        drains: false,
                    }; 16],
                )
                .unwrap(),
            },
            Ctl::DrainsRefused { req: 13 },
            Ctl::Redeemed {
                req: 11,
                items: Items::from_slice(&[
                    Item {
                        value: 0x40,
                        kind: WireKind::Receiver,
                        drains: true,
                    },
                    Item {
                        value: 0x44,
                        kind: WireKind::Receiver,
                        drains: false,
                    },
                ])
                .unwrap(),
            },
        ]
    }

    #[test]
    fn every_record_round_trips() {
        for c in every_kind() {
            let b = encode_ctl(&c);
            assert!(b.len() <= MAX_CTL_LEN);
            assert_eq!(decode_ctl(&b), Ok(c));
        }
    }

    #[test]
    fn malformed_records_are_refused() {
        let good = encode_ctl(&Ctl::Bind {
            rx: 4,
            rx_id: 5,
            peer: 1,
        });
        let mut bad = good.clone();
        bad[0] ^= 1;
        assert_eq!(decode_ctl(&bad), Err(FrameError::BadMagic));
        let mut bad = good.clone();
        bad[4] = 2;
        assert_eq!(decode_ctl(&bad), Err(FrameError::BadVersion));
        let mut bad = good.clone();
        bad[5] = 0;
        assert_eq!(decode_ctl(&bad), Err(FrameError::BadKind));
        let mut bad = good.clone();
        bad[6] = 1;
        assert_eq!(decode_ctl(&bad), Err(FrameError::BadCount));
        let mut bad = good.clone();
        bad[7] = 1;
        assert_eq!(decode_ctl(&bad), Err(FrameError::Reserved));
        let mut bad = good.clone();
        bad.push(0);
        assert_eq!(decode_ctl(&bad), Err(FrameError::Long));
        assert_eq!(decode_ctl(&good[..good.len() - 1]), Err(FrameError::Short));
        let mut relay = encode_ctl(&Ctl::Relay {
            req: 1,
            carrier: 4,
            items: items(1),
        });
        // The first item's kind byte: header 8, request 8, carrier 8,
        // value 8.
        relay[32] = 0;
        assert_eq!(decode_ctl(&relay), Err(FrameError::BadItemKind));
        relay[6] = 17;
        assert_eq!(decode_ctl(&relay), Err(FrameError::BadCount));
        // The drain flag (the byte after the kind) is refused in a Relay,
        // and on anything but a Receiver in a Redeemed.
        let mut relay = encode_ctl(&Ctl::Relay {
            req: 1,
            carrier: 4,
            items: items(1),
        });
        relay[33] = 1;
        assert_eq!(decode_ctl(&relay), Err(FrameError::Reserved));
        let sender = Items::from_slice(&[Item {
            value: 8,
            kind: WireKind::Sender,
            drains: false,
        }])
        .unwrap();
        let mut redeemed = encode_ctl(&Ctl::Redeemed {
            req: 1,
            items: sender,
        });
        // Header 8, request 8, value 8, kind 1.
        redeemed[25] = 1;
        assert_eq!(decode_ctl(&redeemed), Err(FrameError::Reserved));
        let mut rebound = encode_ctl(&Ctl::Rebound {
            req: 1,
            switched: true,
            park: false,
        });
        rebound[16] = 3;
        assert_eq!(decode_ctl(&rebound), Err(FrameError::Reserved));
        // A `Drains` item never carries the flag.
        let mut drains = encode_ctl(&Ctl::Drains {
            req: 1,
            items: Items::from_slice(&[Item {
                value: 8,
                kind: WireKind::Receiver,
                drains: false,
            }])
            .unwrap(),
        });
        drains[25] = 1;
        assert_eq!(decode_ctl(&drains), Err(FrameError::Reserved));
        // A Park with a count, a TakeDrains one byte short.
        let mut park = encode_ctl(&Ctl::Park { old: 8 });
        park[6] = 1;
        assert_eq!(decode_ctl(&park), Err(FrameError::BadCount));
        let take = encode_ctl(&Ctl::TakeDrains { req: 1, rx: 8 });
        assert_eq!(decode_ctl(&take[..23]), Err(FrameError::Short));
    }

    #[test]
    fn data_messages_split_at_the_ticket() {
        let m = join_data(&[3; 16], b"record");
        assert_eq!(m.len(), 16 + 6 + TRAILER_LEN);
        assert_eq!(split_data(&m), Ok(([3; 16], &b"record"[..])));
        assert_eq!(split_data(&m[..19]), Err(FrameError::Short));
        let long = vec![0u8; MAX_PIPE_DATA + 1];
        assert_eq!(split_data(&long), Err(FrameError::Long));
        // The longest record still fits, trailer and all.
        let big = join_data(&NO_TICKET, &vec![1u8; MAX_DATA_RECORD]);
        assert_eq!(big.len(), MAX_PIPE_DATA);
        assert!(split_data(&big).is_ok());
        // A wrong trailer is refused.
        let mut bad = m.clone();
        bad[m.len() - 1] = 1;
        assert_eq!(split_data(&bad), Err(FrameError::Suffix));
    }

    /// W13c: a read left pending by a former holder took the head of a
    /// message, and the rest reaches the new holder as a message of its
    /// own. Shaped here as an attacker would: a payload that is itself a
    /// whole message (ticket, record, trailer), cut just after the outer
    /// header. Without the trailer check that suffix decoded as a valid
    /// record; now every proper suffix is refused.
    #[test]
    fn no_suffix_of_a_message_is_accepted() {
        use crate::ipc::common::codec::{DATA_HEADER_LEN, decode_data, encode_data};
        let inner = join_data(&NO_TICKET, &encode_data(7, &[], b"forged").unwrap());
        let outer = join_data(&NO_TICKET, &encode_data(7, &[], &inner).unwrap());
        let cut = TICKET_LEN + DATA_HEADER_LEN;
        let suffix = &outer[cut..];
        // The suffix starts with the forged message, whole.
        assert_eq!(&suffix[..inner.len()], &inner[..]);
        assert_eq!(split_data(suffix), Err(FrameError::Suffix));
        // Without the trailer check, the forged record would decode, with
        // the expected seq.
        let forged = &suffix[TICKET_LEN..suffix.len() - TRAILER_LEN - TRAILER_LEN];
        assert_eq!(decode_data(forged).unwrap().seq, 7);
        for k in 1..=outer.len() {
            assert!(split_data(&outer[k..]).is_err(), "suffix at {k}");
        }
    }
}
