//! FP-M0 / M1 phase B: the OS substrate at rest. Starts the real
//! substrate (all worker and timer threads), runs one task and one
//! delayed task so every thread has been used, prints its pid, then
//! idles. The FP-M0 wakeup counters (`measurements/milestones/fp-m0/
//! harness/wakeups-*`) read it; footprint 6.8 expects zero wakeups.

use std::sync::mpsc::sync_channel;

use hg_substrate::{Priority, Task};
use hg_substrate_os::{Threads, os_substrate};
use hg_types::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let threads = Threads::for_this_machine();
    let s = os_substrate(threads)?;
    let seq = s.exec.sequence("idle-check", Priority::UserVisible);
    let (tx, rx) = sync_channel(2);
    let now_tx = tx.clone();
    s.exec.post(
        seq,
        Task::new(move || {
            let _ = now_tx.send("task");
        }),
        None,
        Duration::ZERO,
    )?;
    s.exec.post(
        seq,
        Task::new(move || {
            let _ = tx.send("timer");
        }),
        Some(Duration::from_millis(50)),
        Duration::from_millis(10),
    )?;
    let first = rx.recv()?;
    let second = rx.recv()?;
    let threads_started = threads
        .general
        .saturating_add(threads.may_block)
        .saturating_add(1);
    println!(
        "pid={} ran={first},{second} substrate_threads={threads_started} idle",
        std::process::id()
    );
    // The substrate is idle now; the main thread waits with no timer.
    loop {
        std::thread::park();
    }
}
