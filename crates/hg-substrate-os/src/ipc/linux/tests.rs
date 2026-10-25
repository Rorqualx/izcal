//! Tests of the Linux transport: in-process channel semantics, records
//! crafted to break the protocol, and cross-process runs where this test
//! binary is started again as the child (`d3_child`), with its launch
//! channels at descriptors 3 and 4.

use std::ffi::CString;
use std::os::fd::{AsFd, OwnedFd};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use hg_substrate::ipc::{
    ChannelReceiver, ChannelSender, Delivery, Fault, FaultKind, Flow, Handle, HandleKind, Ipc,
    IpcError, Transfer,
};
use hg_substrate::{Executor, Priority};

use super::sys::{self, SendOutcome};
use super::{BOOTSTRAP_FDS, LinuxEnd, LinuxIpc, adopt_parent_ends, launch_ends, launch_pair};
use crate::executor::OsExecutor;
use crate::ipc::common::codec::{HandleDesc, WireKind, encode_data};

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
    ipc: LinuxIpc,
}

fn rig() -> Rig {
    rig_with(None)
}

fn rig_with(boot: Option<(OwnedFd, OwnedFd)>) -> Rig {
    let exec = Arc::new(OsExecutor::new(2, 1).unwrap());
    let ipc = LinuxIpc::start(Arc::clone(&exec), boot).unwrap();
    Rig { exec, ipc }
}

impl Rig {
    fn listen(&self, rx: ChannelReceiver) -> (Receiver<Ev>, hg_substrate::ipc::Listening) {
        let (tx, out) = sync_channel::<Ev>(4096);
        let seq = self.exec.sequence("test", Priority::UserVisible);
        let l = self.ipc.listen(rx, seq, sink(tx)).expect("listen");
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

/// The raw descriptor behind one of this transport's handles (tests only:
/// to write records the transport itself would never write).
fn raw(ipc: &LinuxIpc, h: &Handle) -> Arc<OwnedFd> {
    let e = h.object().downcast_ref::<LinuxEnd>().unwrap();
    Arc::clone(&ipc.inner.lock().ends.get(&e.id).unwrap().fd)
}

fn me() -> i32 {
    i32::try_from(std::process::id()).unwrap()
}

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
    // The received sender works and reaches the original receiver.
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

/// The kernel buffer holds a handful of 64 KiB records; the rest wait in
/// the outbox, and `Full` comes only at the channel's capacity.
#[test]
fn bursts_wait_in_the_outbox_never_spuriously_full() {
    let r = rig();
    for cap in [64u32, 1024] {
        let (tx, rx) = r.ipc.channel(cap, Transfer::Free).unwrap();
        let big = vec![7u8; hg_substrate::ipc::MAX_MESSAGE_BYTES];
        for i in 0..cap {
            let mut m = big.clone();
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
            assert_eq!(b.len(), big.len());
            assert_eq!(b[0], u8::try_from(i % 251).unwrap(), "order at {i}");
        }
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

#[test]
fn a_dropped_receiver_closes_the_sender() {
    // `Closed` is eventual, not immediate: a child process being started
    // by another thread holds a copy of every descriptor until it execs,
    // so a send in that window is accepted and then lost (see the module
    // documentation). Within a short deadline every send is `Closed`.
    let r = rig();
    let (tx, rx) = r.ipc.channel(4, Transfer::Free).unwrap();
    drop(rx);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match r.ipc.send(&tx, b"x".to_vec(), Vec::new()) {
            Err(IpcError::Closed) => break,
            Ok(()) | Err(IpcError::Full) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(1));
            }
            other => panic!("expected Closed, got {other:?}"),
        }
    }
    assert_eq!(
        r.ipc.send(&tx, b"y".to_vec(), Vec::new()),
        Err(IpcError::Closed),
        "and it stays closed"
    );
}

#[test]
fn bound_senders_move_once_and_bound_receivers_never() {
    let r = rig();
    let (carrier, _crx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (btx, brx) = r.ipc.channel(8, Transfer::Bound).unwrap();
    assert_eq!(btx.handle().kind(), HandleKind::BoundChannelSender);
    // The receiver of a bound channel never moves.
    assert_eq!(
        r.ipc.send(&carrier, Vec::new(), vec![brx.into_handle()]),
        Err(IpcError::Refused)
    );
    // Its sender moves once, from here.
    r.ipc
        .send(&carrier, b"grant".to_vec(), vec![btx.into_handle()])
        .unwrap();
}

#[test]
fn a_channel_cannot_carry_its_own_end() {
    let r = rig();
    let (tx, _rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (tx2, _rx2) = r.ipc.channel(8, Transfer::Free).unwrap();
    let same = ChannelSender::from_handle(tx.into_handle()).unwrap();
    let h = Handle::new(Box::new(LinuxEnd {
        inner: Arc::clone(&r.ipc.inner),
        id: same
            .handle()
            .object()
            .downcast_ref::<LinuxEnd>()
            .unwrap()
            .id,
        kind: HandleKind::ChannelSender,
        armed: false,
    }));
    assert_eq!(
        r.ipc.send(&same, Vec::new(), vec![h]),
        Err(IpcError::Refused)
    );
    drop(tx2);
}

/// Writes `bytes` with `fds` straight onto a channel's sender socket.
fn inject(r: &Rig, tx: &ChannelSender, bytes: &[u8], fds: &[std::os::fd::BorrowedFd<'_>]) {
    let fd = raw(&r.ipc, tx.handle());
    assert_eq!(sys::send_record(fd.as_fd(), bytes, fds), SendOutcome::Sent);
}

fn desc(kind: WireKind) -> HandleDesc {
    HandleDesc {
        kind,
        capacity: 8,
        sent: 0,
        delivered: 0,
    }
}

fn expect_fault(out: &Receiver<Ev>, kind: FaultKind, sender: Option<u32>) {
    match next(out) {
        Ev::Fault(f) => {
            assert_eq!(f.kind, kind, "{f:?}");
            assert_eq!(f.sender, sender, "{f:?}");
        }
        other => panic!("expected a violation, got {other:?}"),
    }
}

#[test]
fn every_foreign_descriptor_type_is_refused() {
    let my = u32::try_from(me()).unwrap();
    // A file, a pipe, a stream and a datagram Unix socket, an inet socket.
    let file = std::fs::File::open("/proc/self/stat").unwrap();
    let (pipe_r, _pipe_w) = std::io::pipe().unwrap();
    let (stream, _s2) = std::os::unix::net::UnixStream::pair().unwrap();
    let (dgram, _d2) = std::os::unix::net::UnixDatagram::pair().unwrap();
    let tcp = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let candidates: Vec<std::os::fd::BorrowedFd<'_>> = vec![
        file.as_fd(),
        pipe_r.as_fd(),
        stream.as_fd(),
        dgram.as_fd(),
        tcp.as_fd(),
    ];
    for fd in candidates {
        let r = rig();
        let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
        let (out, _l) = r.listen(rx);
        let rec = encode_data(0, &[desc(WireKind::Sender)], b"x").unwrap();
        inject(&r, &tx, &rec, &[fd]);
        expect_fault(&out, FaultKind::BadHandle, Some(my));
    }
}

#[test]
fn malformed_records_are_violations_named_by_the_kernel() {
    let my = u32::try_from(me()).unwrap();
    let cases: Vec<(Vec<u8>, FaultKind)> = vec![
        // Empty.
        (Vec::new(), FaultKind::Malformed),
        // Garbage.
        (b"not a record".to_vec(), FaultKind::Malformed),
        // A descriptor claimed but none attached.
        (
            encode_data(0, &[desc(WireKind::Sender)], b"x").unwrap(),
            FaultKind::BadHandle,
        ),
        // Larger than any record can be.
        (vec![0u8; super::MAX_DATA_RECORD + 1], FaultKind::Malformed),
    ];
    for (bytes, kind) in cases {
        let r = rig();
        let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
        let (out, _l) = r.listen(rx);
        inject(&r, &tx, &bytes, &[]);
        expect_fault(&out, kind, Some(my));
    }
}

#[test]
fn a_sequence_gap_is_a_violation_without_blame() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    let rec = encode_data(5, &[], b"x").unwrap();
    inject(&r, &tx, &rec, &[]);
    expect_fault(&out, FaultKind::Sequence, None);
}

#[test]
fn a_held_end_sent_back_is_refused() {
    let my = u32::try_from(me()).unwrap();
    let r = rig();
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (other_tx, _other_rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    // A duplicate of an end this process still holds.
    let dup = raw(&r.ipc, other_tx.handle()).try_clone().unwrap();
    let rec = encode_data(0, &[desc(WireKind::Sender)], b"x").unwrap();
    inject(&r, &tx, &rec, &[dup.as_fd()]);
    expect_fault(&out, FaultKind::BadHandle, Some(my));
}

#[test]
fn swapped_ends_install_nothing_on_the_credit_path() {
    // A receiver end handed over as if it were a sender: sends on it go
    // to a channel the peer controls, and the credit reads use no
    // control buffer, so a descriptor sent back is never installed.
    let r = rig();
    let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (out, _l) = r.listen(rx);
    let (a, b) = sys::socketpair().unwrap();
    let rec = encode_data(0, &[desc(WireKind::Sender)], b"x").unwrap();
    inject(&r, &tx, &rec, &[a.as_fd()]);
    let (_, mut hs) = expect_msg(&out);
    let fake = ChannelSender::from_handle(hs.remove(0)).unwrap();
    // The "receiver" side writes a descriptor back the wrong way.
    let (x, _y) = sys::socketpair().unwrap();
    assert_eq!(
        sys::send_record(b.as_fd(), b"zzzz", &[x.as_fd()]),
        SendOutcome::Sent
    );
    assert_eq!(sys::count_refs(x.as_fd(), 4096), 1);
    let res = r.ipc.send(&fake, b"m".to_vec(), Vec::new());
    assert_eq!(
        res,
        Err(IpcError::Closed),
        "a garbage credit breaks the end"
    );
    // Counted by the file, not by how many descriptors are open: other
    // tests open and close their own at the same time.
    assert_eq!(
        sys::count_refs(x.as_fd(), 4096),
        1,
        "no descriptor installed"
    );
}

// ---- cross-process ----------------------------------------------------

/// The child side, run by the test binary started again with
/// `IZCAL_D3_CHILD=<case>`; a no-op in a normal run.
#[test]
fn d3_child() {
    let Some(case) = std::env::var_os("IZCAL_D3_CHILD") else {
        return;
    };
    let case = case.into_string().unwrap();
    let code = child(&case);
    std::process::exit(code);
}

fn child(case: &str) -> i32 {
    if case == "flood" {
        sys::set_fd_limit(64);
    }
    if case == "headroom" {
        // Room for the transport's own descriptors, the headroom, and
        // about six held ends.
        let open = sys::count_open(1024);
        sys::set_fd_limit(open + 4 + super::FD_HEADROOM + 6);
    }
    if case == "sandbox" {
        // Lock down first (the transport's IO thread starts after, and
        // TSYNC would cover it anyway).
        crate::sandbox::linux::apply_empty_process_profile().unwrap();
    }
    let boot = sys::take_test_child_fds(BOOTSTRAP_FDS).and_then(|(u, d)| launch_ends(u, d).ok());
    let r = rig_with(boot);
    let Some(boot) = r.ipc.bootstrap() else {
        return 10;
    };
    let up = boot.to_parent;
    let (from_parent, _l) = r.listen(boot.from_parent);
    match case {
        "burst-exit" => {
            for i in 0u8..5 {
                r.ipc.send(&up, vec![i], Vec::new()).unwrap();
            }
            // Ends abruptly: the parent must still get all five.
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
        "forward-as-bound" => {
            // Receives a free sender made by the parent and sends it back
            // claiming it is a bound sender: the parent's transport sees
            // that this process did not create it (SO_PEERCRED).
            let Ev::Msg(_, mut hs) = next(&from_parent) else {
                return 11;
            };
            let h = hs.remove(0);
            let fd = raw(&r.ipc, &h);
            let rec = encode_data(0, &[desc(WireKind::BoundSender)], b"bound?").unwrap();
            let upfd = raw(&r.ipc, up.handle());
            match sys::send_record(upfd.as_fd(), &rec, &[fd.as_fd()]) {
                SendOutcome::Sent => 0,
                _ => 12,
            }
        }
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
            // Wait for the parent to finish.
            let _ = next(&from_parent);
            0
        }
        "sandbox" => {
            // Under the filter: make a channel, send its sender up, send
            // on it, report what fork and a user namespace got.
            let (tx, rx) = r.ipc.channel(8, Transfer::Free).unwrap();
            r.ipc
                .send(&up, b"chan".to_vec(), vec![rx.into_handle()])
                .unwrap();
            r.ipc.send(&tx, b"inside".to_vec(), Vec::new()).unwrap();
            let (fork, userns) = crate::sandbox::linux::probe_process_creation();
            let report = format!("fork={fork} userns={userns}");
            r.ipc.send(&up, report.into_bytes(), Vec::new()).unwrap();
            let _ = next(&from_parent);
            0
        }
        "flood" => {
            // Both ends of fresh channels, never read by the parent until
            // this process ends: in-flight descriptors pass the limit set
            // below and a send fails with ETOOMANYREFS, seen as Closed.
            for i in 0..200 {
                let (tx, rx) = r.ipc.channel(1, Transfer::Free).unwrap();
                match r
                    .ipc
                    .send(&up, vec![1], vec![tx.into_handle(), rx.into_handle()])
                {
                    Ok(()) => {}
                    Err(IpcError::Closed) => return 20 + i,
                    Err(e) => panic!("{e}"),
                }
            }
            1
        }
        "headroom" => {
            // Keeps every received handle: reads pause when descriptor
            // slots run short, and resume once the handles are dropped.
            let mut kept = Vec::new();
            let mut got = 0u32;
            loop {
                match from_parent.recv_timeout(Duration::from_millis(1500)) {
                    Ok(Ev::Msg(_, hs)) => {
                        got += 1;
                        kept.extend(hs);
                    }
                    Ok(_) => return 13,
                    Err(_) => break,
                }
            }
            r.ipc
                .send(&up, format!("paused-at={got}").into_bytes(), Vec::new())
                .unwrap();
            drop(kept);
            while got < 12 {
                match next(&from_parent) {
                    Ev::Msg(..) => got += 1,
                    _ => return 14,
                }
            }
            r.ipc
                .send(&up, format!("total={got}").into_bytes(), Vec::new())
                .unwrap();
            let _ = next(&from_parent);
            0
        }
        _ => 99,
    }
}

struct Child {
    pid: libc::pid_t,
    from_child: Receiver<Ev>,
    to_child: ChannelSender,
    _l: hg_substrate::ipc::Listening,
}

fn spawn(r: &Rig, case: &str) -> Child {
    let (pid, rx, tx) = spawn_unheard(r, case);
    let (from_child, l) = r.listen(rx);
    Child {
        pid,
        from_child,
        to_child: tx,
        _l: l,
    }
}

/// Starts a child; its channel to us is not read yet.
fn spawn_unheard(r: &Rig, case: &str) -> (libc::pid_t, ChannelReceiver, ChannelSender) {
    let ((from_child_fd, to_child_fd), (up, down)) = launch_pair().unwrap();
    let exe = CString::new(
        std::env::current_exe()
            .unwrap()
            .into_os_string()
            .into_encoded_bytes(),
    )
    .unwrap();
    let args: Vec<CString> = [
        "--exact",
        "ipc::linux::tests::d3_child",
        "--nocapture",
        "--test-threads",
        "1",
    ]
    .iter()
    .map(|a| CString::new(*a).unwrap())
    .collect();
    let arg_refs: Vec<&std::ffi::CStr> = args.iter().map(CString::as_c_str).collect();
    let env = CString::new(format!("IZCAL_D3_CHILD={case}")).unwrap();
    let pid = sys::spawn_with_fds(
        &exe,
        &arg_refs,
        &[env.as_c_str()],
        &[
            (up.as_fd(), BOOTSTRAP_FDS.0),
            (down.as_fd(), BOOTSTRAP_FDS.1),
        ],
    )
    .unwrap();
    drop((up, down));
    let (rx, tx) = adopt_parent_ends(&r.ipc, from_child_fd, to_child_fd).unwrap();
    (pid, rx, tx)
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
        .send(&c.to_child, b"hello".to_vec(), vec![tx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&c.from_child).0, b"hello");
    // The sender we gave the child reached our receiver from there.
    assert_eq!(expect_msg(&out).0, b"via-child");
    drop(c.to_child);
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

#[test]
fn a_forwarded_end_claimed_as_bound_is_refused() {
    let r = rig();
    let c = spawn(&r, "forward-as-bound");
    let (tx, _rx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&c.to_child, b"take".to_vec(), vec![tx.into_handle()])
        .unwrap();
    let child = u32::try_from(c.pid).unwrap();
    expect_fault(&c.from_child, FaultKind::Identity, Some(child));
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

#[test]
fn a_bound_receiver_hears_only_its_grantee() {
    let r = rig();
    let c = spawn(&r, "use-grant");
    let (btx, brx) = r.ipc.channel(8, Transfer::Bound).unwrap();
    // Keep a raw copy of the bound sender before granting it.
    let ours = raw(&r.ipc, btx.handle()).try_clone().unwrap();
    let (out, _l) = r.listen(brx);
    r.ipc
        .send(&c.to_child, b"grant".to_vec(), vec![btx.into_handle()])
        .unwrap();
    assert_eq!(expect_msg(&out).0, b"from-child");
    assert_eq!(expect_msg(&c.from_child).0, b"done");
    // Now another process (this one) writes on the bound channel.
    let rec = encode_data(1, &[], b"impostor").unwrap();
    assert_eq!(sys::send_record(ours.as_fd(), &rec, &[]), SendOutcome::Sent);
    expect_fault(
        &out,
        FaultKind::Identity,
        Some(u32::try_from(me()).unwrap()),
    );
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

#[test]
fn ipc_works_inside_the_sandbox_and_processes_cannot_be_made() {
    let r = rig();
    let c = spawn(&r, "sandbox");
    let (b, mut hs) = expect_msg(&c.from_child);
    assert_eq!(b, b"chan");
    let rx = ChannelReceiver::from_handle(hs.remove(0)).unwrap();
    let (out, _l) = r.listen(rx);
    assert_eq!(expect_msg(&out).0, b"inside");
    // Both refused with EPERM by the filter.
    assert_eq!(
        String::from_utf8(expect_msg(&c.from_child).0).unwrap(),
        format!("fork={} userns={}", libc::EPERM, libc::EPERM)
    );
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

#[test]
fn too_many_descriptors_in_flight_closes_the_channel() {
    let r = rig();
    let (pid, rx, _tx) = spawn_unheard(&r, "flood");
    let code = sys::wait_child(pid).unwrap();
    assert!((21..200).contains(&code), "child exit {code}");
    let sent = code - 20;
    let (out, _l) = r.listen(rx);
    for _ in 0..sent {
        let (_, hs) = expect_msg(&out);
        assert_eq!(hs.len(), 2);
    }
    assert!(matches!(next(&out), Ev::Closed));
}

#[test]
fn reads_pause_without_descriptor_room_and_resume() {
    let r = rig();
    let c = spawn(&r, "headroom");
    for _ in 0..12 {
        let (tx, _rx) = r.ipc.channel(1, Transfer::Free).unwrap();
        r.ipc
            .send(&c.to_child, b"h".to_vec(), vec![tx.into_handle()])
            .unwrap();
    }
    let paused = String::from_utf8(expect_msg(&c.from_child).0).unwrap();
    let k: u32 = paused.strip_prefix("paused-at=").unwrap().parse().unwrap();
    assert!(k > 0 && k < 12, "{paused}");
    assert_eq!(expect_msg(&c.from_child).0, b"total=12");
    r.ipc
        .send(&c.to_child, b"bye".to_vec(), Vec::new())
        .unwrap();
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}

#[test]
fn a_transfer_waits_for_the_carried_sender_s_queue() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(64, Transfer::Free).unwrap();
    let big = vec![3u8; hg_substrate::ipc::MAX_MESSAGE_BYTES];
    for i in 0u8..20 {
        let mut m = big.clone();
        m[0] = i;
        r.ipc.send(&tx, m, Vec::new()).unwrap();
    }
    let (carrier, crx) = r.ipc.channel(8, Transfer::Free).unwrap();
    r.ipc
        .send(&carrier, b"take".to_vec(), vec![tx.into_handle()])
        .unwrap();
    // Read the first channel, then take the moved sender and use it.
    let (out, _l) = r.listen(rx);
    let (cout, _cl) = r.listen(crx);
    let (_, mut hs) = expect_msg(&cout);
    let moved = ChannelSender::from_handle(hs.remove(0)).unwrap();
    r.ipc.send(&moved, b"after".to_vec(), Vec::new()).unwrap();
    for i in 0u8..20 {
        assert_eq!(expect_msg(&out).0[0], i);
    }
    assert_eq!(expect_msg(&out).0, b"after");
}

#[test]
fn a_transfer_gives_up_on_an_unread_queue_at_its_deadline() {
    let r = rig();
    let (tx, _rx) = r.ipc.channel(64, Transfer::Free).unwrap();
    let big = vec![3u8; hg_substrate::ipc::MAX_MESSAGE_BYTES];
    for _ in 0..20 {
        r.ipc.send(&tx, big.clone(), Vec::new()).unwrap();
    }
    let (carrier, crx) = r.ipc.channel(8, Transfer::Free).unwrap();
    let (cout, _cl) = r.listen(crx);
    r.ipc
        .send(&carrier, b"take".to_vec(), vec![tx.into_handle()])
        .unwrap();
    // Nobody reads the first channel: after the deadline the carried end
    // is closed for writing and the carrier goes.
    let (_, mut hs) = expect_msg(&cout);
    let moved = ChannelSender::from_handle(hs.remove(0)).unwrap();
    assert_eq!(
        r.ipc.send(&moved, b"x".to_vec(), Vec::new()),
        Err(IpcError::Closed)
    );
}

#[test]
fn a_former_holder_cannot_wedge_or_shrink_the_channel() {
    let r = rig();
    let (tx, rx) = r.ipc.channel(64, Transfer::Free).unwrap();
    let dup = raw(&r.ipc, tx.handle()).try_clone().unwrap();
    // A blocking open file description: sends still never block.
    sys::clear_nonblock(dup.as_fd());
    let big = vec![9u8; hg_substrate::ipc::MAX_MESSAGE_BYTES];
    let (done_tx, done) = sync_channel::<()>(1);
    std::thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..30 {
                r.ipc.send(&tx, big.clone(), Vec::new()).unwrap();
            }
            done_tx.send(()).unwrap();
        });
        done.recv_timeout(WAIT).expect("sends did not block");
    });
    // A shrunk send buffer: a maximum record no longer fits; the channel
    // reports closed rather than misbehaving.
    let (tx2, _rx2) = r.ipc.channel(8, Transfer::Free).unwrap();
    let dup2 = raw(&r.ipc, tx2.handle()).try_clone().unwrap();
    let _ = sys::set_sndbuf(dup2.as_fd(), 1024);
    assert_eq!(
        r.ipc.send(&tx2, big.clone(), Vec::new()),
        Err(IpcError::Closed)
    );
    drop(rx);
}

/// The D3 measurement: round-trip latency of a 64-byte message to a
/// child process and back (p50, p99), and one-way throughput of 64 KiB
/// messages. Run under an exclusive booking with `--ignored --nocapture`;
/// prints one line.
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
    let big = vec![5u8; hg_substrate::ipc::MAX_MESSAGE_BYTES];
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
        "D3-MEASURE rtt64B p50={:.1}us p99={:.1}us max={:.1}us | 64KiB echo {mib:.0} MiB/s (both ways) over {n} msgs",
        p(50),
        p(99),
        *lat.last().unwrap() as f64 / 1000.0
    );
    drop(c.to_child);
    assert_eq!(sys::wait_child(c.pid).unwrap(), 0);
}
