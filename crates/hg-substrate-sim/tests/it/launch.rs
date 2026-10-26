//! Medium tests: the simulated launcher (M1 phase E; E-R3) and the
//! launch faults (E-R8).

use std::sync::{Arc, Mutex};

use hg_substrate::Memory;
use hg_substrate::ipc::{Delivery, Flow, Listening, OsPeer};
use hg_substrate::launch::{ChildKind, Launched};
use hg_substrate::{Priority, Substrate};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{ChildEntry, ChildTable, Limits, ProcessId, RunEnd, SimConfig, World};
use hg_types::Instant;

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
    let mut c = SimConfig::new(seed);
    c.keep_trace_text = true;
    World::new(c)
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
    let mut st = seen.lock().unwrap();
    st.keep.clear();
    st.to_child.clear();
}
