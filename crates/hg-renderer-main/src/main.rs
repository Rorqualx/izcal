//! `hg-renderer-main`: the renderer executable (Stage 1 S1-M1b; launch
//! design E-R1). It parses nothing: the OS substrate's child start reads
//! the kind its broker named, locks the process down and probes it; then
//! the renderer library runs until its bootstrap channel ends.

// Windows: the GUI subsystem. A console-subsystem child started with
// CREATE_NO_WINDOW still gets a windowless conhost.exe of its own (FP-M0
// decision log, 23 October 2026): measured again in M1 phase G on
// rig253, one conhost per child, 944 KiB private and 6.2 MB working set
// each, and about 18 ms of FP-27's spawn-to-ready. The child writes
// nothing to a console (its stdio is null), so it needs none.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::process::ExitCode;

use hg_renderer::{LAUNCHED_ROUNDS, RendererConfig};
use hg_substrate_os::Threads;
use hg_substrate_os::launch::{Exe, os_child_substrate};

/// The library could not serve its bootstrap channel.
const EXIT_RUN: u8 = 70;

fn main() -> ExitCode {
    // FP 6.3 and FP-32: `main` runs the sequences and the IPC transport
    // its IO thread, two threads at rest; a MayBlock worker and the
    // timer thread start only when used.
    let threads = Threads::CHILD;
    let child = match os_child_substrate(threads, Exe::Renderer) {
        Ok(c) => c,
        Err(e) => return ExitCode::from(e.exit_code()),
    };
    let config = RendererConfig::launched(LAUNCHED_ROUNDS, &child.sandbox);
    let Some(running) = hg_renderer::run(&child.substrate, config) else {
        return ExitCode::from(EXIT_RUN);
    };
    child.main.run_until(|| running.ended());
    ExitCode::SUCCESS
}
