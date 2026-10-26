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

use hg_substrate::launch::{ChildKind, LaunchError, Launched, Launcher};
use hg_substrate::{Memory, Substrate};

use crate::world::{ProcessId, Shared, World};

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
}

impl SimLauncher {
    pub(crate) fn new(shared: Arc<Shared>, parent: ProcessId, table: Arc<ChildTable>) -> Self {
        Self {
            shared,
            parent,
            table,
            restart_required: AtomicBool::new(false),
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
        let world = World {
            shared: Arc::clone(&self.shared),
        };
        let entry: crate::world::Entry = Arc::clone(&child.entry);
        let pid = world.spawn_inner(child.name, (child.backing)(), false, entry, None, None);
        {
            let mut st = self.shared.lock();
            st.trace.record(format_args!(
                "launch {kind:?} by {} as {}",
                self.parent.0, pid.0
            ));
        }
        let (from_child, to_child) = world
            .link(self.parent, pid, capacity)
            .ok_or(LaunchError::Failed)?;
        let peer = world.os_peer(pid).ok_or(LaunchError::Failed)?;
        Ok(Launched {
            from_child,
            to_child,
            peer,
        })
    }

    fn restart_required(&self) {
        if !self.restart_required.swap(true, Ordering::SeqCst) {
            let mut st = self.shared.lock();
            st.trace
                .record(format_args!("restart-required in {}", self.parent.0));
        }
    }
}
