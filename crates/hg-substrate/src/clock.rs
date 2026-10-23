//! Time (Stage 1 design 3.1). Production reads the OS clocks; simulation
//! returns virtual time, advanced only by the scheduler.

use hg_types::{Instant, SystemTime};

/// The process's clocks.
pub trait Clock: Send + Sync {
    /// Monotonic time: never goes backwards, unaffected by wall-clock
    /// changes. For intervals, deadlines and timers.
    fn monotonic(&self) -> Instant;

    /// Wall-clock time. For `Date.now()`, cookies and HTTP dates; it can
    /// jump, so never for intervals.
    fn wall(&self) -> SystemTime;
}
