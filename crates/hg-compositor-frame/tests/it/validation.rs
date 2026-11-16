//! The fifteen things the wire format cannot say, each refused, and a
//! frame that says none of them, accepted.
//!
//! The accepted frame is not a courtesy: a validator that refuses
//! everything passes every refusal test, so each case below is a
//! single-field mutation of [`valid`] and the suite only means
//! something because [`valid`] itself passes.

use hg_compositor_frame::check::{MAX_MUTABLE_NODES, NO_NODE, NO_PARENT, mutable_node_count};
use hg_compositor_frame::compositor_frame::{
    AnimatedProperty, Animation, ClipNode, ClipShape, CompositorFrameError, Corners, EffectNode,
    Embed, Frame, Keyframe, KeyframeValue, Layer, Rect, ScrollNode, TextureRef, Tile,
    TimingFunction, Transform, TransformNode,
};
use hg_compositor_frame::validate;
use hg_ipc::BoundedVec;

fn bv<T, const N: usize>(v: Vec<T>) -> BoundedVec<T, N> {
    BoundedVec::try_new(v).unwrap()
}

fn identity() -> Transform {
    Transform {
        a: 1 << 32,
        b: 0,
        c: 0,
        d: 1 << 32,
        tx: 0,
        ty: 0,
    }
}

fn rect(x: i32, y: i32, width: i32, height: i32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn corners(r: i32) -> Corners {
    Corners {
        top_left_x: r,
        top_left_y: r,
        top_right_x: r,
        top_right_y: r,
        bottom_right_x: r,
        bottom_right_y: r,
        bottom_left_x: r,
        bottom_left_y: r,
    }
}

fn texture(id: u64) -> TextureRef {
    TextureRef {
        id,
        generation: 1,
        width: 256,
        height: 256,
    }
}

fn layer() -> Layer {
    Layer {
        origin_x: 0,
        origin_y: 0,
        tile_width: 256,
        tile_height: 256,
        columns: 2,
        rows: 2,
        transform: 0,
        clip: 0,
        effect: 0,
        scroll: NO_NODE,
        is_opaque: true,
        tiles: bv(vec![
            Tile {
                column: 0,
                row: 0,
                texture: texture(1),
            },
            Tile {
                column: 1,
                row: 0,
                texture: texture(2),
            },
        ]),
    }
}

fn animation(target: u32) -> Animation {
    Animation {
        id: 7,
        target,
        property: AnimatedProperty::Opacity,
        start_time_us: 1_000,
        duration_us: 250_000,
        bounds: rect(0, 0, 512, 512),
        keyframes: bv(vec![
            Keyframe {
                offset: 0,
                value: KeyframeValue::Opacity { opacity: 0 },
                easing: TimingFunction::Linear,
            },
            Keyframe {
                offset: 65_536,
                value: KeyframeValue::Opacity { opacity: 255 },
                easing: TimingFunction::CubicBezier {
                    x1: 16_384,
                    y1: 0,
                    x2: 49_152,
                    y2: 65_536,
                },
            },
        ]),
    }
}

/// A frame that says none of the fifteen things.
fn valid() -> Frame {
    Frame {
        frame_id: 1,
        fence: 42,
        device_scale_num: 3,
        device_scale_den: 2,
        viewport: rect(0, 0, 1024, 768),
        transforms: bv(vec![
            TransformNode {
                parent: NO_PARENT,
                matrix: identity(),
            },
            TransformNode {
                parent: 0,
                matrix: identity(),
            },
        ]),
        clips: bv(vec![ClipNode {
            parent: NO_PARENT,
            transform: 0,
            shape: ClipShape::Rounded {
                bounds: rect(0, 0, 100, 100),
                radii: corners(8 * 256),
            },
        }]),
        effects: bv(vec![EffectNode {
            parent: NO_PARENT,
            opacity: 200,
        }]),
        scrolls: bv(vec![ScrollNode {
            parent: NO_PARENT,
            transform: 0,
            container: rect(0, 0, 1024, 768),
            content: rect(0, 0, 1024, 4096),
            offset_x: 0,
            offset_y: 120,
            user_scrollable_x: false,
            user_scrollable_y: true,
        }]),
        layers: bv(vec![layer()]),
        embeds: bv(vec![Embed {
            surface: 11,
            generation: 1,
            transform: 1,
            clip: 0,
            bounds: rect(0, 0, 320, 240),
        }]),
        animations: bv(vec![animation(0)]),
        damage: bv(vec![rect(0, 0, 64, 64)]),
    }
}

#[test]
fn cf02_a_frame_that_says_none_of_them_is_accepted() {
    assert_eq!(validate(&valid()), Ok(()));
}

#[test]
fn cf03_a_parent_outside_its_own_tree_is_refused() {
    let mut f = valid();
    f.transforms = bv(vec![TransformNode {
        parent: 9,
        matrix: identity(),
    }]);
    assert_eq!(validate(&f), Err(CompositorFrameError::ParentOutOfRange));
}

#[test]
fn cf04_a_parent_at_or_after_its_child_is_refused() {
    // A forest in topological order is what makes a cycle
    // unexpressible. Node 0 naming node 0 is the shortest cycle there
    // is, and it is refused for being not-earlier rather than for
    // being a cycle -- there is no cycle variant, by design.
    let mut f = valid();
    f.transforms = bv(vec![TransformNode {
        parent: 0,
        matrix: identity(),
    }]);
    assert_eq!(validate(&f), Err(CompositorFrameError::ParentNotEarlier));
}

#[test]
fn cf05_a_two_node_cycle_is_refused_by_the_same_rule() {
    let mut f = valid();
    f.transforms = bv(vec![
        TransformNode {
            parent: 1,
            matrix: identity(),
        },
        TransformNode {
            parent: 0,
            matrix: identity(),
        },
    ]);
    assert_eq!(validate(&f), Err(CompositorFrameError::ParentNotEarlier));
}

#[test]
fn cf06_a_layer_naming_a_node_that_does_not_exist_is_refused() {
    let mut f = valid();
    let mut l = layer();
    l.transform = 5;
    f.layers = bv(vec![l]);
    assert_eq!(validate(&f), Err(CompositorFrameError::NodeOutOfRange));
}

#[test]
fn cf07_an_embed_without_a_clip_is_refused() {
    // LY-15 holds against a compromised child only because the
    // parent's clip is enforced at the embed, so `NO_NODE` here is
    // not "no clip" but a missing node.
    let mut f = valid();
    f.embeds = bv(vec![Embed {
        surface: 11,
        generation: 1,
        transform: 1,
        clip: NO_NODE,
        bounds: rect(0, 0, 320, 240),
    }]);
    assert_eq!(validate(&f), Err(CompositorFrameError::NodeOutOfRange));
}

#[test]
fn cf08_more_mutable_nodes_than_the_cap_is_refused() {
    let mut f = valid();
    // 16 embeds is already the cap; with the animation and the
    // scrollable node it is over.
    let embeds: Vec<Embed> = (0_u64..16)
        .map(|i| Embed {
            surface: i,
            generation: 1,
            transform: 1,
            clip: 0,
            bounds: rect(0, 0, 10, 10),
        })
        .collect();
    f.embeds = bv(embeds);
    assert!(mutable_node_count(&f) > MAX_MUTABLE_NODES);
    assert_eq!(validate(&f), Err(CompositorFrameError::TooManyMutableNodes));
}

#[test]
fn cf09_a_negative_extent_is_refused() {
    let mut f = valid();
    f.viewport = rect(0, 0, -1, 768);
    assert_eq!(validate(&f), Err(CompositorFrameError::NegativeExtent));
}

#[test]
fn cf10_a_tile_outside_its_own_grid_is_refused() {
    let mut f = valid();
    let mut l = layer();
    l.tiles = bv(vec![Tile {
        column: 2,
        row: 0,
        texture: texture(1),
    }]);
    f.layers = bv(vec![l]);
    assert_eq!(validate(&f), Err(CompositorFrameError::TileOutsideGrid));
}

#[test]
fn cf11_two_tiles_in_one_cell_are_refused() {
    let mut f = valid();
    let mut l = layer();
    l.tiles = bv(vec![
        Tile {
            column: 1,
            row: 1,
            texture: texture(1),
        },
        Tile {
            column: 1,
            row: 1,
            texture: texture(2),
        },
    ]);
    f.layers = bv(vec![l]);
    assert_eq!(validate(&f), Err(CompositorFrameError::DuplicateTile));
}

#[test]
fn cf12_a_zero_tile_extent_is_refused() {
    let mut f = valid();
    let mut l = layer();
    l.tile_width = 0;
    f.layers = bv(vec![l]);
    assert_eq!(validate(&f), Err(CompositorFrameError::ZeroTileExtent));
}

#[test]
fn cf13_a_negative_radius_is_refused() {
    let mut f = valid();
    f.clips = bv(vec![ClipNode {
        parent: NO_PARENT,
        transform: 0,
        shape: ClipShape::Rounded {
            bounds: rect(0, 0, 100, 100),
            radii: corners(-1),
        },
    }]);
    assert_eq!(validate(&f), Err(CompositorFrameError::NegativeRadius));
}

#[test]
fn cf14_radii_wider_than_their_rectangle_are_refused() {
    // CSS would scale these. The compositor refuses them, because
    // scaling is a decision and every decision about page pixels
    // belongs to the content GPU process (6.4).
    let mut f = valid();
    f.clips = bv(vec![ClipNode {
        parent: NO_PARENT,
        transform: 0,
        shape: ClipShape::Rounded {
            bounds: rect(0, 0, 10, 10),
            radii: corners(9 * 256),
        },
    }]);
    assert_eq!(validate(&f), Err(CompositorFrameError::RadiiExceedBounds));
}

#[test]
fn cf15_a_bad_device_scale_is_refused() {
    let mut f = valid();
    f.device_scale_den = 0;
    assert_eq!(validate(&f), Err(CompositorFrameError::BadDeviceScale));
    let mut g = valid();
    g.device_scale_den = 65_537;
    assert_eq!(validate(&g), Err(CompositorFrameError::BadDeviceScale));
}

#[test]
fn cf16_damage_outside_the_viewport_is_refused() {
    let mut f = valid();
    f.damage = bv(vec![rect(1000, 700, 100, 100)]);
    assert_eq!(
        validate(&f),
        Err(CompositorFrameError::DamageOutsideViewport)
    );
}

#[test]
fn cf17_unordered_or_overrunning_keyframes_are_refused() {
    let mut f = valid();
    let mut a = animation(0);
    a.keyframes = bv(vec![
        Keyframe {
            offset: 100,
            value: KeyframeValue::Opacity { opacity: 0 },
            easing: TimingFunction::Linear,
        },
        Keyframe {
            offset: 100,
            value: KeyframeValue::Opacity { opacity: 255 },
            easing: TimingFunction::Linear,
        },
    ]);
    f.animations = bv(vec![a]);
    assert_eq!(validate(&f), Err(CompositorFrameError::KeyframesUnordered));

    let mut g = valid();
    let mut b = animation(0);
    b.keyframes = bv(vec![
        Keyframe {
            offset: 0,
            value: KeyframeValue::Opacity { opacity: 0 },
            easing: TimingFunction::Linear,
        },
        Keyframe {
            offset: 65_537,
            value: KeyframeValue::Opacity { opacity: 255 },
            easing: TimingFunction::Linear,
        },
    ]);
    g.animations = bv(vec![b]);
    assert_eq!(validate(&g), Err(CompositorFrameError::KeyframesUnordered));
}

#[test]
fn cf18_fewer_than_two_keyframes_is_refused() {
    let mut f = valid();
    let mut a = animation(0);
    a.keyframes = bv(vec![Keyframe {
        offset: 0,
        value: KeyframeValue::Opacity { opacity: 0 },
        easing: TimingFunction::Linear,
    }]);
    f.animations = bv(vec![a]);
    assert_eq!(validate(&f), Err(CompositorFrameError::TooFewKeyframes));
}

#[test]
fn cf19_a_steps_function_with_zero_steps_is_refused() {
    let mut f = valid();
    let mut a = animation(0);
    a.keyframes = bv(vec![
        Keyframe {
            offset: 0,
            value: KeyframeValue::Opacity { opacity: 0 },
            easing: TimingFunction::Steps {
                count: 0,
                jump_at_start: false,
                jump_at_end: true,
            },
        },
        Keyframe {
            offset: 65_536,
            value: KeyframeValue::Opacity { opacity: 255 },
            easing: TimingFunction::Linear,
        },
    ]);
    f.animations = bv(vec![a]);
    assert_eq!(validate(&f), Err(CompositorFrameError::ZeroSteps));
}

#[test]
fn cf26_an_animation_targeting_the_wrong_tree_is_refused() {
    // `valid()` has 2 transform nodes and 1 effect node. An Opacity
    // animation indexes the effects, so target 1 names nothing --
    // although it is a perfectly good *transform* index. Accepting it
    // would make the two index spaces interchangeable to a sender.
    let mut f = valid();
    let mut a = animation(1);
    a.property = AnimatedProperty::Opacity;
    f.animations = bv(vec![a]);
    assert_eq!(validate(&f), Err(CompositorFrameError::TargetWrongKind));

    // The twin: the same index as a Transform animation is fine,
    // which is what makes the refusal above about the tree and not
    // about the index.
    let mut g = valid();
    let mut b = animation(1);
    b.property = AnimatedProperty::Transform;
    b.keyframes = bv(vec![
        Keyframe {
            offset: 0,
            value: KeyframeValue::Transform { matrix: identity() },
            easing: TimingFunction::Linear,
        },
        Keyframe {
            offset: 65_536,
            value: KeyframeValue::Transform { matrix: identity() },
            easing: TimingFunction::Linear,
        },
    ]);
    g.animations = bv(vec![b]);
    assert_eq!(validate(&g), Ok(()));
}

#[test]
fn cf27_a_keyframe_of_the_wrong_property_is_refused() {
    let mut f = valid();
    let mut a = animation(0);
    a.property = AnimatedProperty::Opacity;
    a.keyframes = bv(vec![
        Keyframe {
            offset: 0,
            value: KeyframeValue::Opacity { opacity: 0 },
            easing: TimingFunction::Linear,
        },
        Keyframe {
            offset: 65_536,
            // A transform value in an opacity animation: the union
            // admits it, the animation does not.
            value: KeyframeValue::Transform { matrix: identity() },
            easing: TimingFunction::Linear,
        },
    ]);
    f.animations = bv(vec![a]);
    assert_eq!(
        validate(&f),
        Err(CompositorFrameError::KeyframeWrongProperty)
    );
}

#[test]
fn cf28_mutable_nodes_are_counted_per_tree_not_per_index() {
    // The defect this guards: keying a mutable node by its index
    // alone makes transform 0, effect 0 and scroll 0 one node, so a
    // sender could hold many more than MAX_MUTABLE_NODES while
    // appearing not to.
    //
    // `valid()` has an Opacity animation on effect 0, a scrollable
    // scroll node at index 0, and one embed. Keyed by index that is
    // {0} plus one embed = 2; keyed by (tree, index) it is
    // {(Effect,0), (Scroll,0)} plus one embed = 3.
    let f = valid();
    assert_eq!(mutable_node_count(&f), 3);

    // Two animations of the *same* node are still one node, so the
    // deduplication is real and not simply absent.
    let mut g = valid();
    g.animations = bv(vec![animation(0), animation(0)]);
    assert_eq!(mutable_node_count(&g), 3);
}

/// §6.19's threat row and GR-11 promised "generated readers with Kani
/// harnesses" for this format. It has none, and cannot: the generator
/// emits a validator harness **per interface message**, and this IDL
/// is a type library that declares no interface. GR-M0's review §3.8
/// corrected the row to put the harnesses on the transport message
/// that carries the buffer handle and length (§3.1's decision (a)),
/// and the payload's proof on the validator and the mutation suites.
///
/// That correction rests on a measurement, so it is measured here
/// rather than asserted in prose: if this IDL ever declares an
/// interface, the row's premise is false and this fails, which is the
/// point.
#[test]
fn cf29_the_format_declares_no_interface_so_no_harness_can_attach() {
    let idl = include_str!("../../idl/compositor_frame.hgidl");
    let interfaces = idl
        .lines()
        .filter(|l| l.trim_start().starts_with("interface "))
        .count();
    assert_eq!(
        interfaces, 0,
        "a type library declares no interface; an interface here means \
         the review's §3.8 correction and §6.19's row are both stale"
    );

    // And the generated reader therefore carries no Kani attribute.
    let generated = include_str!("../../src/generated/compositor_frame.rs");
    let harnesses = generated.matches("kani::proof").count();
    assert_eq!(
        harnesses, 0,
        "no interface, so hg-ipc-codegen emits no harness; a harness \
         here means the generator changed and §3.8 needs re-measuring"
    );
}
