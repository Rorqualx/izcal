//! The html5lib **encoding** suite, run against `hg-encoding`'s
//! sniffing helpers.
//!
//! The tokenizer suite measures the tokenizer; this one measures the
//! step before it — HTML's "determine the character encoding", which
//! decides what the bytes *are* before anything reads them. Each case
//! is a byte stream and the encoding a conforming user agent picks for
//! it.
//!
//! The order the runner applies is HTML's, with the parts that need a
//! document left out because a `.dat` case has none: a **byte order
//! mark** first, then the **prescan** of the first 1,024 bytes, then
//! the default. html5lib's own control case says what the default must
//! be — "for the other tests to work, this should pass — you may have
//! to set your defaults appropriately" — and expects windows-1252.
//!
//! The `.dat` format here is simpler than the tokenizer's: `#data`
//! holds **raw bytes**, not escapes, and `#encoding` holds a label to
//! resolve through "get an encoding".

use std::collections::BTreeMap;
use std::path::Path;

use hg_encoding::{Encoding, HTML_LIMIT, bom_sniff, prescan};

/// Where the suite lives, under the pinned tree.
pub const DIR: &str = "encoding";

/// The default a document falls back to, which the suite's own control
/// case fixes.
pub const DEFAULT: Encoding = Encoding::Windows1252;

/// How many cases the pinned suite holds.
pub const CASES: usize = 82;

/// The number that must not fall: 75 of 82 at the 1,024-byte window.
///
/// The seven that differ all put the character encoding declaration
/// **past** the window, at byte 2020, 2021, 4069, 4070, 8213, 8214 and
/// 8317. Each is a document the HTML Standard itself calls
/// non-conforming: "The element containing the character encoding
/// declaration must be serialized completely within the first 1024
/// bytes of the document." The standard draws the UA's rule from that
/// one — "The authoring conformance requirements ... limit them to only
/// appearing in the first 1024 bytes. User agents are therefore
/// encouraged to use the prescan algorithm below on the first 1024
/// bytes, but not to stall beyond that."
///
/// So the window is a decision, not an oversight, and the runner prints
/// what every other choice would score:
///
/// | window | passes |
/// |---|---|
/// | 1,024 | 75 |
/// | 2,048 | 77 |
/// | 4,096 | 79 |
/// | 8,192 | 81 |
/// | the whole stream | 82 |
///
/// Scanning further buys those seven and costs every page the delay
/// before decoding can start. If a corpus measurement at ST-M0 shows
/// real pages placing the declaration late, the number to change is
/// [`hg_encoding::HTML_LIMIT`] and this baseline with it.
pub const BASELINE: usize = 75;

/// One case: the bytes, and the encoding they should resolve to.
#[derive(Debug, Clone)]
pub struct Case {
    /// Which file it came from.
    pub file: String,
    /// Its index in that file, counting from one.
    pub index: usize,
    /// The document's bytes.
    pub data: Vec<u8>,
    /// The label the suite expects, as written.
    pub label: String,
}

/// HTML's "determine the character encoding", for a stream with no
/// transport-layer charset and no parent document.
#[must_use]
pub fn determine(bytes: &[u8]) -> Encoding {
    determine_within(bytes, HTML_LIMIT)
}

/// The same, with the prescan's end condition given.
///
/// HTML leaves the window to the user agent — "the end condition being
/// when the user agent decides that scanning further bytes would not
/// be efficient", with 1,024 bytes only *encouraged*. This exists so
/// the choice can be measured against the suite rather than assumed.
#[must_use]
pub fn determine_within(bytes: &[u8], limit: usize) -> Encoding {
    // A byte order mark outranks everything, including a header. The
    // Encoding Standard says why: it "has been found to be more
    // accurate in deployed content".
    if let Some((e, _)) = bom_sniff(bytes) {
        return e;
    }
    prescan(bytes, limit).unwrap_or(DEFAULT)
}

/// Reads every `.dat` file of the suite, in name order.
///
/// Subdirectories are left out: `scripted/` needs a script-running
/// parser and `chardet/` is for statistical detection, which Izcal does
/// not do (ST-D16).
///
/// # Errors
/// A message naming the file when the directory or a file cannot be
/// read, or a case is not the shape above.
pub fn read_all(dir: &Path) -> Result<Vec<Case>, String> {
    let mut names: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if path.extension().is_some_and(|x| x == "dat") {
            names.push(path);
        }
    }
    names.sort();
    if names.is_empty() {
        return Err(format!("{}: no .dat files", dir.display()));
    }
    let mut out = Vec::new();
    for path in names {
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let name = path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into(),
        );
        out.extend(parse(&name, &bytes)?);
    }
    Ok(out)
}

/// Splits one file into cases.
///
/// # Errors
/// A message when a case has no `#encoding` section.
pub fn parse(file: &str, bytes: &[u8]) -> Result<Vec<Case>, String> {
    let mut out = Vec::new();
    // Sections are introduced by a line that is exactly `#data` or
    // `#encoding`. Everything between belongs to the section, bytes and
    // all, so the split is on line boundaries and not on the text.
    let mut data: Option<Vec<u8>> = None;
    let mut label: Option<Vec<u8>> = None;
    let mut current: Vec<u8> = Vec::new();
    let mut section = Section::None;
    let finish = |section: &Section,
                  current: &mut Vec<u8>,
                  data: &mut Option<Vec<u8>>,
                  label: &mut Option<Vec<u8>>| {
        // The last newline belongs to the separator, not the section.
        if current.last() == Some(&b'\n') {
            current.pop();
        }
        match section {
            Section::Data => *data = Some(core::mem::take(current)),
            Section::Encoding => *label = Some(core::mem::take(current)),
            Section::None => current.clear(),
        }
    };
    for line in split_lines(bytes) {
        let next = match line {
            b"#data" => Some(Section::Data),
            b"#encoding" => Some(Section::Encoding),
            _ => None,
        };
        let Some(next) = next else {
            current.extend_from_slice(line);
            current.push(b'\n');
            continue;
        };
        finish(&section, &mut current, &mut data, &mut label);
        if next == Section::Data
            && let (Some(d), Some(l)) = (data.take(), label.take())
        {
            out.push(case(file, out.len().saturating_add(1), d, &l)?);
        }
        section = next;
    }
    finish(&section, &mut current, &mut data, &mut label);
    if let (Some(d), Some(l)) = (data.take(), label.take()) {
        out.push(case(file, out.len().saturating_add(1), d, &l)?);
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Data,
    Encoding,
}

fn case(file: &str, index: usize, data: Vec<u8>, label: &[u8]) -> Result<Case, String> {
    let text = core::str::from_utf8(label)
        .map_err(|e| format!("{file} case {index}: the expected label is not text: {e}"))?;
    Ok(Case {
        file: file.to_owned(),
        index,
        data,
        label: text.trim().to_owned(),
    })
}

/// Splits on LF, keeping neither the LF nor a trailing CR.
fn split_lines(bytes: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut start = 0_usize;
    for (i, b) in bytes.iter().enumerate() {
        if *b != b'\n' {
            continue;
        }
        let mut end = i;
        if bytes.get(end.wrapping_sub(1)) == Some(&b'\r') {
            end = end.saturating_sub(1);
        }
        out.push(bytes.get(start..end).unwrap_or(&[]));
        start = i.saturating_add(1);
    }
    if start < bytes.len() {
        out.push(bytes.get(start..).unwrap_or(&[]));
    }
    out
}

/// How the suite came out.
#[derive(Debug, Default, Clone)]
pub struct Ledger {
    /// Cases run.
    pub runs: usize,
    /// Cases whose encoding matched.
    pub pass: usize,
    /// Per file: runs, then passes.
    pub by_file: BTreeMap<String, (usize, usize)>,
    /// The first few differences, for the report to name.
    pub examples: Vec<String>,
}

/// How many differences the report prints.
pub const EXAMPLES: usize = 20;

/// Runs every case.
#[must_use]
pub fn run(cases: &[Case]) -> Ledger {
    run_within(cases, HTML_LIMIT)
}

/// The same, with the prescan window given.
#[must_use]
pub fn run_within(cases: &[Case], limit: usize) -> Ledger {
    let mut l = Ledger::default();
    for c in cases {
        l.runs = l.runs.saturating_add(1);
        let e = l.by_file.entry(c.file.clone()).or_insert((0, 0));
        e.0 = e.0.saturating_add(1);
        let want = Encoding::for_label(&c.label);
        let got = determine_within(&c.data, limit);
        if want == Some(got) {
            l.pass = l.pass.saturating_add(1);
            e.1 = e.1.saturating_add(1);
        } else if l.examples.len() < EXAMPLES {
            let head: String = c
                .data
                .iter()
                .take(60)
                .map(|b| {
                    if b.is_ascii_graphic() || *b == b' ' {
                        char::from(*b).to_string()
                    } else {
                        format!("\\x{b:02X}")
                    }
                })
                .collect();
            l.examples.push(format!(
                "{} case {}: expected {} ({:?}), got {} — {head}",
                c.file,
                c.index,
                want.map_or("an unknown label", Encoding::name),
                c.label,
                got.name()
            ));
        }
    }
    l
}
