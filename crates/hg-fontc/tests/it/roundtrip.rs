//! Every table this crate can write, read back by `hg-fontfmt`.
//!
//! This is TX-D24's rule made mechanical: "every output re-read by
//! `hg-fontfmt` before it enters the font package". A round-trip proves
//! less than it looks — see `bytes.rs` for why — but it proves the one
//! thing the byte tests cannot cover at scale: that the offset chain
//! the reader computes for a table with arrays lands on the bytes the
//! writer put there.

use hg_fontc::writers::{
    CmapFormat4Writer, CmapWriter, HmtxWriter, LocaShortWriter, TableDirectoryWriter,
    TtcHeaderWriter,
};
use hg_fontfmt::tables::{
    Cmap, CmapFormat4, EncodingRecord, Hmtx, LocaShort, LongHorMetric, TableDirectory, TableRecord,
    TtcHeader,
};

#[test]
fn fc04_a_table_directory_round_trips_with_its_records() {
    let records = [
        TableRecord {
            table_tag: u32::from_be_bytes(*b"head"),
            checksum: 0x1234_5678,
            offset: 28,
            length: 54,
        },
        TableRecord {
            table_tag: u32::from_be_bytes(*b"hhea"),
            checksum: 0x9ABC_DEF0,
            offset: 82,
            length: 36,
        },
    ];
    let mut w = TableDirectoryWriter::new();
    w.set_sfnt_version(0x0001_0000)
        .set_num_tables(2)
        .set_search_range(32)
        .set_entry_selector(1)
        .set_range_shift(0)
        .extend_table_records(&records);
    let bytes = w.finish().expect("the counts agree");

    let t = TableDirectory::new(&bytes);
    assert_eq!(t.sfnt_version(), Some(0x0001_0000));
    assert_eq!(t.num_tables(), Some(2));
    assert_eq!(t.table_records_len(), Some(2));
    assert_eq!(t.table_records(0), Some(records[0]));
    assert_eq!(t.table_records(1), Some(records[1]));
    assert_eq!(t.table_records(2), None, "one past the end is not there");
    assert_eq!(bytes.len(), 12 + 2 * 16);
}

#[test]
fn fc05_hmtx_round_trips_with_both_of_its_arrays() {
    let metrics = [
        LongHorMetric {
            advance_width: 600,
            lsb: 40,
        },
        LongHorMetric {
            advance_width: 600,
            lsb: -12,
        },
        LongHorMetric {
            advance_width: 1200,
            lsb: 0,
        },
    ];
    // numberOfHMetrics comes from hhea and numGlyphs from maxp, so both
    // are parameters: hmtx carries no count of its own.
    let mut w = HmtxWriter::new(3, 6);
    w.extend_h_metrics(&metrics)
        .extend_left_side_bearings(&[7, -7, 0]);
    let bytes = w.finish().expect("3 metrics and 6 - 3 bearings");

    let t = Hmtx::new(&bytes, 3, 6);
    assert_eq!(t.h_metrics_len(), Some(3));
    assert_eq!(t.left_side_bearings_len(), Some(3));
    for (i, m) in metrics.iter().enumerate() {
        assert_eq!(t.h_metrics(i), Some(*m));
    }
    assert_eq!(t.left_side_bearings(0), Some(7));
    assert_eq!(t.left_side_bearings(1), Some(-7));
    assert_eq!(t.left_side_bearings(2), Some(0));
    assert_eq!(bytes.len(), 3 * 4 + 3 * 2);
}

/// `numGlyphs` below `numberOfHMetrics` gives a negative count. The
/// reader clamps it at zero rather than wrapping, and the writer has to
/// clamp in the same place or the two disagree about a font that exists
/// in the wild.
#[test]
fn fc06_hmtx_clamps_a_negative_bearing_count_where_the_reader_does() {
    let mut w = HmtxWriter::new(4, 2);
    w.extend_h_metrics(
        &[LongHorMetric {
            advance_width: 500,
            lsb: 0,
        }; 4],
    );
    assert_eq!(
        w.left_side_bearings_len(),
        Ok(0),
        "2 - 4 clamps to 0, as `acc.max(0)` does in the reader"
    );
    let bytes = w.finish().expect("no bearings are expected");
    let t = Hmtx::new(&bytes, 4, 2);
    assert_eq!(t.left_side_bearings_len(), Some(0));
    assert_eq!(t.left_side_bearings(0), None);
}

#[test]
fn fc07_cmap_s_header_round_trips() {
    let records = [
        EncodingRecord {
            platform_id: 3,
            encoding_id: 1,
            subtable_offset: 20,
        },
        EncodingRecord {
            platform_id: 3,
            encoding_id: 10,
            subtable_offset: 60,
        },
    ];
    let mut w = CmapWriter::new();
    w.set_version(0)
        .set_num_tables(2)
        .extend_encoding_records(&records);
    let bytes = w.finish().expect("numTables matches");
    let t = Cmap::new(&bytes);
    assert_eq!(t.num_tables(), Some(2));
    assert_eq!(t.encoding_records(0), Some(records[0]));
    assert_eq!(t.encoding_records(1), Some(records[1]));
}

/// Format 4's glyph array has no count of its own: it fills whatever
/// `length` leaves, which makes `length` a field the compiler has to
/// get right and the writer the place that says so.
#[test]
fn fc08_cmap_format_4_round_trips_and_its_length_decides_the_glyph_array() {
    // Two segments, three glyph-array entries. The fixed part and the
    // four parallel arrays are 16 + 4 * segCountX2 = 32 bytes.
    let length = 32 + 3 * 2;
    let mut w = CmapFormat4Writer::new();
    w.set_format(4)
        .set_length(length)
        .set_language(0)
        .set_seg_count_x2(4)
        .set_search_range(4)
        .set_entry_selector(1)
        .set_range_shift(0)
        .extend_end_code(&[0x00FF, 0xFFFF])
        .set_reserved_pad(0)
        .extend_start_code(&[0x0020, 0xFFFF])
        .extend_id_delta(&[-29, 1])
        .extend_id_range_offsets(&[0, 0])
        .extend_glyph_id_array(&[1, 2, 3]);
    assert_eq!(w.glyph_id_array_len(), Ok(3));
    let bytes = w.finish().expect("every count agrees");
    assert_eq!(bytes.len(), usize::from(length));

    let t = CmapFormat4::new(&bytes);
    assert_eq!(t.format(), Some(4));
    assert_eq!(t.length(), Some(length));
    assert_eq!(t.end_code_len(), Some(2));
    assert_eq!(t.end_code(1), Some(0xFFFF));
    assert_eq!(t.start_code(0), Some(0x0020));
    assert_eq!(t.id_delta(0), Some(-29));
    assert_eq!(t.glyph_id_array_len(), Some(3));
    assert_eq!(t.glyph_id_array(2), Some(3));
    assert_eq!(t.glyph_id_array(3), None);
}

#[test]
fn fc09_loca_short_holds_one_more_entry_than_there_are_glyphs() {
    let mut w = LocaShortWriter::new(4);
    w.extend_offsets(&[0, 10, 10, 24, 40]);
    assert_eq!(
        w.offsets_len(),
        Ok(5),
        "numGlyphs + 1, so the last end is known"
    );
    let bytes = w.finish().expect("5 offsets for 4 glyphs");
    let t = LocaShort::new(&bytes, 4);
    assert_eq!(t.offsets_len(), Some(5));
    assert_eq!(t.offsets(4), Some(40));
    assert_eq!(t.offsets(5), None);
}

#[test]
fn fc10_a_collection_header_round_trips() {
    let mut w = TtcHeaderWriter::new();
    w.set_ttc_tag(u32::from_be_bytes(*b"ttcf"))
        .set_major_version(1)
        .set_minor_version(0)
        .set_num_fonts(2)
        .extend_table_directory_offsets(&[12, 1024]);
    let bytes = w.finish().expect("numFonts matches");
    let t = TtcHeader::new(&bytes);
    assert_eq!(t.num_fonts(), Some(2));
    assert_eq!(t.table_directory_offsets(0), Some(12));
    assert_eq!(t.table_directory_offsets(1), Some(1024));
}
