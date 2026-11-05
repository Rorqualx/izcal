//! The checked model: types (language note Section 4), effect sets (5.1),
//! the built-in tables the descriptions are checked against (kinds, host
//! hooks, error kinds, operand and slot kinds, feedback predicates), and
//! the resolved declarations.

use std::collections::BTreeSet;
use std::fmt;

use crate::ast::{AccMode, AfterCall, Anchor, CallKind, InputKind};
use crate::diag::Span;
use crate::tir::{Body, TExpr, TPat};

/// A reference kind (language note Section 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// A heap number.
    HeapNumber,
    /// A string (flat or rope).
    String,
    /// A BigInt.
    BigInt,
    /// A Symbol.
    Symbol,
    /// Any object.
    Object,
    /// A dictionary object.
    Dict,
    /// A WeakRef.
    WeakRef,
    /// A FinalizationRegistry.
    FinReg,
    /// A host function object.
    HostFn,
}

impl Kind {
    /// Every kind.
    pub const ALL: [Self; 9] = [
        Self::HeapNumber,
        Self::String,
        Self::BigInt,
        Self::Symbol,
        Self::Object,
        Self::Dict,
        Self::WeakRef,
        Self::FinReg,
        Self::HostFn,
    ];

    /// The kind's name as written.
    pub const fn name(self) -> &'static str {
        match self {
            Self::HeapNumber => "HeapNumber",
            Self::String => "String",
            Self::BigInt => "BigInt",
            Self::Symbol => "Symbol",
            Self::Object => "Object",
            Self::Dict => "Dict",
            Self::WeakRef => "WeakRef",
            Self::FinReg => "FinReg",
            Self::HostFn => "HostFn",
        }
    }

    /// The kind named `s`.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.name() == s)
    }

    /// Whether this is an object kind (a refinement of `Object`).
    pub const fn is_object_kind(self) -> bool {
        matches!(
            self,
            Self::Dict | Self::WeakRef | Self::FinReg | Self::HostFn
        )
    }

    /// Whether a `Ref<self>` is also a `Ref<of>`.
    pub fn within(self, of: Self) -> bool {
        self == of || (of == Self::Object && self.is_object_kind())
    }

    /// The value atoms this kind covers (C2).
    pub fn atoms(self) -> Vec<Atom> {
        match self {
            Self::HeapNumber => vec![Atom::HeapNumber],
            Self::String => vec![Atom::String],
            Self::BigInt => vec![Atom::BigInt],
            Self::Symbol => vec![Atom::Symbol],
            Self::Object => vec![Atom::Dict, Atom::WeakRef, Atom::FinReg, Atom::HostFn],
            Self::Dict => vec![Atom::Dict],
            Self::WeakRef => vec![Atom::WeakRef],
            Self::FinReg => vec![Atom::FinReg],
            Self::HostFn => vec![Atom::HostFn],
        }
    }

    /// The Rust marker type in `hg_js_ops::kind`.
    pub const fn rust(self) -> &'static str {
        match self {
            Self::HeapNumber => "kind::HeapNumber",
            Self::String => "kind::String",
            Self::BigInt => "kind::BigInt",
            Self::Symbol => "kind::Symbol",
            Self::Object => "kind::Object",
            Self::Dict => "kind::Dict",
            Self::WeakRef => "kind::WeakRef",
            Self::FinReg => "kind::FinReg",
            Self::HostFn => "kind::HostFn",
        }
    }
}

/// One value class, for exhaustiveness (C2): the classes the hand-written
/// classifier distinguishes, plus `Empty` for weak slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Atom {
    /// undefined
    Undefined,
    /// null
    Null,
    /// true
    True,
    /// false
    False,
    /// A small integer.
    Smi,
    /// A heap number.
    HeapNumber,
    /// A string.
    String,
    /// A BigInt.
    BigInt,
    /// A Symbol.
    Symbol,
    /// A dictionary object.
    Dict,
    /// A WeakRef.
    WeakRef,
    /// A FinalizationRegistry.
    FinReg,
    /// A host function.
    HostFn,
    /// A cleared weak slot.
    Empty,
    /// A weak slot holding a value.
    Present,
}

impl Atom {
    /// The atoms of a JS value.
    pub const VALUE: [Self; 13] = [
        Self::Undefined,
        Self::Null,
        Self::True,
        Self::False,
        Self::Smi,
        Self::HeapNumber,
        Self::String,
        Self::BigInt,
        Self::Symbol,
        Self::Dict,
        Self::WeakRef,
        Self::FinReg,
        Self::HostFn,
    ];

    /// The atoms of a weak slot.
    pub const WEAK: [Self; 2] = [Self::Empty, Self::Present];
}

/// An oddball value (a per-heap singleton object).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Oddball {
    /// undefined
    Undefined,
    /// null
    Null,
    /// true
    True,
    /// false
    False,
}

impl Oddball {
    /// The oddball named `s`.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "Undefined" => Self::Undefined,
            "Null" => Self::Null,
            "True" => Self::True,
            "False" => Self::False,
            _ => return None,
        })
    }

    /// The name as written.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Undefined => "Undefined",
            Self::Null => "Null",
            Self::True => "True",
            Self::False => "False",
        }
    }

    /// Its atom.
    pub const fn atom(self) -> Atom {
        match self {
            Self::Undefined => Atom::Undefined,
            Self::Null => Atom::Null,
            Self::True => Atom::True,
            Self::False => Atom::False,
        }
    }
}

/// A type (language note Section 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ty {
    /// Any JS value.
    Value,
    /// A reference of a kind.
    Ref(Kind),
    /// An interned property key.
    Key,
    /// `Empty | Value`.
    Weak,
    /// i32
    I32,
    /// u32
    U32,
    /// i64
    I64,
    /// f64
    F64,
    /// bool
    Bool,
    /// The kind of an object (`Dict`, `WeakRef`, `FinReg`, `HostFn`).
    ObjectKind,
    /// No value.
    Unit,
    /// Does not complete normally (a terminal).
    Never,
}

impl Ty {
    /// Whether values of this type are heap values (they matter for C6).
    pub const fn is_heap(self) -> bool {
        matches!(self, Self::Value | Self::Ref(_) | Self::Key | Self::Weak)
    }

    /// Whether this is an integer type.
    pub const fn is_int(self) -> bool {
        matches!(self, Self::I32 | Self::U32 | Self::I64)
    }

    /// Whether this is a numeric scalar.
    pub const fn is_numeric(self) -> bool {
        self.is_int() || matches!(self, Self::F64)
    }

    /// Whether a value of `self` may be used where `want` is expected
    /// (the only implicit conversions are reference upcasts).
    pub fn fits(self, want: Self) -> bool {
        if self == want || self == Self::Never {
            return true;
        }
        match (self, want) {
            (Self::Ref(k), Self::Ref(of)) => k.within(of),
            (Self::Ref(_) | Self::Key, Self::Value) => true,
            _ => false,
        }
    }

    /// The integer range of the type.
    pub const fn range(self) -> Option<(i128, i128)> {
        match self {
            Self::I32 => Some((i32::MIN as i128, i32::MAX as i128)),
            Self::U32 => Some((0, u32::MAX as i128)),
            Self::I64 => Some((i64::MIN as i128, i64::MAX as i128)),
            _ => None,
        }
    }

    /// The value atoms a scrutinee of this type is matched over (C2).
    pub fn atoms(self) -> Option<Vec<Atom>> {
        match self {
            Self::Value | Self::Ref(_) | Self::Key => Some(Atom::VALUE.to_vec()),
            Self::Weak => Some(Atom::WEAK.to_vec()),
            _ => None,
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value => write!(f, "Value"),
            Self::Ref(k) => write!(f, "Ref<{}>", k.name()),
            Self::Key => write!(f, "Key"),
            Self::Weak => write!(f, "Empty | Value"),
            Self::I32 => write!(f, "i32"),
            Self::U32 => write!(f, "u32"),
            Self::I64 => write!(f, "i64"),
            Self::F64 => write!(f, "f64"),
            Self::Bool => write!(f, "bool"),
            Self::ObjectKind => write!(f, "ObjectKind"),
            Self::Unit => write!(f, "()"),
            Self::Never => write!(f, "!"),
        }
    }
}

/// An error kind a description may throw (`any` is a user value).
pub const ERROR_KINDS: [&str; 7] = [
    "TypeError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "EvalError",
    "URIError",
    "any",
];

/// A host hook's context (hook table R0-H4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookContext {
    /// No heap access.
    None,
    /// A shared borrow of the mutator: read the heap, no GC point.
    Read,
    /// May allocate (a GC point), no script.
    Alloc,
    /// May allocate and run script.
    Script,
}

/// The audited hook list (R0-H1): name, anchor, context, the Rust method
/// of `hg_js_ops::Host` for the hooks the JS-M0 core set reaches.
pub const HOOKS: [(&str, &str, HookContext, Option<&str>); 18] = [
    (
        "HostEnsureCanAddPrivateElement",
        "sec-hostensurecanaddprivateelement",
        HookContext::None,
        None,
    ),
    (
        "HostMakeJobCallback",
        "sec-hostmakejobcallback",
        HookContext::Read,
        Some("make_job_callback"),
    ),
    (
        "HostCallJobCallback",
        "sec-hostcalljobcallback",
        HookContext::Script,
        None,
    ),
    (
        "HostEnqueueGenericJob",
        "sec-hostenqueuegenericjob",
        HookContext::None,
        None,
    ),
    (
        "HostEnqueuePromiseJob",
        "sec-hostenqueuepromisejob",
        HookContext::None,
        None,
    ),
    (
        "HostEnqueueTimeoutJob",
        "sec-hostenqueuetimeoutjob",
        HookContext::None,
        None,
    ),
    (
        "HostEnqueueFinalizationRegistryCleanupJob",
        "sec-host-cleanup-finalization-registry",
        HookContext::None,
        Some("enqueue_cleanup_job"),
    ),
    (
        "HostGetImportMetaProperties",
        "sec-hostgetimportmetaproperties",
        HookContext::Alloc,
        None,
    ),
    (
        "HostFinalizeImportMeta",
        "sec-hostfinalizeimportmeta",
        HookContext::Alloc,
        None,
    ),
    (
        "HostLoadImportedModule",
        "sec-HostLoadImportedModule",
        HookContext::Alloc,
        None,
    ),
    (
        "HostGetSupportedImportAttributes",
        "sec-hostgetsupportedimportattributes",
        HookContext::None,
        None,
    ),
    (
        "HostEnsureCanCompileStrings",
        "sec-hostensurecancompilestrings",
        HookContext::Script,
        None,
    ),
    (
        "HostGetCodeForEval",
        "sec-hostgetcodeforeval",
        HookContext::Alloc,
        None,
    ),
    (
        "HostHasSourceTextAvailable",
        "sec-hosthassourcetextavailable",
        HookContext::None,
        None,
    ),
    (
        "HostResizeArrayBuffer",
        "sec-hostresizearraybuffer",
        HookContext::None,
        None,
    ),
    (
        "HostGrowSharedArrayBuffer",
        "sec-hostgrowsharedarraybuffer",
        HookContext::None,
        None,
    ),
    (
        "HostPromiseRejectionTracker",
        "sec-host-promise-rejection-tracker",
        HookContext::Read,
        None,
    ),
    (
        "HostSystemUTCEpochNanoseconds",
        "sec-hostsystemutcepochnanoseconds",
        HookContext::None,
        Some("system_utc_epoch_nanoseconds"),
    ),
];

/// The hook that reads the clock (C11).
pub const CLOCK_HOOK: &str = "HostSystemUTCEpochNanoseconds";

/// Constant kinds (bytecode note 4.5).
pub const CONST_KINDS: [&str; 3] = ["number", "string", "key"];

/// IC slot kinds (bytecode note 4.6).
pub const SLOT_KINDS: [&str; 8] = [
    "load",
    "store",
    "keyed-load",
    "keyed-store",
    "binary",
    "unary",
    "compare",
    "loop",
];

/// The feedback predicates each slot kind offers to `speculate` at JS-M0
/// (open question Q-R0-4: shape predicates arrive with the IC language).
pub fn predicates(slot_kind: &str) -> &'static [&'static str] {
    match slot_kind {
        "binary" | "compare" => &["smi", "number", "string"],
        "unary" => &["smi", "number"],
        _ => &[],
    }
}

/// A resolved operand kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperandKind {
    /// A frame register.
    Reg,
    /// A register count.
    Cnt,
    /// A signed immediate (the verifier keeps it a small integer).
    Imm,
    /// A forward jump distance.
    Fwd,
    /// A backward jump distance.
    Back,
    /// An intrinsic id.
    Intr,
    /// A constant index accepting these kinds (sorted).
    Const(Vec<String>),
    /// An IC slot index of this kind.
    Slot(String),
}

impl OperandKind {
    /// The Rust `OperandKind` constructor in `hg_js_bytecode`.
    pub fn rust(&self) -> String {
        match self {
            Self::Reg => "OperandKind::Reg".into(),
            Self::Cnt => "OperandKind::Cnt".into(),
            Self::Imm => "OperandKind::Imm".into(),
            Self::Fwd => "OperandKind::Fwd".into(),
            Self::Back => "OperandKind::Back".into(),
            Self::Intr => "OperandKind::Intr".into(),
            Self::Const(ks) => {
                // One named constant so the table stays a promotable
                // constant expression.
                let names: Vec<String> = ks.iter().map(|k| k.to_ascii_uppercase()).collect();
                format!("OperandKind::Const(ConstKinds::{})", names.join("_"))
            }
            Self::Slot(k) => format!("OperandKind::Slot(SlotKind::{})", camel(k)),
        }
    }

    /// The kind as written in `.hgop`.
    pub fn text(&self) -> String {
        match self {
            Self::Reg => "reg".into(),
            Self::Cnt => "cnt".into(),
            Self::Imm => "imm".into(),
            Self::Fwd => "fwd".into(),
            Self::Back => "back".into(),
            Self::Intr => "intr".into(),
            Self::Const(ks) => format!("const<{}>", ks.join("|")),
            Self::Slot(k) => format!("slot<{k}>"),
        }
    }
}

/// `keyed-load` → `KeyedLoad`.
pub fn camel(s: &str) -> String {
    s.split(['-', '_'])
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or(String::new(), |f| {
                f.to_ascii_uppercase().to_string() + c.as_str()
            })
        })
        .collect()
}

/// `StringConcat` → `string_concat`.
pub fn snake(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            let prev_lower = i > 0 && chars[i - 1].is_ascii_lowercase();
            let next_lower = chars.get(i + 1).is_some_and(char::is_ascii_lowercase);
            let prev_upper = i > 0 && chars[i - 1].is_ascii_uppercase();
            if i > 0 && (prev_lower || (prev_upper && next_lower)) {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// A heap category reference: a declared category and its parameter
/// (`Field(any)`), as effects and fields name it (R0-O12).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cat {
    /// The category's name.
    pub name: String,
    /// Its parameter, if given (`any` matches every value).
    pub arg: Option<String>,
}

impl fmt::Display for Cat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.arg {
            Some(a) => write!(f, "{}({a})", self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

/// The root of the category tree: every heap category is under it.
pub const HEAP: &str = "Heap";

/// A declared category.
#[derive(Debug, Clone)]
pub struct CategoryInfo {
    /// Its name.
    pub name: String,
    /// Its parameter's type, if parameterised.
    pub param: Option<Ty>,
    /// Its parent's index.
    pub parent: Option<usize>,
    /// Changed at GC points (OC-9).
    pub gc_mutable: bool,
    /// Where it was declared.
    pub span: Span,
}

/// The category tree (R0-O12), in declaration order.
#[derive(Debug, Clone, Default)]
pub struct CatTree {
    /// The categories.
    pub cats: Vec<CategoryInfo>,
}

impl CatTree {
    /// The category named `name`.
    pub fn find(&self, name: &str) -> Option<usize> {
        self.cats.iter().position(|c| c.name == name)
    }

    /// Whether `outer` includes every location `inner` names: `inner` is
    /// `outer` or below it in the tree, with a matching parameter.
    pub fn covers(&self, outer: &Cat, inner: &Cat) -> bool {
        let wild = |a: &Option<String>| a.as_deref().is_none_or(|x| x == "any");
        if outer.name == inner.name {
            return wild(&outer.arg) || outer.arg == inner.arg;
        }
        if !wild(&outer.arg) {
            return false;
        }
        let Some(target) = self.find(&outer.name) else {
            return false;
        };
        let mut at = self.find(&inner.name);
        while let Some(i) = at {
            if i == target {
                return true;
            }
            at = self.cats[i].parent;
        }
        false
    }

    /// Whether `c` is changed at every GC point (`weak`).
    pub fn gc_mutable(&self, c: &Cat) -> bool {
        self.find(&c.name).is_some_and(|i| self.cats[i].gc_mutable)
    }
}

/// The units formula an engine-created error charges (bytecode note
/// Section 6: an error object and its `message` property).
pub const ERROR_UNITS: &str = "error_units";

/// A set of effects (language note 5.1), declared or inferred.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Effects {
    /// `alloc(u)` by units name.
    pub alloc: BTreeSet<String>,
    /// `alloc_internal`.
    pub alloc_internal: bool,
    /// `reads(c)`.
    pub reads: BTreeSet<Cat>,
    /// `writes(c)`.
    pub writes: BTreeSet<Cat>,
    /// `throw(k)`, with `any` for a user value.
    pub throws: BTreeSet<String>,
    /// `call_user`.
    pub call_user: bool,
    /// `host(H)`.
    pub host: BTreeSet<String>,
    /// `input(...)`.
    pub input: BTreeSet<InputKind>,
    /// `interrupt`.
    pub interrupt: bool,
}

/// A hook's context, from the hook table.
pub fn hook_context(name: &str) -> Option<HookContext> {
    HOOKS.iter().find(|h| h.0 == name).map(|h| h.2)
}

impl Effects {
    /// Whether the set throws an error the engine creates (which
    /// allocates, so it is a GC point: language note 5.1).
    pub fn throws_engine_error(&self) -> bool {
        self.throws.iter().any(|k| k != "any")
    }

    /// Whether any GC point is reachable (language note 5.3): an
    /// allocation, an interrupt check, user code, an engine-created error,
    /// or a hook whose context may allocate.
    pub fn gc(&self) -> bool {
        !self.alloc.is_empty()
            || self.alloc_internal
            || self.interrupt
            || self.call_user
            || self.throws_engine_error()
            || self.host.iter().any(|h| {
                matches!(
                    hook_context(h),
                    Some(HookContext::Alloc | HookContext::Script)
                )
            })
    }

    /// Whether user code may run (directly, or through a script hook).
    pub fn user(&self) -> bool {
        self.call_user
            || self
                .host
                .iter()
                .any(|h| hook_context(h) == Some(HookContext::Script))
    }

    /// Whether the set ends an epoch for every category (OC-9): user code
    /// or a host hook.
    pub fn ends_epoch(&self) -> bool {
        self.user() || !self.host.is_empty()
    }

    /// Whether the set blocks a later `speculate` (OC-7): a GC point, a
    /// write, user code, a host hook or an input.
    pub fn blocks_speculation(&self) -> bool {
        self.gc()
            || !self.writes.is_empty()
            || self.call_user
            || !self.host.is_empty()
            || !self.input.is_empty()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Adds `other`.
    pub fn add(&mut self, other: &Self) {
        self.alloc.extend(other.alloc.iter().cloned());
        self.alloc_internal |= other.alloc_internal;
        self.reads.extend(other.reads.iter().cloned());
        self.writes.extend(other.writes.iter().cloned());
        self.throws.extend(other.throws.iter().cloned());
        self.call_user |= other.call_user;
        self.host.extend(other.host.iter().cloned());
        self.input.extend(other.input.iter().copied());
        self.interrupt |= other.interrupt;
    }

    /// The set with what its effects imply (language note 5.1): an engine
    /// error allocates `error_units`; a hook's context gives reads of
    /// `Heap` (`&HostCx`), a GC point and writes of `Heap` (`&mut
    /// HostCx`) or user code (`&mut ScriptCx`); user code reads and writes
    /// every category, throws any value and checks for interrupts.
    pub fn closure(&self) -> Self {
        let mut c = self.clone();
        let heap = Cat {
            name: HEAP.into(),
            arg: None,
        };
        if c.throws_engine_error() {
            c.alloc.insert(ERROR_UNITS.into());
        }
        for h in &self.host {
            match hook_context(h) {
                Some(HookContext::Read) => {
                    c.reads.insert(heap.clone());
                }
                Some(HookContext::Alloc) => {
                    c.alloc_internal = true;
                    c.writes.insert(heap.clone());
                }
                Some(HookContext::Script) => c.call_user = true,
                Some(HookContext::None) | None => {}
            }
        }
        if c.call_user {
            c.reads.insert(heap.clone());
            c.writes.insert(heap);
            c.throws.insert("any".into());
            c.interrupt = true;
            c.alloc_internal = true;
        }
        c
    }

    /// What `self` has that `declared` (closed under implication) does
    /// not cover, as written effects.
    pub fn excess_over(&self, declared: &Self, tree: &CatTree) -> Vec<String> {
        let d = declared.closure();
        let mut out = Vec::new();
        for u in &self.alloc {
            if !d.alloc.contains(u) {
                out.push(format!("alloc({u})"));
            }
        }
        if self.alloc_internal && !d.alloc_internal {
            out.push("alloc_internal".into());
        }
        for c in &self.reads {
            if !d.reads.iter().any(|o| tree.covers(o, c)) {
                out.push(format!("reads({c})"));
            }
        }
        for c in &self.writes {
            if !d.writes.iter().any(|o| tree.covers(o, c)) {
                out.push(format!("writes({c})"));
            }
        }
        for k in &self.throws {
            if !d.throws.contains(k) && !d.throws.contains("any") {
                out.push(format!("throw({k})"));
            }
        }
        if self.call_user && !d.call_user {
            out.push("call_user".into());
        }
        for h in &self.host {
            if !d.host.contains(h) {
                out.push(format!("host({h})"));
            }
        }
        for i in &self.input {
            if !d.input.contains(i) {
                out.push(format!("input({})", input_name(*i)));
            }
        }
        if self.interrupt && !d.interrupt {
            out.push("interrupt".into());
        }
        out
    }

    /// The set as written, in a fixed order (OC-15).
    pub fn text(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for u in &self.alloc {
            parts.push(format!("alloc({u})"));
        }
        if self.alloc_internal {
            parts.push("alloc_internal".into());
        }
        if !self.reads.is_empty() {
            let cs: Vec<String> = self.reads.iter().map(ToString::to_string).collect();
            parts.push(format!("reads({})", cs.join(", ")));
        }
        if !self.writes.is_empty() {
            let cs: Vec<String> = self.writes.iter().map(ToString::to_string).collect();
            parts.push(format!("writes({})", cs.join(", ")));
        }
        if !self.throws.is_empty() {
            let ks: Vec<&str> = self.throws.iter().map(String::as_str).collect();
            parts.push(format!("throw({})", ks.join(", ")));
        }
        if self.call_user {
            parts.push("call_user".into());
        }
        for h in &self.host {
            parts.push(format!("host({h})"));
        }
        for i in &self.input {
            parts.push(format!("input({})", input_name(*i)));
        }
        if self.interrupt {
            parts.push("interrupt".into());
        }
        parts.join(", ")
    }

    /// The debugger's class of an operation with these effects (language
    /// note Section 7): the most effectful that applies.
    pub fn debugger_class(&self) -> &'static str {
        if self.user() {
            "may-run-user-code"
        } else if !self.host.is_empty() {
            "reaches-host-hook"
        } else if !self.input.is_empty() {
            "reads-input"
        } else if !self.writes.is_empty() || self.gc() {
            "writes-heap"
        } else {
            "no-effect"
        }
    }
}

/// A fact kind and its invalidating categories.
#[derive(Debug, Clone)]
pub struct FactKind {
    /// Its name.
    pub name: String,
    /// The categories whose writes invalidate it.
    pub on: Vec<Cat>,
}

/// The facts clauses of one item.
#[derive(Debug, Clone, Default)]
pub struct ItemFacts {
    /// Fact kinds the item may invalidate.
    pub invalidates: Vec<String>,
    /// Result representations.
    pub results: Vec<String>,
    /// The type-transfer helper.
    pub transfer: Option<String>,
    /// The SMT encoding id.
    pub smt: Option<String>,
    /// Canonical depth bytes when called as a frame.
    pub frame_charge: Option<u64>,
}

/// An input's name as written.
pub const fn input_name(i: InputKind) -> &'static str {
    match i {
        InputKind::Entropy => "entropy",
        InputKind::Clock => "clock",
    }
}

/// A named constant.
#[derive(Debug, Clone)]
pub struct Const {
    /// Its name.
    pub name: String,
    /// Its type.
    pub ty: Ty,
    /// Its value.
    pub value: ConstValue,
    /// Where it was declared.
    pub span: Span,
}

/// A constant's value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstValue {
    /// An integer.
    Int(i128),
    /// A float.
    Float(f64),
    /// A Boolean.
    Bool(bool),
}

/// A canonical-units formula.
#[derive(Debug, Clone)]
pub struct Units {
    /// Its name.
    pub name: String,
    /// Parameters (all integers).
    pub params: Vec<(String, Ty)>,
    /// The formula, over parameters (as locals 0..n) and constants.
    pub body: TExpr,
    /// Where it was declared.
    pub span: Span,
}

/// A layout field.
#[derive(Debug, Clone)]
pub struct Field {
    /// Its name.
    pub name: String,
    /// Its type.
    pub ty: Ty,
    /// The category it lives in if it is mutable; `None` if immutable.
    pub category: Option<Cat>,
    /// Representation-dependent.
    pub repr: bool,
}

/// The name a tail call's kind has in the generated Rust.
pub const fn call_kind_rust(k: CallKind) -> &'static str {
    match k {
        CallKind::Call => "CallKind::Call",
        CallKind::Construct => "CallKind::Construct",
        CallKind::Getter => "CallKind::Getter",
        CallKind::Setter => "CallKind::Setter",
        CallKind::IterStep => "CallKind::IterStep",
    }
}

/// A layout (the fields descriptions may read, and the allocation of
/// fixed-size kinds).
#[derive(Debug, Clone)]
pub struct Layout {
    /// The kind's name (a [`Kind`], or `Shape` or `Rope`).
    pub name: String,
    /// The reference kind field reads take, if fields are readable.
    pub read_from: Option<Kind>,
    /// The reference kind an allocation yields, if allocatable.
    pub alloc_as: Option<Kind>,
    /// The fields.
    pub fields: Vec<Field>,
    /// Where it was declared.
    pub span: Span,
}

/// The kinds the hand-written heap layer can allocate (typed heap
/// operations of hg-js-ops, language note Section 4).
pub const ALLOCATABLE: [&str; 5] = ["HeapNumber", "Rope", "WeakRef", "FinReg", "HostFn"];

/// A declared signature.
#[derive(Debug, Clone)]
pub struct Sig {
    /// Parameters.
    pub params: Vec<(String, Ty)>,
    /// Which parameters are declared `frame` (OC-6).
    pub frame_params: Vec<bool>,
    /// The after-call accumulator mode.
    pub after_call: Option<AfterCall>,
    /// The facts clauses.
    pub facts: ItemFacts,
    /// Result type.
    pub ret: Ty,
    /// Declared effects.
    pub effects: Effects,
    /// The item's anchor.
    pub anchor: Anchor,
}

/// A runtime function.
#[derive(Debug, Clone)]
pub struct Runtime {
    /// Its name.
    pub name: String,
    /// Its signature.
    pub sig: Sig,
    /// `ensures result is P`.
    pub ensures: Option<TPat>,
    /// Where it was declared.
    pub span: Span,
}

/// What kind of described item a body belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    /// An operation.
    Op,
    /// A helper.
    Helper,
    /// An intrinsic.
    Intrinsic,
}

impl ItemKind {
    /// The kind's name, for facts and anchors.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Op => "op",
            Self::Helper => "helper",
            Self::Intrinsic => "intrinsic",
        }
    }
}

/// A helper or an intrinsic.
#[derive(Debug, Clone)]
pub struct Func {
    /// Its name.
    pub name: String,
    /// Its signature.
    pub sig: Sig,
    /// The checked body.
    pub body: Body,
    /// Where it was declared.
    pub span: Span,
    /// The file it came from.
    pub file: u16,
}

/// An operation.
#[derive(Debug, Clone)]
pub struct Op {
    /// Its name and mnemonic.
    pub name: String,
    /// Its opcode.
    pub opcode: u8,
    /// Operands: name and kind.
    pub operands: Vec<(String, OperandKind)>,
    /// Accumulator mode.
    pub acc: AccMode,
    /// The after-call accumulator mode.
    pub after_call: Option<AfterCall>,
    /// The facts clauses.
    pub facts: ItemFacts,
    /// Declared effects.
    pub effects: Effects,
    /// Anchor.
    pub anchor: Anchor,
    /// The checked body.
    pub body: Body,
    /// Where it was declared.
    pub span: Span,
    /// The file it came from.
    pub file: u16,
}

/// Everything the descriptions declare, checked.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// The input files, as given (relative paths).
    pub files: Vec<String>,
    /// The category tree.
    pub cats: CatTree,
    /// Fact kinds.
    pub fact_kinds: Vec<FactKind>,
    /// Constants.
    pub consts: Vec<Const>,
    /// Units formulas.
    pub units: Vec<Units>,
    /// Layouts.
    pub layouts: Vec<Layout>,
    /// Runtime functions.
    pub runtimes: Vec<Runtime>,
    /// Helpers.
    pub helpers: Vec<Func>,
    /// Intrinsics.
    pub intrinsics: Vec<Func>,
    /// Operations, in opcode order.
    pub ops: Vec<Op>,
}
