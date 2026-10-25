//! What crosses a Windows pipe besides the shared record formats
//! (IPC-R6: one fixed little-endian layout, decoded without allocating).
//!
//! **Data messages.** A channel is one duplex message-mode pipe: data
//! messages from the sender end, credit records
//! ([`crate::ipc::common::codec`]) back from the receiver end. Each data
//! message is a [`TICKET_LEN`]-byte ticket followed by a data record. The
//! ticket is all zeros when the record carries no handles; otherwise it
//! names the handles the broker holds for this record, which the
//! receiver redeems over its control link. Handle values never travel
//! in a channel: a value written by a peer would name whatever this
//! process happens to hold under that number (design review B1).
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
//!
//! An item is a handle value (u64) in the process that sent or receives
//! it, the claimed [`WireKind`] (u8) and 7 reserved bytes. Only `Relay`
//! and `Redeemed` have items, at most [`MAX_MESSAGE_HANDLES`].

use hg_substrate::ipc::MAX_MESSAGE_HANDLES;

use crate::ipc::common::codec::{MAX_DATA_RECORD, WireKind};

/// A ticket: 128 random bits naming handles the broker holds.
pub(crate) type Ticket = [u8; TICKET_LEN];

/// The length of a ticket.
pub(crate) const TICKET_LEN: usize = 16;

/// The ticket of a record that carries no handles.
pub(crate) const NO_TICKET: Ticket = [0; TICKET_LEN];

/// The longest legal data message.
pub(crate) const MAX_PIPE_DATA: usize = TICKET_LEN + MAX_DATA_RECORD;

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
}

/// Splits a data message into its ticket and its data record.
///
/// # Errors
/// [`FrameError::Short`] for fewer than [`TICKET_LEN`] bytes,
/// [`FrameError::Long`] for more than [`MAX_PIPE_DATA`].
pub(crate) fn split_data(msg: &[u8]) -> Result<(Ticket, &[u8]), FrameError> {
    if msg.len() > MAX_PIPE_DATA {
        return Err(FrameError::Long);
    }
    let (t, rest) = msg
        .split_first_chunk::<TICKET_LEN>()
        .ok_or(FrameError::Short)?;
    Ok((*t, rest))
}

/// A data message: `ticket`, then `record`.
pub(crate) fn join_data(ticket: &Ticket, record: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(TICKET_LEN.saturating_add(record.len()));
    out.extend_from_slice(ticket);
    out.extend_from_slice(record);
    out
}

/// One handle in a control record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Item {
    /// The handle's value in the process the record is to or from.
    pub(crate) value: u64,
    /// What the handle claims to be.
    pub(crate) kind: WireKind,
}

impl Item {
    const EMPTY: Self = Self {
        value: 0,
        kind: WireKind::Sender,
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
        /// The child's receiver end.
        rx: u64,
        /// The peer's process ID.
        peer: u32,
    },
    /// The sender end of the channel the child bound with `Bind` is no
    /// longer held by `peer` alone.
    BindBroken {
        /// The child's receiver end, as given in `Bind`.
        rx: u64,
        /// The peer it was bound to.
        peer: u32,
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
        }
    }
}

/// Appends little-endian fields to a record.
struct Out(Vec<u8>);

impl Out {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, v: &[u8]) {
        self.0.extend_from_slice(v);
    }
    fn zeros(&mut self, n: usize) {
        self.0.extend(core::iter::repeat_n(0, n));
    }
    fn items(&mut self, items: &Items) {
        for it in items.as_slice() {
            self.u64(it.value);
            self.u8(it.kind.to_byte());
            self.zeros(7);
        }
    }
}

/// Encodes a control record.
pub(crate) fn encode_ctl(c: &Ctl) -> Vec<u8> {
    let count = match c {
        Ctl::Relay { items, .. } | Ctl::Redeemed { items, .. } => items.as_slice().len(),
        Ctl::Hello { .. }
        | Ctl::RelayOk { .. }
        | Ctl::RelayRefused { .. }
        | Ctl::Redeem { .. }
        | Ctl::RedeemRefused { .. }
        | Ctl::Bind { .. }
        | Ctl::BindBroken { .. } => 0,
    };
    let mut o = Out(Vec::with_capacity(MAX_CTL_LEN));
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
            o.zeros(7);
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
        Ctl::Bind { rx, peer } | Ctl::BindBroken { rx, peer } => {
            o.u64(*rx);
            o.u32(*peer);
            o.u32(0);
        }
    }
    o.0
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
    fn items(&mut self, count: usize) -> Result<Items, FrameError> {
        let mut out = Items::NONE;
        for (i, slot) in out.items.iter_mut().enumerate() {
            if i >= count {
                break;
            }
            let value = self.u64()?;
            let kind = WireKind::from_byte(self.u8()?).ok_or(FrameError::BadItemKind)?;
            self.zeros::<7>()?;
            *slot = Item { value, kind };
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
    let has_items = matches!(kind, 2 | 6);
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
            let items = r.items(count)?;
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
            let items = r.items(count)?;
            Ctl::Redeemed { req, items }
        }
        8 | 9 => {
            let rx = r.u64()?;
            let peer = r.u32()?;
            r.zeros::<4>()?;
            if kind == 8 {
                Ctl::Bind { rx, peer }
            } else {
                Ctl::BindBroken { rx, peer }
            }
        }
        _ => return Err(FrameError::BadKind),
    };
    r.end()?;
    Ok(c)
}

/// VER-1 for the control records (IPC-R6): no input makes the decoder
/// panic or read out of bounds, and everything it accepts re-encodes to
/// the same bytes.
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

    /// A one-item `Relay` (40 bytes) is reachable, so the harness above
    /// is not vacuous for items.
    #[kani::proof]
    #[kani::unwind(42)]
    fn a_relay_decodes() {
        let input: [u8; 40] = kani::any();
        let r = decode_ctl(&input);
        kani::cover!(matches!(r, Ok(Ctl::Relay { .. })), "a relay with one item");
        kani::cover!(matches!(r, Ok(Ctl::Redeem { .. })), "a redeem");
    }

    /// Data messages: the split never panics and keeps every byte.
    #[kani::proof]
    #[kani::unwind(20)]
    fn data_split_keeps_every_byte() {
        let input: [u8; 18] = kani::any();
        let n: usize = kani::any_where(|&n: &usize| n <= 18);
        match split_data(&input[..n]) {
            Ok((t, rest)) => {
                assert!(n >= TICKET_LEN);
                assert!(rest.len() == n - TICKET_LEN);
                for i in 0..TICKET_LEN {
                    assert!(t[i] == input[i]);
                }
            }
            Err(e) => assert!(n < TICKET_LEN && e == FrameError::Short),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(n: usize) -> Items {
        let v: Vec<Item> = (1..=n)
            .map(|i| Item {
                value: u64::try_from(i).unwrap().checked_mul(4).unwrap(),
                kind: if i % 2 == 0 {
                    WireKind::Sender
                } else {
                    WireKind::Receiver
                },
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
            Ctl::Bind { rx: 0x30, peer: 77 },
            Ctl::BindBroken { rx: 0x30, peer: 77 },
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
        let good = encode_ctl(&Ctl::Bind { rx: 4, peer: 1 });
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
    }

    #[test]
    fn data_messages_split_at_the_ticket() {
        let m = join_data(&[3; 16], b"record");
        assert_eq!(split_data(&m), Ok(([3; 16], &b"record"[..])));
        assert_eq!(split_data(&m[..15]), Err(FrameError::Short));
        let long = vec![0u8; MAX_PIPE_DATA + 1];
        assert_eq!(split_data(&long), Err(FrameError::Long));
    }
}
