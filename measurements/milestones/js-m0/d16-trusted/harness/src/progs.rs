//! The programs, written in the bytecode note's text assembler. JS-M0 has
//! no parser and no corpus (JS-M1 brings both), and no JS-to-JS call
//! opcode at all: `CallIntrinsic` is the only call, so the call cost of
//! JS-D16 is measured on the unit-entry path (`entry.rs`) and not on a
//! call instruction.

use hg_js_bytecode::{AsmError, Unit, assemble};

/// The arithmetic loop: ten instructions an iteration, every operand kind
/// a tier re-checks except constants — registers, an immediate, a
/// comparison, binary and unary feedback slots, a forward branch and a
/// backward loop edge. `n` iterations sum to `n(n+1)/2`, which the gate
/// checks against what the real interpreter returns.
pub fn arith(n: u32) -> Result<Unit, AsmError> {
    one(&format!(
        ".function arith params=0 registers=4
.slot s0 compare
.slot s1 binary
.slot s2 unary
.slot s3 loop
        LoadInt #0
        StoreReg r0
        LoadInt #1
        StoreReg r1
top:    LoadInt #{n}
        Le r1, s0
        BranchFalse @done
        LoadReg r1
        Add r0, s1
        StoreReg r0
        LoadReg r1
        Inc s2
        StoreReg r1
        LoopBack @top, s3
done:   LoadReg r0
        Return
.end
"
    ))
}

/// Constant-heavy: every instruction carries a constant index, which is
/// the operand whose re-check costs (b) the most (a count, a section
/// bound and a kind byte, all re-read).
pub fn consts(count: u32) -> Result<Unit, AsmError> {
    let mut text = String::from(".function consts params=0 registers=4\n.slot s0 binary\n");
    for i in 0..count {
        text.push_str(&format!(".const k{i} number {}\n", i as f64 + 0.5));
    }
    text.push_str("top:    LoadConst k0\n        StoreReg r0\n");
    for i in 0..count {
        text.push_str(&format!("        LoadConst k{i}\n        Add r0, s0\n"));
    }
    text.push_str("        StoreReg r0\n        LoadReg r0\n        Return\n.end\n");
    one(&text)
}

/// Property access: the shape the JS-D16 row names beside calls ("one
/// table indirection on calls and IC access") — a key constant and an IC
/// slot on every instruction.
pub fn props() -> Result<Unit, AsmError> {
    one(".function props params=0 registers=4
.const k0 key \"alpha\"
.const k1 key \"beta\"
.slot s0 load
.slot s1 store
.slot s2 load
.slot s3 store
.slot s4 loop
        LoadNull
        StoreReg r0
        CallIntrinsic %ObjectCreate, r0:1
        StoreReg r1
top:    GetNamed r1, k0, s0
        SetNamed r1, k1, s1
        GetNamed r1, k1, s2
        SetNamed r1, k0, s3
        LoopBack @top, s4
.end
")
}

/// Branch-heavy: forward and backward targets, re-checked against a code
/// length that (b) must re-read from the cage.
pub fn branchy() -> Result<Unit, AsmError> {
    one(".function branchy params=0 registers=4
.slot s0 loop
.slot s1 unary
        LoadInt #0
        StoreReg r0
top:    LoadReg r0
        BranchTruthy @mid
        LoadInt #1
        Jump @join
mid:    LoadInt #0
join:   StoreReg r0
        LoadReg r0
        Not
        BranchFalsy @back
        LoadInt #3
        StoreReg r1
back:   LoopBack @top, s0
.end
")
}

/// A short unit, for the per-entry measurement: the smallest body that
/// still returns a checkable value.
pub fn tiny(consts: u32) -> Result<Unit, AsmError> {
    let mut text = String::from(".function tiny params=0 registers=2\n");
    for i in 0..consts {
        text.push_str(&format!(".const k{i} number {}\n", i as f64 + 0.5));
    }
    text.push_str(
        "        LoadInt #7\n        StoreReg r0\n        LoadReg r0\n        Return\n.end\n",
    );
    one(&text)
}

fn one(text: &str) -> Result<Unit, AsmError> {
    let mut units = assemble(text)?;
    Ok(units.remove(0))
}
