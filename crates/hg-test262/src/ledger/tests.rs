//! Ledger tests: the text format round-trips, per-file combination,
//! directory and feature summaries, and comparison.

use super::*;
use crate::exclusions::Exclusions;

fn e(status: Status, mode: Mode, path: &str, features: &[&str], detail: &str) -> Entry {
    Entry {
        status,
        mode,
        path: path.into(),
        features: features.iter().map(|s| s.to_string()).collect(),
        detail: detail.into(),
    }
}

fn sample() -> Ledger {
    let mut l = Ledger {
        header: vec![
            ("engine".into(), "fake 1".into()),
            ("test262".into(), "abc".into()),
        ],
        entries: vec![
            e(
                Status::Fail,
                Mode::Strict,
                "built-ins/A/b.js",
                &["Symbol"],
                "uncaught\tTypeError:\nline 2 \\ x",
            ),
            e(
                Status::Pass,
                Mode::Sloppy,
                "built-ins/A/b.js",
                &["Symbol"],
                "",
            ),
            e(Status::Pass, Mode::Module, "language/m.js", &[], ""),
            e(
                Status::Excluded,
                Mode::Sloppy,
                "intl402/c.js",
                &["Intl"],
                "path intl402/",
            ),
            e(
                Status::Timeout,
                Mode::Sloppy,
                "built-ins/B/t.js",
                &[],
                "slow",
            ),
            e(
                Status::Pass,
                Mode::Sloppy,
                "language/neg.js",
                &[],
                "phase not reported; expected parse",
            ),
        ],
    };
    l.sort();
    l
}

#[test]
fn text_round_trip() {
    let l = sample();
    let text = l.to_text();
    assert!(text.starts_with(MAGIC));
    assert_eq!(Ledger::parse(&text).unwrap(), l);
    assert_eq!(l.field("test262"), Some("abc"));
    // Sorted by path, then mode.
    assert_eq!(l.entries[0].path, "built-ins/A/b.js");
    assert_eq!(l.entries[0].mode, Mode::Sloppy);
}

#[test]
fn refuses_bad_text() {
    assert!(Ledger::parse("not a ledger").is_err());
    assert!(
        Ledger::parse(&format!("{MAGIC}\npass\tstrict\tx.js"))
            .unwrap_err()
            .contains("5 tab-separated")
    );
    assert!(
        Ledger::parse(&format!("{MAGIC}\nok\tstrict\tx.js\t-\t"))
            .unwrap_err()
            .contains("unknown status")
    );
    assert!(
        Ledger::parse(&format!("{MAGIC}\npass\tloose\tx.js\t-\t"))
            .unwrap_err()
            .contains("unknown mode")
    );
}

#[test]
fn files_combine_runs() {
    let l = sample();
    let f = files(&l.entries);
    let status = |p: &str| f.iter().find(|r| r.path == p).unwrap().status;
    assert_eq!(f.len(), 5);
    assert_eq!(status("built-ins/A/b.js"), Status::Fail);
    assert_eq!(status("intl402/c.js"), Status::Excluded);
    assert_eq!(status("language/m.js"), Status::Pass);
    assert!(
        f.iter()
            .find(|r| r.path == "language/neg.js")
            .unwrap()
            .phase_unverified
    );
}

#[test]
fn summaries_by_directory_and_feature() {
    let l = sample();
    let d = by_dir(&l.entries, 1);
    assert_eq!(
        d["built-ins"],
        Counts {
            pass: 0,
            fail: 2,
            timeout: 1,
            excluded: 0,
            phase_unverified: 0
        }
    );
    assert_eq!(
        d["language"],
        Counts {
            pass: 2,
            fail: 0,
            timeout: 0,
            excluded: 0,
            phase_unverified: 1
        }
    );
    assert_eq!(d["intl402"].excluded, 1);
    let d2 = by_dir(&l.entries, 2);
    assert!(d2.contains_key("built-ins/A") && d2.contains_key("language"));
    let feat = by_feature(&l.entries);
    assert_eq!(feat["Symbol"].fail, 1);
    assert_eq!(feat["Intl"].excluded, 1);
    let (files, runs) = totals(&l.entries);
    assert_eq!((files.pass, files.fail, files.excluded), (2, 2, 1));
    assert_eq!((runs.pass, runs.fail, runs.excluded), (3, 2, 1));
    let t = table("Directory", &d);
    assert!(t.contains("| language | 2 | 2 | 0 (0) | 0 | 100.00% | 100.00% |"));
    assert!(t.contains("| **all** | 5 | 2 | 2 (1) | 1 | 50.00% | 40.00% |"));
    assert_eq!(dir_key("a/b/c.js", 5), "a/b");
    assert_eq!(dir_key("c.js", 1), "");
}

#[test]
fn exclusions_restate_a_ledger() {
    let mut l = sample();
    apply_exclusions(
        &mut l.entries,
        &Exclusions::parse("feature Symbol").unwrap(),
    );
    let (files, _) = totals(&l.entries);
    assert_eq!((files.pass, files.fail, files.excluded), (2, 1, 2));
}

#[test]
fn compare_finds_regressions_and_improvements() {
    let a = sample();
    let mut b = sample();
    for x in &mut b.entries {
        if x.path == "language/m.js" {
            x.status = Status::Fail;
        }
        if x.path == "built-ins/A/b.js" && x.mode == Mode::Strict {
            x.status = Status::Pass;
        }
    }
    b.entries.retain(|x| x.path != "built-ins/B/t.js");
    b.entries
        .push(e(Status::Pass, Mode::Sloppy, "new.js", &[], ""));
    let d = compare(&a, &b);
    assert_eq!(d.regressions.len(), 1);
    assert_eq!(d.regressions[0].path, "language/m.js");
    assert_eq!(d.improvements.len(), 1);
    assert_eq!(d.improvements[0].mode, Mode::Strict);
    assert_eq!(d.other.len(), 2);
    assert!(compare(&a, &a).regressions.is_empty());
}

#[test]
fn long_paths_survive_and_details_are_cut() {
    let long = format!("{}/x.js", "d".repeat(600));
    let l = Ledger {
        header: vec![("command".into(), "c".repeat(700))],
        entries: vec![e(Status::Fail, Mode::Sloppy, &long, &[], &"m".repeat(900))],
    };
    let back = Ledger::parse(&l.to_text()).unwrap();
    assert_eq!(back.entries[0].path, long);
    assert_eq!(back.header[0].1.len(), 700);
    assert_eq!(back.entries[0].detail.len(), MAX_DETAIL);
}
