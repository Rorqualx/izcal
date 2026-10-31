//! engine262's side: the cases written as scripts and run black-box
//! under Node through engine262's documented command line
//! (`node <package>/lib/node/bin.mjs --no-inspect FILE`, its README's
//! "engine262 CLI"), which defines `print`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use crate::cases::Case;

/// Cases per script, so the work spreads over the jobs.
const CHUNK: usize = 2_000;

/// Defines `run(i, f)`: it calls `f` and prints `i` and the result in
/// the encoding of [`crate::izcal::encode`], or `E` and the constructor
/// name of what it threw. The helpers are bound before any case runs.
const PRELUDE: &str = r"'use strict';
const dv = new DataView(new ArrayBuffer(8));
const hex4 = (u) => u.toString(16).padStart(4, '0');
const enc = (v) => {
  switch (typeof v) {
    case 'undefined': return 'u';
    case 'boolean': return v ? 't' : 'f';
    case 'number': {
      if (v !== v) return 'NaN';
      dv.setFloat64(0, v);
      return 'n' + dv.getBigUint64(0).toString(16).padStart(16, '0');
    }
    case 'string': {
      let s = 's';
      for (let i = 0; i < v.length; i++) s += hex4(v.charCodeAt(i));
      return s;
    }
    case 'object': return v === null ? 'z' : 'o';
    default: return '?' + typeof v;
  }
};
const say = typeof print === 'function' ? print : console.log;
const run = (i, f) => {
  let r;
  try { r = enc(f()); } catch (e) {
    r = 'E' + (e !== null && typeof e === 'object' && e.constructor ? e.constructor.name : '?');
  }
  say(i + ' ' + r);
};
";

/// The script for cases `first..first + cases.len()`.
fn script(first: usize, cases: &[Case]) -> String {
    let mut s = String::from(PRELUDE);
    for (j, c) in cases.iter().enumerate() {
        let i = first + j;
        let b =
            c.b.as_ref()
                .map_or_else(String::new, |b| format!(" let b = {};", b.js()));
        s.push_str(&format!(
            "run({i}, () => {{ let a = {};{b} return {}; }});\n",
            c.a.js(),
            c.op.js
        ));
    }
    s
}

/// A result line that engine262 did not print because it crashed on the
/// case: `!` and its error's first line.
pub const CRASH: char = '!';

/// Runs `cases` (numbered from `first`) under `argv` and returns one
/// answer each. A run that stops early has crashed on the case after the
/// last one it printed: that case's answer is [`CRASH`] and the error's
/// first line, and the run resumes from the case after it.
fn run_cases(
    first: usize,
    cases: &[Case],
    dir: &Path,
    argv: &[&std::ffi::OsStr],
) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::with_capacity(cases.len());
    while out.len() < cases.len() {
        let from = first + out.len();
        let path = dir.join(format!("cases-{from}.js"));
        std::fs::write(&path, script(from, &cases[out.len()..]))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let (prog, rest) = argv.split_first().ok_or("no program")?;
        let run = Command::new(prog)
            .args(rest)
            .arg(&path)
            .output()
            .map_err(|e| format!("running {}: {e}", Path::new(prog).display()))?;
        let _ = std::fs::remove_file(&path);
        let stdout = String::from_utf8_lossy(&run.stdout);
        for l in stdout.lines() {
            let Some((i, r)) = l.split_once(' ') else {
                continue;
            };
            // Lines arrive in case order; anything else is not a result.
            if i.parse::<usize>().ok() == Some(first + out.len()) {
                out.push(r.to_owned());
            }
        }
        if out.len() < cases.len() {
            if run.status.success() {
                return Err(format!(
                    "{} exited normally after {} of {} cases",
                    path.display(),
                    out.len(),
                    cases.len()
                ));
            }
            let stderr = String::from_utf8_lossy(&run.stderr);
            // Only the error's own message line (`NameError: text`), never
            // the source excerpt Node prints above it (M-1 part 3).
            let why = stderr
                .lines()
                .map(str::trim)
                .find(|l| {
                    l.split_once(": ").is_some_and(|(name, _)| {
                        name.ends_with("Error") && name.chars().all(|c| c.is_ascii_alphanumeric())
                    })
                })
                .unwrap_or("no message");
            out.push(format!("{CRASH}{why}"));
        }
    }
    Ok(out)
}

/// The answers to `cases` from `argv` (a program and its leading
/// arguments; the script's path is appended), over `jobs` workers.
fn run_all(cases: &[Case], argv: &[&std::ffi::OsStr], jobs: usize) -> Result<Vec<String>, String> {
    let dir = private_scratch("hg-js-oracle").map_err(|e| format!("a scratch directory: {e}"))?;
    let chunks: Vec<(usize, &[Case])> = cases
        .chunks(CHUNK)
        .enumerate()
        .map(|(n, part)| (n * CHUNK, part))
        .collect();
    let queue = Mutex::new(chunks.iter());
    let answers = Mutex::new(vec![String::new(); cases.len()]);
    let failure = Mutex::new(None::<String>);
    std::thread::scope(|s| {
        for _ in 0..jobs {
            s.spawn(|| {
                loop {
                    let next = queue.lock().map(|mut q| q.next()).unwrap_or(None);
                    let Some(&(first, part)) = next else { break };
                    match run_cases(first, part, &dir, argv) {
                        Ok(got) => {
                            if let Ok(mut a) = answers.lock() {
                                for (j, r) in got.into_iter().enumerate() {
                                    a[first + j] = r;
                                }
                            }
                        }
                        Err(e) => {
                            if let Ok(mut f) = failure.lock() {
                                f.get_or_insert(e);
                            }
                        }
                    }
                }
            });
        }
    });
    let _ = std::fs::remove_dir_all(&dir);
    if let Some(e) = failure.into_inner().unwrap_or(None) {
        return Err(e);
    }
    answers
        .into_inner()
        .map_err(|_| "a worker panicked".to_owned())
}

/// engine262's answer to every case.
pub fn run(
    cases: &[Case],
    engine262: &Path,
    node: &Path,
    jobs: usize,
) -> Result<Vec<String>, String> {
    let bin = engine262.join("lib/node/bin.mjs");
    if !bin.is_file() {
        return Err(format!(
            "{} is not an unpacked engine262 package (no lib/node/bin.mjs); \
             tools/fetch-test-data.sh fetches the pinned one",
            engine262.display()
        ));
    }
    run_all(
        cases,
        &[
            node.as_os_str(),
            bin.as_os_str(),
            std::ffi::OsStr::new("--no-inspect"),
        ],
        jobs,
    )
}

/// Node's own answer (its engine, run black-box) to `cases`: the second
/// witness for a case engine262 crashed on. Node has no `print`; the
/// prelude falls back to `console.log`.
pub fn run_node(cases: &[Case], node: &Path) -> Result<Vec<String>, String> {
    run_all(cases, &[node.as_os_str()], 1)
}

/// A new scratch directory under the system temp directory, made by this
/// call alone: the name has the process ID, the time and a counter, and
/// it is created exclusively (and, on Unix, readable only by this user),
/// so a path someone else placed there first is never used, and removing
/// the directory afterwards removes only what this process made.
fn private_scratch(prefix: &str) -> std::io::Result<PathBuf> {
    let base = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    for n in 0..100u32 {
        let path = base.join(format!("{prefix}-{}-{nanos:x}-{n}", std::process::id()));
        #[cfg(unix)]
        let b = {
            let mut b = std::fs::DirBuilder::new();
            std::os::unix::fs::DirBuilderExt::mode(&mut b, 0o700);
            b
        };
        #[cfg(not(unix))]
        let b = std::fs::DirBuilder::new();
        match b.create(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "no free scratch directory name",
    ))
}
