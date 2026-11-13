//! `hg-html5lib`'s own tests: the suite parses, and the measured pass
//! count does not fall below the recorded baseline.
//!
//! Skipped with a note when the suite is not fetched;
//! `IZCAL_REQUIRE_HTML_DATA=1`, which the `html5lib` CI job sets, makes
//! a missing suite a failure instead, so the check cannot be skipped
//! where it must run.

use hg_html_syntax::Back;
use hg_html5lib::{BASELINE, Ledger, Outcome, cases, run_case, suite_dir};

fn suite() -> Option<Vec<hg_html5lib::Case>> {
    let dir = suite_dir().ok()?;
    if !dir.is_dir() {
        assert!(
            std::env::var_os("IZCAL_REQUIRE_HTML_DATA").is_none_or(|v| v != "1"),
            "IZCAL_REQUIRE_HTML_DATA=1 but html5lib-tests is not in the cache; run \
             `tools/fetch-test-data.sh html5lib`"
        );
        eprintln!(
            "note: html5lib-tests is not fetched; this test is skipped. Run \
             `tools/fetch-test-data.sh html5lib`."
        );
        return None;
    }
    Some(cases::read_dir(&dir).expect("the suite parses"))
}

fn measure(cases: &[hg_html5lib::Case]) -> Ledger {
    let mut ledger = Ledger::default();
    for case in cases {
        if case.lone_surrogate {
            ledger.lone_surrogate += 1;
        }
        for state in &case.initial_states {
            for back in [Back::Generated, Back::Table] {
                let got = run_case(case, *state, back);
                let outcome = if got.tokens == case.output {
                    if got.errors == case.errors {
                        Outcome::Pass
                    } else {
                        Outcome::ErrorsDiffer
                    }
                } else {
                    Outcome::TokensDiffer
                };
                ledger.record(&case.file, outcome, || case.description.clone());
            }
        }
    }
    ledger
}

#[test]
fn st_1_the_measured_pass_count_does_not_fall() {
    let Some(cases) = suite() else { return };
    let ledger = measure(&cases);
    print!("{}", ledger.render());
    assert!(
        ledger.pass >= BASELINE,
        "{} runs pass, below the recorded baseline of {BASELINE}. If this is deliberate, the \
         baseline moves in its own commit with the date.",
        ledger.pass
    );
}

#[test]
fn the_two_back_ends_agree_on_the_whole_suite() {
    // The generator's two back ends share the description and no code,
    // so a disagreement anywhere in 6,800 cases is a generator bug.
    let Some(cases) = suite() else { return };
    let mut differ = Vec::new();
    for case in &cases {
        for state in &case.initial_states {
            let a = run_case(case, *state, Back::Generated);
            let b = run_case(case, *state, Back::Table);
            if (a.tokens, a.errors) != (b.tokens, b.errors) && differ.len() < 10 {
                differ.push(format!("{} {:?}", case.file, case.description));
            }
        }
    }
    assert!(differ.is_empty(), "the back ends disagree on: {differ:?}");
}

#[test]
fn every_case_in_the_suite_is_read() {
    let Some(cases) = suite() else { return };
    // The suite's size moves with its pin; this is the number at the
    // pinned commit, so a pin that silently drops files is caught.
    assert_eq!(cases.len(), 6820, "cases at {}", hg_html5lib::PIN);
    assert!(cases.iter().all(|c| !c.initial_states.is_empty()));
}

#[test]
fn a_double_escaped_astral_character_survives_as_one_character() {
    // The review found this twice. First that `unescape_again` took
    // each escape on its own, so a surrogate pair became two U+FFFD on
    // the input *and* on the expected output, and the case would pass
    // while testing nothing. Then that the test written for it held a
    // literal emoji rather than the escapes, and so tested nothing
    // either. The assertions below are checked against the file's own
    // bytes by the one after them.
    use hg_html5lib::cases::unescape_again;
    assert_eq!(unescape_again(PAIR), "a\u{1F600}b");
    assert_eq!(unescape_again(LONE_HIGH), "a\u{FFFD}b");
    assert_eq!(unescape_again(LONE_LOW), "\u{FFFD}");
    assert_eq!(unescape_again(ASCII), "AB");
    assert_eq!(unescape_again("no escapes"), "no escapes");
    // The pair is one character, not two replacement characters.
    assert_eq!(unescape_again(PAIR).chars().count(), 3);
}

const PAIR: &str = r"a\uD83D\uDE00b";
const LONE_HIGH: &str = r"a\uD83Db";
const LONE_LOW: &str = r"\uDE00";
const ASCII: &str = r"\u0041\u0042";

#[test]
fn the_escapes_above_are_written_as_escapes() {
    // A literal astral character in the source would make the test
    // above vacuous in exactly the way it exists to prevent, and it
    // looks identical in a diff. So the source is read back.
    let source = include_str!("main.rs");
    assert!(source.contains(r#"const PAIR: &str = r"a\uD83D\uDE00b";"#));
    assert!(source.contains(r#"const ASCII: &str = r"\u0041\u0042";"#));
    assert!(
        PAIR.is_ascii(),
        "the pair is written as escapes, not as the character"
    );
    assert_eq!(PAIR.len(), 14, "a, 12 escape characters, b");
}
