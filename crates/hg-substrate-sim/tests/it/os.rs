//! Medium tests: the scripted OS model's delivery (OS integration 6.1).
//! Inputs not yet delivered survive the listener going away, and a
//! launch storm larger than a sequence's capacity loses nothing.

use std::sync::{Arc, Mutex};

use hg_substrate::os::{OsDelivery, OsListening};
use hg_substrate::{Memory, Priority, Task};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{Limits, OsScript, RunEnd, SEQUENCE_CAPACITY, SimConfig, World};
use hg_types::{Duration, Instant};

type Log = Arc<Mutex<Vec<String>>>;

fn backing() -> Arc<dyn Memory> {
    Arc::new(OsMemory::new())
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn until(ms: u64) -> Limits {
    Limits {
        max_steps: 1_000_000,
        max_time: Instant::from_nanos(ms.saturating_mul(1_000_000)),
    }
}

/// A script of one-byte-name frames at the given delays, answering
/// nothing.
fn script(inputs: &[(u64, &str)]) -> OsScript {
    OsScript {
        inputs: inputs
            .iter()
            .map(|&(d, name)| (ms(d), name.as_bytes().to_vec()))
            .collect(),
        answer: Box::new(|_id, _frame| Vec::new()),
    }
}

/// A sink that logs each frame as `prefix` + its text.
fn sink(log: &Log, prefix: &'static str) -> hg_substrate::os::OsSink {
    let log = Arc::clone(log);
    Box::new(move |d| {
        if let OsDelivery::Frame(f) = d {
            log.lock()
                .unwrap()
                .push(format!("{prefix}{}", String::from_utf8_lossy(&f)));
        }
    })
}

#[test]
fn inputs_not_yet_delivered_wait_for_the_restarted_broker() {
    let log: Log = Arc::default();
    let w = World::new(SimConfig::new(5));
    let entry_log = Arc::clone(&log);
    let broker = w.spawn_broker(
        "browser",
        backing(),
        true,
        move |s| {
            let os = s.os.clone().unwrap();
            let seq = s.exec.sequence("browser.os", Priority::UserVisible);
            std::mem::forget(os.listen(seq, sink(&entry_log, "")).unwrap());
        },
        script(&[(0, "a"), (10, "b"), (20, "c")]),
    );
    assert_eq!(w.run(until(5)), RunEnd::TimeLimit);
    assert_eq!(*log.lock().unwrap(), ["a"]);
    assert!(w.crash(broker));
    assert_eq!(w.run(until(60_000)), RunEnd::Idle);
    assert_eq!(*log.lock().unwrap(), ["a", "b", "c"]);
}

#[test]
fn inputs_not_yet_delivered_wait_for_the_next_listener() {
    let log: Log = Arc::default();
    let w = World::new(SimConfig::new(6));
    let entry_log = Arc::clone(&log);
    let _broker = w.spawn_broker(
        "browser",
        backing(),
        false,
        move |s| {
            let os = s.os.clone().unwrap();
            let seq = s.exec.sequence("browser.os", Priority::UserVisible);
            let first: Arc<Mutex<Option<OsListening>>> = Arc::new(Mutex::new(Some(
                os.listen(seq, sink(&entry_log, "1:")).unwrap(),
            )));
            let log = Arc::clone(&entry_log);
            // At 5 ms the dispatcher stops listening and starts again.
            s.exec
                .post(
                    seq,
                    Task::new(move || {
                        drop(first.lock().unwrap().take());
                        std::mem::forget(os.listen(seq, sink(&log, "2:")).unwrap());
                    }),
                    Some(ms(5)),
                    Duration::ZERO,
                )
                .unwrap();
        },
        script(&[(0, "a"), (10, "b"), (20, "c")]),
    );
    assert_eq!(w.run(until(60_000)), RunEnd::Idle);
    assert_eq!(*log.lock().unwrap(), ["1:a", "2:b", "2:c"]);
}

#[test]
fn a_launch_storm_beyond_a_sequences_capacity_loses_nothing() {
    // OS integration OI-5: 1,000 launches at once; more than a sequence
    // holds, while the dispatcher's sequence has other work queued.
    let storm = SEQUENCE_CAPACITY + 100;
    let names: Vec<String> = (0..storm).map(|i| i.to_string()).collect();
    let inputs: Vec<(u64, &str)> = names.iter().map(|n| (0, n.as_str())).collect();
    let log: Log = Arc::default();
    let w = World::new(SimConfig::new(7));
    let entry_log = Arc::clone(&log);
    let _broker = w.spawn_broker(
        "browser",
        backing(),
        false,
        move |s| {
            let os = s.os.clone().unwrap();
            let seq = s.exec.sequence("browser.os", Priority::UserVisible);
            for i in 0..100 {
                let log = Arc::clone(&entry_log);
                s.exec
                    .post(
                        seq,
                        Task::new(move || log.lock().unwrap().push(format!("task {i}"))),
                        None,
                        Duration::ZERO,
                    )
                    .unwrap();
            }
            std::mem::forget(os.listen(seq, sink(&entry_log, "")).unwrap());
        },
        script(&inputs),
    );
    assert_eq!(w.run(until(60_000)), RunEnd::Idle);
    let lines = log.lock().unwrap().clone();
    let tasks = lines.iter().filter(|l| l.starts_with("task ")).count();
    let frames: Vec<&String> = lines.iter().filter(|l| !l.starts_with("task ")).collect();
    assert_eq!(tasks, 100);
    assert_eq!(frames.len(), storm, "frames lost");
    for (i, f) in frames.iter().enumerate() {
        assert_eq!(**f, i.to_string());
    }
}
