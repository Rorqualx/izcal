//! Virtual time: the scheduler's clock, read by every simulated process.

use std::sync::Arc;

use hg_substrate::Clock;
use hg_types::{Instant, SystemTime};

use crate::world::Shared;

pub(crate) struct SimClock {
    pub(crate) shared: Arc<Shared>,
}

impl Clock for SimClock {
    fn monotonic(&self) -> Instant {
        self.shared.lock().now
    }

    fn wall(&self) -> SystemTime {
        let st = self.shared.lock();
        let since_zero = st.now.saturating_duration_since(Instant::from_nanos(0));
        st.config
            .wall_epoch
            .checked_add(since_zero)
            .unwrap_or(st.config.wall_epoch)
    }
}
