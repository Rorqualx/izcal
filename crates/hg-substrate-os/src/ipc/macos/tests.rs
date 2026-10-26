//! Medium tests of the macOS transport: one process, then a child
//! process (this test binary started again with `IZCAL_D4_CHILD=<case>`
//! and its bootstrap port set to a port of ours, as phase E's launcher
//! will), then hostile messages the transport itself never sends.

use std::ffi::CString;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use hg_substrate::ipc::{
    ChannelReceiver, ChannelSender, Delivery, Fault, FaultKind, Flow, Handle, HandleKind, Ipc,
    IpcError, MAX_MESSAGE_BYTES, OsPeer, Transfer,
};
use hg_substrate::{Executor, Priority};

use super::sys::{self, test_support};
use super::{MacEnd, MacIpc, Role};
use crate::executor::OsExecutor;
use crate::ipc::common::codec::{HandleDesc, WireKind, encode_data};
use crate::ipc::mach::wire::{
    self, BITS_COMPLEX, COPY_SEND, DESC_OOL, ID_DATA, MAKE_SEND_ONCE, MOVE_RECEIVE, NAME_NULL,
    Outgoing,
};

const WAIT: Duration = Duration::from_secs(20);

/// What a test listener saw.
#[derive(Debug)]
enum Ev {
    Msg(Vec<u8>, Vec<Handle>),
    Closed,
    Fault(Fault),
}

struct Rig {
    exec: Arc<OsExecutor>,
    ipc: MacIpc,
}

fn rig() -> Rig {
    rig_with(false)
}

fn rig_with(launched: bool) -> Rig {
    let exec = Arc::new(OsExecutor::new(2, 1).unwrap());
    let ipc = MacIpc::start(Arc::clone(&exec), launched).unwrap();
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

    /// The data and credit port names behind one of this transport's
    /// sender ends (tests only).
    fn sender_names(&self, h: &Handle) -> (u32, u32) {
        let e = h.object().downcast_ref::<MacEnd>().unwrap();
        match self.ipc.inner.lock().ends.get(&e.id).unwrap() {
            Role::Sender { data, credit, .. } => (*data, *credit),
            Role::Receiver { .. } => panic!("not a sender"),
        }
    }

    /// The data port name behind one of this transport's sender ends
    /// (tests only: to send messages the transport itself never would).
    fn data_name(&self, h: &Handle) -> u32 {
        let e = h.object().downcast_ref::<MacEnd>().unwrap();
        match self.ipc.inner.lock().ends.get(&e.id).unwrap() {
            Role::Sender { data, .. } | Role::Receiver { data, .. } => *data,
        }
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

fn expect_fault(out: &Receiver<Ev>, kind: FaultKind, sender: Option<u32>) {
    match next(out) {
        Ev::Fault(f) => assert_eq!((f.kind, f.sender), (kind, sender), "{f:?}"),
        other => panic!("expected a fault, got {other:?}"),
    }
}

fn me() -> u32 {
    std::process::id()
}

fn desc(kind: WireKind) -> HandleDesc {
    HandleDesc {
        kind,
        capacity: 8,
        sent: 0,
        delivered: 0,
    }
}

// ---- one process ----

#[test]
fn messages_arrive_in_order_with_their_handles() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(16, Transfer::Free).unwrap();
    let (tx2, rx2) = r.ipc.channel(16, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    for i in 0u8..5 {
        r.ipc.send(&tx, vec![i; 3], Vec::new()).unwrap();
    }
    r.ipc
        .send(&tx, b"carry".to_vec(), vec![tx2.into_handle()])
        .unwrap();
    for i in 0u8..5 {
        assert_eq!(expect_msg(&out).0, vec![i; 3]);
    }
    let (bytes, mut handles) = expect_msg(&out);
    assert_eq!(bytes, b"carry");
    assert_eq!(handles.len(), 1);
    assert_eq!(handles[0].kind(), HandleKind::ChannelSender);
    // The received sender works and reaches the original receiver, here
    // in the same process (its receive right and the received send right
    // share one name, P32).
    let got = ChannelSender::from_handle(handles.remove(0)).unwrap();
    let (out2, _l2) = r.listen(rx2);
    r.ipc.send(&got, b"via".to_vec(), Vec::new()).unwrap();
    assert_eq!(expect_msg(&out2).0, b"via");
}

#[test]
fn full_is_exact_at_capacity() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(4, Transfer::Free).unwrap();
    for _ in 0..4 {
        r.ipc.send(&tx, b"x".to_vec(), Vec::new()).unwrap();
    }
    assert_eq!(
        r.ipc.send(&tx, b"x".to_vec(), Vec::new()),
        Err(IpcError::Full)
    );
    let (out, _l) = r.listen(rx);
    for _ in 0..4 {
        expect_msg(&out);
    }
    // Credits went back as each delivery started: room again.
    r.ipc.send(&tx, b"y".to_vec(), Vec::new()).unwrap();
    assert_eq!(expect_msg(&out).0, b"y");
}

/// The queue limit is the capacity, up to 1024 (P16): above that, the
/// rest wait in the outbox, and `Full` comes only at the capacity.
#[test]
fn bursts_wait_in_the_outbox_never_spuriously_full() {
    let r = rig();
    for (cap, size) in [(64u32, MAX_MESSAGE_BYTES), (1500, 64)] {
        let (tx, rx) = r.ipc.channel(cap, Transfer::Free).unwrap();
        let body = vec![7u8; size];
        for i in 0..cap {
            let mut m = body.clone();
            m[0] = u8::try_from(i % 251).unwrap();
            r.ipc
                .send(&tx, m, Vec::new())
                .unwrap_or_else(|e| panic!("send {i}: {e}"));
        }
        assert_eq!(
            r.ipc.send(&tx, b"x".to_vec(), Vec::new()),
            Err(IpcError::Full)
        );
        let (out, _l) = r.listen(rx);
        for i in 0..cap {
            let (b, _) = expect_msg(&out);
            assert_eq!(b.len(), size);
            assert_eq!(b[0], u8::try_from(i % 251).unwrap(), "order at {i}");
        }
    }
}

/// Rule M3: however many messages wait undelivered, the receiver holds
/// one reply right to the sender's credit port, not one per message.
#[test]
fn a_receiver_holds_one_reply_right_per_channel() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(64, Transfer::Free).unwrap();
    let (data, credit) = r.sender_names(tx.handle());
    let (gate_tx, gate_rx) = sync_channel::<()>(64);
    let gate = Mutex::new(gate_rx);
    let seq = r.exec.sequence("blocked", Priority::UserVisible);
    let _l = r
        .ipc
        .listen(
            rx,
            seq,
            Box::new(move |_d| {
                let _ = gate.lock().unwrap().recv_timeout(WAIT);
                Flow::Continue
            }),
        )
        .unwrap();
    for i in 0u8..64 {
        r.ipc.send(&tx, vec![i], Vec::new()).unwrap();
    }
    // Wait for the IO thread to have read all 64 off the data port.
    let until = std::time::Instant::now() + WAIT;
    while sys::receive_status(data).unwrap()[4] != 0 {
        assert!(std::time::Instant::now() < until, "not read in time");
        std::thread::sleep(Duration::from_millis(1));
    }
    // A send-once right counts until its message is dequeued (R8b): the
    // released replies' notifications wait on the credit port, so the
    // rights still held are the count less the messages queued there.
    let status = sys::receive_status(credit).unwrap();
    let held = status[wire::STATUS_SEND_ONCE] - status[wire::STATUS_MSGCOUNT];
    assert!(held <= 1, "{held} reply rights held ({status:?})");
    for _ in 0..64 {
        let _ = gate_tx.send(());
    }
}

#[test]
fn closed_comes_after_every_queued_message() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(16, Transfer::Free).unwrap();
    for i in 0u8..5 {
        r.ipc.send(&tx, vec![i], Vec::new()).unwrap();
    }
    drop(tx);
    let (out, _l) = r.listen(rx);
    for i in 0u8..5 {
        assert_eq!(expect_msg(&out).0, vec![i]);
    }
    assert!(matches!(next(&out), Ev::Closed));
}

/// Rights are never inherited by a child, so there is no spawn window:
/// the first send after the receiver is gone reports `Closed`.
#[test]
fn a_dropped_receiver_closes_the_sender_at_once() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(4, Transfer::Free).unwrap();
    drop(rx);
    assert_eq!(
        r.ipc.send(&tx, b"x".to_vec(), Vec::new()),
        Err(IpcError::Closed)
    );
    assert_eq!(
        r.ipc.send(&tx, b"y".to_vec(), Vec::new()),
        Err(IpcError::Closed)
    );
}

#[test]
fn bound_senders_move_once_and_bound_receivers_never() {
    let r = rig();
    let (carrier, _crx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (btx, brx) = r.ipc.channel(8, Transfer::Bound).unwrap();
    assert_eq!(btx.handle().kind(), HandleKind::BoundChannelSender);
    assert_eq!(
        r.ipc.send(&carrier, Vec::new(), vec![brx.into_handle()]),
        Err(IpcError::Refused)
    );
    r.ipc
        .send(&carrier, b"grant".to_vec(), vec![btx.into_handle()])
        .unwrap();
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
    let (_tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    assert!(
        r.ipc
            .listen_bound(rx, OsPeer::new(me(), 1), seq(), nothing())
            .is_ok()
    );
    let (_btx, brx) = r.ipc.channel(8, Transfer::Bound).unwrap();
    assert_eq!(
        r.ipc
            .listen_bound(brx, OsPeer::new(0, 0), seq(), nothing())
            .err(),
        Some(IpcError::Refused)
    );
}

#[test]
fn a_channel_cannot_carry_its_own_end() {
    let r = rig();
    let (tx, _rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let id = tx.handle().object().downcast_ref::<MacEnd>().unwrap().id;
    let same = Handle::new(Box::new(MacEnd {
        inner: Arc::clone(&r.ipc.inner),
        id,
        kind: HandleKind::ChannelSender,
        armed: false,
    }));
    assert_eq!(
        r.ipc.send(&tx, Vec::new(), vec![same]),
        Err(IpcError::Refused)
    );
}

#[test]
fn a_transfer_waits_for_the_carried_sender_s_queue() {
    let r = rig();
    // Above the 1024 queue limit, so the outbox holds the rest.
    let (tx, rx) = r.ipc.channel(1500, Transfer::Free).unwrap();
    for i in 0u32..1100 {
        r.ipc
            .send(&tx, i.to_le_bytes().to_vec(), Vec::new())
            .unwrap();
    }
    let (carrier, crx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&carrier, b"take".to_vec(), vec![tx.into_handle()])
        .unwrap();
    let (out, _l) = r.listen(rx);
    let (cout, _cl) = r.listen(crx);
    let (_, mut hs) = expect_msg(&cout);
    let moved = ChannelSender::from_handle(hs.remove(0)).unwrap();
    r.ipc.send(&moved, b"after".to_vec(), Vec::new()).unwrap();
    for i in 0u32..1100 {
        assert_eq!(expect_msg(&out).0, i.to_le_bytes(), "order at {i}");
    }
    assert_eq!(expect_msg(&out).0, b"after");
}

#[test]
fn a_transfer_gives_up_on_an_unread_queue_at_its_deadline() {
    let r = rig();
    let (tx, _rx) = r.ipc.channel(1500, Transfer::Free).unwrap();
    for i in 0u32..1100 {
        r.ipc
            .send(&tx, i.to_le_bytes().to_vec(), Vec::new())
            .unwrap();
    }
    let (carrier, crx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (cout, _cl) = r.listen(crx);
    r.ipc
        .send(&carrier, b"take".to_vec(), vec![tx.into_handle()])
        .unwrap();
    // Nobody reads the first channel: after the deadline the carrier goes
    // with a stand-in end whose channel closes at its first message.
    let (_, mut hs) = expect_msg(&cout);
    let moved = ChannelSender::from_handle(hs.remove(0)).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        match r.ipc.send(&moved, b"x".to_vec(), Vec::new()) {
            Err(IpcError::Closed) => break,
            Ok(()) if std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(1));
            }
            other => panic!("expected Closed, got {other:?}"),
        }
    }
}

/// The transfer deadline when no stand-in can be built: the carried end
/// goes as it is, without its queued messages, and its receiver sees the
/// gap (finding 3: its rights were released while the waiting message
/// still named them, and the carrier was lost).
#[test]
fn a_transfer_whose_stand_in_fails_still_carries_the_real_end() {
    let r = rig();
    r.ipc
        .inner
        .fail_stand_in
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let (tx, rx) = r.ipc.channel(1500, Transfer::Free).unwrap();
    for i in 0u32..1100 {
        r.ipc
            .send(&tx, i.to_le_bytes().to_vec(), Vec::new())
            .unwrap();
    }
    let (carrier, crx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (cout, _cl) = r.listen(crx);
    r.ipc
        .send(&carrier, b"take".to_vec(), vec![tx.into_handle()])
        .unwrap();
    let (_, mut hs) = expect_msg(&cout);
    let moved = ChannelSender::from_handle(hs.remove(0)).unwrap();
    // The kernel queue held the first 1024 (P16); the rest were given up.
    let (out, _l) = r.listen(rx);
    for i in 0u32..1024 {
        assert_eq!(expect_msg(&out).0, i.to_le_bytes(), "message {i}");
    }
    r.ipc.send(&moved, b"x".to_vec(), Vec::new()).unwrap();
    expect_fault(&out, FaultKind::Sequence, None);
}

/// Finding 8: a received buffer's rights are released once, and only a
/// received buffer's.
#[test]
fn a_received_message_is_released_once() {
    let dest = test_support::port_with_send();
    let p = test_support::port_with_send();
    let refs = || {
        sys::slot_facts(wire::Desc::Port {
            name: p,
            disposition: wire::PORT_SEND,
        })
        .send_refs
    };
    test_support::send_raw(&Outgoing {
        dest,
        dest_disposition: COPY_SEND,
        reply: 0,
        reply_disposition: 0,
        id: 99,
        ports: &[(p, COPY_SEND)],
        body: &[0; 4],
    })
    .unwrap();
    // Unreceived, the buffer holds nothing.
    let mut buf = sys::Buf::new(wire::RECEIVE_BUFFER);
    sys::release_received(&buf);
    assert_eq!(refs(), 1);
    sys::receive_now(dest, &mut buf).unwrap();
    assert_eq!(refs(), 2, "the carried copy merged into our name (R5)");
    sys::release_received(&buf);
    sys::release_received(&buf);
    assert_eq!(refs(), 1);
    sys::destroy_receive(p);
    sys::destroy_receive(dest);
}

// ---- hostile messages ----

/// A data message from this process to `dest` with the given rights and
/// claimed table.
fn raw(dest: u32, ports: &[(u32, u8)], claimed: &[WireKind], seq: u64) -> Result<(), sys::Kr> {
    let descs: Vec<HandleDesc> = claimed.iter().map(|&k| desc(k)).collect();
    let body = wire::frame(&encode_data(seq, &descs, b"raw").unwrap()).unwrap();
    let reply = test_support::port_with_send();
    test_support::send_raw(&Outgoing {
        dest,
        dest_disposition: COPY_SEND,
        reply,
        reply_disposition: MAKE_SEND_ONCE,
        id: ID_DATA,
        ports,
        body: &body,
    })
}

/// A listening channel and its data port's name.
fn victim(
    r: &Rig,
) -> (
    ChannelSender,
    Receiver<Ev>,
    hg_substrate::ipc::Listening,
    u32,
) {
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let d = r.data_name(tx.handle());
    let (out, l) = r.listen(rx);
    (tx, out, l, d)
}

#[test]
fn malformed_messages_are_violations_named_by_the_kernel() {
    let r = rig();
    // Not our id.
    let (_tx, out, _l, d) = victim(&r);
    let body = encode_data(0, &[], b"x").unwrap();
    test_support::send_raw(&Outgoing {
        dest: d,
        dest_disposition: COPY_SEND,
        reply: test_support::port_with_send(),
        reply_disposition: MAKE_SEND_ONCE,
        id: 77,
        ports: &[],
        body: &body,
    })
    .unwrap();
    expect_fault(&out, FaultKind::Malformed, Some(me()));
    // No reply right.
    let (_tx, out, _l, d) = victim(&r);
    test_support::send_raw(&Outgoing {
        dest: d,
        dest_disposition: COPY_SEND,
        reply: NAME_NULL,
        reply_disposition: 0,
        id: ID_DATA,
        ports: &[],
        body: &body,
    })
    .unwrap();
    expect_fault(&out, FaultKind::Malformed, Some(me()));
    // A body that is not a record.
    let (_tx, out, _l, d) = victim(&r);
    test_support::send_raw(&Outgoing {
        dest: d,
        dest_disposition: COPY_SEND,
        reply: test_support::port_with_send(),
        reply_disposition: MAKE_SEND_ONCE,
        id: ID_DATA,
        ports: &[],
        body: b"junk",
    })
    .unwrap();
    expect_fault(&out, FaultKind::Malformed, Some(me()));
    // Sent on a send-once right made from the receive right (no peer is
    // ever given one).
    let (_tx, out, _l, d) = victim(&r);
    test_support::send_raw(&Outgoing {
        dest: d,
        dest_disposition: MAKE_SEND_ONCE,
        reply: test_support::port_with_send(),
        reply_disposition: MAKE_SEND_ONCE,
        id: ID_DATA,
        ports: &[],
        body: &body,
    })
    .unwrap();
    expect_fault(&out, FaultKind::Malformed, Some(me()));
}

#[test]
fn out_of_line_memory_is_refused_and_released() {
    let r = rig();
    let (_tx, out, _l, d) = victim(&r);
    let region = [9u8; 64];
    let body = encode_data(0, &[], b"x").unwrap();
    let reply = test_support::port_with_send();
    let mut m = Vec::new();
    let size = 24 + 4 + 16 + body.len().next_multiple_of(4);
    for w in [
        u32::from(COPY_SEND) | (u32::from(MAKE_SEND_ONCE) << 8) | BITS_COMPLEX,
        u32::try_from(size).unwrap(),
        d,
        reply,
        0,
        ID_DATA,
        1,
    ] {
        m.extend_from_slice(&w.to_le_bytes());
    }
    let addr = region.as_ptr() as u64;
    m.extend_from_slice(&(addr as u32).to_le_bytes());
    m.extend_from_slice(&((addr >> 32) as u32).to_le_bytes());
    m.extend_from_slice(&(u32::from(DESC_OOL) << 24).to_le_bytes());
    m.extend_from_slice(&64u32.to_le_bytes());
    m.extend_from_slice(&body);
    m.resize(size, 0);
    test_support::send_bytes(&m).unwrap();
    expect_fault(&out, FaultKind::Malformed, Some(me()));
}

#[test]
fn kernel_objects_and_rights_already_held_are_refused() {
    let r = rig();
    // A kernel object where a data send right should be (P8, P9).
    let (_tx, out, _l, d) = victim(&r);
    let c = test_support::port_with_send();
    raw(
        d,
        &[(test_support::host_port(), COPY_SEND), (c, MOVE_RECEIVE)],
        &[WireKind::Sender],
        0,
    )
    .unwrap();
    expect_fault(&out, FaultKind::BadHandle, Some(me()));
    // A send right this process already held: its bootstrap port.
    let (_tx, out, _l, d) = victim(&r);
    let c = test_support::port_with_send();
    let boot = sys::bootstrap_port().unwrap();
    raw(
        d,
        &[(boot, COPY_SEND), (c, MOVE_RECEIVE)],
        &[WireKind::Sender],
        0,
    )
    .unwrap();
    expect_fault(&out, FaultKind::BadHandle, Some(me()));
    // One port's send right in two handles.
    let (_tx, out, _l, d) = victim(&r);
    let p = test_support::port_with_send();
    let (c1, c2) = (
        test_support::port_with_send(),
        test_support::port_with_send(),
    );
    raw(
        d,
        &[
            (p, COPY_SEND),
            (c1, MOVE_RECEIVE),
            (p, COPY_SEND),
            (c2, MOVE_RECEIVE),
        ],
        &[WireKind::Sender, WireKind::Sender],
        0,
    )
    .unwrap();
    expect_fault(&out, FaultKind::BadHandle, Some(me()));
    // A table that claims a sender end with one right.
    let (_tx, out, _l, d) = victim(&r);
    let p = test_support::port_with_send();
    raw(d, &[(p, COPY_SEND)], &[WireKind::Sender], 0).unwrap();
    expect_fault(&out, FaultKind::BadHandle, Some(me()));
}

#[test]
fn a_sequence_gap_is_a_violation_without_blame() {
    let r = rig();
    let (_tx, out, _l, d) = victim(&r);
    raw(d, &[], &[], 5).unwrap();
    expect_fault(&out, FaultKind::Sequence, None);
}

#[test]
fn a_bound_receiver_blames_its_peer_for_another_sender() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(8, Transfer::Bound).unwrap();
    let d = r.data_name(tx.handle());
    let (out, _l) = r.listen_bound(rx, OsPeer::new(4_000_000, 7));
    raw(d, &[], &[], 0).unwrap();
    expect_fault(&out, FaultKind::Identity, Some(4_000_000));
}

/// A message too large for the receive buffer is read anyway (the
/// kernel names no port for it, R1) and refused, blamed on its sender;
/// every other channel keeps working.
#[test]
fn an_oversized_message_ends_only_its_channel() {
    let r = rig();
    let (_tx, out, _l, d) = victim(&r);
    let (tx2, rx2) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out2, _l2) = r.listen(rx2);
    let huge = vec![1u8; wire::RECEIVE_BUFFER + 4096];
    test_support::send_raw(&Outgoing {
        dest: d,
        dest_disposition: COPY_SEND,
        reply: NAME_NULL,
        reply_disposition: 0,
        id: ID_DATA,
        ports: &[],
        body: &huge,
    })
    .unwrap();
    // The kernel destroys it and names only the port (R10): no blame.
    expect_fault(&out, FaultKind::Malformed, None);
    r.ipc.send(&tx2, b"still".to_vec(), Vec::new()).unwrap();
    assert_eq!(expect_msg(&out2).0, b"still");
}

// ---- cross-process ----

/// The child side, run by the test binary started again with
/// `IZCAL_D4_CHILD=<case>`; a no-op in a normal run.
#[test]
fn d4_child() {
    let Some(case) = std::env::var_os("IZCAL_D4_CHILD") else {
        return;
    };
    let case = case.into_string().unwrap();
    let code = child(&case);
    std::process::exit(code);
}

fn child(case: &str) -> i32 {
    if case == "sandbox" {
        // Locked down first: the hello and everything after run inside.
        crate::sandbox::macos::apply_empty_process_profile().unwrap();
    }
    // A refused launch fails start (no hello reply): exit 10, quietly.
    let exec = Arc::new(OsExecutor::new(2, 1).unwrap());
    let Ok(ipc) = MacIpc::start(Arc::clone(&exec), true) else {
        return 10;
    };
    let r = Rig { exec, ipc };
    let Some(boot) = r.ipc.bootstrap() else {
        return 10;
    };
    let up = boot.to_parent;
    let (from_parent, _l) = r.listen_bound(boot.from_parent, boot.parent);
    match case {
        "burst-exit" => {
            for i in 0u8..5 {
                r.ipc.send(&up, vec![i], Vec::new()).unwrap();
            }
            0
        }
        "echo" => loop {
            match next(&from_parent) {
                Ev::Msg(b, hs) => {
                    for h in hs {
                        if let Ok(s) = ChannelSender::from_handle(h) {
                            r.ipc.send(&s, b"via-child".to_vec(), Vec::new()).unwrap();
                        }
                    }
                    r.ipc.send(&up, b, Vec::new()).unwrap();
                }
                _ => return 0,
            }
        },
        "use-grant" => {
            let Ev::Msg(_, mut hs) = next(&from_parent) else {
                return 11;
            };
            let grant = ChannelSender::from_handle(hs.remove(0)).unwrap();
            assert_eq!(grant.handle().kind(), HandleKind::BoundChannelSender);
            r.ipc
                .send(&grant, b"from-child".to_vec(), Vec::new())
                .unwrap();
            r.ipc.send(&up, b"done".to_vec(), Vec::new()).unwrap();
            let _ = next(&from_parent);
            0
        }
        // A hostile child: its own credit port, whose reply right this
        // very message carries, passed off as a receiver end.
        "credit-as-receiver" => {
            let (data, credit) = r.sender_names(up.handle());
            let body =
                wire::frame(&encode_data(0, &[desc(WireKind::Receiver)], b"c").unwrap()).unwrap();
            let sent = test_support::send_raw(&Outgoing {
                dest: data,
                dest_disposition: COPY_SEND,
                reply: credit,
                reply_disposition: MAKE_SEND_ONCE,
                id: ID_DATA,
                ports: &[(credit, MOVE_RECEIVE)],
                body: &body,
            });
            if sent.is_err() {
                return 12;
            }
            // The raw send moved the credit port's receive right away
            // behind the transport's back: its end must not release that
            // name. Under Enhanced Security, releasing a name the task no
            // longer holds is fatal (EXC_GUARD INVALID_NAME, R13).
            super::disarm(up.into_handle());
            let _ = next(&from_parent);
            0
        }
        "sandbox" => {
            let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
            r.ipc
                .send(&up, b"chan".to_vec(), vec![rx.into_handle()])
                .unwrap();
            r.ipc.send(&tx, b"inside".to_vec(), Vec::new()).unwrap();
            let _ = next(&from_parent);
            0
        }
        _ => 99,
    }
}

struct Child {
    pid: libc::pid_t,
    peer: OsPeer,
    from_child: Receiver<Ev>,
    to_child: ChannelSender,
    _l: hg_substrate::ipc::Listening,
}

fn spawn(r: &Rig, case: &str) -> Child {
    let boot = test_support::port_with_send();
    let exe = CString::new(
        std::env::current_exe()
            .unwrap()
            .into_os_string()
            .into_encoded_bytes(),
    )
    .unwrap();
    let args: Vec<CString> = [
        "--exact",
        "ipc::macos::tests::d4_child",
        "--nocapture",
        "--test-threads",
        "1",
    ]
    .iter()
    .map(|a| CString::new(*a).unwrap())
    .collect();
    let arg_refs: Vec<&std::ffi::CStr> = args.iter().map(CString::as_c_str).collect();
    let env = CString::new(format!("IZCAL_D4_CHILD={case}")).unwrap();
    let child = sys::spawn(&exe, &arg_refs, &[env.as_c_str()], boot).unwrap();
    // The child has its own send right now; ours is not needed.
    sys::deallocate(boot);
    let (rx, tx) = r.ipc.adopt_child(boot, child.token).expect("hello");
    let peer = OsPeer::new(child.token.pid, u64::from(child.token.version));
    let (from_child, l) = r.listen_bound(rx, peer);
    Child {
        pid: child.pid,
        peer,
        from_child,
        to_child: tx,
        _l: l,
    }
}

/// Scenario C: a peer's own credit port, passed off as a receiver end,
/// is refused and blamed on that peer (R8: its reply right in the same
/// message is a send-once right to the port). Accepted, the credit for
/// it would come back on that port from this process, and the fault
/// would name this process.
#[test]
fn a_peer_s_own_credit_port_as_a_receiver_is_blamed_on_it() {
    let r = rig();
    let c = spawn(&r, "credit-as-receiver");
    expect_fault(&c.from_child, FaultKind::BadHandle, Some(c.peer.pid()));
    drop(c.to_child);
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

/// A child whose audit token cannot be read is killed before it runs,
/// and reaped (finding 7: it was resumed and left running).
#[test]
fn a_child_without_a_token_never_runs() {
    let boot = test_support::port_with_send();
    let exe = CString::new(
        std::env::current_exe()
            .unwrap()
            .into_os_string()
            .into_encoded_bytes(),
    )
    .unwrap();
    let args = [
        CString::new("--exact").unwrap(),
        CString::new("ipc::macos::tests::d4_child").unwrap(),
    ];
    let arg_refs: Vec<&std::ffi::CStr> = args.iter().map(CString::as_c_str).collect();
    let env = CString::new("IZCAL_D4_CHILD=burst-exit").unwrap();
    test_support::FAIL_CHILD_TOKEN.set(true);
    let got = sys::spawn(&exe, &arg_refs, &[env.as_c_str()], boot);
    test_support::FAIL_CHILD_TOKEN.set(false);
    sys::destroy_receive(boot);
    assert!(got.is_err());
    let pid = test_support::LAST_PID.get();
    assert!(pid > 0);
    // Already reaped: no such child of ours.
    let got = test_support::reap_now(pid);
    assert_eq!(
        got.map_err(|e| e.raw_os_error()),
        Err(Some(libc::ECHILD)),
        "the child is still ours to reap"
    );
}

#[test]
fn a_dead_child_s_messages_all_arrive_then_closed() {
    let r = rig();
    let c = spawn(&r, "burst-exit");
    for i in 0u8..5 {
        assert_eq!(expect_msg(&c.from_child).0, vec![i]);
    }
    assert!(matches!(next(&c.from_child), Ev::Closed));
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

#[test]
fn handles_cross_processes_and_work_there() {
    let r = rig();
    let c = spawn(&r, "echo");
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    r.ipc
        .send(&c.to_child, b"ping".to_vec(), vec![tx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&out).0, b"via-child");
    assert_eq!(expect_msg(&c.from_child).0, b"ping");
    drop(c.to_child);
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

#[test]
fn a_bound_receiver_hears_its_grantee_across_processes() {
    let r = rig();
    let c = spawn(&r, "use-grant");
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
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

#[test]
fn ipc_works_inside_the_sandbox() {
    let r = rig();
    let c = spawn(&r, "sandbox");
    let (b, mut hs) = expect_msg(&c.from_child);
    assert_eq!(b, b"chan");
    let rx = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
    let (out, _l) = r.listen(rx);
    assert_eq!(expect_msg(&out).0, b"inside");
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

#[test]
fn a_wrong_launch_identity_gets_no_channels() {
    let r = rig();
    let boot = test_support::port_with_send();
    let exe = CString::new(
        std::env::current_exe()
            .unwrap()
            .into_os_string()
            .into_encoded_bytes(),
    )
    .unwrap();
    let args: Vec<CString> = ["--exact", "ipc::macos::tests::d4_child"]
        .iter()
        .map(|a| CString::new(*a).unwrap())
        .collect();
    let arg_refs: Vec<&std::ffi::CStr> = args.iter().map(CString::as_c_str).collect();
    let env = CString::new("IZCAL_D4_CHILD=burst-exit").unwrap();
    let child = sys::spawn(&exe, &arg_refs, &[env.as_c_str()], boot).unwrap();
    sys::deallocate(boot);
    let wrong = wire::Token {
        pid: child.token.pid,
        version: child.token.version.wrapping_add(1),
    };
    assert!(r.ipc.adopt_child(boot, wrong).is_none());
    // The child gets no answer and exits on its own (code 10 or a failed
    // start).
    assert_ne!(sys::wait_child(child.pid).unwrap(), 0);
}

/// The D4 measurement: round-trip latency of a 64-byte message to a
/// child process and back (p50, p99), and throughput of 64 KiB messages
/// echoed with up to 32 in flight. Run under an exclusive booking with
/// `--ignored --nocapture`; prints one line.
#[test]
#[ignore = "a measurement, run by hand on the lab machine"]
fn measure_round_trip_and_throughput() {
    let r = rig();
    let c = spawn(&r, "echo");
    let mut lat = Vec::with_capacity(10_000);
    for i in 0..10_000u32 {
        let t = std::time::Instant::now();
        r.ipc
            .send(&c.to_child, i.to_le_bytes().repeat(16), Vec::new())
            .unwrap();
        let _ = expect_msg(&c.from_child);
        lat.push(t.elapsed().as_nanos());
    }
    lat.sort_unstable();
    let p = |q: usize| lat[lat.len() * q / 100] as f64 / 1000.0;
    let big = vec![5u8; MAX_MESSAGE_BYTES];
    let n = 2_000u32;
    let t = std::time::Instant::now();
    let mut inflight = 0u32;
    let mut done = 0u32;
    let mut sent = 0u32;
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
        "D4-MEASURE rtt64B p50={:.1}us p99={:.1}us max={:.1}us | 64KiB echo {mib:.0} MiB/s (both ways) over {n} msgs",
        p(50),
        p(99),
        *lat.last().unwrap() as f64 / 1000.0
    );
    drop(c.to_child);
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

#[test]
fn no_launch_without_a_launcher() {
    // A process not started by a launcher has no bootstrap channels.
    let r = rig();
    assert!(r.ipc.bootstrap().is_none());
}
