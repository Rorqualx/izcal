//! A reference built from a slot value the caller chose. Under H-2 the
//! heap already treats everything in the cage as untrusted, but a `Gc`
//! minted outside would carry a brand it was never given.
//!
//! EXPECT: does not compile — the fields are private.

// EXPECT-ERROR: E0451
use hg_heap::{Any, Gc};

pub fn case() {
    let _g: Gc<'static, 'static, Any> = Gc {
        slot: todo!(),
        _borrow: todo!(),
        _t: todo!(),
    };
}
