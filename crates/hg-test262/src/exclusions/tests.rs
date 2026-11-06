//! Exclusion list tests.

use super::*;

#[test]
fn parses_and_matches() {
    let ex = Exclusions::parse(
        "# why\nfeature Atomics  # no global\npath test/intl402/\n\npath staging/x/",
    )
    .unwrap();
    assert_eq!(ex.entries.len(), 3);
    let f = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(
        ex.matches("built-ins/A/x.js", &f(&["Atomics"]))
            .unwrap()
            .to_string(),
        "feature Atomics"
    );
    assert_eq!(
        ex.matches("intl402/Collator/x.js", &[])
            .unwrap()
            .to_string(),
        "path intl402/"
    );
    assert!(ex.matches("built-ins/A/x.js", &f(&["Symbol"])).is_none());
    assert!(ex.matches("staging/y/z.js", &[]).is_none());
}

#[test]
fn refuses_bad_lines() {
    assert!(Exclusions::parse("feature").unwrap_err().contains("line 1"));
    assert!(Exclusions::parse("feature a b").is_err());
    assert!(
        Exclusions::parse("tag a")
            .unwrap_err()
            .contains("unknown entry kind")
    );
    assert!(
        Exclusions::parse("feature a\nfeature a")
            .unwrap_err()
            .contains("twice")
    );
}
