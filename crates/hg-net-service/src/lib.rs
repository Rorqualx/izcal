//! `hg-net-service`: the network service process's logic (Stage 1 design 8.1 M1).
//!
//! Layer L5, tier P (design review M5). One library serves the three
//! configurations of networking NW-D3 (system, container, partition);
//! `hg-net-main` calls it with the OS substrate, and the simulation
//! binary with a simulated one. At M1 it is a skeleton: it reports
//! `ready`, waits for `start`, and answers each `work` the browser routes
//! to it. Sockets, DNS and the protocol stack are NW-M0's own plan.
//!
//! Identity (design review M8): the service trusts its bootstrap
//! channel because it came at birth and only the broker holds the other
//! end; it never attests a peer by pid.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use hg_ipc::{BoundedBytes, CallerIdentity, Endpoint, PeerBinding, PeerId, Remote, Serving};
use hg_ipc_gen::process::{
    BuildId, ChildConfig, ExitReason, ProcessControl, ProcessControlHandler, ProcessError,
    ProcessHost, ProcessHostRemote, ProcessKind, SandboxReport,
};
use hg_substrate::launch::{ChildKind, SandboxReached};
use hg_substrate::{Priority, Substrate};

/// What the service has done, for its owner to watch.
#[derive(Debug, Default)]
pub struct Progress {
    /// Work answered.
    pub answered: AtomicU64,
    /// Whether `start` arrived.
    pub started: AtomicU64,
}

/// How a network service runs.
#[derive(Debug, Clone)]
pub struct NetConfig {
    /// Which NW-D3 configuration this process is.
    pub kind: ProcessKind,
    /// The sandbox the process reached, reported in `ready`.
    pub sandbox: SandboxReport,
    /// Progress counters.
    pub progress: Arc<Progress>,
}

impl NetConfig {
    /// The configuration of a launched network service (`hg-net-main`)
    /// of `kind`, reporting what its sandbox probes found. `None` for the
    /// renderer kind.
    #[must_use]
    pub fn launched(kind: ChildKind, sandbox: &SandboxReached) -> Option<Self> {
        let kind = match kind {
            ChildKind::Renderer => return None,
            ChildKind::NetworkSystem => ProcessKind::NetworkSystem,
            ChildKind::NetworkContainer => ProcessKind::NetworkContainer,
            ChildKind::NetworkPartition => ProcessKind::NetworkPartition,
        };
        Some(Self {
            kind,
            sandbox: hg_ipc_gen::sandbox::wire_report(sandbox),
            progress: Arc::default(),
        })
    }
}

/// A running service: keep it for as long as the process runs.
#[derive(Debug)]
pub struct Running {
    _serving: Serving,
    ended: Arc<Ended>,
}

/// Set once the bootstrap channel has ended.
#[derive(Debug, Default)]
struct Ended {
    done: std::sync::Mutex<bool>,
    cv: std::sync::Condvar,
}

impl Ended {
    fn set(&self) {
        *self
            .done
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
        self.cv.notify_all();
    }
}

impl Running {
    /// Blocks until the bootstrap channel ends (the browser closed it,
    /// told this process to exit, or broke the protocol): then the
    /// process has nothing left to do. For `main`.
    pub fn wait(&self) {
        let mut done = self
            .ended
            .done
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while !*done {
            done = self
                .ended
                .cv
                .wait(done)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }
}

struct Side {
    host: Remote<ProcessHost>,
    kind: ProcessKind,
    progress: Arc<Progress>,
    /// This incarnation's `start` arrived (the counters may be shared
    /// across incarnations).
    started: bool,
}

/// The answer to a work payload: the bytes reversed, marked as the
/// network's (a stand-in for a fetch until NW-M0).
fn answer_for(payload: &BoundedBytes<256>) -> BoundedBytes<256> {
    let mut v: Vec<u8> = payload.as_slice().iter().rev().copied().collect();
    v.truncate(255);
    v.insert(0, b'N');
    BoundedBytes::try_new(v).unwrap_or_default()
}

impl ProcessControlHandler for Side {
    fn ping(&mut self, _from: &CallerIdentity, nonce: u64) -> Result<(), ProcessError> {
        // A closed browser is fine: this process is ending too.
        let _ = self.host.pong(nonce);
        Ok(())
    }

    fn exit(&mut self, _from: &CallerIdentity, _reason: ExitReason) -> Result<(), ProcessError> {
        Err(ProcessError::ShuttingDown)
    }

    fn start(&mut self, _from: &CallerIdentity, config: ChildConfig) -> Result<(), ProcessError> {
        let fits = matches!(
            (self.kind, config),
            (ProcessKind::NetworkSystem, ChildConfig::NetworkSystem)
                | (
                    ProcessKind::NetworkContainer,
                    ChildConfig::NetworkContainer { .. }
                )
                | (
                    ProcessKind::NetworkPartition,
                    ChildConfig::NetworkPartition { .. }
                )
        );
        if !fits || self.started {
            // A configuration for another kind, or a second `start`.
            return Err(ProcessError::Unsupported);
        }
        self.started = true;
        self.progress.started.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn work(
        &mut self,
        _from: &CallerIdentity,
        id: u64,
        payload: BoundedBytes<256>,
    ) -> Result<(), ProcessError> {
        if !self.started {
            return Err(ProcessError::Unsupported);
        }
        self.progress.answered.fetch_add(1, Ordering::SeqCst);
        self.host
            .answer(id, answer_for(&payload))
            .map_err(|_e| ProcessError::ShuttingDown)
    }

    fn answered(
        &mut self,
        _from: &CallerIdentity,
        _id: u64,
        _payload: BoundedBytes<256>,
    ) -> Result<(), ProcessError> {
        // The network service asks nobody.
        Err(ProcessError::Unsupported)
    }
}

/// Runs the network service on `s`. `None` when the process has no bootstrap
/// channels (it was not launched by a broker) or cannot serve them.
#[must_use]
pub fn run(s: &Substrate, config: NetConfig) -> Option<Running> {
    let boot = s.ipc.bootstrap()?;
    let host = Remote::<ProcessHost>::from_sender(&s.ipc, boot.to_parent);
    if config.kind == ProcessKind::Renderer {
        return None;
    }
    host.ready(
        config.kind,
        BuildId {
            id: hg_types::BUILD_ID,
        },
        config.sandbox,
    )
    .ok()?;
    let mut side = Side {
        host,
        kind: config.kind,
        progress: config.progress,
        started: false,
    };
    let seq = s.exec.sequence("control", Priority::UserVisible);
    let ended = Arc::new(Ended::default());
    let on_end = Arc::clone(&ended);
    let serving = Endpoint::<ProcessControl>::from_receiver(&s.ipc, boot.from_parent)
        .serve_bound(
            seq,
            PeerBinding::new(PeerId::new(0, 0), boot.parent),
            move |m, from| m.dispatch(&mut side, from),
            move |_why| on_end.set(),
        )
        .ok()?;
    Some(Running {
        _serving: serving,
        ended,
    })
}
