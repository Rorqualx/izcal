//! `hg-unicode-build`: see the library docs.
//!
//! ```text
//! hg-unicode-build [--root DIR] [generate | check]
//! ```
//!
//! `generate` writes `hg-ucd`'s tables; `check` (the default) fails when
//! what is checked in is stale. Exit status: 0 on success, 1 when stale,
//! 2 when an input cannot be read.

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: hg-unicode-build [--root DIR] [generate | check]";

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
    let paths = match hg_unicode_build::Paths::from_env(root) {
        Ok(p) => p,
        Err(e) => return fail(&e),
    };
    if mode == "generate" {
        return match hg_unicode_build::generate(&paths) {
            Ok(p) => {
                println!("hg-unicode-build: wrote {}", p.display());
                ExitCode::SUCCESS
            }
            Err(e) => fail(&e),
        };
    }
    match hg_unicode_build::check(&paths) {
        Ok(()) => {
            println!("hg-unicode-build: {} is fresh", hg_unicode_build::OUTPUT);
            ExitCode::SUCCESS
        }
        Err(e) if e.contains("is stale") => {
            eprintln!("hg-unicode-build: {e}");
            ExitCode::from(1)
        }
        Err(e) => fail(&e),
    }
}

fn fail(e: &str) -> ExitCode {
    eprintln!("hg-unicode-build: {e}");
    eprintln!(
        "note: the database is fetched by `tools/fetch-test-data.sh unicode-ucd unicode-idna-mapping`"
    );
    ExitCode::from(2)
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
