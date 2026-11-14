//! Compositing and blending, in integers.

use hg_display_list::display_list::{BlendMode, Colour};
use hg_raster::blend::{blend_channel, div255, mul255, over, scale, unpremultiply};

fn c(red: u8, green: u8, blue: u8, alpha: u8) -> Colour {
    Colour {
        red,
        green,
        blue,
        alpha,
    }
}

/// The add-shift identity equals `round(x / 255)` for **every** input
/// a product of two bytes can give — not for a few spot values.
///
/// "Approximately ÷255" is how a rasterizer ends up one level off on
/// one device and not another, which is the whole of what GR-D3
/// forbids, so the check is exhaustive rather than sampled.
#[test]
fn ra01_div255_is_exact_over_its_whole_range() {
    for x in 0_u32..=65_535 {
        let want = (u64::from(x) * 2 + 255) / 510;
        assert_eq!(
            u64::from(div255(x)),
            want,
            "div255({x}) should be round({x}/255)"
        );
    }
    // And the product form, over every pair of bytes.
    for a in 0_u8..=255 {
        for b in 0_u8..=255 {
            let want = (u32::from(a) * u32::from(b) * 2 + 255) / 510;
            assert_eq!(u32::from(mul255(a, b)), want, "mul255({a}, {b})");
        }
    }
}

/// The two identities a compositor relies on: ÷255 of 255·x is x, and
/// a scale by 255 changes nothing.
#[test]
fn ra02_the_identities_hold_exactly() {
    for x in 0_u8..=255 {
        assert_eq!(mul255(x, 255), x, "255 is the identity");
        assert_eq!(mul255(x, 0), 0);
        assert_eq!(mul255(255, x), x);
    }
    let colour = c(10, 20, 30, 40);
    assert_eq!(scale(colour.clone(), 255), colour);
}

/// Source-over: opaque source replaces, transparent source does
/// nothing, and the result is still premultiplied.
#[test]
fn ra03_source_over() {
    let dst = c(20, 40, 60, 80);
    let opaque = c(255, 0, 0, 255);
    assert_eq!(over(opaque.clone(), dst.clone()), opaque);
    let clear = c(0, 0, 0, 0);
    assert_eq!(over(clear, dst.clone()), dst);

    // Half-covered red over the backdrop: 128 of the source, and
    // 127/255 of the backdrop.
    let half = c(128, 0, 0, 128);
    let got = over(half, dst);
    assert_eq!(got.alpha, 128 + mul255(80, 127));
    assert_eq!(got.red, 128 + mul255(20, 127));
    assert!(
        got.red <= got.alpha && got.green <= got.alpha && got.blue <= got.alpha,
        "still premultiplied: {got:?}"
    );
}

/// Compositing 1's separable formulas, at the values the standard's
/// own text pins down.
#[test]
fn ra04_the_separable_blend_modes() {
    // normal is the source.
    assert_eq!(blend_channel(BlendMode::Normal, 77, 200), Some(77));
    // multiply by white is the backdrop; by black is black.
    assert_eq!(blend_channel(BlendMode::Multiply, 255, 137), Some(137));
    assert_eq!(blend_channel(BlendMode::Multiply, 0, 137), Some(0));
    // screen with black is the backdrop; with white is white.
    assert_eq!(blend_channel(BlendMode::Screen, 0, 137), Some(137));
    assert_eq!(blend_channel(BlendMode::Screen, 255, 137), Some(255));
    // darken and lighten.
    assert_eq!(blend_channel(BlendMode::Darken, 10, 200), Some(10));
    assert_eq!(blend_channel(BlendMode::Lighten, 10, 200), Some(200));
    // difference and exclusion with black are the backdrop.
    assert_eq!(blend_channel(BlendMode::Difference, 0, 137), Some(137));
    assert_eq!(blend_channel(BlendMode::Exclusion, 0, 137), Some(137));
    // dodge and burn at their stated edges.
    assert_eq!(blend_channel(BlendMode::ColorDodge, 100, 0), Some(0));
    assert_eq!(blend_channel(BlendMode::ColorDodge, 255, 10), Some(255));
    assert_eq!(blend_channel(BlendMode::ColorBurn, 100, 255), Some(255));
    assert_eq!(blend_channel(BlendMode::ColorBurn, 0, 10), Some(0));
    // overlay is hard-light with the arguments swapped, by definition.
    for s in [0_u8, 1, 63, 127, 128, 200, 255] {
        for b in [0_u8, 1, 63, 127, 128, 200, 255] {
            assert_eq!(
                blend_channel(BlendMode::Overlay, s, b),
                blend_channel(BlendMode::HardLight, b, s),
                "overlay({s}, {b})"
            );
        }
    }
}

/// Every separable mode stays inside 0..=255 for every pair of
/// inputs, and every non-separable mode says it is not here.
#[test]
fn ra05_no_mode_leaves_the_range_and_none_is_silently_missing() {
    let separable = [
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::Darken,
        BlendMode::Lighten,
        BlendMode::ColorDodge,
        BlendMode::ColorBurn,
        BlendMode::HardLight,
        BlendMode::SoftLight,
        BlendMode::Difference,
        BlendMode::Exclusion,
    ];
    for mode in separable {
        for s in 0_u8..=255 {
            for b in [0_u8, 1, 64, 127, 128, 192, 254, 255] {
                assert!(
                    blend_channel(mode, s, b).is_some(),
                    "{mode:?} refused ({s}, {b})"
                );
            }
        }
    }
    for mode in [
        BlendMode::Hue,
        BlendMode::Saturation,
        BlendMode::Color,
        BlendMode::Luminosity,
    ] {
        assert_eq!(
            blend_channel(mode, 100, 100),
            None,
            "{mode:?} is non-separable and must say so rather than guess"
        );
    }
}

/// Un-premultiplying undoes premultiplying only as far as the alpha
/// allows, and the limit is a property of premultiplied 8-bit storage
/// rather than of this code.
///
/// At alpha 1 the stored colour has two representable values, so a
/// straight 64 premultiplies to 0 and comes back 0. Any formula that
/// "recovered" 64 from 0 would be inventing it. What the round trip
/// does promise is that the error is inside the quantum alpha leaves:
/// `255 / alpha`, rounded up.
///
/// This is worth a test rather than a comment because it is the
/// reason §6.5 makes premultiplied the *working* encoding and not the
/// authoring one, and the reason a blend mode un-premultiplies before
/// its formula and not after.
#[test]
fn ra06_unpremultiply_recovers_what_the_alpha_can_hold() {
    for alpha in 1_u8..=255 {
        let quantum = 255_u32.div_ceil(u32::from(alpha));
        for straight in [0_u8, 1, 64, 127, 200, 255] {
            let premultiplied = mul255(straight, alpha);
            let back = unpremultiply(premultiplied, alpha);
            assert!(
                u32::from(back.abs_diff(straight)) <= quantum,
                "alpha {alpha} holds {quantum} per level; straight {straight} \
                 came back {back}"
            );
        }
    }
    // At full alpha it is exact, which is the case that matters for
    // the blend modes' own formulas.
    for straight in 0_u8..=255 {
        assert_eq!(unpremultiply(mul255(straight, 255), 255), straight);
    }
    assert_eq!(unpremultiply(0, 0), 0, "no colour at zero alpha");
}
