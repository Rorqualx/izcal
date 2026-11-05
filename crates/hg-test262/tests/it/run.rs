//! The runner end to end on the synthetic suite with the fake engine.

use std::path::PathBuf;
use std::time::Duration;

use hg_test262::classify::Status;
use hg_test262::exclusions::Exclusions;
use hg_test262::expand::Mode;
use hg_test262::ledger::{self, Ledger};
use hg_test262::runner::{self, RunConfig};

use crate::fake::Fake;

fn suite() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/it/t262")
}

fn config(filters: &[&str]) -> RunConfig {
    RunConfig {
        test262: suite(),
        filters: filters.iter().map(|s| s.to_string()).collect(),
        jobs: 3,
        timeout: Duration::from_secs(5),
        exclusions: Exclusions::parse("feature fake-missing-feature # the fake has none").unwrap(),
        progress: false,
    }
}

fn status(entries: &[ledger::Entry], path: &str, mode: Mode) -> Status {
    entries
        .iter()
        .find(|e| e.path == path && e.mode == mode)
        .unwrap_or_else(|| panic!("no run {path} [{mode}]"))
        .status
}

#[test]
fn discovery_skips_fixtures_and_filters_by_prefix() {
    let all = runner::discover(&suite(), &[]).unwrap();
    assert_eq!(all.len(), 16);
    assert!(all.iter().all(|p| !p.contains("_FIXTURE")));
    assert!(all.windows(2).all(|w| w[0] < w[1]), "sorted");
    let some = runner::discover(&suite(), &["neg/".into(), "raw".into()]).unwrap();
    assert_eq!(
        some,
        [
            "neg/no-error.js",
            "neg/parse.js",
            "neg/wrong-phase.js",
            "raw/r.js"
        ]
    );
}

#[test]
fn every_kind_of_test_is_run_and_classified() {
    let fake = Fake::default();
    let entries = runner::run(&config(&[]), &fake).unwrap();
    use Mode::*;
    use Status::*;
    let expect: &[(&str, Mode, Status)] = &[
        ("async/failure.js", Sloppy, Fail),
        ("async/missing.js", Strict, Fail),
        ("async/ok.js", Sloppy, Pass),
        ("async/ok.js", Strict, Pass),
        ("bad/malformed.js", Sloppy, Error),
        ("excluded/feat.js", Sloppy, Excluded),
        ("excluded/feat.js", Strict, Excluded),
        ("host/agent.js", Sloppy, Excluded),
        ("host/agent.js", Strict, Excluded),
        ("host/slow.js", Sloppy, Timeout),
        ("includes/inc.js", Sloppy, Pass),
        ("includes/inc.js", Strict, Pass),
        ("module/m.js", Module, Pass),
        ("neg/no-error.js", Sloppy, Fail),
        ("neg/parse.js", Sloppy, Pass),
        ("neg/parse.js", Strict, Pass),
        ("neg/wrong-phase.js", Sloppy, Fail),
        ("neg/wrong-phase.js", Strict, Fail),
        ("pass/only-strict.js", Strict, Pass),
        ("pass/simple.js", Sloppy, Pass),
        ("pass/simple.js", Strict, Pass),
        ("pass/strict-fails.js", Sloppy, Pass),
        ("pass/strict-fails.js", Strict, Fail),
        ("raw/r.js", Sloppy, Pass),
    ];
    let got: Vec<(&str, Mode, Status)> = entries
        .iter()
        .map(|e| (e.path.as_str(), e.mode, e.status))
        .collect();
    assert_eq!(got, expect);
    assert_eq!(status(&entries, "excluded/feat.js", Sloppy), Excluded);
    let detail = |p: &str| entries.iter().find(|e| e.path == p).unwrap().detail.clone();
    assert_eq!(detail("excluded/feat.js"), "feature fake-missing-feature");
    assert!(detail("host/agent.js").contains("$262.agent"));
    assert!(detail("bad/malformed.js").contains("unknown flag `noSuchFlag`"));
    assert!(detail("async/failure.js").contains("nope"));

    // Excluded tests never reach the engine; every other run does, once.
    let jobs = fake.jobs.lock().unwrap();
    let ran = entries
        .iter()
        .filter(|e| !matches!(e.status, Excluded | Error))
        .count();
    assert_eq!(jobs.len(), ran);
    let raw = jobs.iter().find(|j| j.path == "raw/r.js").unwrap();
    assert!(raw.raw && raw.harness.is_empty() && raw.prefix.is_empty());
    let strict = jobs
        .iter()
        .find(|j| j.path == "pass/simple.js" && !j.prefix.is_empty())
        .unwrap();
    assert!(strict.script_source().starts_with("\"use strict\";\n"));
    let module = jobs.iter().find(|j| j.path == "module/m.js").unwrap();
    assert!(module.prefix.is_empty() && module.harness.contains("H:sta"));
}

#[test]
fn ledger_summary_and_compare() {
    let entries = runner::run(&config(&[]), &Fake::default()).unwrap();
    let mut l = Ledger {
        header: vec![("engine".into(), "fake 1".into())],
        entries,
    };
    l.sort();
    let back = Ledger::parse(&l.to_text()).unwrap();
    assert_eq!(back, l);

    let dirs = ledger::by_dir(&l.entries, 1);
    assert_eq!((dirs["pass"].pass, dirs["pass"].fail), (2, 1));
    assert_eq!((dirs["neg"].pass, dirs["neg"].fail), (1, 2));
    assert_eq!(
        (
            dirs["host"].excluded,
            dirs["host"].fail,
            dirs["host"].timeout
        ),
        (1, 1, 1)
    );
    let (files, runs) = ledger::totals(&l.entries);
    assert_eq!((files.pass, files.fail, files.excluded), (7, 7, 2));
    assert_eq!((runs.pass, runs.fail, runs.excluded), (12, 8, 4));

    // Excluding a whole directory afterwards, then comparing.
    let mut restated = l.clone();
    ledger::apply_exclusions(
        &mut restated.entries,
        &Exclusions::parse("path pass/").unwrap(),
    );
    let diff = ledger::compare(&l, &restated);
    assert!(diff.regressions.is_empty() && diff.improvements.is_empty());
    assert_eq!(diff.other.len(), 5);
    let filtered = runner::run(&config(&["pass/"]), &Fake::default()).unwrap();
    let diff = ledger::compare(
        &l,
        &Ledger {
            header: vec![],
            entries: filtered,
        },
    );
    assert!(diff.regressions.is_empty());
    assert_eq!(
        diff.other.len(),
        l.entries.len() - 5,
        "runs absent from the filtered ledger"
    );
}
