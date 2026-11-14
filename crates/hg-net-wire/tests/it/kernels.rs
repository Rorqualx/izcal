//! The checked kernels against the RFCs' own worked numbers.
//!
//! Kani proves these total for every input (`src/proofs.rs`). What it
//! cannot prove is that "total" means the same thing the standard
//! meant, so these tests carry the examples the RFCs print and the
//! edges their prose names.

use hg_net_wire::{ack, flow, packet_number, varint};

// --- varint (RFC 9000 §16) -------------------------------------------------

/// The four examples of RFC 9000 §16, by their published bytes.
#[test]
fn nw10_the_standards_four_worked_examples() {
    // "the eight-byte sequence c2 19 7c 5e ff 14 e8 8c decodes to the
    // decimal value 151288809941952652"
    assert_eq!(
        varint::decode(&[0xc2, 0x19, 0x7c, 0x5e, 0xff, 0x14, 0xe8, 0x8c]),
        Some((151_288_809_941_952_652, 8))
    );
    // "the four-byte sequence 9d 7f 3e 7d decodes to 494878333"
    assert_eq!(
        varint::decode(&[0x9d, 0x7f, 0x3e, 0x7d]),
        Some((494_878_333, 4))
    );
    // "the two-byte sequence 7b bd decodes to 15293"
    assert_eq!(varint::decode(&[0x7b, 0xbd]), Some((15_293, 2)));
    // "the single byte 25 decodes to 37 (as does the two-byte sequence
    // 40 25)"
    assert_eq!(varint::decode(&[0x25]), Some((37, 1)));
    assert_eq!(varint::decode(&[0x40, 0x25]), Some((37, 2)));
}

/// The standard does not require the minimal encoding, so a decoder
/// that refused `40 25` would reject a conforming peer. §12.4's
/// separate rule about frame types is a separate question, and
/// `is_minimal` is where it is asked.
#[test]
fn nw11_a_non_minimal_encoding_decodes_but_is_not_minimal() {
    let (v, len) = varint::decode(&[0x40, 0x25]).expect("two bytes");
    assert_eq!(v, 37);
    assert_eq!(len, 2);
    assert!(!varint::is_minimal(v, len));
    assert!(varint::is_minimal(v, 1));
}

/// The boundaries of the four lengths, each at its largest value and
/// one past it.
#[test]
fn nw12_each_length_takes_exactly_its_range() {
    for (v, want) in [
        (0_u64, 1_usize),
        (63, 1),
        (64, 2),
        (16_383, 2),
        (16_384, 4),
        (1_073_741_823, 4),
        (1_073_741_824, 8),
        (varint::MAX, 8),
    ] {
        assert_eq!(varint::encoded_len(v), Some(want), "{v}");
        let mut out = [0_u8; 8];
        assert_eq!(varint::encode(v, &mut out), Some(want));
        assert_eq!(varint::decode(&out), Some((v, want)));
    }
    assert_eq!(
        varint::encoded_len(varint::MAX.saturating_add(1)),
        None,
        "2^62 does not fit 62 bits"
    );
}

/// Fewer bytes than the length prefix claims is `None`, not a partial
/// value: a truncated frame must not decode to a number.
#[test]
fn nw13_a_truncated_varint_declines() {
    assert_eq!(varint::decode(&[]), None);
    assert_eq!(varint::decode(&[0xc2]), None, "claims 8, holds 1");
    assert_eq!(varint::decode(&[0x9d, 0x7f]), None, "claims 4, holds 2");
    assert_eq!(varint::decode(&[0x7b]), None, "claims 2, holds 1");
}

// --- packet numbers (RFC 9000 Appendix A) ----------------------------------

/// Appendix A.3's own worked example: "if the highest successfully
/// authenticated packet had a packet number of 0xa82f30ea, then a
/// packet containing a 16-bit value of 0x9b32 will be decoded as
/// 0xa82f9b32".
#[test]
fn nw14_appendix_a3s_worked_example() {
    assert_eq!(
        packet_number::decode(0x9b32, 2, Some(0xa82f_30ea)),
        Ok(0xa82f_9b32)
    );
}

/// The edge the guards exist for: at the start of a connection
/// `expected` is below half a window, so A.3's `expected_pn - pn_hwin`
/// would go negative. The answer must be the candidate itself, not a
/// number that wrapped.
#[test]
fn nw15a_the_start_of_a_connection_does_not_underflow() {
    // Nothing processed yet: expected is 0.
    assert_eq!(packet_number::decode(0x00, 1, None), Ok(0));
    assert_eq!(packet_number::decode(0xff, 1, None), Ok(0xff));
    // One packet processed: expected is 1, half a window is 0x80.
    assert_eq!(packet_number::decode(0x01, 1, Some(0)), Ok(1));
    assert_eq!(
        packet_number::decode(0xfe, 1, Some(0)),
        Ok(0xfe),
        "not 0xfe - 0x100, which would wrap below zero"
    );
}

/// The other edge: near the top of the space, adding a window would
/// leave it. A.3's `candidate_pn < (1 << 62) - pn_win` is what stops
/// that, and the result is still a packet number.
#[test]
fn nw16_the_top_of_the_space_stays_in_the_space() {
    let near_top = packet_number::MAX.saturating_sub(4);
    for truncated in [0_u32, 1, 0x7f, 0xff] {
        let pn = packet_number::decode(truncated, 1, Some(near_top)).expect("in range");
        assert!(pn <= packet_number::MAX, "{truncated:#x} gave {pn}");
    }
    // The space exhausted: `largest` at MAX must not push `expected`
    // out of it.
    let pn = packet_number::decode(0, 4, Some(packet_number::MAX)).expect("in range");
    assert!(pn <= packet_number::MAX);
}

/// A truncated number carrying more bits than its length holds is
/// refused, not masked. Found by Kani, not by a test: a parser reading
/// one byte never passes 0x1234_5678, which is why nothing here would
/// have caught the first version OR-ing all 32 bits into the
/// candidate.
#[test]
fn nw17a_a_truncated_number_wider_than_its_length_is_refused() {
    assert_eq!(
        packet_number::decode(0x1234_5678, 1, Some(0)),
        Err(packet_number::PacketNumberError::TruncatedTooLarge {
            value: 0x1234_5678,
            len: 1,
        })
    );
    // The twin: the same low byte, within one byte.
    assert_eq!(packet_number::decode(0x78, 1, Some(0)), Ok(0x78));
    // Each length's own boundary, and one past it.
    for (len, max) in [(1_usize, 0xFF_u32), (2, 0xFFFF), (3, 0x00FF_FFFF)] {
        assert!(
            packet_number::decode(max, len, Some(0)).is_ok(),
            "{len}: its own maximum fits"
        );
        assert!(
            packet_number::decode(max + 1, len, Some(0)).is_err(),
            "{len}: one past it does not"
        );
    }
    // Four bytes is the whole of u32, so nothing is too large.
    assert!(packet_number::decode(u32::MAX, 4, Some(0)).is_ok());
}

/// A length the standard does not define is refused rather than
/// guessed at.
#[test]
fn nw17_a_bad_length_is_refused() {
    assert_eq!(
        packet_number::decode(0, 0, None),
        Err(packet_number::PacketNumberError::BadLength(0))
    );
    assert_eq!(
        packet_number::decode(0, 5, None),
        Err(packet_number::PacketNumberError::BadLength(5))
    );
    assert_eq!(
        packet_number::truncate(7, 9),
        Err(packet_number::PacketNumberError::BadLength(9))
    );
}

/// Appendix A.2: the length must span twice the range that could be
/// in flight.
#[test]
fn nw18_the_sent_length_spans_twice_the_range() {
    // The standard's own pair. 0xac5c02 - 0xabe8b3 = 29,519; twice
    // that is 59,038, which fits 16 bits, so two bytes.
    assert_eq!(packet_number::encoded_len(0xac5c02, Some(0xabe8b3)), Ok(2));
    // 0xace8fe - 0xabe8b3 = 65,611; twice that is 131,222, which does
    // not fit 16 bits, so three.
    assert_eq!(packet_number::encoded_len(0xace8fe, Some(0xabe8b3)), Ok(3));
    assert_eq!(
        packet_number::encoded_len(packet_number::MAX.saturating_add(1), None),
        Err(packet_number::PacketNumberError::OutOfRange(
            packet_number::MAX.saturating_add(1)
        ))
    );
}

// --- ACK ranges (RFC 9000 §19.3) -------------------------------------------

/// A frame that acknowledges 100..=100, then 90..=95, then 80..=80.
#[test]
fn nw19_a_frame_walks_downwards() {
    let (first, c) = ack::first(100, 0).expect("100 alone");
    assert_eq!(
        first,
        ack::Range {
            largest: 100,
            smallest: 100
        }
    );
    assert_eq!(first.count(), 1);

    // Gap 4 means 99, 98, 97, 96 are missing: largest = 100 - 4 - 2.
    let (second, c) = c.next_range(4, 5).expect("95 down to 90");
    assert_eq!(
        second,
        ack::Range {
            largest: 94,
            smallest: 89
        }
    );
    assert!(second.contains(90));
    assert!(!second.contains(95));

    let (third, _) = c.next_range(0, 0).expect("87 alone");
    assert_eq!(
        third,
        ack::Range {
            largest: 87,
            smallest: 87
        }
    );
}

/// §19.3.1: "if any computed packet number is negative, an endpoint
/// MUST generate a connection error of type FRAME_ENCODING_ERROR".
/// Saturating here would accept a frame the standard says to reject.
#[test]
fn nw20_a_range_below_zero_is_an_error_not_a_clamp() {
    assert_eq!(ack::first(5, 6), Err(ack::AckError::Negative));
    let (_, c) = ack::first(5, 0).expect("5 alone");
    // 5 - 0 - 2 = 3, fine; then 3 - 3 - 2 would be negative.
    let (_, c) = c.next_range(0, 0).expect("3 alone");
    assert_eq!(c.next_range(3, 0), Err(ack::AckError::Negative));
    // And a length that reaches below zero from a valid largest.
    let (_, c2) = ack::first(5, 0).expect("5 alone");
    assert_eq!(c2.next_range(0, 4), Err(ack::AckError::Negative));
}

/// A largest acknowledged above the packet-number space is refused
/// before any arithmetic is done on it.
#[test]
fn nw21_an_impossible_largest_is_refused() {
    let too_big = packet_number::MAX.saturating_add(1);
    assert_eq!(
        ack::first(too_big, 0),
        Err(ack::AckError::OutOfRange(too_big))
    );
}

// --- flow control (RFC 9000 §4) --------------------------------------------

/// §4.1: sending past the limit is a `FLOW_CONTROL_ERROR`, not a
/// clamp.
#[test]
fn nw22_consuming_past_the_limit_is_an_error() {
    let mut w = flow::Window::new(100);
    assert_eq!(w.consume(100), Ok(()));
    assert_eq!(w.available(), 0);
    assert_eq!(
        w.consume(1),
        Err(flow::FlowError::Exceeded {
            would_be: 101,
            limit: 100
        })
    );
    assert_eq!(w.consumed(), 100, "a refused consume changes nothing");
}

/// §4.1: a `MAX_DATA` that does not raise the limit "MUST be ignored"
/// — ignored, not an error, because a retransmission arriving out of
/// order is ordinary.
#[test]
fn nw23_a_limit_never_goes_down() {
    let mut w = flow::Window::new(100);
    assert!(w.raise_limit(200));
    assert_eq!(w.limit(), 200);
    assert!(!w.raise_limit(150), "ignored");
    assert_eq!(w.limit(), 200);
    assert!(!w.raise_limit(200), "ignored: not an increase");
    assert_eq!(w.limit(), 200);
}

/// §4.5: two different final sizes for one stream is a
/// `FINAL_SIZE_ERROR`, and so is data past a final size already known.
#[test]
fn nw24_a_final_size_is_final() {
    let mut w = flow::Window::new(100);
    assert_eq!(w.consume(40), Ok(()));
    assert_eq!(w.set_final_size(50), Ok(()));
    assert_eq!(w.set_final_size(50), Ok(()), "the same size again is fine");
    assert_eq!(
        w.set_final_size(60),
        Err(flow::FlowError::FinalSize {
            known: 50,
            claimed: 60
        })
    );
    assert_eq!(w.consume(10), Ok(()), "up to the final size");
    assert_eq!(
        w.consume(1),
        Err(flow::FlowError::FinalSize {
            known: 50,
            claimed: 51
        })
    );
}

/// A final size below what has already arrived is an error too: the
/// bytes cannot be unsent.
#[test]
fn nw25_a_final_size_below_what_arrived_is_refused() {
    let mut w = flow::Window::new(100);
    assert_eq!(w.consume(40), Ok(()));
    assert_eq!(
        w.set_final_size(30),
        Err(flow::FlowError::FinalSize {
            known: 40,
            claimed: 30
        })
    );
    assert_eq!(w.final_size(), None);
}

/// The update rule: once half the window is spent, not before and not
/// only at the limit.
#[test]
fn nw26_the_window_update_rule() {
    let mut w = flow::Window::new(100);
    assert!(!w.should_update(100));
    assert_eq!(w.consume(49), Ok(()));
    assert!(!w.should_update(100), "51 left of 100");
    assert_eq!(w.consume(1), Ok(()));
    assert!(w.should_update(100), "50 left of 100");
}
