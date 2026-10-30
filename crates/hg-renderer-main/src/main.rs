//! `hg-renderer-main`: the renderer executable (Stage 1 S1-M1b; launch
//! design E-R1). It parses nothing: the OS substrate's child start reads
//! the kind its broker named, locks the process down and probes it; then
//! the renderer library runs until its bootstrap channel ends.

use std::process::ExitCode;

use hg_renderer::{LAUNCHED_ROUNDS, RendererConfig};
use hg_substrate_os::Threads;
use hg_substrate_os::launch::{Exe, os_child_substrate};

/// The library could not serve its bootstrap channel.
const EXIT_RUN: u8 = 70;

fn main() -> ExitCode {
    let threads = Threads {
        general: 2,
        may_block: 1,
    };
    let child = match os_child_substrate(threads, Exe::Renderer) {
        Ok(c) => c,
        Err(e) => return ExitCode::from(e.exit_code()),
    };
    let config = RendererConfig::launched(LAUNCHED_ROUNDS, &child.sandbox);
    let Some(running) = hg_renderer::run(&child.substrate, config) else {
        return ExitCode::from(EXIT_RUN);
    };
    running.wait();
    ExitCode::SUCCESS
}
