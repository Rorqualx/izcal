//! The record formats of the Unix transport (IPC-R6: one fixed
//! little-endian layout, decoded without allocating).
//!
//! A channel is a `SOCK_SEQPACKET` socket pair, so each `sendmsg(2)` is
//! one record and `recvmsg(2)` returns it whole (unix(7)). Two record
//! kinds exist, one per direction, each with its own magic so a swapped
//! end is caught on the first record:
//!
//! **Data record** (sender to receiver), [`DATA_HEADER_LEN`] bytes, then
//! `count` descriptors of [`DESC_LEN`] bytes, then the payload:
//!
//! | Offset | Size | Field |
//! |---|---|---|
//! | 0 | 4 | [`DATA_MAGIC`] |
//! | 4 | 1 | [`VERSION`] |
//! | 5 | 1 | handle count, at most [`MAX_MESSAGE_HANDLES`] |
//! | 6 | 2 | reserved, 0 |
//! | 8 | 8 | `seq`: the sender's sent count before this send |
//!
//! A descriptor: wire kind (1 byte, [`WireKind`]), 3 reserved bytes (0),
//! `capacity` (u32), `sent` (u64), `delivered` (u64). For a sender these
//! are its holder's sent and delivered-known counts; for a receiver its
//! delivered count and capacity, with `sent` = its received count. The
//! descriptor table is in the same order as the record's `SCM_RIGHTS`
//! descriptors.
//!
//! **Credit record** (receiver to sender), exactly [`CREDIT_LEN`] bytes:
//! [`CREDIT_MAGIC`], [`VERSION`], 3 reserved bytes (0), `capacity` (u32),
//! cumulative `delivered` (u64).
//!
//! Decoding is strict: every reserved byte must be 0, so a record has
//! exactly one encoding and decode then encode gives the input back.

use hg_substrate::ipc::{MAX_MESSAGE_BYTES, MAX_MESSAGE_HANDLES};

/// The first four bytes of a data record: `IZDT` (little-endian u32).
pub(crate) const DATA_MAGIC: u32 = u32::from_le_bytes(*b"IZDT");

/// The first four bytes of a credit record: `IZCR`. Distinct from
/// [`DATA_MAGIC`], so a record read from the wrong end is refused.
pub(crate) const CREDIT_MAGIC: u32 = u32::from_le_bytes(*b"IZCR");

/// The only record version this transport speaks.
pub(crate) const VERSION: u8 = 1;

/// Bytes in a data record's fixed header.
pub(crate) const DATA_HEADER_LEN: usize = 16;

/// Bytes in one handle descriptor.
pub(crate) const DESC_LEN: usize = 24;

/// The longest data record: the header, 16 descriptors and 64 KiB.
pub(crate) const MAX_DATA_RECORD: usize =
    DATA_HEADER_LEN + MAX_MESSAGE_HANDLES * DESC_LEN + MAX_MESSAGE_BYTES;

/// Bytes in a credit record (it has no variable part).
pub(crate) const CREDIT_LEN: usize = 20;

const _: () = assert!(MAX_DATA_RECORD == 16 + 16 * 24 + 65_536);
const _: () = assert!(MAX_MESSAGE_HANDLES <= 255, "the count is one byte");

/// What a descriptor says its handle is. The transport checks the claim
/// against the kernel's facts before believing it (see `accept`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum WireKind {
    /// A free channel's sending end (wire value 1).
    Sender,
    /// A bound channel's sending end (wire value 2).
    BoundSender,
    /// A channel's receiving end (wire value 3).
    Receiver,
}

impl WireKind {
    /// The wire byte.
    pub(crate) const fn to_byte(self) -> u8 {
        match self {
            Self::Sender => 1,
            Self::BoundSender => 2,
            Self::Receiver => 3,
        }
    }

    /// The kind for a wire byte; 0 and every value above 3 are invalid.
    pub(crate) const fn from_byte(b: u8) -> Option<Self> {
        match b {
            1 => Some(Self::Sender),
            2 => Some(Self::BoundSender),
            3 => Some(Self::Receiver),
            _ => None,
        }
    }
}

/// One handle descriptor, as carried in a data record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct HandleDesc {
    /// The claimed kind.
    pub(crate) kind: WireKind,
    /// The channel's capacity.
    pub(crate) capacity: u32,
    /// Sender: messages sent so far. Receiver: records received so far.
    pub(crate) sent: u64,
    /// Sender: deliveries known from credits. Receiver: deliveries made.
    pub(crate) delivered: u64,
}

impl HandleDesc {
    /// A filler for the unused slots of [`DataView::descs`].
    pub(crate) const EMPTY: Self = Self {
        kind: WireKind::Sender,
        capacity: 0,
        sent: 0,
        delivered: 0,
    };
}

/// A decoded data record, borrowing its payload from the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DataView<'a> {
    /// The sender's sent count before this send.
    pub(crate) seq: u64,
    /// The descriptors; slots at and past `count` are filler.
    descs: [HandleDesc; MAX_MESSAGE_HANDLES],
    /// How many descriptors there are, at most [`MAX_MESSAGE_HANDLES`].
    count: usize,
    /// The message bytes, at most [`MAX_MESSAGE_BYTES`].
    pub(crate) payload: &'a [u8],
}

impl<'a> DataView<'a> {
    /// A view built directly, for the decision's harness: any result
    /// [`decode_data`] could give, without bytes to decode.
    #[cfg(kani)]
    pub(crate) fn for_proofs(
        seq: u64,
        descs: [HandleDesc; MAX_MESSAGE_HANDLES],
        count: usize,
        payload: &'a [u8],
    ) -> Self {
        Self {
            seq,
            descs,
            count: count.min(MAX_MESSAGE_HANDLES),
            payload,
        }
    }

    /// The descriptors, in `SCM_RIGHTS` order.
    pub(crate) fn descs(&self) -> &[HandleDesc] {
        self.descs.get(..self.count).unwrap_or(&[])
    }
}

/// A decoded credit record: the receiver has started `delivered`
/// deliveries in all, out of a channel of `capacity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Credit {
    /// The channel's capacity, repeated so a credit for another channel
    /// is caught.
    pub(crate) capacity: u32,
    /// Cumulative deliveries started.
    pub(crate) delivered: u64,
}

/// Why a record was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CodecError {
    /// Fewer bytes than the fixed parts need.
    Short,
    /// The magic is not this record kind's (garbage, or a swapped end).
    BadMagic,
    /// A version this transport does not speak.
    BadVersion,
    /// More than [`MAX_MESSAGE_HANDLES`] descriptors.
    TooManyHandles,
    /// A reserved byte is not 0.
    Reserved,
    /// A descriptor's kind byte is not a [`WireKind`].
    BadKind,
    /// A payload over [`MAX_MESSAGE_BYTES`].
    TooLarge,
    /// A credit record longer than [`CREDIT_LEN`].
    Trailing,
}

impl core::fmt::Display for CodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Short => "record too short",
            Self::BadMagic => "wrong record magic",
            Self::BadVersion => "unknown record version",
            Self::TooManyHandles => "too many handle descriptors",
            Self::Reserved => "nonzero reserved bytes",
            Self::BadKind => "invalid handle kind",
            Self::TooLarge => "payload over the message limit",
            Self::Trailing => "bytes after the credit record",
        })
    }
}

impl std::error::Error for CodecError {}

/// Takes the next `N` bytes of `input`, or fails with [`CodecError::Short`].
fn take<const N: usize>(input: &mut &[u8]) -> Result<[u8; N], CodecError> {
    let (head, rest) = input.split_first_chunk::<N>().ok_or(CodecError::Short)?;
    *input = rest;
    Ok(*head)
}

fn take_u8(input: &mut &[u8]) -> Result<u8, CodecError> {
    take::<1>(input).map(|[b]| b)
}

fn take_u32(input: &mut &[u8]) -> Result<u32, CodecError> {
    take::<4>(input).map(u32::from_le_bytes)
}

fn take_u64(input: &mut &[u8]) -> Result<u64, CodecError> {
    take::<8>(input).map(u64::from_le_bytes)
}

fn reserved<const N: usize>(input: &mut &[u8]) -> Result<(), CodecError> {
    if take::<N>(input)? == [0; N] {
        Ok(())
    } else {
        Err(CodecError::Reserved)
    }
}

fn desc_bytes(d: &HandleDesc) -> [u8; DESC_LEN] {
    let mut out = [0u8; DESC_LEN];
    let (kind, rest) = out.split_at_mut(4);
    if let Some(k) = kind.first_mut() {
        *k = d.kind.to_byte();
    }
    let (cap, rest) = rest.split_at_mut(4);
    cap.copy_from_slice(&d.capacity.to_le_bytes());
    let (sent, delivered) = rest.split_at_mut(8);
    sent.copy_from_slice(&d.sent.to_le_bytes());
    delivered.copy_from_slice(&d.delivered.to_le_bytes());
    out
}

fn decode_desc(input: &mut &[u8]) -> Result<HandleDesc, CodecError> {
    let kind = WireKind::from_byte(take_u8(input)?).ok_or(CodecError::BadKind)?;
    reserved::<3>(input)?;
    Ok(HandleDesc {
        kind,
        capacity: take_u32(input)?,
        sent: take_u64(input)?,
        delivered: take_u64(input)?,
    })
}

/// Encodes a data record.
///
/// # Errors
/// [`CodecError::TooManyHandles`] for more than [`MAX_MESSAGE_HANDLES`]
/// descriptors, [`CodecError::TooLarge`] for a payload over
/// [`MAX_MESSAGE_BYTES`]; nothing is encoded.
pub(crate) fn encode_data(
    seq: u64,
    descs: &[HandleDesc],
    payload: &[u8],
) -> Result<Vec<u8>, CodecError> {
    let len = data_len(descs, payload)?;
    let mut out = vec![0u8; len];
    encode_into(seq, descs, payload, &mut out)?;
    Ok(out)
}

/// The length of the data record for `descs` and `payload`, or why it
/// cannot be encoded.
fn data_len(descs: &[HandleDesc], payload: &[u8]) -> Result<usize, CodecError> {
    if descs.len() > MAX_MESSAGE_HANDLES {
        return Err(CodecError::TooManyHandles);
    }
    if payload.len() > MAX_MESSAGE_BYTES {
        return Err(CodecError::TooLarge);
    }
    // Both bounds are checked above, so this is at most MAX_DATA_RECORD.
    descs
        .len()
        .checked_mul(DESC_LEN)
        .and_then(|d| d.checked_add(DATA_HEADER_LEN))
        .and_then(|d| d.checked_add(payload.len()))
        .ok_or(CodecError::TooLarge)
}

/// [`encode_data`] into the start of `out`, without allocating; gives the
/// record's length. `out` shorter than the record is
/// [`CodecError::TooLarge`].
pub(crate) fn encode_into(
    seq: u64,
    descs: &[HandleDesc],
    payload: &[u8],
    out: &mut [u8],
) -> Result<usize, CodecError> {
    let len = data_len(descs, payload)?;
    let count = u8::try_from(descs.len()).map_err(|_e| CodecError::TooManyHandles)?;
    let rec = out.get_mut(..len).ok_or(CodecError::TooLarge)?;
    let (header, rest) = rec.split_at_mut(DATA_HEADER_LEN);
    let (magic, header) = header.split_at_mut(4);
    magic.copy_from_slice(&DATA_MAGIC.to_le_bytes());
    let (fixed, seq_bytes) = header.split_at_mut(4);
    fixed.copy_from_slice(&[VERSION, count, 0, 0]);
    seq_bytes.copy_from_slice(&seq.to_le_bytes());
    let (table, body) = rest.split_at_mut(descs.len().saturating_mul(DESC_LEN));
    let (slots, _none) = table.as_chunks_mut::<DESC_LEN>();
    for (slot, d) in slots.iter_mut().zip(descs) {
        *slot = desc_bytes(d);
    }
    body.copy_from_slice(payload);
    Ok(len)
}

/// Decodes a data record: the whole of one `recvmsg(2)` read.
///
/// # Errors
/// Every [`CodecError`] but [`CodecError::Trailing`]: everything after
/// the descriptors is payload.
pub(crate) fn decode_data(input: &[u8]) -> Result<DataView<'_>, CodecError> {
    let mut r = input;
    if take_u32(&mut r)? != DATA_MAGIC {
        return Err(CodecError::BadMagic);
    }
    if take_u8(&mut r)? != VERSION {
        return Err(CodecError::BadVersion);
    }
    let count = usize::from(take_u8(&mut r)?);
    if count > MAX_MESSAGE_HANDLES {
        return Err(CodecError::TooManyHandles);
    }
    reserved::<2>(&mut r)?;
    let seq = take_u64(&mut r)?;
    let mut descs = [HandleDesc::EMPTY; MAX_MESSAGE_HANDLES];
    for (i, slot) in descs.iter_mut().enumerate() {
        if i >= count {
            break;
        }
        *slot = decode_desc(&mut r)?;
    }
    if r.len() > MAX_MESSAGE_BYTES {
        return Err(CodecError::TooLarge);
    }
    Ok(DataView {
        seq,
        descs,
        count,
        payload: r,
    })
}

/// Encodes a credit record.
pub(crate) fn encode_credit(c: Credit) -> [u8; CREDIT_LEN] {
    let mut out = [0u8; CREDIT_LEN];
    let (magic, rest) = out.split_at_mut(4);
    magic.copy_from_slice(&CREDIT_MAGIC.to_le_bytes());
    let (version, rest) = rest.split_at_mut(4);
    if let Some(v) = version.first_mut() {
        *v = VERSION;
    }
    let (cap, delivered) = rest.split_at_mut(4);
    cap.copy_from_slice(&c.capacity.to_le_bytes());
    delivered.copy_from_slice(&c.delivered.to_le_bytes());
    out
}

/// Decodes a credit record: exactly [`CREDIT_LEN`] bytes.
///
/// # Errors
/// [`CodecError::Short`], [`CodecError::BadMagic`],
/// [`CodecError::BadVersion`], [`CodecError::Reserved`],
/// [`CodecError::Trailing`].
pub(crate) fn decode_credit(input: &[u8]) -> Result<Credit, CodecError> {
    let mut r = input;
    if take_u32(&mut r)? != CREDIT_MAGIC {
        return Err(CodecError::BadMagic);
    }
    if take_u8(&mut r)? != VERSION {
        return Err(CodecError::BadVersion);
    }
    reserved::<3>(&mut r)?;
    let capacity = take_u32(&mut r)?;
    let delivered = take_u64(&mut r)?;
    if !r.is_empty() {
        return Err(CodecError::Trailing);
    }
    Ok(Credit {
        capacity,
        delivered,
    })
}

/// VER-1 for the record formats (IPC-R6). Kani's bounds are stated per
/// harness; the payload bound is reduced to keep the solver fast, which
/// loses nothing: the payload is copied as one slice, never inspected.
#[cfg(kani)]
pub(crate) mod proofs {
    use super::*;

    /// Decoding any `K` bytes never panics; an accepted record has as
    /// many descriptors as its count byte, and re-encodes to the same
    /// bytes. The length is fixed per harness and the bytes symbolic (the
    /// D1 lesson); the lengths below sit on each boundary of the format:
    /// empty, inside the header, the header alone, one descriptor short
    /// and exact, and one descriptor with an 8-byte payload (which also
    /// reads as no descriptor and a 32-byte payload, or as two
    /// descriptors cut short).
    fn data_round_trip<const K: usize>() {
        let input: [u8; K] = kani::any();
        let Ok(v) = decode_data(&input) else {
            return;
        };
        // An accepted record is exactly the bytes encode_data writes for
        // it, checked field by field against the input (re-encoding
        // would build a Vec of symbolic length).
        let count = v.descs().len();
        assert!(count <= MAX_MESSAGE_HANDLES);
        assert!(u32::from_le_bytes([input[0], input[1], input[2], input[3]]) == DATA_MAGIC);
        assert!(input[4] == VERSION);
        assert!(usize::from(input[5]) == count);
        assert!(input[6] == 0 && input[7] == 0);
        let mut seq = [0u8; 8];
        seq.copy_from_slice(&input[8..16]);
        assert!(v.seq == u64::from_le_bytes(seq));
        for i in 0..MAX_MESSAGE_HANDLES {
            if i < count {
                let want = desc_bytes(&v.descs()[i]);
                for j in 0..DESC_LEN {
                    assert!(input[DATA_HEADER_LEN + i * DESC_LEN + j] == want[j]);
                }
            }
        }
        let body = DATA_HEADER_LEN + count * DESC_LEN;
        assert!(v.payload.len() + body == K);
        assert!(core::ptr::eq(v.payload.as_ptr(), input[body..].as_ptr()));
    }

    #[kani::proof]
    #[kani::unwind(18)]
    fn data_decode_round_trips_0() {
        data_round_trip::<0>();
    }

    #[kani::proof]
    #[kani::unwind(18)]
    fn data_decode_round_trips_3() {
        data_round_trip::<3>();
    }

    #[kani::proof]
    #[kani::unwind(18)]
    fn data_decode_round_trips_15() {
        data_round_trip::<15>();
    }

    #[kani::proof]
    #[kani::unwind(18)]
    fn data_decode_round_trips_16() {
        data_round_trip::<16>();
    }

    #[kani::proof]
    #[kani::unwind(41)]
    fn data_decode_round_trips_39() {
        data_round_trip::<{ DATA_HEADER_LEN + DESC_LEN - 1 }>();
    }

    #[kani::proof]
    #[kani::unwind(42)]
    fn data_decode_round_trips_40() {
        data_round_trip::<{ DATA_HEADER_LEN + DESC_LEN }>();
    }

    #[kani::proof]
    #[kani::unwind(50)]
    fn data_decode_round_trips_48() {
        data_round_trip::<{ DATA_HEADER_LEN + DESC_LEN + 8 }>();
    }

    /// A record with a handle is accepted at all (so the harnesses above
    /// are not vacuous on the descriptor path).
    #[kani::proof]
    #[kani::unwind(42)]
    fn a_record_with_a_handle_decodes() {
        let input: [u8; DATA_HEADER_LEN + DESC_LEN] = kani::any();
        kani::cover!(
            decode_data(&input).is_ok_and(|v| v.descs().len() == 1),
            "a record with a handle is accepted"
        );
    }

    /// Encoding any record of `N` descriptors and `P` payload bytes, all
    /// symbolic, then decoding it, gives the same record. The sizes are
    /// fixed per harness (the D1 lesson: concrete sizes, symbolic
    /// contents); the harnesses below cover 0 to 2 descriptors with and
    /// without a payload.
    fn encode_round_trip<const N: usize, const P: usize>() {
        let descs: [HandleDesc; N] = core::array::from_fn(|_| any_desc());
        let payload: [u8; P] = kani::any();
        let seq: u64 = kani::any();
        // Room for the largest record these harnesses make.
        let mut out = [0u8; DATA_HEADER_LEN + 2 * DESC_LEN + 4];
        let Ok(len) = encode_into(seq, &descs, &payload, &mut out) else {
            panic!("a record within the limits encodes");
        };
        assert!(len == DATA_HEADER_LEN + N * DESC_LEN + P);
        let Ok(v) = decode_data(&out[..len]) else {
            panic!("an encoded record decodes");
        };
        assert!(v.seq == seq);
        assert!(v.descs().len() == N);
        for i in 0..N {
            assert!(v.descs()[i] == descs[i]);
        }
        assert!(v.payload.len() == P);
        for i in 0..P {
            assert!(v.payload[i] == payload[i]);
        }
    }

    #[kani::proof]
    #[kani::unwind(70)]
    fn data_encode_round_trips_0_0() {
        encode_round_trip::<0, 0>();
    }

    #[kani::proof]
    #[kani::unwind(70)]
    fn data_encode_round_trips_0_4() {
        encode_round_trip::<0, 4>();
    }

    #[kani::proof]
    #[kani::unwind(70)]
    fn data_encode_round_trips_1_0() {
        encode_round_trip::<1, 0>();
    }

    #[kani::proof]
    #[kani::unwind(70)]
    fn data_encode_round_trips_1_4() {
        encode_round_trip::<1, 4>();
    }

    #[kani::proof]
    #[kani::unwind(70)]
    fn data_encode_round_trips_2_0() {
        encode_round_trip::<2, 0>();
    }

    #[kani::proof]
    #[kani::unwind(70)]
    fn data_encode_round_trips_2_4() {
        encode_round_trip::<2, 4>();
    }

    /// Bounds: 0 to 21 bytes (one past a credit record).
    #[kani::proof]
    #[kani::unwind(22)]
    fn credit_round_trips() {
        let bytes: [u8; CREDIT_LEN + 1] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= CREDIT_LEN + 1);
        let Some(input) = bytes.get(..len) else {
            return;
        };
        let decoded = decode_credit(input);
        kani::cover!(decoded.is_ok(), "some credit is accepted");
        if let Ok(c) = decoded {
            assert!(encode_credit(c).as_slice() == input);
        }
        let c = Credit {
            capacity: kani::any(),
            delivered: kani::any(),
        };
        assert!(decode_credit(&encode_credit(c)) == Ok(c));
        // A credit is never mistaken for a data record, or the reverse.
        assert!(decode_data(&encode_credit(c)) == Err(CodecError::BadMagic));
    }

    /// Any descriptor.
    pub(crate) fn any_desc() -> HandleDesc {
        let kind = match kani::any::<u8>() % 3 {
            0 => WireKind::Sender,
            1 => WireKind::BoundSender,
            _ => WireKind::Receiver,
        };
        HandleDesc {
            kind,
            capacity: kani::any(),
            sent: kani::any(),
            delivered: kani::any(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desc(kind: WireKind, n: u64) -> HandleDesc {
        HandleDesc {
            kind,
            capacity: 64,
            sent: n,
            delivered: n / 2,
        }
    }

    #[test]
    fn data_round_trips_with_the_documented_layout() {
        let d = [desc(WireKind::Sender, 9), desc(WireKind::Receiver, 4)];
        let bytes = encode_data(7, &d, b"hi").unwrap();
        assert_eq!(bytes.len(), DATA_HEADER_LEN + 2 * DESC_LEN + 2);
        assert_eq!(&bytes[..8], &[b'I', b'Z', b'D', b'T', 1, 2, 0, 0]);
        assert_eq!(&bytes[8..16], &7u64.to_le_bytes());
        assert_eq!(&bytes[16..24], &[1, 0, 0, 0, 64, 0, 0, 0]);
        assert_eq!(&bytes[24..32], &9u64.to_le_bytes());
        assert_eq!(&bytes[32..40], &4u64.to_le_bytes());
        assert_eq!(bytes[40], 3);
        let v = decode_data(&bytes).unwrap();
        assert_eq!((v.seq, v.descs(), v.payload), (7, &d[..], &b"hi"[..]));
    }

    #[test]
    fn data_limits_hold_both_ways() {
        let full = vec![0u8; MAX_MESSAGE_BYTES];
        let descs = [desc(WireKind::BoundSender, 1); MAX_MESSAGE_HANDLES];
        let max = encode_data(u64::MAX, &descs, &full).unwrap();
        assert_eq!(max.len(), MAX_DATA_RECORD);
        assert_eq!(decode_data(&max).unwrap().descs().len(), 16);

        let mut over = max.clone();
        over.push(0);
        assert_eq!(decode_data(&over), Err(CodecError::TooLarge));
        let big = vec![0u8; MAX_MESSAGE_BYTES + 1];
        assert_eq!(encode_data(0, &[], &big), Err(CodecError::TooLarge));
        let seventeen = [desc(WireKind::Sender, 0); 17];
        assert_eq!(
            encode_data(0, &seventeen, &[]),
            Err(CodecError::TooManyHandles)
        );
    }

    #[test]
    fn data_refusals() {
        let good = encode_data(1, &[desc(WireKind::Sender, 3)], b"x").unwrap();
        let with = |at: usize, b: u8| {
            let mut v = good.clone();
            v[at] = b;
            decode_data(&v).map(|_view| ())
        };
        assert_eq!(with(0, b'X'), Err(CodecError::BadMagic));
        assert_eq!(with(4, 2), Err(CodecError::BadVersion));
        assert_eq!(with(5, 17), Err(CodecError::TooManyHandles));
        // A count of 2 with one descriptor's worth of bytes left: the
        // payload byte is read as the second kind.
        assert_eq!(with(5, 2), Err(CodecError::BadKind));
        let mut no_payload = encode_data(1, &[desc(WireKind::Sender, 3)], &[]).unwrap();
        no_payload[5] = 2;
        assert_eq!(
            decode_data(&no_payload).map(|_v| ()),
            Err(CodecError::Short)
        );
        assert_eq!(with(6, 1), Err(CodecError::Reserved));
        assert_eq!(with(7, 1), Err(CodecError::Reserved));
        assert_eq!(with(16, 0), Err(CodecError::BadKind));
        assert_eq!(with(16, 4), Err(CodecError::BadKind));
        assert_eq!(with(17, 1), Err(CodecError::Reserved));
        assert_eq!(with(19, 1), Err(CodecError::Reserved));
        for n in 0..DATA_HEADER_LEN + DESC_LEN {
            assert_eq!(
                decode_data(&good[..n]).map(|_v| ()),
                Err(CodecError::Short),
                "{n}"
            );
        }
        // The swapped end: a credit read where data is expected.
        let credit = encode_credit(Credit {
            capacity: 1,
            delivered: 0,
        });
        assert_eq!(decode_data(&credit), Err(CodecError::BadMagic));
        // An empty payload is a valid record.
        let empty = encode_data(0, &[], &[]).unwrap();
        assert!(decode_data(&empty).unwrap().payload.is_empty());
    }

    #[test]
    fn credit_round_trips_and_refusals() {
        let c = Credit {
            capacity: 1024,
            delivered: 1 << 40,
        };
        let bytes = encode_credit(c);
        assert_eq!(&bytes[..8], &[b'I', b'Z', b'C', b'R', 1, 0, 0, 0]);
        assert_eq!(decode_credit(&bytes), Ok(c));
        let with = |at: usize, b: u8| {
            let mut v = bytes;
            v[at] = b;
            decode_credit(&v)
        };
        assert_eq!(with(0, 0), Err(CodecError::BadMagic));
        assert_eq!(with(4, 0), Err(CodecError::BadVersion));
        for at in 5..8 {
            assert_eq!(with(at, 1), Err(CodecError::Reserved));
        }
        for n in 0..CREDIT_LEN {
            assert_eq!(decode_credit(&bytes[..n]), Err(CodecError::Short));
        }
        let mut long = bytes.to_vec();
        long.push(0);
        assert_eq!(decode_credit(&long), Err(CodecError::Trailing));
        // A data record read where a credit is expected.
        let data = encode_data(0, &[], &[0; 4]).unwrap();
        assert_eq!(decode_credit(&data), Err(CodecError::BadMagic));
    }

    #[test]
    fn wire_kinds_are_exactly_one_to_three() {
        for b in 0..=u8::MAX {
            let k = WireKind::from_byte(b);
            assert_eq!(k.is_some(), (1..=3).contains(&b));
            assert_eq!(k.map(WireKind::to_byte), k.map(|_| b));
        }
    }
}
