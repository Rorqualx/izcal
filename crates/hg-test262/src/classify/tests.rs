//! Classification tests (INTERPRETING.md, "Test Results", `negative`,
//! `async`).

use super::*;
use crate::frontmatter::Phase;

fn out(completion: Completion, prints: &[&str]) -> Outcome {
    Outcome {
        completion,
        prints: prints.iter().map(|s| s.to_string()).collect(),
        diagnostics: String::new(),
    }
}

fn threw(phase: Option<Phase>, name: &str) -> Completion {
    Completion::Threw {
        phase,
        name: name.into(),
        message: "m".into(),
    }
}

fn neg(phase: Phase, ty: &str) -> Negative {
    Negative {
        phase,
        error_type: ty.into(),
    }
}

#[test]
fn positive_tests() {
    assert_eq!(
        classify(None, false, &out(Completion::Normal, &[])).status,
        Status::Pass
    );
    let v = classify(
        None,
        false,
        &out(threw(Some(Phase::Runtime), "Test262Error"), &[]),
    );
    assert_eq!(v.status, Status::Fail);
    assert!(v.detail.contains("Test262Error"));
    assert_eq!(
        classify(None, false, &out(Completion::Timeout, &[])).status,
        Status::Timeout
    );
    assert_eq!(
        classify(None, false, &out(Completion::Crashed("x".into()), &[])).status,
        Status::Crash
    );
}

#[test]
fn negative_tests_match_name_and_phase() {
    let n = neg(Phase::Parse, "SyntaxError");
    assert_eq!(
        classify(
            Some(&n),
            false,
            &out(threw(Some(Phase::Parse), "SyntaxError"), &[])
        )
        .status,
        Status::Pass
    );
    // Right error, wrong phase.
    let v = classify(
        Some(&n),
        false,
        &out(threw(Some(Phase::Runtime), "SyntaxError"), &[]),
    );
    assert_eq!(v.status, Status::Fail);
    assert!(v.detail.contains("phase"));
    // Wrong error.
    assert_eq!(
        classify(
            Some(&n),
            false,
            &out(threw(Some(Phase::Parse), "ReferenceError"), &[])
        )
        .status,
        Status::Fail
    );
    // No error.
    assert_eq!(
        classify(Some(&n), false, &out(Completion::Normal, &[])).status,
        Status::Fail
    );
    // `$DONOTEVALUATE` ran: the harness's own error escaped instead.
    assert_eq!(
        classify(
            Some(&n),
            false,
            &out(threw(Some(Phase::Runtime), "Test262Error"), &[])
        )
        .status,
        Status::Fail
    );
    // A resolution error is not a parse error.
    let r = neg(Phase::Resolution, "SyntaxError");
    assert_eq!(
        classify(
            Some(&r),
            false,
            &out(threw(Some(Phase::Parse), "SyntaxError"), &[])
        )
        .status,
        Status::Fail
    );
    assert_eq!(
        classify(
            Some(&r),
            false,
            &out(threw(Some(Phase::Resolution), "SyntaxError"), &[])
        )
        .status,
        Status::Pass
    );
}

#[test]
fn unknown_phase_passes_on_the_name_and_says_so() {
    let n = neg(Phase::Parse, "SyntaxError");
    let v = classify(Some(&n), false, &out(threw(None, "SyntaxError"), &[]));
    assert_eq!(v.status, Status::Pass);
    assert!(v.phase_unverified);
    assert!(v.detail.starts_with(PHASE_UNVERIFIED));
    assert_eq!(
        classify(Some(&n), false, &out(threw(None, "TypeError"), &[])).status,
        Status::Fail
    );
}

#[test]
fn async_protocol() {
    assert_eq!(
        classify(None, true, &out(Completion::Normal, &["x", ASYNC_COMPLETE])).status,
        Status::Pass
    );
    let v = classify(None, true, &out(Completion::Normal, &[]));
    assert_eq!(v.status, Status::Fail);
    assert!(v.detail.contains("without printing"));
    let v = classify(
        None,
        true,
        &out(
            Completion::Normal,
            &["Test262:AsyncTestFailure:Test262Error: no"],
        ),
    );
    assert_eq!(v.status, Status::Fail);
    assert!(v.detail.contains("Test262Error: no"));
    // A failure report wins over a completion report.
    let fail_then_ok = ["Test262:AsyncTestFailure:x", ASYNC_COMPLETE];
    assert_eq!(
        classify(None, true, &out(Completion::Normal, &fail_then_ok)).status,
        Status::Fail
    );
    // A synchronous throw fails an async test.
    assert_eq!(
        classify(
            None,
            true,
            &out(threw(Some(Phase::Runtime), "TypeError"), &[ASYNC_COMPLETE])
        )
        .status,
        Status::Fail
    );
    // An async negative test is decided by the error, as any negative test.
    let n = neg(Phase::Parse, "SyntaxError");
    assert_eq!(
        classify(
            Some(&n),
            true,
            &out(threw(Some(Phase::Parse), "SyntaxError"), &[])
        )
        .status,
        Status::Pass
    );
}
