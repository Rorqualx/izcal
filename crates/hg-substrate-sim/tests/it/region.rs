//! Medium tests: simulated shared-memory regions (M1 phase D6; IPC-R1,
//! IPC-R7; the D6 review dispositions M1, M3, M7, minor 6).

use std::sync::{Arc, Mutex};

use hg_substrate::ipc::{
    AccessError, ChannelReceiver, ChannelSender, ConsumerRegion, Delivery, Flow, Handle, IpcError,
    Listening, OsPeer, ReadOnlyRegion, RegionWriter,
};
use hg_substrate::{Memory, Priority, Substrate};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{Limits, RunEnd, SimConfig, SimOs, World};
use hg_types::Instant;

/// The child as the simulator identifies it: spawned second.
const OWNER: OsPeer = OsPeer::new(1, 0);

const LIMITS: Limits = Limits {
    max_steps: 100_000,
    max_time: Instant::from_nanos(86_400_000_000_000),
};

type Slot<T> = Arc<Mutex<Option<T>>>;
type Log = Arc<Mutex<Vec<String>>>;

fn backing() -> Arc<dyn Memory> {
    Arc::new(OsMemory::new())
}

fn say(log: &Log, s: impl Into<String>) {
    log.lock().unwrap().push(s.into());
}

/// Listens on `rx`, handing each message's handles to `f` and logging
/// closings and violations.
fn on_message(
    s: &Substrate,
    rx: ChannelReceiver,
    log: &Log,
    f: impl Fn(&Substrate, Vec<u8>, Vec<Handle>) + Send + 'static,
) -> Listening {
    let seq = s.exec.sequence("inbox", Priority::UserVisible);
    let (log, s2) = (Arc::clone(log), s.clone());
    s.ipc
        .listen(
            rx,
            seq,
            Box::new(move |d| {
                match d {
                    Delivery::Message { bytes, handles } => f(&s2, bytes, handles),
                    Delivery::Closed => say(&log, "closed"),
                    Delivery::Violation(v) => {
                        say(&log, format!("violation {:?} {:?}", v.kind, v.sender))
                    }
                }
                Flow::Continue
            }),
        )
        .unwrap()
}

/// A broker and one child it launched; the broker's entry gets its ends.
fn world(
    os: SimOs,
    seed: u64,
    broker: impl Fn(Substrate, ChannelReceiver, ChannelSender) + Send + Sync + 'static,
    child: impl Fn(Substrate) + Send + Sync + 'static,
) -> World {
    let mut config = SimConfig::new(seed);
    config.keep_trace_text = true;
    config.region_os = os;
    let w = World::new(config);
    let ends: Slot<(ChannelReceiver, ChannelSender)> = Arc::default();
    let e = Arc::clone(&ends);
    let b = w.spawn("broker", backing(), false, move |s| {
        if let Some((from_child, to_child)) = e.lock().unwrap().take() {
            broker(s, from_child, to_child);
        }
    });
    let c = w.spawn("child", backing(), false, child);
    *ends.lock().unwrap() = w.link(b, c, 16);
    w
}

fn keep<T: Send + 'static>(slot: &Slot<Vec<Box<dyn std::any::Any + Send>>>, t: T) {
    slot.lock()
        .unwrap()
        .get_or_insert_with(Vec::new)
        .push(Box::new(t));
}

fn read_all(s: &Substrate, h: &Handle, len: usize) -> Vec<u8> {
    let m = s.ipc.map_region(h).unwrap();
    let mut out = vec![0; len];
    m.read_into(0, &mut out).unwrap();
    out
}

/// The broker fills and seals a region and sends it; the child maps the
/// declared length and reads it; on Linux the seal makes it immutable.
#[test]
fn a_sealed_region_crosses_and_reads_back() {
    let log: Log = Arc::default();
    let held: Slot<Vec<Box<dyn std::any::Any + Send>>> = Arc::default();
    let (l2, h2) = (Arc::clone(&log), Arc::clone(&held));
    let w = world(
        SimOs::Linux,
        1,
        |s, _from, to| {
            let wr = s.ipc.allocate_region(8192, OWNER).unwrap();
            {
                let m = s.ipc.map_region(wr.handle()).unwrap();
                m.write_from(100, b"hello region").unwrap();
                m.store_u32_release(0, 7).unwrap();
            }
            let ro = s.ipc.seal_region(wr).unwrap();
            s.ipc
                .send(&to, b"ro".to_vec(), vec![ro.into_handle()])
                .unwrap();
        },
        move |s| {
            let boot = s.ipc.bootstrap().unwrap();
            let (l3, h3) = (Arc::clone(&l2), Arc::clone(&h2));
            let lis = on_message(&s, boot.from_parent, &l2, move |s, _b, mut hs| {
                let ro = ReadOnlyRegion::from_handle(hs.remove(0)).unwrap();
                let bytes = read_all(s, ro.handle(), ro.len());
                let m = s.ipc.map_region(ro.handle()).unwrap();
                say(
                    &l3,
                    format!(
                        "len {} immutable {} word {} text {:?} write {:?}",
                        ro.len(),
                        ro.immutable(),
                        m.load_u32_acquire(0).unwrap(),
                        String::from_utf8_lossy(&bytes[100..112]),
                        m.write_from(0, b"x"),
                    ),
                );
                // A child cannot allocate, and cannot send a writer.
                say(
                    &l3,
                    format!(
                        "child allocate {:?}",
                        s.ipc.allocate_region(4096, OWNER).err()
                    ),
                );
                keep(&h3, ro);
            });
            keep(&h2, lis);
        },
    );
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(
        *log.lock().unwrap(),
        [
            "len 8192 immutable true word 7 text \"hello region\" write Err(ReadOnly)",
            "child allocate Some(Unsupported)",
            "closed",
        ]
    );
    drop(held);
}

/// Linux refuses a seal while a mapping is live: an error to the caller,
/// with the writer given back, not a violation (review minor 6).
#[test]
fn linux_refuses_a_seal_while_a_mapping_is_live() {
    let log: Log = Arc::default();
    let l2 = Arc::clone(&log);
    let w = world(
        SimOs::Linux,
        2,
        move |s, _f, _t| {
            let wr = s.ipc.allocate_region(4096, OWNER).unwrap();
            let m = s.ipc.map_region(wr.handle()).unwrap();
            let refused = s.ipc.seal_region(wr).unwrap_err();
            say(&l2, format!("{:?}", refused.error));
            let wr: RegionWriter = refused.into_writer().unwrap();
            drop(m);
            say(
                &l2,
                format!("after unmap: {}", s.ipc.seal_region(wr).is_ok()),
            );
        },
        |_s| {},
    );
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(*log.lock().unwrap(), ["Busy", "after unmap: true"]);
}

/// A writer from the broker is filled and sealed by the child, which sends
/// the read-only region back; the child cannot pass the writer on.
#[test]
fn a_child_fills_a_writer_and_cannot_forward_it() {
    let log: Log = Arc::default();
    let held: Slot<Vec<Box<dyn std::any::Any + Send>>> = Arc::default();
    let (l2, h2, l4) = (Arc::clone(&log), Arc::clone(&held), Arc::clone(&log));
    let w = world(
        SimOs::Linux,
        3,
        move |s, from, to| {
            let wr = s.ipc.allocate_region(4096, OWNER).unwrap();
            s.ipc
                .send(&to, b"w".to_vec(), vec![wr.into_handle()])
                .unwrap();
            let l3 = Arc::clone(&l2);
            let lis = on_message(&s, from, &l2, move |s, _b, mut hs| {
                let ro = ReadOnlyRegion::from_handle(hs.remove(0)).unwrap();
                say(
                    &l3,
                    format!(
                        "broker reads {:?}",
                        String::from_utf8_lossy(&read_all(s, ro.handle(), 5))
                    ),
                );
            });
            keep(&h2, lis);
            keep(&h2, to);
        },
        move |s| {
            let boot = s.ipc.bootstrap().unwrap();
            let to = Arc::new(boot.to_parent);
            let (l5, to2) = (Arc::clone(&l4), Arc::clone(&to));
            let lis = on_message(&s, boot.from_parent, &l4, move |s, _b, mut hs| {
                let wr = RegionWriter::from_handle(hs.remove(0)).unwrap();
                // Forwarding a writer is refused: only the broker sends one.
                let (ch_tx, _ch_rx) = s.ipc.channel(4, hg_substrate::ipc::Transfer::Free).unwrap();
                let wr2 = s.ipc.allocate_region(1, OWNER).err();
                say(&l5, format!("child allocate {wr2:?}"));
                {
                    let m = s.ipc.map_region(wr.handle()).unwrap();
                    m.write_from(0, b"child").unwrap();
                }
                let ro = s.ipc.seal_region(wr).unwrap();
                s.ipc
                    .send(&to2, b"ro".to_vec(), vec![ro.into_handle()])
                    .unwrap();
                drop(ch_tx);
            });
            std::mem::forget(lis);
            std::mem::forget(to);
        },
    );
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(
        *log.lock().unwrap(),
        ["child allocate Some(Unsupported)", "broker reads \"child\""]
    );
    drop(held);
}

/// A process that holds a writer it did not allocate cannot send it on.
#[test]
fn only_the_broker_sends_a_writer() {
    let log: Log = Arc::default();
    let l2 = Arc::clone(&log);
    let w = world(
        SimOs::Linux,
        4,
        |s, _from, to| {
            let wr = s.ipc.allocate_region(4096, OWNER).unwrap();
            s.ipc
                .send(&to, b"w".to_vec(), vec![wr.into_handle()])
                .unwrap();
        },
        move |s| {
            let boot = s.ipc.bootstrap().unwrap();
            let to = boot.to_parent;
            let l3 = Arc::clone(&l2);
            let lis = on_message(&s, boot.from_parent, &l2, move |s, _b, hs| {
                say(
                    &l3,
                    format!("forward {:?}", s.ipc.send(&to, b"x".to_vec(), hs).err()),
                );
            });
            std::mem::forget(lis);
        },
    );
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(*log.lock().unwrap(), ["forward Some(Refused)", "closed"]);
}

/// A producer keeps writing; a consumer in another process sees the words
/// it publishes with release stores. On Linux the producer cannot be
/// mapped again.
#[test]
fn a_producer_publishes_to_a_consumer() {
    let log: Log = Arc::default();
    let held: Slot<Vec<Box<dyn std::any::Any + Send>>> = Arc::default();
    let (l2, h2) = (Arc::clone(&log), Arc::clone(&held));
    let h4 = Arc::clone(&held);
    let w = world(
        SimOs::Linux,
        5,
        move |s, _from, to| {
            let wr = s.ipc.allocate_region(4096, OWNER).unwrap();
            let p = s.ipc.share_region(wr).unwrap();
            p.mapping().store_u64_release(8, 41).unwrap();
            say(
                &l2,
                format!("producer remap {:?}", s.ipc.map_region(p.handle()).err()),
            );
            let c = s.ipc.consumer_of(&p).unwrap();
            s.ipc
                .send(&to, b"c".to_vec(), vec![c.into_handle()])
                .unwrap();
            p.mapping().store_u64_release(8, 42).unwrap();
            keep(&h2, p);
        },
        move |s| {
            let boot = s.ipc.bootstrap().unwrap();
            let h3 = Arc::clone(&h4);
            let lis = on_message(
                &s,
                boot.from_parent,
                &Arc::default(),
                move |s, _b, mut hs| {
                    let c = ConsumerRegion::from_handle(hs.remove(0)).unwrap();
                    let m = s.ipc.map_region(c.handle()).unwrap();
                    let v = m.load_u64_acquire(8).unwrap();
                    keep(
                        &h3,
                        format!(
                            "consumer reads {v} misaligned {:?} oob {:?}",
                            m.load_u64_acquire(4).err(),
                            m.load_u32_acquire(4096).err()
                        ),
                    );
                    keep(&h3, (c, m));
                },
            );
            keep(&h4, lis);
        },
    );
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(*log.lock().unwrap(), ["producer remap Some(Refused)"]);
    let seen: Vec<String> = held
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .iter()
        .filter_map(|b| b.downcast_ref::<String>().cloned())
        .collect();
    assert_eq!(
        seen,
        [format!(
            "consumer reads 42 misaligned {:?} oob {:?}",
            Some(AccessError::Misaligned),
            Some(AccessError::OutOfBounds)
        )]
    );
}

/// The OS personalities of review M7: what a sandboxed filler that kept a
/// writable mapping across its seal can still change after the receiver
/// mapped the region, and what `immutable` buys on each OS.
fn filler_keeps_writing(os: SimOs, want_immutable: bool) -> (bool, String) {
    type Kept = Arc<Mutex<Option<hg_substrate::ipc::Mapping>>>;
    let out: Slot<(bool, String)> = Arc::default();
    let o2 = Arc::clone(&out);
    let held: Slot<Vec<Box<dyn std::any::Any + Send>>> = Arc::default();
    let (h2, h3) = (Arc::clone(&held), Arc::clone(&held));
    let w = world(
        os,
        6,
        move |s, from, to| {
            let wr = s.ipc.allocate_region(4096, OWNER).unwrap();
            s.ipc
                .send(&to, b"w".to_vec(), vec![wr.into_handle()])
                .unwrap();
            let to = Arc::new(to);
            let (o3, h3, to2) = (Arc::clone(&o2), Arc::clone(&h3), Arc::clone(&to));
            let lis = on_message(&s, from, &Arc::default(), move |s, b, mut hs| {
                if b == b"ro" {
                    // Map it, then let the filler write.
                    let ro = ReadOnlyRegion::from_handle(hs.remove(0)).unwrap();
                    let m = s.ipc.map_region(ro.handle()).unwrap();
                    keep(&h3, (ro, m));
                    s.ipc.send(&to2, b"go".to_vec(), vec![]).unwrap();
                } else {
                    let g = h3.lock().unwrap();
                    let (ro, m) = g
                        .as_ref()
                        .unwrap()
                        .iter()
                        .find_map(|b| {
                            b.downcast_ref::<(ReadOnlyRegion, hg_substrate::ipc::Mapping)>()
                        })
                        .unwrap();
                    let mut buf = [0u8; 5];
                    m.read_into(0, &mut buf).unwrap();
                    *o3.lock().unwrap() =
                        Some((ro.immutable(), String::from_utf8_lossy(&buf).into_owned()));
                }
            });
            keep(&h2, lis);
            keep(&h2, to);
        },
        move |s| {
            let boot = s.ipc.bootstrap().unwrap();
            let to = boot.to_parent;
            let kept: Kept = Arc::default();
            let lis = on_message(
                &s,
                boot.from_parent,
                &Arc::default(),
                move |s, b, mut hs| {
                    if b == b"go" {
                        // The kept mapping writes after the seal and the map.
                        if let Some(m) = kept.lock().unwrap().as_ref() {
                            m.write_from(0, b"later").unwrap();
                            m.store_u32_release(8, 2).unwrap();
                        }
                        s.ipc.send(&to, b"check".to_vec(), vec![]).unwrap();
                        return;
                    }
                    let wr = RegionWriter::from_handle(hs.remove(0)).unwrap();
                    let m = s.ipc.map_region(wr.handle()).unwrap();
                    m.write_from(0, b"first").unwrap();
                    m.store_u32_release(8, 1).unwrap();
                    let ro = match s.ipc.seal_region(wr) {
                        Ok(ro) => {
                            *kept.lock().unwrap() = Some(m);
                            ro
                        }
                        Err(r) => {
                            // Linux: the seal waits for the mapping (L1).
                            drop(m);
                            s.ipc.seal_region(r.into_writer().unwrap()).unwrap()
                        }
                    };
                    if want_immutable {
                        ro.want_immutable();
                    }
                    s.ipc
                        .send(&to, b"ro".to_vec(), vec![ro.into_handle()])
                        .unwrap();
                },
            );
            std::mem::forget(lis);
        },
    );
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    let r = out.lock().unwrap().take().unwrap();
    drop(held);
    r
}

#[test]
fn each_os_attests_what_it_can() {
    // Linux: the seal waited for the mapping; nothing changes after.
    assert_eq!(
        filler_keeps_writing(SimOs::Linux, false),
        (true, "first".into())
    );
    // macOS: shared unless `immutable`, then a snapshot.
    assert_eq!(
        filler_keeps_writing(SimOs::MacOs, false),
        (false, "later".into())
    );
    assert_eq!(
        filler_keeps_writing(SimOs::MacOs, true),
        (true, "first".into())
    );
    // Windows: a sandboxed seal is filler-trusted; `immutable` makes the
    // broker copy it first.
    assert_eq!(
        filler_keeps_writing(SimOs::Windows, false),
        (false, "later".into())
    );
    assert_eq!(
        filler_keeps_writing(SimOs::Windows, true),
        (true, "first".into())
    );
}

/// The broker's region quota, and an empty region refused.
#[test]
fn the_region_quota_holds() {
    let out: Slot<(usize, Option<IpcError>, Option<IpcError>)> = Arc::default();
    let o2 = Arc::clone(&out);
    let w = world(
        SimOs::Linux,
        7,
        move |s, _f, _t| {
            let mut v = Vec::new();
            let err = loop {
                match s.ipc.allocate_region(4096, OWNER) {
                    Ok(r) => v.push(r),
                    Err(e) => break Some(e),
                }
            };
            let zero = s.ipc.allocate_region(0, OWNER).err();
            // Once the owner's charges are released, it may allocate again.
            s.ipc.release_owner(OWNER);
            let again = s.ipc.allocate_region(4096, OWNER);
            assert!(again.is_ok(), "{:?}", again.err());
            *o2.lock().unwrap() = Some((v.len(), err, zero));
        },
        |_s| {},
    );
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(
        out.lock().unwrap().take().unwrap(),
        (
            hg_substrate::ipc::MAX_REGIONS_PER_OWNER,
            Some(IpcError::Quota),
            Some(IpcError::Refused)
        )
    );
}

/// Unordered writes tear at byte granularity: some seeds show a reader
/// half of a write that no release store published. The same seed shows
/// the same thing every run.
#[test]
fn unpublished_writes_tear_deterministically() {
    let run = |seed| {
        let out: Slot<Vec<u8>> = Arc::default();
        let o2 = Arc::clone(&out);
        let held: Slot<Vec<Box<dyn std::any::Any + Send>>> = Arc::default();
        let h2 = Arc::clone(&held);
        let w = world(
            SimOs::Linux,
            seed,
            move |s, _from, to| {
                let wr = s.ipc.allocate_region(4096, OWNER).unwrap();
                let p = s.ipc.share_region(wr).unwrap();
                let c = s.ipc.consumer_of(&p).unwrap();
                p.mapping().write_from(0, &[0xab; 32]).unwrap();
                s.ipc
                    .send(&to, b"c".to_vec(), vec![c.into_handle()])
                    .unwrap();
                keep(&h2, p);
            },
            move |s| {
                let boot = s.ipc.bootstrap().unwrap();
                let o3 = Arc::clone(&o2);
                let lis = on_message(
                    &s,
                    boot.from_parent,
                    &Arc::default(),
                    move |s, _b, mut hs| {
                        let c = ConsumerRegion::from_handle(hs.remove(0)).unwrap();
                        let m = s.ipc.map_region(c.handle()).unwrap();
                        let mut b = vec![0u8; 32];
                        m.read_into(0, &mut b).unwrap();
                        *o3.lock().unwrap() = Some(b);
                    },
                );
                std::mem::forget(lis);
            },
        );
        assert_eq!(w.run(LIMITS), RunEnd::Idle);
        let b = out.lock().unwrap().take().unwrap();
        drop(held);
        b.iter().filter(|&&x| x == 0xab).count()
    };
    let seen: Vec<usize> = (0..40).map(run).collect();
    assert_eq!(seen, (0..40).map(run).collect::<Vec<_>>(), "deterministic");
    assert!(seen.iter().any(|&n| n < 32), "some read is torn: {seen:?}");
    assert!(seen.contains(&32), "some read is whole: {seen:?}");
}
