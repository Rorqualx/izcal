//! The OS executor behind the substrate's `Executor` trait (Stage 1
//! design 3.1; CS-AS-3, CS-AS-4, CS-AS-6, CS-AS-7, CS-AS-12; footprint
//! 6.3, 6.8, FP-32).
//!
//! - **Sequences.** Each sequence is a FIFO of tasks; at most one of its
//!   tasks runs at a time, so its tasks run in order. A sequence waiting
//!   to run sits in its priority's ready list.
//! - **General workers** run `UserBlocking`, then `UserVisible`, then
//!   `BestEffort` sequences. They start with the executor and live as
//!   long as it. With none, the process's main thread is the general
//!   worker ([`OsExecutor::run_here`]): a launched child runs its
//!   sequences on `main` instead of parking it (FP 6.3, FP-32: an idle
//!   empty site process has two threads, the one that runs its
//!   sequences and the IPC thread).
//! - **On-demand threads.** `MayBlock` workers (a separate pool, so a
//!   blocking task never holds up the others, CS-AS-7) and the timer
//!   thread start when their first task is posted, and each ends after
//!   [`IDLE_EXIT`] with nothing to do, by a one-shot timed wait armed
//!   when it goes idle (footprint 6.3's worker rule). A `MayBlock`
//!   worker stays while a delayed `MayBlock` task is pending, and the
//!   timer thread while any timer is, so a due task always finds its
//!   thread.
//! - **No wakeups at rest.** An idle general worker waits with no
//!   timeout; an on-demand thread wakes once, [`IDLE_EXIT`] after its
//!   last work, to end. An executor at rest makes no wakeups (footprint
//!   6.8, FP-18).
//! - **Timers.** A delayed post is a one-shot timer. The timer thread
//!   sleeps until the earliest *latest-allowed* time (deadline plus
//!   tolerance), then moves every task whose deadline has passed onto
//!   its sequence. That fires each task no earlier than its delay and no
//!   later than its band, and batches timers that can share a wakeup.
//! - **Capacity.** A sequence holds at most [`DEFAULT_CAPACITY`] pending
//!   tasks, queued and delayed together (CS-AS-6 (c)); past it, `post`
//!   returns `QueueFull`. A post whose thread cannot be started, with
//!   none of its kind alive, returns `NoThread` and queues nothing.
//! - **Phases.** `fork_join` runs an index-addressed loop on scoped
//!   threads, one per pool worker, with indices dealt round-robin
//!   (CS-AS-12). They exist only while the phase runs, so a process
//!   between phases holds none.
//!
//! The executor owns every thread it uses (CS-AS-3), and
//! [`OsExecutor::thread_count`] says how many run now. Dropping it stops
//! the threads once their current task finishes; pending tasks are
//! dropped unrun.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration as StdDuration, Instant as StdInstant};

use hg_substrate::{Executor, PhasePool, PostError, Priority, SequenceHandle, Task};
use hg_types::Duration;

/// Pending tasks a sequence may hold (CS-AS-6 (c)): 1024†.
pub const DEFAULT_CAPACITY: usize = 1024;

/// How long an on-demand thread waits idle before it ends: 10 s†
/// (footprint 6.3: a pool worker "exits after 10 s† idle, by a one-shot
/// timer armed when it goes idle").
pub const IDLE_EXIT: StdDuration = StdDuration::from_secs(10);

/// The `MayBlock` class's index.
const MAY_BLOCK: usize = 3;

/// Which pool a priority runs in, and its rank inside the general pool.
const fn class(p: Priority) -> usize {
    match p {
        Priority::UserBlocking => 0,
        Priority::UserVisible => 1,
        Priority::BestEffort => 2,
        Priority::MayBlock => MAY_BLOCK,
    }
}

struct Sequence {
    name: &'static str,
    class: usize,
    queue: VecDeque<Task>,
    /// Delayed tasks not yet due.
    delayed: usize,
    /// A worker is running one of its tasks.
    running: bool,
    /// It is in a ready list.
    listed: bool,
}

struct Timer {
    deadline: StdInstant,
    seq: u32,
    task: Task,
}

struct State {
    sequences: Vec<Sequence>,
    /// Ready sequence indices: classes 0..=2 (general), 3 (may block).
    ready: [VecDeque<u32>; 4],
    /// Timers by latest-allowed time; the u64 keeps equal times in post order.
    timers: BinaryHeap<Reverse<(StdInstant, u64)>>,
    /// Pending timers by post order.
    timer_tasks: BTreeMap<u64, Timer>,
    next_timer: u64,
    /// Delayed `MayBlock` tasks not yet due: a `MayBlock` worker stays
    /// while there are any.
    delayed_may_block: usize,
    stopping: bool,
    /// General worker threads (they live as long as the executor).
    general: usize,
    /// Live `MayBlock` workers, and how many of them wait for work.
    may_block_live: usize,
    may_block_idle: usize,
    /// The most `MayBlock` workers at once.
    may_block_max: usize,
    /// `MayBlock` workers ever started, to name them.
    may_block_started: usize,
    /// The timer thread runs.
    timer_live: bool,
}

struct Shared {
    state: Mutex<State>,
    general: Condvar,
    may_block: Condvar,
    timer: Condvar,
    capacity: usize,
    /// [`IDLE_EXIT`]; shorter in tests.
    idle_exit: StdDuration,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        // Every panic aborts the process (CS-ER-2), so a poisoned lock
        // cannot hide a half-finished update in a running process.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Lists sequence `i` as ready if it has work and is not running.
    fn make_ready(&self, st: &mut State, i: u32) {
        let Some(seq) = st
            .sequences
            .get_mut(usize::try_from(i).unwrap_or(usize::MAX))
        else {
            return;
        };
        if seq.running || seq.listed || seq.queue.is_empty() {
            return;
        }
        seq.listed = true;
        let class = seq.class;
        if let Some(list) = st.ready.get_mut(class) {
            list.push_back(i);
        }
        if class == MAY_BLOCK {
            self.may_block.notify_one();
        } else {
            self.general.notify_one();
        }
    }

    /// Takes sequence `i`'s next task and marks it running.
    fn take(st: &mut State, i: u32) -> Option<Task> {
        st.sequences
            .get_mut(usize::try_from(i).unwrap_or(usize::MAX))
            .and_then(|seq| {
                seq.listed = false;
                let t = seq.queue.pop_front();
                seq.running = t.is_some();
                t
            })
    }

    /// Sequence `i`'s task has returned: list it again if it has more.
    fn finish(&self, st: &mut State, i: u32) {
        if let Some(seq) = st
            .sequences
            .get_mut(usize::try_from(i).unwrap_or(usize::MAX))
        {
            seq.running = false;
        }
        self.make_ready(st, i);
    }
}

/// The OS executor.
pub struct OsExecutor {
    shared: Arc<Shared>,
    /// Every thread started; finished ones are dropped at the next start.
    threads: Mutex<Vec<JoinHandle<()>>>,
    pool: OsPhasePool,
}

impl OsExecutor {
    /// An executor with `general` workers for the three ordinary
    /// priorities, started now, and up to `may_block` (at least 1)
    /// workers for `MayBlock` sequences, each started when needed. The
    /// timer thread starts with the first delayed post.
    ///
    /// With `general` 0 no general worker is started: the caller's
    /// thread must run them with [`Self::run_here`] (a launched child's
    /// `main`), and fork/join phases get footprint 6.3's pool size.
    ///
    /// # Errors
    /// The OS error if a thread cannot be started.
    pub fn new(general: usize, may_block: usize) -> std::io::Result<Self> {
        Self::with_idle_exit(general, may_block, IDLE_EXIT)
    }

    fn with_idle_exit(
        general: usize,
        may_block: usize,
        idle_exit: StdDuration,
    ) -> std::io::Result<Self> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                sequences: Vec::new(),
                ready: Default::default(),
                timers: BinaryHeap::new(),
                timer_tasks: BTreeMap::new(),
                next_timer: 0,
                delayed_may_block: 0,
                stopping: false,
                general,
                may_block_live: 0,
                may_block_idle: 0,
                may_block_max: may_block.max(1),
                may_block_started: 0,
                timer_live: false,
            }),
            general: Condvar::new(),
            may_block: Condvar::new(),
            timer: Condvar::new(),
            capacity: DEFAULT_CAPACITY,
            idle_exit,
        });
        let mut threads = Vec::new();
        for n in 0..general {
            let s = Arc::clone(&shared);
            threads.push(spawn(format!("hg-worker-{n}"), move || {
                general_worker(&s, None);
            })?);
        }
        let workers = if general == 0 {
            phase_pool_size()
        } else {
            general
        };
        Ok(Self {
            shared,
            threads: Mutex::new(threads),
            pool: OsPhasePool { workers },
        })
    }

    /// The number of threads this executor runs now: its general
    /// workers, live `MayBlock` workers and the timer thread if it runs
    /// (CS-AS-3's check: the process's thread count is the substrate's).
    /// A `main` running [`Self::run_here`] is the process's own and not
    /// counted.
    pub fn thread_count(&self) -> usize {
        let st = self.shared.lock();
        st.general + st.may_block_live + usize::from(st.timer_live)
    }

    /// Runs general sequences on the calling thread, as one more general
    /// worker, until `done` holds or the executor stops. `done` is
    /// checked first and after each task this thread runs, so it must
    /// become true inside such a task, as a served endpoint's end does
    /// (the transport delivers it in the endpoint's delivery task). With
    /// no general worker ([`Self::new`] with 0), every general task runs
    /// here.
    pub fn run_here(&self, done: &dyn Fn() -> bool) {
        if done() {
            return;
        }
        general_worker(&self.shared, Some(done));
    }

    /// Starts a thread and keeps its handle, dropping those that ended.
    fn start(&self, name: String, f: impl FnOnce() + Send + 'static) -> std::io::Result<()> {
        let t = spawn(name, f)?;
        let mut threads = self.threads.lock().unwrap_or_else(PoisonError::into_inner);
        threads.retain(|h| !h.is_finished());
        threads.push(t);
        Ok(())
    }

    /// Makes sure a `MayBlock` task about to be queued will find a
    /// worker: starts one if none is live, or if none waits and the pool
    /// is not full. Called with the state lock held; the new worker takes
    /// it once the caller lets go.
    ///
    /// # Errors
    /// [`PostError::NoThread`] if none could be started and none lives.
    fn want_may_block(&self, st: &mut State) -> Result<(), PostError> {
        let need = st.may_block_live == 0
            || (st.may_block_idle == 0 && st.may_block_live < st.may_block_max);
        if !need {
            return Ok(());
        }
        let n = st.may_block_started;
        let s = Arc::clone(&self.shared);
        match self.start(format!("hg-may-block-{n}"), move || {
            may_block_worker(&s);
        }) {
            Ok(()) => {
                st.may_block_started = n.saturating_add(1);
                st.may_block_live = st.may_block_live.saturating_add(1);
                Ok(())
            }
            Err(_e) if st.may_block_live > 0 => Ok(()),
            Err(_e) => Err(PostError::NoThread),
        }
    }

    /// Makes sure the timer thread runs before a timer is queued.
    ///
    /// # Errors
    /// [`PostError::NoThread`] if it is not running and cannot start.
    fn want_timers(&self, st: &mut State) -> Result<(), PostError> {
        if st.timer_live {
            return Ok(());
        }
        let s = Arc::clone(&self.shared);
        self.start("hg-timers".into(), move || timers(&s))
            .map_err(|_e| PostError::NoThread)?;
        st.timer_live = true;
        Ok(())
    }
}

/// Footprint 6.3's pool size: at most min(cores − 1, 4)†, at least 1.
fn phase_pool_size() -> usize {
    let cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    cores.saturating_sub(1).clamp(1, 4)
}

fn spawn(name: String, f: impl FnOnce() + Send + 'static) -> std::io::Result<JoinHandle<()>> {
    std::thread::Builder::new().name(name).spawn(f)
}

impl Drop for OsExecutor {
    fn drop(&mut self) {
        {
            let mut st = self.shared.lock();
            st.stopping = true;
        }
        self.shared.general.notify_all();
        self.shared.may_block.notify_all();
        self.shared.timer.notify_all();
        let threads =
            std::mem::take(&mut *self.threads.lock().unwrap_or_else(PoisonError::into_inner));
        // A task that drops the last handle runs this on a worker; that
        // worker exits on its own once the task returns.
        let me = std::thread::current().id();
        for t in threads {
            if t.thread().id() != me {
                let _ = t.join();
            }
        }
    }
}

/// A general worker: a started one (`done` None) or the caller's thread
/// in [`OsExecutor::run_here`].
fn general_worker(shared: &Shared, done: Option<&dyn Fn() -> bool>) {
    let mut st = shared.lock();
    loop {
        if st.stopping {
            return;
        }
        let Some(i) = st.ready.iter_mut().take(3).find_map(VecDeque::pop_front) else {
            // No timeout: an idle worker makes no wakeups.
            st = shared
                .general
                .wait(st)
                .unwrap_or_else(PoisonError::into_inner);
            continue;
        };
        let Some(task) = Shared::take(&mut st, i) else {
            continue;
        };
        drop(st);
        task.run();
        let finished = done.is_some_and(|d| d());
        st = shared.lock();
        shared.finish(&mut st, i);
        if finished {
            return;
        }
    }
}

/// A `MayBlock` worker: ends after [`IDLE_EXIT`] with nothing to run and
/// no delayed `MayBlock` task pending.
fn may_block_worker(shared: &Shared) {
    let mut st = shared.lock();
    let mut idle_since = StdInstant::now();
    loop {
        if st.stopping {
            st.may_block_live = st.may_block_live.saturating_sub(1);
            return;
        }
        if let Some(i) = st.ready.get_mut(MAY_BLOCK).and_then(VecDeque::pop_front) {
            let Some(task) = Shared::take(&mut st, i) else {
                continue;
            };
            drop(st);
            task.run();
            st = shared.lock();
            shared.finish(&mut st, i);
            idle_since = StdInstant::now();
            continue;
        }
        let idle = idle_since.elapsed();
        if st.delayed_may_block == 0 && idle >= shared.idle_exit {
            st.may_block_live = st.may_block_live.saturating_sub(1);
            return;
        }
        st.may_block_idle = st.may_block_idle.saturating_add(1);
        st = if st.delayed_may_block == 0 {
            // The one-shot idle timer: one wakeup, IDLE_EXIT after the
            // last task.
            shared
                .may_block
                .wait_timeout(st, shared.idle_exit.saturating_sub(idle))
                .unwrap_or_else(PoisonError::into_inner)
                .0
        } else {
            shared
                .may_block
                .wait(st)
                .unwrap_or_else(PoisonError::into_inner)
        };
        st.may_block_idle = st.may_block_idle.saturating_sub(1);
    }
}

/// The timer thread: ends after [`IDLE_EXIT`] with no timer pending.
fn timers(shared: &Shared) {
    let mut st = shared.lock();
    let mut idle_since = StdInstant::now();
    loop {
        if st.stopping {
            st.timer_live = false;
            return;
        }
        let now = StdInstant::now();
        // Fire every timer whose deadline has passed, in deadline order
        // (post order between equal deadlines), so a late or batched
        // wakeup still runs a sequence's timers in the order they fall due.
        let mut due: Vec<(StdInstant, u64)> = st
            .timer_tasks
            .iter()
            .filter(|(_, t)| t.deadline <= now)
            .map(|(&id, t)| (t.deadline, id))
            .collect();
        due.sort_unstable();
        for (_, id) in due {
            if let Some(t) = st.timer_tasks.remove(&id) {
                if let Some(seq) = st
                    .sequences
                    .get_mut(usize::try_from(t.seq).unwrap_or(usize::MAX))
                {
                    seq.delayed = seq.delayed.saturating_sub(1);
                    let may_block = seq.class == MAY_BLOCK;
                    seq.queue.push_back(t.task);
                    if may_block {
                        st.delayed_may_block = st.delayed_may_block.saturating_sub(1);
                    }
                }
                shared.make_ready(&mut st, t.seq);
            }
        }
        // Drop heap entries for fired timers; wake at the earliest
        // latest-allowed time of the rest.
        while let Some(Reverse((_, id))) = st.timers.peek() {
            if st.timer_tasks.contains_key(id) {
                break;
            }
            st.timers.pop();
        }
        st = match st.timers.peek().map(|Reverse((latest, _))| *latest) {
            None => {
                let idle = idle_since.elapsed();
                if idle >= shared.idle_exit {
                    st.timer_live = false;
                    return;
                }
                // The one-shot idle timer, as for a MayBlock worker.
                shared
                    .timer
                    .wait_timeout(st, shared.idle_exit.saturating_sub(idle))
                    .unwrap_or_else(PoisonError::into_inner)
                    .0
            }
            Some(latest) => {
                idle_since = now;
                let wait = latest.saturating_duration_since(now);
                shared
                    .timer
                    .wait_timeout(st, wait)
                    .unwrap_or_else(PoisonError::into_inner)
                    .0
            }
        };
    }
}

impl Executor for OsExecutor {
    fn sequence(&self, name: &'static str, priority: Priority) -> SequenceHandle {
        let mut st = self.shared.lock();
        let index = u32::try_from(st.sequences.len()).unwrap_or(u32::MAX);
        st.sequences.push(Sequence {
            name,
            class: class(priority),
            queue: VecDeque::new(),
            delayed: 0,
            running: false,
            listed: false,
        });
        SequenceHandle::from_raw(index)
    }

    fn post(
        &self,
        seq: SequenceHandle,
        task: Task,
        delay: Option<Duration>,
        tolerance: Duration,
    ) -> Result<(), PostError> {
        let mut st = self.shared.lock();
        if st.stopping {
            return Err(PostError::ShuttingDown);
        }
        let i = seq.raw();
        let Some(s) = st.sequences.get(usize::try_from(i).unwrap_or(usize::MAX)) else {
            return Err(PostError::UnknownSequence);
        };
        if s.queue.len().saturating_add(s.delayed) >= self.shared.capacity {
            return Err(PostError::QueueFull);
        }
        let may_block = s.class == MAY_BLOCK;
        // The threads this task needs, before anything is queued.
        if may_block {
            self.want_may_block(&mut st)?;
        }
        if delay.is_some() {
            self.want_timers(&mut st)?;
        }
        let Some(s) = st
            .sequences
            .get_mut(usize::try_from(i).unwrap_or(usize::MAX))
        else {
            return Err(PostError::UnknownSequence);
        };
        match delay {
            None => {
                s.queue.push_back(task);
                self.shared.make_ready(&mut st, i);
            }
            Some(d) => {
                s.delayed = s.delayed.saturating_add(1);
                if may_block {
                    st.delayed_may_block = st.delayed_may_block.saturating_add(1);
                }
                let now = StdInstant::now();
                let deadline = now.checked_add(d).unwrap_or(now);
                let latest = deadline.checked_add(tolerance).unwrap_or(deadline);
                let id = st.next_timer;
                st.next_timer = st.next_timer.saturating_add(1);
                st.timers.push(Reverse((latest, id)));
                st.timer_tasks.insert(
                    id,
                    Timer {
                        deadline,
                        seq: i,
                        task,
                    },
                );
                self.shared.timer.notify_one();
            }
        }
        Ok(())
    }

    fn fork_join(&self) -> &dyn PhasePool {
        &self.pool
    }
}

impl OsExecutor {
    /// Whether `seq` names one of this executor's sequences: the IPC
    /// transport's check before it listens on one ([`IpcError::Sequence`]).
    ///
    /// [`IpcError::Sequence`]: hg_substrate::ipc::IpcError::Sequence
    #[cfg_attr(
        not(any(target_os = "linux", target_os = "macos", windows, test)),
        expect(
            dead_code,
            reason = "used by the Linux, macOS and Windows IPC transports"
        )
    )]
    pub(crate) fn has_sequence(&self, seq: SequenceHandle) -> bool {
        let st = self.shared.lock();
        usize::try_from(seq.raw()).is_ok_and(|i| i < st.sequences.len())
    }

    /// Posts an IPC delivery task, now, past the sequence's capacity if
    /// need be. The transport keeps at most one delivery task per
    /// listening channel on a sequence (the drain-task model), so the
    /// excess is bounded by the number of listeners, and a delivery is
    /// never lost because the receiver's sequence is busy: a sender in
    /// another process cannot be told "full" for that (CS-AS-6 (b)).
    ///
    /// # Errors
    /// [`PostError::ShuttingDown`], [`PostError::UnknownSequence`], or
    /// [`PostError::NoThread`] for a `MayBlock` sequence whose worker
    /// cannot start (the transport posts again with its next item).
    #[cfg_attr(
        not(any(target_os = "linux", target_os = "macos", windows, test)),
        expect(
            dead_code,
            reason = "used by the Linux, macOS and Windows IPC transports"
        )
    )]
    pub(crate) fn post_reserved(&self, seq: SequenceHandle, task: Task) -> Result<(), PostError> {
        let mut st = self.shared.lock();
        if st.stopping {
            return Err(PostError::ShuttingDown);
        }
        let i = seq.raw();
        let Some(s) = st.sequences.get(usize::try_from(i).unwrap_or(usize::MAX)) else {
            return Err(PostError::UnknownSequence);
        };
        if s.class == MAY_BLOCK {
            self.want_may_block(&mut st)?;
        }
        if let Some(s) = st
            .sequences
            .get_mut(usize::try_from(i).unwrap_or(usize::MAX))
        {
            s.queue.push_back(task);
        }
        self.shared.make_ready(&mut st, i);
        Ok(())
    }
}

impl std::fmt::Debug for OsExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let st = self.shared.lock();
        let names: Vec<&str> = st.sequences.iter().map(|s| s.name).collect();
        f.debug_struct("OsExecutor")
            .field("sequences", &names)
            .finish()
    }
}

/// The fork/join pool: scoped threads, indices dealt round-robin.
#[derive(Debug)]
struct OsPhasePool {
    workers: usize,
}

impl PhasePool for OsPhasePool {
    fn for_each_index(&self, n: usize, f: &(dyn Fn(usize) + Sync)) {
        let k = self.workers.min(n);
        if k <= 1 {
            (0..n).for_each(f);
            return;
        }
        std::thread::scope(|scope| {
            for t in 0..k {
                scope.spawn(move || (t..n).step_by(k).for_each(f));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::sync_channel;
    use std::time::Duration as StdDuration;

    fn exec() -> OsExecutor {
        OsExecutor::new(4, 1).unwrap()
    }

    #[test]
    fn a_sequence_runs_its_tasks_in_order_one_at_a_time() {
        let e = exec();
        let seq = e.sequence("order", Priority::UserVisible);
        let log = Arc::new(Mutex::new(Vec::new()));
        let active = Arc::new(AtomicUsize::new(0));
        let (tx, rx) = sync_channel(1);
        for n in 0..200 {
            let (log, active) = (Arc::clone(&log), Arc::clone(&active));
            let tx = tx.clone();
            e.post(
                seq,
                Task::new(move || {
                    // ordering: SeqCst — the test checks mutual exclusion.
                    assert_eq!(active.fetch_add(1, Ordering::SeqCst), 0);
                    log.lock().unwrap().push(n);
                    active.fetch_sub(1, Ordering::SeqCst);
                    if n == 199 {
                        tx.send(()).unwrap();
                    }
                }),
                None,
                Duration::ZERO,
            )
            .unwrap();
        }
        rx.recv().unwrap();
        assert_eq!(*log.lock().unwrap(), (0..200).collect::<Vec<_>>());
    }

    #[test]
    fn different_sequences_run_in_parallel() {
        let e = exec();
        let a = e.sequence("a", Priority::UserVisible);
        let b = e.sequence("b", Priority::UserVisible);
        // Each task waits for the other: only parallel execution finishes.
        let (to_b, from_a) = sync_channel(1);
        let (to_a, from_b) = sync_channel(1);
        let (done, finished) = sync_channel(2);
        let d1 = done.clone();
        e.post(
            a,
            Task::new(move || {
                to_b.send(()).unwrap();
                from_b.recv().unwrap();
                d1.send(()).unwrap();
            }),
            None,
            Duration::ZERO,
        )
        .unwrap();
        e.post(
            b,
            Task::new(move || {
                to_a.send(()).unwrap();
                from_a.recv().unwrap();
                done.send(()).unwrap();
            }),
            None,
            Duration::ZERO,
        )
        .unwrap();
        finished.recv().unwrap();
        finished.recv().unwrap();
    }

    #[test]
    fn a_delayed_task_runs_no_earlier_than_its_delay() {
        let e = exec();
        let seq = e.sequence("timer", Priority::UserVisible);
        let (tx, rx) = sync_channel(1);
        let start = StdInstant::now();
        e.post(
            seq,
            Task::new(move || tx.send(StdInstant::now()).unwrap()),
            Some(Duration::from_millis(50)),
            Duration::from_millis(10),
        )
        .unwrap();
        let ran = rx.recv().unwrap();
        let waited = ran.duration_since(start);
        assert!(
            waited >= StdDuration::from_millis(50),
            "ran after {waited:?}"
        );
        assert!(
            waited < StdDuration::from_millis(500),
            "ran after {waited:?}"
        );
    }

    #[test]
    fn timers_fire_in_deadline_order() {
        let e = exec();
        let seq = e.sequence("timers", Priority::UserVisible);
        let log = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = sync_channel(1);
        for (n, ms) in [(2u32, 60u64), (0, 20), (1, 40)] {
            let log = Arc::clone(&log);
            let tx = tx.clone();
            e.post(
                seq,
                Task::new(move || {
                    let mut l = log.lock().unwrap();
                    l.push(n);
                    if l.len() == 3 {
                        tx.send(()).unwrap();
                    }
                }),
                Some(Duration::from_millis(ms)),
                Duration::ZERO,
            )
            .unwrap();
        }
        rx.recv().unwrap();
        assert_eq!(*log.lock().unwrap(), [0, 1, 2]);
    }

    #[test]
    fn batched_timers_still_run_in_deadline_order() {
        let e = exec();
        let seq = e.sequence("batched", Priority::UserVisible);
        let log = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = sync_channel(1);
        // A wide tolerance lets one wakeup fire all three; the later
        // deadline is posted first.
        for (n, ms) in [(2u32, 30u64), (1, 20), (0, 10)] {
            let log = Arc::clone(&log);
            let tx = tx.clone();
            e.post(
                seq,
                Task::new(move || {
                    let mut l = log.lock().unwrap();
                    l.push(n);
                    if l.len() == 3 {
                        tx.send(()).unwrap();
                    }
                }),
                Some(Duration::from_millis(ms)),
                Duration::from_millis(200),
            )
            .unwrap();
        }
        rx.recv().unwrap();
        assert_eq!(*log.lock().unwrap(), [0, 1, 2]);
    }

    #[test]
    fn a_full_sequence_refuses_more_and_unknown_sequences_are_refused() {
        let e = exec();
        let seq = e.sequence("full", Priority::BestEffort);
        // Block the sequence's first task so the queue fills.
        let (release, wait) = sync_channel::<()>(1);
        e.post(
            seq,
            Task::new(move || {
                let _ = wait.recv();
            }),
            None,
            Duration::ZERO,
        )
        .unwrap();
        let mut accepted = 1;
        let refused = loop {
            match e.post(seq, Task::new(|| {}), None, Duration::ZERO) {
                Ok(()) => accepted += 1,
                Err(err) => break err,
            }
            assert!(accepted <= DEFAULT_CAPACITY + 1, "no capacity limit");
        };
        assert_eq!(refused, PostError::QueueFull);
        assert!(accepted >= DEFAULT_CAPACITY);
        // An IPC delivery task still goes in past the capacity, and runs.
        let (done, ran) = sync_channel::<()>(1);
        assert!(e.has_sequence(seq));
        assert!(!e.has_sequence(SequenceHandle::from_raw(999)));
        e.post_reserved(
            seq,
            Task::new(move || {
                let _ = done.send(());
            }),
        )
        .unwrap();
        assert_eq!(
            e.post_reserved(SequenceHandle::from_raw(999), Task::new(|| {})),
            Err(PostError::UnknownSequence)
        );
        release.send(()).unwrap();
        ran.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(
            e.post(
                SequenceHandle::from_raw(999),
                Task::new(|| {}),
                None,
                Duration::ZERO
            ),
            Err(PostError::UnknownSequence)
        );
    }

    #[test]
    fn may_block_work_does_not_hold_up_other_sequences() {
        let e = OsExecutor::new(1, 1).unwrap();
        let slow = e.sequence("io", Priority::MayBlock);
        let fast = e.sequence("ui", Priority::UserBlocking);
        let (release, wait) = sync_channel::<()>(1);
        e.post(
            slow,
            Task::new(move || {
                let _ = wait.recv();
            }),
            None,
            Duration::ZERO,
        )
        .unwrap();
        let (tx, rx) = sync_channel(1);
        e.post(
            fast,
            Task::new(move || tx.send(()).unwrap()),
            None,
            Duration::ZERO,
        )
        .unwrap();
        rx.recv_timeout(StdDuration::from_secs(5))
            .expect("the general pool ran while MayBlock was blocked");
        release.send(()).unwrap();
    }

    #[test]
    fn fork_join_fills_every_index() {
        let e = exec();
        let out: Vec<AtomicUsize> = (0..100).map(|_| AtomicUsize::new(0)).collect();
        e.fork_join().for_each_index(100, &|i| {
            // ordering: Relaxed — each index is written once; scope join
            // orders the writes before the reads below.
            out[i].store(i.saturating_mul(3), Ordering::Relaxed);
        });
        assert!(
            out.iter()
                .enumerate()
                .all(|(i, v)| v.load(Ordering::Relaxed) == i * 3)
        );
    }

    #[test]
    fn the_executor_starts_its_general_workers_and_stops_them_on_drop() {
        let e = OsExecutor::new(3, 2).unwrap();
        // MayBlock workers and the timer thread start on demand.
        assert_eq!(e.thread_count(), 3);
        drop(e); // joins every thread; a hang here fails the test run
    }

    /// Waits up to 5 s for `f`.
    fn eventually(f: impl Fn() -> bool) -> bool {
        let t = StdInstant::now();
        while t.elapsed() < StdDuration::from_secs(5) {
            if f() {
                return true;
            }
            std::thread::sleep(StdDuration::from_millis(5));
        }
        false
    }

    /// FP-32 and footprint 6.3: a thread that is not needed at rest
    /// starts with its first task and ends after the idle period, so an
    /// idle process holds only its general workers.
    #[test]
    fn on_demand_threads_start_with_their_first_task_and_end_when_idle() {
        let e = OsExecutor::with_idle_exit(1, 1, StdDuration::from_millis(100)).unwrap();
        assert_eq!(e.thread_count(), 1);
        let io = e.sequence("io", Priority::MayBlock);
        let (tx, rx) = sync_channel(4);
        let t = tx.clone();
        e.post(
            io,
            Task::new(move || t.send(()).unwrap()),
            None,
            Duration::ZERO,
        )
        .unwrap();
        rx.recv_timeout(StdDuration::from_secs(5)).unwrap();
        assert_eq!(e.thread_count(), 2, "a MayBlock worker");
        let ui = e.sequence("ui", Priority::UserVisible);
        let t = tx.clone();
        e.post(
            ui,
            Task::new(move || t.send(()).unwrap()),
            Some(Duration::from_millis(10)),
            Duration::ZERO,
        )
        .unwrap();
        rx.recv_timeout(StdDuration::from_secs(5)).unwrap();
        assert!(e.thread_count() >= 2);
        assert!(
            eventually(|| e.thread_count() == 1),
            "the MayBlock worker and the timer thread end when idle"
        );
        // And start again when needed.
        e.post(
            io,
            Task::new(move || tx.send(()).unwrap()),
            None,
            Duration::ZERO,
        )
        .unwrap();
        rx.recv_timeout(StdDuration::from_secs(5)).unwrap();
        assert!(eventually(|| e.thread_count() == 1));
    }

    /// A delayed MayBlock task due after the idle period still finds its
    /// worker: the worker stays while one is pending.
    #[test]
    fn a_delayed_may_block_task_outlasts_the_idle_period() {
        let e = OsExecutor::with_idle_exit(1, 1, StdDuration::from_millis(30)).unwrap();
        let io = e.sequence("io", Priority::MayBlock);
        let (tx, rx) = sync_channel(1);
        e.post(
            io,
            Task::new(move || tx.send(()).unwrap()),
            Some(Duration::from_millis(300)),
            Duration::ZERO,
        )
        .unwrap();
        rx.recv_timeout(StdDuration::from_secs(5))
            .expect("the delayed MayBlock task ran");
    }

    /// FP-32: with no general worker, the caller's thread (a launched
    /// child's `main`) runs every general sequence until `done`.
    #[test]
    fn with_no_general_worker_the_caller_runs_the_sequences() {
        let e = Arc::new(OsExecutor::new(0, 1).unwrap());
        assert_eq!(e.thread_count(), 0);
        let seq = e.sequence("control", Priority::UserVisible);
        let me = std::thread::current().id();
        let ran_here = Arc::new(AtomicUsize::new(0));
        let done = Arc::new(AtomicUsize::new(0));
        let poster = {
            let (e, ran_here, done) = (Arc::clone(&e), Arc::clone(&ran_here), Arc::clone(&done));
            std::thread::spawn(move || {
                for n in 0..10 {
                    let (ran_here, done) = (Arc::clone(&ran_here), Arc::clone(&done));
                    e.post(
                        seq,
                        Task::new(move || {
                            if std::thread::current().id() == me {
                                // ordering: SeqCst — read after run_here returns.
                                ran_here.fetch_add(1, Ordering::SeqCst);
                            }
                            if n == 9 {
                                done.store(1, Ordering::SeqCst);
                            }
                        }),
                        None,
                        Duration::ZERO,
                    )
                    .unwrap();
                }
            })
        };
        e.run_here(&|| done.load(Ordering::SeqCst) == 1);
        poster.join().unwrap();
        assert_eq!(ran_here.load(Ordering::SeqCst), 10);
        assert_eq!(e.thread_count(), 0);
    }
}
