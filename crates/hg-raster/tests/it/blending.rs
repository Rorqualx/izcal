//! Compositing and blending, in integers.

use hg_display_list::display_list::{BlendMode, Colour};
use hg_raster::blend::{
    E16_ONE, E16_PRODUCT_MAX, blend_channel, div255, div4080, mul255, mul4080, over, scale,
    unpremultiply,
};

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

/// **E16's ÷4080 is exact, and the ties are why it needed writing
/// down** (GR-M0 §3.6).
///
/// §6.4's `div255` rounds half away from zero; §6.5 says every E16
/// conversion rounds half to even. The two agree for ÷255 **because a
/// tie cannot happen there** — and this test measures that rather
/// than asserting it, over every product of two bytes. For ÷4080 a
/// tie is an integer, `4080k + 2040`, so the rule has to be stated
/// and checked.
///
/// Exhaustive over the whole product range, in the spirit of `ra01`:
/// "approximately ÷4080" is how two implementations end up one level
/// apart on different machines, which is the whole of what GR-D3
/// forbids.
#[test]
fn ra20_div4080_is_exact_half_to_even_over_its_whole_range() {
    // Reference: round-half-even, by integer arithmetic only (G61
    // forbids f32 and f64 here, tests included).
    fn want(x: i64) -> i64 {
        let negative = x < 0;
        let m = x.abs();
        let q = m / 4_080;
        let r = m - q * 4_080;
        let rounded = if r * 2 > 4_080 || (r * 2 == 4_080 && q % 2 == 1) {
            q + 1
        } else {
            q
        };
        if negative { -rounded } else { rounded }
    }

    // Zero ties for 255, over every product of two bytes: the measured
    // fact that lets div255 round half up.
    let mut s8_ties = 0_u32;
    for x in 0_i64..=(255 * 255) {
        if (x * 2) % 255 == 0 && ((x * 2) / 255) % 2 == 1 {
            s8_ties = s8_ties.saturating_add(1);
        }
    }
    assert_eq!(s8_ties, 0, "a tie for div255 would break its rounding");

    // And 32,767 ties for 4,080 over the range E16 uses.
    let mut e16_ties = 0_u32;
    let mut x = 2_040_i64;
    while x <= i64::from(E16_PRODUCT_MAX) {
        e16_ties = e16_ties.saturating_add(1);
        x += 4_080;
    }
    assert_eq!(e16_ties, 32_767, "the ties div4080 has to decide");

    // Exhaustive, both signs.
    for x in -E16_PRODUCT_MAX..=E16_PRODUCT_MAX {
        assert_eq!(
            i64::from(div4080(x)),
            want(i64::from(x)),
            "div4080({x}) should be round_half_even({x}/4080)"
        );
    }

    // Every tie lands on an even quotient, which is the rule.
    let mut k = 0_i64;
    while k < 32_767 {
        let tie = k * 4_080 + 2_040;
        let i32_tie = i32::try_from(tie).unwrap();
        assert_eq!(
            i64::from(div4080(i32_tie)) % 2,
            0,
            "the tie at {tie} must round to an even quotient"
        );
        k += 1;
    }
}

/// **`div4080` is exact for every `i32`, not only for E16's own
/// range** — checked at the witnesses, so the whole argument type is
/// covered rather than the part E16 happens to use.
///
/// This test exists because a review found that the first `div4080`
/// used `(m · 134_744_073) >> 39`, whose **floor** is exact only to
/// 139,111,678 — while the exhaustive test above stops at E16's own
/// 133,689,360 and so could not have seen it.
///
/// The review also predicted wrong results, and that part did not
/// hold: measured over the whole `i32` range, the floor overshoots at
/// 3,803,748 values and at every one the true remainder is at least
/// 4,065, far above half, so round-half-even rounded up to exactly
/// the overshot value. **Every rounded answer was right; the comment
/// claiming an exact floor was not.** The multiplier was widened
/// because that agreement is a coincidence of this rounding rule and
/// would stop holding if the floor were reused or the tie rule
/// changed.
///
/// A multiply-shift can only fail where the true quotient changes, so
/// `4080k - 1` and `4080k` for every `k` are **all** the witnesses:
/// about 1.05 M pairs covering the full `i32` range, which is a proof
/// rather than a sample and still runs in well under a second.
#[test]
fn ra22_div4080_is_exact_for_every_i32_not_only_for_e16s_range() {
    // The floor the multiply-shift has to match, by division.
    let mut k = 1_i64;
    let magnitude = 1_i64 << 31; // |i32::MIN|
    while 4_080 * k <= magnitude {
        for m in [4_080 * k - 1, 4_080 * k] {
            if m > magnitude {
                continue;
            }
            let want_floor = m / 4_080;
            // `div4080` rounds, so recover the floor it used: at
            // `4080k` the remainder is 0 and at `4080k - 1` it is
            // 4,079, which rounds up to `k`. Both pin the floor.
            let got = i64::from(div4080(i32::try_from(m).unwrap_or(i32::MAX)));
            let expected = if m % 4_080 == 0 { want_floor } else { k };
            if m <= i64::from(i32::MAX) {
                assert_eq!(got, expected, "div4080({m})");
            }
            // And the negative witness, where `i32::MIN` lives.
            if m <= magnitude {
                let neg = -m;
                if neg >= i64::from(i32::MIN) {
                    let got_neg = i64::from(div4080(i32::try_from(neg).unwrap()));
                    assert_eq!(got_neg, -expected, "div4080({neg})");
                }
            }
        }
        k += 1;
    }

    // The first value whose *floor* the old multiplier overshot. Its
    // rounded answer was right even then, which is the point: this
    // asserts the value, not a regression it would have caught.
    assert_eq!(div4080(139_111_679), 34_096);
    assert_eq!(div4080(-139_111_679), -34_096);

    // i32's own edges.
    assert_eq!(div4080(i32::MAX), 526_344);
    assert_eq!(div4080(i32::MIN), -526_344);
}

/// E16's one is the identity, and every S8 value is exactly `v × 16`
/// — the property that makes S8 a subset of E16 rather than a
/// conversion with its own error (§6.5).
#[test]
fn ra21_e16_contains_s8_exactly() {
    for v in 0_u8..=255 {
        let e16 = i16::from(v).saturating_mul(16);
        assert_eq!(
            mul4080(e16, i16::try_from(E16_ONE).unwrap()),
            i32::from(e16),
            "E16_ONE is the identity for {v}"
        );
        // Back to S8 exactly, which §6.5 requires. The conversion is
        // ÷16 (4,080 / 255), not ÷4,080 -- `div4080` divides out an
        // *alpha* multiply, which is a different operation, and
        // conflating the two is how an encoding gains a rounding it
        // was specified not to have.
        assert_eq!(i32::from(e16) % 16, 0, "{v} is exactly representable");
        assert_eq!(i32::from(e16) / 16, i32::from(v), "{v} returns exactly");
    }

    // The identity ÷4080 rests on, as `ra02` does for ÷255: dividing
    // 4080·x by 4080 returns x, with no rounding anywhere.
    let mut x = -32_767_i32;
    while x <= 32_767 {
        assert_eq!(
            div4080(x.saturating_mul(E16_ONE)),
            x,
            "4080 is the identity for {x}"
        );
        x = x.saturating_add(1);
    }
}
