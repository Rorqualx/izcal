//! Properties (ECMA-262 10.1): dictionary objects hold data properties in
//! a key/value table; every property is a writable, enumerable,
//! configurable data property (JS-M0 has no accessors and no
//! `defineProperty`), and every dictionary is extensible.

use hg_heap::{Any, Gc, Mutator, Root, Scope, Value};
use hg_js_ops::generated::defs::units;
use hg_js_ops::heap::{self, Class, Dict, PropTable};
use hg_js_ops::support::{ErrorKind, OutOfMemory, ScriptToken};

use crate::JsRuntime;
use crate::conv::R;

/// Whether property keys `a` and `b` are the same key: strings by their
/// code units, symbols by identity.
pub(crate) fn key_eq<'h>(m: &Mutator<'h>, a: Value<'h, '_>, b: Value<'h, '_>) -> bool {
    if heap::same(a, b) {
        return true;
    }
    heap::is_string(m, a)
        && heap::is_string(m, b)
        && heap::string_units(m, a) == heap::string_units(m, b)
}

fn dict<'h, 'm>(m: &'m Mutator<'h>, v: Value<'h, 'm>) -> Option<Gc<'h, 'm, Dict>> {
    match v {
        Value::Ref(g) => g.cast::<Dict>(m),
        Value::Small(_) => None,
    }
}

/// The table slot of `key` in dictionary `d`, if it has the property.
fn find<'h>(m: &Mutator<'h>, d: Gc<'h, '_, Dict>, key: Value<'h, '_>) -> Option<u32> {
    let Value::Ref(t) = d.get_value(m, Dict::PROPS) else {
        return None;
    };
    let t = t.cast::<PropTable>(m)?;
    let n = d.raw(m, Dict::COUNT).min(t.len(m) / 2);
    (0..n).find(|i| t.tail(m, 2 * i).is_some_and(|k| key_eq(m, k, key)))
}

/// The string exotic object's own property `key` of string `s`: `length`
/// or an index (ECMA-262 10.4.3.5).
fn string_own<'h>(m: &Mutator<'h>, s: Value<'h, '_>, key: Value<'h, '_>) -> Option<StringOwn> {
    if !heap::is_string(m, key) {
        return None;
    }
    let k = heap::string_units(m, key);
    let len = u32::try_from(heap::string_units(m, s).len()).unwrap_or(u32::MAX);
    if k == "length".encode_utf16().collect::<Vec<_>>() {
        return Some(StringOwn::Length(len));
    }
    // A canonical array index: digits, no leading zero (except "0").
    if k.is_empty() || k.len() > 10 || (k.len() > 1 && k[0] == u16::from(b'0')) {
        return None;
    }
    let mut i: u64 = 0;
    for u in &k {
        let d = u.checked_sub(u16::from(b'0')).filter(|d| *d <= 9)?;
        i = i * 10 + u64::from(d);
    }
    let i = u32::try_from(i).ok()?;
    (i < len).then_some(StringOwn::Index(i))
}

enum StringOwn {
    Length(u32),
    Index(u32),
}

impl<'h> JsRuntime<'h, '_> {
    /// OrdinaryGet (ECMA-262 10.1.8.1) along the prototype chain. JS-M0's
    /// properties are data properties, so no user code runs.
    pub(crate) fn ordinary_get<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        o: Root<'h, 's, Any>,
        key: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        let mut cur = o.get_value(m);
        let k = key.get_value(m);
        // Prototype chains are acyclic (no SetPrototypeOf at JS-M0); the
        // bound only guards a corrupted heap.
        for _ in 0..1_000_000 {
            let Some(d) = dict(m, cur) else { break };
            if let Some(i) = find(m, d, k) {
                let v = match d.get_value(m, Dict::PROPS) {
                    Value::Ref(t) => t.cast::<PropTable>(m).and_then(|t| t.tail(m, 2 * i + 1)),
                    Value::Small(_) => None,
                };
                let v = v.unwrap_or_else(|| self.realm.undefined(m));
                return Ok(s.root_value(m, v));
            }
            cur = d.get_value(m, Dict::PROTO);
        }
        let u = self.realm.undefined(m);
        Ok(s.root_value(m, u))
    }

    /// `base.[[Get]](key, base)` for any value `base` (GetV, ECMA-262
    /// 7.3.3): a primitive's wrapper has no prototype at JS-M0, so only a
    /// string's own properties are found.
    pub(crate) fn get_of<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        _t: &mut ScriptToken,
        s: &Scope<'h, 's>,
        base: Root<'h, 's, Any>,
        key: Root<'h, 's, Any>,
    ) -> R<'h, 's, Root<'h, 's, Any>> {
        let b = base.get_value(m);
        match heap::classify(m, b) {
            Class::Dict(_) => self.ordinary_get(m, s, base, key),
            Class::String(_) => match string_own(m, b, key.get_value(m)) {
                Some(StringOwn::Length(n)) => Ok(heap::number(m, s, f64::from(n))?),
                Some(StringOwn::Index(i)) => {
                    let u = heap::string_units(m, b);
                    let one = [u
                        .get(usize::try_from(i).unwrap_or(usize::MAX))
                        .copied()
                        .unwrap_or(0)];
                    // A one-code-unit string is internal: any tier may cache it.
                    Ok(heap::alloc_flat(m, s, &one)?.erase())
                }
                None => {
                    let u = self.realm.undefined(m);
                    Ok(s.root_value(m, u))
                }
            },
            _ => {
                let u = self.realm.undefined(m);
                Ok(s.root_value(m, u))
            }
        }
    }

    /// Adds data property `key: v` to dictionary `o` (charging
    /// `property_units`), growing its table.
    fn add_property<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        o: Root<'h, 's, Any>,
        key: Root<'h, 's, Any>,
        v: Root<'h, 's, Any>,
    ) -> Result<(), OutOfMemory> {
        self.charge(m, units::property_units());
        let (count, cap) = {
            let d = dict(m, o.get_value(m)).ok_or(OutOfMemory)?;
            let cap = match d.get_value(m, Dict::PROPS) {
                Value::Ref(t) => t.cast::<PropTable>(m).map_or(0, |t| t.len(m) / 2),
                Value::Small(_) => 0,
            };
            (d.raw(m, Dict::COUNT), cap)
        };
        if count >= cap {
            let new_cap = cap.checked_mul(2).ok_or(OutOfMemory)?.max(4);
            let bigger = m.alloc::<PropTable>(s, new_cap.checked_mul(2).ok_or(OutOfMemory)?)?;
            let d = dict(m, o.get_value(m)).ok_or(OutOfMemory)?;
            let b = bigger.get(m);
            if let Value::Ref(t) = d.get_value(m, Dict::PROPS)
                && let Some(t) = t.cast::<PropTable>(m)
            {
                for i in 0..2 * count {
                    if let Some(x) = t.tail(m, i) {
                        b.set_tail(m, i, x);
                    }
                }
            }
            d.set_value(m, Dict::PROPS, Value::Ref(b.erase()));
        }
        let d = dict(m, o.get_value(m)).ok_or(OutOfMemory)?;
        let Value::Ref(t) = d.get_value(m, Dict::PROPS) else {
            return Err(OutOfMemory);
        };
        let t = t.cast::<PropTable>(m).ok_or(OutOfMemory)?;
        t.set_tail(m, 2 * count, key.get_value(m));
        t.set_tail(m, 2 * count + 1, v.get_value(m));
        d.set_raw(m, Dict::COUNT, count + 1);
        Ok(())
    }

    /// OrdinarySet (ECMA-262 10.1.9.1) with OrdinarySetWithOwnDescriptor:
    /// every property is a writable data property, so the property is
    /// written on the receiver, added there if absent.
    pub(crate) fn ordinary_set<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        key: Root<'h, 's, Any>,
        v: Root<'h, 's, Any>,
        receiver: Root<'h, 's, Any>,
    ) -> R<'h, 's, bool> {
        // Step 2.c: the receiver must be an object that holds properties.
        let Some(d) = dict(m, receiver.get_value(m)) else {
            return Ok(false);
        };
        if let Some(i) = find(m, d, key.get_value(m)) {
            if let Value::Ref(t) = d.get_value(m, Dict::PROPS)
                && let Some(t) = t.cast::<PropTable>(m)
            {
                t.set_tail(m, 2 * i + 1, v.get_value(m));
            }
            return Ok(true);
        }
        // CreateDataProperty on an extensible receiver.
        self.add_property(m, s, receiver, key, v)?;
        Ok(true)
    }

    /// PutValue's `baseObj.[[Set]](key, v, base)` (ECMA-262 6.2.5.6 step
    /// 3.d) and its strict-mode TypeError.
    pub(crate) fn set_of<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        base: Root<'h, 's, Any>,
        key: Root<'h, 's, Any>,
        v: Root<'h, 's, Any>,
        strict: bool,
    ) -> R<'h, 's, bool> {
        let ok = match heap::classify(m, base.get_value(m)) {
            // A dictionary is its own receiver.
            Class::Dict(_) => self.ordinary_set(m, s, key, v, base)?,
            // A primitive's wrapper: the receiver is the primitive, not an
            // object, so OrdinarySetWithOwnDescriptor returns false; the
            // other JS-M0 objects hold no properties.
            _ => false,
        };
        if !ok && strict {
            return self.throw(m, ErrorKind::TypeError);
        }
        Ok(ok)
    }
}
