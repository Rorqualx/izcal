//! TX-1's normalization half: `NormalizationTest.txt` at 100%.
//!
//! The file's own header states the conditions, and they are checked
//! here as written. For each line `c1;c2;c3;c4;c5`:
//!
//! ```text
//! NFC:  c2 == toNFC(c1)  == toNFC(c2)  == toNFC(c3)
//!       c4 == toNFC(c4)  == toNFC(c5)
//! NFD:  c3 == toNFD(c1)  == toNFD(c2)  == toNFD(c3)
//!       c5 == toNFD(c4)  == toNFD(c5)
//! NFKC: c4 == toNFKC(c1) == toNFKC(c2) == toNFKC(c3) == toNFKC(c4) == toNFKC(c5)
//! NFKD: c5 == toNFKD(c1) == toNFKD(c2) == toNFKD(c3) == toNFKD(c4) == toNFKD(c5)
//! ```
//!
//! And the invariance condition: every code point **not** listed in
//! `@Part1` is unchanged by all four forms. That half matters more than
//! the lines do — it covers the whole code space, so a table that
//! wrongly maps some character cannot hide in the gaps between the
//! file's own cases.
//!
//! The data is pinned (`tools/test-data.toml`, `unicode-ucd`) and never
//! committed. Without it the test says so and passes, as the `ast-grep`
//! checks do; with `IZCAL_REQUIRE_UNICODE_DATA=1` — which the `tx-1` CI
//! job sets — a missing file fails instead, so the oracle cannot be
//! skipped where it must run.

use std::collections::BTreeSet;
use std::path::PathBuf;

use hg_unicode::{nfc, nfd, nfkc, nfkd};

/// One conformance check: the form's name, the column it must equal, the
/// column it is applied to, and the function.
type Case<'a> = (&'static str, &'a str, &'a str, fn(&str) -> String);

/// The pinned Unicode version, which must match the generator's.
const VERSION: &str = "18.0.0";

fn data_file(name: &str) -> Option<PathBuf> {
    let cache = match std::env::var_os("IZCAL_TEST_DATA") {
        Some(v) => PathBuf::from(v),
        None => PathBuf::from(std::env::var_os("HOME")?).join(".cache/izcal/test-data"),
    };
    let p = cache.join(format!("unicode-ucd-{VERSION}")).join(name);
    p.is_file().then_some(p)
}

/// Reads the pinned file, or `None` with a note when it is not fetched.
fn read(name: &str) -> Option<String> {
    match data_file(name) {
        Some(p) => std::fs::read_to_string(&p).ok(),
        None => {
            assert!(
                std::env::var_os("IZCAL_REQUIRE_UNICODE_DATA").is_none_or(|v| v != "1"),
                "IZCAL_REQUIRE_UNICODE_DATA=1 but {name} is not in the cache; run \
                 `tools/fetch-test-data.sh unicode-ucd`"
            );
            eprintln!(
                "note: {name} is not fetched; this test is skipped. \
                 Run `tools/fetch-test-data.sh unicode-ucd`."
            );
            None
        }
    }
}

fn parse(field: &str) -> String {
    field
        .split_whitespace()
        .filter_map(|t| u32::from_str_radix(t, 16).ok())
        .filter_map(char::from_u32)
        .collect()
}

struct Line {
    part: u8,
    cols: [String; 5],
    raw: String,
    number: usize,
}

fn lines(text: &str) -> Vec<Line> {
    let mut part = 0_u8;
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        if let Some(rest) = raw.strip_prefix("@Part") {
            part = rest
                .chars()
                .next()
                .and_then(|c| c.to_digit(10))
                .and_then(|d| u8::try_from(d).ok())
                .unwrap_or(part);
            continue;
        }
        let body = raw.split('#').next().unwrap_or("").trim();
        if body.is_empty() {
            continue;
        }
        let f: Vec<&str> = body.split(';').collect();
        if f.len() < 5 {
            continue;
        }
        out.push(Line {
            part,
            cols: [
                parse(f[0]),
                parse(f[1]),
                parse(f[2]),
                parse(f[3]),
                parse(f[4]),
            ],
            raw: body.to_owned(),
            number: i + 1,
        });
    }
    out
}

#[test]
fn tx_1_normalization_test_passes_on_every_line() {
    let Some(text) = read("NormalizationTest.txt") else {
        return;
    };
    let all = lines(&text);
    assert!(
        all.len() > 15_000,
        "only {} lines parsed; is the pinned file the right one?",
        all.len()
    );

    let mut checks = 0_usize;
    let mut failures: Vec<String> = Vec::new();
    for l in &all {
        let [c1, c2, c3, c4, c5] = &l.cols;
        let cases: [Case<'_>; 18] = [
            ("NFC", c2, c1, nfc),
            ("NFC", c2, c2, nfc),
            ("NFC", c2, c3, nfc),
            ("NFC", c4, c4, nfc),
            ("NFC", c4, c5, nfc),
            ("NFD", c3, c1, nfd),
            ("NFD", c3, c2, nfd),
            ("NFD", c3, c3, nfd),
            ("NFD", c5, c4, nfd),
            ("NFD", c5, c5, nfd),
            ("NFKC", c4, c1, nfkc),
            ("NFKC", c4, c2, nfkc),
            ("NFKC", c4, c3, nfkc),
            ("NFKC", c4, c4, nfkc),
            ("NFKC", c4, c5, nfkc),
            ("NFKD", c5, c1, nfkd),
            ("NFKD", c5, c2, nfkd),
            ("NFKD", c5, c3, nfkd),
        ];
        let tail: [Case<'_>; 2] = [("NFKD", c5, c4, nfkd), ("NFKD", c5, c5, nfkd)];
        for (form, want, input, f) in cases.into_iter().chain(tail) {
            checks += 1;
            let got = f(input);
            if got != want && failures.len() < 20 {
                failures.push(format!(
                    "line {} ({}): to{form}({}) = {}, want {}",
                    l.number,
                    l.raw,
                    hex(input),
                    hex(&got),
                    hex(want)
                ));
            } else if got != want {
                failures.push(String::new());
            }
        }
    }
    let named: Vec<&String> = failures.iter().filter(|f| !f.is_empty()).collect();
    assert!(
        failures.is_empty(),
        "{} of {checks} checks failed over {} lines; first {}:\n{}",
        failures.len(),
        all.len(),
        named.len(),
        named
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    println!(
        "TX-1 NormalizationTest: {checks} checks over {} lines, 0 failed",
        all.len()
    );
}

#[test]
fn tx_1_every_code_point_outside_part1_is_unchanged() {
    let Some(text) = read("NormalizationTest.txt") else {
        return;
    };
    let listed: BTreeSet<char> = lines(&text)
        .iter()
        .filter(|l| l.part == 1)
        .filter_map(|l| l.cols[0].chars().next())
        .collect();
    assert!(
        listed.len() > 1_000,
        "only {} code points in @Part1; is the pinned file the right one?",
        listed.len()
    );

    let mut checked = 0_usize;
    let mut failures = Vec::new();
    for cp in 0_u32..=0x10_FFFF {
        let Some(c) = char::from_u32(cp) else {
            continue;
        };
        if listed.contains(&c) {
            continue;
        }
        let s = c.to_string();
        checked += 1;
        for (form, f) in [
            ("NFC", nfc as fn(&str) -> String),
            ("NFD", nfd),
            ("NFKC", nfkc),
            ("NFKD", nfkd),
        ] {
            let got = f(&s);
            if got != s && failures.len() < 20 {
                failures.push(format!(
                    "U+{cp:04X}: to{form}() = {}, want itself",
                    hex(&got)
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} code points outside @Part1 changed (of {checked} checked):\n{}",
        failures.len(),
        failures.join("\n")
    );
    println!("TX-1 invariance: {checked} code points outside @Part1, 0 changed");
}

fn hex(s: &str) -> String {
    s.chars()
        .map(|c| format!("{:04X}", u32::from(c)))
        .collect::<Vec<_>>()
        .join(" ")
}
