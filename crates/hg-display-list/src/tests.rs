//! Round trip and mutation over the leaf types, from seeded random
//! values (CS-TE-10), and the one measurement the bounds imply.
//!
//! The leaves and not `Chunk`, `Item` or `Path`: §6.3's bounds are
//! 65,536 segments and 4,096 items *by design*, so a random maximal
//! chunk is gigabytes and sampling one says nothing a smaller case
//! does not. The composites are covered by hand-built values in
//! `tests/it/wire.rs`, and the size the bounds imply is asserted
//! below rather than sampled.

use hg_ipc::{BadMessage, Reader, Wire, decode_exact, encode_to_vec, validate_exact};

use crate::generated::display_list as dl;
use crate::sample::{Gen, Sample};

/// The run's seed: fixed, printed, so a failure replays.
const SEED: u64 = 0x6772_6170;
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

/// Every mutation either fails to decode or decodes to something that
/// re-encodes to exactly the mutated bytes, and the validator gives
/// the decoder's own verdict.
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
fn every_leaf_type_round_trips() {
    round_trip::<dl::Point>("Point");
    round_trip::<dl::Segment>("Segment");
    round_trip::<dl::FillRule>("FillRule");
    round_trip::<dl::Colour>("Colour");
    round_trip::<dl::BlendMode>("BlendMode");
    round_trip::<dl::Rect>("Rect");
    round_trip::<dl::ImageRef>("ImageRef");
    round_trip::<dl::Sampling>("Sampling");
    round_trip::<dl::DisplayListError>("DisplayListError");
}

#[test]
fn mutated_leaves_are_rejected_or_exact() {
    for (name, (rej, acc)) in [
        ("Point", mutations::<dl::Point>("Point")),
        ("Segment", mutations::<dl::Segment>("Segment")),
        ("FillRule", mutations::<dl::FillRule>("FillRule")),
        ("Colour", mutations::<dl::Colour>("Colour")),
        ("BlendMode", mutations::<dl::BlendMode>("BlendMode")),
        ("Rect", mutations::<dl::Rect>("Rect")),
        ("ImageRef", mutations::<dl::ImageRef>("ImageRef")),
        ("Sampling", mutations::<dl::Sampling>("Sampling")),
        (
            "DisplayListError",
            mutations::<dl::DisplayListError>("DisplayListError"),
        ),
    ] {
        assert!(rej > 0 && acc > 0, "{name}: rejected {rej}, accepted {acc}");
    }
}

/// **A chunk does not fit in an IPC message, and this records by how
/// much.**
///
/// Appendix C caps a generated message at `MAX_MESSAGE_BYTES`
/// (64 KiB). §6.3's bounds — 4,096 items, each able to hold a path of
/// 65,536 segments — put a maximal chunk far above that, so a chunk
/// cannot travel as one message and the transport has to be the
/// copied-then-validated shared buffer §6.3 and [FP 6.4] describe.
///
/// The number is asserted rather than stated, so that a later change
/// to either bound is seen here. **GR-M0's review owes the decision**:
/// shrink the bounds to a message, or write the shared-buffer
/// transport into the format. See the review document's §3.
#[test]
fn a_maximal_chunk_is_far_larger_than_a_message() {
    let cap = hg_substrate::ipc::MAX_MESSAGE_BYTES;
    let chunk = <dl::Chunk as Wire>::MAX_LEN;
    assert!(chunk > cap, "chunk {chunk} B, message cap {cap} B");
    // The leaves, by contrast, all fit many times over.
    const { assert!(<dl::Segment as Wire>::MAX_LEN < 64) };
    const { assert!(<dl::Rect as Wire>::MAX_LEN <= 16) };
}

/// The measured numbers, printed so the review document can quote
/// them rather than estimate them. `cargo test -p hg-display-list --
/// --nocapture the_measured_bounds`.
#[test]
fn the_measured_bounds() {
    println!(
        "Chunk::MAX_LEN = {} B; Item::MAX_LEN = {} B; Path::MAX_LEN = {} B; \
         Segment::MAX_LEN = {} B; MAX_MESSAGE_BYTES = {} B",
        <dl::Chunk as Wire>::MAX_LEN,
        <dl::Item as Wire>::MAX_LEN,
        <dl::Path as Wire>::MAX_LEN,
        <dl::Segment as Wire>::MAX_LEN,
        hg_substrate::ipc::MAX_MESSAGE_BYTES,
    );
}
