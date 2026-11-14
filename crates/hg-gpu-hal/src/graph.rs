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
//! The graph is also where three errors are caught before any command
//! reaches a driver: a read of something nothing writes, a cycle, and
//! a resource from another device or another epoch.

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
    /// the graph is a *sequence*, so a cycle is not expressible.
    /// [`ValidationError::Cycle`] exists for the day the graph gains
    /// explicit edges, and the test says so rather than leaving a
    /// variant nothing produces.
    #[must_use]
    pub fn compile(&self) -> (Vec<Barrier>, Validation) {
        let mut validation = Validation::new();
        let mut barriers = Vec::new();
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
}
