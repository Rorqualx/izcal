//! Medium tests: the simulator itself, under explicit seeds.

use std::sync::{Arc, Mutex};

use hg_substrate::{
    EntropyPurpose, Memory, MemoryError, MemoryKind, PostError, Priority, SequenceHandle,
    Substrate, Task,
};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{FaultPlan, Limits, RunEnd, SEQUENCE_CAPACITY, SimConfig, World};
use hg_types::{Duration, Instant};

fn backing() -> Arc<dyn Memory> {
    Arc::new(OsMemory::new())
}

const LIMITS: Limits = Limits {
    max_steps: 100_000,
    max_time: Instant::from_nanos(86_400_000_000_000),
};

type Log = Arc<Mutex<Vec<String>>>;

fn log() -> Log {
    Arc::new(Mutex::new(Vec::new()))
}

fn post(s: &Substrate, seq: SequenceHandle, f: impl FnOnce() + Send + 'static) {
    s.exec
        .post(seq, Task::new(f), None, Duration::ZERO)
        .unwrap();
}

/// Two processes trading messages through each other's sequences, with
/// random delays, memory traffic, a fork/join phase and crash faults.
fn workload(seed: u64, crashes: bool) -> World {
    let mut config = SimConfig::new(seed);
    config.keep_trace_text = true;
    if crashes {
        config.faults = FaultPlan {
            crash_per_million_steps: 20_000,
            ..FaultPlan::NONE
        };
    }
    let world = World::new(config);
    type Inbox = Arc<Mutex<Option<(Substrate, SequenceHandle)>>>;
    let renderer_inbox: Inbox = Arc::new(Mutex::new(None));
    let inbox = Arc::clone(&renderer_inbox);
    world.spawn("renderer", backing(), true, move |s| {
        let seq = s.exec.sequence("inbox", Priority::UserVisible);
        *inbox.lock().unwrap() = Some((s, seq));
    });
    let inbox = Arc::clone(&renderer_inbox);
    world.spawn("browser", backing(), false, move |s| {
        let ui = s.exec.sequence("ui", Priority::UserBlocking);
        let io = s.exec.sequence("io", Priority::MayBlock);
        let mut rng = s.entropy.stream(EntropyPurpose::Random);
        for n in 0..40u64 {
            let delay = Duration::from_micros(rng.next_u64() % 5_000);
            let target = if n % 2 == 0 { ui } else { io };
            let inbox = Arc::clone(&inbox);
            let s2 = s.clone();
            s.exec
                .post(
                    target,
                    Task::new(move || {
                        let r = s2.mem.reserve(64 * 1024, MemoryKind::PerAgent).unwrap();
                        let _ = s2.mem.commit(&r, 8 * 1024);
                        s2.mem.release(r);
                        let guard = inbox.lock().unwrap();
                        if let Some((rs, seq)) = guard.as_ref() {
                            let rs2 = rs.clone();
                            // The renderer may have crashed: a refused post is fine.
                            let _ = rs.exec.post(
                                *seq,
                                Task::new(move || {
                                    let sum = Arc::new(Mutex::new(0u64));
                                    let s3 = Arc::clone(&sum);
                                    rs2.exec.fork_join().for_each_index(8, &move |i| {
                                        *s3.lock().unwrap() += i as u64;
                                    });
                                    assert_eq!(*sum.lock().unwrap(), 28);
                                }),
                                None,
                                Duration::ZERO,
                            );
                        }
                    }),
                    Some(delay),
                    Duration::from_micros(500),
                )
                .unwrap();
        }
    });
    world.run(LIMITS);
    world
}

#[test]
fn same_seed_same_trace_and_different_seeds_differ() {
    for crashes in [false, true] {
        let a = workload(7, crashes);
        let b = workload(7, crashes);
        assert_eq!(a.summary(), b.summary());
        assert_eq!(a.trace_text(), b.trace_text());
        assert!(crashes || a.summary().steps > 80, "{:?}", a.summary());
        let c = workload(8, crashes);
        assert_ne!(a.summary().trace_hash, c.summary().trace_hash);
    }
}

#[test]
fn crash_faults_crash_and_restart() {
    let w = (0..20)
        .map(|seed| workload(seed, true))
        .find(|w| w.summary().crashes > 0)
        .expect("a 2% crash rate crashes some seed in 20");
    let text = w.trace_text().unwrap();
    assert!(text.contains("crash "), "{text}");
}

#[test]
fn virtual_time_jumps_to_the_timer_and_fires_inside_its_band() {
    let w = World::new(SimConfig::new(1));
    let seen: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    let out = Arc::clone(&seen);
    w.spawn("p", backing(), false, move |s| {
        let seq = s.exec.sequence("t", Priority::BestEffort);
        let s2 = s.clone();
        let out = Arc::clone(&out);
        s.exec
            .post(
                seq,
                Task::new(move || *out.lock().unwrap() = Some(s2.clock.monotonic())),
                Some(Duration::from_secs(3600)),
                Duration::from_secs(2),
            )
            .unwrap();
    });
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    let at = seen.lock().unwrap().unwrap().as_nanos();
    assert!(
        (3_600_000_000_000..=3_602_000_000_000).contains(&at),
        "{at}"
    );
}

#[test]
fn timers_fire_in_firing_order_and_sequences_keep_post_order() {
    let w = World::new(SimConfig::new(2));
    let l = log();
    let out = Arc::clone(&l);
    w.spawn("p", backing(), false, move |s| {
        let seq = s.exec.sequence("q", Priority::UserVisible);
        for (n, ms) in [(2, 30u64), (0, 10), (1, 20)] {
            let out = Arc::clone(&out);
            s.exec
                .post(
                    seq,
                    Task::new(move || out.lock().unwrap().push(format!("t{n}"))),
                    Some(Duration::from_millis(ms)),
                    Duration::ZERO,
                )
                .unwrap();
        }
        for n in 0..5 {
            let out = Arc::clone(&out);
            post(&s, seq, move || out.lock().unwrap().push(format!("q{n}")));
        }
    });
    w.run(LIMITS);
    assert_eq!(
        *l.lock().unwrap(),
        ["q0", "q1", "q2", "q3", "q4", "t0", "t1", "t2"]
    );
}

#[test]
fn a_full_sequence_refuses_and_a_crashed_process_shuts_its_doors() {
    let w = World::new(SimConfig::new(3));
    let kept: Arc<Mutex<Option<(Substrate, SequenceHandle)>>> = Arc::new(Mutex::new(None));
    let out = Arc::clone(&kept);
    let pid = w.spawn("p", backing(), false, move |s| {
        let seq = s.exec.sequence("q", Priority::UserVisible);
        for _ in 0..SEQUENCE_CAPACITY {
            post(&s, seq, || {});
        }
        assert_eq!(
            s.exec.post(seq, Task::new(|| {}), None, Duration::ZERO),
            Err(PostError::QueueFull)
        );
        assert_eq!(
            s.exec.post(
                SequenceHandle::from_raw(99),
                Task::new(|| {}),
                None,
                Duration::ZERO
            ),
            Err(PostError::UnknownSequence)
        );
        *out.lock().unwrap() = Some((s, seq));
    });
    w.step(); // the entry task
    assert!(w.crash(pid));
    assert!(!w.is_alive(pid));
    let (s, seq) = kept.lock().unwrap().take().unwrap();
    assert_eq!(
        s.exec.post(seq, Task::new(|| {}), None, Duration::ZERO),
        Err(PostError::ShuttingDown)
    );
    // The queued tasks went with the crash.
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(w.summary().steps, 1);
}

#[test]
fn a_restart_runs_the_entry_again_with_a_fresh_substrate_and_frees_memory() {
    let w = World::new(SimConfig::new(4));
    let starts = Arc::new(Mutex::new(0u32));
    let back = Arc::new(OsMemory::new());
    let count = Arc::clone(&starts);
    let pid = w.spawn("p", Arc::clone(&back) as Arc<dyn Memory>, true, move |s| {
        *count.lock().unwrap() += 1;
        let r = s.mem.reserve(1 << 20, MemoryKind::ProcessPart).unwrap();
        s.mem.commit(&r, 4096).unwrap();
        assert_eq!(s.mem.stats().committed, 4096);
        // The region stays reserved and committed until the crash.
        let _held = r;
    });
    w.step();
    assert!(back.stats().reserved > 0);
    w.crash(pid);
    assert_eq!(back.stats().reserved, 0, "the crash released the region");
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert!(w.is_alive(pid));
    assert_eq!(*starts.lock().unwrap(), 2);
    // Restarted between 1 ms and 11 ms after the crash.
    let t = w.now().as_nanos();
    assert!((1_000_000..=11_000_000).contains(&t), "{t}");
}

#[test]
fn the_budget_and_commit_faults_refuse_commits() {
    let mut config = SimConfig::new(5);
    config.memory_budget = Some(8192);
    let w = World::new(config);
    w.spawn("p", backing(), false, |s| {
        let r = s.mem.reserve(1 << 20, MemoryKind::PerAgent).unwrap();
        s.mem.commit(&r, 8192).unwrap();
        assert_eq!(s.mem.commit(&r, 8193), Err(MemoryError::CommitFailed));
        s.mem.decommit(&r, 4096);
        assert_eq!(s.mem.stats().committed, 4096);
        s.mem.release(r);
        assert_eq!(s.mem.stats().reserved, 0);
    });
    w.run(LIMITS);

    let mut config = SimConfig::new(6);
    config.faults.commit_fail_per_million = 1_000_000;
    let w = World::new(config);
    w.spawn("p", backing(), false, |s| {
        let r = s.mem.reserve(1 << 16, MemoryKind::PerAgent).unwrap();
        assert_eq!(s.mem.commit(&r, 1), Err(MemoryError::CommitFailed));
        s.mem.release(r);
    });
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(w.summary().steps, 1);
}

#[test]
fn phases_cover_every_index_in_a_seeded_order() {
    let order_for = |seed| {
        let w = World::new(SimConfig::new(seed));
        let l = log();
        let out = Arc::clone(&l);
        w.spawn("p", backing(), false, move |s| {
            let out = Arc::clone(&out);
            s.exec.fork_join().for_each_index(16, &move |i| {
                out.lock().unwrap().push(i.to_string());
            });
        });
        w.run(LIMITS);
        l.lock().unwrap().clone()
    };
    let a = order_for(10);
    let mut sorted: Vec<usize> = a.iter().map(|x| x.parse().unwrap()).collect();
    sorted.sort_unstable();
    assert_eq!(sorted, (0..16).collect::<Vec<_>>());
    assert_eq!(a, order_for(10));
    assert!((11..20).any(|seed| order_for(seed) != a));
}

#[test]
fn randomness_and_wall_time_follow_the_seed() {
    let draw = |seed| {
        let w = World::new(SimConfig::new(seed));
        let out = Arc::new(Mutex::new((0u64, [0u8; 16], 0i64)));
        let o = Arc::clone(&out);
        w.spawn("p", backing(), false, move |s| {
            let mut key = [0u8; 16];
            s.entropy.crypto().fill(&mut key).unwrap();
            let r = s.entropy.stream(EntropyPurpose::Random).next_u64();
            *o.lock().unwrap() = (r, key, s.clock.wall().as_unix_nanos());
        });
        w.run(LIMITS);
        *out.lock().unwrap()
    };
    let a = draw(11);
    assert_eq!(a, draw(11));
    assert_ne!(a.0, draw(12).0);
    assert_ne!(a.1, draw(12).1);
    assert_eq!(a.2, SimConfig::new(0).wall_epoch.as_unix_nanos());
}

#[test]
fn limits_stop_the_run() {
    let w = World::new(SimConfig::new(12));
    w.spawn("p", backing(), false, |s| {
        let seq = s.exec.sequence("q", Priority::UserVisible);
        for _ in 0..10 {
            post(&s, seq, || {});
        }
        s.exec
            .post(
                seq,
                Task::new(|| {}),
                Some(Duration::from_secs(10)),
                Duration::ZERO,
            )
            .unwrap();
    });
    let end = w.run(Limits {
        max_steps: 5,
        ..LIMITS
    });
    assert_eq!(end, RunEnd::StepLimit);
    let end = w.run(Limits {
        max_steps: 1000,
        max_time: Instant::from_nanos(1_000_000_000),
    });
    assert_eq!(end, RunEnd::TimeLimit);
    assert_eq!(w.summary().steps, 11);
    assert_eq!(w.run(LIMITS), RunEnd::Idle);
    assert_eq!(w.now().as_nanos(), 10_000_000_000);
}

/// M1's recording: a run's header carries this build's ID, its data
/// versions (none) and its decision-point log (the trace), and decodes
/// back; a header without the log is refused.
#[test]
fn a_run_is_a_recording_with_the_shared_header() {
    let run = |seed| {
        let w = workload(seed, false);
        let _ = w.run(LIMITS);
        (w.summary(), w.recording_header().unwrap())
    };
    let (summary, h) = run(5);
    assert_eq!(h.build_id(), hg_types::BUILD_ID);
    assert!(h.data_versions().is_empty());
    let log = h.decision_log().unwrap();
    assert_eq!(log.trace_hash, summary.trace_hash);
    assert!(log.entries > 0);
    assert_eq!(
        h.policy(),
        hg_types::HeaderPolicy::Digest(hg_types::NO_POLICY)
    );
    assert_eq!(h.process(), hg_types::ProcessKind::Simulation);
    assert_eq!(
        hg_types::RecordingHeader::decode(&h.encode()),
        Ok(h.clone())
    );
    // A function of the run: the same seed gives the same header.
    assert_eq!(run(5).1, h);
    assert_ne!(run(6).1.profile(), h.profile());
}

/// DU-M0a's install model, the device half of the "child binary replaced
/// between two launches" scenario: a handle opened before an update keeps
/// the old binary while a fresh open sees the new one, and locks never
/// wait. The launcher half (the BuildId check and DU-3's restart-required
/// state) is phase E's.
#[test]
fn the_install_model_keeps_open_files_across_an_update() {
    use hg_substrate::install::{InstallError, InstallPath, LockMode};
    let p = |s: &str| InstallPath::new(s).unwrap();
    let w = World::new(SimConfig::new(9));
    let d = w.install_device();
    d.write_new(&p("builds/1/renderer"), b"build 1").unwrap();
    let held = d.open_inventory(&[p("builds/1/renderer")]).unwrap();
    // The update stages a new binary and renames it over the old path.
    d.write_new(&p("staging/renderer"), b"build 2").unwrap();
    d.rename(&p("staging/renderer"), &p("builds/1/renderer"))
        .unwrap();
    assert_eq!(held[0].read_all().unwrap(), b"build 1");
    assert_eq!(
        d.open_read(&p("builds/1/renderer"))
            .unwrap()
            .read_all()
            .unwrap(),
        b"build 2"
    );
    // A new build is renamed into a new directory; renaming over an
    // existing build is refused, as on the OS, and a path renamed to
    // itself is unchanged.
    d.write_new(&p("staging2/renderer"), b"build 3").unwrap();
    assert_eq!(
        d.rename(&p("staging2"), &p("builds/1")).err(),
        Some(InstallError::Invalid)
    );
    d.rename(&p("staging2"), &p("builds/2")).unwrap();
    d.rename(&p("builds/2"), &p("builds/2")).unwrap();
    assert_eq!(
        d.open_read(&p("builds/2/renderer"))
            .unwrap()
            .read_all()
            .unwrap(),
        b"build 3"
    );
    assert_eq!(d.open_read(&p("builds")).err(), Some(InstallError::Invalid));
    assert_eq!(
        d.write_new(&p("builds/1/renderer"), b"x").err(),
        Some(InstallError::AlreadyExists)
    );
    let shared = d.lock(&p("build.lock"), LockMode::Shared).unwrap();
    assert_eq!(
        d.lock(&p("build.lock"), LockMode::Exclusive).err(),
        Some(InstallError::WouldBlock)
    );
    drop(shared);
    let _x = d.lock(&p("build.lock"), LockMode::Exclusive).unwrap();
    d.delete_tree(&p("builds")).unwrap();
    assert_eq!(
        d.open_read(&p("builds/1/renderer")).err(),
        Some(InstallError::NotFound)
    );
    // Every operation is in the trace, so the same steps give the same
    // hash.
    assert_ne!(
        w.summary().trace_hash,
        World::new(SimConfig::new(9)).summary().trace_hash
    );
}
