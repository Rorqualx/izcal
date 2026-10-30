//! The M1 process skeletons under the simulator (`dst-sim`): the browser,
//! renderer and network libraries, launched by the simulated launcher
//! (Stage 1 design 8.1 M1; distribution DU-3). Each scenario's own
//! invariants are asserted inside `dst-sim`; a broken one panics, which
//! the runner reports as a failure.

use std::path::PathBuf;
use std::time::Duration;

use hg_dst::{Outcome, Target};

fn sim(args: &[&str]) -> Target {
    Target {
        bin: PathBuf::from(env!("CARGO_BIN_EXE_dst-sim")),
        args: args.iter().map(|s| (*s).to_owned()).collect(),
        timeout: Duration::from_secs(60),
    }
}

fn all_pass_and_rerun(t: &Target) {
    let results = t.run_seeds(0..30, 4);
    assert!(
        results
            .iter()
            .all(|r| matches!(r.outcome, Outcome::Pass(_))),
        "{results:?}"
    );
    assert!(t.rerun_mismatches(0..30, 4).is_empty());
}

/// Every round trip completes through the browser with no faults.
#[test]
fn the_round_trip_completes() {
    all_pass_and_rerun(&sim(&["--scenario", "roundtrip"]));
}

/// Under crash faults the browser relaunches within its budget and the
/// run still ends, the same way twice.
#[test]
fn crashed_children_are_relaunched() {
    all_pass_and_rerun(&sim(&["--crash-ppm", "20000"]));
}

/// DU-3: a child binary replaced between two launches puts the browser in
/// restart-required, with no launch after the mismatch.
#[test]
fn a_replaced_binary_stops_all_launches() {
    all_pass_and_rerun(&sim(&["--scenario", "du3"]));
}

/// An untrusted renderer cannot grow the broker's queue of asks: the one
/// past the cap ends its channel (review of hg-browser).
#[test]
fn a_flooding_renderer_is_cut_off_at_the_cap() {
    all_pass_and_rerun(&sim(&["--scenario", "flood"]));
}

/// The network ends mid-conversation: its unanswered work goes to the
/// relaunched network and the renderer still gets every answer.
#[test]
fn work_survives_a_network_relaunch() {
    all_pass_and_rerun(&sim(&["--scenario", "netcrash"]));
}

/// Review M3: a child whose sandbox report names another tier than its
/// launcher applied is killed and not relaunched.
#[test]
fn a_child_reporting_another_sandbox_tier_is_killed() {
    all_pass_and_rerun(&sim(&["--scenario", "liar"]));
}
