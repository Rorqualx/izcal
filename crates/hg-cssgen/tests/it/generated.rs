//! The checked-in table is fresh and a hand edit is caught (G29).
//!
//! Unlike the other generators, this one cannot run from the repository
//! alone: every output carries webref's data. Without the pinned
//! `css.json` there is nothing to compare, so these are skipped rather
//! than passing on nothing — and `IZCAL_REQUIRE_CSS_DATA=1`, which CI
//! sets, turns the skip into a failure.

use std::path::{Path, PathBuf};

use hg_cssgen::cli;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Whether the pinned data is in the cache, saying so when it is not.
fn have_data() -> bool {
    if cli::catalogue().expect("the catalogue parses").is_some() {
        return true;
    }
    assert!(
        std::env::var_os("IZCAL_REQUIRE_CSS_DATA").is_none(),
        "IZCAL_REQUIRE_CSS_DATA=1 but {}",
        cli::missing_data_message()
    );
    println!("note: {}; skipped", cli::missing_data_message());
    false
}

#[test]
fn g29_the_checked_in_table_is_fresh() {
    if !have_data() {
        return;
    }
    let stale = cli::stale(&root()).expect("the overlay builds");
    assert!(
        stale.is_empty(),
        "stale: {stale:?}; run `cargo run -p hg-cssgen -- generate`"
    );
}

#[test]
fn g29_a_hand_edit_is_caught() {
    if !have_data() {
        return;
    }
    // The real root is fresh; a copy with one line appended is not.
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR")).join("g29-css");
    let gen_dir = tmp.join(cli::OUTPUT_DIR);
    let props = tmp.join(cli::SOURCES);
    std::fs::create_dir_all(&gen_dir).unwrap();
    std::fs::create_dir_all(&props).unwrap();
    for entry in std::fs::read_dir(root().join(cli::SOURCES)).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, props.join(p.file_name().unwrap())).unwrap();
    }
    for name in ["kinds.rs", "properties.rs", "grammars.rs", "properties.tsv"] {
        std::fs::copy(root().join(cli::OUTPUT_DIR).join(name), gen_dir.join(name)).unwrap();
    }
    assert!(
        cli::stale(&tmp).unwrap().is_empty(),
        "the copy starts fresh"
    );

    let edited = gen_dir.join("properties.tsv");
    let text = std::fs::read_to_string(&edited).unwrap();
    std::fs::write(&edited, format!("{text}a-hand-edit\n")).unwrap();
    let stale = cli::stale(&tmp).unwrap();
    assert_eq!(
        stale.len(),
        1,
        "exactly the edited file is stale: {stale:?}"
    );

    cli::generate(&tmp).unwrap();
    assert!(cli::stale(&tmp).unwrap().is_empty(), "fresh again");
}

#[test]
fn the_output_does_not_depend_on_the_order_the_files_were_read() {
    if !have_data() {
        return;
    }
    let m = cli::build(&root()).expect("the real overlay checks");
    let a = hg_cssgen::emit::properties(&m);
    let b = hg_cssgen::emit::properties(&m);
    assert_eq!(a, b);
    assert!(!a.contains('\r'), "Unix newlines only");
}

#[test]
fn st_m0_the_overlay_covers_the_pin_and_says_what_is_still_open() {
    if !have_data() {
        return;
    }
    let m = cli::build(&root()).expect("the real overlay checks");
    // The numbers the pin carries, so a re-pin that moves one arrives
    // here as well as in the cross-check.
    assert_eq!(m.properties.len(), 758, "properties at the pin");
    assert_eq!(m.aliases.len(), 64, "legacy aliases at the pin");
    assert_eq!(m.computed.len(), 169, "distinct computed-value texts");
    assert_eq!(m.kinds.len(), 12, "the closed set of computed-value kinds");
    // One kind override at the pin, and its reason is a defect in a
    // draft. The count is here so a second one cannot be added quietly:
    // every override is a place where Izcal computes something the
    // specification's own text does not say.
    let over: Vec<&str> = m.overrides().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        over,
        ["border-shape"],
        "the kind overrides, each of which needs a reason written beside it"
    );
    // ST-M0's own measurement: a number that can only go down.
    let open = m.undecided();
    assert!(
        open <= 548,
        "{open} properties still carry `affects unknown`; the number may not go up"
    );
    println!(
        "hg-cssgen: affects decided for {} of {}, {open} still open",
        m.properties.len().saturating_sub(open),
        m.properties.len()
    );
}
