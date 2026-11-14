//! Image sampling in integers.

use hg_display_list::display_list::{Colour, Sampling};
use hg_raster::image::Image;

/// A 2×2 image: black, white / white, black, opaque.
fn checker() -> Vec<u8> {
    vec![
        0, 0, 0, 255, 255, 255, 255, 255, // row 0
        255, 255, 255, 255, 0, 0, 0, 255, // row 1
    ]
}

fn image(texels: &[u8]) -> Image<'_> {
    Image {
        width: 2,
        height: 2,
        texels,
    }
}

/// At a texel's centre, bilinear returns that texel exactly. An image
/// drawn at its own size must be its own pixels, or every page with a
/// 1:1 image is subtly blurred.
#[test]
fn ra20_bilinear_at_a_texel_centre_is_that_texel() {
    let t = checker();
    let img = image(&t);
    for (x, y, want) in [
        (128_i64, 128_i64, 0_u8),
        (384, 128, 255),
        (128, 384, 255),
        (384, 384, 0),
    ] {
        let c = img.bilinear(x, y);
        assert_eq!(c.red, want, "({x}, {y})");
        assert_eq!(c.alpha, 255);
    }
}

/// Halfway between two texels is their mean, by one rounding.
#[test]
fn ra21_bilinear_halfway_is_the_mean() {
    let t = checker();
    let img = image(&t);
    // Halfway along the top row, between black and white.
    let c = img.bilinear(256, 128);
    assert_eq!(c.red, 128, "round(255/2)");
    // The centre of the checker: two black and two white.
    let c = img.bilinear(256, 256);
    assert_eq!(c.red, 128);
    assert_eq!(c.alpha, 255);
}

/// `pixelated` takes the nearest texel and never mixes, which is what
/// the property is for.
#[test]
fn ra22_pixelated_never_mixes() {
    let t = checker();
    let img = image(&t);
    for x in 0..512_i64 {
        for y in 0..512_i64 {
            let c = img.sample(x, y, Sampling::Pixelated, false);
            assert!(c.red == 0 || c.red == 255, "({x}, {y}) gave {}", c.red);
        }
    }
}

/// Exact-area minification of the whole checker is its mean, exactly.
#[test]
fn ra23_minification_is_the_exact_area_mean() {
    let t = checker();
    let img = image(&t);
    let c = img.area(0, 0, 512, 512);
    assert_eq!(
        c,
        Colour {
            red: 128,
            green: 128,
            blue: 128,
            alpha: 255
        },
        "two black and two white texels, weighted equally"
    );
    // A box over one texel is that texel.
    assert_eq!(img.area(0, 0, 256, 256).red, 0);
    assert_eq!(img.area(256, 0, 512, 256).red, 255);
}

/// Sampling outside the image clamps to the edge rather than wrapping
/// or reading nothing.
#[test]
fn ra24_sampling_outside_clamps() {
    let t = checker();
    let img = image(&t);
    assert_eq!(img.texel(-5, -5), img.texel(0, 0));
    assert_eq!(img.texel(100, 100), img.texel(1, 1));
    let empty: Vec<u8> = Vec::new();
    let none = Image {
        width: 0,
        height: 0,
        texels: &empty,
    };
    assert_eq!(none.texel(0, 0).alpha, 0, "an empty image is transparent");
}
