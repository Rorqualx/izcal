//! The OS executor behind the substrate's `Executor` trait (Stage 1
//! design 3.1; CS-AS-3, CS-AS-4, CS-AS-6, CS-AS-7, CS-AS-12; footprint 6.8).
//!
//! - **Sequences.** Each sequence is a FIFO of tasks; at most one of its
//!   tasks runs at a time, so its tasks run in order. A sequence waiting
//!   to run sits in its priority's ready list.
//! - **Workers.** A general pool runs `UserBlocking`, then `UserVisible`,
//!   then `BestEffort` sequences; a separate pool runs `MayBlock`
//!   sequences, so a blocking task never holds up the others (CS-AS-7).
//!   An idle worker waits on a condition variable with **no timeout**:
//!   an idle executor makes no wakeups (footprint 6.8, FP-18).
//! - **Timers.** A delayed post is a one-shot timer. One timer thread
//!   sleeps until the earliest *latest-allowed* time (deadline plus
//!   tolerance), then moves every task whose deadline has passed onto
//!   its sequence. That fires each task no earlier than its delay and no
//!   later than its band, and batches timers that can share a wakeup.
//!   With no timers it waits with no timeout.
//! - **Capacity.** A sequence holds at most [`DEFAULT_CAPACITY`] pending
//!   tasks, queued and delayed together (CS-AS-6 (c)); past it, `post`
//!   returns `QueueFull`.
//! - **Phases.** `fork_join` runs an index-addressed loop on scoped
//!   threads, one per worker, with indices dealt round-robin (CS-AS-12).
//!
//! The executor owns every thread it uses (CS-AS-3). Dropping it stops
//! the threads once their current task finishes; pending tasks are
//! dropped unrun.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Instant as StdInstant;

use hg_substrate::{Executor, PhasePool, PostError, Priority, SequenceHandle, Task};
use hg_types::Duration;

/// Pending tasks a sequence may hold (CS-AS-6 (c)): 1024†.
pub const DEFAULT_CAPACITY: usize = 1024;

/// Which pool a priority runs in, and its rank inside the general pool.
const fn class(p: Priority) -> usize {
    match p {
        Priority::UserBlocking => 0,
        Priority::UserVisible => 1,
        Priority::BestEffort => 2,
        Priority::MayBlock => 3,
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
    stopping: bool,
}

struct Shared {
    state: Mutex<State>,
    general: Condvar,
    may_block: Condvar,
    timer: Condvar,
    capacity: usize,
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
        if class == 3 {
            self.may_block.notify_one();
        } else {
            self.general.notify_one();
        }
    }
}

/// The OS executor.
pub struct OsExecutor {
    shared: Arc<Shared>,
    threads: Mutex<Vec<JoinHandle<()>>>,
    pool: OsPhasePool,
}

impl OsExecutor {
    /// An executor with `general` workers for the three ordinary
    /// priorities and `may_block` workers for `MayBlock` sequences, plus
    /// one timer thread.
    ///
    /// # Errors
    /// The OS error if a thread cannot be started.
    pub fn new(general: usize, may_block: usize) -> std::io::Result<Self> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                sequences: Vec::new(),
                ready: Default::default(),
                timers: BinaryHeap::new(),
                timer_tasks: BTreeMap::new(),
                next_timer: 0,
                stopping: false,
            }),
            general: Condvar::new(),
            may_block: Condvar::new(),
            timer: Condvar::new(),
            capacity: DEFAULT_CAPACITY,
        });
        let mut threads = Vec::new();
        for n in 0..general.max(1) {
            let s = Arc::clone(&shared);
            threads.push(spawn(format!("hg-worker-{n}"), move || worker(&s, false))?);
        }
        for n in 0..may_block.max(1) {
            let s = Arc::clone(&shared);
            threads.push(spawn(format!("hg-may-block-{n}"), move || {
                worker(&s, true)
            })?);
        }
        let s = Arc::clone(&shared);
        threads.push(spawn("hg-timers".into(), move || timers(&s))?);
        Ok(Self {
            shared,
            threads: Mutex::new(threads),
            pool: OsPhasePool {
                workers: general.max(1),
            },
        })
    }

    /// The number of threads this executor started (CS-AS-3's check:
    /// the process's thread count after startup is the substrate's).
    pub fn thread_count(&self) -> usize {
        self.threads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
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

fn worker(shared: &Shared, may_block: bool) {
    let cv = if may_block {
        &shared.may_block
    } else {
        &shared.general
    };
    let mut st = shared.lock();
    loop {
        if st.stopping {
            return;
        }
        let picked = if may_block {
            st.ready.get_mut(3).and_then(VecDeque::pop_front)
        } else {
            st.ready.iter_mut().take(3).find_map(VecDeque::pop_front)
        };
        let Some(i) = picked else {
            // No timeout: an idle worker makes no wakeups.
            st = cv.wait(st).unwrap_or_else(PoisonError::into_inner);
            continue;
        };
        let task = st
            .sequences
            .get_mut(usize::try_from(i).unwrap_or(usize::MAX))
            .and_then(|seq| {
                seq.listed = false;
                let t = seq.queue.pop_front();
                seq.running = t.is_some();
                t
            });
        let Some(task) = task else { continue };
        drop(st);
        task.run();
        st = shared.lock();
        if let Some(seq) = st
            .sequences
            .get_mut(usize::try_from(i).unwrap_or(usize::MAX))
        {
            seq.running = false;
        }
        shared.make_ready(&mut st, i);
    }
}

fn timers(shared: &Shared) {
    let mut st = shared.lock();
    loop {
        if st.stopping {
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
                    seq.queue.push_back(t.task);
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
            None => shared
                .timer
                .wait(st)
                .unwrap_or_else(PoisonError::into_inner),
            Some(latest) => {
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
        let Some(s) = st
            .sequences
            .get_mut(usize::try_from(i).unwrap_or(usize::MAX))
        else {
            return Err(PostError::UnknownSequence);
        };
        if s.queue.len().saturating_add(s.delayed) >= self.shared.capacity {
            return Err(PostError::QueueFull);
        }
        match delay {
            None => {
                s.queue.push_back(task);
                self.shared.make_ready(&mut st, i);
            }
            Some(d) => {
                s.delayed = s.delayed.saturating_add(1);
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
        not(any(target_os = "linux", target_os = "macos", test)),
        expect(
            dead_code,
            reason = "used by the Linux and macOS IPC transports; D5 adds Windows"
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
    /// [`PostError::ShuttingDown`] or [`PostError::UnknownSequence`].
    #[cfg_attr(
        not(any(target_os = "linux", target_os = "macos", test)),
        expect(
            dead_code,
            reason = "used by the Linux and macOS IPC transports; D5 adds Windows"
        )
    )]
    pub(crate) fn post_reserved(&self, seq: SequenceHandle, task: Task) -> Result<(), PostError> {
        let mut st = self.shared.lock();
        if st.stopping {
            return Err(PostError::ShuttingDown);
        }
        let i = seq.raw();
        let Some(s) = st
            .sequences
            .get_mut(usize::try_from(i).unwrap_or(usize::MAX))
        else {
            return Err(PostError::UnknownSequence);
        };
        s.queue.push_back(task);
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
    fn the_executor_owns_a_fixed_set_of_threads_and_stops_them_on_drop() {
        let e = OsExecutor::new(3, 2).unwrap();
        assert_eq!(e.thread_count(), 3 + 2 + 1);
        drop(e); // joins every thread; a hang here fails the test run
    }
}
