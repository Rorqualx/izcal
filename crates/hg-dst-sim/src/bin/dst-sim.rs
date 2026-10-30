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
//! - `faults`: the round trip under every phase E fault (E-R8, E-R9 1):
//!   IPC delay, reorder across channels and drop-as-Closed; spawner
//!   death, refused launches and children that miss the `ready`
//!   deadline; and crashes. Checked: the run ends; no answer is ever
//!   misrouted, duplicated or out of order; and unless the browser itself
//!   crashed or spent a kind's budget, a renderer completes every round.
//! - `relaunch`: the renderer is crashed at a seeded point mid-
//!   conversation (E-R9 1, relaunch after a crash). Checked: the browser
//!   relaunches it and the new renderer completes every round; answers
//!   for the dead renderer's asks never reach the new one.
//! - `liar`: the renderer's `ready` names another sandbox tier than its
//!   launcher applied (review M3). Checked: it is killed, never
//!   relaunched for it, and no ask of it is routed.
//!
//! Each fault class can also be set alone: `--delay-ppm`, `--reorder-ppm`,
//! `--drop-ppm`, `--spawner-death-ppm`, `--launch-fail-ppm` and
//! `--stall-ppm`. `--os linux|macos|windows` picks the OS whose behaviour
//! the simulator models; no scenario here depends on it, so every OS
//! model gives the same trace (OI-M0: hashes compared across OSes).
//!
//! Usage: `dst-sim --seed N [--crash-ppm N] [--scenario
//! roundtrip|du3|flood|netcrash|faults|relaunch|liar] [--os OS] [fault
//! rates]`.
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
use hg_substrate_sim::{ChildEntry, ChildTable, FaultPlan, Limits, SimConfig, SimOs, World};
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
    du3: bool,
    flood: bool,
    netcrash: bool,
    faults: bool,
    relaunch: bool,
    liar: bool,
    os: SimOs,
    plan: FaultPlan,
}

/// The `faults` scenario's rates: every phase E class at once.
const FAULTS: FaultPlan = FaultPlan {
    crash_per_million_steps: 1_000,
    ipc_delay_per_million: 200_000,
    ipc_reorder_per_million: 200_000,
    ipc_drop_per_million: 2_000,
    spawner_death_per_million: 100_000,
    launch_fail_per_million: 100_000,
    ready_stall_per_million: 50_000,
    ..FaultPlan::NONE
};

fn parse() -> Option<Args> {
    let mut a = std::env::args().skip(1);
    let mut args = Args {
        seed: 0,
        du3: false,
        flood: false,
        netcrash: false,
        faults: false,
        relaunch: false,
        liar: false,
        os: SimOs::Linux,
        plan: FaultPlan::NONE,
    };
    let mut seed = None;
    // Rates given on the command line, applied after the scenario's.
    let mut rates: Vec<(String, u32)> = Vec::new();
    while let Some(k) = a.next() {
        match k.as_str() {
            "--seed" => seed = Some(a.next()?.parse().ok()?),
            "--scenario" => match a.next()?.as_str() {
                "roundtrip" => args.du3 = false,
                "du3" => args.du3 = true,
                "flood" => args.flood = true,
                "netcrash" => args.netcrash = true,
                "faults" => args.faults = true,
                "relaunch" => args.relaunch = true,
                "liar" => args.liar = true,
                _ => return None,
            },
            "--os" => {
                args.os = match a.next()?.as_str() {
                    "linux" => SimOs::Linux,
                    "macos" => SimOs::MacOs,
                    "windows" => SimOs::Windows,
                    _ => return None,
                }
            }
            "--crash-ppm"
            | "--delay-ppm"
            | "--reorder-ppm"
            | "--drop-ppm"
            | "--spawner-death-ppm"
            | "--launch-fail-ppm"
            | "--stall-ppm" => {
                rates.push((k, a.next()?.parse().ok()?));
            }
            _ => return None,
        }
    }
    if args.faults {
        args.plan = FAULTS;
    }
    for (k, v) in rates {
        let p = &mut args.plan;
        match k.as_str() {
            "--crash-ppm" => p.crash_per_million_steps = v,
            "--delay-ppm" => p.ipc_delay_per_million = v,
            "--reorder-ppm" => p.ipc_reorder_per_million = v,
            "--drop-ppm" => p.ipc_drop_per_million = v,
            "--spawner-death-ppm" => p.spawner_death_per_million = v,
            "--launch-fail-ppm" => p.launch_fail_per_million = v,
            _ => p.ready_stall_per_million = v,
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
    liar: bool,
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
                    // `liar`: a report naming a tier its launcher did not
                    // apply (review M3).
                    sandbox: if liar {
                        SandboxReport {
                            tier: SandboxTier::LinuxFull,
                            missing: hg_ipc::BoundedVec::default(),
                        }
                    } else {
                        simulated()
                    },
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
            "usage: dst-sim --seed N [--scenario roundtrip|du3|flood|netcrash|faults|relaunch|liar] [--os linux|macos|windows] [--crash-ppm N] [--delay-ppm N] [--reorder-ppm N] [--drop-ppm N] [--spawner-death-ppm N] [--launch-fail-ppm N] [--stall-ppm N]"
        );
        return ExitCode::from(2);
    };
    let mut config = SimConfig::new(args.seed);
    config.faults = args.plan;
    config.region_os = args.os;
    let world = World::new(config);
    let keep: Keep = Arc::default();
    let renderer = Arc::new(hg_renderer::Progress::default());
    let browser = Arc::new(hg_browser::Progress::default());
    let table = ChildTable::new()
        .with(ChildKind::NetworkPartition, net_entry(&keep, args.flood))
        .with(
            ChildKind::Renderer,
            renderer_entry(&keep, &renderer, args.du3, args.flood, args.liar),
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
    let mut renderer_crashed = false;
    let mut completed_before = 0;
    if args.relaunch {
        // Run until the renderer has some answers, then end it (the first
        // renderer is process 2); the browser must relaunch it.
        let want = 1 + args.seed % (ROUNDS - 1);
        while renderer.answered.load(Ordering::SeqCst) < want {
            if world.step() == hg_substrate_sim::Step::Idle {
                break;
            }
        }
        // A seeded few steps more, so the crash lands at varied points of
        // a round (an ask or a work in flight, or none).
        world.run(Limits {
            max_steps: world.summary().steps + args.seed % 7,
            ..limits
        });
        completed_before = renderer.completed.load(Ordering::SeqCst);
        renderer_crashed = world.crash(hg_substrate_sim::ProcessId(2));
    }
    let end = world.run(limits);
    let browser_alive = world.is_alive(hg_substrate_sim::ProcessId(0));
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
    let completed = renderer.completed.load(Ordering::SeqCst);
    let wrong = renderer.refused.load(Ordering::SeqCst);
    let gave_up = browser.gave_up.load(Ordering::SeqCst);
    let missed = browser.missed_deadlines.load(Ordering::SeqCst);
    let refused = browser.refused_asks.load(Ordering::SeqCst);
    let max_pending = browser.max_pending.load(Ordering::SeqCst);
    let sandbox_mismatches = browser.sandbox_mismatches.load(Ordering::SeqCst);
    // The browser and its children hold channels into the world: end them
    // before the world goes.
    running.lock().unwrap().take();
    keep.lock().unwrap().clear();
    eprintln!(
        "dst-sim: launches={launches} relaunches={relaunches} routed={routed} returned={returned} answered={answered} completed={completed} refused={wrong} missed={missed} gave_up={gave_up} mismatches={mismatches} halted={halted} end={end:?}"
    );
    assert_eq!(end, hg_substrate_sim::RunEnd::Idle, "the run ends");
    assert!(relaunches <= 2 * u64::from(BUDGET), "relaunch budget");
    assert!(returned <= routed, "an answer without an ask");
    assert_eq!(wrong, 0, "an answer misrouted, duplicated or out of order");
    if args.liar {
        // Review M3: the renderer's report names another tier than the
        // launcher applied: it is killed, never relaunched for it, and no
        // ask of it is routed.
        assert_eq!(sandbox_mismatches, 1, "liar: one mismatch");
        assert_eq!(relaunches, 0, "liar: no relaunch");
        assert_eq!(answered, 0, "liar: no round trip");
    } else if args.flood {
        // The 65th ask ends the renderer's channel; the queue never grew
        // past the cap, and a violating renderer is not relaunched.
        assert_eq!(refused, 1, "flood: one refusal ends the channel");
        assert_eq!(max_pending, hg_browser::MAX_PENDING as u64, "flood: capped");
        // The silent network never reports ready: each launch of it
        // misses the deadline, is killed and launched again until its
        // budget is spent (E-R3). The violating renderer is never
        // relaunched, so every relaunch is the network's.
        assert_eq!(
            relaunches + gave_up,
            missed,
            "flood: no relaunch after a handler error"
        );
        assert_eq!(gave_up, 1, "flood: the network's budget is spent");
    } else if args.netcrash {
        assert_eq!(
            answered, ROUNDS,
            "netcrash: the renderer still gets every answer"
        );
        if net_crashed {
            assert!(relaunches >= 1, "netcrash: the network was relaunched");
        }
    } else if args.faults {
        if browser_alive && gave_up == 0 {
            assert!(completed >= 1, "faults: a renderer completes every round");
        }
    } else if args.relaunch {
        assert!(renderer_crashed, "relaunch: the renderer was running");
        assert!(relaunches >= 1, "relaunch: the renderer was relaunched");
        assert_eq!(
            completed,
            completed_before + 1,
            "relaunch: the new renderer completes"
        );
    } else if args.du3 {
        assert_eq!(mismatches, 1, "DU-3: one mismatch");
        assert!(halted, "DU-3: restart-required");
        // network + first renderer + the replaced renderer; nothing after.
        assert_eq!(launches, 3, "DU-3: no launch after the mismatch");
        // The mismatched child was told to exit, not missed: its deadline
        // ends it without counting a missed deadline.
        assert_eq!(missed, 0, "DU-3: no missed deadline");
    } else {
        assert!(!halted && mismatches == 0 && sandbox_mismatches == 0);
        if args.plan == FaultPlan::NONE {
            assert_eq!(answered, ROUNDS, "every round completes without faults");
        }
    }
    println!("{}", world.summary().report_line());
    ExitCode::SUCCESS
}
