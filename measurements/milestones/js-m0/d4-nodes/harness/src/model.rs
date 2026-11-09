//! Bytes per node, as arithmetic over the field lists the specs fix.
//!
//! The metric JS-D4 names is "total bytes per node across every copy", so
//! every term is kept separate and printed: a total that hides which copy
//! it came from cannot be checked. The field lists are the JS spec's
//! (Section 6.2, the node-column enumeration) and the heap design's
//! object layout (heap.md H-4). Nothing here is measured; it is the model
//! that `heap.rs` then checks against what it really allocated.

/// How many bytes a cage object spends naming a node.
///
/// **This is the open decision.** JS-D4 (a) requires the wrapper to hold
/// "the node's generational ID" and the collector to follow that edge,
/// but neither heap.md nor the JS spec fixes its representation: H-4's
/// object layout has no node-reference field kind and JS-D1's slot
/// encoding has only two tags. The alternatives, and what each costs
/// here:
///
/// * `4` — a slot index alone, with no epoch. Cheapest, but it gives up
///   the stale-handle detection that Stage 1 5.2 and ST 6.x rely on, so
///   it is listed for the sweep's lower bound rather than as a candidate.
/// * `8` — slot plus epoch in one 8-byte field. The likely answer: a new
///   `node` field kind in `Layout`, or one raw word declared as a node
///   reference. This is the default.
/// * `12` — a 4-byte slot that the collector traces plus a separate
///   8-byte raw word for the epoch, if the two cannot share a field.
///
/// Every total below is reported as a function of this value, so the
/// owner's answer can be substituted without redoing the work.
pub const NODE_REF_BYTES_DEFAULT: u32 = 8;

/// The values swept for [`NODE_REF_BYTES_DEFAULT`].
pub const NODE_REF_BYTES_SWEEP: [u32; 3] = [4, 8, 12];

/// Which of JS-D4's layouts is being counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Design {
    /// (a), the chosen option: node columns outside the cage, traced as a
    /// node space; a lazily created wrapper held strongly once created;
    /// text in an arena shared with JS as external strings.
    A,
    /// (c), the trigger's comparand, counted at its smallest: the node is
    /// a cage heap object and nothing outside the cage needs a stable
    /// name for it.
    CMin,
    /// (c) with the stable-ID indirection that the mutation log, the
    /// semantic plane and B8's stable node IDs need once nodes can move.
    CId,
}

impl Design {
    /// The name used in the TSV and the note.
    pub fn label(self) -> &'static str {
        match self {
            Design::A => "a",
            Design::CMin => "c-min",
            Design::CId => "c-id",
        }
    }
}

/// One node's bytes, by the copy they live in. Fractional because the
/// mark bitmap costs an eighth of a byte.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PerNode {
    /// (a): the node columns outside the cage.
    pub columns: f64,
    /// The collector's mark bit for the node itself: one bit per slot in
    /// (a)'s side bitmap, one per 8-byte granule of the object in (c).
    /// Outside the cage in both.
    pub mark_bitmap: f64,
    /// (a): the granule mark bits of the wrapper, which is an ordinary
    /// cage object and so falls under the granule bitmap.
    pub mark_bitmap_wrapper: f64,
    /// (a): the rare-data record, which is where the wrapper's external
    /// traced slot lives (DOM spec 6.4). Only nodes that have a wrapper
    /// pay it here; no other rare field is modelled.
    pub rare_data: f64,
    /// (a): the wrapper object in the cage, for nodes script has touched.
    pub wrapper: f64,
    /// (c): the node itself, as a cage heap object under H-4.
    pub cage_object: f64,
    /// (c): the stable-ID table that survives evacuation.
    pub id_table: f64,
    /// DOM text, stored once in both designs.
    pub text: f64,
}

impl PerNode {
    /// Total bytes per node across every copy.
    pub fn total(&self) -> f64 {
        self.columns
            + self.mark_bitmap
            + self.mark_bitmap_wrapper
            + self.rare_data
            + self.wrapper
            + self.cage_object
            + self.id_table
            + self.text
    }

    /// The part of the total that lives in the node space itself: the
    /// columns, the side mark bitmap and the rare-data records. The gate
    /// that checks the model against what was really allocated compares
    /// this, because it is exactly what `NodeSpace` allocates.
    pub fn node_space_bytes(&self) -> f64 {
        self.columns + self.mark_bitmap + self.rare_data
    }

    /// The total without text, which is the same in both designs and so
    /// discriminates nothing. FP-11 and DM-30 both count per-node bytes
    /// with text excluded, so this is the number to compare.
    pub fn total_without_text(&self) -> f64 {
        self.total() - self.text
    }
}

/// (a)'s node columns, field by field, from the JS spec's Section 6.2
/// list: five links, tag, flags and element state, attribute-set
/// reference, inline-style reference, rare-data index, generation. Every
/// one is a 32-bit column except flags and element state, which the spec
/// fixes at 8 bytes ("8 B hold 32 bits of node flags and 32 bits of the
/// most used element state").
pub const COLUMN_FIELDS: [(&str, u32); 12] = [
    ("parent", 4),
    ("first_child", 4),
    ("last_child", 4),
    ("prev_sibling", 4),
    ("next_sibling", 4),
    ("tag", 4),
    ("flags", 4),
    ("element_state", 4),
    ("attribute_set", 4),
    ("inline_style", 4),
    ("rare_index", 4),
    ("generation", 4),
];

/// The sum of [`COLUMN_FIELDS`]. The JS spec calls this the 48 B† core,
/// so a mismatch means the field list was mis-transcribed.
pub const COLUMNS_BYTES: u32 = 48;

/// (a)'s rare-data record as modelled: the wrapper's external traced slot
/// (a 32-bit atomic, JS spec 6.5) and the record's own link. Only the
/// wrapper field is modelled; a node with listeners or observers pays
/// more, equally in both designs, so it is left out of both.
pub const RARE_RECORD_BYTES: u32 = 8;

/// A cage object's header: the type index and the length word (H-4).
pub const CAGE_HEADER_BYTES: u32 = 8;

/// (c)'s stable-ID table entry: a 32-bit current slot per live ID.
pub const ID_TABLE_BYTES: u32 = 4;

/// Rounds `n` up to a multiple of 8, as every cage object is (H-4).
const fn align8(n: u32) -> u32 {
    n.next_multiple_of(8)
}

/// The size of a cage object with `traced` 32-bit traced slots and
/// `raw_words` 64-bit raw words, under H-4's layout: header, then the
/// slots padded to 8, then the raw words.
pub const fn cage_object_bytes(traced: u32, raw_words: u32) -> u32 {
    CAGE_HEADER_BYTES + align8(traced * 4) + raw_words * 8
}

/// The wrapper object in (a): a cage object whose only content is the
/// node reference. With `node_ref` of 4 the reference still occupies a
/// whole 8-byte-aligned body, so 4 and 8 give the same wrapper size; the
/// difference between them shows up only in (c)-style layouts and is
/// reported anyway so the sweep is legible.
pub const fn wrapper_bytes(node_ref: u32) -> u32 {
    CAGE_HEADER_BYTES + align8(node_ref)
}

/// (c)'s node object: five links as traced slots, then the remaining
/// fields packed two-per-raw-word. `c-min` packs tag+flags,
/// state+attribute-set and inline-style+rare-index into three words;
/// `c-id` adds the generation, which needs a fourth.
pub const fn c_node_bytes(with_generation: bool) -> u32 {
    cage_object_bytes(5, if with_generation { 4 } else { 3 })
}

/// The bytes one node costs under `design`, with `touched` the fraction
/// of nodes script has touched (so that a wrapper exists for them),
/// `text` the mean text bytes per node, and `node_ref` the open decision
/// above.
///
/// `touched` is the term with no published value: the DOM spec's Q-DM-6
/// records that no figure exists for the fraction of nodes a real page's
/// script touches, so every total here is a function of it and the note
/// reports the crossover rather than a single number.
pub fn per_node(design: Design, touched: f64, text: f64, node_ref: u32) -> PerNode {
    let mut p = PerNode {
        text,
        ..PerNode::default()
    };
    match design {
        Design::A => {
            p.columns = f64::from(COLUMNS_BYTES);
            // One bit per node slot, plus — for a node whose wrapper
            // exists — one bit per 8-byte granule of that wrapper, since
            // it is an ordinary cage object under the granule bitmap.
            let wrapper = f64::from(wrapper_bytes(node_ref));
            p.mark_bitmap = 0.125;
            p.mark_bitmap_wrapper = touched * wrapper / 8.0 / 8.0;
            p.rare_data = touched * f64::from(RARE_RECORD_BYTES);
            p.wrapper = touched * wrapper;
        }
        Design::CMin | Design::CId => {
            let has_gen = design == Design::CId;
            p.cage_object = f64::from(c_node_bytes(has_gen));
            // One bit per 8-byte granule of the object.
            p.mark_bitmap = f64::from(c_node_bytes(has_gen)) / 8.0 / 8.0;
            if has_gen {
                p.id_table = f64::from(ID_TABLE_BYTES);
            }
            // No wrapper: in (c) the cage object *is* the JS object, so a
            // touched node costs nothing extra. This is (c)'s advantage
            // and it is given to it in full.
        }
    }
    p
}

/// The touched fraction at which `design` first costs as much per node as
/// `base`, or `None` if it never does over `[0, 1]`. This is the number
/// the comparison turns on, since `touched` is unmeasured.
pub fn crossover(
    design: Design,
    base: Design,
    text: f64,
    node_ref: u32,
    ratio: f64,
) -> Option<f64> {
    let mut lo = 0.0f64;
    let mut hi = 1.0f64;
    let f = |t: f64| {
        per_node(design, t, text, node_ref).total_without_text()
            - ratio * per_node(base, t, text, node_ref).total_without_text()
    };
    if f(lo) > 0.0 || f(hi) < 0.0 {
        return None;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if f(mid) < 0.0 { lo = mid } else { hi = mid }
    }
    Some(0.5 * (lo + hi))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_column_list_sums_to_the_spec_s_core() {
        let sum: u32 = COLUMN_FIELDS.iter().map(|&(_, b)| b).sum();
        assert_eq!(sum, COLUMNS_BYTES, "the 48 B core of JS spec 6.2");
    }

    #[test]
    fn h4_layout_sizes() {
        // Five traced slots pad from 20 to 24.
        assert_eq!(cage_object_bytes(5, 0), 8 + 24);
        assert_eq!(c_node_bytes(false), 56);
        assert_eq!(c_node_bytes(true), 64);
        assert_eq!(wrapper_bytes(8), 16);
        assert_eq!(wrapper_bytes(12), 24);
    }

    #[test]
    fn an_untouched_page_favours_a_and_a_fully_touched_one_favours_c() {
        let a0 = per_node(Design::A, 0.0, 0.0, 8).total_without_text();
        let a1 = per_node(Design::A, 1.0, 0.0, 8).total_without_text();
        let c = per_node(Design::CMin, 0.0, 0.0, 8).total_without_text();
        assert!(a0 < c, "untouched: {a0} vs {c}");
        assert!(a1 > c, "fully touched: {a1} vs {c}");
    }
}
