//! `hg-graph-lint`: checks the workspace's crate graph (see the library
//! docs for the rules).
//!
//! ```text
//! hg-graph-lint [--root DIR] [--target TRIPLE]... [--locked] [--offline]
//!               [--ast-grep PATH | --no-code-checks]
//! ```
//!
//! `--root` defaults to the current directory; `--target` defaults to
//! the three reference triples. Exit status: 0 when clean, 1 when a rule
//! is broken, 2 when an input cannot be read.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage: hg-graph-lint [--root DIR] [--target TRIPLE]... [--locked] [--offline] [--ast-grep PATH | --no-code-checks]";

fn main() -> ExitCode {
    let mut opts = hg_graph_lint::Options::new(".");
    let mut targets = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => match args.next() {
                Some(v) => opts.root = PathBuf::from(v),
                None => return usage(),
            },
            "--target" => match args.next() {
                Some(v) => targets.push(v),
                None => return usage(),
            },
            "--ast-grep" => match args.next() {
                Some(v) => opts.ast_grep = Some(PathBuf::from(v)),
                None => return usage(),
            },
            "--no-code-checks" => opts.ast_grep = None,
            "--locked" => opts.cargo.locked = true,
            "--offline" => opts.cargo.offline = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => return usage(),
        }
    }
    if !targets.is_empty() {
        opts.triples = targets;
    }
    let started = Instant::now();
    match hg_graph_lint::run(&opts) {
        Ok(report) => {
            print!("{}", report.render());
            let n = report.findings().len();
            let secs = started.elapsed().as_secs_f64();
            if report.is_clean() {
                println!(
                    "hg-graph-lint: clean ({} triple(s), {secs:.1}s)",
                    opts.triples.len()
                );
                ExitCode::SUCCESS
            } else {
                println!("hg-graph-lint: {n} violation(s) ({secs:.1}s)");
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("hg-graph-lint: {e}");
            ExitCode::from(2)
        }
    }
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
