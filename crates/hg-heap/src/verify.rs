//! The heap verifier (JS spec 6.5, soundness step 8; JS-M0 plan Phase 2
//! item 4). After every collection in a debug build it walks the heap
//! from the roots and checks what the collector is supposed to have left
//! behind. A violation is a bug in the collector, so it panics rather
//! than returning: a heap that has lost an object cannot be allowed to
//! keep running and produce a plausible wrong answer.
//!
//! **What it is for.** The bug class it exists to catch is an
//! **evacuation that moved an object without updating every reference to
//! it** (heap design H-7: "every slot that reached it is updated"). That
//! bug is reachable — the JS-D4 spike's own harness had it, and its
//! symptom there was a cycle that reported a 50% larger live set as if it
//! were correct — and it is invisible to an ordinary test, because the
//! stale reference still decodes to an 8-aligned offset inside the cage
//! (H-2 guarantees that much) and reads back whatever now lives there.
//! Only a walk that insists every reachable slot names a *live, marked*
//! object of *this* heap catches it.
//!
//! **What it is not for.** It assumes the cage holds what the collector
//! and the mutator put there. Under the in-cage corruption API (JS spec
//! 6.1) that assumption is false by design: H-2's promise is that
//! corrupted contents give wrong answers inside the cage, never an access
//! outside it, and a corrupted slot legitimately names a dead object. So
//! a fuzzer that writes arbitrary words into the cage turns the verifier
//! off ([`Mutator::set_verify`](crate::Mutator::set_verify)); the two
//! check different properties and must not be run against each other.
//!
//! No `unsafe` code: every cage read goes through the `cage` module's
//! existing masked, aligned, atomic accessors, and every piece of
//! collector bookkeeping it consults is ordinary Rust data outside the
//! cage.

use std::collections::BTreeMap;

use crate::heap::State;
use crate::object::{Layout, Tail};
use crate::slot::Slot;

/// What the verifier found wrong. Every variant names where, so a
/// failure points at an object and a slot rather than at "the
/// collector". `holder` is 0 when the slot is a root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Violation {
    /// A reachable slot names memory **this heap owns** that is not a
    /// live object: a block the sweep freed, the interior of a large
    /// object, or a block never handed out. **This is what an evacuation
    /// that forgot to update a reference leaves behind**, since the old
    /// copy's block is freed by the same sweep. A slot naming another
    /// heap's chunk is a different case and is counted in
    /// [`Audit::foreign`], not reported here (H-12).
    Dangling {
        /// The object holding the slot, or 0 for a root.
        holder: u64,
        /// The slot's cage offset.
        slot: u64,
        /// What it names.
        target: u64,
    },
    /// A reachable object the cycle did not mark. The collector reached
    /// it or it would not be reachable, so a missing mark means the sweep
    /// was free to give its lines away.
    Unmarked {
        /// The object.
        at: u64,
    },
    /// A reachable object whose header does not name a type in the type
    /// table, or whose length makes it unmeasurable.
    BadHeader {
        /// The object.
        at: u64,
    },
    /// A live object covers a line its block does not have marked, so the
    /// allocator could hand out memory inside it.
    UnmarkedLine {
        /// The object.
        at: u64,
        /// The line, counted from the start of its block.
        line: usize,
    },
    /// Two live objects overlap.
    Overlap {
        /// The lower object.
        at: u64,
        /// Its size.
        size: u64,
        /// The next object's offset, which is inside it.
        next: u64,
    },
    /// A weak slot still names an object that is not live, after a cycle
    /// that should have cleared or updated it (heap design H-10).
    DeadWeak {
        /// The object holding the weak slot.
        holder: u64,
        /// The slot's cage offset.
        slot: u64,
        /// What it names.
        target: u64,
    },
}

impl core::fmt::Display for Violation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Dangling {
                holder,
                slot,
                target,
            } => write!(
                f,
                "slot {slot:#x} of object {holder:#x} names {target:#x}, which is not a live object of this heap"
            ),
            Self::Unmarked { at } => {
                write!(f, "reachable object {at:#x} was not marked by the cycle")
            }
            Self::BadHeader { at } => write!(f, "object {at:#x} has no usable header"),
            Self::UnmarkedLine { at, line } => write!(
                f,
                "live object {at:#x} covers line {line} of its block, which is not marked"
            ),
            Self::Overlap { at, size, next } => write!(
                f,
                "live object {at:#x} of {size} bytes overlaps the next live object at {next:#x}"
            ),
            Self::DeadWeak {
                holder,
                slot,
                target,
            } => write!(
                f,
                "weak slot {slot:#x} of object {holder:#x} still names the dead object {target:#x}"
            ),
        }
    }
}

/// What a passing verification saw. Useful as a check in its own right:
/// a cycle that was supposed to keep N objects and kept a different
/// number is a bug the counts show.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Audit {
    /// Live objects reachable from the roots.
    pub objects: u64,
    /// Their total size in bytes.
    pub bytes: u64,
    /// Strong slots (including slot tails) examined.
    pub slots: u64,
    /// Weak slots examined.
    pub weak_slots: u64,
    /// Root slots examined, including the kept-alive and cleared lists.
    pub roots: u64,
    /// Slots naming a chunk this heap does not own. H-12 says such a slot
    /// is never followed, so it is not a violation; it is counted so that
    /// it is visible rather than silently skipped. Safe code cannot
    /// create one — the brands make a cross-heap store a type error — so
    /// a non-zero count means corruption or a test that planted one.
    pub foreign: u64,
    /// Whether the mark checks ran. They are meaningful only when no
    /// allocation has happened since the last cycle: allocation marks an
    /// object's lines but not its mark bit, so a freshly allocated live
    /// object is legitimately unmarked. Every other check always runs.
    pub marks_checked: bool,
}

/// The walk's state: the live objects found so far with their sizes, the
/// objects still to scan, the count of foreign slots, and whether the
/// mark checks apply.
struct Walk {
    live: BTreeMap<u64, u64>,
    work: Vec<u64>,
    foreign: u64,
    marks: bool,
}

/// Walks the heap from its roots and checks the collector's invariants.
///
/// # Errors
/// The first [`Violation`] found. The walk stops there: one lost object
/// usually makes every later check meaningless.
pub(crate) fn verify(st: &State) -> Result<Audit, Violation> {
    // Allocation marks lines but not mark bits, so between a cycle and
    // the next one an unmarked live object is not a bug. The automatic
    // call site is the end of a cycle, where this is always true.
    let marks = !st.allocated_since_cycle;
    let mut audit = Audit {
        marks_checked: marks,
        ..Audit::default()
    };
    let mut w = Walk {
        live: BTreeMap::new(),
        work: Vec::new(),
        foreign: 0,
        marks,
    };

    {
        // Roots, the agent's kept-alive list, and the objects whose weak
        // slots a cycle cleared and the embedder has not taken yet: the
        // same three sets the collector treats as roots.
        let roots = st.roots.borrow();
        let kept = st.kept.borrow();
        let cleared = st.cleared.borrow();
        for s in roots
            .iter()
            .flatten()
            .chain(kept.iter())
            .chain(cleared.iter())
        {
            audit.roots += 1;
            follow(st, 0, 0, *s, &mut w)?;
        }
    }

    while let Some(off) = w.work.pop() {
        let Some((layout, size)) = st.shape(off) else {
            return Err(Violation::BadHeader { at: off });
        };
        if let Err(line) = st.space.lines_marked(off, size) {
            return Err(Violation::UnmarkedLine { at: off, line });
        }
        for i in 0..layout.traced {
            let at = off + Layout::slot_offset(i);
            if at + 4 > off + size {
                continue;
            }
            audit.slots += 1;
            let s = Slot::from_raw(st.cage.load32(at));
            follow(st, off, at, s, &mut w)?;
        }
        // Weak slots are not strong edges, so they are checked without
        // being followed: a live target is reachable strongly from
        // somewhere else and will be walked there. What is checked is
        // H-10's promise, that a cycle leaves no weak slot naming a dead
        // object.
        for i in layout.traced..layout.traced.saturating_add(layout.weak) {
            let at = off + Layout::slot_offset(i);
            if at + 4 > off + size {
                continue;
            }
            audit.weak_slots += 1;
            let s = Slot::from_raw(st.cage.load32(at));
            if let Some(t) = s.as_reference()
                && st.space.owns(t)
                && !(st.space.is_object(t) && (!marks || st.space.is_marked(t)))
            {
                return Err(Violation::DeadWeak {
                    holder: off,
                    slot: at,
                    target: t,
                });
            }
        }
        if layout.tail == Tail::Slots {
            let mut at = off + layout.tail_offset();
            while at + 4 <= off + size {
                audit.slots += 1;
                let s = Slot::from_raw(st.cage.load32(at));
                follow(st, off, at, s, &mut w)?;
                at += 4;
            }
        }
    }

    // No two live objects may overlap. `live` is ordered by offset, so
    // one pass over it is enough.
    audit.foreign = w.foreign;
    let mut prev: Option<(u64, u64)> = None;
    for (&off, &size) in &w.live {
        if let Some((p, ps)) = prev
            && p + ps > off
        {
            return Err(Violation::Overlap {
                at: p,
                size: ps,
                next: off,
            });
        }
        prev = Some((off, size));
        audit.objects += 1;
        audit.bytes += size;
    }
    Ok(audit)
}

/// Checks the slot `s`, held at cage offset `slot` of object `holder`,
/// and queues its target if this is the first time it has been seen.
fn follow(st: &State, holder: u64, slot: u64, s: Slot, w: &mut Walk) -> Result<(), Violation> {
    let Some(target) = s.as_reference() else {
        return Ok(()); // a small integer, or an empty field
    };
    if !st.space.owns(target) {
        // H-12: a slot naming another agent heap's chunk, or cage memory
        // no heap holds, is never followed. The collector's contract is
        // to ignore it, not to reject it, so this is counted and skipped.
        w.foreign += 1;
        return Ok(());
    }
    if !st.space.is_object(target) {
        return Err(Violation::Dangling {
            holder,
            slot,
            target,
        });
    }
    if w.marks && !st.space.is_marked(target) {
        return Err(Violation::Unmarked { at: target });
    }
    let Some((_, size)) = st.shape(target) else {
        return Err(Violation::BadHeader { at: target });
    };
    if w.live.insert(target, size).is_none() {
        w.work.push(target);
    }
    Ok(())
}
