//! Only built with `--features misuse`. Every call below must be flagged by
//! clippy `disallowed-methods` (criterion 2). Not run.
#![allow(dead_code)]

use v8::{Function, Local, Object, PinScope, Value};

pub fn direct_call(scope: &PinScope<'_, '_>, f: Local<Function>, recv: Local<Value>) {
    f.call(scope, recv, &[]); // method syntax through Local's Deref
}

pub fn ufcs_call(scope: &PinScope<'_, '_>, f: Local<Function>, recv: Local<Value>) {
    Function::call(&f, scope, recv, &[]); // path syntax
}

pub fn fn_pointer() -> usize {
    let p = Function::call; // taken as a value, called later
    p as usize
}

pub fn property_get(scope: &PinScope<'_, '_>, o: Local<Object>, k: Local<Value>) {
    let _ = o.get(scope, k);
}

pub fn conversion(scope: &PinScope<'_, '_>, v: Local<Value>) -> String {
    v.to_rust_string_lossy(scope) // transitive: calls Value::to_string inside rusty_v8
}

pub fn microtasks(scope: &mut PinScope<'_, '_>) {
    scope.perform_microtask_checkpoint(); // through PinScope's DerefMut to Isolate
}

pub fn script_run(scope: &PinScope<'_, '_>, s: Local<v8::Script>) {
    let _ = s.run(scope);
}

/// The hg_script wrapper: not flagged.
pub fn via_wrapper(
    tok: &mut token_cell::ScriptToken,
    scope: &PinScope<'_, '_>,
    f: Local<Function>,
    recv: Local<Value>,
) {
    crate::hg_script::call(tok, scope, f, recv, &[]);
}
