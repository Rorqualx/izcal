//! `hg-raster` against the exact rasterizer.
//!
//! This is what GR-M0 asks the pair of crates for. The two share no
//! algorithm and no arithmetic — signed-area accumulation in `i64`
//! against polygon clipping over exact rationals — so an agreement
//! between them is evidence and not a tautology.
//!
//! The production path carries **one specified rounding** that the
//! reference does not: the x where an edge crosses a horizontal pixel
//! boundary. So the two are not expected to agree everywhere, and the
//! test records *how far apart they are* rather than asserting they
//! are identical. Both numbers are baselines: they can go down and
//! not up.

#![expect(
    clippy::arithmetic_side_effects,
    reason = "the shapes are written as arithmetic on small constants, which is what makes them readable as shapes"
)]

use hg_display_list::display_list::{FillRule, Point};
use hg_raster::cover::Coverage;
use hg_raster::flatten::SUB;
use hg_raster_ref::{check, coverage};

fn p(x: i64, y: i64) -> Point {
    Point {
        x: i32::try_from(x).expect("in range"),
        y: i32::try_from(y).expect("in range"),
    }
}

/// The largest difference, in 8-bit levels, between the production
/// rasterizer and the exact one over every case below.
///
/// **1**, measured 13 November 2026. One level is the most a single
/// rounding of a crossing can move a pixel, so this is the number the
/// method's own design predicts — and a change that made it 2 would
/// mean a second rounding had crept in.
pub const WORST_BASELINE: u8 = 1;

/// How many pixels differ at all, over every case below, out of the
/// 720 compared.
///
/// **18**, measured 13 November 2026, and every one of them by a
/// single level: 0 on the two rectangles and the sliver, 4 on the
/// triangle, 12 on the diamond, 2 on the near-horizontal wedge. A
/// number that can only go down.
pub const DIFFERING_BASELINE: usize = 18;

struct Case {
    name: &'static str,
    points: Vec<Point>,
}

fn cases() -> Vec<Case> {
    let s = SUB;
    vec![
        Case {
            name: "aligned rectangle",
            points: vec![
                p(2 * s, 2 * s),
                p(10 * s, 2 * s),
                p(10 * s, 8 * s),
                p(2 * s, 8 * s),
            ],
        },
        Case {
            name: "offset rectangle",
            points: vec![
                p(2 * s + 37, 2 * s + 91),
                p(10 * s + 37, 2 * s + 91),
                p(10 * s + 37, 8 * s + 91),
                p(2 * s + 37, 8 * s + 91),
            ],
        },
        Case {
            name: "triangle",
            points: vec![p(s, s), p(11 * s, 3 * s), p(4 * s, 9 * s)],
        },
        Case {
            name: "thin sliver",
            points: vec![p(s, s), p(11 * s, s + 40), p(11 * s, s + 80), p(s, s + 20)],
        },
        Case {
            name: "diamond",
            points: vec![p(6 * s, s), p(11 * s, 5 * s), p(6 * s, 9 * s), p(s, 5 * s)],
        },
        Case {
            name: "almost horizontal wedge",
            points: vec![
                p(0, 3 * s),
                p(12 * s, 3 * s + 5),
                p(12 * s, 6 * s),
                p(0, 6 * s),
            ],
        },
    ]
}

/// Every case, both ways, with the difference recorded.
#[test]
fn rr10_the_production_rasterizer_agrees_with_the_exact_one() {
    let (w, h) = (12_usize, 10_usize);
    let mut worst = 0_u8;
    let mut differing = 0_usize;
    for case in cases() {
        let mut cov = Coverage::new(w, h);
        cov.add_polygon(&case.points);
        let got = cov.resolve(FillRule::NonZero);
        let want = coverage(&case.points, w, h).expect("the exact one stays in range");
        let d = check(&got, &want);
        println!(
            "hg-raster-ref: {}: {} of {} pixels differ, worst {} at {:?}",
            case.name,
            d.pixels,
            w * h,
            d.worst,
            d.at
        );
        assert!(
            d.worst <= WORST_BASELINE,
            "{}: worst difference {} is above the baseline {WORST_BASELINE}",
            case.name,
            d.worst
        );
        worst = worst.max(d.worst);
        differing = differing.saturating_add(d.pixels);
    }
    println!(
        "hg-raster-ref: {differing} of {} pixels differ over {} cases, worst {worst}",
        w * h * cases().len(),
        cases().len()
    );
    assert!(
        differing <= DIFFERING_BASELINE,
        "{differing} pixels differ, baseline {DIFFERING_BASELINE}"
    );
}

/// Where the production path has no rounding to make — an edge on a
/// pixel boundary — the two agree **exactly**. This is the half of
/// the comparison that is not "within one", and it is what says the
/// difference above is the crossing rounding and not something else.
#[test]
fn rr11_axis_aligned_shapes_agree_exactly() {
    let s = SUB;
    let (w, h) = (8_usize, 8_usize);
    for (name, points) in [
        (
            "whole pixels",
            vec![p(s, s), p(6 * s, s), p(6 * s, 5 * s), p(s, 5 * s)],
        ),
        (
            "half a pixel wide",
            vec![p(s / 2, 0), p(s, 0), p(s, 8 * s), p(s / 2, 8 * s)],
        ),
        (
            "a third of a pixel tall",
            vec![p(0, 0), p(8 * s, 0), p(8 * s, s / 3), p(0, s / 3)],
        ),
    ] {
        let mut cov = Coverage::new(w, h);
        cov.add_polygon(&points);
        let got = cov.resolve(FillRule::NonZero);
        let want = coverage(&points, w, h).expect("in range");
        let d = check(&got, &want);
        assert_eq!(
            d.pixels, 0,
            "{name}: {} pixels differ, worst {} at {:?}",
            d.pixels, d.worst, d.at
        );
    }
}

/// The output is the same bytes whatever the machine, which is
/// GR-D3's own claim and the reason there is no floating point in
/// either crate. The hash is of the production rasterizer's mask over
/// every case, and it is a constant because integer arithmetic is.
#[test]
fn rr12_the_output_is_one_value_on_every_machine() {
    let (w, h) = (12_usize, 10_usize);
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for case in cases() {
        let mut cov = Coverage::new(w, h);
        cov.add_polygon(&case.points);
        for b in cov.resolve(FillRule::NonZero).bytes() {
            hash ^= u64::from(*b);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    println!("hg-raster: canonical hash {hash:#018x}");
    assert_eq!(
        hash, EXPECTED_HASH,
        "the raster changed; if that was intended, record the new hash with the date"
    );
}

/// The FNV-1a hash of every case's mask, measured 13 November 2026.
/// The same value on x86-64 and AArch64, which is what GR-D3 asks of
/// the CPU path.
const EXPECTED_HASH: u64 = 0xb004_7df6_484f_657c;
