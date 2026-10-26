//! The runner against the real `dst-smoke` simulation binary.

use std::path::PathBuf;
use std::time::Duration;

use hg_dst::{Outcome, Target, group_failures};

fn smoke(args: &[&str]) -> Target {
    Target {
        bin: PathBuf::from(env!("CARGO_BIN_EXE_dst-smoke")),
        args: args.iter().map(|s| (*s).to_owned()).collect(),
        timeout: Duration::from_secs(60),
    }
}

#[test]
fn clean_seeds_pass_and_rerun_identically() {
    let t = smoke(&[]);
    let results = t.run_seeds(0..24, 4);
    assert_eq!(results.len(), 24);
    assert!(
        results
            .iter()
            .all(|r| matches!(r.outcome, Outcome::Pass(_))),
        "{results:?}"
    );
    assert!(t.rerun_mismatches(0..24, 4).is_empty());
}

#[test]
fn a_planted_bug_is_one_signature_with_a_working_replay() {
    let t = smoke(&["--bug"]);
    let results = t.run_seeds(0..40, 4);
    let groups = group_failures(&results);
    assert_eq!(groups.len(), 1, "{groups:?}");
    let (signature, seeds) = groups.iter().next().unwrap();
    assert!(
        signature.starts_with("panic at crates/hg-dst-sim/src/bin/dst-smoke.rs:"),
        "{signature}"
    );
    assert!(
        !seeds.is_empty() && seeds.len() < 40,
        "only some seeds reach the bug: {seeds:?}"
    );
    // The replay reproduces the same failure.
    let again = t.run_seed(seeds[0]);
    assert_eq!(Some(&again), results.iter().find(|r| r.seed == seeds[0]));
    assert!(
        t.replay_command(seeds[0])
            .ends_with(&format!("--seed {} --bug", seeds[0]))
    );
}

#[test]
fn ipc_seeds_pass_and_rerun_identically() {
    let t = smoke(&["--ipc"]);
    let results = t.run_seeds(0..24, 4);
    assert!(
        results
            .iter()
            .all(|r| matches!(r.outcome, Outcome::Pass(_))),
        "{results:?}"
    );
    assert!(t.rerun_mismatches(0..24, 4).is_empty());
}

#[test]
fn a_planted_ipc_bug_is_one_signature() {
    let t = smoke(&["--ipc", "--bug"]);
    let results = t.run_seeds(0..40, 4);
    let groups = group_failures(&results);
    assert_eq!(groups.len(), 1, "{groups:?}");
    let (signature, seeds) = groups.iter().next().unwrap();
    assert!(signature.contains("dst-smoke.rs"), "{signature}");
    assert!(
        !seeds.is_empty() && seeds.len() < 40,
        "only some seeds reach the bug: {seeds:?}"
    );
    let again = t.run_seed(seeds[0]);
    assert_eq!(Some(&again), results.iter().find(|r| r.seed == seeds[0]));
}

/// The phase E fault classes draw from streams of their own, and only
/// when on: with them off, `dst-smoke --ipc` over seeds 0..59 gives the
/// report lines it gave before they existed. The value is the SHA-256 of
/// the 60 report lines, one per line, as the phase D and E commits record
/// it (10996dfe…961b since df7fc67). A change that moves it must say why
/// in its commit.
#[test]
fn the_ipc_seeds_keep_their_traces() {
    let results = smoke(&["--ipc"]).run_seeds(0..60, 4);
    let mut text = String::new();
    for r in &results {
        let Outcome::Pass(p) = &r.outcome else {
            panic!("seed {} failed: {:?}", r.seed, r.outcome);
        };
        text.push_str(&format!(
            "hg-dst: seed={} steps={} time_ns={} crashes={} trace={:016x}\n",
            r.seed, p.steps, p.time_ns, p.crashes, p.trace
        ));
    }
    let hash = hg_build_id::sha256::hex(&hg_build_id::sha256::sha256(text.as_bytes()));
    assert_eq!(
        hash,
        "10996dfeec1ca7d40351ffec698e0bf225635ed56b82ac2672193408bac1961b"
    );
}
