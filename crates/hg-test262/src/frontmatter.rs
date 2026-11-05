//! test262 front matter: the `/*--- ... ---*/` block of each test file.
//!
//! test262's CONTRIBUTING.md ("Frontmatter") defines the block as YAML and
//! INTERPRETING.md ("Metadata") says what each key means to a runner. The
//! files use a small, regular subset of YAML, and this module reads exactly
//! that subset itself (no YAML library):
//!
//! - top-level `key: value` lines at column 0, each key at most once;
//! - flow sequences on one line, `[a, b, 'c']` (CONTRIBUTING: "scalar
//!   values spanning multiple lines may not be expressed using flow
//!   notation"), and block sequences (`key:` then indented `- item` lines);
//! - one-level mappings, block (`negative:` then indented `phase: parse`)
//!   or flow on one line (`{phase: parse, type: SyntaxError}`);
//! - plain scalars, folded over indented continuation lines; quoted
//!   scalars; block scalars introduced by `|` or `>`.
//!
//! Anything outside the subset is refused with the line it is on, and so
//! are unknown flags, an unknown `negative` phase and contradictory flags:
//! a runner that guessed would report a result for a test it did not run
//! as written.

use std::fmt;

/// The phase in which a negative test expects its error (INTERPRETING.md,
/// `negative`), and the phase an engine reports for an uncaught error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    /// Parsing or early errors of the test's own source text.
    Parse,
    /// Module resolution (linking, or a dependency's parse or early error).
    Resolution,
    /// Evaluation.
    Runtime,
}

impl Phase {
    /// The phase's name as test262 writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Parse => "parse",
            Phase::Resolution => "resolution",
            Phase::Runtime => "runtime",
        }
    }

    /// Parses test262's phase name.
    pub fn parse(s: &str) -> Option<Phase> {
        match s {
            "parse" => Some(Phase::Parse),
            "resolution" => Some(Phase::Resolution),
            "runtime" => Some(Phase::Runtime),
            _ => None,
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A negative test's expectation: the phase and the error constructor's
/// name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Negative {
    /// The phase the error must occur in.
    pub phase: Phase,
    /// The name of the expected error's constructor, such as `SyntaxError`.
    pub error_type: String,
}

/// The `flags` key (INTERPRETING.md, `flags`; CONTRIBUTING.md, `flags`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    /// `onlyStrict`: run once, in strict mode.
    pub only_strict: bool,
    /// `noStrict`: run once, in non-strict mode.
    pub no_strict: bool,
    /// `module`: the source is module code; run once.
    pub module: bool,
    /// `raw`: run the source unmodified, without harness files, once.
    pub raw: bool,
    /// `async`: completion is signalled through `print`.
    pub is_async: bool,
    /// `CanBlockIsFalse`: run only where the agent's [[CanBlock]] is false.
    pub can_block_is_false: bool,
    /// `CanBlockIsTrue`: run only where the agent's [[CanBlock]] is true.
    pub can_block_is_true: bool,
    /// `generated`: informative only.
    pub generated: bool,
    /// `non-deterministic`: informative; the result is not a conformance
    /// signal.
    pub non_deterministic: bool,
}

/// The parsed front matter of one test file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrontMatter {
    /// `description` (required by CONTRIBUTING.md; one test at the pinned
    /// revision has none, so its absence is not refused).
    pub description: Option<String>,
    /// `esid`, when present.
    pub esid: Option<String>,
    /// `flags`.
    pub flags: Flags,
    /// `includes`: harness files, in the order given.
    pub includes: Vec<String>,
    /// `negative`, when present.
    pub negative: Option<Negative>,
    /// `features`.
    pub features: Vec<String>,
    /// `locale`.
    pub locale: Vec<String>,
}

/// Why a front matter block was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontMatterError {
    /// The 1-based line of the test file the problem is on (0 when it is
    /// about the file as a whole).
    pub line: usize,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for FrontMatterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "front matter: {}", self.message)
        } else {
            write!(f, "front matter, line {}: {}", self.line, self.message)
        }
    }
}

impl std::error::Error for FrontMatterError {}

fn err<T>(line: usize, message: impl Into<String>) -> Result<T, FrontMatterError> {
    Err(FrontMatterError {
        line,
        message: message.into(),
    })
}

/// A YAML value of the supported subset.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Scalar(String),
    List(Vec<String>),
    Map(Vec<(String, String)>),
    Empty,
}

const OPEN: &str = "/*---";
const CLOSE: &str = "---*/";

/// Finds the front matter block of a test file and parses it.
pub fn parse_test_file(source: &str) -> Result<FrontMatter, FrontMatterError> {
    let Some(start) = source.find(OPEN) else {
        return err(0, "no `/*---` block (INTERPRETING.md, Metadata)");
    };
    let body_start = start + OPEN.len();
    let Some(len) = source[body_start..].find(CLOSE) else {
        return err(line_of(source, start), "`/*---` is never closed by `---*/`");
    };
    let body_end = body_start + len;
    if let Some(again) = source[body_end + CLOSE.len()..].find(OPEN) {
        return err(
            line_of(source, body_end + CLOSE.len() + again),
            "a second `/*---` block (CONTRIBUTING.md: exactly one per test)",
        );
    }
    // The body starts on the line holding `/*---`; text after the opener on
    // that line is not part of any key. YAML line breaks are LF, CRLF or a
    // lone CR (one test at the pinned revision uses CR alone).
    let first_line = line_of(source, start);
    let body = &source[body_start..body_end];
    let mut lines: Vec<(usize, &str)> = split_lines(body)
        .into_iter()
        .enumerate()
        .map(|(i, l)| (first_line + i, l))
        .collect();
    if let Some((_, first)) = lines.first()
        && !first.trim().is_empty()
    {
        return err(first_line, "text after `/*---` on its line");
    }
    lines.remove(0);
    // The top-level mapping may itself be indented (one test indents it by
    // a space); its first key sets the column every top-level key uses.
    let base = lines
        .iter()
        .find(|(_, t)| !is_blank_or_comment(t))
        .map_or(0, |(_, t)| indent_of(t));
    let mut shifted = Vec::with_capacity(lines.len());
    for (ln, t) in lines {
        if is_blank_or_comment(t) {
            shifted.push((ln, &t[indent_of(t).min(base)..]));
        } else if indent_of(t) < base {
            return err(
                ln,
                format!("less indented than the first key: `{}`", t.trim()),
            );
        } else {
            shifted.push((ln, &t[base..]));
        }
    }
    parse_body(&shifted)
}

/// Splits text at LF, CRLF or a lone CR.
fn split_lines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                out.push(&text[start..i]);
                start = i + 1;
            }
            b'\r' => {
                out.push(&text[start..i]);
                if bytes.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&text[start..]);
    out
}

fn line_of(source: &str, byte: usize) -> usize {
    split_lines(&source[..byte]).len()
}

/// Parses the lines between the delimiters, each with its file line number.
pub(crate) fn parse_body(lines: &[(usize, &str)]) -> Result<FrontMatter, FrontMatterError> {
    let entries = parse_entries(lines)?;
    let mut fm = FrontMatter::default();
    let mut flags_line = 0;
    for (line, key, value) in entries {
        match key.as_str() {
            "description" => fm.description = Some(scalar(line, &key, value)?),
            "esid" => fm.esid = Some(scalar(line, &key, value)?),
            "includes" => {
                fm.includes = list(line, &key, value)?;
                if let Some(bad) = fm.includes.iter().find(|i| !i.ends_with(".js")) {
                    return err(line, format!("include `{bad}` does not end in `.js`"));
                }
            }
            "features" => fm.features = list(line, &key, value)?,
            "locale" => fm.locale = list(line, &key, value)?,
            "flags" => {
                flags_line = line;
                for flag in list(line, &key, value)? {
                    let f = &mut fm.flags;
                    let slot = match flag.as_str() {
                        "onlyStrict" => &mut f.only_strict,
                        "noStrict" => &mut f.no_strict,
                        "module" => &mut f.module,
                        "raw" => &mut f.raw,
                        "async" => &mut f.is_async,
                        "CanBlockIsFalse" => &mut f.can_block_is_false,
                        "CanBlockIsTrue" => &mut f.can_block_is_true,
                        "generated" => &mut f.generated,
                        "non-deterministic" => &mut f.non_deterministic,
                        other => {
                            return err(
                                line,
                                format!("unknown flag `{other}` (INTERPRETING.md, flags)"),
                            );
                        }
                    };
                    if *slot {
                        return err(line, format!("flag `{flag}` given twice"));
                    }
                    *slot = true;
                }
            }
            "negative" => fm.negative = Some(negative(line, value)?),
            // Informative keys: CONTRIBUTING.md lists them; a runner has no
            // use for their values.
            "info" | "author" | "es5id" | "es6id" => {}
            other => {
                return err(
                    line,
                    format!("unknown key `{other}` (CONTRIBUTING.md, Frontmatter)"),
                );
            }
        }
    }
    let f = &fm.flags;
    let conflicts = [
        (f.only_strict && f.no_strict, "onlyStrict and noStrict"),
        (
            f.only_strict && f.raw,
            "onlyStrict and raw (raw implies noStrict)",
        ),
        (
            f.only_strict && f.module,
            "onlyStrict and module (module code is always strict)",
        ),
        (
            f.no_strict && f.module,
            "noStrict and module (module code is always strict)",
        ),
        (
            f.can_block_is_false && f.can_block_is_true,
            "CanBlockIsFalse and CanBlockIsTrue",
        ),
    ];
    if let Some((_, what)) = conflicts.iter().find(|(c, _)| *c) {
        return err(flags_line, format!("contradictory flags: {what}"));
    }
    Ok(fm)
}

fn scalar(line: usize, key: &str, v: Value) -> Result<String, FrontMatterError> {
    match v {
        Value::Scalar(s) => Ok(s),
        Value::Empty => Ok(String::new()),
        _ => err(line, format!("`{key}` must be a string")),
    }
}

fn list(line: usize, key: &str, v: Value) -> Result<Vec<String>, FrontMatterError> {
    match v {
        Value::List(l) => Ok(l),
        Value::Empty => Ok(Vec::new()),
        _ => err(
            line,
            format!("`{key}` must be a list, `[a, b]` or `- a` lines"),
        ),
    }
}

fn negative(line: usize, v: Value) -> Result<Negative, FrontMatterError> {
    let Value::Map(pairs) = v else {
        return err(line, "`negative` must be a mapping with `phase` and `type`");
    };
    let mut phase = None;
    let mut ty = None;
    for (k, val) in pairs {
        match k.as_str() {
            "phase" => {
                let Some(p) = Phase::parse(&val) else {
                    return err(
                        line,
                        format!("negative phase `{val}` is not parse, resolution or runtime"),
                    );
                };
                phase = Some(p);
            }
            "type" => {
                if val.is_empty()
                    || !val
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
                {
                    return err(
                        line,
                        format!("negative type `{val}` is not a constructor name"),
                    );
                }
                ty = Some(val);
            }
            other => return err(line, format!("unknown key `{other}` in `negative`")),
        }
    }
    match (phase, ty) {
        (Some(phase), Some(error_type)) => Ok(Negative { phase, error_type }),
        (None, _) => err(line, "`negative` has no `phase`"),
        (_, None) => err(line, "`negative` has no `type`"),
    }
}

fn indent_of(s: &str) -> usize {
    s.len() - s.trim_start_matches(' ').len()
}

fn is_blank_or_comment(s: &str) -> bool {
    let t = s.trim();
    t.is_empty() || t.starts_with('#')
}

/// Splits the body into top-level entries: (line, key, value).
fn parse_entries(lines: &[(usize, &str)]) -> Result<Vec<(usize, String, Value)>, FrontMatterError> {
    let mut out: Vec<(usize, String, Value)> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let (ln, text) = lines[i];
        if text.starts_with('\t') {
            return err(ln, "a tab in indentation (YAML forbids it)");
        }
        if is_blank_or_comment(text) {
            i += 1;
            continue;
        }
        if indent_of(text) != 0 {
            return err(
                ln,
                format!("indented line outside any key: `{}`", text.trim()),
            );
        }
        let Some(colon) = key_colon(text) else {
            return err(ln, format!("expected `key: value`, found `{text}`"));
        };
        let key = text[..colon].to_string();
        if out.iter().any(|(_, k, _)| *k == key) {
            return err(ln, format!("key `{key}` given twice"));
        }
        let rest = text[colon + 1..].trim();
        // The lines that belong to this key: indented or blank, up to the
        // next column-0 line.
        let mut j = i + 1;
        while j < lines.len() {
            let t = lines[j].1;
            if t.starts_with('\t') {
                return err(lines[j].0, "a tab in indentation (YAML forbids it)");
            }
            if !t.trim().is_empty() && indent_of(t) == 0 {
                break;
            }
            j += 1;
        }
        let nested = &lines[i + 1..j];
        let value = parse_value(ln, rest, nested)?;
        out.push((ln, key, value));
        i = j;
    }
    Ok(out)
}

/// The byte index of the colon ending a `key:` at the start of `text`.
fn key_colon(text: &str) -> Option<usize> {
    let colon = text.find(':')?;
    let key = &text[..colon];
    let ok = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    let after = text[colon + 1..].chars().next();
    (ok && after.is_none_or(|c| c == ' ')).then_some(colon)
}

fn parse_value(ln: usize, rest: &str, nested: &[(usize, &str)]) -> Result<Value, FrontMatterError> {
    let has_nested = nested.iter().any(|(_, t)| !is_blank_or_comment(t));
    if rest.starts_with('|') || rest.starts_with('>') {
        return block_scalar(ln, rest, nested).map(Value::Scalar);
    }
    if rest.is_empty() || rest.starts_with('#') {
        if !has_nested {
            return Ok(Value::Empty);
        }
        return block_collection(nested);
    }
    if rest.starts_with('[') {
        if has_nested {
            return err(
                ln,
                "a flow sequence must be on one line (CONTRIBUTING.md, Frontmatter)",
            );
        }
        return flow_list(ln, rest).map(Value::List);
    }
    if rest.starts_with('{') {
        if has_nested {
            return err(ln, "a flow mapping must be on one line");
        }
        return flow_map(ln, rest).map(Value::Map);
    }
    // A scalar, possibly continued on indented lines (folded with spaces).
    let mut text = rest.to_string();
    for (l, t) in nested {
        let t = t.trim();
        if t.is_empty() {
            text.push('\n');
        } else {
            // YAML: a plain scalar's continuation cannot hold `key: value`.
            if key_colon(t).is_some() && !rest.starts_with(['\'', '"']) {
                return err(
                    *l,
                    format!("a nested `{t}` under a key that already has a value"),
                );
            }
            if !text.ends_with('\n') {
                text.push(' ');
            }
            text.push_str(t);
        }
    }
    scalar_text(ln, text.trim_end()).map(Value::Scalar)
}

/// A `|` or `>` block scalar: the indented lines, with their common
/// indentation removed; `>` folds single newlines into spaces.
fn block_scalar(
    ln: usize,
    header: &str,
    nested: &[(usize, &str)],
) -> Result<String, FrontMatterError> {
    let ind = header[1..].trim();
    let ind = ind.split('#').next().unwrap_or("").trim();
    if !ind.chars().all(|c| matches!(c, '-' | '+' | '1'..='9')) {
        return err(ln, format!("bad block scalar header `{header}`"));
    }
    let min = nested
        .iter()
        .filter(|(_, t)| !t.trim().is_empty())
        .map(|(_, t)| indent_of(t))
        .min()
        .unwrap_or(0);
    let body: Vec<&str> = nested
        .iter()
        .map(|(_, t)| if t.trim().is_empty() { "" } else { &t[min..] })
        .collect();
    let text = if header.starts_with('>') {
        let mut s = String::new();
        for l in body {
            if l.is_empty() {
                s.push('\n');
            } else {
                if !s.is_empty() && !s.ends_with('\n') {
                    s.push(' ');
                }
                s.push_str(l);
            }
        }
        s
    } else {
        body.join("\n")
    };
    Ok(text.trim_end().to_string())
}

fn block_collection(nested: &[(usize, &str)]) -> Result<Value, FrontMatterError> {
    let items: Vec<(usize, &str)> = nested
        .iter()
        .filter(|(_, t)| !is_blank_or_comment(t))
        .map(|&(l, t)| (l, t.trim()))
        .collect();
    let first = items[0].1;
    if first == "-" || first.starts_with("- ") {
        let mut out = Vec::new();
        for (l, t) in items {
            let Some(item) = t.strip_prefix('-') else {
                return err(
                    l,
                    format!("expected `- item` in a block sequence, found `{t}`"),
                );
            };
            if !item.is_empty() && !item.starts_with(' ') {
                return err(l, format!("expected `- item`, found `{t}`"));
            }
            out.push(scalar_text(l, item.trim())?);
        }
        return Ok(Value::List(out));
    }
    let mut out: Vec<(String, String)> = Vec::new();
    for (l, t) in items {
        let Some(colon) = key_colon(t) else {
            return err(
                l,
                format!("expected `key: value` in a mapping, found `{t}`"),
            );
        };
        let k = t[..colon].to_string();
        if out.iter().any(|(e, _)| *e == k) {
            return err(l, format!("key `{k}` given twice"));
        }
        let v = t[colon + 1..].trim();
        if v.is_empty() || v.starts_with(['[', '{', '|', '>']) {
            return err(
                l,
                format!("only scalar values are supported under a mapping key, found `{t}`"),
            );
        }
        out.push((k, scalar_text(l, v)?));
    }
    Ok(Value::Map(out))
}

fn flow_list(ln: usize, rest: &str) -> Result<Vec<String>, FrontMatterError> {
    let inner = flow_inner(ln, rest, '[', ']')?;
    split_flow(ln, inner)?
        .into_iter()
        .map(|item| {
            if item.is_empty() {
                return err(ln, format!("an empty item in `{rest}`"));
            }
            scalar_text(ln, item)
        })
        .collect()
}

fn flow_map(ln: usize, rest: &str) -> Result<Vec<(String, String)>, FrontMatterError> {
    let inner = flow_inner(ln, rest, '{', '}')?;
    let mut out: Vec<(String, String)> = Vec::new();
    for item in split_flow(ln, inner)? {
        let Some(colon) = key_colon(item) else {
            return err(ln, format!("expected `key: value` in `{rest}`"));
        };
        let k = item[..colon].to_string();
        if out.iter().any(|(e, _)| *e == k) {
            return err(ln, format!("key `{k}` given twice"));
        }
        out.push((k, scalar_text(ln, item[colon + 1..].trim())?));
    }
    Ok(out)
}

/// The text between a flow collection's brackets; refuses trailing text
/// other than a comment.
fn flow_inner(ln: usize, rest: &str, open: char, close: char) -> Result<&str, FrontMatterError> {
    let Some(end) = rest.rfind(close) else {
        return err(
            ln,
            format!("`{open}` is not closed by `{close}` on its line"),
        );
    };
    let tail = rest[end + 1..].trim();
    if !(tail.is_empty() || tail.starts_with('#')) {
        return err(ln, format!("text after `{close}`: `{tail}`"));
    }
    Ok(rest[1..end].trim())
}

/// Splits flow items at commas outside quotes.
fn split_flow(ln: usize, inner: &str) -> Result<Vec<&str>, FrontMatterError> {
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut quote: Option<char> = None;
    let mut start = 0;
    for (i, c) in inner.char_indices() {
        match (quote, c) {
            (None, '\'' | '"') => quote = Some(c),
            (Some(q), _) if c == q => quote = None,
            (None, ',') => {
                out.push(inner[start..i].trim());
                start = i + 1;
            }
            (None, '[' | ']' | '{' | '}') => {
                return err(ln, "nested flow collections are not supported");
            }
            _ => {}
        }
    }
    if quote.is_some() {
        return err(ln, "an unclosed quote in a flow collection");
    }
    let last = inner[start..].trim();
    if !last.is_empty() || out.is_empty() {
        out.push(last);
    }
    Ok(out)
}

/// A scalar's text: quotes removed (with `''` and the common `"` escapes),
/// or a plain scalar with a trailing ` #` comment removed.
fn scalar_text(ln: usize, s: &str) -> Result<String, FrontMatterError> {
    if let Some(body) = s.strip_prefix('\'') {
        let Some(body) = body.strip_suffix('\'') else {
            return err(ln, format!("unclosed single quote in `{s}`"));
        };
        return Ok(body.replace("''", "'"));
    }
    if let Some(body) = s.strip_prefix('"') {
        let Some(body) = body.strip_suffix('"') else {
            return err(ln, format!("unclosed double quote in `{s}`"));
        };
        let mut out = String::new();
        let mut chars = body.chars();
        while let Some(c) = chars.next() {
            if c != '\\' {
                out.push(c);
                continue;
            }
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(c @ ('"' | '\\' | '/')) => out.push(c),
                Some(other) => return err(ln, format!("unsupported escape `\\{other}` in `{s}`")),
                None => return err(ln, format!("a trailing backslash in `{s}`")),
            }
        }
        return Ok(out);
    }
    let plain = match s.find(" #") {
        Some(i) => &s[..i],
        None => s,
    };
    Ok(plain.trim().to_string())
}

#[cfg(test)]
mod tests;
