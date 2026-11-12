//! A root built by naming its index instead of taking one from a scope.
//! A forged index would name whatever the root list holds at that place.
//!
//! EXPECT: does not compile — the fields are private.

// EXPECT-ERROR: E0451
use hg_heap::{Any, Root};

pub fn case() {
    let _r: Root<'static, 'static, Any> = Root {
        level: todo!(),
        index: todo!(),
        _h: todo!(),
        _s: todo!(),
        _t: todo!(),
    };
}
