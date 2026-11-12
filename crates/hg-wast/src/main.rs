//! `hg-wast`: reads the WebAssembly specification's `.wast` scripts.
//!
//! ```text
//! hg-wast parse --suite <dir> [--quiet]
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

use hg_wast::lex::position;
use hg_wast::script;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut suite: Option<PathBuf> = None;
    let mut quiet = false;
    let mut verify = false;
    let mut floors: Vec<(String, usize)> = Vec::new();
    let mut i = 0;
    if args.first().map(String::as_str) != Some("parse") {
        eprintln!(
            "usage: hg-wast parse --suite <dir> [--quiet] [--verify] [--floor <keyword>=<n>]"
        );
        return ExitCode::from(2);
    }
    i += 1;
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
