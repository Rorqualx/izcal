//! The OS clock (Stage 1 design 3.1). Monotonic time counts from the
//! clock's creation; wall time is the OS's.

use std::time::{Instant as StdInstant, SystemTime as StdSystemTime, UNIX_EPOCH};

use hg_substrate::Clock;
use hg_types::{Instant, SystemTime};

/// The OS clock.
#[derive(Debug)]
pub struct OsClock {
    origin: StdInstant,
}

impl OsClock {
    /// A clock whose monotonic origin is now.
    pub fn new() -> Self {
        Self {
            origin: StdInstant::now(),
        }
    }
}

impl Default for OsClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for OsClock {
    fn monotonic(&self) -> Instant {
        // Saturates after ~584 years of uptime.
        let nanos = u64::try_from(self.origin.elapsed().as_nanos()).unwrap_or(u64::MAX);
        Instant::from_nanos(nanos)
    }

    fn wall(&self) -> SystemTime {
        let now = StdSystemTime::now();
        let nanos = match now.duration_since(UNIX_EPOCH) {
            Ok(after) => i64::try_from(after.as_nanos()).unwrap_or(i64::MAX),
            Err(before) => i64::try_from(before.duration().as_nanos())
                .map_or(i64::MIN, |n| n.checked_neg().unwrap_or(i64::MIN)),
        };
        SystemTime::from_unix_nanos(nanos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotonic_never_goes_back_and_wall_is_recent() {
        let c = OsClock::new();
        let a = c.monotonic();
        let b = c.monotonic();
        assert!(b >= a);
        // After 2020-01-01 (1.577e18 ns), before 2100 (4.1e18 ns).
        let w = c.wall().as_unix_nanos();
        assert!(w > 1_577_836_800_000_000_000 && w < 4_102_444_800_000_000_000);
    }
}
