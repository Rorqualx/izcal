//! Medium tests: the simulated IPC transport (Stage 1 4.2; IPC-R1, R2,
//! R8; CS-AS-6 (b)).

use hg_substrate::ipc::PeerAttest;
use std::sync::{Arc, Mutex};

use hg_substrate::ipc::{
    ChannelReceiver, ChannelSender, Delivery, Flow, Handle, IpcError, Listening, OsPeer, Transfer,
};
use hg_substrate::{Memory, Priority, Substrate};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{Limits, ProcessId, RunEnd, SimConfig, World};
use hg_types::Instant;

const LIMITS: Limits = Limits {
    max_steps: 100_000,
    max_time: Instant::from_nanos(86_400_000_000_000),
};

fn backing() -> Arc<dyn Memory> {
    Arc::new(OsMemory::new())
}

type Slot<T> = Arc<Mutex<Option<T>>>;

fn slot<T>() -> Slot<T> {
    Arc::new(Mutex::new(None))
}

/// What a listener saw: message bytes as text, "closed", or
/// "violation <kind> <sender>".
type Seen = Arc<Mutex<Vec<String>>>;

/// Listens on `rx` in `s`, recording each delivery into `seen`, and
/// keeps every received handle in `got`.
fn record(s: &Substrate, rx: ChannelReceiver, seen: &Seen, got: &Slot<Vec<Handle>>) -> Listening {
    let seq = s.exec.sequence("inbox", Priority::UserVisible);
    let (seen, got) = (Arc::clone(seen), Arc::clone(got));
    s.ipc
        .listen(
            rx,
            seq,
            Box::new(move |d| {
                match d {
                    Delivery::Message { bytes, handles } => {
                        seen.lock()
                            .unwrap()
                            .push(String::from_utf8_lossy(&bytes).into_owned());
                        got.lock()
                            .unwrap()
                            .get_or_insert_with(Vec::new)
                            .extend(handles);
                    }
                    Delivery::Closed => seen.lock().unwrap().push("closed".into()),
                    Delivery::Violation(f) => seen
                        .lock()
                        .unwrap()
                        .push(format!("violation {:?} {:?}", f.kind, f.sender)),
                }
                Flow::Continue
            }),
        )
        .unwrap()
}

/// A parent and a linked child; the parent's entry gets its ends.
struct Pair {
    world: World,
    parent: ProcessId,
    child: ProcessId,
}

fn pair(
    seed: u64,
    capacity: u32,
    parent_entry: impl Fn(Substrate, ChannelReceiver, ChannelSender) + Send + Sync + 'static,
    child_entry: impl Fn(Substrate) + Send + Sync + 'static,
) -> Pair {
    let mut config = SimConfig::new(seed);
    config.keep_trace_text = true;
    let world = World::new(config);
    let ends: Slot<(ChannelReceiver, ChannelSender)> = slot();
    let e = Arc::clone(&ends);
    let parent = world.spawn("parent", backing(), false, move |s| {
        if let Some((from_child, to_child)) = e.lock().unwrap().take() {
            parent_entry(s, from_child, to_child);
        }
    });
    let child = world.spawn("child", backing(), false, child_entry);
    *ends.lock().unwrap() = world.link(parent, child, capacity);
    Pair {
        world,
        parent,
        child,
    }
}

#[test]
fn messages_arrive_in_order_then_closed() {
    let seen: Seen = Arc::default();
    let got = slot();
    let keep: Slot<Listening> = slot();
    let (seen2, got2, keep2) = (Arc::clone(&seen), Arc::clone(&got), Arc::clone(&keep));
    let p = pair(
        7,
        64,
        |s, _from_child, to_child| {
            for n in 0..20 {
                s.ipc
                    .send(&to_child, format!("m{n}").into_bytes(), Vec::new())
                    .unwrap();
            }
            // Dropping `to_child` closes the channel after the 20.
        },
        move |s| {
            let boot = s.ipc.bootstrap().unwrap();
            *keep2.lock().unwrap() = Some(record(&s, boot.from_parent, &seen2, &got2));
            assert!(s.ipc.bootstrap().is_none(), "taken once");
        },
    );
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    let mut want: Vec<String> = (0..20).map(|n| format!("m{n}")).collect();
    want.push("closed".into());
    assert_eq!(*seen.lock().unwrap(), want);
}

#[test]
fn a_full_receiver_refuses_and_a_closed_one_reports_it() {
    let results: Slot<Vec<Result<(), IpcError>>> = slot();
    let r = Arc::clone(&results);
    let p = pair(
        1,
        3,
        move |s, _from_child, to_child| {
            let out: Vec<_> = (0..4)
                .map(|_| s.ipc.send(&to_child, b"x".to_vec(), Vec::new()))
                .collect();
            *r.lock().unwrap() = Some(out);
        },
        // The child never listens.
        |_s| {},
    );
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    let got = results.lock().unwrap().take().unwrap();
    assert_eq!(got, [Ok(()), Ok(()), Ok(()), Err(IpcError::Full)]);

    // Once the child is gone, sends report the closed end.
    assert!(p.world.crash(p.child));
    let s2: Slot<Result<(), IpcError>> = slot();
    let s3 = Arc::clone(&s2);
    let w = World::new(SimConfig::new(2));
    let a = w.spawn("a", backing(), false, move |s| {
        let (tx, rx) = s.ipc.channel(4, Transfer::Free).unwrap();
        drop(rx);
        *s3.lock().unwrap() = Some(s.ipc.send(&tx, b"x".to_vec(), Vec::new()));
    });
    let _ = a;
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(s2.lock().unwrap().take(), Some(Err(IpcError::Closed)));
}

#[test]
fn a_crash_closes_the_peers_channel() {
    let seen: Seen = Arc::default();
    let got = slot();
    let keep: Slot<Listening> = slot();
    let (seen2, got2, keep2) = (Arc::clone(&seen), Arc::clone(&got), Arc::clone(&keep));
    let held: Slot<ChannelSender> = slot();
    let held2 = Arc::clone(&held);
    let p = pair(
        3,
        8,
        move |s, from_child, _to_child| {
            *keep2.lock().unwrap() = Some(record(&s, from_child, &seen2, &got2));
        },
        move |s| {
            let boot = s.ipc.bootstrap().unwrap();
            s.ipc
                .send(&boot.to_parent, b"hello".to_vec(), Vec::new())
                .unwrap();
            // Kept alive past the entry: only the crash closes it.
            *held2.lock().unwrap() = Some(boot.to_parent);
        },
    );
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    assert!(p.world.crash(p.child));
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    assert_eq!(*seen.lock().unwrap(), ["hello", "closed"]);
    // The crashed process's end, dropped late, changes nothing.
    drop(held.lock().unwrap().take());
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    assert_eq!(seen.lock().unwrap().len(), 2);
    let _ = p.parent;
}

#[test]
fn a_sent_receiver_delivers_in_its_new_process() {
    let seen: Seen = Arc::default();
    let got: Slot<Vec<Handle>> = slot();
    let keep: Slot<Listening> = slot();
    let inner: Slot<Listening> = slot();
    let (seen2, got2, keep2, inner2) = (
        Arc::clone(&seen),
        Arc::clone(&got),
        Arc::clone(&keep),
        Arc::clone(&inner),
    );
    let p = pair(
        11,
        8,
        |s, _from_child, to_child| {
            let (tx, rx) = s.ipc.channel(8, Transfer::Free).unwrap();
            s.ipc
                .send(&to_child, b"take".to_vec(), vec![rx.into_handle()])
                .unwrap();
            s.ipc
                .send(&tx, b"via-new-channel".to_vec(), Vec::new())
                .unwrap();
        },
        move |s| {
            let boot = s.ipc.bootstrap().unwrap();
            let s2 = s.clone();
            let (seen3, inner3) = (Arc::clone(&seen2), Arc::clone(&inner2));
            let seq = s.exec.sequence("boot", Priority::UserVisible);
            let l = s
                .ipc
                .listen(
                    boot.from_parent,
                    seq,
                    Box::new(move |d| {
                        if let Delivery::Message { mut handles, .. } = d
                            && let Some(h) = handles.pop()
                        {
                            let rx = ChannelReceiver::from_handle(h).unwrap();
                            let none = slot();
                            *inner3.lock().unwrap() = Some(record(&s2, rx, &seen3, &none));
                        }
                        Flow::Continue
                    }),
                )
                .unwrap();
            *keep2.lock().unwrap() = Some(l);
            let _ = &got2;
        },
    );
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    assert_eq!(*seen.lock().unwrap(), ["via-new-channel", "closed"]);
}

#[test]
fn a_bound_sender_is_granted_once_and_never_forwarded() {
    let results: Slot<Vec<Result<(), IpcError>>> = slot();
    let r = Arc::clone(&results);
    let keep: Slot<Listening> = slot();
    let keep2 = Arc::clone(&keep);
    let p = pair(
        5,
        8,
        move |s, from_child, to_child| {
            let (tx, _rx) = s.ipc.channel(8, Transfer::Bound).unwrap();
            let first = s
                .ipc
                .send(&to_child, b"grant".to_vec(), vec![tx.into_handle()]);
            // The parent listens for the child's attempt to send it back.
            let seq = s.exec.sequence("back", Priority::UserVisible);
            let r2 = Arc::clone(&r);
            *keep2.lock().unwrap() = Some(
                s.ipc
                    .listen(from_child, seq, Box::new(|_d| Flow::Continue))
                    .unwrap(),
            );
            r2.lock().unwrap().get_or_insert_with(Vec::new).push(first);
        },
        {
            let r = Arc::clone(&results);
            move |s| {
                let boot = s.ipc.bootstrap().unwrap();
                let s2 = s.clone();
                let r2 = Arc::clone(&r);
                let up = Arc::new(Mutex::new(Some(boot.to_parent)));
                let seq = s.exec.sequence("in", Priority::UserVisible);
                let l = s
                    .ipc
                    .listen(
                        boot.from_parent,
                        seq,
                        Box::new(move |d| {
                            if let Delivery::Message { handles, .. } = d {
                                let guard = up.lock().unwrap();
                                if let Some(to_parent) = guard.as_ref() {
                                    let back = s2.ipc.send(to_parent, b"back".to_vec(), handles);
                                    r2.lock().unwrap().get_or_insert_with(Vec::new).push(back);
                                }
                            }
                            Flow::Continue
                        }),
                    )
                    .unwrap();
                std::mem::forget(l);
            }
        },
    );
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    let got = results.lock().unwrap().take().unwrap();
    assert_eq!(got, [Ok(()), Err(IpcError::Refused)]);
}

#[test]
fn a_handle_this_process_does_not_hold_is_refused() {
    // The parent makes a channel and leaks its sender to the child through
    // shared test memory, as no real process could; the transport knows
    // the child does not hold it.
    let smuggled: Slot<ChannelSender> = slot();
    let result: Slot<Result<(), IpcError>> = slot();
    let (sm2, sm3, res2) = (
        Arc::clone(&smuggled),
        Arc::clone(&smuggled),
        Arc::clone(&result),
    );
    let p = pair(
        9,
        8,
        move |s, _from_child, _to_child| {
            let (tx, rx) = s.ipc.channel(8, Transfer::Free).unwrap();
            std::mem::forget(rx);
            *sm2.lock().unwrap() = Some(tx);
        },
        move |s| {
            let seq = s.exec.sequence("later", Priority::BestEffort);
            let (s2, sm, res) = (s.clone(), Arc::clone(&sm3), Arc::clone(&res2));
            s.exec
                .post(
                    seq,
                    hg_substrate::Task::new(move || {
                        if let Some(tx) = sm.lock().unwrap().as_ref() {
                            *res.lock().unwrap() = Some(s2.ipc.send(tx, b"x".to_vec(), Vec::new()));
                        }
                    }),
                    Some(hg_types::Duration::from_millis(1)),
                    hg_types::Duration::ZERO,
                )
                .unwrap();
        },
    );
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    assert_eq!(result.lock().unwrap().take(), Some(Err(IpcError::Refused)));
}

fn chatter(seed: u64) -> (u64, String) {
    let seen: Seen = Arc::default();
    let keep: Slot<Listening> = slot();
    let (seen2, keep2) = (Arc::clone(&seen), Arc::clone(&keep));
    let p = pair(
        seed,
        64,
        |s, _from_child, to_child| {
            let a = s.exec.sequence("a", Priority::UserVisible);
            let b = s.exec.sequence("b", Priority::UserVisible);
            let tx = Arc::new(to_child);
            for n in 0..10 {
                for (seq, tag) in [(a, "a"), (b, "b")] {
                    let (s2, tx2) = (s.clone(), Arc::clone(&tx));
                    s.exec
                        .post(
                            seq,
                            hg_substrate::Task::new(move || {
                                let _ =
                                    s2.ipc
                                        .send(&tx2, format!("{tag}{n}").into_bytes(), Vec::new());
                            }),
                            None,
                            hg_types::Duration::ZERO,
                        )
                        .unwrap();
                }
            }
        },
        move |s| {
            let boot = s.ipc.bootstrap().unwrap();
            let none = slot();
            *keep2.lock().unwrap() = Some(record(&s, boot.from_parent, &seen2, &none));
        },
    );
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    let order = seen.lock().unwrap().join(",");
    (p.world.summary().trace_hash, order)
}

#[test]
fn the_same_seed_gives_the_same_trace_and_order() {
    let (h1, o1) = chatter(42);
    let (h2, o2) = chatter(42);
    assert_eq!((h1, &o1), (h2, &o2));
    // Interleaving across the two sending sequences follows the seed.
    let differs = (0..8).any(|seed| chatter(seed).1 != o1);
    assert!(differs, "the seed should choose the interleaving: {o1}");
}

#[test]
fn a_bound_receiver_never_moves() {
    let result: Slot<Result<(), IpcError>> = slot();
    let r = Arc::clone(&result);
    let w = World::new(SimConfig::new(13));
    let _p = w.spawn("p", backing(), false, move |s| {
        let (_tx, rx) = s.ipc.channel(4, Transfer::Bound).unwrap();
        let (carrier, _keep) = s.ipc.channel(4, Transfer::Free).unwrap();
        *r.lock().unwrap() = Some(s.ipc.send(&carrier, Vec::new(), vec![rx.into_handle()]));
    });
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(result.lock().unwrap().take(), Some(Err(IpcError::Refused)));
}

/// The adversarial-peer fault: a process's next record is refused by the
/// receiving transport; the receiver gets a violation naming the sender,
/// and nothing after.
#[test]
fn a_refused_record_is_a_violation_naming_its_sender() {
    let seen: Seen = Arc::default();
    let got = slot();
    let keep: Slot<Listening> = slot();
    let (seen2, got2, keep2) = (Arc::clone(&seen), Arc::clone(&got), Arc::clone(&keep));
    let p = pair(
        11,
        64,
        move |s, from_child, _to_child| {
            *keep2.lock().unwrap() = Some(record(&s, from_child, &seen2, &got2));
        },
        |s| {
            let boot = s.ipc.bootstrap().unwrap();
            // The world marked this process before it ran: this send is
            // refused at the receiver, though `send` reports success, as
            // a compromised peer's would; the channel is then closed.
            s.ipc
                .send(&boot.to_parent, b"bad".to_vec(), Vec::new())
                .unwrap();
            assert_eq!(
                s.ipc.send(&boot.to_parent, b"late".to_vec(), Vec::new()),
                Err(IpcError::Closed)
            );
        },
    );
    p.world
        .corrupt_next_send(p.child, hg_substrate::ipc::FaultKind::Malformed);
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert!(seen[0].starts_with("violation Malformed Some("), "{seen:?}");
}

/// A bound receiver names the one process whose messages it accepts
/// (IPC-R2): it cannot listen without one, takes its grantee's message,
/// and refuses one from another process, naming the bound peer.
#[test]
fn a_bound_receiver_hears_only_the_peer_it_names() {
    for (named_child, expect) in [(true, "hi"), (false, "violation Identity Some(99)")] {
        let seen: Seen = Arc::default();
        let refused = slot();
        let keep: Slot<Listening> = slot();
        let child_os: Slot<OsPeer> = slot();
        let (seen2, refused2, keep2, cos) = (
            Arc::clone(&seen),
            Arc::clone(&refused),
            Arc::clone(&keep),
            Arc::clone(&child_os),
        );
        let p = pair(
            17,
            8,
            move |s, _from_child, to_child| {
                let nothing = || -> hg_substrate::ipc::Sink { Box::new(|_d| Flow::Continue) };
                let seq = s.exec.sequence("bound", Priority::UserVisible);
                let (tx, rx) = s.ipc.channel(8, Transfer::Bound).unwrap();
                let (_tx2, rx2) = s.ipc.channel(8, Transfer::Bound).unwrap();
                *refused2.lock().unwrap() = Some(s.ipc.listen(rx2, seq, nothing()).err());
                let peer = if named_child {
                    cos.lock().unwrap().unwrap()
                } else {
                    OsPeer::new(99, 0)
                };
                let seen3 = Arc::clone(&seen2);
                let l = s
                    .ipc
                    .listen_bound(
                        rx,
                        PeerAttest::Os(peer),
                        seq,
                        Box::new(move |d| {
                            seen3.lock().unwrap().push(match d {
                                Delivery::Message { bytes, .. } => {
                                    String::from_utf8_lossy(&bytes).into_owned()
                                }
                                Delivery::Closed => "closed".into(),
                                Delivery::Violation(f) => {
                                    format!("violation {:?} {:?}", f.kind, f.sender)
                                }
                            });
                            Flow::Continue
                        }),
                    )
                    .unwrap();
                *keep2.lock().unwrap() = Some(l);
                s.ipc
                    .send(&to_child, b"grant".to_vec(), vec![tx.into_handle()])
                    .unwrap();
            },
            |s| {
                let boot = s.ipc.bootstrap().unwrap();
                let s2 = s.clone();
                let seq = s.exec.sequence("in", Priority::UserVisible);
                let l = s
                    .ipc
                    .listen_bound(
                        boot.from_parent,
                        boot.parent,
                        seq,
                        Box::new(move |d| {
                            if let Delivery::Message { mut handles, .. } = d
                                && let Some(h) = handles.pop()
                                && let Ok(grant) = ChannelSender::from_handle(h)
                            {
                                s2.ipc.send(&grant, b"hi".to_vec(), Vec::new()).unwrap();
                                // Two more, each carrying a handle: refused
                                // with the first when the peer is wrong, so
                                // their handles close with the channel.
                                for _ in 0..2 {
                                    let (tx, _rx) = s2.ipc.channel(1, Transfer::Free).unwrap();
                                    s2.ipc
                                        .send(&grant, b"more".to_vec(), vec![tx.into_handle()])
                                        .unwrap();
                                }
                            }
                            Flow::Continue
                        }),
                    )
                    .unwrap();
                std::mem::forget(l);
            },
        );
        *child_os.lock().unwrap() = p.world.os_peer(p.child);
        assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
        assert_eq!(
            refused.lock().unwrap().take(),
            Some(Some(IpcError::Refused))
        );
        let seen = seen.lock().unwrap();
        assert_eq!(seen.first().map(String::as_str), Some(expect), "{seen:?}");
    }
}

/// Design review M7: `PeerAttest::Inherited` binds the process's own
/// bootstrap receiver and nothing else; bound that way, it hears its
/// launcher.
#[test]
fn inherited_binds_only_the_bootstrap_receiver() {
    let seen: Seen = Arc::default();
    let refused: Slot<Option<IpcError>> = slot();
    let keep: Slot<Listening> = slot();
    let (seen2, refused2, keep2) = (Arc::clone(&seen), Arc::clone(&refused), Arc::clone(&keep));
    let p = pair(
        11,
        8,
        |s, _from_child, to_child| {
            s.ipc
                .send(&to_child, b"hello".to_vec(), Vec::new())
                .unwrap();
            std::mem::forget(to_child);
        },
        move |s| {
            let seq = s.exec.sequence("in", Priority::UserVisible);
            let (_tx, other) = s.ipc.channel(1, Transfer::Free).unwrap();
            *refused2.lock().unwrap() = Some(
                s.ipc
                    .listen_bound(
                        other,
                        PeerAttest::Inherited,
                        seq,
                        Box::new(|_d| Flow::Continue),
                    )
                    .err(),
            );
            let boot = s.ipc.bootstrap().unwrap();
            let seen3 = Arc::clone(&seen2);
            let l = s
                .ipc
                .listen_bound(
                    boot.from_parent,
                    PeerAttest::Inherited,
                    seq,
                    Box::new(move |d| {
                        if let Delivery::Message { bytes, .. } = d {
                            seen3
                                .lock()
                                .unwrap()
                                .push(String::from_utf8_lossy(&bytes).into_owned());
                        }
                        Flow::Continue
                    }),
                )
                .unwrap();
            *keep2.lock().unwrap() = Some(l);
        },
    );
    assert_eq!(p.world.run(LIMITS), RunEnd::Idle);
    assert_eq!(
        refused.lock().unwrap().take(),
        Some(Some(IpcError::Refused))
    );
    assert_eq!(
        seen.lock().unwrap().first().map(String::as_str),
        Some("hello")
    );
}
