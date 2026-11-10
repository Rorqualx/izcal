//! What the generated interpreter runs in: the frame, the realm's
//! singletons, the constants, the hooks, the completion types and the
//! interpreter context [`Cx`], with its canonical-units pacing (R0-B11).

use hg_heap::{Any, Mutator, Root, Scope, Value};
use hg_js_bytecode::{Instr, Intrinsic, OperandKind, UnitView};
use hg_substrate::RandomStream;

pub use hg_js_bytecode::Reg;

use crate::heap::{self, ConstArray, FrameObj, Oddball, Ref, kind, tag};

/// The cage is full: the agent cannot continue (JS spec 6.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfMemory;

/// The error constructors an engine-thrown error names.
#[allow(clippy::enum_variant_names)] // the specification's names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// TypeError.
    TypeError,
    /// RangeError.
    RangeError,
    /// ReferenceError.
    ReferenceError,
    /// SyntaxError.
    SyntaxError,
    /// EvalError.
    EvalError,
    /// URIError.
    URIError,
    /// AggregateError.
    AggregateError,
}

impl ErrorKind {
    /// Every kind, by code (`code() - 1`).
    pub const ALL: [Self; 7] = [
        Self::TypeError,
        Self::RangeError,
        Self::ReferenceError,
        Self::SyntaxError,
        Self::EvalError,
        Self::URIError,
        Self::AggregateError,
    ];

    /// The constructor's name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::TypeError => "TypeError",
            Self::RangeError => "RangeError",
            Self::ReferenceError => "ReferenceError",
            Self::SyntaxError => "SyntaxError",
            Self::EvalError => "EvalError",
            Self::URIError => "URIError",
            Self::AggregateError => "AggregateError",
        }
    }

    /// A non-zero code (the frame's pending-error word).
    pub const fn code(self) -> u32 {
        match self {
            Self::TypeError => 1,
            Self::RangeError => 2,
            Self::ReferenceError => 3,
            Self::SyntaxError => 4,
            Self::EvalError => 5,
            Self::URIError => 6,
            Self::AggregateError => 7,
        }
    }

    /// The kind with `code`.
    pub fn from_code(code: u32) -> Option<Self> {
        let i = usize::try_from(code.checked_sub(1)?).ok()?;
        Self::ALL.get(i).copied()
    }
}

/// An abrupt completion inside an instruction.
pub enum Throw<'h, 's> {
    /// The engine throws a new error of this kind (its units already
    /// charged; the object is made when the completion leaves the job).
    Error(ErrorKind),
    /// User code threw this value.
    Value(Root<'h, 's, Any>),
    /// The cage is full.
    OutOfMemory,
    /// The instruction stream is not one the verifier would admit.
    BadBytecode,
}

impl From<OutOfMemory> for Throw<'_, '_> {
    fn from(_: OutOfMemory) -> Self {
        Self::OutOfMemory
    }
}

/// What an instruction does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// The next instruction.
    Next,
    /// The instruction at this offset.
    Jump(u32),
    /// Return the accumulator.
    Return,
    /// The frame holds a thrown value.
    Throw,
    /// The interrupt check asked the unit to stop.
    Interrupt,
    /// Bytecode the verifier would reject.
    Bad,
}

/// How a unit's run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    /// It returned; the value is the frame's accumulator.
    Normal,
    /// It threw; the frame holds the value or the pending error.
    Throw,
    /// The interrupt check stopped it.
    Interrupted,
    /// The instruction at this offset is not admissible.
    BadBytecode(u32),
}

/// The five tail user-call kinds (R0-O11). JS-M0's descriptions make
/// none; the runtime's [`RuntimeBase::tail_call`] is their one entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallKind {
    /// `[[Call]]`.
    Call,
    /// `[[Construct]]`.
    Construct,
    /// A getter.
    Getter,
    /// A setter.
    Setter,
    /// An iterator step.
    IterStep,
}

/// The capability to run script: only a runtime method that declares
/// `call_user` receives it, so a method without it cannot run user code
/// (language note 5.1). The agent makes one per job.
///
/// This is the **script** token of CS-D3 and Q-JS-10, deliberately not
/// the same thing as `Mutator`, which is the **GC** token: running script
/// takes both, and allocating takes only the mutator (Q-CS-7). It carries
/// no data; what it carries is the fact that the path reached it from a
/// job, which the generated signatures then propagate.
#[derive(Debug)]
pub struct ScriptToken {
    _p: (),
}

impl ScriptToken {
    /// The token for one job. Only the agent's job loop calls this:
    /// `clippy.toml` disallows it everywhere else, so the two places that
    /// mint a token are the two that carry an `#[expect]` saying why.
    ///
    /// It takes the mutator because CS-D3 criterion 1 asks that the token
    /// be derivable only from `&mut Mutator`. Before the JS-M0 re-run it
    /// took nothing at all, so any crate could make one from thin air;
    /// nothing came of it, because no path can run script without a
    /// mutator either, but the criterion is about the mechanism and the
    /// mechanism was a comment.
    ///
    /// The token is **not** branded with `'h`. A brand would stop it
    /// outliving its heap, which sounds stronger and buys nothing here:
    /// the token holds no heap data, and every method that consumes it
    /// takes a `&mut Mutator<'h>` beside it, so a token that outlived its
    /// `enter` could not be used with the next one's mutator anyway.
    #[doc(hidden)]
    pub fn for_job(_m: &mut Mutator<'_>) -> Self {
        Self { _p: () }
    }
}

/// A value passed to or from the host (no heap reference crosses).
#[derive(Debug, Clone, PartialEq)]
pub enum HostValue {
    /// `undefined`.
    Undefined,
    /// `null`.
    Null,
    /// A Boolean.
    Bool(bool),
    /// A Number.
    Number(f64),
    /// A String's code units.
    String(Vec<u16>),
    /// An object (the host sees only that it is one).
    Object,
}

/// The host hooks JS-M0 reaches (JS host-hooks note): each is passed only
/// to a runtime method that declares it.
pub trait Host<'h> {
    /// HostMakeJobCallback: the host-defined value for a callback.
    fn make_job_callback(&mut self) -> i64;
    /// HostSystemUTCEpochNanoseconds (Temporal): the wall clock.
    fn system_utc_epoch_nanoseconds(&mut self) -> i128;
    /// HostEnqueueFinalizationRegistryCleanupJob: registry `job` has cells
    /// to clean up; the host runs `job` later, as its own job.
    fn enqueue_cleanup_job(&mut self, job: u64);
}

/// What every runtime provides besides the description-declared methods.
pub trait RuntimeBase<'h> {
    /// A tail user call (R0-O11): the last effect of its path; its result
    /// is the accumulator.
    ///
    /// # Errors
    /// What the callee throws.
    #[allow(clippy::too_many_arguments)]
    fn tail_call<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        kind: CallKind,
        callee: Root<'h, 's, Any>,
        this: Root<'h, 's, Any>,
        args: &[Root<'h, 's, Any>],
    ) -> Result<Root<'h, 's, Any>, Throw<'h, 's>>;

    /// The agent's canonical-units clock. The runtime's own methods
    /// charge through it too, so every charge counts toward the trigger.
    fn pacer(&mut self) -> &mut Pacer;

    /// Called after each canonical cycle, at its decision point: the
    /// runtime looks at what the cycle cleared (finalization cells) and
    /// enqueues cleanup jobs.
    fn on_canonical_cycle(&mut self, m: &mut Mutator<'h>, host: &mut dyn Host<'h>);
}

/// The realm's singletons (JS-M0: the oddballs), rooted for the agent's
/// life.
#[derive(Clone, Copy)]
pub struct Realm<'h, 's> {
    undefined: Root<'h, 's, Any>,
    null: Root<'h, 's, Any>,
    true_: Root<'h, 's, Any>,
    false_: Root<'h, 's, Any>,
    empty: Root<'h, 's, Any>,
}

impl<'h, 's> Realm<'h, 's> {
    /// Allocates the oddballs, rooted in `s`.
    ///
    /// # Errors
    /// [`OutOfMemory`] if the cage is full.
    pub fn new(m: &mut Mutator<'h>, s: &Scope<'h, 's>) -> Result<Self, OutOfMemory> {
        let mut one = |t: u32| -> Result<Root<'h, 's, Any>, OutOfMemory> {
            let o = m.alloc::<Oddball>(s, 0)?;
            let g = o.get(m);
            g.set_raw(m, Oddball::TAG, t);
            Ok(s.root(m, g.erase()))
        };
        Ok(Self {
            undefined: one(tag::UNDEFINED)?,
            null: one(tag::NULL)?,
            true_: one(tag::TRUE)?,
            false_: one(tag::FALSE)?,
            empty: one(tag::EMPTY)?,
        })
    }

    /// A realm over existing oddball roots (the agent re-roots its realm
    /// in each job's scope).
    pub fn from_values(
        undefined: Root<'h, 's, Any>,
        null: Root<'h, 's, Any>,
        true_: Root<'h, 's, Any>,
        false_: Root<'h, 's, Any>,
        empty: Root<'h, 's, Any>,
    ) -> Self {
        Self {
            undefined,
            null,
            true_,
            false_,
            empty,
        }
    }

    /// `undefined`.
    pub fn undefined<'m>(&self, m: &'m Mutator<'h>) -> Value<'h, 'm> {
        self.undefined.get_value(m)
    }
    /// `null`.
    pub fn null<'m>(&self, m: &'m Mutator<'h>) -> Value<'h, 'm> {
        self.null.get_value(m)
    }
    /// `true`.
    pub fn true_value<'m>(&self, m: &'m Mutator<'h>) -> Value<'h, 'm> {
        self.true_.get_value(m)
    }
    /// `false`.
    pub fn false_value<'m>(&self, m: &'m Mutator<'h>) -> Value<'h, 'm> {
        self.false_.get_value(m)
    }
    /// The empty value.
    pub fn empty<'m>(&self, m: &'m Mutator<'h>) -> Value<'h, 'm> {
        self.empty.get_value(m)
    }
    /// `true` or `false`.
    pub fn boolean<'m>(&self, m: &'m Mutator<'h>, b: bool) -> Value<'h, 'm> {
        if b {
            self.true_value(m)
        } else {
            self.false_value(m)
        }
    }
}

/// A unit's constants for one agent: a rooted `ConstArray`.
#[derive(Clone, Copy)]
pub struct Consts<'h, 's> {
    array: Root<'h, 's, Any>,
}

impl<'h, 's> Consts<'h, 's> {
    /// The constants `array` holds (a `ConstArray`, built by the runtime).
    pub fn new(array: Root<'h, 's, Any>) -> Self {
        Self { array }
    }

    /// An empty table, for a unit without constants.
    ///
    /// # Errors
    /// [`OutOfMemory`] if the cage is full.
    pub fn empty(m: &mut Mutator<'h>, s: &Scope<'h, 's>) -> Result<Self, OutOfMemory> {
        let a = m.alloc::<ConstArray>(s, 0)?;
        let g = a.get(m).erase();
        Ok(Self::new(s.root(m, g)))
    }

    /// Constant `k` (admitted: in range, of the operand's kind).
    pub fn get<'m>(&self, m: &'m Mutator<'h>, k: u32) -> Value<'h, 'm> {
        let Value::Ref(g) = self.array.get_value(m) else {
            return Value::Small(0);
        };
        g.cast::<ConstArray>(m)
            .and_then(|a| a.tail(m, k))
            .unwrap_or(Value::Small(0))
    }

    /// Constant `k` as a property key.
    pub fn key<'m>(&self, m: &'m Mutator<'h>, k: u32) -> Ref<'h, 'm, kind::Key> {
        Ref::new(self.get(m, k))
    }

    /// The number of constants.
    pub fn len(&self, m: &Mutator<'h>) -> u32 {
        match self.array.get_value(m) {
            Value::Ref(g) => g.len(m),
            Value::Small(_) => 0,
        }
    }

    /// Whether there are none.
    pub fn is_empty(&self, m: &Mutator<'h>) -> bool {
        self.len(m) == 0
    }
}

/// An interpreter frame (bytecode note 4.3), a rooted `FrameObj`.
#[derive(Clone, Copy)]
pub struct Frame<'h, 's> {
    obj: Root<'h, 's, FrameObj>,
    params: u32,
    registers: u32,
}

/// The accumulator's register index: the header's accumulator spill
/// slot, `R − 28`.
pub const ACC_INDEX: i32 = -2;

impl<'h, 's> Frame<'h, 's> {
    /// A frame for `params` parameters and `registers` locals, with the
    /// receiver `this`, the arguments `args` (missing ones undefined),
    /// every local and the accumulator undefined.
    ///
    /// # Errors
    /// [`OutOfMemory`] if the cage is full or the frame too large.
    pub fn new(
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        realm: &Realm<'h, '_>,
        params: u32,
        registers: u32,
        this: Root<'h, 's, Any>,
        args: &[Root<'h, 's, Any>],
    ) -> Result<Self, OutOfMemory> {
        let len = params
            .checked_add(13)
            .and_then(|n| n.checked_add(registers))
            .ok_or(OutOfMemory)?;
        let obj = m.alloc::<FrameObj>(s, len)?;
        let f = Self {
            obj,
            params,
            registers,
        };
        let g = obj.get(m);
        g.set_raw(m, FrameObj::PARAMS, params);
        g.set_raw(m, FrameObj::REGISTERS, registers);
        let u = realm.undefined(m);
        for i in 0..len {
            g.set_tail(m, i, u);
        }
        // The untagged header slots (status, identity, argc, bytecode
        // offset, the saved fp and return address) hold small integers.
        for v in [-12, -11, -10, -9, -8, -7, -4] {
            if let Some(i) = f.index(Reg(v)) {
                g.set_tail(m, i, Value::Small(0));
            }
        }
        let argc = i32::try_from(args.len()).unwrap_or(i32::MAX);
        if let Some(i) = f.index(Reg(-6)) {
            g.set_tail(m, i, Value::Small(argc.min((1 << 30) - 1)));
        }
        f.set_reg(m, Reg::RECEIVER, this.get_value(m));
        for (j, a) in (0u32..params).zip(args) {
            if let Some(r) = Reg::param(j) {
                f.set_reg(m, r, a.get_value(m));
            }
        }
        Ok(f)
    }

    /// The tail slot of register `r`, if `r` is in this frame (a header
    /// index is in the object, but only the frame's own code reaches it).
    fn index(&self, r: Reg) -> Option<u32> {
        let i = i64::from(self.params) + 13 + i64::from(r.0);
        let len = i64::from(self.params) + 13 + i64::from(self.registers);
        if (0..len).contains(&i) {
            u32::try_from(i).ok()
        } else {
            None
        }
    }

    /// The frame's object.
    pub fn root(&self) -> Root<'h, 's, FrameObj> {
        self.obj
    }

    /// The parameter count.
    pub fn params(&self) -> u32 {
        self.params
    }

    /// The register count.
    pub fn registers(&self) -> u32 {
        self.registers
    }

    /// Register `r` (admitted, so in the frame).
    pub fn reg<'m>(&self, m: &'m Mutator<'h>, r: Reg) -> Value<'h, 'm> {
        self.index(r)
            .and_then(|i| self.obj.get(m).tail(m, i))
            .unwrap_or(Value::Small(0))
    }

    /// Writes register `r`.
    pub fn set_reg(&self, m: &Mutator<'h>, r: Reg, v: Value<'h, '_>) {
        if let Some(i) = self.index(r) {
            self.obj.get(m).set_tail(m, i, v);
        }
    }

    /// The accumulator.
    pub fn acc<'m>(&self, m: &'m Mutator<'h>) -> Value<'h, 'm> {
        self.reg(m, Reg(ACC_INDEX))
    }

    /// Writes the accumulator.
    pub fn set_acc(&self, m: &Mutator<'h>, v: Value<'h, '_>) {
        self.set_reg(m, Reg(ACC_INDEX), v);
    }

    /// Records an abrupt completion and says how the unit continues.
    pub fn set_thrown(&self, m: &Mutator<'h>, e: Throw<'h, '_>) -> Flow {
        let g = self.obj.get(m);
        match e {
            Throw::Error(k) => {
                g.set_raw(m, FrameObj::THROWN_KIND, k.code());
                g.set_value(m, FrameObj::THROWN, Value::Small(0));
                Flow::Throw
            }
            Throw::Value(v) => {
                g.set_raw(m, FrameObj::THROWN_KIND, 0);
                g.set_value(m, FrameObj::THROWN, v.get_value(m));
                Flow::Throw
            }
            Throw::OutOfMemory => {
                g.set_raw(m, FrameObj::THROWN_KIND, THROWN_OOM);
                Flow::Throw
            }
            Throw::BadBytecode => Flow::Bad,
        }
    }

    /// What the frame threw, after [`Completion::Throw`].
    pub fn thrown<'m>(&self, m: &'m Mutator<'h>) -> Thrown<'h, 'm> {
        let g = self.obj.get(m);
        match g.raw(m, FrameObj::THROWN_KIND) {
            0 => Thrown::Value(g.get_value(m, FrameObj::THROWN)),
            THROWN_OOM => Thrown::OutOfMemory,
            k => ErrorKind::from_code(k).map_or(Thrown::OutOfMemory, Thrown::Error),
        }
    }
}

const THROWN_OOM: u32 = 0xFFFF;

/// What a frame threw.
#[derive(Debug, Clone, Copy)]
pub enum Thrown<'h, 'm> {
    /// A value.
    Value(Value<'h, 'm>),
    /// A new error of this kind, not yet made.
    Error(ErrorKind),
    /// The cage filled.
    OutOfMemory,
}

/// A register list operand: `count` consecutive locals from `base`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegList {
    base: Reg,
    count: u32,
}

impl RegList {
    /// The list `base:count`.
    pub fn new(base: Reg, count: u32) -> Self {
        Self { base, count }
    }

    /// Element `i` (the interpreter admitted `i < count`).
    pub fn at(&self, i: u32) -> Reg {
        let v = i64::from(self.base.0) + i64::from(i.min(self.count.saturating_sub(1)));
        Reg(i32::try_from(v).unwrap_or(i32::MIN))
    }

    /// The count.
    pub fn count(&self) -> u32 {
        self.count
    }
}

/// The canonical-units clock of one agent (R0-B10, R0-B11): what has
/// been charged since the last canonical cycle, and the trigger. A
/// charge that reaches the trigger requests a cycle; the cycle runs at
/// the next decision point (a `LoopBack`, a function or job entry, or a
/// job's end), so where cycles happen depends only on the canonical
/// units and the progress counter, never on representation or tier.
#[derive(Debug, Clone)]
pub struct Pacer {
    debt: u64,
    trigger: u64,
    requested: bool,
    /// The interrupt checks left before the agent is interrupted (tests
    /// and the embedder's watchdog); `None` never interrupts.
    pub fuel: Option<u64>,
    cycles: u64,
}

impl Pacer {
    /// A clock that requests a cycle every `trigger` units.
    pub fn new(trigger: u64) -> Self {
        Self {
            debt: 0,
            trigger: trigger.max(1),
            requested: false,
            fuel: None,
            cycles: 0,
        }
    }

    /// Charges `units`.
    pub fn charge(&mut self, units: u64) {
        self.debt = self.debt.saturating_add(units);
        if self.debt >= self.trigger {
            self.requested = true;
        }
    }

    /// `charge(u)` at a GC point: charges `units` here and in the heap's
    /// own count, and reaches the heap's safepoint (where its stress
    /// modes move objects). The heap's trigger is never reached: the
    /// agent makes its heap with a trigger of `u64::MAX`, so canonical
    /// cycles run only at decision points.
    pub fn charge_at(&mut self, m: &mut Mutator<'_>, units: u64) {
        self.charge(units);
        m.charge(units);
        m.safepoint();
    }

    /// Whether a cycle is requested.
    pub fn requested(&self) -> bool {
        self.requested
    }

    /// The canonical cycles run so far.
    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    /// A decision point: runs the requested cycle, if any, and returns
    /// whether it ran.
    pub fn decide(&mut self, m: &mut Mutator<'_>) -> bool {
        if !self.requested {
            return false;
        }
        self.requested = false;
        self.debt = 0;
        self.cycles += 1;
        m.collect();
        true
    }
}

/// The interpreter's context for one frame: the capabilities the
/// runtime methods receive, the realm, the frame, the unit's constants
/// and strictness, and the pacing clock.
pub struct Cx<'a, 'h, 's, R> {
    /// The script capability of the running job.
    pub t: &'a mut ScriptToken,
    /// The runtime.
    pub rt: &'a mut R,
    /// The host hooks.
    pub host: &'a mut dyn Host<'h>,
    /// The realm's `Math.random` stream (`Entropy::stream(Random)`).
    pub random: &'a mut dyn RandomStream,
    /// The realm.
    pub realm: &'a Realm<'h, 's>,
    /// The frame.
    pub frame: Frame<'h, 's>,
    /// The unit's constants.
    pub consts: Consts<'h, 's>,
    /// Whether the unit is strict.
    pub strict: bool,
}

impl<'h, R: RuntimeBase<'h>> Cx<'_, 'h, '_, R> {
    /// `charge(u)`: charges `units` canonical units (a GC point: the
    /// heap's stress modes move objects here). A cycle the charge
    /// requests runs at the next decision point (R0-B11).
    pub fn charge(&mut self, m: &mut Mutator<'h>, units: u64) {
        self.rt.pacer().charge_at(m, units);
    }

    /// `interrupt.poll()` at a `LoopBack`: the progress counter advances,
    /// a requested cycle runs (with its cleanup bookkeeping), the
    /// safepoint is reached, and the agent's fuel is checked. Returns
    /// whether to continue.
    pub fn poll(&mut self, m: &mut Mutator<'h>) -> bool {
        m.advance(1);
        if self.rt.pacer().decide(m) {
            self.rt.on_canonical_cycle(m, &mut *self.host);
        }
        m.safepoint();
        match &mut self.rt.pacer().fuel {
            Some(0) => false,
            Some(n) => {
                *n -= 1;
                true
            }
            None => true,
        }
    }

    /// Whether `i` is admissible in `unit` and this frame: what JS-M1's
    /// verifier will prove once per unit, checked here per instruction
    /// (bytecode note 8). Registers are in the frame; constants and slots
    /// exist and are of the operand's kind; jumps stay in the code; an
    /// intrinsic exists and its list has its arity; a `LoadInt` immediate
    /// is a small integer.
    pub fn admits(&self, i: &Instr, unit: &UnitView<'_>) -> bool {
        let kinds = i.op.info().operands;
        let code_len = u32::try_from(unit.code().len()).unwrap_or(u32::MAX);
        let in_frame = |r: Reg| -> bool {
            r.resolve(self.frame.params(), self.frame.registers())
                .is_some()
        };
        for (j, k) in kinds.iter().enumerate() {
            let ok = match *k {
                OperandKind::Reg => {
                    if kinds.get(j + 1) == Some(&OperandKind::Cnt) {
                        let n = i.index(j + 1);
                        let base = i.reg(j);
                        n == 0
                            || (in_frame(base)
                                && in_frame(RegList::new(base, n).at(n - 1))
                                && base.0 >= 0)
                    } else {
                        in_frame(i.reg(j))
                    }
                }
                OperandKind::Cnt => true,
                OperandKind::Imm => heap::is_smi_range(i.operand(j)),
                OperandKind::Const(want) => unit
                    .constant_kind(i.index(j))
                    .is_some_and(|c| want.contains(c)),
                OperandKind::Slot(want) => unit.slot_kind(i.index(j)) == Some(want),
                OperandKind::Fwd => i.forward_target(j) < code_len && i.index(j) > 0,
                OperandKind::Back => i.backward_target(j) < code_len,
                OperandKind::Intr => match Intrinsic::from_index(i.index(j)) {
                    Some(x) => {
                        kinds.get(j + 2) == Some(&OperandKind::Cnt) && i.index(j + 2) == x.arity()
                    }
                    None => false,
                },
            };
            if !ok {
                return false;
            }
        }
        true
    }
}

/// The primitives the descriptions name (language note 4.3): exact, with
/// no implementation-approximated step (OC-14).
pub mod prim {
    /// ToInt32 on a Number.
    pub fn to_int32(x: f64) -> i32 {
        hg_js_number::to_int32(x)
    }

    /// ToUint32 on a Number.
    pub fn to_uint32(x: f64) -> u32 {
        hg_js_number::to_uint32(x)
    }

    /// Number::remainder.
    pub fn remainder(n: f64, d: f64) -> f64 {
        hg_js_number::remainder(n, d)
    }

    /// A shift count: the low five bits of the right operand.
    pub trait ShiftCount {
        /// The count, modulo 32.
        fn count(self) -> u32;
    }
    impl ShiftCount for u32 {
        fn count(self) -> u32 {
            self & 31
        }
    }
    impl ShiftCount for i32 {
        fn count(self) -> u32 {
            self.cast_unsigned() & 31
        }
    }

    /// Number::leftShift on the converted operands.
    pub fn shl(a: i32, b: impl ShiftCount) -> i32 {
        a.wrapping_shl(b.count())
    }

    /// Number::signedRightShift.
    pub fn sar(a: i32, b: impl ShiftCount) -> i32 {
        a >> b.count()
    }

    /// Number::unsignedRightShift.
    pub fn shr(a: u32, b: impl ShiftCount) -> u32 {
        a >> b.count()
    }

    /// An integer the descriptions proved to be within ±2^53, as a double.
    #[allow(clippy::cast_precision_loss)]
    pub fn exact_f64(x: i64) -> f64 {
        x as f64
    }

    /// An integer proved in i32's range.
    pub fn to_i32(x: i64) -> i32 {
        i32::try_from(x).unwrap_or(0)
    }

    /// An integer proved in u32's range.
    pub fn to_u32(x: i64) -> u32 {
        u32::try_from(x).unwrap_or(0)
    }
}
