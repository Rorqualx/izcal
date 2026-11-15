//! The render graph: passes, what they touch, and the barriers that
//! follow from it.
//!
//! GR-M0's review item. The graph exists so that **barriers are
//! derived rather than written**: a pass declares what it reads and
//! writes, and the transitions between passes are computed. Hand-
//! written barriers are the GPU equivalent of hand-written locks —
//! they are right on the machine they were tested on and wrong on the
//! one with a different queue layout.
//!
//! The graph is also where four errors are caught before any command
//! reaches a driver: a read of something nothing writes, a resource
//! from another device or another epoch, an edge that points forwards,
//! and two passes that share a resource with nothing ordering them.
//!
//! # Why the edges point backwards (GR-M0's §3.5 decision)
//!
//! A pass declares the passes it runs **after**, by index, and every
//! such index must be smaller than the pass's own. Two consequences,
//! and the second is the point:
//!
//! - **A cycle is unexpressible**, not merely detected. Any cycle
//!   needs an edge that points forwards or at itself, and those are
//!   refused one at a time without looking at the graph at all. This
//!   is why there is no `Cycle` variant: the graph used to be a plain
//!   sequence, which also made cycles unexpressible, and the variant
//!   sat unreachable for exactly that reason. Gaining edges did not
//!   bring the cycle back, so the variant went instead.
//! - **Independence became expressible, and so did a new mistake.** A
//!   sequence orders every pass against every other, so it can never
//!   tell you that you ordered two passes you did not mean to order.
//!   With edges, two passes can be genuinely unordered — and two
//!   unordered passes touching one resource, at least one writing,
//!   is a result that depends on how a driver schedules. That is
//!   [`ValidationError::UnorderedConflict`], and it is the check the
//!   sequence could not have.
//!
//! The barrier walk is unchanged, and the edges are what make it
//! *correct* rather than merely plausible. Because edges point
//! backwards, the submission order is always a topological order; and
//! because every conflicting pair must be ordered by an edge path,
//! the submission order agrees with the dependency order wherever the
//! agreement matters. Before, walking the sequence was right by
//! assumption.

use crate::device::{Validation, ValidationError};
use crate::resource::{ResourceId, State, Usage};

/// One resource a pass touches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Access {
    /// Which resource.
    pub resource: ResourceId,
    /// How the pass uses it.
    pub usage: Usage,
}

/// One pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pass {
    /// A name, for the decision log (GR-D24: every placement and
    /// every fallback is a logged, explainable event).
    pub name: &'static str,
    /// What it touches.
    pub accesses: Vec<Access>,
    /// The passes this one runs after, by index into the graph.
    ///
    /// Every index must be **smaller** than this pass's own, which is
    /// what keeps a cycle unexpressible. Empty means the pass depends
    /// on nothing and may run as early as the device allows.
    pub after: Vec<usize>,
}

impl Pass {
    /// A pass that depends on nothing.
    #[must_use]
    pub const fn new(name: &'static str, accesses: Vec<Access>) -> Self {
        Self {
            name,
            accesses,
            after: Vec::new(),
        }
    }

    /// The same pass, running after `earlier`.
    #[must_use]
    pub fn after(mut self, earlier: &[usize]) -> Self {
        self.after = earlier.to_vec();
        self
    }
}

/// A transition the back end must emit before a pass runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Barrier {
    /// Which resource.
    pub resource: ResourceId,
    /// The state it is in.
    pub from: State,
    /// The state the next pass needs.
    pub to: State,
    /// Which pass needs it, by index.
    pub before: usize,
}

/// A graph of passes for one device and epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGraph {
    device: u32,
    epoch: u64,
    passes: Vec<Pass>,
}

impl RenderGraph {
    /// An empty graph for one device and epoch.
    #[must_use]
    pub const fn new(device: u32, epoch: u64) -> Self {
        Self {
            device,
            epoch,
            passes: Vec::new(),
        }
    }

    /// Adds a pass. Order is submission order.
    pub fn add(&mut self, pass: Pass) -> &mut Self {
        self.passes.push(pass);
        self
    }

    /// The passes, in order.
    #[must_use]
    pub fn passes(&self) -> &[Pass] {
        &self.passes
    }

    /// The barriers the graph implies, and the errors it contains.
    ///
    /// Walks the passes in order, tracking each resource's state. A
    /// transition that needs a barrier produces one; a read of a
    /// resource nothing has written is a
    /// [`ValidationError::UnwrittenRead`]; a resource from another
    /// device or epoch is a [`ValidationError::ForeignResource`].
    ///
    /// There is no cycle check here and there does not need to be:
    /// edges point backwards, so a cycle is not expressible. An edge
    /// that points forwards is a [`ValidationError::ForwardEdge`],
    /// refused without looking at the rest of the graph; and two
    /// passes sharing a resource with nothing ordering them are a
    /// [`ValidationError::UnorderedConflict`].
    #[must_use]
    pub fn compile(&self) -> (Vec<Barrier>, Validation) {
        let mut validation = Validation::new();
        let mut barriers = Vec::new();
        let reach = self.check_edges(&mut validation);
        self.check_conflicts(&reach, &mut validation);
        // The state of each resource, in the order first seen.
        let mut states: Vec<(ResourceId, State)> = Vec::new();

        for (index, pass) in self.passes.iter().enumerate() {
            for access in &pass.accesses {
                if access.resource.device != self.device || access.resource.epoch != self.epoch {
                    validation.record(ValidationError::ForeignResource);
                    continue;
                }
                let needed = State::for_usage(access.usage);
                let found = states.iter_mut().find(|(r, _)| *r == access.resource);
                match found {
                    Some((_, current)) => {
                        if current.needs_barrier_to(needed) {
                            barriers.push(Barrier {
                                resource: access.resource,
                                from: *current,
                                to: needed,
                                before: index,
                            });
                        }
                        *current = needed;
                    }
                    None => {
                        // First use. Reading something nothing has
                        // written is reading whatever the driver left
                        // there, which is a different image on every
                        // machine and sometimes another page's.
                        if needed.is_read_only() {
                            validation.record(ValidationError::UnwrittenRead);
                        }
                        barriers.push(Barrier {
                            resource: access.resource,
                            from: State::Undefined,
                            to: needed,
                            before: index,
                        });
                        states.push((access.resource, needed));
                    }
                }
            }
        }
        (barriers, validation)
    }

    /// Checks that every edge points backwards, and returns
    /// reachability: `reach[i][j]` is true when pass `j` must finish
    /// before pass `i` starts.
    ///
    /// One forward pass suffices precisely because edges point
    /// backwards: when `i` is reached, every pass it depends on has
    /// its own row already complete, so the transitive closure is a
    /// union of finished rows. A general digraph would need a
    /// traversal with a visited set and a cycle guard; this needs
    /// neither, which is the representation paying for itself.
    fn check_edges(&self, validation: &mut Validation) -> Vec<Vec<bool>> {
        let n = self.passes.len();
        let mut reach: Vec<Vec<bool>> = vec![vec![false; n]; n];
        for (i, pass) in self.passes.iter().enumerate() {
            let mut row = vec![false; n];
            for &j in &pass.after {
                if j >= i {
                    validation.record(ValidationError::ForwardEdge);
                    continue;
                }
                // `j` itself, and everything `j` already waits for.
                if let Some(cell) = row.get_mut(j) {
                    *cell = true;
                }
                let waits_for = reach.get(j).cloned().unwrap_or_default();
                for (k, from_j) in waits_for.iter().enumerate() {
                    if let (true, Some(cell)) = (*from_j, row.get_mut(k)) {
                        *cell = true;
                    }
                }
            }
            if let Some(slot) = reach.get_mut(i) {
                *slot = row;
            }
        }
        reach
    }

    /// Records an [`UnorderedConflict`] for each pair of passes that
    /// touch one resource with at least one of them writing and
    /// neither ordered against the other.
    ///
    /// [`UnorderedConflict`]: ValidationError::UnorderedConflict
    fn check_conflicts(&self, reach: &[Vec<bool>], validation: &mut Validation) {
        for (i, pass) in self.passes.iter().enumerate() {
            for (j, earlier) in self.passes.iter().enumerate().take(i) {
                let ordered = reach
                    .get(i)
                    .and_then(|row| row.get(j))
                    .copied()
                    .unwrap_or(false);
                if ordered {
                    continue;
                }
                if Self::conflict(pass, earlier) {
                    validation.record(ValidationError::UnorderedConflict);
                }
            }
        }
    }

    /// Whether two passes touch one resource with at least one write.
    ///
    /// Two reads of the same resource are no conflict: neither
    /// changes it, so the order between them cannot change the
    /// result, and requiring an edge there would force every pass
    /// that samples a texture into a chain.
    fn conflict(a: &Pass, b: &Pass) -> bool {
        a.accesses.iter().any(|x| {
            b.accesses.iter().any(|y| {
                x.resource == y.resource
                    && !(State::for_usage(x.usage).is_read_only()
                        && State::for_usage(y.usage).is_read_only())
            })
        })
    }
}
