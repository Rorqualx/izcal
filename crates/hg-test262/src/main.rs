//! `hg-test262`: runs test262 on an engine and compares result ledgers.
//!
//! ```text
//! hg-test262 run --engine engine262 [--filter PREFIX]... [--jobs N]
//!                [--ledger OUT] [--timeout SECS] [--exclusions FILE | --no-exclusions]
//!                [--test262 DIR] [--engine262 DIR] [--node PATH] [--depth N]
//! hg-test262 summary LEDGER [--exclusions FILE] [--depth N] [--features]
//! hg-test262 compare A B
//! hg-test262 frontmatter [--test262 DIR] [--filter PREFIX]...
//! ```
//!
//! `run` prints a per-directory summary and writes the ledger. `summary`
//! restates a ledger, optionally under an exclusion list. `compare` lists
//! the runs whose status changed from A to B. `frontmatter` parses every
//! test file's front matter and lists the ones refused (run it when the
//! test262 pin moves). Exit status: 0 when done
//! (for `compare`, when nothing regressed), 1 when `compare` finds a
//! regression, 2 on a usage or input error.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use cli_error::{Error, Result};
use hg_test262::classify::Status;
use hg_test262::data;
use hg_test262::engine::Engine as _;
use hg_test262::engine262::Engine262;
use hg_test262::exclusions::Exclusions;
use hg_test262::ledger::{self, Ledger};
use hg_test262::runner::{self, RunConfig};

/// A tiny error type: the CLI reports a message and exits 2.
mod cli_error {
    /// A message for the user.
    #[derive(Debug)]
    pub struct Error(pub String);
    /// The CLI's result.
    pub type Result<T> = std::result::Result<T, Error>;
    impl<E: std::fmt::Display> From<E> for Error {
        fn from(e: E) -> Error {
            Error(e.to_string())
        }
    }
}

const USAGE: &str = "usage:
  hg-test262 run --engine engine262 [--filter PREFIX]... [--jobs N] [--ledger OUT]
                 [--timeout SECS] [--exclusions FILE | --no-exclusions]
                 [--test262 DIR] [--engine262 DIR] [--node PATH] [--depth N]
  hg-test262 summary LEDGER [--exclusions FILE] [--depth N] [--features]
  hg-test262 compare A B
  hg-test262 frontmatter [--test262 DIR] [--filter PREFIX]...";

fn default_exclusions(engine: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("exclusions/{engine}.txt"))
}

fn read_exclusions(path: &PathBuf) -> Result<Exclusions> {
    let text =
        std::fs::read_to_string(path).map_err(|e| Error(format!("{}: {e}", path.display())))?;
    Exclusions::parse(&text).map_err(|e| Error(format!("{}: {e}", path.display())))
}

fn value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| Error(format!("{flag} needs a value")))
}

fn print_summary(entries: &[ledger::Entry], depth: usize, features: bool) {
    let (files, runs) = ledger::totals(entries);
    println!(
        "files: {} pass, {} fail ({} timeout), {} excluded; runs: {} pass, {} fail, {} excluded",
        files.pass, files.fail, files.timeout, files.excluded, runs.pass, runs.fail, runs.excluded
    );
    if runs.phase_unverified > 0 {
        println!(
            "negative-test passes with the phase not reported by the adapter: {} runs",
            runs.phase_unverified
        );
    }
    println!();
    print!(
        "{}",
        ledger::table("Directory", &ledger::by_dir(entries, depth))
    );
    if features {
        println!();
        print!("{}", ledger::table("Feature", &ledger::by_feature(entries)));
    }
}

fn cmd_run(mut args: impl Iterator<Item = String>) -> Result<ExitCode> {
    let mut engine_name = None;
    let mut filters = Vec::new();
    let mut jobs = std::thread::available_parallelism().map_or(4, |n| n.get());
    let mut ledger_out = None;
    let mut timeout = 120u64;
    let mut exclusions_path = None;
    let mut no_exclusions = false;
    let mut test262 = None;
    let mut engine262 = None;
    let mut node = PathBuf::from("node");
    let mut depth = 1;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--engine" => engine_name = Some(value(&mut args, &a)?),
            "--filter" => filters.push(
                value(&mut args, &a)?
                    .trim_start_matches("test/")
                    .to_string(),
            ),
            "--jobs" => jobs = value(&mut args, &a)?.parse()?,
            "--ledger" => ledger_out = Some(PathBuf::from(value(&mut args, &a)?)),
            "--timeout" => timeout = value(&mut args, &a)?.parse()?,
            "--exclusions" => exclusions_path = Some(PathBuf::from(value(&mut args, &a)?)),
            "--no-exclusions" => no_exclusions = true,
            "--test262" => test262 = Some(PathBuf::from(value(&mut args, &a)?)),
            "--engine262" => engine262 = Some(PathBuf::from(value(&mut args, &a)?)),
            "--node" => node = PathBuf::from(value(&mut args, &a)?),
            "--depth" => depth = value(&mut args, &a)?.parse()?,
            other => return Err(Error(format!("unknown option {other}\n{USAGE}"))),
        }
    }
    let engine_name = engine_name.ok_or_else(|| Error(format!("--engine is required\n{USAGE}")))?;
    if engine_name != "engine262" {
        return Err(Error(format!(
            "unknown engine `{engine_name}` (available: engine262)"
        )));
    }
    let (test262, test262_commit) = match test262 {
        Some(d) => (d, "unpinned".to_string()),
        None => {
            let (d, set) = data::locate("test262")?;
            (d, set.commit)
        }
    };
    let engine262 = match engine262 {
        Some(d) => d,
        None => data::locate("engine262")?.0,
    };
    let exclusions = if no_exclusions {
        Exclusions::default()
    } else {
        read_exclusions(
            &exclusions_path
                .clone()
                .unwrap_or_else(|| default_exclusions(&engine_name)),
        )?
    };
    let scratch = private_scratch("hg-test262").map_err(|e| format!("a scratch directory: {e}"))?;
    let engine = Engine262::new(engine262, node.clone(), scratch.clone());
    let cfg = RunConfig {
        test262,
        filters: filters.clone(),
        jobs,
        timeout: Duration::from_secs(timeout),
        exclusions,
        progress: true,
    };
    let started = std::time::Instant::now();
    let entries = runner::run(&cfg, &engine)?;
    let elapsed = started.elapsed();
    let _ = std::fs::remove_dir_all(&scratch);

    let node_version = std::process::Command::new(&node)
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let mut ledger = Ledger {
        header: vec![
            (
                "engine".into(),
                format!("{} {}", engine.name(), engine.version()),
            ),
            (
                "host".into(),
                format!("node {node_version}; features {}", engine.features),
            ),
            ("test262".into(), test262_commit),
            ("date".into(), data::today_utc()),
            (
                "exclusions".into(),
                if no_exclusions {
                    "none".into()
                } else {
                    exclusions_path
                        .unwrap_or_else(|| default_exclusions(&engine_name))
                        .display()
                        .to_string()
                },
            ),
            (
                "filters".into(),
                if filters.is_empty() {
                    "-".into()
                } else {
                    filters.join(" ")
                },
            ),
            ("jobs".into(), jobs.to_string()),
            ("timeout".into(), format!("{timeout}s")),
            ("elapsed".into(), format!("{:.0}s", elapsed.as_secs_f64())),
        ],
        entries,
    };
    ledger.sort();
    if let Some(out) = &ledger_out {
        std::fs::write(out, ledger.to_text())
            .map_err(|e| Error(format!("{}: {e}", out.display())))?;
        eprintln!("ledger: {}", out.display());
    }
    println!(
        "{} on test262 {} in {:.0}s",
        ledger.field("engine").unwrap_or(""),
        ledger.field("test262").unwrap_or(""),
        elapsed.as_secs_f64()
    );
    print_summary(&ledger.entries, depth, false);
    Ok(ExitCode::SUCCESS)
}

fn read_ledger(path: &str) -> Result<Ledger> {
    let text = std::fs::read_to_string(path).map_err(|e| Error(format!("{path}: {e}")))?;
    Ledger::parse(&text).map_err(|e| Error(format!("{path}: {e}")))
}

fn cmd_summary(mut args: impl Iterator<Item = String>) -> Result<ExitCode> {
    let mut path = None;
    let mut exclusions = None;
    let mut depth = 1;
    let mut features = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--exclusions" => exclusions = Some(PathBuf::from(value(&mut args, &a)?)),
            "--depth" => depth = value(&mut args, &a)?.parse()?,
            "--features" => features = true,
            _ if path.is_none() && !a.starts_with('-') => path = Some(a),
            other => return Err(Error(format!("unexpected argument {other}\n{USAGE}"))),
        }
    }
    let path = path.ok_or_else(|| Error(format!("summary needs a ledger\n{USAGE}")))?;
    let mut ledger = read_ledger(&path)?;
    for (k, v) in &ledger.header {
        println!("{k}: {v}");
    }
    if let Some(ex) = exclusions {
        ledger::apply_exclusions(&mut ledger.entries, &read_exclusions(&ex)?);
        println!("restated under: {}", ex.display());
    }
    println!();
    print_summary(&ledger.entries, depth, features);
    Ok(ExitCode::SUCCESS)
}

fn status_name(s: Option<Status>) -> &'static str {
    s.map_or("absent", Status::as_str)
}

fn cmd_compare(mut args: impl Iterator<Item = String>) -> Result<ExitCode> {
    let (Some(a), Some(b), None) = (args.next(), args.next(), args.next()) else {
        return Err(Error(format!("compare needs two ledgers\n{USAGE}")));
    };
    let (la, lb) = (read_ledger(&a)?, read_ledger(&b)?);
    let diff = ledger::compare(&la, &lb);
    let (fa, _) = ledger::totals(&la.entries);
    let (fb, _) = ledger::totals(&lb.entries);
    println!(
        "A: {a}: {} files pass, {} fail, {} excluded",
        fa.pass, fa.fail, fa.excluded
    );
    println!(
        "B: {b}: {} files pass, {} fail, {} excluded",
        fb.pass, fb.fail, fb.excluded
    );
    for (title, list) in [
        ("regressions (pass -> not pass)", &diff.regressions),
        ("improvements (not pass -> pass)", &diff.improvements),
        ("other changes", &diff.other),
    ] {
        println!("\n{title}: {}", list.len());
        for c in list {
            println!(
                "  {} -> {}\t{} [{}]\t{}",
                status_name(c.before),
                status_name(c.after),
                c.path,
                c.mode,
                c.detail
            );
        }
    }
    Ok(if diff.regressions.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn cmd_frontmatter(mut args: impl Iterator<Item = String>) -> Result<ExitCode> {
    let mut test262 = None;
    let mut filters = Vec::new();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--test262" => test262 = Some(PathBuf::from(value(&mut args, &a)?)),
            "--filter" => filters.push(
                value(&mut args, &a)?
                    .trim_start_matches("test/")
                    .to_string(),
            ),
            other => return Err(Error(format!("unknown option {other}\n{USAGE}"))),
        }
    }
    let root = match test262 {
        Some(d) => d,
        None => data::locate("test262")?.0,
    };
    let tests = runner::discover(&root, &filters)?;
    let mut refused = 0;
    let mut flags = std::collections::BTreeMap::<&str, usize>::new();
    for rel in &tests {
        let text = std::fs::read_to_string(root.join("test").join(rel))?;
        match hg_test262::frontmatter::parse_test_file(&text) {
            Ok(fm) => {
                let f = &fm.flags;
                for (on, name) in [
                    (f.only_strict, "onlyStrict"),
                    (f.no_strict, "noStrict"),
                    (f.module, "module"),
                    (f.raw, "raw"),
                    (f.is_async, "async"),
                    (f.can_block_is_false, "CanBlockIsFalse"),
                    (f.can_block_is_true, "CanBlockIsTrue"),
                    (f.generated, "generated"),
                    (f.non_deterministic, "non-deterministic"),
                ] {
                    if on {
                        *flags.entry(name).or_default() += 1;
                    }
                }
                if fm.negative.is_some() {
                    *flags.entry("(negative)").or_default() += 1;
                }
            }
            Err(e) => {
                refused += 1;
                println!("{rel}: {e}");
            }
        }
    }
    println!("{} test files, {refused} refused", tests.len());
    for (k, v) in flags {
        println!("  {k}: {v}");
    }
    Ok(if refused == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let result = match args.next().as_deref() {
        Some("run") => cmd_run(args),
        Some("summary") => cmd_summary(args),
        Some("compare") => cmd_compare(args),
        Some("frontmatter") => cmd_frontmatter(args),
        Some("-h" | "--help") => {
            println!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        _ => Err(Error(USAGE.to_string())),
    };
    match result {
        Ok(code) => code,
        Err(Error(msg)) => {
            eprintln!("hg-test262: {msg}");
            ExitCode::from(2)
        }
    }
}

/// A new scratch directory under the system temp directory, made by this
/// call alone: the name has the process ID, the time and a counter, and
/// it is created exclusively (and, on Unix, readable only by this user),
/// so a path someone else placed there first is never used, and removing
/// the directory afterwards removes only what this process made.
fn private_scratch(prefix: &str) -> std::io::Result<std::path::PathBuf> {
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
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "no free scratch directory name",
    ))
}
