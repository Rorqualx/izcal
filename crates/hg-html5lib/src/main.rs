//! `hg-html5lib`: see the library docs.
//!
//! ```text
//! hg-html5lib [--dir DIR] [--back generated|table|both] [--suite tokenizer|encoding]
//! ```
//!
//! `--dir` names the **pinned tree**, not one suite's directory: the
//! tokenizer suite is `tokenizer/` under it and the encoding suite is
//! `encoding/`. Omitted, it is found in the test-data cache.
//!
//! Exit status: 0 when the pass count is at least `BASELINE`, 1 when it
//! is below it, 2 when the suite cannot be read.

use std::path::PathBuf;
use std::process::ExitCode;

use hg_html_syntax::Back;
use hg_html5lib::{BASELINE, Ledger, Outcome, cases, encoding, run_case};

const USAGE: &str =
    "usage: hg-html5lib [--dir DIR] [--back generated|table|both] [--suite tokenizer|encoding]";

fn main() -> ExitCode {
    let mut dir: Option<PathBuf> = None;
    let mut backs = vec![Back::Generated, Back::Table];
    let mut suite = Suite::Tokenizer;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--dir" => match args.next() {
                Some(v) => dir = Some(PathBuf::from(v)),
                None => return usage(),
            },
            "--back" => match args.next().as_deref() {
                Some("generated") => backs = vec![Back::Generated],
                Some("table") => backs = vec![Back::Table],
                Some("both") => backs = vec![Back::Generated, Back::Table],
                _ => return usage(),
            },
            "--suite" => match args.next().as_deref() {
                Some("tokenizer") => suite = Suite::Tokenizer,
                Some("encoding") => suite = Suite::Encoding,
                _ => return usage(),
            },
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => return usage(),
        }
    }
    if suite == Suite::Encoding {
        return run_encoding(dir);
    }
    // `--dir` is the pinned tree, for both suites; each appends its
    // own directory. One flag, one meaning.
    let dir = match dir.map_or_else(hg_html5lib::tree_dir, Ok) {
        Ok(d) => d.join(hg_html5lib::TOKENIZER_DIR),
        Err(e) => return fail(&e),
    };
    if !dir.is_dir() {
        return fail(&format!(
            "{} is not there; run `tools/fetch-test-data.sh html5lib`",
            dir.display()
        ));
    }
    let cases = match cases::read_dir(&dir) {
        Ok(c) => c,
        Err(e) => return fail(&e),
    };

    let mut ledger = Ledger::default();
    for case in &cases {
        if case.lone_surrogate {
            ledger.lone_surrogate = ledger.lone_surrogate.saturating_add(1);
        }
        for state in &case.initial_states {
            for back in &backs {
                let got = run_case(case, *state, *back);
                let outcome = if got.tokens == case.output {
                    if got.errors == case.errors {
                        Outcome::Pass
                    } else {
                        Outcome::ErrorsDiffer
                    }
                } else {
                    Outcome::TokensDiffer
                };
                ledger.record(&case.file, outcome, || {
                    format!(
                        "{} [{}] {:?}: {:?}\n      wanted {:?}\n      got    {:?}\n      errors \
                         wanted {:?}\n      errors got    {:?}",
                        case.file,
                        state.name(),
                        case.description,
                        case.input,
                        case.output,
                        got.tokens,
                        case.errors,
                        got.errors
                    )
                });
            }
        }
    }
    print!("{}", ledger.render());
    println!("hg-html5lib: {} cases at {}", cases.len(), hg_html5lib::PIN);
    if ledger.pass < BASELINE {
        eprintln!(
            "hg-html5lib: {} passes, below the recorded baseline of {BASELINE}",
            ledger.pass
        );
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn fail(e: &str) -> ExitCode {
    eprintln!("hg-html5lib: {e}");
    ExitCode::from(2)
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

/// Which suite to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Suite {
    Tokenizer,
    Encoding,
}

/// The encoding suite: HTML's "determine the character encoding" over
/// the pinned cases (ST-D16).
fn run_encoding(dir: Option<PathBuf>) -> ExitCode {
    let dir = match dir.map_or_else(hg_html5lib::tree_dir, Ok) {
        Ok(d) => d.join(encoding::DIR),
        Err(e) => {
            eprintln!("hg-html5lib: {e}");
            return ExitCode::from(2);
        }
    };
    let cases = match encoding::read_all(&dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("hg-html5lib: {e}");
            return ExitCode::from(2);
        }
    };
    // The window HTML leaves to the user agent, measured rather than
    // assumed: the suite says what each choice costs.
    for limit in [1024_usize, 2048, 4096, 8192, usize::MAX] {
        let m = encoding::run_within(&cases, limit);
        println!(
            "  window {:>5}: {}/{}",
            if limit == usize::MAX {
                "all".to_owned()
            } else {
                limit.to_string()
            },
            m.pass,
            m.runs
        );
    }
    let l = encoding::run(&cases);
    for (file, (runs, pass)) in &l.by_file {
        println!("  {file}: {pass}/{runs}");
    }
    for e in &l.examples {
        println!("  {e}");
    }
    let share = l.pass.saturating_mul(1000).wrapping_div(l.runs.max(1));
    println!(
        "hg-html5lib: encoding {}/{} ({}.{}%), baseline {}",
        l.pass,
        l.runs,
        share.wrapping_div(10),
        share.wrapping_rem(10),
        encoding::BASELINE
    );
    if l.pass < encoding::BASELINE {
        eprintln!(
            "hg-html5lib: {} below the baseline of {}",
            l.pass,
            encoding::BASELINE
        );
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
