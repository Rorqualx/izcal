//! Decision points (JS spec 6.9, JS-D15): the moments where a process
//! applies the result of work whose timing depends on other threads, such
//! as finishing a GC cycle (and so clearing weak references) or
//! installing compiled code. Each one is a recorded input (B1).
//!
//! - **Production.** The work is applied at the first safepoint after it
//!   is ready. When the session is recorded, the canonical position at
//!   which it was applied is logged. Production recording is not built
//!   yet, so [`DecisionLog::record`] does nothing there.
//! - **Simulation.** Background work runs on the seeded scheduler, so a
//!   decision's position is a function of the seed. The simulator writes
//!   each decision into its trace, which is the run's one ordered
//!   decision log (the recording header's `DecisionLogRef`).
//!
//! Every subsystem's decisions share the one stream, keyed by position
//! (JS spec 6.9's consolidated list); each owning spec adds its kinds.

/// A canonical position: the agent and its progress counter. It is a
/// count of the agent's own work, never time or a host address, so the
/// same input gives the same position on every OS and tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    /// The agent (an event loop with its own heap) within the process.
    pub agent: u32,
    /// The agent's progress counter when the decision was applied.
    pub progress: u64,
}

/// What kind of decision was applied. The owning spec assigns each value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DecisionKind(pub u16);

impl DecisionKind {
    /// JS engine: compiled code installed.
    pub const CODE_INSTALL: Self = Self(1);
    /// JS engine: a GC cycle finished, and its weak references cleared.
    pub const GC_COMPLETE: Self = Self(2);
    /// JS engine: an off-thread parse or a code-cache load installed.
    pub const PARSE_INSTALL: Self = Self(3);
    /// JS engine: a memory-pressure signal applied.
    pub const PRESSURE: Self = Self(4);
    /// JS engine: an intervention decision (JS spec 6.8).
    pub const INTERVENTION: Self = Self(5);
    /// JS engine: a real-time worklet installed at a quantum boundary.
    pub const WORKLET_INSTALL: Self = Self(6);
}

/// The decision log of one process.
pub trait DecisionLog: Send + Sync {
    /// Records that a decision of `kind` was applied at `at`. `detail` is
    /// the kind's own summary of what was decided (for a GC cycle, for
    /// example, how many weak references it cleared), so two runs that
    /// decide differently at the same position log differently.
    fn record(&self, kind: DecisionKind, at: Position, detail: u64);
}

/// The log of a process that is not recording: it keeps nothing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NotRecording;

impl DecisionLog for NotRecording {
    fn record(&self, _kind: DecisionKind, _at: Position, _detail: u64) {}
}
