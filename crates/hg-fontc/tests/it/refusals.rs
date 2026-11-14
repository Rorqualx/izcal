//! What the writer refuses, and that the same table without the fault
//! is accepted.
//!
//! Every case here is the twin discipline JS-10 established: a planted
//! fault must be refused *and* the identical tree without the plant
//! must pass. A refusal on its own proves nothing — a writer that
//! refused everything would pass the first half of each of these tests.

use hg_fontc::Error;
use hg_fontc::writers::{CmapWriter, HheaWriter, HmtxWriter, LocaShortWriter};
use hg_fontfmt::tables::{EncodingRecord, LongHorMetric};

fn filled_hhea() -> HheaWriter {
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
    w
}

/// The forgotten field. A writer that wrote zero for it would produce a
/// `descender` of 0, which is a font whose text sits on the wrong line
/// and which every reader accepts.
#[test]
fn fc11_a_field_that_was_never_set_is_named_and_the_twin_passes() {
    // The plant: `filled_hhea` with one line removed, nothing else.
    let mut planted = HheaWriter::new();
    planted
        .set_major_version(1)
        .set_minor_version(0)
        .set_ascender(1900)
        // .set_descender(-500) -- the plant
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
    let err = planted.finish().expect_err("descender was never set");
    assert_eq!(
        err,
        Error::Unset {
            table: "Hhea",
            field: "descender",
        }
    );
    assert_eq!(err.to_string(), "Hhea.descender was never set");

    // The twin: the same table with the field set.
    assert!(
        filled_hhea().finish().is_ok(),
        "the twin without the plant passes"
    );
}

/// An array shorter than the count the table declares. Padding it with
/// zeroes would be a font with three metrics claiming four, which the
/// reader declines — at load time, in the user's browser, rather than
/// here.
#[test]
fn fc12_too_few_elements_are_refused_with_both_numbers() {
    let mut w = CmapWriter::new();
    w.set_version(0)
        .set_num_tables(3)
        .push_encoding_records(EncodingRecord {
            platform_id: 3,
            encoding_id: 1,
            subtable_offset: 12,
        });
    let err = w.finish().expect_err("numTables says 3 and one was pushed");
    assert_eq!(
        err,
        Error::Count {
            table: "Cmap",
            field: "encoding_records",
            declared: 3,
            given: 1,
        }
    );
    assert_eq!(
        err.to_string(),
        "Cmap.encoding_records: the table declares 3 and 1 were given"
    );

    // The twin: the same writer with the count corrected.
    w.set_num_tables(1);
    assert!(w.finish().is_ok(), "the twin without the plant passes");
}

/// An array longer than the count. Truncating would silently drop a
/// subtable, which is the mistake that is hardest to see in the result.
#[test]
fn fc13_too_many_elements_are_refused() {
    let mut w = HmtxWriter::new(2, 4);
    let m = LongHorMetric {
        advance_width: 500,
        lsb: 0,
    };
    w.extend_h_metrics(&[m, m, m])
        .extend_left_side_bearings(&[0, 0]);
    assert_eq!(
        w.finish(),
        Err(Error::Count {
            table: "Hmtx",
            field: "h_metrics",
            declared: 2,
            given: 3,
        })
    );

    // The twin: two metrics, as hhea's numberOfHMetrics says.
    let mut ok = HmtxWriter::new(2, 4);
    ok.extend_h_metrics(&[m, m])
        .extend_left_side_bearings(&[0, 0]);
    assert!(ok.finish().is_ok(), "the twin without the plant passes");
}

/// A count that names an unset field is reported against that field,
/// not against the array, because that is where the fix goes.
#[test]
fn fc14_an_unset_count_field_is_named_rather_than_the_array() {
    let mut w = CmapWriter::new();
    w.set_version(0);
    assert_eq!(
        w.encoding_records_len(),
        Err(Error::Unset {
            table: "Cmap",
            field: "num_tables",
        })
    );
}

/// `numGlyphs + 1` overflows at the top of the parameter's range. The
/// reader accumulates in `i64` and so does the writer, so neither
/// wraps; the writer says so instead.
#[test]
fn fc15_a_count_at_the_top_of_the_range_does_not_wrap() {
    let w = LocaShortWriter::new(u16::MAX);
    assert_eq!(
        w.offsets_len(),
        Ok(65_536),
        "65535 + 1 widens rather than wrapping to 0"
    );
}

/// The writer is a builder, so the obvious misuse is to fill it in the
/// wrong order. Order does not matter: only the values do.
#[test]
fn fc16_the_order_fields_are_set_in_does_not_change_the_bytes() {
    let forwards = filled_hhea().finish().expect("filled");
    let mut backwards = HheaWriter::new();
    backwards
        .set_number_of_h_metrics(3)
        .set_metric_data_format(0)
        .set_reserved3(0)
        .set_reserved2(0)
        .set_reserved1(0)
        .set_reserved0(0)
        .set_caret_offset(0)
        .set_caret_slope_run(0)
        .set_caret_slope_rise(1)
        .set_x_max_extent(2000)
        .set_min_right_side_bearing(-150)
        .set_min_left_side_bearing(-200)
        .set_advance_width_max(2048)
        .set_line_gap(0)
        .set_descender(-500)
        .set_ascender(1900)
        .set_minor_version(0)
        .set_major_version(1);
    assert_eq!(backwards.finish().expect("filled"), forwards);
}
