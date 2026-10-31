//! The type conversions (ECMA-262 7.1) and the host value bridge. Every
//! heap value that lives across a GC point is a `Root`.

use hg_heap::{Any, Mutator, Root, Scope, Value};
use hg_js_ops::generated::defs::{consts, units};
use hg_js_ops::heap::{self, Class, Rooted, kind};
use hg_js_ops::support::{ErrorKind, HostValue, OutOfMemory, ScriptToken, Throw};

use crate::JsRuntime;

/// The result of a step that may throw.
pub(crate) type R<'h, 's, T> = Result<T, Throw<'h, 's>>;

/// ToPrimitive's preferred type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hint {
    /// No preference (`+`, `==`).
    Default,
    /// Number.
    Number,
    /// String.
    String,
}

/// Whether `c` is an Object.
pub(crate) fn is_object(c: Class<'_, '_>) -> bool {
    matches!(
        c,
        Class::Dict(_) | Class::WeakRef(_) | Class::FinReg(_) | Class::HostFn(_)
    )
}

fn units_of(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

impl<'h> JsRuntime<'h, '_> {
    /// Charges `units` canonical units through the agent's clock.
    pub(crate) fn charge(&mut self, m: &mut Mutator<'h>, units: u64) {
        self.pacer.charge_at(m, units);
    }

    /// Throws a new error of kind `k` (charging its units).
    pub(crate) fn throw<'s, T>(&mut self, m: &mut Mutator<'h>, k: ErrorKind) -> R<'h, 's, T> {
        self.charge(m, units::error_units());
        Err(Throw::Error(k))
    }

    /// A new string of `units`, charged canonically; RangeError past the
    /// canonical length limit.
    pub(crate) fn new_string<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        units: &[u16],
    ) -> R<'h, 's, Rooted<'h, 's, kind::String>> {
        let n = i64::try_from(units.len()).unwrap_or(i64::MAX);
        if n > consts::MAX_STRING_LENGTH {
            return self.throw(m, ErrorKind::RangeError);
        }
        self.charge(m, units::string_units(n));
        Ok(heap::alloc_flat(m, s, units)?)
    }

    /// A string the runtime uses internally (a property name it looks up):
    /// not charged.
    pub(crate) fn internal_string<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        text: &str,
    ) -> Result<Root<'h, 's, Any>, OutOfMemory> {
        Ok(heap::alloc_flat(m, s, &units_of(text))?.erase())
    }

    /// The concatenation of strings `a` and `b` (ECMA-262
    /// [string-concatenation](https://tc39.es/ecma262/#string-concatenation)),
    /// charged as the `StringConcat` description charges it.
    pub(crate) fn concat<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        a: Rooted<'h, 's, kind::String>,
        b: Rooted<'h, 's, kind::String>,
    ) -> R<'h, 's, Rooted<'h, 's, kind::String>> {
        let la = heap::string_length(m, a.get(m));
        let lb = heap::string_length(m, b.get(m));
        let n = i64::from(la) + i64::from(lb);
        if n > consts::MAX_STRING_LENGTH {
            return self.throw(m, ErrorKind::RangeError);
        }
        self.charge(m, units::string_units(n));
        if la == 0 {
            return Ok(b);
        }
        if lb == 0 {
            return Ok(a);
        }
        let len = u32::try_from(n).unwrap_or(u32::MAX);
        if n < consts::ROPE_MIN_LENGTH {
            let mut u = heap::string_units(m, a.get(m).value());
            u.extend(heap::string_units(m, b.get(m).value()));
            return Ok(heap::alloc_flat(m, s, &u)?);
        }
        let realm = self.realm;
        Ok(heap::alloc_rope(m, s, &realm, a, b, len)?)
    }

    /// ToPrimitive (ECMA-262 7.1.1). JS-M0 has no well-known symbols, so
    /// `@@toPrimitive` is never found and OrdinaryToPrimitive decides.
    pub(crate) fn prim_of<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        v: Root<'h, 's, Any>,
        hint: Hint,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        if !is_object(heap::classify(m, v.get_value(m))) {
            return Ok(v);
        }
        // OrdinaryToPrimitive (7.1.1.1).
        let names = if hint == Hint::String {
            ["toString", "valueOf"]
        } else {
            ["valueOf", "toString"]
        };
        for name in names {
            let key = self.internal_string(m, s, name)?;
            let f = self.get_of(m, t, s, v, key)?;
            if self.is_callable(m, f.get_value(m)) {
                let r = self.call(m, t, s, f, v, &[])?;
                if !is_object(heap::classify(m, r.get_value(m))) {
                    return Ok(r);
                }
            }
        }
        self.throw(m, ErrorKind::TypeError)
    }

    /// IsCallable: a host function, at JS-M0.
    pub(crate) fn is_callable(&self, m: &Mutator<'h>, v: Value<'h, '_>) -> bool {
        matches!(heap::classify(m, v), Class::HostFn(_))
    }

    /// Call (ECMA-262 7.3.14) of a host function.
    pub(crate) fn call<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        _t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        f: Root<'h, 's, Any>,
        this: Root<'h, 's, Any>,
        args: &[Root<'h, 's, Any>],
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        let id = match heap::classify(m, f.get_value(m)) {
            Class::HostFn(h) => heap::host_fn_id(m, h),
            _ => return self.throw(m, ErrorKind::TypeError),
        };
        let this_v = self.host_view(m, this.get_value(m));
        let arg_v: Vec<HostValue> = args
            .iter()
            .map(|a| self.host_view(m, a.get_value(m)))
            .collect();
        match self.functions.call(id, &this_v, &arg_v) {
            Ok(r) => Ok(self.value_from_host(m, s, &r)?),
            Err(e) => {
                let v = self.value_from_host(m, s, &e)?;
                Err(Throw::Value(v))
            }
        }
    }

    /// ToNumber (ECMA-262 7.1.4).
    pub(crate) fn number_of<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        v: Root<'h, 's, Any>,
    ) -> R<'h, 's, f64> {
        let c = heap::classify(m, v.get_value(m));
        match c {
            Class::Undefined => Ok(f64::NAN),
            Class::Null | Class::False => Ok(0.0),
            Class::True => Ok(1.0),
            Class::Smi(_) | Class::HeapNumber(_) => Ok(c.number(m)),
            Class::String(st) => {
                let u = heap::string_units(m, st.value());
                Ok(hg_js_number::string_to_number(&u))
            }
            Class::Symbol(_) | Class::BigInt(_) => self.throw(m, ErrorKind::TypeError),
            Class::Dict(_) | Class::WeakRef(_) | Class::FinReg(_) | Class::HostFn(_) => {
                let p = self.prim_of(m, t, s, v, Hint::Number)?;
                self.number_of(m, t, s, p)
            }
        }
    }

    /// ToNumeric (ECMA-262 7.1.3): a Number (canonical) or a BigInt.
    pub(crate) fn numeric_of<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        v: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        let p = self.prim_of(m, t, s, v, Hint::Number)?;
        match heap::classify(m, p.get_value(m)) {
            Class::BigInt(_) | Class::Smi(_) | Class::HeapNumber(_) => Ok(p),
            _ => {
                let x = self.number_of(m, t, s, p)?;
                Ok(heap::number(m, s, x)?)
            }
        }
    }

    /// ToString (ECMA-262 7.1.17).
    pub(crate) fn string_of<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        v: Root<'h, 's, Any>,
    ) -> R<'h, 's, Rooted<'h, 's, kind::String>> {
        let c = heap::classify(m, v.get_value(m));
        let text = match c {
            Class::String(_) => return Ok(Rooted::new(v)),
            Class::Undefined => "undefined".to_string(),
            Class::Null => "null".to_string(),
            Class::True => "true".to_string(),
            Class::False => "false".to_string(),
            Class::Smi(_) | Class::HeapNumber(_) => hg_js_number::number_to_string(c.number(m)),
            Class::Symbol(_) | Class::BigInt(_) => return self.throw(m, ErrorKind::TypeError),
            Class::Dict(_) | Class::WeakRef(_) | Class::FinReg(_) | Class::HostFn(_) => {
                let p = self.prim_of(m, t, s, v, Hint::String)?;
                return self.string_of(m, t, s, p);
            }
        };
        self.new_string(m, s, &units_of(&text))
    }

    /// ToPropertyKey (ECMA-262 7.1.19): a Symbol or a String.
    pub(crate) fn key_of<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        v: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        let p = self.prim_of(m, t, s, v, Hint::String)?;
        if matches!(heap::classify(m, p.get_value(m)), Class::Symbol(_)) {
            return Ok(p);
        }
        Ok(self.string_of(m, t, s, p)?.erase())
    }

    /// The host's view of `v`.
    pub(crate) fn host_view(&self, m: &Mutator<'h>, v: Value<'h, '_>) -> HostValue {
        let c = heap::classify(m, v);
        match c {
            Class::Undefined => HostValue::Undefined,
            Class::Null => HostValue::Null,
            Class::True => HostValue::Bool(true),
            Class::False => HostValue::Bool(false),
            Class::Smi(_) | Class::HeapNumber(_) => HostValue::Number(c.number(m)),
            Class::String(st) => HostValue::String(heap::string_units(m, st.value())),
            _ => HostValue::Object,
        }
    }

    /// A heap value for the host's `v` (an object becomes `undefined`:
    /// the host holds no objects, only [`Handle`]s).
    pub(crate) fn value_from_host<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        v: &HostValue,
    ) -> Result<Root<'h, 's, Any>, OutOfMemory> {
        let realm = self.realm;
        Ok(match v {
            HostValue::Undefined | HostValue::Object => s.root_value(m, realm.undefined(m)),
            HostValue::Null => s.root_value(m, realm.null(m)),
            HostValue::Bool(b) => s.root_value(m, realm.boolean(m, *b)),
            HostValue::Number(x) => heap::number(m, s, *x)?,
            HostValue::String(u) => heap::alloc_flat(m, s, u)?.erase(),
        })
    }
}
