//! Round trip and mutation over the leaf types, from seeded random
//! values (CS-TE-10), and the two measurements the bounds imply.
//!
//! The leaves and not `Frame`, `Layer` or `Animation`: the bounds are
//! 256 layers of 4,096 tiles *by design*, so a random maximal frame is
//! megabytes and sampling one says nothing a smaller case does not.
//! The composites are covered by hand-built values in
//! `tests/it/wire.rs`, and the size the bounds imply is asserted
//! below rather than sampled.

use hg_ipc::{BadMessage, Reader, Wire, decode_exact, encode_to_vec, validate_exact};

use crate::generated::compositor_frame as cf;
use crate::sample::{Gen, Sample};

/// The run's seed: fixed, printed, so a failure replays.
const SEED: u64 = 0x636f_6d70;
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
    round_trip::<cf::Transform>("Transform");
    round_trip::<cf::Rect>("Rect");
    round_trip::<cf::Corners>("Corners");
    round_trip::<cf::ClipShape>("ClipShape");
    round_trip::<cf::TransformNode>("TransformNode");
    round_trip::<cf::ClipNode>("ClipNode");
    round_trip::<cf::EffectNode>("EffectNode");
    round_trip::<cf::ScrollNode>("ScrollNode");
    round_trip::<cf::TextureRef>("TextureRef");
    round_trip::<cf::Tile>("Tile");
    round_trip::<cf::Embed>("Embed");
    round_trip::<cf::AnimatedProperty>("AnimatedProperty");
    round_trip::<cf::KeyframeValue>("KeyframeValue");
    round_trip::<cf::TimingFunction>("TimingFunction");
    round_trip::<cf::Keyframe>("Keyframe");
    round_trip::<cf::CompositorFrameError>("CompositorFrameError");
}

#[test]
fn mutated_leaves_are_rejected_or_exact() {
    for (name, (rej, acc)) in [
        ("Transform", mutations::<cf::Transform>("Transform")),
        ("Rect", mutations::<cf::Rect>("Rect")),
        ("Corners", mutations::<cf::Corners>("Corners")),
        ("ClipShape", mutations::<cf::ClipShape>("ClipShape")),
        (
            "TransformNode",
            mutations::<cf::TransformNode>("TransformNode"),
        ),
        ("ClipNode", mutations::<cf::ClipNode>("ClipNode")),
        ("EffectNode", mutations::<cf::EffectNode>("EffectNode")),
        ("ScrollNode", mutations::<cf::ScrollNode>("ScrollNode")),
        ("TextureRef", mutations::<cf::TextureRef>("TextureRef")),
        ("Tile", mutations::<cf::Tile>("Tile")),
        ("Embed", mutations::<cf::Embed>("Embed")),
        (
            "AnimatedProperty",
            mutations::<cf::AnimatedProperty>("AnimatedProperty"),
        ),
        (
            "KeyframeValue",
            mutations::<cf::KeyframeValue>("KeyframeValue"),
        ),
        (
            "TimingFunction",
            mutations::<cf::TimingFunction>("TimingFunction"),
        ),
        ("Keyframe", mutations::<cf::Keyframe>("Keyframe")),
        (
            "CompositorFrameError",
            mutations::<cf::CompositorFrameError>("CompositorFrameError"),
        ),
    ] {
        assert!(rej > 0 && acc > 0, "{name}: rejected {rej}, accepted {acc}");
    }
}

/// **A maximal frame does not fit in an IPC message either, and this
/// records by how much.**
///
/// The display list has the same problem (`hg-display-list`'s
/// `a_maximal_chunk_is_far_larger_than_a_message`), and for the
/// compositor frame it lands differently, because **GR-18 asks this
/// format to be small**. So the number is not only a transport
/// question here; it is evidence about the target.
///
/// Asserted rather than stated, so a later change to any bound is seen
/// here. **GR-M0's review owes the decision**, with the same two
/// options the display list has — shrink the bounds to a message, or
/// write the copied-then-validated shared-buffer transport into the
/// format — and one more that is specific to this format: GR-18's
/// published size may be the place to record the worst-case *bytes*
/// alongside the type and variant counts, since a format can be small
/// in types and enormous in bytes, and it is the bytes an untrusted
/// peer makes the compositor allocate.
#[test]
fn a_maximal_frame_is_far_larger_than_a_message() {
    let cap = hg_substrate::ipc::MAX_MESSAGE_BYTES;
    let frame = <cf::Frame as Wire>::MAX_LEN;
    assert!(frame > cap, "frame {frame} B, message cap {cap} B");
    // The leaves, by contrast, all fit many times over.
    const { assert!(<cf::Tile as Wire>::MAX_LEN <= 32) };
    const { assert!(<cf::Rect as Wire>::MAX_LEN <= 16) };
    const { assert!(<cf::Transform as Wire>::MAX_LEN <= 48) };
}

/// The measured numbers, printed so the review document can quote them
/// rather than estimate them. `cargo test -p hg-compositor-frame --
/// --nocapture the_measured_bounds`.
#[test]
fn the_measured_bounds() {
    println!(
        "Frame::MAX_LEN = {} B; Layer::MAX_LEN = {} B; Animation::MAX_LEN = {} B; \
         Tile::MAX_LEN = {} B; Transform::MAX_LEN = {} B; MAX_MESSAGE_BYTES = {} B",
        <cf::Frame as Wire>::MAX_LEN,
        <cf::Layer as Wire>::MAX_LEN,
        <cf::Animation as Wire>::MAX_LEN,
        <cf::Tile as Wire>::MAX_LEN,
        <cf::Transform as Wire>::MAX_LEN,
        hg_substrate::ipc::MAX_MESSAGE_BYTES,
    );
}
