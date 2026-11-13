//! Classification, and what the pinned suite holds.

use hg_wpt_layout::{
    FILES, HARNESS, Kind, LAYOUT_TESTS, MANUAL, REFTESTS, RUNNABLE_BASELINE, SUPPORT, classify,
    measure, read_suite, suite_dir,
};

#[test]
fn p01_a_layout_test_is_one_with_machine_readable_expectations() {
    // The kind that needs no rendering: the used values are in the
    // markup.
    let e = classify(
        "normal-flow/auto-margins.html",
        r#"<link rel="help" href="x">
           <script src="/resources/check-layout-th.js"></script>
           <div data-expected-margin-left="25" data-expected-margin-right="25"></div>"#,
    );
    assert_eq!(e.kind, Kind::Layout);
    assert_eq!(e.expectations, 2);
    assert_eq!(e.chapter, "normal-flow");
}

#[test]
fn p02_a_reftest_is_one_with_a_match_or_mismatch_link() {
    for link in [
        r#"<link rel="match" href="a-ref.xht">"#,
        "<link rel='match' href='a-ref.xht'>",
        r#"<link rel="mismatch" href="a-notref.xht">"#,
    ] {
        let e = classify("floats/float-001.xht", link);
        assert_eq!(e.kind, Kind::Reftest, "{link}");
    }
}

#[test]
fn p03_a_reference_is_not_a_test() {
    for name in [
        "floats/float-001-ref.xht",
        "floats/float-001-notref.html",
        "floats/support/thing.html",
        "support/thing.html",
        "floats/thing.png",
        "floats/thing.css",
    ] {
        assert_eq!(classify(name, "").kind, Kind::Support, "{name}");
    }
}

#[test]
fn p04_a_harness_test_without_expectations_needs_a_script_runtime() {
    let e = classify(
        "syntax/a.html",
        r#"<script src="/resources/testharness.js"></script><script>test(() => {});</script>"#,
    );
    assert_eq!(e.kind, Kind::Harness);
    assert_eq!(e.expectations, 0);
}

#[test]
fn p05_everything_else_is_self_describing_and_needs_a_person() {
    let e = classify(
        "backgrounds/background-001.xht",
        "<p>Test passes if there is a green square.</p>",
    );
    assert_eq!(e.kind, Kind::Manual);
    assert_eq!(Kind::Manual.needs(), "a person");
}

#[test]
fn p06_a_match_link_wins_over_expectations() {
    // A reftest that also carries expectations still needs a raster,
    // so the stricter requirement is the one reported.
    let e = classify(
        "a/b.html",
        r#"<link rel="match" href="r.html"><div data-expected-width="1"></div>"#,
    );
    assert_eq!(e.kind, Kind::Reftest);
    assert_eq!(e.expectations, 1, "the attributes are still counted");
}

/// The pinned suite, when it is in the cache.
fn suite() -> Option<Vec<hg_wpt_layout::Entry>> {
    let dir = suite_dir().ok()?;
    if !dir.is_dir() {
        eprintln!(
            "skipping: {} is not there; run `tools/fetch-test-data.sh wpt-css2`",
            dir.display()
        );
        return None;
    }
    read_suite(&dir).ok()
}

#[test]
fn p07_the_pinned_suite_holds_what_was_measured() {
    let Some(entries) = suite() else {
        return;
    };
    let l = measure(&entries);
    assert_eq!(l.files, FILES);
    assert_eq!(l.by_kind.get("layout").copied().unwrap_or(0), LAYOUT_TESTS);
    assert_eq!(l.by_kind.get("reftest").copied().unwrap_or(0), REFTESTS);
    assert_eq!(l.by_kind.get("harness").copied().unwrap_or(0), HARNESS);
    assert_eq!(l.by_kind.get("manual").copied().unwrap_or(0), MANUAL);
    assert_eq!(l.by_kind.get("support").copied().unwrap_or(0), SUPPORT);
}

#[test]
fn p08_the_runnable_count_is_the_baseline_and_says_what_it_waits_on() {
    let Some(entries) = suite() else {
        return;
    };
    let l = measure(&entries);
    // Zero, and honestly so: every kind needs an HTML tree builder,
    // which is ST-M1's. The number is computed from that fact rather
    // than written down, so it moves on its own.
    assert_eq!(l.runnable, RUNNABLE_BASELINE);
    // And there is something for it to move to: the layout tests are
    // measured from the suite, not written down.
    assert_eq!(l.by_kind.get("layout").copied().unwrap_or(0), LAYOUT_TESTS);
}

#[test]
fn p08a_the_suite_still_classifies_as_recorded() {
    // The guard a zero baseline cannot be: a change here is either
    // the pin moving or the classifier drifting.
    let Some(entries) = suite() else {
        return;
    };
    let drift = hg_wpt_layout::check(&measure(&entries));
    assert!(drift.is_empty(), "{drift:?}");
}

#[test]
fn p09_every_chapter_holds_tests_and_the_counts_add_up() {
    let Some(entries) = suite() else {
        return;
    };
    let l = measure(&entries);
    let tests: usize = l.by_chapter.values().sum();
    let support = l.by_kind.get("support").copied().unwrap_or(0);
    assert_eq!(tests.saturating_add(support), l.files);
    assert!(l.by_chapter.len() > 20, "the suite has its chapters");
    assert!(l.by_chapter.contains_key("normal-flow"));
    assert!(l.by_chapter.contains_key("floats"));
}
