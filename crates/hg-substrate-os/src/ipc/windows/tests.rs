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
        "region-roundtrip" | "region-forge" => {
            region_child(case.trim_start_matches("sandboxed-"), &r, &up, &from_parent)
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
        given: BTreeMap::new(),
        own: BTreeMap::new(),
        next_proc: 9,
        regions: std::collections::BTreeMap::new(),
        next_region: 0,
        quota: crate::ipc::common::quota::Ledger::default(),
    };
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
    use std::collections::{BTreeMap, BTreeSet};

    let mut b = Broker {
        ledger: Ledger::default(),
        held: BTreeMap::new(),
        children: BTreeMap::new(),
        reserved_sids: BTreeSet::new(),
        creator_sids: BTreeMap::new(),
        given: BTreeMap::new(),
        own: BTreeMap::new(),
        next_proc: 9,
        regions: std::collections::BTreeMap::new(),
        next_region: 0,
        quota: crate::ipc::common::quota::Ledger::default(),
    };
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
    let base = r
        .ipc
        .inner
        .lock()
        .broker
        .as_ref()
        .map(|b| b.ledger.entry_count())
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
            .is_some_and(|b| b.ledger.entry_count() <= base && b.given.is_empty())
    });
    let (ec, gl) = {
        let st = r.ipc.inner.lock();
        let b = st.broker.as_ref().unwrap();
        (b.ledger.entry_count(), b.given.len())
    };
    assert!(
        forgotten,
        "the broker kept a closed channel: base={base} entries={ec} given={gl}"
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
