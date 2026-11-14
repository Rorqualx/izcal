//! The wire: a chunk survives the crossing, and a declared count
//! above the format's own bound is refused before anything is read.

use hg_display_list::display_list::{
    Chunk, Colour, DisplayListError, FillRule, Item, Path, Point, Segment,
};
use hg_ipc::BoundedVec;
use hg_ipc::wire::{Wire, Writer, decode_exact, encode_to_vec, put_count, validate_exact};

fn sample() -> Chunk {
    Chunk {
        origin_x: -9_007_199_254_740_993,
        origin_y: 4_294_967_296,
        items: BoundedVec::try_new(vec![Item::FillPath {
            path: Path {
                start: Point {
                    x: -8_388_608,
                    y: 8_388_607,
                },
                segments: BoundedVec::try_new(vec![
                    Segment::Quad {
                        control: Point { x: 1, y: 2 },
                        to: Point { x: 3, y: 4 },
                    },
                    Segment::Cubic {
                        control0: Point { x: 5, y: 6 },
                        control1: Point { x: 7, y: 8 },
                        to: Point { x: 9, y: 10 },
                    },
                    Segment::Arc {
                        radii: Point { x: 11, y: 12 },
                        rotation: -45 * 256,
                        large: true,
                        sweep: false,
                        to: Point { x: 13, y: 14 },
                    },
                ])
                .unwrap(),
                rule: FillRule::EvenOdd,
            },
            colour: Colour {
                red: 1,
                green: 2,
                blue: 3,
                alpha: 255,
            },
        }])
        .unwrap(),
    }
}

/// Every field survives, including the i64 origin and the booleans
/// inside an arc, which is where a wire format usually loses a bit.
#[test]
fn dl08_a_chunk_round_trips() {
    let c = sample();
    let bytes = encode_to_vec(&c);
    assert_eq!(validate_exact::<Chunk>(&bytes), Ok(()));
    assert_eq!(decode_exact::<Chunk>(&bytes), Ok(c));
}

/// A count above the bound is refused by the reader, not by `check`:
/// the format has a word for a count, so the limit is enforced where
/// the count is read and no memory is reserved for it.
#[test]
fn dl09_a_declared_count_above_the_bound_is_refused() {
    let mut w = Writer::new();
    0_i64.encode(&mut w);
    0_i64.encode(&mut w);
    put_count(&mut w, 4_097);
    let bytes = w.into_bytes();
    assert!(decode_exact::<Chunk>(&bytes).is_err(), "4,097 items");
    assert!(validate_exact::<Chunk>(&bytes).is_err(), "4,097 items");

    // The twin: the same message declaring the bound exactly, with the
    // items to match, is accepted — so the refusal above is about the
    // count and not about the shape of the message.
    let item = Item::PopGroup;
    let ok = Chunk {
        origin_x: 0,
        origin_y: 0,
        items: BoundedVec::try_new(vec![item; 4_096]).unwrap(),
    };
    let ok_bytes = encode_to_vec(&ok);
    assert_eq!(decode_exact::<Chunk>(&ok_bytes), Ok(ok));
}

/// The error type crosses the boundary too, and every variant is a
/// limit the format names (CS-ER-6: no free text).
#[test]
fn dl10_every_refusal_round_trips() {
    for e in [
        DisplayListError::TooManySegments,
        DisplayListError::TooManyItems,
        DisplayListError::NotPremultiplied,
        DisplayListError::NegativeExtent,
        DisplayListError::UnbalancedGroup,
        DisplayListError::GroupTooDeep,
        DisplayListError::SourceOutsideImage,
        DisplayListError::NegativeRadius,
    ] {
        assert_eq!(decode_exact::<DisplayListError>(&encode_to_vec(&e)), Ok(e));
    }
    // An unknown discriminant is refused rather than mapped to one of
    // them: a renderer that invents a reason is a renderer the GPU
    // process has no answer for.
    let mut w = Writer::new();
    8_u32.encode(&mut w);
    assert!(decode_exact::<DisplayListError>(&w.into_bytes()).is_err());
}
