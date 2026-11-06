//! Discovering test files, expanding them into runs, and running them on
//! an engine in parallel.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::classify::{Status, classify};
use crate::engine::{Engine, Job};
use crate::exclusions::Exclusions;
use crate::expand::{self, Harness, Mode};
use crate::frontmatter::{self, FrontMatter};
use crate::ledger::Entry;

/// What to run.
#[derive(Clone, Debug)]
pub struct RunConfig {
    /// test262's root (holding `test/` and `harness/`).
    pub test262: PathBuf,
    /// Path prefixes under `test/`; empty runs everything.
    pub filters: Vec<String>,
    /// Concurrent runs.
    pub jobs: usize,
    /// Per-run time limit.
    pub timeout: Duration,
    /// Features and paths not to run.
    pub exclusions: Exclusions,
    /// Print progress to stderr.
    pub progress: bool,
}

/// Lists the test files under `root/test`, as sorted paths relative to
/// `test/`. Files whose name holds `_FIXTURE` are not tests
/// (INTERPRETING.md, "Modules").
pub fn discover(root: &Path, filters: &[String]) -> std::io::Result<Vec<String>> {
    let base = root.join("test");
    let mut out = Vec::new();
    let mut stack = vec![base.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let ty = entry.file_type()?;
            if ty.is_dir() {
                stack.push(path);
                continue;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.ends_with(".js") || name.contains("_FIXTURE") {
                continue;
            }
            let rel = path
                .strip_prefix(&base)
                .unwrap()
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            if filters.is_empty() || filters.iter().any(|f| rel.starts_with(f.as_str())) {
                out.push(rel);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// A test file prepared for running: its runs, or why it has none.
enum Prepared {
    Runs(Vec<(Mode, Job)>, FrontMatter),
    NotRun(Vec<Mode>, Vec<String>, Status, String),
}

fn prepare(cfg: &RunConfig, harness: &Harness, engine: &dyn Engine, rel: &str) -> Prepared {
    let file = cfg.test262.join("test").join(rel);
    let source = match std::fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            return Prepared::NotRun(
                vec![Mode::Sloppy],
                Vec::new(),
                Status::Error,
                format!("read: {e}"),
            );
        }
    };
    let fm = match frontmatter::parse_test_file(&source) {
        Ok(fm) => fm,
        Err(e) => {
            return Prepared::NotRun(vec![Mode::Sloppy], Vec::new(), Status::Error, e.to_string());
        }
    };
    let modes = expand::modes(&fm);
    if let Some(m) = cfg.exclusions.matches(rel, &fm.features) {
        return Prepared::NotRun(modes, fm.features, Status::Excluded, m.to_string());
    }
    if let Some(gap) = expand::host_gap(&fm, &source, &engine.host()) {
        return Prepared::NotRun(modes, fm.features, Status::Excluded, gap);
    }
    let harness_text = match harness.text_for(&fm) {
        Ok(h) => h,
        Err(e) => return Prepared::NotRun(modes, fm.features, Status::Error, e),
    };
    let runs = modes
        .iter()
        .map(|&m| {
            (
                m,
                expand::job(rel, &file, &source, &fm, &harness_text, m, cfg.timeout),
            )
        })
        .collect();
    Prepared::Runs(runs, fm)
}

/// Runs the selected tests and returns one ledger entry per run, sorted.
pub fn run(cfg: &RunConfig, engine: &dyn Engine) -> std::io::Result<Vec<Entry>> {
    let tests = discover(&cfg.test262, &cfg.filters)?;
    let harness = Harness::new(cfg.test262.join("harness"));
    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results: Mutex<Vec<Entry>> = Mutex::new(Vec::with_capacity(tests.len() * 2));
    let jobs = cfg.jobs.max(1);
    std::thread::scope(|s| {
        for worker in 0..jobs {
            let (tests, harness, next, done, results) = (&tests, &harness, &next, &done, &results);
            s.spawn(move || {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(rel) = tests.get(i) else { break };
                    let mut out = Vec::new();
                    match prepare(cfg, harness, engine, rel) {
                        Prepared::NotRun(modes, features, status, detail) => {
                            for mode in modes {
                                out.push(Entry {
                                    status,
                                    mode,
                                    path: rel.clone(),
                                    features: features.clone(),
                                    detail: detail.clone(),
                                });
                            }
                        }
                        Prepared::Runs(runs, fm) => {
                            for (mode, job) in runs {
                                let outcome = engine.run(&job, worker);
                                let mut v =
                                    classify(fm.negative.as_ref(), fm.flags.is_async, &outcome);
                                if v.status != Status::Pass && !outcome.diagnostics.is_empty() {
                                    v.detail = format!("{} | {}", v.detail, outcome.diagnostics);
                                }
                                out.push(Entry {
                                    status: v.status,
                                    mode,
                                    path: rel.clone(),
                                    features: fm.features.clone(),
                                    detail: v.detail,
                                });
                            }
                        }
                    }
                    results.lock().unwrap().extend(out);
                    let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                    if cfg.progress && (n % 500 == 0 || n == tests.len()) {
                        eprint!("\r{n}/{} files", tests.len());
                        let _ = std::io::stderr().flush();
                    }
                }
            });
        }
    });
    if cfg.progress {
        eprintln!();
    }
    let mut entries = results.into_inner().unwrap();
    entries.sort_by(|a, b| (a.path.as_str(), a.mode).cmp(&(b.path.as_str(), b.mode)));
    Ok(entries)
}
