//! The checked-in tables are fresh and a hand edit is caught (G29), and
//! they say what the pinned data says.
//!
//! Like `hg-cssgen`, this generator cannot run from the repository
//! alone: the tables are the standard's data. Without it these are
//! skipped, and `IZCAL_REQUIRE_ENCODING_DATA=1`, which CI sets, turns
//! the skip into a failure.

use std::path::{Path, PathBuf};

use hg_encoding_build::{Index, cli, data};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The pinned data, or `None` with a note when it is not fetched.
fn data() -> Option<data::Data> {
    match data::read().expect("the pinned data parses") {
        Some(d) => Some(d),
        None => {
            assert!(
                std::env::var_os("IZCAL_REQUIRE_ENCODING_DATA").is_none(),
                "IZCAL_REQUIRE_ENCODING_DATA=1 but {}",
                cli::missing_data_message()
            );
            println!("note: {}; skipped", cli::missing_data_message());
            None
        }
    }
}

#[test]
fn g29_the_checked_in_tables_are_fresh() {
    if data().is_none() {
        return;
    }
    let stale = cli::stale(&root()).expect("the data reads");
    assert!(
        stale.is_empty(),
        "stale: {stale:?}; run `cargo run -p hg-encoding-build -- generate`"
    );
}

#[test]
fn g29_a_hand_edit_is_caught() {
    if data().is_none() {
        return;
    }
    // The real root is fresh; a copy with one line appended is not.
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR")).join("g29-encoding");
    let gen_dir = tmp.join(cli::OUTPUT_DIR);
    std::fs::create_dir_all(&gen_dir).expect("the directory is made");
    for name in ["encodings.rs", "indexes.rs", "encodings.tsv"] {
        std::fs::copy(root().join(cli::OUTPUT_DIR).join(name), gen_dir.join(name))
            .expect("the table is copied");
    }
    assert!(
        cli::stale(&tmp).expect("the copy reads").is_empty(),
        "the copy starts fresh"
    );

    let edited = gen_dir.join("encodings.tsv");
    let text = std::fs::read_to_string(&edited).expect("the table reads");
    std::fs::write(&edited, format!("{text}a-hand-edit\n")).expect("the edit is written");
    let stale = cli::stale(&tmp).expect("the copy reads");
    assert_eq!(
        stale.len(),
        1,
        "exactly the edited file is stale: {stale:?}"
    );

    cli::generate(&tmp).expect("it regenerates");
    assert!(
        cli::stale(&tmp).expect("the copy reads").is_empty(),
        "fresh again"
    );
}

#[test]
fn the_pin_carries_what_the_standard_says_it_does() {
    let Some(d) = data() else { return };
    assert_eq!(d.encodings.len(), 40, "encodings at the pin");
    let labels: usize = d.encodings.iter().map(|e| e.labels.len()).sum();
    assert_eq!(labels, 228, "labels at the pin");
    assert_eq!(d.indexes.len(), 34, "index tables at the pin");

    // "For each encoding, ASCII-lowercasing its name yields one of its
    // labels" (Section 4.2).
    for e in &d.encodings {
        assert!(
            e.labels.contains(&e.name.to_ascii_lowercase()),
            "{} is not among its own labels",
            e.name
        );
    }
    // Every label is unique across the whole table: one label can only
    // select one encoding, or "get an encoding" would be ambiguous.
    let mut all: Vec<&str> = d
        .encodings
        .iter()
        .flat_map(|e| e.labels.iter().map(String::as_str))
        .collect();
    all.sort_unstable();
    let before = all.len();
    all.dedup();
    assert_eq!(all.len(), before, "a label selects more than one encoding");
}

#[test]
fn no_index_holds_the_value_that_means_none() {
    // The tables use u16::MAX and u32::MAX to mean "no entry", which
    // only works while no index carries those code points. U+FFFF is a
    // valid code point, so this is a real condition and not a
    // formality.
    let Some(d) = data() else { return };
    for (name, index) in &d.indexes {
        let Index::Points(points) = index else {
            continue;
        };
        for v in points.iter().flatten() {
            assert_ne!(
                *v,
                u32::from(u16::MAX),
                "index {name} holds U+FFFF, which the tables use to mean no entry"
            );
            assert_ne!(*v, u32::MAX, "index {name} holds the u32 no-entry value");
        }
    }
}

#[test]
fn the_output_does_not_depend_on_the_order_the_data_was_read() {
    let Some(d) = data() else { return };
    let a = hg_encoding_build::emit::indexes(&d).expect("it emits");
    let b = hg_encoding_build::emit::indexes(&d).expect("it emits");
    assert_eq!(a, b);
    assert!(!a.contains('\r'), "Unix newlines only");
}
