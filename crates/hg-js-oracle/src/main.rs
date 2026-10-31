//! `hg-js-oracle`: Izcal's JS oracles, test-only and never shipped.
//!
//! `hg-js-oracle ops` is the operation-level oracle of JS-M0 (plan
//! Phase 1, ORC-1). Every core operation of the generated reference
//! interpreter runs on every operand (or pair) from a fixed set of edge
//! values, and its result, or the error it throws, is compared with the
//! same expression evaluated by engine262. engine262 runs black-box under
//! Node through its documented command line; it is never linked or
//! vendored, and its source is never read (HGBA-2 M-1 part 3). This
//! catches description bugs, which a differential test between Izcal's
//! own tiers cannot (JS spec 6.3, the common-mode risk).
//!
//! Each case also runs under the moving test collector, which must give
//! the same answer.
//!
//! ```text
//! hg-js-oracle ops --engine262 DIR [--node PATH] [--jobs N] [--only OP]
//! ```
//!
//! `DIR` is the unpacked engine262 npm package pinned in
//! `tools/test-data.toml`. Exit status: 0 when every case agrees, 1 when
//! one does not, 2 for a usage or setup error.

mod cases;
mod izcal;
mod js;

use std::path::PathBuf;
use std::process::ExitCode;

use cases::Case;

/// The most mismatches printed in full.
const SHOWN: usize = 60;

struct Args {
    engine262: PathBuf,
    node: PathBuf,
    jobs: usize,
    only: Option<String>,
}

fn usage() -> ExitCode {
    eprintln!("usage: hg-js-oracle ops --engine262 DIR [--node PATH] [--jobs N] [--only OP]");
    ExitCode::from(2)
}

fn parse() -> Option<Args> {
    let mut it = std::env::args().skip(1);
    if it.next().as_deref() != Some("ops") {
        return None;
    }
    let mut engine262 = None;
    let mut node = PathBuf::from("node");
    let mut jobs = std::thread::available_parallelism().map_or(4, usize::from);
    let mut only = None;
    while let Some(a) = it.next() {
        match a.as_str() {
            "--engine262" => engine262 = it.next().map(PathBuf::from),
            "--node" => node = PathBuf::from(it.next()?),
            "--jobs" => jobs = it.next()?.parse().ok()?,
            "--only" => only = it.next(),
            _ => return None,
        }
    }
    Some(Args {
        engine262: engine262?,
        node,
        jobs: jobs.max(1),
        only,
    })
}

fn main() -> ExitCode {
    let Some(args) = parse() else {
        return usage();
    };
    let mut all = cases::all();
    if let Some(op) = &args.only {
        all.retain(|c| c.op.mnemonic == op.as_str());
        if all.is_empty() {
            eprintln!("hg-js-oracle: no operation `{op}` in the core set");
            return ExitCode::from(2);
        }
    }
    let ours = match izcal::run(&all) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("hg-js-oracle: the reference interpreter: {e}");
            return ExitCode::from(2);
        }
    };
    let theirs = match js::run(&all, &args.engine262, &args.node, args.jobs) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("hg-js-oracle: engine262: {e}");
            return ExitCode::from(2);
        }
    };
    // A case engine262 crashed on is decided by a second engine, Node's
    // own, run black-box the same way.
    let crashed: Vec<usize> = (0..all.len())
        .filter(|&i| theirs[i].starts_with(js::CRASH))
        .collect();
    let picked: Vec<Case> = crashed.iter().map(|&i| all[i].clone()).collect();
    let second = match js::run_node(&picked, &args.node) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("hg-js-oracle: node: {e}");
            return ExitCode::from(2);
        }
    };
    let mut expected = theirs.clone();
    for (&i, n) in crashed.iter().zip(&second) {
        expected[i].clone_from(n);
    }
    for (&i, n) in crashed.iter().zip(&second) {
        println!(
            "ENGINE262-CRASHED {}: izcal={} node={} engine262: {}",
            all[i].describe(),
            ours[i].plain,
            n,
            &theirs[i][js::CRASH.len_utf8()..]
        );
    }
    report(&all, &ours, &expected, crashed.len())
}

/// Whether two different Number answers to an `Exp` case are adjacent
/// doubles of one sign. ECMA-262 leaves Number::exponentiate's general
/// case implementation-approximated (step 13); Izcal's interim version
/// promises at most 1 ulp (hg-js-number, `pow`), and correct rounding
/// belongs to hg-math (JS spec 6.9). Every other operation must agree
/// exactly.
fn within_one_ulp(c: &Case, ours: &str, theirs: &str) -> bool {
    if c.op.mnemonic != "Exp" || ours == theirs {
        return false;
    }
    let bits = |s: &str| {
        s.strip_prefix('n')
            .and_then(|h| u64::from_str_radix(h, 16).ok())
    };
    match (bits(ours), bits(theirs)) {
        // Same sign bit, and the magnitudes' bit patterns one apart.
        (Some(a), Some(b)) => (a >> 63) == (b >> 63) && a.abs_diff(b) == 1,
        _ => false,
    }
}

/// Prints the per-operation counts and the mismatches.
fn report(all: &[Case], ours: &[izcal::Answer], theirs: &[String], crashed: usize) -> ExitCode {
    let mut per_op: Vec<(&str, usize, usize)> = Vec::new();
    let mut bad = Vec::new();
    let mut approximate = Vec::new();
    for (i, c) in all.iter().enumerate() {
        let a = &ours[i];
        let close = a.moving == a.plain && within_one_ulp(c, &a.plain, &theirs[i]);
        if close {
            approximate.push(i);
        }
        let ok = close || (a.plain == theirs[i] && a.moving == a.plain);
        match per_op.last_mut() {
            Some((op, n, f)) if *op == c.op.mnemonic => {
                *n += 1;
                *f += usize::from(!ok);
            }
            _ => per_op.push((c.op.mnemonic, 1, usize::from(!ok))),
        }
        if !ok {
            bad.push(i);
        }
    }
    for (op, n, f) in &per_op {
        println!("{op:<14} {n:>6} cases  {f:>5} mismatched");
    }
    for &i in bad.iter().take(SHOWN) {
        let c = &all[i];
        let a = &ours[i];
        print!(
            "MISMATCH {}: izcal={} engine262={}",
            c.describe(),
            a.plain,
            theirs[i]
        );
        if a.moving != a.plain {
            print!(" izcal-moving={}", a.moving);
        }
        println!();
    }
    for &i in &approximate {
        println!(
            "WITHIN-1-ULP {}: izcal={} engine262={}",
            all[i].describe(),
            ours[i].plain,
            theirs[i]
        );
    }
    if bad.len() > SHOWN {
        println!("... and {} more", bad.len() - SHOWN);
    }
    println!(
        "hg-js-oracle ops: {} cases, {} operations, {} mismatched, {} within 1 ulp, {} decided by Node (engine262 crashed)",
        all.len(),
        per_op.len(),
        bad.len(),
        approximate.len(),
        crashed
    );
    if bad.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
