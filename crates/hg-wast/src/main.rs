//! `hg-wast`: reads the WebAssembly specification's `.wast` scripts.
//!
//! ```text
//! hg-wast parse --suite <dir> [--quiet] [--verify] [--floor <k>=<n>]
//! hg-wast check --suite <dir> --engine <exe> --out <dir> [--mutate]
//! ```
//!
//! Parses every `.wast` file under `<dir>` and prints a ledger of the
//! commands it found, by keyword and by file. The exit status is 0 only
//! if every file parsed.
//!
//! The suite is the pinned WebAssembly specification tree, fetched by
//! `tools/fetch-test-data.sh wasm-spec`; its path is given on the
//! command line rather than looked up, so the same binary reads a
//! single file during development and the whole suite in CI.
//!
//! # Why the counts are printed, and floors are checked
//!
//! A reader that stopped recognising a command would still parse every
//! file: it would just find fewer commands, and report success. The
//! census below is what makes that visible — the same reason JS-10
//! (c)'s corruption harness counts what it corrupted. `--floor` fails
//! the run when a keyword's count drops below what the pinned suite is
//! known to hold.
//!
//! `--verify` adds the check that matters most to a reader built on
//! spans: every module the script names is re-parsed from its own span,
//! and must come back the same module. A span that is one byte short
//! hands an engine a module that is not the one the test wrote, and
//! nothing else here would notice.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use hg_wast::engine::{Reference, ScriptEngine};
use hg_wast::lex::position;
use hg_wast::{mutate, render, script};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("parse") => parse_main(&args),
        Some("check") => check_main(&args),
        _ => {
            eprintln!("usage: hg-wast parse --suite <dir> [--quiet] [--verify] [--floor <k>=<n>]");
            eprintln!("       hg-wast check --suite <dir> --engine <exe> --out <dir> [--mutate]");
            ExitCode::from(2)
        }
    }
}

/// `hg-wast check`: the differential against an engine.
///
/// For every script in the suite: run it through the engine; parse it;
/// write it back out from what was parsed; run that through the engine
/// too. The two verdicts must agree. Then, under `--mutate`, change one
/// value the script expects and require the engine to *refuse* the
/// result -- without that, a renderer that wrote vacuous assertions, or
/// none, would agree just as well.
fn check_main(args: &[String]) -> ExitCode {
    let mut suite: Option<PathBuf> = None;
    let mut exe: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut quiet = false;
    let mut mutate_too = false;
    let mut i = 1;
    while i < args.len() {
        let flag = args[i].clone();
        match flag.as_str() {
            "--suite" | "--engine" | "--out" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("hg-wast: {flag} needs a path");
                    return ExitCode::from(2);
                };
                let target = match flag.as_str() {
                    "--suite" => &mut suite,
                    "--engine" => &mut exe,
                    _ => &mut out,
                };
                *target = Some(PathBuf::from(v));
            }
            "--quiet" => quiet = true,
            "--mutate" => mutate_too = true,
            other => {
                eprintln!("hg-wast: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }
    let (Some(suite), Some(exe), Some(out)) = (suite, exe, out) else {
        eprintln!("hg-wast: check needs --suite, --engine and --out");
        return ExitCode::from(2);
    };

    let engine = Reference::new(&exe);
    let version = engine.version();
    println!("engine {} {version}", engine.name());

    let mut files = Vec::new();
    if let Err(e) = collect(&suite, &mut files) {
        eprintln!("hg-wast: {}: {e}", suite.display());
        return ExitCode::FAILURE;
    }
    files.sort();

    let mut agreed = 0usize;
    let mut disagreed = 0usize;
    let mut mutated = 0usize;
    let mut uncaught = 0usize;
    let mut unmutable = 0usize;
    let mut baseline_failed = 0usize;

    for path in &files {
        let rel = show(&suite, path);
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                println!("FAIL {rel}: {e}");
                disagreed += 1;
                continue;
            }
        };
        let base = engine.run_script(path);
        if !base.passed {
            // The engine does not pass the original. Nothing can be
            // concluded about the rendering from such a file, so it is
            // reported rather than counted either way.
            println!("BASE {rel}: the engine does not pass the original");
            baseline_failed += 1;
            continue;
        }
        let cmds = match script::parse(&src) {
            Ok(c) => c,
            Err(e) => {
                let p = position(&src, e.offset);
                println!("FAIL {rel}:{}:{}: {}", p.line, p.column, e.message);
                disagreed += 1;
                continue;
            }
        };
        let rendered = match write_rendering(&out, &rel, &render::script(&src, &cmds)) {
            Ok(p) => p,
            Err(e) => {
                println!("FAIL {rel}: {e}");
                disagreed += 1;
                continue;
            }
        };
        let after = engine.run_script(&rendered);
        if !after.passed {
            println!("DIFF {rel}: the engine passes the original and refuses the rendering");
            for line in after.output.lines().take(4) {
                println!("     {line}");
            }
            disagreed += 1;
            continue;
        }
        agreed += 1;
        if !quiet {
            println!("ok   {rel}");
        }

        if mutate_too {
            let mut changed = cmds.clone();
            let Some(what) = mutate::flip_one_expected(&src, &mut changed) else {
                unmutable += 1;
                continue;
            };
            let name = format!("{rel}.mutated");
            let p = match write_rendering(&out, &name, &render::script(&src, &changed)) {
                Ok(p) => p,
                Err(e) => {
                    println!("FAIL {rel}: {e}");
                    uncaught += 1;
                    continue;
                }
            };
            if engine.run_script(&p).passed {
                println!("MUTE {rel}: {what}, and the engine still passes it");
                uncaught += 1;
            } else {
                mutated += 1;
            }
        }
    }

    println!();
    println!(
        "files {}, agreed {agreed}, disagreed {disagreed}, engine refuses the original {baseline_failed}",
        files.len()
    );
    if mutate_too {
        println!("mutations caught {mutated}, uncaught {uncaught}, nothing to change {unmutable}");
    }
    if disagreed > 0 || uncaught > 0 || baseline_failed > 0 {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Writes a rendering under `out`, keeping the suite's own layout so a
/// failure names a file a person can open.
fn write_rendering(out: &Path, rel: &str, text: &str) -> Result<PathBuf, String> {
    let path = out.join(rel);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// `hg-wast parse`: the reader and its census.
fn parse_main(args: &[String]) -> ExitCode {
    let mut suite: Option<PathBuf> = None;
    let mut quiet = false;
    let mut verify = false;
    let mut floors: Vec<(String, usize)> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--suite" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("hg-wast: --suite needs a directory");
                    return ExitCode::from(2);
                };
                suite = Some(PathBuf::from(v));
            }
            "--quiet" => quiet = true,
            "--verify" => verify = true,
            "--floor" => {
                i += 1;
                let Some((k, n)) = args.get(i).and_then(|v| v.split_once('=')) else {
                    eprintln!("hg-wast: --floor needs <keyword>=<n>");
                    return ExitCode::from(2);
                };
                let Ok(n) = n.parse() else {
                    eprintln!("hg-wast: --floor needs a number, not `{n}`");
                    return ExitCode::from(2);
                };
                floors.push((k.to_string(), n));
            }
            other => {
                eprintln!("hg-wast: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }
    let Some(suite) = suite else {
        eprintln!("hg-wast: parse needs --suite <dir>");
        return ExitCode::from(2);
    };

    let mut files = Vec::new();
    if let Err(e) = collect(&suite, &mut files) {
        eprintln!("hg-wast: {}: {e}", suite.display());
        return ExitCode::FAILURE;
    }
    files.sort();
    if files.is_empty() {
        eprintln!("hg-wast: no .wast files under {}", suite.display());
        return ExitCode::FAILURE;
    }

    let mut counts: Vec<(&'static str, usize)> = Vec::new();
    let mut total = 0usize;
    let mut failed = 0usize;
    let mut modules = 0usize;
    for path in &files {
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                println!("FAIL {}: {e}", show(&suite, path));
                failed += 1;
                continue;
            }
        };
        match script::parse(&src) {
            Ok(cmds) => {
                if verify {
                    let mut bad = Vec::new();
                    for c in &cmds {
                        for m in c.modules() {
                            modules += 1;
                            if let Err(why) = m.reparses(&src) {
                                let p = position(&src, m.span.start);
                                bad.push(format!("{}:{}: {why}", p.line, p.column));
                            }
                        }
                    }
                    if !bad.is_empty() {
                        for b in &bad {
                            println!("SPAN {} {b}", show(&suite, path));
                        }
                        failed += 1;
                        continue;
                    }
                }
                for c in &cmds {
                    let k = c.keyword();
                    match counts.iter_mut().find(|(n, _)| *n == k) {
                        Some((_, n)) => *n += 1,
                        None => counts.push((k, 1)),
                    }
                }
                total += cmds.len();
                if !quiet {
                    println!("ok   {} {} commands", show(&suite, path), cmds.len());
                }
            }
            Err(e) => {
                let p = position(&src, e.offset);
                println!(
                    "FAIL {}:{}:{}: {}",
                    show(&suite, path),
                    p.line,
                    p.column,
                    e.message
                );
                failed += 1;
            }
        }
    }

    counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!();
    println!(
        "files {}, parsed {}, failed {failed}",
        files.len(),
        files.len() - failed
    );
    println!("commands {total}");
    if verify {
        println!("module spans re-parsed {modules}");
    }
    for (k, n) in &counts {
        println!("  {k:<16} {n}");
    }

    let mut short = 0usize;
    for (k, want) in &floors {
        let got = counts.iter().find(|(n, _)| n == k).map_or(0, |(_, n)| *n);
        if got < *want {
            println!("FLOOR {k}: {got}, at least {want} expected");
            short += 1;
        }
    }
    if failed > 0 || short > 0 {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Every `.wast` file under `dir`, recursively.
fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "wast") {
            out.push(path);
        }
    }
    Ok(())
}

/// A path as it reads under the suite root.
fn show(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
