//! Reading the suite's `.test` files.
//!
//! The format is html5lib-tests' own, documented in its `README.md`:
//! one object per file with a `tests` array, each case carrying
//! `description`, `input`, `output`, and optionally `initialStates`,
//! `lastStartTag`, `errors` and `doubleEscaped`.

use std::collections::BTreeMap;
use std::path::Path;

use hg_html_syntax::State;
use hg_htmlgen::json::{self, Json};

/// One expected token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expected {
    /// A run of character tokens, which the suite writes as one string.
    Character(String),
    /// A start tag.
    StartTag {
        /// Its name.
        name: String,
        /// Its attributes, which the suite writes as an object.
        attrs: BTreeMap<String, String>,
        /// Its self-closing flag, written only when set.
        self_closing: bool,
    },
    /// An end tag, with its name.
    EndTag(String),
    /// A comment, with its data.
    Comment(String),
    /// A processing instruction, with its target and data.
    ProcessingInstruction {
        /// Its target.
        target: String,
        /// Its data.
        data: String,
    },
    /// A DOCTYPE. `correct` is the suite's own word for "force-quirks
    /// is off".
    Doctype {
        /// Its name.
        name: Option<String>,
        /// Its public identifier.
        public_id: Option<String>,
        /// Its system identifier.
        system_id: Option<String>,
        /// Whether force-quirks is off.
        correct: bool,
    },
}

/// One expected parse error, with where the suite says it happens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedError {
    /// Its name.
    pub code: String,
    /// The line, counting from one.
    pub line: u32,
    /// The column, counting from one.
    pub col: u32,
}

/// One case.
#[derive(Debug, Clone)]
pub struct Case {
    /// The file it came from.
    pub file: String,
    /// Its description, which the suite uses as its name.
    pub description: String,
    /// The input, decoded.
    pub input: String,
    /// The tokens it should produce.
    pub output: Vec<Expected>,
    /// The states it should be run from.
    pub initial_states: Vec<State>,
    /// The last start tag to pretend was emitted.
    pub last_start_tag: String,
    /// The parse errors it should raise.
    pub errors: Vec<ExpectedError>,
    /// Whether the input holds a lone surrogate, which Rust's `String`
    /// cannot carry (see the crate docs).
    pub lone_surrogate: bool,
}

/// Parse errors the suite expects that this crate does not judge,
/// because the standard raises them somewhere the tokenizer is not.
///
/// The first three belong to the input stream (§13.2.3.5), which
/// `hg-html-driver` owns. The last two belong to the tree construction
/// stage — "end-tag-with-attributes" is raised where an end tag is
/// processed, not where it is tokenized — which is ST-M1's.
pub const NOT_THE_TOKENIZERS: [&str; 5] = [
    "surrogate-in-input-stream",
    "noncharacter-in-input-stream",
    "control-character-in-input-stream",
    "end-tag-with-attributes",
    "end-tag-with-trailing-solidus",
];

fn state_of(name: &str) -> Option<State> {
    Some(match name {
        "Data state" => State::Data,
        "PLAINTEXT state" => State::Plaintext,
        "RCDATA state" => State::Rcdata,
        "RAWTEXT state" => State::Rawtext,
        "Script data state" => State::ScriptData,
        "CDATA section state" => State::CdataSection,
        _ => return None,
    })
}

/// The suite's `doubleEscaped`: the string has been through JSON
/// escaping twice, so `\uXXXX` in the decoded text is still an escape.
fn unescape_again(s: &str) -> String {
    let mut out = String::new();
    let mut cs = s.chars().peekable();
    while let Some(c) = cs.next() {
        if c != '\\' || cs.peek() != Some(&'u') {
            out.push(c);
            continue;
        }
        cs.next();
        let hex: String = cs.by_ref().take(4).collect();
        match u32::from_str_radix(&hex, 16) {
            // A lone surrogate becomes the replacement character: see
            // the crate docs.
            Ok(n) => out.push(char::from_u32(n).unwrap_or('\u{FFFD}')),
            Err(_) => {
                out.push_str("\\u");
                out.push_str(&hex);
            }
        }
    }
    out
}

fn has_lone_surrogate(raw: &str) -> bool {
    let mut at = 0;
    while let Some(i) = raw.get(at..).and_then(|r| r.find("\\u")) {
        let start = at + i + 2;
        let hex = raw.get(start..start + 4).unwrap_or("");
        if let Ok(n) = u32::from_str_radix(hex, 16)
            && (0xD800..0xE000).contains(&n)
        {
            return true;
        }
        at = start;
    }
    false
}

fn token_of(v: &Json, double: bool) -> Result<Expected, String> {
    let parts = v.as_arr().ok_or("a token is not an array")?;
    let kind = parts
        .first()
        .and_then(Json::as_str)
        .ok_or("a token has no kind")?;
    let text = |i: usize| -> String {
        let s = parts.get(i).and_then(Json::as_str).unwrap_or_default();
        if double {
            unescape_again(s)
        } else {
            s.to_owned()
        }
    };
    let opt = |i: usize| -> Option<String> {
        match parts.get(i) {
            Some(Json::Str(s)) => Some(if double { unescape_again(s) } else { s.clone() }),
            _ => None,
        }
    };
    Ok(match kind {
        "Character" => Expected::Character(text(1)),
        "Comment" => Expected::Comment(text(1)),
        "EndTag" => Expected::EndTag(text(1)),
        "ProcessingInstruction" => Expected::ProcessingInstruction {
            target: text(1),
            data: text(2),
        },
        "StartTag" => {
            let mut attrs = BTreeMap::new();
            if let Some(Json::Obj(kv)) = parts.get(2) {
                for (k, val) in kv {
                    let name = if double { unescape_again(k) } else { k.clone() };
                    let value = val.as_str().unwrap_or_default();
                    let value = if double {
                        unescape_again(value)
                    } else {
                        value.to_owned()
                    };
                    attrs.insert(name, value);
                }
            }
            Expected::StartTag {
                name: text(1),
                attrs,
                self_closing: parts.get(3).and_then(Json::as_bool).unwrap_or(false),
            }
        }
        "DOCTYPE" => Expected::Doctype {
            name: opt(1),
            public_id: opt(2),
            system_id: opt(3),
            correct: parts.get(4).and_then(Json::as_bool).unwrap_or(true),
        },
        other => return Err(format!("unknown token kind {other:?}")),
    })
}

/// Reads every case in one `.test` file.
///
/// # Errors
/// A message naming the file when it cannot be read or is not the shape
/// above.
pub fn read_file(path: &Path) -> Result<Vec<Case>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into(),
    );
    let doc = json::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let Some(tests) = doc.get("tests").and_then(Json::as_arr) else {
        // `xmlViolation.test` has a `xmlViolationTests` key instead and
        // describes a different algorithm; it is not this runner's.
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for t in tests {
        let double = t
            .get("doubleEscaped")
            .and_then(Json::as_bool)
            .unwrap_or(false);
        let raw_input = t.get("input").and_then(Json::as_str).unwrap_or_default();
        let input = if double {
            unescape_again(raw_input)
        } else {
            raw_input.to_owned()
        };
        let mut output = Vec::new();
        for token in t.get("output").and_then(Json::as_arr).unwrap_or_default() {
            output.push(token_of(token, double).map_err(|e| format!("{file}: {e}"))?);
        }
        let initial_states = match t.get("initialStates").and_then(Json::as_arr) {
            Some(v) => v
                .iter()
                .filter_map(Json::as_str)
                .filter_map(state_of)
                .collect(),
            None => vec![State::Data],
        };
        let mut errors = Vec::new();
        for e in t.get("errors").and_then(Json::as_arr).unwrap_or_default() {
            let code = e.get("code").and_then(Json::as_str).unwrap_or_default();
            if NOT_THE_TOKENIZERS.contains(&code) {
                continue;
            }
            errors.push(ExpectedError {
                code: code.to_owned(),
                line: u32::try_from(e.get("line").and_then(Json::as_i64).unwrap_or(0)).unwrap_or(0),
                col: u32::try_from(e.get("col").and_then(Json::as_i64).unwrap_or(0)).unwrap_or(0),
            });
        }
        out.push(Case {
            file: file.clone(),
            description: t
                .get("description")
                .and_then(Json::as_str)
                .unwrap_or("(no description)")
                .to_owned(),
            input,
            output,
            initial_states,
            last_start_tag: t
                .get("lastStartTag")
                .and_then(Json::as_str)
                .unwrap_or_default()
                .to_owned(),
            errors,
            lone_surrogate: has_lone_surrogate(raw_input) || raw_input.contains('\u{FFFD}'),
        });
    }
    Ok(out)
}

/// Reads every `.test` file in `dir`, sorted by name.
///
/// # Errors
/// A message when the directory or a file cannot be read.
pub fn read_dir(dir: &Path) -> Result<Vec<Case>, String> {
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let p = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if p.extension().is_some_and(|x| x == "test") {
            paths.push(p);
        }
    }
    paths.sort();
    let mut out = Vec::new();
    for p in paths {
        out.extend(read_file(&p)?);
    }
    Ok(out)
}
