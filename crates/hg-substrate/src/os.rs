//! The broker's door to the operating system's non-input services
//! (OS integration 6.1, OI-D1, OI-D26; Appendix C item 1).
//!
//! One trait, shaped like [`crate::Ipc`] as built: frames are bytes (the
//! `os.hgidl` encodings, decoded by `hg-os-types` above this layer), a
//! command never blocks, and inputs arrive as tasks on a sequence, in
//! order, until the listener's guard is dropped (CS-AS-4: no `async`).
//! Only the broker's substrate has one ([`crate::Substrate::os`]):
//! production is `hg-substrate-os`'s per-OS modules (OI-M1), simulation
//! a scripted OS model, and from OI-M1 a recorded stream. Every frame
//! crossing it is a recorded input or command.

use core::any::Any;

use crate::SequenceHandle;

/// A command's id: monotonic per broker incarnation. An answer arrives as
/// an input frame carrying it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OsCallId(u64);

impl OsCallId {
    /// Id `raw`. For `OsPort` implementations only.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The raw value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// What a listener is given.
#[derive(Debug, PartialEq, Eq)]
pub enum OsDelivery {
    /// One `OsInput` frame.
    Frame(Vec<u8>),
    /// The port is gone; nothing follows.
    Closed,
}

/// A listener's callback: runs as a task on its sequence, once per
/// delivery, in order.
pub type OsSink = Box<dyn FnMut(OsDelivery) + Send>;

/// Why an `OsPort` call failed. Closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsError {
    /// This substrate has no OS port, or not this command yet.
    Unsupported,
    /// A frame over the 64 KiB message cap.
    TooLarge,
    /// The port is gone.
    Closed,
    /// Refused: a second listener, or a frame the port does not accept.
    Refused,
}

impl core::fmt::Display for OsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Unsupported => "no OS port for this",
            Self::TooLarge => "an OS frame over the message cap",
            Self::Closed => "the OS port is closed",
            Self::Refused => "the OS port refused the call",
        })
    }
}

impl std::error::Error for OsError {}

/// Keeps a listener listening; dropping it stops delivery.
pub struct OsListening {
    /// Held only for its `Drop`.
    _guard: Box<dyn Any + Send>,
}

impl OsListening {
    /// Wraps the implementation's guard. For `OsPort` implementations
    /// only.
    pub fn new(guard: Box<dyn Any + Send>) -> Self {
        Self { _guard: guard }
    }
}

impl core::fmt::Debug for OsListening {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("OsListening")
    }
}

/// The broker's door to the operating system's non-input services.
pub trait OsPort: Send + Sync {
    /// Asks the OS to do something; never blocks. The answer arrives as an
    /// input frame carrying the returned id.
    ///
    /// # Errors
    /// [`OsError::TooLarge`] for a frame over the cap, [`OsError::Closed`],
    /// [`OsError::Refused`], [`OsError::Unsupported`].
    fn command(&self, frame: Vec<u8>) -> Result<OsCallId, OsError>;

    /// Delivers every OS input frame, in order, as tasks on `seq`, until
    /// the guard is dropped. One broker dispatcher fans inputs out to
    /// consumers, so a second listener is refused.
    ///
    /// # Errors
    /// [`OsError::Refused`] for a second listener, [`OsError::Closed`],
    /// [`OsError::Unsupported`].
    fn listen(&self, seq: SequenceHandle, sink: OsSink) -> Result<OsListening, OsError>;
}
