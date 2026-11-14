//! `hg-wpt-url`: see the library docs.
//!
//! ```text
//! hg-wpt-url [--suite urltestdata|toascii|all]
//! ```
//!
//! Exit status: 0 when every suite it ran met its baseline, 1 when one
//! did not, 2 when the data could not be read.

use std::process::ExitCode;

use hg_wpt_url::{PERCENT_ENCODING_BASELINE, TOASCII_BASELINE, URLTESTDATA_BASELINE};

const USAGE: &str = "usage: hg-wpt-url [--suite urltestdata|toascii|percent-encoding|all]";

fn main() -> ExitCode {
    let mut suite = "all".to_owned();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--suite" => match args.next().as_deref() {
                Some(v @ ("urltestdata" | "toascii" | "percent-encoding" | "all")) => {
                    suite = v.to_owned();
                }
                _ => return usage(),
            },
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => return usage(),
        }
    }
    let mut below = false;
    if suite == "urltestdata" || suite == "all" {
        match run_urltestdata() {
            Ok(ok) => below |= !ok,
            Err(e) => return fail(&e),
        }
    }
    if suite == "toascii" || suite == "all" {
        match run_toascii() {
            Ok(ok) => below |= !ok,
            Err(e) => return fail(&e),
        }
    }
    if suite == "percent-encoding" || suite == "all" {
        match run_percent_encoding() {
            Ok(ok) => below |= !ok,
            Err(e) => return fail(&e),
        }
    }
    if below {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_urltestdata() -> Result<bool, String> {
    let path = hg_wpt_url::data_file("wpt-urltestdata")?;
    let json = hg_wpt_url::read_json(&path)?;
    let cases = hg_wpt_url::cases(&json);
    let l = hg_wpt_url::run(&cases);
    for (kind, n) in &l.kinds {
        println!("  {kind}: {n}");
    }
    for e in &l.examples {
        println!("  {e}");
    }
    println!(
        "hg-wpt-url: urltestdata {}/{}, baseline {URLTESTDATA_BASELINE}, at {}",
        l.pass,
        l.runs,
        hg_wpt_url::PIN
    );
    if l.pass < URLTESTDATA_BASELINE {
        eprintln!(
            "hg-wpt-url: {} below the baseline of {URLTESTDATA_BASELINE}",
            l.pass
        );
        return Ok(false);
    }
    Ok(true)
}

fn run_toascii() -> Result<bool, String> {
    let path = hg_wpt_url::data_file("wpt-toascii")?;
    let json = hg_wpt_url::read_json(&path)?;
    let l = hg_wpt_url::run_toascii(&json);
    for e in &l.examples {
        println!("  {e}");
    }
    println!(
        "hg-wpt-url: toascii {}/{}, baseline {TOASCII_BASELINE}",
        l.pass, l.runs
    );
    if l.pass < TOASCII_BASELINE {
        eprintln!(
            "hg-wpt-url: {} below the baseline of {TOASCII_BASELINE}",
            l.pass
        );
        return Ok(false);
    }
    Ok(true)
}

fn run_percent_encoding() -> Result<bool, String> {
    let path = hg_wpt_url::data_file("wpt-percent-encoding")?;
    let json = hg_wpt_url::read_json(&path)?;
    let l = hg_wpt_url::run_percent_encoding(&json);
    for e in &l.examples {
        println!("  {e}");
    }
    println!(
        "hg-wpt-url: percent-encoding {}/{}, baseline {PERCENT_ENCODING_BASELINE}",
        l.pass, l.runs
    );
    if l.pass < PERCENT_ENCODING_BASELINE {
        eprintln!(
            "hg-wpt-url: {} below the baseline of {PERCENT_ENCODING_BASELINE}",
            l.pass
        );
        return Ok(false);
    }
    Ok(true)
}

fn fail(e: &str) -> ExitCode {
    eprintln!("hg-wpt-url: {e}");
    ExitCode::from(2)
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
