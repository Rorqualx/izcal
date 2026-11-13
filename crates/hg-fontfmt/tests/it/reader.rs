//! The generated readers, exercised on real and hostile bytes.
//!
//! Memory safety is Kani's to prove for every input (TX-11); these tests
//! are about the other half — that a well-formed table reads back the
//! values the prose says, and that a table whose declared counts exceed
//! its bytes declines instead of inventing them.

use hg_fontfmt::tables::{Cmap, CmapFormat4, Head, Hhea, Hmtx, LocaShort, TableDirectory};

fn be16(v: u16) -> [u8; 2] {
    v.to_be_bytes()
}

#[test]
fn a_table_directory_reads_its_records() {
    let mut d = Vec::new();
    d.extend_from_slice(&0x0001_0000_u32.to_be_bytes()); // sfntVersion
    d.extend_from_slice(&be16(2)); // numTables
    d.extend_from_slice(&be16(32)); // searchRange
    d.extend_from_slice(&be16(1)); // entrySelector
    d.extend_from_slice(&be16(0)); // rangeShift
    for (tag, off, len) in [(*b"head", 28_u32, 54_u32), (*b"hhea", 82, 36)] {
        d.extend_from_slice(&tag);
        d.extend_from_slice(&0_u32.to_be_bytes()); // checksum
        d.extend_from_slice(&off.to_be_bytes());
        d.extend_from_slice(&len.to_be_bytes());
    }
    let t = TableDirectory::new(&d);
    assert_eq!(t.sfnt_version(), Some(0x0001_0000));
    assert_eq!(t.num_tables(), Some(2));
    assert_eq!(t.table_records_len(), Some(2));
    let r0 = t.table_records(0).unwrap();
    assert_eq!(r0.table_tag, u32::from_be_bytes(*b"head"));
    assert_eq!(r0.offset, 28);
    assert_eq!(r0.length, 54);
    assert_eq!(t.table_records(1).unwrap().length, 36);
    // One past the end is None, not a wrap or a panic.
    assert_eq!(t.table_records(2), None);
}

#[test]
fn a_declared_count_the_bytes_do_not_back_reads_none() {
    // numTables says 1000; there are no records at all.
    let mut d = Vec::new();
    d.extend_from_slice(&0x0001_0000_u32.to_be_bytes());
    d.extend_from_slice(&be16(1000));
    d.extend_from_slice(&be16(0));
    d.extend_from_slice(&be16(0));
    d.extend_from_slice(&be16(0));
    let t = TableDirectory::new(&d);
    assert_eq!(t.table_records_len(), Some(1000));
    assert_eq!(t.table_records(0), None, "the count is a claim, not bytes");
    assert_eq!(t.table_records(999), None);
}

#[test]
fn head_reads_the_fields_at_their_specified_offsets() {
    let mut d = vec![0_u8; 54];
    d[0..2].copy_from_slice(&be16(1)); // majorVersion
    d[2..4].copy_from_slice(&be16(0)); // minorVersion
    d[12..16].copy_from_slice(&0x5F0F_3CF5_u32.to_be_bytes()); // magicNumber
    d[18..20].copy_from_slice(&be16(2048)); // unitsPerEm
    d[50..52].copy_from_slice(&1_i16.to_be_bytes()); // indexToLocFormat
    let h = Head::new(&d);
    assert_eq!(h.major_version(), Some(1));
    assert_eq!(h.magic_number(), Some(0x5F0F_3CF5));
    assert_eq!(h.units_per_em(), Some(2048));
    assert_eq!(h.index_to_loc_format(), Some(1));
    // Truncated by one byte: the last field declines, the rest do not.
    let short = Head::new(&d[..53]);
    assert_eq!(short.units_per_em(), Some(2048));
    assert_eq!(short.glyph_data_format(), None);
}

#[test]
fn hmtx_takes_its_counts_from_hhea_and_maxp() {
    // numberOfHMetrics = 2, numGlyphs = 4: two full metrics then two
    // bare left side bearings.
    let mut d = Vec::new();
    for (aw, lsb) in [(600_u16, 30_i16), (700, -20)] {
        d.extend_from_slice(&be16(aw));
        d.extend_from_slice(&lsb.to_be_bytes());
    }
    d.extend_from_slice(&11_i16.to_be_bytes());
    d.extend_from_slice(&(-12_i16).to_be_bytes());
    let t = Hmtx::new(&d, 2, 4);
    assert_eq!(t.h_metrics_len(), Some(2));
    assert_eq!(t.h_metrics(0).unwrap().advance_width, 600);
    assert_eq!(t.h_metrics(1).unwrap().lsb, -20);
    assert_eq!(t.left_side_bearings_len(), Some(2));
    assert_eq!(t.left_side_bearings(0), Some(11));
    assert_eq!(t.left_side_bearings(1), Some(-12));
    assert_eq!(t.left_side_bearings(2), None);
}

#[test]
fn hmtx_clamps_a_negative_count_instead_of_wrapping() {
    // A font that says it has fewer glyphs than full metrics. The
    // difference is negative; the reader must report no trailing
    // bearings, not 65_534 of them.
    let d = [0_u8; 8];
    let t = Hmtx::new(&d, 4, 1);
    assert_eq!(t.left_side_bearings_len(), Some(0));
    assert_eq!(t.left_side_bearings(0), None);
}

#[test]
fn loca_holds_one_more_entry_than_there_are_glyphs() {
    let mut d = Vec::new();
    for v in [0_u16, 10, 24] {
        d.extend_from_slice(&be16(v));
    }
    let t = LocaShort::new(&d, 2);
    assert_eq!(t.offsets_len(), Some(3));
    assert_eq!(t.offsets(2), Some(24));
    assert_eq!(t.offsets(3), None);
}

#[test]
fn cmap_format_4_sizes_its_parallel_arrays_from_seg_count() {
    // One segment: segCountX2 = 2, so each array holds one entry and
    // the glyph array holds (length - 16 - 4*2) / 2 = 1.
    let seg_count_x2 = 2_u16;
    let length = 16 + 4 * seg_count_x2 + 2;
    let mut d = Vec::new();
    d.extend_from_slice(&be16(4)); // format
    d.extend_from_slice(&be16(length)); // length
    d.extend_from_slice(&be16(0)); // language
    d.extend_from_slice(&be16(seg_count_x2));
    d.extend_from_slice(&be16(2)); // searchRange
    d.extend_from_slice(&be16(0)); // entrySelector
    d.extend_from_slice(&be16(0)); // rangeShift
    d.extend_from_slice(&be16(0xFFFF)); // endCode[0]
    d.extend_from_slice(&be16(0)); // reservedPad
    d.extend_from_slice(&be16(0x0041)); // startCode[0]
    d.extend_from_slice(&(-0x0040_i16).to_be_bytes()); // idDelta[0]
    d.extend_from_slice(&be16(0)); // idRangeOffsets[0]
    d.extend_from_slice(&be16(7)); // glyphIdArray[0]

    let t = CmapFormat4::new(&d);
    assert_eq!(t.format(), Some(4));
    assert_eq!(t.end_code_len(), Some(1));
    assert_eq!(t.end_code(0), Some(0xFFFF));
    assert_eq!(t.start_code(0), Some(0x0041));
    assert_eq!(t.id_delta(0), Some(-0x0040));
    assert_eq!(t.id_range_offsets(0), Some(0));
    assert_eq!(t.glyph_id_array_len(), Some(1));
    assert_eq!(t.glyph_id_array(0), Some(7));
    assert_eq!(t.glyph_id_array(1), None);
}

#[test]
fn a_cmap_header_with_no_bytes_declines_every_record() {
    let t = Cmap::new(&[]);
    assert_eq!(t.version(), None);
    assert_eq!(t.num_tables(), None);
    assert_eq!(t.encoding_records_len(), None);
    assert_eq!(t.encoding_records(0), None);
}

#[test]
fn an_empty_slice_never_panics_in_any_reader() {
    let empty: &[u8] = &[];
    assert_eq!(Head::new(empty).units_per_em(), None);
    assert_eq!(Hhea::new(empty).number_of_h_metrics(), None);
    assert_eq!(
        Hmtx::new(empty, u16::MAX, u16::MAX).h_metrics(usize::MAX),
        None
    );
    assert_eq!(LocaShort::new(empty, u16::MAX).offsets(usize::MAX), None);
    assert_eq!(CmapFormat4::new(empty).glyph_id_array(usize::MAX), None);
}
