//! JS-D4's spike: bytes per node across every copy, and the node space's
//! share of GC time, for options (a) and (c).
//!
//! Usage:
//!
//! ```text
//! d4-nodes gates            # the validity gates alone
//! d4-nodes bytes            # the byte accounting sweep, as TSV
//! d4-nodes gc [repeats]     # the GC-time measurement, as TSV
//! ```
//!
//! Every mode runs the gates first. A failed gate prints `FAIL` and exits
//! non-zero instead of printing a measurement: a fast wrong number is the
//! failure mode this spike most has to avoid.

mod heap;
mod model;
mod tree;

use std::process::ExitCode;
use std::sync::atomic::Ordering;

use heap::{Cage, HeapA, NodeSpace, REST_GENERATION, TY_JS, TY_WRAPPER, encode_ref, type_table};
use model::{Design, NODE_REF_BYTES_DEFAULT, NODE_REF_BYTES_SWEEP};
use tree::Shape;

/// The sizes the trees are built at, from the Web Almanac's 2024 Markup
/// chapter, Figure 2.11 (elements per page, mobile): the 10th, 25th,
/// 50th, 75th and 90th percentiles, then two larger sizes so the trace
/// leaves the caches. The note records the edition and the figure.
const SIZES: [(&str, usize); 7] = [
    ("almanac2024-p10", 180),
    ("almanac2024-p25", 342),
    ("almanac2024-p50", 594),
    ("almanac2024-p75", 1_010),
    ("almanac2024-p90", 1_716),
    ("large-50k", 50_000),
    ("large-500k", 500_000),
];

/// The deepest nesting the generators produce. The published depth
/// distribution puts the median site in the 15-to-20-level bucket (see
/// the note), so 18 is the representative limit and 40 the tail.
const DEPTHS: [usize; 2] = [18, 40];

/// The touched fractions swept. No published figure exists for the real
/// one (DOM spec Q-DM-6), so the comparison is reported as a function of
/// it and the crossover is computed.
const TOUCHED: [f64; 6] = [0.0, 0.01, 0.05, 0.10, 0.33, 1.00];

/// Mean text bytes per node. Reported but never compared: both designs
/// store text once, so it cancels. FP-11 and DM-30 both count per-node
/// bytes with text excluded.
const TEXT_BYTES: f64 = 0.0;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("gates");

    let mut failures = Vec::new();
    run_gates(&mut failures);
    for f in &failures {
        println!("FAIL\t{f}");
    }
    if !failures.is_empty() {
        eprintln!(
            "d4-nodes: {} validity gate(s) failed; no measurement printed",
            failures.len()
        );
        return ExitCode::FAILURE;
    }
    eprintln!("d4-nodes: all validity gates passed");

    match mode {
        "gates" => {
            println!("gates\tPASS");
            ExitCode::SUCCESS
        }
        "bytes" => {
            bytes_sweep();
            ExitCode::SUCCESS
        }
        "gc" => {
            let repeats = args
                .get(1)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(5);
            gc_sweep(repeats);
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("d4-nodes: unknown mode {other}");
            ExitCode::FAILURE
        }
    }
}

// --- the validity gates -----------------------------------------------------

fn run_gates(out: &mut Vec<String>) {
    gate_column_sum(out);
    gate_model_matches_allocation(out);
    gate_live_and_dead(out);
    gate_evacuation_preserves_the_graph(out);
    gate_strong_wrapper_keeps_a_detached_node(out);
}

/// G1. The column list must sum to the 48-byte core the JS spec states.
/// A mis-transcribed field list would otherwise shift every (a) total.
fn gate_column_sum(out: &mut Vec<String>) {
    let sum: u32 = model::COLUMN_FIELDS.iter().map(|&(_, b)| b).sum();
    if sum != model::COLUMNS_BYTES {
        out.push(format!(
            "G1 column-sum: the field list sums to {sum}, not the spec's {}",
            model::COLUMNS_BYTES
        ));
    }
}

/// G2. The arithmetic model must agree with what the harness really
/// allocated. This is the gate that catches a per-node figure derived
/// from a layout the code does not actually build.
fn gate_model_matches_allocation(out: &mut Vec<String>) {
    let shape = Shape {
        attached: 2_000,
        orphans: 0,
        depth_limit: 18,
        touched: 0.25,
        rooted_touched: 0.5,
        seed: 1,
    };
    let node_ref = NODE_REF_BYTES_DEFAULT;
    let (h, e) = tree::build_a(&shape, node_ref, 0.0);

    // The node space's own allocation: the columns, the side mark bitmap
    // and the rare-data records. Compared against the model evaluated
    // over the same slot count and the same wrapper count, so the two
    // sides describe the same heap.
    let slots = h.space.slots() as f64;
    let touched_real = e.wrappers as f64 / slots;
    let p = model::per_node(Design::A, touched_real, 0.0, node_ref);
    let modelled = p.node_space_bytes() * slots;
    let measured = h.space.bytes() as f64;
    // The model charges an eighth of a byte per slot for the mark bit;
    // the allocation rounds it up to whole 64-bit words.
    let slack = 8.0 * (slots / 64.0).ceil() - slots / 8.0 + 8.0;
    if (modelled - measured).abs() > slack {
        out.push(format!(
            "G2 node space: model {modelled:.0} B, allocated {measured:.0} B, slack {slack:.0} B"
        ));
    }

    // The cage side: the wrappers the model predicts against those the
    // builder really allocated.
    let modelled_wrappers = p.wrapper * slots;
    let allocated_wrappers = f64::from(model::wrapper_bytes(node_ref)) * e.wrappers as f64;
    if (modelled_wrappers - allocated_wrappers).abs() > 1.0 {
        out.push(format!(
            "G2 wrappers: model {modelled_wrappers:.0} B, allocated {allocated_wrappers:.0} B"
        ));
    }

    if tree::rare_count(&h) != e.wrappers as usize {
        out.push(format!(
            "G2 rare-records: {} records for {} wrappers",
            tree::rare_count(&h),
            e.wrappers
        ));
    }
}

/// G3 and G4. A cycle must mark every attached node and no orphan, in
/// both designs. A trace that fell out of the tree would be fast and
/// wrong, which is exactly what this spike must not report.
fn gate_live_and_dead(out: &mut Vec<String>) {
    let shape = Shape {
        attached: 5_000,
        orphans: 1_000,
        depth_limit: 18,
        touched: 0.1,
        rooted_touched: 0.5,
        seed: 7,
    };
    let (mut a, ea) = tree::build_a(&shape, NODE_REF_BYTES_DEFAULT, 0.0);
    let ca = a.collect();
    if ca.stale != 0 {
        out.push(format!("G3 (a): {} stale slots after a cycle", ca.stale));
    }
    if ca.others_marked != ea.live_others {
        out.push(format!(
            "G3 (a) live cage objects: marked {}, expected {}",
            ca.others_marked, ea.live_others
        ));
    }
    if ca.nodes_marked != ea.live_nodes {
        out.push(format!(
            "G3 (a) live nodes: marked {}, expected {}",
            ca.nodes_marked, ea.live_nodes
        ));
    }
    assert_eq!(ea.dead_nodes, shape.orphans as u64);
    for id in (shape.attached as u32 + 1)..=(shape.attached + ea.dead_nodes as usize) as u32 {
        if a.space.is_marked(id) {
            out.push(format!("G4 (a): orphan node {id} survived a cycle"));
            break;
        }
    }
    for (label, with_id) in [("c-min", false), ("c-id", true)] {
        let (mut c, ec) = tree::build_c(&shape, with_id, 0.0);
        let cc = c.collect();
        if cc.nodes_marked != ec.live_nodes {
            out.push(format!(
                "G3 ({label}) live nodes: marked {}, expected {}",
                cc.nodes_marked, ec.live_nodes
            ));
        }
        if cc.stale != 0 {
            out.push(format!("G3 ({label}): {} stale slots", cc.stale));
        }
    }
}

/// G5. After evacuation every live node must still be reachable: the
/// second cycle must mark exactly what the first did. A collector that
/// failed to update a slot would lose a subtree and look fast.
fn gate_evacuation_preserves_the_graph(out: &mut Vec<String>) {
    let shape = Shape {
        attached: 3_000,
        orphans: 0,
        depth_limit: 18,
        touched: 0.2,
        rooted_touched: 0.5,
        seed: 11,
    };
    let (mut a, _) = tree::build_a(&shape, NODE_REF_BYTES_DEFAULT, 1.0);
    let first = a.collect();
    let second = a.collect();
    if first.stale != 0 || second.stale != 0 {
        out.push(format!(
            "G5 (a) stale slots: {} in the evacuating cycle, {} in the next",
            first.stale, second.stale
        ));
    }
    if first.moved == 0 {
        out.push("G5 (a): evacuation moved nothing".into());
    }
    if first.nodes_marked != second.nodes_marked || first.others_marked != second.others_marked {
        out.push(format!(
            "G5 (a) after evacuation: {} nodes/{} others, then {} nodes/{} others",
            first.nodes_marked, first.others_marked, second.nodes_marked, second.others_marked
        ));
    }
    for (label, with_id) in [("c-min", false), ("c-id", true)] {
        let (mut c, _) = tree::build_c(&shape, with_id, 0.5);
        let first = c.collect();
        let second = c.collect();
        if first.nodes_marked != second.nodes_marked {
            out.push(format!(
                "G5 ({label}) after evacuation: {} nodes, then {} nodes",
                first.nodes_marked, second.nodes_marked
            ));
        }
        if first.moved == 0 {
            out.push(format!("G5 ({label}): evacuation moved nothing"));
        }
        if second.moved != 0 {
            out.push(format!(
                "G5 ({label}): the second cycle moved {} objects, but its candidate set was already evacuated",
                second.moved
            ));
        }
        if first.stale != 0 || second.stale != 0 {
            out.push(format!(
                "G5 ({label}) stale slots: {} then {}",
                first.stale, second.stale
            ));
        }
    }
}

/// G6. The strong-once-created rule: a node detached from the tree whose
/// wrapper a JS object still holds must survive. This is the direction
/// that needs the cage-to-node edge, so it is also the gate that would
/// fail if that edge were not traced.
fn gate_strong_wrapper_keeps_a_detached_node(out: &mut Vec<String>) {
    let node_ref = NODE_REF_BYTES_DEFAULT;
    let types = type_table(3, u64::from(node_ref).div_ceil(8) as u32);
    let mut cage = Cage::new(1 << 16);
    let mut space = NodeSpace::new(64);
    // Two slots: 1 is the document, 2 is detached and script-held.
    for id in 1..=2u32 {
        space.rest[REST_GENERATION][id as usize].store(1, Ordering::Relaxed);
    }
    let detached = 2u32;
    let w = cage
        .alloc(u64::from(model::wrapper_bytes(node_ref)))
        .expect("room");
    cage.store32(w, TY_WRAPPER);
    cage.store32(w + 4, 0);
    cage.store32(w + u64::from(model::CAGE_HEADER_BYTES), detached);
    let r = space.add_rare(detached);
    space.rare_wrapper[r as usize].store(encode_ref(w), Ordering::Relaxed);
    // A rooted JS object holds the wrapper.
    let root = cage.alloc(types[TY_JS as usize].size()).expect("room");
    cage.store32(root, TY_JS);
    cage.store32(root + 4, 0);
    cage.store32(heap::slot_off(root, 1), encode_ref(w));

    let mut h = HeapA {
        space,
        cage,
        types,
        document: 1,
        roots: vec![root],
        candidate_end: 0,
    };
    let c = h.collect();
    if !h.space.is_marked(detached) {
        out.push(
            "G6 strong wrapper: a detached node whose wrapper is rooted did not survive".into(),
        );
    }
    if c.nodes_marked != 2 {
        out.push(format!(
            "G6 strong wrapper: marked {} nodes, expected 2 (document and the detached node)",
            c.nodes_marked
        ));
    }
}

// --- mode: bytes ------------------------------------------------------------

fn bytes_sweep() {
    println!(
        "design\tnode_ref_bytes\ttouched\tcolumns\tmark_bitmap\tmark_bitmap_wrapper\trare_data\twrapper\tcage_object\tid_table\ttotal_no_text"
    );
    for &design in &[Design::A, Design::CMin, Design::CId] {
        for &nr in &NODE_REF_BYTES_SWEEP {
            // (c) has no wrapper, so the node-reference parameter cannot
            // change it; print it once rather than three identical rows.
            if design != Design::A && nr != NODE_REF_BYTES_DEFAULT {
                continue;
            }
            for &t in &TOUCHED {
                let p = model::per_node(design, t, TEXT_BYTES, nr);
                println!(
                    "{}\t{}\t{:.2}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                    design.label(),
                    nr,
                    t,
                    p.columns,
                    p.mark_bitmap,
                    p.mark_bitmap_wrapper,
                    p.rare_data,
                    p.wrapper,
                    p.cage_object,
                    p.id_table,
                    p.total_without_text()
                );
            }
        }
    }
    println!();
    println!("crossover_design\tbase\tnode_ref_bytes\tratio\ttouched_fraction");
    for &base in &[Design::CMin, Design::CId] {
        for &nr in &NODE_REF_BYTES_SWEEP {
            for &(name, ratio) in &[("equal", 1.0), ("trigger-1.10", 1.10)] {
                let x = model::crossover(Design::A, base, TEXT_BYTES, nr, ratio);
                println!(
                    "a\t{}\t{}\t{}\t{}",
                    base.label(),
                    nr,
                    name,
                    x.map_or("never".to_string(), |v| format!("{v:.4}"))
                );
            }
        }
    }
}

// --- mode: gc ---------------------------------------------------------------

fn gc_sweep(repeats: usize) {
    println!(
        "size\tnodes\tdepth_limit\tmax_depth\tmean_depth\tdesign\ttouched\tevac\trepeat\tnodes_ns\tother_ns\tsweep_ns\ttotal_ns\tnode_share\tns_per_node\tnodes_marked\tmoved"
    );
    for &(label, n) in &SIZES {
        for &depth in &DEPTHS {
            for &touched in &[0.05f64, 0.33] {
                for &evac in &[0.0f64, 0.25] {
                    let shape = Shape {
                        attached: n,
                        orphans: n / 10,
                        depth_limit: depth,
                        touched,
                        rooted_touched: 0.5,
                        seed: 0xD4u64 ^ n as u64,
                    };
                    for r in 0..repeats {
                        // A fresh heap per repeat: a cycle that has
                        // already evacuated has nothing left to move, so
                        // reusing one would measure the second cycle.
                        let (mut a, ea) = tree::build_a(&shape, NODE_REF_BYTES_DEFAULT, evac);
                        let c = a.collect();
                        row(label, n, depth, &ea, "a", touched, evac, r, &c);
                        for (dl, with_id) in [("c-min", false), ("c-id", true)] {
                            let (mut h, e) = tree::build_c(&shape, with_id, evac);
                            let c = h.collect();
                            row(label, n, depth, &e, dl, touched, evac, r, &c);
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn row(
    label: &str,
    n: usize,
    depth: usize,
    e: &tree::Expect,
    design: &str,
    touched: f64,
    evac: f64,
    repeat: usize,
    c: &heap::Cycle,
) {
    let p = c.phases;
    let per = if c.nodes_marked == 0 {
        0.0
    } else {
        p.nodes_ns as f64 / c.nodes_marked as f64
    };
    println!(
        "{label}\t{n}\t{depth}\t{}\t{:.2}\t{design}\t{touched:.2}\t{evac:.2}\t{repeat}\t{}\t{}\t{}\t{}\t{:.4}\t{:.2}\t{}\t{}",
        e.max_depth,
        e.mean_depth,
        p.nodes_ns,
        p.other_ns,
        p.sweep_ns,
        p.total_ns(),
        p.node_share(),
        per,
        c.nodes_marked,
        c.moved
    );
}
