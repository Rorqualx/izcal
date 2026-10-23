//! `hg-dst`: runs a simulation binary over a range of seeds.
//!
//! ```text
//! hg-dst run   <sim-binary> [--seeds A..B] [--jobs N] [--timeout-secs S] [-- ARGS...]
//! hg-dst rerun <sim-binary> [same options]     # CS-T5: same seed, same trace
//! ```
//!
//! Exit status: 0 when every seed passed (and, for `rerun`, matched), 1
//! when any did not, 2 on a usage error.

use std::ops::Range;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use hg_dst::{Outcome, Target, group_failures};

const USAGE: &str = "usage: hg-dst run|rerun <sim-binary> [--seeds A..B] [--jobs N] [--timeout-secs S] [-- ARGS...]";

struct Options {
    rerun: bool,
    target: Target,
    seeds: Range<u64>,
    jobs: usize,
}

fn parse() -> Result<Options, String> {
    let mut args = std::env::args().skip(1);
    let rerun = match args.next().as_deref() {
        Some("run") => false,
        Some("rerun") => true,
        _ => return Err(USAGE.to_owned()),
    };
    let bin = PathBuf::from(args.next().ok_or(USAGE)?);
    let mut seeds = 0..100;
    let mut jobs = std::thread::available_parallelism().map_or(4, |n| n.get());
    let mut timeout = Duration::from_secs(60);
    let mut extra = Vec::new();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--seeds" => {
                let v = args.next().ok_or(USAGE)?;
                let (a, b) = v.split_once("..").ok_or("--seeds takes A..B")?;
                seeds = a.parse().map_err(|_e| "bad seed")?..b.parse().map_err(|_e| "bad seed")?;
            }
            "--jobs" => {
                jobs = args
                    .next()
                    .ok_or(USAGE)?
                    .parse()
                    .map_err(|_e| "bad --jobs")?
            }
            "--timeout-secs" => {
                timeout = Duration::from_secs(
                    args.next()
                        .ok_or(USAGE)?
                        .parse()
                        .map_err(|_e| "bad --timeout-secs")?,
                );
            }
            "--" => extra.extend(args.by_ref()),
            other => return Err(format!("unknown option {other}\n{USAGE}")),
        }
    }
    Ok(Options {
        rerun,
        target: Target {
            bin,
            args: extra,
            timeout,
        },
        seeds,
        jobs,
    })
}

fn main() -> ExitCode {
    let opts = match parse() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let n = opts.seeds.end.saturating_sub(opts.seeds.start);
    if opts.rerun {
        let bad = opts.target.rerun_mismatches(opts.seeds, opts.jobs);
        for (a, b) in &bad {
            println!(
                "MISMATCH seed={} first={:?} second={:?}\n  replay: {}",
                a.seed,
                a.outcome,
                b.outcome,
                opts.target.replay_command(a.seed)
            );
        }
        println!("rerun: {n} seeds, {} mismatched", bad.len());
        return if bad.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    let results = opts.target.run_seeds(opts.seeds, opts.jobs);
    let groups = group_failures(&results);
    for (signature, seeds) in &groups {
        let first = seeds.first().copied().unwrap_or_default();
        let detail = results
            .iter()
            .find(|r| r.seed == first)
            .and_then(|r| match &r.outcome {
                Outcome::Fail { detail, .. } => Some(detail.as_str()),
                Outcome::Pass(_) => None,
            })
            .unwrap_or_default();
        println!(
            "FAIL {signature}: {} seed(s), first {first}\n  replay: {}\n  {}",
            seeds.len(),
            opts.target.replay_command(first),
            detail.replace('\n', "\n  ")
        );
    }
    let failed: usize = groups.values().map(Vec::len).sum();
    println!(
        "run: {n} seeds, {} passed, {failed} failed, {} distinct failure(s)",
        results.len().saturating_sub(failed),
        groups.len()
    );
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
