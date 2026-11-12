//! The in-cage corruption API (JS spec 6.1; JS-10 (c)) and the harness
//! that drives it.
//!
//! **The premise being tested.** Security scope ATK-2 assumes an
//! attacker who can read and write their own process's memory, the cage
//! included, so a type rule about what Rust *can* store in the cage is
//! necessary but not sufficient. The property that has to hold is H-2:
//! nothing outside the cage trusts a value it reads from the cage, and
//! so a cage holding arbitrary words gives *wrong answers inside the
//! cage* and never an access to memory outside it, its side tables or a
//! checked table entry. JS spec 6.1 says this is "tested rather than
//! asserted", and JS-10 (c) is the test: a fuzzer with arbitrary in-cage
//! read and write runs against a working heap, and **0 out-of-cage
//! faults** is the pass.
//!
//! **What this module exposes** is [`Corrupt`], reached through
//! [`Mutator::corrupt`](crate::Mutator::corrupt) in a test build only
//! (`cfg(test)`, or the `testing` feature for the tests of crates built
//! on the heap, which is how JS-M1's fuzzer will drive it with script
//! running). It is
//! deliberately *not* a way out of the cage: every access it performs
//! goes through the same masked, aligned, atomic accessors of the `cage`
//! module that the collector uses, so what it grants is an arbitrary
//! *value* at an arbitrary *in-cage offset*, which is exactly the
//! attacker of the premise, and nothing more.
//!
//! **How a violation is seen.** The claim is negative — no access
//! outside the cage — so the harness needs detectors rather than
//! assertions:
//!
//! 1. **The guard ring.** Every `TestMemory` reservation is ringed with
//!    a canary ([`GUARD`](crate::cage::test_memory::GUARD) bytes either
//!    side, a chunk's worth), checked after every run. A write that left
//!    the cage by a block or a chunk lands in the ring and is reported
//!    with the distance it reached. This detector runs in the ordinary
//!    `cargo test` on all three OSes.
//! 2. **Miri**, which checks the provenance and the bounds of every
//!    access the cage module makes through its raw base pointer, and so
//!    catches a write that left the reservation altogether, as well as
//!    a race between the two threads of the two-heap test. The `miri`
//!    CI job runs this module's harness (`--lib`). It earned its place
//!    on the first run: see [`Corrupt::confine`] for what it found.
//! 3. **A panic is a fault.** A corrupted cage reaching a bounds check,
//!    an `unwrap` or an overflow in the collector would abort the
//!    renderer, so each run is caught and classified rather than left to
//!    the test harness.
//! 4. **A hang is a fault.** Each run has a wall-clock budget; a cycle
//!    that does not finish is a finding, not a slow test.
//!
//! **The heap verifier is off during corruption**, and only during it,
//! for the reason the `verify` module states: it assumes the cage holds
//! what the collector and the mutator put there, which is the assumption
//! this harness exists to violate.

use std::marker::PhantomData;
use std::sync::Arc;

use crate::cage::Cage;
use crate::slot::Slot;

/// Arbitrary read and write inside the cage, as JS spec 6.1's test-only
/// corruption API grants it. Offsets are *arbitrary*: the cage masks
/// every one into its own reservation, so no value of `offset` can name
/// memory outside it, and that masking is itself part of what JS-10 (c)
/// measures.
///
/// The handle is `!Send` and `!Sync`, by the `PhantomData` below. Both
/// of its fields are `Sync`, so without that marker the type would be
/// `Send` and `Sync` by inference, and a test could move it to another
/// thread -- which is the handle [`Corrupt::confine`] records as
/// withdrawn, and which Miri showed to be undefined against a live
/// heap. A doc comment cannot withdraw it; the marker can.
#[derive(Debug)]
pub struct Corrupt<'a> {
    cage: &'a Arc<Cage>,
    chunks: &'a [u64],
    /// Keeps the handle on the thread that made it: see above.
    not_send: PhantomData<*const ()>,
}

impl<'a> Corrupt<'a> {
    pub(crate) const fn new(cage: &'a Arc<Cage>, chunks: &'a [u64]) -> Self {
        Self {
            cage,
            chunks,
            not_send: PhantomData,
        }
    }

    /// The cage's size: the range an offset is masked into.
    #[must_use]
    pub fn cage_bytes(&self) -> u64 {
        self.cage.len()
    }

    /// The cage offsets of the chunks this heap holds. A uniformly drawn
    /// offset into a 16 GiB cage lands on committed memory with
    /// vanishing probability, so a fuzzer that only drew uniformly would
    /// test almost nothing; these are where the objects are.
    #[must_use]
    pub const fn chunks(&self) -> &[u64] {
        self.chunks
    }

    /// The cage offset a slot names, for aiming a write at the object a
    /// reference points to. `None` for a small integer.
    #[must_use]
    pub const fn target(s: Slot) -> Option<u64> {
        s.as_reference()
    }

    /// Reads the 32-bit word at `offset`.
    #[must_use]
    pub fn read32(&self, offset: u64) -> u32 {
        self.cage.load32(offset)
    }

    /// Writes the 32-bit word `v` at `offset`.
    pub fn write32(&self, offset: u64, v: u32) {
        self.cage.store32(offset, v);
    }

    /// Reads the 64-bit word at `offset`.
    #[must_use]
    pub fn read64(&self, offset: u64) -> u64 {
        self.cage.load64(offset)
    }

    /// Writes the 64-bit word `v` at `offset`.
    pub fn write64(&self, offset: u64, v: u64) {
        self.cage.store64(offset, v);
    }

    /// Writes `pattern` over `len` bytes (rounded down to a multiple of
    /// 8) at `offset`: the sweep of a whole object or line that an
    /// attacker with a write primitive does rather than one word.
    pub fn fill(&self, offset: u64, len: u64, pattern: u64) {
        self.cage.fill(offset, len & !7, pattern);
    }

    /// Confines `offset` to this heap's own chunks, folding anything
    /// outside them into the first one. Two agent heaps share a cage
    /// (JS-D17), and a write into the chunks of the *other* heap would
    /// overlap that heap's accesses at whatever width it uses there.
    ///
    /// **Why that matters, measured 10 November 2026.** A cage word may
    /// be reached at two widths — 32 bits for slots, headers, length
    /// words and tails, 64 bits for raw words, node slots, and the
    /// `copy` and `fill` that evacuation and the sweep perform over
    /// whole objects and lines. Overlapping unsynchronised atomic
    /// accesses of *different* sizes are undefined whatever their
    /// ordering. A first version of this API handed out a `Send` handle
    /// so a second thread could corrupt a live heap's cage while it
    /// collected; Miri reported "overlapping unsynchronized atomic
    /// accesses must use the same access size", correctly, and the
    /// handle was withdrawn rather than documented around — a fuzzer
    /// cannot know which width a given word is reached at, so there was
    /// no sound way to use it. What remains is the concurrency the
    /// design does license: one thread per agent heap, each in its own
    /// chunks, which is what this method is for.
    #[must_use]
    pub fn confine(&self, offset: u64) -> Option<u64> {
        if self
            .chunks
            .iter()
            .any(|&b| offset >= b && offset < b + crate::cage::CHUNK)
        {
            return Some(offset);
        }
        self.chunks
            .first()
            .map(|&base| base + offset % crate::cage::CHUNK)
    }
}

#[cfg(test)]
mod harness {
    //! JS-10 (c)'s harness: a seeded fuzzer with [`Corrupt`]'s arbitrary
    //! in-cage write, running against a heap that allocates, mutates,
    //! collects, evacuates, clears weak slots and traces a node space.
    //!
    //! A run is **not** checked against an expected answer. Under
    //! arbitrary cage contents the answers are meaningless by design
    //! (H-2 promises they are wrong, not that they are anything in
    //! particular), so what the run is checked for is the four faults in
    //! this module's header: a broken guard ring, a sanitizer report
    //! (Miri), a panic and a hang. Pass is 0 of them, which is JS-10's
    //! "0 out-of-cage faults from (c)".

    use std::rc::Rc;
    use std::sync::Arc;

    use hg_substrate::{DecisionKind, DecisionLog, Position};

    use crate::cage::test_memory::TestMemory;
    use crate::node::{NodeSpace, TestNodeSpace};
    use crate::tests::{Array, Bytes, Cell, Mixed, WeakCell, Wrapper};
    use crate::trusted::{TrustedId, TrustedKind, TrustedRegion};
    use crate::{Heap, HeapConfig, Mutator, Scope, Stress, Value};

    crate::object! {
        /// A cage object holding a trusted-region id in a raw word, as a
        /// function object holds its code-table index (JS spec 6.1,
        /// mechanism 4). It exists for this harness: corrupting that
        /// word is the only way to test the claim that an index read
        /// from the cage "can name another live entry of the same kind
        /// at worst, never other memory", and no other object type
        /// holds one at JS-M0.
        pub(crate) Holder {
            raw ENTRY: u64,
            tail none,
        }
    }

    /// The process's trusted region and the entries live in it. The
    /// region is reserved through the same guarded `TestMemory` as the
    /// cage, so a write that left *it* is reported by the same ring:
    /// that is the "its side tables" half of JS spec 6.1's claim.
    struct Trusted {
        region: TrustedRegion,
        live: Vec<(TrustedId, TrustedKind, Vec<u8>)>,
    }

    /// The kinds entries are filed under, so that a forged id has
    /// several to confuse and the kind check has something to do.
    const TRUSTED_KINDS: [TrustedKind; 3] = [
        TrustedKind::BytecodeUnit,
        TrustedKind::ConstantKinds,
        TrustedKind::CodeEntry,
    ];

    impl Trusted {
        fn new(mem: &Arc<TestMemory>) -> Self {
            let region =
                TrustedRegion::new(Arc::clone(mem) as Arc<dyn hg_substrate::Memory>, 1 << 16)
                    .expect("the trusted region");
            Self {
                region,
                live: Vec::new(),
            }
        }

        /// Files a new entry under `kind`, with a payload no other entry
        /// has, so that a lookup's answer names exactly one entry.
        fn insert(&mut self, kind: TrustedKind, tag: u64) -> Option<TrustedId> {
            let payload = tag.to_le_bytes().to_vec();
            if self.live.iter().any(|(_, _, p)| *p == payload) {
                return None;
            }
            let id = self.region.insert(kind, &payload).ok()?;
            self.live.push((id, kind, payload));
            Some(id)
        }

        /// Ends an entry, so that an id held in the cage goes stale and
        /// the generation check has something to catch.
        fn remove(&mut self, which: usize) {
            if self.live.is_empty() {
                return;
            }
            let (id, ..) = self.live.remove(which % self.live.len());
            self.region.remove(id);
        }

        /// Looks up whatever 64 bits the cage now holds. **The property
        /// under test**: if the lookup answers, the bytes are one of the
        /// live entries of that kind — a forged index can confuse one
        /// entry for another and nothing else. A failure here is a
        /// panic, which the harness reports as a fault with its seed.
        fn lookup(&self, bits: u64, census: &mut Census) {
            let id = TrustedId::from_bits(bits);
            // Every kind, not one drawn at random: the kind the forged
            // bits would name is not knowable from here, and a lookup
            // under the wrong kind is refused before the bounds and
            // generation checks are reached. It widens the window only
            // a little — measured 10 November 2026, a liveness check
            // deleted from `get` was caught in 1 run of 48 with one
            // random kind and 4 with all three — which is why the
            // property also has the deterministic test below.
            for kind in TRUSTED_KINDS {
                if let Some(bytes) = self.region.get(id, kind) {
                    assert!(
                        self.live
                            .iter()
                            .any(|(_, k, p)| *k == kind && p.as_slice() == bytes),
                        "a trusted entry reached through a corrupted cage word \
                         was not one of the live entries of its kind"
                    );
                    census.entries_found += 1;
                } else {
                    census.entries_refused += 1;
                }
            }
        }
    }

    /// The cage each run gets: four chunks, so the heap fills it, frees
    /// blocks, reuses them and runs out of room for evacuation copies —
    /// every path the collector has — within one run.
    ///
    /// The same size under Miri. One chunk was tried there, on the
    /// theory that the sweep's poison — the cage's size in 8-byte stores
    /// per cycle, each one interpreted — was what made this module slow.
    /// Measured 10 November 2026: 117 s against 118 s, so it was not, and
    /// the smaller cage bought a `cfg` and nothing else.
    const CAGE: u64 = 4 << 20;

    /// Live roots a run keeps at most. Past this a new object replaces an
    /// old root, so objects become unreachable and the sweep has work.
    const ROOTS: usize = 24;

    /// A run's wall-clock budget. A corrupted heap that does not finish a
    /// cycle is a fault (a renderer that stops answering), so the budget
    /// is a detector, not a test timeout: it is far above the measured
    /// cost of a run (about 2 ms at 400 operations, 10 Nov 2026).
    #[cfg(not(miri))]
    const BUDGET: std::time::Duration = std::time::Duration::from_secs(30);

    /// Decisions go nowhere: this harness checks for faults, not for the
    /// decision log, and a corrupted heap's cycle count is arbitrary.
    struct Quiet;
    impl DecisionLog for Quiet {
        fn record(&self, _: DecisionKind, _: Position, _: u64) {}
    }

    /// xorshift64*, so a seed names a run exactly and a failing seed can
    /// be re-run on its own.
    struct Rng(u64);

    impl Rng {
        fn new(seed: u64) -> Self {
            // 0 is a fixed point of xorshift; the odd constant avoids it.
            Self(seed ^ 0x9E37_79B9_7F4A_7C15)
        }

        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next() % n.max(1)
        }

        fn u32(&mut self) -> u32 {
            u32::try_from(self.next() >> 32).unwrap_or(u32::MAX)
        }
    }

    /// What a run did, so that a harness which quietly stopped
    /// corrupting anything cannot pass. The floors are asserted on the
    /// total and the figures go in the write-up.
    #[derive(Debug, Default, Clone, Copy)]
    struct Census {
        /// Corrupting writes performed.
        writes: u64,
        /// Objects allocated.
        allocs: u64,
        /// Field accesses refused because the object's header no longer
        /// named its type: corruption reaching the object model, and the
        /// checked cast refusing it (H-2).
        casts_refused: u64,
        /// Cycles run, canonical and stress.
        cycles: u64,
        /// Objects the collector moved.
        moved: u64,
        /// Weak slots a canonical cycle cleared.
        cleared: u64,
        /// Trusted-region lookups through a cage word that answered,
        /// each checked to be a live entry of the kind asked for.
        entries_found: u64,
        /// Lookups the bounds, kind or generation check refused.
        entries_refused: u64,
    }

    impl Census {
        fn add(&mut self, o: Self) {
            self.writes += o.writes;
            self.allocs += o.allocs;
            self.casts_refused += o.casts_refused;
            self.cycles += o.cycles;
            self.moved += o.moved;
            self.cleared += o.cleared;
            self.entries_found += o.entries_found;
            self.entries_refused += o.entries_refused;
        }
    }

    /// What a run ended as. Only [`Outcome::Completed`] passes.
    #[derive(Debug, PartialEq, Eq)]
    enum Outcome {
        /// The script ran to the end and every guard ring was intact.
        Completed,
        /// A write reached the guard ring: an access outside the cage,
        /// at this offset from the cage's base (negative below it).
        GuardBroken(i64),
        /// The heap panicked on corrupted cage contents.
        Panicked(String),
        /// The run did not finish inside [`BUDGET`].
        #[cfg_attr(
            miri,
            expect(
                dead_code,
                reason = "under Miri a run has no budget: Miri's clock is not the \
                          machine's and a hang shows as the job's own timeout"
            )
        )]
        TimedOut,
    }

    /// One object the script holds a root to, and what type it is. The
    /// kind is kept beside the root because a corrupted header makes
    /// [`crate::Gc::cast`] answer `None`, and the script has to be able
    /// to try the field access that a wrong answer then refuses.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Kind {
        Cell,
        Weak,
        Array,
        Bytes,
        Wrapper,
        Mixed,
        Holder,
    }

    const KINDS: [Kind; 7] = [
        Kind::Cell,
        Kind::Weak,
        Kind::Array,
        Kind::Bytes,
        Kind::Wrapper,
        Kind::Mixed,
        Kind::Holder,
    ];

    /// Where a corrupting write is aimed. A uniform offset into a cage is
    /// almost always uncommitted space, so a fuzzer that drew only
    /// uniformly would write almost nothing an object ever reads; these
    /// four shares are what make the harness reach the collector.
    #[derive(Clone, Copy, Debug)]
    enum Aim {
        /// A word of a live object: its header, its length word, a slot,
        /// a raw word or a tail element.
        Word,
        /// Near a live object: its neighbours, the line and block edges,
        /// and the space between objects.
        Near,
        /// Anywhere in a chunk this heap holds.
        Chunk,
        /// Anywhere in the cage, committed or not.
        Wild,
    }

    /// The patterns a write uses. `Forged` is the one that matters most:
    /// a word whose low bit is set decodes as a reference, so it is the
    /// shape an attacker writes to make the collector follow something.
    fn pattern(rng: &mut Rng, target: Option<u64>) -> u64 {
        match rng.below(8) {
            0 => 0,
            1 => u64::MAX,
            2 => 0xDEAD_BEEF_DEAD_BEEF,
            3 => 1,                   // a reference to cage offset 0
            4 => u64::from(u32::MAX), // the highest offset a slot can name
            5 => target.map_or_else(|| rng.next(), |t| (t / 8) << 1 | 1),
            _ => rng.next(),
        }
    }

    /// Builds a heap in its own guarded cage and returns both.
    fn heap(stress: Stress) -> (Heap, Arc<TestMemory>, Rc<TestNodeSpace>) {
        let mem = Arc::new(TestMemory::default());
        let mut h = Heap::new(
            Arc::clone(&mem) as Arc<dyn hg_substrate::Memory>,
            Arc::new(Quiet),
            HeapConfig {
                cage_bytes: CAGE,
                trigger: 32,
                stress,
                agent: 1,
            },
        )
        .expect("the cage");
        // Off for corruption runs, and only for them: the verifier
        // assumes the cage holds what the collector and the mutator put
        // there, which is the assumption this harness exists to break.
        // See the `verify` module's own statement of this.
        h.set_verify(false);
        let ns = Rc::new(TestNodeSpace::new(64));
        h.set_node_space(Rc::clone(&ns) as Rc<dyn NodeSpace>);
        (h, mem, ns)
    }

    /// One corrupting write, aimed as `Aim` says. With `own_chunks`, an
    /// offset outside this heap's own chunks is folded back into one of
    /// them: in the two-thread test, a write into the *other* heap's
    /// chunks would overlap that heap's accesses at whatever width it
    /// uses there, and overlapping atomic accesses of different sizes
    /// are undefined (see [`Corrupt::confine`]).
    fn corrupt<'h>(
        m: &Mutator<'h>,
        rng: &mut Rng,
        live: &[(Kind, u64)],
        census: &mut Census,
        own_chunks: bool,
    ) {
        census.writes += 1;
        let c = m.corrupt();
        let len = c.cage_bytes();
        let pick = live.get(usize::try_from(rng.below(live.len() as u64)).unwrap_or(0));
        let aim = match rng.below(10) {
            0..=4 => Aim::Word,
            5..=6 => Aim::Near,
            7..=8 => Aim::Chunk,
            _ => Aim::Wild,
        };
        let at = match (aim, pick) {
            (Aim::Word, Some(&(_, off))) => off + rng.below(16) * 8,
            (Aim::Near, Some(&(_, off))) => off
                .wrapping_add(rng.below(2 * crate::space::BLOCK))
                .wrapping_sub(crate::space::BLOCK),
            (Aim::Chunk, _) | (Aim::Word | Aim::Near, None) => {
                let chunks = c.chunks();
                match chunks.get(usize::try_from(rng.below(chunks.len() as u64)).unwrap_or(0)) {
                    Some(&base) => base + rng.below(crate::cage::CHUNK),
                    None => rng.below(len),
                }
            }
            (Aim::Wild, _) => rng.below(len),
        };
        let at = if own_chunks {
            match c.confine(at) {
                Some(at) => at,
                None => return,
            }
        } else {
            at
        };
        let target = live.first().map(|&(_, off)| off);
        let v = pattern(rng, target);
        // A sweep must not run out of the chunk it started in when the
        // writes are confined: chunks are handed out in order, so the
        // next one along may be the other heap's, and a 64-bit fill
        // over words that heap reads as 32-bit slots is the mixed-size
        // access `Corrupt::confine` exists to avoid.
        let span = 8 + rng.below(512);
        let span = if own_chunks {
            span.min((at / crate::cage::CHUNK + 1) * crate::cage::CHUNK - at)
        } else {
            span
        };
        match rng.below(8) {
            0..=3 => c.write32(at, u32::try_from(v & 0xFFFF_FFFF).unwrap_or(0)),
            4..=6 => c.write64(at, v),
            // A write primitive used as a sweep, not a word: a whole
            // object's worth, which is what overwrites a header, a
            // length word and the slots after it in one go.
            _ => c.fill(at, span, v),
        }
        // A read is part of the API too, and a read of uncommitted space
        // is as much a bounds question as a write. A read is as
        // size-sensitive as a write, so it is confined the same way.
        if let Some(at) = if own_chunks {
            c.confine(rng.below(len))
        } else {
            Some(rng.below(len))
        } {
            let _ = c.read64(at);
        }
    }

    /// A checked cast of a root whose kind the script recorded when it
    /// allocated the object. A refusal here means the object's header
    /// no longer names its type, which is corruption reaching the object
    /// model and the cast refusing to answer — the H-2 behaviour this
    /// harness is for. It is counted, because a run in which it never
    /// happens is a run whose writes never landed.
    fn cast<'h, 'm, T: crate::Object>(
        g: crate::Gc<'h, 'm, crate::Any>,
        m: &'m Mutator<'h>,
        census: &mut Census,
    ) -> Option<crate::Gc<'h, 'm, T>> {
        let r = g.cast::<T>(m);
        if r.is_none() {
            census.casts_refused += 1;
        }
        r
    }

    /// Runs one operation of the script.
    #[expect(
        clippy::too_many_lines,
        reason = "one match over the operation set; splitting it would hide the distribution"
    )]
    #[expect(
        clippy::too_many_arguments,
        reason = "one run's state; a struct holding it would be borrowed field by field anyway"
    )]
    fn step<'h, 's>(
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        ns: &TestNodeSpace,
        rng: &mut Rng,
        roots: &mut Vec<(Kind, crate::Root<'h, 's, crate::Any>)>,
        census: &mut Census,
        tr: &mut Trusted,
        own_chunks: bool,
    ) {
        // The live set, re-read every operation because evacuation moves
        // objects: a corrupting write aims at where they are *now*.
        let live: Vec<(Kind, u64)> = roots
            .iter()
            .filter_map(|&(k, r)| super::Corrupt::target(r.get(m).slot()).map(|off| (k, off)))
            .collect();
        let any = |rng: &mut Rng| -> Option<usize> {
            (!roots.is_empty()).then(|| usize::try_from(rng.below(roots.len() as u64)).unwrap_or(0))
        };
        match rng.below(100) {
            // --- Allocation, which is a GC point ------------------------
            0..=19 => {
                // Every kind, drawn from the table's own length: a
                // kind the table has and the draw does not is a kind
                // the harness never tests, which is what the census
                // caught when `Holder` was added (10 November 2026).
                let kind = KINDS[usize::try_from(rng.below(KINDS.len() as u64)).unwrap_or(0)];
                let len = u32::try_from(rng.below(40)).unwrap_or(0);
                let r = match kind {
                    Kind::Cell => m.alloc::<Cell>(s, 0).map(|r| r.get(m).erase()),
                    Kind::Weak => m.alloc::<WeakCell>(s, 0).map(|r| r.get(m).erase()),
                    Kind::Array => m.alloc::<Array>(s, len).map(|r| r.get(m).erase()),
                    Kind::Bytes => m.alloc::<Bytes>(s, len).map(|r| r.get(m).erase()),
                    Kind::Wrapper => m.alloc::<Wrapper>(s, 0).map(|r| r.get(m).erase()),
                    Kind::Mixed => m.alloc::<Mixed>(s, len).map(|r| r.get(m).erase()),
                    Kind::Holder => m.alloc::<Holder>(s, 0).map(|r| r.get(m).erase()),
                };
                // Out of memory is an ordinary answer in a 4 MiB cage
                // that corruption has made the collector keep.
                if let Ok(g) = r {
                    census.allocs += 1;
                    let root = (kind, s.root(m, g));
                    if roots.len() < ROOTS {
                        roots.push(root);
                    } else if let Some(i) = any(rng) {
                        roots[i] = root;
                    }
                }
            }
            // --- Stores -------------------------------------------------
            20..=39 => {
                let (Some(i), Some(j)) = (any(rng), any(rng)) else {
                    return;
                };
                let (ki, ri) = roots[i];
                let (_, rj) = roots[j];
                let (a, b) = (ri.get(m), rj.get(m));
                match ki {
                    Kind::Cell => {
                        if let Some(c) = cast::<Cell>(a, m, census) {
                            if rng.below(2) == 0 {
                                c.set(m, Cell::NEXT, b.cast::<Cell>(m));
                            } else {
                                c.set(m, Cell::VALUE, Some(b));
                            }
                            c.set_raw(m, Cell::N, rng.u32());
                            c.set_raw(m, Cell::W, f64::from(rng.u32()) / 7.0);
                        }
                    }
                    Kind::Weak => {
                        if let Some(c) = cast::<WeakCell>(a, m, census) {
                            c.set_weak(m, WeakCell::TARGET, Some(b));
                        }
                    }
                    Kind::Array => {
                        if let Some(c) = cast::<Array>(a, m, census) {
                            let at = rng.u32() % 64;
                            if rng.below(2) == 0 {
                                c.set_tail(m, at, Value::Ref(b));
                            } else {
                                c.set_tail(
                                    m,
                                    at,
                                    Value::Small(i32::try_from(rng.below(1 << 20)).unwrap_or(0)),
                                );
                            }
                        }
                    }
                    Kind::Bytes => {
                        if let Some(c) = cast::<Bytes>(a, m, census) {
                            c.set_byte(
                                m,
                                rng.u32() % 64,
                                u8::try_from(rng.below(256)).unwrap_or(0),
                            );
                        }
                    }
                    Kind::Wrapper => {
                        if let Some(c) = cast::<Wrapper>(a, m, census) {
                            let id = u32::try_from(rng.below(16) + 1).unwrap_or(1);
                            c.set_node(m, Wrapper::NODE, ns.node_ref(id));
                            if ns.is_allocated(id) {
                                ns.wrapper(id).set(m, Some(a));
                            }
                        }
                    }
                    // A code-table index, written into the cage the way
                    // a function object holds one.
                    Kind::Holder => {
                        if let Some(c) = cast::<Holder>(a, m, census) {
                            let which = usize::try_from(rng.next()).unwrap_or(0);
                            let bits = match tr.live.get(which % tr.live.len().max(1)) {
                                Some(&(id, ..)) if rng.below(4) > 0 => id.to_bits(),
                                _ => rng.next(),
                            };
                            c.set_raw(m, Holder::ENTRY, bits);
                        }
                    }
                    Kind::Mixed => {
                        if let Some(c) = cast::<Mixed>(a, m, census) {
                            c.set(m, Mixed::NEXT, Some(b));
                            c.set_weak(m, Mixed::WATCH, Some(b));
                            let id = u32::try_from(rng.below(16) + 1).unwrap_or(1);
                            c.set_node(m, Mixed::NODE_A, ns.node_ref(id));
                            c.set_raw(m, Mixed::COUNT, rng.u32());
                            c.set_tail(m, rng.u32() % 64, Value::Ref(b));
                        }
                    }
                }
            }
            // --- Reads, which is where a wrong answer shows ------------
            40..=54 => {
                let Some(i) = any(rng) else { return };
                let (k, r) = roots[i];
                let g = r.get(m);
                let _ = g.len(m);
                match k {
                    Kind::Cell => {
                        if let Some(c) = cast::<Cell>(g, m, census) {
                            let _ = c.get(m, Cell::NEXT);
                            let _ = c.get_value(m, Cell::VALUE);
                            let _: u32 = c.raw(m, Cell::N);
                            let _: f64 = c.raw(m, Cell::W);
                        }
                    }
                    Kind::Weak => {
                        if let Some(c) = cast::<WeakCell>(g, m, census) {
                            let _ = c.get_weak(m, WeakCell::TARGET);
                        }
                    }
                    Kind::Array => {
                        if let Some(c) = cast::<Array>(g, m, census) {
                            for at in 0..c.len(m).min(64) {
                                let _ = c.tail(m, at);
                            }
                            let _ = c.tail(m, rng.u32());
                        }
                    }
                    Kind::Bytes => {
                        if let Some(c) = cast::<Bytes>(g, m, census) {
                            for at in 0..c.len(m).min(64) {
                                let _ = c.byte(m, at);
                            }
                            let _ = c.byte(m, rng.u32());
                        }
                    }
                    Kind::Wrapper => {
                        if let Some(c) = cast::<Wrapper>(g, m, census) {
                            let _ = c.node(m, Wrapper::NODE);
                        }
                    }
                    // The lookup: whatever 64 bits the cage now holds,
                    // under whichever kind, bounds-, kind- and
                    // generation-checked on the way.
                    Kind::Holder => {
                        if let Some(c) = cast::<Holder>(g, m, census) {
                            let bits: u64 = c.raw(m, Holder::ENTRY);
                            tr.lookup(bits, census);
                        }
                    }
                    Kind::Mixed => {
                        if let Some(c) = cast::<Mixed>(g, m, census) {
                            let _ = c.get(m, Mixed::NEXT);
                            let _ = c.get_weak(m, Mixed::WATCH);
                            let _ = c.node(m, Mixed::NODE_A);
                            let _ = c.node(m, Mixed::NODE_B);
                            let _: u32 = c.raw(m, Mixed::COUNT);
                            let _ = c.tail(m, rng.u32() % 64);
                        }
                    }
                }
            }
            // --- The node space ----------------------------------------
            55..=61 => {
                let id = u32::try_from(rng.below(16) + 1).unwrap_or(1);
                match rng.below(5) {
                    0 => {
                        if !ns.is_allocated(id) {
                            ns.create(id);
                        }
                    }
                    1 => {
                        let p = u32::try_from(rng.below(16) + 1).unwrap_or(1);
                        if p != id && ns.is_allocated(p) && ns.is_allocated(id) {
                            ns.append(p, id);
                        }
                    }
                    2 => ns.detach(id),
                    3 => {
                        if let Some(r) = ns.node_ref(id) {
                            let h = m.add_node_root(r);
                            if rng.below(2) == 0 {
                                m.remove_node_root(h);
                            }
                        }
                    }
                    // Draining the release list is the embedder's job,
                    // and it closes the window in which a dead node's
                    // external slots still hold untraced references.
                    _ => {
                        for r in m.take_node_releases() {
                            ns.release(r);
                        }
                    }
                }
            }
            // --- GC points and the weak protocol -----------------------
            62..=79 => match rng.below(6) {
                0 | 1 => m.charge(rng.below(48)),
                2 => m.safepoint(),
                3 => m.collect(),
                4 => {
                    if let Some(i) = any(rng) {
                        let g = roots[i].1.get(m);
                        m.keep_alive(g);
                    }
                    if rng.below(4) == 0 {
                        m.clear_kept_objects();
                    }
                }
                _ => {
                    let taken = m.take_cleared(s);
                    for r in taken {
                        if rng.below(2) == 0 && roots.len() < ROOTS {
                            roots.push((Kind::Cell, r));
                        }
                    }
                }
            },
            // --- Dropping a root, so the sweep has something to free ---
            80..=84 => {
                if let Some(i) = any(rng) {
                    roots.swap_remove(i);
                }
            }
            // --- A whole-heap walk over corrupted contents -------------
            85..=86 => {
                let _ = m.digest();
                let _ = m.stats();
            }
            // --- The trusted region grows and entries go stale ---------
            87..=89 => {
                let kind = TRUSTED_KINDS
                    [usize::try_from(rng.below(TRUSTED_KINDS.len() as u64)).unwrap_or(0)];
                if rng.below(3) == 0 {
                    tr.remove(usize::try_from(rng.next()).unwrap_or(0));
                } else {
                    tr.insert(kind, rng.next());
                }
            }
            // --- The corruption itself ---------------------------------
            _ => corrupt(m, rng, &live, census, own_chunks),
        }
    }

    /// The script: `ops` operations under one scope, then a last cycle
    /// and a last walk, so a corrupted heap has to survive a collection
    /// after the final write rather than ending mid-mutation.
    fn script(
        h: &mut Heap,
        ns: &TestNodeSpace,
        tr: &mut Trusted,
        census: &mut Census,
        seed: u64,
        ops: u32,
        own_chunks: bool,
    ) {
        let mut rng = Rng::new(seed);
        // Entries of every kind before the script starts, so a forged
        // index has live entries to be confused with from the first
        // lookup.
        for k in TRUSTED_KINDS {
            for i in 0..3 {
                tr.insert(k, rng.next() ^ i);
            }
        }
        h.enter(|m| {
            m.scope(|m, s| {
                let mut roots: Vec<(Kind, crate::Root<'_, '_, crate::Any>)> = Vec::new();
                // Objects first, so that the heap holds chunks and live
                // objects before the first corrupting write: the `Word`
                // and `Near` aims have nothing to aim at otherwise, and
                // confined writes have no chunk to be folded into.
                for _ in 0..8 {
                    if let Ok(r) = m.alloc::<Cell>(s, 0) {
                        census.allocs += 1;
                        roots.push((Kind::Cell, s.root(m, r.get(m).erase())));
                    }
                }
                for _ in 0..ops {
                    step(m, s, ns, &mut rng, &mut roots, census, tr, own_chunks);
                }
                m.collect();
                let _ = m.digest();
            });
        });
    }

    /// One run, with every detector in place.
    #[cfg_attr(
        not(miri),
        expect(
            clippy::disallowed_methods,
            reason = "G3's replacement cannot carry this: a budget needs a thread \
                      that outlives the run it is timing, `hg-heap` (L2e) must not \
                      depend on `hg-substrate-os` for the only threaded Executor, \
                      and an inline one would run the body on this thread and time \
                      nothing"
        )
    )]
    fn run(seed: u64, stress: Stress, ops: u32) -> (Outcome, Census) {
        let body = move || {
            let (mut h, mem, ns) = heap(stress);
            let mut tr = Trusted::new(&mem);
            let mut census = Census::default();
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                script(&mut h, &ns, &mut tr, &mut census, seed, ops, false);
            }));
            let st = h.stats();
            census.cycles = st.canonical_cycles + st.other_cycles;
            census.moved = st.moved;
            census.cleared = st.cleared;
            // The guard rings are checked whatever the script did: a
            // panic after an out-of-cage write must report the write.
            if let Some(at) = mem.guard_break() {
                return (Outcome::GuardBroken(at), census);
            }
            match r {
                Ok(()) => (Outcome::Completed, census),
                Err(e) => (Outcome::Panicked(panic_message(&e)), census),
            }
        };
        // Under Miri there is no budget: Miri's own clock is not the
        // machine's, a run costs seconds rather than milliseconds, and a
        // hang shows as a job timeout. Everywhere else the run is on its
        // own thread so that a cycle which never finishes is reported as
        // a fault instead of hanging the test binary.
        #[cfg(miri)]
        return body();
        #[cfg(not(miri))]
        {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || drop(tx.send(body())));
            rx.recv_timeout(BUDGET)
                .unwrap_or((Outcome::TimedOut, Census::default()))
        }
    }

    fn panic_message(e: &Box<dyn std::any::Any + Send>) -> String {
        e.downcast_ref::<String>().map_or_else(
            || {
                e.downcast_ref::<&str>()
                    .map_or_else(|| "a non-string panic".to_owned(), |s| (*s).to_owned())
            },
            Clone::clone,
        )
    }

    /// The collector modes a run is repeated under. Under Miri, only
    /// the moving test collector: it moves every object on every
    /// allocation, so it reaches every path the other two do and more,
    /// and the `miri` job's budget is the binding constraint.
    fn modes() -> &'static [Stress] {
        if cfg!(miri) {
            &[Stress::MoveOnAlloc]
        } else {
            &[Stress::None, Stress::MoveOnAlloc, Stress::MoveAtSafepoint]
        }
    }

    /// Runs per stress mode. One size everywhere, unlike JS-10 (b)'s
    /// variants, which are sampled locally and run in full in CI: that
    /// split buys something when the full set costs 23.6 s of
    /// compilation, and this whole suite costs under a second, so a
    /// local run gets the same coverage CI does. Under Miri it is the
    /// fewest that still exercises all three collector modes; the
    /// `miri` job's 60-minute budget is the binding constraint and the
    /// write-up measures what this costs it.
    const fn runs() -> u64 {
        if cfg!(miri) { 1 } else { 48 }
    }

    /// Operations per run.
    const fn ops() -> u32 {
        if cfg!(miri) { 30 } else { 400 }
    }

    /// Reports every run that did not complete, with its seed, so a
    /// failure is reproducible from the message alone, and then that the
    /// runs did the work they claim: a harness whose writes stopped
    /// landing, or whose collector stopped running, would otherwise pass
    /// by doing nothing.
    fn check(name: &str, found: &[(u64, Stress, Outcome)], runs: u64, c: Census) {
        assert!(
            found.is_empty(),
            "JS-10 (c): {} of {runs} runs faulted outside the cage:\n{}",
            found.len(),
            found
                .iter()
                .map(|(seed, stress, o)| format!("  seed {seed} ({stress:?}): {o:?}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        println!(
            "{name}: {runs} runs, {w} corrupting writes, {a} objects, {cy} cycles, \
             {mv} moved, {cl} weak slots cleared, {cr} field accesses refused by \
             a corrupted header, {ef} trusted entries reached and {er} refused",
            w = c.writes,
            a = c.allocs,
            cy = c.cycles,
            mv = c.moved,
            cl = c.cleared,
            cr = c.casts_refused,
            ef = c.entries_found,
            er = c.entries_refused,
        );
        assert!(c.writes >= runs, "{} writes in {runs} runs", c.writes);
        assert!(c.allocs >= 8 * runs, "{} objects in {runs} runs", c.allocs);
        assert!(c.cycles >= runs, "{} cycles in {runs} runs", c.cycles);
        // A corrupted header refusing a field access is the clearest
        // sign the writes reached the object model. Miri's two runs of
        // 60 operations are too few to count on it.
        assert!(
            cfg!(miri) || c.casts_refused > 0,
            "no write ever reached an object's header"
        );
    }

    /// A corrupted length word cannot make the collector walk past the
    /// object's block.
    ///
    /// The size a cycle traces comes from the layout and the length
    /// word, and the length word is cage data, so `State::shape` clamps
    /// it to the room the object's block leaves (`Space::room`). This
    /// property gets a test of its own rather than being left to the
    /// fuzzer, because its failure mode is not a fault any of the four
    /// detectors catches: an unclamped size makes the tracer build a
    /// slot list of billions of entries, and the process dies of an
    /// allocation failure — which looks like a crashed test runner, not
    /// like a finding with a seed.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "a bound over cage data, not an unsafe-code or concurrency \
                  property, so Miri adds nothing to it; tracing a block's worth \
                  of forged slots costs the miri job about a minute, and the \
                  collect-over-corrupted-contents path it also walks is what \
                  every fuzzer run does there anyway"
    )]
    fn a_corrupted_length_word_bounds_the_traced_region_to_the_block() {
        let (mut h, mem, _ns) = heap(Stress::None);
        h.enter(|m| {
            m.scope(|m, s| {
                let a = m.alloc::<Array>(s, 8).expect("an array");
                let off = super::Corrupt::target(a.get(m).slot()).expect("a reference");
                // What the layout alone would say about this length: a
                // tail of 2^32 slots, so about 16 GiB, in a 4 MiB cage.
                let unclamped = <Array as crate::Object>::LAYOUT
                    .size(u32::MAX)
                    .expect("a size");
                assert!(unclamped > 16 * (1 << 30));

                let c = m.corrupt();
                c.write32(off + 4, u32::MAX);
                // Every tail slot forged to name the array itself, so
                // the cycle has as much to follow as the clamp allows.
                // One block is all the clamp can let through, so that
                // is what has to be forged; the wider fill is breadth
                // for the ordinary run, and under Miri every one of
                // those stores is interpreted.
                let span = if cfg!(miri) {
                    8 << 10
                } else {
                    2 * crate::space::BLOCK
                };
                c.fill(off + 8, span, (off / 8) << 1 | 1);

                let (_, size) = m
                    .state_for_tests()
                    .shape(off)
                    .expect("the object still measures");
                assert!(
                    size <= crate::space::BLOCK,
                    "a corrupted length word measured {size} bytes, past its block"
                );
                // And the cycle finishes, which is the property the
                // bound exists for.
                m.collect();
                let _ = m.digest();
            });
        });
        assert_eq!(mem.guard_break(), None);
    }

    /// A trusted-region index held in the cage cannot outlive its entry,
    /// and no bits written over it reach anything but a live entry of
    /// the kind asked for (JS spec 6.1, mechanism 4).
    ///
    /// This is the deterministic half of the property the fuzzer checks
    /// as it goes. It is here because the fuzzer's window on it is
    /// narrow: for the stale case to arise by chance, a word has to hold
    /// a real id, that entry has to end, and the word has to be read
    /// again before anything overwrites it. Measured 10 November 2026,
    /// with the liveness check deleted from `TrustedRegion::get` the
    /// fuzzer reported 4 runs of 48 and this test reported every time.
    #[test]
    fn a_trusted_index_in_the_cage_cannot_outlive_its_entry() {
        let (mut h, mem, _ns) = heap(Stress::None);
        let mut tr = Trusted::new(&mem);
        let mut census = Census::default();
        let one = tr.insert(TrustedKind::CodeEntry, 0x0101).expect("an entry");

        h.enter(|m| {
            m.scope(|m, s| {
                let holder = m.alloc::<Holder>(s, 0).expect("a holder");
                let off = super::Corrupt::target(holder.get(m).slot()).expect("a reference");
                holder.get(m).set_raw(m, Holder::ENTRY, one.to_bits());
                // While the entry is live, the word in the cage reaches
                // it — and only under its own kind.
                assert_eq!(
                    tr.region.get(
                        TrustedId::from_bits(holder.get(m).raw::<u64>(m, Holder::ENTRY)),
                        TrustedKind::CodeEntry
                    ),
                    Some(&0x0101u64.to_le_bytes()[..])
                );
                assert_eq!(
                    tr.region.get(
                        TrustedId::from_bits(holder.get(m).raw::<u64>(m, Holder::ENTRY)),
                        TrustedKind::BytecodeUnit
                    ),
                    None,
                    "the kind is checked on every use"
                );

                // The entry ends. The word in the cage still holds its
                // bits, and a stale index must reach nothing.
                tr.remove(0);
                assert_eq!(
                    holder.get(m).raw::<u64>(m, Holder::ENTRY),
                    one.to_bits(),
                    "the cage word is unchanged"
                );
                assert_eq!(
                    tr.region.get(
                        TrustedId::from_bits(holder.get(m).raw::<u64>(m, Holder::ENTRY)),
                        TrustedKind::CodeEntry
                    ),
                    None,
                    "an index whose entry ended reaches nothing"
                );

                // The index is reused under a new generation. The old
                // bits in the cage must not reach the new entry.
                let two = tr.insert(TrustedKind::CodeEntry, 0x0202).expect("a second");
                assert_eq!(two.to_bits() & 0xFFFF_FFFF, one.to_bits() & 0xFFFF_FFFF);
                assert_ne!(two.to_bits(), one.to_bits(), "a new generation");
                assert_eq!(
                    tr.region.get(
                        TrustedId::from_bits(holder.get(m).raw::<u64>(m, Holder::ENTRY)),
                        TrustedKind::CodeEntry
                    ),
                    None,
                    "a stale generation does not reach the index's new occupant"
                );

                // And then every pattern the fuzzer writes, over the
                // same word: whatever answers is a live entry of the
                // kind asked for, which `Trusted::lookup` asserts.
                let c = m.corrupt();
                for p in 0..if cfg!(miri) { 8 } else { 64 } {
                    let at = off + <Holder as crate::Object>::LAYOUT.raw_offset(0);
                    c.write64(at, pattern(&mut Rng::new(p), Some(off)));
                    tr.lookup(holder.get(m).raw::<u64>(m, Holder::ENTRY), &mut census);
                }
            });
        });
        assert!(
            census.entries_refused > 0,
            "no forged index was refused: {census:?}"
        );
        assert_eq!(mem.guard_break(), None);
    }

    /// JS-10 (c): a fuzzer with arbitrary read and write inside the cage,
    /// against every collector mode. **0** out-of-cage faults.
    #[test]
    fn arbitrary_in_cage_writes_never_fault_outside_the_cage() {
        let runs = runs();
        let (mut found, mut total, mut census) = (Vec::new(), 0u64, Census::default());
        for &stress in modes() {
            for i in 0..runs {
                let seed = 0x1000 + i * 7 + (stress as u64) * 1_000_003;
                total += 1;
                let (o, c) = run(seed, stress, ops());
                census.add(c);
                if o != Outcome::Completed {
                    found.push((seed, stress, o));
                }
            }
        }
        check("in-cage corruption", &found, total, census);
        assert!(census.moved > 0, "no cycle moved an object");
    }

    /// Two agent heaps in one cage, on two threads, each corrupting its
    /// own chunks: the "while other work runs" half of JS spec 6.1's
    /// API, in the only shape the design licenses.
    ///
    /// What this checks is the claim `cage.rs` makes when it declares
    /// the cage `Send` and `Sync` — that agent heaps on different
    /// threads may share it — under corruption, with the chunk pool
    /// handing out and taking back chunks from both threads at once.
    /// Miri's race detector is what reads the result.
    ///
    /// Each thread writes **only inside its own chunks**. The first
    /// shape of this test pointed one thread at the whole cage, and
    /// Miri reported a mixed-size atomic access (10 November 2026),
    /// correctly: a 64-bit write over a word another thread reaches as
    /// a 32-bit slot is undefined however it is ordered. The write-up
    /// records what that means for a concurrent collector.
    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "the subject is two OS threads sharing one cage, which is what \
                  the `Send` claim on `Cage` asserts; posting to an Executor \
                  would test the Executor, and the only threaded one lives in \
                  `hg-substrate-os`, which this crate must not depend on"
    )]
    fn two_agent_heaps_on_two_threads_corrupt_their_own_chunks() {
        let mem = Arc::new(TestMemory::default());
        let cage = Arc::new(
            crate::ProcessCage::new(Arc::clone(&mem) as Arc<dyn hg_substrate::Memory>, 16 << 20)
                .expect("the cage"),
        );
        let ops = if cfg!(miri) { 20 } else { 200 };
        let threads: Vec<_> = [Stress::None, Stress::MoveOnAlloc]
            .into_iter()
            .enumerate()
            .map(|(i, stress)| {
                let cage = Arc::clone(&cage);
                let mem = Arc::clone(&mem);
                std::thread::spawn(move || {
                    // The heap is built inside the thread: a `Heap` is
                    // `!Send`, which is half of why no two threads
                    // reach one heap's memory.
                    let mut h = Heap::in_cage(
                        &cage,
                        Arc::new(Quiet),
                        HeapConfig {
                            cage_bytes: 0,
                            trigger: 32,
                            stress,
                            agent: u32::try_from(i).unwrap_or(0),
                        },
                    );
                    h.set_verify(false);
                    let ns = Rc::new(TestNodeSpace::new(64));
                    h.set_node_space(Rc::clone(&ns) as Rc<dyn NodeSpace>);
                    let mut tr = Trusted::new(&mem);
                    let mut census = Census::default();
                    let seed = 0x5000 + i as u64 * 11;
                    script(&mut h, &ns, &mut tr, &mut census, seed, ops, true);
                    let st = h.stats();
                    census.cycles = st.canonical_cycles + st.other_cycles;
                    census.moved = st.moved;
                    census
                })
            })
            .collect();

        let mut census = Census::default();
        for (i, t) in threads.into_iter().enumerate() {
            // A panic in either thread is a fault, and it carries which
            // agent saw it.
            census.add(t.join().unwrap_or_else(|_| panic!("agent {i} panicked")));
        }
        assert_eq!(mem.guard_break(), None, "a write left the cage");
        check("two agent heaps", &[], 2, census);
    }

    /// The marker that keeps [`Corrupt`] on one thread is load-bearing
    /// and invisible: both of the handle's real fields are `Sync`, so
    /// deleting the `PhantomData` would make the type `Send` and `Sync`
    /// again by inference, with nothing to say so. That is the handle
    /// Miri showed to be undefined against a live heap, so this asserts
    /// the marker is still there. The assertions are `const`, so losing
    /// the marker fails the build rather than a test run.
    ///
    /// The `ui` fixtures would be the usual place, but they compile
    /// against `hg-heap` built without the `testing` feature, where this
    /// module does not exist: the fixture would be refused for the wrong
    /// reason and still read as a pass.
    #[test]
    fn the_corruption_handle_stays_on_the_thread_that_made_it() {
        use super::Corrupt;
        use std::marker::PhantomData;

        struct Probe<T>(PhantomData<T>);
        trait Inferred {
            const SEND: bool = false;
        }
        impl<T> Inferred for Probe<T> {}
        impl<T: Send> Probe<T> {
            const SEND: bool = true;
        }
        const {
            assert!(
                <Probe<u32>>::SEND,
                "the probe cannot see `Send`, so its other answer means nothing"
            );
        }
        const {
            assert!(
                !<Probe<Corrupt<'static>>>::SEND,
                "`Corrupt` is `Send`: the marker that withdrew the cross-thread \
                 handle is gone, and a test can race mixed-width writes against \
                 a live heap again"
            );
        }
    }
}
