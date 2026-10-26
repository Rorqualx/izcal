//! Medium tests: the simulated launcher (M1 phase E; E-R3) and the
//! launch faults (E-R8).

use std::sync::{Arc, Mutex};

use hg_substrate::Memory;
use hg_substrate::ipc::{Delivery, Flow, Listening, OsPeer};
use hg_substrate::launch::{ChildKind, Launched};
use hg_substrate::{Priority, Substrate};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{
    ChildEntry, ChildTable, FaultPlan, Limits, ProcessId, RunEnd, SimConfig, World,
};
use hg_types::{Duration, Instant};

const LIMITS: Limits = Limits {
    max_steps: 100_000,
    max_time: Instant::from_nanos(86_400_000_000_000),
};

fn backing() -> Arc<dyn Fn() -> Arc<dyn Memory> + Send + Sync> {
    Arc::new(|| -> Arc<dyn Memory> { Arc::new(OsMemory::new()) })
}

/// A child that takes its bootstrap and holds it for the run.
fn holding_child() -> ChildEntry {
    let keep: Arc<Mutex<Vec<hg_substrate::ipc::Bootstrap>>> = Arc::default();
    ChildEntry {
        name: "child",
        backing: backing(),
        entry: Arc::new(move |s: Substrate| {
            if let Some(b) = s.ipc.bootstrap() {
                keep.lock().unwrap().push(b);
            }
        }),
    }
}

/// What the browser saw on its child's channel, and the peers it launched.
#[derive(Default)]
struct Seen {
    events: Vec<String>,
    peers: Vec<OsPeer>,
    keep: Vec<Listening>,
    to_child: Vec<hg_substrate::ipc::ChannelSender>,
}

/// A browser that launches one renderer, listens to it, and then runs
/// `then` with its launcher and the child's peer.
fn browser(
    world: &World,
    seen: &Arc<Mutex<Seen>>,
    then: impl Fn(&Substrate, OsPeer) + Send + Sync + 'static,
) -> ProcessId {
    let seen = Arc::clone(seen);
    world.spawn_launcher(
        "browser",
        backing()(),
        false,
        move |s| {
            let launcher = s.launcher.clone().unwrap();
            let Ok(Launched {
                from_child,
                to_child,
                peer,
            }) = launcher.launch(ChildKind::Renderer, 8)
            else {
                seen.lock().unwrap().events.push("launch failed".into());
                return;
            };
            let seq = s.exec.sequence("host", Priority::UserVisible);
            let seen2 = Arc::clone(&seen);
            let l = s
                .ipc
                .listen_bound(
                    from_child,
                    peer,
                    seq,
                    Box::new(move |d| {
                        seen2.lock().unwrap().events.push(match d {
                            Delivery::Message { .. } => "message".into(),
                            Delivery::Closed => "closed".into(),
                            Delivery::Violation(f) => format!("violation {:?}", f.kind),
                        });
                        Flow::Continue
                    }),
                )
                .unwrap();
            {
                let mut st = seen.lock().unwrap();
                st.keep.push(l);
                st.peers.push(peer);
                st.to_child.push(to_child);
            }
            then(&s, peer);
        },
        ChildTable::new().with(ChildKind::Renderer, holding_child()),
    )
}

fn world(seed: u64) -> World {
    faulty(seed, FaultPlan::NONE)
}

fn faulty(seed: u64, faults: FaultPlan) -> World {
    let mut c = SimConfig::new(seed);
    c.keep_trace_text = true;
    c.faults = faults;
    World::new(c)
}

fn drop_ends(seen: &Arc<Mutex<Seen>>) {
    let mut st = seen.lock().unwrap();
    st.keep.clear();
    st.to_child.clear();
}

/// A spawner death and a refused launch each fail the launch with
/// `Failed`, leave no process behind, and say which in the trace (E-R2,
/// E-R8).
#[test]
fn launch_faults_fail_the_launch_and_leave_nothing() {
    for (plan, said) in [
        (
            FaultPlan {
                spawner_death_per_million: 1_000_000,
                ..FaultPlan::NONE
            },
            "failed: spawner died, restarted",
        ),
        (
            FaultPlan {
                launch_fail_per_million: 1_000_000,
                ..FaultPlan::NONE
            },
            "failed: refused",
        ),
    ] {
        let w = faulty(5, plan);
        let seen: Arc<Mutex<Seen>> = Arc::default();
        browser(&w, &seen, |_s, _peer| {});
        assert_eq!(w.run(LIMITS), RunEnd::Idle);
        assert_eq!(seen.lock().unwrap().events, ["launch failed"]);
        assert!(!w.is_alive(ProcessId(1)), "no child was made");
        assert!(w.trace_text().unwrap().contains(said));
    }
}

/// A stalled child starts only after the stall, in virtual time: long
/// enough to miss the broker's `ready` deadline (E-R3).
#[test]
fn a_stalled_child_starts_late() {
    let w = faulty(
        6,
        FaultPlan {
            ready_stall_per_million: 1_000_000,
            ready_stall: Duration::from_secs(10),
            ..FaultPlan::NONE
        },
    );
    let seen: Arc<Mutex<Seen>> = Arc::default();
    browser(&w, &seen, |_s, _peer| {});
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    let trace = w.trace_text().unwrap();
    assert!(
        trace.contains("start child.0 stalled until 10000000000"),
        "{trace}"
    );
    assert_eq!(w.now().as_nanos(), 10_000_000_000);
    assert!(w.is_alive(ProcessId(1)));
    drop_ends(&seen);
}

/// `kill` ends the child it names: its channels close, so the browser
/// sees `Closed`.
#[test]
fn kill_ends_the_named_child() {
    let w = world(3);
    let seen: Arc<Mutex<Seen>> = Arc::default();
    browser(&w, &seen, |s, peer| s.launcher.as_ref().unwrap().kill(peer));
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    let st = seen.lock().unwrap();
    assert_eq!(st.events, ["closed"]);
    assert!(!w.is_alive(ProcessId(1)));
    assert!(w.trace_text().unwrap().contains("kill 1 by 0"));
}

/// `kill` never reaches a process the launcher did not start, nor a
/// later incarnation than the one it names.
#[test]
fn kill_reaches_only_its_own_child() {
    let w = world(4);
    let seen: Arc<Mutex<Seen>> = Arc::default();
    browser(&w, &seen, |s, peer| {
        let l = s.launcher.as_ref().unwrap();
        // The browser itself (pid 0), a pid never launched, and the
        // child under another generation.
        l.kill(OsPeer::new(0, 0));
        l.kill(OsPeer::new(7, 0));
        l.kill(OsPeer::new(peer.pid(), peer.generation().wrapping_add(1)));
    });
    w.run(LIMITS);
    assert!(w.is_alive(ProcessId(0)));
    assert!(w.is_alive(ProcessId(1)));
    assert!(seen.lock().unwrap().events.is_empty());
    assert!(!w.trace_text().unwrap().contains("kill "));
    // Drop the browser's ends before the world.
    drop_ends(&seen);
}
