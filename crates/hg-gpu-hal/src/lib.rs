//! `hg-gpu-hal`: the contract, before any back end.
//!
//! §6.6 gives this crate three native APIs and only three — **Metal**,
//! **Direct3D 12** and **Vulkan** — and five properties they must all
//! present the same way:
//!
//! > devices advertise capabilities, never vendors; all work is
//! > asynchronous and returns fences on a per-device timeline; there
//! > is no blocking map; validation errors are sticky; device loss
//! > resolves exactly once with an epoch.
//!
//! GR-M0's row asks for the **design review across three back ends**
//! — "resource type states, render graph, allocator, fences, epochs"
//! — and not for the back ends. So what is here is the contract as
//! types, with the invariants that a back end cannot keep by
//! accident, and `tests/it` is where each one is held to.
//!
//! # Why there is no `Vendor`
//!
//! A renderer that asks "is this an Intel GPU" has to be right about
//! every Intel GPU ever made, including the ones that do not exist
//! yet, and the question it actually wanted to ask was about a
//! capability. §6.6's rule — **capabilities, never vendors** — is a
//! type here rather than a convention: [`Capabilities`] has no field
//! naming a manufacturer, and nothing in this crate can be asked for
//! one. A driver workaround is then a capability the back end
//! reports as absent, which is a thing the rest of the engine already
//! knows how to handle.
//!
//! # Why there is no blocking map
//!
//! A `map` that blocks until the GPU is finished is a frame stalled
//! on a device that may never answer. Every read-back here returns a
//! [`Fence`] and the caller checks it, so a hostile or wedged device
//! costs its own work and nobody else's — which is G72's rule for
//! the compositor stated at the layer below it.
//!
//! # Why device loss has an epoch
//!
//! A lost device is discovered by several callers at once, and each
//! of them wants to rebuild. Without an epoch they each rebuild, and
//! the rebuilds race. [`DeviceLoss::resolve`] answers "yes, you are
//! the one" exactly once per epoch, and everybody else is told to
//! wait for the new device — which is the shape §6.6 asks for and
//! the one a test can hold.

pub mod device;
pub mod fence;
pub mod graph;
pub mod resource;

pub use device::{Capabilities, DeviceLoss, Epoch, Validation, ValidationError};
pub use fence::{Fence, Timeline};
pub use graph::{Access, Barrier, Pass, RenderGraph};
pub use resource::{Format, ResourceId, ResourceKind, State, Usage};
