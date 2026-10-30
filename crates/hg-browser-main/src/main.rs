//! `hg-browser-main`: the browser (broker) executable (Stage 1 S1-M1b;
//! launch design E-R1). It parses nothing: the broker's OS substrate
//! opens the build's executables and starts the launcher (on Linux, the
//! spawner); the browser library launches and routes. At M1 the browser
//! has no window, so it runs until it is ended.

use std::process::ExitCode;
use std::sync::Arc;

use hg_browser::BrowserConfig;
use hg_substrate_os::{Threads, os_broker_substrate};

/// Relaunches per kind (the simulator's budget, `dst-sim`).
const RESTART_BUDGET: u32 = 8;
/// The substrate could not start.
const EXIT_START: u8 = 66;
/// The first launch failed.
const EXIT_LAUNCH: u8 = 71;

fn main() -> ExitCode {
    let Ok(s) = os_broker_substrate(Threads::for_this_machine()) else {
        return ExitCode::from(EXIT_START);
    };
    let config = BrowserConfig {
        restart_budget: RESTART_BUDGET,
        progress: Arc::default(),
    };
    let Ok(_running) = hg_browser::run(&s, config) else {
        return ExitCode::from(EXIT_LAUNCH);
    };
    loop {
        std::thread::park();
    }
}
