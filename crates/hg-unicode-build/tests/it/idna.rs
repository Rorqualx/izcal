//! TX-1's `IdnaTestV2.txt`, over `hg-unicode`'s UTS #46.
//!
//! The file gives, per line, a source string and the result of three
//! operations on it: `toUnicode`, `toASCII` with Nontransitional
//! processing, and `toASCII` with Transitional processing, each with a
//! set of status codes. A blank result means "the same as the column
//! before"; `""` means the empty string.
//!
//! What is compared is the **string and whether there was an error**,
//! not which status codes were recorded. The codes name the step that
//! found the problem, and a conforming implementation may find the same
//! problem at a different step — the file's own conformance note asks
//! for "the resulting strings and relevant status values", and the
//! strings plus the presence of an error is what every operation here
//! actually promises.
//!
//! One exception the file states: a status of `X4_2` is about
//! `toUnicode` returning an empty label, which the file's own notes
//! call "issues" rather than errors.

use std::path::PathBuf;

use hg_unicode::idna::{Flags, to_ascii, to_unicode};

/// What the data set is called in `tools/test-data.toml`.
const DATA_SET: &str = "unicode-idna-test";
/// The pinned version.
const PIN: &str = "18.0.0";

fn data() -> Option<String> {
    let cache = match std::env::var_os("IZCAL_TEST_DATA") {
        Some(v) => PathBuf::from(v),
        None => {
            let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
            PathBuf::from(home).join(".cache/izcal/test-data")
        }
    };
    let path = cache
        .join(format!("{DATA_SET}-{PIN}"))
        .join("IdnaTestV2.txt");
    if !path.is_file() {
        assert!(
            std::env::var_os("IZCAL_REQUIRE_UNICODE_DATA").is_none(),
            "IZCAL_REQUIRE_UNICODE_DATA=1 but {} is not in the cache; run \
             `tools/fetch-test-data.sh {DATA_SET}`",
            path.display()
        );
        println!("note: IdnaTestV2.txt is not fetched; skipped");
        return None;
    }
    std::fs::read_to_string(&path).ok()
}

/// `\uXXXX` and `\x{XXXX}` escapes, which the file uses "where they
/// could otherwise have a confusing display".
fn unescape(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0_usize;
    while let Some(c) = chars.get(i).copied() {
        if c != '\\' {
            out.push(c);
            i = i.saturating_add(1);
            continue;
        }
        match chars.get(i.saturating_add(1)).copied() {
            Some('u') => {
                let hex: String = chars
                    .get(i.saturating_add(2)..i.saturating_add(6))
                    .unwrap_or(&[])
                    .iter()
                    .collect();
                match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    Some(v) => {
                        out.push(v);
                        i = i.saturating_add(6);
                    }
                    None => {
                        out.push(c);
                        i = i.saturating_add(1);
                    }
                }
            }
            Some('x') => {
                let rest: String = chars
                    .get(i.saturating_add(3)..)
                    .unwrap_or(&[])
                    .iter()
                    .collect();
                match rest.split_once('}') {
                    Some((hex, _)) => {
                        match u32::from_str_radix(hex, 16).ok().and_then(char::from_u32) {
                            Some(v) => {
                                out.push(v);
                                i = i.saturating_add(4).saturating_add(hex.chars().count());
                            }
                            None => {
                                out.push(c);
                                i = i.saturating_add(1);
                            }
                        }
                    }
                    None => {
                        out.push(c);
                        i = i.saturating_add(1);
                    }
                }
            }
            _ => {
                out.push(c);
                i = i.saturating_add(1);
            }
        }
    }
    out
}

/// A column's value: `""` is the empty string, blank is "the same as
/// the one before".
fn column(raw: &str, before: &str) -> String {
    let t = raw.trim();
    if t == "\"\"" {
        return String::new();
    }
    if t.is_empty() {
        return before.to_owned();
    }
    unescape(t)
}

/// The status codes this run does not count, and why.
///
/// The file's own rule: "Implementations that allow values of
/// particular input flags to be false would ignore the corresponding
/// status codes listed in the table below when testing for errors."
///
/// - `U1` is `UseSTD3ASCIIRules`, which is **off** here. It cannot be
///   on: with it on, `disallowed_STD3_mapped` code points are left
///   unmapped, and the file's expected *strings* are written for the
///   default. The URL Standard turns it on only under `beStrict`.
/// - `X4_2` is not a flag at all. The file's notes keep the `Xn` codes
///   apart as `toUnicode` *issues* — an empty label — rather than
///   errors.
///
/// Every other check is on: `CheckHyphens`, `CheckBidi`,
/// `CheckJoiners` and `VerifyDnsLength`, so `V2`, `V3`, `Bn`, `Cn`,
/// `A4_1` and `A4_2` are all counted.
const IGNORED: [&str; 2] = ["U1", "X4_2"];

/// Whether a status set records an error.
fn is_error(raw: &str) -> bool {
    let t = raw.trim();
    if t.is_empty() || t == "[]" {
        return false;
    }
    t.trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .any(|code| !IGNORED.contains(&code))
}

/// Whether a raw field names a code point Rust's `String` cannot
/// hold: an unpaired surrogate.
///
/// The file allows this: "Implementations that cannot work with
/// ill-formed strings would skip test cases that contain them. For
/// example, the status code A3 is set for a Punycode encoding error,
/// which may be due to an unpaired surrogate." They are counted rather
/// than dropped, so the number of checks is not quietly inflated.
fn has_lone_surrogate(raw: &str) -> bool {
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0_usize;
    let mut pending_high = false;
    while i < chars.len() {
        let is_escape = chars.get(i) == Some(&'\\') && chars.get(i.saturating_add(1)) == Some(&'u');
        if !is_escape {
            if pending_high {
                return true;
            }
            i = i.saturating_add(1);
            continue;
        }
        let hex: String = chars
            .get(i.saturating_add(2)..i.saturating_add(6))
            .unwrap_or(&[])
            .iter()
            .collect();
        let v = u32::from_str_radix(&hex, 16).unwrap_or(0);
        if pending_high {
            if !(0xDC00..0xE000).contains(&v) {
                return true;
            }
            pending_high = false;
        } else if (0xD800..0xDC00).contains(&v) {
            pending_high = true;
        } else if (0xDC00..0xE000).contains(&v) {
            return true;
        }
        i = i.saturating_add(6);
    }
    pending_high
}

struct Row {
    line: usize,
    source: String,
    unicode: String,
    unicode_error: bool,
    ascii_n: String,
    ascii_n_error: bool,
    ascii_t: String,
    ascii_t_error: bool,
}

fn rows(text: &str) -> (Vec<Row>, usize) {
    let mut out = Vec::new();
    let mut skipped = 0_usize;
    for (n, line) in text.lines().enumerate() {
        let body = line.split('#').next().unwrap_or("").trim();
        if body.is_empty() {
            continue;
        }
        let f: Vec<&str> = body.split(';').collect();
        let (Some(src), Some(uni), Some(unis), Some(an), Some(ans), Some(at), Some(ats)) = (
            f.first(),
            f.get(1),
            f.get(2),
            f.get(3),
            f.get(4),
            f.get(5),
            f.get(6),
        ) else {
            continue;
        };
        // Column 1 uses the same `""` convention as the rest.
        let source = column(src, "");
        let unicode = column(uni, &source);
        let unicode_status = unis.trim().to_owned();
        let ascii_n = column(an, &unicode);
        let ascii_n_status = if ans.trim().is_empty() {
            unicode_status.clone()
        } else {
            ans.trim().to_owned()
        };
        let ascii_t = column(at, &ascii_n);
        let ascii_t_status = if ats.trim().is_empty() {
            ascii_n_status.clone()
        } else {
            ats.trim().to_owned()
        };
        if [src, uni, an, at]
            .iter()
            .any(|f| has_lone_surrogate(f.trim()))
        {
            skipped = skipped.saturating_add(1);
            continue;
        }
        out.push(Row {
            line: n.saturating_add(1),
            source,
            unicode,
            unicode_error: is_error(&unicode_status),
            ascii_n,
            ascii_n_error: is_error(&ascii_n_status),
            ascii_t,
            ascii_t_error: is_error(&ascii_t_status),
        });
    }
    (out, skipped)
}

/// One checked operation.
struct Check {
    name: &'static str,
    want: String,
    want_error: bool,
    got: Option<String>,
    got_error: bool,
}

#[test]
fn tx_1_idna_test_v2() {
    let Some(text) = data() else { return };
    let (rows, skipped) = rows(&text);
    assert!(rows.len() > 5000, "only {} rows", rows.len());
    // Two rows at this pin, both holding an unpaired surrogate.
    assert_eq!(skipped, 2, "rows skipped for an unpaired surrogate");

    let mut fail: Vec<String> = Vec::new();
    let mut checks = 0_usize;
    for r in &rows {
        let uni = to_unicode(&r.source, Flags::conformance(false));
        // VerifyDnsLength is on: the file's `A4_1` and `A4_2` status
        // codes are ToASCII step 4, which is that check, and lines
        // 204 to 213 exist to exercise it.
        let an = to_ascii(&r.source, Flags::conformance(false), true);
        let at = to_ascii(&r.source, Flags::conformance(true), true);
        for c in [
            Check {
                name: "toUnicode",
                want: r.unicode.clone(),
                want_error: r.unicode_error,
                got: Some(uni.text.clone()),
                got_error: uni.error,
            },
            Check {
                name: "toAsciiN",
                want: r.ascii_n.clone(),
                want_error: r.ascii_n_error,
                got: an.clone(),
                got_error: an.is_none(),
            },
            Check {
                name: "toAsciiT",
                want: r.ascii_t.clone(),
                want_error: r.ascii_t_error,
                got: at.clone(),
                got_error: at.is_none(),
            },
        ] {
            checks = checks.saturating_add(1);
            if c.got_error != c.want_error {
                if fail.len() < 25 {
                    fail.push(format!(
                        "line {}: {} {:?}: error {} wanted {}",
                        r.line, c.name, r.source, c.got_error, c.want_error
                    ));
                }
                continue;
            }
            // When both agree there is an error, the converted string
            // is not compared: the standard only promises to convert
            // "as much as is possible".
            if c.want_error {
                continue;
            }
            if c.got.as_deref() != Some(c.want.as_str()) && fail.len() < 25 {
                fail.push(format!(
                    "line {}: {} {:?}: got {:?} wanted {:?}",
                    r.line, c.name, r.source, c.got, c.want
                ));
            }
        }
    }
    println!(
        "hg-unicode: IdnaTestV2 {} rows ({skipped} skipped for an unpaired surrogate), \
         {checks} checks, {} differences",
        rows.len(),
        fail.len()
    );
    for f in &fail {
        println!("  {f}");
    }
    assert!(fail.is_empty(), "{} of {checks} checks differ", fail.len());
}
