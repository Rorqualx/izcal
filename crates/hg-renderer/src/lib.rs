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
use hg_substrate::{Priority, Substrate};

/// What the renderer has done, for its owner to watch (tests, DST and
/// FP-27's timing hooks).
#[derive(Debug, Default)]
pub struct Progress {
    /// Answers received.
    pub answered: AtomicU64,
    /// Whether `start` arrived.
    pub started: AtomicU64,
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

/// A running renderer: keep it for as long as the process runs.
#[derive(Debug)]
pub struct Running {
    _serving: Serving,
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
        _payload: BoundedBytes<256>,
    ) -> Result<(), ProcessError> {
        if id != self.next {
            // Answers come in order, one per ask.
            return Err(ProcessError::Unsupported);
        }
        self.next = id.saturating_add(1);
        self.progress.answered.fetch_add(1, Ordering::SeqCst);
        let next = id.saturating_add(1);
        if next < self.rounds {
            self.ask(next)?;
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
    let serving = Endpoint::<ProcessControl>::from_receiver(&s.ipc, boot.from_parent)
        .serve_bound(
            seq,
            PeerBinding::new(PeerId::new(0, 0), boot.parent),
            move |m, from| m.dispatch(&mut side, from),
            |_why| {},
        )
        .ok()?;
    Some(Running { _serving: serving })
}
