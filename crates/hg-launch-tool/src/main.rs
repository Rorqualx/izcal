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
//! - `measure-phases N` (Windows, `--features measure-launch` only): as
//!   `measure-spawn`, then one row per launch phase (`run`, `kind`,
//!   `pid`, `side`, `phase`, `ms` since the launch request), the broker's
//!   stamps and the child's, for M1 phase G's FP-27 breakdown. A
//!   measurement build; production builds have no stamps.
//! - `hold SECS`: the renderer skeleton at rest, for M1 phase G's
//!   samplers (FP-M0's method: the OS counters are read from outside, by
//!   process). Runs `round-trip`, prints `hold: ready broker=<pid>` and
//!   `hold: children <Kind>=<pid> ...`, then keeps the browser and its
//!   children alive and idle for SECS seconds with no wakeup of its own
//!   (one sleep), and exits.

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
    hold(None)
}

/// `round-trip`, then, with `secs`, the hold.
fn hold(secs: Option<u64>) -> ExitCode {
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
    let times = p
        .spawn_to_ready
        .lock()
        .map(|t| t.clone())
        .unwrap_or_default();
    let tiers = p
        .sandbox_tiers
        .lock()
        .map(|t| t.clone())
        .unwrap_or_default();
    let line = format!(
        "launches={} routed={} returned={} sandbox_mismatches={} ready={:?} tiers={:?}",
        p.launches.load(Ordering::SeqCst),
        p.routed.load(Ordering::SeqCst),
        p.returned.load(Ordering::SeqCst),
        p.sandbox_mismatches.load(Ordering::SeqCst),
        times,
        tiers
    );
    let ok = done && p.sandbox_mismatches.load(Ordering::SeqCst) == 0;
    if let (true, Some(secs)) = (ok, secs) {
        println!("round-trip: ok {line}");
        println!("hold: ready broker={}", std::process::id());
        let pids = p.ready_pids.lock().map(|t| t.clone()).unwrap_or_default();
        let pids: Vec<String> = pids.iter().map(|(k, pid)| format!("{k:?}={pid}")).collect();
        println!("hold: children {}", pids.join(" "));
        let _ = std::io::Write::flush(&mut std::io::stdout());
        std::thread::sleep(Duration::from_secs(secs));
        drop(running);
        println!("hold: done");
        return ExitCode::SUCCESS;
    }
    drop(running);
    if ok {
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
        let both = wait_for(|| {
            p.spawn_to_ready
                .lock()
                .map(|t| t.len() >= 2)
                .unwrap_or(false)
        });
        let times = p
            .spawn_to_ready
            .lock()
            .map(|t| t.clone())
            .unwrap_or_default();
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

#[cfg(all(windows, feature = "measure-launch"))]
fn measure_phases(n: u32) -> ExitCode {
    use hg_substrate_os::launch::windows::timing;
    println!("run\tkind\tpid\tside\tphase\tms");
    for run in 1..=n {
        timing::clear();
        let Ok(s) = os_broker_substrate(Threads::for_this_machine()) else {
            eprintln!("measure-phases: run {run}: the substrate did not start");
            return ExitCode::FAILURE;
        };
        let p = Arc::new(Progress::default());
        let config = BrowserConfig {
            restart_budget: 0,
            progress: Arc::clone(&p),
        };
        let Ok(running) = hg_browser::run(&s, config) else {
            eprintln!("measure-phases: run {run}: launch failed");
            return ExitCode::FAILURE;
        };
        let both = wait_for(|| {
            p.spawn_to_ready
                .lock()
                .map(|t| t.len() >= 2)
                .unwrap_or(false)
        });
        // The children publish their stamps before `ready`; read them
        // while they are alive.
        let rows = timing::report();
        let times = p
            .spawn_to_ready
            .lock()
            .map(|t| t.clone())
            .unwrap_or_default();
        drop(running);
        if !both {
            eprintln!("measure-phases: run {run}: no ready from both kinds");
            return ExitCode::FAILURE;
        }
        for r in rows {
            println!("{run}\t{r}");
        }
        for (kind, d) in times {
            #[expect(clippy::cast_precision_loss, reason = "milliseconds for a TSV")]
            let ms = d.as_nanos() as f64 / 1e6;
            println!("{run}\t{kind:?}\t-\tbroker\tready\t{ms:.3}");
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
        ["hold", secs] => match secs.parse() {
            Ok(secs) => hold(Some(secs)),
            Err(_) => usage(),
        },
        ["measure-spawn", n] => match n.parse() {
            Ok(n) => measure_spawn(n),
            Err(_) => usage(),
        },
        #[cfg(all(windows, feature = "measure-launch"))]
        ["measure-phases", n] => match n.parse() {
            Ok(n) => measure_phases(n),
            Err(_) => usage(),
        },
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: hg-launch-tool round-trip | measure-spawn N | hold SECS");
    ExitCode::from(2)
}
