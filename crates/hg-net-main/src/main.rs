//! `hg-net-main`: the network service executable, for the three NW-D3
//! kinds (Stage 1 S1-M1b; launch design E-R1). It parses nothing: the OS
//! substrate's child start reads the kind its broker named (E-R7
//! disposition), locks the process down and probes it; then the network
//! library runs until its bootstrap channel ends.

use std::process::ExitCode;

use hg_net_service::NetConfig;
use hg_substrate_os::Threads;
use hg_substrate_os::launch::{Exe, os_child_substrate};

/// The library could not serve its bootstrap channel.
const EXIT_RUN: u8 = 70;

fn main() -> ExitCode {
    let threads = Threads {
        general: 2,
        may_block: 1,
    };
    let child = match os_child_substrate(threads, Exe::Net) {
        Ok(c) => c,
        Err(e) => return ExitCode::from(e.exit_code()),
    };
    let Some(config) = NetConfig::launched(child.kind, &child.sandbox) else {
        return ExitCode::from(EXIT_RUN);
    };
    let Some(running) = hg_net_service::run(&child.substrate, config) else {
        return ExitCode::from(EXIT_RUN);
    };
    running.wait();
    ExitCode::SUCCESS
}
