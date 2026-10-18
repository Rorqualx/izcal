//! Token-taking wrappers (criterion 2) and the token's V8 entry points
//! (criterion 1).
//!
//! Every rusty_v8 method that can run script is listed in `clippy.toml`
//! `disallowed-methods`; this module is the one place that `allow`s it.
#![allow(clippy::disallowed_methods)]

use token_cell::{Reentry, ScriptToken};
use v8::{Function, FunctionCallbackArguments, Local, Object, PinScope, ReturnValue, Value};

/// Criterion (1): the token is derivable only here (top of a task, from a
/// mutable handle scope) or from [`prologue`] (a callback, from the scope V8
/// passes in, and only while the outer token is lent).
pub fn enter<'s, 'i, R>(
    scope: &mut PinScope<'s, 'i>,
    f: impl for<'t> FnOnce(&mut PinScope<'s, 'i>, &'t mut ScriptToken) -> R,
) -> R {
    token_cell::enter(|tok| f(scope, tok))
}

/// The generated callback prologue. A binding's body gets a token for its own
/// frame; if the token was not lent (script reached V8 without a wrapper, or
/// the callback runs from GC), the call throws instead of minting.
pub fn prologue<'s, 'i>(
    scope: &mut PinScope<'s, 'i>,
    args: FunctionCallbackArguments<'s>,
    mut rv: ReturnValue<'s, Value>,
    body: impl for<'t> FnOnce(
        &'t mut ScriptToken,
        &mut PinScope<'s, 'i>,
        &FunctionCallbackArguments<'s>,
        &mut ReturnValue<'s, Value>,
    ),
) {
    let r = token_cell::try_reenter(|tok| body(tok, scope, &args, &mut rv));
    if let Err(e) = r {
        let msg = match e {
            Reentry::NotLent => "re-entrancy violation: script ran without a lent ScriptToken",
            Reentry::NoTask => "re-entrancy violation: callback outside any task",
        };
        let msg = v8::String::new(scope, msg).unwrap();
        let exc = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exc);
    }
}

/// Declares a V8 callback whose body runs under [`prologue`]. Stands in for
/// the binding generator (`hg-webidl-codegen`).
#[macro_export]
macro_rules! binding {
    ($vis:vis fn $name:ident = $body:path) => {
        $vis fn $name<'s, 'i>(
            scope: &mut v8::PinScope<'s, 'i>,
            args: v8::FunctionCallbackArguments<'s>,
            rv: v8::ReturnValue<'s, v8::Value>,
        ) {
            $crate::hg_script::prologue(scope, args, rv, $body)
        }
    };
}

pub fn call<'s>(
    tok: &mut ScriptToken,
    scope: &PinScope<'s, '_>,
    f: Local<Function>,
    recv: Local<Value>,
    args: &[Local<Value>],
) -> Option<Local<'s, Value>> {
    token_cell::lend(tok, || f.call(scope, recv, args))
}

pub fn run<'s>(
    tok: &mut ScriptToken,
    scope: &PinScope<'s, '_>,
    src: &str,
) -> Option<Local<'s, Value>> {
    let src = v8::String::new(scope, src)?;
    let script = v8::Script::compile(scope, src, None)?;
    token_cell::lend(tok, || script.run(scope))
}

pub fn get<'s>(
    tok: &mut ScriptToken,
    scope: &PinScope<'s, '_>,
    obj: Local<Object>,
    key: Local<Value>,
) -> Option<Local<'s, Value>> {
    token_cell::lend(tok, || obj.get(scope, key))
}

pub fn set(
    tok: &mut ScriptToken,
    scope: &PinScope<'_, '_>,
    obj: Local<Object>,
    key: Local<Value>,
    v: Local<Value>,
) -> Option<bool> {
    token_cell::lend(tok, || obj.set(scope, key, v))
}

pub fn to_rust_string(tok: &mut ScriptToken, scope: &PinScope<'_, '_>, v: Local<Value>) -> String {
    token_cell::lend(tok, || v.to_rust_string_lossy(scope))
}

pub fn int32(tok: &mut ScriptToken, scope: &PinScope<'_, '_>, v: Local<Value>) -> Option<i32> {
    token_cell::lend(tok, || v.int32_value(scope))
}

pub fn microtask_checkpoint(tok: &mut ScriptToken, scope: &mut PinScope<'_, '_>) {
    token_cell::lend(tok, || scope.perform_microtask_checkpoint())
}

/// Install a native function on `obj` (property set can run setters, so it
/// takes the token too).
pub fn install(
    tok: &mut ScriptToken,
    scope: &mut PinScope<'_, '_>,
    obj: Local<Object>,
    name: &str,
    cb: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let f = Function::new(scope, cb).unwrap();
    let k = v8::String::new(scope, name).unwrap();
    set(tok, scope, obj, k.into(), f.into()).unwrap();
}
