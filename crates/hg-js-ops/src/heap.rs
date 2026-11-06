//! The JS-M0 value layer over `hg-heap`: the object layouts, the typed
//! references the generated interpreter uses, the classification of a
//! value into the descriptions' kinds, and the typed heap operations of
//! the language note (Section 4). Each typed operation is O(1), reads the
//! cage as untrusted bytes (H-2: a wrong type gives a default answer,
//! never an access outside the cage) and is not a GC point unless it
//! allocates.
//!
//! **Canonical Numbers.** A Number is a small integer when it is
//! integral, in [−2^30, 2^30 − 1] and not −0; otherwise it is a
//! `HeapNumber` holding the double, with NaN stored as the one canonical
//! quiet NaN (JS-D12 (a)). [`number`] is the only way to make one, so
//! every tier sees the same representation.

use core::marker::PhantomData;

use hg_heap::{AllocError, Any, Gc, Mutator, Object, Root, Scope, Value};

use crate::support::{OutOfMemory, Realm};

hg_heap::object! {
    /// `undefined`, `null`, `true`, `false` and the empty value: one
    /// object per heap, each held as a root by the realm.
    pub Oddball {
        raw TAG: u32,
        tail none,
    }
}

hg_heap::object! {
    /// A Number that is not a small integer.
    pub HeapNumber {
        raw VALUE: f64,
        tail none,
    }
}

hg_heap::object! {
    /// A flat string: its UTF-16 code units, little-endian, two bytes each.
    pub FlatString {
        tail bytes,
    }
}

hg_heap::object! {
    /// A concatenation not yet flattened: two strings and the total length
    /// (language note 9.4; `ROPE_MIN_LENGTH` code units or more).
    pub Rope {
        traced LEFT: Any,
        traced RIGHT: Any,
        raw LENGTH: u32,
        tail none,
    }
}

hg_heap::object! {
    /// A BigInt: a sign (0 zero, 1 positive, 2 negative) and the magnitude's
    /// bytes, little-endian. JS-M0 makes none; the kind exists so every
    /// description's match is exhaustive.
    pub BigIntObj {
        raw SIGN: u32,
        tail bytes,
    }
}

hg_heap::object! {
    /// A Symbol: its description (a string or undefined) and whether it is
    /// in the global symbol registry (`Symbol.for`).
    pub SymbolObj {
        traced DESCRIPTION: Any,
        raw REGISTERED: bool,
        tail none,
    }
}

hg_heap::object! {
    /// An ordinary object in dictionary mode: its prototype (an object or
    /// null), its property table and the number of properties in use.
    pub Dict {
        traced PROTO: Any,
        traced PROPS: Any,
        raw COUNT: u32,
        raw FLAGS: u32,
        tail none,
    }
}

hg_heap::object! {
    /// A dictionary's properties: key, value, key, value, ... (keys are
    /// interned strings or symbols, compared by identity).
    pub PropTable {
        tail slots,
    }
}

hg_heap::object! {
    /// A WeakRef: its target, held weakly (heap design H-10).
    pub WeakRefObj {
        weak TARGET: Any,
        tail none,
    }
}

hg_heap::object! {
    /// A FinalizationRegistry: the cleanup callback, its host-defined
    /// value from HostMakeJobCallback, its cells (a list), and its pending
    /// cleanup job plus one (0 when none is pending).
    pub FinRegObj {
        traced CLEANUP: Any,
        traced CELLS: Any,
        raw HOST_DEFINED: u64,
        raw PENDING: u32,
        tail none,
    }
}

hg_heap::object! {
    /// One [[Cells]] record of a registry: the held value and the next cell
    /// are strong, the target and the unregister token weak. `STATE` is 0
    /// while registered, 1 once its cleanup is pending, 2 once removed.
    pub Cell {
        traced HELD: Any,
        traced REGISTRY: Any,
        traced NEXT: Any,
        weak TARGET: Any,
        weak TOKEN: Any,
        raw STATE: u32,
        tail none,
    }
}

hg_heap::object! {
    /// A function the embedder implements, named by a small integer.
    pub HostFnObj {
        raw ID: i32,
        tail none,
    }
}

hg_heap::object! {
    /// An interpreter frame (bytecode note 4.3): the thrown value and the
    /// register file. Tail slot `params + 13 + v` holds register `v`, so
    /// the parameters, the receiver, the twelve header slots and the
    /// locals are in the frame record's order; the accumulator is the
    /// header's accumulator spill slot, `v = −2`.
    pub FrameObj {
        traced THROWN: Any,
        raw PARAMS: u32,
        raw REGISTERS: u32,
        raw THROWN_KIND: u32,
        tail slots,
    }
}

hg_heap::object! {
    /// A unit's constants, materialized for one agent (bytecode note 4.5).
    pub ConstArray {
        tail slots,
    }
}

/// The oddball tags.
pub mod tag {
    /// `undefined`.
    pub const UNDEFINED: u32 = 1;
    /// `null`.
    pub const NULL: u32 = 2;
    /// `true`.
    pub const TRUE: u32 = 3;
    /// `false`.
    pub const FALSE: u32 = 4;
    /// The empty value (a cleared weak slot, a hole).
    pub const EMPTY: u32 = 5;
}

/// The bit of `Dict::FLAGS` that marks an `[[IsHTMLDDA]]` object (Annex
/// B.3.6); only a test host makes one at JS-M0.
pub const DICT_HTMLDDA: u32 = 1;

/// The kinds the descriptions name, as type markers for [`Ref`] and
/// [`Rooted`]. `Key` is a property key: a String or a Symbol.
pub mod kind {
    /// A String (flat or rope).
    #[derive(Debug)]
    pub enum String {}
    /// A heap Number.
    #[derive(Debug)]
    pub enum HeapNumber {}
    /// A BigInt.
    #[derive(Debug)]
    pub enum BigInt {}
    /// A Symbol.
    #[derive(Debug)]
    pub enum Symbol {}
    /// An Object of any object kind.
    #[derive(Debug)]
    pub enum Object {}
    /// A property key.
    #[derive(Debug)]
    pub enum Key {}
    /// A WeakRef.
    #[derive(Debug)]
    pub enum WeakRef {}
    /// A FinalizationRegistry.
    #[derive(Debug)]
    pub enum FinReg {}
    /// A host function.
    #[derive(Debug)]
    pub enum HostFn {}
}

/// An object's kind (the descriptions' `ObjectKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    /// An ordinary object in dictionary mode.
    Dict,
    /// A WeakRef.
    WeakRef,
    /// A FinalizationRegistry.
    FinReg,
    /// A host function.
    HostFn,
}

/// An unrooted value the descriptions know to be of kind `K`. It cannot
/// live across a GC point (it borrows the mutator).
pub struct Ref<'h, 'm, K> {
    v: Value<'h, 'm>,
    _k: PhantomData<fn() -> K>,
}

impl<K> Clone for Ref<'_, '_, K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K> Copy for Ref<'_, '_, K> {}

impl<K> core::fmt::Debug for Ref<'_, '_, K> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Ref({:?})", self.v)
    }
}

impl<'h, 'm, K> Ref<'h, 'm, K> {
    /// `v` as a `K`. The caller has classified it; a wrong kind gives
    /// default answers from the typed operations, never unsafety.
    pub fn new(v: Value<'h, 'm>) -> Self {
        Self { v, _k: PhantomData }
    }

    /// The value.
    pub fn value(self) -> Value<'h, 'm> {
        self.v
    }

    /// Roots it in `s`.
    pub fn root<'s>(self, m: &Mutator<'h>, s: &Scope<'h, 's>) -> Rooted<'h, 's, K> {
        Rooted {
            r: s.root_value(m, self.v),
            _k: PhantomData,
        }
    }

    /// The same value as another kind (a `String` as a `Key`).
    pub fn cast<L>(self) -> Ref<'h, 'm, L> {
        Ref::new(self.v)
    }

    fn get<T: Object>(self, m: &'m Mutator<'h>) -> Option<Gc<'h, 'm, T>> {
        match self.v {
            Value::Ref(g) => g.cast::<T>(m),
            Value::Small(_) => None,
        }
    }
}

/// A rooted value of kind `K`: survives GC points within its scope.
pub struct Rooted<'h, 's, K> {
    r: Root<'h, 's, Any>,
    _k: PhantomData<fn() -> K>,
}

impl<K> Clone for Rooted<'_, '_, K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K> Copy for Rooted<'_, '_, K> {}

impl<'h, 's, K> Rooted<'h, 's, K> {
    /// `r` as a `K` (the caller has classified it).
    pub fn new(r: Root<'h, 's, Any>) -> Self {
        Self { r, _k: PhantomData }
    }

    /// The value, for as long as the mutator is borrowed.
    pub fn get<'m>(&self, m: &'m Mutator<'h>) -> Ref<'h, 'm, K> {
        Ref::new(self.r.get_value(m))
    }

    /// The root, untyped.
    pub fn erase(self) -> Root<'h, 's, Any> {
        self.r
    }

    /// The same root as another kind.
    pub fn cast<L>(self) -> Rooted<'h, 's, L> {
        Rooted::new(self.r)
    }
}

/// A value classified into the descriptions' kinds (language note 4.1):
/// the oddballs, a small integer, and each heap kind.
#[derive(Debug, Clone, Copy)]
pub enum Class<'h, 'm> {
    /// `undefined` (also what an unknown or corrupted reference reads as).
    Undefined,
    /// `null`.
    Null,
    /// `true`.
    True,
    /// `false`.
    False,
    /// A small integer.
    Smi(i32),
    /// A heap Number.
    HeapNumber(Ref<'h, 'm, kind::HeapNumber>),
    /// A String.
    String(Ref<'h, 'm, kind::String>),
    /// A BigInt.
    BigInt(Ref<'h, 'm, kind::BigInt>),
    /// A Symbol.
    Symbol(Ref<'h, 'm, kind::Symbol>),
    /// An ordinary dictionary object.
    Dict(Ref<'h, 'm, kind::Object>),
    /// A WeakRef.
    WeakRef(Ref<'h, 'm, kind::WeakRef>),
    /// A FinalizationRegistry.
    FinReg(Ref<'h, 'm, kind::FinReg>),
    /// A host function.
    HostFn(Ref<'h, 'm, kind::HostFn>),
}

impl<'h, 'm> Class<'h, 'm> {
    /// A Number class's value (`Num(x)` in the descriptions); NaN for
    /// any other class, which the descriptions never ask.
    pub fn number(self, m: &'m Mutator<'h>) -> f64 {
        match self {
            Self::Smi(i) => f64::from(i),
            Self::HeapNumber(h) => heap_number_value(m, h),
            _ => f64::NAN,
        }
    }

    /// An object class as `Ref<Object>`. For a class that is not an
    /// object (never asked by the descriptions) it is a reference that
    /// every typed operation answers by default.
    pub fn object(self) -> Ref<'h, 'm, kind::Object> {
        match self {
            Self::Dict(r) => r,
            Self::WeakRef(r) => r.cast(),
            Self::FinReg(r) => r.cast(),
            Self::HostFn(r) => r.cast(),
            Self::HeapNumber(r) => r.cast(),
            Self::String(r) => r.cast(),
            Self::BigInt(r) => r.cast(),
            Self::Symbol(r) => r.cast(),
            Self::Smi(i) => Ref::new(Value::Small(i)),
            Self::Undefined | Self::Null | Self::True | Self::False => Ref::new(Value::Small(0)),
        }
    }
}

/// The class of `v`.
pub fn classify<'h, 'm>(m: &'m Mutator<'h>, v: Value<'h, 'm>) -> Class<'h, 'm> {
    let g = match v {
        Value::Small(i) => return Class::Smi(i),
        Value::Ref(g) => g,
    };
    if let Some(o) = g.cast::<Oddball>(m) {
        return match o.raw(m, Oddball::TAG) {
            tag::NULL => Class::Null,
            tag::TRUE => Class::True,
            tag::FALSE => Class::False,
            _ => Class::Undefined,
        };
    }
    if g.cast::<HeapNumber>(m).is_some() {
        Class::HeapNumber(Ref::new(v))
    } else if g.cast::<FlatString>(m).is_some() || g.cast::<Rope>(m).is_some() {
        Class::String(Ref::new(v))
    } else if g.cast::<Dict>(m).is_some() {
        Class::Dict(Ref::new(v))
    } else if g.cast::<WeakRefObj>(m).is_some() {
        Class::WeakRef(Ref::new(v))
    } else if g.cast::<FinRegObj>(m).is_some() {
        Class::FinReg(Ref::new(v))
    } else if g.cast::<HostFnObj>(m).is_some() {
        Class::HostFn(Ref::new(v))
    } else if g.cast::<SymbolObj>(m).is_some() {
        Class::Symbol(Ref::new(v))
    } else if g.cast::<BigIntObj>(m).is_some() {
        Class::BigInt(Ref::new(v))
    } else {
        Class::Undefined
    }
}

impl From<AllocError> for OutOfMemory {
    fn from(_: AllocError) -> Self {
        OutOfMemory
    }
}

/// The small integer `i` (the caller has proved it in range; out of
/// range it would read back as 0, so [`number`] is used otherwise).
pub fn smi<'h, 'm>(i: i32) -> Value<'h, 'm> {
    Value::Small(i)
}

/// Whether `i` is a small integer's value: [−2^30, 2^30 − 1].
pub const fn is_smi_range(i: i64) -> bool {
    i >= -(1 << 30) && i < (1 << 30)
}

/// The canonical Number for `x`: a small integer when `x` is integral, in
/// range and not −0, else a new `HeapNumber` (NaN canonicalized). A GC
/// point when it allocates.
///
/// # Errors
/// [`OutOfMemory`] if the cage is full.
pub fn number<'h, 's>(
    m: &mut Mutator<'h>,
    s: &Scope<'h, 's>,
    x: f64,
) -> Result<Root<'h, 's, Any>, OutOfMemory> {
    if let Some(i) = small_of(x) {
        return Ok(s.root_value(m, Value::Small(i)));
    }
    let h = m.alloc::<HeapNumber>(s, 0)?;
    let x = if x.is_nan() { f64::NAN } else { x };
    h.get(m).set_raw(m, HeapNumber::VALUE, x);
    let g = h.get(m).erase();
    Ok(s.root(m, g))
}

/// `x` as a small integer, if the canonical form makes it one.
pub fn small_of(x: f64) -> Option<i32> {
    if x.is_nan() || x.is_infinite() || x.trunc() != x || (x == 0.0 && x.is_sign_negative()) {
        return None;
    }
    if !(-1_073_741_824.0..=1_073_741_823.0).contains(&x) {
        return None;
    }
    // Integral and in [−2^30, 2^30 − 1], so the conversion is exact.
    let i = x as i32;
    Some(i)
}

/// SameValue by identity: two small integers with the same value, or two
/// references to the same object.
pub fn same(a: Value<'_, '_>, b: Value<'_, '_>) -> bool {
    match (a, b) {
        (Value::Small(x), Value::Small(y)) => x == y,
        (Value::Ref(x), Value::Ref(y)) => x.slot() == y.slot(),
        _ => false,
    }
}

/// `HeapNumber.value`.
pub fn heap_number_value<'h, 'm>(m: &'m Mutator<'h>, h: Ref<'h, 'm, kind::HeapNumber>) -> f64 {
    h.get::<HeapNumber>(m)
        .map_or(f64::NAN, |g| g.raw(m, HeapNumber::VALUE))
}

/// `String.length`: the code units of a flat string or a rope.
pub fn string_length<'h, 'm>(m: &'m Mutator<'h>, s: Ref<'h, 'm, kind::String>) -> u32 {
    if let Some(f) = s.get::<FlatString>(m) {
        return f.len(m) / 2;
    }
    s.get::<Rope>(m).map_or(0, |r| r.raw(m, Rope::LENGTH))
}

/// `BigInt.is_zero`.
pub fn big_int_is_zero<'h, 'm>(m: &'m Mutator<'h>, n: Ref<'h, 'm, kind::BigInt>) -> bool {
    n.get::<BigIntObj>(m)
        .is_none_or(|g| g.raw(m, BigIntObj::SIGN) == 0)
}

/// `Object.kind`.
pub fn object_kind<'h, 'm>(m: &'m Mutator<'h>, o: Ref<'h, 'm, kind::Object>) -> ObjectKind {
    if o.get::<WeakRefObj>(m).is_some() {
        ObjectKind::WeakRef
    } else if o.get::<FinRegObj>(m).is_some() {
        ObjectKind::FinReg
    } else if o.get::<HostFnObj>(m).is_some() {
        ObjectKind::HostFn
    } else {
        ObjectKind::Dict
    }
}

/// `Object.is_callable`: a host function, at JS-M0.
pub fn object_is_callable<'h, 'm>(m: &'m Mutator<'h>, o: Ref<'h, 'm, kind::Object>) -> bool {
    o.get::<HostFnObj>(m).is_some()
}

/// `Object.is_htmldda` (Annex B.3.6).
pub fn object_is_htmldda<'h, 'm>(m: &'m Mutator<'h>, o: Ref<'h, 'm, kind::Object>) -> bool {
    o.get::<Dict>(m)
        .is_some_and(|d| d.raw(m, Dict::FLAGS) & DICT_HTMLDDA != 0)
}

/// `WeakRef.target`: `None` once a canonical cycle has cleared it.
pub fn weak_ref_target<'h, 'm>(
    m: &'m Mutator<'h>,
    w: Ref<'h, 'm, kind::WeakRef>,
) -> Option<Value<'h, 'm>> {
    let g = w.get::<WeakRefObj>(m)?;
    g.get_weak(m, WeakRefObj::TARGET).map(Value::Ref)
}

/// `HostFn.id`.
pub fn host_fn_id<'h, 'm>(m: &'m Mutator<'h>, f: Ref<'h, 'm, kind::HostFn>) -> i32 {
    f.get::<HostFnObj>(m).map_or(0, |g| g.raw(m, HostFnObj::ID))
}

/// `kept.push`: AddToKeptObjects (ECMA-262 9.10.3). Not a GC point.
pub fn kept_push<'h>(m: &Mutator<'h>, v: Value<'h, '_>) {
    if let Value::Ref(g) = v {
        m.keep_alive(g);
    }
}

/// Allocates a rope over `a` and `b` of `length` code units.
///
/// # Errors
/// [`OutOfMemory`] if the cage is full.
pub fn alloc_rope<'h, 's>(
    m: &mut Mutator<'h>,
    s: &Scope<'h, 's>,
    _realm: &Realm<'h, '_>,
    a: Rooted<'h, 's, kind::String>,
    b: Rooted<'h, 's, kind::String>,
    length: u32,
) -> Result<Rooted<'h, 's, kind::String>, OutOfMemory> {
    let r = m.alloc::<Rope>(s, 0)?;
    let g = r.get(m);
    g.set_value(m, Rope::LEFT, a.get(m).value());
    g.set_value(m, Rope::RIGHT, b.get(m).value());
    g.set_raw(m, Rope::LENGTH, length);
    Ok(Rooted::new(s.root(m, g.erase())))
}

/// Allocates a WeakRef to `target` (an object or an unregistered symbol).
///
/// # Errors
/// [`OutOfMemory`] if the cage is full.
pub fn alloc_weak_ref<'h, 's>(
    m: &mut Mutator<'h>,
    s: &Scope<'h, 's>,
    _realm: &Realm<'h, '_>,
    target: Root<'h, 's, Any>,
) -> Result<Rooted<'h, 's, kind::WeakRef>, OutOfMemory> {
    let w = m.alloc::<WeakRefObj>(s, 0)?;
    let g = w.get(m);
    if let Value::Ref(t) = target.get_value(m) {
        g.set_weak(m, WeakRefObj::TARGET, Some(t));
    }
    Ok(Rooted::new(s.root(m, g.erase())))
}

/// Allocates a FinalizationRegistry with `cleanup` and its host-defined
/// value, and no cells.
///
/// # Errors
/// [`OutOfMemory`] if the cage is full.
pub fn alloc_fin_reg<'h, 's>(
    m: &mut Mutator<'h>,
    s: &Scope<'h, 's>,
    realm: &Realm<'h, '_>,
    cleanup: Root<'h, 's, Any>,
    host_defined: i64,
) -> Result<Rooted<'h, 's, kind::FinReg>, OutOfMemory> {
    let r = m.alloc::<FinRegObj>(s, 0)?;
    let g = r.get(m);
    g.set_value(m, FinRegObj::CLEANUP, cleanup.get_value(m));
    g.set_value(m, FinRegObj::CELLS, realm.undefined(m));
    g.set_raw(m, FinRegObj::HOST_DEFINED, host_defined.cast_unsigned());
    Ok(Rooted::new(s.root(m, g.erase())))
}

/// Allocates a host function named `id`.
///
/// # Errors
/// [`OutOfMemory`] if the cage is full.
pub fn alloc_host_fn<'h, 's>(
    m: &mut Mutator<'h>,
    s: &Scope<'h, 's>,
    _realm: &Realm<'h, '_>,
    id: i32,
) -> Result<Rooted<'h, 's, kind::HostFn>, OutOfMemory> {
    let f = m.alloc::<HostFnObj>(s, 0)?;
    let g = f.get(m);
    g.set_raw(m, HostFnObj::ID, id);
    Ok(Rooted::new(s.root(m, g.erase())))
}

// --- Strings, for the runtime -----------------------------------------------

/// Allocates a flat string of `units`.
///
/// # Errors
/// [`OutOfMemory`] if the cage is full or the string too long.
pub fn alloc_flat<'h, 's>(
    m: &mut Mutator<'h>,
    s: &Scope<'h, 's>,
    units: &[u16],
) -> Result<Rooted<'h, 's, kind::String>, OutOfMemory> {
    let bytes = units
        .len()
        .checked_mul(2)
        .and_then(|b| u32::try_from(b).ok())
        .ok_or(OutOfMemory)?;
    let f = m.alloc::<FlatString>(s, bytes)?;
    let g = f.get(m);
    for (i, u) in (0u32..).zip(units) {
        let [lo, hi] = u.to_le_bytes();
        g.set_byte(m, 2 * i, lo);
        g.set_byte(m, 2 * i + 1, hi);
    }
    Ok(Rooted::new(s.root(m, g.erase())))
}

/// The code units of string `v` (flat or rope; ropes are walked with an
/// explicit stack, so a deep rope cannot overflow the native stack).
/// Not a GC point.
pub fn string_units<'h>(m: &Mutator<'h>, v: Value<'h, '_>) -> Vec<u16> {
    let mut out = Vec::new();
    let mut stack = vec![v];
    while let Some(v) = stack.pop() {
        let Value::Ref(g) = v else { continue };
        if let Some(f) = g.cast::<FlatString>(m) {
            let n = f.len(m) / 2;
            out.reserve(usize::try_from(n).unwrap_or(0));
            for i in 0..n {
                let lo = f.byte(m, 2 * i).unwrap_or(0);
                let hi = f.byte(m, 2 * i + 1).unwrap_or(0);
                out.push(u16::from_le_bytes([lo, hi]));
            }
        } else if let Some(r) = g.cast::<Rope>(m) {
            stack.push(r.get_value(m, Rope::RIGHT));
            stack.push(r.get_value(m, Rope::LEFT));
        }
    }
    out
}

/// Whether `v` is a String.
pub fn is_string<'h>(m: &Mutator<'h>, v: Value<'h, '_>) -> bool {
    matches!(v, Value::Ref(g) if g.cast::<FlatString>(m).is_some() || g.cast::<Rope>(m).is_some())
}
