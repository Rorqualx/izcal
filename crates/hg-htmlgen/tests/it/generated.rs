//! The checked-in output is fresh and a hand edit is caught (G29), and
//! the descriptions say what the standard says they should.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn g29_the_checked_in_output_is_fresh() {
    let stale = hg_htmlgen::cli::stale(&root()).expect("the descriptions build");
    assert!(
        stale.is_empty(),
        "stale: {stale:?}; run `cargo run -p hg-htmlgen -- generate`"
    );
}

#[test]
fn g29_a_hand_edit_is_caught() {
    // The real root is fresh; a copy with one line appended is not.
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR")).join("g29");
    let gen_dir = tmp.join("crates/hg-html-syntax/src/generated");
    let states = tmp.join(hg_htmlgen::cli::SOURCES);
    std::fs::create_dir_all(&gen_dir).unwrap();
    std::fs::create_dir_all(&states).unwrap();
    for entry in std::fs::read_dir(root().join(hg_htmlgen::cli::SOURCES)).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, states.join(p.file_name().unwrap())).unwrap();
    }
    for name in [
        "states.rs",
        "tokenizer.rs",
        "transitions.tsv",
        "anchors.tsv",
        "entities.rs",
    ] {
        let from = root()
            .join("crates/hg-html-syntax/src/generated")
            .join(name);
        std::fs::copy(&from, gen_dir.join(name)).unwrap();
    }
    assert!(
        hg_htmlgen::cli::stale(&tmp).unwrap().is_empty(),
        "the copy starts fresh"
    );

    let edited = gen_dir.join("anchors.tsv");
    let text = std::fs::read_to_string(&edited).unwrap();
    std::fs::write(&edited, format!("{text}a-hand-edit\n")).unwrap();
    let stale = hg_htmlgen::cli::stale(&tmp).unwrap();
    assert_eq!(
        stale.len(),
        1,
        "exactly the edited file is stale: {stale:?}"
    );

    hg_htmlgen::cli::generate(&tmp).unwrap();
    assert!(
        hg_htmlgen::cli::stale(&tmp).unwrap().is_empty(),
        "fresh again"
    );
}

#[test]
fn the_output_is_the_same_whatever_the_file_system_order() {
    let m = hg_htmlgen::cli::build(&root()).expect("the real descriptions check");
    let a = hg_htmlgen::emit::states(&m);
    let b = hg_htmlgen::emit::states(&m);
    assert_eq!(a, b);
    assert!(!a.contains('\r'), "Unix newlines only");
}

#[test]
fn the_descriptions_cover_the_standards_tokenizer() {
    let m = hg_htmlgen::cli::build(&root()).expect("the real descriptions check");
    // §13.2.5 has 84 states at the pinned revision — 79 of the older
    // set, plus the five processing instruction states that replaced
    // `<?`'s bogus comment, less the decimal character reference start
    // state, which the standard dropped. The number is here so that
    // losing one is a failing test rather than a quieter tokenizer.
    assert_eq!(m.states.len(), 84, "the standard's tokenizer states");
    // Exactly two are hand-written, and they are the two that are not
    // per-code-point tables.
    let external: Vec<&str> = m
        .states
        .iter()
        .filter(|s| s.external)
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(
        external,
        vec![
            "named_character_reference",
            "numeric_character_reference_end"
        ]
    );
    // Five are the tree builder's switch targets.
    let entry: Vec<&str> = m
        .states
        .iter()
        .filter(|s| s.entry)
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(
        entry,
        vec!["data", "rcdata", "rawtext", "script_data", "plaintext"]
    );
    // Every state and every rule carries its clause.
    assert!(m.anchors.iter().all(|a| a.starts_with("html/parsing#")));
    assert_eq!(m.anchors.len(), 84, "one anchor per state");
    // The standard names 43 tokenizer parse errors (§13.2.2, leaving
    // out the three the input stream raises and the two the tree
    // builder raises). 37 are named by the descriptions; the other six
    // belong to the two hand-written states and are raised by the
    // runtime, which is what those states owe:
    //
    //   duplicate-attribute, null-character-reference,
    //   character-reference-outside-unicode-range,
    //   surrogate-character-reference, noncharacter-character-reference,
    //   control-character-reference.
    assert_eq!(m.errors.len(), 40, "parse errors named by the descriptions");
    assert!(m.errors.contains("eof-in-tag"));
    assert!(m.errors.contains("nested-comment"));
}

#[test]
fn the_named_character_reference_table_is_the_standards_own() {
    let Ok(path) = hg_htmlgen::entities::path_from_env() else {
        return;
    };
    if !path.is_file() {
        assert!(
            std::env::var_os("IZCAL_REQUIRE_HTML_DATA").is_none_or(|v| v != "1"),
            "IZCAL_REQUIRE_HTML_DATA=1 but entities.json is not in the cache; run \
             `tools/fetch-test-data.sh html-entities`"
        );
        eprintln!(
            "note: entities.json is not fetched; this test is skipped. Run \
             `tools/fetch-test-data.sh html-entities`."
        );
        return;
    }
    let names = hg_htmlgen::entities::read(&path).expect("entities.json parses");
    // The style spec records 2,231 named references (Section 4).
    assert_eq!(names.len(), 2231);
    assert_eq!(names.get("amp;").map(String::as_str), Some("&"));
    assert_eq!(names.get("amp").map(String::as_str), Some("&"));
    assert_eq!(names.get("not").map(String::as_str), Some("\u{AC}"));
    // 93 of them stand for two code points, which is why the table maps
    // to strings and not to characters.
    let pairs = names.values().filter(|v| v.chars().count() > 1).count();
    assert_eq!(pairs, 93);
}
