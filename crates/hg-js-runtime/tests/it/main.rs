//! The JS-M0 interpreter's tests: every opcode run by assembled programs
//! with results derived from ECMA-262, the probe's two jobs with WeakRef
//! and FinalizationRegistry across ClearKeptObjects, and the moving
//! collector's stress modes giving the same results and heap digest.

#![forbid(unsafe_code)]

mod harness;
mod opcodes;
mod weak;
