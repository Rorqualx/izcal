//! Time values. Only the substrate's `Clock` creates them from a real or
//! virtual clock (Stage 1 design 3.1); everything else does arithmetic.
//! All arithmetic is checked (CS-TY-12): an overflow is `None`, never a
//! wrapped value.

pub use core::time::Duration;

/// A monotonic point in time: nanoseconds since the clock's origin
/// (process start in production, zero in simulation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    nanos: u64,
}

impl Instant {
    /// The clock's origin.
    pub const ORIGIN: Self = Self { nanos: 0 };

    /// An instant `nanos` after the clock's origin. For clock
    /// implementations; other code gets instants from a `Clock`.
    pub const fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }

    /// Nanoseconds since the clock's origin.
    pub const fn as_nanos(self) -> u64 {
        self.nanos
    }

    /// `self + d`, or `None` if it does not fit.
    pub fn checked_add(self, d: Duration) -> Option<Self> {
        let d = u64::try_from(d.as_nanos()).ok()?;
        self.nanos.checked_add(d).map(Self::from_nanos)
    }

    /// The time from `earlier` to `self`, or `None` if `earlier` is later.
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        self.nanos
            .checked_sub(earlier.nanos)
            .map(Duration::from_nanos)
    }

    /// The time from `earlier` to `self`, or zero if `earlier` is later.
    pub fn saturating_duration_since(self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier)
            .unwrap_or(Duration::ZERO)
    }
}

/// A wall-clock time: signed nanoseconds since the Unix epoch
/// (1970-01-01T00:00:00Z), which covers the years 1677 to 2262. For
/// `Date.now()`, cookies and HTTP dates; never for measuring intervals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime {
    unix_nanos: i64,
}

impl SystemTime {
    /// The Unix epoch.
    pub const UNIX_EPOCH: Self = Self { unix_nanos: 0 };

    /// A time `unix_nanos` from the Unix epoch. For clock
    /// implementations; other code gets times from a `Clock`.
    pub const fn from_unix_nanos(unix_nanos: i64) -> Self {
        Self { unix_nanos }
    }

    /// Signed nanoseconds since the Unix epoch.
    pub const fn as_unix_nanos(self) -> i64 {
        self.unix_nanos
    }

    /// `self + d`, or `None` if it does not fit.
    pub fn checked_add(self, d: Duration) -> Option<Self> {
        let d = i64::try_from(d.as_nanos()).ok()?;
        self.unix_nanos.checked_add(d).map(Self::from_unix_nanos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instant_arithmetic_is_checked() {
        let a = Instant::from_nanos(1_000);
        let b = a.checked_add(Duration::from_nanos(500)).unwrap();
        assert_eq!(b.as_nanos(), 1_500);
        assert_eq!(b.checked_duration_since(a), Some(Duration::from_nanos(500)));
        assert_eq!(a.checked_duration_since(b), None);
        assert_eq!(a.saturating_duration_since(b), Duration::ZERO);
        assert_eq!(
            Instant::from_nanos(u64::MAX).checked_add(Duration::from_nanos(1)),
            None
        );
        assert_eq!(a.checked_add(Duration::MAX), None);
    }

    #[test]
    fn system_time_is_signed_and_checked() {
        let before = SystemTime::from_unix_nanos(-5);
        assert!(before < SystemTime::UNIX_EPOCH);
        assert_eq!(
            before.checked_add(Duration::from_nanos(5)),
            Some(SystemTime::UNIX_EPOCH)
        );
        assert_eq!(
            SystemTime::from_unix_nanos(i64::MAX).checked_add(Duration::from_nanos(1)),
            None
        );
    }
}
