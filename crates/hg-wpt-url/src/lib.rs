//! `hg-wpt-url`: the WPT `url/` suite, run against [`hg_url`].
//!
//! Three pinned files, each a different question:
//!
//! - `urltestdata.json` — 904 cases, each an input and a base, with
//!   either the eight serialized components a conforming parser must
//!   produce or `failure: true`. This is the oracle for the parser as
//!   a whole.
//! - `toascii.json` — the domain parser alone, which is UTS 46 with
//!   the URL Standard's flags.
//! - `percent-encoding.json` — "percent-encode after encoding", the
//!   one place a legacy encoding reaches a URL.
//!
//! The suite is **test-only** (BSD-3-Clause, OD-L2): fetched at a
//! pinned commit with a SHA-256 check, never committed and never
//! linked into anything shipped. Reading it is an ambient effect, so
//! the runner is tier T and `hg-url` itself never opens a file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use hg_json::Json;
use hg_url::{Host, Url};

/// The pinned WPT commit.
pub const PIN: &str = "33926aca0f4313541f0ac5d67d4a4ab949d50b16";

/// The data-set names in `tools/test-data.toml`.
pub const SETS: [(&str, &str); 3] = [
    ("wpt-urltestdata", "urltestdata.json"),
    ("wpt-toascii", "toascii.json"),
    ("wpt-percent-encoding", "percent-encoding.json"),
];

/// How many `urltestdata.json` entries are cases rather than comments.
pub const URLTESTDATA_CASES: usize = 904;

/// The number of `urltestdata.json` cases that must pass.
///
/// The suite is the exit criterion's own measure, so the baseline is
/// the whole suite: every case, with no exclusions. A case that cannot
/// pass would have to be named here with its reason, and none is.
pub const URLTESTDATA_BASELINE: usize = 904;

/// The number of `toascii.json` cases that must pass: all 87.
pub const TOASCII_BASELINE: usize = 87;

/// The number of `percent-encoding.json` checks that must pass.
///
/// Seven inputs, each carrying only the encodings its own entry lists
/// — **16** checks in all, not 7 × 8. They are chosen for what they
/// catch: `\u{0E}A` in ISO-2022-JP needs the stateful encoder, U+E5E5
/// is gb18030's asymmetric pair, and `†` in Big5 is the unmappable
/// case that becomes `%26%238224%3B`.
pub const PERCENT_ENCODING_BASELINE: usize = 16;

/// How many `percent-encoding.json` entries are cases.
pub const PERCENT_ENCODING_CASES: usize = 7;

/// Where a pinned data file lives in the cache.
///
/// # Errors
/// A message when neither `IZCAL_TEST_DATA` nor `HOME` is set, or the
/// set is not one of [`SETS`].
pub fn data_file(set: &str) -> Result<PathBuf, String> {
    let cache = match std::env::var_os("IZCAL_TEST_DATA") {
        Some(v) => PathBuf::from(v),
        None => {
            let home = std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .ok_or("neither IZCAL_TEST_DATA nor HOME is set")?;
            PathBuf::from(home).join(".cache/izcal/test-data")
        }
    };
    let name = SETS
        .iter()
        .find(|(s, _)| *s == set)
        .map(|(_, f)| *f)
        .ok_or_else(|| format!("no such data set: {set}"))?;
    let short = PIN.get(..12).unwrap_or(PIN);
    Ok(cache.join(format!("{set}-{short}")).join(name))
}

/// Reads and parses one pinned file.
///
/// # Errors
/// A message naming the file when it cannot be read or is not JSON.
pub fn read_json(path: &Path) -> Result<Json, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("{}: {e}; run `tools/fetch-test-data.sh`", path.display()))?;
    hg_json::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// One `urltestdata.json` case.
#[derive(Debug, Clone, Default)]
pub struct Case {
    /// The input to parse.
    pub input: String,
    /// The base URL, if any. `null` in the file means none.
    pub base: Option<String>,
    /// Whether parsing must fail.
    pub failure: bool,
    /// The expected `href`, when it must succeed.
    pub href: String,
    /// The expected `protocol`, with its `:`.
    pub protocol: String,
    /// The expected `username`.
    pub username: String,
    /// The expected `password`.
    pub password: String,
    /// The expected `host`: the host with its port.
    pub host: String,
    /// The expected `hostname`: the host alone.
    pub hostname: String,
    /// The expected `port`, as a string; empty for none.
    pub port: String,
    /// The expected `pathname`.
    pub pathname: String,
    /// The expected `search`, with its `?`; empty for none.
    pub search: String,
    /// The expected `hash`, with its `#`; empty for none.
    pub hash: String,
}

fn field<'a>(o: &'a [(String, Json)], key: &str) -> Option<&'a Json> {
    o.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn text(o: &[(String, Json)], key: &str) -> String {
    match field(o, key) {
        Some(Json::Str(s)) => s.clone(),
        _ => String::new(),
    }
}

/// Reads the cases out of a parsed `urltestdata.json`.
///
/// The file interleaves comment strings with case objects; only the
/// objects are cases.
#[must_use]
pub fn cases(json: &Json) -> Vec<Case> {
    let Json::Arr(items) = json else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in items {
        let Json::Obj(o) = item else {
            continue;
        };
        if field(o, "input").is_none() {
            continue;
        }
        out.push(Case {
            input: text(o, "input"),
            base: match field(o, "base") {
                Some(Json::Str(s)) => Some(s.clone()),
                _ => None,
            },
            failure: matches!(field(o, "failure"), Some(Json::Bool(true))),
            href: text(o, "href"),
            protocol: text(o, "protocol"),
            username: text(o, "username"),
            password: text(o, "password"),
            host: text(o, "host"),
            hostname: text(o, "hostname"),
            port: text(o, "port"),
            pathname: text(o, "pathname"),
            search: text(o, "search"),
            hash: text(o, "hash"),
        });
    }
    out
}

/// The `URL` class's `host` attribute: the host, then `:port` when
/// there is one.
#[must_use]
pub fn host_attribute(url: &Url) -> String {
    let Some(host) = &url.host else {
        return String::new();
    };
    match url.port {
        Some(p) => format!("{}:{p}", host.serialize()),
        None => host.serialize(),
    }
}

/// The `URL` class's `pathname` attribute.
#[must_use]
pub fn pathname(url: &Url) -> String {
    url.serialize_path()
}

/// What one case measured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// It behaved as the suite says.
    Pass,
    /// It parsed something the suite says must fail.
    ShouldHaveFailed,
    /// It failed on something the suite says must parse.
    ShouldHaveParsed,
    /// It parsed, but a component differs. The string names which.
    Differs(String),
}

/// Runs one case.
#[must_use]
pub fn run_case(case: &Case) -> Outcome {
    let base = match &case.base {
        Some(b) => match Url::parse(b) {
            Some(u) => Some(u),
            // A case whose own base does not parse is a case about the
            // base, and the suite has none: every `base` in the file
            // parses. Reporting it as a failed parse makes that visible
            // rather than silently skipping the case.
            None => return Outcome::ShouldHaveParsed,
        },
        None => None,
    };
    let got = match base.as_ref() {
        Some(b) => Url::parse_with_base(&case.input, b),
        None => Url::parse(&case.input),
    };
    let Some(url) = got else {
        return if case.failure {
            Outcome::Pass
        } else {
            Outcome::ShouldHaveParsed
        };
    };
    if case.failure {
        return Outcome::ShouldHaveFailed;
    }
    let checks: [(&str, String, &str); 9] = [
        ("href", url.serialize(false), &case.href),
        ("protocol", format!("{}:", url.scheme), &case.protocol),
        ("username", url.username.clone(), &case.username),
        ("password", url.password.clone(), &case.password),
        ("host", host_attribute(&url), &case.host),
        (
            "hostname",
            url.host.as_ref().map(Host::serialize).unwrap_or_default(),
            &case.hostname,
        ),
        (
            "port",
            url.port.map(|p| p.to_string()).unwrap_or_default(),
            &case.port,
        ),
        ("pathname", pathname(&url), &case.pathname),
        (
            "search",
            match &url.query {
                Some(q) if !q.is_empty() => format!("?{q}"),
                _ => String::new(),
            },
            &case.search,
        ),
    ];
    for (name, got, want) in checks {
        if got != want {
            return Outcome::Differs(format!("{name}: got {got:?}, wanted {want:?}"));
        }
    }
    let hash = match &url.fragment {
        Some(f) if !f.is_empty() => format!("#{f}"),
        _ => String::new(),
    };
    if hash != case.hash {
        return Outcome::Differs(format!("hash: got {hash:?}, wanted {:?}", case.hash));
    }
    Outcome::Pass
}

/// What a run measured.
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    /// How many cases ran.
    pub runs: usize,
    /// How many passed.
    pub pass: usize,
    /// The first few differences, for the report.
    pub examples: Vec<String>,
    /// Passes and runs by outcome kind.
    pub kinds: BTreeMap<String, usize>,
}

impl Ledger {
    /// Records one case.
    pub fn record(&mut self, case: &Case, outcome: &Outcome) {
        self.runs = self.runs.saturating_add(1);
        let kind = match outcome {
            Outcome::Pass => {
                self.pass = self.pass.saturating_add(1);
                "pass"
            }
            Outcome::ShouldHaveFailed => "parsed what must fail",
            Outcome::ShouldHaveParsed => "failed what must parse",
            Outcome::Differs(_) => "component differs",
        };
        *self.kinds.entry(kind.to_owned()).or_default() += 1;
        if *outcome != Outcome::Pass && self.examples.len() < 20 {
            let detail = match outcome {
                Outcome::Differs(d) => d.clone(),
                Outcome::ShouldHaveFailed => "parsed, but the suite says failure".to_owned(),
                Outcome::ShouldHaveParsed => "failed, but the suite says it parses".to_owned(),
                Outcome::Pass => String::new(),
            };
            self.examples.push(format!(
                "{:?} against {:?}: {detail}",
                case.input,
                case.base.as_deref().unwrap_or("<none>")
            ));
        }
    }
}

/// Runs every case.
#[must_use]
pub fn run(cases: &[Case]) -> Ledger {
    let mut l = Ledger::default();
    for case in cases {
        let outcome = run_case(case);
        l.record(case, &outcome);
    }
    l
}

/// Runs `toascii.json`, the domain parser's own suite.
///
/// WPT measures it the way a page would: it builds
/// `https://<input>/x` and reads `url.host`, so a case whose `output`
/// is `null` is one the *URL parser* must reject. That is what is run
/// here, through the host parser — which is where the domain parser,
/// the ends-in-a-number checker and the IPv4 parser all meet.
#[must_use]
pub fn run_toascii(json: &Json) -> Ledger {
    let mut l = Ledger::default();
    let Json::Arr(items) = json else {
        return l;
    };
    for item in items {
        let Json::Obj(o) = item else {
            continue;
        };
        let Some(Json::Str(input)) = field(o, "input") else {
            continue;
        };
        let want = match field(o, "output") {
            Some(Json::Str(s)) => Some(s.clone()),
            _ => None,
        };
        let got = Url::parse(&format!("https://{input}/x")).map(|u| host_attribute(&u));
        l.runs = l.runs.saturating_add(1);
        if got == want {
            l.pass = l.pass.saturating_add(1);
        } else if l.examples.len() < 20 {
            l.examples
                .push(format!("{input:?}: got {got:?}, wanted {want:?}"));
        }
    }
    l
}

/// Runs `percent-encoding.json`.
///
/// WPT measures it the way HTML does: a document in some encoding
/// resolves `https://example.com/?<input>`, and `url.search` is read.
/// That path runs the query state's "percent-encode after encoding"
/// with the special-query percent-encode set, which is the one place a
/// legacy encoding reaches a URL at all.
#[must_use]
pub fn run_percent_encoding(json: &Json) -> Ledger {
    let mut l = Ledger::default();
    let Json::Arr(items) = json else {
        return l;
    };
    for item in items {
        let Json::Obj(o) = item else {
            continue;
        };
        let (Some(Json::Str(input)), Some(Json::Obj(outputs))) =
            (field(o, "input"), field(o, "output"))
        else {
            continue;
        };
        for (label, want) in outputs {
            let Json::Str(want) = want else {
                continue;
            };
            l.runs = l.runs.saturating_add(1);
            let Some(encoding) = hg_encoding::Encoding::for_label(label) else {
                l.examples.push(format!("{label}: no such encoding"));
                continue;
            };
            let got = hg_url::parse(&format!("https://example.com/?{input}"), None, encoding)
                .and_then(|u| u.query);
            if got.as_deref() == Some(want.as_str()) {
                l.pass = l.pass.saturating_add(1);
            } else if l.examples.len() < 20 {
                l.examples.push(format!(
                    "{input:?} in {label}: got {got:?}, wanted {want:?}"
                ));
            }
        }
    }
    l
}
