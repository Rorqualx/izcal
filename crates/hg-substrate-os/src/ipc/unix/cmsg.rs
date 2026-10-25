//! Our own parser of the ancillary ("control") data `recvmsg(2)`
//! returns, written from the documented layout in cmsg(3) and unix(7)
//! for 64-bit Linux.
//!
//! The buffer is a run of control messages. Each starts at a multiple of
//! 8 bytes (`CMSG_ALIGN`) with a 16-byte header: `cmsg_len` (a `size_t`,
//! 8 bytes, the header plus data, without trailing padding), then
//! `cmsg_level` and `cmsg_type` (`int` each). Its data follows the header
//! directly (`CMSG_LEN(n)` = 16 + n), and the next message starts after
//! the padding (`CMSG_SPACE(n)` = 16 + `CMSG_ALIGN(n)`).
//!
//! This transport asks for two kinds only, both at level `SOL_SOCKET`:
//! `SCM_RIGHTS` (an array of `int` descriptors the kernel has already
//! installed in this process) and `SCM_CREDENTIALS` (a `struct ucred`:
//! pid, uid, gid). Anything else is a protocol violation. Because the
//! kernel installs `SCM_RIGHTS` descriptors before we see them, the
//! parser lists every descriptor it sees, in every `SCM_RIGHTS` message,
//! even in a buffer it refuses, so the caller can close them all.
//!
//! The constants here are checked against the C library's `CMSG_*`
//! macros by a test in the sys module, on Linux.

use hg_substrate::ipc::MAX_MESSAGE_HANDLES;

/// Bytes in a `cmsghdr` on 64-bit Linux: `cmsg_len` (8) + `cmsg_level`
/// (4) + `cmsg_type` (4).
pub(crate) const CMSG_HDR_LEN: usize = 16;

/// The alignment of each control message and its data (`CMSG_ALIGN`).
pub(crate) const CMSG_ALIGN_TO: usize = 8;

/// `SOL_SOCKET`, the socket-level option and control-message level.
pub(crate) const SOL_SOCKET: i32 = 1;

/// `SCM_RIGHTS`: an array of file descriptors.
pub(crate) const SCM_RIGHTS: i32 = 1;

/// `SCM_CREDENTIALS`: the sender's `struct ucred`.
pub(crate) const SCM_CREDENTIALS: i32 = 2;

/// Bytes in a descriptor (`int`).
pub(crate) const FD_LEN: usize = 4;

/// Bytes in a `struct ucred`: pid (`pid_t`), uid, gid, 4 bytes each.
pub(crate) const UCRED_LEN: usize = 12;

/// `CMSG_ALIGN(n)`: `n` rounded up to [`CMSG_ALIGN_TO`], or `None` on
/// overflow.
pub(crate) const fn cmsg_align(n: usize) -> Option<usize> {
    match n.checked_add(CMSG_ALIGN_TO - 1) {
        Some(v) => Some(v & !(CMSG_ALIGN_TO - 1)),
        None => None,
    }
}

/// `CMSG_LEN(n)`: the `cmsg_len` of a message with `n` data bytes.
/// Gives `usize::MAX` when that would overflow, which no buffer holds.
pub(crate) const fn cmsg_len(n: usize) -> usize {
    match n.checked_add(CMSG_HDR_LEN) {
        Some(v) => v,
        None => usize::MAX,
    }
}

/// `CMSG_SPACE(n)`: the buffer space a message with `n` data bytes
/// takes. Gives `usize::MAX` when that would overflow.
pub(crate) const fn cmsg_space(n: usize) -> usize {
    match cmsg_align(n) {
        Some(a) => cmsg_len(a),
        None => usize::MAX,
    }
}

/// The control buffer the transport reads into: room for one
/// `SCM_RIGHTS` of [`MAX_MESSAGE_HANDLES`] descriptors and one
/// `SCM_CREDENTIALS`. A sender that sends more gets `MSG_CTRUNC`.
pub(crate) const CONTROL_LEN: usize =
    cmsg_space(MAX_MESSAGE_HANDLES * FD_LEN) + cmsg_space(UCRED_LEN);

/// The most descriptor slots any buffer of [`CONTROL_LEN`] bytes can
/// hold: one `SCM_RIGHTS` header and the rest descriptors. Bounds the
/// close set.
pub(crate) const MAX_SEEN_FDS: usize = (CONTROL_LEN - CMSG_HDR_LEN) / FD_LEN;

/// The most control messages a buffer of [`CONTROL_LEN`] bytes can hold.
/// Bounds the parser's loop.
const MAX_CMSGS: usize = CONTROL_LEN / CMSG_HDR_LEN;

const _: () = assert!(CONTROL_LEN == 112);
const _: () = assert!(MAX_SEEN_FDS == 24);
const _: () = assert!(MAX_CMSGS == 7);

/// The sender's credentials, as the kernel attests them (unix(7),
/// `SO_PASSCRED`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Ucred {
    /// Process ID.
    pub(crate) pid: i32,
    /// User ID.
    pub(crate) uid: u32,
    /// Group ID.
    pub(crate) gid: u32,
}

/// Why a control buffer was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CmsgError {
    /// A level or type other than `SCM_RIGHTS` or `SCM_CREDENTIALS` at
    /// `SOL_SOCKET`.
    Unknown {
        /// Its `cmsg_level`.
        level: i32,
        /// Its `cmsg_type`.
        kind: i32,
    },
    /// A `cmsg_len` below `CMSG_LEN(0)`.
    ShortLen,
    /// A message whose `cmsg_len` runs past the buffer.
    PastEnd,
    /// `SCM_RIGHTS` data that is not a whole number of descriptors.
    RightsLength,
    /// More than one `SCM_RIGHTS`.
    TwoRights,
    /// More than [`MAX_MESSAGE_HANDLES`] descriptors.
    TooManyFds,
    /// More than one `SCM_CREDENTIALS`.
    TwoCredentials,
    /// `SCM_CREDENTIALS` data that is not one `struct ucred`.
    CredentialsLength,
    /// Bytes after the last message too few to be a header.
    Trailing,
    /// A buffer longer than [`CONTROL_LEN`]: the transport never reads
    /// more, so the caller is broken.
    Oversize,
}

/// What [`parse`] found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParsedControl {
    /// The first `SCM_RIGHTS` message's descriptors, up to
    /// [`MAX_MESSAGE_HANDLES`]; slots at and past `fd_count` are -1.
    fds: [i32; MAX_MESSAGE_HANDLES],
    fd_count: usize,
    /// How many `SCM_RIGHTS` messages there were.
    pub(crate) rights_msgs: usize,
    /// The credentials, if exactly one well-formed `SCM_CREDENTIALS`.
    pub(crate) cred: Option<Ucred>,
    /// Every descriptor seen in any `SCM_RIGHTS`, in order; slots at and
    /// past `close_count` are -1.
    close: [i32; MAX_SEEN_FDS],
    close_count: usize,
    /// Whether the buffer is acceptable, or the first reason it is not.
    pub(crate) outcome: Result<(), CmsgError>,
}

impl ParsedControl {
    /// An empty result: no messages.
    pub(crate) const EMPTY: Self = Self {
        fds: [-1; MAX_MESSAGE_HANDLES],
        fd_count: 0,
        rights_msgs: 0,
        cred: None,
        close: [-1; MAX_SEEN_FDS],
        close_count: 0,
        outcome: Ok(()),
    };

    /// The received descriptors, in order. Meaningful only when
    /// `outcome` is `Ok`.
    pub(crate) fn fds(&self) -> &[i32] {
        self.fds.get(..self.fd_count).unwrap_or(&[])
    }

    /// How many descriptors [`Self::fds`] holds.
    pub(crate) const fn fd_count(&self) -> usize {
        self.fd_count
    }

    /// Every descriptor the kernel installed, which the caller closes
    /// unless it accepts the record.
    pub(crate) fn close_set(&self) -> &[i32] {
        self.close.get(..self.close_count).unwrap_or(&[])
    }

    /// A result as [`parse`] might give it, built directly, for the
    /// decision's tests and harnesses: `fds` received (and closed), then
    /// `extra` closed only.
    #[cfg(any(test, kani))]
    pub(crate) fn from_parts(
        fds: &[i32],
        extra: &[i32],
        rights_msgs: usize,
        cred: Option<Ucred>,
        outcome: Result<(), CmsgError>,
    ) -> Self {
        let mut p = Self::EMPTY;
        for &fd in fds.iter().take(MAX_MESSAGE_HANDLES) {
            p.saw_fd(fd, true);
        }
        for &fd in extra.iter().take(MAX_SEEN_FDS) {
            p.saw_fd(fd, false);
        }
        p.rights_msgs = rights_msgs;
        p.cred = cred;
        p.outcome = outcome;
        p
    }

    fn fail(&mut self, e: CmsgError) {
        if self.outcome.is_ok() {
            self.outcome = Err(e);
        }
    }

    /// Notes a descriptor of the first `SCM_RIGHTS` (`first`) or of any
    /// later one: it always goes in the close set.
    fn saw_fd(&mut self, fd: i32, first: bool) {
        if !push(&mut self.close, &mut self.close_count, fd) {
            // Only a buffer over CONTROL_LEN can get here, and that is
            // refused already.
            self.fail(CmsgError::Oversize);
        }
        if first && !push(&mut self.fds, &mut self.fd_count, fd) {
            self.fail(CmsgError::TooManyFds);
        }
    }
}

/// Appends `fd` at `slots[*count]`; false if the array is full.
fn push<const N: usize>(slots: &mut [i32; N], count: &mut usize, fd: i32) -> bool {
    match (slots.get_mut(*count), count.checked_add(1)) {
        (Some(slot), Some(next)) => {
            *slot = fd;
            *count = next;
            true
        }
        (None, _) | (_, None) => false,
    }
}

/// Parses a control buffer: exactly the `msg_controllen` bytes
/// `recvmsg(2)` reported.
///
/// Never panics, for any input. Semantic faults (an unknown message, a
/// second `SCM_RIGHTS`, too many descriptors) do not stop the walk, so
/// descriptors in later messages are still seen and closed; a framing
/// fault (a length that cannot be followed) does, since nothing after it
/// can be located.
pub(crate) fn parse(control: &[u8]) -> ParsedControl {
    // A length over CONTROL_LEN is refused by parse_fixed, which still
    // walks the first CONTROL_LEN bytes so their descriptors are closed.
    parse_fixed(&load(control), control.len())
}

/// The first [`CONTROL_LEN`] bytes of `control`, zero-filled after it.
fn load(control: &[u8]) -> [u8; CONTROL_LEN] {
    let mut buf = [0u8; CONTROL_LEN];
    let len = control.len().min(CONTROL_LEN);
    if let (Some(dst), Some(src)) = (buf.get_mut(..len), control.get(..len)) {
        dst.copy_from_slice(src);
    }
    buf
}

/// Byte `i` of `buf`, or 0 past its end (never reached: every caller
/// checks `i` against the buffer's used length first).
fn byte(buf: &[u8; CONTROL_LEN], i: usize) -> u8 {
    buf.get(i).copied().unwrap_or(0)
}

fn word_at(buf: &[u8; CONTROL_LEN], at: usize) -> [u8; 4] {
    [
        byte(buf, at),
        byte(buf, at.wrapping_add(1)),
        byte(buf, at.wrapping_add(2)),
        byte(buf, at.wrapping_add(3)),
    ]
}

fn u64_at(buf: &[u8; CONTROL_LEN], at: usize) -> u64 {
    let lo = u32::from_le_bytes(word_at(buf, at));
    let hi = u32::from_le_bytes(word_at(buf, at.wrapping_add(4)));
    u64::from(lo) | (u64::from(hi) << 32)
}

/// [`parse`] over the first `len` bytes of a fixed buffer.
///
/// Written with indices into one fixed-size array and loops bounded by
/// constants, never slices at data-dependent offsets, so its proof stays
/// small (the D1 lesson: concrete sizes, symbolic contents). `len` above
/// [`CONTROL_LEN`] is treated as [`CONTROL_LEN`] and refused.
pub(crate) fn parse_fixed(buf: &[u8; CONTROL_LEN], len: usize) -> ParsedControl {
    let mut p = ParsedControl::EMPTY;
    let end = if len > CONTROL_LEN {
        p.fail(CmsgError::Oversize);
        CONTROL_LEN
    } else {
        len
    };
    let mut off = 0usize;
    // Each message takes at least a header, and each step moves at least
    // CMSG_HDR_LEN bytes on, so MAX_CMSGS steps reach the end of any
    // buffer of CONTROL_LEN bytes (VER-1 bound).
    for _ in 0..MAX_CMSGS {
        if off >= end {
            break;
        }
        match step(buf, end, off, &mut p) {
            Step::Next(next) => off = next,
            Step::Stop => break,
        }
    }
    if off < end {
        // Only reachable after a framing fault, which already failed, or
        // when the loop bound ran out: more messages than CONTROL_LEN can
        // hold.
        p.fail(CmsgError::Oversize);
    }
    p
}

/// What one [`step`] of the walk leaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// The next message starts here (or this is `end`).
    Next(usize),
    /// A framing fault: nothing after `off` can be located.
    Stop,
}

/// Reads the one control message at `off` (`off < end <= CONTROL_LEN`)
/// into `p`. The proofs reason about this step from any state `p` can be
/// in, which covers every walk by induction over its at most
/// [`MAX_CMSGS`] steps.
fn step(buf: &[u8; CONTROL_LEN], end: usize, off: usize, p: &mut ParsedControl) -> Step {
    let avail = end.saturating_sub(off);
    if avail < CMSG_HDR_LEN {
        p.fail(CmsgError::Trailing);
        return Step::Stop;
    }
    let Ok(mlen) = usize::try_from(u64_at(buf, off)) else {
        p.fail(CmsgError::PastEnd);
        return Step::Stop;
    };
    let level = i32::from_le_bytes(word_at(buf, off.saturating_add(8)));
    let kind = i32::from_le_bytes(word_at(buf, off.saturating_add(12)));
    // The data length `cmsg_len` declares; below CMSG_LEN(0) there is
    // no telling where the next message starts.
    let Some(declared) = mlen.checked_sub(CMSG_HDR_LEN) else {
        p.fail(CmsgError::ShortLen);
        return Step::Stop;
    };
    // The data actually in the buffer: all of it, or (when the length
    // runs past the end) what is there, so its descriptors are seen.
    let data_len = if mlen <= avail {
        declared
    } else {
        p.fail(CmsgError::PastEnd);
        avail.saturating_sub(CMSG_HDR_LEN)
    };
    let data = off.saturating_add(CMSG_HDR_LEN);
    match (level, kind) {
        (SOL_SOCKET, SCM_RIGHTS) => {
            let first = p.rights_msgs == 0;
            if !first {
                p.fail(CmsgError::TwoRights);
            }
            if let Some(n) = p.rights_msgs.checked_add(1) {
                p.rights_msgs = n;
            }
            if !declared.is_multiple_of(FD_LEN) {
                p.fail(CmsgError::RightsLength);
            }
            let whole = data_len / FD_LEN;
            for i in 0..MAX_SEEN_FDS {
                if i >= whole {
                    break;
                }
                let fd =
                    i32::from_le_bytes(word_at(buf, data.saturating_add(i.saturating_mul(FD_LEN))));
                p.saw_fd(fd, first);
            }
        }
        (SOL_SOCKET, SCM_CREDENTIALS) => {
            if p.cred.is_some() {
                p.fail(CmsgError::TwoCredentials);
            } else if declared != UCRED_LEN || data_len != UCRED_LEN {
                p.fail(CmsgError::CredentialsLength);
            } else {
                p.cred = Some(Ucred {
                    pid: i32::from_le_bytes(word_at(buf, data)),
                    uid: u32::from_le_bytes(word_at(buf, data.saturating_add(4))),
                    gid: u32::from_le_bytes(word_at(buf, data.saturating_add(8))),
                });
            }
        }
        _ => p.fail(CmsgError::Unknown { level, kind }),
    }
    // The next message starts at CMSG_ALIGN(len). The kernel may end
    // the buffer before the last message's padding, so a step past the
    // end is the end.
    match cmsg_align(mlen) {
        Some(next) if next <= avail => Step::Next(off.saturating_add(next)),
        _ => Step::Next(end),
    }
}

/// Writes one `SCM_RIGHTS` message carrying `fds` at the start of `out`,
/// for `sendmsg(2)`, and gives the `msg_controllen` to pass: its
/// `CMSG_SPACE`. Padding bytes are zeroed.
///
/// `None` (and `out` unspecified) for more than [`MAX_MESSAGE_HANDLES`]
/// descriptors, none at all (send no control data instead), or an `out`
/// shorter than the message.
pub(crate) fn encode_rights(fds: &[i32], out: &mut [u8]) -> Option<usize> {
    if fds.is_empty() || fds.len() > MAX_MESSAGE_HANDLES {
        return None;
    }
    let data_len = fds.len().checked_mul(FD_LEN)?;
    let len = cmsg_len(data_len);
    let space = cmsg_space(data_len);
    let msg = out.get_mut(..space)?;
    msg.fill(0);
    let (hdr, data) = msg.split_at_mut(CMSG_HDR_LEN);
    let (len_bytes, rest) = hdr.split_at_mut(8);
    len_bytes.copy_from_slice(&u64::try_from(len).ok()?.to_le_bytes());
    let (level, kind) = rest.split_at_mut(4);
    level.copy_from_slice(&SOL_SOCKET.to_le_bytes());
    kind.copy_from_slice(&SCM_RIGHTS.to_le_bytes());
    let (slots, _padding) = data.as_chunks_mut::<FD_LEN>();
    for (slot, fd) in slots.iter_mut().zip(fds.iter()) {
        slot.copy_from_slice(&fd.to_le_bytes());
    }
    Some(space)
}

/// Encodes control data the way the kernel lays it out, for tests and
/// harnesses: an `SCM_RIGHTS` of `fds` (if any; more than
/// [`MAX_MESSAGE_HANDLES`] allowed here), then credentials (if any).
#[cfg(any(test, kani))]
pub(crate) fn encode(fds: &[i32], cred: Option<Ucred>) -> Vec<u8> {
    fn header(out: &mut Vec<u8>, data_len: usize, kind: i32) {
        let len = u64::try_from(cmsg_len(data_len)).unwrap_or(0);
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&SOL_SOCKET.to_le_bytes());
        out.extend_from_slice(&kind.to_le_bytes());
    }
    fn pad(out: &mut Vec<u8>) {
        while !out.len().is_multiple_of(CMSG_ALIGN_TO) {
            out.push(0);
        }
    }
    let mut out = Vec::new();
    if !fds.is_empty() {
        header(
            &mut out,
            fds.len().checked_mul(FD_LEN).unwrap_or(0),
            SCM_RIGHTS,
        );
        for fd in fds {
            out.extend_from_slice(&fd.to_le_bytes());
        }
        pad(&mut out);
    }
    if let Some(c) = cred {
        header(&mut out, UCRED_LEN, SCM_CREDENTIALS);
        out.extend_from_slice(&c.pid.to_le_bytes());
        out.extend_from_slice(&c.uid.to_le_bytes());
        out.extend_from_slice(&c.gid.to_le_bytes());
        pad(&mut out);
    }
    out
}

/// VER-1 for the control-data parser (SEC-4 (b), IPC-R1).
#[cfg(kani)]
mod proofs {
    use super::*;

    /// The invariant every state of the walk keeps: the counts fit their
    /// arrays, the received descriptors are a prefix of the close set, no
    /// descriptor is seen before an `SCM_RIGHTS`, and an accepted state
    /// reports exactly what it installed, from at most one message.
    fn invariant(p: &ParsedControl) -> bool {
        let mut prefix = true;
        for i in 0..MAX_MESSAGE_HANDLES {
            if i < p.fd_count && p.fds[i] != p.close[i] {
                prefix = false;
            }
        }
        p.fd_count <= MAX_MESSAGE_HANDLES
            && p.close_count <= MAX_SEEN_FDS
            && p.fd_count <= p.close_count
            && prefix
            && (p.rights_msgs > 0 || p.close_count == 0)
            && (p.outcome.is_err() || (p.fd_count == p.close_count && p.rights_msgs <= 1))
    }

    /// Any state that keeps the invariant.
    fn any_state() -> ParsedControl {
        let p = ParsedControl {
            fds: kani::any(),
            fd_count: kani::any(),
            rights_msgs: kani::any_where(|&n: &usize| n <= MAX_CMSGS),
            cred: if kani::any() {
                Some(Ucred {
                    pid: kani::any(),
                    uid: kani::any(),
                    gid: kani::any(),
                })
            } else {
                None
            },
            close: kani::any(),
            close_count: kani::any(),
            outcome: if kani::any() {
                Ok(())
            } else {
                Err(CmsgError::Trailing)
            },
        };
        kani::assume(invariant(&p));
        p
    }

    /// One step of the walk, from any state that keeps the invariant, on
    /// any bytes at any offset: no panic or out-of-bounds read; the
    /// invariant still holds; the close set only grows, keeping every
    /// descriptor already in it; an `SCM_RIGHTS` message adds every
    /// whole descriptor it carries, in order (while the close set has
    /// room, which a buffer of CONTROL_LEN bytes never exceeds); a
    /// second credential is refused; and the walk moves on by at least a
    /// header, so MAX_CMSGS steps reach the end. With the empty state as
    /// the base case, this covers every walk by induction.
    #[kani::proof]
    #[kani::unwind(25)]
    fn a_step_keeps_the_invariant() {
        let buf: [u8; CONTROL_LEN] = kani::any();
        let end: usize = kani::any_where(|&e: &usize| e <= CONTROL_LEN);
        let off: usize = kani::any_where(|&o: &usize| o < end);
        let mut p = any_state();
        let before = p;
        let r = step(&buf, end, off, &mut p);
        assert!(invariant(&p));
        // The close set only grows, and keeps what it had.
        assert!(p.close_count >= before.close_count);
        // (Picked only when there is something to keep: an any_where over an
        // empty range would prune the empty-close-set paths.)
        if before.close_count > 0 {
            let i: usize = kani::any_where(|&i: &usize| i < before.close_count);
            assert!(p.close[i] == before.close[i]);
        }
        // A refusal is never undone.
        if before.outcome.is_err() {
            assert!(p.outcome.is_err());
        }
        if before.cred.is_some() && p.outcome.is_ok() {
            assert!(p.cred == before.cred);
        }
        match r {
            Step::Next(next) => assert!(next == end || (next >= off + CMSG_HDR_LEN && next <= end)),
            Step::Stop => assert!(p.outcome.is_err()),
        }
        // A well-framed SCM_RIGHTS message: each whole descriptor in it
        // is in the close set, in order, after what was there.
        let mlen = u64_at(&buf, off);
        let level = i32::from_le_bytes(word_at(&buf, off + 8));
        let kind = i32::from_le_bytes(word_at(&buf, off + 12));
        if end - off >= CMSG_HDR_LEN
            && mlen >= CMSG_HDR_LEN as u64
            && level == SOL_SOCKET
            && kind == SCM_RIGHTS
        {
            let have = (end - off).min(mlen as usize);
            let n = (have - CMSG_HDR_LEN) / FD_LEN;
            if before.close_count + n <= MAX_SEEN_FDS {
                assert!(p.close_count == before.close_count + n);
                if n > 0 {
                    let j: usize = kani::any_where(|&j: &usize| j < n);
                    let fd = i32::from_le_bytes(word_at(&buf, off + CMSG_HDR_LEN + j * FD_LEN));
                    assert!(p.close[before.close_count + j] == fd);
                }
            }
        }
        kani::cover!(
            p.outcome.is_ok() && p.fd_count > before.fd_count,
            "descriptors accepted"
        );
        kani::cover!(
            p.outcome.is_err() && p.close_count > before.close_count,
            "refused, fds still seen"
        );
        kani::cover!(
            before.close_count == 0 && p.outcome.is_ok() && p.fd_count > 0,
            "a clean first SCM_RIGHTS"
        );
        kani::cover!(
            before.cred.is_some() && p.outcome.is_ok(),
            "credentials kept"
        );
    }

    /// The whole walk on short buffers (up to two headers' worth), any
    /// bytes, any used length: the end-to-end check of the step lemma.
    #[kani::proof]
    #[kani::unwind(25)]
    fn parse_short_buffers() {
        let mut buf = [0u8; CONTROL_LEN];
        let head: [u8; 48] = kani::any();
        buf[..48].copy_from_slice(&head);
        let len: usize = kani::any_where(|&n: &usize| n <= 48);
        let p = parse_fixed(&buf, len);
        assert!(invariant(&p));
        kani::cover!(
            p.outcome.is_ok() && !p.fds().is_empty(),
            "descriptors accepted"
        );
        kani::cover!(
            p.outcome.is_ok() && p.cred.is_some(),
            "credentials accepted"
        );
    }

    /// The base case: the walk starts from a state that keeps the
    /// invariant, and a length over CONTROL_LEN is refused.
    #[kani::proof]
    #[kani::unwind(25)]
    fn the_walk_starts_in_the_invariant() {
        assert!(invariant(&ParsedControl::EMPTY));
        let over: usize = kani::any_where(|&n: &usize| n > CONTROL_LEN);
        let p = parse_fixed(&[0u8; CONTROL_LEN], over);
        assert!(p.outcome.is_err());
    }

    /// `parse` hands `parse_fixed` exactly the bytes it was given, and
    /// their real length.
    #[kani::proof]
    #[kani::unwind(120)]
    fn load_copies_exactly() {
        let bytes: [u8; CONTROL_LEN + 1] = kani::any();
        let n: usize = kani::any_where(|&n: &usize| n <= CONTROL_LEN + 1);
        let control = &bytes[..n];
        let buf = load(control);
        let i: usize = kani::any_where(|&i: &usize| i < CONTROL_LEN);
        if i < n {
            assert!(buf[i] == bytes[i]);
        } else {
            assert!(buf[i] == 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CRED: Ucred = Ucred {
        pid: 4242,
        uid: 1000,
        gid: 100,
    };

    fn hdr(len: u64, level: i32, kind: i32) -> Vec<u8> {
        let mut v = len.to_le_bytes().to_vec();
        v.extend_from_slice(&level.to_le_bytes());
        v.extend_from_slice(&kind.to_le_bytes());
        v
    }

    #[test]
    fn macros_match_the_documented_layout() {
        assert_eq!(cmsg_align(0), Some(0));
        assert_eq!(cmsg_align(1), Some(8));
        assert_eq!(cmsg_align(12), Some(16));
        assert_eq!(cmsg_align(usize::MAX), None);
        assert_eq!(cmsg_len(0), 16);
        assert_eq!(cmsg_len(12), 28);
        assert_eq!(cmsg_len(usize::MAX), usize::MAX);
        assert_eq!(cmsg_space(4), 24);
        assert_eq!(cmsg_space(64), 80);
        assert_eq!(cmsg_space(12), 32);
        assert_eq!(cmsg_space(usize::MAX), usize::MAX);
        assert_eq!(CONTROL_LEN, 80 + 32);
    }

    #[test]
    fn empty_and_well_formed_buffers_are_accepted() {
        let p = parse(&[]);
        assert_eq!(p, ParsedControl::EMPTY);

        let p = parse(&encode(&[], Some(CRED)));
        assert_eq!((p.outcome, p.cred, p.fds()), (Ok(()), Some(CRED), &[][..]));

        let fds: Vec<i32> = (3..19).collect();
        let buf = encode(&fds, Some(CRED));
        assert_eq!(buf.len(), CONTROL_LEN);
        let p = parse(&buf);
        assert_eq!(p.outcome, Ok(()));
        assert_eq!(p.fds(), &fds[..]);
        assert_eq!(p.close_set(), &fds[..]);
        assert_eq!(p.cred, Some(CRED));
        assert_eq!(p.rights_msgs, 1);

        // Credentials first, then rights; and the last message's padding
        // cut off by the end of the buffer.
        let mut buf = encode(&[], Some(CRED));
        buf.extend_from_slice(&encode(&[5], None));
        buf.truncate(buf.len() - 4);
        let p = parse(&buf);
        assert_eq!((p.outcome, p.fds()), (Ok(()), &[5][..]));
    }

    #[test]
    fn refusals_still_list_every_descriptor() {
        // Unknown type before rights: refused, but fd 7 is closed.
        let mut buf = hdr(16, SOL_SOCKET, 3);
        buf.extend_from_slice(&encode(&[7, 8], None));
        let p = parse(&buf);
        assert_eq!(
            p.outcome,
            Err(CmsgError::Unknown {
                level: SOL_SOCKET,
                kind: 3
            })
        );
        assert_eq!(p.close_set(), &[7, 8]);

        // Unknown level.
        let mut buf = hdr(20, 0, SCM_RIGHTS);
        buf.extend_from_slice(&9i32.to_le_bytes());
        assert!(matches!(
            parse(&buf).outcome,
            Err(CmsgError::Unknown { level: 0, .. })
        ));

        // Two SCM_RIGHTS.
        let mut buf = encode(&[3], None);
        buf.extend_from_slice(&encode(&[4, 5], None));
        let p = parse(&buf);
        assert_eq!(p.outcome, Err(CmsgError::TwoRights));
        assert_eq!(p.close_set(), &[3, 4, 5]);

        // Seventeen descriptors.
        let fds: Vec<i32> = (3..20).collect();
        let p = parse(&encode(&fds, None));
        assert_eq!(p.outcome, Err(CmsgError::TooManyFds));
        assert_eq!(p.close_set(), &fds[..]);
        assert_eq!(p.fds().len(), 16);

        // The most a CONTROL_LEN buffer can hold: one header, 24 fds.
        let fds: Vec<i32> = (100..124).collect();
        let buf = encode(&fds, None);
        assert_eq!(buf.len(), CONTROL_LEN);
        let p = parse(&buf);
        assert_eq!(p.close_set(), &fds[..]);
    }

    #[test]
    fn framing_refusals() {
        // cmsg_len below CMSG_LEN(0).
        assert_eq!(
            parse(&hdr(15, SOL_SOCKET, SCM_RIGHTS)).outcome,
            Err(CmsgError::ShortLen)
        );
        // Running past the buffer: the fds that are there are seen.
        let mut buf = hdr(32, SOL_SOCKET, SCM_RIGHTS);
        buf.extend_from_slice(&3i32.to_le_bytes());
        buf.extend_from_slice(&4i32.to_le_bytes());
        let p = parse(&buf);
        assert_eq!(p.outcome, Err(CmsgError::PastEnd));
        assert_eq!(p.close_set(), &[3, 4]);
        // A huge cmsg_len.
        assert_eq!(
            parse(&hdr(u64::MAX, SOL_SOCKET, SCM_RIGHTS)).outcome,
            Err(CmsgError::PastEnd)
        );
        // Rights data not a multiple of 4.
        let mut buf = hdr(22, SOL_SOCKET, SCM_RIGHTS);
        buf.extend_from_slice(&[3, 0, 0, 0, 9, 9]);
        let p = parse(&buf);
        assert_eq!(p.outcome, Err(CmsgError::RightsLength));
        assert_eq!(p.close_set(), &[3]);
        // Credentials of the wrong length, and twice.
        let mut buf = hdr(24, SOL_SOCKET, SCM_CREDENTIALS);
        buf.extend_from_slice(&[0; 8]);
        assert_eq!(parse(&buf).outcome, Err(CmsgError::CredentialsLength));
        let mut buf = encode(&[], Some(CRED));
        buf.extend_from_slice(&encode(&[], Some(CRED)));
        assert_eq!(parse(&buf).outcome, Err(CmsgError::TwoCredentials));
        // Trailing bytes too few for a header.
        for n in 1..CMSG_HDR_LEN {
            let mut buf = encode(&[3], None);
            buf.extend_from_slice(&vec![0; n]);
            let p = parse(&buf);
            assert_eq!(p.outcome, Err(CmsgError::Trailing), "{n}");
            assert_eq!(p.close_set(), &[3]);
        }
        // Too long a buffer.
        let p = parse(&[0; CONTROL_LEN + 1]);
        assert_eq!(p.outcome, Err(CmsgError::Oversize));
    }

    #[test]
    fn encode_rights_matches_the_kernel_layout() {
        let mut out = [0xAAu8; CONTROL_LEN];
        assert_eq!(encode_rights(&[3, 4, 5], &mut out), Some(32));
        assert_eq!(&out[..32], &encode(&[3, 4, 5], None)[..]);
        assert_eq!(&out[..8], &28u64.to_le_bytes());
        let p = parse(&out[..32]);
        assert_eq!(
            (p.outcome, p.fds(), p.fd_count()),
            (Ok(()), &[3, 4, 5][..], 3)
        );

        let sixteen: Vec<i32> = (3..19).collect();
        assert_eq!(encode_rights(&sixteen, &mut out), Some(80));
        assert_eq!(encode_rights(&[], &mut out), None);
        let seventeen: Vec<i32> = (3..20).collect();
        assert_eq!(encode_rights(&seventeen, &mut out), None);
        assert_eq!(encode_rights(&[3], &mut out[..23]), None);
        assert_eq!(encode_rights(&[3], &mut out[..24]), Some(24));
    }

    #[test]
    fn no_panic_on_any_short_buffer() {
        // A cheap exhaustive sweep of the header fields that matter.
        for len in [0u64, 1, 15, 16, 17, 20, 28, 31, 32, 112, 113, 1 << 63] {
            for kind in [0, SCM_RIGHTS, SCM_CREDENTIALS, 3] {
                for tail in 0..40 {
                    let mut buf = hdr(len, SOL_SOCKET, kind);
                    buf.extend((0..tail).map(|i| u8::try_from(i).unwrap()));
                    let p = parse(&buf);
                    assert!(p.fds().len() <= 16 && p.close_set().len() <= MAX_SEEN_FDS);
                }
            }
        }
    }
}
