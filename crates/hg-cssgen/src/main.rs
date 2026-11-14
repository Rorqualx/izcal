//! `hg-cssgen`: see the library docs.
//!
//! ```text
//! hg-cssgen [--root DIR] [generate | check | report]
//! ```
//!
//! `report` prints what the overlay covers and what it still leaves
//! open, which is ST-M0's measurement; `check` additionally requires the
//! checked-in table to be fresh.
//!
//! Exit status: 0 on success, 1 when the checked-in output is stale or
//! the overlay and webref disagree, 2 when something cannot be read.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: hg-cssgen [--root DIR] [generate | check | report]";

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
            "generate" | "check" | "report" => mode = a,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => return usage(),
        }
    }
    match mode.as_str() {
        "generate" => match hg_cssgen::cli::generate(&root) {
            Ok(written) => {
                for p in written {
                    println!("hg-cssgen: wrote {}", p.display());
                }
                ExitCode::SUCCESS
            }
            Err(e) => fail(&e),
        },
        "report" => report(&root),
        _ => check(&root),
    }
}

fn report(root: &Path) -> ExitCode {
    let (m, found) = match hg_cssgen::cli::verified(root, false) {
        Ok(v) => v,
        Err(e) => return fail(&e),
    };
    println!(
        "hg-cssgen: {} properties, {} legacy aliases, {} computed-value texts in {} kinds",
        m.properties.len(),
        m.aliases.len(),
        m.computed.len(),
        m.kinds.len()
    );
    let undecided = m.undecided();
    let decided = m.properties.len().saturating_sub(undecided);
    println!(
        "hg-cssgen: affects decided for {decided} of {}, {undecided} still open",
        m.properties.len()
    );
    for p in m.overrides() {
        println!(
            "hg-cssgen: {} overrides its kind: the specification's computed-value text is {:?}, \
             which is not this property's",
            p.name,
            p.computed.as_deref().unwrap_or("")
        );
    }
    let mappings = m.mappings();
    if !mappings.is_empty() {
        println!(
            "hg-cssgen: {} name(s) map to another property and are never stored: {}",
            mappings.len(),
            mappings
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let supplied = m.supplied_initials();
    if !supplied.is_empty() {
        println!(
            "hg-cssgen: {} initial value(s) webref did not extract, supplied by the overlay: {}",
            supplied.len(),
            supplied
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if found.is_empty() {
        println!("hg-cssgen: 0 disagreements with the pinned webref");
        ExitCode::SUCCESS
    } else {
        eprintln!("hg-cssgen: {}", hg_cssgen::cli::report(&found));
        ExitCode::from(1)
    }
}

fn check(root: &Path) -> ExitCode {
    match hg_cssgen::cli::stale(root) {
        Ok(stale) if stale.is_empty() => {
            println!("hg-cssgen: the generated files are fresh");
            ExitCode::SUCCESS
        }
        Ok(stale) => {
            for p in &stale {
                eprintln!("hg-cssgen: G29: {} is stale", p.display());
            }
            eprintln!("hg-cssgen: run `cargo run -p hg-cssgen -- generate`");
            ExitCode::from(1)
        }
        Err(e) => fail(&e),
    }
}

fn fail(e: &str) -> ExitCode {
    eprintln!("hg-cssgen: {e}");
    ExitCode::from(2)
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
