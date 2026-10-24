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
        signature.starts_with("panic at crates/hg-dst/src/bin/dst-smoke.rs:"),
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
