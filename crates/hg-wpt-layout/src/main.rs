//! `hg-wpt-layout`: see the library docs.
//!
//! ```text
//! hg-wpt-layout [--chapters]
//! ```
//!
//! Exit status: 0 when the suite was read and the runnable count is at
//! least the baseline, 1 when it is below, 2 when the suite cannot be
//! read.

use std::process::ExitCode;

use hg_wpt_layout::{Kind, LAYOUT_TESTS, PIN, RUNNABLE_BASELINE};

const USAGE: &str = "usage: hg-wpt-layout [--chapters]";

fn main() -> ExitCode {
    let mut chapters = false;
    for a in std::env::args().skip(1) {
        match a.as_str() {
            "--chapters" => chapters = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    let dir = match hg_wpt_layout::suite_dir() {
        Ok(d) => d,
        Err(e) => return fail(&e),
    };
    let entries = match hg_wpt_layout::read_suite(&dir) {
        Ok(e) => e,
        Err(e) => return fail(&e),
    };
    let l = hg_wpt_layout::measure(&entries);

    println!("hg-wpt-layout: css/CSS2 at {PIN}");
    println!("  files: {}", l.files);
    for kind in [
        Kind::Layout,
        Kind::Reftest,
        Kind::Harness,
        Kind::Manual,
        Kind::Support,
    ] {
        let n = l.by_kind.get(kind.name()).copied().unwrap_or(0);
        println!("  {:>8}: {n:>6}  needs {}", kind.name(), kind.needs());
    }
    println!("  data-expected-* attributes: {}", l.expectations);
    if chapters {
        for (chapter, n) in &l.by_chapter {
            println!("  {chapter}: {n}");
        }
    }
    println!(
        "hg-wpt-layout: runnable {} / {} tests, baseline {RUNNABLE_BASELINE}",
        l.runnable,
        l.files
            .saturating_sub(l.by_kind.get("support").copied().unwrap_or(0))
    );
    println!("  the first {LAYOUT_TESTS} become runnable with an HTML tree builder (ST-M1)");
    // A zero baseline guards nothing on its own, so what is checked
    // is that the suite still classifies as recorded: a change is
    // either the pin moving or the classifier drifting.
    let drift = hg_wpt_layout::check(&l);
    if drift.is_empty() {
        return ExitCode::SUCCESS;
    }
    for d in &drift {
        eprintln!(
            "hg-wpt-layout: {} is {} and was recorded as {}",
            d.what, d.measured, d.recorded
        );
    }
    ExitCode::from(1)
}

fn fail(e: &str) -> ExitCode {
    eprintln!("hg-wpt-layout: {e}");
    ExitCode::from(2)
}
