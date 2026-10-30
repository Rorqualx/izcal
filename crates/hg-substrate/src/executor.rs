//! Tasks, sequences and the phase pool (Stage 1 design 3.1; CS-D8,
//! CS-AS-3, CS-AS-4, CS-AS-6, CS-AS-7, CS-AS-12).
//!
//! The substrate owns every thread. Work is a [`Task`] posted to a named
//! sequence: tasks on one sequence run one at a time, in order, each to
//! completion. Parallel phases use [`PhasePool`], whose outputs are
//! index-addressed so the pool's interleaving cannot change a result.

use hg_types::Duration;

/// A closure that runs to completion on its sequence (CS-AS-4).
pub struct Task(Box<dyn FnOnce() + Send + 'static>);

impl Task {
    /// Wraps a closure as a task.
    pub fn new(f: impl FnOnce() + Send + 'static) -> Self {
        Self(Box::new(f))
    }

    /// Runs the task. Called by executors only.
    pub fn run(self) {
        (self.0)();
    }
}

impl core::fmt::Debug for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Task")
    }
}

/// A sequence's scheduling class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Priority {
    /// Input and frame production: the user is waiting on it.
    UserBlocking,
    /// Visible work that is not on the input-to-frame path.
    UserVisible,
    /// Work nobody is waiting on.
    BestEffort,
    /// The one class whose tasks may block (CS-AS-7): file and other
    /// synchronous OS calls, on threads apart from the others.
    MayBlock,
}

/// A handle naming one sequence of one executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SequenceHandle(u32);

impl SequenceHandle {
    /// A handle from the executor's own index. For executors only.
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// The executor's index for this sequence. For executors only.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Why a task was not accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostError {
    /// The sequence holds its capacity of pending tasks (CS-AS-6 (c):
    /// 1024† by default). The caller decides: drop with a counter, or
    /// retry later.
    QueueFull,
    /// The handle names no sequence of this executor.
    UnknownSequence,
    /// The executor is shutting down.
    ShuttingDown,
    /// The task needs a thread the executor starts on demand (a
    /// `MayBlock` worker, the timer thread), none of its kind is alive,
    /// and the OS would not start one. Nothing was queued; the caller
    /// decides as for [`Self::QueueFull`].
    NoThread,
}

impl core::fmt::Display for PostError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::QueueFull => "the sequence's task queue is full",
            Self::UnknownSequence => "no such sequence",
            Self::ShuttingDown => "the executor is shutting down",
            Self::NoThread => "no thread could be started for the task",
        })
    }
}

impl std::error::Error for PostError {}

/// Fork/join for deterministic parallel phases (CS-AS-12).
pub trait PhasePool: Send + Sync {
    /// Runs `f(0)`, ..., `f(n - 1)`, in parallel where possible, and
    /// returns when all have finished. Each call must write only its own
    /// index's output, so the result is the same in any order.
    fn for_each_index(&self, n: usize, f: &(dyn Fn(usize) + Sync));
}

/// The task scheduler.
pub trait Executor: Send + Sync {
    /// Creates a sequence named `name` (for diagnostics and the
    /// simulator's trace) with the given scheduling class.
    fn sequence(&self, name: &'static str, priority: Priority) -> SequenceHandle;

    /// Posts `task` to `seq`. With `delay`, the task runs no earlier than
    /// `delay` from now; `tolerance` is the band the scheduler may add to
    /// coalesce wakeups. A delayed task is a one-shot timer, armed by this
    /// call: the substrate has no periodic timers (footprint 6.8).
    ///
    /// # Errors
    /// [`PostError`] when the task was not accepted; the task is dropped
    /// unrun and the caller decides what to do (CS-ER-7).
    fn post(
        &self,
        seq: SequenceHandle,
        task: Task,
        delay: Option<Duration>,
        tolerance: Duration,
    ) -> Result<(), PostError>;

    /// The fork/join pool for parallel phases.
    fn fork_join(&self) -> &dyn PhasePool;
}
