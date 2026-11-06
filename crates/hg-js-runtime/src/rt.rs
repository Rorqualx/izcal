//! The generated `Runtime` trait's methods: the generic paths of the
//! operators (ECMA-262 13.15.3 ApplyStringOrNumericBinaryOperator, 7.2.13
//! IsLessThan, 7.2.14 IsLooselyEqual), property access and the
//! intrinsics' runtime steps.

use hg_heap::{Any, Mutator, Root, Scope, Value};
use hg_js_ops::Runtime;
use hg_js_ops::generated::defs::{consts, units};
use hg_js_ops::heap::{self, Class, Dict, Ref, Rooted, kind};
use hg_js_ops::support::{
    CallKind, ErrorKind, Host, OutOfMemory, Pacer, RuntimeBase, ScriptToken, Throw,
};
use hg_substrate::RandomStream;

use crate::conv::{Hint, R};
use crate::{AgentRoots, JsRuntime};

/// The result of IsLessThan: true, false or undefined (a NaN operand).
fn less_than(a: f64, b: f64) -> Option<bool> {
    if a.is_nan() || b.is_nan() {
        None
    } else {
        Some(a < b)
    }
}

/// Code-unit order of two strings (IsLessThan step 3).
fn units_less(a: &[u16], b: &[u16]) -> bool {
    a < b
}

/// The Number operators of ECMA-262 6.1.6.1 for `op` (a `OP_*` tag).
fn number_op(op: i32, a: f64, b: f64) -> f64 {
    use hg_js_ops::support::prim::{sar, shl, shr, to_int32, to_uint32};
    match op {
        consts::OP_SUB => a - b,
        consts::OP_MUL => a * b,
        consts::OP_DIV => a / b,
        consts::OP_MOD => hg_js_number::remainder(a, b),
        consts::OP_EXP => hg_js_number::exponentiate(a, b),
        consts::OP_BIT_AND => f64::from(to_int32(a) & to_int32(b)),
        consts::OP_BIT_OR => f64::from(to_int32(a) | to_int32(b)),
        consts::OP_BIT_XOR => f64::from(to_int32(a) ^ to_int32(b)),
        consts::OP_SHL => f64::from(shl(to_int32(a), to_uint32(b))),
        consts::OP_SAR => f64::from(sar(to_int32(a), to_uint32(b))),
        consts::OP_SHR => f64::from(shr(to_uint32(a), to_uint32(b))),
        _ => f64::NAN,
    }
}

/// The type of a value for IsLooselyEqual's dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ty {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Symbol,
    BigInt,
    Object,
}

fn ty(c: Class<'_, '_>) -> Ty {
    match c {
        Class::Undefined => Ty::Undefined,
        Class::Null => Ty::Null,
        Class::True | Class::False => Ty::Boolean,
        Class::Smi(_) | Class::HeapNumber(_) => Ty::Number,
        Class::String(_) => Ty::String,
        Class::Symbol(_) => Ty::Symbol,
        Class::BigInt(_) => Ty::BigInt,
        Class::Dict(_) | Class::WeakRef(_) | Class::FinReg(_) | Class::HostFn(_) => Ty::Object,
    }
}

impl<'h> JsRuntime<'h, '_> {
    fn boolean<'s>(&self, m: &Mutator<'h>, s: &Scope<'h, 's>, b: bool) -> Root<'h, 's, Any> {
        s.root_value(m, self.realm.boolean(m, b))
    }

    /// IsStrictlyEqual (ECMA-262 7.2.15).
    fn strictly_equal(&self, m: &Mutator<'h>, x: Value<'h, '_>, y: Value<'h, '_>) -> bool {
        let (cx, cy) = (heap::classify(m, x), heap::classify(m, y));
        match (ty(cx), ty(cy)) {
            (Ty::Number, Ty::Number) => cx.number(m) == cy.number(m),
            (Ty::String, Ty::String) => heap::string_units(m, x) == heap::string_units(m, y),
            (Ty::BigInt, Ty::BigInt) => match (cx, cy) {
                (Class::BigInt(a), Class::BigInt(b)) => self.rt_bigint_equal(m, a, b),
                _ => false,
            },
            _ => heap::same(x, y),
        }
    }

    /// IsLessThan (ECMA-262 7.2.13).
    fn is_less_than<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        x: Root<'h, 's, Any>,
        y: Root<'h, 's, Any>,
        left_first: bool,
    ) -> R<'h, 's, Option<bool>> {
        let (px, py) = if left_first {
            let px = self.prim_of(m, t, s, x, Hint::Number)?;
            let py = self.prim_of(m, t, s, y, Hint::Number)?;
            (px, py)
        } else {
            let py = self.prim_of(m, t, s, y, Hint::Number)?;
            let px = self.prim_of(m, t, s, x, Hint::Number)?;
            (px, py)
        };
        let (a, b) = (px.get_value(m), py.get_value(m));
        if heap::is_string(m, a) && heap::is_string(m, b) {
            return Ok(Some(units_less(
                &heap::string_units(m, a),
                &heap::string_units(m, b),
            )));
        }
        let nx = self.numeric_of(m, t, s, px)?;
        let ny = self.numeric_of(m, t, s, py)?;
        let (cx, cy) = (
            heap::classify(m, nx.get_value(m)),
            heap::classify(m, ny.get_value(m)),
        );
        if matches!(cx, Class::BigInt(_)) || matches!(cy, Class::BigInt(_)) {
            // No BigInt value exists at JS-M0 (no BigInt constants).
            return self.throw(m, ErrorKind::TypeError);
        }
        Ok(less_than(cx.number(m), cy.number(m)))
    }
}

impl<'h> RuntimeBase<'h> for JsRuntime<'h, '_> {
    fn tail_call<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        kind: CallKind,
        callee: Root<'h, 's, Any>,
        this: Root<'h, 's, Any>,
        args: &[Root<'h, 's, Any>],
    ) -> Result<Root<'h, 's, Any>, Throw<'h, 's>> {
        match kind {
            CallKind::Call => self.call(m, t, s, callee, this, args),
            // JS-M0 has no constructors, accessors or iterators.
            CallKind::Construct | CallKind::Getter | CallKind::Setter | CallKind::IterStep => {
                self.throw(m, ErrorKind::TypeError)
            }
        }
    }

    fn pacer(&mut self) -> &mut Pacer {
        &mut self.pacer
    }

    fn on_canonical_cycle(&mut self, m: &mut Mutator<'h>, host: &mut dyn Host<'h>) {
        self.after_cycle(m, host);
    }
}

impl<'h> Runtime<'h> for JsRuntime<'h, '_> {
    fn rt_add<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        x: Root<'h, 's, Any>,
        y: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        // ApplyStringOrNumericBinaryOperator, step 1 (opText `+`).
        let lp = self.prim_of(m, t, s, x, Hint::Default)?;
        let rp = self.prim_of(m, t, s, y, Hint::Default)?;
        if heap::is_string(m, lp.get_value(m)) || heap::is_string(m, rp.get_value(m)) {
            let ls = self.string_of(m, t, s, lp)?;
            let rs = self.string_of(m, t, s, rp)?;
            return Ok(self.concat(m, s, ls, rs)?.erase());
        }
        self.rt_numeric_add(m, t, s, lp, rp)
    }

    fn rt_numeric<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        op: i32,
        x: Root<'h, 's, Any>,
        y: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        // Steps 3-8: ToNumeric of each operand, left first.
        let ln = self.numeric_of(m, t, s, x)?;
        let rn = self.numeric_of(m, t, s, y)?;
        let (a, b) = (
            heap::classify(m, ln.get_value(m)),
            heap::classify(m, rn.get_value(m)),
        );
        if matches!(a, Class::BigInt(_)) || matches!(b, Class::BigInt(_)) {
            return self.throw(m, ErrorKind::TypeError);
        }
        let r = number_op(op, a.number(m), b.number(m));
        Ok(heap::number(m, s, r)?)
    }

    fn rt_unary<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        op: i32,
        x: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        let n = self.numeric_of(m, t, s, x)?;
        let c = heap::classify(m, n.get_value(m));
        if matches!(c, Class::BigInt(_)) {
            return self.throw(m, ErrorKind::TypeError);
        }
        let v = c.number(m);
        let r = match op {
            consts::UN_NEG => -v,
            consts::UN_BIT_NOT => f64::from(!hg_js_number::to_int32(v)),
            consts::UN_INC => v + 1.0,
            consts::UN_DEC => v - 1.0,
            _ => f64::NAN,
        };
        Ok(heap::number(m, s, r)?)
    }

    fn rt_exponentiate<'m>(&self, _m: &'m Mutator<'h>, base: f64, exponent: f64) -> f64 {
        hg_js_number::exponentiate(base, exponent)
    }

    fn rt_compare<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        op: i32,
        x: Root<'h, 's, Any>,
        y: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        // ECMA-262 13.10.1: `<`, `>`, `<=`, `>=`.
        let b = match op {
            consts::CMP_LT => self.is_less_than(m, t, s, x, y, true)? == Some(true),
            consts::CMP_GT => self.is_less_than(m, t, s, y, x, false)? == Some(true),
            consts::CMP_LE => self.is_less_than(m, t, s, y, x, false)? == Some(false),
            consts::CMP_GE => self.is_less_than(m, t, s, x, y, true)? == Some(false),
            _ => false,
        };
        Ok(self.boolean(m, s, b))
    }

    fn rt_loose_equal<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        x: Root<'h, 's, Any>,
        y: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        let (mut x, mut y) = (x, y);
        // Each conversion strictly lowers a type, so this ends in a few
        // rounds; the bound guards a corrupted heap.
        for _ in 0..8 {
            let (cx, cy) = (
                heap::classify(m, x.get_value(m)),
                heap::classify(m, y.get_value(m)),
            );
            let (tx, ty_) = (ty(cx), ty(cy));
            // Step 1.
            if tx == ty_ {
                let b = self.strictly_equal(m, x.get_value(m), y.get_value(m));
                return Ok(self.boolean(m, s, b));
            }
            let nullish = |t: Ty| matches!(t, Ty::Undefined | Ty::Null);
            // Steps 2-3.
            if nullish(tx) && nullish(ty_) {
                return Ok(self.boolean(m, s, true));
            }
            // Step 4 (Annex B.3.6.2): an [[IsHTMLDDA]] object equals undefined and null.
            if (tx == Ty::Object && nullish(ty_) && heap::object_is_htmldda(m, cx.object()))
                || (ty_ == Ty::Object && nullish(tx) && heap::object_is_htmldda(m, cy.object()))
            {
                return Ok(self.boolean(m, s, true));
            }
            match (tx, ty_) {
                // Steps 5-6: Number and String.
                (Ty::Number, Ty::String) => {
                    let n = self.number_of(m, t, s, y)?;
                    y = heap::number(m, s, n)?;
                }
                (Ty::String, Ty::Number) => {
                    let n = self.number_of(m, t, s, x)?;
                    x = heap::number(m, s, n)?;
                }
                // Steps 9-10: a Boolean becomes a Number.
                (Ty::Boolean, _) => {
                    let n = self.number_of(m, t, s, x)?;
                    x = heap::number(m, s, n)?;
                }
                (_, Ty::Boolean) => {
                    let n = self.number_of(m, t, s, y)?;
                    y = heap::number(m, s, n)?;
                }
                // Steps 11-12: a primitive and an Object.
                (Ty::String | Ty::Number | Ty::BigInt | Ty::Symbol, Ty::Object) => {
                    y = self.prim_of(m, t, s, y, Hint::Default)?;
                }
                (Ty::Object, Ty::String | Ty::Number | Ty::BigInt | Ty::Symbol) => {
                    x = self.prim_of(m, t, s, x, Hint::Default)?;
                }
                // Steps 7-8 and 13 concern BigInts, which JS-M0 cannot
                // make; step 14.
                _ => return Ok(self.boolean(m, s, false)),
            }
        }
        Ok(self.boolean(m, s, false))
    }

    fn rt_to_number<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        x: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        let n = self.number_of(m, t, s, x)?;
        Ok(heap::number(m, s, n)?)
    }

    fn rt_to_numeric<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        x: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        self.numeric_of(m, t, s, x)
    }

    fn rt_to_string<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        x: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        Ok(self.string_of(m, t, s, x)?.erase())
    }

    fn rt_number_to_string<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        x: f64,
    ) -> Result<Rooted<'h, 's, kind::String>, OutOfMemory> {
        let u: Vec<u16> = hg_js_number::number_to_string(x).encode_utf16().collect();
        self.charge(
            m,
            units::string_units(i64::try_from(u.len()).unwrap_or(i64::MAX)),
        );
        heap::alloc_flat(m, s, &u)
    }

    fn rt_typeof_name<'m>(&self, m: &'m Mutator<'h>, tag: i32) -> Ref<'h, 'm, kind::String> {
        let v = u32::try_from(tag)
            .ok()
            .and_then(|i| self.array(m, AgentRoots::NAMES)?.tail(m, i))
            .unwrap_or(Value::Small(0));
        Ref::new(v)
    }

    fn rt_string_equal<'m>(
        &self,
        m: &'m Mutator<'h>,
        a: Ref<'h, 'm, kind::String>,
        b: Ref<'h, 'm, kind::String>,
    ) -> bool {
        heap::string_units(m, a.value()) == heap::string_units(m, b.value())
    }

    fn rt_string_less_than<'m>(
        &self,
        m: &'m Mutator<'h>,
        a: Ref<'h, 'm, kind::String>,
        b: Ref<'h, 'm, kind::String>,
    ) -> bool {
        units_less(
            &heap::string_units(m, a.value()),
            &heap::string_units(m, b.value()),
        )
    }

    fn rt_string_flat_concat<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        a: Rooted<'h, 's, kind::String>,
        b: Rooted<'h, 's, kind::String>,
        _length: i32,
    ) -> Result<Rooted<'h, 's, kind::String>, OutOfMemory> {
        let mut u = heap::string_units(m, a.get(m).value());
        u.extend(heap::string_units(m, b.get(m).value()));
        heap::alloc_flat(m, s, &u)
    }

    fn rt_bigint_equal<'m>(
        &self,
        m: &'m Mutator<'h>,
        a: Ref<'h, 'm, kind::BigInt>,
        b: Ref<'h, 'm, kind::BigInt>,
    ) -> bool {
        // Sign and magnitude bytes; JS-M0 makes no BigInts, so this is
        // identity in practice.
        heap::same(a.value(), b.value())
            || (heap::big_int_is_zero(m, a) && heap::big_int_is_zero(m, b))
    }

    fn rt_symbol_registered<'m>(&self, m: &'m Mutator<'h>, sym: Ref<'h, 'm, kind::Symbol>) -> bool {
        match sym.value() {
            Value::Ref(g) => g
                .cast::<heap::SymbolObj>(m)
                .is_some_and(|g| g.raw(m, heap::SymbolObj::REGISTERED)),
            Value::Small(_) => false,
        }
    }

    fn rt_ordinary_get<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        _t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        o: Rooted<'h, 's, kind::Object>,
        k: Rooted<'h, 's, kind::Key>,
        _receiver: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        self.ordinary_get(m, s, o.erase(), k.erase())
    }

    fn rt_get<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        base: Root<'h, 's, Any>,
        k: Rooted<'h, 's, kind::Key>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        self.get_of(m, t, s, base, k.erase())
    }

    fn rt_get_keyed<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        base: Root<'h, 's, Any>,
        key: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        // GetValue step 3.c: ToPropertyKey, then [[Get]].
        let k = self.key_of(m, t, s, key)?;
        self.get_of(m, t, s, base, k)
    }

    fn rt_ordinary_set<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        _t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        _o: Rooted<'h, 's, kind::Object>,
        k: Rooted<'h, 's, kind::Key>,
        v: Root<'h, 's, Any>,
        receiver: Root<'h, 's, Any>,
        strict: bool,
    ) -> R<'h, 's, bool> {
        let ok = self.ordinary_set(m, s, k.erase(), v, receiver)?;
        if !ok && strict {
            return self.throw(m, ErrorKind::TypeError);
        }
        Ok(ok)
    }

    fn rt_set<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        _t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        base: Root<'h, 's, Any>,
        k: Rooted<'h, 's, kind::Key>,
        v: Root<'h, 's, Any>,
        strict: bool,
    ) -> R<'h, 's, bool> {
        self.set_of(m, s, base, k.erase(), v, strict)
    }

    fn rt_set_keyed<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        base: Root<'h, 's, Any>,
        key: Root<'h, 's, Any>,
        v: Root<'h, 's, Any>,
        strict: bool,
    ) -> R<'h, 's, bool> {
        // PutValue step 3.c: ToPropertyKey, then [[Set]].
        let k = self.key_of(m, t, s, key)?;
        self.set_of(m, s, base, k, v, strict)
    }

    fn rt_dict_new<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        proto: Root<'h, 's, Any>,
    ) -> Result<Rooted<'h, 's, kind::Object>, OutOfMemory> {
        let d = m.alloc::<Dict>(s, 0)?;
        let g = d.get(m);
        g.set_value(m, Dict::PROTO, proto.get_value(m));
        g.set_value(m, Dict::PROPS, Value::Small(0));
        Ok(Rooted::new(s.root(m, g.erase())))
    }

    fn rt_finreg_register<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        r: Rooted<'h, 's, kind::FinReg>,
        target: Root<'h, 's, Any>,
        held: Root<'h, 's, Any>,
        token: Root<'h, 's, Any>,
        has_token: bool,
    ) -> Result<bool, OutOfMemory> {
        self.register(m, s, r.erase(), target, held, token, has_token)
    }

    fn rt_finreg_unregister<'m>(
        &mut self,
        m: &'m Mutator<'h>,
        r: Ref<'h, 'm, kind::FinReg>,
        token: Value<'h, 'm>,
    ) -> bool {
        self.unregister(m, r.value(), token)
    }

    fn rt_math_random<'m>(&mut self, _m: &'m Mutator<'h>, random: &mut dyn RandomStream) -> f64 {
        // The 53 high bits of the next draw, times 2^-53: uniform on [0, 1).
        let bits = random.next_u64() >> 11;
        // bits < 2^53, so the conversion is exact.
        #[allow(clippy::cast_precision_loss)]
        let x = bits as f64;
        x * (1.0 / 9_007_199_254_740_992.0)
    }

    fn rt_date_now<'m>(&mut self, _m: &'m Mutator<'h>, host: &mut dyn Host<'h>) -> f64 {
        // Date.now: floor(epoch nanoseconds / 10^6), as a time value.
        let ns = host.system_utc_epoch_nanoseconds();
        let ms = ns.div_euclid(1_000_000);
        // A time value is within ±8.64 × 10^15 ms; clamp a host's
        // out-of-range clock rather than convert inexactly.
        let ms = ms.clamp(-8_640_000_000_000_000, 8_640_000_000_000_000);
        #[allow(clippy::cast_precision_loss)]
        let x = i64::try_from(ms).unwrap_or(0) as f64;
        x
    }

    fn rt_make_job_callback<'m>(
        &mut self,
        _m: &'m Mutator<'h>,
        host: &mut dyn Host<'h>,
        _callback: Ref<'h, 'm, kind::Object>,
    ) -> i64 {
        host.make_job_callback()
    }
}

impl<'h> JsRuntime<'h, '_> {
    /// `+` on two Numeric primitives (step 2 of the `+` case onward).
    fn rt_numeric_add<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        lp: Root<'h, 's, Any>,
        rp: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        let ln = self.numeric_of(m, t, s, lp)?;
        let rn = self.numeric_of(m, t, s, rp)?;
        let (a, b) = (
            heap::classify(m, ln.get_value(m)),
            heap::classify(m, rn.get_value(m)),
        );
        if matches!(a, Class::BigInt(_)) || matches!(b, Class::BigInt(_)) {
            return self.throw(m, ErrorKind::TypeError);
        }
        Ok(heap::number(m, s, a.number(m) + b.number(m))?)
    }
}
