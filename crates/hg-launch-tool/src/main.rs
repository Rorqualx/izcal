//! `hg-launch-tool`: the OS launch path end to end (M1 phase E).
//!
//! - `round-trip`: the M1 exit test on this OS. A broker on the OS
//!   substrate launches `hg-net-main` and `hg-renderer-main` from this
//!   tool's own directory (the build) in their sandboxes; the renderer
//!   makes `hg_renderer::LAUNCHED_ROUNDS` round trips through the browser
//!   and the network service. Prints one result line; exit 0 on success.
//! - `measure-spawn N`: FP-27's runs (E-R12). N fresh brokers, each
//!   launching both kinds; prints a TSV (`run`, `kind`,
//!   `spawn_to_ready_ms`) of the time from the launch request to the
//!   child's accepted `ready`, on the substrate clock. Cold runs are the
//!   caller's job (drop the page cache before each, E-Q6).

use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use hg_browser::{BrowserConfig, Progress};
use hg_substrate_os::{Threads, os_broker_substrate};

/// How long a run may take.
const DEADLINE: Duration = Duration::from_secs(20);

fn wait_for(what: impl Fn() -> bool) -> bool {
    let end = Instant::now() + DEADLINE;
    while Instant::now() < end {
        if what() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

fn round_trip() -> ExitCode {
    let s = match os_broker_substrate(Threads::for_this_machine()) {
        Ok(s) => s,
        Err(e) => {
            println!("round-trip: FAIL start: {e}");
            return ExitCode::FAILURE;
        }
    };
    let p = Arc::new(Progress::default());
    let config = BrowserConfig {
        restart_budget: 0,
        progress: Arc::clone(&p),
    };
    let running = match hg_browser::run(&s, config) {
        Ok(r) => r,
        Err(e) => {
            println!("round-trip: FAIL launch: {e:?}");
            return ExitCode::FAILURE;
        }
    };
    let rounds = hg_renderer::LAUNCHED_ROUNDS;
    let done = wait_for(|| p.returned.load(Ordering::SeqCst) >= rounds);
    let times = p.ready_times.lock().map(|t| t.clone()).unwrap_or_default();
    let line = format!(
        "launches={} routed={} returned={} sandbox_mismatches={} ready={:?}",
        p.launches.load(Ordering::SeqCst),
        p.routed.load(Ordering::SeqCst),
        p.returned.load(Ordering::SeqCst),
        p.sandbox_mismatches.load(Ordering::SeqCst),
        times
    );
    drop(running);
    if done && p.sandbox_mismatches.load(Ordering::SeqCst) == 0 {
        println!("round-trip: ok {line}");
        ExitCode::SUCCESS
    } else {
        println!("round-trip: FAIL {line}");
        ExitCode::FAILURE
    }
}

fn measure_spawn(n: u32) -> ExitCode {
    println!("run\tkind\tspawn_to_ready_ms");
    for run in 1..=n {
        let Ok(s) = os_broker_substrate(Threads::for_this_machine()) else {
            eprintln!("measure-spawn: run {run}: the substrate did not start");
            return ExitCode::FAILURE;
        };
        let p = Arc::new(Progress::default());
        let config = BrowserConfig {
            restart_budget: 0,
            progress: Arc::clone(&p),
        };
        let Ok(running) = hg_browser::run(&s, config) else {
            eprintln!("measure-spawn: run {run}: launch failed");
            return ExitCode::FAILURE;
        };
        let both = wait_for(|| p.ready_times.lock().map(|t| t.len() >= 2).unwrap_or(false));
        let times = p.ready_times.lock().map(|t| t.clone()).unwrap_or_default();
        drop(running);
        if !both {
            eprintln!("measure-spawn: run {run}: no ready from both kinds");
            return ExitCode::FAILURE;
        }
        for (kind, d) in times {
            #[expect(clippy::cast_precision_loss, reason = "milliseconds for a TSV")]
            let ms = d.as_nanos() as f64 / 1e6;
            println!("{run}\t{kind:?}\t{ms:.3}");
        }
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["round-trip"] => round_trip(),
        ["measure-spawn", n] => match n.parse() {
            Ok(n) => measure_spawn(n),
            Err(_) => usage(),
        },
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: hg-launch-tool round-trip | measure-spawn N");
    ExitCode::from(2)
}
