//! The synthetic trees, built the same way for both designs.
//!
//! Sizes come from the HTTP Archive Web Almanac's published element
//! counts; the note states the edition and the figure each tree is sized
//! from. Shape comes from the open-element stack an HTML parser keeps, so
//! a node's children are contiguous in allocation order, which is what
//! the real parser produces and what the trace's locality depends on.

use std::sync::atomic::Ordering;

use crate::heap::{
    Cage, HeapA, HeapC, NodeId, NodeSpace, REST_GENERATION, TY_JS, TY_NODE, TY_WRAPPER, encode_ref,
    slot_off, type_table,
};
use crate::model;

const ORD: Ordering = Ordering::Relaxed;

/// A seeded generator (SplitMix64), so every run of a shape is the same
/// tree on every machine.
pub struct Rng(u64);

impl Rng {
    /// A generator from `seed`.
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `[0, n)`.
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }

    /// A value in `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// What tree to build.
#[derive(Debug, Clone, Copy)]
pub struct Shape {
    /// Nodes attached to the document.
    pub attached: usize,
    /// Nodes allocated and then left unreachable, as a parser-created and
    /// dropped node is. They must not survive a cycle.
    pub orphans: usize,
    /// The deepest nesting the generator will produce.
    pub depth_limit: usize,
    /// The fraction of attached nodes script has touched, so a wrapper
    /// exists for them. Unmeasured in the literature (DOM spec Q-DM-6),
    /// which is why it is swept.
    pub touched: f64,
    /// The fraction of touched nodes whose wrapper a plain JS object also
    /// holds, so the node is live through the cage rather than the tree.
    pub rooted_touched: f64,
    /// The seed.
    pub seed: u64,
}

/// What the build expects a cycle to find. The gates compare these with
/// what the collector actually marked.
#[derive(Debug, Clone, Copy, Default)]
pub struct Expect {
    /// Nodes that must be marked live.
    pub live_nodes: u64,
    /// Non-node cage objects that must be marked live.
    pub live_others: u64,
    /// Nodes that must not be marked.
    pub dead_nodes: u64,
    /// Wrappers created.
    pub wrappers: u64,
    /// The deepest nesting the generator actually produced.
    pub max_depth: usize,
    /// The mean nesting depth it actually produced.
    pub mean_depth: f64,
}

/// The parent of every attached node, and the depths reached. Node ids
/// run 1..=attached, with 1 the document.
fn parents(shape: &Shape) -> (Vec<NodeId>, usize, f64) {
    let n = shape.attached;
    let mut parent = vec![0u32; n + 1];
    let mut depth = vec![0usize; n + 1];
    let mut path: Vec<NodeId> = vec![1];
    let mut rng = Rng::new(shape.seed);
    let mut sum = 0usize;
    let mut max = 0usize;
    for id in 2..=n as u32 {
        // Descend while there is room, else close some elements. 0.62
        // keeps the mean depth near half the limit, which is what the
        // published depth buckets show; the achieved depths are reported
        // rather than assumed.
        if path.len() >= shape.depth_limit || rng.unit() >= 0.62 {
            let close = 1 + rng.below(path.len().saturating_sub(1).max(1));
            for _ in 0..close {
                if path.len() > 1 {
                    path.pop();
                }
            }
        }
        let p = *path.last().expect("the document is never popped");
        parent[id as usize] = p;
        depth[id as usize] = depth[p as usize] + 1;
        sum += depth[id as usize];
        max = max.max(depth[id as usize]);
        path.push(id);
    }
    (parent, max, sum as f64 / (n.max(1) as f64))
}

/// Which attached nodes are touched, and which of those are also held by
/// a JS object. Orphans are never touched, so the dead-node gate has a
/// clean expectation.
fn touched_sets(shape: &Shape) -> (Vec<NodeId>, Vec<NodeId>) {
    let mut rng = Rng::new(shape.seed ^ 0x5EED);
    let mut touched = Vec::new();
    let mut rooted = Vec::new();
    for id in 2..=shape.attached as u32 {
        if rng.unit() < shape.touched {
            touched.push(id);
            if rng.unit() < shape.rooted_touched {
                rooted.push(id);
            }
        }
    }
    (touched, rooted)
}

/// How many plain JS objects the heap gets. The same in both designs, so
/// "share of GC time" has the same denominator.
fn js_objects(shape: &Shape) -> usize {
    (shape.attached / 4).max(1)
}

/// A cage big enough for everything a shape allocates, with room for the
/// evacuation copies: the next power of two above four times the need.
fn cage_for(bytes: u64) -> u64 {
    (bytes.saturating_mul(4).max(1 << 16)).next_power_of_two()
}

/// Builds design (a): columns outside the cage, wrappers inside it.
pub fn build_a(shape: &Shape, node_ref: u32, evac: f64) -> (HeapA, Expect) {
    let (parent, max_depth, mean_depth) = parents(shape);
    let (touched, rooted) = touched_sets(shape);
    let slots = shape.attached + shape.orphans + 1;
    let mut space = NodeSpace::new(slots);

    // Link the tree: parent, first and last child, previous and next
    // sibling, as the five link columns.
    let mut last_child = vec![0u32; slots];
    for id in 2..=shape.attached as u32 {
        let p = parent[id as usize];
        space.links[0][id as usize].store(p, ORD);
        let prev = last_child[p as usize];
        if prev == 0 {
            space.links[1][p as usize].store(id, ORD);
        } else {
            space.links[3][id as usize].store(prev, ORD);
            space.links[4][prev as usize].store(id, ORD);
        }
        space.links[2][p as usize].store(id, ORD);
        last_child[p as usize] = id;
    }
    // Every allocated slot gets a generation, so the sweep can tell an
    // allocated slot from a never-used one.
    for id in 1..slots as u32 {
        space.rest[REST_GENERATION][id as usize].store(1, ORD);
    }

    let wrapper_size = u64::from(model::wrapper_bytes(node_ref));
    let js = js_objects(shape);
    let types = type_table(3, u64::from(node_ref).div_ceil(8) as u32);
    let need = wrapper_size * touched.len() as u64 + types[TY_JS as usize].size() * js as u64;
    let mut cage = Cage::new(cage_for(need));

    // The wrappers, and the node's external traced slot that holds each.
    let mut wrapper_of = std::collections::HashMap::new();
    for &id in &touched {
        let off = cage.alloc(wrapper_size).expect("cage sized for wrappers");
        cage.store32(off, TY_WRAPPER);
        cage.store32(off + 4, 0);
        // The node reference: the open decision, written where a declared
        // node field would sit (after the traced slots, of which the
        // wrapper has none).
        cage.store32(off + u64::from(model::CAGE_HEADER_BYTES), id);
        let r = space.add_rare(id);
        space.rare_wrapper[r as usize].store(encode_ref(off), ORD);
        wrapper_of.insert(id, off);
    }

    // The plain JS heap: a chain, with the rooted wrappers hung off it.
    let mut roots = Vec::new();
    let mut prev = 0u64;
    for k in 0..js {
        let off = cage
            .alloc(types[TY_JS as usize].size())
            .expect("cage sized for the JS heap");
        cage.store32(off, TY_JS);
        cage.store32(off + 4, 0);
        if prev != 0 {
            cage.store32(slot_off(prev, 0), encode_ref(off));
        } else {
            roots.push(off);
        }
        if let Some(&w) = rooted.get(k).and_then(|id| wrapper_of.get(id)) {
            cage.store32(slot_off(off, 1), encode_ref(w));
        }
        prev = off;
    }

    let expect = Expect {
        live_nodes: shape.attached as u64,
        live_others: (js + touched.len()) as u64,
        dead_nodes: shape.orphans as u64,
        wrappers: touched.len() as u64,
        max_depth,
        mean_depth,
    };
    (
        HeapA {
            space,
            cage,
            types,
            document: 1,
            roots,
            // The wrappers were allocated first, so the oldest of them
            // are the lowest offsets: the fragmented blocks a normal
            // cycle picks (H-7's opportunistic evacuation).
            candidate_end: 8 + ((wrapper_size * touched.len() as u64) as f64 * evac) as u64,
        },
        expect,
    )
}

/// Builds design (c): the node is a cage heap object. `with_id` adds the
/// generation field and the stable-ID table.
pub fn build_c(shape: &Shape, with_id: bool, evac: f64) -> (HeapC, Expect) {
    let (parent, max_depth, mean_depth) = parents(shape);
    let (_, rooted) = touched_sets(shape);
    let types = type_table(if with_id { 4 } else { 3 }, 1);
    let node_size = types[TY_NODE as usize].size();
    let js = js_objects(shape);
    let total = shape.attached + shape.orphans;
    let need = node_size * total as u64 + types[TY_JS as usize].size() * js as u64;
    let mut cage = Cage::new(cage_for(need));

    // Allocate every node, attached and orphan, in document order.
    let mut off_of = vec![0u64; total + 1];
    for id in 1..=total as u32 {
        let off = cage.alloc(node_size).expect("cage sized for nodes");
        cage.store32(off, TY_NODE);
        cage.store32(off + 4, 0);
        off_of[id as usize] = off;
    }
    let mut last_child = vec![0u32; total + 1];
    for id in 2..=shape.attached as u32 {
        let p = parent[id as usize];
        let (me, pa) = (off_of[id as usize], off_of[p as usize]);
        cage.store32(slot_off(me, 0), encode_ref(pa));
        let prev = last_child[p as usize];
        if prev == 0 {
            cage.store32(slot_off(pa, 1), encode_ref(me));
        } else {
            let pv = off_of[prev as usize];
            cage.store32(slot_off(me, 3), encode_ref(pv));
            cage.store32(slot_off(pv, 4), encode_ref(me));
        }
        cage.store32(slot_off(pa, 2), encode_ref(me));
        last_child[p as usize] = id;
    }

    // The plain JS heap, with the touched nodes referenced directly: in
    // (c) there is no wrapper, so script holds the node object itself.
    let mut roots = Vec::new();
    let mut prev = 0u64;
    for k in 0..js {
        let off = cage
            .alloc(types[TY_JS as usize].size())
            .expect("cage sized for the JS heap");
        cage.store32(off, TY_JS);
        cage.store32(off + 4, 0);
        if prev != 0 {
            cage.store32(slot_off(prev, 0), encode_ref(off));
        } else {
            roots.push(off);
        }
        if let Some(&id) = rooted.get(k) {
            cage.store32(slot_off(off, 1), encode_ref(off_of[id as usize]));
        }
        prev = off;
    }

    let id_table = if with_id {
        (1..=shape.attached as u32)
            .map(|id| encode_ref(off_of[id as usize]))
            .collect()
    } else {
        Vec::new()
    };

    let expect = Expect {
        live_nodes: shape.attached as u64,
        live_others: js as u64,
        dead_nodes: shape.orphans as u64,
        wrappers: 0,
        max_depth,
        mean_depth,
    };
    (
        HeapC {
            cage,
            types,
            document: off_of[1],
            roots,
            candidate_end: 8 + ((node_size * total as u64) as f64 * evac) as u64,
            id_table,
        },
        expect,
    )
}

/// A helper the gates use: the number of rare records (a) created.
pub fn rare_count(h: &HeapA) -> usize {
    h.space.rare_count()
}
