//! `hg-html5lib`'s own tests: the suite parses, and the measured pass
//! count does not fall below the recorded baseline.
//!
//! Skipped with a note when the suite is not fetched;
//! `IZCAL_REQUIRE_HTML_DATA=1`, which the `html5lib` CI job sets, makes
//! a missing suite a failure instead, so the check cannot be skipped
//! where it must run.

mod encoding;

use hg_html_syntax::Back;
use hg_html5lib::{
    BASELINE, ExpectedError, Ledger, Outcome, cases, corrections, run_case, suite_dir,
};

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
    // ST-1 is a ship blocker at 100%, and with the suite's own defects
    // corrected (ST-D21) there is no ceiling below that, so the ratchet
    // above is no longer the strongest thing true: assert the whole of
    // it. A single failing run is then a finding on the day it appears
    // rather than a slow slide that stays above a baseline.
    assert_eq!(
        ledger.pass,
        hg_html5lib::RUNS,
        "ST-1 asks for 100%: {} of {} runs pass",
        ledger.pass,
        hg_html5lib::RUNS
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
    // A literal astral character in place of the escapes would make the
    // test above vacuous in exactly the way it exists to prevent, and
    // it looks identical in a diff. These two say it cannot be one:
    // `a` + twelve escape characters + `b`, all ASCII.
    //
    // An earlier version also read this file back and asked whether it
    // contained the constant's text. The review pointed out that the
    // needle appears in the assertion itself, so it matched whatever
    // the constant held. These two do the work.
    assert!(
        PAIR.is_ascii(),
        "the pair is written as escapes, not as the character"
    );
    assert_eq!(PAIR.len(), 14, "a, 12 escape characters, b");
    assert!(ASCII.is_ascii() && ASCII.len() == 12, "two escapes");
}

// --- ST-D21: the correction table --------------------------------------

/// The table is consistent with itself, whatever the data says.
#[test]
fn st_d21_the_table_is_well_formed() {
    corrections::check().expect("the table is consistent");
    assert_eq!(corrections::open_count(), 28, "open corrections");
    assert_eq!(
        corrections::by_class(corrections::Class::LintProven),
        9,
        "lint-proven: 8 series-2 inputs and case 500's"
    );
    assert_eq!(
        corrections::by_class(corrections::Class::SpecAndSuite),
        17,
        "spec-and-suite: 16 columns and case 500's second error"
    );
    assert_eq!(
        corrections::by_class(corrections::Class::SeriesInferred),
        2,
        "series-inferred: case 1174's input and description"
    );
}

/// A correction that no longer finds its value **fails the load**.
///
/// This is the whole discipline. A patch set whose hunks silently stop
/// applying is how a fork rots: upstream fixes the case, the patch does
/// nothing, and the tree still claims to be correcting something. Here
/// the next re-pin turns that into a message naming the case.
#[test]
fn st_d21_a_correction_that_no_longer_matches_is_a_finding() {
    let mut description = String::from("something else entirely");
    let mut input = String::from("<?a a");
    let mut errors = Vec::new();
    let e = corrections::apply(
        corrections::FILE,
        1175,
        corrections::Target {
            description: &mut description,
            input: &mut input,
            errors: &mut errors,
        },
    )
    .expect_err("case 1175 has one column correction, and this case has no errors");
    assert!(
        e.contains("no error 0"),
        "expected the missing-error message; got: {e}"
    );

    // And the input hunk, which does match, with the value changed.
    let mut input = String::from("<?a A");
    let mut errors = vec![ExpectedError {
        code: "eof-in-processing-instruction".into(),
        line: 1,
        col: 4,
    }];
    let e = corrections::apply(
        corrections::FILE,
        1175,
        corrections::Target {
            description: &mut description,
            input: &mut input,
            errors: &mut errors,
        },
    )
    .expect_err("the input is not what the table names");
    assert!(
        e.contains("expected \"<?a a\" and found \"<?a A\"") && e.contains("not a no-op"),
        "expected the mismatch message; got: {e}"
    );
}

/// The series-inferred hunks can be dropped without leaving a field
/// half-corrected: case 1174's column has a value under each gate, and
/// exactly one of them is live.
#[test]
fn st_d21_the_series_inferred_hunk_is_droppable() {
    let cols: Vec<_> = corrections::CORRECTIONS
        .iter()
        .filter(|c| c.index == 1174 && matches!(c.field, corrections::Field::ErrorCol { .. }))
        .collect();
    assert_eq!(cols.len(), 2, "one column value per gate");
    assert_ne!(
        cols[0].to, cols[1].to,
        "the two gates disagree, as they must"
    );
    assert_eq!(
        cols.iter().filter(|c| c.gate.open()).count(),
        1,
        "exactly one is live"
    );
}

/// The suite, as read, carries the corrected values — the table is not
/// merely present but applied.
#[test]
fn st_d21_the_suite_is_read_corrected() {
    let Some(cases) = suite() else { return };
    let t3: Vec<_> = cases
        .iter()
        .filter(|c| c.file == corrections::FILE)
        .collect();

    let by_input = |s: &str| t3.iter().filter(|c| c.input == s).count();
    assert_eq!(by_input("<?Z Z?"), 1, "case 1182 regained its ?");
    assert_eq!(
        by_input("<?Z Z"),
        1,
        "case 1174 lost the one that was not its"
    );

    // Every case that reaches the end of the file inside a processing
    // instruction reports it at the end of the input — the nine the
    // suite already had right and the sixteen corrected here, by the
    // same rule rather than by a list. Cases whose single error is
    // `invalid-first-character-of-processing-instruction-target` are
    // not these: that state consumed the offending character and
    // reports *its* position, which is why the sweep names the code.
    let mut eof_cases = 0;
    for c in &t3 {
        if c.errors.len() != 1 || c.errors[0].code != "eof-in-processing-instruction" {
            continue;
        }
        eof_cases += 1;
        let want = u32::try_from(c.input.chars().count() + 1).unwrap();
        assert_eq!(
            c.errors[0].col, want,
            "{:?}: eof-in-processing-instruction is at the end of the input",
            c.description
        );
    }
    assert_eq!(eof_cases, 25, "9 the suite had right and 16 corrected");
}
