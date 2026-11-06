//! The typed intermediate form the checker produces from the syntax tree:
//! names resolved, every expression typed, implicit upcasts made
//! explicit only where representation changes (none at JS-M0). The flow
//! analysis (`flow`) and the emitters read this form.

use crate::ast::{Anchor, BinOp, CallKind, UnOp};
use crate::diag::Span;
use crate::model::{Kind, Oddball, Ty};

/// A local's index in its body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalId(pub u32);

/// Where a local's value comes from, syntactically (R0-O6's frame-backed
/// values are the `Frame` origins; whether the frame slot is still
/// unwritten at a use is checked per route).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Read from the frame: the accumulator, or the register operand `n`.
    Frame(FrameLoc),
    /// The `i`th parameter of an intrinsic: a register of the argument
    /// list, which an intrinsic cannot write.
    RegParam(usize),
    /// The `i`th parameter of a helper, declared `frame`: every caller
    /// passes a frame-backed value (OC-6 checks the call sites).
    FrameParam(usize),
    /// The `i`th parameter of a helper with a GC point: a `Root` in the
    /// helper's own scope.
    RootedParam(usize),
    /// The `i`th parameter of a helper without a GC point.
    Param(usize),
    /// A constant-pool value (rooted by the unit's constant array).
    Const,
    /// Anything else: a temporary.
    Temp,
}

/// A frame location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrameLoc {
    /// The accumulator.
    Acc,
    /// The register named by operand `n`.
    Reg(usize),
}

/// A local variable or binder.
#[derive(Debug, Clone)]
pub struct Local {
    /// Its name as written.
    pub name: String,
    /// Its type.
    pub ty: Ty,
    /// Where it was bound.
    pub span: Span,
    /// Where its value comes from.
    pub origin: Origin,
    /// Whether any expression reads it.
    pub used: bool,
}

/// A path's facts as declared (language note 5.4, 5.5).
#[derive(Debug, Clone)]
pub struct PathInfo {
    /// Its name (`main` for the item's root).
    pub name: String,
    /// The anchor in force: its own or the nearest enclosing one.
    pub anchor: Anchor,
    /// Whether the anchor is the path's own.
    pub own_anchor: bool,
    /// `speculate slot.predicate`: the slot operand's index and the
    /// predicate.
    pub speculate: Option<(usize, String)>,
    /// The enclosing path.
    pub parent: Option<usize>,
    /// The guard that selects the path, as written, for the facts.
    pub guard: String,
    /// Where.
    pub span: Span,
}

/// A checked body: an operation's, a helper's or an intrinsic's.
#[derive(Debug, Clone)]
pub struct Body {
    /// The statements.
    pub block: TBlock,
    /// Its locals (parameters first, for helpers and intrinsics).
    pub locals: Vec<Local>,
    /// Its paths; path 0 is the root, `main`.
    pub paths: Vec<PathInfo>,
}

/// A primitive operation (total, with SMT-LIB semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prim {
    /// `f64.add`
    FAdd,
    /// `f64.sub`
    FSub,
    /// `f64.mul`
    FMul,
    /// `f64.div`
    FDiv,
    /// `f64.neg`
    FNeg,
    /// `f64.lt`
    FLt,
    /// `f64.le`
    FLe,
    /// `f64.gt`
    FGt,
    /// `f64.ge`
    FGe,
    /// `f64.eq`
    FEq,
    /// `f64.is_nan`
    FIsNan,
    /// `f64.to_int32` (ToInt32's modular semantics)
    FToInt32,
    /// `f64.to_uint32` (ToUint32's modular semantics)
    FToUint32,
    /// `f64.fmod` (Number::remainder's truncating semantics)
    FMod,
}

impl Prim {
    /// The primitive named `f64.<name>`: its parameter count and types.
    pub fn parse(name: &str) -> Option<(Self, &'static [Ty], Ty)> {
        const FF: &[Ty] = &[Ty::F64, Ty::F64];
        const F: &[Ty] = &[Ty::F64];
        Some(match name {
            "add" => (Self::FAdd, FF, Ty::F64),
            "sub" => (Self::FSub, FF, Ty::F64),
            "mul" => (Self::FMul, FF, Ty::F64),
            "div" => (Self::FDiv, FF, Ty::F64),
            "neg" => (Self::FNeg, F, Ty::F64),
            "lt" => (Self::FLt, FF, Ty::Bool),
            "le" => (Self::FLe, FF, Ty::Bool),
            "gt" => (Self::FGt, FF, Ty::Bool),
            "ge" => (Self::FGe, FF, Ty::Bool),
            "eq" => (Self::FEq, FF, Ty::Bool),
            "is_nan" => (Self::FIsNan, F, Ty::Bool),
            "to_int32" => (Self::FToInt32, F, Ty::I32),
            "to_uint32" => (Self::FToUint32, F, Ty::U32),
            "fmod" => (Self::FMod, FF, Ty::F64),
            _ => return None,
        })
    }
}

/// A typed expression.
#[derive(Debug, Clone)]
pub struct TExpr {
    /// What it is.
    pub kind: TE,
    /// Its type.
    pub ty: Ty,
    /// Where.
    pub span: Span,
}

/// Typed expression forms.
#[derive(Debug, Clone)]
pub enum TE {
    /// An integer literal (or an integer constant, inlined).
    Int(i128),
    /// A float literal or constant.
    Float(f64),
    /// A Boolean literal or constant.
    Bool(bool),
    /// A local.
    Local(LocalId),
    /// The accumulator.
    Acc,
    /// `reg[operand]`.
    Reg(usize),
    /// `const[operand]`; `true` when the operand's kind is `key`.
    ConstPool(usize, bool),
    /// An `imm` operand's value.
    Imm(usize),
    /// An oddball.
    Oddball(Oddball),
    /// An object kind (`Dict`, ...) as an `ObjectKind` value.
    ObjKind(Kind),
    /// A runtime call.
    Runtime(usize, Vec<TExpr>),
    /// A helper call.
    Helper(usize, Vec<TExpr>),
    /// `intrinsic(id, list, count)`: dispatch to the intrinsic named by
    /// the operand (operand indices).
    Intrinsic {
        /// The `intr` operand.
        id: usize,
        /// The first register of the list.
        args: usize,
        /// The `cnt` operand.
        count: usize,
    },
    /// A primitive.
    Prim(Prim, Vec<TExpr>),
    /// A typed heap read: layout, field, the reference.
    Field(usize, usize, Box<TExpr>),
    /// `kept.push(v)`.
    KeptPush(Box<TExpr>),
    /// `interrupt.poll()`.
    Poll,
    /// `unit.strict()`.
    UnitStrict,
    /// `smi(e)`.
    Smi(Box<TExpr>),
    /// `number(e)`.
    Number(Box<TExpr>),
    /// `bool(e)`.
    BoolValue(Box<TExpr>),
    /// `alloc K { ... }`: the layout and the field values in layout order.
    Alloc(usize, Vec<TExpr>),
    /// A conversion to an integer or float type.
    Conv(Ty, Box<TExpr>),
    /// A binary operation on scalars.
    Bin(BinOp, Box<TExpr>, Box<TExpr>),
    /// Identity of two heap values (`==`, or `!=` when the flag is set).
    Same(Box<TExpr>, Box<TExpr>, bool),
    /// A unary operation.
    Un(UnOp, Box<TExpr>),
    /// A `match` expression.
    Match(Box<TMatch>),
    /// A block expression.
    Block(Box<TBlock>),
}

/// A typed statement.
#[derive(Debug, Clone)]
pub enum TStmt {
    /// `let`.
    Let(LocalId, TExpr),
    /// `acc = e`.
    SetAcc(TExpr, Span),
    /// `reg[operand] = e`.
    SetReg(usize, TExpr, Span),
    /// `if`.
    If(TExpr, TBlock, Option<TBlock>),
    /// A `match` statement.
    Match(TMatch),
    /// A `path`.
    Path(TPath),
    /// `charge(units(args))`.
    Charge(usize, Vec<TExpr>, Span),
    /// An expression for its effect.
    Expr(TExpr),
    /// A terminal.
    Term(TTerm),
}

/// A typed terminal.
#[derive(Debug, Clone)]
pub enum TTerm {
    /// `next`.
    Next(Span),
    /// `jump operand`; `true` for a backward jump.
    Jump(usize, bool, Span),
    /// `return [e]`.
    Return(Option<TExpr>, Span),
    /// `throw Kind`.
    Throw(String, Span),
    /// `call_user kind(callee, this, args...)`, then `next`.
    TailCall(CallKind, Vec<TExpr>, Span),
}

/// A typed block.
#[derive(Debug, Clone)]
pub struct TBlock {
    /// Statements.
    pub stmts: Vec<TStmt>,
    /// The value, if any.
    pub tail: Option<TExpr>,
}

/// A typed match.
#[derive(Debug, Clone)]
pub struct TMatch {
    /// The scrutinees (one, or the parts of a tuple).
    pub scrutinees: Vec<TExpr>,
    /// The arms.
    pub arms: Vec<TArm>,
    /// The match's type (`Unit` for a statement).
    pub ty: Ty,
    /// Where.
    pub span: Span,
}

/// A typed arm.
#[derive(Debug, Clone)]
pub struct TArm {
    /// One pattern per scrutinee.
    pub pats: Vec<TPat>,
    /// The guard.
    pub guard: Option<TExpr>,
    /// The body.
    pub body: TArmBody,
    /// The pattern as written, for the facts.
    pub text: String,
}

/// A typed arm body.
#[derive(Debug, Clone)]
pub enum TArmBody {
    /// A path.
    Path(TPath),
    /// A block.
    Block(TBlock),
    /// A terminal.
    Term(TTerm),
    /// A value.
    Expr(TExpr),
}

/// A typed path.
#[derive(Debug, Clone)]
pub struct TPath {
    /// Its index in the body's path table.
    pub id: usize,
    /// Its body.
    pub body: TBlock,
}

/// A typed pattern.
#[derive(Debug, Clone)]
pub enum TPat {
    /// `_`
    Wild,
    /// An oddball.
    Oddball(Oddball),
    /// `Empty` (a cleared weak slot).
    Empty,
    /// `Smi(x)`.
    Smi(Option<LocalId>),
    /// `Num(x)`.
    Num(Option<LocalId>),
    /// `Ref<K>(x)`.
    Ref(Kind, Option<LocalId>),
    /// `Value(x)`.
    Value(Option<LocalId>),
    /// Alternatives (without binders).
    Or(Vec<TPat>),
}
