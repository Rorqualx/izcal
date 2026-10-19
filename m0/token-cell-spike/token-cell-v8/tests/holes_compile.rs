//! Why neither the scope nor the isolate can be the token: the hole functions
//! in `src/holes.rs` type-check in safe Rust. This test only names them, so a
//! build with `--features holes` proves they compile; it never calls them,
//! because the call would be undefined behaviour (see `src/holes.rs`).
#![cfg(feature = "holes")]

use token_cell_v8::holes;

#[test]
fn scope_as_token_and_upstream_gc_cell_compile() {
    let fns = [
        holes::scope_cell_borrow_across_call as usize,
        holes::scope_cell_callback as usize,
        holes::gc_cell_borrow_across_call as usize,
        holes::gc_cell_callback as usize,
    ];
    assert!(fns.iter().all(|&p| p != 0));
}
