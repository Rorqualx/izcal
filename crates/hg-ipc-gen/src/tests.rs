//! Round trip (IPC-R6) and mutation (IPC-R5) over every generated
//! interface, from seeded random values (CS-TE-10).

use hg_ipc::{BadMessage, Reader, Wire, decode_exact, encode_to_vec, validate_exact};

use crate::generated::{process, sample};
use crate::sample::{Gen, Sample};

/// The run's seed: fixed, printed, so a failure replays.
const SEED: u64 = 0x1dc0_ffee;
const CASES: usize = 4_000;

/// A per-type seed, so each type gets its own sequence.
fn seed_for(name: &str) -> u64 {
    name.bytes()
        .fold(SEED, |h, b| h.rotate_left(5) ^ u64::from(b))
}

fn round_trip<T: Wire + Sample + PartialEq + core::fmt::Debug>(name: &str) {
    let mut g = Gen::new(seed_for(name));
    for case in 0..CASES {
        let v = T::sample(&mut g);
        let bytes = encode_to_vec(&v);
        assert!(
            bytes.len() >= T::MIN_LEN && bytes.len() <= T::MAX_LEN,
            "{name} #{case}: size"
        );
        let back = decode_exact::<T>(&bytes).unwrap_or_else(|e| panic!("{name} #{case}: {e}"));
        // Re-encoding is the identity on bytes (floats compare by bits
        // through this, not through PartialEq).
        assert_eq!(
            encode_to_vec(&back),
            bytes,
            "{name} #{case} (seed {SEED:#x})"
        );
    }
}

/// The decoder alone, without the validator in front of it.
fn decode_unvalidated<T: Wire>(bytes: &[u8]) -> Result<T, BadMessage> {
    let mut r = Reader::new(bytes);
    let v = T::decode(&mut r)?;
    r.finish()?;
    Ok(v)
}

/// Every mutation of a valid message either fails to decode or decodes
/// to a message that re-encodes to exactly the mutated bytes, and the
/// validator gives the decoder's own verdict on it (`Wire::validate`).
fn mutations<T: Wire + Sample>(name: &str) -> (usize, usize) {
    let mut g = Gen::new(seed_for(name).rotate_left(7));
    let (mut rejected, mut accepted) = (0usize, 0usize);
    for _ in 0..CASES {
        let mut bytes = encode_to_vec(&T::sample(&mut g));
        match g.below(4) {
            0 => {
                let i = g.below(bytes.len().max(1));
                let bit = 1u8
                    .checked_shl(u32::try_from(g.below(8)).unwrap_or(0))
                    .unwrap_or(1);
                if let Some(b) = bytes.get_mut(i) {
                    *b ^= bit;
                }
            }
            1 => bytes.truncate(g.below(bytes.len().max(1))),
            2 => bytes.push(g.bits().to_le_bytes()[0]),
            _ => {
                let i = g.below(bytes.len().max(1));
                if let Some(b) = bytes.get_mut(i) {
                    *b = g.bits().to_le_bytes()[0];
                }
            }
        }
        assert_eq!(
            validate_exact::<T>(&bytes),
            decode_unvalidated::<T>(&bytes).map(|_v| ()),
            "{name}: the validator and the decoder disagree"
        );
        match decode_exact::<T>(&bytes) {
            Ok(v) => {
                assert_eq!(
                    encode_to_vec(&v),
                    bytes,
                    "{name}: accepted bytes must re-encode"
                );
                accepted = accepted.saturating_add(1);
            }
            Err(_) => rejected = rejected.saturating_add(1),
        }
    }
    (rejected, accepted)
}

#[test]
fn every_interface_round_trips() {
    round_trip::<process::ProcessHost>("process.ProcessHost");
    round_trip::<process::ProcessControl>("process.ProcessControl");
    round_trip::<sample::Strict>("sample.Strict");
    round_trip::<sample::Loose>("sample.Loose");
}

#[test]
fn mutated_messages_are_rejected_or_exact() {
    for (name, (rej, acc)) in [
        (
            "process.ProcessHost",
            mutations::<process::ProcessHost>("process.ProcessHost"),
        ),
        (
            "process.ProcessControl",
            mutations::<process::ProcessControl>("process.ProcessControl"),
        ),
        (
            "sample.Strict",
            mutations::<sample::Strict>("sample.Strict"),
        ),
        ("sample.Loose", mutations::<sample::Loose>("sample.Loose")),
    ] {
        assert!(rej > 0 && acc > 0, "{name}: rejected {rej}, accepted {acc}");
    }
}

#[test]
fn the_wire_bytes_are_fixed() {
    let m = process::ProcessControl::Exit {
        reason: process::ExitReason::BuildMismatch {
            expected: process::BuildId { id: 0x0102 },
        },
    };
    // ordinal 1 (u16), union tag 1 (u32), id (u64), all little-endian.
    assert_eq!(
        encode_to_vec(&m),
        [1, 0, 1, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        decode_exact::<process::ProcessControl>(&[3, 0]),
        Err(BadMessage::UnknownOrdinal)
    );
    assert_eq!(
        decode_exact::<process::ProcessControl>(&[1, 0, 9, 0, 0, 0]),
        Err(BadMessage::UnknownTag)
    );
    assert_eq!(process::ProcessHost::INFO.name, "process.ProcessHost");
    // ready: ordinal, kind, build id, then the sandbox report (tier, a
    // vec count and at most eight features, each a u32).
    assert_eq!(
        process::ProcessHost::INFO.max_message_len,
        2 + 4 + 8 + 4 + 4 + 8 * 4
    );
    assert_eq!(sample::Loose::INFO.binding, hg_ipc::Binding::Free);
}
