//! `hg-launch-wire`: IZLH v1, the launch hand-off message (OS
//! integration 6.2, OI-D2; Appendix C item 2).
//!
//! Layer L2s, tier P, no dependencies outside `core` and `alloc` (G232).
//! It is the **only message format Izcal accepts from another build**: a
//! launcher of one build hands a launch to a running broker of another,
//! during an update, so unlike `.hgidl` (same-build only, VER-1) it is
//! versioned. It carries no endpoint and no handle. Threading: pure
//! functions, any thread.
//!
//! # Layout
//!
//! Little-endian throughout.
//!
//! | Part | Bytes |
//! |---|---|
//! | Header ([`HEADER_LEN`]) | magic `IZLH`; major u8 = 1; minor u8; message type u8 ([`MessageType`]); flags u8 (zero in v1, ignored on receipt); total length u32, header included, at most [`MAX_TOTAL`] |
//! | Field (repeated, at most [`MAX_FIELDS`]) | tag u16; flags u8 (bit 0 = critical); reserved u8 = 0; length u32; value |
//!
//! A receiver that does not know a tag skips the field, unless its
//! critical bit is set: then it refuses the whole message with
//! [`Refusal::UnsupportedCriticalField`]. A v1 receiver accepts any
//! major-1 message and refuses any other major.
//!
//! # Fields of a request
//!
//! | Tag | Field | Critical | Value |
//! |---|---|---|---|
//! | 1 | Intent ([`Intent`]) | yes | one byte, required |
//! | 2 | URL, repeatable | yes | text, at most [`MAX_OPERAND`] bytes |
//! | 3 | File path, repeatable | yes | text, at most [`MAX_OPERAND`] bytes |
//! | 4 | Working directory | no | text, at most [`MAX_OPERAND`] bytes |
//! | 5 | Profile selector ([`Profile`]) | yes | text, at most [`MAX_SHORT`] bytes |
//! | 6 | Activation token | no | text, at most [`MAX_SHORT`] bytes |
//! | 7 | Origin ([`Origin`]) | no | one byte |
//! | 8 | Sender build ([`SenderBuild`]) | no | build id u64; version and channel, each a u16 length and text |
//! | 9 | Nonce and batch ([`Nonce`]) | no | nonce u64, then optionally batch index u16 and count u16 |
//!
//! URLs and paths are the request's operands, in the order the user
//! gave them (`izcal a.html https://x b.html`), at most [`MAX_OPERANDS`]
//! per request; they may interleave, and the order is kept. "Text" is
//! well-formed UTF-8 with no NUL.
//!
//! The spec lists the request's tags and the reply's codes but not how
//! the reply, `commit` and `started` messages lay them out. This crate
//! uses the same tag numbers for the same meanings: a reply's outcome is
//! tag 1 (critical: status byte 1 accepted, 2 busy-retry with a u32
//! retry-after in milliseconds, 3 refused with a reason byte,
//! [`Reason`]) with the request's nonce echoed as tag 9; `commit` echoes
//! the nonce as tag 9; `started` carries the broker's build as tag 8.
//! The one-byte codes of every closed set start at 1, in the order the
//! spec lists them, so 0 is never valid.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

mod utf8;

/// `IZLH`.
pub const MAGIC: [u8; 4] = *b"IZLH";
/// The major version this crate speaks.
pub const MAJOR: u8 = 1;
/// The minor version this crate writes.
pub const MINOR: u8 = 0;
/// Header bytes.
pub const HEADER_LEN: usize = 12;
/// Bytes before each field's value.
pub const FIELD_HEADER_LEN: usize = 8;
/// The most bytes in one message, header included, so a request always
/// fits the IPC generator's 64 KiB cap when the broker re-encodes it as
/// a `Launch` input.
pub const MAX_TOTAL: usize = 64_512;
/// The most bytes in one URL, path or working directory.
pub const MAX_OPERAND: usize = 8_192;
/// The most bytes in any other text value.
pub const MAX_SHORT: usize = 512;
/// The most URLs plus paths in one request; a larger launch is split
/// into a batch of requests on one connection.
pub const MAX_OPERANDS: usize = 6;
/// The most fields in one message, unknown ones included.
pub const MAX_FIELDS: usize = 32;
/// The critical bit of a field's flags.
pub const CRITICAL: u8 = 0x01;

/// The four message types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageType {
    /// Launcher to broker: what to open.
    Request,
    /// Broker to launcher: accepted, busy, or refused.
    Reply,
    /// Launcher to broker, after `accepted` and the foreground grant.
    Commit,
    /// A newly started broker to the launcher that started it.
    Started,
}

impl MessageType {
    const fn code(self) -> u8 {
        match self {
            Self::Request => 1,
            Self::Reply => 2,
            Self::Commit => 3,
            Self::Started => 4,
        }
    }

    const fn from_code(c: u8) -> Option<Self> {
        match c {
            1 => Some(Self::Request),
            2 => Some(Self::Reply),
            3 => Some(Self::Commit),
            4 => Some(Self::Started),
            _ => None,
        }
    }
}

/// What the launch asks for. Closed: no automation intent exists
/// (AGT-R8, SEC-20).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Intent {
    /// Bring the browser forward.
    Activate,
    /// Open the URLs and paths.
    Open,
    /// Open a new window.
    NewWindow,
    /// Open a new private window.
    NewPrivateWindow,
}

impl Intent {
    const fn code(self) -> u8 {
        match self {
            Self::Activate => 1,
            Self::Open => 2,
            Self::NewWindow => 3,
            Self::NewPrivateWindow => 4,
        }
    }

    const fn from_code(c: u8) -> Option<Self> {
        match c {
            1 => Some(Self::Activate),
            2 => Some(Self::Open),
            3 => Some(Self::NewWindow),
            4 => Some(Self::NewPrivateWindow),
            _ => None,
        }
    }
}

/// Where a launch came from.
///
/// The codes follow the order of OS integration 6.2's field table
/// (command-line, os-open, jump-list, notification, dock-menu).
/// `os.hgidl`'s `LaunchOrigin` follows 6.1's `Launch` row, which lists
/// notification before jump-list, so the two orders differ for those
/// two; the broker maps by name, never by number. The spec does not say
/// which order is canonical, so neither is renumbered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    /// A command line.
    CommandLine,
    /// An open the OS delivered.
    OsOpen,
    /// A jump-list item.
    JumpList,
    /// A notification.
    Notification,
    /// The Dock menu.
    DockMenu,
}

impl Origin {
    const fn code(self) -> u8 {
        match self {
            Self::CommandLine => 1,
            Self::OsOpen => 2,
            Self::JumpList => 3,
            Self::Notification => 4,
            Self::DockMenu => 5,
        }
    }

    const fn from_code(c: u8) -> Option<Self> {
        match c {
            1 => Some(Self::CommandLine),
            2 => Some(Self::OsOpen),
            3 => Some(Self::JumpList),
            4 => Some(Self::Notification),
            5 => Some(Self::DockMenu),
            _ => None,
        }
    }
}

/// Which profile: its id, or the default. An unknown profile is refused
/// by the broker, never created.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Profile {
    /// `default`.
    Default,
    /// A profile id.
    Id(String),
}

/// The text that selects the default profile.
const DEFAULT_PROFILE: &str = "default";

/// The sending build.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SenderBuild {
    /// Its `BuildId`.
    pub build_id: u64,
    /// Its release version.
    pub version: String,
    /// Its channel.
    pub channel: String,
}

/// One request's place in a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Batch {
    /// This request's index, from 0.
    pub index: u16,
    /// How many requests the launch was split into.
    pub count: u16,
}

/// A nonce, echoed in the reply, and the batch position if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nonce {
    /// The nonce.
    pub nonce: u64,
    /// Set when one launch was split across several requests.
    pub batch: Option<Batch>,
}

/// One URL or path a launch opens.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    /// A URL (tag 2).
    Url(String),
    /// An absolute file path (tag 3).
    Path(String),
}

/// A launch request.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Request {
    /// What to do.
    pub intent: Intent,
    /// URLs and absolute file paths, in the order given (as
    /// `os.hgidl`'s `LaunchRequest.operands`).
    pub operands: Vec<Operand>,
    /// To resolve relative paths the launcher could not.
    pub working_dir: Option<String>,
    /// The profile; absent means the default.
    pub profile: Option<Profile>,
    /// The xdg-activation token or startup id (Linux).
    pub activation_token: Option<String>,
    /// Where the launch came from.
    pub origin: Option<Origin>,
    /// The launcher's build.
    pub sender_build: Option<SenderBuild>,
    /// The nonce and batch position.
    pub nonce: Option<Nonce>,
}

/// Why the broker refused a request. Closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reason {
    /// It did not decode.
    Malformed,
    /// It has a critical field this broker does not know.
    UnsupportedCriticalField,
    /// Over a bound.
    TooLarge,
    /// No such profile.
    UnknownProfile,
    /// An operand outside the inbound allowlist.
    NotAllowed,
    /// The request queue is full.
    QueueFull,
    /// The broker is shutting down.
    ShuttingDown,
}

impl Reason {
    const fn code(self) -> u8 {
        match self {
            Self::Malformed => 1,
            Self::UnsupportedCriticalField => 2,
            Self::TooLarge => 3,
            Self::UnknownProfile => 4,
            Self::NotAllowed => 5,
            Self::QueueFull => 6,
            Self::ShuttingDown => 7,
        }
    }

    const fn from_code(c: u8) -> Option<Self> {
        match c {
            1 => Some(Self::Malformed),
            2 => Some(Self::UnsupportedCriticalField),
            3 => Some(Self::TooLarge),
            4 => Some(Self::UnknownProfile),
            5 => Some(Self::NotAllowed),
            6 => Some(Self::QueueFull),
            7 => Some(Self::ShuttingDown),
            _ => None,
        }
    }
}

/// The broker's answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// Accepted; the launcher grants foreground rights and commits.
    Accepted,
    /// Try again after this many milliseconds.
    BusyRetry {
        /// The wait.
        retry_after_ms: u32,
    },
    /// Refused.
    Refused(Reason),
}

/// A reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reply {
    /// The answer.
    pub outcome: Outcome,
    /// The request's nonce, echoed.
    pub nonce: Option<u64>,
}

/// A commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Commit {
    /// The request's nonce, echoed.
    pub nonce: Option<u64>,
}

/// A newly started broker's first message.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Started {
    /// The broker's build.
    pub build: Option<SenderBuild>,
}

/// One IZLH message.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Message {
    /// See [`Request`].
    Request(Request),
    /// See [`Reply`].
    Reply(Reply),
    /// See [`Commit`].
    Commit(Commit),
    /// See [`Started`].
    Started(Started),
}

impl Message {
    /// Its type.
    pub const fn kind(&self) -> MessageType {
        match self {
            Self::Request(_) => MessageType::Request,
            Self::Reply(_) => MessageType::Reply,
            Self::Commit(_) => MessageType::Commit,
            Self::Started(_) => MessageType::Started,
        }
    }
}

/// A decoded message and the minor version it was written with.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Decoded {
    /// The sender's minor version (any is accepted).
    pub minor: u8,
    /// The message.
    pub message: Message,
}

/// Why a message was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Refusal {
    /// It does not decode: framing, an unknown code, text that is not
    /// well-formed, a missing or repeated field.
    Malformed,
    /// A major version other than 1.
    UnsupportedVersion,
    /// A critical field with an unknown tag.
    UnsupportedCriticalField,
    /// Over a bound: total length, a value's length, the operand count
    /// or the field count.
    TooLarge,
}

impl Refusal {
    /// The reply reason for it. The spec's closed reasons have no
    /// version case, so another major version is `malformed` to a v1
    /// receiver.
    pub const fn reason(self) -> Reason {
        match self {
            Self::Malformed | Self::UnsupportedVersion => Reason::Malformed,
            Self::UnsupportedCriticalField => Reason::UnsupportedCriticalField,
            Self::TooLarge => Reason::TooLarge,
        }
    }
}

/// The parts of a header that matter after it is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Header {
    /// The minor version.
    pub minor: u8,
    /// The message type.
    pub kind: MessageType,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    let end = at.checked_add(2)?;
    let s = b.get(at..end)?;
    let mut a = [0u8; 2];
    a.copy_from_slice(s);
    Some(u16::from_le_bytes(a))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    let end = at.checked_add(4)?;
    let s = b.get(at..end)?;
    let mut a = [0u8; 4];
    a.copy_from_slice(s);
    Some(u32::from_le_bytes(a))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    let end = at.checked_add(8)?;
    let s = b.get(at..end)?;
    let mut a = [0u8; 8];
    a.copy_from_slice(s);
    Some(u64::from_le_bytes(a))
}

/// Checks the header of `msg`, the whole message as received.
///
/// # Errors
/// [`Refusal::TooLarge`] if `msg` or its stated length is over
/// [`MAX_TOTAL`]; [`Refusal::UnsupportedVersion`] for another major;
/// [`Refusal::Malformed`] otherwise.
pub fn decode_header(msg: &[u8]) -> Result<Header, Refusal> {
    if msg.len() > MAX_TOTAL {
        return Err(Refusal::TooLarge);
    }
    let magic = msg.get(..4).ok_or(Refusal::Malformed)?;
    if magic != MAGIC.as_slice() {
        return Err(Refusal::Malformed);
    }
    let major = *msg.get(4).ok_or(Refusal::Malformed)?;
    if major != MAJOR {
        return Err(Refusal::UnsupportedVersion);
    }
    let minor = *msg.get(5).ok_or(Refusal::Malformed)?;
    let kind = msg
        .get(6)
        .and_then(|&c| MessageType::from_code(c))
        .ok_or(Refusal::Malformed)?;
    // Byte 7 holds the flags: zero in v1, and ignored, so a later minor
    // version may use them without breaking a v1 receiver.
    let total = u32_at(msg, 8).ok_or(Refusal::Malformed)?;
    let total = usize::try_from(total).map_err(|_e| Refusal::TooLarge)?;
    if total > MAX_TOTAL {
        return Err(Refusal::TooLarge);
    }
    if total != msg.len() || total < HEADER_LEN {
        return Err(Refusal::Malformed);
    }
    Ok(Header { minor, kind })
}

/// One field's header and where its value is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldAt {
    /// The tag.
    pub tag: u16,
    /// The critical bit is set.
    pub critical: bool,
    /// The value's first byte.
    pub start: usize,
    /// One past the value's last byte; also where the next field starts.
    pub end: usize,
}

/// The field at `at` in `msg`, or `None` at the end of the message.
///
/// # Errors
/// [`Refusal::Malformed`] if the field header or its value runs past the
/// end, or its reserved byte is not zero.
pub fn field_at(msg: &[u8], at: usize) -> Result<Option<FieldAt>, Refusal> {
    if at >= msg.len() {
        return Ok(None);
    }
    let tag = u16_at(msg, at).ok_or(Refusal::Malformed)?;
    let flags = *msg.get(at.saturating_add(2)).ok_or(Refusal::Malformed)?;
    let reserved = *msg.get(at.saturating_add(3)).ok_or(Refusal::Malformed)?;
    if reserved != 0 {
        return Err(Refusal::Malformed);
    }
    let len = u32_at(msg, at.saturating_add(4)).ok_or(Refusal::Malformed)?;
    let len = usize::try_from(len).map_err(|_e| Refusal::Malformed)?;
    let start = at.checked_add(FIELD_HEADER_LEN).ok_or(Refusal::Malformed)?;
    let end = start.checked_add(len).ok_or(Refusal::Malformed)?;
    if end > msg.len() {
        return Err(Refusal::Malformed);
    }
    Ok(Some(FieldAt {
        tag,
        critical: flags & CRITICAL != 0,
        start,
        end,
    }))
}

/// A text value of at most `max` bytes.
///
/// # Errors
/// [`Refusal::TooLarge`] over `max`; [`Refusal::Malformed`] if empty,
/// not well-formed UTF-8, or holding a NUL.
pub fn decode_text(v: &[u8], max: usize) -> Result<String, Refusal> {
    if v.len() > max {
        return Err(Refusal::TooLarge);
    }
    if v.is_empty() || !utf8::is_text(v) {
        return Err(Refusal::Malformed);
    }
    let s = core::str::from_utf8(v).map_err(|_e| Refusal::Malformed)?;
    Ok(String::from(s))
}

/// A one-byte code, decoded by `from`.
fn decode_code<T>(v: &[u8], from: impl FnOnce(u8) -> Option<T>) -> Result<T, Refusal> {
    match v {
        [c] => from(*c).ok_or(Refusal::Malformed),
        _ => Err(Refusal::Malformed),
    }
}

/// Field 1 of a request.
///
/// # Errors
/// [`Refusal::Malformed`] unless exactly one known code byte.
pub fn decode_intent(v: &[u8]) -> Result<Intent, Refusal> {
    decode_code(v, Intent::from_code)
}

/// Field 7 of a request.
///
/// # Errors
/// [`Refusal::Malformed`] unless exactly one known code byte.
pub fn decode_origin(v: &[u8]) -> Result<Origin, Refusal> {
    decode_code(v, Origin::from_code)
}

/// Field 5 of a request.
///
/// # Errors
/// As [`decode_text`] with [`MAX_SHORT`].
pub fn decode_profile(v: &[u8]) -> Result<Profile, Refusal> {
    let s = decode_text(v, MAX_SHORT)?;
    Ok(if s == DEFAULT_PROFILE {
        Profile::Default
    } else {
        Profile::Id(s)
    })
}

/// A u16 length and that many bytes of text at `at`; returns the text and
/// the offset after it.
fn short_text_at(v: &[u8], at: usize) -> Result<(String, usize), Refusal> {
    let n = usize::from(u16_at(v, at).ok_or(Refusal::Malformed)?);
    let start = at.checked_add(2).ok_or(Refusal::Malformed)?;
    let end = start.checked_add(n).ok_or(Refusal::Malformed)?;
    let bytes = v.get(start..end).ok_or(Refusal::Malformed)?;
    Ok((decode_text(bytes, MAX_SHORT)?, end))
}

/// Field 8 of a request, and of `started`.
///
/// # Errors
/// [`Refusal::Malformed`] unless a u64, two length-prefixed texts and
/// nothing after; [`Refusal::TooLarge`] for a text over [`MAX_SHORT`].
pub fn decode_sender_build(v: &[u8]) -> Result<SenderBuild, Refusal> {
    let build_id = u64_at(v, 0).ok_or(Refusal::Malformed)?;
    let (version, at) = short_text_at(v, 8)?;
    let (channel, end) = short_text_at(v, at)?;
    if end != v.len() {
        return Err(Refusal::Malformed);
    }
    Ok(SenderBuild {
        build_id,
        version,
        channel,
    })
}

/// Field 9 of a request.
///
/// # Errors
/// [`Refusal::Malformed`] unless 8 bytes, or 12 with a batch whose index
/// is below its count.
pub fn decode_nonce(v: &[u8]) -> Result<Nonce, Refusal> {
    let nonce = u64_at(v, 0).ok_or(Refusal::Malformed)?;
    match v.len() {
        8 => Ok(Nonce { nonce, batch: None }),
        12 => {
            let index = u16_at(v, 8).ok_or(Refusal::Malformed)?;
            let count = u16_at(v, 10).ok_or(Refusal::Malformed)?;
            if index >= count {
                return Err(Refusal::Malformed);
            }
            Ok(Nonce {
                nonce,
                batch: Some(Batch { index, count }),
            })
        }
        _ => Err(Refusal::Malformed),
    }
}

/// Field 9 of a reply or commit: the nonce alone.
///
/// # Errors
/// [`Refusal::Malformed`] unless exactly 8 bytes.
pub fn decode_echo(v: &[u8]) -> Result<u64, Refusal> {
    if v.len() != 8 {
        return Err(Refusal::Malformed);
    }
    u64_at(v, 0).ok_or(Refusal::Malformed)
}

/// Field 1 of a reply.
///
/// # Errors
/// [`Refusal::Malformed`] unless a known status with exactly its
/// payload.
pub fn decode_outcome(v: &[u8]) -> Result<Outcome, Refusal> {
    match v {
        [1] => Ok(Outcome::Accepted),
        [2, a, b, c, d] => Ok(Outcome::BusyRetry {
            retry_after_ms: u32::from_le_bytes([*a, *b, *c, *d]),
        }),
        [3, r] => Reason::from_code(*r)
            .map(Outcome::Refused)
            .ok_or(Refusal::Malformed),
        _ => Err(Refusal::Malformed),
    }
}

/// Request tags.
mod tag {
    pub(crate) const INTENT: u16 = 1;
    pub(crate) const URL: u16 = 2;
    pub(crate) const PATH: u16 = 3;
    pub(crate) const WORKING_DIR: u16 = 4;
    pub(crate) const PROFILE: u16 = 5;
    pub(crate) const ACTIVATION: u16 = 6;
    pub(crate) const ORIGIN: u16 = 7;
    pub(crate) const SENDER_BUILD: u16 = 8;
    pub(crate) const NONCE: u16 = 9;
    /// A reply's outcome shares the intent's tag: each message type has
    /// its own tag space, numbered alike.
    pub(crate) const OUTCOME: u16 = 1;
}

/// Sets `slot` once; a second value is a repeated field.
fn once<T>(slot: &mut Option<T>, v: T) -> Result<(), Refusal> {
    if slot.is_some() {
        return Err(Refusal::Malformed);
    }
    *slot = Some(v);
    Ok(())
}

/// What a request has collected so far.
#[derive(Default)]
struct RequestParts {
    intent: Option<Intent>,
    operands: Vec<Operand>,
    working_dir: Option<String>,
    profile: Option<Profile>,
    activation_token: Option<String>,
    origin: Option<Origin>,
    sender_build: Option<SenderBuild>,
    nonce: Option<Nonce>,
}

impl RequestParts {
    /// Takes one field; `Ok(false)` if the tag is not a request's.
    fn take(&mut self, tag: u16, v: &[u8]) -> Result<bool, Refusal> {
        match tag {
            tag::INTENT => once(&mut self.intent, decode_intent(v)?)?,
            tag::URL | tag::PATH => {
                if self.operands.len() >= MAX_OPERANDS {
                    return Err(Refusal::TooLarge);
                }
                let s = decode_text(v, MAX_OPERAND)?;
                self.operands.push(if tag == tag::URL {
                    Operand::Url(s)
                } else {
                    Operand::Path(s)
                });
            }
            tag::WORKING_DIR => once(&mut self.working_dir, decode_text(v, MAX_OPERAND)?)?,
            tag::PROFILE => once(&mut self.profile, decode_profile(v)?)?,
            tag::ACTIVATION => once(&mut self.activation_token, decode_text(v, MAX_SHORT)?)?,
            tag::ORIGIN => once(&mut self.origin, decode_origin(v)?)?,
            tag::SENDER_BUILD => once(&mut self.sender_build, decode_sender_build(v)?)?,
            tag::NONCE => once(&mut self.nonce, decode_nonce(v)?)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn finish(self) -> Result<Request, Refusal> {
        Ok(Request {
            intent: self.intent.ok_or(Refusal::Malformed)?,
            operands: self.operands,
            working_dir: self.working_dir,
            profile: self.profile,
            activation_token: self.activation_token,
            origin: self.origin,
            sender_build: self.sender_build,
            nonce: self.nonce,
        })
    }
}

/// Decodes one whole message.
///
/// # Errors
/// The first [`Refusal`]; the broker answers it with
/// [`Refusal::reason`].
pub fn decode(msg: &[u8]) -> Result<Decoded, Refusal> {
    let header = decode_header(msg)?;
    let mut req = RequestParts::default();
    let mut outcome = None;
    let mut echo = None;
    let mut build = None;
    let mut at = HEADER_LEN;
    let mut fields = 0usize;
    // Each field moves `at` forward by at least its 8-byte header, so the
    // loop runs at most `MAX_TOTAL / 8` times, and the field count stops
    // it after `MAX_FIELDS`.
    while let Some(f) = field_at(msg, at)? {
        fields = fields.saturating_add(1);
        if fields > MAX_FIELDS {
            return Err(Refusal::TooLarge);
        }
        let v = msg.get(f.start..f.end).ok_or(Refusal::Malformed)?;
        let known = match header.kind {
            MessageType::Request => req.take(f.tag, v)?,
            MessageType::Reply => match f.tag {
                tag::OUTCOME => once(&mut outcome, decode_outcome(v)?).map(|()| true)?,
                tag::NONCE => once(&mut echo, decode_echo(v)?).map(|()| true)?,
                _ => false,
            },
            MessageType::Commit => match f.tag {
                tag::NONCE => once(&mut echo, decode_echo(v)?).map(|()| true)?,
                _ => false,
            },
            MessageType::Started => match f.tag {
                tag::SENDER_BUILD => once(&mut build, decode_sender_build(v)?).map(|()| true)?,
                _ => false,
            },
        };
        if !known && f.critical {
            return Err(Refusal::UnsupportedCriticalField);
        }
        at = f.end;
    }
    let message = match header.kind {
        MessageType::Request => Message::Request(req.finish()?),
        MessageType::Reply => Message::Reply(Reply {
            outcome: outcome.ok_or(Refusal::Malformed)?,
            nonce: echo,
        }),
        MessageType::Commit => Message::Commit(Commit { nonce: echo }),
        MessageType::Started => Message::Started(Started { build }),
    };
    Ok(Decoded {
        minor: header.minor,
        message,
    })
}

/// Why a message could not be encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodeError {
    /// Over a bound: a value's length, the operand count or the total.
    /// The launcher refuses such a launch before sending, with a message.
    TooLarge,
    /// Text that is empty or holds a NUL, or a batch index not below its
    /// count.
    Invalid,
}

/// Builds one message.
struct Out {
    buf: Vec<u8>,
}

impl Out {
    fn field(&mut self, tag: u16, critical: bool, value: &[u8]) -> Result<(), EncodeError> {
        let len = u32::try_from(value.len()).map_err(|_e| EncodeError::TooLarge)?;
        self.buf.extend_from_slice(&tag.to_le_bytes());
        self.buf.push(if critical { CRITICAL } else { 0 });
        self.buf.push(0);
        self.buf.extend_from_slice(&len.to_le_bytes());
        self.buf.extend_from_slice(value);
        Ok(())
    }

    fn text(&mut self, tag: u16, critical: bool, s: &str, max: usize) -> Result<(), EncodeError> {
        text_ok(s, max)?;
        self.field(tag, critical, s.as_bytes())
    }
}

fn text_ok(s: &str, max: usize) -> Result<(), EncodeError> {
    if s.len() > max {
        return Err(EncodeError::TooLarge);
    }
    if s.is_empty() || s.as_bytes().contains(&0) {
        return Err(EncodeError::Invalid);
    }
    Ok(())
}

fn short_text(v: &mut Vec<u8>, s: &str) -> Result<(), EncodeError> {
    text_ok(s, MAX_SHORT)?;
    let n = u16::try_from(s.len()).map_err(|_e| EncodeError::TooLarge)?;
    v.extend_from_slice(&n.to_le_bytes());
    v.extend_from_slice(s.as_bytes());
    Ok(())
}

fn sender_build_bytes(b: &SenderBuild) -> Result<Vec<u8>, EncodeError> {
    let mut v = Vec::new();
    v.extend_from_slice(&b.build_id.to_le_bytes());
    short_text(&mut v, &b.version)?;
    short_text(&mut v, &b.channel)?;
    Ok(v)
}

/// Encodes `m` as a v1.0 message: fields in tag order, except that the
/// operands (tags 2 and 3) keep their own order.
///
/// # Errors
/// [`EncodeError`] if `m` breaks a bound or holds invalid text.
pub fn encode(m: &Message) -> Result<Vec<u8>, EncodeError> {
    let mut o = Out { buf: Vec::new() };
    o.buf.extend_from_slice(&MAGIC);
    o.buf.push(MAJOR);
    o.buf.push(MINOR);
    o.buf.push(m.kind().code());
    o.buf.push(0);
    o.buf.extend_from_slice(&[0u8; 4]);
    match m {
        Message::Request(r) => {
            if r.operands.len() > MAX_OPERANDS {
                return Err(EncodeError::TooLarge);
            }
            o.field(tag::INTENT, true, &[r.intent.code()])?;
            for operand in &r.operands {
                match operand {
                    Operand::Url(u) => o.text(tag::URL, true, u, MAX_OPERAND)?,
                    Operand::Path(p) => o.text(tag::PATH, true, p, MAX_OPERAND)?,
                }
            }
            if let Some(w) = &r.working_dir {
                o.text(tag::WORKING_DIR, false, w, MAX_OPERAND)?;
            }
            match &r.profile {
                None => {}
                Some(Profile::Default) => {
                    o.text(tag::PROFILE, true, DEFAULT_PROFILE, MAX_SHORT)?;
                }
                Some(Profile::Id(id)) => {
                    if id == DEFAULT_PROFILE {
                        return Err(EncodeError::Invalid);
                    }
                    o.text(tag::PROFILE, true, id, MAX_SHORT)?;
                }
            }
            if let Some(t) = &r.activation_token {
                o.text(tag::ACTIVATION, false, t, MAX_SHORT)?;
            }
            if let Some(g) = r.origin {
                o.field(tag::ORIGIN, false, &[g.code()])?;
            }
            if let Some(b) = &r.sender_build {
                o.field(tag::SENDER_BUILD, false, &sender_build_bytes(b)?)?;
            }
            if let Some(n) = r.nonce {
                let mut v = Vec::new();
                v.extend_from_slice(&n.nonce.to_le_bytes());
                if let Some(b) = n.batch {
                    if b.index >= b.count {
                        return Err(EncodeError::Invalid);
                    }
                    v.extend_from_slice(&b.index.to_le_bytes());
                    v.extend_from_slice(&b.count.to_le_bytes());
                }
                o.field(tag::NONCE, false, &v)?;
            }
        }
        Message::Reply(r) => {
            let v: Vec<u8> = match r.outcome {
                Outcome::Accepted => alloc::vec![1],
                Outcome::BusyRetry { retry_after_ms } => {
                    let mut v = alloc::vec![2];
                    v.extend_from_slice(&retry_after_ms.to_le_bytes());
                    v
                }
                Outcome::Refused(reason) => alloc::vec![3, reason.code()],
            };
            o.field(tag::OUTCOME, true, &v)?;
            if let Some(n) = r.nonce {
                o.field(tag::NONCE, false, &n.to_le_bytes())?;
            }
        }
        Message::Commit(c) => {
            if let Some(n) = c.nonce {
                o.field(tag::NONCE, false, &n.to_le_bytes())?;
            }
        }
        Message::Started(s) => {
            if let Some(b) = &s.build {
                o.field(tag::SENDER_BUILD, false, &sender_build_bytes(b)?)?;
            }
        }
    }
    let total = o.buf.len();
    if total > MAX_TOTAL {
        return Err(EncodeError::TooLarge);
    }
    let total = u32::try_from(total).map_err(|_e| EncodeError::TooLarge)?;
    if let Some(len) = o.buf.get_mut(8..12) {
        len.copy_from_slice(&total.to_le_bytes());
    }
    Ok(o.buf)
}

/// VER-1 for IZLH (G232): the header, every field decoder and the length
/// arithmetic, for any bytes in the bounds given. Each harness checks
/// that decoding never panics, overflows or reads out of bounds (Kani's
/// automatic checks) plus the property stated.
#[cfg(kani)]
mod proofs {
    use super::*;

    /// Any header-sized input, plus up to three more bytes: a header is
    /// accepted only if its stated length is the input's.
    #[kani::proof]
    #[kani::unwind(6)]
    fn header() {
        let bytes: [u8; 15] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= 15);
        let msg = &bytes[..len];
        if let Ok(h) = decode_header(msg) {
            assert!(len >= HEADER_LEN);
            assert!(usize::try_from(u32_at(msg, 8).unwrap_or(0)) == Ok(len));
            assert!(h.kind.code() == msg[6]);
        }
        kani::cover!(decode_header(msg).is_ok(), "a header is accepted");
    }

    /// The length arithmetic: for any bytes and any offset, a field found
    /// lies inside the message and past its header.
    #[kani::proof]
    #[kani::unwind(4)]
    fn field_bounds() {
        let bytes: [u8; 24] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= 24);
        let at: usize = kani::any();
        let msg = &bytes[..len];
        if let Ok(Some(f)) = field_at(msg, at) {
            assert!(f.start == at + FIELD_HEADER_LEN);
            assert!(f.start <= f.end && f.end <= len);
        }
        kani::cover!(
            matches!(field_at(msg, at), Ok(Some(f)) if f.end > f.start),
            "a field with a value"
        );
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn codes() {
        let bytes: [u8; 2] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= 2);
        let v = &bytes[..len];
        if decode_intent(v).is_ok() || decode_origin(v).is_ok() {
            assert!(len == 1 && v[0] != 0);
        }
        kani::cover!(decode_intent(v).is_ok(), "an intent");
        kani::cover!(decode_origin(v).is_ok(), "an origin");
    }

    #[kani::proof]
    #[kani::unwind(8)]
    fn text() {
        let bytes: [u8; 6] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= 6);
        let max: usize = kani::any_where(|&n: &usize| n <= 6);
        let v = &bytes[..len];
        match decode_text(v, max) {
            Ok(s) => assert!(s.len() == len && len <= max && len > 0),
            Err(Refusal::TooLarge) => assert!(len > max),
            Err(_) => assert!(len <= max),
        }
        kani::cover!(decode_text(v, max).is_ok(), "text is accepted");
    }

    #[kani::proof]
    #[kani::unwind(8)]
    fn sender_build() {
        let bytes: [u8; 16] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= 16);
        let v = &bytes[..len];
        kani::cover!(decode_sender_build(v).is_ok(), "a sender build");
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn nonce_and_outcome() {
        let bytes: [u8; 13] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= 13);
        let v = &bytes[..len];
        if let Ok(n) = decode_nonce(v) {
            assert!(n.batch.is_none_or(|b| b.index < b.count));
        }
        kani::cover!(
            matches!(decode_nonce(v), Ok(Nonce { batch: Some(_), .. })),
            "a batch"
        );
        kani::cover!(
            matches!(decode_outcome(v), Ok(Outcome::BusyRetry { .. })),
            "busy-retry"
        );
        kani::cover!(
            matches!(decode_outcome(v), Ok(Outcome::Refused(_))),
            "a refusal"
        );
    }
}
