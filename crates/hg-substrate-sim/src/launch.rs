//! The simulated launcher (M1 phase E): the browser's
//! [`hg_substrate::Launcher`] under simulation.
//!
//! A launch spawns a new simulated process from the entry the browser
//! registered for that kind ([`ChildTable`]) and links its bootstrap
//! channels to the browser, exactly what [`World::spawn`] plus
//! [`World::link`] do for a test. Launched children are never restarted
//! by the world: after a crash the browser sees `Closed` and relaunches.
//! The restart-required state (distribution DU-3) lives in the launcher,
//! so it ends with the browser's incarnation, as on a real restart.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use hg_substrate::ipc::OsPeer;
use hg_substrate::launch::{ChildKind, LaunchError, Launched, Launcher, SandboxTier};
use hg_substrate::{Memory, Substrate};

use crate::world::{ProcessId, Shared};

/// One launchable kind: its process name, its memory backing and its
/// entry function.
#[derive(Clone)]
pub struct ChildEntry {
    /// The process name in the trace.
    pub name: &'static str,
    /// Makes each child's memory backing.
    pub backing: Arc<dyn Fn() -> Arc<dyn Memory> + Send + Sync>,
    /// The child's entry function.
    pub entry: Arc<dyn Fn(Substrate) + Send + Sync>,
}

/// What a simulated browser can launch, by kind.
#[derive(Clone, Default)]
pub struct ChildTable {
    entries: BTreeMap<ChildKind, ChildEntry>,
}

impl ChildTable {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `kind`'s entry.
    #[must_use]
    pub fn with(mut self, kind: ChildKind, entry: ChildEntry) -> Self {
        self.entries.insert(kind, entry);
        self
    }
}

pub(crate) struct SimLauncher {
    shared: Arc<Shared>,
    parent: ProcessId,
    table: Arc<ChildTable>,
    restart_required: AtomicBool,
    /// The children this launcher started, with the incarnation each was
    /// started as: `kill` reaches only these.
    started: std::sync::Mutex<BTreeMap<ProcessId, u32>>,
}

impl SimLauncher {
    pub(crate) fn new(shared: Arc<Shared>, parent: ProcessId, table: Arc<ChildTable>) -> Self {
        Self {
            shared,
            parent,
            table,
            restart_required: AtomicBool::new(false),
            started: std::sync::Mutex::default(),
        }
    }
}

impl Launcher for SimLauncher {
    fn launch(&self, kind: ChildKind, capacity: u32) -> Result<Launched, LaunchError> {
        if self.restart_required.load(Ordering::SeqCst) {
            return Err(LaunchError::RestartRequired);
        }
        let Some(child) = self.table.entries.get(&kind) else {
            return Err(LaunchError::Unsupported);
        };
        // The launch faults (E-R8), each from its own stream, drawn in
        // this fixed order and only when its rate is not zero.
        let stall = {
            let mut st = self.shared.lock();
            let plan = st.config.faults;
            if crate::world::fires(&mut st.faults.spawner_death, plan.spawner_death_per_million) {
                // E-R2: the spawner died with this spawn in flight. The
                // broker restarts it; this launch is `Failed{spawner}`.
                st.trace.record(format_args!(
                    "launch {kind:?} by {} failed: spawner died, restarted",
                    self.parent.0
                ));
                return Err(LaunchError::Failed);
            }
            if crate::world::fires(&mut st.faults.launch_fail, plan.launch_fail_per_million) {
                st.trace.record(format_args!(
                    "launch {kind:?} by {} failed: refused",
                    self.parent.0
                ));
                return Err(LaunchError::Failed);
            }
            crate::world::fires(&mut st.faults.ready_stall, plan.ready_stall_per_million)
                .then_some(plan.ready_stall)
        };
        let entry: crate::world::Entry = Arc::clone(&child.entry);
        let pid = crate::world::spawn_in(
            &self.shared,
            child.name,
            (child.backing)(),
            entry,
            crate::world::SpawnOptions {
                stall,
                ..crate::world::SpawnOptions::default()
            },
        );
        {
            let mut st = self.shared.lock();
            st.trace.record(format_args!(
                "launch {kind:?} by {} as {}",
                self.parent.0, pid.0
            ));
            let ns = st.config.pid_namespaces;
            if let Some(p) = st
                .procs
                .get_mut(usize::try_from(pid.0).unwrap_or(usize::MAX))
            {
                p.pid_ns = ns;
            }
        }
        // A child that cannot be linked is ended at once: a failed launch
        // leaves no process behind.
        let Some((from_child, to_child)) =
            crate::ipc::link(&self.shared, self.parent, pid, capacity)
        else {
            crate::world::crash_in(&self.shared, pid);
            return Err(LaunchError::Failed);
        };
        let Some(peer) = crate::world::os_peer_in(&self.shared, pid) else {
            crate::world::crash_in(&self.shared, pid);
            return Err(LaunchError::Failed);
        };
        if let Ok(generation) = u32::try_from(peer.generation()) {
            self.started
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(pid, generation);
        }
        Ok(Launched {
            from_child,
            to_child,
            peer,
            sandbox: SandboxTier::Simulated,
        })
    }

    fn kill(&self, peer: OsPeer) {
        let pid = ProcessId(peer.pid());
        let ours = self
            .started
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&pid)
            .is_some_and(|&g| u64::from(g) == peer.generation());
        if !ours {
            return;
        }
        {
            let mut st = self.shared.lock();
            st.trace
                .record(format_args!("kill {} by {}", pid.0, self.parent.0));
        }
        crate::world::crash_in_generation(&self.shared, pid, peer.generation());
    }

    fn restart_required(&self) {
        if !self.restart_required.swap(true, Ordering::SeqCst) {
            let mut st = self.shared.lock();
            st.trace
                .record(format_args!("restart-required in {}", self.parent.0));
        }
    }
}
