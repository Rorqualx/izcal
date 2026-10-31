//! Result ledgers: one line per run, readable, diffable and comparable.
//!
//! ```text
//! # hg-test262 ledger 1
//! @engine engine262 0.0.1-d553e7b...
//! @test262 91c70759...
//! pass<TAB>strict<TAB>built-ins/Array/length.js<TAB>-<TAB>
//! fail<TAB>sloppy<TAB>built-ins/x.js<TAB>Symbol,Proxy<TAB>uncaught TypeError: ...
//! ```
//!
//! Lines starting `@` are header fields (`@key value`); `#` lines are
//! comments. A run line has five tab-separated columns: status, mode,
//! path under `test/`, the test's features (comma-separated, `-` for
//! none) and a detail, with `\`, tab, CR and LF escaped as `\\`, `\t`,
//! `\r`, `\n`. Lines are sorted by path then mode, so two ledgers of the
//! same suite diff line by line.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::classify::Status;
use crate::expand::Mode;

/// The first line of every ledger.
pub const MAGIC: &str = "# hg-test262 ledger 1";

/// The detail prefix of a negative-test pass whose phase the adapter could
/// not report.
pub const PHASE_UNVERIFIED: &str = "phase not reported";

/// The longest detail kept, in characters.
pub const MAX_DETAIL: usize = 400;

/// One run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The verdict.
    pub status: Status,
    /// The run's mode.
    pub mode: Mode,
    /// The test's path under `test/`.
    pub path: String,
    /// The test's `features`.
    pub features: Vec<String>,
    /// Why, for anything but a plain pass.
    pub detail: String,
}

impl Entry {
    /// A negative test that passed on its error name alone.
    pub fn phase_unverified(&self) -> bool {
        self.status == Status::Pass && self.detail.starts_with(PHASE_UNVERIFIED)
    }
}

/// A ledger: header fields and runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    /// `@key value` fields, in order.
    pub header: Vec<(String, String)>,
    /// The runs, sorted by path then mode.
    pub entries: Vec<Entry>,
}

/// Escapes a detail, kept to [`MAX_DETAIL`] characters.
fn escape_detail(s: &str) -> String {
    escape(&s.chars().take(MAX_DETAIL).collect::<String>())
}

/// Escapes a field; paths and header values are never cut.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out
}

fn unescape(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            other => {
                return Err(format!(
                    "bad escape `\\{}`",
                    other.map(String::from).unwrap_or_default()
                ));
            }
        }
    }
    Ok(out)
}

impl Ledger {
    /// A header field's value.
    pub fn field(&self, key: &str) -> Option<&str> {
        self.header
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Sorts the runs by path, then mode.
    pub fn sort(&mut self) {
        self.entries
            .sort_by(|a, b| (a.path.as_str(), a.mode).cmp(&(b.path.as_str(), b.mode)));
    }

    /// The ledger's text.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(MAGIC);
        out.push('\n');
        for (k, v) in &self.header {
            let _ = writeln!(out, "@{k} {}", escape(v));
        }
        for e in &self.entries {
            let features = if e.features.is_empty() {
                "-".to_string()
            } else {
                e.features.join(",")
            };
            let _ = writeln!(
                out,
                "{}\t{}\t{}\t{}\t{}",
                e.status,
                e.mode,
                escape(&e.path),
                features,
                escape_detail(&e.detail)
            );
        }
        out
    }

    /// Parses a ledger's text; errors name the line.
    pub fn parse(text: &str) -> Result<Ledger, String> {
        let mut lines = text.lines().enumerate();
        match lines.next() {
            Some((_, l)) if l == MAGIC => {}
            _ => return Err(format!("line 1: not a ledger (expected `{MAGIC}`)")),
        }
        let mut ledger = Ledger::default();
        for (i, line) in lines {
            let n = i + 1;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(h) = line.strip_prefix('@') {
                let (k, v) = h.split_once(' ').unwrap_or((h, ""));
                ledger.header.push((
                    k.to_string(),
                    unescape(v).map_err(|e| format!("line {n}: {e}"))?,
                ));
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            let [status, mode, path, features, detail] = cols[..] else {
                return Err(format!(
                    "line {n}: expected 5 tab-separated columns, found {}",
                    cols.len()
                ));
            };
            let status = Status::parse(status)
                .ok_or_else(|| format!("line {n}: unknown status `{status}`"))?;
            let mode =
                Mode::parse(mode).ok_or_else(|| format!("line {n}: unknown mode `{mode}`"))?;
            let features = if features == "-" {
                Vec::new()
            } else {
                features.split(',').map(str::to_string).collect()
            };
            ledger.entries.push(Entry {
                status,
                mode,
                path: unescape(path).map_err(|e| format!("line {n}: {e}"))?,
                features,
                detail: unescape(detail).map_err(|e| format!("line {n}: {e}"))?,
            });
        }
        Ok(ledger)
    }
}

/// Counts of runs or files by status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// Passed.
    pub pass: usize,
    /// Failed (fail, timeout, crash or error).
    pub fail: usize,
    /// Of `fail`, the timeouts.
    pub timeout: usize,
    /// Excluded, not run.
    pub excluded: usize,
    /// Of `pass`, negative tests whose phase was not verified.
    pub phase_unverified: usize,
}

impl Counts {
    /// Every counted item.
    pub fn total(&self) -> usize {
        self.pass + self.fail + self.excluded
    }

    /// The items that ran.
    pub fn ran(&self) -> usize {
        self.pass + self.fail
    }

    fn add_status(&mut self, s: Status, phase_unverified: bool) {
        match s {
            Status::Pass => {
                self.pass += 1;
                if phase_unverified {
                    self.phase_unverified += 1;
                }
            }
            Status::Excluded => self.excluded += 1,
            Status::Timeout => {
                self.fail += 1;
                self.timeout += 1;
            }
            Status::Fail | Status::Crash | Status::Error => self.fail += 1,
        }
    }
}

/// One test file's result over its runs: excluded if every run was
/// excluded, pass if every run that ran passed, else the first non-pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileResult<'a> {
    /// The path under `test/`.
    pub path: &'a str,
    /// The file's features.
    pub features: &'a [String],
    /// The combined status.
    pub status: Status,
    /// Some run passed only on its error name.
    pub phase_unverified: bool,
}

/// Combines runs into per-file results (the unit test262.fyi counts).
pub fn files(entries: &[Entry]) -> Vec<FileResult<'_>> {
    let mut out: Vec<FileResult<'_>> = Vec::new();
    for e in entries {
        if let Some(last) = out.last_mut()
            && last.path == e.path
        {
            last.phase_unverified |= e.phase_unverified();
            last.status = match (last.status, e.status) {
                (Status::Excluded, s) | (s, Status::Excluded) => s,
                (Status::Pass, s) => s,
                (s, _) => s,
            };
            continue;
        }
        out.push(FileResult {
            path: &e.path,
            features: &e.features,
            status: e.status,
            phase_unverified: e.phase_unverified(),
        });
    }
    out
}

/// The directory key of a path at a depth: its first `depth` directories.
pub fn dir_key(path: &str, depth: usize) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let dirs = &parts[..parts.len().saturating_sub(1)];
    dirs[..depth.min(dirs.len())].join("/")
}

/// Per-file counts by directory at `depth` (1: `built-ins`, `language`...).
pub fn by_dir(entries: &[Entry], depth: usize) -> BTreeMap<String, Counts> {
    let mut out: BTreeMap<String, Counts> = BTreeMap::new();
    for f in files(entries) {
        out.entry(dir_key(f.path, depth))
            .or_default()
            .add_status(f.status, f.phase_unverified);
    }
    out
}

/// Per-file counts by feature tag (a file counts under each of its tags).
pub fn by_feature(entries: &[Entry]) -> BTreeMap<String, Counts> {
    let mut out: BTreeMap<String, Counts> = BTreeMap::new();
    for f in files(entries) {
        for feat in f.features {
            out.entry(feat.clone())
                .or_default()
                .add_status(f.status, f.phase_unverified);
        }
    }
    out
}

/// Per-file and per-run totals.
pub fn totals(entries: &[Entry]) -> (Counts, Counts) {
    let mut by_file = Counts::default();
    for f in files(entries) {
        by_file.add_status(f.status, f.phase_unverified);
    }
    let mut by_run = Counts::default();
    for e in entries {
        by_run.add_status(e.status, e.phase_unverified());
    }
    (by_file, by_run)
}

fn pct(n: usize, d: usize) -> String {
    if d == 0 {
        "-".into()
    } else {
        format!("{:.2}%", 100.0 * n as f64 / d as f64)
    }
}

/// A Markdown table of counts: name, files, passed, failed (of which
/// timeouts), excluded, pass rate of the files that ran and of all files.
pub fn table(title: &str, rows: &BTreeMap<String, Counts>) -> String {
    let mut out = format!(
        "| {title} | Files | Pass | Fail (timeout) | Excluded | Pass / ran | Pass / all |\n|---|---:|---:|---:|---:|---:|---:|\n"
    );
    let mut sum = Counts::default();
    for (k, c) in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} ({}) | {} | {} | {} |",
            if k.is_empty() { "(root)" } else { k },
            c.total(),
            c.pass,
            c.fail,
            c.timeout,
            c.excluded,
            pct(c.pass, c.ran()),
            pct(c.pass, c.total())
        );
        sum.pass += c.pass;
        sum.fail += c.fail;
        sum.timeout += c.timeout;
        sum.excluded += c.excluded;
        sum.phase_unverified += c.phase_unverified;
    }
    let _ = writeln!(
        out,
        "| **all** | {} | {} | {} ({}) | {} | {} | {} |",
        sum.total(),
        sum.pass,
        sum.fail,
        sum.timeout,
        sum.excluded,
        pct(sum.pass, sum.ran()),
        pct(sum.pass, sum.total())
    );
    out
}

/// Applies an exclusion list after the fact: runs it matches become
/// excluded (to restate a full run's results under an engine's list).
pub fn apply_exclusions(entries: &mut [Entry], ex: &crate::exclusions::Exclusions) {
    for e in entries {
        if e.status != Status::Excluded
            && let Some(m) = ex.matches(&e.path, &e.features)
        {
            e.status = Status::Excluded;
            e.detail = m.to_string();
        }
    }
}

/// A change in one run between two ledgers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// The path under `test/`.
    pub path: String,
    /// The mode.
    pub mode: Mode,
    /// The status in the first ledger (`None`: absent there).
    pub before: Option<Status>,
    /// The status in the second ledger (`None`: absent there).
    pub after: Option<Status>,
    /// The second ledger's detail, or the first's if absent.
    pub detail: String,
}

/// The differences between two ledgers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Diff {
    /// Passed before, ran and did not pass after.
    pub regressions: Vec<Change>,
    /// Ran and did not pass before, passed after.
    pub improvements: Vec<Change>,
    /// Every other status change, and runs present in only one ledger.
    pub other: Vec<Change>,
}

/// Compares two ledgers run by run.
pub fn compare(a: &Ledger, b: &Ledger) -> Diff {
    let key = |e: &Entry| (e.path.clone(), e.mode);
    let am: BTreeMap<_, &Entry> = a.entries.iter().map(|e| (key(e), e)).collect();
    let bm: BTreeMap<_, &Entry> = b.entries.iter().map(|e| (key(e), e)).collect();
    let mut keys: Vec<_> = am.keys().chain(bm.keys()).cloned().collect();
    keys.sort();
    keys.dedup();
    let mut diff = Diff::default();
    for k in keys {
        let before = am.get(&k).map(|e| e.status);
        let after = bm.get(&k).map(|e| e.status);
        if before == after {
            continue;
        }
        let detail = bm
            .get(&k)
            .or(am.get(&k))
            .map(|e| e.detail.clone())
            .unwrap_or_default();
        let change = Change {
            path: k.0,
            mode: k.1,
            before,
            after,
            detail,
        };
        let ran = |s: Option<Status>| matches!(s, Some(s) if s != Status::Excluded);
        if before == Some(Status::Pass) && ran(after) {
            diff.regressions.push(change);
        } else if after == Some(Status::Pass) && ran(before) {
            diff.improvements.push(change);
        } else {
            diff.other.push(change);
        }
    }
    diff
}

#[cfg(test)]
mod tests;
