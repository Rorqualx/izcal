//! The html5lib encoding suite: HTML's "determine the character
//! encoding" over the pinned cases.

use hg_encoding::{Encoding, HTML_LIMIT, extract_from_meta, prescan, xml_encoding};
use hg_html5lib::encoding;

fn suite() -> Option<Vec<encoding::Case>> {
    let dir = match hg_html5lib::tree_dir() {
        Ok(d) => d.join(encoding::DIR),
        Err(e) => panic!("{e}"),
    };
    if !dir.is_dir() {
        assert!(
            std::env::var_os("IZCAL_REQUIRE_HTML_DATA").is_none(),
            "IZCAL_REQUIRE_HTML_DATA=1 but the html5lib suite is not in the cache; run \
             `tools/fetch-test-data.sh html5lib`"
        );
        println!("note: the html5lib suite is not fetched; skipped");
        return None;
    }
    Some(encoding::read_all(&dir).expect("the suite reads"))
}

#[test]
fn st_d16_the_encoding_suite() {
    let Some(cases) = suite() else { return };
    assert_eq!(cases.len(), encoding::CASES, "cases at the pin");
    let l = encoding::run(&cases);
    for e in &l.examples {
        println!("  {e}");
    }
    println!(
        "hg-html5lib: encoding {}/{}, baseline {}",
        l.pass,
        l.runs,
        encoding::BASELINE
    );
    assert!(
        l.pass >= encoding::BASELINE,
        "{} of {}, below the baseline of {}",
        l.pass,
        l.runs,
        encoding::BASELINE
    );
}

#[test]
fn the_window_is_a_decision_and_this_is_what_it_costs() {
    // Every case that fails at 1,024 bytes passes with the whole
    // stream, and each is a document the standard calls
    // non-conforming. If this stops holding, the seven are no longer
    // only about the window and the baseline's reason is wrong.
    let Some(cases) = suite() else { return };
    let narrow = encoding::run_within(&cases, HTML_LIMIT);
    let whole = encoding::run_within(&cases, usize::MAX);
    assert_eq!(narrow.pass, 75);
    assert_eq!(
        whole.pass,
        encoding::CASES,
        "the whole stream finds them all"
    );
    // And the window is monotone: a bigger one never finds fewer.
    let mut last = 0_usize;
    for limit in [512_usize, 1024, 2048, 4096, 8192, usize::MAX] {
        let m = encoding::run_within(&cases, limit);
        assert!(
            m.pass >= last,
            "window {limit} found {} after {last}",
            m.pass
        );
        last = m.pass;
    }
}

#[test]
fn the_pieces_the_prescan_is_built_from() {
    // "Extracting a character encoding from a meta element", which is
    // also what a Content-Type header's charset goes through.
    assert_eq!(
        extract_from_meta(b"text/html; charset=utf-8"),
        Some(Encoding::Utf8)
    );
    assert_eq!(
        extract_from_meta(b"text/html;charset='iso-8859-2'"),
        Some(Encoding::Iso8859_2)
    );
    assert_eq!(
        extract_from_meta(b"text/html; charset = \"windows-1251\" ; x=y"),
        Some(Encoding::Windows1251)
    );
    // "charset" that is not followed by `=` does not end the search.
    assert_eq!(
        extract_from_meta(b"charsetcharset=utf-8"),
        Some(Encoding::Utf8)
    );
    assert_eq!(extract_from_meta(b"text/html"), None);
    assert_eq!(extract_from_meta(b"charset="), None);
    assert_eq!(
        extract_from_meta(b"charset=\"utf-8"),
        None,
        "an unmatched quote"
    );

    // "Get an XML encoding", which the prescan falls back to.
    assert_eq!(
        xml_encoding(b"<?xml version='1.0' encoding='iso-8859-2'?>"),
        Some(Encoding::Iso8859_2)
    );
    // UTF-16BE/LE becomes UTF-8 there too.
    assert_eq!(
        xml_encoding(b"<?xml encoding=\"utf-16\"?>"),
        Some(Encoding::Utf8)
    );
    assert_eq!(xml_encoding(b"<html>"), None);
    assert_eq!(
        xml_encoding(b"<?xml encoding='utf 8'?>"),
        None,
        "a space in the label"
    );
}

#[test]
fn the_two_substitutions_the_prescan_makes() {
    // x-user-defined becomes windows-1252, and UTF-16 becomes UTF-8.
    // Both stop a page naming an encoding under which the ASCII the
    // prescan has already read would mean something else.
    assert_eq!(
        prescan(b"<meta charset=x-user-defined>", HTML_LIMIT),
        Some(Encoding::Windows1252)
    );
    assert_eq!(
        prescan(b"<meta charset=utf-16le>", HTML_LIMIT),
        Some(Encoding::Utf8)
    );
}

#[test]
fn running_out_of_bytes_aborts_rather_than_answering() {
    // The algorithm: "If at any point during these steps (including
    // during instances of the get an attribute algorithm invoked by
    // this one) the user agent ... runs out of bytes ... then abort".
    // An unterminated element has not been seen whole, so it does not
    // count. The suite's tests2 case 5 is exactly this.
    assert_eq!(prescan(b"<meta charset=euc-jp", HTML_LIMIT), None);
    assert_eq!(prescan(b"<meta charset=\"euc-jp", HTML_LIMIT), None);
    assert_eq!(
        prescan(b"<meta charset=euc-jp>", HTML_LIMIT),
        Some(Encoding::EucJp),
        "and a terminated one does"
    );
}
