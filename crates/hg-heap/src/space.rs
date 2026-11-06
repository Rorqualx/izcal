//! Immix blocks and lines (heap design H-5, H-6). The cage is divided
//! into 32 KiB blocks of 128-byte lines, committed in 1 MiB chunks from
//! the bottom up. Every block's state, line marks and object mark bits
//! live here, outside the cage, so corrupted cage contents can never
//! corrupt the collector's own bookkeeping (H-2).

use hg_substrate::MemoryKind;

use crate::cage::{Cage, CageError};

/// Bytes per block.
pub(crate) const BLOCK: u64 = 32 * 1024;
/// Bytes per line.
pub(crate) const LINE: u64 = 128;
/// Lines per block.
const LINES: usize = 256;
/// Bytes committed at a time: a multiple of every page size Izcal runs on.
pub(crate) const CHUNK: u64 = 1 << 20;
/// Blocks per chunk.
const CHUNK_BLOCKS: usize = 32;
/// Objects larger than this go to the large-object space.
pub(crate) const MAX_SMALL: u64 = 8 * 1024;
/// 8-byte granules per block: one mark bit each.
const GRANULES: usize = 4096;

/// A block's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    /// Committed and empty.
    Free,
    /// Holds small objects (some lines may be free).
    Small,
    /// The first block of a large object spanning `n` blocks.
    LargeHead(u32),
    /// A later block of a large object.
    LargeTail,
}

#[derive(Debug, Clone)]
struct Meta {
    state: State,
    /// Lines holding a live (or, between cycles, allocated) object.
    lines: [u64; LINES / 64],
    /// One bit per 8-byte granule: the object starting there is marked.
    marks: Box<[u64; GRANULES / 64]>,
    /// Live lines found by the last canonical cycle, for evacuation.
    live_lines: u16,
}

impl Meta {
    fn new() -> Self {
        Self {
            state: State::Free,
            lines: [0; LINES / 64],
            marks: Box::new([0; GRANULES / 64]),
            live_lines: 0,
        }
    }
}

/// A bump region: `[cursor, limit)` in cage offsets.
#[derive(Debug, Clone, Copy, Default)]
struct Bump {
    cursor: u64,
    limit: u64,
}

impl Bump {
    fn take(&mut self, size: u64) -> Option<u64> {
        let end = self.cursor.checked_add(size)?;
        if end > self.limit {
            return None;
        }
        let at = self.cursor;
        self.cursor = end;
        Some(at)
    }
}

/// Every block of the cage that has been committed.
#[derive(Debug)]
pub(crate) struct Space {
    blocks: Vec<Meta>,
    /// Holes in recyclable blocks, then fresh blocks.
    small: Bump,
    /// Medium objects that do not fit the current hole (Immix overflow).
    overflow: Bump,
    /// Copies made by evacuation, into blocks no cycle is evacuating.
    to_space: Bump,
    /// The next block to look at for holes, after a cycle.
    recycle_from: usize,
    kind: MemoryKind,
}

/// Why an allocation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    /// The cage is full or a commit failed.
    OutOfMemory,
}

impl From<CageError> for AllocError {
    fn from(_: CageError) -> Self {
        Self::OutOfMemory
    }
}

const fn block_of(off: u64) -> usize {
    (off / BLOCK) as usize
}

const fn block_start(b: usize) -> u64 {
    b as u64 * BLOCK
}

impl Space {
    /// An empty space; its blocks are charged to `kind`.
    pub(crate) fn new(kind: MemoryKind) -> Self {
        Self {
            blocks: Vec::new(),
            small: Bump::default(),
            overflow: Bump::default(),
            to_space: Bump::default(),
            recycle_from: 0,
            kind,
        }
    }

    /// Commits the next chunk. Block 0 is never handed out, so no object
    /// lives at cage offset 0.
    fn grow(&mut self, cage: &Cage) -> Result<(), AllocError> {
        let start = self.blocks.len() as u64 * BLOCK;
        if start.checked_add(CHUNK).is_none_or(|end| end > cage.len()) {
            return Err(AllocError::OutOfMemory);
        }
        cage.commit(start, CHUNK, self.kind)?;
        for _ in 0..CHUNK_BLOCKS {
            self.blocks.push(Meta::new());
        }
        if start == 0
            && let Some(first) = self.blocks.first_mut()
        {
            first.state = State::LargeTail; // reserved, never free
        }
        Ok(())
    }

    /// The lowest free block (committing a chunk if none is free), marked
    /// as holding small objects. `skip` excludes blocks being evacuated.
    fn fresh_block(&mut self, cage: &Cage, skip: &[bool]) -> Result<usize, AllocError> {
        loop {
            let found = self.blocks.iter().enumerate().position(|(i, m)| {
                m.state == State::Free && !skip.get(i).copied().unwrap_or(false)
            });
            if let Some(b) = found {
                if let Some(m) = self.blocks.get_mut(b) {
                    m.state = State::Small;
                }
                return Ok(b);
            }
            self.grow(cage)?;
        }
    }

    /// The next hole (a run of free lines) at or after `self.recycle_from`
    /// in a block holding small objects.
    fn next_hole(&mut self) -> Option<Bump> {
        while let Some(m) = self.blocks.get(self.recycle_from) {
            let b = self.recycle_from;
            self.recycle_from += 1;
            if m.state != State::Small || b == 0 {
                continue;
            }
            // The longest-first rule is Immix's refinement; the first hole
            // keeps the order simple and deterministic.
            let mut line = 0usize;
            while line < LINES {
                if is_set(&m.lines, line) {
                    line += 1;
                    continue;
                }
                let start = line;
                while line < LINES && !is_set(&m.lines, line) {
                    line += 1;
                }
                // Revisit this block for its later holes.
                self.recycle_from = b;
                let mut hole = Bump {
                    cursor: block_start(b) + start as u64 * LINE,
                    limit: block_start(b) + line as u64 * LINE,
                };
                // Mark the hole's lines used, so the next search skips it.
                if let Some(m) = self.blocks.get_mut(b) {
                    for l in start..line {
                        set(&mut m.lines, l);
                    }
                }
                hole.cursor = hole.cursor.max(8);
                return Some(hole);
            }
        }
        None
    }

    /// Allocates `size` bytes (a multiple of 8) for a new object.
    pub(crate) fn alloc(&mut self, cage: &Cage, size: u64) -> Result<u64, AllocError> {
        if size > MAX_SMALL {
            return self.alloc_large(cage, size);
        }
        if let Some(at) = self.small.take(size) {
            return Ok(self.used(at, size));
        }
        if size > LINE {
            // A medium object that does not fit the current hole goes to
            // the overflow block rather than wasting the hole.
            if let Some(at) = self.overflow.take(size) {
                return Ok(self.used(at, size));
            }
            let b = self.fresh_block(cage, &[])?;
            self.overflow = whole_block(b);
            let at = self.overflow.take(size).ok_or(AllocError::OutOfMemory)?;
            return Ok(self.used(at, size));
        }
        while let Some(hole) = self.next_hole() {
            self.small = hole;
            if let Some(at) = self.small.take(size) {
                return Ok(self.used(at, size));
            }
        }
        let b = self.fresh_block(cage, &[])?;
        self.small = whole_block(b);
        let at = self.small.take(size).ok_or(AllocError::OutOfMemory)?;
        Ok(self.used(at, size))
    }

    /// Allocates `size` bytes in a block that `evacuating` does not name.
    pub(crate) fn alloc_copy(
        &mut self,
        cage: &Cage,
        size: u64,
        evacuating: &[bool],
    ) -> Result<u64, AllocError> {
        if let Some(at) = self.to_space.take(size) {
            return Ok(self.used(at, size));
        }
        let b = self.fresh_block(cage, evacuating)?;
        self.to_space = whole_block(b);
        let at = self.to_space.take(size).ok_or(AllocError::OutOfMemory)?;
        Ok(self.used(at, size))
    }

    /// Records `[at, at + size)` as used and returns `at`.
    fn used(&mut self, at: u64, size: u64) -> u64 {
        self.mark_lines(at, size);
        at
    }

    fn alloc_large(&mut self, cage: &Cage, size: u64) -> Result<u64, AllocError> {
        let n = usize::try_from(size.div_ceil(BLOCK)).map_err(|_| AllocError::OutOfMemory)?;
        loop {
            // The lowest run of n free blocks.
            let mut run = 0usize;
            let mut found = None;
            for (i, m) in self.blocks.iter().enumerate() {
                if m.state == State::Free {
                    run += 1;
                    if run == n {
                        found = Some(i + 1 - n);
                        break;
                    }
                } else {
                    run = 0;
                }
            }
            if let Some(first) = found {
                for (k, m) in self.blocks.iter_mut().skip(first).take(n).enumerate() {
                    m.state = if k == 0 {
                        State::LargeHead(u32::try_from(n).unwrap_or(u32::MAX))
                    } else {
                        State::LargeTail
                    };
                }
                return Ok(block_start(first));
            }
            self.grow(cage)?;
        }
    }

    fn mark_lines(&mut self, at: u64, size: u64) {
        let b = block_of(at);
        let Some(m) = self.blocks.get_mut(b) else {
            return;
        };
        if m.state != State::Small {
            return;
        }
        let first = ((at % BLOCK) / LINE) as usize;
        let last = (((at % BLOCK) + size.max(1) - 1) / LINE) as usize;
        for l in first..=last.min(LINES - 1) {
            set(&mut m.lines, l);
        }
    }

    /// Whether `off` can be the start of a live object: inside a block
    /// that holds small objects (not block 0), or the start of a large
    /// object. Anything else read from a slot is not followed (H-2).
    pub(crate) fn is_object(&self, off: u64) -> bool {
        match self.blocks.get(block_of(off)).map(|m| m.state) {
            Some(State::Small) => block_of(off) != 0,
            Some(State::LargeHead(_)) => off.is_multiple_of(BLOCK),
            Some(State::Free | State::LargeTail) | None => false,
        }
    }

    /// The bytes an object at `off` may occupy at most: to the end of its
    /// block, or of its large-object run.
    pub(crate) fn room(&self, off: u64) -> u64 {
        match self.blocks.get(block_of(off)).map(|m| m.state) {
            Some(State::LargeHead(n)) => u64::from(n) * BLOCK,
            Some(State::Small) => BLOCK - off % BLOCK,
            _ => 0,
        }
    }

    /// Whether `off` is in a small-object block.
    pub(crate) fn is_small(&self, off: u64) -> bool {
        self.blocks
            .get(block_of(off))
            .is_some_and(|m| m.state == State::Small)
    }

    // --- Collection -------------------------------------------------------

    /// Clears every mark and line mark, ready for a cycle.
    pub(crate) fn start_cycle(&mut self) {
        for m in &mut self.blocks {
            m.marks.fill(0);
            if m.state == State::Small {
                m.lines = [0; LINES / 64];
            }
        }
        self.small = Bump::default();
        self.overflow = Bump::default();
        self.to_space = Bump::default();
    }

    /// The blocks to evacuate: every small-object block when `all`, else
    /// the fragmented ones (few live lines at the last canonical cycle).
    pub(crate) fn candidates(&self, all: bool) -> Vec<bool> {
        self.blocks
            .iter()
            .enumerate()
            .map(|(i, m)| {
                i != 0
                    && m.state == State::Small
                    && (all || (m.live_lines > 0 && usize::from(m.live_lines) <= LINES / 4))
            })
            .collect()
    }

    /// Marks the object at `off` (of `size` bytes); returns whether it was
    /// already marked.
    pub(crate) fn mark(&mut self, off: u64, size: u64) -> bool {
        let b = block_of(off);
        let g = ((off % BLOCK) / 8) as usize;
        let Some(m) = self.blocks.get_mut(b) else {
            return true;
        };
        if is_set(&m.marks[..], g) {
            return true;
        }
        set(&mut m.marks[..], g);
        if m.state == State::Small {
            self.mark_lines(off, size);
        }
        false
    }

    /// Whether the object at `off` is marked.
    pub(crate) fn is_marked(&self, off: u64) -> bool {
        let g = ((off % BLOCK) / 8) as usize;
        self.blocks
            .get(block_of(off))
            .is_some_and(|m| is_set(&m.marks[..], g))
    }

    /// Frees what the cycle left unmarked, and the evacuated blocks.
    /// `canonical` cycles record each block's live lines for the next
    /// cycle's evacuation choice. Freed memory is overwritten with
    /// `poison`, so a stale reference reads garbage, never old data.
    pub(crate) fn sweep(&mut self, cage: &Cage, evacuated: &[bool], canonical: bool, poison: u64) {
        let n = self.blocks.len();
        for b in 1..n {
            let Some(m) = self.blocks.get(b) else {
                continue;
            };
            match m.state {
                State::Small => {
                    let live = count(&m.lines);
                    let gone = evacuated.get(b).copied().unwrap_or(false) || live == 0;
                    if let Some(m) = self.blocks.get_mut(b) {
                        if canonical {
                            m.live_lines = u16::try_from(live).unwrap_or(u16::MAX);
                        }
                        if gone {
                            m.state = State::Free;
                            m.live_lines = 0;
                            m.lines = [0; LINES / 64];
                        }
                    }
                    if gone {
                        cage.fill(block_start(b), BLOCK, poison);
                    } else {
                        // Poison the free lines too.
                        for l in 0..LINES {
                            let used = self.blocks.get(b).is_some_and(|m| is_set(&m.lines, l));
                            if !used {
                                cage.fill(block_start(b) + l as u64 * LINE, LINE, poison);
                            }
                        }
                    }
                }
                State::LargeHead(k) => {
                    if !is_set(&m.marks[..], 0) {
                        let k = usize::try_from(k).unwrap_or(0);
                        for j in b..(b + k).min(n) {
                            if let Some(m) = self.blocks.get_mut(j) {
                                m.state = State::Free;
                            }
                        }
                        cage.fill(block_start(b), k as u64 * BLOCK, poison);
                    }
                }
                State::Free | State::LargeTail => {}
            }
        }
        self.recycle_from = 0;
        self.small = Bump::default();
        self.overflow = Bump::default();
        self.to_space = Bump::default();
    }

    /// Blocks committed, and blocks holding objects.
    pub(crate) fn census(&self) -> (usize, usize) {
        let used = self
            .blocks
            .iter()
            .skip(1)
            .filter(|m| m.state != State::Free)
            .count();
        (self.blocks.len(), used)
    }
}

fn whole_block(b: usize) -> Bump {
    let start = block_start(b);
    Bump {
        cursor: start.max(8),
        limit: start + BLOCK,
    }
}

fn is_set(bits: &[u64], i: usize) -> bool {
    bits.get(i / 64).is_some_and(|w| w & (1 << (i % 64)) != 0)
}

fn set(bits: &mut [u64], i: usize) {
    if let Some(w) = bits.get_mut(i / 64) {
        *w |= 1 << (i % 64);
    }
}

fn count(bits: &[u64]) -> usize {
    bits.iter().map(|w| w.count_ones() as usize).sum()
}
