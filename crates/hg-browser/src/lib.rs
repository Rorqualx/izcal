//! `hg-browser`: the browser (broker) process's logic (Stage 1 design 8.1
//! M1).
//!
//! Layer L5, tier P. `hg-browser-main` calls [`run`] with the OS
//! substrate; the simulation binary calls it with a simulated one. At M1
//! the browser launches one renderer and one network partition service
//! through its [`Launcher`], routes each renderer `ask` to the network as
//! `work` and each `answer` back as `answered`, and relaunches a child
//! that ends, within a budget.
//!
//! - **Build check (distribution DU-3, 13 item 6).** A child whose `ready`
//!   names another `BuildId` is told `exit(BuildMismatch)` and the
//!   launcher enters restart-required: no further launches of any kind,
//!   no retry.
//! - **Identity (design review M6, M8).** Each child's channel is bound to
//!   the `OsPeer` its launch returned; a record from any other process is
//!   a violation the transport reports, and the child is not relaunched
//!   for it.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use hg_ipc::{
    BoundedBytes, CallerIdentity, EndReason, Endpoint, PeerBinding, PeerId, Remote, Serving,
};
use hg_ipc_gen::process::{
    BuildId, ChildConfig, ExitReason, ProcessControl, ProcessControlRemote, ProcessError,
    ProcessHost, ProcessHostHandler, ProcessKind, SandboxReport,
};
use hg_substrate::ipc::PeerAttest;
use hg_substrate::launch::{ChildKind, LaunchError, Launcher};
use hg_substrate::{Priority, SequenceHandle, Substrate, Task};

/// Bootstrap channel capacity for each child.
const CAPACITY: u32 = 64;
/// Asks held while the network is not ready. The renderer is untrusted:
/// one more ask than this ends its channel.
pub const MAX_PENDING: usize = 64;

/// What the browser has done, for its owner to watch (tests, DST and
/// FP-27's timing hooks).
#[derive(Debug, Default)]
pub struct Progress {
    /// Successful launches, first ones included.
    pub launches: AtomicU64,
    /// Launches that failed.
    pub launch_failures: AtomicU64,
    /// Children that ended and were launched again.
    pub relaunches: AtomicU64,
    /// `ready` messages with another build (each enters restart-required).
    pub mismatches: AtomicU64,
    /// Asks routed to the network.
    pub routed: AtomicU64,
    /// Answers routed back to the renderer.
    pub returned: AtomicU64,
    /// Asks refused because [`MAX_PENDING`] were already waiting.
    pub refused_asks: AtomicU64,
    /// The most asks ever waiting at once.
    pub max_pending: AtomicU64,
}

/// How the browser runs.
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// Relaunches allowed per kind before the browser stops relaunching it.
    pub restart_budget: u32,
    /// Progress counters.
    pub progress: Arc<Progress>,
}

/// A running browser: keep it for as long as the process runs.
#[derive(Clone)]
pub struct Running {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for Running {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Running").finish_non_exhaustive()
    }
}

impl Running {
    /// Whether the browser is in the restart-required state (DU-3).
    #[must_use]
    pub fn restart_required(&self) -> bool {
        self.inner.lock().halted
    }
}

/// One child the browser holds.
struct Child {
    control: Remote<ProcessControl>,
    ready: bool,
    /// Set once serving started; the slot is filled before, so a `ready`
    /// on the browser's sequence always finds its child.
    serving: Option<Serving>,
}

#[derive(Default)]
struct State {
    renderer: Option<Child>,
    network: Option<Child>,
    /// Asks waiting for a ready network, in order: (renderer's id, payload).
    pending: Vec<(u64, BoundedBytes<256>)>,
    /// Asks sent to the network and not yet answered, in order. If the
    /// network ends they go back to the front of `pending`, so the
    /// relaunched network answers them and the renderer is not left
    /// waiting.
    inflight: Vec<(u64, BoundedBytes<256>)>,
    budget_renderer: u32,
    budget_network: u32,
    /// Restart-required: set once, never cleared (DU-3).
    halted: bool,
}

struct Inner {
    s: Substrate,
    launcher: Arc<dyn Launcher>,
    seq: SequenceHandle,
    progress: Arc<Progress>,
    state: Mutex<State>,
}

/// The two kinds M1's browser runs, and their wire names.
fn wire_kind(kind: ChildKind) -> ProcessKind {
    match kind {
        ChildKind::Renderer => ProcessKind::Renderer,
        ChildKind::NetworkSystem => ProcessKind::NetworkSystem,
        ChildKind::NetworkContainer => ProcessKind::NetworkContainer,
        ChildKind::NetworkPartition => ProcessKind::NetworkPartition,
    }
}

fn config_for(kind: ChildKind) -> ChildConfig {
    match kind {
        ChildKind::Renderer => ChildConfig::Renderer { site: 1 },
        ChildKind::NetworkSystem => ChildConfig::NetworkSystem,
        ChildKind::NetworkContainer => ChildConfig::NetworkContainer { container: 1 },
        ChildKind::NetworkPartition => ChildConfig::NetworkPartition {
            top_site: 1,
            container: 1,
        },
    }
}

impl Inner {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn slot(st: &mut State, kind: ChildKind) -> &mut Option<Child> {
        if kind == ChildKind::Renderer {
            &mut st.renderer
        } else {
            &mut st.network
        }
    }

    /// Launches `kind` and serves its bootstrap channel.
    fn launch(self: &Arc<Self>, kind: ChildKind) -> Result<(), LaunchError> {
        if self.lock().halted {
            return Err(LaunchError::RestartRequired);
        }
        let launched = match self.launcher.launch(kind, CAPACITY) {
            Ok(l) => l,
            Err(e) => {
                self.progress.launch_failures.fetch_add(1, Ordering::SeqCst);
                return Err(e);
            }
        };
        let control = Remote::<ProcessControl>::from_sender(&self.s.ipc, launched.to_child);
        let mut side = HostSide {
            inner: Arc::clone(self),
            kind,
        };
        let on_end = {
            let inner = Arc::clone(self);
            move |why: EndReason<ProcessError>| inner.ended(kind, &why)
        };
        let peer_id = PeerId::new(if kind == ChildKind::Renderer { 1 } else { 2 }, 0);
        *Self::slot(&mut self.lock(), kind) = Some(Child {
            control,
            ready: false,
            serving: None,
        });
        let served = Endpoint::<ProcessHost>::from_receiver(&self.s.ipc, launched.from_child)
            .serve_bound(
                self.seq,
                PeerBinding::new(peer_id, PeerAttest::Os(launched.peer)),
                move |m, from| m.dispatch(&mut side, from),
                on_end,
            );
        let mut st = self.lock();
        let Ok(serving) = served else {
            // Dropping the child's ends closes its channels; it ends.
            *Self::slot(&mut st, kind) = None;
            drop(st);
            self.progress.launch_failures.fetch_add(1, Ordering::SeqCst);
            return Err(LaunchError::Failed);
        };
        if let Some(child) = Self::slot(&mut st, kind).as_mut() {
            child.serving = Some(serving);
        }
        drop(st);
        self.progress.launches.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// A child's channel ended: relaunch it within the budget, unless
    /// the browser is halted or the end was a violation by its peer
    /// (IPC-R5: a violating child is not given a new process).
    fn ended(self: &Arc<Self>, kind: ChildKind, why: &EndReason<ProcessError>) {
        let relaunch = {
            let mut st = self.lock();
            *Self::slot(&mut st, kind) = None;
            if kind == ChildKind::Renderer {
                // Its asks die with it: a relaunched renderer starts over.
                st.pending.clear();
                st.inflight.clear();
            } else {
                // The network's unanswered work goes back in line.
                let mut back = std::mem::take(&mut st.inflight);
                back.append(&mut st.pending);
                st.pending = back;
            }
            let halted = st.halted;
            let budget = if kind == ChildKind::Renderer {
                &mut st.budget_renderer
            } else {
                &mut st.budget_network
            };
            let ok = !halted && matches!(why, EndReason::PeerClosed) && *budget > 0;
            if ok {
                *budget = budget.saturating_sub(1);
            }
            ok
        };
        if !relaunch {
            return;
        }
        let inner = Arc::clone(self);
        // Relaunch on the browser's own sequence, never inside a delivery.
        let _ = self.s.exec.post(
            self.seq,
            Task::new(move || {
                if inner.launch(kind).is_ok() {
                    inner.progress.relaunches.fetch_add(1, Ordering::SeqCst);
                }
            }),
            None,
            hg_types::Duration::ZERO,
        );
    }

    /// Sends the waiting asks to a ready network, in order.
    fn flush(st: &mut State, progress: &Progress) {
        let Some(net) = st.network.as_ref().filter(|n| n.ready) else {
            return;
        };
        let mut sent = Vec::new();
        for (id, payload) in st.pending.drain(..) {
            // A failed send means the network is gone; its closing puts
            // these back in line.
            let _ = net.control.work(id, payload.clone());
            progress.routed.fetch_add(1, Ordering::SeqCst);
            sent.push((id, payload));
        }
        st.inflight.append(&mut sent);
    }
}

struct HostSide {
    inner: Arc<Inner>,
    kind: ChildKind,
}

impl ProcessHostHandler for HostSide {
    fn ready(
        &mut self,
        _from: &CallerIdentity,
        kind: ProcessKind,
        build: BuildId,
        _sandbox: SandboxReport,
    ) -> Result<(), ProcessError> {
        if kind != wire_kind(self.kind) {
            return Err(ProcessError::Unsupported);
        }
        let mut st = self.inner.lock();
        let Some(child) = Inner::slot(&mut st, self.kind).as_mut() else {
            return Err(ProcessError::ShuttingDown);
        };
        if build.id != hg_types::BUILD_ID {
            // DU-3: tell it why, then stop launching anything.
            let _ = child.control.exit(ExitReason::BuildMismatch {
                expected: BuildId {
                    id: hg_types::BUILD_ID,
                },
            });
            st.halted = true;
            drop(st);
            self.inner.launcher.restart_required();
            self.inner
                .progress
                .mismatches
                .fetch_add(1, Ordering::SeqCst);
            return Err(ProcessError::ShuttingDown);
        }
        if child.ready {
            return Err(ProcessError::Unsupported);
        }
        child.ready = true;
        // A child gone before `start` reaches it is not at fault: its own
        // channel's closing ends it and drives the relaunch.
        let _ = child.control.start(config_for(self.kind));
        Inner::flush(&mut st, &self.inner.progress);
        Ok(())
    }

    fn pong(&mut self, _from: &CallerIdentity, _nonce: u64) -> Result<(), ProcessError> {
        Ok(())
    }

    fn ask(
        &mut self,
        _from: &CallerIdentity,
        id: u64,
        payload: BoundedBytes<256>,
    ) -> Result<(), ProcessError> {
        if self.kind != ChildKind::Renderer {
            return Err(ProcessError::Unsupported);
        }
        let mut st = self.inner.lock();
        if !st.renderer.as_ref().is_some_and(|r| r.ready) {
            // No asks before `ready` passed the build check.
            return Err(ProcessError::Unsupported);
        }
        if st.pending.len().saturating_add(st.inflight.len()) >= MAX_PENDING {
            self.inner
                .progress
                .refused_asks
                .fetch_add(1, Ordering::SeqCst);
            return Err(ProcessError::Unsupported);
        }
        st.pending.push((id, payload));
        let waiting =
            u64::try_from(st.pending.len().saturating_add(st.inflight.len())).unwrap_or(u64::MAX);
        self.inner
            .progress
            .max_pending
            .fetch_max(waiting, Ordering::SeqCst);
        Inner::flush(&mut st, &self.inner.progress);
        Ok(())
    }

    fn answer(
        &mut self,
        _from: &CallerIdentity,
        id: u64,
        payload: BoundedBytes<256>,
    ) -> Result<(), ProcessError> {
        if self.kind == ChildKind::Renderer {
            return Err(ProcessError::Unsupported);
        }
        let mut st = self.inner.lock();
        let Some(pos) = st.inflight.iter().position(|(i, _)| *i == id) else {
            // Work the browser did not send, or already answered.
            return Err(ProcessError::Unsupported);
        };
        st.inflight.remove(pos);
        if let Some(r) = st.renderer.as_ref().filter(|r| r.ready)
            && r.control.answered(id, payload).is_ok()
        {
            self.inner.progress.returned.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }
}

/// Runs the browser on `s`: launches a renderer and a network partition
/// service and routes between them.
///
/// # Errors
/// [`LaunchError::Unsupported`] when `s` has no launcher (only a broker
/// does), or the first launch's error.
pub fn run(s: &Substrate, config: BrowserConfig) -> Result<Running, LaunchError> {
    let launcher = s.launcher.clone().ok_or(LaunchError::Unsupported)?;
    let inner = Arc::new(Inner {
        s: s.clone(),
        launcher,
        seq: s.exec.sequence("host", Priority::UserVisible),
        progress: config.progress,
        state: Mutex::new(State {
            budget_renderer: config.restart_budget,
            budget_network: config.restart_budget,
            ..State::default()
        }),
    });
    inner.launch(ChildKind::NetworkPartition)?;
    inner.launch(ChildKind::Renderer)?;
    Ok(Running { inner })
}
