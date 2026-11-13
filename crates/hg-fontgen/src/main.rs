//! `hg-fontgen`: see the library docs.
//!
//! ```text
//! hg-fontgen [--root DIR] [generate | check]
//! ```
//!
//! Exit status: 0 on success, 1 when the checked-in output is stale, 2
//! when a description cannot be read, parsed or checked.

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: hg-fontgen [--root DIR] [generate | check]";

fn main() -> ExitCode {
    let mut root = PathBuf::from(".");
    let mut mode = "check".to_owned();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => match args.next() {
                Some(v) => root = PathBuf::from(v),
                None => return usage(),
            },
            "generate" | "check" => mode = a,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => return usage(),
        }
    }
    if mode == "generate" {
        return match hg_fontgen::cli::generate(&root) {
            Ok(written) => {
                for p in written {
                    println!("hg-fontgen: wrote {}", p.display());
                }
                ExitCode::SUCCESS
            }
            Err(e) => fail(&e),
        };
    }
    match hg_fontgen::cli::stale(&root) {
        Ok(stale) if stale.is_empty() => {
            println!("hg-fontgen: the generated files are fresh");
            ExitCode::SUCCESS
        }
        Ok(stale) => {
            for p in &stale {
                eprintln!("hg-fontgen: G29: {} is stale", p.display());
            }
            eprintln!("hg-fontgen: run `cargo run -p hg-fontgen -- generate`");
            ExitCode::from(1)
        }
        Err(e) => fail(&e),
    }
}

fn fail(e: &str) -> ExitCode {
    eprintln!("hg-fontgen: {e}");
    ExitCode::from(2)
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
