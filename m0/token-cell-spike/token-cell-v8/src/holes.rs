//! Why neither the scope nor the isolate can be the token (criterion 1's
//! "separate type or the scope itself" question).
//!
//! In the fork (rusty_v8 152.2.0), every script-running method takes
//! `&PinScope` (shared), and every callback receives a fresh `&mut PinScope`
//! built by V8's trampoline (`function.rs`, `CallbackScope::new(info)`).
//! So a borrow tied to `&PinScope` survives `Function::call`, and the callback
//! can take a `&mut` borrow of the same cell: `&T` and `&mut T` coexist in
//! safe code. Upstream `cppgc::GcCell` (borrows tied to `&Isolate` /
//! `&mut Isolate`) has the same hole, because both scopes deref to the isolate.
//!
//! Everything in this module compiles; that is the finding.
#![allow(clippy::disallowed_methods)]

use std::cell::UnsafeCell;
use v8::{Function, FunctionCallbackArguments, Local, PinScope, ReturnValue, Value};

/// A cell whose borrows are tied to the scope (the "scope is the token" option).
pub struct ScopeCell<T>(UnsafeCell<T>);

impl<T> ScopeCell<T> {
    pub const fn new(v: T) -> Self {
        ScopeCell(UnsafeCell::new(v))
    }
    pub fn borrow<'a>(&'a self, _s: &'a PinScope<'_, '_>) -> &'a T {
        // SAFETY: none; this type exists to show the design is unsound.
        unsafe { &*self.0.get() }
    }
    pub fn borrow_mut<'a>(&'a self, _s: &'a mut PinScope<'_, '_>) -> &'a mut T {
        // SAFETY: none; this type exists to show the design is unsound.
        unsafe { &mut *self.0.get() }
    }
}

thread_local! {
    pub static SCOPE_CELL: ScopeCell<u64> = const { ScopeCell::new(1) };
    pub static GC_CELL: v8::cppgc::GcCell<u64> = v8::cppgc::GcCell::new(1);
}

/// Outer frame: a shared borrow (tied to the scope) alive across a call that
/// runs script. Compiles.
pub fn scope_cell_borrow_across_call(scope: &PinScope<'_, '_>, f: Local<Function>) -> (u64, u64) {
    SCOPE_CELL.with(|c| {
        let r: &u64 = c.borrow(scope);
        let before = *r;
        let recv = v8::undefined(scope).into();
        f.call(scope, recv, &[]);
        (before, *r)
    })
}

/// Callback: V8 hands it `&mut PinScope`, so it can borrow the same cell
/// mutably while the outer `&u64` is alive. Compiles.
pub fn scope_cell_callback(
    scope: &mut PinScope<'_, '_>,
    _a: FunctionCallbackArguments,
    _r: ReturnValue<Value>,
) {
    SCOPE_CELL.with(|c| *c.borrow_mut(scope) = 42);
}

/// The same with upstream `GcCell`: `get(&Isolate)` alive across a call.
pub fn gc_cell_borrow_across_call(scope: &PinScope<'_, '_>, f: Local<Function>) -> (u64, u64) {
    GC_CELL.with(|c| {
        let r: &u64 = c.get(scope);
        let before = *r;
        let recv = v8::undefined(scope).into();
        f.call(scope, recv, &[]);
        (before, *r)
    })
}

pub fn gc_cell_callback(
    scope: &mut PinScope<'_, '_>,
    _a: FunctionCallbackArguments,
    _r: ReturnValue<Value>,
) {
    GC_CELL.with(|c| *c.get_mut(scope) = 42);
}
