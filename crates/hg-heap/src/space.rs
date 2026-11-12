//! Immix blocks and lines (heap design H-5, H-6), for one agent heap.
//! The process's cage hands each heap whole 1 MiB chunks; the heap
//! divides them into 32 KiB blocks of 128-byte lines. Blocks are numbered
//! locally, in the order the heap acquired its chunks, so a heap's layout
//! never depends on what other agents did (JS-D17: one cage, each agent
//! heap owns its own blocks). Every block's state, line marks and object
//! mark bits live here, outside the cage, so corrupted cage contents can
//! never corrupt the collector's own bookkeeping (H-2); and a slot that
//! points into a chunk another heap owns is never followed.

use std::collections::BTreeMap;

use crate::cage::{CHUNK, Cage};

/// Bytes per block.
pub(crate) const BLOCK: u64 = 32 * 1024;
/// Bytes per line.
pub(crate) const LINE: u64 = 128;
/// Lines per block.
const LINES: usize = 256;
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

/// One agent heap's blocks.
#[derive(Debug)]
pub(crate) struct Space {
    /// The cage offset of each chunk, in the order the heap acquired them.
    chunks: Vec<u64>,
    /// Cage chunk index → local chunk index.
    owned: BTreeMap<u64, usize>,
    blocks: Vec<Meta>,
    /// Holes in recyclable blocks, then fresh blocks.
    small: Bump,
    /// Medium objects that do not fit the current hole (Immix overflow).
    overflow: Bump,
    /// Copies made by evacuation, into blocks no cycle is evacuating.
    to_space: Bump,
    /// The next local block to look at for holes, after a cycle.
    recycle_from: usize,
}

/// Why an allocation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    /// The cage is full or a commit failed.
    OutOfMemory,
}

impl Space {
    /// An empty space.
    pub(crate) fn new() -> Self {
        Self {
            chunks: Vec::new(),
            owned: BTreeMap::new(),
            blocks: Vec::new(),
            small: Bump::default(),
            overflow: Bump::default(),
            to_space: Bump::default(),
            recycle_from: 0,
        }
    }

    /// The cage offset of local block `b`.
    fn start(&self, b: usize) -> u64 {
        let c = self.chunks.get(b / CHUNK_BLOCKS).copied().unwrap_or(0);
        c + (b % CHUNK_BLOCKS) as u64 * BLOCK
    }

    /// The local block holding cage offset `off`, if this heap owns it.
    fn local(&self, off: u64) -> Option<usize> {
        let c = self.owned.get(&(off / CHUNK))?;
        Some(c * CHUNK_BLOCKS + ((off % CHUNK) / BLOCK) as usize)
    }

    fn meta(&self, off: u64) -> Option<&Meta> {
        self.blocks.get(self.local(off)?)
    }

    /// Takes `n` adjacent chunks from the cage and adds their blocks.
    fn grow(&mut self, cage: &Cage, n: u64) -> Result<usize, AllocError> {
        let at = cage.acquire(n).map_err(|_| AllocError::OutOfMemory)?;
        let first = self.blocks.len();
        for k in 0..n {
            let off = at + k * CHUNK;
            self.owned.insert(off / CHUNK, self.chunks.len());
            self.chunks.push(off);
            for _ in 0..CHUNK_BLOCKS {
                self.blocks.push(Meta::new());
            }
        }
        Ok(first)
    }

    /// Gives every chunk back to the cage: the heap's whole memory is
    /// returned without a trace (JS spec 6.5, navigation away).
    pub(crate) fn release(&mut self, cage: &Cage) {
        for &c in &self.chunks {
            cage.release(c);
        }
        self.chunks.clear();
        self.owned.clear();
        self.blocks.clear();
    }

    /// The lowest free local block (taking a chunk if none is free),
    /// marked as holding small objects. `skip` excludes blocks being
    /// evacuated.
    fn fresh_block(&mut self, cage: &Cage, skip: &[bool]) -> Result<usize, AllocError> {
        let found =
            self.blocks.iter().enumerate().position(|(i, m)| {
                m.state == State::Free && !skip.get(i).copied().unwrap_or(false)
            });
        let b = match found {
            Some(b) => b,
            None => self.grow(cage, 1)?,
        };
        if let Some(m) = self.blocks.get_mut(b) {
            m.state = State::Small;
        }
        Ok(b)
    }

    /// The next hole (a run of free lines) at or after `recycle_from` in
    /// a block holding small objects.
    fn next_hole(&mut self) -> Option<Bump> {
        while let Some(m) = self.blocks.get(self.recycle_from) {
            let b = self.recycle_from;
            self.recycle_from += 1;
            if m.state != State::Small {
                continue;
            }
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
                let base = self.start(b);
                if let Some(m) = self.blocks.get_mut(b) {
                    for l in start..line {
                        set(&mut m.lines, l);
                    }
                }
                return Some(Bump {
                    cursor: base + start as u64 * LINE,
                    limit: base + line as u64 * LINE,
                });
            }
        }
        None
    }

    fn whole(&self, b: usize) -> Bump {
        let s = self.start(b);
        Bump {
            cursor: s,
            limit: s + BLOCK,
        }
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
            self.overflow = self.whole(b);
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
        self.small = self.whole(b);
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
        self.to_space = self.whole(b);
        let at = self.to_space.take(size).ok_or(AllocError::OutOfMemory)?;
        Ok(self.used(at, size))
    }

    /// Records `[at, at + size)` as used and returns `at`.
    fn used(&mut self, at: u64, size: u64) -> u64 {
        self.mark_lines(at, size);
        at
    }

    /// A large object: `n` free blocks inside one chunk, or for more than
    /// a chunk's worth, a fresh run of adjacent chunks.
    fn alloc_large(&mut self, cage: &Cage, size: u64) -> Result<u64, AllocError> {
        let n = usize::try_from(size.div_ceil(BLOCK)).map_err(|_| AllocError::OutOfMemory)?;
        let first = if n > CHUNK_BLOCKS {
            let chunks =
                u64::try_from(n.div_ceil(CHUNK_BLOCKS)).map_err(|_| AllocError::OutOfMemory)?;
            self.grow(cage, chunks)?
        } else {
            match self.free_run(n) {
                Some(f) => f,
                None => self.grow(cage, 1)?,
            }
        };
        for (k, m) in self.blocks.iter_mut().skip(first).take(n).enumerate() {
            m.state = if k == 0 {
                State::LargeHead(u32::try_from(n).unwrap_or(u32::MAX))
            } else {
                State::LargeTail
            };
        }
        Ok(self.start(first))
    }

    /// The lowest run of `n` free blocks inside one chunk.
    fn free_run(&self, n: usize) -> Option<usize> {
        for (c, chunk) in self.blocks.chunks(CHUNK_BLOCKS).enumerate() {
            let mut run = 0usize;
            for (i, m) in chunk.iter().enumerate() {
                if m.state == State::Free {
                    run += 1;
                    if run == n {
                        return Some(c * CHUNK_BLOCKS + i + 1 - n);
                    }
                } else {
                    run = 0;
                }
            }
        }
        None
    }

    fn mark_lines(&mut self, at: u64, size: u64) {
        let Some(b) = self.local(at) else {
            return;
        };
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

    /// Whether `off` can be the start of a live object of this heap:
    /// inside a block of this heap that holds small objects, or the start
    /// of one of its large objects. Anything else read from a slot is not
    /// followed (H-2), including any offset in another heap's chunks.
    pub(crate) fn is_object(&self, off: u64) -> bool {
        match self.meta(off).map(|m| m.state) {
            Some(State::Small) => true,
            Some(State::LargeHead(_)) => off.is_multiple_of(BLOCK),
            Some(State::Free | State::LargeTail) | None => false,
        }
    }

    /// The bytes an object at `off` may occupy at most: to the end of its
    /// block, or of its large-object run.
    pub(crate) fn room(&self, off: u64) -> u64 {
        match self.meta(off).map(|m| m.state) {
            Some(State::LargeHead(n)) => u64::from(n) * BLOCK,
            Some(State::Small) => BLOCK - off % BLOCK,
            _ => 0,
        }
    }

    /// Whether `off` lies in a chunk this heap owns. A slot naming a
    /// chunk of another agent heap, or one the cage has not handed out,
    /// is never followed (heap design H-12); the verifier uses this to
    /// tell that allowed case from a reference into this heap's own freed
    /// memory, which is a collector bug.
    pub(crate) fn owns(&self, off: u64) -> bool {
        self.owned.contains_key(&(off / CHUNK))
    }

    /// Whether `off` is in a small-object block of this heap.
    pub(crate) fn is_small(&self, off: u64) -> bool {
        self.meta(off).is_some_and(|m| m.state == State::Small)
    }

    /// The local block of `off`, for evacuation's candidate test.
    pub(crate) fn block_index(&self, off: u64) -> Option<usize> {
        self.local(off)
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

    /// The local blocks to evacuate: every small-object block when `all`,
    /// else the fragmented ones (few live lines at the last canonical
    /// cycle).
    pub(crate) fn candidates(&self, all: bool) -> Vec<bool> {
        self.blocks
            .iter()
            .map(|m| {
                m.state == State::Small
                    && (all || (m.live_lines > 0 && usize::from(m.live_lines) <= LINES / 4))
            })
            .collect()
    }

    /// Marks the object at `off` (of `size` bytes); returns whether it was
    /// already marked.
    pub(crate) fn mark(&mut self, off: u64, size: u64) -> bool {
        let g = ((off % BLOCK) / 8) as usize;
        let Some(b) = self.local(off) else {
            return true;
        };
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

    /// Checks that every line the object at `off` of `size` bytes covers
    /// is marked in its block, which is what stops the allocator handing
    /// out memory inside a live object. Large objects hold whole blocks
    /// and have no line marks, so they always pass.
    ///
    /// # Errors
    /// The first unmarked line, counted from the start of the block.
    pub(crate) fn lines_marked(&self, off: u64, size: u64) -> Result<(), usize> {
        let Some(m) = self.meta(off) else {
            return Ok(()); // not this heap's memory; `is_object` reports that
        };
        if m.state != State::Small {
            return Ok(());
        }
        let first = ((off % BLOCK) / LINE) as usize;
        let last = (((off % BLOCK) + size.max(1) - 1) / LINE) as usize;
        for l in first..=last.min(LINES - 1) {
            if !is_set(&m.lines, l) {
                return Err(l);
            }
        }
        Ok(())
    }

    /// Whether the object at `off` is marked.
    pub(crate) fn is_marked(&self, off: u64) -> bool {
        let g = ((off % BLOCK) / 8) as usize;
        self.meta(off).is_some_and(|m| is_set(&m.marks[..], g))
    }

    /// Frees what the cycle left unmarked, and the evacuated blocks.
    /// `canonical` cycles record each block's live lines for the next
    /// cycle's evacuation choice. Freed memory is overwritten with
    /// `poison`, so a stale reference reads garbage, never old data.
    pub(crate) fn sweep(&mut self, cage: &Cage, evacuated: &[bool], canonical: bool, poison: u64) {
        let n = self.blocks.len();
        for b in 0..n {
            let base = self.start(b);
            let Some(m) = self.blocks.get(b) else {
                continue;
            };
            match m.state {
                State::Small => {
                    let live = count(&m.lines);
                    let gone = evacuated.get(b).copied().unwrap_or(false) || live == 0;
                    let lines = m.lines;
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
                        cage.fill(base, BLOCK, poison);
                    } else {
                        for l in 0..LINES {
                            if !is_set(&lines, l) {
                                cage.fill(base + l as u64 * LINE, LINE, poison);
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
                        for j in b..(b + k).min(n) {
                            cage.fill(self.start(j), BLOCK, poison);
                        }
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

    /// The cage offsets of this heap's chunks, in the order it acquired
    /// them: where a fuzzer with an in-cage write finds memory that is
    /// committed and in use (JS-10 (c)). A uniform offset into a 16 GiB
    /// cage almost never hits one.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn chunk_offsets(&self) -> &[u64] {
        &self.chunks
    }

    /// Blocks owned, and blocks holding objects.
    pub(crate) fn census(&self) -> (usize, usize) {
        let used = self
            .blocks
            .iter()
            .filter(|m| m.state != State::Free)
            .count();
        (self.blocks.len(), used)
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
