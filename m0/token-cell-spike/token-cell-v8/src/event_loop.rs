//! Criterion (1), the outer half: only the event loop mints a thread's
//! outermost `ScriptToken`. `hg_script::enter` is private to this crate, and
//! `token_cell::enter` is in clippy's `disallowed-methods`, so other code can
//! start a task only through [`EventLoop::run_task`].

use std::cell::Cell;
use std::marker::PhantomData;
use token_cell::ScriptToken;
use v8::PinScope;

thread_local! {
    static EXISTS: Cell<bool> = const { Cell::new(false) };
}

/// The thread's event loop: at most one per thread, `!Send`.
pub struct EventLoop {
    _not_send: PhantomData<*mut ()>,
}

impl EventLoop {
    /// Panics if this thread already has an event loop.
    pub fn new() -> Self {
        assert!(
            !EXISTS.with(|e| e.replace(true)),
            "EventLoop::new: this thread already has an event loop"
        );
        EventLoop {
            _not_send: PhantomData,
        }
    }

    /// Run one task with the thread's token (panics if a task is running).
    pub fn run_task<'s, 'i, R>(
        &mut self,
        scope: &mut PinScope<'s, 'i>,
        f: impl for<'t> FnOnce(&mut PinScope<'s, 'i>, &'t mut ScriptToken) -> R,
    ) -> R {
        crate::hg_script::enter(scope, f)
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        EXISTS.with(|e| e.set(false));
    }
}
