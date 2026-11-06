//! The simulated decision log (JS-D15): each decision point a process
//! applies becomes one line of the run's trace, so it is hashed into the
//! same ordered stream as the scheduler's decisions and the recording
//! header's `DecisionLogRef` covers it.

use std::sync::Arc;

use hg_substrate::{DecisionKind, DecisionLog, Position};

use crate::world::{ProcessId, Shared};

pub(crate) struct SimDecisions {
    pub(crate) shared: Arc<Shared>,
    pub(crate) pid: ProcessId,
    pub(crate) generation: u32,
}

impl DecisionLog for SimDecisions {
    fn record(&self, kind: DecisionKind, at: Position, detail: u64) {
        let mut st = self.shared.lock();
        // A process that has crashed decides nothing.
        let Some(p) = st.proc(self.pid, self.generation) else {
            return;
        };
        let (name, generation) = (p.name, p.generation);
        st.trace.record(format_args!(
            "decide {name}.{generation} k{} a{} p{} d{detail}",
            kind.0, at.agent, at.progress
        ));
    }
}
