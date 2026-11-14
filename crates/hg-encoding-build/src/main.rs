//! `hg-encoding-build`: see the library docs.
//!
//! ```text
//! hg-encoding-build [--root DIR] [generate | check]
//! ```
//!
//! Exit status: 0 on success, 1 when the checked-in tables are stale, 2
//! when the pinned data cannot be read.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: hg-encoding-build [--root DIR] [generate | check]";

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
        return match hg_encoding_build::cli::generate(&root) {
            Ok(written) => {
                for p in written {
                    println!("hg-encoding-build: wrote {}", p.display());
                }
                ExitCode::SUCCESS
            }
            Err(e) => fail(&e),
        };
    }
    check(&root)
}

fn check(root: &Path) -> ExitCode {
    match hg_encoding_build::cli::stale(root) {
        Ok(stale) if stale.is_empty() => {
            println!("hg-encoding-build: the generated tables are fresh");
            ExitCode::SUCCESS
        }
        Ok(stale) => {
            for p in &stale {
                eprintln!("hg-encoding-build: G29: {} is stale", p.display());
            }
            eprintln!("hg-encoding-build: run `cargo run -p hg-encoding-build -- generate`");
            ExitCode::from(1)
        }
        Err(e) => fail(&e),
    }
}

fn fail(e: &str) -> ExitCode {
    eprintln!("hg-encoding-build: {e}");
    ExitCode::from(2)
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
