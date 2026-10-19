//! CS-D3 spike, V8-free core.
//!
//! - [`token`]: `ScriptToken`, the per-thread proof that no other code on this
//!   thread can run script or touch script-reachable state right now, and the
//!   `enter` / `lend` / `reenter` state machine that mints it.
//! - [`cell`]: `TokenCell<T>`, whose borrows are tied to the token.
//! - [`traced`]: `NoTraced`, the bound a GC object's `TokenCell` fields need
//!   (criterion 3).
//! - [`dom`]: a DOM-like model (nodes, `insertBefore`, a synchronous observer
//!   that re-enters "script") used for criterion (4) and the benchmark.
//!
//! The only `unsafe` is in `cell.rs` (three blocks) and the `NoTraced` impls
//! in `traced.rs`. Everything else is safe.
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod cell;
pub mod dom;
pub mod token;
pub mod traced;

pub use cell::{TokenCell, borrow_mut_2, borrow_mut_3};
pub use token::{Reentry, ScriptToken, enter, lend, reenter, try_reenter};
pub use traced::NoTraced;
