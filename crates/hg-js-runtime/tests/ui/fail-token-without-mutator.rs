//! The script token is not a licence of its own, and nor is the agent: a
//! job is run with the GC token too. `Agent::run_job` takes
//! `&mut Mutator<'h>` first, so there is no path from "I have an agent"
//! to "script ran" that does not also name the point where a cycle may
//! happen (CS-TY-10).
//!
//! Every generated runtime method that declares `call_user` takes both
//! tokens as well, which is what `tools/token-audit.py` checks across
//! all 28 declarations.
//!
//! EXPECT: does not compile — the call wants the mutator too.

// EXPECT-ERROR: E0061
use hg_js_bytecode::UnitView;
use hg_js_ops::support::Host;
use hg_js_runtime::Agent;

pub fn case<'h, 'o>(agent: &mut Agent<'h, 'o>, host: &mut dyn Host<'h>, unit: &UnitView<'_>) {
    let _ = agent.run_job(host, unit, None, &[]);
}
