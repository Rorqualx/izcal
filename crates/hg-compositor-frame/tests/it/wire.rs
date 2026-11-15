//! The wire: a frame survives the crossing, a declared count above
//! the format's own bound is refused before anything is read, and
//! every refusal crosses back.

use hg_compositor_frame::compositor_frame::{
    AnimatedProperty, Animation, ClipNode, ClipShape, CompositorFrameError, Corners, EffectNode,
    Embed, Frame, Keyframe, KeyframeValue, Layer, Rect, ScrollNode, TextureRef, Tile,
    TimingFunction, Transform, TransformNode,
};
use hg_ipc::BoundedVec;
use hg_ipc::wire::{Wire, Writer, decode_exact, encode_to_vec, put_count, validate_exact};

fn bv<T, const N: usize>(v: Vec<T>) -> BoundedVec<T, N> {
    BoundedVec::try_new(v).unwrap()
}

/// A frame whose every field is a value a default would not produce:
/// extreme `i64` fixed point, a negative translation, the whole
/// corner set distinct, both booleans of a scroll node, and each
/// union carrying its less obvious case.
fn sample() -> Frame {
    Frame {
        frame_id: u64::MAX,
        fence: 9_007_199_254_740_993,
        device_scale_num: 7,
        device_scale_den: 65_536,
        viewport: Rect {
            x: -2_147_483_648,
            y: 2_147_483_647,
            width: 0,
            height: 0,
        },
        transforms: bv(vec![TransformNode {
            parent: u32::MAX,
            matrix: Transform {
                a: i64::MIN,
                b: -1,
                c: 1,
                d: i64::MAX,
                tx: -4_294_967_296,
                ty: 4_294_967_296,
            },
        }]),
        clips: bv(vec![ClipNode {
            parent: u32::MAX,
            transform: 0,
            shape: ClipShape::Rounded {
                bounds: Rect {
                    x: 0,
                    y: 0,
                    width: 2_000_000,
                    height: 2_000_000,
                },
                radii: Corners {
                    top_left_x: 1,
                    top_left_y: 2,
                    top_right_x: 3,
                    top_right_y: 4,
                    bottom_right_x: 5,
                    bottom_right_y: 6,
                    bottom_left_x: 7,
                    bottom_left_y: 8,
                },
            },
        }]),
        effects: bv(vec![EffectNode {
            parent: u32::MAX,
            opacity: 255,
        }]),
        scrolls: bv(vec![ScrollNode {
            parent: u32::MAX,
            transform: 0,
            container: Rect {
                x: 1,
                y: 2,
                width: 3,
                height: 4,
            },
            content: Rect {
                x: 5,
                y: 6,
                width: 7,
                height: 8,
            },
            offset_x: -9,
            offset_y: 10,
            user_scrollable_x: true,
            user_scrollable_y: true,
        }]),
        layers: bv(vec![Layer {
            origin_x: -9_007_199_254_740_993,
            origin_y: 4_294_967_296,
            tile_width: 1,
            tile_height: u32::MAX,
            columns: 1,
            rows: 1,
            transform: 0,
            clip: u32::MAX,
            effect: u32::MAX,
            scroll: u32::MAX,
            is_opaque: false,
            tiles: bv(vec![Tile {
                column: 0,
                row: 0,
                texture: TextureRef {
                    id: u64::MAX,
                    generation: u32::MAX,
                    width: 1,
                    height: 1,
                },
            }]),
        }]),
        embeds: bv(vec![Embed {
            surface: 1,
            generation: 2,
            transform: 0,
            clip: 0,
            bounds: Rect {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
        }]),
        animations: bv(vec![Animation {
            id: 3,
            target: 0,
            property: AnimatedProperty::Transform,
            start_time_us: i64::MIN,
            duration_us: i64::MAX,
            bounds: Rect {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            keyframes: bv(vec![
                Keyframe {
                    offset: 0,
                    value: KeyframeValue::Transform {
                        matrix: Transform {
                            a: 1,
                            b: 2,
                            c: 3,
                            d: 4,
                            tx: 5,
                            ty: 6,
                        },
                    },
                    easing: TimingFunction::Steps {
                        count: u32::MAX,
                        jump_at_start: true,
                        jump_at_end: true,
                    },
                },
                Keyframe {
                    offset: 65_536,
                    value: KeyframeValue::Opacity { opacity: 1 },
                    easing: TimingFunction::CubicBezier {
                        x1: -1,
                        y1: i32::MIN,
                        x2: i32::MAX,
                        y2: 0,
                    },
                },
            ]),
        }]),
        damage: bv(vec![Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }]),
    }
}

/// Every field survives, including the `i64` fixed point, the
/// `u32::MAX` sentinels and the booleans inside a scroll node, which
/// is where a wire format usually loses a bit.
#[test]
fn cf22_a_frame_round_trips() {
    let f = sample();
    let bytes = encode_to_vec(&f);
    assert_eq!(validate_exact::<Frame>(&bytes), Ok(()));
    assert_eq!(decode_exact::<Frame>(&bytes), Ok(f));
}

/// A count above the bound is refused by the reader, not by `check`:
/// the format has a word for a count, so the limit is enforced where
/// the count is read and no memory is reserved for it.
#[test]
fn cf23_a_declared_count_above_the_bound_is_refused() {
    // The frame's first vector is `transforms`, bounded at 1,024.
    let mut w = Writer::new();
    0_u64.encode(&mut w); // frame_id
    0_u64.encode(&mut w); // fence
    1_u32.encode(&mut w); // device_scale_num
    1_u32.encode(&mut w); // device_scale_den
    0_i32.encode(&mut w); // viewport.x
    0_i32.encode(&mut w); // viewport.y
    0_i32.encode(&mut w); // viewport.width
    0_i32.encode(&mut w); // viewport.height
    put_count(&mut w, 1_025);
    let bytes = w.into_bytes();
    assert!(decode_exact::<Frame>(&bytes).is_err(), "1,025 transforms");
    assert!(validate_exact::<Frame>(&bytes).is_err(), "1,025 transforms");

    // The twin: the same message declaring the bound exactly, with the
    // nodes to match, is accepted -- so the refusal above is about the
    // count and not about the shape of the message.
    let node = TransformNode {
        parent: u32::MAX,
        matrix: Transform {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            tx: 0,
            ty: 0,
        },
    };
    let mut ok = sample();
    ok.transforms = bv(vec![node; 1_024]);
    let ok_bytes = encode_to_vec(&ok);
    assert_eq!(decode_exact::<Frame>(&ok_bytes), Ok(ok));
}

/// Both counts of the one nested vector hold at once: 256 layers of
/// 4,096 tiles is the frame's worst allocation, and it is bounded
/// because each count is.
#[test]
fn cf24_the_nested_counts_are_both_enforced() {
    let mut f = sample();
    let tile = Tile {
        column: 0,
        row: 0,
        texture: TextureRef {
            id: 0,
            generation: 0,
            width: 1,
            height: 1,
        },
    };
    assert!(
        BoundedVec::<Tile, 4096>::try_new(vec![tile.clone(); 4_097]).is_err(),
        "4,097 tiles in one layer"
    );
    let mut layer = f.layers.as_slice().first().unwrap().clone();
    layer.tiles = bv(vec![tile; 4_096]);
    assert!(
        BoundedVec::<Layer, 256>::try_new(vec![layer.clone(); 257]).is_err(),
        "257 layers in one frame"
    );
    f.layers = bv(vec![layer; 256]);
    let bytes = encode_to_vec(&f);
    assert_eq!(validate_exact::<Frame>(&bytes), Ok(()));
}

/// The error type crosses the boundary too, and every variant is a
/// limit the format names (CS-ER-6: no free text).
#[test]
fn cf25_every_refusal_round_trips() {
    let all = [
        CompositorFrameError::ParentOutOfRange,
        CompositorFrameError::ParentNotEarlier,
        CompositorFrameError::NodeOutOfRange,
        CompositorFrameError::TooManyMutableNodes,
        CompositorFrameError::NegativeExtent,
        CompositorFrameError::TileOutsideGrid,
        CompositorFrameError::DuplicateTile,
        CompositorFrameError::ZeroTileExtent,
        CompositorFrameError::NegativeRadius,
        CompositorFrameError::RadiiExceedBounds,
        CompositorFrameError::BadDeviceScale,
        CompositorFrameError::DamageOutsideViewport,
        CompositorFrameError::KeyframesUnordered,
        CompositorFrameError::TooFewKeyframes,
        CompositorFrameError::ZeroSteps,
        CompositorFrameError::TargetWrongKind,
        CompositorFrameError::KeyframeWrongProperty,
    ];
    for e in all {
        assert_eq!(
            decode_exact::<CompositorFrameError>(&encode_to_vec(&e)),
            Ok(e)
        );
    }

    // An unknown discriminant is refused rather than mapped to one of
    // them: a content GPU process that invents a reason is one the
    // compositor has no answer for.
    let mut w = Writer::new();
    let past_the_end = u32::try_from(all.len()).unwrap();
    past_the_end.encode(&mut w);
    assert!(decode_exact::<CompositorFrameError>(&w.into_bytes()).is_err());
}
