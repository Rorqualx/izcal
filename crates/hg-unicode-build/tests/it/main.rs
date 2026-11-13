//! `hg-unicode-build`'s tests: the checked-in tables are fresh and a
//! hand edit is caught (G16, CS-ST-17), and TX-1's normalization half
//! runs against the pinned conformance file.
//!
//! The conformance run lives here rather than in `hg-unicode` because
//! reading a pinned data file is an ambient effect, and `hg-unicode` is
//! tier P. Every oracle in the tree sits in a tier T crate for the same
//! reason.

#[cfg(test)]
mod conformance;

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The paths, or `None` with a note when the database is not fetched.
/// With `IZCAL_REQUIRE_UNICODE_DATA=1` — which the `tx-1` CI job sets —
/// a missing database fails instead, so the check cannot be skipped
/// where it must run.
fn paths() -> Option<hg_unicode_build::Paths> {
    let p = hg_unicode_build::Paths::from_env(root()).ok()?;
    if p.ucd.is_dir() && p.idna.is_dir() {
        return Some(p);
    }
    assert!(
        std::env::var_os("IZCAL_REQUIRE_UNICODE_DATA").is_none_or(|v| v != "1"),
        "IZCAL_REQUIRE_UNICODE_DATA=1 but the Unicode database is not in the cache; run \
         `tools/fetch-test-data.sh unicode-ucd unicode-idna-mapping`"
    );
    eprintln!(
        "note: the Unicode database is not fetched; this test is skipped. Run \
         `tools/fetch-test-data.sh unicode-ucd unicode-idna-mapping`."
    );
    None
}

#[test]
fn g16_the_checked_in_tables_are_fresh_and_a_hand_edit_is_caught() {
    let Some(p) = paths() else { return };
    hg_unicode_build::check(&p).expect("the checked-in tables are fresh");

    // The same tree with one line appended must be refused, so the
    // check above is attributable to the content and not to the file
    // merely existing.
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR")).join("g16");
    let out = tmp.join(hg_unicode_build::OUTPUT);
    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    let real = root().join(hg_unicode_build::OUTPUT);
    let text = std::fs::read_to_string(&real).unwrap();
    std::fs::write(&out, format!("{text}// a hand edit\n")).unwrap();
    let edited = hg_unicode_build::Paths {
        root: tmp.clone(),
        ..p.clone()
    };
    let e = hg_unicode_build::check(&edited).expect_err("a hand edit is caught");
    assert!(e.contains("is stale"), "got: {e}");

    // And regenerating the copy makes it fresh again.
    hg_unicode_build::generate(&edited).unwrap();
    hg_unicode_build::check(&edited).expect("fresh after regenerating");
}

#[test]
fn the_generated_tables_are_deterministic_and_use_unix_newlines() {
    let Some(p) = paths() else { return };
    let a = hg_unicode_build::render(&p).unwrap();
    let b = hg_unicode_build::render(&p).unwrap();
    assert_eq!(a, b, "two renders of the same database differ");
    assert!(!a.contains('\r'), "the output carries a carriage return");
    assert!(
        !a.contains(env!("CARGO_MANIFEST_DIR")),
        "the output names a local absolute path"
    );
}

#[test]
fn the_pinned_version_is_the_one_the_spec_names() {
    assert_eq!(hg_unicode_build::VERSION, "18.0.0");
}
