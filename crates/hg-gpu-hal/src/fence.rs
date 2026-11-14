//! Fences on one monotonic timeline per device (§6.6).
//!
//! One timeline per device, not one object per submission. A fence is
//! a **value** on that timeline, and "has this finished" is a
//! comparison — which means a caller can wait for several pieces of
//! work by waiting for the largest of their values, and a wedged
//! submission cannot hold a handle nobody can free.
//!
//! Nothing here waits. §6.6 forbids a blocking map and G72 forbids
//! the compositor waiting on a fence it did not create; both reduce
//! to the same rule, which is that a fence is *observed*. The
//! timeline's value moves when the back end says it has.

use core::fmt;

/// A point on a device's timeline.
///
/// Monotonic and per device: comparing fences from two devices is a
/// bug, and [`Timeline::signal`] is the only thing that moves one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fence {
    device: u32,
    value: u64,
}

impl Fence {
    /// Which device's timeline it is on.
    #[must_use]
    pub const fn device(self) -> u32 {
        self.device
    }

    /// Its value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }
}

impl fmt::Display for Fence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "device {} at {}", self.device, self.value)
    }
}

/// One device's timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timeline {
    device: u32,
    submitted: u64,
    completed: u64,
}

impl Timeline {
    /// A timeline for `device`, with nothing submitted.
    #[must_use]
    pub const fn new(device: u32) -> Self {
        Self {
            device,
            submitted: 0,
            completed: 0,
        }
    }

    /// The fence a submission returns: the next value.
    pub const fn submit(&mut self) -> Fence {
        self.submitted = self.submitted.saturating_add(1);
        Fence {
            device: self.device,
            value: self.submitted,
        }
    }

    /// The back end reports work completed up to `value`.
    ///
    /// Returns whether the timeline moved. A value that is not an
    /// advance is **ignored**, not an error: a driver that reports
    /// completion out of order is a driver, and treating it as fatal
    /// would make Izcal's correctness depend on every vendor's.
    /// Moving backwards, though, would make a finished fence
    /// unfinished, which is what the comparison below must never
    /// allow.
    pub const fn signal(&mut self, value: u64) -> bool {
        if value > self.completed && value <= self.submitted {
            self.completed = value;
            true
        } else {
            false
        }
    }

    /// Whether `fence` has completed.
    ///
    /// `false` for a fence from another device: the question has no
    /// answer here, and answering `true` would be the worst of the
    /// two wrong answers.
    #[must_use]
    pub const fn is_complete(&self, fence: Fence) -> bool {
        fence.device == self.device && fence.value <= self.completed
    }

    /// The largest completed value.
    #[must_use]
    pub const fn completed(&self) -> u64 {
        self.completed
    }

    /// The largest submitted value.
    #[must_use]
    pub const fn submitted(&self) -> u64 {
        self.submitted
    }

    /// Whether everything submitted has completed.
    #[must_use]
    pub const fn is_idle(&self) -> bool {
        self.completed == self.submitted
    }
}
