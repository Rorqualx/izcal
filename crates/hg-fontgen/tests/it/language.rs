//! Each rule of `.hgfont` refuses the thing it exists to refuse, and the
//! same description without that thing is accepted — so a refusal is
//! attributable to the rule and not to the fixture.

use std::path::{Path, PathBuf};

use hg_fontgen::{model, syntax};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A description that parses and checks, used as the twin for every
/// refusal below.
const GOOD: &str = r#"
record Pair @ "spec#pair" {
  a u16 @ "spec#a"
  b i16 @ "spec#b"
}

table Thing(count: u16) @ "spec#thing" {
  version u16 @ "spec#version"
  pairs [count] Pair @ "spec#pairs"
}
"#;

fn refuse(src: &str) -> String {
    let e = model(&[("t.hgfont".into(), src.into())]).expect_err("should be refused");
    e.to_string()
}

#[test]
fn the_twin_is_accepted() {
    let m = model(&[("t.hgfont".into(), GOOD.into())]).expect("the twin parses and checks");
    assert_eq!(m.records.len(), 1);
    assert_eq!(m.tables.len(), 1);
    assert!(m.anchors.contains("spec#pairs"));
}

#[test]
fn an_anchor_is_required_on_every_field() {
    let src = GOOD.replace(" a u16 @ \"spec#a\"", " a u16");
    let e = refuse(&src);
    assert!(e.contains("expected '@'"), "{e}");
}

#[test]
fn an_empty_anchor_is_refused() {
    let src = GOOD.replace("\"spec#a\"", "\"\"");
    assert!(refuse(&src).contains("may not be empty"));
}

#[test]
fn a_record_may_not_hold_an_array() {
    let src = GOOD.replace("  b i16 @ \"spec#b\"", "  b [count] i16 @ \"spec#b\"");
    let e = refuse(&src);
    assert!(e.contains("may not hold an array"), "{e}");
}

#[test]
fn a_length_may_name_only_a_parameter_or_an_earlier_scalar() {
    let src = GOOD.replace("[count] Pair", "[nowhere] Pair");
    let e = refuse(&src);
    assert!(e.contains("not a parameter"), "{e}");
}

#[test]
fn a_length_may_not_name_a_tag() {
    let src = GOOD
        .replace(
            "  version u16 @ \"spec#version\"",
            "  which tag @ \"spec#which\"",
        )
        .replace("[count] Pair", "[which] Pair");
    let e = refuse(&src);
    assert!(e.contains("a tag, not a number"), "{e}");
}

#[test]
fn a_divisor_of_zero_is_refused() {
    let src = GOOD.replace("[count] Pair", "[count / 0] Pair");
    assert!(refuse(&src).contains("may not be zero"));
}

#[test]
fn a_record_that_does_not_exist_is_refused() {
    let src = GOOD.replace("[count] Pair", "[count] Missing");
    let e = refuse(&src);
    assert!(
        e.contains("cannot be sized") || e.contains("no record called"),
        "{e}"
    );
}

#[test]
fn records_that_contain_each_other_are_refused() {
    let src = r#"
record A @ "s#a" { x B @ "s#x" }
record B @ "s#b" { y A @ "s#y" }
"#;
    let e = refuse(src);
    assert!(
        e.contains("refer to each other") || e.contains("cannot be sized"),
        "{e}"
    );
}

#[test]
fn names_must_be_the_right_shape() {
    assert!(refuse(&GOOD.replace("record Pair", "record pair")).contains("UpperCamelCase"));
    assert!(refuse(&GOOD.replace("  a u16", "  A u16")).contains("snake_case"));
}

#[test]
fn a_duplicate_field_is_refused() {
    let src = GOOD.replace("  b i16 @ \"spec#b\"", "  a i16 @ \"spec#b\"");
    assert!(refuse(&src).contains("declared twice"));
}

#[test]
fn an_anchor_may_carry_a_fragment() {
    // The comment rule must not cut a string at its `#`: every anchor
    // in the real descriptions carries one.
    let f = syntax::parse(
        "record R @ \"otspec/head#xMin\" { a u8 @ \"otspec/head#a\" }",
        "t",
    )
    .expect("parses");
    assert_eq!(f.records[0].anchor, "otspec/head#xMin");
}

#[test]
fn g29_the_checked_in_output_is_fresh() {
    let stale = hg_fontgen::cli::stale(&root()).expect("the descriptions build");
    assert!(
        stale.is_empty(),
        "stale: {:?}; run `cargo run -p hg-fontgen -- generate`",
        stale
    );
}

#[test]
fn g29_a_hand_edit_is_caught() {
    // The real root is fresh; a copy with one line appended is not.
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR")).join("g29");
    let gen_dir = tmp.join("crates/hg-fontfmt/src/generated");
    let fontc_dir = tmp.join("crates/hg-fontc/src/generated");
    std::fs::create_dir_all(&gen_dir).unwrap();
    std::fs::create_dir_all(&fontc_dir).unwrap();
    std::fs::create_dir_all(tmp.join("crates/hg-fontfmt/tables")).unwrap();
    for entry in std::fs::read_dir(root().join("crates/hg-fontfmt/tables")).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(
            &p,
            tmp.join("crates/hg-fontfmt/tables")
                .join(p.file_name().unwrap()),
        )
        .unwrap();
    }
    for name in ["tables.rs", "proofs.rs", "grammar.tsv", "anchors.tsv"] {
        let from = root().join("crates/hg-fontfmt/src/generated").join(name);
        std::fs::copy(&from, gen_dir.join(name)).unwrap();
    }
    std::fs::copy(
        root().join("crates/hg-fontc/src/generated/writers.rs"),
        fontc_dir.join("writers.rs"),
    )
    .unwrap();
    assert!(
        hg_fontgen::cli::stale(&tmp).unwrap().is_empty(),
        "the copy starts fresh"
    );

    let edited = gen_dir.join("anchors.tsv");
    let text = std::fs::read_to_string(&edited).unwrap();
    std::fs::write(&edited, format!("{text}a-hand-edit\n")).unwrap();
    let stale = hg_fontgen::cli::stale(&tmp).unwrap();
    assert_eq!(
        stale.len(),
        1,
        "exactly the edited file is stale: {stale:?}"
    );

    hg_fontgen::cli::generate(&tmp).unwrap();
    assert!(
        hg_fontgen::cli::stale(&tmp).unwrap().is_empty(),
        "fresh again"
    );
}

#[test]
fn every_described_field_carries_an_anchor_into_the_index() {
    let m = hg_fontgen::cli::build(&root()).expect("the real descriptions check");
    let fields: usize = m.tables.iter().map(|t| t.fields.len()).sum::<usize>()
        + m.records
            .values()
            .map(|(r, _)| r.fields.len())
            .sum::<usize>();
    assert!(fields >= 100, "only {fields} fields described");
    assert!(
        m.anchors.iter().all(|a| !a.is_empty()),
        "an anchor is empty"
    );
}
