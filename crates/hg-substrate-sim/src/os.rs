//! The scripted OS model (OS integration 6.1, OI-D26; Appendix C item 1):
//! the simulator's `OsPort`, given to the one process a test names as the
//! broker ([`crate::World::spawn_broker`]).
//!
//! A script is the OS: input frames it delivers at virtual times after
//! the broker first listens ("1,000 launches at once", "sleep in
//! 3 s"), and a function that answers each command with more input
//! frames. Frames are opaque bytes here (`hg-os-types` encodes them,
//! above this layer). Every input delivered and every command taken goes
//! into the trace, so the stream is part of the run's recording and its
//! hash. The recorded `OsPort` follows at OI-M1.
//!
//! Inputs stay in the model until delivered: the listener's sequence
//! holds one delivery task at a time, which hands over the earliest due
//! input and posts the next (as the IPC drain task does). So a launch
//! storm larger than a sequence's capacity loses nothing and does not
//! crowd out the sequence's other work, and an input not yet delivered
//! when the listener goes (a broker crash, or a guard dropped) waits for
//! the next listener. The model's queue has no bound: the OS holds what
//! the broker has not read, and a dispatcher reads at its own pace.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError, Weak};

use hg_substrate::ipc::MAX_MESSAGE_BYTES;
use hg_substrate::os::{OsCallId, OsDelivery, OsError, OsListening, OsPort, OsSink};
use hg_substrate::{SequenceHandle, Task};
use hg_types::Duration;

use crate::world::{ProcessId, Shared, Slot, State};

/// Input frames and when each arrives, after the previous step (listening,
/// or the command it answers).
pub type OsFrames = Vec<(Duration, Vec<u8>)>;

/// How the scripted OS answers one command: the input frames that answer
/// it, each with its delay after the command.
pub type OsAnswer = Box<dyn FnMut(OsCallId, &[u8]) -> OsFrames + Send>;

/// A scripted OS.
pub struct OsScript {
    /// Inputs delivered once the broker first listens, each at its delay
    /// after that moment.
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
    /// The script's inputs, until the first listener fixes their times.
    unanchored: OsFrames,
    /// Inputs not yet delivered, by (due time in ns, arrival order).
    due: BTreeMap<(u64, u64), Vec<u8>>,
    /// The next arrival order.
    next_order: u64,
    /// The answer function; taken out while it runs, so it never runs
    /// with the world locked.
    answer: Option<OsAnswer>,
    /// The next command's id.
    next_call: u64,
    /// The next delivery task's token.
    next_token: u64,
    /// The listener: its incarnation, sequence, sink and liveness.
    listener: Option<Listener>,
}

struct Listener {
    generation: u32,
    seq: SequenceHandle,
    sink: Arc<Mutex<OsSink>>,
    live: Arc<AtomicBool>,
    /// The delivery task on `seq`, if one is posted: its token and the
    /// due time it was posted for.
    task: Option<(u64, u64)>,
}

impl OsModel {
    pub(crate) fn new(script: OsScript) -> Self {
        Self {
            unanchored: script.inputs,
            due: BTreeMap::new(),
            next_order: 0,
            answer: Some(script.answer),
            next_call: 0,
            next_token: 0,
            listener: None,
        }
    }

    /// Queues `frames`, each due at its delay after `now_ns`.
    fn anchor(&mut self, now_ns: u64, frames: OsFrames) {
        for (delay, frame) in frames {
            let delay = u64::try_from(delay.as_nanos()).unwrap_or(u64::MAX);
            let order = self.next_order;
            self.next_order = self.next_order.saturating_add(1);
            self.due
                .insert((now_ns.saturating_add(delay), order), frame);
        }
    }
}

/// Posts a delivery task for `pid`'s earliest due input, unless one is
/// already posted for that time or earlier, or no listener is live.
fn schedule(st: &mut State, pid: ProcessId) {
    let now = st.now.as_nanos();
    let shared = st.ipc_self.clone();
    let Some(m) = st.os.get_mut(&pid) else { return };
    let Some(&(due, _)) = m.due.keys().next() else {
        return;
    };
    let token = m.next_token;
    let Some(l) = m
        .listener
        .as_mut()
        .filter(|l| l.live.load(Ordering::Acquire))
    else {
        return;
    };
    if l.task.is_some_and(|(_, at)| at <= due) {
        return;
    }
    let (generation, seq) = (l.generation, l.seq);
    l.task = Some((token, due));
    m.next_token = m.next_token.saturating_add(1);
    // The one OS delivery task may go past the sequence's capacity, as
    // the IPC drain task may: the inputs it stands for are the OS's.
    let si = usize::try_from(seq.raw()).unwrap_or(usize::MAX);
    let number = st
        .proc_mut(pid, generation)
        .and_then(|p| p.seqs.get_mut(si))
        .map(|s| {
            let n = s.posted;
            s.posted = s.posted.saturating_add(1);
            s.delayed = s.delayed.saturating_add(1);
            n
        });
    let Some(number) = number else {
        // The listener's incarnation is gone; the next listener posts.
        if let Some(l) = st.os.get_mut(&pid).and_then(|m| m.listener.as_mut()) {
            l.task = None;
        }
        return;
    };
    let task = Task::new(move || deliver(&shared, pid, token));
    let slot = Slot {
        pid,
        generation,
        seq: si,
        number,
    };
    st.add_timer(
        slot,
        task,
        Duration::from_nanos(due.saturating_sub(now)),
        Duration::ZERO,
    );
}

/// One delivery task: the earliest due input, then the next task.
fn deliver(shared: &Weak<Shared>, pid: ProcessId, token: u64) {
    let Some(shared) = shared.upgrade() else {
        return;
    };
    let mut st = shared.lock();
    let now = st.now.as_nanos();
    let Some(m) = st.os.get_mut(&pid) else { return };
    let Some(l) = m.listener.as_mut() else { return };
    if l.task.map(|(t, _)| t) != Some(token) {
        // Superseded by an earlier input's task, or by a new listener.
        return;
    }
    l.task = None;
    if !l.live.load(Ordering::Acquire) {
        return;
    }
    let sink = Arc::clone(&l.sink);
    let ready = m
        .due
        .first_key_value()
        .is_some_and(|(&(due, _), _)| due <= now);
    let frame = if ready {
        m.due.pop_first().map(|(_, f)| f)
    } else {
        None
    };
    if let Some(f) = &frame {
        st.trace.record(format_args!("os input {} bytes", f.len()));
    }
    schedule(&mut st, pid);
    drop(st);
    if let Some(frame) = frame {
        sink.lock().unwrap_or_else(PoisonError::into_inner)(OsDelivery::Frame(frame));
    }
}

/// The broker's simulated `OsPort`.
pub(crate) struct SimOsPort {
    pub(crate) shared: Arc<Shared>,
    pub(crate) pid: ProcessId,
    pub(crate) generation: u32,
}

impl SimOsPort {
    /// Whether this incarnation has a live listener.
    fn listening(&self, m: &OsModel) -> bool {
        m.listener
            .as_ref()
            .is_some_and(|l| l.generation == self.generation && l.live.load(Ordering::Acquire))
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
        let now = st.now.as_nanos();
        let Some(m) = st.os.get_mut(&self.pid) else {
            return Err(OsError::Closed);
        };
        m.answer = Some(answer);
        // An answer arrives at its time whether or not anyone listens.
        m.anchor(now, frames);
        schedule(&mut st, self.pid);
        Ok(id)
    }

    fn listen(&self, seq: SequenceHandle, sink: OsSink) -> Result<OsListening, OsError> {
        let mut st = self.shared.lock();
        let now = st.now.as_nanos();
        let Some(m) = st.os.get_mut(&self.pid) else {
            return Err(OsError::Unsupported);
        };
        if self.listening(m) {
            return Err(OsError::Refused);
        }
        let first = std::mem::take(&mut m.unanchored);
        m.anchor(now, first);
        let live = Arc::new(AtomicBool::new(true));
        m.listener = Some(Listener {
            generation: self.generation,
            seq,
            sink: Arc::new(Mutex::new(sink)),
            live: Arc::clone(&live),
            task: None,
        });
        let pending = m.due.len();
        st.trace.record(format_args!("os listen {pending} pending"));
        schedule(&mut st, self.pid);
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
