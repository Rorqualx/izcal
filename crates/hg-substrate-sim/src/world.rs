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
//! **Faults** come from streams of their own, so a fault-free run draws
//! nothing from them and its trace does not depend on the fault settings.
//! Each fault class added since phase C has its own stream ([`tag`]), so
//! turning one on, or adding a new class, never moves another class's
//! draws: a seed's trace with the new classes off is the trace it had
//! before they existed. Crashes, their restart delays and commit failures
//! share the phase C stream, as they always have.
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
    /// Chance, per million IPC sends, that the message is held back by up
    /// to [`FaultPlan::ipc_delay_max`] of virtual time (phase E rule
    /// E-R8). Later messages on the same channel wait behind it: a
    /// channel stays FIFO.
    pub ipc_delay_per_million: u32,
    /// The longest injected IPC delay.
    pub ipc_delay_max: Duration,
    /// Chance, per million deliveries, that a channel's next delivery
    /// steps back behind the work already queued on its receiver's
    /// sequence, so deliveries from other channels overtake it (E-R8:
    /// reordering across channels only; one channel's messages keep
    /// their order, the transport's contract).
    pub ipc_reorder_per_million: u32,
    /// Chance, per million IPC sends, that the message is lost and its
    /// channel closes: the receiver gets what was queued before it, then
    /// `Closed`; the sender's later sends fail with `Closed`. A lost
    /// message is never silent (E-R8).
    pub ipc_drop_per_million: u32,
    /// Chance, per million launches, that the Linux spawner dies with the
    /// launch in flight: the launch fails and the spawner is restarted
    /// for the next one (E-R2).
    pub spawner_death_per_million: u32,
    /// Chance, per million launches, that the OS refuses the launch (the
    /// spawner's `Failed`, a failed `posix_spawn` or `CreateProcess`).
    pub launch_fail_per_million: u32,
    /// Chance, per million launches, that the child starts only after
    /// [`FaultPlan::ready_stall`] of virtual time, so it misses the
    /// broker's `ready` deadline (E-R3).
    pub ready_stall_per_million: u32,
    /// How long a stalled child waits before its first task runs.
    pub ready_stall: Duration,
}

impl FaultPlan {
    /// No faults.
    pub const NONE: Self = Self {
        crash_per_million_steps: 0,
        commit_fail_per_million: 0,
        restart_delay_max: Duration::from_millis(10),
        ipc_delay_per_million: 0,
        ipc_delay_max: Duration::from_millis(50),
        ipc_reorder_per_million: 0,
        ipc_drop_per_million: 0,
        spawner_death_per_million: 0,
        launch_fail_per_million: 0,
        ready_stall_per_million: 0,
        ready_stall: Duration::from_secs(10),
    };
}

/// One seeded stream per fault class added in phase E (E-R8; design
/// review DST minor 1).
pub(crate) struct FaultStreams {
    pub(crate) ipc_delay: Xoshiro256StarStar,
    pub(crate) ipc_reorder: Xoshiro256StarStar,
    pub(crate) ipc_drop: Xoshiro256StarStar,
    pub(crate) spawner_death: Xoshiro256StarStar,
    pub(crate) launch_fail: Xoshiro256StarStar,
    pub(crate) ready_stall: Xoshiro256StarStar,
}

impl FaultStreams {
    fn new(seed: u64) -> Self {
        let s = |class: u64| Xoshiro256StarStar::from_seed(derive(&[seed, tag::FAULT, class]));
        Self {
            ipc_delay: s(fault_class::IPC_DELAY),
            ipc_reorder: s(fault_class::IPC_REORDER),
            ipc_drop: s(fault_class::IPC_DROP),
            spawner_death: s(fault_class::SPAWNER_DEATH),
            launch_fail: s(fault_class::LAUNCH_FAIL),
            ready_stall: s(fault_class::READY_STALL),
        }
    }
}

/// Whether a fault with `rate` per million fires, drawing from `rng` only
/// when the rate is not zero (so a class that is off draws nothing).
pub(crate) fn fires(rng: &mut Xoshiro256StarStar, rate: u32) -> bool {
    rate > 0 && below(rng, 1_000_000) < u64::from(rate)
}

/// The fault classes' sub-tags under [`tag::FAULT`]. A new class takes
/// the next number; none is ever reused.
pub(crate) mod fault_class {
    pub(crate) const IPC_DELAY: u64 = 1;
    pub(crate) const IPC_REORDER: u64 = 2;
    pub(crate) const IPC_DROP: u64 = 3;
    pub(crate) const SPAWNER_DEATH: u64 = 4;
    pub(crate) const LAUNCH_FAIL: u64 = 5;
    pub(crate) const READY_STALL: u64 = 6;
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
    /// Whose shared-memory behaviour the regions model (D6 review M7).
    pub region_os: crate::region::SimOs,
    /// Children the simulated launcher starts run in a modelled PID
    /// namespace (design review M7, M8): their bootstrap's parent is
    /// [`hg_substrate::ipc::PeerAttest::Inherited`], they attest no
    /// sender by pid (a receiver bound to a pid refuses every record, as
    /// the Linux restricted tier's pid 0 would), a region writer reaches
    /// them only on the inherited bootstrap channel, and every fault they
    /// report names no sender. On by default: the contract (M8) is that a
    /// child's identity check is always the broker-issued channel binding,
    /// so code written to it passes on every OS. Processes linked with
    /// [`World::link`] are not affected.
    pub pid_namespaces: bool,
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
            region_os: crate::region::SimOs::Linux,
            pid_namespaces: true,
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
    /// Launched by another process ([`World::link`]): never the broker.
    pub(crate) launched: bool,
    /// Started by the simulated launcher in a modelled PID namespace
    /// ([`SimConfig::pid_namespaces`]).
    pub(crate) pid_ns: bool,
    /// The kinds this process may launch: the broker's only.
    pub(crate) children: Option<Arc<crate::launch::ChildTable>>,
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
    /// A channel whose next message was held back is due (E-R8 delay).
    IpcWake(u64),
}

pub(crate) struct State {
    pub(crate) config: SimConfig,
    pub(crate) now: Instant,
    steps: u64,
    crashes: u64,
    pub(crate) sched: Xoshiro256StarStar,
    pub(crate) fault: Xoshiro256StarStar,
    /// The phase E fault classes' streams.
    pub(crate) faults: FaultStreams,
    pub(crate) procs: Vec<Proc>,
    /// Pending timers by (firing time in ns, post order).
    timers: BTreeMap<(u64, u64), TimerItem>,
    next_timer: u64,
    pub(crate) trace: Trace,
    /// Every IPC channel.
    pub(crate) ipc: crate::ipc::Channels,
    /// Every shared-memory region.
    pub(crate) regions: crate::region::Regions,
    /// The world itself, for tasks the state makes (IPC deliveries).
    pub(crate) ipc_self: Weak<Shared>,
    /// The scripted OS of each process that has one (the broker).
    pub(crate) os: BTreeMap<ProcessId, crate::os::OsModel>,
    /// The install tree every process shares.
    pub(crate) install: crate::install::InstallTree,
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
    pub(crate) const REGION: u64 = 5;
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

impl State {
    /// Wakes channel `id` at virtual time `at_ns` (an injected delay ran
    /// out). No draw: the time is the delay's own.
    pub(crate) fn add_ipc_wake(&mut self, id: u64, at_ns: u64) {
        let key = self.next_timer;
        self.next_timer = self.next_timer.saturating_add(1);
        self.timers.insert((at_ns, key), TimerItem::IpcWake(id));
    }

    /// Queues `task` on `(pid, generation)`'s sequence `seq` at virtual
    /// time `at_ns`, with no draw (a stalled child's first task).
    fn add_start_timer(&mut self, slot: Slot, task: Task, at_ns: u64) {
        let key = self.next_timer;
        self.next_timer = self.next_timer.saturating_add(1);
        if let Some(s) = self
            .proc_mut(slot.pid, slot.generation)
            .and_then(|p| p.seqs.get_mut(slot.seq))
        {
            s.delayed = s.delayed.saturating_add(1);
        }
        self.timers.insert(
            (at_ns, key),
            TimerItem::Task {
                pid: slot.pid,
                generation: slot.generation,
                seq: slot.seq,
                number: slot.number,
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
    pub(crate) shared: Arc<Shared>,
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
            faults: FaultStreams::new(seed),
            procs: Vec::new(),
            timers: BTreeMap::new(),
            next_timer: 0,
            trace: Trace::new(config.keep_trace_text),
            ipc: crate::ipc::Channels::default(),
            regions: crate::region::Regions::new(Xoshiro256StarStar::from_seed(derive(&[
                seed,
                tag::REGION,
            ]))),
            ipc_self: Weak::new(),
            os: BTreeMap::new(),
            install: crate::install::InstallTree::default(),
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
        self.spawn_inner(name, backing, restart, Arc::new(entry), None, None)
    }

    /// As [`World::spawn`], for a browser that launches children: its
    /// substrate's `launcher` starts a process of each kind in `children`
    /// (M1 phase E). Launched children are not restarted by the world;
    /// relaunching after a crash is the browser's job.
    pub fn spawn_launcher(
        &self,
        name: &'static str,
        backing: Arc<dyn Memory>,
        restart: bool,
        entry: impl Fn(Substrate) + Send + Sync + 'static,
        children: crate::launch::ChildTable,
    ) -> ProcessId {
        self.spawn_inner(
            name,
            backing,
            restart,
            Arc::new(entry),
            None,
            Some(Arc::new(children)),
        )
    }

    /// As [`World::spawn`], for the broker: its substrate's `os` is the
    /// scripted OS `script` (OS integration 6.1). The script survives
    /// restarts; inputs not yet delivered wait for the next listener.
    pub fn spawn_broker(
        &self,
        name: &'static str,
        backing: Arc<dyn Memory>,
        restart: bool,
        entry: impl Fn(Substrate) + Send + Sync + 'static,
        script: crate::os::OsScript,
    ) -> ProcessId {
        self.spawn_inner(name, backing, restart, Arc::new(entry), Some(script), None)
    }

    fn spawn_inner(
        &self,
        name: &'static str,
        backing: Arc<dyn Memory>,
        restart: bool,
        entry: Entry,
        script: Option<crate::os::OsScript>,
        children: Option<Arc<crate::launch::ChildTable>>,
    ) -> ProcessId {
        spawn_in(
            &self.shared,
            name,
            backing,
            entry,
            SpawnOptions {
                restart,
                script,
                children,
                stall: None,
            },
        )
    }

    /// Brings incarnation `generation` of `pid` up and queues its entry.
    fn start(&self, st: &mut State, pid: ProcessId, generation: u32) {
        start_in(&self.shared, st, pid, generation, None);
    }
}

/// [`World::crash`]'s work, on the shared state.
pub(crate) fn crash_in(shared: &Arc<Shared>, pid: ProcessId) -> bool {
    let mut st = shared.lock();
    let wreck = World::crash_locked(&mut st, pid);
    drop(st);
    let crashed = wreck.is_some();
    if let Some(w) = wreck {
        w.clear();
    }
    crashed
}

/// Crashes `pid` only if it is still incarnation `generation`: a kill
/// aimed at an earlier incarnation never reaches a later one.
pub(crate) fn crash_in_generation(shared: &Arc<Shared>, pid: ProcessId, generation: u64) -> bool {
    let mut st = shared.lock();
    let current = st
        .procs
        .get(usize::try_from(pid.0).unwrap_or(usize::MAX))
        .is_some_and(|p| p.alive && u64::from(p.generation) == generation);
    if !current {
        return false;
    }
    let wreck = World::crash_locked(&mut st, pid);
    drop(st);
    let crashed = wreck.is_some();
    if let Some(w) = wreck {
        w.clear();
    }
    crashed
}

/// [`World::os_peer`]'s work, on the shared state.
pub(crate) fn os_peer_in(
    shared: &Arc<Shared>,
    pid: ProcessId,
) -> Option<hg_substrate::ipc::OsPeer> {
    let st = shared.lock();
    let p = st.procs.get(usize::try_from(pid.0).ok()?)?;
    p.alive
        .then(|| hg_substrate::ipc::OsPeer::new(pid.0, u64::from(p.generation)))
}

/// How [`spawn_in`] starts a process.
#[derive(Default)]
pub(crate) struct SpawnOptions {
    /// Restart it after a crash.
    pub(crate) restart: bool,
    /// Its scripted OS (the broker's).
    pub(crate) script: Option<crate::os::OsScript>,
    /// What it may launch (the broker's).
    pub(crate) children: Option<Arc<crate::launch::ChildTable>>,
    /// Hold its first task back this long (a launch fault).
    pub(crate) stall: Option<Duration>,
}

/// [`World::spawn`]'s work, on the shared state: what the simulated
/// launcher calls from inside a running task (a temporary [`World`] would
/// tear the run down when dropped).
pub(crate) fn spawn_in(
    shared: &Arc<Shared>,
    name: &'static str,
    backing: Arc<dyn Memory>,
    entry: Entry,
    opts: SpawnOptions,
) -> ProcessId {
    let SpawnOptions {
        restart,
        script,
        children,
        stall,
    } = opts;
    let mut st = shared.lock();
    let pid = ProcessId(u32::try_from(st.procs.len()).unwrap_or(u32::MAX));
    if let Some(script) = script {
        st.os.insert(pid, crate::os::OsModel::new(script));
    }
    st.procs.push(Proc {
        name,
        generation: 0,
        alive: false,
        entry,
        restart,
        backing,
        seqs: Vec::new(),
        regions: BTreeMap::new(),
        launched: false,
        pid_ns: false,
        reserved: 0,
        committed: 0,
        children,
    });
    start_in(shared, &mut st, pid, 0, stall);
    pid
}

/// Brings incarnation `generation` of `pid` up and queues its entry, or,
/// with `stall`, holds the entry back that long (a child that misses its
/// `ready` deadline).
fn start_in(
    shared: &Arc<Shared>,
    st: &mut State,
    pid: ProcessId,
    generation: u32,
    stall: Option<Duration>,
) {
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
    let has_os = st.os.contains_key(&pid);
    let children = p.children.clone();
    let substrate = crate::substrate_for(
        shared,
        pid,
        generation,
        derive(&[seed, tag::PROCESS, u64::from(pid.0), u64::from(generation)]),
        has_os,
        children,
    );
    let entry = Arc::clone(&p.entry);
    let name = p.name;
    let task = Task::new(move || entry(substrate));
    if let Some(stall) = stall {
        let at = st
            .now
            .checked_add(stall)
            .unwrap_or(Instant::from_nanos(u64::MAX))
            .as_nanos();
        st.add_start_timer(
            Slot {
                pid,
                generation,
                seq: 0,
                number: 0,
            },
            task,
            at,
        );
        st.trace
            .record(format_args!("start {name}.{generation} stalled until {at}"));
    } else {
        if let Some(main) = p.seqs.first_mut() {
            main.queue.push_back((0, task));
        }
        st.trace.record(format_args!("start {name}.{generation}"));
    }
}

impl World {
    /// Crashes `pid` now, as an injected fault would. Returns false if it
    /// was not running.
    pub fn crash(&self, pid: ProcessId) -> bool {
        crash_in(&self.shared, pid)
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
                TimerItem::IpcWake(id) => crate::ipc::wake(&mut st, id),
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

    /// Makes `pid` a compromised process that maps regions with the OS
    /// directly, behind its backend's back: on macOS and Windows its own
    /// seal no longer sees those mappings and succeeds while one is live;
    /// the Linux kernel still sees them (fact L1). The adversarial-filler
    /// fault of the D6 review (M7).
    pub fn hide_mappings(&self, pid: ProcessId) {
        let mut st = self.shared.lock();
        crate::region::hide_mappings(&mut st, pid);
    }

    /// Virtual time now.
    pub fn now(&self) -> Instant {
        self.shared.lock().now
    }

    /// `pid`'s current incarnation as its OS identity, the value a
    /// bound receiver is given ([`hg_substrate::ipc::Ipc::listen_bound`]),
    /// or `None` if no such process is running.
    pub fn os_peer(&self, pid: ProcessId) -> Option<hg_substrate::ipc::OsPeer> {
        os_peer_in(&self.shared, pid)
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

    /// The world's install device: one in-memory install tree that every
    /// simulated process shares (distribution 13 item 3), so a scenario
    /// can replace a child binary or a build between two launches.
    pub fn install_device(&self) -> Arc<dyn hg_substrate::InstallDevice> {
        Arc::new(crate::install::SimInstallDevice {
            shared: Arc::clone(&self.shared),
        })
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
