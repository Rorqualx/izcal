//! `hg-renderer`: the renderer process's logic (Stage 1 design 8.1 M1).
//!
//! Layer L4, tier R. The logic lives in this library, whose entry
//! function takes a [`Substrate`]; `hg-renderer-main` calls it with the
//! OS substrate, and the simulation binary with a simulated one. At M1
//! the renderer is a skeleton: it reports `ready` on its bootstrap
//! channel, waits for `start`, then asks the browser `rounds` times for
//! something only the network service can answer, one at a time
//! (the M1 exit's IPC round trip through the browser).
//!
//! Identity (design review M8): the renderer trusts its bootstrap
//! channel because it came at birth and only the broker holds the other
//! end; it never attests a peer by pid.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use hg_ipc::{BoundedBytes, CallerIdentity, Endpoint, PeerBinding, PeerId, Remote, Serving};
use hg_ipc_gen::process::{
    BuildId, ChildConfig, ExitReason, ProcessControl, ProcessControlHandler, ProcessError,
    ProcessHost, ProcessHostRemote, ProcessKind, SandboxReport,
};
use hg_substrate::launch::SandboxReached;
use hg_substrate::{Priority, Substrate};

/// Round trips a launched renderer (`hg-renderer-main`) makes after
/// `start`: the M1 exit's IPC round trip on the OS.
pub const LAUNCHED_ROUNDS: u64 = 8;

/// What the renderer has done, for its owner to watch (tests, DST and
/// FP-27's timing hooks).
#[derive(Debug, Default)]
pub struct Progress {
    /// Answers received.
    pub answered: AtomicU64,
    /// Whether `start` arrived.
    pub started: AtomicU64,
    /// Incarnations that received the answer to every one of their asks.
    pub completed: AtomicU64,
    /// Answers refused: out of order, or not the answer to the ask they
    /// name (a misrouted or duplicated answer).
    pub refused: AtomicU64,
}

/// How a renderer runs.
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Round trips to make after `start`.
    pub rounds: u64,
    /// The sandbox the process reached, reported in `ready`.
    pub sandbox: SandboxReport,
    /// Progress counters.
    pub progress: Arc<Progress>,
}

impl RendererConfig {
    /// The configuration of a launched renderer (`hg-renderer-main`):
    /// `rounds` round trips, reporting what its sandbox probes found.
    #[must_use]
    pub fn launched(rounds: u64, sandbox: &SandboxReached) -> Self {
        Self {
            rounds,
            sandbox: hg_ipc_gen::sandbox::wire_report(sandbox),
            progress: Arc::default(),
        }
    }
}

/// A running renderer: keep it for as long as the process runs.
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
    /// Whether the bootstrap channel has ended. It is set in the
    /// channel's delivery task, on the sequence this process serves it
    /// on, so a `main` that runs the sequences checks it after each
    /// task (FP-32: `main` runs them instead of parking).
    pub fn ended(&self) -> bool {
        *self
            .ended
            .done
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

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
    rounds: u64,
    progress: Arc<Progress>,
    /// The next answer this incarnation expects.
    next: u64,
    started: bool,
}

/// The payload of ask `n`.
fn ask_payload(n: u64) -> BoundedBytes<256> {
    BoundedBytes::try_new(n.to_le_bytes().to_vec()).unwrap_or_default()
}

/// What the network answers to ask `n`: `N` and the ask's bytes reversed
/// (`hg-net-service`'s answer).
fn answer_payload(n: u64) -> Vec<u8> {
    let mut v = vec![b'N'];
    v.extend(n.to_le_bytes().iter().rev());
    v
}

impl Side {
    fn ask(&self, n: u64) -> Result<(), ProcessError> {
        self.host
            .ask(n, ask_payload(n))
            .map_err(|_e| ProcessError::ShuttingDown)
    }
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
        if !matches!(config, ChildConfig::Renderer { .. }) {
            return Err(ProcessError::Unsupported);
        }
        if self.started {
            // `start` comes once.
            return Err(ProcessError::Unsupported);
        }
        self.started = true;
        self.progress.started.fetch_add(1, Ordering::SeqCst);
        if self.rounds == 0 {
            return Ok(());
        }
        self.ask(0)
    }

    fn work(
        &mut self,
        _from: &CallerIdentity,
        _id: u64,
        _payload: BoundedBytes<256>,
    ) -> Result<(), ProcessError> {
        // A renderer does no work for others.
        Err(ProcessError::Unsupported)
    }

    fn answered(
        &mut self,
        _from: &CallerIdentity,
        id: u64,
        payload: BoundedBytes<256>,
    ) -> Result<(), ProcessError> {
        if id != self.next || payload.as_slice() != answer_payload(id).as_slice() {
            // Answers come in order, one per ask, each the answer to its
            // own ask.
            self.progress.refused.fetch_add(1, Ordering::SeqCst);
            return Err(ProcessError::Unsupported);
        }
        self.next = id.saturating_add(1);
        self.progress.answered.fetch_add(1, Ordering::SeqCst);
        let next = id.saturating_add(1);
        if next < self.rounds {
            self.ask(next)?;
        } else {
            self.progress.completed.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }
}

/// Runs the renderer on `s`. `None` when the process has no bootstrap
/// channels (it was not launched by a broker) or cannot serve them.
#[must_use]
pub fn run(s: &Substrate, config: RendererConfig) -> Option<Running> {
    let boot = s.ipc.bootstrap()?;
    let host = Remote::<ProcessHost>::from_sender(&s.ipc, boot.to_parent);
    host.ready(
        ProcessKind::Renderer,
        BuildId {
            id: hg_types::BUILD_ID,
        },
        config.sandbox,
    )
    .ok()?;
    let mut side = Side {
        host,
        rounds: config.rounds,
        progress: config.progress,
        next: 0,
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
