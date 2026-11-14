//! `hg-crypto-check`: the Wycheproof runner and the timing test.
//!
//! ```text
//! hg-crypto-check wycheproof   # run every vector, report per algorithm
//! hg-crypto-check report       # the classification, without running
//! hg-crypto-check timing       # the constant-time measurement
//! ```

use std::process::ExitCode;

use hg_crypto_check::run::Summary;
use hg_crypto_check::{run, timing, vectors};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let what = args.first().map_or("wycheproof", String::as_str);
    match what {
        "wycheproof" => wycheproof(true),
        "report" => wycheproof(false),
        "timing" => timing_check(),
        other => {
            eprintln!("hg-crypto-check: unknown command {other}");
            eprintln!("usage: hg-crypto-check [wycheproof | report | timing]");
            ExitCode::from(2)
        }
    }
}

fn wycheproof(run_them: bool) -> ExitCode {
    let dir = match vectors::vectors_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("hg-crypto-check: {e}");
            return ExitCode::from(2);
        }
    };
    let files = match vectors::all(&dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("hg-crypto-check: {e}");
            eprintln!("hg-crypto-check: run `tools/fetch-test-data.sh wycheproof` first");
            return ExitCode::from(2);
        }
    };

    let mut summary = Summary::default();
    let mut unreadable = Vec::new();
    for path in &files {
        match vectors::read(path) {
            Ok(suite) => {
                if run_them {
                    run::run_suite(&suite, &mut summary);
                } else {
                    let out = summary
                        .by_algorithm
                        .entry(suite.algorithm.clone())
                        .or_default();
                    out.unimplemented = out.unimplemented.saturating_add(suite.case_count());
                }
            }
            Err(e) => unreadable.push(e),
        }
    }

    println!(
        "hg-crypto-check: {} files, {} algorithms, {} cases, at {}",
        files.len(),
        summary.by_algorithm.len(),
        summary.total(),
        vectors::PIN
    );
    for (algorithm, out) in &summary.by_algorithm {
        let ran = out.pass.saturating_add(out.fail);
        if ran > 0 {
            println!(
                "  {algorithm}: {}/{ran} pass, {} acceptable ({})",
                out.pass,
                out.acceptable,
                run::hash_name()
            );
            for f in &out.failures {
                println!("    {f}");
            }
        } else {
            println!("  {algorithm}: {} cases, no primitive yet", out.total());
        }
    }
    for m in &summary.miscounted {
        println!("  miscount: {m}");
    }
    for e in &unreadable {
        println!("  unreadable: {e}");
    }

    if !unreadable.is_empty() || !summary.miscounted.is_empty() || summary.failed() > 0 {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn timing_check() -> ExitCode {
    const LEN: usize = 4096;
    const ROUNDS: usize = 400;

    // A majority of three, so one noisy run on a shared agent does not
    // decide a merge gate (see `timing::majority`).
    let (planted, planted_leaks) = timing::majority(timing::leaky_eq, LEN, ROUNDS, 3);
    let (constant, constant_leaks) = timing::majority(hg_crypto::ct::eq, LEN, ROUNDS, 3);

    println!("hg-crypto-check: {LEN} bytes, {ROUNDS} rounds");
    for (what, m) in [("leaky_eq (the plant)", planted), ("ct::eq", constant)] {
        println!(
            "  {what}: early min {} ns, late min {} ns, noise {} ns, ratio {:.1}",
            m.early.min,
            m.late.min,
            m.noise,
            m.ratio()
        );
    }

    if !planted_leaks {
        eprintln!(
            "hg-crypto-check: the planted leak was NOT detected (ratio {:.1}); \
             this machine is too noisy for the measurement to mean anything",
            planted.ratio()
        );
        return ExitCode::FAILURE;
    }
    println!("  the plant is detected in a majority of runs, so the measurement discriminates");
    if constant_leaks {
        eprintln!(
            "hg-crypto-check: ct::eq looks data-dependent (ratio {:.4}, \
             in a majority of runs)",
            constant.ratio()
        );
        return ExitCode::FAILURE;
    }
    println!(
        "  ct::eq is {:.4}, under the 0.25 a quarter of the work would be",
        constant.ratio()
    );
    ExitCode::SUCCESS
}
