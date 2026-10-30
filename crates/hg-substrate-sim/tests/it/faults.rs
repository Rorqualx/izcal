//! Medium tests: the simulator's phase E faults (rule E-R8): IPC delay,
//! reorder across channels and drop-as-Closed, each from its own stream.

use std::sync::{Arc, Mutex};

use hg_substrate::ipc::{ChannelSender, Delivery, Flow, IpcError, Listening, Transfer};
use hg_substrate::{Memory, Priority, Substrate};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{FaultPlan, Limits, RunEnd, SimConfig, World};
use hg_types::{Duration, Instant};

const LIMITS: Limits = Limits {
    max_steps: 100_000,
    max_time: Instant::from_nanos(86_400_000_000_000),
};

fn backing() -> Arc<dyn Memory> {
    Arc::new(OsMemory::new())
}

/// What one run saw: each delivery as "A3", "B0", "A closed", and the
/// result of every send.
#[derive(Default, Debug)]
struct Run {
    seen: Vec<String>,
    sends: Vec<Result<(), IpcError>>,
    keep: Vec<Listening>,
    senders: Vec<ChannelSender>,
    trace: String,
    end_ns: u64,
}

/// One process with two channels, A and B, both listened to on one
/// sequence. It sends `n` messages on each, alternating A, B, A, B.
fn two_channels(seed: u64, faults: FaultPlan, n: u32) -> Run {
    let mut config = SimConfig::new(seed);
    config.keep_trace_text = true;
    config.faults = faults;
    let world = World::new(config);
    let run: Arc<Mutex<Run>> = Arc::default();
    let r = Arc::clone(&run);
    world.spawn("p", backing(), false, move |s: Substrate| {
        let seq = s.exec.sequence("inbox", Priority::UserVisible);
        let mut txs = Vec::new();
        for name in ["A", "B"] {
            let (tx, rx) = s.ipc.channel(64, Transfer::Free).unwrap();
            let r2 = Arc::clone(&r);
            let l = s
                .ipc
                .listen(
                    rx,
                    seq,
                    Box::new(move |d| {
                        r2.lock().unwrap().seen.push(match d {
                            Delivery::Message { bytes, .. } => {
                                format!("{name}{}", String::from_utf8_lossy(&bytes))
                            }
                            Delivery::Closed => format!("{name} closed"),
                            Delivery::Violation(f) => format!("{name} violation {:?}", f.kind),
                        });
                        Flow::Continue
                    }),
                )
                .unwrap();
            r.lock().unwrap().keep.push(l);
            txs.push(tx);
        }
        for i in 0..n {
            for tx in &txs {
                let res = s.ipc.send(tx, i.to_string().into_bytes(), Vec::new());
                r.lock().unwrap().sends.push(res);
            }
        }
        r.lock().unwrap().senders = txs;
    });
    assert_eq!(world.run(LIMITS), RunEnd::Idle);
    let mut out = std::mem::take(&mut *run.lock().unwrap());
    out.trace = world.trace_text().unwrap();
    out.end_ns = world.now().as_nanos();
    // Ends close before the world goes.
    out.keep.clear();
    out.senders.clear();
    out
}

/// The messages of channel `name`, in delivery order, as numbers.
fn of(seen: &[String], name: &str) -> Vec<u32> {
    seen.iter()
        .filter_map(|s| s.strip_prefix(name)?.parse().ok())
        .collect()
}

/// A delay holds messages back in virtual time, and every channel
/// still delivers in send order (FIFO, the transport's contract).
#[test]
fn delays_keep_each_channel_in_order() {
    let faults = FaultPlan {
        ipc_delay_per_million: 500_000,
        ipc_delay_max: Duration::from_millis(40),
        ..FaultPlan::NONE
    };
    for seed in 0..20 {
        let r = two_channels(seed, faults, 30);
        assert_eq!(of(&r.seen, "A"), (0..30).collect::<Vec<_>>(), "seed {seed}");
        assert_eq!(of(&r.seen, "B"), (0..30).collect::<Vec<_>>(), "seed {seed}");
        assert!(r.trace.contains("ipc fault delay #"), "seed {seed}");
        assert!(r.end_ns > 0, "seed {seed}: virtual time moved");
    }
}

/// Reordering happens across channels only: some B message overtakes an
/// earlier A message, while each channel keeps its own order.
#[test]
fn reorder_crosses_channels_but_never_within_one() {
    let faults = FaultPlan {
        ipc_reorder_per_million: 300_000,
        ..FaultPlan::NONE
    };
    let calm = two_channels(0, FaultPlan::NONE, 30);
    let mut overtaken = 0;
    for seed in 0..20 {
        let r = two_channels(seed, faults, 30);
        assert_eq!(of(&r.seen, "A"), (0..30).collect::<Vec<_>>(), "seed {seed}");
        assert_eq!(of(&r.seen, "B"), (0..30).collect::<Vec<_>>(), "seed {seed}");
        assert!(r.trace.contains("ipc fault reorder #"), "seed {seed}");
        if r.seen != calm.seen {
            overtaken += 1;
        }
    }
    assert!(overtaken > 0, "no seed reordered across channels");
    // Without the fault the two channels interleave as sent.
    let interleaved: Vec<String> = (0..30)
        .flat_map(|i| [format!("A{i}"), format!("B{i}")])
        .collect();
    assert_eq!(calm.seen, interleaved);
}

/// A drop loses one message and closes its channel: the receiver sees
/// what came before, then `Closed`; later sends fail with `Closed`.
#[test]
fn a_dropped_message_closes_its_channel() {
    let faults = FaultPlan {
        ipc_drop_per_million: 50_000,
        ..FaultPlan::NONE
    };
    let mut dropped = 0;
    for seed in 0..20 {
        let r = two_channels(seed, faults, 30);
        for name in ["A", "B"] {
            let got = of(&r.seen, name);
            // A prefix of the sends, in order.
            assert_eq!(
                got,
                (0..30).take(got.len()).collect::<Vec<_>>(),
                "seed {seed}"
            );
            if got.len() < 30 {
                dropped += 1;
                let closed = format!("{name} closed");
                assert_eq!(
                    r.seen.iter().filter(|s| **s == closed).count(),
                    1,
                    "seed {seed}: the loss is never silent"
                );
            }
        }
        let refused = r
            .sends
            .iter()
            .filter(|s| **s == Err(IpcError::Closed))
            .count();
        let lost = r.trace.matches("ipc fault drop #").count();
        // Each lost message's channel refuses every later send.
        assert_eq!(
            refused + lost + of(&r.seen, "A").len() + of(&r.seen, "B").len(),
            60,
            "seed {seed}"
        );
    }
    assert!(dropped > 0);
}

/// Each class draws from its own stream: with every phase E class off
/// (rate 0) a run's trace is the one it has with the plan before they
/// existed, and turning on reorder leaves the delay class's draws as
/// they were (the same delays, in the same order).
#[test]
fn fault_classes_draw_from_their_own_streams() {
    let off = two_channels(9, FaultPlan::NONE, 30);
    let zero = two_channels(
        9,
        FaultPlan {
            ipc_delay_max: Duration::from_secs(3),
            ready_stall: Duration::from_secs(1),
            ..FaultPlan::NONE
        },
        30,
    );
    assert_eq!(off.trace, zero.trace);
    let delays = |t: &str| -> Vec<String> {
        t.lines()
            .filter_map(|l| l.strip_prefix("ipc fault delay "))
            .map(str::to_owned)
            .collect()
    };
    let delay = FaultPlan {
        ipc_delay_per_million: 400_000,
        ..FaultPlan::NONE
    };
    let alone = two_channels(9, delay, 30);
    let with_reorder = two_channels(
        9,
        FaultPlan {
            ipc_reorder_per_million: 400_000,
            ..delay
        },
        30,
    );
    assert!(!delays(&alone.trace).is_empty());
    // Every send happens in the entry, before any delivery, so the delay
    // draws are made in the same order in both runs.
    assert_eq!(delays(&alone.trace), delays(&with_reorder.trace));
    assert_ne!(alone.trace, with_reorder.trace);
}
