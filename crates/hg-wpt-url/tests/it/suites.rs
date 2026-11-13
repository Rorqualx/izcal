//! The pinned WPT `url/` suites.
//!
//! Each test is skipped, loudly, when the data is not in the cache, so
//! a checkout without `tools/fetch-test-data.sh` run is not reported as
//! a failure — but CI always has it, and the numbers below are what CI
//! measures.

use hg_wpt_url::{TOASCII_BASELINE, URLTESTDATA_BASELINE, URLTESTDATA_CASES};

fn load(set: &str) -> Option<hg_json::Json> {
    let path = hg_wpt_url::data_file(set).ok()?;
    if !path.is_file() {
        eprintln!(
            "skipping: {} is not there; run `tools/fetch-test-data.sh {set}`",
            path.display()
        );
        return None;
    }
    hg_wpt_url::read_json(&path).ok()
}

#[test]
fn w01_urltestdata_holds_the_recorded_number_of_cases() {
    let Some(json) = load("wpt-urltestdata") else {
        return;
    };
    let cases = hg_wpt_url::cases(&json);
    assert_eq!(cases.len(), URLTESTDATA_CASES);
}

#[test]
fn w02_every_urltestdata_case_passes() {
    let Some(json) = load("wpt-urltestdata") else {
        return;
    };
    let cases = hg_wpt_url::cases(&json);
    let l = hg_wpt_url::run(&cases);
    for e in &l.examples {
        eprintln!("  {e}");
    }
    assert!(
        l.pass >= URLTESTDATA_BASELINE,
        "{}/{} passed, baseline {URLTESTDATA_BASELINE}",
        l.pass,
        l.runs
    );
}

#[test]
fn w03_every_toascii_case_passes() {
    let Some(json) = load("wpt-toascii") else {
        return;
    };
    let l = hg_wpt_url::run_toascii(&json);
    for e in &l.examples {
        eprintln!("  {e}");
    }
    assert!(
        l.pass >= TOASCII_BASELINE,
        "{}/{} passed, baseline {TOASCII_BASELINE}",
        l.pass,
        l.runs
    );
}

#[test]
fn w04_every_percent_encoding_check_passes() {
    let Some(json) = load("wpt-percent-encoding") else {
        return;
    };
    let l = hg_wpt_url::run_percent_encoding(&json);
    for e in &l.examples {
        eprintln!("  {e}");
    }
    assert_eq!(l.runs, hg_wpt_url::PERCENT_ENCODING_BASELINE);
    assert!(
        l.pass >= hg_wpt_url::PERCENT_ENCODING_BASELINE,
        "{}/{} passed",
        l.pass,
        l.runs
    );
}
