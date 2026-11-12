//! Positive control: an ordinary host function compiles, so a refusal in
//! the cases beside this one is the engine's boundary rejecting something
//! and not the harness being broken.
//!
//! EXPECT: compiles.
use hg_js_ops::support::HostValue;
use hg_js_runtime::HostFunctions;

/// A host function that counts its calls, which is all a JS-M0 host
/// function can do with what it is given.
pub struct Counter {
    calls: u32,
}

impl HostFunctions for Counter {
    fn call(
        &mut self,
        _id: i32,
        _this: &HostValue,
        args: &[HostValue],
    ) -> Result<HostValue, HostValue> {
        self.calls += 1;
        Ok(HostValue::Number(f64::from(args.len() as u32)))
    }
}

pub fn case() -> Box<dyn HostFunctions> {
    Box::new(Counter { calls: 0 })
}
