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
//! - **Ready deadline (E-R3).** A child that has not reported `ready`
//!   within [`READY_DEADLINE`] of its launch, on the substrate's clock, is
//!   killed and launched again within the kind's budget. So is a launch
//!   that fails.
//! - **Routing.** The browser names each piece of work it sends the
//!   network with its own id, never the renderer's, and remembers which
//!   renderer launch asked: an answer reaches only that renderer, and an
//!   answer for a renderer that has ended is dropped.
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
use hg_ipc_gen::sandbox::launch_tier;
use hg_substrate::ipc::{OsPeer, PeerAttest};
use hg_substrate::launch::{ChildKind, LaunchError, Launcher, SandboxTier};
use hg_substrate::{Priority, SequenceHandle, Substrate, Task};
use hg_types::{Duration, Instant};

/// How long a launched child has to report `ready` before it is killed
/// and the spawn counts as a failure (phase E rule E-R3: 5 s†). Measured
/// on the substrate's clock, so the simulator runs it in virtual time.
pub const READY_DEADLINE: Duration = Duration::from_secs(5);

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
    /// Children killed because their sandbox report named another tier
    /// than their launcher applied (design review M3).
    pub sandbox_mismatches: AtomicU64,
    /// Children killed for missing their `ready` deadline.
    pub missed_deadlines: AtomicU64,
    /// Kinds the browser stopped relaunching: their budget was spent.
    pub gave_up: AtomicU64,
    /// Spawn to ready of every child that reported `ready` with this
    /// build, by the substrate's clock (FP-27's hook, E-R12).
    pub spawn_to_ready: Mutex<Vec<(ChildKind, Duration)>>,
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
    /// The sandbox tier its launcher applied (review M3).
    sandbox: SandboxTier,
    ready: bool,
    /// Which launch this is: never repeats in the browser's life. A
    /// message, an end or a deadline of an earlier launch of the kind
    /// carries an older serial and changes nothing.
    serial: u64,
    /// The identity the launch returned, to kill it by.
    peer: OsPeer,
    /// Substrate time when the launch was requested (FP-27).
    requested_at: Instant,
    /// Set once serving started; the slot is filled before, so a `ready`
    /// on the browser's sequence always finds its child.
    serving: Option<Serving>,
}

/// A renderer's ask, waiting for a ready network.
struct Ask {
    /// The launch serial of the renderer that asked.
    renderer: u64,
    /// The renderer's own id for it.
    id: u64,
    payload: BoundedBytes<256>,
}

/// Work sent to the network and not yet answered.
struct Work {
    /// The browser's id for it: unique in the browser's life, so an
    /// answer can never be taken for another renderer's ask.
    work: u64,
    ask: Ask,
}

#[derive(Default)]
struct State {
    renderer: Option<Child>,
    network: Option<Child>,
    /// Asks waiting for a ready network, in order.
    pending: Vec<Ask>,
    /// Work sent to the network and not yet answered, in order. If the
    /// network ends, the current renderer's go back to the front of
    /// `pending`, so the relaunched network answers them and the
    /// renderer is not left waiting. Work of a renderer that ended stays
    /// until the network answers it, and the answer is dropped.
    inflight: Vec<Work>,
    budget_renderer: u32,
    budget_network: u32,
    next_serial: u64,
    next_work: u64,
    /// Restart-required: set once, never cleared (DU-3).
    halted: bool,
}

impl State {
    fn current(&self, kind: ChildKind) -> Option<u64> {
        let slot = if kind == ChildKind::Renderer {
            &self.renderer
        } else {
            &self.network
        };
        slot.as_ref().map(|c| c.serial)
    }
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

    /// Launches `kind`, serves its bootstrap channel and arms its `ready`
    /// deadline.
    fn launch(self: &Arc<Self>, kind: ChildKind) -> Result<(), LaunchError> {
        let serial = {
            let mut st = self.lock();
            if st.halted {
                return Err(LaunchError::RestartRequired);
            }
            st.next_serial = st.next_serial.saturating_add(1);
            st.next_serial
        };
        let requested_at = self.s.clock.monotonic();
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
            serial,
        };
        let on_end = {
            let inner = Arc::clone(self);
            move |why: EndReason<ProcessError>| inner.ended(kind, serial, &why)
        };
        let peer_id = PeerId::new(if kind == ChildKind::Renderer { 1 } else { 2 }, 0);
        *Self::slot(&mut self.lock(), kind) = Some(Child {
            control,
            sandbox: launched.sandbox,
            ready: false,
            serial,
            peer: launched.peer,
            requested_at,
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
            self.launcher.kill(launched.peer);
            self.progress.launch_failures.fetch_add(1, Ordering::SeqCst);
            return Err(LaunchError::Failed);
        };
        if let Some(child) = Self::slot(&mut st, kind).as_mut() {
            child.serving = Some(serving);
        }
        drop(st);
        // E-R3: `ready` within the deadline, by the substrate's clock, or
        // the child is killed and the spawn is a failure.
        let inner = Arc::clone(self);
        let _ = self.s.exec.post(
            self.seq,
            Task::new(move || inner.deadline(kind, serial)),
            Some(READY_DEADLINE),
            Duration::ZERO,
        );
        self.progress.launches.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Launch `serial` of `kind`'s deadline: if it is still the current
    /// child and not ready, kill it and launch again within the budget.
    fn deadline(self: &Arc<Self>, kind: ChildKind, serial: u64) {
        let gone = {
            let mut st = self.lock();
            let slot = Self::slot(&mut st, kind);
            if slot
                .as_ref()
                .is_some_and(|c| c.serial == serial && !c.ready)
            {
                slot.take()
            } else {
                None
            }
        };
        let Some(child) = gone else { return };
        self.progress
            .missed_deadlines
            .fetch_add(1, Ordering::SeqCst);
        let peer = child.peer;
        // Its channels close with it.
        drop(child);
        self.launcher.kill(peer);
        self.forget(kind);
        self.relaunch(kind);
    }

    /// What a child that ended leaves behind. The renderer's waiting asks
    /// die with it (a relaunched renderer starts over); its work already
    /// at the network stays until answered. The network's unanswered work
    /// for the current renderer goes back in line; work for a renderer
    /// that ended is dropped.
    fn forget(&self, kind: ChildKind) {
        let mut st = self.lock();
        if kind == ChildKind::Renderer {
            st.pending.clear();
        } else {
            let current = st.current(ChildKind::Renderer);
            let mut back: Vec<Ask> = std::mem::take(&mut st.inflight)
                .into_iter()
                .map(|w| w.ask)
                .filter(|a| Some(a.renderer) == current)
                .collect();
            back.append(&mut st.pending);
            st.pending = back;
        }
    }

    /// Launches `kind` again on the browser's own sequence, never inside
    /// a delivery, within its budget. A launch that fails (a refused
    /// launch, a spawner that died) is tried again, also within the
    /// budget; a refusal of another kind (restart-required, below the
    /// floor) ends the attempts.
    fn relaunch(self: &Arc<Self>, kind: ChildKind) {
        let go = {
            let mut st = self.lock();
            let halted = st.halted;
            let budget = if kind == ChildKind::Renderer {
                &mut st.budget_renderer
            } else {
                &mut st.budget_network
            };
            if halted {
                false
            } else if *budget == 0 {
                self.progress.gave_up.fetch_add(1, Ordering::SeqCst);
                false
            } else {
                *budget = budget.saturating_sub(1);
                true
            }
        };
        if !go {
            return;
        }
        let inner = Arc::clone(self);
        let _ = self.s.exec.post(
            self.seq,
            Task::new(move || match inner.launch(kind) {
                Ok(()) => {
                    inner.progress.relaunches.fetch_add(1, Ordering::SeqCst);
                }
                Err(LaunchError::Failed) => inner.relaunch(kind),
                Err(_) => {}
            }),
            None,
            Duration::ZERO,
        );
    }

    /// Launch `serial` of `kind`'s channel ended: relaunch it within the
    /// budget, unless the browser is halted, a later launch has already
    /// replaced it, or the end was a violation by its peer (IPC-R5: a
    /// violating child is not given a new process).
    fn ended(self: &Arc<Self>, kind: ChildKind, serial: u64, why: &EndReason<ProcessError>) {
        {
            let mut st = self.lock();
            let slot = Self::slot(&mut st, kind);
            if slot.as_ref().is_none_or(|c| c.serial != serial) {
                return;
            }
            *slot = None;
        }
        self.forget(kind);
        if matches!(why, EndReason::PeerClosed) {
            self.relaunch(kind);
        }
    }

    /// Sends the waiting asks to a ready network, in order.
    fn flush(st: &mut State, progress: &Progress) {
        if !st.network.as_ref().is_some_and(|n| n.ready) {
            return;
        }
        let asks = std::mem::take(&mut st.pending);
        for ask in asks {
            st.next_work = st.next_work.saturating_add(1);
            let work = st.next_work;
            // A failed send means the network is gone; its closing puts
            // these back in line.
            if let Some(net) = st.network.as_ref() {
                let _ = net.control.work(work, ask.payload.clone());
            }
            progress.routed.fetch_add(1, Ordering::SeqCst);
            st.inflight.push(Work { work, ask });
        }
    }
}

struct HostSide {
    inner: Arc<Inner>,
    kind: ChildKind,
    serial: u64,
}

impl HostSide {
    /// This channel's child, if it is still the current one of its kind.
    fn mine<'a>(&self, st: &'a mut State) -> Option<&'a mut Child> {
        Inner::slot(st, self.kind)
            .as_mut()
            .filter(|c| c.serial == self.serial)
    }
}

impl ProcessHostHandler for HostSide {
    fn ready(
        &mut self,
        _from: &CallerIdentity,
        kind: ProcessKind,
        build: BuildId,
        sandbox: SandboxReport,
    ) -> Result<(), ProcessError> {
        if kind != wire_kind(self.kind) {
            return Err(ProcessError::Unsupported);
        }
        let now = self.inner.s.clock.monotonic();
        let mut st = self.inner.lock();
        let Some(child) = self.mine(&mut st) else {
            return Err(ProcessError::ShuttingDown);
        };
        // Review M3: the child's own report is a cross-check of the tier
        // its launcher applied; any other tier ends it, and it is not
        // launched again for it (the handler error ends the channel as
        // the child's fault).
        if launch_tier(sandbox.tier) != child.sandbox {
            let peer = child.peer;
            drop(st);
            self.inner.launcher.kill(peer);
            self.inner
                .progress
                .sandbox_mismatches
                .fetch_add(1, Ordering::SeqCst);
            return Err(ProcessError::Unsupported);
        }
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
        // FP-27: spawn to ready, by the substrate's clock.
        let took = now.saturating_duration_since(child.requested_at);
        self.inner
            .progress
            .spawn_to_ready
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((self.kind, took));
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
        if !self.mine(&mut st).is_some_and(|r| r.ready) {
            // No asks before `ready` passed the build check.
            return Err(ProcessError::Unsupported);
        }
        let serial = self.serial;
        let waiting = st
            .pending
            .iter()
            .filter(|a| a.renderer == serial)
            .count()
            .saturating_add(
                st.inflight
                    .iter()
                    .filter(|w| w.ask.renderer == serial)
                    .count(),
            );
        if waiting >= MAX_PENDING {
            self.inner
                .progress
                .refused_asks
                .fetch_add(1, Ordering::SeqCst);
            return Err(ProcessError::Unsupported);
        }
        st.pending.push(Ask {
            renderer: serial,
            id,
            payload,
        });
        let waiting = u64::try_from(waiting.saturating_add(1)).unwrap_or(u64::MAX);
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
        if self.mine(&mut st).is_none() {
            return Err(ProcessError::ShuttingDown);
        }
        let Some(pos) = st.inflight.iter().position(|w| w.work == id) else {
            // Work the browser did not send, or already answered.
            return Err(ProcessError::Unsupported);
        };
        let Work { ask, .. } = st.inflight.remove(pos);
        // The answer goes to the renderer that asked, if it is still the
        // current one; a dead renderer's answer is dropped.
        if let Some(r) = st
            .renderer
            .as_ref()
            .filter(|r| r.ready && r.serial == ask.renderer)
            && r.control.answered(ask.id, payload).is_ok()
        {
            self.inner.progress.returned.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }
}

/// Runs the browser on `s`: launches a network partition service and a
/// renderer and routes between them. A first launch that fails is tried
/// again within the kind's budget.
///
/// # Errors
/// [`LaunchError::Unsupported`] when `s` has no launcher (only a broker
/// does), or a first launch's refusal other than
/// [`LaunchError::Failed`].
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
    for kind in [ChildKind::NetworkPartition, ChildKind::Renderer] {
        match inner.launch(kind) {
            Ok(()) => {}
            Err(LaunchError::Failed) => inner.relaunch(kind),
            Err(e) => return Err(e),
        }
    }
    Ok(Running { inner })
}
