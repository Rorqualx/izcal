//! The `.hgop` syntax tree, as the parser builds it (language note
//! Section 3). Names are resolved and types checked by `check`.

use crate::diag::Span;

/// A name and where it was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    /// The name.
    pub text: String,
    /// Where.
    pub span: Span,
}

/// A type as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    /// A named type: `Value`, `i32`, `u32`, `i64`, `f64`, `bool`, `Key`,
    /// `ObjectKind`.
    Named(Name),
    /// `Ref<K>`.
    Ref(Name),
    /// `Empty | Value`: a weak slot's contents.
    Weak(Span),
}

/// A literal.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// An integer.
    Int(i128),
    /// A float, as written.
    Float(String),
    /// `true` or `false`.
    Bool(bool),
}

/// A declared parameter.
#[derive(Debug, Clone)]
pub struct Param {
    /// Its name.
    pub name: Name,
    /// `frame`: every caller passes a frame-backed value (OC-6).
    pub frame: bool,
    /// Its type.
    pub ty: TypeExpr,
}

/// A reference to a declared heap category, with its parameter if the
/// category takes one (`Field(any)`, `weak`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatRef {
    /// The category's name.
    pub name: Name,
    /// The parameter, as written (`any` or a literal), if given.
    pub arg: Option<String>,
}

/// `category NAME [(p: T)] [under PARENT] [gc_mutable];` (R0-O12).
#[derive(Debug, Clone)]
pub struct CategoryDecl {
    /// The name.
    pub name: Name,
    /// The parameter.
    pub param: Option<(Name, TypeExpr)>,
    /// The parent in the tree.
    pub under: Option<Name>,
    /// Changed at GC points (a cycle clears weak slots).
    pub gc_mutable: bool,
}

/// `fact NAME on c, c;`: a fact kind and the categories whose writes
/// invalidate it (R0-O12).
#[derive(Debug, Clone)]
pub struct FactDecl {
    /// The name.
    pub name: Name,
    /// The invalidating categories.
    pub on: Vec<CatRef>,
}

/// A `facts` clause the optimizer reads (part 2, optimizer-ir Section 9).
#[derive(Debug, Clone)]
pub enum FactClause {
    /// `invalidates(F, ...)`.
    Invalidates(Vec<Name>),
    /// `results(repr, ...)`: `smi`, `f64`, `tagged`, `bool`.
    Results(Vec<Name>),
    /// `transfer IDENT`.
    Transfer(Name),
    /// `smt IDENT`.
    Smt(Name),
    /// `frame_charge N`.
    FrameCharge(u64, Span),
}

/// What the accumulator holds after an item's user call returns (R0-O11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterCall {
    /// The call's value.
    Result,
    /// The accumulator is left unchanged.
    Preserve,
}

/// The kind of a tail user call (R0-O11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallKind {
    /// `call`
    Call,
    /// `construct`
    Construct,
    /// `getter`
    Getter,
    /// `setter`
    Setter,
    /// `iter_step`
    IterStep,
}

impl CallKind {
    /// The kind named `s`.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "call" => Self::Call,
            "construct" => Self::Construct,
            "getter" => Self::Getter,
            "setter" => Self::Setter,
            "iter_step" => Self::IterStep,
            _ => return None,
        })
    }

    /// The name as written.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Construct => "construct",
            Self::Getter => "getter",
            Self::Setter => "setter",
            Self::IterStep => "iter_step",
        }
    }
}

/// A B1 input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputKind {
    /// The realm's entropy stream.
    Entropy,
    /// The clock, through the host's time hook.
    Clock,
}

/// One declared effect (language note 5.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// `alloc(units)`: a semantic allocation.
    Alloc(Name),
    /// `alloc_internal`.
    AllocInternal(Span),
    /// `reads(...)`.
    Reads(Vec<CatRef>, Span),
    /// `writes(...)`.
    Writes(Vec<CatRef>, Span),
    /// `throw(...)`.
    Throw(Vec<Name>),
    /// `call_user`.
    CallUser(Span),
    /// `host(H)`.
    Host(Name),
    /// `input(entropy)` or `input(clock)`.
    Input(InputKind, Span),
    /// `interrupt`.
    Interrupt(Span),
}

/// Which specification an anchor names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Spec {
    /// ECMA-262.
    Ecma262,
    /// ECMA-402.
    Ecma402,
    /// Temporal.
    Temporal,
    /// Dynamic Code Brand Checks.
    Dcbc,
    /// HTML.
    Html,
    /// Web IDL.
    WebIdl,
    /// WebAssembly.
    Wasm,
}

impl Spec {
    /// The name as written.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ecma262 => "ecma262",
            Self::Ecma402 => "ecma402",
            Self::Temporal => "temporal",
            Self::Dcbc => "dcbc",
            Self::Html => "html",
            Self::WebIdl => "webidl",
            Self::Wasm => "wasm",
        }
    }

    /// The spec named `s`.
    pub fn parse(s: &str) -> Option<Self> {
        [
            Self::Ecma262,
            Self::Ecma402,
            Self::Temporal,
            Self::Dcbc,
            Self::Html,
            Self::WebIdl,
            Self::Wasm,
        ]
        .into_iter()
        .find(|x| x.name() == s)
    }
}

/// An anchor (language note 5.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anchor {
    /// A clause id at the pinned revision, with an optional step.
    Spec {
        /// The specification.
        spec: Spec,
        /// The clause id.
        clause: String,
        /// The step, if given.
        step: Option<String>,
        /// Where.
        span: Span,
    },
    /// An engine-defined path, with its reason.
    Izcal {
        /// The reason.
        reason: String,
        /// Where.
        span: Span,
    },
}

impl Anchor {
    /// Where the anchor was written.
    pub const fn span(&self) -> Span {
        match self {
            Self::Spec { span, .. } | Self::Izcal { span, .. } => *span,
        }
    }
}

/// An operand kind (bytecode note 4.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperandKind {
    /// A frame register.
    Reg,
    /// A register count.
    Cnt,
    /// A signed immediate.
    Imm,
    /// A forward jump distance.
    Fwd,
    /// A backward jump distance.
    Back,
    /// An intrinsic id.
    Intr,
    /// A constant index, with the kinds it accepts.
    Const(Vec<Name>),
    /// An IC slot index, with its slot kind.
    Slot(Name),
}

/// An operation's operand.
#[derive(Debug, Clone)]
pub struct Operand {
    /// Its name.
    pub name: Name,
    /// Its kind.
    pub kind: OperandKind,
}

/// An operation's accumulator mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccMode {
    /// Neither read nor written.
    None,
    /// Read.
    In,
    /// Written.
    Out,
    /// Read and written.
    InOut,
}

impl AccMode {
    /// The mode as written.
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::In => "in",
            Self::Out => "out",
            Self::InOut => "inout",
        }
    }
}

/// A field's mutability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutability {
    /// Never changes after the object is published (matters for C9).
    Immutable,
    /// Mutable, in this heap category.
    In(CatRef),
}

/// A layout field.
#[derive(Debug, Clone)]
pub struct FieldDecl {
    /// Its name.
    pub name: Name,
    /// Its type.
    pub ty: TypeExpr,
    /// Mutability.
    pub mutability: Mutability,
    /// Representation-dependent (C8 rejects it in a charge).
    pub repr: bool,
}

/// `layout K [is K'] { ... }`.
#[derive(Debug, Clone)]
pub struct LayoutDecl {
    /// The kind.
    pub kind: Name,
    /// Allocation yields a reference of this kind (`Rope is String`).
    pub is: Option<Name>,
    /// The fields.
    pub fields: Vec<FieldDecl>,
}

/// A runtime contract: `ensures result is P`.
#[derive(Debug, Clone)]
pub struct Contract {
    /// The pattern the result matches.
    pub pattern: Pattern,
    /// Where.
    pub span: Span,
}

/// `runtime f(...) -> T effects {...} anchor ... [ensures ...];`.
#[derive(Debug, Clone)]
pub struct RuntimeDecl {
    /// The name.
    pub name: Name,
    /// Parameters.
    pub params: Vec<Param>,
    /// Result type.
    pub ret: TypeExpr,
    /// Declared effects.
    pub effects: Vec<Effect>,
    /// Facts clauses.
    pub facts: Vec<FactClause>,
    /// Anchor.
    pub anchor: Anchor,
    /// Contract.
    pub ensures: Option<Contract>,
}

/// A helper or an intrinsic.
#[derive(Debug, Clone)]
pub struct FnDecl {
    /// The name (without `%` for an intrinsic).
    pub name: Name,
    /// Parameters.
    pub params: Vec<Param>,
    /// Result type.
    pub ret: TypeExpr,
    /// The after-call accumulator mode (intrinsics).
    pub after_call: Option<AfterCall>,
    /// Declared effects.
    pub effects: Vec<Effect>,
    /// Facts clauses.
    pub facts: Vec<FactClause>,
    /// Anchor.
    pub anchor: Anchor,
    /// Body.
    pub body: Block,
}

/// `op Name(...) acc mode effects {...} anchor ... { ... }`.
#[derive(Debug, Clone)]
pub struct OpDecl {
    /// The name, which is also the mnemonic.
    pub name: Name,
    /// Operands, in encoding order.
    pub operands: Vec<Operand>,
    /// Accumulator mode.
    pub acc: AccMode,
    /// The after-call accumulator mode.
    pub after_call: Option<AfterCall>,
    /// Declared effects.
    pub effects: Vec<Effect>,
    /// Facts clauses.
    pub facts: Vec<FactClause>,
    /// Anchor.
    pub anchor: Anchor,
    /// Body.
    pub body: Block,
}

/// `const NAME: T = literal;`.
#[derive(Debug, Clone)]
pub struct ConstDecl {
    /// The name.
    pub name: Name,
    /// Its scalar type.
    pub ty: TypeExpr,
    /// Its value.
    pub value: Literal,
    /// A leading `-` was written.
    pub negative: bool,
}

/// `units name(params) = expr;`: a canonical-units formula (R0-B10).
#[derive(Debug, Clone)]
pub struct UnitsDecl {
    /// The name.
    pub name: Name,
    /// Parameters (semantic quantities).
    pub params: Vec<Param>,
    /// The formula.
    pub body: Expr,
}

/// A top-level item.
#[derive(Debug, Clone)]
pub enum Item {
    /// A constant.
    Const(ConstDecl),
    /// A units formula.
    Units(UnitsDecl),
    /// A layout.
    Layout(LayoutDecl),
    /// A heap category.
    Category(CategoryDecl),
    /// A fact kind.
    Fact(FactDecl),
    /// A runtime function.
    Runtime(RuntimeDecl),
    /// A helper.
    Helper(FnDecl),
    /// An intrinsic.
    Intrinsic(FnDecl),
    /// An operation.
    Op(OpDecl),
}

/// A block: statements and an optional trailing value.
#[derive(Debug, Clone)]
pub struct Block {
    /// Statements.
    pub stmts: Vec<Stmt>,
    /// The trailing expression, which is the block's value.
    pub tail: Option<Box<Expr>>,
    /// Where it opens.
    pub span: Span,
}

/// An assignment target.
#[derive(Debug, Clone)]
pub enum Place {
    /// The accumulator.
    Acc(Span),
    /// `reg[operand]`.
    Reg(Name),
}

/// A `path`: a named, anchored region of the decision tree.
#[derive(Debug, Clone)]
pub struct PathDecl {
    /// The name.
    pub name: Name,
    /// The path's own anchor, if any.
    pub anchor: Option<Anchor>,
    /// `speculate slot.predicate`, if any.
    pub speculate: Option<(Name, Name)>,
    /// The body.
    pub body: Block,
}

/// A terminal.
#[derive(Debug, Clone)]
pub enum Terminal {
    /// `next;`
    Next(Span),
    /// `jump target;`
    Jump(Name),
    /// `return [e];`
    Return(Option<Box<Expr>>, Span),
    /// `throw Kind;`
    Throw(Name),
    /// `call_user kind(callee, this, args...);`: a tail user call, then
    /// `next`.
    TailCall(CallKind, Vec<Expr>, Span),
}

impl Terminal {
    /// Where.
    pub fn span(&self) -> Span {
        match self {
            Self::Next(s) | Self::Return(_, s) | Self::TailCall(_, _, s) => *s,
            Self::Jump(n) | Self::Throw(n) => n.span,
        }
    }
}

/// A statement.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let x [: T] = e;`
    Let {
        /// The bound name.
        name: Name,
        /// The declared type.
        ty: Option<TypeExpr>,
        /// The value.
        value: Expr,
    },
    /// `place = e;`
    Assign {
        /// The target.
        place: Place,
        /// The value.
        value: Expr,
    },
    /// `if c { } [else ...]`
    If {
        /// The condition.
        cond: Expr,
        /// The then-block.
        then: Block,
        /// The else-block (an `else if` is a block holding one `If`).
        els: Option<Block>,
        /// Where.
        span: Span,
    },
    /// A `match` statement.
    Match(Match),
    /// A `path`.
    Path(PathDecl),
    /// `charge(units(args));`
    Charge {
        /// The units formula.
        units: Name,
        /// Its arguments.
        args: Vec<Expr>,
        /// Where.
        span: Span,
    },
    /// An expression evaluated for its effect (a typed heap operation or
    /// a call whose result is unused).
    Expr(Expr),
    /// A terminal.
    Terminal(Terminal),
}

/// A pattern (language note Section 3).
#[derive(Debug, Clone)]
pub enum Pattern {
    /// `Undefined`, `Null`, `True`, `False` or `Empty`.
    Oddball(Name),
    /// `Smi(x)`.
    Smi(Name),
    /// `Num(x)`.
    Num(Name),
    /// `Ref<K>(x)`.
    Ref(Name, Name),
    /// `Value(x)`.
    Value(Name),
    /// `(p, q)`.
    Tuple(Vec<Pattern>, Span),
    /// `p | q`.
    Or(Vec<Pattern>),
    /// `_`.
    Wild(Span),
}

impl Pattern {
    /// Where the pattern starts.
    pub fn span(&self) -> Span {
        match self {
            Self::Oddball(n) | Self::Smi(n) | Self::Num(n) | Self::Value(n) | Self::Ref(n, _) => {
                n.span
            }
            Self::Tuple(_, s) | Self::Wild(s) => *s,
            Self::Or(ps) => ps.first().map(Pattern::span).unwrap_or_default(),
        }
    }
}

/// A match arm's body.
#[derive(Debug, Clone)]
pub enum ArmBody {
    /// A path.
    Path(PathDecl),
    /// A block.
    Block(Block),
    /// A terminal.
    Terminal(Terminal),
    /// A value, for a match expression.
    Expr(Expr),
}

/// A match arm.
#[derive(Debug, Clone)]
pub struct Arm {
    /// The pattern.
    pub pattern: Pattern,
    /// The guard.
    pub guard: Option<Expr>,
    /// The body.
    pub body: ArmBody,
}

/// A `match`, as a statement or an expression.
#[derive(Debug, Clone)]
pub struct Match {
    /// The scrutinee (a tuple expression for a tuple match).
    pub scrutinee: Box<Expr>,
    /// The arms.
    pub arms: Vec<Arm>,
    /// Where.
    pub span: Span,
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// `||`
    Or,
    /// `&&`
    And,
    /// `==`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `|`
    BitOr,
    /// `^`
    BitXor,
    /// `&`
    BitAnd,
    /// `<<`
    Shl,
    /// `>>`
    Sar,
    /// `>>>`
    Shr,
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Rem,
}

impl BinOp {
    /// The operator as written.
    pub const fn text(self) -> &'static str {
        match self {
            Self::Or => "||",
            Self::And => "&&",
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::BitAnd => "&",
            Self::Shl => "<<",
            Self::Sar => ">>",
            Self::Shr => ">>>",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Rem => "%",
        }
    }
}

/// A unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    /// `!`
    Not,
    /// `-`
    Neg,
    /// `~`
    BitNot,
}

/// An expression.
#[derive(Debug, Clone)]
pub enum Expr {
    /// A literal.
    Lit(Literal, Span),
    /// A name: a local, a parameter, an operand, a constant, an oddball
    /// (`Undefined`, `Null`, `True`, `False`) or an object kind.
    Ident(Name),
    /// `acc`.
    Acc(Span),
    /// `reg[operand]`.
    Reg(Name),
    /// `const[operand]`.
    ConstPool(Name),
    /// `f(args)`: a runtime function, a helper or a constructor.
    Call(Name, Vec<Expr>),
    /// `q.f(args)`: a typed heap operation or a primitive.
    QCall(Name, Name, Vec<Expr>),
    /// `alloc K { field: e, ... }`.
    Alloc(Name, Vec<(Name, Expr)>),
    /// A `match` expression.
    Match(Match),
    /// A block expression.
    Block(Block),
    /// `l op r`.
    Binary(BinOp, Box<Expr>, Box<Expr>, Span),
    /// `op e`.
    Unary(UnOp, Box<Expr>, Span),
    /// `(a, b, ...)`: only as a match scrutinee.
    Tuple(Vec<Expr>, Span),
}

impl Expr {
    /// Where the expression starts.
    pub fn span(&self) -> Span {
        match self {
            Self::Lit(_, s)
            | Self::Acc(s)
            | Self::Binary(_, _, _, s)
            | Self::Unary(_, _, s)
            | Self::Tuple(_, s) => *s,
            Self::Ident(n)
            | Self::Reg(n)
            | Self::ConstPool(n)
            | Self::Call(n, _)
            | Self::QCall(n, _, _)
            | Self::Alloc(n, _) => n.span,
            Self::Match(m) => m.span,
            Self::Block(b) => b.span,
        }
    }
}
