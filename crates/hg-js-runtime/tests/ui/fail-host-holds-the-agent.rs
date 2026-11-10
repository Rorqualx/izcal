//! CS-D3 criterion 4, the heart of it: a host function that tries to run
//! script again while a mutation is in progress.
//!
//! At JS-M0 this is not refused by a token's uniqueness but by the shape
//! of the boundary. `HostFunctions::call` is handed only `HostValue`s —
//! owned, heap-free copies — and the agent owns its host as a
//! `Box<dyn HostFunctions>`, which is `+ 'static`. So a host cannot hold
//! the agent or anything else branded with the heap, and without a
//! `&mut Mutator<'h>` there is no way to run a job. Re-entry is not
//! guarded; it is unrepresentable.
//!
//! EXPECT: does not compile — a host cannot hold anything branded `'h`.

// EXPECT-ERROR: LIFETIME
// EXPECT-NOTE: requires that `'h` must outlive `'static`
use hg_js_ops::support::HostValue;
use hg_js_runtime::{Agent, HostFunctions};

/// A host that keeps the agent so it can call back into it.
pub struct Reentrant<'h, 'o> {
    agent: Agent<'h, 'o>,
}

impl HostFunctions for Reentrant<'_, '_> {
    fn call(
        &mut self,
        _id: i32,
        _this: &HostValue,
        _args: &[HostValue],
    ) -> Result<HostValue, HostValue> {
        // There is no mutator here to run a job with, and no way to get
        // one: that is the point. This case fails earlier, at the box.
        Ok(HostValue::Undefined)
    }
}

pub fn case<'h, 'o>(agent: Agent<'h, 'o>) -> Box<dyn HostFunctions> {
    Box::new(Reentrant { agent })
}
