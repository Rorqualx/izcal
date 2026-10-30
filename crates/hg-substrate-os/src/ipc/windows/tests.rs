//! The Windows transport's tests: in one process, then across processes
//! with children started (sandboxed, in their own AppContainers) by this
//! test binary acting as their broker.

use hg_substrate::ipc::PeerAttest;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use hg_substrate::ipc::{
    ChannelReceiver, ChannelSender, Delivery, Fault, FaultKind, Flow, Handle, HandleKind, Ipc,
    IpcError, OsPeer, Transfer,
};
use hg_substrate::{Executor, Priority};

use super::WindowsIpc;
use crate::executor::OsExecutor;

const WAIT: Duration = Duration::from_secs(20);

#[derive(Debug)]
enum Ev {
    Msg(Vec<u8>, Vec<Handle>),
    Closed,
    Fault(Fault),
}

struct Rig {
    exec: Arc<OsExecutor>,
    ipc: WindowsIpc,
}

fn rig() -> Rig {
    let exec = Arc::new(OsExecutor::new(2, 1).unwrap());
    let ipc = WindowsIpc::start(Arc::clone(&exec), None).unwrap();
    Rig { exec, ipc }
}

impl Rig {
    fn listen(&self, rx: ChannelReceiver) -> (Receiver<Ev>, hg_substrate::ipc::Listening) {
        let (tx, out) = sync_channel::<Ev>(4096);
        let seq = self.exec.sequence("test", Priority::UserVisible);
        let l = self.ipc.listen(rx, seq, sink(tx)).expect("listen");
        (out, l)
    }

    fn listen_bound(
        &self,
        rx: ChannelReceiver,
        peer: impl Into<PeerAttest>,
    ) -> (Receiver<Ev>, hg_substrate::ipc::Listening) {
        let (tx, out) = sync_channel::<Ev>(4096);
        let seq = self.exec.sequence("test", Priority::UserVisible);
        let l = self
            .ipc
            .listen_bound(rx, peer.into(), seq, sink(tx))
            .expect("listen_bound");
        (out, l)
    }
}

fn sink(tx: SyncSender<Ev>) -> hg_substrate::ipc::Sink {
    let tx = Mutex::new(tx);
    Box::new(move |d| {
        let ev = match d {
            Delivery::Message { bytes, handles } => Ev::Msg(bytes, handles),
            Delivery::Closed => Ev::Closed,
            Delivery::Violation(f) => Ev::Fault(f),
        };
        let _ = tx.lock().unwrap().send(ev);
        Flow::Continue
    })
}

fn next(rx: &Receiver<Ev>) -> Ev {
    rx.recv_timeout(WAIT).expect("a delivery in time")
}

fn expect_msg(rx: &Receiver<Ev>) -> (Vec<u8>, Vec<Handle>) {
    match next(rx) {
        Ev::Msg(b, h) => (b, h),
        other => panic!("expected a message, got {other:?}"),
    }
}

#[test]
fn messages_arrive_in_order_with_their_handles() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(16, Transfer::Free).unwrap();
    let (tx2, rx2) = r.ipc.channel(16, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    for i in 0u8..10 {
        r.ipc.send(&tx, vec![i], Vec::new()).unwrap();
    }
    r.ipc
        .send(
            &tx,
            b"carry".to_vec(),
            vec![rx2.into_handle(), tx2.into_handle()],
        )
        .unwrap();
    for i in 0u8..10 {
        assert_eq!(expect_msg(&out).0, vec![i]);
    }
    let (b, mut hs) = expect_msg(&out);
    assert_eq!(b, b"carry");
    assert_eq!(hs.len(), 2);
    let tx2 = ChannelSender::from_handle(hs.pop().unwrap()).unwrap();
    let rx2 = ChannelReceiver::from_handle(hs.pop().unwrap()).unwrap();
    let (out2, _l2) = r.listen(rx2);
    r.ipc.send(&tx2, b"moved".to_vec(), Vec::new()).unwrap();
    assert_eq!(expect_msg(&out2).0, b"moved");
}

#[test]
fn a_sender_moved_twice_still_delivers() {
    // The second move starts on a pipe the broker made at the first: that
    // pipe must still name the channel when the move is decided (W3).
    let r = rig();
    let (tx, rx) = r.ipc.channel(16, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    let (ctx, crx) = r.ipc.channel(16, Transfer::Free).unwrap();
    let (cout, _cl) = r.listen(crx);
    let mut tx = tx;
    for hop in 0u8..2 {
        r.ipc.send(&tx, vec![hop], Vec::new()).unwrap();
        r.ipc
            .send(&ctx, b"hop".to_vec(), vec![tx.into_handle()])
            .unwrap();
        let (_, mut hs) = expect_msg(&cout);
        tx = ChannelSender::from_handle(hs.remove(0)).unwrap();
    }
    r.ipc.send(&tx, vec![2], Vec::new()).unwrap();
    for i in 0u8..3 {
        assert_eq!(expect_msg(&out).0, vec![i]);
    }
}

#[test]
fn full_is_exact_and_credits_free_slots() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(4, Transfer::Free).unwrap();
    for i in 0u8..4 {
        r.ipc.send(&tx, vec![i], Vec::new()).unwrap();
    }
    assert_eq!(r.ipc.send(&tx, vec![9], Vec::new()), Err(IpcError::Full));
    let (out, _l) = r.listen(rx);
    for i in 0u8..4 {
        assert_eq!(expect_msg(&out).0, vec![i]);
    }
    // Credits arrive asynchronously; within the wait the slot frees.
    let deadline = std::time::Instant::now() + WAIT;
    loop {
        match r.ipc.send(&tx, vec![4], Vec::new()) {
            Ok(()) => break,
            Err(IpcError::Full) if std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(1));
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(expect_msg(&out).0, vec![4]);
}

#[test]
fn a_dropped_sender_closes_after_its_messages() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    r.ipc.send(&tx, b"last".to_vec(), Vec::new()).unwrap();
    drop(tx);
    assert_eq!(expect_msg(&out).0, b"last");
    assert!(matches!(next(&out), Ev::Closed));
}

#[test]
fn a_receiver_listened_to_after_its_sender_closed_ends_in_closed() {
    // The read on a pipe whose writer is gone fails as it starts (109),
    // not through a completion; the listener must still end.
    let r = rig();
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    drop(tx);
    std::thread::sleep(Duration::from_millis(100));
    let (out, _l) = r.listen(rx);
    assert!(matches!(next(&out), Ev::Closed));
}

#[test]
fn a_dropped_receiver_closes_the_sender() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    drop(rx);
    // Closed is eventual: the first sends may be accepted.
    let deadline = std::time::Instant::now() + WAIT;
    loop {
        match r.ipc.send(&tx, b"x".to_vec(), Vec::new()) {
            Err(IpcError::Closed) => break,
            Ok(()) | Err(IpcError::Full) if std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(1));
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(
        r.ipc.send(&tx, b"x".to_vec(), Vec::new()),
        Err(IpcError::Closed)
    );
}

#[test]
fn bound_receivers_listen_only_with_their_peer_named() {
    let r = rig();
    let seq = || r.exec.sequence("test", Priority::UserVisible);
    let nothing = || -> hg_substrate::ipc::Sink { Box::new(|_d| Flow::Continue) };
    let (_btx, brx) = r.ipc.channel(8, Transfer::Bound).unwrap();
    assert_eq!(
        r.ipc.listen(brx, seq(), nothing()).err(),
        Some(IpcError::Refused)
    );
    let (btx, brx) = r.ipc.channel(8, Transfer::Bound).unwrap();
    assert_eq!(btx.handle().kind(), HandleKind::BoundChannelSender);
    let (out, _l) = r.listen_bound(brx, OsPeer::new(std::process::id(), 0));
    r.ipc.send(&btx, b"own".to_vec(), Vec::new()).unwrap();
    assert_eq!(expect_msg(&out).0, b"own");
    // A bound receiver never moves; a listening one neither.
    let (carrier, _crx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (_b2, brx2) = r.ipc.channel(8, Transfer::Bound).unwrap();
    assert_eq!(
        r.ipc.send(&carrier, Vec::new(), vec![brx2.into_handle()]),
        Err(IpcError::Refused)
    );
}

// ---- across processes -------------------------------------------------

/// The child side, run by this test binary started again with
/// `IZCAL_D5_CHILD=<case>`; a no-op in a normal run.
#[test]
fn d5_child() {
    let Some(case) = std::env::var_os("IZCAL_D5_CHILD") else {
        return;
    };
    let case = case.into_string().unwrap();
    std::process::exit(child(&case));
}

fn child(case: &str) -> i32 {
    // A process that only lives (it holds whatever it inherited): ended
    // by its test.
    if case == "linger" {
        std::thread::sleep(Duration::from_secs(120));
        return 0;
    }
    if case.starts_with("sandboxed-") {
        crate::sandbox::windows::apply_empty_process_profile().unwrap();
        // Really in its own container: the launch is what the test
        // claims.
        if super::sys::app_container_sid().is_none() {
            return 15;
        }
    } else if super::sys::app_container_sid().is_some() {
        return 16;
    }
    let Some(link) = super::take_link() else {
        return 10;
    };
    let exec = Arc::new(OsExecutor::new(2, 1).unwrap());
    let Ok(ipc) = WindowsIpc::start(Arc::clone(&exec), Some(link)) else {
        return 11;
    };
    let r = Rig { exec, ipc };
    let Some(boot) = r.ipc.bootstrap() else {
        return 12;
    };
    let up = boot.to_parent;
    let (from_parent, _l) = r.listen_bound(boot.from_parent, boot.parent);
    match case.trim_start_matches("sandboxed-") {
        "burst-exit" => {
            for i in 0u8..5 {
                r.ipc.send(&up, vec![i], Vec::new()).unwrap();
            }
            // Give the IO thread time to write them, then end abruptly.
            std::thread::sleep(Duration::from_millis(200));
            0
        }
        "echo" => loop {
            match from_parent.recv_timeout(WAIT) {
                Ok(Ev::Msg(b, hs)) => {
                    let mut back = Vec::new();
                    for h in hs {
                        match ChannelSender::from_handle(h) {
                            Ok(s) => {
                                r.ipc.send(&s, b"via-child".to_vec(), Vec::new()).unwrap();
                            }
                            Err(h) => back.push(h),
                        }
                    }
                    r.ipc.send(&up, b, back).unwrap();
                }
                _ => return 0,
            }
        },
        "make-channel" => {
            // A channel made here: its receiver goes up, then a message
            // on its sender.
            let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
            r.ipc
                .send(&up, b"chan".to_vec(), vec![rx.into_handle()])
                .unwrap();
            r.ipc.send(&tx, b"inside".to_vec(), Vec::new()).unwrap();
            let _ = from_parent.recv_timeout(WAIT);
            0
        }
        "use-grant" => {
            let Ok(Ev::Msg(_, mut hs)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            let grant = ChannelSender::from_handle(hs.remove(0)).unwrap();
            assert_eq!(grant.handle().kind(), HandleKind::BoundChannelSender);
            r.ipc
                .send(&grant, b"from-child".to_vec(), Vec::new())
                .unwrap();
            r.ipc.send(&up, b"done".to_vec(), Vec::new()).unwrap();
            let _ = from_parent.recv_timeout(WAIT);
            0
        }
        "forward-grant" => {
            // Its own transport refuses to pass a bound sender on.
            let Ok(Ev::Msg(_, mut hs)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            let grant = hs.remove(0);
            let (carrier, _crx) = r.ipc.channel(8, Transfer::Free).unwrap();
            let res = r.ipc.send(&carrier, Vec::new(), vec![grant]);
            let report = format!("{res:?}");
            r.ipc.send(&up, report.into_bytes(), Vec::new()).unwrap();
            let _ = from_parent.recv_timeout(WAIT);
            0
        }
        "slow-reader" => {
            // Receives a receiver end, then reads it slowly so the
            // parent's writes back up, counting messages until a 2 s
            // idle gap, then reports the count.
            let Ok(Ev::Msg(_, mut hs)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            let Ok(rx2) = ChannelReceiver::from_handle(hs.remove(0)) else {
                return 17;
            };
            let (out, _l) = r.listen(rx2);
            let mut count = 0u32;
            while let Ok(Ev::Msg(..)) = out.recv_timeout(Duration::from_secs(2)) {
                count = count.saturating_add(1);
                std::thread::sleep(Duration::from_millis(30));
            }
            r.ipc
                .send(&up, count.to_le_bytes().to_vec(), Vec::new())
                .unwrap();
            let _ = from_parent.recv_timeout(WAIT);
            0
        }
        "take-and-drop" => {
            // Receive a handle, drop it at once, acknowledge.
            loop {
                match from_parent.recv_timeout(WAIT) {
                    Ok(Ev::Msg(_, hs)) => {
                        drop(hs);
                        r.ipc.send(&up, b"dropped".to_vec(), Vec::new()).unwrap();
                    }
                    _ => return 0,
                }
            }
        }
        "stuck-relay" => {
            // Names a synchronous pipe end with a read pending in a relay
            // record, so the broker's queries for it wait (probe W8)
            // until this process exits. Then a message carrying an end of
            // a real channel: its relay is queued behind the first one.
            let Ok((server, _client)) = super::sys::sync_pipe_with_read_pending() else {
                return 18;
            };
            {
                let mut st = r.ipc.inner.lock();
                let req = st.req();
                let Some(items) = super::Items::from_slice(&[]) else {
                    return 19;
                };
                let relay = super::Ctl::Relay {
                    req,
                    carrier: super::sys::value(&server),
                    items,
                };
                r.ipc.inner.send_ctl(&mut st, None, &relay);
            }
            r.ipc.inner.wake();
            r.ipc.send(&up, b"stuck".to_vec(), Vec::new()).unwrap();
            let (_tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
            r.ipc
                .send(&up, b"behind".to_vec(), vec![rx.into_handle()])
                .unwrap();
            let _ = from_parent.recv_timeout(WAIT);
            drop(server);
            0
        }
        "bind-unbind" => {
            // Fifty bound listeners on channels made here, each dropped at
            // once (a Bind, then an Unbind and a Closed), then a message
            // carrying an end, whose relay is queued behind all of them.
            // Then one more bound listener, kept, and another such
            // message.
            let mut senders = Vec::new();
            for _ in 0..50 {
                let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
                let (_o, l) = r.listen_bound(rx, boot.parent);
                senders.push(tx);
                drop(l);
            }
            let (_t1, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
            r.ipc
                .send(&up, b"unbound".to_vec(), vec![rx.into_handle()])
                .unwrap();
            let Ok(Ev::Msg(..)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
            let (_o, kept) = r.listen_bound(rx, boot.parent);
            let (_t2, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
            r.ipc
                .send(&up, b"bound".to_vec(), vec![rx.into_handle()])
                .unwrap();
            let _ = from_parent.recv_timeout(WAIT);
            drop((tx, kept, senders));
            0
        }
        "region-roundtrip" | "region-forge" | "region-big" | "region-big-exit" => {
            region_child(case.trim_start_matches("sandboxed-"), &r, &up, &from_parent)
        }
        "stale-reader" | "stale-split" | "stale-whole" => {
            // A compromised holder (W3, probes W10, W13): it receives a
            // receiver end, leaves a read pending on it behind its
            // transport's back, and gives the end away. "stale-reader"
            // gives it away at once; the others first say "armed" and
            // wait until the read has taken something (the head of a
            // record, or a whole one). Then it reports what the read got.
            let Ok(Ev::Msg(_, mut hs)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            let rx = hs.remove(0);
            let cap = match case.trim_start_matches("sandboxed-") {
                // The ticket and the record header of the next message.
                "stale-split" => 32,
                _ => 128 * 1024,
            };
            let stale = super::sys::stale_read(raw_value(&r.ipc, &rx), cap, 8_000);
            let early = if case.ends_with("stale-reader") {
                None
            } else {
                r.ipc.send(&up, b"armed".to_vec(), Vec::new()).unwrap();
                Some(stale.recv_timeout(WAIT))
            };
            r.ipc.send(&up, b"back".to_vec(), vec![rx]).unwrap();
            let (status, got) = match early {
                Some(e) => e.unwrap_or((usize::MAX, Vec::new())),
                None => stale.recv_timeout(WAIT).unwrap_or((usize::MAX, Vec::new())),
            };
            let mut report = format!("{status:#x}:").into_bytes();
            report.extend(got);
            r.ipc.send(&up, report, Vec::new()).unwrap();
            let _ = from_parent.recv_timeout(WAIT);
            0
        }
        "bounce" => loop {
            // Every handle goes straight back up, unused, with its
            // message's bytes.
            match from_parent.recv_timeout(WAIT) {
                Ok(Ev::Msg(b, hs)) => r.ipc.send(&up, b, hs).unwrap(),
                _ => return 0,
            }
        },
        "bounce-on-go" => {
            // Holds the handles it is given until "go", then sends them
            // all back up: two such children moving the two ends of one
            // channel at once (W3).
            let mut held = Vec::new();
            loop {
                match from_parent.recv_timeout(WAIT) {
                    Ok(Ev::Msg(b, _)) if b == b"go" => {
                        r.ipc
                            .send(&up, b"back".to_vec(), core::mem::take(&mut held))
                            .unwrap();
                    }
                    Ok(Ev::Msg(_, hs)) => held.extend(hs),
                    _ => return 0,
                }
            }
        }
        "send-when-free" => {
            // Receives a full sender and sends once a credit frees a slot.
            let Ok(Ev::Msg(_, mut hs)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            let tx = ChannelSender::from_handle(hs.remove(0)).unwrap();
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            let report = loop {
                match r.ipc.send(&tx, b"b".to_vec(), Vec::new()) {
                    Ok(()) => break "sent",
                    Err(IpcError::Full) if std::time::Instant::now() < deadline => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break "stuck",
                }
            };
            r.ipc
                .send(&up, report.as_bytes().to_vec(), Vec::new())
                .unwrap();
            let _ = from_parent.recv_timeout(WAIT);
            0
        }
        "take-and-hold" => {
            // Keeps what it is given, unlistened, until it is killed.
            let Ok(Ev::Msg(_, hs)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            r.ipc.send(&up, b"held".to_vec(), Vec::new()).unwrap();
            let _ = from_parent.recv_timeout(Duration::from_secs(60));
            drop(hs);
            0
        }
        "deaf" => {
            // Takes a sender end and then ignores every rebind.
            let Ok(Ev::Msg(_, hs)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            r.ipc.inner.lock().deaf = true;
            r.ipc.send(&up, b"deaf".to_vec(), Vec::new()).unwrap();
            let _ = from_parent.recv_timeout(Duration::from_secs(60));
            drop(hs);
            0
        }
        "keep" => {
            // Keeps every handle it is given, unused, and acknowledges each.
            let mut kept = Vec::new();
            while let Ok(Ev::Msg(_, hs)) = from_parent.recv_timeout(WAIT) {
                kept.extend(hs);
                r.ipc.send(&up, b"kept".to_vec(), Vec::new()).unwrap();
            }
            drop(kept);
            0
        }
        "deaf-holder" => {
            // Takes an end and ignores every rebind; on "go" sends the end
            // back up.
            let Ok(Ev::Msg(_, hs)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            r.ipc.inner.lock().deaf = true;
            r.ipc.send(&up, b"deaf".to_vec(), Vec::new()).unwrap();
            let Ok(Ev::Msg(..)) = from_parent.recv_timeout(Duration::from_secs(60)) else {
                return 13;
            };
            r.ipc.send(&up, b"back".to_vec(), hs).unwrap();
            let _ = from_parent.recv_timeout(Duration::from_secs(60));
            0
        }
        "swap-sync" => {
            // Names a copy of a sender end it holds in a relay, then, while
            // the broker waits for the peer's rebind, puts a synchronous
            // pipe end with a read pending at that value: an end whose
            // kernel queries wait until this process exits (probe W8).
            let Ok(Ev::Msg(_, mut hs)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            let tx = ChannelSender::from_handle(hs.remove(0)).unwrap();
            r.ipc.send(&up, b"have".to_vec(), Vec::new()).unwrap();
            let Ok(Ev::Msg(..)) = from_parent.recv_timeout(WAIT) else {
                return 13;
            };
            let value_of = |h: &Handle| {
                let id = r.ipc.held_end(h).unwrap();
                let st = r.ipc.inner.lock();
                super::sys::duplicate(&st.ends.get(&id).unwrap().h).unwrap()
            };
            let copy = value_of(tx.handle());
            let named = super::sys::value(&copy);
            let carrier = value_of(up.handle());
            {
                let mut st = r.ipc.inner.lock();
                let req = st.req();
                let Some(items) = super::Items::from_slice(&[super::CtlItem::end(
                    named,
                    super::WireKind::Sender,
                )]) else {
                    return 19;
                };
                let relay = super::Ctl::Relay {
                    req,
                    carrier: super::sys::value(&carrier),
                    items,
                };
                r.ipc.inner.send_ctl(&mut st, None, &relay);
            }
            r.ipc.inner.wake();
            std::thread::sleep(Duration::from_millis(500));
            let Ok((sync, _client)) = super::sys::sync_pipe_with_read_pending() else {
                return 18;
            };
            drop(copy);
            let mut extra = Vec::new();
            let mut swapped = false;
            for _ in 0..256 {
                let Ok(h) = super::sys::duplicate(&sync) else {
                    break;
                };
                if super::sys::value(&h) == named {
                    extra.push(h);
                    swapped = true;
                    break;
                }
                extra.push(h);
            }
            let report: &[u8] = if swapped { b"swapped" } else { b"noswap" };
            r.ipc.send(&up, report.to_vec(), Vec::new()).unwrap();
            let _ = from_parent.recv_timeout(Duration::from_secs(60));
            drop((extra, sync, carrier, tx));
            0
        }
        "sync-relay" => {
            // Names an idle synchronous pipe end in a relay.
            let Ok((sync, _client)) = super::sys::sync_pipe() else {
                return 18;
            };
            let up_id = r.ipc.held_end(up.handle()).unwrap();
            {
                let mut st = r.ipc.inner.lock();
                let req = st.req();
                let Some(items) = super::Items::from_slice(&[super::CtlItem::end(
                    super::sys::value(&sync),
                    super::WireKind::Receiver,
                )]) else {
                    return 19;
                };
                let carrier = super::sys::value(&st.ends.get(&up_id).unwrap().h);
                let relay = super::Ctl::Relay {
                    req,
                    carrier,
                    items,
                };
                r.ipc.inner.send_ctl(&mut st, None, &relay);
            }
            r.ipc.inner.wake();
            r.ipc.send(&up, b"sent".to_vec(), Vec::new()).unwrap();
            let _ = from_parent.recv_timeout(WAIT);
            drop(sync);
            0
        }
        "time-take" => {
            // For the measurement: receives receivers whose channel has an
            // old pipe queued, and times each listen's TakeDrains round
            // trip (until the old pipes are in), in microseconds.
            let mut times = Vec::new();
            while let Ok(Ev::Msg(b, mut hs)) = from_parent.recv_timeout(WAIT) {
                if b == b"done" {
                    break;
                }
                let rx = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
                let id = rx
                    .handle()
                    .object()
                    .downcast_ref::<super::WinEnd>()
                    .unwrap()
                    .id;
                let t = std::time::Instant::now();
                let (_o, l) = r.listen(rx);
                loop {
                    let st = r.ipc.inner.lock();
                    let taking = matches!(
                        st.ends.get(&id).map(|e| &e.role),
                        Some(super::Role::Receiver { listener: Some(l), .. }) if l.taking.is_some()
                    );
                    drop(st);
                    if !taking {
                        break;
                    }
                    std::thread::yield_now();
                }
                times.push(u64::try_from(t.elapsed().as_micros()).unwrap_or(u64::MAX));
                drop(l);
                r.ipc.send(&up, b"next".to_vec(), Vec::new()).unwrap();
            }
            let bytes: Vec<u8> = times.iter().flat_map(|t| t.to_le_bytes()).collect();
            r.ipc.send(&up, bytes, Vec::new()).unwrap();
            let _ = from_parent.recv_timeout(WAIT);
            0
        }
        _ => 14,
    }
}

/// The handle value of a channel end held by `ipc` (W3 tests: what a
/// compromised holder would name to leave a read pending).
fn raw_value(ipc: &WindowsIpc, h: &Handle) -> u64 {
    let id = h.object().downcast_ref::<super::WinEnd>().unwrap().id;
    let st = ipc.inner.lock();
    super::sys::value(&st.ends.get(&id).unwrap().h)
}

/// A child started by this process; its container is removed on drop.
struct Kid {
    child: super::sys::spawn::Child,
    peer: OsPeer,
    to_child: ChannelSender,
    from_child: Receiver<Ev>,
    _l: hg_substrate::ipc::Listening,
    container: Option<String>,
}

impl Drop for Kid {
    fn drop(&mut self) {
        super::sys::spawn::kill(&self.child);
        let _ = super::sys::spawn::wait(&self.child, 5000);
        if let Some(c) = &self.container {
            super::sys::spawn::delete_container(c);
        }
    }
}

impl Kid {
    fn exit_code(&self) -> Option<u32> {
        super::sys::spawn::wait(&self.child, 20_000)
    }
}

fn launch(r: &Rig, case: &str, sandboxed: bool) -> Kid {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let exe = std::env::current_exe().unwrap();
    let case = if sandboxed {
        format!("sandboxed-{case}")
    } else {
        case.to_owned()
    };
    let env = vec![("IZCAL_D5_CHILD".to_owned(), case)];
    let args = "--exact ipc::windows::tests::d5_child --nocapture --test-threads 1";
    let mut container = None;
    let launched = if sandboxed {
        let name = format!(
            "izcal.d5test.{}.{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        );
        let sid = crate::sandbox::windows::app_container(&name, &name).unwrap();
        let token = crate::sandbox::windows::restricted_token().unwrap();
        let job = crate::sandbox::windows::job_object().unwrap();
        container = Some(name);
        let l = r
            .ipc
            .launch(super::LaunchSpec {
                exe: &exe,
                args,
                env,
                sandbox: super::Sandbox::Contained {
                    container: &sid,
                    token: &token,
                    job: &job,
                },
            })
            .unwrap();
        // Closing the job would end the child at once: it stays open for
        // the test process's life (kill-on-close covers a crashed test).
        std::mem::forget(job);
        l
    } else {
        r.ipc
            .launch(super::LaunchSpec {
                exe: &exe,
                args,
                env,
                sandbox: super::Sandbox::Plain,
            })
            .unwrap()
    };
    let super::Launched {
        child,
        from_child,
        to_child,
        peer,
    } = launched;
    let (from_child, l) = r.listen_bound(from_child, peer);
    Kid {
        child,
        peer,
        to_child,
        from_child,
        _l: l,
        container,
    }
}

#[test]
fn two_children_cannot_share_one_app_container_sid() {
    let r = rig();
    let exe = std::env::current_exe().unwrap();
    let name = format!("izcal.d5test.dup.{}", std::process::id());
    let sid = crate::sandbox::windows::app_container(&name, &name).unwrap();
    let token = crate::sandbox::windows::restricted_token().unwrap();
    let job = crate::sandbox::windows::job_object().unwrap();
    let args = "--exact ipc::windows::tests::d5_child --nocapture --test-threads 1";
    let spec = || super::LaunchSpec {
        exe: &exe,
        args,
        env: vec![("IZCAL_D5_CHILD".to_owned(), "sandboxed-echo".to_owned())],
        sandbox: super::Sandbox::Contained {
            container: &sid,
            token: &token,
            job: &job,
        },
    };
    let first = r.ipc.launch(spec()).expect("first launch");
    // The second launch names the same container: refused.
    let second = r.ipc.launch(spec());
    assert!(second.is_err(), "a second child reused the SID");
    super::sys::spawn::kill(&first.child);
    let _ = super::sys::spawn::wait(&first.child, 5000);
    super::sys::spawn::delete_container(&name);
}

#[test]
fn a_dead_childs_messages_all_arrive_then_closed() {
    let r = rig();
    let c = launch(&r, "burst-exit", false);
    for i in 0u8..5 {
        assert_eq!(expect_msg(&c.from_child).0, vec![i]);
    }
    assert!(matches!(next(&c.from_child), Ev::Closed));
    assert_eq!(c.exit_code(), Some(0));
}

#[test]
fn a_handle_value_of_minus_one_is_not_adopted() {
    assert!(super::sys::adopt_pipe(u64::MAX, true).is_none());
    assert!(super::sys::adopt_pipe(u64::MAX, false).is_none());
}

#[test]
fn a_pipe_end_of_the_wrong_kind_is_not_adopted_and_stays_open() {
    use super::sys;
    let (server, client) = sys::pipe_pair(None, &[0x5a; 16]).unwrap();
    // The server end, where a client end is expected: refused, and the
    // handle is left alone (still a pipe's server end).
    assert!(sys::adopt_pipe(sys::value(&server), false).is_none());
    let f = sys::facts(&server);
    assert!(f.is_pipe && f.server_end);
    // The right kind is adopted: the adopted handle names the same end,
    // so the original is released rather than closed twice.
    let v = sys::value(&client);
    let adopted = sys::adopt_pipe(v, false).unwrap();
    assert_eq!(sys::value(&adopted), v);
    sys::forget(client);
    drop(adopted);
}

fn round_trips(sandboxed: bool) {
    let r = rig();
    let c = launch(&r, "echo", sandboxed);
    for i in 0u8..20 {
        r.ipc.send(&c.to_child, vec![i; 100], Vec::new()).unwrap();
        assert_eq!(expect_msg(&c.from_child).0, vec![i; 100]);
    }
    // A sender end moves to the child through a ticket; the child sends
    // on it, and our receiver hears it.
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    r.ipc
        .send(&c.to_child, b"take".to_vec(), vec![tx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&out).0, b"via-child");
    assert_eq!(expect_msg(&c.from_child).0, b"take");
    // A receiver end goes there and comes back.
    let (tx2, rx2) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&c.to_child, b"round".to_vec(), vec![rx2.into_handle()])
        .unwrap();
    let (b, mut hs) = expect_msg(&c.from_child);
    assert_eq!(b, b"round");
    let rx2 = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
    let (out2, _l2) = r.listen(rx2);
    r.ipc.send(&tx2, b"home".to_vec(), Vec::new()).unwrap();
    assert_eq!(expect_msg(&out2).0, b"home");
    let to = c.to_child.handle().kind();
    assert_eq!(to, HandleKind::ChannelSender);
}

#[test]
fn messages_and_handles_cross_processes() {
    round_trips(false);
}

#[test]
fn moving_a_sender_waits_for_a_remote_slow_readers_drain() {
    // A deterministic version of the above: the receiver is held by a
    // separate, slow child, so the parent's writes genuinely back up in
    // the pipe. Moving the sender must wait for that backlog to drain
    // rather than cancel the pending write (item 4). The slow reader
    // must see every one of the queued messages plus the echo child's
    // one message.
    let r = rig();
    let reader = launch(&r, "slow-reader", false);
    let echoer = launch(&r, "echo", false);
    let (tx, rx) = r.ipc.channel(256, Transfer::Free).unwrap();
    r.ipc
        .send(&reader.to_child, b"rx".to_vec(), vec![rx.into_handle()])
        .unwrap();
    // 120 messages of 2 KiB is 240 KiB, well past the 128 KiB pipe
    // buffer, so writes pend and one is in flight at the move.
    const N: u32 = 120;
    for i in 0..N {
        r.ipc.send(&tx, vec![i as u8; 2000], Vec::new()).unwrap();
    }
    r.ipc
        .send(&echoer.to_child, b"take".to_vec(), vec![tx.into_handle()])
        .unwrap();
    let (b, _) = expect_msg(&reader.from_child);
    let count = u32::from_le_bytes(b.try_into().expect("a u32 count"));
    assert_eq!(count, N + 1, "the slow reader lost messages at the move");
}

#[test]
fn messages_and_handles_cross_into_the_sandbox() {
    round_trips(true);
}

#[test]
fn a_channel_made_in_a_sandboxed_child_works_here() {
    let r = rig();
    let c = launch(&r, "make-channel", true);
    let (b, mut hs) = expect_msg(&c.from_child);
    assert_eq!(b, b"chan");
    let rx = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
    let (out, _l) = r.listen(rx);
    assert_eq!(expect_msg(&out).0, b"inside");
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
    assert_eq!(c.exit_code(), Some(0), "the child ran in its own container");
}

#[test]
fn a_bound_receiver_hears_its_grantee() {
    let r = rig();
    let c = launch(&r, "use-grant", true);
    let (btx, brx) = r.ipc.channel(8, Transfer::Bound).unwrap();
    let (out, _l) = r.listen_bound(brx, c.peer);
    r.ipc
        .send(&c.to_child, b"grant".to_vec(), vec![btx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&out).0, b"from-child");
    assert_eq!(expect_msg(&c.from_child).0, b"done");
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
    assert_eq!(c.exit_code(), Some(0));
}

#[test]
fn a_grantee_cannot_pass_its_bound_sender_on() {
    let r = rig();
    let c = launch(&r, "forward-grant", true);
    let (btx, brx) = r.ipc.channel(8, Transfer::Bound).unwrap();
    let (_out, _l) = r.listen_bound(brx, c.peer);
    r.ipc
        .send(&c.to_child, b"grant".to_vec(), vec![btx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&c.from_child).0, b"Err(Refused)");
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
}

#[test]
fn a_bound_receiver_bound_to_another_process_reports_identity() {
    // Bound to a process that will never hold the sender: granting it to
    // the child breaks the binding, and the fault names the bound peer.
    let r = rig();
    let c = launch(&r, "use-grant", false);
    let (btx, brx) = r.ipc.channel(8, Transfer::Bound).unwrap();
    let other = OsPeer::new(4, 0);
    let (out, _l) = r.listen_bound(brx, other);
    r.ipc
        .send(&c.to_child, b"grant".to_vec(), vec![btx.into_handle()])
        .unwrap();
    match next(&out) {
        Ev::Fault(f) => {
            assert_eq!(f.kind, FaultKind::Identity);
            assert_eq!(f.sender, Some(4));
        }
        other => panic!("expected an identity fault, got {other:?}"),
    }
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
}

/// The D5 measurement: round-trip latency of a 64-byte message to a
/// child process and back (p50, p99); the same with a receiver end
/// carried each way, which costs two broker relays and two redemptions
/// per trip; and throughput of 64 KiB messages echoed. Run under an
/// exclusive booking with `--ignored --nocapture`; prints the trips
/// over 1 ms, then one line of results.
#[test]
#[ignore = "a measurement, run by hand on the lab machine"]
fn measure_round_trip_and_throughput() {
    let r = rig();
    let c = launch(&r, "echo", false);
    // One untimed round trip first: `launch` returns before the child has
    // reached its echo loop, so the first message also waited out the
    // child's start-up (about 11-13 ms on rig253) and was the measured
    // max. Measured steady state starts once the child has answered.
    r.ipc.send(&c.to_child, vec![0; 64], Vec::new()).unwrap();
    let _ = expect_msg(&c.from_child);
    let pct = |lat: &mut Vec<u128>, q: usize| {
        lat.sort_unstable();
        lat[lat.len() * q / 100] as f64 / 1000.0
    };
    let mut lat = Vec::with_capacity(10_000);
    for i in 0..10_000u32 {
        let t = std::time::Instant::now();
        r.ipc
            .send(&c.to_child, i.to_le_bytes().repeat(16), Vec::new())
            .unwrap();
        let _ = expect_msg(&c.from_child);
        lat.push(t.elapsed().as_nanos());
    }
    // Which trips are slow, and when: the index and microseconds of every
    // trip over 1 ms, before sorting.
    let slow: Vec<String> = lat
        .iter()
        .enumerate()
        .filter(|(_, ns)| **ns > 1_000_000)
        .map(|(i, ns)| format!("#{i}={}us", ns / 1000))
        .collect();
    println!("D5-SLOW {} over 1ms: {}", slow.len(), slow.join(" "));
    let (p50, p99) = (pct(&mut lat, 50), pct(&mut lat, 99));
    let max = *lat.last().unwrap() as f64 / 1000.0;

    let mut moved = Vec::with_capacity(1_000);
    for _ in 0..1_000u32 {
        let (_tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
        let t = std::time::Instant::now();
        r.ipc
            .send(&c.to_child, vec![7; 64], vec![rx.into_handle()])
            .unwrap();
        let (_b, hs) = expect_msg(&c.from_child);
        moved.push(t.elapsed().as_nanos());
        assert_eq!(hs.len(), 1);
    }
    let (m50, m99) = (pct(&mut moved, 50), pct(&mut moved, 99));

    let big = vec![5u8; hg_substrate::ipc::MAX_MESSAGE_BYTES];
    let n = 2_000u32;
    let t = std::time::Instant::now();
    let (mut inflight, mut done, mut sent) = (0u32, 0u32, 0u32);
    while done < n {
        while sent < n && inflight < 32 {
            r.ipc.send(&c.to_child, big.clone(), Vec::new()).unwrap();
            sent += 1;
            inflight += 1;
        }
        let _ = expect_msg(&c.from_child);
        done += 1;
        inflight -= 1;
    }
    let secs = t.elapsed().as_secs_f64();
    let mib = f64::from(n) * big.len() as f64 * 2.0 / (1024.0 * 1024.0) / secs;
    println!(
        "D5-MEASURE rtt64B p50={p50:.1}us p99={p99:.1}us max={max:.1}us | with a receiver end each way p50={m50:.1}us p99={m99:.1}us | 64KiB echo {mib:.0} MiB/s (both ways) over {n} msgs"
    );
}

/// Item 5: the broker keeps a channel's creator key after the creator
/// dies, so a channel a dead child made is still resolved for the live
/// processes that hold its ends.
#[test]
fn a_dead_creators_channel_is_still_keyed() {
    use super::{BROKER, Broker, Inner};
    use crate::ipc::win::ledger::Ledger;

    let sid = "S-1-15-2-1-2-3-4-5-6-7";
    let name = format!(
        r"\Sessions\0\AppContainerNamedObjects\{sid}\izcal-{}",
        "a".repeat(32)
    );
    let facts = super::sys::RawFacts {
        is_pipe: true,
        name: Some(name),
        ..Default::default()
    };
    let mut b = Broker::new(Ledger::default(), 9);
    // Child 7 made the channel, then died: it is gone from `children`,
    // but its SID is still mapped because the ledger holds its channel.
    b.creator_sids.insert(sid.to_owned(), 7);
    let key = Inner::key_of(&b, &facts).expect("the dead creator's channel is still keyed");
    assert_eq!(key.creator, 7);
    assert_ne!(key.creator, BROKER);
}

/// Items 2, 3: Windows reuses a handle value once its end is closed, so a
/// redeem can hand a child a value that an end awaiting its `Closed` still
/// occupies. The control link delivers the child's records in order and a
/// value is reused only after the child closed the end that held it, so
/// the broker releases grants at a shared value oldest first and a
/// `Closed` never frees the wrong channel's end.
#[test]
fn a_reused_handle_value_releases_grants_oldest_first() {
    use super::{Broker, PipeEnd, PipeKey};
    use crate::ipc::win::ledger::Ledger;

    let mut b = Broker::new(Ledger::default(), 9);
    let who = 7;
    let value = 0x40;
    let first = (
        PipeKey {
            creator: who,
            id: [1u8; 16],
        },
        PipeEnd::Sender,
    );
    let second = (
        PipeKey {
            creator: who,
            id: [2u8; 16],
        },
        PipeEnd::Receiver,
    );
    // The older end is granted at the value; the newer end reuses the same
    // value after the older was closed but before its `Closed` is read.
    b.grant(who, value, first.0, first.1);
    b.grant(who, value, second.0, second.1);
    // Each `Closed` for the value frees the oldest outstanding grant.
    assert_eq!(b.take_grant(who, value), Some(first));
    assert_eq!(b.take_grant(who, value), Some(second));
    assert_eq!(b.take_grant(who, value), None);
}

/// Item 1 (M8): a child's requests are handled by its own worker, so one
/// whose kernel queries wait (a synchronous pipe end with a read pending,
/// probe W8) delays only that child. Another child's relays and
/// redemptions complete meanwhile, and the waiting child's later relay
/// stays queued behind its first.
#[test]
fn a_childs_waiting_request_delays_no_other_child() {
    let r = rig();
    let stuck = launch(&r, "stuck-relay", false);
    assert_eq!(expect_msg(&stuck.from_child).0, b"stuck");
    let other = launch(&r, "echo", false);
    for i in 0u8..20 {
        let (_tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
        r.ipc
            .send(&other.to_child, vec![i], vec![rx.into_handle()])
            .unwrap();
        let (b, hs) = expect_msg(&other.from_child);
        assert_eq!(b, vec![i]);
        assert_eq!(hs.len(), 1);
    }
    // The first child's second relay is still waiting behind its first.
    assert!(!matches!(
        stuck.from_child.recv_timeout(Duration::from_millis(500)),
        Ok(Ev::Msg(..))
    ));
}

/// Item 1 (M8): a child's Binds and Unbinds are handled in the order it
/// sent them, so an Unbind is never overtaken by the Bind it undoes and no
/// binding outlives its listener. After fifty bound listeners the child
/// dropped at once, the broker holds only the binding of the child's own
/// bootstrap listener; after one more, kept, it holds two. Each count is
/// read after a relay the child sent last, which its worker handles after
/// everything before it.
#[test]
fn a_childs_binds_and_unbinds_are_handled_in_order() {
    let r = rig();
    let c = launch(&r, "bind-unbind", true);
    let bindings = || {
        r.ipc
            .inner
            .lock()
            .broker
            .as_ref()
            .map(|b| b.ledger.bindings_by(1))
            .unwrap()
    };
    let (b, hs) = expect_msg(&c.from_child);
    assert_eq!((b.as_slice(), hs.len()), (&b"unbound"[..], 1));
    assert_eq!(bindings(), 1, "only the bootstrap listener's binding");
    r.ipc
        .send(&c.to_child, b"next".to_vec(), Vec::new())
        .unwrap();
    let (b, hs) = expect_msg(&c.from_child);
    assert_eq!((b.as_slice(), hs.len()), (&b"bound"[..], 1));
    assert_eq!(bindings(), 2, "the bootstrap listener's and the kept one");
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
}

/// Item 1 (M8): a child that will not drain its control link cannot make
/// the broker buffer records without bound. Flooding the broker's queue
/// to a child past the cap drops that child's link.
#[test]
fn a_child_that_will_not_read_its_link_is_dropped() {
    let r = rig();
    let c = launch(&r, "echo", false);
    {
        let mut st = r.ipc.inner.lock();
        let link_id = st
            .broker
            .as_ref()
            .and_then(|b| b.children.values().next())
            .map(|c| c.link)
            .expect("the child has a link");
        if let Some(e) = st.ends.get_mut(&link_id)
            && let super::Role::Link { out, .. } = &mut e.role
        {
            for _ in 0..=super::MAX_LINK_BACKLOG {
                out.push_back(Vec::new());
            }
        }
    }
    r.ipc.inner.wake();
    let gone = (0..200).any(|_| {
        std::thread::sleep(Duration::from_millis(25));
        r.ipc
            .inner
            .lock()
            .broker
            .as_ref()
            .is_some_and(|b| b.children.is_empty())
    });
    assert!(gone, "the broker kept a child that would not read its link");
    drop(c);
}

/// Items 2 and 3: a channel end closed normally releases its ledger
/// entry. The parent gives a child one channel's receiver, the child
/// drops it, the parent drops the sender, and the broker then tracks no
/// more channels than before the channel existed.
#[test]
fn a_closed_channel_is_forgotten_by_the_broker() {
    let r = rig();
    let c = launch(&r, "take-and-drop", false);
    // The child's launch ends are given too (W3), and stay while it runs.
    let (base, base_given) = r
        .ipc
        .inner
        .lock()
        .broker
        .as_ref()
        .map(|b| (b.ledger.entry_count(), b.given.len()))
        .unwrap();
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&c.to_child, b"take".to_vec(), vec![rx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&c.from_child).0, b"dropped");
    drop(tx);
    // The child's Closed and the parent's own close release both ends;
    // the broker then sweeps the channel.
    let forgotten = (0..200).any(|_| {
        std::thread::sleep(Duration::from_millis(25));
        r.ipc
            .inner
            .lock()
            .broker
            .as_ref()
            .is_some_and(|b| b.ledger.entry_count() <= base && b.given.len() <= base_given)
    });
    let (ec, gl) = {
        let st = r.ipc.inner.lock();
        let b = st.broker.as_ref().unwrap();
        (b.ledger.entry_count(), b.given.len())
    };
    assert!(
        forgotten,
        "the broker kept a closed channel: base={base}/{base_given} entries={ec} given={gl}"
    );
}

// ---- shared-memory regions (D6) ----

use hg_substrate::ipc::{ConsumerRegion, MAX_REGIONS_PER_OWNER, ReadOnlyRegion, RegionWriter};

/// Sets the kind and length a region handle's table entry travels as
/// (tests only: a hostile child's own transport, lying to the broker).
fn relabel(r: &Rig, h: &Handle, kind: HandleKind, len: u32) {
    let id = r.ipc.held_end(h).unwrap();
    let mut st = r.ipc.inner.lock();
    if let Some(super::End {
        role: super::Role::Region {
            kind: k, len: l, ..
        },
        ..
    }) = st.ends.get_mut(&id)
    {
        *k = kind;
        *l = len;
    }
}

/// The section handle behind a region handle (tests only).
fn section(r: &Rig, h: &Handle) -> Arc<std::os::windows::io::OwnedHandle> {
    let id = r.ipc.held_end(h).unwrap();
    Arc::clone(&r.ipc.inner.lock().ends.get(&id).unwrap().h)
}

fn region_child(case: &str, r: &Rig, up: &ChannelSender, from_parent: &Receiver<Ev>) -> i32 {
    let Ev::Msg(mode, hs) = next(from_parent) else {
        return 30;
    };
    let mut hs = hs.into_iter();
    match case {
        "region-roundtrip" => {
            // A writer from the broker is mapped, written, refused a seal
            // while mapped, then sealed and sent `immutable` (the broker
            // copies it); a second is sealed and sent plain
            // (filler-trusted); a third becomes a producer whose
            // consumer goes up. A writer cannot be sent on from here.
            let w1 = RegionWriter::from_handle(hs.next().unwrap()).unwrap();
            let w2 = RegionWriter::from_handle(hs.next().unwrap()).unwrap();
            let w3 = RegionWriter::from_handle(hs.next().unwrap()).unwrap();
            let w4 = hs.next().unwrap();
            let m = r.ipc.map_region(w1.handle()).unwrap();
            let mut hi = [0u8; 2];
            m.read_into(0, &mut hi).unwrap();
            m.write_from(0, b"child").unwrap();
            let refused = r.ipc.seal_region(w1).unwrap_err();
            let busy = refused.error;
            let w1 = refused.into_writer().unwrap();
            drop(m);
            let frozen = r.ipc.seal_region(w1).unwrap();
            frozen.want_immutable();
            let m2 = r.ipc.map_region(w2.handle()).unwrap();
            m2.write_from(0, b"ft").unwrap();
            drop(m2);
            let plain = r.ipc.seal_region(w2).unwrap();
            let p = r.ipc.share_region(w3).unwrap();
            let c = r.ipc.consumer_of(&p).unwrap();
            p.mapping().store_u32_release(0, 42).unwrap();
            let writer_on = r.ipc.send(up, b"w".to_vec(), vec![w4]).unwrap_err();
            // Fact S9: no read-only handle here can be widened, though no
            // container SID is in the section's DACL.
            let widened = [frozen.handle(), plain.handle(), c.handle()]
                .iter()
                .filter(|h| super::sys::narrow(&section(r, h), 0x6).is_ok())
                .count();
            let report = format!(
                "hi={} busy={busy:?} writer={writer_on:?} widened={widened}",
                String::from_utf8_lossy(&hi)
            );
            r.ipc
                .send(
                    up,
                    report.into_bytes(),
                    vec![frozen.into_handle(), plain.into_handle(), c.into_handle()],
                )
                .unwrap();
            let _ = next(from_parent);
            drop(p);
            0
        }
        "region-big" | "region-big-exit" => {
            // Fills each large writer from the broker, seals it and sends
            // them all `immutable` in one record: the broker copies each
            // (review Q1).
            let mut out = Vec::new();
            for (i, h) in hs.enumerate() {
                let w = RegionWriter::from_handle(h).unwrap();
                let m = r.ipc.map_region(w.handle()).unwrap();
                let step = u32::try_from(w.len().min(1 << 20)).unwrap();
                let chunk: Vec<u8> = (0..step)
                    .map(|j| u8::try_from((j + u32::try_from(i).unwrap()) % 251).unwrap())
                    .collect();
                for off in (0..w.len()).step_by(chunk.len()) {
                    m.write_from(off, &chunk).unwrap();
                }
                drop(m);
                let ro = r.ipc.seal_region(w).unwrap();
                ro.want_immutable();
                out.push(ro.into_handle());
            }
            r.ipc.send(up, b"big".to_vec(), out).unwrap();
            // The broker's copies take a while: wait well past them, or
            // until the parent says to end.
            let _ = from_parent.recv_timeout(Duration::from_secs(300));
            if case == "region-big-exit" {
                // Dies abruptly, when the parent says (while the broker
                // holds its copies).
                std::process::exit(0);
            }
            0
        }
        "region-forge" => {
            // A hostile transport: a region sent as what it is not, on a
            // channel to the parent. The broker refuses the relay, which
            // closes that channel (review M11); the report says whether
            // the next send on it was refused.
            let tx = ChannelSender::from_handle(hs.next().unwrap()).unwrap();
            let w = RegionWriter::from_handle(hs.next().unwrap()).unwrap();
            let mut kept = None;
            let h = match mode.as_slice() {
                b"consumer-as-ro" => {
                    let p = r.ipc.share_region(w).unwrap();
                    let c = r.ipc.consumer_of(&p).unwrap().into_handle();
                    relabel(r, &c, HandleKind::ReadOnlyRegion, 4096);
                    std::mem::forget(p);
                    c
                }
                b"wrong-len" => {
                    let ro = r.ipc.seal_region(w).unwrap().into_handle();
                    relabel(r, &ro, HandleKind::ReadOnlyRegion, 8192);
                    ro
                }
                b"kept-writer" => {
                    kept = Some(super::sys::narrow(&section(r, w.handle()), 0x6).unwrap());
                    r.ipc.seal_region(w).unwrap().into_handle()
                }
                _ => r.ipc.seal_region(w).unwrap().into_handle(),
            };
            r.ipc.send(&tx, b"region".to_vec(), vec![h]).unwrap();
            std::thread::sleep(Duration::from_millis(1500));
            let after = r.ipc.send(&tx, b"after".to_vec(), Vec::new());
            r.ipc
                .send(up, format!("after={after:?}").into_bytes(), Vec::new())
                .unwrap();
            let _ = next(from_parent);
            drop(kept);
            0
        }
        _ => 99,
    }
}

#[test]
fn regions_cross_to_a_sandboxed_child_and_back() {
    let r = rig();
    let c = launch(&r, "region-roundtrip", true);
    let w1 = r.ipc.allocate_region(4096, c.peer).unwrap();
    let m = r.ipc.map_region(w1.handle()).unwrap();
    m.write_from(0, b"hi").unwrap();
    drop(m);
    let ws: Vec<Handle> = [w1]
        .into_iter()
        .chain((0..3).map(|_| r.ipc.allocate_region(4096, c.peer).unwrap()))
        .map(RegionWriter::into_handle)
        .collect();
    r.ipc.send(&c.to_child, b"take".to_vec(), ws).unwrap();
    let (report, mut hs) = expect_msg(&c.from_child);
    assert_eq!(
        String::from_utf8(report).unwrap(),
        "hi=hi busy=Busy writer=Refused widened=0"
    );
    let cons = ConsumerRegion::from_handle(hs.pop().unwrap()).unwrap();
    let plain = ReadOnlyRegion::from_handle(hs.pop().unwrap()).unwrap();
    let frozen = ReadOnlyRegion::from_handle(hs.pop().unwrap()).unwrap();
    // Sealed by a child: filler-trusted, unless the broker copied it.
    assert!(frozen.immutable());
    assert!(!plain.immutable());
    assert!(!cons.immutable());
    let mut got = [0u8; 5];
    r.ipc
        .map_region(frozen.handle())
        .unwrap()
        .read_into(0, &mut got)
        .unwrap();
    assert_eq!(&got, b"child");
    let mut ft = [0u8; 2];
    r.ipc
        .map_region(plain.handle())
        .unwrap()
        .read_into(0, &mut ft)
        .unwrap();
    assert_eq!(&ft, b"ft");
    let cm = r.ipc.map_region(cons.handle()).unwrap();
    assert_eq!(cm.load_u32_acquire(0), Ok(42));
    assert!(cm.write_from(0, b"x").is_err());
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
    assert_eq!(c.exit_code(), Some(0));
    // The producer is gone; the consumer's view still reads (fact S4).
    assert_eq!(cm.load_u32_acquire(0), Ok(42));
    r.ipc.release_owner(c.peer);
}

/// Any holder passes a consumer on, as on Linux and macOS (D6 review
/// finding 4): a sandboxed child's consumer reaches the broker, which
/// forwards it to a second sandboxed child, which sends it back. Neither
/// relayer is the producer.
#[test]
fn any_holder_forwards_a_consumer() {
    let r = rig();
    let c = launch(&r, "region-roundtrip", true);
    let ws: Vec<Handle> = (0..4)
        .map(|_| r.ipc.allocate_region(4096, c.peer).unwrap().into_handle())
        .collect();
    r.ipc.send(&c.to_child, b"take".to_vec(), ws).unwrap();
    let (_report, mut hs) = expect_msg(&c.from_child);
    let cons = ConsumerRegion::from_handle(hs.pop().unwrap()).unwrap();
    let b = launch(&r, "echo", true);
    r.ipc
        .send(&b.to_child, b"cons".to_vec(), vec![cons.into_handle()])
        .unwrap();
    let (m, mut hs) = expect_msg(&b.from_child);
    assert_eq!(m, b"cons");
    let back = ConsumerRegion::from_handle(hs.remove(0)).unwrap();
    assert!(!back.immutable());
    let bm = r.ipc.map_region(back.handle()).unwrap();
    assert_eq!(bm.load_u32_acquire(0), Ok(42));
    assert!(bm.write_from(0, b"x").is_err());
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
    assert_eq!(c.exit_code(), Some(0));
    r.ipc.release_owner(c.peer);
}

/// D6 review finding 5 (fact S12): a child outside a container at this
/// process's own integrity could open this process and duplicate its
/// sections, so a region is never given to it; its channel ends still
/// move.
#[test]
fn a_child_outside_a_container_gets_no_regions() {
    let r = rig();
    let c = launch(&r, "echo", false);
    let w = r.ipc.allocate_region(4096, c.peer).unwrap();
    let ro = r.ipc.seal_region(w).unwrap();
    // The send is the broker's own and succeeds; the child's redemption
    // is refused, its channel closes, and it never echoes the region
    // (before the fix: `msg "ro" 1`).
    r.ipc
        .send(&c.to_child, b"ro".to_vec(), vec![ro.into_handle()])
        .unwrap();
    let seen = match next(&c.from_child) {
        Ev::Msg(m, hs) => format!("msg {:?} {}", String::from_utf8_lossy(&m), hs.len()),
        other => format!("{other:?}"),
    };
    assert_eq!(seen, "Closed");
}

/// Every section carries a no-write-up label at the broker's own
/// integrity (D6 review finding 5, fact S12).
#[test]
fn sections_carry_the_brokers_integrity_label() {
    let h = super::sys::create_section(4096).unwrap();
    let sddl = super::sys::label_sddl(&h).unwrap();
    let own = super::sys::own_integrity().unwrap();
    let alias = match own {
        0x1000 => "LW",
        0x2000 => "ME",
        0x3000 => "HI",
        0x4000 => "SI",
        _ => "",
    };
    assert!(
        sddl.contains(&format!("(ML;;NW;;;{alias})"))
            || sddl.contains(&format!("(ML;;NW;;;S-1-16-{own})")),
        "{sddl}"
    );
}

/// What a forged region relay from a sandboxed child gets: the child's
/// report, and what this process's receiver saw.
fn forge(mode: &str) -> (String, Ev) {
    let r = rig();
    let c = launch(&r, "region-forge", true);
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    let w = r.ipc.allocate_region(4096, c.peer).unwrap();
    r.ipc
        .send(
            &c.to_child,
            mode.as_bytes().to_vec(),
            vec![tx.into_handle(), w.into_handle()],
        )
        .unwrap();
    let report = String::from_utf8(expect_msg(&c.from_child).0).unwrap();
    // The child's end of `tx` closes when it exits, after its last word.
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
    let seen = next(&out);
    (report, seen)
}

/// Review B1's test: a consumer region sent as read-only is refused by
/// the broker's registry; so are a wrong length (minor 1) and a seal
/// whose writer handle was kept. The honest relay passes.
#[test]
fn the_broker_refuses_regions_that_are_not_what_they_claim() {
    for mode in ["consumer-as-ro", "wrong-len", "kept-writer"] {
        let (report, seen) = forge(mode);
        assert_eq!(report, "after=Err(Closed)", "{mode}");
        assert!(matches!(seen, Ev::Closed), "{mode}: {seen:?}");
    }
    let (report, seen) = forge("honest");
    assert_eq!(report, "after=Ok(())");
    match seen {
        Ev::Msg(b, hs) => {
            assert_eq!(b, b"region");
            let ro = ReadOnlyRegion::from_handle(hs.into_iter().next().unwrap()).unwrap();
            assert!(!ro.immutable());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_broker_sealed_region_is_immutable_and_exact() {
    let r = rig();
    let me = OsPeer::new(std::process::id(), 0);
    let w = r.ipc.allocate_region(100, me).unwrap();
    let m = r.ipc.map_region(w.handle()).unwrap();
    m.write_from(96, b"tail").unwrap();
    let refused = r.ipc.seal_region(w).unwrap_err();
    assert_eq!(refused.error, IpcError::Busy);
    let w = refused.into_writer().unwrap();
    drop(m);
    let ro = r.ipc.seal_region(w).unwrap();
    assert!(ro.immutable());
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    r.ipc
        .send(&tx, b"ro".to_vec(), vec![ro.into_handle()])
        .unwrap();
    let (_, mut hs) = expect_msg(&out);
    let got = ReadOnlyRegion::from_handle(hs.remove(0)).unwrap();
    assert!(got.immutable());
    assert_eq!(got.len(), 100);
    let gm = r.ipc.map_region(got.handle()).unwrap();
    let mut tail = [0u8; 4];
    gm.read_into(96, &mut tail).unwrap();
    assert_eq!(&tail, b"tail");
    assert!(gm.read_into(97, &mut tail).is_err());
}

#[test]
fn a_producer_keeps_its_one_mapping_and_never_crosses() {
    let r = rig();
    let me = OsPeer::new(std::process::id(), 0);
    let p = r
        .ipc
        .share_region(r.ipc.allocate_region(4096, me).unwrap())
        .unwrap();
    assert!(p.mapping().writable());
    assert_eq!(r.ipc.map_region(p.handle()).err(), Some(IpcError::Refused));
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    let c = r.ipc.consumer_of(&p).unwrap();
    p.mapping().write_from(0, b"live").unwrap();
    r.ipc
        .send(&tx, b"c".to_vec(), vec![c.into_handle()])
        .unwrap();
    let (_, mut hs) = expect_msg(&out);
    let got = ConsumerRegion::from_handle(hs.remove(0)).unwrap();
    assert!(!got.immutable());
    let mut b = [0u8; 4];
    r.ipc
        .map_region(got.handle())
        .unwrap()
        .read_into(0, &mut b)
        .unwrap();
    assert_eq!(&b, b"live");
}

#[test]
fn allocation_is_the_broker_s_within_quota() {
    let r = rig();
    let owner = OsPeer::new(77, 0);
    assert_eq!(
        r.ipc.allocate_region(0, owner).map(|_w| ()),
        Err(IpcError::Refused)
    );
    for _ in 0..MAX_REGIONS_PER_OWNER {
        drop(r.ipc.allocate_region(4096, owner).unwrap());
    }
    assert_eq!(
        r.ipc.allocate_region(4096, owner).map(|_w| ()),
        Err(IpcError::Quota)
    );
    r.ipc.release_owner(owner);
    assert!(r.ipc.allocate_region(4096, owner).is_ok());
}

/// A relay the ledger refuses after the registry moved on puts the
/// registry back: states restored, copies forgotten and refunded.
#[test]
fn a_refused_relay_leaves_the_registry_as_it_was() {
    use crate::ipc::win::regions::{Outcome, State};
    let r = rig();
    let owner = OsPeer::new(77, 0);
    let w = r.ipc.allocate_region(4096, owner).unwrap();
    let mut st = r.ipc.inner.lock();
    let b = st.broker.as_mut().unwrap();
    let (&id, _) = b.regions.iter().next().unwrap();
    b.regions.get_mut(&id).unwrap().state = State::WriterOut(Some(3));
    let copy = Outcome {
        next: State::SealedByChild,
        filler_trusted: false,
        copy: true,
    };
    let mut cands = vec![super::Cand {
        value: 0,
        kind: crate::ipc::common::codec::WireKind::ReadOnlyRegion,
        held: Some(super::sys::create_section(4096).unwrap()),
        own: false,
        facts: super::sys::RawFacts::default(),
        chan: None,
        pipe: None,
        ticket_end: None,
        queue_old: false,
        let_go: None,
        changed: false,
        len: 4096,
        flags: 0,
        section: super::sys::SectionFacts::default(),
    }];
    let mut later: Vec<Box<dyn core::any::Any + Send>> = Vec::new();
    // The copy, planned and made as a relay does (charged at planning).
    let jobs = super::Inner::plan_copies(b, 3, core::iter::empty()).unwrap();
    assert!(jobs.is_empty());
    assert!(b.quota.charge(owner, 4096));
    let master = super::sys::create_section(4096).unwrap();
    let ro = super::sys::narrow(&master, 0x4).unwrap();
    let mut prepared = vec![super::Prepared {
        src: id,
        len: 4096,
        owner,
        master,
        ro,
    }];
    let (grants, undo) = super::Inner::region_commit(
        b,
        &[(
            0,
            id,
            copy,
            crate::ipc::common::codec::WireKind::ReadOnlyRegion,
            4096,
        )],
        &mut cands,
        &mut later,
        &mut prepared,
    )
    .unwrap();
    assert!(prepared.is_empty());
    assert_eq!(b.regions.len(), 2);
    assert_ne!(grants[0].1.id, id);
    assert_eq!(b.regions[&id].state, State::SealedByChild);
    super::Inner::region_rollback(b, undo);
    assert_eq!(b.regions.len(), 1);
    assert_eq!(b.regions[&id].state, State::WriterOut(Some(3)));
    // The copy's charge came back: the owner holds one region again.
    for _ in 1..MAX_REGIONS_PER_OWNER {
        assert!(b.quota.charge(owner, 4096));
    }
    assert!(!b.quota.charge(owner, 4096));
    drop(st);
    drop(w);
}

/// The longest a handle round trip through another child may take while
/// a child's large `immutable` regions are copied (D5 review M8: no slow
/// work under the broker's lock). Measured on rig253 (eight 64 MiB
/// copies, about 20 s): with the copies under the lock the worst round
/// trip was 16 to 19 s; with them off it, 19 to 22 ms (one run 109 ms).
const STALL_BOUND: Duration = Duration::from_millis(STALL_BOUND_MS);
const STALL_BOUND_MS: u64 = 500;

/// While one child seals eight 64 MiB regions `immutable` (the broker
/// copies each, review Q1), another child's relays and redeems (a
/// receiver end sent to it and back) still finish within
/// [`STALL_BOUND`].
#[test]
fn a_large_copy_stalls_no_other_child() {
    const MIB64: usize = 64 << 20;
    let r = rig();
    let a = launch(&r, "region-big", true);
    let b = launch(&r, "echo", true);
    let ws: Vec<Handle> = (0..8)
        .map(|_| r.ipc.allocate_region(MIB64, a.peer).unwrap().into_handle())
        .collect();
    r.ipc.send(&a.to_child, b"fill".to_vec(), ws).unwrap();
    let mut times = Vec::new();
    let started = std::time::Instant::now();
    let big = loop {
        match a.from_child.try_recv() {
            Ok(Ev::Msg(m, hs)) => break (m, hs),
            Ok(other) => panic!("{other:?}"),
            Err(_) => {}
        }
        assert!(started.elapsed() < Duration::from_secs(120), "no regions");
        let (_tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
        let t = std::time::Instant::now();
        r.ipc
            .send(&b.to_child, b"h".to_vec(), vec![rx.into_handle()])
            .unwrap();
        let (_, back) = expect_msg(&b.from_child);
        times.push(t.elapsed());
        assert_eq!(back.len(), 1);
    };
    // A few more after the copies, so the window is covered.
    for _ in 0..5 {
        let (_tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
        let t = std::time::Instant::now();
        r.ipc
            .send(&b.to_child, b"h".to_vec(), vec![rx.into_handle()])
            .unwrap();
        let _ = expect_msg(&b.from_child);
        times.push(t.elapsed());
    }
    let (m, hs) = big;
    assert_eq!(m, b"big");
    assert_eq!(hs.len(), 8);
    for (i, h) in hs.into_iter().enumerate() {
        let ro = ReadOnlyRegion::from_handle(h).unwrap();
        assert!(ro.immutable());
        let mut x = [0u8; 1];
        r.ipc
            .map_region(ro.handle())
            .unwrap()
            .read_into(12_345, &mut x)
            .unwrap();
        assert_eq!(usize::from(x[0]), (12_345 + i) % 251);
    }
    times.sort_unstable();
    let max = *times.last().unwrap();
    let p50 = times[times.len() / 2];
    println!(
        "D6-STALL round trips={} p50={:.1}ms max={:.1}ms",
        times.len(),
        p50.as_secs_f64() * 1e3,
        max.as_secs_f64() * 1e3
    );
    r.ipc
        .send(&a.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
    assert!(max < STALL_BOUND, "a round trip took {max:?}");
    r.ipc.release_owner(a.peer);
}

/// Marks the section behind `h` as sealed by a child (tests only: the
/// state a child's seal leaves, without a child).
fn mark_child_sealed(r: &Rig, h: &Handle) {
    let s = section(r, h);
    let mut st = r.ipc.inner.lock();
    let b = st.broker.as_mut().unwrap();
    let id = super::Inner::region_id(b, &s).unwrap();
    b.regions.get_mut(&id).unwrap().state = crate::ipc::win::regions::State::SealedByChild;
}

/// The broker's own send of a child-sealed region `immutable`: the copy
/// is made off the lock on its own thread, the record waits for it, and
/// the receiver gets the copy, attested immutable, with the contents.
#[test]
fn the_broker_copies_off_the_lock_for_its_own_sends() {
    let r = rig();
    let owner = OsPeer::new(77, 0);
    let w = r.ipc.allocate_region(1 << 20, owner).unwrap();
    let m = r.ipc.map_region(w.handle()).unwrap();
    m.write_from(1000, b"copied").unwrap();
    drop(m);
    let ro = r.ipc.seal_region(w).unwrap();
    mark_child_sealed(&r, ro.handle());
    ro.want_immutable();
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    r.ipc
        .send(&tx, b"c".to_vec(), vec![ro.into_handle()])
        .unwrap();
    let (_, mut hs) = expect_msg(&out);
    let got = ReadOnlyRegion::from_handle(hs.remove(0)).unwrap();
    assert!(got.immutable());
    let mut b = [0u8; 6];
    r.ipc
        .map_region(got.handle())
        .unwrap()
        .read_into(1000, &mut b)
        .unwrap();
    assert_eq!(&b, b"copied");
    // The original and its copy, both charged to the owner.
    let st = r.ipc.inner.lock();
    let br = st.broker.as_ref().unwrap();
    assert_eq!(br.regions.values().filter(|e| e.owner == owner).count(), 2);
}

/// If the broker's copy thread cannot start, the plan is taken back
/// whole: the charge refunded and the source no longer marked as being
/// copied, so the next relay that needs the copy plans it at once rather
/// than wait COPY_WAIT_MAX and be refused (merge-hook finding on D6).
#[test]
fn a_copy_thread_that_cannot_start_leaves_no_mark() {
    let r = rig();
    let owner = OsPeer::new(84, 0);
    let ro = r
        .ipc
        .seal_region(r.ipc.allocate_region(4096, owner).unwrap())
        .unwrap();
    mark_child_sealed(&r, ro.handle());
    ro.want_immutable();
    let h = section(&r, ro.handle());
    let facts = super::sys::section_facts(&h);
    let src = {
        let st = r.ipc.inner.lock();
        super::Inner::region_id(st.broker.as_ref().unwrap(), &h).unwrap()
    };
    r.ipc.inner.fail_copy_spawn.store(true, Ordering::Release);
    let (tx, _rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&tx, b"c".to_vec(), vec![ro.into_handle()])
        .unwrap();
    // The record is given up: its channel breaks.
    let t0 = std::time::Instant::now();
    while r.ipc.send(&tx, b"x".to_vec(), Vec::new()).is_ok() {
        assert!(
            t0.elapsed() < Duration::from_secs(5),
            "the send was not given up"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    let mut st = r.ipc.inner.lock();
    let b = st.broker.as_mut().unwrap();
    assert!(!b.regions.get(&src).unwrap().copying, "still marked");
    // No copy made or charged: only the original is the owner's.
    assert_eq!(b.regions.values().filter(|e| e.owner == owner).count(), 1);
    // The next relay that needs the copy plans it now.
    let item = core::iter::once((
        &*h,
        crate::ipc::common::codec::WireKind::ReadOnlyRegion,
        4096u32,
        1u8,
        facts,
    ));
    let jobs = super::Inner::plan_copies(b, super::BROKER, item);
    // Plan::Wait here is the leaked mark.
    assert!(
        matches!(&jobs, Ok(j) if j.len() == 1),
        "the copy was not planned"
    );
    super::Inner::release_jobs(b, jobs.unwrap_or_default());
}

/// Planning charges each copy to the owner (its quota and the browser's
/// cap); a plan that does not fit charges nothing; copies a relay does
/// not use (refused, child dead, record given up) are refunded.
#[test]
fn copies_are_charged_when_planned_and_refunded_when_unused() {
    let r = rig();
    let owner = OsPeer::new(78, 0);
    let ro = r
        .ipc
        .seal_region(r.ipc.allocate_region(4096, owner).unwrap())
        .unwrap();
    mark_child_sealed(&r, ro.handle());
    let h = section(&r, ro.handle());
    let facts = super::sys::section_facts(&h);
    let mut st = r.ipc.inner.lock();
    let b = st.broker.as_mut().unwrap();
    let item = || {
        core::iter::once((
            &*h,
            crate::ipc::common::codec::WireKind::ReadOnlyRegion,
            4096u32,
            1u8,
            facts,
        ))
    };
    let jobs = super::Inner::plan_copies(b, super::BROKER, item()).unwrap();
    assert_eq!(jobs.len(), 1);
    // Charged: the owner holds two of its 256.
    let (made, failed) = super::Inner::make_copies(jobs);
    assert!(failed.is_empty());
    let spare = super::Inner::release_prepared(b, made);
    assert_eq!(spare.len(), 2);
    // Refunded: 255 more fit, then none.
    for _ in 1..MAX_REGIONS_PER_OWNER {
        assert!(b.quota.charge(owner, 4096));
    }
    assert!(!b.quota.charge(owner, 4096));
    // A full owner: the plan is refused and charges nothing.
    assert!(matches!(
        super::Inner::plan_copies(b, super::BROKER, item()),
        Err(super::Plan::Quota)
    ));
    b.quota.release(owner);
    drop(st);
}

/// The child that sent eight large regions for copying dies before its
/// copies are made: they are refunded and dropped, and the copying
/// stalls no other child. Each step waits for the one before it, so the
/// order is fixed and no step depends on how fast the machine is: the
/// broker plans and charges the copies while the child lives, then holds
/// them; only then is the child told to die; the hold is let go once the
/// broker has forgotten it; and the counts are read once the copies are
/// refunded. Before, the child died on a 10 s fuse and the test gave the
/// whole sequence 45 s, which a slow machine overran: the child's fill
/// alone measured 5 to 8 s on rig253 with 32 CPUs and 21 to 29 s with
/// one or two, so it could still be alive at 45 s (PR #54's CI run, 29
/// Oct; reproduced on rig253 on one CPU shared with a busy loop, where
/// the fill ended at 36.7 s). Before that, it relied on 8×64 MiB taking
/// longer to copy than the fuse ("kept 16").
#[test]
fn a_child_dying_mid_copy_stalls_no_one_and_keeps_no_copy() {
    const MIB64: usize = 64 << 20;
    /// Fails a step that never comes rather than hanging; never reached
    /// on a working broker, however slow the machine.
    const STEP_MAX: Duration = Duration::from_secs(600);
    /// Lets the held copies go however the test ends.
    struct Release<'a>(&'a std::sync::atomic::AtomicBool);
    impl Drop for Release<'_> {
        fn drop(&mut self) {
            self.0.store(false, Ordering::Release);
        }
    }
    let r = rig();
    let a = launch(&r, "region-big-exit", true);
    let b = launch(&r, "echo", true);
    let ws: Vec<Handle> = (0..8)
        .map(|_| r.ipc.allocate_region(MIB64, a.peer).unwrap().into_handle())
        .collect();
    let inner = &r.ipc.inner;
    inner.hold_copies.store(true, Ordering::Release);
    let _release = Release(&inner.hold_copies);
    let planned = inner.copies_planned.load(Ordering::Acquire);
    let settled = inner.copies_settled.load(Ordering::Acquire);
    r.ipc.send(&a.to_child, b"fill".to_vec(), ws).unwrap();
    let mut max = Duration::ZERO;
    let mut n = 0u32;
    // One round trip through `b`, timed.
    let mut round_trip = || {
        let (_tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
        let t = std::time::Instant::now();
        r.ipc
            .send(&b.to_child, b"h".to_vec(), vec![rx.into_handle()])
            .unwrap();
        let _ = expect_msg(&b.from_child);
        max = max.max(t.elapsed());
        n += 1;
    };
    // Round trips until `done`, which must come within STEP_MAX.
    let mut until = |what: &str, done: &dyn Fn() -> bool| {
        let since = std::time::Instant::now();
        while !done() {
            assert!(since.elapsed() < STEP_MAX, "{what} never came");
            round_trip();
        }
    };
    // 1. The child's copies are planned and charged while it lives, and
    // held.
    until("the copies' plan", &|| {
        inner.copies_planned.load(Ordering::Acquire) > planned
    });
    // 2. The child dies; the broker forgets it (only `b` is left).
    r.ipc
        .send(&a.to_child, b"die".to_vec(), Vec::new())
        .unwrap();
    until("the child's death", &|| children(&r) == 1);
    // 3. The copies are made for a dead child, then refunded.
    inner.hold_copies.store(false, Ordering::Release);
    until("the copies' refund", &|| {
        inner.copies_settled.load(Ordering::Acquire) > settled
    });
    // A few more, so the window after the copies is covered too.
    for _ in 0..5 {
        round_trip();
    }
    println!(
        "D6-STALL-EXIT round trips={n} max={:.1}ms",
        max.as_secs_f64() * 1e3
    );
    assert_eq!(a.exit_code(), Some(0));
    assert!(max < STALL_BOUND, "a round trip took {max:?}");
    // Only the eight sections the broker allocated: no copy was kept.
    assert_eq!(owned_by(&r, a.peer), 8);
}

/// The other order, which the test above used to meet by chance: copies
/// made while their child lives are registered as the sources' copies
/// (charged to the owner until it is released), so they outlive it.
#[test]
fn copies_made_before_their_child_dies_are_kept() {
    let r = rig();
    let a = launch(&r, "region-big-exit", true);
    let ws: Vec<Handle> = (0..8)
        .map(|_| {
            r.ipc
                .allocate_region(64 << 10, a.peer)
                .unwrap()
                .into_handle()
        })
        .collect();
    r.ipc.send(&a.to_child, b"fill".to_vec(), ws).unwrap();
    let _ = expect_msg(&a.from_child);
    r.ipc
        .send(&a.to_child, b"die".to_vec(), Vec::new())
        .unwrap();
    assert_eq!(a.exit_code(), Some(0));
    assert_eq!(owned_by(&r, a.peer), 16);
}

/// Probe P12, live: a process another thread starts with inheritance on
/// and no handle list, while a launch is under way, gets no copy of the
/// child's link end; and so the broker still sees the child die while
/// that process runs.
#[test]
fn a_process_started_during_a_launch_inherits_no_link() {
    /// Ends the inheriting process however the test ends.
    struct Reap(std::process::Child);
    impl Drop for Reap {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let r = rig();
    // Held as a `Reap` from the start, so a failed launch ends it too.
    let slot: Arc<Mutex<Option<Reap>>> = Arc::new(Mutex::new(None));
    let put = Arc::clone(&slot);
    // A plain `std::process::Command`: inheritance on, no handle list.
    *r.ipc.inner.in_launch_window.lock().unwrap() = Some(Box::new(move || {
        let c = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "ipc::windows::tests::d5_child",
                "--nocapture",
                "--test-threads",
                "1",
            ])
            .env("IZCAL_D5_CHILD", "linger")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        *put.lock().unwrap() = Some(Reap(c));
    }));
    let before = children(&r);
    let a = launch(&r, "burst-exit", true);
    let taken = slot.lock().unwrap().take();
    let mut leaker = taken.expect("the launch ran the hook");
    // 1. The link end's holders: this test's handle and the child's.
    let end = r.ipc.inner.last_link_end.lock().unwrap().take().unwrap();
    let holders = super::sys::count_handles(&end);
    drop(end);
    assert_eq!(holders, 2, "another process holds the child's link end");
    // 2. The child ends; the broker forgets it while the other process
    // still runs.
    assert_eq!(a.exit_code(), Some(0));
    let since = std::time::Instant::now();
    while children(&r) != before {
        assert!(since.elapsed() < WAIT, "the child was never forgotten");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        leaker.0.try_wait().unwrap().is_none(),
        "the inheriting process ran throughout"
    );
}

/// How many children the broker knows.
fn children(r: &Rig) -> usize {
    r.ipc.inner.lock().broker.as_ref().unwrap().children.len()
}

/// The registry entries charged to `peer`.
fn owned_by(r: &Rig, peer: OsPeer) -> usize {
    let st = r.ipc.inner.lock();
    st.broker
        .as_ref()
        .unwrap()
        .regions
        .values()
        .filter(|e| e.owner == peer)
        .count()
}

/// Review finding 3: the broker's live mappings are the section's, so a
/// writer that came back to the broker through a relay (a new table
/// entry) still cannot be sealed while an earlier mapping writes.
#[test]
fn a_writer_relayed_back_cannot_be_sealed_while_mapped() {
    let r = rig();
    let w = r.ipc.allocate_region(4096, OsPeer::new(77, 0)).unwrap();
    let m = r.ipc.map_region(w.handle()).unwrap();
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    r.ipc
        .send(&tx, b"w".to_vec(), vec![w.into_handle()])
        .unwrap();
    let (_, mut hs) = expect_msg(&out);
    let back = RegionWriter::from_handle(hs.remove(0)).unwrap();
    let refused = r.ipc.seal_region(back).unwrap_err();
    assert_eq!(refused.error, IpcError::Busy);
    let back = refused.into_writer().unwrap();
    m.write_from(0, b"late").unwrap();
    drop(m);
    let ro = r.ipc.seal_region(back).unwrap();
    assert!(ro.immutable());
}

/// Review finding 3: a seal or allocation whose handle cannot be made
/// leaves the registry and the charges as they were.
#[test]
fn a_failed_handle_leaves_the_registry_as_it_was() {
    let r = rig();
    let owner = OsPeer::new(79, 0);
    let count = |r: &Rig| {
        let st = r.ipc.inner.lock();
        st.broker
            .as_ref()
            .unwrap()
            .regions
            .values()
            .filter(|e| e.owner == owner)
            .count()
    };
    r.ipc
        .inner
        .fail_region_handle
        .store(true, Ordering::Release);
    assert!(r.ipc.allocate_region(4096, owner).is_err());
    assert_eq!(count(&r), 0);
    let w = r.ipc.allocate_region(4096, owner).unwrap();
    r.ipc
        .inner
        .fail_region_handle
        .store(true, Ordering::Release);
    let w = r.ipc.seal_region(w).unwrap_err().into_writer().unwrap();
    // Still a writer the broker holds: it seals, attested immutable.
    let ro = r.ipc.seal_region(w).unwrap();
    assert!(ro.immutable());
    // One region charged: 255 more fit, then none.
    let mut st = r.ipc.inner.lock();
    let b = st.broker.as_mut().unwrap();
    for _ in 1..MAX_REGIONS_PER_OWNER {
        assert!(b.quota.charge(owner, 4096));
    }
    assert!(!b.quota.charge(owner, 4096));
}

/// Review finding 1: relays of duplicated handles to one child-sealed
/// section, each asking `immutable`, make one copy and one charge; every
/// receiver gets that copy.
#[test]
fn duplicates_share_one_copy_and_one_charge() {
    let r = rig();
    let owner = OsPeer::new(80, 0);
    let ro = r
        .ipc
        .seal_region(r.ipc.allocate_region(4096, owner).unwrap())
        .unwrap();
    mark_child_sealed(&r, ro.handle());
    let s = section(&r, ro.handle());
    let (tx, rx) = r.ipc.channel(16, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    let mut got = Vec::new();
    for _ in 0..5 {
        let dup = super::sys::narrow(&s, 0x4).unwrap();
        let h = {
            let mut st = r.ipc.inner.lock();
            r.ipc
                .region_handle(&mut st, dup, HandleKind::ReadOnlyRegion, 4096, false)
                .unwrap()
        };
        let dup = ReadOnlyRegion::from_handle(h).unwrap();
        dup.want_immutable();
        r.ipc
            .send(&tx, b"d".to_vec(), vec![dup.into_handle()])
            .unwrap();
        let (_, mut hs) = expect_msg(&out);
        got.push(ReadOnlyRegion::from_handle(hs.remove(0)).unwrap());
    }
    assert!(got.iter().all(ReadOnlyRegion::immutable));
    let st = r.ipc.inner.lock();
    let b = st.broker.as_ref().unwrap();
    // The source and its one copy.
    assert_eq!(b.regions.values().filter(|e| e.owner == owner).count(), 2);
    drop(st);
    // Two charges: 254 more fit, then none.
    let mut st = r.ipc.inner.lock();
    let b = st.broker.as_mut().unwrap();
    for _ in 2..MAX_REGIONS_PER_OWNER {
        assert!(b.quota.charge(owner, 4096));
    }
    assert!(!b.quota.charge(owner, 4096));
}

/// A relay that needs a copy another relay is still making waits for it
/// (planning says Wait, and charges nothing), then reuses it.
#[test]
fn a_copy_in_progress_is_waited_for_not_refused() {
    let r = rig();
    let owner = OsPeer::new(82, 0);
    let ro = r
        .ipc
        .seal_region(r.ipc.allocate_region(4096, owner).unwrap())
        .unwrap();
    mark_child_sealed(&r, ro.handle());
    let h = section(&r, ro.handle());
    let facts = super::sys::section_facts(&h);
    let mut st = r.ipc.inner.lock();
    let b = st.broker.as_mut().unwrap();
    let item = || {
        core::iter::once((
            &*h,
            crate::ipc::common::codec::WireKind::ReadOnlyRegion,
            4096u32,
            1u8,
            facts,
        ))
    };
    // The relayer does not matter to the copy plan (a region relay comes
    // only from the broker or a child that may hold regions, finding 5;
    // these registry tests have no such child), so both relays are the
    // broker's: the wait is per section.
    let first = super::Inner::plan_copies(b, super::BROKER, item()).unwrap();
    assert_eq!(first.len(), 1);
    assert!(matches!(
        super::Inner::plan_copies(b, super::BROKER, item()),
        Err(super::Plan::Wait)
    ));
    // Once the first relay's copy is registered, the second plans no
    // copy of its own: it will reuse that one.
    let (mut made, failed) = super::Inner::make_copies(first);
    assert!(failed.is_empty());
    let c = made.remove(0);
    let src = c.src;
    let (copy_id, _ro) = super::Inner::register_copy(b, c);
    let e = b.regions.get_mut(&src).unwrap();
    e.copy = Some(copy_id);
    e.copying = false;
    assert!(
        super::Inner::plan_copies(b, super::BROKER, item())
            .unwrap()
            .is_empty()
    );
}

/// Review finding 2: a copy that cannot be made is refunded and leaves
/// nothing registered (the source is not marked as being copied).
#[test]
fn a_failed_copy_is_refunded_and_not_registered() {
    let r = rig();
    let owner = OsPeer::new(81, 0);
    let ro = r
        .ipc
        .seal_region(r.ipc.allocate_region(4096, owner).unwrap())
        .unwrap();
    mark_child_sealed(&r, ro.handle());
    let h = section(&r, ro.handle());
    let facts = super::sys::section_facts(&h);
    let mut st = r.ipc.inner.lock();
    let b = st.broker.as_mut().unwrap();
    let item = core::iter::once((
        &*h,
        crate::ipc::common::codec::WireKind::ReadOnlyRegion,
        4096u32,
        1u8,
        facts,
    ));
    let mut jobs = super::Inner::plan_copies(b, super::BROKER, item).unwrap();
    assert_eq!(jobs.len(), 1);
    assert!(b.regions.values().any(|e| e.copying));
    // A source handle that cannot be mapped for reading (write access
    // only): the copy fails.
    let other = super::sys::create_section(4096).unwrap();
    jobs[0].from = super::sys::narrow(&other, 0x2).unwrap();
    let (made, failed) = super::Inner::make_copies(jobs);
    assert!(made.is_empty());
    assert_eq!(failed.len(), 1);
    super::Inner::release_jobs(b, failed);
    assert_eq!(b.regions.values().filter(|e| e.owner == owner).count(), 1);
    assert!(b.regions.values().all(|e| !e.copying && e.copy.is_none()));
    // Refunded: the owner holds one region; 255 more fit, then none.
    for _ in 1..MAX_REGIONS_PER_OWNER {
        assert!(b.quota.charge(owner, 4096));
    }
    assert!(!b.quota.charge(owner, 4096));
}

// ---- W3: a fresh pipe per move -------------------------------------------

/// The child's report of its stale read: the `OVERLAPPED` status and the
/// bytes it got.
fn stale_report(c: &Kid) -> (String, Vec<u8>) {
    let (b, _) = expect_msg(&c.from_child);
    let colon = b.iter().position(|x| *x == b':').unwrap();
    (
        String::from_utf8(b[..colon].to_vec()).unwrap(),
        b[colon + 1..].to_vec(),
    )
}

/// W3 (probes W10, W11): a compromised holder leaves a read pending on a
/// receiver end and gives the end away. Everything sent after the move
/// goes to the new holder; the stale read gets nothing (the old pipe's
/// writer leaves it, so it fails with the broken pipe status). Before
/// the fresh pipe per move, the stale read took the first message sent
/// after the move and the new holder never saw it.
#[test]
fn an_old_holders_pending_read_gets_nothing_after_a_move() {
    let r = rig();
    let c = launch(&r, "stale-reader", true);
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&c.to_child, b"take".to_vec(), vec![rx.into_handle()])
        .unwrap();
    let (b, mut hs) = expect_msg(&c.from_child);
    assert_eq!(b, b"back");
    let rx = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
    let (out, _l) = r.listen(rx);
    r.ipc.send(&tx, b"after".to_vec(), Vec::new()).unwrap();
    assert_eq!(expect_msg(&out).0, b"after", "the new holder hears it");
    let (status, got) = stale_report(&c);
    assert!(
        !got.windows(5).any(|w| w == b"after"),
        "the old holder's read took a record sent after the move ({status})"
    );
    // STATUS_PIPE_BROKEN: the read ended when the old pipe's writer left.
    assert_eq!(status, "0xc000014b");
}

/// W3 (probe W13c): a stale read with a buffer shorter than a message
/// takes its head; the rest reaches the new holder as a message of its
/// own. The payload here is shaped as an attacker would: a whole message
/// with the expected `seq`, cut just after the outer ticket and header.
/// Without the trailer the new holder accepted that suffix as a record;
/// now it is refused as Malformed.
#[test]
fn a_record_split_by_a_stale_read_is_refused() {
    use crate::ipc::common::codec::encode_data;
    use crate::ipc::win::frame::{NO_TICKET, join_data};
    let r = rig();
    let c = launch(&r, "stale-split", true);
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&c.to_child, b"take".to_vec(), vec![rx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&c.from_child).0, b"armed");
    let forged = join_data(&NO_TICKET, &encode_data(0, &[], b"forged").unwrap());
    r.ipc.send(&tx, forged, Vec::new()).unwrap();
    let (b, mut hs) = expect_msg(&c.from_child);
    assert_eq!(b, b"back");
    let rx = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
    let (out, _l) = r.listen(rx);
    match next(&out) {
        Ev::Fault(f) => {
            assert_eq!(f.kind, FaultKind::Malformed);
            assert_eq!(f.sender, None);
        }
        other => panic!("expected a Malformed fault, got {other:?}"),
    }
    let (status, got) = stale_report(&c);
    assert_eq!(status, "0x80000005", "STATUS_BUFFER_OVERFLOW");
    assert_eq!(got.len(), 32, "the head of the message");
}

/// W3 (probe W13a): a stale read that takes a whole record sent before
/// the move leaves a gap in the sequence, which the new holder reports.
#[test]
fn a_record_taken_by_a_stale_read_shows_as_a_gap() {
    let r = rig();
    let c = launch(&r, "stale-whole", true);
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&c.to_child, b"take".to_vec(), vec![rx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&c.from_child).0, b"armed");
    r.ipc.send(&tx, b"one".to_vec(), Vec::new()).unwrap();
    r.ipc.send(&tx, b"two".to_vec(), Vec::new()).unwrap();
    let (b, mut hs) = expect_msg(&c.from_child);
    assert_eq!(b, b"back");
    let rx = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
    let (out, _l) = r.listen(rx);
    match next(&out) {
        Ev::Fault(f) => {
            assert_eq!(f.kind, FaultKind::Sequence);
            assert_eq!(f.sender, None);
        }
        other => panic!("expected a Sequence fault, got {other:?}"),
    }
    let (status, got) = stale_report(&c);
    assert_eq!(status, "0x0");
    assert!(got.windows(3).any(|w| w == b"one"));
}

/// W3: records queued on a channel stay in order across a chain of moves
/// of both its ends while nobody listens: each move leaves an old pipe,
/// read in turn when the receiver is finally listened to.
#[test]
fn order_holds_across_moves_with_records_queued() {
    let r = rig();
    let c = launch(&r, "bounce", true);
    let (mut tx, mut rx) = r.ipc.channel(64, Transfer::Free).unwrap();
    let mut n = 0u8;
    let mut send_two = |tx: &ChannelSender| {
        for _ in 0..2 {
            r.ipc.send(tx, vec![n], Vec::new()).unwrap();
            n += 1;
        }
    };
    for _ in 0..3 {
        send_two(&tx);
        // The sender goes away and back: the unlistened receiver here is
        // moved to a new pipe twice, its old ones queued.
        r.ipc
            .send(&c.to_child, b"tx".to_vec(), vec![tx.into_handle()])
            .unwrap();
        let (_, mut hs) = expect_msg(&c.from_child);
        tx = ChannelSender::from_handle(hs.remove(0)).unwrap();
        send_two(&tx);
        // Then the receiver itself, unread.
        r.ipc
            .send(&c.to_child, b"rx".to_vec(), vec![rx.into_handle()])
            .unwrap();
        let (_, mut hs) = expect_msg(&c.from_child);
        rx = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
    }
    send_two(&tx);
    let (out, _l) = r.listen(rx);
    for i in 0..n {
        assert_eq!(expect_msg(&out).0, vec![i]);
    }
    drop(tx);
    assert!(matches!(next(&out), Ev::Closed));
}

/// W3: both ends of one channel, held by two children, moved back at
/// once. The broker settles one move, then the other; the channel works
/// afterwards and nothing queued before is lost.
#[test]
fn both_ends_moving_at_once_settle() {
    let r = rig();
    let a = launch(&r, "bounce-on-go", true);
    let b = launch(&r, "bounce-on-go", true);
    for round in 0u8..5 {
        let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
        r.ipc.send(&tx, vec![round, 0], Vec::new()).unwrap();
        r.ipc
            .send(&a.to_child, b"tx".to_vec(), vec![tx.into_handle()])
            .unwrap();
        r.ipc
            .send(&b.to_child, b"rx".to_vec(), vec![rx.into_handle()])
            .unwrap();
        r.ipc.send(&a.to_child, b"go".to_vec(), Vec::new()).unwrap();
        r.ipc.send(&b.to_child, b"go".to_vec(), Vec::new()).unwrap();
        let (_, mut ht) = expect_msg(&a.from_child);
        let (_, mut hr) = expect_msg(&b.from_child);
        let tx = ChannelSender::from_handle(ht.remove(0)).unwrap();
        let rx = ChannelReceiver::from_handle(hr.remove(0)).unwrap();
        let (out, _l) = r.listen(rx);
        r.ipc.send(&tx, vec![round, 1], Vec::new()).unwrap();
        assert_eq!(expect_msg(&out).0, vec![round, 0]);
        assert_eq!(expect_msg(&out).0, vec![round, 1]);
    }
}

/// W3: a sender moved while full is not stuck: its new holder gets the
/// receiver's credit on the new pipe (a credit left in the old one is
/// superseded by the receiver's current count).
#[test]
fn a_moved_full_sender_gets_its_credit_on_the_new_pipe() {
    let r = rig();
    let c = launch(&r, "send-when-free", true);
    let (tx, rx) = r.ipc.channel(1, Transfer::Free).unwrap();
    r.ipc.send(&tx, b"a".to_vec(), Vec::new()).unwrap();
    assert_eq!(
        r.ipc.send(&tx, b"x".to_vec(), Vec::new()),
        Err(IpcError::Full)
    );
    r.ipc
        .send(&c.to_child, b"tx".to_vec(), vec![tx.into_handle()])
        .unwrap();
    let (out, _l) = r.listen(rx);
    assert_eq!(expect_msg(&out).0, b"a");
    assert_eq!(expect_msg(&out).0, b"b");
    assert_eq!(expect_msg(&c.from_child).0, b"sent");
}

/// W3: a channel keeps at most 16 old pipes for its receiver. A receiver
/// moved 16 times without anyone listening has 16 queued; the 17th move
/// is refused, with its reason recorded, and the channel ends in Closed.
#[test]
fn the_seventeenth_queued_old_pipe_closes_the_channel() {
    use crate::ipc::win::accept::Refusal;
    use crate::ipc::win::ledger::MAX_OLD_PIPES;
    let r = rig();
    let (tx, mut rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (ctx, crx) = r.ipc.channel(64, Transfer::Free).unwrap();
    let (carried, _cl) = r.listen(crx);
    for i in 0..MAX_OLD_PIPES {
        r.ipc
            .send(&ctx, vec![u8::try_from(i).unwrap()], vec![rx.into_handle()])
            .unwrap();
        let (_, mut hs) = expect_msg(&carried);
        rx = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
    }
    r.ipc
        .send(&ctx, b"17".to_vec(), vec![rx.into_handle()])
        .unwrap();
    assert!(
        carried.recv_timeout(Duration::from_secs(2)).is_err(),
        "the 17th move went through"
    );
    let refused = |code: u8| {
        r.ipc
            .inner
            .lock()
            .broker
            .as_ref()
            .and_then(|b| b.refused.get(&code).copied())
            .unwrap_or(0)
    };
    assert_eq!(refused(Refusal::OldPipes.code()), 1);
    // The carrier is closed for its sender, and the moved channel too.
    assert_eq!(
        r.ipc.send(&ctx, b"x".to_vec(), Vec::new()),
        Err(IpcError::Closed)
    );
    let deadline = std::time::Instant::now() + WAIT;
    loop {
        match r.ipc.send(&tx, b"x".to_vec(), Vec::new()) {
            Err(IpcError::Closed) => break,
            Ok(()) | Err(IpcError::Full) if std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(5));
            }
            other => panic!("{other:?}"),
        }
    }
}

/// W3: the old pipes a broker keeps for a receiver are released, with
/// their charge, when the channel closes.
#[test]
fn old_pipes_are_released_when_the_channel_closes() {
    let r = rig();
    let (tx, mut rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (ctx, crx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (carried, _cl) = r.listen(crx);
    for _ in 0..3 {
        r.ipc
            .send(&ctx, Vec::new(), vec![rx.into_handle()])
            .unwrap();
        let (_, mut hs) = expect_msg(&carried);
        rx = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
    }
    let old = || {
        r.ipc
            .inner
            .lock()
            .broker
            .as_ref()
            .map(|b| b.old.len())
            .unwrap()
    };
    assert_eq!(old(), 3);
    drop(rx);
    drop(tx);
    let released = (0..200).any(|_| {
        std::thread::sleep(Duration::from_millis(25));
        old() == 0
    });
    assert!(released, "the broker kept old pipes of a closed channel");
}

/// W3: the old pipes kept for a receiver are released when the process
/// holding that receiver dies.
#[test]
fn old_pipes_are_released_when_their_holder_dies() {
    let r = rig();
    let c = launch(&r, "take-and-hold", true);
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&c.to_child, b"rx".to_vec(), vec![rx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&c.from_child).0, b"held");
    let old = || {
        r.ipc
            .inner
            .lock()
            .broker
            .as_ref()
            .map(|b| b.old.len())
            .unwrap()
    };
    assert_eq!(old(), 1, "the receiver's first pipe, queued at its move");
    drop(c);
    let released = (0..200).any(|_| {
        std::thread::sleep(Duration::from_millis(25));
        old() == 0
    });
    assert!(released, "the broker kept a dead holder's old pipes");
    drop(tx);
}

/// W3: a holder that never answers a rebind cannot hold a move up past
/// the deadline: the move is refused, with its reason recorded, and the
/// carrying channel ends in Closed.
#[test]
fn a_peer_that_never_rebinds_fails_the_move_at_the_deadline() {
    use crate::ipc::win::accept::Refusal;
    let r = rig();
    let deaf = launch(&r, "deaf", true);
    let other = launch(&r, "bounce", true);
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&deaf.to_child, b"tx".to_vec(), vec![tx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&deaf.from_child).0, b"deaf");
    let t = std::time::Instant::now();
    r.ipc
        .send(&other.to_child, b"rx".to_vec(), vec![rx.into_handle()])
        .unwrap();
    let deadline = std::time::Instant::now() + WAIT;
    loop {
        match r.ipc.send(&other.to_child, b"x".to_vec(), Vec::new()) {
            Err(IpcError::Closed) => break,
            Ok(()) | Err(IpcError::Full) if std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            other => panic!("{other:?}"),
        }
    }
    let waited = t.elapsed();
    assert!(waited >= super::REBIND_DEADLINE, "refused after {waited:?}");
    let refused = r
        .ipc
        .inner
        .lock()
        .broker
        .as_ref()
        .and_then(|b| b.refused.get(&Refusal::Rebind.code()).copied());
    assert_eq!(refused, Some(1));
}

/// The W3 measurement: the time a child's listen waits for its
/// `TakeDrains` answer (a receiver with one old pipe queued), p50 and
/// p99 over 500 listens. Run with the D5 measurement, under an exclusive
/// booking.
#[test]
#[ignore = "a measurement, run by hand on the lab machine"]
fn measure_take_drains() {
    let r = rig();
    let c = launch(&r, "time-take", false);
    for _ in 0..500 {
        // A receiver moved once here and back... once to the child: its
        // first pipe is queued, so the child's listen asks for it.
        let (_tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
        r.ipc
            .send(&c.to_child, b"rx".to_vec(), vec![rx.into_handle()])
            .unwrap();
        assert_eq!(expect_msg(&c.from_child).0, b"next");
    }
    r.ipc
        .send(&c.to_child, b"done".to_vec(), Vec::new())
        .unwrap();
    let (b, _) = expect_msg(&c.from_child);
    let mut us: Vec<u64> = b
        .chunks(8)
        .map(|x| u64::from_le_bytes(x.try_into().unwrap()))
        .collect();
    us.sort_unstable();
    println!(
        "W3-TAKE listen with one old pipe: p50={}us p99={}us max={}us over {}",
        us[us.len() / 2],
        us[us.len() * 99 / 100],
        us[us.len() - 1],
        us.len()
    );
}

/// Probe W14: a file object's synchronous mode is fixed once it is
/// opened. Setting `FileModeInformation` neither makes an overlapped end
/// synchronous nor the reverse, so an end the broker peeked as overlapped
/// is still overlapped when it pulls the same object.
#[test]
fn a_file_objects_io_mode_is_fixed_once_opened() {
    let (server, client) = super::sys::pipe_pair(None, &[0x14; 16]).unwrap();
    let (sync_server, sync_client) = super::sys::sync_pipe().unwrap();
    for (h, sync) in [
        (&server, false),
        (&client, false),
        (&sync_server, true),
        (&sync_client, true),
    ] {
        let before = super::sys::file_mode(h).expect("the mode reads");
        assert_eq!(super::sys::synchronous(h), sync, "mode {before:#x}");
        let to = if sync { 0 } else { 0x20 };
        let status = super::sys::set_file_mode(h, to);
        let after = super::sys::file_mode(h).expect("the mode reads");
        eprintln!(
            "W14: sync={sync} mode {before:#x} set {to:#x} -> status {status:#x}, mode {after:#x}"
        );
        assert_eq!(super::sys::synchronous(h), sync, "mode {after:#x}");
    }
    // An overlapped end's queries do not wait behind a pending read
    // (unlike a synchronous one's, W8).
    let reader = super::sys::duplicate(&server).unwrap();
    let _pending = super::sys::stale_read(super::sys::value(&reader), 64, 3000);
    let t = std::time::Instant::now();
    let f = super::sys::facts(&server);
    let took = t.elapsed();
    eprintln!("W14: facts on an overlapped end with a read pending took {took:?}");
    assert!(f.is_pipe && f.server_end && !f.synchronous);
    assert!(took < Duration::from_millis(500));
    super::sys::cancel(&reader);
    drop(reader);
    drop(client);
}

/// A relay that finds a synchronous pipe end at the value it pulls, put
/// there after the peek, must not hold its channel: the kernel's queries
/// on such an end wait as long as its read is pending (probe W8). Another
/// holder's move of the same channel goes through once the first relay is
/// refused (review finding, W3).
#[test]
fn a_swapped_in_synchronous_end_does_not_stall_its_channel() {
    let r = rig();
    let holder = launch(&r, "deaf-holder", false);
    let swapper = launch(&r, "swap-sync", false);
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&swapper.to_child, b"tx".to_vec(), vec![tx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&swapper.from_child).0, b"have");
    r.ipc
        .send(&holder.to_child, b"rx".to_vec(), vec![rx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&holder.from_child).0, b"deaf");
    r.ipc
        .send(&swapper.to_child, b"swap".to_vec(), Vec::new())
        .unwrap();
    assert_eq!(expect_msg(&swapper.from_child).0, b"swapped");
    // The swapper's relay now waits for the deaf holder's rebind; the
    // holder's own move of the receiver queues behind it.
    r.ipc
        .send(&holder.to_child, b"go".to_vec(), Vec::new())
        .unwrap();
    let (b, hs) = expect_msg(&holder.from_child);
    assert_eq!(b, b"back");
    assert_eq!(hs.len(), 1);
}

/// An end whose file object does synchronous I/O is refused at the peek,
/// before its relay claims anything (review finding, W3).
#[test]
fn a_synchronous_end_is_refused_at_the_peek() {
    let r = rig();
    let c = launch(&r, "sync-relay", false);
    assert_eq!(expect_msg(&c.from_child).0, b"sent");
    let code = crate::ipc::win::accept::Refusal::Synchronous.code();
    let refused = (0..400).any(|_| {
        std::thread::sleep(Duration::from_millis(25));
        r.ipc
            .inner
            .lock()
            .broker
            .as_ref()
            .is_some_and(|b| b.refused.get(&code) == Some(&1))
    });
    let all = r.ipc.inner.lock().broker.as_ref().unwrap().refused.clone();
    assert!(
        refused,
        "the synchronous end was not refused at the peek: {all:?}"
    );
}

/// The cost of a move (W3): a sender end goes to a child and straight
/// back (two moves), with its receiver held by the broker (each move
/// switches the peer locally) or by another child (each move sends a
/// `Rebind` and waits for its `Rebound`). Run by hand on the lab machine.
#[test]
#[ignore = "a measurement, run by hand on the lab machine"]
fn measure_move() {
    let r = rig();
    let bounce = launch(&r, "bounce", false);
    let keep = launch(&r, "keep", false);
    let pct = |v: &mut Vec<u128>, q: usize| {
        v.sort_unstable();
        v[v.len() * q / 100] as f64 / 1000.0
    };
    // Warm up both children.
    r.ipc.send(&bounce.to_child, vec![0], Vec::new()).unwrap();
    let _ = expect_msg(&bounce.from_child);
    r.ipc.send(&keep.to_child, vec![0], Vec::new()).unwrap();
    let _ = expect_msg(&keep.from_child);
    let mut local = Vec::with_capacity(500);
    let mut remote = Vec::with_capacity(500);
    let mut held = Vec::new();
    for _ in 0..500 {
        for remote_peer in [false, true] {
            let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
            if remote_peer {
                r.ipc
                    .send(&keep.to_child, b"rx".to_vec(), vec![rx.into_handle()])
                    .unwrap();
                assert_eq!(expect_msg(&keep.from_child).0, b"kept");
            } else {
                held.push(rx);
            }
            let t = std::time::Instant::now();
            r.ipc
                .send(&bounce.to_child, b"tx".to_vec(), vec![tx.into_handle()])
                .unwrap();
            let (_b, hs) = expect_msg(&bounce.from_child);
            let ns = t.elapsed().as_nanos();
            assert_eq!(hs.len(), 1);
            if remote_peer {
                remote.push(ns);
            } else {
                local.push(ns);
            }
        }
    }
    println!(
        "W3-MOVE sender there and back (2 moves): peer in the broker p50={:.1}us p99={:.1}us | peer in a child p50={:.1}us p99={:.1}us | over {}",
        pct(&mut local, 50),
        pct(&mut local, 99),
        pct(&mut remote, 50),
        pct(&mut remote, 99),
        local.len()
    );
    drop(held);
}
