//! The two designs, really allocated and really traced.
//!
//! One cage model and one collector serve both, so the only difference
//! between the variants is where the nodes live and how the collector
//! reaches them. That is deliberate: a comparison in which each design
//! got its own collector would price the collectors, not the designs.
//!
//! **What this is not.** It is not `hg-heap`. `hg-heap` has no node space
//! (that is the blocked stage-1 work), so the absolute times below are
//! this harness's, not Izcal's. What the trigger asks for is a ratio
//! between two layouts under one collector, and that is what this
//! measures.
//!
//! The cage model follows the real one where it matters for cost: a
//! power-of-two reservation, every offset masked to its length and
//! aligned down to the access width before use, 32-bit atomic slots, the
//! `(offset / 8) << 1 | 1` reference encoding of JS-D1 (a), H-4's object
//! header and field order, and mark bits in a side bitmap outside the
//! cage. It is safe Rust: the mask makes an out-of-range offset index
//! another part of the same `Vec`, which is the same property the real
//! module's `unsafe` has to establish by hand.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use crate::model;

/// Slot ordering. The real heap uses `Relaxed` for mutator stores and
/// marker loads (JS spec 6.5), so the cost measured here is the same.
const ORD: Ordering = Ordering::Relaxed;

/// A node slot index. 0 is never a node, so it doubles as "none", the
/// way cage offset 0 does in the real cage.
pub type NodeId = u32;

/// Encodes a cage byte offset as a 32-bit reference slot (JS-D1 (a)).
pub fn encode_ref(off: u64) -> u32 {
    debug_assert!(off.is_multiple_of(8));
    (((off / 8) as u32) << 1) | 1
}

/// The cage byte offset a slot references, if it holds a reference. Every
/// bit pattern decodes to an 8-aligned offset, as the real slot does.
pub fn decode_ref(v: u32) -> Option<u64> {
    if v & 1 == 0 {
        return None;
    }
    Some(u64::from(v >> 1) * 8)
}

// --- the cage ---------------------------------------------------------------

/// The process cage: a power-of-two run of 32-bit atomic words, a bump
/// cursor, and a mark bit per 8-byte granule kept outside it.
pub struct Cage {
    words: Vec<AtomicU32>,
    /// `len - 1`, for masking (the real module's condition).
    mask: u64,
    cursor: u64,
    /// One bit per 8-byte granule, outside the cage so corrupt cage
    /// contents cannot reach the collector's bookkeeping (H-2).
    marks: Vec<u64>,
}

impl Cage {
    /// A cage of `bytes`, which must be a power of two of at least 8.
    pub fn new(bytes: u64) -> Self {
        assert!(bytes.is_power_of_two() && bytes >= 8);
        let words = usize::try_from(bytes / 4).expect("cage fits in memory");
        let granules = usize::try_from(bytes / 8).expect("cage fits in memory");
        Self {
            words: (0..words).map(|_| AtomicU32::new(0)).collect(),
            mask: bytes - 1,
            // Granule 0 is never handed out, so no object lives at 0.
            cursor: 8,
            marks: vec![0u64; granules.div_ceil(64)],
        }
    }

    /// The cage's length in bytes.
    pub fn len(&self) -> u64 {
        self.mask + 1
    }

    /// Bytes handed out so far.
    pub fn used(&self) -> u64 {
        self.cursor - 8
    }

    fn index(&self, off: u64) -> usize {
        // Mask to the length, then align down to the access width: the
        // real module's verification condition, in safe code.
        usize::try_from((off & self.mask) / 4).expect("masked offset fits")
    }

    /// Reads the 32-bit word at `off`.
    pub fn load32(&self, off: u64) -> u32 {
        self.words[self.index(off)].load(ORD)
    }

    /// Writes the 32-bit word at `off`.
    pub fn store32(&self, off: u64, v: u32) {
        self.words[self.index(off)].store(v, ORD);
    }

    /// Bump-allocates `size` bytes, rounded up to 8.
    pub fn alloc(&mut self, size: u64) -> Option<u64> {
        let size = size.next_multiple_of(8);
        let at = self.cursor;
        let end = at.checked_add(size)?;
        if end > self.len() {
            return None;
        }
        self.cursor = end;
        Some(at)
    }

    /// Marks the object at `off`; returns whether it was already marked.
    pub fn mark(&mut self, off: u64) -> bool {
        let g = usize::try_from((off & self.mask) / 8).expect("granule fits");
        let (w, b) = (g / 64, g % 64);
        let was = self.marks[w] & (1 << b) != 0;
        self.marks[w] |= 1 << b;
        was
    }

    /// Whether the object at `off` is marked.
    pub fn is_marked(&self, off: u64) -> bool {
        let g = usize::try_from((off & self.mask) / 8).expect("granule fits");
        self.marks[g / 64] & (1 << (g % 64)) != 0
    }

    /// Clears every mark, ready for a cycle.
    pub fn clear_marks(&mut self) {
        self.marks.fill(0);
    }

    /// Copies `size` bytes from `from` to `to`, word by word through the
    /// masked accessors, as evacuation does (H-7: a loop of atomic word
    /// copies, never a `memcpy`).
    pub fn copy(&self, from: u64, to: u64, size: u64) {
        let mut i = 0;
        while i < size {
            self.store32(to + i, self.load32(from + i));
            i += 4;
        }
    }
}

/// An object's traced slot `i` lives at this offset from its start (H-4:
/// the 8-byte header, then the traced slots).
pub fn slot_off(obj: u64, i: u32) -> u64 {
    obj + u64::from(model::CAGE_HEADER_BYTES) + u64::from(i) * 4
}

/// A type index for the collector's layout lookup, kept outside the cage
/// as H-4 requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeRow {
    /// Traced 32-bit slots at the start of the body.
    pub traced: u32,
    /// 64-bit raw words after them.
    pub raw_words: u32,
}

impl TypeRow {
    /// The object's size in bytes.
    pub fn size(self) -> u64 {
        u64::from(model::cage_object_bytes(self.traced, self.raw_words))
    }
}

/// Type 0 is a plain JS object (two traced slots, one raw word): the
/// non-node heap both designs are given, so "share of GC time" has the
/// same denominator in each.
pub const TY_JS: u32 = 0;
/// Type 1 is (a)'s wrapper: no traced slots, one raw word holding the
/// node reference.
pub const TY_WRAPPER: u32 = 1;
/// Type 2 is (c)'s node: five traced links, then the packed raw words.
pub const TY_NODE: u32 = 2;

/// The type table, outside the cage.
pub fn type_table(node_raw_words: u32, wrapper_words: u32) -> Vec<TypeRow> {
    vec![
        TypeRow {
            traced: 2,
            raw_words: 1,
        },
        TypeRow {
            traced: 0,
            raw_words: wrapper_words,
        },
        TypeRow {
            traced: 5,
            raw_words: node_raw_words,
        },
    ]
}

// --- design (a): columns outside the cage -----------------------------------

/// The five link columns, in the order the collector reads them.
pub const LINKS: usize = 5;

/// (a)'s node space: twelve 32-bit columns outside the cage, a side mark
/// bitmap with one bit per slot, and the rare-data records that hold the
/// wrapper's external traced slot.
pub struct NodeSpace {
    /// parent, first child, last child, previous and next sibling.
    pub links: [Vec<AtomicU32>; LINKS],
    /// tag, flags, element state, attribute set, inline style, rare
    /// index, generation: the rest of the 48-byte core.
    pub rest: [Vec<AtomicU32>; 7],
    /// One bit per node slot: the side mark bitmap of JS spec 6.2.
    marks: Vec<u64>,
    /// Rare-data records, allocated only for nodes that have one. Each is
    /// an external traced slot (the wrapper) and the record's own link.
    pub rare_wrapper: Vec<AtomicU32>,
    rare_link: Vec<u32>,
}

/// Which column of [`NodeSpace::rest`] holds what.
pub const REST_RARE: usize = 5;
/// The generation column.
pub const REST_GENERATION: usize = 6;

impl NodeSpace {
    /// A node space with room for `n` slots (slot 0 unused).
    pub fn new(n: usize) -> Self {
        let col = || (0..n).map(|_| AtomicU32::new(0)).collect::<Vec<_>>();
        Self {
            links: [col(), col(), col(), col(), col()],
            rest: [col(), col(), col(), col(), col(), col(), col()],
            marks: vec![0u64; n.div_ceil(64)],
            rare_wrapper: Vec::new(),
            rare_link: Vec::new(),
        }
    }

    /// How many node slots the space has columns for.
    pub fn slots(&self) -> usize {
        self.links[0].len()
    }

    /// Adds a rare-data record for `node`, returning its index. The
    /// node's rare column then names it (index + 1; 0 means none).
    pub fn add_rare(&mut self, node: NodeId) -> u32 {
        self.rare_wrapper.push(AtomicU32::new(0));
        self.rare_link.push(0);
        let i = u32::try_from(self.rare_wrapper.len() - 1).expect("rare index fits");
        self.rest[REST_RARE][node as usize].store(i + 1, ORD);
        i
    }

    /// Marks `node`; returns whether it was already marked.
    pub fn mark(&mut self, node: NodeId) -> bool {
        let (w, b) = ((node as usize) / 64, (node as usize) % 64);
        let was = self.marks[w] & (1 << b) != 0;
        self.marks[w] |= 1 << b;
        was
    }

    /// Whether `node` is marked.
    pub fn is_marked(&self, node: NodeId) -> bool {
        self.marks[(node as usize) / 64] & (1 << ((node as usize) % 64)) != 0
    }

    /// Clears every mark.
    pub fn clear_marks(&mut self) {
        self.marks.fill(0);
    }

    /// The bytes this space really occupies: the twelve columns, the side
    /// mark bitmap, and the rare records.
    pub fn bytes(&self) -> u64 {
        let slots = self.links[0].len() as u64;
        let columns = slots * 4 * (LINKS as u64 + 7);
        let bitmap = (self.marks.len() * 8) as u64;
        let rare = (self.rare_wrapper.len() * 4 + self.rare_link.len() * 4) as u64;
        columns + bitmap + rare
    }

    /// The number of rare records.
    pub fn rare_count(&self) -> usize {
        self.rare_wrapper.len()
    }
}

// --- the collector ----------------------------------------------------------

/// The type word of an object evacuation has moved. H-4's header is "a
/// type index plus a forwarding bit"; here the whole word is the marker
/// and the length word holds the new location, so a slot can be fixed up
/// with one load and no side table.
pub const TY_FORWARDED: u32 = u32::MAX;

/// Where the object that used to be at `off` now lives, if it moved.
pub fn forwarded(cage: &Cage, off: u64) -> Option<u64> {
    if cage.load32(off) != TY_FORWARDED {
        return None;
    }
    decode_ref(cage.load32(off + 4))
}

/// Moves the object at `from` to fresh space and leaves a forwarding
/// header behind. Returns the new offset.
fn evacuate_one(cage: &mut Cage, from: u64, size: u64) -> Option<u64> {
    let to = cage.alloc(size)?;
    cage.copy(from, to, size);
    cage.store32(from, TY_FORWARDED);
    cage.store32(from + 4, encode_ref(to));
    Some(to)
}

/// Where a cycle spent its time, in nanoseconds.
#[derive(Debug, Clone, Copy, Default)]
pub struct Phases {
    /// Tracing nodes: the node space in (a), the node objects in (c).
    pub nodes_ns: u128,
    /// Tracing everything else: the plain JS heap, and (a)'s wrappers.
    pub other_ns: u128,
    /// Sweeping.
    pub sweep_ns: u128,
}

impl Phases {
    /// The whole cycle. Evacuation has no phase of its own: H-7's one
    /// tracing loop marks and evacuates together, so a moved object's
    /// copy is paid for inside the phase that traced it.
    pub fn total_ns(&self) -> u128 {
        self.nodes_ns + self.other_ns + self.sweep_ns
    }

    /// The node part's share of the cycle, as a fraction. For (a) this is
    /// literally "the node space's share of GC time"; for (c), where
    /// nodes are ordinary heap objects, it is the share spent on them,
    /// which is the comparable quantity.
    pub fn node_share(&self) -> f64 {
        let t = self.total_ns();
        if t == 0 {
            return 0.0;
        }
        self.nodes_ns as f64 / t as f64
    }
}

/// A cycle's outcome, with the counts the validity gates check.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cycle {
    /// Where the time went.
    pub phases: Phases,
    /// Nodes the cycle marked live.
    pub nodes_marked: u64,
    /// Non-node cage objects the cycle marked live.
    pub others_marked: u64,
    /// Objects the cycle moved.
    pub moved: u64,
    /// Traced slots that still named a moved object: a missed fix-up. It
    /// must be 0, and a gate checks it, because a stale slot is how a
    /// collector loses a subtree and looks fast doing it.
    pub stale: u64,
}

/// Times `f`, in nanoseconds.
fn timed<R>(f: impl FnOnce() -> R) -> (R, u128) {
    let t = Instant::now();
    let r = f();
    (r, t.elapsed().as_nanos())
}

/// One tracing loop serves marking and evacuation (H-7). `visit` follows
/// the slot at `at`: it updates the slot if its target moved, copies the
/// target if it is in an evacuation candidate and not yet marked, marks
/// it, and returns the offset to scan. There is no separate fix-up pass,
/// because the loop that marks is the loop that updates.
fn visit(
    cage: &mut Cage,
    at: u64,
    candidate_end: u64,
    candidate_ty: u32,
    types: &[TypeRow],
    moved: &mut u64,
) -> Option<u64> {
    let mut t = decode_ref(cage.load32(at))?;
    if let Some(to) = forwarded(cage, t) {
        cage.store32(at, encode_ref(to));
        t = to;
    } else if t < candidate_end && cage.load32(t) == candidate_ty && !cage.is_marked(t) {
        let size = types[candidate_ty as usize].size();
        if let Some(to) = evacuate_one(cage, t, size) {
            cage.store32(at, encode_ref(to));
            *moved += 1;
            t = to;
        }
    }
    if cage.mark(t) { None } else { Some(t) }
}

/// Design (a)'s heap: the node space outside the cage, wrappers and the
/// plain JS heap inside it.
pub struct HeapA {
    /// The node columns.
    pub space: NodeSpace,
    /// The cage holding wrappers and the plain JS heap.
    pub cage: Cage,
    /// The type table, outside the cage.
    pub types: Vec<TypeRow>,
    /// The node space's root.
    pub document: NodeId,
    /// Cage roots: the plain JS objects script holds.
    pub roots: Vec<u64>,
    /// Cage offsets below this are evacuation candidates.
    pub candidate_end: u64,
}

impl HeapA {
    /// Runs one cycle. The node space is non-moving, so a node never
    /// changes address and no link is ever rewritten; only wrappers move.
    pub fn collect(&mut self) -> Cycle {
        self.space.clear_marks();
        self.cage.clear_marks();
        let mut c = Cycle::default();
        let mut node_work: Vec<NodeId> = Vec::with_capacity(1024);

        // Phase 1: the plain JS heap, and the cage-to-node edges a rooted
        // wrapper carries (JS-D4's strong-once-created rule).
        let roots = core::mem::take(&mut self.roots);
        let mut cage_work: Vec<u64> = Vec::with_capacity(roots.len() + 16);
        let cage = &mut self.cage;
        let types = &self.types;
        let cand = self.candidate_end;
        let ((others, stale, moved_1, roots), other_ns) = timed(|| {
            let mut n = 0u64;
            let mut stale = 0u64;
            let mut moved = 0u64;
            let mut roots = roots;
            // A root is a handle, not a slot in the cage, so it is
            // forwarded by hand; every other edge goes through `visit`.
            for r in roots.iter_mut() {
                if let Some(to) = forwarded(cage, *r) {
                    *r = to;
                }
                if !cage.mark(*r) {
                    cage_work.push(*r);
                    n += 1;
                }
            }
            while let Some(off) = cage_work.pop() {
                let ty = cage.load32(off);
                if ty == TY_FORWARDED {
                    stale += 1;
                    continue;
                }
                if ty == TY_WRAPPER {
                    // The wrapper's node reference: the open decision,
                    // read where a declared node field would sit.
                    let node = cage.load32(off + u64::from(model::CAGE_HEADER_BYTES));
                    if node != 0 {
                        node_work.push(node);
                    }
                    continue;
                }
                for i in 0..types[ty as usize].traced {
                    if let Some(t) =
                        visit(cage, slot_off(off, i), cand, TY_WRAPPER, types, &mut moved)
                    {
                        cage_work.push(t);
                        n += 1;
                    }
                }
            }
            (n, stale, moved, roots)
        });
        self.roots = roots;
        c.others_marked = others;
        c.stale = stale;
        c.phases.other_ns = other_ns;

        // Phase 2: the node space. Five link columns per node, plus the
        // external traced slot in rare data that holds the wrapper.
        node_work.push(self.document);
        let space = &mut self.space;
        let ((nodes, moved_2, wrappers), nodes_ns) = timed(|| {
            let mut n = 0u64;
            let mut moved = 0u64;
            let mut w = 0u64;
            while let Some(id) = node_work.pop() {
                if id == 0 || space.is_marked(id) {
                    continue;
                }
                space.mark(id);
                n += 1;
                for l in space.links.iter() {
                    let t = l[id as usize].load(ORD);
                    if t != 0 {
                        node_work.push(t);
                    }
                }
                // The external traced slot lives outside the cage, so it
                // is read and written here rather than through `visit`.
                let rare = space.rest[REST_RARE][id as usize].load(ORD);
                if rare == 0 {
                    continue;
                }
                let slot = &space.rare_wrapper[(rare - 1) as usize];
                let Some(mut t) = decode_ref(slot.load(ORD)) else {
                    continue;
                };
                if let Some(to) = forwarded(cage, t) {
                    slot.store(encode_ref(to), ORD);
                    t = to;
                } else if t < cand && cage.load32(t) == TY_WRAPPER && !cage.is_marked(t) {
                    if let Some(to) = evacuate_one(cage, t, types[TY_WRAPPER as usize].size()) {
                        slot.store(encode_ref(to), ORD);
                        moved += 1;
                        t = to;
                    }
                }
                if !cage.mark(t) {
                    w += 1;
                }
            }
            (n, moved, w)
        });
        c.nodes_marked = nodes;
        c.others_marked += wrappers;
        c.moved = moved_1 + moved_2;
        c.phases.nodes_ns = nodes_ns;

        // Sweep: the node space's side mark bitmap. A dead slot goes on
        // the release list JS 6.5 processes between tasks.
        let (_, sweep_ns) = timed(|| {
            let mut released = 0u64;
            for id in 1..space.slots() as u32 {
                if !space.is_marked(id) && space.rest[REST_GENERATION][id as usize].load(ORD) != 0 {
                    released += 1;
                }
            }
            released
        });
        c.phases.sweep_ns = sweep_ns;
        c
    }
}

/// Design (c)'s heap: nodes are cage objects, so they move.
pub struct HeapC {
    /// The cage holding nodes and the plain JS heap.
    pub cage: Cage,
    /// The type table.
    pub types: Vec<TypeRow>,
    /// The document node's cage offset.
    pub document: u64,
    /// The plain JS heap's roots.
    pub roots: Vec<u64>,
    /// The stable-ID table: ID to current cage reference, for `c-id`.
    pub id_table: Vec<u32>,
    /// Cage offsets below this are evacuation candidates.
    pub candidate_end: u64,
}

impl HeapC {
    /// Runs one cycle, in the same two phases and with the same one
    /// clock read per phase as [`HeapA::collect`].
    pub fn collect(&mut self) -> Cycle {
        self.cage.clear_marks();
        let mut c = Cycle::default();
        let cand = self.candidate_end;

        // Phase 1: the plain JS heap. A node object met here is deferred
        // to phase 2, which changes no work and keeps the timing split
        // identical to (a)'s.
        let roots = core::mem::take(&mut self.roots);
        let mut node_work: Vec<u64> = Vec::with_capacity(1024);
        let mut cage_work: Vec<u64> = Vec::with_capacity(roots.len() + 16);
        let cage = &mut self.cage;
        let types = &self.types;
        let ((others, found_nodes, stale, moved_1, roots), other_ns) = timed(|| {
            let mut n = 0u64;
            let mut nodes = 0u64;
            let mut stale = 0u64;
            let mut moved = 0u64;
            let mut roots = roots;
            for r in roots.iter_mut() {
                if let Some(to) = forwarded(cage, *r) {
                    *r = to;
                }
                if !cage.mark(*r) {
                    cage_work.push(*r);
                    n += 1;
                }
            }
            while let Some(off) = cage_work.pop() {
                let ty = cage.load32(off);
                if ty == TY_FORWARDED {
                    stale += 1;
                    continue;
                }
                for i in 0..types[ty as usize].traced {
                    let at = slot_off(off, i);
                    // A slot of a JS object may name a node, so the
                    // candidate type for this edge is the node's.
                    let target = decode_ref(cage.load32(at));
                    let is_node = target.is_some_and(|t| {
                        let h = cage.load32(t);
                        h == TY_NODE || h == TY_FORWARDED
                    });
                    let ct = if is_node { TY_NODE } else { TY_JS };
                    if let Some(t) = visit(cage, at, cand, ct, types, &mut moved) {
                        if cage.load32(t) == TY_NODE {
                            node_work.push(t);
                            nodes += 1;
                        } else {
                            cage_work.push(t);
                            n += 1;
                        }
                    }
                }
            }
            (n, nodes, stale, moved, roots)
        });
        self.roots = roots;
        c.others_marked = others;
        c.phases.other_ns = other_ns;

        // Phase 2: the nodes. Every link is a cage slot, so every link of
        // every live node is read and, when its target moved, written —
        // the work (a)'s non-moving node space never does.
        let mut document = self.document;
        let id_table = core::mem::take(&mut self.id_table);
        let ((nodes, stale_2, moved_2, id_table), nodes_ns) = timed(|| {
            let mut n = 0u64;
            let mut stale = 0u64;
            let mut moved = 0u64;
            let mut id_table = id_table;
            if let Some(to) = forwarded(cage, document) {
                document = to;
            }
            if !cage.mark(document) {
                node_work.push(document);
                n += 1;
            }
            while let Some(off) = node_work.pop() {
                let ty = cage.load32(off);
                if ty == TY_FORWARDED {
                    stale += 1;
                    continue;
                }
                for i in 0..5u32 {
                    if let Some(t) = visit(cage, slot_off(off, i), cand, TY_NODE, types, &mut moved)
                    {
                        node_work.push(t);
                        n += 1;
                    }
                }
            }
            // The stable-ID table is a root set of its own: `c-id` pays
            // one read, and a write per moved node, that `c-min` does not.
            for e in id_table.iter_mut() {
                if let Some(t) = decode_ref(*e)
                    && let Some(to) = forwarded(cage, t)
                {
                    *e = encode_ref(to);
                }
            }
            (n, stale, moved, id_table)
        });
        self.document = document;
        self.id_table = id_table;
        c.nodes_marked = nodes + found_nodes;
        c.stale = stale + stale_2;
        c.moved = moved_1 + moved_2;
        c.phases.nodes_ns = nodes_ns;

        let (_, sweep_ns) = timed(|| {
            let mut live = 0u64;
            let mut off = 8u64;
            let end = cage.used() + 8;
            while off < end {
                if cage.is_marked(off) {
                    live += 1;
                }
                off += 8;
            }
            live
        });
        c.phases.sweep_ns = sweep_ns;
        c
    }
}
