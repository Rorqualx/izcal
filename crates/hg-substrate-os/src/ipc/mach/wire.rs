//! The Mach message layout, as the D4 probes measured it on macOS 27
//! (Apple M6, 25 October 2026), and our own parser of the buffer
//! `mach_msg` fills.
//!
//! A message is a 24-byte header of six 32-bit words, `[bits, size,
//! remote, local, voucher, id]` (P1, P11): `bits` holds the remote
//! right's disposition in its low byte and the local right's in the
//! next (P12), and bit 31 marks a message that carries descriptors
//! ("complex", P11). A complex message continues with a 32-bit
//! descriptor count and the descriptors: a port descriptor is 12 bytes
//! `{name, 0, pad:16 | disposition:8 | type:8}` with type 0; an
//! out-of-line memory descriptor is 16 bytes `{address: u64, deallocate:8
//! | copy:8 | pad:8 | type:8, size}` with type 1; an out-of-line port
//! array has the same shape with type 2 and its disposition in byte 2
//! (P11, P12, P31). The body follows. On receive the kernel appends a
//! trailer at the body's end rounded up to 4 bytes: `[type, size,
//! seqno, security token x2, audit token x8]`, 52 bytes when 3 trailer
//! elements are asked for (P1). Audit-token word 5 is the sender's pid
//! and word 7 its pid version, which changes on exec and is never
//! reused (P1, P3). A sender cannot write the trailer: bytes it adds
//! arrive as body (P2). The kernel's own notifications carry an all-zero
//! token (P4).
//!
//! All words are little-endian: the transport runs on 64-bit Apple
//! silicon and x86-64 only.
//!
//! The parser never trusts a count or an offset it reads: every read is
//! bounds-checked against the buffer, and the descriptor walk stops at
//! the first descriptor it cannot read or does not know.

use super::super::common::codec::MAX_DATA_RECORD;
use hg_substrate::ipc::MAX_MESSAGE_HANDLES;

// ---- constants, each found black-box (probe ids from mach_facts.md) ----

/// `mach_msg` option: send (P12).
pub(crate) const OPT_SEND: i32 = 0x1;
/// `mach_msg` option: receive (P12).
pub(crate) const OPT_RCV: i32 = 0x2;
/// `mach_msg` option: a message too large for the buffer stays queued
/// and its size is reported (P24).
pub(crate) const OPT_RCV_LARGE: i32 = 0x4;
/// `mach_msg` option: the send times out (with timeout 0, at once) when
/// the queue is full (P17).
pub(crate) const OPT_SEND_TIMEOUT: i32 = 0x10;
/// `mach_msg` option: a send that fails because the queue is full arms
/// the destination's send-possible request (P19).
pub(crate) const OPT_SEND_NOTIFY: i32 = 0x80;
/// `mach_msg` option: the receive times out (P12).
pub(crate) const OPT_RCV_TIMEOUT: i32 = 0x100;
/// `mach_msg` option: three trailer elements, which include the audit
/// token (P1).
pub(crate) const OPT_TRAILER_AUDIT: i32 = 3 << 24;

/// The only send options this transport passes. An unknown option bit
/// gets the sending process killed by the kernel (P2), so every send is
/// built from these and nothing else.
pub(crate) const SEND_OPTIONS: i32 = OPT_SEND | OPT_SEND_TIMEOUT | OPT_SEND_NOTIFY;
/// The only receive options this transport passes, without a timeout.
pub(crate) const RCV_OPTIONS: i32 = OPT_RCV | OPT_RCV_LARGE | OPT_TRAILER_AUDIT;

/// Return: the destination queue is full (P16, P17).
pub(crate) const SEND_TIMED_OUT: i32 = 0x1000_0004;
/// Return: the destination port is gone (P21).
pub(crate) const SEND_INVALID_DEST: i32 = 0x1000_0003;
/// Return: the message did not fit the buffer (P24).
pub(crate) const RCV_TOO_LARGE: i32 = 0x1000_4004;
/// Return: nothing arrived before the timeout (P12).
pub(crate) const RCV_TIMED_OUT: i32 = 0x1000_4003;

/// Disposition: move the receive right (P12).
pub(crate) const MOVE_RECEIVE: u8 = 16;
/// Disposition: move a send right (P12).
pub(crate) const MOVE_SEND: u8 = 17;
/// Disposition: move a send-once right (P12).
pub(crate) const MOVE_SEND_ONCE: u8 = 18;
/// Disposition: copy a send right (P12).
pub(crate) const COPY_SEND: u8 = 19;
/// Disposition: make a send right from a receive right (P12).
pub(crate) const MAKE_SEND: u8 = 20;
/// Disposition: make a send-once right from a receive right (P12).
pub(crate) const MAKE_SEND_ONCE: u8 = 21;
/// A received receive right's disposition (P12).
pub(crate) const PORT_RECEIVE: u8 = 16;
/// A received send right's disposition (P12).
pub(crate) const PORT_SEND: u8 = 17;
/// A received send-once right's disposition (P12).
pub(crate) const PORT_SEND_ONCE: u8 = 18;

/// `bits`: the message carries descriptors (P11).
pub(crate) const BITS_COMPLEX: u32 = 0x8000_0000;

/// Descriptor type: one port right (P11).
pub(crate) const DESC_PORT: u8 = 0;
/// Descriptor type: out-of-line memory (P11).
pub(crate) const DESC_OOL: u8 = 1;
/// Descriptor type: an out-of-line array of port rights (P31).
pub(crate) const DESC_OOL_PORTS: u8 = 2;

/// Right index for `mach_port_*` calls: a send right (P7).
pub(crate) const RIGHT_SEND: u32 = 0;
/// Right index: the receive right (P7).
pub(crate) const RIGHT_RECEIVE: u32 = 1;
/// Right index: a port set (P7).
pub(crate) const RIGHT_PORT_SET: u32 = 3;

/// `mach_port_type` bit: holds send rights (P7).
pub(crate) const TYPE_SEND: u32 = 1 << 16;
/// `mach_port_type` bit: holds the receive right (P7).
pub(crate) const TYPE_RECEIVE: u32 = 1 << 17;

/// `mach_port_kobject`'s type for a plain port, not a kernel object
/// (P8): a task, thread, host, memory entry or IOKit port has another.
pub(crate) const KOBJECT_NONE: u32 = 0xFFFF_FFFF;

/// Notification ids (P20, P21, P36): a dead-name request whose name went
/// away; send possible; port destroyed; no senders; a send-once right
/// destroyed unused; dead name.
pub(crate) const NOTIFY_PORT_DELETED: u32 = 65;
/// See [`NOTIFY_PORT_DELETED`].
pub(crate) const NOTIFY_SEND_POSSIBLE: u32 = 66;
/// See [`NOTIFY_PORT_DELETED`].
pub(crate) const NOTIFY_PORT_DESTROYED: u32 = 69;
/// See [`NOTIFY_PORT_DELETED`].
pub(crate) const NOTIFY_NO_SENDERS: u32 = 70;
/// See [`NOTIFY_PORT_DELETED`].
pub(crate) const NOTIFY_SEND_ONCE: u32 = 71;
/// See [`NOTIFY_PORT_DELETED`].
pub(crate) const NOTIFY_DEAD_NAME: u32 = 72;
/// A no-senders notification's size: the header, 8 bytes of NDR record
/// and the make-send count (R2a, R9).
pub(crate) const NO_SENDERS_SIZE: u32 = 36;

/// Port attribute flavor: the queue limit, one word, 0 to 1024 (P16).
pub(crate) const ATTR_LIMITS: i32 = 1;
/// Port attribute flavor: the receive status, ten words (P10).
pub(crate) const ATTR_RECEIVE_STATUS: i32 = 2;
/// Words of [`ATTR_RECEIVE_STATUS`] (P10).
pub(crate) const RECEIVE_STATUS_WORDS: usize = 10;
/// The receive status word counting the messages queued on the port
/// (P10).
pub(crate) const STATUS_MSGCOUNT: usize = 4;
/// The receive status word counting the port's send-once rights,
/// wherever they are (R8).
pub(crate) const STATUS_SEND_ONCE: usize = 5;
/// The receive status word that is nonzero when a port-destroyed
/// request is set (R8).
pub(crate) const STATUS_DESTROY_REQUEST: usize = 7;
/// The highest queue limit a port takes (P16).
pub(crate) const QLIMIT_MAX: u32 = 1024;

/// `task_info` flavor whose eight words equal the audit token in this
/// process's messages (P1).
pub(crate) const TASK_AUDIT_TOKEN: u32 = 15;
/// `task_get_special_port` / `posix_spawnattr_setspecialport_np` index
/// of the bootstrap port (P27).
pub(crate) const TASK_BOOTSTRAP_PORT: i32 = 4;

/// The null port name, and the name of a right that died in flight
/// (P22).
pub(crate) const NAME_NULL: u32 = 0;
/// See [`NAME_NULL`].
pub(crate) const NAME_DEAD: u32 = 0xFFFF_FFFF;

// ---- this transport's own message ids ----

/// A data record (sender to receiver).
pub(crate) const ID_DATA: u32 = 0x495A_4401;
/// A credit record (receiver to sender, on a send-once reply right).
pub(crate) const ID_CREDIT: u32 = 0x495A_4302;
/// A launched process's hello to its launcher, on its bootstrap port.
pub(crate) const ID_HELLO: u32 = 0x495A_4803;
/// The launcher's answer: the launch channel ends.
pub(crate) const ID_HELLO_REPLY: u32 = 0x495A_4804;

// ---- sizes ----

/// Header bytes (P11).
pub(crate) const HEADER_LEN: usize = 24;
/// The descriptor count word of a complex message (P11).
pub(crate) const COUNT_LEN: usize = 4;
/// A port descriptor (P11).
pub(crate) const PORT_DESC_LEN: usize = 12;
/// An out-of-line descriptor, memory or ports (P11, P31).
pub(crate) const OOL_DESC_LEN: usize = 16;
/// The trailer with three elements (P1).
pub(crate) const TRAILER_LEN: usize = 52;
/// The largest trailer any element count gives (P1); the receive buffer
/// always has room for it.
pub(crate) const TRAILER_MAX: usize = 68;
/// Rights one channel end puts on the wire: a sender end is its data
/// send right and its credit receive right; a receiver end, its data
/// receive right.
pub(crate) const MAX_SLOTS: usize = MAX_MESSAGE_HANDLES * 2;
/// The length word in front of a data record (a message body is padded
/// to 4 bytes, so the record's own end must be stated).
pub(crate) const FRAME_LEN: usize = 4;
/// The largest data message this transport sends.
pub(crate) const MAX_DATA_MESSAGE: usize =
    HEADER_LEN + COUNT_LEN + MAX_SLOTS * PORT_DESC_LEN + FRAME_LEN + MAX_DATA_RECORD + 3;
/// The receive buffer: the largest data message and the largest trailer.
/// A larger message is a peer's doing (P24 reports it; the transport
/// refuses it).
pub(crate) const RECEIVE_BUFFER: usize = MAX_DATA_MESSAGE + TRAILER_MAX;

/// Rounds `n` up to a multiple of 4, or `None` on overflow.
pub(crate) const fn round4(n: usize) -> Option<usize> {
    match n.checked_add(3) {
        Some(v) => Some(v & !3),
        None => None,
    }
}

// ---- reading ----

/// The six header words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Header {
    /// Dispositions and the complex bit.
    pub(crate) bits: u32,
    /// The message size the kernel reports, header to body end.
    pub(crate) size: u32,
    /// The reply right's name, in the receiver's space.
    pub(crate) remote: u32,
    /// The port it arrived on, in the receiver's space.
    pub(crate) local: u32,
    /// A voucher right's name, or 0.
    pub(crate) voucher: u32,
    /// The message id.
    pub(crate) id: u32,
}

impl Header {
    /// The reply right's disposition as received.
    pub(crate) const fn remote_disposition(&self) -> u8 {
        self.bits.to_le_bytes()[0] & 0x1f
    }

    /// The destination right's disposition as received.
    pub(crate) const fn local_disposition(&self) -> u8 {
        self.bits.to_le_bytes()[1] & 0x1f
    }

    /// Whether descriptors follow.
    pub(crate) const fn complex(&self) -> bool {
        self.bits & BITS_COMPLEX != 0
    }
}

/// One descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Desc {
    /// A port right.
    Port {
        /// Its name in the receiver's space.
        name: u32,
        /// Its disposition as received.
        disposition: u8,
    },
    /// Out-of-line memory now mapped in the receiver.
    Ool {
        /// Where.
        address: u64,
        /// How many bytes.
        size: u32,
    },
    /// An out-of-line array of port names now mapped in the receiver.
    OolPorts {
        /// Where.
        address: u64,
        /// How many names.
        count: u32,
        /// Their disposition.
        disposition: u8,
    },
}

/// Why a message's layout could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum WireError {
    /// Shorter than its header, or its size runs past the buffer.
    Short,
    /// A descriptor runs past the body, or the count does.
    Truncated,
    /// A descriptor of a type this transport does not know.
    UnknownDescriptor,
    /// No trailer with an audit token.
    NoTrailer,
}

/// A sender as the kernel attests it: the audit token's pid and pid
/// version (P1, P3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Token {
    /// The process ID.
    pub(crate) pid: u32,
    /// The pid version: changes on exec, never reused (P3).
    pub(crate) version: u32,
}

impl Token {
    /// The kernel's own messages carry an all-zero token (P4); no user
    /// process has pid 0.
    pub(crate) const fn is_kernel(self) -> bool {
        self.pid == 0 && self.version == 0
    }
}

/// The 32-bit little-endian word at `at`, if the buffer holds it.
pub(crate) fn word(buf: &[u8], at: usize) -> Option<u32> {
    let end = at.checked_add(4)?;
    let b = buf.get(at..end)?;
    let mut w = [0u8; 4];
    w.copy_from_slice(b);
    Some(u32::from_le_bytes(w))
}

/// The header, if the buffer holds one and the size it claims fits.
pub(crate) fn header(buf: &[u8]) -> Result<Header, WireError> {
    let w = |i: usize| word(buf, i).ok_or(WireError::Short);
    let h = Header {
        bits: w(0)?,
        size: w(4)?,
        remote: w(8)?,
        local: w(12)?,
        voucher: w(16)?,
        id: w(20)?,
    };
    let size = usize::try_from(h.size).map_err(|_e| WireError::Short)?;
    if size < HEADER_LEN || size > buf.len() {
        return Err(WireError::Short);
    }
    Ok(h)
}

/// The descriptor at `at`, and where the next starts. `end` is the
/// message size: no descriptor may run past it.
pub(crate) fn desc_at(buf: &[u8], at: usize, end: usize) -> Result<(Desc, usize), WireError> {
    let w = |i: usize| -> Result<u32, WireError> {
        at.checked_add(i)
            .and_then(|o| word(buf, o))
            .ok_or(WireError::Truncated)
    };
    let third = w(8)?.to_le_bytes();
    let ty = third[3];
    let len = match ty {
        DESC_PORT => PORT_DESC_LEN,
        DESC_OOL | DESC_OOL_PORTS => OOL_DESC_LEN,
        _ => return Err(WireError::UnknownDescriptor),
    };
    let next = at.checked_add(len).ok_or(WireError::Truncated)?;
    if next > end {
        return Err(WireError::Truncated);
    }
    let desc = match ty {
        DESC_PORT => Desc::Port {
            name: w(0)?,
            disposition: third[2],
        },
        DESC_OOL => Desc::Ool {
            address: u64::from(w(0)?) | (u64::from(w(4)?) << 32),
            size: w(12)?,
        },
        _ => Desc::OolPorts {
            address: u64::from(w(0)?) | (u64::from(w(4)?) << 32),
            count: w(12)?,
            disposition: third[2],
        },
    };
    Ok((desc, next))
}

/// Where a message's parts are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Layout {
    /// The header.
    pub(crate) header: Header,
    /// The descriptor count the message claims (0 if not complex).
    pub(crate) count: u32,
    /// Where the first descriptor starts.
    pub(crate) descs_at: usize,
    /// Where the body starts: after the last descriptor, if the walk
    /// read them all.
    pub(crate) body_at: usize,
    /// The message size (the body ends here).
    pub(crate) end: usize,
    /// The walk read every descriptor; if not, the error.
    pub(crate) walk: Result<(), WireError>,
    /// The sender as the kernel attests it.
    pub(crate) token: Result<Token, WireError>,
}

/// Reads a received message's layout. `Err` only when not even the
/// header can be read; otherwise every problem is in the result, so the
/// caller can still release every right the message carries.
pub(crate) fn layout(buf: &[u8]) -> Result<Layout, WireError> {
    let header = header(buf)?;
    let end = usize::try_from(header.size).map_err(|_e| WireError::Short)?;
    let token = trailer_token(buf, end);
    let (count, descs_at) = if header.complex() {
        match word(buf, HEADER_LEN) {
            Some(c) if HEADER_LEN + COUNT_LEN <= end => (c, HEADER_LEN + COUNT_LEN),
            _ => {
                return Ok(Layout {
                    header,
                    count: 0,
                    descs_at: end,
                    body_at: end,
                    end,
                    walk: Err(WireError::Truncated),
                    token,
                });
            }
        }
    } else {
        (0, HEADER_LEN)
    };
    // Each descriptor is at least 12 bytes, so the size bounds the walk.
    let mut at = descs_at;
    let mut walk = Ok(());
    let mut seen: u32 = 0;
    for _ in 0..max_descs(end) {
        if seen >= count {
            break;
        }
        match desc_at(buf, at, end) {
            Ok((_d, next)) => {
                at = next;
                seen = seen.saturating_add(1);
            }
            Err(e) => {
                walk = Err(e);
                break;
            }
        }
    }
    if walk.is_ok() && seen < count {
        walk = Err(WireError::Truncated);
    }
    Ok(Layout {
        header,
        count,
        descs_at,
        body_at: at,
        end,
        walk,
        token,
    })
}

/// The most descriptors a message of `size` bytes can hold.
pub(crate) const fn max_descs(size: usize) -> usize {
    match size.checked_sub(HEADER_LEN + COUNT_LEN) {
        Some(v) => v / PORT_DESC_LEN,
        None => 0,
    }
}

/// The audit token from the trailer after a message of `size` bytes.
pub(crate) fn trailer_token(buf: &[u8], size: usize) -> Result<Token, WireError> {
    let at = round4(size).ok_or(WireError::NoTrailer)?;
    let len =
        word(buf, at.checked_add(4).ok_or(WireError::NoTrailer)?).ok_or(WireError::NoTrailer)?;
    if usize::try_from(len).map_or(true, |l| l < TRAILER_LEN) {
        return Err(WireError::NoTrailer);
    }
    // Token words 5 (pid) and 7 (pid version); the token starts at
    // trailer word 5 (P1).
    let pid = at.checked_add(40).and_then(|o| word(buf, o));
    let version = at.checked_add(48).and_then(|o| word(buf, o));
    match (pid, version) {
        (Some(pid), Some(version)) => Ok(Token { pid, version }),
        _ => Err(WireError::NoTrailer),
    }
}

/// Every descriptor of a message, in order, as far as the walk read
/// them: what the caller must release, accepted or not.
pub(crate) fn descs<'a>(buf: &'a [u8], l: &Layout) -> DescIter<'a> {
    DescIter {
        buf,
        at: l.descs_at,
        end: l.end,
        left: l.count,
    }
}

/// See [`descs`].
pub(crate) struct DescIter<'a> {
    buf: &'a [u8],
    at: usize,
    end: usize,
    left: u32,
}

impl Iterator for DescIter<'_> {
    type Item = Desc;
    fn next(&mut self) -> Option<Desc> {
        if self.left == 0 {
            return None;
        }
        let (d, next) = desc_at(self.buf, self.at, self.end).ok()?;
        self.at = next;
        self.left = self.left.saturating_sub(1);
        Some(d)
    }
}

// ---- writing ----

/// A message to send: the header's rights and id, the port rights to
/// carry (name, disposition) and the body. On the wire the destination
/// is the header's third word and the reply its fourth; the kernel
/// swaps them on delivery, so the receiver reads the reply as `remote`
/// and its own port as `local` ([`Header`]).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Outgoing<'a> {
    /// Where to: a send or send-once right name.
    pub(crate) dest: u32,
    /// How the destination right is used: copy a send right, or move a
    /// send-once right.
    pub(crate) dest_disposition: u8,
    /// The reply right, or [`NAME_NULL`].
    pub(crate) reply: u32,
    /// Its disposition (0 with no reply).
    pub(crate) reply_disposition: u8,
    /// The message id.
    pub(crate) id: u32,
    /// Port rights to carry, in order.
    pub(crate) ports: &'a [(u32, u8)],
    /// The body.
    pub(crate) body: &'a [u8],
}

/// Writes `m` into `out`; returns the send size, or `None` if `out` is
/// too small or more than [`MAX_SLOTS`] rights are given.
pub(crate) fn encode(m: &Outgoing<'_>, out: &mut [u8]) -> Option<usize> {
    if m.ports.len() > MAX_SLOTS {
        return None;
    }
    let complex = !m.ports.is_empty();
    let descs_len = if complex {
        COUNT_LEN.checked_add(m.ports.len().checked_mul(PORT_DESC_LEN)?)?
    } else {
        0
    };
    let body_at = HEADER_LEN.checked_add(descs_len)?;
    let size = round4(body_at.checked_add(m.body.len())?)?;
    let dst = out.get_mut(..size)?;
    dst.fill(0);
    let bits = u32::from(m.dest_disposition)
        | (u32::from(m.reply_disposition) << 8)
        | if complex { BITS_COMPLEX } else { 0 };
    let size32 = u32::try_from(size).ok()?;
    let head = [bits, size32, m.dest, m.reply, 0, m.id];
    for (chunk, w) in dst.as_chunks_mut::<4>().0.iter_mut().zip(head) {
        *chunk = w.to_le_bytes();
    }
    if complex {
        let count = u32::try_from(m.ports.len()).ok()?;
        dst.get_mut(HEADER_LEN..HEADER_LEN + COUNT_LEN)?
            .copy_from_slice(&count.to_le_bytes());
        let mut at = HEADER_LEN + COUNT_LEN;
        for &(name, disposition) in m.ports {
            let next = at.checked_add(PORT_DESC_LEN)?;
            let d = dst.get_mut(at..next)?;
            d.get_mut(0..4)?.copy_from_slice(&name.to_le_bytes());
            let third = (u32::from(disposition) << 16) | (u32::from(DESC_PORT) << 24);
            d.get_mut(8..12)?.copy_from_slice(&third.to_le_bytes());
            at = next;
        }
    }
    let body_end = body_at.checked_add(m.body.len())?;
    dst.get_mut(body_at..body_end)?.copy_from_slice(m.body);
    Some(size)
}

/// A data record framed for a message body: its length, then the
/// record. The kernel pads a body to 4 bytes (P11), so without the length
/// the receiver could not tell the record's end; `None` past `u32`.
pub(crate) fn frame(record: &[u8]) -> Option<Vec<u8>> {
    let len = u32::try_from(record.len()).ok()?;
    let mut out = Vec::with_capacity(record.len().checked_add(FRAME_LEN)?);
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(record);
    Some(out)
}

/// The record in a framed body: the stated length must fit, and what
/// follows it can only be the kernel's padding (fewer than 4 bytes).
pub(crate) fn unframe(body: &[u8]) -> Option<&[u8]> {
    let len = usize::try_from(word(body, 0)?).ok()?;
    let rest = body.get(FRAME_LEN..)?;
    let pad = rest.len().checked_sub(len)?;
    if pad >= 4 {
        return None;
    }
    rest.get(..len)
}

/// VER-1 for the parser: no panic and no read outside the buffer for any
/// bytes, a walk that stops, and every descriptor the iterator yields
/// lies inside the message.
#[cfg(kani)]
mod proofs {
    use super::*;

    /// Any 80 bytes: room for a header and a trailer, or for a header, a
    /// count and up to four port descriptors (or fewer larger ones).
    #[kani::proof]
    #[kani::unwind(8)]
    fn layout_reads_any_bytes() {
        let buf: [u8; 80] = kani::any();
        if let Ok(l) = layout(&buf) {
            assert!(l.end <= buf.len());
            assert!(l.body_at <= l.end || l.walk.is_err());
            let mut n = 0u32;
            for _d in descs(&buf, &l) {
                n = n.saturating_add(1);
            }
            assert!(n <= l.count);
            kani::cover!(l.walk.is_ok() && l.count == 2, "two descriptors read");
            kani::cover!(l.token.is_ok(), "a trailer read");
        }
    }

    /// `unframe` never reads outside the body, and what it returns ends
    /// within the body's last 3 bytes; what `frame` writes it reads back.
    #[kani::proof]
    #[kani::unwind(12)]
    fn unframe_reads_any_bytes_and_inverts_frame() {
        let body: [u8; 10] = kani::any();
        let n: usize = kani::any_where(|&n: &usize| n <= 10);
        if let Some(r) = unframe(&body[..n]) {
            assert!(r.len() + FRAME_LEN <= n);
            assert!(n - r.len() - FRAME_LEN < 4);
        }
        let rec: [u8; 5] = kani::any();
        let m: usize = kani::any_where(|&m: &usize| m <= 5);
        if let Some(f) = frame(&rec[..m]) {
            let mut padded = [0u8; 12];
            padded[..f.len()].copy_from_slice(&f);
            let p: usize = kani::any_where(|&p: &usize| p < 4 && f.len() + p <= 12);
            let back = unframe(&padded[..f.len() + p]);
            assert!(back.is_some());
            if let Some(b) = back {
                assert!(b.len() == m);
            }
        }
    }

    /// What `encode` writes, `layout` reads back: the header, every
    /// carried right in order, and the body.
    #[kani::proof]
    #[kani::unwind(8)]
    fn encode_then_layout() {
        let names: [u32; 2] = kani::any();
        let disps: [u8; 2] = [MOVE_SEND, MOVE_RECEIVE];
        let n: usize = kani::any_where(|&n: &usize| n <= 2);
        let ports = [(names[0], disps[0]), (names[1], disps[1])];
        let body: [u8; 5] = kani::any();
        let m = Outgoing {
            dest: kani::any(),
            dest_disposition: COPY_SEND,
            reply: kani::any(),
            reply_disposition: MAKE_SEND_ONCE,
            id: ID_DATA,
            ports: &ports[..n],
            body: &body,
        };
        let mut out = [0u8; 64];
        let size = encode(&m, &mut out);
        assert!(size.is_some());
        let size = size.unwrap_or(0);
        let l = layout(&out[..size]);
        assert!(l.is_ok());
        if let Ok(l) = l {
            assert!(l.walk.is_ok());
            assert!(l.header.id == ID_DATA);
            assert!(l.header.local == m.reply);
            assert!(usize::try_from(l.count).ok() == Some(n));
            let mut i = 0;
            for d in descs(&out[..size], &l) {
                assert!(
                    d == Desc::Port {
                        name: ports[i].0,
                        disposition: ports[i].1
                    }
                );
                i += 1;
            }
            assert!(i == n);
            for k in 0..5 {
                assert!(out[l.body_at + k] == body[k]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trailer(buf: &mut Vec<u8>, pid: u32, version: u32) {
        while !buf.len().is_multiple_of(4) {
            buf.push(0);
        }
        let mut t = vec![0u32; 13];
        t[1] = 52;
        t[10] = pid;
        t[12] = version;
        for w in t {
            buf.extend_from_slice(&w.to_le_bytes());
        }
    }

    #[test]
    fn a_data_message_round_trips_with_its_token() {
        let ports = [(0x1103, MOVE_SEND), (0x1207, MOVE_RECEIVE)];
        let m = Outgoing {
            dest: 0x2203,
            dest_disposition: COPY_SEND,
            reply: 0x3303,
            reply_disposition: MAKE_SEND_ONCE,
            id: ID_DATA,
            ports: &ports,
            body: b"hello",
        };
        let mut out = vec![0u8; 256];
        let size = encode(&m, &mut out).unwrap();
        assert_eq!(size, 24 + 4 + 24 + 8);
        out.truncate(size);
        trailer(&mut out, 4242, 77);
        let l = layout(&out).unwrap();
        assert!(l.header.complex());
        assert_eq!(l.header.local_disposition(), MAKE_SEND_ONCE);
        assert_eq!(l.header.remote_disposition(), COPY_SEND);
        assert_eq!(l.walk, Ok(()));
        assert_eq!(&out[l.body_at..l.body_at + 5], b"hello");
        assert_eq!(
            descs(&out, &l).collect::<Vec<_>>(),
            vec![
                Desc::Port {
                    name: 0x1103,
                    disposition: MOVE_SEND
                },
                Desc::Port {
                    name: 0x1207,
                    disposition: MOVE_RECEIVE
                }
            ]
        );
        assert_eq!(
            l.token,
            Ok(Token {
                pid: 4242,
                version: 77
            })
        );
    }

    #[test]
    fn lying_counts_and_sizes_are_caught() {
        let mut out = vec![0u8; 64];
        let m = Outgoing {
            dest: 1,
            dest_disposition: COPY_SEND,
            reply: 0,
            reply_disposition: 0,
            id: ID_DATA,
            ports: &[(5, MOVE_SEND)],
            body: b"",
        };
        let size = encode(&m, &mut out).unwrap();
        out.truncate(size);
        // A count of 1000 with one descriptor's room.
        out[24..28].copy_from_slice(&1000u32.to_le_bytes());
        let l = layout(&out).unwrap();
        assert_eq!(l.walk, Err(WireError::Truncated));
        assert_eq!(descs(&out, &l).count(), 1);
        // A size past the buffer.
        out[4..8].copy_from_slice(&4096u32.to_le_bytes());
        assert_eq!(layout(&out), Err(WireError::Short));
        // An unknown descriptor type.
        let size = encode(&m, &mut out).unwrap();
        out.truncate(size);
        out[24 + 4 + 11] = 9;
        assert_eq!(
            layout(&out).unwrap().walk,
            Err(WireError::UnknownDescriptor)
        );
        // No trailer.
        assert_eq!(layout(&out).unwrap().token, Err(WireError::NoTrailer));
    }

    #[test]
    fn framing_survives_padding_and_refuses_lies() {
        let f = frame(b"abcde").unwrap();
        let mut padded = f.clone();
        padded.extend_from_slice(&[0, 0, 0]);
        assert_eq!(unframe(&padded), Some(&b"abcde"[..]));
        assert_eq!(unframe(&f), Some(&b"abcde"[..]));
        padded.push(0);
        assert_eq!(unframe(&padded), None);
        let mut lie = f.clone();
        lie[0] = 200;
        assert_eq!(unframe(&lie), None);
        assert_eq!(unframe(&[1, 0]), None);
    }

    #[test]
    fn ool_descriptors_are_walked() {
        let mut out = vec![0u8; 24 + 4 + 16 + 16];
        out[0..4].copy_from_slice(&BITS_COMPLEX.to_le_bytes());
        let size = u32::try_from(out.len()).unwrap();
        out[4..8].copy_from_slice(&size.to_le_bytes());
        out[24..28].copy_from_slice(&2u32.to_le_bytes());
        // OOL memory at 0x1_0000_2000, 64 bytes.
        out[28..32].copy_from_slice(&0x2000u32.to_le_bytes());
        out[32..36].copy_from_slice(&1u32.to_le_bytes());
        out[36..40].copy_from_slice(&(u32::from(DESC_OOL) << 24).to_le_bytes());
        out[40..44].copy_from_slice(&64u32.to_le_bytes());
        // OOL ports: 3 send rights.
        out[52..56].copy_from_slice(
            &((u32::from(DESC_OOL_PORTS) << 24) | (u32::from(PORT_SEND) << 16)).to_le_bytes(),
        );
        out[56..60].copy_from_slice(&3u32.to_le_bytes());
        let l = layout(&out).unwrap();
        assert_eq!(l.walk, Ok(()));
        assert_eq!(
            descs(&out, &l).collect::<Vec<_>>(),
            vec![
                Desc::Ool {
                    address: 0x1_0000_2000,
                    size: 64
                },
                Desc::OolPorts {
                    address: 0,
                    count: 3,
                    disposition: PORT_SEND
                }
            ]
        );
    }
}
