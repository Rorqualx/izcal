//! Medium tests: the simulated launcher (M1 phase E; E-R3) and the
//! launch faults (E-R8).

use std::sync::{Arc, Mutex};

use hg_substrate::Memory;
use hg_substrate::ipc::{Delivery, Flow, Listening, OsPeer, PeerAttest};
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
                    peer.into(),
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

/// What a launched child saw, for the PID-namespace tests.
type Saw = Arc<Mutex<Vec<String>>>;

fn sink_into(saw: &Saw) -> hg_substrate::ipc::Sink {
    let saw = Arc::clone(saw);
    Box::new(move |d| {
        saw.lock().unwrap().push(match d {
            Delivery::Message { bytes, handles } => {
                format!("{} +{}", String::from_utf8_lossy(&bytes), handles.len())
            }
            Delivery::Closed => "closed".into(),
            Delivery::Violation(f) => format!("violation {:?} {:?}", f.kind, f.sender),
        });
        Flow::Continue
    })
}

/// A browser that launches one child running `child`, then runs `then`
/// with its sender to the child; the child records what it sees.
fn launch_with(
    pid_namespaces: bool,
    child: impl Fn(Substrate, &Saw, &Arc<Mutex<Vec<Listening>>>) + Send + Sync + 'static,
    then: impl Fn(&Substrate, &hg_substrate::ipc::ChannelSender, OsPeer) + Send + Sync + 'static,
) -> (World, Saw, Arc<Mutex<Seen>>) {
    let mut c = SimConfig::new(11);
    c.keep_trace_text = true;
    c.pid_namespaces = pid_namespaces;
    let w = World::new(c);
    let saw: Saw = Arc::default();
    let keep: Arc<Mutex<Vec<Listening>>> = Arc::default();
    let (saw2, keep2) = (Arc::clone(&saw), Arc::clone(&keep));
    let seen: Arc<Mutex<Seen>> = Arc::default();
    let seen2 = Arc::clone(&seen);
    w.spawn_launcher(
        "browser",
        backing()(),
        false,
        move |s| {
            let launcher = s.launcher.clone().unwrap();
            let l = launcher.launch(ChildKind::Renderer, 8).unwrap();
            then(&s, &l.to_child, l.peer);
            let mut st = seen2.lock().unwrap();
            st.to_child.push(l.to_child);
            st.peers.push(l.peer);
            drop(l.from_child);
        },
        ChildTable::new().with(
            ChildKind::Renderer,
            ChildEntry {
                name: "child",
                backing: backing(),
                entry: Arc::new(move |s: Substrate| child(s, &saw2, &keep2)),
            },
        ),
    );
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    keep.lock().unwrap().clear();
    (w, saw, seen)
}

/// Design review M7: a child the launcher starts in a PID namespace gets
/// `PeerAttest::Inherited` as its parent, and its bootstrap receiver,
/// bound that way, takes the broker's messages: the channel attests.
#[test]
fn a_namespaced_child_binds_its_launcher_by_channel() {
    let (_w, saw, seen) = launch_with(
        true,
        |s, saw, keep| {
            let boot = s.ipc.bootstrap().unwrap();
            saw.lock()
                .unwrap()
                .push(format!("parent {:?}", boot.parent));
            let seq = s.exec.sequence("in", Priority::UserVisible);
            let l = s
                .ipc
                .listen_bound(boot.from_parent, PeerAttest::Inherited, seq, sink_into(saw))
                .unwrap();
            keep.lock().unwrap().push(l);
            std::mem::forget(boot.to_parent);
        },
        |s, to, _peer| s.ipc.send(to, b"hi".to_vec(), Vec::new()).unwrap(),
    );
    assert_eq!(*saw.lock().unwrap(), ["parent Inherited", "hi +0"]);
    drop_ends(&seen);
}

/// A namespaced child cannot attest a peer by pid: every outside sender
/// reads as pid 0, so a receiver bound to the broker's real pid refuses
/// the first record, and names no sender.
#[test]
fn a_namespaced_child_cannot_attest_by_pid() {
    let (_w, saw, seen) = launch_with(
        true,
        |s, saw, keep| {
            let boot = s.ipc.bootstrap().unwrap();
            let seq = s.exec.sequence("in", Priority::UserVisible);
            let broker = PeerAttest::Os(OsPeer::new(0, 0));
            let l = s
                .ipc
                .listen_bound(boot.from_parent, broker, seq, sink_into(saw))
                .unwrap();
            keep.lock().unwrap().push(l);
            std::mem::forget(boot.to_parent);
        },
        |s, to, _peer| s.ipc.send(to, b"hi".to_vec(), Vec::new()).unwrap(),
    );
    assert_eq!(*saw.lock().unwrap(), ["violation Identity None"]);
    drop_ends(&seen);
}

/// Without the modelled namespace the same child attests the broker by
/// its pid, as on macOS and Windows: the parent is `Os`, the pid-bound
/// receiver takes the message, and a refused record names its sender.
#[test]
fn without_a_pid_namespace_the_parent_is_named() {
    let (_w, saw, seen) = launch_with(
        false,
        |s, saw, keep| {
            let boot = s.ipc.bootstrap().unwrap();
            saw.lock()
                .unwrap()
                .push(format!("parent {:?}", boot.parent));
            let seq = s.exec.sequence("in", Priority::UserVisible);
            let l = s
                .ipc
                .listen_bound(boot.from_parent, boot.parent, seq, sink_into(saw))
                .unwrap();
            keep.lock().unwrap().push(l);
            std::mem::forget(boot.to_parent);
        },
        |s, to, _peer| s.ipc.send(to, b"hi".to_vec(), Vec::new()).unwrap(),
    );
    let saw = saw.lock().unwrap().clone();
    assert_eq!(saw.len(), 2, "{saw:?}");
    assert!(saw[0].starts_with("parent Os(OsPeer { pid: 0"), "{saw:?}");
    assert_eq!(saw[1], "hi +0");
    drop_ends(&seen);
}

/// Every fault a namespaced child reports names no sender, even one the
/// transport could attribute from outside (M7); outside a namespace the
/// same fault names the broker.
#[test]
fn a_namespaced_child_reports_faults_with_no_sender() {
    for (ns, want) in [
        (true, "violation Malformed None"),
        (false, "violation Malformed Some(0)"),
    ] {
        let mut c = SimConfig::new(12);
        c.pid_namespaces = ns;
        let w = World::new(c);
        let saw: Saw = Arc::default();
        let keep: Arc<Mutex<Vec<Listening>>> = Arc::default();
        let seen: Arc<Mutex<Seen>> = Arc::default();
        let (saw2, keep2, seen2) = (Arc::clone(&saw), Arc::clone(&keep), Arc::clone(&seen));
        let browser = w.spawn_launcher(
            "browser",
            backing()(),
            false,
            move |s| {
                let l = s
                    .launcher
                    .clone()
                    .unwrap()
                    .launch(ChildKind::Renderer, 8)
                    .unwrap();
                s.ipc.send(&l.to_child, b"x".to_vec(), Vec::new()).unwrap();
                seen2.lock().unwrap().to_child.push(l.to_child);
            },
            ChildTable::new().with(
                ChildKind::Renderer,
                ChildEntry {
                    name: "child",
                    backing: backing(),
                    entry: Arc::new(move |s: Substrate| {
                        let boot = s.ipc.bootstrap().unwrap();
                        let seq = s.exec.sequence("in", Priority::UserVisible);
                        let l = s
                            .ipc
                            .listen_bound(boot.from_parent, boot.parent, seq, sink_into(&saw2))
                            .unwrap();
                        keep2.lock().unwrap().push(l);
                        std::mem::forget(boot.to_parent);
                    }),
                },
            ),
        );
        w.corrupt_next_send(browser, hg_substrate::ipc::FaultKind::Malformed);
        assert_eq!(w.run(LIMITS), RunEnd::Idle);
        assert_eq!(*saw.lock().unwrap(), [want], "ns={ns}");
        keep.lock().unwrap().clear();
        drop_ends(&seen);
    }
}

/// D6 writers under the namespace: a region writer reaches a namespaced
/// child on its inherited bootstrap channel, the one only the broker
/// sends on; on any other channel the child cannot tell who sent it, so
/// the writer is refused, naming no sender.
#[test]
fn a_namespaced_child_takes_writers_only_on_its_bootstrap() {
    const OWNER: OsPeer = OsPeer::new(1, 0);
    // On the bootstrap channel: accepted.
    let (_w, saw, seen) = launch_with(
        true,
        |s, saw, keep| {
            let boot = s.ipc.bootstrap().unwrap();
            let seq = s.exec.sequence("in", Priority::UserVisible);
            let l = s
                .ipc
                .listen_bound(boot.from_parent, boot.parent, seq, sink_into(saw))
                .unwrap();
            keep.lock().unwrap().push(l);
            std::mem::forget(boot.to_parent);
        },
        |s, to, _peer| {
            let wr = s.ipc.allocate_region(4096, OWNER).unwrap();
            s.ipc
                .send(to, b"w".to_vec(), vec![wr.into_handle()])
                .unwrap();
        },
    );
    assert_eq!(*saw.lock().unwrap(), ["w +1"]);
    drop_ends(&seen);
    // On a channel the broker made and handed over: refused.
    let (_w, saw, seen) = launch_with(
        true,
        |s, saw, keep| {
            let boot = s.ipc.bootstrap().unwrap();
            let seq = s.exec.sequence("in", Priority::UserVisible);
            let (s2, saw2, keep2) = (s.clone(), Arc::clone(saw), Arc::clone(keep));
            let l = s
                .ipc
                .listen_bound(
                    boot.from_parent,
                    boot.parent,
                    seq,
                    Box::new(move |d| {
                        if let Delivery::Message { mut handles, .. } = d
                            && let Some(h) = handles.pop()
                            && let Ok(rx) = hg_substrate::ipc::ChannelReceiver::from_handle(h)
                        {
                            let seq = s2.exec.sequence("side", Priority::UserVisible);
                            let l = s2.ipc.listen(rx, seq, sink_into(&saw2)).unwrap();
                            keep2.lock().unwrap().push(l);
                        }
                        Flow::Continue
                    }),
                )
                .unwrap();
            keep.lock().unwrap().push(l);
            std::mem::forget(boot.to_parent);
        },
        |s, to, _peer| {
            let (tx, rx) = s.ipc.channel(4, hg_substrate::ipc::Transfer::Free).unwrap();
            s.ipc
                .send(to, b"rx".to_vec(), vec![rx.into_handle()])
                .unwrap();
            let wr = s.ipc.allocate_region(4096, OWNER).unwrap();
            s.ipc
                .send(&tx, b"w".to_vec(), vec![wr.into_handle()])
                .unwrap();
            std::mem::forget(tx);
        },
    );
    assert_eq!(*saw.lock().unwrap(), ["violation BadHandle None"]);
    drop_ends(&seen);
}
