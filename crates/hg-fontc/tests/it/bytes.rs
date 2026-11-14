//! What the writer puts on disk, checked against bytes written out by
//! hand from the OpenType prose.
//!
//! These are the tests that a round-trip cannot replace. Writer and
//! reader are generated from one description, so they agree with each
//! other whatever that description says: a field in the wrong order, or
//! a width off by two, round-trips perfectly and produces a font no
//! other reader accepts. That is the common-mode risk the JavaScript
//! spec's 6.3 names for differential testing, and the only cure is an
//! expectation that does not come from the description — here, the
//! field order and widths read off the specification's own tables.

use hg_fontc::writers::{HeadWriter, HheaWriter, MaxpHalfWriter};

/// hhea is eighteen 16-bit fields in the order the specification's
/// table gives them, so the whole table is 36 bytes.
#[test]
fn fc01_hhea_is_thirty_six_bytes_in_the_prose_s_order() {
    let mut w = HheaWriter::new();
    w.set_major_version(1)
        .set_minor_version(0)
        .set_ascender(1900)
        .set_descender(-500)
        .set_line_gap(0)
        .set_advance_width_max(2048)
        .set_min_left_side_bearing(-200)
        .set_min_right_side_bearing(-150)
        .set_x_max_extent(2000)
        .set_caret_slope_rise(1)
        .set_caret_slope_run(0)
        .set_caret_offset(0)
        .set_reserved0(0)
        .set_reserved1(0)
        .set_reserved2(0)
        .set_reserved3(0)
        .set_metric_data_format(0)
        .set_number_of_h_metrics(3);
    let got = w.finish().expect("every field is set");

    let mut want = Vec::new();
    want.extend_from_slice(&1_u16.to_be_bytes()); // majorVersion
    want.extend_from_slice(&0_u16.to_be_bytes()); // minorVersion
    want.extend_from_slice(&1900_i16.to_be_bytes()); // ascender
    want.extend_from_slice(&(-500_i16).to_be_bytes()); // descender
    want.extend_from_slice(&0_i16.to_be_bytes()); // lineGap
    want.extend_from_slice(&2048_u16.to_be_bytes()); // advanceWidthMax
    want.extend_from_slice(&(-200_i16).to_be_bytes()); // minLeftSideBearing
    want.extend_from_slice(&(-150_i16).to_be_bytes()); // minRightSideBearing
    want.extend_from_slice(&2000_i16.to_be_bytes()); // xMaxExtent
    want.extend_from_slice(&1_i16.to_be_bytes()); // caretSlopeRise
    want.extend_from_slice(&0_i16.to_be_bytes()); // caretSlopeRun
    want.extend_from_slice(&0_i16.to_be_bytes()); // caretOffset
    want.extend_from_slice(&[0; 8]); // four reserved int16
    want.extend_from_slice(&0_i16.to_be_bytes()); // metricDataFormat
    want.extend_from_slice(&3_u16.to_be_bytes()); // numberOfHMetrics

    assert_eq!(want.len(), 36, "the prose's hhea is 36 bytes");
    assert_eq!(got, want);
}

/// head is 54 bytes, and the two LONGDATETIME fields are the only
/// 64-bit ones: a writer that treated them as 32-bit would be 8 bytes
/// short and every field after `modified` would move.
#[test]
fn fc02_head_is_fifty_four_bytes_and_its_dates_are_sixty_four_bit() {
    let mut w = HeadWriter::new();
    w.set_major_version(1)
        .set_minor_version(0)
        .set_font_revision(0x0001_0000)
        .set_checksum_adjustment(0xB1B0_AFBA)
        .set_magic_number(0x5F0F_3CF5)
        .set_flags(0b11)
        .set_units_per_em(1000)
        .set_created(3_600_000_000)
        .set_modified(3_600_000_001)
        .set_x_min(-100)
        .set_y_min(-250)
        .set_x_max(1100)
        .set_y_max(900)
        .set_mac_style(0)
        .set_lowest_rec_ppem(8)
        .set_font_direction_hint(2)
        .set_index_to_loc_format(0)
        .set_glyph_data_format(0);
    let got = w.finish().expect("every field is set");

    let mut want = Vec::new();
    want.extend_from_slice(&1_u16.to_be_bytes());
    want.extend_from_slice(&0_u16.to_be_bytes());
    want.extend_from_slice(&0x0001_0000_i32.to_be_bytes()); // Fixed
    want.extend_from_slice(&0xB1B0_AFBA_u32.to_be_bytes());
    want.extend_from_slice(&0x5F0F_3CF5_u32.to_be_bytes());
    want.extend_from_slice(&3_u16.to_be_bytes());
    want.extend_from_slice(&1000_u16.to_be_bytes());
    want.extend_from_slice(&3_600_000_000_i64.to_be_bytes()); // LONGDATETIME
    want.extend_from_slice(&3_600_000_001_i64.to_be_bytes());
    want.extend_from_slice(&(-100_i16).to_be_bytes());
    want.extend_from_slice(&(-250_i16).to_be_bytes());
    want.extend_from_slice(&1100_i16.to_be_bytes());
    want.extend_from_slice(&900_i16.to_be_bytes());
    want.extend_from_slice(&0_u16.to_be_bytes());
    want.extend_from_slice(&8_u16.to_be_bytes());
    want.extend_from_slice(&2_i16.to_be_bytes());
    want.extend_from_slice(&0_i16.to_be_bytes());
    want.extend_from_slice(&0_i16.to_be_bytes());

    assert_eq!(want.len(), 54, "the prose's head is 54 bytes");
    assert_eq!(got, want);
    assert_eq!(
        got.len(),
        54,
        "a 32-bit `created` would make it 46 and move everything after it"
    );
}

/// maxp version 0.5 is the whole table for a CFF font: a Fixed version
/// and a glyph count, six bytes.
#[test]
fn fc03_maxp_half_is_six_bytes() {
    let mut w = MaxpHalfWriter::new();
    w.set_version(0x0000_5000).set_num_glyphs(42);
    let got = w.finish().expect("both fields are set");
    let mut want = Vec::new();
    want.extend_from_slice(&0x0000_5000_i32.to_be_bytes());
    want.extend_from_slice(&42_u16.to_be_bytes());
    assert_eq!(got, want);
    assert_eq!(got.len(), 6);
}
