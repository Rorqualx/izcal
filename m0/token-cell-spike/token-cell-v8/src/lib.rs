//! CS-D3 spike, V8 half.
//!
//! - [`hg_script`]: the only code allowed to call rusty_v8 methods that can
//!   run script; each wrapper takes `&mut ScriptToken` and lends it.
//! - [`member_cell`]: criterion (3) sketch, traced links outside TokenCell.
//! - [`holes`]: why the scope itself (or the isolate, as upstream `GcCell`
//!   does) cannot be the token.
pub use token_cell;

pub mod hg_script;
pub mod holes;
pub mod member_cell;

#[cfg(feature = "misuse")]
pub mod misuse;
