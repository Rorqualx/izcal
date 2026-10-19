//! CS-D3 spike, V8 half.
//!
//! - [`hg_script`]: the only code allowed to call rusty_v8 methods that can
//!   run script; each wrapper takes `&mut ScriptToken` and lends it.
//! - [`event_loop`]: the only way to mint a thread's outermost token.
//! - [`member_cell`]: criterion (3), traced links outside TokenCell.
//! - [`gc`]: `gc_object!`, the generator check for criterion (3).
//! - `holes` (feature `holes`): why the scope itself (or the isolate, as
//!   upstream `GcCell` does) cannot be the token. Deliberately unsound; never
//!   copy it into an Izcal crate.
pub use token_cell;
pub use v8;

pub mod event_loop;
pub mod gc;
pub mod hg_script;
#[cfg(feature = "holes")]
pub mod holes;
pub mod member_cell;

#[cfg(feature = "misuse")]
pub mod misuse;
