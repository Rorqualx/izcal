//! Kani harnesses for the checked kernels (G33).
//!
//! Each kernel reads numbers a peer chose, so what has to be proved is
//! not that it is correct on the inputs we thought of but that it is
//! **total**: for every input at all, it answers or declines, and
//! never panics, overflows or produces a value outside the range its
//! type promises.
//!
//! The harnesses are bounded only where the input is a slice, and the
//! bound is chosen to reach every branch: a variable-length integer is
//! at most 8 bytes, so 9 covers "one byte too few" as well.

#![cfg(kani)]

use crate::{ack, budget, flow, packet_number, varint};

/// How many bytes a slice harness gives a decoder.
const BOUND: usize = 9;

// --- varint (RFC 9000 §16) -------------------------------------------------

/// `decode` is total for any bytes at all.
#[kani::proof]
fn varint_decode_is_total() {
    let data: [u8; BOUND] = kani::any();
    let n: usize = kani::any();
    kani::assume(n <= BOUND);
    if let Some((_, _)) = data.get(..n).and_then(varint::decode) {}
}

/// A decoded value is inside the 62-bit space, and the length it
/// reports is one the standard defines.
#[kani::proof]
fn varint_decode_stays_in_the_space() {
    let data: [u8; BOUND] = kani::any();
    if let Some((v, len)) = varint::decode(&data) {
        assert!(v <= varint::MAX);
        assert!(len == 1 || len == 2 || len == 4 || len == 8);
        assert!(len <= BOUND);
    }
}

/// Encoding and decoding round-trip, and the encoding is the minimal
/// one.
///
/// One harness per length rather than one over the whole 62-bit space.
/// The whole space leaves the encoded length symbolic, so every
/// `copy_from_slice` and every bounds check inside `encode` stays
/// symbolic too, and CBMC spends its time on a case split the standard
/// already made: there are exactly four lengths. Splitting the harness
/// pins the length in each one, which is the same property proved four
/// times over four small problems.
fn round_trip(v: u64, want_len: usize) {
    let mut out = [0_u8; 8];
    let len = varint::encode(v, &mut out).expect("every value up to MAX encodes");
    assert!(len == want_len);
    assert!(varint::is_minimal(v, len));
    let (back, read) = varint::decode(&out).expect("what we wrote decodes");
    assert!(back == v);
    assert!(read == len);
}

/// One byte: 0 to 63.
#[kani::proof]
fn varint_round_trips_one_byte() {
    let v: u64 = kani::any();
    kani::assume(v <= 63);
    round_trip(v, 1);
}

/// Two bytes: 64 to 16,383.
#[kani::proof]
fn varint_round_trips_two_bytes() {
    let v: u64 = kani::any();
    kani::assume(v > 63 && v <= 16_383);
    round_trip(v, 2);
}

/// Four bytes: 16,384 to 2^30 - 1.
#[kani::proof]
fn varint_round_trips_four_bytes() {
    let v: u64 = kani::any();
    kani::assume(v > 16_383 && v <= 1_073_741_823);
    round_trip(v, 4);
}

/// Eight bytes: 2^30 to 2^62 - 1.
#[kani::proof]
fn varint_round_trips_eight_bytes() {
    let v: u64 = kani::any();
    kani::assume(v > 1_073_741_823 && v <= varint::MAX);
    round_trip(v, 8);
}

/// `encode` into a buffer that is too short writes nothing and says so.
#[kani::proof]
fn varint_encode_declines_a_short_buffer() {
    let v: u64 = kani::any();
    let n: usize = kani::any();
    kani::assume(n <= 8);
    let mut out = [0_u8; 8];
    let before = out;
    if let Some(slot) = out.get_mut(..n) {
        if varint::encode(v, slot).is_none() {
            assert!(out == before);
        }
    }
}

// --- packet numbers (RFC 9000 Appendix A) ----------------------------------

/// `decode` is total, and its answer is a packet number.
#[kani::proof]
fn packet_number_decode_is_total() {
    let truncated: u32 = kani::any();
    let len: usize = kani::any();
    let has_largest: bool = kani::any();
    let largest: u64 = kani::any();
    let largest = if has_largest { Some(largest) } else { None };
    if let Ok(pn) = packet_number::decode(truncated, len, largest) {
        assert!(pn <= packet_number::MAX);
    }
}

/// The low bytes of a decoded number are the bytes that were on the
/// wire: whatever window the reconstruction chose, it cannot have
/// changed what the peer actually sent.
#[kani::proof]
fn packet_number_decode_keeps_the_wire_bits() {
    let truncated: u32 = kani::any();
    let len: usize = kani::any();
    kani::assume(len >= 1 && len <= 4);
    let largest: u64 = kani::any();
    kani::assume(largest <= packet_number::MAX);
    if let Ok(pn) = packet_number::decode(truncated, len, Some(largest)) {
        let back = packet_number::truncate(pn, len).expect("pn is in range");
        // Whatever window the reconstruction chose, it cannot have
        // changed the bits the peer sent. A `truncated` carrying bits
        // the length does not hold is refused above, so the equality
        // here is unconditional.
        assert!(back == truncated);
    }
}

/// `encoded_len` and `truncate` are total.
#[kani::proof]
fn packet_number_encoding_is_total() {
    let full: u64 = kani::any();
    let largest: u64 = kani::any();
    let has: bool = kani::any();
    let acked = if has { Some(largest) } else { None };
    if let Ok(len) = packet_number::encoded_len(full, acked) {
        assert!(len >= 1 && len <= 4);
        let _ = packet_number::truncate(full, len);
    }
}

// --- ACK ranges (RFC 9000 §19.3) -------------------------------------------

/// `first` is total, and a range it returns is well formed.
#[kani::proof]
fn ack_first_is_total() {
    let largest: u64 = kani::any();
    let first_range: u64 = kani::any();
    if let Ok((r, c)) = ack::first(largest, first_range) {
        assert!(r.smallest <= r.largest);
        assert!(r.largest == largest);
        assert!(c.smallest() == r.smallest);
        assert!(r.count() >= 1);
    }
}

/// A step down is total, well formed, and strictly below the one
/// before it: an ACK frame cannot be made to walk upwards.
#[kani::proof]
fn ack_steps_strictly_downwards() {
    let largest: u64 = kani::any();
    let first_range: u64 = kani::any();
    let gap: u64 = kani::any();
    let len: u64 = kani::any();
    if let Ok((first, cursor)) = ack::first(largest, first_range)
        && let Ok((next, cursor2)) = cursor.next_range(gap, len)
    {
        assert!(next.smallest <= next.largest);
        assert!(next.largest < first.smallest);
        assert!(cursor2.smallest() == next.smallest);
    }
}

// --- flow control (RFC 9000 §4) --------------------------------------------

/// Consuming is total, and never leaves more consumed than the limit
/// allows.
#[kani::proof]
fn flow_consume_is_total() {
    let initial: u64 = kani::any();
    let n: u64 = kani::any();
    let mut w = flow::Window::new(initial);
    let before = w.consumed();
    match w.consume(n) {
        Ok(()) => {
            assert!(w.consumed() <= w.limit());
            assert!(w.consumed() >= before);
        }
        // A refused charge changes nothing.
        Err(_) => assert!(w.consumed() == before),
    }
    assert!(w.available() <= w.limit());
}

/// A limit never goes down, whatever a peer sends.
#[kani::proof]
fn flow_limit_never_decreases() {
    let initial: u64 = kani::any();
    let proposed: u64 = kani::any();
    let mut w = flow::Window::new(initial);
    let before = w.limit();
    let moved = w.raise_limit(proposed);
    assert!(w.limit() >= before);
    if moved {
        assert!(w.limit() == proposed);
    } else {
        assert!(w.limit() == before);
    }
}

/// A final size, once accepted, cannot be changed to another.
#[kani::proof]
fn flow_final_size_is_final() {
    let initial: u64 = kani::any();
    let a: u64 = kani::any();
    let b: u64 = kani::any();
    let mut w = flow::Window::new(initial);
    if w.set_final_size(a).is_ok() {
        assert!(w.final_size() == Some(a));
        let second = w.set_final_size(b);
        if a == b {
            assert!(second.is_ok());
        } else {
            assert!(second.is_err());
        }
        assert!(w.final_size() == Some(a));
    }
}

// --- budgets (§6.2) --------------------------------------------------------

/// Charging is total and never passes the cap; a refused charge
/// changes nothing.
#[kani::proof]
fn budget_charge_is_total() {
    let cap: u64 = kani::any();
    let n: u64 = kani::any();
    let mut b = budget::Budget::new(budget::Limit::AckRanges, cap);
    let before = b.used();
    match b.charge(n) {
        Ok(()) => assert!(b.used() <= b.cap()),
        Err(_) => assert!(b.used() == before),
    }
    assert!(b.remaining() <= b.cap());
}

/// `would_fit` agrees with `charge`: a caller that asked first is not
/// told something different when it commits.
#[kani::proof]
fn budget_would_fit_agrees_with_charge() {
    let cap: u64 = kani::any();
    let used: u64 = kani::any();
    let n: u64 = kani::any();
    let mut b = budget::Budget::new(budget::Limit::AckRanges, cap);
    kani::assume(used <= cap);
    let _ = b.charge(used);
    assert!(b.would_fit(n) == b.charge(n).is_ok());
}

/// The roll-up is all or nothing: when any budget in the chain
/// refuses, every one of them is left as it was.
#[kani::proof]
fn budget_charge_all_is_atomic() {
    let c0: u64 = kani::any();
    let c1: u64 = kani::any();
    let c2: u64 = kani::any();
    let n: u64 = kani::any();
    let mut connection = budget::Budget::new(budget::Limit::AckRanges, c0);
    let mut partition = budget::Budget::new(budget::Limit::PartitionBytes, c1);
    let mut service = budget::Budget::new(budget::Limit::ServiceBytes, c2);
    let before = (connection.used(), partition.used(), service.used());
    let r = budget::charge_all(&mut [&mut connection, &mut partition, &mut service], n);
    let after = (connection.used(), partition.used(), service.used());
    match r {
        Ok(()) => {
            assert!(connection.used() <= connection.cap());
            assert!(partition.used() <= partition.cap());
            assert!(service.used() <= service.cap());
        }
        Err(_) => assert!(after == before),
    }
}
