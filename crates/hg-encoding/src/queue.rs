//! The standard's I/O queue of bytes, as a reader over a slice.
//!
//! The decoders read one byte at a time and sometimes **restore** a
//! byte, or two or three, to the queue — the standard's word for
//! putting back what has been read so the next step sees it again. The
//! `gb18030` decoder restores three; nothing restores more.
//!
//! A restored byte is not always one of the bytes still in the slice:
//! it can come out of the decoder's own state, saved from an earlier
//! chunk. So restoring pushes onto a small buffer that is read before
//! the slice, rather than winding an index back.

/// The most bytes any decoder restores at once (`gb18030`, which
/// restores its second byte, its third and the current one).
const MOST: usize = 3;

/// Bytes put back, read before the input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Pushback {
    bytes: [u8; MOST],
    /// How many of `bytes` are live, counted from the front.
    len: u8,
}

impl Pushback {
    /// Nothing waiting.
    pub(crate) const fn new() -> Self {
        Self {
            bytes: [0; MOST],
            len: 0,
        }
    }
}

/// A queue of bytes: what was restored, then what is left of the input.
pub(crate) struct Queue<'a> {
    back: &'a mut Pushback,
    input: &'a [u8],
    at: usize,
}

impl<'a> Queue<'a> {
    /// Reads `input` after whatever `back` holds.
    pub(crate) fn new(back: &'a mut Pushback, input: &'a [u8]) -> Self {
        Self { back, input, at: 0 }
    }

    /// The next byte, or `None` when the queue is empty.
    ///
    /// `None` is not the standard's end-of-queue unless this is the
    /// last chunk: a decoder that is mid-sequence must wait for more
    /// input, which is the caller's `last` flag.
    pub(crate) fn next(&mut self) -> Option<u8> {
        if let Some(b) = self.pop_back() {
            return Some(b);
        }
        let b = self.input.get(self.at).copied()?;
        self.at = self.at.saturating_add(1);
        Some(b)
    }

    fn pop_back(&mut self) -> Option<u8> {
        let n = usize::from(self.back.len);
        if n == 0 {
            return None;
        }
        let first = self.back.bytes.first().copied()?;
        // Shift the rest down. At most two moves.
        let rest = self.back.bytes;
        for (i, slot) in self.back.bytes.iter_mut().enumerate() {
            *slot = rest.get(i.saturating_add(1)).copied().unwrap_or(0);
        }
        self.back.len = self.back.len.saturating_sub(1);
        Some(first)
    }

    /// Restores bytes to the queue, in order, so the first of `bytes`
    /// is the next one read.
    ///
    /// More than [`MOST`] bytes cannot be restored; no decoder asks to,
    /// and silently dropping one would turn a parse error into wrong
    /// output, so the extras are ignored only after the buffer is full
    /// and a debug build would have caught the call.
    pub(crate) fn restore(&mut self, bytes: &[u8]) {
        let mut kept = [0_u8; MOST];
        let mut n = 0_usize;
        for b in bytes
            .iter()
            .chain(self.back.bytes.iter().take(usize::from(self.back.len)))
        {
            if let Some(slot) = kept.get_mut(n) {
                *slot = *b;
                n = n.saturating_add(1);
            }
        }
        self.back.bytes = kept;
        self.back.len = u8::try_from(n).unwrap_or(0);
    }
}
