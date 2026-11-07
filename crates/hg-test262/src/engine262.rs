//! The engine262 adapter: engine262 run black-box under Node through its
//! documented command line (its README, "engine262 CLI"), as the
//! test-only oracle that validates this runner at JS-M0 (ORC-1).
//!
//! engine262 is never linked or vendored and its source is never read
//! (HGBA-2 M-1 part 3). Everything here rests on the CLI's documented
//! options and on behaviour observed from outside, recorded in
//! `measurements/milestones/js-m0/test262-engine262.md`:
//!
//! - `node <pkg>/lib/node/bin.mjs --no-inspect --features=... FILE` runs
//!   FILE as a script with `print` and `$262` defined (the `$262` of
//!   INTERPRETING.md without `agent` and `IsHTMLDDA`, probed 31 Oct 2026).
//! - Dynamic `import('./x')` from that script resolves against the
//!   script's directory; an absolute path does not load.
//!
//! # How a run is driven
//!
//! Each run writes a small driver script into the worker's scratch
//! directory and runs it. The driver keeps `print` and `$262` in a
//! closure, so a test that replaces them cannot hide its result, and
//! reports through lines starting with [`MARK`].
//!
//! - **Script**: the driver passes the whole source (prefix, harness,
//!   test) to `$262.evalScript` and catches what escapes, so the
//!   constructor name is read from the thrown value itself rather than
//!   from a printed message. Unless the test is `raw`, a `print` of the
//!   evaluation marker is inserted right after the strict prefix: if it
//!   printed, the error came from evaluation (runtime), else from
//!   parsing. For `raw` tests nothing may be inserted and the phase is
//!   reported as unknown.
//! - **Script naming `./` files** (dynamic `import()`): code run by
//!   `$262.evalScript` has no referrer, so a relative specifier cannot
//!   resolve. Such a script runs directly as the CLI's file, beside
//!   copies of the files it names: prefix, evaluation marker, harness,
//!   test, then a completion marker as a fresh statement. An uncaught
//!   exception is read from the CLI's own report on stderr
//!   ([`parse_uncaught`]), which shows the error's `name` rather than its
//!   constructor; for test262's built-in error types the two agree.
//! - **Module**: the driver evaluates the harness with `$262.evalScript`
//!   (global scope, as INTERPRETING.md requires), then `import()`s a copy
//!   of the test placed beside it with the `./` files it names
//!   (recursively), so its specifiers resolve as in test262. Unless the
//!   test is `raw`, every `.js` module of the copied graph starts with the
//!   evaluation marker: no module evaluates before the whole graph has
//!   linked, so a marker means the error is a runtime one. With no
//!   marker, the error is a parse error if engine262's error names a
//!   location (`file:line`) in the test file itself, else a resolution
//!   error (a dependency's parse error or a linking error).
//! - **Async**: every `print` line is collected until the process exits
//!   (engine262 exits when its job queue and timers are empty).

use std::fmt::Write as _;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::engine::{Completion, Engine, HostCapabilities, Job, Outcome, SourceKind};
use crate::frontmatter::Phase;

/// The prefix of the driver's report lines.
pub const MARK: &str = "#izcal-t262#";

/// engine262's optional features enabled for a run: every one its
/// `--help` lists at the pinned commit except `decorators.no-bugfix.1`,
/// which turns a fix off rather than a feature on.
pub const DEFAULT_FEATURES: &str =
    "immutable-arraybuffer,decorators,cleanup-some,regexp-buffer-boundaries,export-defer";

/// The engine262 adapter.
#[derive(Clone, Debug)]
pub struct Engine262 {
    /// The unpacked npm package (holding `lib/node/bin.mjs`).
    pub package: PathBuf,
    /// The Node.js binary.
    pub node: PathBuf,
    /// Extra Node.js options (such as a heap limit).
    pub node_args: Vec<String>,
    /// The `--features=` list.
    pub features: String,
    /// Scratch space; each worker uses `scratch/w<N>/`.
    pub scratch: PathBuf,
}

impl Engine262 {
    /// An adapter with the default features and Node options.
    pub fn new(package: PathBuf, node: PathBuf, scratch: PathBuf) -> Engine262 {
        Engine262 {
            package,
            node,
            // A runaway test must not take the shared machine's memory.
            node_args: vec!["--max-old-space-size=2048".into()],
            features: DEFAULT_FEATURES.into(),
            scratch,
        }
    }

    fn bin(&self) -> PathBuf {
        self.package.join("lib/node/bin.mjs")
    }
}

/// A JavaScript double-quoted string literal holding `s` exactly: ASCII
/// stays, everything else becomes `\uXXXX` UTF-16 escapes.
pub fn js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            ' '..='~' => out.push(c),
            _ => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    let _ = write!(out, "\\u{unit:04x}");
                }
            }
        }
    }
    out.push('"');
    out
}

/// The statement that reports the start of evaluation.
fn eval_marker() -> String {
    format!("print(\"{MARK}evaluating\");")
}

/// The driver's shared part: reporting a thrown value.
const REPORT_FN: &str = r#"
  function report(e) {
    var name, message;
    try {
      if (e === null || (typeof e !== 'object' && typeof e !== 'function')) {
        name = 'non-object:' + (e === null ? 'null' : typeof e);
      } else {
        var c = e.constructor;
        name = c === undefined || c === null ? String(c) : String(c.name);
      }
    } catch (x) { name = '?'; }
    try {
      message = (e !== null && typeof e === 'object' && 'message' in e) ? String(e.message) : String(e);
    } catch (x) { message = '?'; }
    var stack = '';
    try { if (e !== null && typeof e === 'object' && typeof e.stack === 'string') stack = e.stack; } catch (x) {}
    p(M + 'completion throw');
    p(M + 'name ' + name);
    p(M + 'stack ' + stack);
    p(M + 'message ' + message);
    p(M + 'end');
  }
"#;

/// The driver for a script job.
pub fn script_driver(job: &Job) -> String {
    let mut source = String::new();
    source.push_str(job.prefix);
    if !job.raw {
        source.push_str(&eval_marker());
        source.push('\n');
    }
    source.push_str(&job.harness);
    source.push_str(&job.test);
    format!(
        "(function (p, $) {{\n  var M = {mark};{report}\n  var src = {src};\n  try {{ $.evalScript(src); }} catch (e) {{ report(e); return; }}\n  p(M + 'completion normal');\n}})(print, $262);\n",
        mark = js_string(MARK),
        report = REPORT_FN,
        src = js_string(&source),
    )
}

/// The driver for a module job; the module copy is `./<module_name>`.
pub fn module_driver(job: &Job, module_name: &str) -> String {
    format!(
        "(function (p, $) {{\n  var M = {mark};{report}\n  try {{ $.evalScript({harness}); }} catch (e) {{ p(M + 'harness-failed'); report(e); return; }}\n  import({spec}).then(function () {{ p(M + 'completion normal'); }}, report);\n}})(print, $262);\n",
        mark = js_string(MARK),
        report = REPORT_FN,
        harness = js_string(&job.harness),
        spec = js_string(&format!("./{module_name}")),
    )
}

/// A module's text with the evaluation marker first (after a hashbang
/// line). No newline is added, so line numbers stay the file's own.
pub fn instrument_module(text: &str) -> String {
    if text.starts_with("#!") {
        let cut = text.find('\n').map_or(text.len(), |i| i + 1);
        format!("{}{}{}", &text[..cut], eval_marker(), &text[cut..])
    } else {
        format!("{}{}", eval_marker(), text)
    }
}

/// The module copy's text: instrumented unless the test is `raw`.
pub fn module_copy(job: &Job) -> String {
    if job.raw {
        job.test.clone()
    } else {
        instrument_module(&job.test)
    }
}

/// The script a test that names `./` files runs as, directly by the CLI
/// (so `import()` has a referrer): prefix, evaluation marker, harness,
/// test, then a completion marker on a fresh statement.
pub fn direct_script(job: &Job) -> String {
    format!(
        "{}{}\n{}{}\n;print(\"{MARK}completion normal\");\n",
        job.prefix,
        eval_marker(),
        job.harness,
        job.test
    )
}

/// Reads the CLI's own report of an uncaught exception (stderr's first
/// line, observed formats: `TypeError: msg`, `Test262Error { message: ... }`,
/// a quoted string, a number) into a constructor name and message.
pub fn parse_uncaught(stderr: &str) -> Option<(String, String)> {
    let first = stderr.lines().find(|l| !l.trim().is_empty())?.trim();
    let ident_len = first
        .char_indices()
        .find(|&(_, c)| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
        .map_or(first.len(), |(i, _)| i);
    let (ident, rest) = first.split_at(ident_len);
    if ident.is_empty() || ident.starts_with(|c: char| c.is_ascii_digit()) {
        let kind = if first.starts_with(['\'', '"', '`']) {
            "string"
        } else if first.parse::<f64>().is_ok()
            || first.ends_with('n') && first[..first.len() - 1].parse::<f64>().is_ok()
        {
            "number"
        } else {
            return None;
        };
        return Some((format!("non-object:{kind}"), first.to_string()));
    }
    if let Some(msg) = rest.strip_prefix(": ") {
        return Some((ident.to_string(), msg.to_string()));
    }
    if rest.is_empty() || rest.starts_with(" {") || rest == ":" {
        return Some((ident.to_string(), rest.trim().to_string()));
    }
    None
}

/// The `./name` module specifiers a source names, as written (an
/// over-approximation: every `'./...'` or `"./..."` string).
pub fn relative_specifiers(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i + 2 < bytes.len() {
        let q = bytes[i];
        if (q == b'\'' || q == b'"')
            && bytes[i + 1] == b'.'
            && bytes[i + 2] == b'/'
            && let Some(len) = source[i + 1..].find(q as char)
        {
            let spec = &source[i + 1..i + 1 + len];
            if !spec.contains('\n') && !out.iter().any(|s| s == spec) {
                out.push(spec.to_string());
            }
            i += len + 2;
            continue;
        }
        i += 1;
    }
    out
}

/// Whether `rel`, joined to a directory, names a path inside it: only
/// plain names, so no root (`.//etc/x` trims to `/etc/x`, which `join`
/// would take as the whole path), no Windows prefix (`./C:/x`), no `..`.
fn stays_inside(rel: &str) -> bool {
    let mut parts = Path::new(rel).components().peekable();
    parts.peek().is_some() && parts.all(|c| matches!(c, std::path::Component::Normal(_)))
}

/// Copies the files `./` specifiers in `source` name, from `from_dir` into
/// `to_dir`, recursively, skipping `skip`; `.js` copies are instrumented
/// with the evaluation marker when `instrument` is set. Returns how many
/// were copied.
fn copy_module_deps(
    source: &str,
    from_dir: &Path,
    to_dir: &Path,
    skip: Option<&str>,
    instrument: bool,
) -> std::io::Result<usize> {
    let mut queue: Vec<String> = relative_specifiers(source);
    let mut seen: Vec<String> = skip.map(str::to_string).into_iter().collect();
    let mut copied = 0;
    while let Some(spec) = queue.pop() {
        let rel = spec.trim_start_matches("./");
        if !stays_inside(rel) || seen.iter().any(|s| s == rel) {
            continue;
        }
        seen.push(rel.to_string());
        let src = from_dir.join(rel);
        if !src.is_file() {
            continue;
        }
        let dst = to_dir.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match std::fs::read_to_string(&src) {
            Ok(text) => {
                queue.extend(relative_specifiers(&text));
                if instrument && rel.ends_with(".js") {
                    std::fs::write(&dst, instrument_module(&text))?;
                } else {
                    std::fs::write(&dst, text)?;
                }
            }
            Err(_) => {
                std::fs::copy(&src, &dst)?;
            }
        }
        copied += 1;
    }
    Ok(copied)
}

/// The child's output.
struct Output {
    stdout: String,
    stderr: String,
    timed_out: bool,
    status: Option<std::process::ExitStatus>,
}

fn run_with_timeout(mut cmd: Command, timeout: Duration) -> std::io::Result<Output> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    std::thread::scope(|s| {
        let ho = s.spawn(move || {
            let mut b = Vec::new();
            let _ = out.read_to_end(&mut b);
            b
        });
        let he = s.spawn(move || {
            let mut b = Vec::new();
            let _ = err.read_to_end(&mut b);
            b
        });
        let start = Instant::now();
        let mut timed_out = false;
        let mut pause = Duration::from_micros(500);
        let status = loop {
            let polled = match child.try_wait() {
                Ok(p) => p,
                Err(e) => {
                    // Stop the child, or the readers would wait for pipes
                    // that never close.
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(e);
                }
            };
            match polled {
                Some(st) => break Some(st),
                None if start.elapsed() >= timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break None;
                }
                None => {
                    std::thread::sleep(pause);
                    pause = (pause * 2).min(Duration::from_millis(10));
                }
            }
        };
        let stdout = String::from_utf8_lossy(&ho.join().unwrap()).into_owned();
        let stderr = String::from_utf8_lossy(&he.join().unwrap()).into_owned();
        Ok(Output {
            stdout,
            stderr,
            timed_out,
            status,
        })
    })
}

/// Reads the driver's report from the engine's output.
pub fn parse_report(
    stdout: &str,
    kind: SourceKind,
    raw: bool,
    module_file: Option<&str>,
) -> (Option<Completion>, Vec<String>, bool) {
    let mut prints = Vec::new();
    let mut evaluating = false;
    let mut completion = None;
    let mut harness_failed = false;
    let mut name = String::new();
    let mut stack = String::new();
    let mut message: Option<String> = None;
    let mut threw = false;
    let mut in_stack = false;
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix(MARK) {
            in_stack = false;
            if rest == "evaluating" {
                evaluating = true;
            } else if rest == "harness-failed" {
                harness_failed = true;
            } else if rest == "completion normal" {
                completion.get_or_insert(Completion::Normal);
            } else if rest == "completion throw" {
                threw = true;
            } else if let Some(n) = rest.strip_prefix("name ") {
                name = n.to_string();
            } else if let Some(s) = rest.strip_prefix("stack ") {
                stack = s.to_string();
                in_stack = true;
            } else if let Some(m) = rest.strip_prefix("message ") {
                message = Some(m.to_string());
            } else if rest == "end" && (!threw || completion.is_some()) {
                // A report that cannot apply (a second one): end its
                // message, so later lines are prints again.
                message = None;
            } else if rest == "end" {
                let phase = if raw {
                    None
                } else if evaluating {
                    Some(Phase::Runtime)
                } else {
                    match (kind, module_file) {
                        (SourceKind::Script, _) if stack_shows_parse_error(&stack) => {
                            Some(Phase::Parse)
                        }
                        // No statement ran, yet the error has a frame of the
                        // evaluated script: GlobalDeclarationInstantiation,
                        // part of ScriptEvaluation, so runtime.
                        (SourceKind::Script, _) => Some(Phase::Runtime),
                        (SourceKind::Module, Some(file)) if names_location(&stack, file) => {
                            Some(Phase::Parse)
                        }
                        (SourceKind::Module, _) => Some(Phase::Resolution),
                    }
                };
                completion = Some(Completion::Threw {
                    phase,
                    name: name.clone(),
                    message: message.take().unwrap_or_default(),
                });
            }
            continue;
        }
        if let Some(m) = message.as_mut() {
            m.push('\n');
            m.push_str(line);
            continue;
        }
        if in_stack {
            stack.push('\n');
            stack.push_str(line);
            continue;
        }
        prints.push(line.to_string());
    }
    (completion, prints, harness_failed)
}

/// Whether an engine262 error report (an `Error.stack`, or the CLI's
/// stderr) shows a parse or early error: observed, such reports carry the
/// offending source line and a `^` caret line before any `at` frame, and
/// an error thrown while the script runs (including in
/// GlobalDeclarationInstantiation, before any statement) has an `at` frame
/// of the script first. A report with neither counts as a parse error.
pub fn stack_shows_parse_error(report: &str) -> bool {
    for line in report.lines().skip(1) {
        let t = line.trim();
        if t.starts_with("at ") {
            return false;
        }
        if !t.is_empty() && t.chars().all(|c| c == '^') {
            return true;
        }
    }
    true
}

/// `text` holds `file:<digit>`, a source location in `file`.
fn names_location(text: &str, file: &str) -> bool {
    let pat = format!("{file}:");
    text.match_indices(&pat)
        .any(|(i, _)| text[i + pat.len()..].starts_with(|c: char| c.is_ascii_digit()))
}

fn tail(s: &str, n: usize) -> String {
    let s = s.trim();
    let chars: Vec<char> = s.chars().collect();
    let start = chars.len().saturating_sub(n);
    chars[start..].iter().collect()
}

impl Engine for Engine262 {
    fn name(&self) -> &str {
        "engine262"
    }

    fn version(&self) -> String {
        let pkg = std::fs::read_to_string(self.package.join("package.json")).unwrap_or_default();
        pkg.split("\"version\"")
            .nth(1)
            .and_then(|r| r.split('"').nth(1))
            .unwrap_or("unknown")
            .to_string()
    }

    fn host(&self) -> HostCapabilities {
        // Probed through the CLI on 31 Oct 2026: `$262` has no `agent` and
        // no `IsHTMLDDA`, and there is no `Atomics`, so [[CanBlock]] cannot
        // be observed; it is taken as false, which runs the two
        // CanBlockIsFalse tests and excludes the CanBlockIsTrue ones.
        HostCapabilities {
            can_block: false,
            agent: false,
            is_html_dda: false,
        }
    }

    fn run(&self, job: &Job, worker: usize) -> Outcome {
        let dir = self.scratch.join(format!("w{worker}"));
        let crashed = |why: String| Outcome {
            completion: Completion::Crashed(why),
            prints: Vec::new(),
            diagnostics: String::new(),
        };
        let _ = std::fs::remove_dir_all(&dir);
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return crashed(format!("scratch {}: {e}", dir.display()));
        }
        let driver_path = dir.join("__izcal_t262_driver.js");
        let from = job.file.parent().unwrap_or(Path::new("."));
        let mut module_file = None;
        // A script that names `./` files (dynamic `import()`) needs a
        // referrer, which `$262.evalScript` code lacks: it runs directly.
        let direct = job.kind == SourceKind::Script
            && !job.raw
            && !relative_specifiers(&job.test).is_empty();
        let driver = match job.kind {
            SourceKind::Script if direct => {
                if let Err(e) = copy_module_deps(&job.test, from, &dir, None, false) {
                    return crashed(format!("script dependencies: {e}"));
                }
                direct_script(job)
            }
            SourceKind::Script => script_driver(job),
            SourceKind::Module => {
                let name = job
                    .file
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "test.js".into());
                let copy = dir.join(&name);
                if let Err(e) = std::fs::write(&copy, module_copy(job)) {
                    return crashed(format!("module copy: {e}"));
                }
                // Every module of the graph starts with the marker, so an
                // error thrown while a dependency evaluates is runtime.
                if let Err(e) = copy_module_deps(&job.test, from, &dir, Some(&name), !job.raw) {
                    return crashed(format!("module dependencies: {e}"));
                }
                module_file = Some(copy.to_string_lossy().into_owned());
                module_driver(job, &name)
            }
        };
        if let Err(e) = std::fs::write(&driver_path, driver) {
            return crashed(format!("driver: {e}"));
        }
        let mut cmd = Command::new(&self.node);
        cmd.args(&self.node_args)
            .arg(self.bin())
            .arg("--no-inspect")
            .arg(format!("--features={}", self.features))
            .arg(&driver_path)
            .current_dir(&dir);
        let out = match run_with_timeout(cmd, job.timeout) {
            Ok(o) => o,
            Err(e) => return crashed(format!("spawn {}: {e}", self.node.display())),
        };
        let (mut completion, prints, harness_failed) =
            parse_report(&out.stdout, job.kind, job.raw, module_file.as_deref());
        let evaluating = out
            .stdout
            .lines()
            .any(|l| l.strip_prefix(MARK) == Some("evaluating"));
        if direct
            && completion.is_none()
            && let Some((name, message)) = parse_uncaught(&out.stderr)
        {
            let phase = Some(if evaluating || !stack_shows_parse_error(&out.stderr) {
                Phase::Runtime
            } else {
                Phase::Parse
            });
            completion = Some(Completion::Threw {
                phase,
                name,
                message,
            });
        }
        // The first line names the failure (an uncaught error, or Node's own
        // "FATAL ERROR" for an exhausted heap); the tail holds the latest
        // frames.
        let first = out
            .stderr
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim();
        let rest = tail(&out.stderr, 200);
        let diagnostics = if rest.starts_with(first) {
            rest
        } else {
            format!("{first} ... {rest}")
        };
        let completion = if out.timed_out {
            Completion::Timeout
        } else if harness_failed {
            Completion::Crashed(format!(
                "the harness threw before the module ran: {completion:?}"
            ))
        } else {
            match completion {
                Some(c) => c,
                None => Completion::Crashed(format!(
                    "no completion reported (exit {})",
                    out.status
                        .and_then(|s| s.code())
                        .map_or("signal".to_string(), |c| c.to_string())
                )),
            }
        };
        Outcome {
            completion,
            prints,
            diagnostics,
        }
    }
}

#[cfg(test)]
mod tests;
