//! The simulated world: every simulated process, one seeded scheduler,
//! virtual time and the fault schedule (Stage 1 design 3.2).
//!
//! **One step** runs one task to completion. The scheduler collects every
//! runnable sequence of every live process, in a fixed order, and picks
//! one with the scheduling stream. When nothing is runnable, virtual time
//! jumps to the earliest timer and every timer due by then fires, in
//! firing-time order. A delayed task's firing time is drawn inside its
//! tolerance band when it is posted, and the trace records the draw.
//!
//! **Faults** come from their own stream, so a fault-free run draws
//! nothing from it and its trace does not depend on the fault settings.
//! A crash is a scheduler action (CS-ER-2): it drops the process's
//! queued tasks, timers and memory, never panics, and may schedule a
//! restart that runs the process's entry function again with a fresh
//! substrate.
//!
//! The world is driven by one thread. Tasks run with no lock held, so
//! they may post, read the clock and draw randomness freely, but a task
//! must not call [`World::step`] or [`World::run`].

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

use hg_substrate::prng::{Xoshiro256StarStar, splitmix64};
use hg_substrate::{Memory, MemoryKind, Priority, RandomStream, Region, Substrate, Task};
use hg_types::{
    Common, DecisionLogRef, Duration, HeaderError, Instant, NO_POLICY, ProcessKind,
    ProfileIdentity, RecordingHeader, SandboxTier, SystemTime, Text,
};

use crate::trace::Trace;

/// Mixed into the seed for the recording's profile identity, so it is
/// drawn from a stream the run itself never uses.
const PROFILE_STREAM: u64 = 0x5052_4f46_494c_4531;

/// Pending tasks a sequence may hold, queued and delayed together, as in
/// the OS executor (CS-AS-6 (c)): 1024†.
pub const SEQUENCE_CAPACITY: usize = 1024;

/// A simulated process: an index into the world, stable across restarts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(pub u32);

/// Which faults the scheduler injects, as rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaultPlan {
    /// Chance, per million steps, that a random live process crashes
    /// before the step.
    pub crash_per_million_steps: u32,
    /// Chance, per million commits, that a commit fails.
    pub commit_fail_per_million: u32,
    /// A restart happens 1 ms plus up to this much virtual time after the
    /// crash.
    pub restart_delay_max: Duration,
}

impl FaultPlan {
    /// No faults.
    pub const NONE: Self = Self {
        crash_per_million_steps: 0,
        commit_fail_per_million: 0,
        restart_delay_max: Duration::from_millis(10),
    };
}

/// A world's settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimConfig {
    /// The seed every random choice derives from.
    pub seed: u64,
    /// The injected faults.
    pub faults: FaultPlan,
    /// The wall clock at virtual time zero.
    pub wall_epoch: SystemTime,
    /// Committed bytes each process may hold; past it, commits fail.
    pub memory_budget: Option<u64>,
    /// The page size the ledger rounds to. Fixed, not the host's, so a
    /// trace is the same on every OS.
    pub page_size: u64,
    /// Keep the trace's text, not only its hash.
    pub keep_trace_text: bool,
}

impl SimConfig {
    /// Defaults for `seed`: no faults, wall time 2026-01-01T00:00:00Z at
    /// virtual time zero, no memory budget, 4 KiB pages, hash-only trace.
    pub const fn new(seed: u64) -> Self {
        Self {
            seed,
            faults: FaultPlan::NONE,
            wall_epoch: SystemTime::from_unix_nanos(1_767_225_600_000_000_000),
            memory_budget: None,
            page_size: 4096,
            keep_trace_text: false,
        }
    }
}

/// What one call to [`World::step`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Ran one task.
    Ran,
    /// Nothing was runnable; virtual time advanced and timers fired.
    Advanced,
    /// Nothing is runnable and no timer is pending.
    Idle,
    /// The next timer lies past the run's time limit; time did not move.
    TimeLimit,
}

/// Bounds for [`World::run`]. The run stops at whichever comes first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Most tasks to run.
    pub max_steps: u64,
    /// Virtual time not to pass.
    pub max_time: Instant,
}

/// Why [`World::run`] returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEnd {
    /// No task and no timer is left.
    Idle,
    /// `max_steps` tasks ran.
    StepLimit,
    /// The next timer lies past `max_time`.
    TimeLimit,
}

/// A run's totals: what a determinism check compares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    /// The seed.
    pub seed: u64,
    /// Tasks run.
    pub steps: u64,
    /// Virtual time reached.
    pub now: Instant,
    /// Crashes injected or requested.
    pub crashes: u64,
    /// The trace's hash.
    pub trace_hash: u64,
}

impl Summary {
    /// The one-line report a simulation binary prints last on success;
    /// `hg-dst` parses it.
    pub fn report_line(&self) -> String {
        format!(
            "hg-dst: seed={} steps={} time_ns={} crashes={} trace={:016x}",
            self.seed,
            self.steps,
            self.now.as_nanos(),
            self.crashes,
            self.trace_hash
        )
    }
}

/// A process's entry function: called with a fresh substrate when the
/// process starts and after each restart.
pub type Entry = Arc<dyn Fn(Substrate) + Send + Sync>;

pub(crate) struct Seq {
    pub(crate) name: &'static str,
    pub(crate) queue: VecDeque<(u64, Task)>,
    pub(crate) delayed: usize,
    pub(crate) posted: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct Reservation {
    pub(crate) len: u64,
    pub(crate) committed: u64,
    pub(crate) kind: MemoryKind,
    pub(crate) host_len: usize,
}

pub(crate) struct Proc {
    pub(crate) name: &'static str,
    pub(crate) generation: u32,
    pub(crate) alive: bool,
    entry: Entry,
    restart: bool,
    pub(crate) backing: Arc<dyn Memory>,
    pub(crate) seqs: Vec<Seq>,
    pub(crate) regions: BTreeMap<usize, Reservation>,
    pub(crate) reserved: u64,
    pub(crate) committed: u64,
}

/// Where a delayed task goes when it fires.
#[derive(Clone, Copy)]
pub(crate) struct Slot {
    pub(crate) pid: ProcessId,
    pub(crate) generation: u32,
    pub(crate) seq: usize,
    pub(crate) number: u64,
}

enum TimerItem {
    Task {
        pid: ProcessId,
        generation: u32,
        seq: usize,
        number: u64,
        task: Task,
    },
    Restart(ProcessId),
}

pub(crate) struct State {
    pub(crate) config: SimConfig,
    pub(crate) now: Instant,
    steps: u64,
    crashes: u64,
    pub(crate) sched: Xoshiro256StarStar,
    pub(crate) fault: Xoshiro256StarStar,
    pub(crate) procs: Vec<Proc>,
    /// Pending timers by (firing time in ns, post order).
    timers: BTreeMap<(u64, u64), TimerItem>,
    next_timer: u64,
    pub(crate) trace: Trace,
    /// Every IPC channel.
    pub(crate) ipc: crate::ipc::Channels,
    /// The world itself, for tasks the state makes (IPC deliveries).
    pub(crate) ipc_self: Weak<Shared>,
}

pub(crate) struct Shared {
    state: Mutex<State>,
}

impl Shared {
    pub(crate) fn lock(&self) -> MutexGuard<'_, State> {
        // Simulation binaries abort on panic (CS-ER-2), so a poisoned lock
        // only survives in a test that is already failing.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Draws a value in `0..n` (n > 0) from `rng`.
pub(crate) fn below(rng: &mut Xoshiro256StarStar, n: u64) -> u64 {
    rng.next_u64().checked_rem(n).unwrap_or(0)
}

/// A seed derived from `parts`, independent of the order streams are made.
pub(crate) fn derive(parts: &[u64]) -> u64 {
    let mut s = 0x6a09_e667_f3bc_c908_u64;
    for &p in parts {
        s ^= p;
        s = splitmix64(&mut s);
    }
    s
}

/// Stream tags for [`derive`].
pub(crate) mod tag {
    pub(crate) const SCHED: u64 = 1;
    pub(crate) const FAULT: u64 = 2;
    pub(crate) const HASH_KEY: u64 = 3;
    pub(crate) const PROCESS: u64 = 4;
}

impl State {
    pub(crate) fn proc_mut(&mut self, pid: ProcessId, generation: u32) -> Option<&mut Proc> {
        self.procs
            .get_mut(usize::try_from(pid.0).ok()?)
            .filter(|p| p.alive && p.generation == generation)
    }

    pub(crate) fn proc(&self, pid: ProcessId, generation: u32) -> Option<&Proc> {
        self.procs
            .get(usize::try_from(pid.0).ok()?)
            .filter(|p| p.alive && p.generation == generation)
    }

    pub(crate) fn add_timer(
        &mut self,
        slot: Slot,
        task: Task,
        delay: Duration,
        tolerance: Duration,
    ) {
        let Slot {
            pid,
            generation,
            seq,
            number,
        } = slot;
        let due = self
            .now
            .checked_add(delay)
            .unwrap_or(Instant::from_nanos(u64::MAX));
        let band = u64::try_from(tolerance.as_nanos()).unwrap_or(u64::MAX);
        let slack = match band.checked_add(1) {
            Some(width) => below(&mut self.sched, width),
            None => self.sched.next_u64(),
        };
        let fire = due.as_nanos().saturating_add(slack);
        let (pname, gen_, sname) = self
            .proc(pid, generation)
            .map_or(("?", generation, "?"), |p| {
                (
                    p.name,
                    p.generation,
                    p.seqs.get(seq).map_or("?", |s| s.name),
                )
            });
        self.trace.record(format_args!(
            "timer {pname}.{gen_}/{sname}#{number} due={} fire={fire}",
            due.as_nanos()
        ));
        let id = self.next_timer;
        self.next_timer = self.next_timer.saturating_add(1);
        self.timers.insert(
            (fire, id),
            TimerItem::Task {
                pid,
                generation,
                seq,
                number,
                task,
            },
        );
    }
}

/// What a crash removed, dropped and released once the lock is free (a
/// task's captures may themselves reach the world when dropped).
#[derive(Default)]
struct Wreckage {
    tasks: Vec<Task>,
    regions: Vec<(Arc<dyn Memory>, Region)>,
    /// IPC messages, handles and listeners the crash closed.
    ipc: crate::ipc::Dropped,
}

impl Wreckage {
    fn clear(self) {
        drop(self.tasks);
        drop(self.ipc);
        for (backing, region) in self.regions {
            backing.release(region);
        }
    }
}

/// The simulated world.
pub struct World {
    shared: Arc<Shared>,
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.summary();
        f.debug_struct("World")
            .field("seed", &s.seed)
            .field("steps", &s.steps)
            .field("now", &s.now)
            .finish()
    }
}

impl World {
    /// A world for `config`. Also sets the OS process's hash key from the
    /// seed (CS-AS-2) if no key is set yet: `hg-dst` runs one seed per OS
    /// process, so there the maps are keyed by the seed; several worlds in
    /// one test process share the first world's key.
    pub fn new(config: SimConfig) -> Self {
        let seed = config.seed;
        let _first_key_wins = hg_types::set_process_hash_key(
            derive(&[seed, tag::HASH_KEY, 0]),
            derive(&[seed, tag::HASH_KEY, 1]),
        );
        let state = State {
            config,
            now: Instant::from_nanos(0),
            steps: 0,
            crashes: 0,
            sched: Xoshiro256StarStar::from_seed(derive(&[seed, tag::SCHED])),
            fault: Xoshiro256StarStar::from_seed(derive(&[seed, tag::FAULT])),
            procs: Vec::new(),
            timers: BTreeMap::new(),
            next_timer: 0,
            trace: Trace::new(config.keep_trace_text),
            ipc: crate::ipc::Channels::default(),
            ipc_self: Weak::new(),
        };
        Self {
            shared: Arc::new_cyclic(|me| {
                let mut state = state;
                state.ipc_self = me.clone();
                Shared {
                    state: Mutex::new(state),
                }
            }),
        }
    }

    /// Starts a process named `name`. Its entry function runs as the
    /// first task of its `main` sequence, with a substrate whose memory
    /// is backed by `backing` (the simulator keeps the ledger, budget and
    /// faults; the backing provides real addresses). With `restart`, a
    /// crash schedules a restart.
    pub fn spawn(
        &self,
        name: &'static str,
        backing: Arc<dyn Memory>,
        restart: bool,
        entry: impl Fn(Substrate) + Send + Sync + 'static,
    ) -> ProcessId {
        let mut st = self.shared.lock();
        let pid = ProcessId(u32::try_from(st.procs.len()).unwrap_or(u32::MAX));
        st.procs.push(Proc {
            name,
            generation: 0,
            alive: false,
            entry: Arc::new(entry),
            restart,
            backing,
            seqs: Vec::new(),
            regions: BTreeMap::new(),
            reserved: 0,
            committed: 0,
        });
        self.start(&mut st, pid, 0);
        pid
    }

    /// Brings incarnation `generation` of `pid` up and queues its entry.
    fn start(&self, st: &mut State, pid: ProcessId, generation: u32) {
        let seed = st.config.seed;
        let Some(p) = st
            .procs
            .get_mut(usize::try_from(pid.0).unwrap_or(usize::MAX))
        else {
            return;
        };
        p.generation = generation;
        p.alive = true;
        p.seqs = vec![Seq {
            name: "main",
            queue: VecDeque::new(),
            delayed: 0,
            posted: 1,
        }];
        let substrate = crate::substrate_for(
            &self.shared,
            pid,
            generation,
            derive(&[seed, tag::PROCESS, u64::from(pid.0), u64::from(generation)]),
        );
        let entry = Arc::clone(&p.entry);
        if let Some(main) = p.seqs.first_mut() {
            main.queue
                .push_back((0, Task::new(move || entry(substrate))));
        }
        let name = p.name;
        st.trace.record(format_args!("start {name}.{generation}"));
    }

    /// Crashes `pid` now, as an injected fault would. Returns false if it
    /// was not running.
    pub fn crash(&self, pid: ProcessId) -> bool {
        let mut st = self.shared.lock();
        let wreck = Self::crash_locked(&mut st, pid);
        drop(st);
        let crashed = wreck.is_some();
        if let Some(w) = wreck {
            w.clear();
        }
        crashed
    }

    fn crash_locked(st: &mut State, pid: ProcessId) -> Option<Wreckage> {
        let index = usize::try_from(pid.0).ok()?;
        let p = st.procs.get_mut(index).filter(|p| p.alive)?;
        p.alive = false;
        let mut wreck = Wreckage::default();
        for s in p.seqs.drain(..) {
            wreck.tasks.extend(s.queue.into_iter().map(|(_, t)| t));
        }
        for (base, r) in std::mem::take(&mut p.regions) {
            wreck.regions.push((
                Arc::clone(&p.backing),
                Region::new(base, r.host_len, r.kind),
            ));
        }
        p.reserved = 0;
        p.committed = 0;
        let (name, generation, restart) = (p.name, p.generation, p.restart);
        let doomed: Vec<(u64, u64)> = st
            .timers
            .iter()
            .filter(|(_, item)| matches!(item, TimerItem::Task { pid: p2, .. } if *p2 == pid))
            .map(|(&k, _)| k)
            .collect();
        for k in doomed {
            if let Some(TimerItem::Task { task, .. }) = st.timers.remove(&k) {
                wreck.tasks.push(task);
            }
        }
        st.crashes = st.crashes.saturating_add(1);
        st.trace.record(format_args!("crash {name}.{generation}"));
        crate::ipc::Channels::crash(st, pid, &mut wreck.ipc);
        if restart {
            let extra = u64::try_from(st.config.faults.restart_delay_max.as_nanos())
                .unwrap_or(u64::MAX)
                .saturating_add(1);
            let delay = 1_000_000_u64.saturating_add(below(&mut st.fault, extra));
            let at = st.now.as_nanos().saturating_add(delay);
            let id = st.next_timer;
            st.next_timer = st.next_timer.saturating_add(1);
            st.timers.insert((at, id), TimerItem::Restart(pid));
            st.trace.record(format_args!("restart-at {name} t={at}"));
        }
        Some(wreck)
    }

    /// Takes one scheduler step (see the module docs).
    pub fn step(&self) -> Step {
        self.step_until(None)
    }

    fn step_until(&self, max_time: Option<Instant>) -> Step {
        let mut st = self.shared.lock();

        // An injected crash, before choosing work.
        let crash_rate = st.config.faults.crash_per_million_steps;
        if crash_rate > 0 && below(&mut st.fault, 1_000_000) < u64::from(crash_rate) {
            let live: Vec<ProcessId> = st
                .procs
                .iter()
                .enumerate()
                .filter(|(_, p)| p.alive)
                .filter_map(|(i, _)| u32::try_from(i).ok().map(ProcessId))
                .collect();
            let n = u64::try_from(live.len()).unwrap_or(0);
            if n > 0 {
                let pick = usize::try_from(below(&mut st.fault, n)).unwrap_or(0);
                if let Some(&pid) = live.get(pick)
                    && let Some(w) = Self::crash_locked(&mut st, pid)
                {
                    drop(st);
                    w.clear();
                    st = self.shared.lock();
                }
            }
        }

        // Every runnable sequence of every live process, in a fixed order.
        let runnable: Vec<(usize, usize)> = st
            .procs
            .iter()
            .enumerate()
            .filter(|(_, p)| p.alive)
            .flat_map(|(pi, p)| {
                p.seqs
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| !s.queue.is_empty())
                    .map(move |(si, _)| (pi, si))
            })
            .collect();
        if let Ok(n) = u64::try_from(runnable.len())
            && n > 0
        {
            let pick = usize::try_from(below(&mut st.sched, n)).unwrap_or(0);
            let (pi, si) = runnable.get(pick).copied().unwrap_or((0, 0));
            let picked = st.procs.get_mut(pi).and_then(|p| {
                let (pname, generation) = (p.name, p.generation);
                let s = p.seqs.get_mut(si)?;
                let (number, task) = s.queue.pop_front()?;
                Some((pname, generation, s.name, number, task))
            });
            if let Some((pname, generation, sname, number, task)) = picked {
                st.steps = st.steps.saturating_add(1);
                let (step, t) = (st.steps, st.now.as_nanos());
                st.trace.record(format_args!(
                    "{step} t={t} run {pname}.{generation}/{sname}#{number}"
                ));
                drop(st);
                task.run();
                return Step::Ran;
            }
        }

        // Nothing runnable: jump to the earliest timer.
        let Some((&(at, _), _)) = st.timers.first_key_value() else {
            return Step::Idle;
        };
        if max_time.is_some_and(|limit| at > limit.as_nanos()) {
            return Step::TimeLimit;
        }
        if at > st.now.as_nanos() {
            st.now = Instant::from_nanos(at);
        }
        let now = st.now.as_nanos();
        let mut restarts = Vec::new();
        while let Some(entry) = st.timers.first_entry() {
            if entry.key().0 > now {
                break;
            }
            match entry.remove() {
                TimerItem::Task {
                    pid,
                    generation,
                    seq,
                    number,
                    task,
                } => {
                    if let Some(s) = st
                        .proc_mut(pid, generation)
                        .and_then(|p| p.seqs.get_mut(seq))
                    {
                        s.delayed = s.delayed.saturating_sub(1);
                        s.queue.push_back((number, task));
                    }
                }
                TimerItem::Restart(pid) => restarts.push(pid),
            }
        }
        for pid in restarts {
            let next = st
                .procs
                .get(usize::try_from(pid.0).unwrap_or(usize::MAX))
                .filter(|p| !p.alive)
                .map(|p| p.generation.saturating_add(1));
            if let Some(generation) = next {
                self.start(&mut st, pid, generation);
            }
        }
        Step::Advanced
    }

    /// Steps until idle or a limit.
    pub fn run(&self, limits: Limits) -> RunEnd {
        loop {
            if self.shared.lock().steps >= limits.max_steps {
                return RunEnd::StepLimit;
            }
            match self.step_until(Some(limits.max_time)) {
                Step::Ran | Step::Advanced => {}
                Step::Idle => return RunEnd::Idle,
                Step::TimeLimit => return RunEnd::TimeLimit,
            }
        }
    }

    /// Makes `child`'s launch channels to `parent`, which `child` takes
    /// with `Ipc::bootstrap`, and returns `parent`'s ends: (from the
    /// child, to the child). What launching a process gives its launcher;
    /// `None` if either process is not running.
    pub fn link(
        &self,
        parent: ProcessId,
        child: ProcessId,
        capacity: u32,
    ) -> Option<(
        hg_substrate::ipc::ChannelReceiver,
        hg_substrate::ipc::ChannelSender,
    )> {
        crate::ipc::link(&self.shared, parent, child, capacity)
    }

    /// Makes `pid`'s next IPC send a record the receiving transport
    /// refuses with `kind`: the peer sees [`hg_substrate::ipc::Delivery::Violation`]
    /// naming `pid`, after the messages queued before it, and the channel
    /// closes. The sender's `send` still returns `Ok`, as a compromised
    /// peer's would. The adversarial-peer fault (security scope 6.6).
    pub fn corrupt_next_send(&self, pid: ProcessId, kind: hg_substrate::ipc::FaultKind) {
        let mut st = self.shared.lock();
        crate::ipc::corrupt_next(&mut st, pid, kind);
    }

    /// Virtual time now.
    pub fn now(&self) -> Instant {
        self.shared.lock().now
    }

    /// `pid`'s current incarnation as its OS identity, the value a
    /// bound receiver is given ([`hg_substrate::ipc::Ipc::listen_bound`]),
    /// or `None` if no such process is running.
    pub fn os_peer(&self, pid: ProcessId) -> Option<hg_substrate::ipc::OsPeer> {
        let st = self.shared.lock();
        let p = st.procs.get(usize::try_from(pid.0).ok()?)?;
        p.alive
            .then(|| hg_substrate::ipc::OsPeer::new(pid.0, u64::from(p.generation)))
    }

    /// Whether `pid` is running (not crashed and awaiting restart).
    pub fn is_alive(&self, pid: ProcessId) -> bool {
        self.shared
            .lock()
            .procs
            .get(usize::try_from(pid.0).unwrap_or(usize::MAX))
            .is_some_and(|p| p.alive)
    }

    /// The run's totals so far.
    pub fn summary(&self) -> Summary {
        let st = self.shared.lock();
        Summary {
            seed: st.config.seed,
            steps: st.steps,
            now: st.now,
            crashes: st.crashes,
            trace_hash: st.trace.hash(),
        }
    }

    /// The trace's text, if the config kept it.
    pub fn trace_text(&self) -> Option<String> {
        self.shared.lock().trace.text().map(str::to_owned)
    }

    /// This run as M1's recording (Stage 1 8.1: "recordings carry the
    /// engine build ID, the data versions and the decision-point log"):
    /// the shared header ([`RecordingHeader`]) for a simulation of every
    /// process, whose decision-point log is this run's trace (its hash and
    /// its number of records). The simulator uses no data sets, M1 has no
    /// policy sources ([`NO_POLICY`]), and the profile identity is drawn
    /// from the seed, so the header is a function of the run.
    ///
    /// # Errors
    /// None in practice: the header has no data versions to bound.
    pub fn recording_header(&self) -> Result<RecordingHeader, HeaderError> {
        let st = self.shared.lock();
        let log = DecisionLogRef {
            trace_hash: st.trace.hash(),
            entries: st.trace.entries(),
        };
        // A stream of its own, so drawing it changes nothing in the run.
        let mut rng = Xoshiro256StarStar::from_seed(st.config.seed ^ PROFILE_STREAM);
        drop(st);
        let mut profile = [0u8; 16];
        let (lo, hi) = profile.split_at_mut(8);
        lo.copy_from_slice(&rng.next_u64().to_le_bytes());
        hi.copy_from_slice(&rng.next_u64().to_le_bytes());
        RecordingHeader::recording(
            Common {
                build_id: hg_types::BUILD_ID,
                data_versions: Vec::new(),
                os_version: Text::new("simulated")?,
                process: ProcessKind::Simulation,
                sandbox: SandboxTier::Simulated,
            },
            log,
            NO_POLICY,
            ProfileIdentity(profile),
        )
    }
}

impl Drop for World {
    fn drop(&mut self) {
        // Tasks hold substrates, which hold the world: drop them here to
        // break the cycle, and release every live region.
        let mut st = self.shared.lock();
        let mut wreck = Wreckage::default();
        for p in &mut st.procs {
            for s in p.seqs.drain(..) {
                wreck.tasks.extend(s.queue.into_iter().map(|(_, t)| t));
            }
            for (base, r) in std::mem::take(&mut p.regions) {
                wreck.regions.push((
                    Arc::clone(&p.backing),
                    Region::new(base, r.host_len, r.kind),
                ));
            }
            p.alive = false;
        }
        for (_, item) in std::mem::take(&mut st.timers) {
            if let TimerItem::Task { task, .. } = item {
                wreck.tasks.push(task);
            }
        }
        st.ipc.drain(&mut wreck.ipc);
        drop(st);
        wreck.clear();
    }
}

pub(crate) fn priority_name(p: Priority) -> &'static str {
    match p {
        Priority::UserBlocking => "user-blocking",
        Priority::UserVisible => "user-visible",
        Priority::BestEffort => "best-effort",
        Priority::MayBlock => "may-block",
    }
}
