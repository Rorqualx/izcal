//! The record types of the generated operation facts (language note
//! Section 7), which T2, the debugger and the T0 generator read.

/// One heap category of the tree (R0-O12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CategoryFacts {
    /// The category's name.
    pub name: &'static str,
    /// Its parent, `None` for `Heap`.
    pub parent: Option<&'static str>,
    /// Whether a GC point changes it (`weak`).
    pub gc_mutable: bool,
}

/// A set of effects (language note 5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectFacts {
    /// `alloc(u)` formulas.
    pub alloc: &'static [&'static str],
    /// Whether it allocates internally (no canonical charge).
    pub alloc_internal: bool,
    /// Categories read.
    pub reads: &'static [&'static str],
    /// Categories written.
    pub writes: &'static [&'static str],
    /// Error kinds thrown (`any` for a user value).
    pub throws: &'static [&'static str],
    /// Whether it may run user code.
    pub call_user: bool,
    /// Host hooks reached.
    pub host: &'static [&'static str],
    /// B1 inputs read.
    pub input: &'static [&'static str],
    /// Whether it checks for interrupts.
    pub interrupt: bool,
    /// Whether it has a GC point.
    pub gc: bool,
}

/// One path of an item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathFacts {
    /// The path's name.
    pub name: &'static str,
    /// The enclosing path, as an index into the item's paths.
    pub parent: Option<usize>,
    /// The guard, as written.
    pub guard: &'static str,
    /// The feedback the path speculates on.
    pub speculate: Option<&'static str>,
    /// The path's own effects.
    pub effects: EffectFacts,
    /// Its GC points and the frame values live across each.
    pub gc_points: &'static [(&'static str, &'static str)],
    /// The units it charges.
    pub units: &'static [&'static str],
    /// Whether it is pure.
    pub pure: bool,
    /// Whether it folds on constant inputs.
    pub foldable: bool,
    /// The runtime functions and helpers it calls.
    pub calls: &'static [&'static str],
    /// Its anchor.
    pub anchor: &'static str,
}

/// One operation, intrinsic or helper.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemFacts {
    /// `op`, `intrinsic` or `helper`.
    pub kind: &'static str,
    /// Its name.
    pub name: &'static str,
    /// Its opcode, for an operation.
    pub opcode: Option<u8>,
    /// Its operands or parameters, as written.
    pub operands: &'static [&'static str],
    /// Its accumulator mode.
    pub acc: &'static str,
    /// Its after-call mode (OC-17).
    pub after_call: &'static str,
    /// Its terminals.
    pub control: &'static [&'static str],
    /// The declared effects.
    pub declared: EffectFacts,
    /// The inferred effects (the union over its paths).
    pub effects: EffectFacts,
    /// Whether it ends a basic block.
    pub terminator: bool,
    /// The debugger's side-effect class.
    pub debugger: &'static str,
    /// The facts it invalidates.
    pub invalidates: &'static [&'static str],
    /// The facts its result carries.
    pub results: &'static [&'static str],
    /// Its transfer function, if declared.
    pub transfer: Option<&'static str>,
    /// Its SMT encoding, if declared.
    pub smt: Option<&'static str>,
    /// Its frame charge, if declared.
    pub frame_charge: Option<&'static str>,
    /// Its paths.
    pub paths: &'static [PathFacts],
}
