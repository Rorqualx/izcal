//! `dst-sim`: the M1 process skeletons under the simulator (Stage 1
//! design 8.1 M1: "an IPC round trip runs in simulation with fault
//! injection"; distribution DU-3).
//!
//! The browser library (`hg-browser`) runs as the simulated broker with a
//! launcher; it launches the network partition service (`hg-net-service`)
//! and a renderer (`hg-renderer`), routes the renderer's asks through the
//! network and back, and relaunches a child that ends.
//!
//! Scenarios:
//! - `roundtrip` (default): [`ROUNDS`] round trips under the fault plan
//!   (`--crash-ppm`). Checked: the run ends; with no crashes every round
//!   completes; the browser never relaunches past its budget; answers
//!   come back no more often than asks went out.
//! - `du3`: the renderer's binary is replaced between two launches. The
//!   first renderer starts and ends; the relaunch reports another
//!   `BuildId`. Checked: the browser enters restart-required on that one
//!   mismatch and launches nothing after it (DU-3: 0 retries).
//!
//! Usage: `dst-sim --seed N [--crash-ppm N] [--scenario roundtrip|du3]`.
//! The last line is `hg_substrate_sim::Summary::report_line`, as
//! `hg-dst` expects; a broken invariant panics, which `hg-dst` records.

use std::any::Any;
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use hg_browser::BrowserConfig;
use hg_ipc::Remote;
use hg_ipc_gen::process::{
    BuildId, ProcessHost, ProcessHostRemote, ProcessKind, SandboxReport, SandboxTier,
};
use hg_net_service::NetConfig;
use hg_renderer::RendererConfig;
use hg_substrate::launch::ChildKind;
use hg_substrate::{Memory, Substrate};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{ChildEntry, ChildTable, FaultPlan, Limits, SimConfig, World};
use hg_types::Instant;

/// Round trips the renderer makes.
const ROUNDS: u64 = 50;
/// Relaunches per kind.
const BUDGET: u32 = 8;

type Keep = Arc<Mutex<Vec<Box<dyn Any + Send>>>>;

fn simulated() -> SandboxReport {
    SandboxReport {
        tier: SandboxTier::Simulated,
        missing: hg_ipc::BoundedVec::default(),
    }
}

fn backing() -> Arc<dyn Fn() -> Arc<dyn Memory> + Send + Sync> {
    Arc::new(|| -> Arc<dyn Memory> { Arc::new(OsMemory::new()) })
}

struct Args {
    seed: u64,
    crash_ppm: u32,
    du3: bool,
    flood: bool,
    netcrash: bool,
}

fn parse() -> Option<Args> {
    let mut a = std::env::args().skip(1);
    let mut args = Args {
        seed: 0,
        crash_ppm: 0,
        du3: false,
        flood: false,
        netcrash: false,
    };
    let mut seed = None;
    while let Some(k) = a.next() {
        match k.as_str() {
            "--seed" => seed = Some(a.next()?.parse().ok()?),
            "--crash-ppm" => args.crash_ppm = a.next()?.parse().ok()?,
            "--scenario" => match a.next()?.as_str() {
                "roundtrip" => args.du3 = false,
                "du3" => args.du3 = true,
                "flood" => args.flood = true,
                "netcrash" => args.netcrash = true,
                _ => return None,
            },
            _ => return None,
        }
    }
    args.seed = seed?;
    Some(args)
}

/// The network partition service's entry. With `silent` it holds its
/// channels and never reports ready (the `flood` scenario).
fn net_entry(keep: &Keep, silent: bool) -> ChildEntry {
    let k = Arc::clone(keep);
    // One set of counters for every incarnation, as a long-lived owner
    // would keep: each incarnation must still take its own `start`.
    let shared = Arc::new(hg_net_service::Progress::default());
    ChildEntry {
        name: "network",
        backing: backing(),
        entry: Arc::new(move |s: Substrate| {
            if silent {
                if let Some(boot) = s.ipc.bootstrap() {
                    k.lock().unwrap().push(Box::new(boot));
                }
                return;
            }
            let running = hg_net_service::run(
                &s,
                NetConfig {
                    kind: ProcessKind::NetworkPartition,
                    sandbox: simulated(),
                    progress: Arc::clone(&shared),
                },
            );
            if let Some(r) = running {
                k.lock().unwrap().push(Box::new(r));
            }
        }),
    }
}

/// The renderer's entry. For `du3`, the first launch ends right after
/// `ready` (its process exits) and every later launch is the replaced
/// binary, which reports another build.
fn renderer_entry(
    keep: &Keep,
    progress: &Arc<hg_renderer::Progress>,
    du3: bool,
    flood: bool,
) -> ChildEntry {
    let (k, p) = (Arc::clone(keep), Arc::clone(progress));
    let launches = Arc::new(AtomicU64::new(0));
    ChildEntry {
        name: "renderer",
        backing: backing(),
        entry: Arc::new(move |s: Substrate| {
            let n = launches.fetch_add(1, Ordering::SeqCst);
            if flood {
                // A hostile renderer: ready, then 100 asks at once.
                if let Some(boot) = s.ipc.bootstrap() {
                    let host = Remote::<ProcessHost>::from_sender(&s.ipc, boot.to_parent);
                    let _ = host.ready(
                        ProcessKind::Renderer,
                        BuildId {
                            id: hg_types::BUILD_ID,
                        },
                        simulated(),
                    );
                    // The channel's own capacity holds 64 undelivered
                    // messages, so the renderer retries each ask the
                    // transport refuses at capacity: once the browser has
                    // taken them off the channel, only its queue cap
                    // stands in the way.
                    k.lock().unwrap().push(Box::new(boot.from_parent));
                    flood_asks(&s, Arc::new(host), 0);
                }
                return;
            }
            if du3 && n > 0 {
                // The replaced binary: same protocol, another build.
                if let Some(boot) = s.ipc.bootstrap() {
                    let host = Remote::<ProcessHost>::from_sender(&s.ipc, boot.to_parent);
                    let _ = host.ready(
                        ProcessKind::Renderer,
                        BuildId {
                            id: hg_types::BUILD_ID ^ 1,
                        },
                        simulated(),
                    );
                    k.lock().unwrap().push(Box::new(host));
                }
                return;
            }
            let running = hg_renderer::run(
                &s,
                RendererConfig {
                    rounds: if du3 { 0 } else { ROUNDS },
                    sandbox: simulated(),
                    progress: Arc::clone(&p),
                },
            );
            if let (Some(r), false) = (running, du3) {
                k.lock().unwrap().push(Box::new(r));
            }
            // With du3 the first renderer drops its endpoints here: its
            // channels close, as when the process exits.
        }),
    }
}

/// Sends asks `n..100`, one per task, retrying one the transport refuses
/// (`PeerUnresponsive`: the channel is at capacity) after a millisecond of virtual time.
fn flood_asks(s: &Substrate, host: Arc<Remote<ProcessHost>>, n: u64) {
    if n >= 100 {
        return;
    }
    let seq = s
        .exec
        .sequence("flood", hg_substrate::Priority::UserVisible);
    let s2 = s.clone();
    let (next, delay) = match host.ask(n, hg_ipc::BoundedBytes::default()) {
        Err(hg_ipc::SendError::PeerUnresponsive) => (n, Some(hg_types::Duration::from_millis(1))),
        Err(_) => return,
        Ok(()) => (n.saturating_add(1), None),
    };
    let _ = s.exec.post(
        seq,
        hg_substrate::Task::new(move || flood_asks(&s2, host, next)),
        delay,
        hg_types::Duration::ZERO,
    );
}

fn main() -> ExitCode {
    let Some(args) = parse() else {
        eprintln!(
            "usage: dst-sim --seed N [--crash-ppm N] [--scenario roundtrip|du3|flood|netcrash]"
        );
        return ExitCode::from(2);
    };
    let mut config = SimConfig::new(args.seed);
    config.faults = FaultPlan {
        crash_per_million_steps: args.crash_ppm,
        ..FaultPlan::NONE
    };
    let world = World::new(config);
    let keep: Keep = Arc::default();
    let renderer = Arc::new(hg_renderer::Progress::default());
    let browser = Arc::new(hg_browser::Progress::default());
    let table = ChildTable::new()
        .with(ChildKind::NetworkPartition, net_entry(&keep, args.flood))
        .with(
            ChildKind::Renderer,
            renderer_entry(&keep, &renderer, args.du3, args.flood),
        );
    let running: Arc<Mutex<Option<hg_browser::Running>>> = Arc::default();
    let (bp, run_slot) = (Arc::clone(&browser), Arc::clone(&running));
    world.spawn_launcher(
        "browser",
        backing()(),
        false,
        move |s| {
            let r = hg_browser::run(
                &s,
                BrowserConfig {
                    restart_budget: BUDGET,
                    progress: Arc::clone(&bp),
                },
            );
            if let Err(e) = &r {
                eprintln!(
                    "dst-sim: browser run failed: {e:?}; launcher failures {}",
                    bp.launch_failures.load(Ordering::SeqCst)
                );
            }
            *run_slot.lock().unwrap() = r.ok();
        },
        table,
    );
    let limits = Limits {
        max_steps: 2_000_000,
        max_time: Instant::from_nanos(120_000_000_000),
    };
    let mut net_crashed = false;
    if args.netcrash {
        // Run to a seeded point mid-conversation, end the network (the
        // first launch, so process 1), and go on: its unanswered work must
        // reach the relaunched network.
        let at = 20 + args.seed % 150;
        world.run(Limits {
            max_steps: at,
            ..limits
        });
        net_crashed = world.crash(hg_substrate_sim::ProcessId(1));
    }
    world.run(limits);
    let halted = running
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(hg_browser::Running::restart_required);
    let launches = browser.launches.load(Ordering::SeqCst);
    let relaunches = browser.relaunches.load(Ordering::SeqCst);
    let routed = browser.routed.load(Ordering::SeqCst);
    let returned = browser.returned.load(Ordering::SeqCst);
    let mismatches = browser.mismatches.load(Ordering::SeqCst);
    let answered = renderer.answered.load(Ordering::SeqCst);
    let refused = browser.refused_asks.load(Ordering::SeqCst);
    let max_pending = browser.max_pending.load(Ordering::SeqCst);
    // The browser and its children hold channels into the world: end them
    // before the world goes.
    running.lock().unwrap().take();
    keep.lock().unwrap().clear();
    eprintln!(
        "dst-sim: launches={launches} relaunches={relaunches} routed={routed} returned={returned} answered={answered} mismatches={mismatches} halted={halted}"
    );
    assert!(relaunches <= 2 * u64::from(BUDGET), "relaunch budget");
    assert!(returned <= routed, "an answer without an ask");
    if args.flood {
        // The 65th ask ends the renderer's channel; the queue never grew
        // past the cap, and a violating renderer is not relaunched.
        assert_eq!(refused, 1, "flood: one refusal ends the channel");
        assert_eq!(max_pending, hg_browser::MAX_PENDING as u64, "flood: capped");
        assert_eq!(relaunches, 0, "flood: no relaunch after a handler error");
    } else if args.netcrash {
        assert_eq!(
            answered, ROUNDS,
            "netcrash: the renderer still gets every answer"
        );
        if net_crashed {
            assert!(relaunches >= 1, "netcrash: the network was relaunched");
        }
    } else if args.du3 {
        assert_eq!(mismatches, 1, "DU-3: one mismatch");
        assert!(halted, "DU-3: restart-required");
        // network + first renderer + the replaced renderer; nothing after.
        assert_eq!(launches, 3, "DU-3: no launch after the mismatch");
    } else {
        assert!(!halted && mismatches == 0);
        if args.crash_ppm == 0 {
            assert_eq!(answered, ROUNDS, "every round completes without faults");
        }
    }
    println!("{}", world.summary().report_line());
    ExitCode::SUCCESS
}
