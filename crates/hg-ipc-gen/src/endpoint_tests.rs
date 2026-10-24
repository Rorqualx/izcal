//! Typed endpoints end to end, in a simulated world (Stage 1 4.1;
//! IPC-R1, R2, R5; CS-AS-6 (b)).

use std::sync::{Arc, Mutex};

use hg_ipc::endpoint::{EndReason, SendError, Serving};
use hg_ipc::{CallerIdentity, Endpoint, PeerBinding, PeerId, Remote, bound_channel, channel};
use hg_substrate::ipc::{ChannelReceiver, ChannelSender};
use hg_substrate::{Memory, Priority, Substrate};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{Limits, RunEnd, SimConfig, World};
use hg_types::Instant;

use crate::generated::process::{
    BuildId, ProcessControl, ProcessControlHandler, ProcessError, ProcessHost, ProcessHostHandler,
    ProcessHostRemote, ProcessKind,
};
use crate::generated::sample::{Handles, HandlesHandler, Loose, LooseRemote, Strict, StrictRemote};

const LIMITS: Limits = Limits {
    max_steps: 100_000,
    max_time: Instant::from_nanos(86_400_000_000_000),
};

type Log = Arc<Mutex<Vec<String>>>;
type Keep = Arc<Mutex<Vec<Serving>>>;

fn note(log: &Log, s: String) {
    log.lock().unwrap().push(s);
}

/// The broker's number for the child, and the child's for the broker.
const CHILD: PeerId = PeerId::new(1, 0);
const BROKER: PeerId = PeerId::new(0, 0);

fn who(from: &CallerIdentity) -> String {
    format!("{}.{}", from.peer().index(), from.peer().generation())
}

/// A broker and a child linked by launch channels. Each entry gets its
/// substrate and its two ends.
fn world(
    seed: u64,
    broker: impl Fn(Substrate, ChannelReceiver, ChannelSender) + Send + Sync + 'static,
    child: impl Fn(Substrate) + Send + Sync + 'static,
) -> World {
    let w = World::new(SimConfig::new(seed));
    let ends: Arc<Mutex<Option<(ChannelReceiver, ChannelSender)>>> = Arc::default();
    let e = Arc::clone(&ends);
    let backing = || -> Arc<dyn Memory> { Arc::new(OsMemory::new()) };
    let b = w.spawn("broker", backing(), false, move |s| {
        if let Some((rx, tx)) = e.lock().unwrap().take() {
            broker(s, rx, tx);
        }
    });
    let c = w.spawn("child", backing(), false, child);
    *ends.lock().unwrap() = w.link(b, c, 64);
    w
}

/// The broker's side of the process protocol.
struct BrokerSide {
    control: Remote<ProcessControl>,
    log: Log,
    refuse_ready: bool,
}

impl ProcessHostHandler for BrokerSide {
    fn ready(
        &mut self,
        from: &CallerIdentity,
        kind: ProcessKind,
        build: BuildId,
    ) -> Result<(), ProcessError> {
        note(
            &self.log,
            format!("ready {kind:?} {} from {}", build.id, who(from)),
        );
        if self.refuse_ready {
            return Err(ProcessError::Unsupported);
        }
        self.control
            .send(ProcessControl::Ping { nonce: 42 })
            .map_err(|_e| ProcessError::ShuttingDown)
    }

    fn pong(&mut self, from: &CallerIdentity, nonce: u64) -> Result<(), ProcessError> {
        note(&self.log, format!("pong {nonce} from {}", who(from)));
        Ok(())
    }
}

/// The child's side: answers pings.
struct ChildSide {
    host: Remote<ProcessHost>,
}

impl ProcessControlHandler for ChildSide {
    fn ping(&mut self, from: &CallerIdentity, nonce: u64) -> Result<(), ProcessError> {
        if from.peer() != BROKER {
            return Err(ProcessError::Unsupported);
        }
        self.host
            .pong(nonce)
            .map_err(|_e| ProcessError::ShuttingDown)
    }

    fn exit(
        &mut self,
        _from: &CallerIdentity,
        _reason: crate::generated::process::ExitReason,
    ) -> Result<(), ProcessError> {
        Ok(())
    }
}

/// The broker serves ProcessHost on its launch channel, bound to the
/// child; `ends` records how the endpoint ended.
fn broker_serves(
    log: &Log,
    ends: &Log,
    keep: &Keep,
    refuse_ready: bool,
) -> impl Fn(Substrate, ChannelReceiver, ChannelSender) + Send + Sync + 'static {
    let (log, ends, keep) = (Arc::clone(log), Arc::clone(ends), Arc::clone(keep));
    move |s, from_child, to_child| {
        let mut side = BrokerSide {
            control: Remote::from_sender(&s.ipc, to_child),
            log: Arc::clone(&log),
            refuse_ready,
        };
        let seq = s.exec.sequence("host", Priority::UserVisible);
        let ends = Arc::clone(&ends);
        let serving = Endpoint::<ProcessHost>::from_receiver(&s.ipc, from_child)
            .serve_bound(
                seq,
                PeerBinding::new(CHILD),
                move |m, from| m.dispatch(&mut side, from),
                move |why| note(&ends, format!("{why:?}")),
            )
            .unwrap();
        keep.lock().unwrap().push(serving);
    }
}

/// The child announces itself and serves ProcessControl.
fn child_runs(keep: &Keep) -> impl Fn(Substrate) + Send + Sync + 'static {
    let keep = Arc::clone(keep);
    move |s| {
        let boot = s.ipc.bootstrap().unwrap();
        let host = Remote::<ProcessHost>::from_sender(&s.ipc, boot.to_parent);
        host.ready(ProcessKind::Renderer, BuildId { id: 7 })
            .unwrap();
        let mut side = ChildSide { host };
        let seq = s.exec.sequence("control", Priority::UserVisible);
        let serving = Endpoint::<ProcessControl>::from_receiver(&s.ipc, boot.from_parent)
            .serve_bound(
                seq,
                PeerBinding::new(BROKER),
                move |m, from| m.dispatch(&mut side, from),
                |_why| {},
            )
            .unwrap();
        keep.lock().unwrap().push(serving);
    }
}

#[test]
fn broker_and_child_talk_with_the_bound_identity() {
    let (log, ends, keep): (Log, Log, Keep) = Default::default();
    let w = world(
        1,
        broker_serves(&log, &ends, &keep, false),
        child_runs(&keep),
    );
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(
        *log.lock().unwrap(),
        ["ready Renderer 7 from 1.0", "pong 42 from 1.0"]
    );
    assert!(ends.lock().unwrap().is_empty());
}

#[test]
fn a_bad_message_ends_the_endpoint_before_any_handler() {
    let (log, ends, keep): (Log, Log, Keep) = Default::default();
    let child = move |s: Substrate| {
        let boot = s.ipc.bootstrap().unwrap();
        // Ordinal 9: no such message.
        s.ipc.send(&boot.to_parent, vec![9, 0], Vec::new()).unwrap();
        let host = Remote::<ProcessHost>::from_sender(&s.ipc, boot.to_parent);
        // Sent before the broker read the bad one; never handled.
        let _ = host.ready(ProcessKind::Network, BuildId { id: 1 });
    };
    let w = world(2, broker_serves(&log, &ends, &keep, false), child);
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert!(log.lock().unwrap().is_empty());
    assert_eq!(*ends.lock().unwrap(), ["BadMessage(UnknownOrdinal)"]);
}

#[test]
fn a_handler_error_ends_the_endpoint() {
    let (log, ends, keep): (Log, Log, Keep) = Default::default();
    let w = world(
        3,
        broker_serves(&log, &ends, &keep, true),
        child_runs(&keep),
    );
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(*log.lock().unwrap(), ["ready Renderer 7 from 1.0"]);
    assert_eq!(*ends.lock().unwrap(), ["Handler(Unsupported)"]);
}

/// The child's side of the sample's handle-carrying interface.
struct HandleSide {
    s: Substrate,
    log: Log,
    keep: Keep,
}

impl HandlesHandler for HandleSide {
    fn connect(
        &mut self,
        target: Remote<Loose>,
        privileged: hg_ipc::Grant<Strict>,
        inbox: Endpoint<Loose>,
        note_byte: u8,
    ) -> Result<(), crate::generated::sample::SampleError> {
        use crate::generated::sample::SampleError::First;
        note(&self.log, format!("connect note={note_byte}"));
        target.tick(7).map_err(|_e| First)?;
        privileged
            .into_remote(&self.s.ipc)
            .nothing()
            .map_err(|_e| First)?;
        let log = Arc::clone(&self.log);
        let seq = self.s.exec.sequence("inbox", Priority::UserVisible);
        let serving = inbox
            .serve(
                seq,
                move |m| {
                    note(&log, format!("inbox {m:?}"));
                    Ok::<(), ()>(())
                },
                |_why| {},
            )
            .map_err(|_e| First)?;
        self.keep.lock().map_err(|_e| First)?.push(serving);
        Ok(())
    }

    fn hand_over(
        &mut self,
        target: Remote<Loose>,
    ) -> Result<(), crate::generated::sample::SampleError> {
        target
            .tick(1)
            .map_err(|_e| crate::generated::sample::SampleError::First)
    }
}

#[test]
fn every_handle_form_crosses_and_works_in_the_new_process() {
    let (log, keep): (Log, Keep) = Default::default();
    let (log_b, keep_b) = (Arc::clone(&log), Arc::clone(&keep));
    let broker = move |s: Substrate, _from_child: ChannelReceiver, to_child: ChannelSender| {
        let to_child = Remote::<Handles>::from_sender(&s.ipc, to_child);
        let seq = s.exec.sequence("svc", Priority::UserVisible);
        let (target, loose_ep) = channel::<Loose>(&s.ipc).unwrap();
        let (grant, strict_ep) = bound_channel::<Strict>(&s.ipc).unwrap();
        let (inbox_tx, inbox_ep) = channel::<Loose>(&s.ipc).unwrap();
        let (l1, l2) = (Arc::clone(&log_b), Arc::clone(&log_b));
        let mut k = keep_b.lock().unwrap();
        k.push(
            loose_ep
                .serve(
                    seq,
                    move |m| {
                        note(&l1, format!("broker got {m:?}"));
                        Ok::<(), ()>(())
                    },
                    |_| {},
                )
                .unwrap(),
        );
        k.push(
            strict_ep
                .serve_bound(
                    seq,
                    PeerBinding::new(CHILD),
                    move |m, from| {
                        note(&l2, format!("broker got {m:?} from {}", who(from)));
                        Ok::<(), ()>(())
                    },
                    |_| {},
                )
                .unwrap(),
        );
        drop(k);
        to_child
            .send(Handles::Connect {
                target,
                privileged: grant,
                inbox: inbox_ep,
                note: 5,
            })
            .unwrap();
        inbox_tx.tick(9).unwrap();
    };
    let (log_c, keep_c) = (Arc::clone(&log), Arc::clone(&keep));
    let child = move |s: Substrate| {
        let boot = s.ipc.bootstrap().unwrap();
        let mut side = HandleSide {
            s: s.clone(),
            log: Arc::clone(&log_c),
            keep: Arc::clone(&keep_c),
        };
        let seq = s.exec.sequence("handles", Priority::UserVisible);
        let serving = Endpoint::<Handles>::from_receiver(&s.ipc, boot.from_parent)
            .serve(seq, move |m| m.dispatch(&mut side), |_| {})
            .unwrap();
        keep_c.lock().unwrap().push(serving);
    };
    let w = world(4, broker, child);
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    let mut got = log.lock().unwrap().clone();
    got.sort();
    assert_eq!(
        got,
        [
            "broker got Nothing from 1.0",
            "broker got Tick { value: 7 }",
            "connect note=5",
            "inbox Tick { value: 9 }",
        ]
    );
}

#[test]
fn overflow_follows_the_interface_policy() {
    let out: Arc<Mutex<Vec<String>>> = Arc::default();
    let o = Arc::clone(&out);
    let w = World::new(SimConfig::new(5));
    let _p = w.spawn("p", Arc::new(OsMemory::new()), false, move |s| {
        // Loose: capacity 1024, drop on overflow.
        let (loose, _ep) = channel::<Loose>(&s.ipc).unwrap();
        let mut last = Ok(());
        for _ in 0..=1024 {
            last = loose.tick(0);
        }
        o.lock()
            .unwrap()
            .push(format!("{last:?} dropped={}", loose.dropped()));
        // Strict: capacity 8, terminate on overflow.
        let (grant, _ep2) = bound_channel::<Strict>(&s.ipc).unwrap();
        let strict = grant.into_remote(&s.ipc);
        let mut last = Ok(());
        for _ in 0..9 {
            last = strict.nothing();
        }
        o.lock().unwrap().push(format!("{last:?}"));
        assert_eq!(last, Err(SendError::PeerUnresponsive));
    });
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(
        *out.lock().unwrap(),
        ["Err(Dropped) dropped=1", "Err(PeerUnresponsive)"]
    );
    let _ = EndReason::<()>::PeerClosed;
}

#[test]
fn a_bound_sender_in_a_free_slot_is_a_bad_message() {
    let (ends, keep): (Log, Keep) = Default::default();
    let broker = move |s: Substrate, _from_child: ChannelReceiver, to_child: ChannelSender| {
        // `hand_over` (ordinal 1) declares one free `remote<Loose>`; send a
        // bound sender in its place, by hand.
        let (grant, _strict) = bound_channel::<Strict>(&s.ipc).unwrap();
        s.ipc
            .send(&to_child, vec![1, 0], vec![grant.into_handle()])
            .unwrap();
    };
    let (ends_c, keep_c) = (Arc::clone(&ends), Arc::clone(&keep));
    let child = move |s: Substrate| {
        let boot = s.ipc.bootstrap().unwrap();
        let ends = Arc::clone(&ends_c);
        let seq = s.exec.sequence("handles", Priority::UserVisible);
        let serving = Endpoint::<Handles>::from_receiver(&s.ipc, boot.from_parent)
            .serve(
                seq,
                |_m: Handles| Ok::<(), ()>(()),
                move |why| note(&ends, format!("{why:?}")),
            )
            .unwrap();
        keep_c.lock().unwrap().push(serving);
    };
    let w = world(6, broker, child);
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(*ends.lock().unwrap(), ["BadMessage(BadHandles)"]);
}
