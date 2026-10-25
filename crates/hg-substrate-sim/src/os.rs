//! The scripted OS model (OS integration 6.1, OI-D26; Appendix C item 1):
//! the simulator's `OsPort`, given to the one process a test names as the
//! broker ([`crate::World::give_os`]).
//!
//! A script is the OS: input frames it delivers at virtual times after
//! the broker starts listening ("1,000 launches at once", "sleep in
//! 3 s"), and a function that answers each command with more input
//! frames. Frames are opaque bytes here (`hg-os-types` encodes them,
//! above this layer). Every input delivered and every command taken goes
//! into the trace, so the stream is part of the run's recording and its
//! hash. The recorded `OsPort` follows at OI-M1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use hg_substrate::ipc::MAX_MESSAGE_BYTES;
use hg_substrate::os::{OsCallId, OsDelivery, OsError, OsListening, OsPort, OsSink};
use hg_substrate::{Executor, SequenceHandle, Task};
use hg_types::Duration;

use crate::world::{ProcessId, Shared};

/// Input frames and when each arrives, after the previous step (listening,
/// or the command it answers).
pub type OsFrames = Vec<(Duration, Vec<u8>)>;

/// How the scripted OS answers one command: the input frames that answer
/// it, each with its delay after the command.
pub type OsAnswer = Box<dyn FnMut(OsCallId, &[u8]) -> OsFrames + Send>;

/// A scripted OS.
pub struct OsScript {
    /// Inputs delivered once the broker listens, each at its delay after
    /// that moment.
    pub inputs: OsFrames,
    /// Answers each command.
    pub answer: OsAnswer,
}

impl std::fmt::Debug for OsScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OsScript")
            .field("inputs", &self.inputs.len())
            .finish_non_exhaustive()
    }
}

/// One process's OS model.
pub(crate) struct OsModel {
    /// Not yet delivered: the script's inputs, then answers that came
    /// before a listener.
    pending: OsFrames,
    /// The answer function; taken out while it runs, so it never runs
    /// with the world locked.
    answer: Option<OsAnswer>,
    /// The next command's id.
    next_call: u64,
    /// The listener: its incarnation, sequence, sink and liveness.
    listener: Option<Listener>,
}

struct Listener {
    generation: u32,
    seq: SequenceHandle,
    sink: Arc<Mutex<OsSink>>,
    live: Arc<AtomicBool>,
}

impl OsModel {
    pub(crate) fn new(script: OsScript) -> Self {
        Self {
            pending: script.inputs,
            answer: Some(script.answer),
            next_call: 0,
            listener: None,
        }
    }
}

/// The broker's simulated `OsPort`.
pub(crate) struct SimOsPort {
    pub(crate) shared: Arc<Shared>,
    pub(crate) pid: ProcessId,
    pub(crate) generation: u32,
    pub(crate) exec: Arc<dyn Executor>,
}

impl SimOsPort {
    /// Posts `frames` to the listener as timed tasks on its sequence, in
    /// order. Frames with equal delays keep their order (the sequence
    /// keeps post order).
    fn deliver(&self, l: &Listener, frames: OsFrames) {
        for (delay, frame) in frames {
            let (shared, sink, live) = (
                Arc::clone(&self.shared),
                Arc::clone(&l.sink),
                Arc::clone(&l.live),
            );
            let task = Task::new(move || {
                if !live.load(Ordering::Acquire) {
                    return;
                }
                shared
                    .lock()
                    .trace
                    .record(format_args!("os input {} bytes", frame.len()));
                sink.lock().unwrap_or_else(PoisonError::into_inner)(OsDelivery::Frame(frame));
            });
            let posted = self.exec.post(l.seq, task, Some(delay), Duration::ZERO);
            if posted.is_err() {
                self.shared
                    .lock()
                    .trace
                    .record(format_args!("os input refused by the sequence"));
            }
        }
    }

    /// The live listener's handles, if this incarnation has one.
    fn live_listener(&self, m: &OsModel) -> Option<Listener> {
        m.listener
            .as_ref()
            .filter(|l| l.generation == self.generation && l.live.load(Ordering::Acquire))
            .map(|l| Listener {
                generation: l.generation,
                seq: l.seq,
                sink: Arc::clone(&l.sink),
                live: Arc::clone(&l.live),
            })
    }
}

impl OsPort for SimOsPort {
    fn command(&self, frame: Vec<u8>) -> Result<OsCallId, OsError> {
        if frame.len() > MAX_MESSAGE_BYTES {
            return Err(OsError::TooLarge);
        }
        let mut st = self.shared.lock();
        let Some(m) = st.os.get_mut(&self.pid) else {
            return Err(OsError::Unsupported);
        };
        let id = OsCallId::new(m.next_call);
        m.next_call = m.next_call.saturating_add(1);
        let answer = m.answer.take();
        st.trace.record(format_args!(
            "os command #{} {} bytes",
            id.raw(),
            frame.len()
        ));
        drop(st);
        let Some(mut answer) = answer else {
            // Re-entered from inside its own answer: not a thing an OS does.
            return Err(OsError::Refused);
        };
        let frames = answer(id, &frame);
        let mut st = self.shared.lock();
        let Some(m) = st.os.get_mut(&self.pid) else {
            return Err(OsError::Closed);
        };
        m.answer = Some(answer);
        match self.live_listener(m) {
            Some(l) => {
                drop(st);
                self.deliver(&l, frames);
            }
            None => m.pending.extend(frames),
        }
        Ok(id)
    }

    fn listen(&self, seq: SequenceHandle, sink: OsSink) -> Result<OsListening, OsError> {
        let mut st = self.shared.lock();
        let Some(m) = st.os.get_mut(&self.pid) else {
            return Err(OsError::Unsupported);
        };
        if self.live_listener(m).is_some() {
            return Err(OsError::Refused);
        }
        let live = Arc::new(AtomicBool::new(true));
        let l = Listener {
            generation: self.generation,
            seq,
            sink: Arc::new(Mutex::new(sink)),
            live: Arc::clone(&live),
        };
        let frames = std::mem::take(&mut m.pending);
        m.listener = Some(Listener {
            generation: l.generation,
            seq,
            sink: Arc::clone(&l.sink),
            live: Arc::clone(&live),
        });
        st.trace
            .record(format_args!("os listen {} pending", frames.len()));
        drop(st);
        self.deliver(&l, frames);
        Ok(OsListening::new(Box::new(Guard(live))))
    }
}

/// Stops delivery when dropped.
struct Guard(Arc<AtomicBool>);

impl Drop for Guard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
