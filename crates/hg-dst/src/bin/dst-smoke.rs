//! A small simulation binary for `hg-dst` to run until the engine's own
//! process skeletons arrive (M1 phase E).
//!
//! A browser process sends 200 messages, spread over virtual time, to a
//! renderer; the renderer answers each on the browser's `ui` sequence
//! and runs a fork/join phase; a network process commits and releases
//! memory under injected commit failures. The renderer and network
//! processes crash and restart under the fault plan.
//!
//! `--ipc` runs the process protocol over simulated IPC instead (M1
//! phase D2): the renderer, launched with channels to the browser,
//! announces itself with `ready`, and the browser plays 200 `ping` /
//! `pong` rounds with it, at seeded delays, through typed endpoints. A
//! crashed renderer's channels close, so the browser sees the closure;
//! the restarted renderer has no launch channels (relaunching is phase
//! E's).
//!
//! `--bug` plants a bug only some seeds reach: the browser treats a
//! refused post to the renderer (it crashed and has not restarted yet),
//! or with `--ipc` a send to a closed renderer, as impossible and
//! panics. DST should find it, group every failing seed under one
//! signature, and hand back a replay command.
//!
//! `--regions` runs shared memory over simulated IPC instead (M1 phase
//! D6): each round the browser (the broker) sends the renderer a fresh
//! region writer; the renderer fills it with a pattern drawn from the
//! round, seals it and sends the read-only region back; the browser maps
//! it and checks every byte. Beside it, the browser publishes a counter in
//! a producer region the renderer reads through its consumer view with
//! acquire loads. `--region-os linux|macos|windows` picks the OS whose
//! region behaviour the simulator models. A wrong byte or a counter that
//! goes backwards panics.
//!
//! Usage: `dst-smoke --seed N [--crash-ppm N] [--ipc | --regions
//! [--region-os OS]] [--bug]`

use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use hg_substrate::{
    EntropyPurpose, Memory, MemoryKind, PostError, Priority, SequenceHandle, Substrate, Task,
};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{FaultPlan, Limits, SimConfig, SimOs, World};
use hg_types::{Duration, Instant};

struct Args {
    seed: u64,
    crash_ppm: u32,
    ipc: bool,
    regions: bool,
    region_os: SimOs,
    bug: bool,
}

fn parse() -> Option<Args> {
    let mut a = Args {
        seed: 0,
        crash_ppm: 5_000,
        ipc: false,
        regions: false,
        region_os: SimOs::Linux,
        bug: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--seed" => a.seed = it.next()?.parse().ok()?,
            "--crash-ppm" => a.crash_ppm = it.next()?.parse().ok()?,
            "--ipc" => a.ipc = true,
            "--regions" => a.regions = true,
            "--region-os" => {
                a.region_os = match it.next()?.as_str() {
                    "linux" => SimOs::Linux,
                    "macos" => SimOs::MacOs,
                    "windows" => SimOs::Windows,
                    _ => return None,
                }
            }
            "--bug" => a.bug = true,
            _ => return None,
        }
    }
    // `--regions` has no planted bug and is not `--ipc`.
    if a.regions && (a.ipc || a.bug) {
        return None;
    }
    Some(a)
}

type Endpoint = Arc<Mutex<Option<(Substrate, SequenceHandle)>>>;

fn main() -> ExitCode {
    let Some(args) = parse() else {
        eprintln!(
            "usage: dst-smoke --seed N [--crash-ppm N] [--ipc | --regions [--region-os linux|macos|windows]] [--bug]"
        );
        return ExitCode::from(2);
    };
    let mut config = SimConfig::new(args.seed);
    config.faults = FaultPlan {
        crash_per_million_steps: args.crash_ppm,
        commit_fail_per_million: 20_000,
        ..FaultPlan::NONE
    };
    config.memory_budget = Some(1 << 20);
    config.region_os = args.region_os;
    let world = World::new(config);
    if args.regions {
        let keep = regions::start(&world);
        world.run(Limits {
            max_steps: 1_000_000,
            max_time: Instant::from_nanos(60_000_000_000),
        });
        keep.lock().unwrap().clear();
        println!("{}", world.summary().report_line());
        return ExitCode::SUCCESS;
    }
    if args.ipc {
        let keep = ipc::start(&world, args.bug);
        world.run(Limits {
            max_steps: 1_000_000,
            max_time: Instant::from_nanos(60_000_000_000),
        });
        keep.lock().unwrap().clear();
        println!("{}", world.summary().report_line());
        return ExitCode::SUCCESS;
    }
    let backing = || -> Arc<dyn Memory> { Arc::new(OsMemory::new()) };

    let renderer: Endpoint = Arc::new(Mutex::new(None));
    let browser: Endpoint = Arc::new(Mutex::new(None));

    // Renderer: registers its inbox on each (re)start.
    let r_ep = Arc::clone(&renderer);
    world.spawn("renderer", backing(), true, move |s| {
        let inbox = s.exec.sequence("inbox", Priority::UserVisible);
        *r_ep.lock().unwrap() = Some((s, inbox));
    });

    // Network: memory traffic under commit faults.
    world.spawn("network", backing(), true, |s| {
        let seq = s.exec.sequence("cache", Priority::BestEffort);
        for n in 0..20u64 {
            let s2 = s.clone();
            let _ = s.exec.post(
                seq,
                Task::new(move || {
                    let r = s2.mem.reserve(256 * 1024, MemoryKind::CanonicalExemptCache);
                    if let Ok(r) = r {
                        // A refused commit is an expected outcome here.
                        let _ = s2.mem.commit(&r, 64 * 1024);
                        s2.mem.decommit(&r, 0);
                        s2.mem.release(r);
                    }
                }),
                Some(Duration::from_millis(n.saturating_mul(3))),
                Duration::from_millis(1),
            );
        }
    });

    // Browser: sends messages; replies come back to `ui`.
    let (r_ep, b_ep) = (Arc::clone(&renderer), Arc::clone(&browser));
    let bug = args.bug;
    world.spawn("browser", backing(), false, move |s| {
        let ui = s.exec.sequence("ui", Priority::UserBlocking);
        *b_ep.lock().unwrap() = Some((s.clone(), ui));
        let mut rng = s.entropy.stream(EntropyPurpose::Random);
        for n in 0..200u64 {
            let delay = Duration::from_micros(rng.next_u64() % 100_000);
            let (r_ep, b_ep) = (Arc::clone(&r_ep), Arc::clone(&b_ep));
            let _ = s.exec.post(
                ui,
                Task::new(move || send(n, &r_ep, &b_ep, bug)),
                Some(delay),
                Duration::from_micros(200),
            );
        }
    });

    world.run(Limits {
        max_steps: 1_000_000,
        max_time: Instant::from_nanos(60_000_000_000),
    });
    // The browser keeps a substrate in `browser`; clear the endpoints
    // so the world's tasks and substrates are freed with it.
    renderer.lock().unwrap().take();
    browser.lock().unwrap().take();
    println!("{}", world.summary().report_line());
    ExitCode::SUCCESS
}

fn send(n: u64, renderer: &Endpoint, browser: &Endpoint, bug: bool) {
    let target = renderer.lock().unwrap().clone();
    let Some((rs, inbox)) = target else { return };
    let reply_to = browser.lock().unwrap().clone();
    let rs2 = rs.clone();
    let result = rs.exec.post(
        inbox,
        Task::new(move || {
            let parts = Arc::new(Mutex::new([0u64; 4]));
            let p = Arc::clone(&parts);
            rs2.exec.fork_join().for_each_index(4, &move |i| {
                if let Some(slot) = p.lock().unwrap().get_mut(i) {
                    *slot = n.wrapping_mul(31).wrapping_add(i as u64);
                }
            });
            if let Some((bs, ui)) = reply_to {
                let _ = bs.exec.post(ui, Task::new(|| {}), None, Duration::ZERO);
            }
        }),
        None,
        Duration::ZERO,
    );
    match result {
        Ok(()) => {}
        Err(PostError::ShuttingDown) if bug => {
            panic!("bug: message {n} posted to a crashed renderer");
        }
        // The renderer crashed; it re-registers when it restarts.
        Err(_) => {}
    }
}

/// The `--ipc` workload: the process protocol over typed endpoints.
mod ipc {
    use std::sync::{Arc, Mutex};

    use hg_ipc::{Endpoint, PeerBinding, PeerId, Remote, SendError, Serving};
    use hg_ipc_gen::process::{
        BuildId, ExitReason, ProcessControl, ProcessControlHandler, ProcessControlRemote,
        ProcessError, ProcessHost, ProcessHostHandler, ProcessHostRemote, ProcessKind,
    };
    use hg_substrate::ipc::{ChannelReceiver, ChannelSender, OsPeer};
    use hg_substrate::{
        EntropyPurpose, Memory, Priority, RandomStream, SequenceHandle, Substrate, Task,
    };
    use hg_substrate_os::memory::OsMemory;
    use hg_substrate_sim::World;
    use hg_types::Duration;

    const ROUNDS: u64 = 200;

    /// The serving endpoints, kept until the run ends.
    pub type Keep = Arc<Mutex<Vec<Serving>>>;

    struct Browser {
        s: Substrate,
        seq: SequenceHandle,
        control: Arc<Remote<ProcessControl>>,
        rng: Box<dyn RandomStream>,
        bug: bool,
    }

    impl Browser {
        /// Pings round `n` after a seeded delay.
        fn ping_later(&mut self, n: u64) {
            let delay = Duration::from_micros(self.rng.next_u64() % 20_000);
            let (control, bug) = (Arc::clone(&self.control), self.bug);
            let _ = self.s.exec.post(
                self.seq,
                Task::new(move || match control.ping(n) {
                    Err(SendError::Closed) if bug => {
                        panic!("bug: ping {n} sent to a closed renderer");
                    }
                    // The renderer crashed: the conversation is over.
                    _ => {}
                }),
                Some(delay),
                Duration::from_micros(200),
            );
        }
    }

    impl ProcessHostHandler for Browser {
        fn ready(
            &mut self,
            _from: &hg_ipc::CallerIdentity,
            _kind: ProcessKind,
            _build: BuildId,
        ) -> Result<(), ProcessError> {
            self.ping_later(0);
            Ok(())
        }

        fn pong(&mut self, _from: &hg_ipc::CallerIdentity, nonce: u64) -> Result<(), ProcessError> {
            let next = nonce.saturating_add(1);
            if next < ROUNDS {
                self.ping_later(next);
            }
            Ok(())
        }
    }

    struct Renderer {
        host: Remote<ProcessHost>,
    }

    impl ProcessControlHandler for Renderer {
        fn ping(&mut self, _from: &hg_ipc::CallerIdentity, nonce: u64) -> Result<(), ProcessError> {
            // A closed browser is fine: this process is ending too.
            let _ = self.host.pong(nonce);
            Ok(())
        }

        fn exit(
            &mut self,
            _from: &hg_ipc::CallerIdentity,
            _reason: ExitReason,
        ) -> Result<(), ProcessError> {
            Ok(())
        }
    }

    /// Spawns the browser and the renderer and links them.
    pub fn start(world: &World, bug: bool) -> Keep {
        let backing = || -> Arc<dyn Memory> { Arc::new(OsMemory::new()) };
        let keep: Keep = Arc::default();
        let ends: Arc<Mutex<Option<(ChannelReceiver, ChannelSender)>>> = Arc::default();
        // The renderer's process, which the browser's launch endpoint
        // accepts records from: known once it is spawned.
        let renderer_os: Arc<Mutex<Option<OsPeer>>> = Arc::default();
        let (e, k, ros) = (
            Arc::clone(&ends),
            Arc::clone(&keep),
            Arc::clone(&renderer_os),
        );
        let browser = world.spawn("browser", backing(), false, move |s| {
            let Some((from_renderer, to_renderer)) = e.lock().unwrap().take() else {
                return;
            };
            let Some(renderer_os) = *ros.lock().unwrap() else {
                return;
            };
            let seq = s.exec.sequence("host", Priority::UserVisible);
            let mut side = Browser {
                s: s.clone(),
                seq,
                control: Arc::new(Remote::from_sender(&s.ipc, to_renderer)),
                rng: s.entropy.stream(EntropyPurpose::Random),
                bug,
            };
            let serving = Endpoint::<ProcessHost>::from_receiver(&s.ipc, from_renderer)
                .serve_bound(
                    seq,
                    PeerBinding::new(PeerId::new(1, 0), renderer_os),
                    move |m, from| m.dispatch(&mut side, from),
                    |_why| {},
                )
                .unwrap();
            k.lock().unwrap().push(serving);
        });
        let k = Arc::clone(&keep);
        let renderer = world.spawn("renderer", backing(), true, move |s| {
            // A restart has no launch channels: relaunching is phase E's.
            let Some(boot) = s.ipc.bootstrap() else {
                return;
            };
            let host = Remote::<ProcessHost>::from_sender(&s.ipc, boot.to_parent);
            let _ = host.ready(ProcessKind::Renderer, BuildId { id: 1 });
            let mut side = Renderer { host };
            let seq = s.exec.sequence("control", Priority::UserVisible);
            let serving = Endpoint::<ProcessControl>::from_receiver(&s.ipc, boot.from_parent)
                .serve_bound(
                    seq,
                    PeerBinding::new(PeerId::new(0, 0), boot.parent),
                    move |m, from| m.dispatch(&mut side, from),
                    |_why| {},
                )
                .unwrap();
            k.lock().unwrap().push(serving);
        });
        *renderer_os.lock().unwrap() = world.os_peer(renderer);
        *ends.lock().unwrap() = world.link(browser, renderer, 64);
        keep
    }
}

/// The `--regions` workload: shared memory over simulated IPC (D6).
mod regions {
    use std::any::Any;
    use std::sync::{Arc, Mutex};

    use hg_substrate::ipc::{
        ChannelReceiver, ChannelSender, ConsumerRegion, Delivery, Flow, Handle, ProducerRegion,
        ReadOnlyRegion, RegionWriter,
    };
    use hg_substrate::{EntropyPurpose, Memory, Priority, Substrate, Task};
    use hg_substrate_os::memory::OsMemory;
    use hg_substrate_sim::World;
    use hg_types::Duration;

    const ROUNDS: u64 = 60;
    const LEN: usize = 16 * 1024;

    /// Everything kept alive until the run ends.
    pub type Keep = Arc<Mutex<Vec<Box<dyn Any + Send>>>>;

    /// The byte at `i` of round `n`'s region.
    fn pattern(n: u64, i: usize) -> u8 {
        let x = n.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ u64::try_from(i).unwrap_or(0);
        x.to_le_bytes()[(i % 8).min(7)]
    }

    fn listen(
        s: &Substrate,
        rx: ChannelReceiver,
        name: &'static str,
        mut f: impl FnMut(Vec<u8>, Vec<Handle>) + Send + 'static,
    ) -> Option<hg_substrate::ipc::Listening> {
        let seq = s.exec.sequence(name, Priority::UserVisible);
        s.ipc
            .listen(
                rx,
                seq,
                Box::new(move |d| {
                    if let Delivery::Message { bytes, handles } = d {
                        f(bytes, handles);
                    }
                    Flow::Continue
                }),
            )
            .ok()
    }

    /// Spawns the browser and the renderer and links them.
    pub fn start(world: &World) -> Keep {
        let backing = || -> Arc<dyn Memory> { Arc::new(OsMemory::new()) };
        let keep: Keep = Arc::default();
        let ends: Arc<Mutex<Option<(ChannelReceiver, ChannelSender)>>> = Arc::default();
        let (e, k) = (Arc::clone(&ends), Arc::clone(&keep));
        let browser = world.spawn("browser", backing(), false, move |s| {
            let Some((from_renderer, to_renderer)) = e.lock().unwrap().take() else {
                return;
            };
            let to = Arc::new(to_renderer);
            // The counter the renderer reads through its consumer view.
            let Ok(w) = s.ipc.allocate_region(4096) else {
                return;
            };
            let Ok(producer) = s.ipc.share_region(w) else {
                return;
            };
            let producer: Arc<ProducerRegion> = Arc::new(producer);
            if let Ok(c) = s.ipc.consumer_of(&producer) {
                let _ = s.ipc.send(&to, b"counter".to_vec(), vec![c.into_handle()]);
            }
            let seq = s.exec.sequence("rounds", Priority::UserVisible);
            let mut rng = s.entropy.stream(EntropyPurpose::Random);
            // Rounds run in a seeded order; the counter counts the rounds
            // done, so it only goes up.
            let done = Arc::new(std::sync::atomic::AtomicU64::new(0));
            for n in 0..ROUNDS {
                let delay = Duration::from_micros(rng.next_u64() % 50_000);
                let (s2, to2, p2, d2) = (
                    s.clone(),
                    Arc::clone(&to),
                    Arc::clone(&producer),
                    Arc::clone(&done),
                );
                let _ = s.exec.post(
                    seq,
                    Task::new(move || {
                        let c = d2.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let _ = p2.mapping().store_u64_release(0, c.saturating_add(1));
                        if let Ok(w) = s2.ipc.allocate_region(LEN) {
                            // The renderer may have crashed: its channel
                            // is closed and the writer is freed.
                            let _ =
                                s2.ipc
                                    .send(&to2, n.to_le_bytes().to_vec(), vec![w.into_handle()]);
                        }
                    }),
                    Some(delay),
                    Duration::from_micros(200),
                );
            }
            let s3 = s.clone();
            let lis = listen(&s, from_renderer, "checks", move |bytes, mut handles| {
                let (Ok(n), Some(h)) = (<[u8; 8]>::try_from(bytes.as_slice()), handles.pop())
                else {
                    return;
                };
                let n = u64::from_le_bytes(n);
                let Ok(ro) = ReadOnlyRegion::from_handle(h) else {
                    panic!("round {n}: not a read-only region");
                };
                let Ok(m) = s3.ipc.map_region(ro.handle()) else {
                    panic!("round {n}: the region does not map");
                };
                let mut got = vec![0u8; LEN];
                m.read_into(0, &mut got).unwrap();
                for (i, b) in got.iter().enumerate() {
                    assert_eq!(*b, pattern(n, i), "round {n} byte {i}");
                }
            });
            let mut kk = k.lock().unwrap();
            kk.push(Box::new(lis));
            kk.push(Box::new(producer));
            kk.push(Box::new(to));
        });
        let k = Arc::clone(&keep);
        let renderer = world.spawn("renderer", backing(), true, move |s| {
            // A restart has no launch channels: relaunching is phase E's.
            let Some(boot) = s.ipc.bootstrap() else {
                return;
            };
            let to = boot.to_parent;
            let s2 = s.clone();
            let mut counter: Option<(ConsumerRegion, hg_substrate::ipc::Mapping)> = None;
            let mut last = 0u64;
            let lis = listen(&s, boot.from_parent, "fill", move |bytes, mut handles| {
                let Some(h) = handles.pop() else { return };
                if bytes == b"counter" {
                    if let Ok(c) = ConsumerRegion::from_handle(h)
                        && let Ok(m) = s2.ipc.map_region(c.handle())
                    {
                        counter = Some((c, m));
                    }
                    return;
                }
                if let Some((_, m)) = &counter {
                    let v = m.load_u64_acquire(0).unwrap();
                    assert!(v >= last, "the counter went back from {last} to {v}");
                    last = v;
                }
                let (Ok(n), Ok(w)) = (
                    <[u8; 8]>::try_from(bytes.as_slice()),
                    RegionWriter::from_handle(h),
                ) else {
                    return;
                };
                let n = u64::from_le_bytes(n);
                {
                    let Ok(m) = s2.ipc.map_region(w.handle()) else {
                        return;
                    };
                    let fill: Vec<u8> = (0..LEN).map(|i| pattern(n, i)).collect();
                    m.write_from(0, &fill).unwrap();
                }
                if let Ok(ro) = s2.ipc.seal_region(w) {
                    let _ = s2.ipc.send(&to, bytes, vec![ro.into_handle()]);
                }
            });
            k.lock().unwrap().push(Box::new(lis));
        });
        *ends.lock().unwrap() = world.link(browser, renderer, 64);
        keep
    }
}
