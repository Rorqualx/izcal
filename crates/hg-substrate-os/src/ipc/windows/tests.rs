//! The Windows transport's tests: in one process, then across processes
//! with children started (sandboxed, in their own AppContainers) by this
//! test binary acting as their broker.

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
        peer: OsPeer,
    ) -> (Receiver<Ev>, hg_substrate::ipc::Listening) {
        let (tx, out) = sync_channel::<Ev>(4096);
        let seq = self.exec.sequence("test", Priority::UserVisible);
        let l = self
            .ipc
            .listen_bound(rx, peer, seq, sink(tx))
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
    let Some(link) = std::env::var(super::LINK_VAR)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .and_then(|v| super::sys::adopt_pipe(v, false))
    else {
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
        _ => 14,
    }
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
                sandbox: Some((&sid, &token, &job)),
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
                sandbox: None,
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
        sandbox: Some((&sid, &token, &job)),
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
/// exclusive booking with `--ignored --nocapture`; prints one line.
#[test]
#[ignore = "a measurement, run by hand on the lab machine"]
fn measure_round_trip_and_throughput() {
    let r = rig();
    let c = launch(&r, "echo", false);
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
    use std::collections::{BTreeMap, BTreeSet};

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
    let mut b = Broker {
        ledger: Ledger::default(),
        held: BTreeMap::new(),
        children: BTreeMap::new(),
        reserved_sids: BTreeSet::new(),
        creator_sids: BTreeMap::new(),
        next_proc: 9,
    };
    // Child 7 made the channel, then died: it is gone from `children`,
    // but its SID is still mapped because the ledger holds its channel.
    b.creator_sids.insert(sid.to_owned(), 7);
    let key = Inner::key_of(&b, &facts).expect("the dead creator's channel is still keyed");
    assert_eq!(key.creator, 7);
    assert_ne!(key.creator, BROKER);
}
