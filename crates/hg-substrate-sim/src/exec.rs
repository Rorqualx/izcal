//! A simulated process's executor: posts go into the world's scheduler.

use std::sync::Arc;

use hg_substrate::{Executor, PhasePool, PostError, Priority, SequenceHandle, Task};
use hg_types::Duration;

use crate::world::{ProcessId, SEQUENCE_CAPACITY, Seq, Shared, Slot, below, priority_name};

pub(crate) struct SimExecutor {
    pub(crate) shared: Arc<Shared>,
    pub(crate) pid: ProcessId,
    pub(crate) generation: u32,
    pub(crate) pool: SimPhasePool,
}

impl Executor for SimExecutor {
    fn sequence(&self, name: &'static str, priority: Priority) -> SequenceHandle {
        let mut st = self.shared.lock();
        let Some(p) = st.proc_mut(self.pid, self.generation) else {
            // A dead incarnation: the handle names nothing, so posts fail.
            return SequenceHandle::from_raw(u32::MAX);
        };
        let index = u32::try_from(p.seqs.len()).unwrap_or(u32::MAX);
        p.seqs.push(Seq {
            name,
            queue: std::collections::VecDeque::new(),
            delayed: 0,
            posted: 0,
        });
        let pname = p.name;
        let generation = self.generation;
        st.trace.record(format_args!(
            "sequence {pname}.{generation}/{name} {}",
            priority_name(priority)
        ));
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
        let Some(p) = st.proc_mut(self.pid, self.generation) else {
            return Err(PostError::ShuttingDown);
        };
        let si = usize::try_from(seq.raw()).unwrap_or(usize::MAX);
        let Some(s) = p.seqs.get_mut(si) else {
            return Err(PostError::UnknownSequence);
        };
        if s.queue.len().saturating_add(s.delayed) >= SEQUENCE_CAPACITY {
            return Err(PostError::QueueFull);
        }
        let number = s.posted;
        s.posted = s.posted.saturating_add(1);
        match delay {
            None => s.queue.push_back((number, task)),
            Some(d) => {
                s.delayed = s.delayed.saturating_add(1);
                let slot = Slot {
                    pid: self.pid,
                    generation: self.generation,
                    seq: si,
                    number,
                };
                st.add_timer(slot, task, d, tolerance);
            }
        }
        Ok(())
    }

    fn fork_join(&self) -> &dyn PhasePool {
        &self.pool
    }
}

/// Runs a phase's indices one at a time in a seeded order, so a phase
/// whose result depends on order (a CS-AS-12 violation) shows up as a
/// seed-dependent result.
pub(crate) struct SimPhasePool {
    pub(crate) shared: Arc<Shared>,
}

impl PhasePool for SimPhasePool {
    fn for_each_index(&self, n: usize, f: &(dyn Fn(usize) + Sync)) {
        let mut order: Vec<usize> = (0..n).collect();
        {
            let mut st = self.shared.lock();
            // Fisher–Yates with the scheduling stream.
            for i in (1..n).rev() {
                let bound = u64::try_from(i).unwrap_or(u64::MAX).saturating_add(1);
                let j = usize::try_from(below(&mut st.sched, bound)).unwrap_or(0);
                order.swap(i, j);
            }
            st.trace.record(format_args!("phase n={n}"));
        }
        for i in order {
            f(i);
        }
    }
}
