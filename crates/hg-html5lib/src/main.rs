//! `hg-html5lib`: see the library docs.
//!
//! ```text
//! hg-html5lib [--dir DIR] [--back generated|table|both]
//! ```
//!
//! Exit status: 0 when the pass count is at least `BASELINE`, 1 when it
//! is below it, 2 when the suite cannot be read.

use std::path::PathBuf;
use std::process::ExitCode;

use hg_html_syntax::Back;
use hg_html5lib::{BASELINE, Ledger, Outcome, cases, run_case};

const USAGE: &str = "usage: hg-html5lib [--dir DIR] [--back generated|table|both]";

fn main() -> ExitCode {
    let mut dir: Option<PathBuf> = None;
    let mut backs = vec![Back::Generated, Back::Table];
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
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => return usage(),
        }
    }
    let dir = match dir.map_or_else(hg_html5lib::suite_dir, Ok) {
        Ok(d) => d,
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
