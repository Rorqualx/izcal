//! The four things the wire format cannot say (`check`).

use hg_display_list::check::{MAX_GROUP_DEPTH, has_extent, is_premultiplied, validate};
use hg_display_list::display_list::{
    BlendMode, Chunk, Colour, DisplayListError, FillRule, ImageRef, Item, Path, Point, Rect,
    Sampling, Segment,
};
use hg_ipc::BoundedVec;

fn chunk(items: Vec<Item>) -> Chunk {
    Chunk {
        origin_x: 0,
        origin_y: 0,
        items: BoundedVec::try_new(items).unwrap(),
    }
}

fn square(n: i32) -> Path {
    Path {
        start: Point { x: 0, y: 0 },
        segments: BoundedVec::try_new(vec![
            Segment::Line {
                to: Point { x: n, y: 0 },
            },
            Segment::Line {
                to: Point { x: n, y: n },
            },
            Segment::Line {
                to: Point { x: 0, y: n },
            },
        ])
        .unwrap(),
        rule: FillRule::NonZero,
    }
}

fn opaque(red: u8, green: u8, blue: u8) -> Colour {
    Colour {
        red,
        green,
        blue,
        alpha: 255,
    }
}

fn fill(colour: Colour) -> Item {
    Item::FillPath {
        path: square(256),
        colour,
    }
}

fn image(source: Rect, dest: Rect) -> Item {
    Item::DrawImage {
        image: ImageRef {
            id: 1,
            width: 16,
            height: 16,
        },
        source,
        dest,
        sampling: Sampling::Auto,
    }
}

fn group() -> Item {
    Item::PushGroup {
        blend: BlendMode::Multiply,
        opacity: 128,
    }
}

/// An ordinary list is accepted, which is the twin every refusal
/// below needs: a validator that refuses everything proves nothing.
#[test]
fn dl02_an_ordinary_chunk_validates() {
    let c = chunk(vec![
        fill(opaque(10, 20, 30)),
        group(),
        fill(opaque(0, 0, 0)),
        Item::PopGroup,
        image(
            Rect {
                x: 0,
                y: 0,
                width: 16,
                height: 16,
            },
            Rect {
                x: 4,
                y: 4,
                width: 32,
                height: 32,
            },
        ),
    ]);
    assert_eq!(validate(&c), Ok(()));
    // And the empty one, which a renderer sends for a blank layer.
    assert_eq!(validate(&chunk(Vec::new())), Ok(()));
}

/// A component above its alpha is not a premultiplied colour.
#[test]
fn dl03_a_colour_above_its_alpha_is_refused() {
    for bad in [
        Colour {
            red: 200,
            green: 0,
            blue: 0,
            alpha: 128,
        },
        Colour {
            red: 0,
            green: 129,
            blue: 0,
            alpha: 128,
        },
        Colour {
            red: 0,
            green: 0,
            blue: 1,
            alpha: 0,
        },
    ] {
        assert!(!is_premultiplied(&bad), "{bad:?}");
        assert_eq!(
            validate(&chunk(vec![fill(bad.clone())])),
            Err(DisplayListError::NotPremultiplied),
            "{bad:?}"
        );
    }
    // At the alpha exactly, which is the fully saturated colour, not
    // an error: the boundary is the common place to get this wrong.
    let edge = Colour {
        red: 128,
        green: 128,
        blue: 128,
        alpha: 128,
    };
    assert!(is_premultiplied(&edge));
    assert_eq!(validate(&chunk(vec![fill(edge.clone())])), Ok(()));
    // Fully transparent black, the other boundary.
    let clear = Colour {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0,
    };
    assert!(is_premultiplied(&clear));
    assert_eq!(validate(&chunk(vec![fill(clear.clone())])), Ok(()));
}

/// A negative width or height, in either rectangle.
#[test]
fn dl04_a_negative_extent_is_refused() {
    let good = Rect {
        x: 0,
        y: 0,
        width: 16,
        height: 16,
    };
    assert!(has_extent(&good));
    for bad in [
        Rect {
            width: -1,
            ..good.clone()
        },
        Rect {
            height: -1,
            ..good.clone()
        },
    ] {
        assert!(!has_extent(&bad), "{bad:?}");
        assert_eq!(
            validate(&chunk(vec![image(bad.clone(), good.clone())])),
            Err(DisplayListError::NegativeExtent),
            "source {bad:?}"
        );
        assert_eq!(
            validate(&chunk(vec![image(good.clone(), bad.clone())])),
            Err(DisplayListError::NegativeExtent),
            "dest {bad:?}"
        );
    }
    // Zero is an extent, not a negative one: an empty rectangle draws
    // nothing, which is a thing a renderer is entitled to ask for.
    let empty = Rect {
        width: 0,
        height: 0,
        ..good
    };
    assert!(has_extent(&empty));
    assert_eq!(
        validate(&chunk(vec![image(empty.clone(), empty.clone())])),
        Ok(())
    );
}

/// A pop with nothing to pop, and a chunk that ends inside a group.
#[test]
fn dl05_groups_must_balance() {
    assert_eq!(
        validate(&chunk(vec![Item::PopGroup])),
        Err(DisplayListError::UnbalancedGroup)
    );
    assert_eq!(
        validate(&chunk(vec![group()])),
        Err(DisplayListError::UnbalancedGroup)
    );
    assert_eq!(
        validate(&chunk(vec![group(), Item::PopGroup, Item::PopGroup])),
        Err(DisplayListError::UnbalancedGroup)
    );
    assert_eq!(
        validate(&chunk(vec![group(), group(), Item::PopGroup])),
        Err(DisplayListError::UnbalancedGroup)
    );
    assert_eq!(
        validate(&chunk(vec![
            group(),
            Item::PopGroup,
            group(),
            Item::PopGroup
        ])),
        Ok(())
    );
}

/// Nesting is bounded, because each open group is a buffer the
/// compositor holds. At the depth, and one past it.
#[test]
fn dl06_group_nesting_is_bounded() {
    let mut at: Vec<Item> = (0..MAX_GROUP_DEPTH).map(|_| group()).collect();
    at.extend((0..MAX_GROUP_DEPTH).map(|_| Item::PopGroup));
    assert_eq!(validate(&chunk(at)), Ok(()));

    let over = MAX_GROUP_DEPTH.checked_add(1).unwrap();
    let mut past: Vec<Item> = (0..over).map(|_| group()).collect();
    past.extend((0..over).map(|_| Item::PopGroup));
    assert_eq!(
        validate(&chunk(past)),
        Err(DisplayListError::TooManyItems),
        "one deeper than {MAX_GROUP_DEPTH}"
    );
}

/// The two counts §6.3 fixes, at the limit and one past it. These are
/// the format's own bounds, so the refusal is a type error rather
/// than a validation one: the over-long value cannot be built.
#[test]
fn dl07_the_counts_are_limits_at_the_limit() {
    let line = Segment::Line {
        to: Point { x: 1, y: 1 },
    };
    assert!(BoundedVec::<Segment, 65_536>::try_new(vec![line.clone(); 65_536]).is_ok());
    assert!(BoundedVec::<Segment, 65_536>::try_new(vec![line; 65_537]).is_err());

    let item = fill(opaque(0, 0, 0));
    assert!(BoundedVec::<Item, 4_096>::try_new(vec![item.clone(); 4_096]).is_ok());
    assert!(BoundedVec::<Item, 4_096>::try_new(vec![item; 4_097]).is_err());
}
