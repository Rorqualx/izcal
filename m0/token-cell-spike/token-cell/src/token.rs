//! `ScriptToken` and the state machine that mints it.
//!
//! Invariant (the whole soundness argument rests on it): on any thread, at any
//! moment, at most one `&mut ScriptToken` is usable. A token is minted only
//!
//! - by [`enter`], at the top of a task, when no token exists on this thread
//!   (state `Idle`). In the V8 crate this is wrapped by `hg_script::enter`,
//!   which also demands `&mut PinScope`, so the token is "derivable only from a
//!   mutable handle scope" (criterion 1);
//! - by [`reenter`], the generated callback prologue, only while the current
//!   token is lent to code that may run script (state `Lent`). `lend` takes the
//!   outer token by `&mut` for the whole call, so every borrow derived from the
//!   outer token is dead while the inner token exists. The borrow checker
//!   proves the outer half; this state check proves the inner half, because
//!   the edge between them goes through V8 (C++), which the borrow checker
//!   cannot see.
//!
//! Tokens are only ever handed out as `&'t mut ScriptToken` with a
//! higher-ranked `'t` scoped to a closure, so a token cannot outlive its frame
//! (no guard object that could be leaked with `Box::leak`).
//!
//! `ScriptToken` is a ZST with a private field: not constructible, not `Clone`,
//! not `Send`, not `Sync`.

use std::cell::Cell;
use std::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    /// No token exists on this thread.
    Idle,
    /// A token exists and is usable by exactly one frame.
    Held,
    /// The token's holder has lent it to code that may run script; the next
    /// callback prologue may mint a token for its own frame.
    Lent,
}

thread_local! {
    static STATE: Cell<State> = const { Cell::new(State::Idle) };
}

/// Proof that the holder is the only code on this thread that may run script
/// or touch script-reachable state. Zero-sized; passing it costs nothing.
pub struct ScriptToken {
    _not_send_sync: PhantomData<*mut ()>,
}

impl ScriptToken {
    #[inline(always)]
    fn mint() -> Self {
        ScriptToken {
            _not_send_sync: PhantomData,
        }
    }
}

/// Restores the thread state when the frame ends, including on unwind.
struct Restore(State);
impl Drop for Restore {
    #[inline(always)]
    fn drop(&mut self) {
        STATE.with(|s| s.set(self.0));
    }
}

#[inline(always)]
fn state() -> State {
    STATE.with(|s| s.get())
}

/// Top-of-task entry: mint the thread's token. Panics if one already exists.
///
/// In the real design this is `pub(crate)` in `hg-script` and only reachable
/// through `hg_script::enter(&mut PinScope, ..)`; it is `pub` here so the
/// V8-free tests can drive it.
#[inline]
pub fn enter<R>(f: impl for<'t> FnOnce(&'t mut ScriptToken) -> R) -> R {
    let prev = state();
    assert!(
        prev == State::Idle,
        "enter: a ScriptToken already exists on this thread ({prev:?})"
    );
    STATE.with(|s| s.set(State::Held));
    let _restore = Restore(State::Idle);
    f(&mut ScriptToken::mint())
}

/// Lend the token to code that may run script. `f` gets no token; a callback
/// that V8 runs inside `f` gets one from [`reenter`].
///
/// Taking `&mut ScriptToken` for the whole call is what makes the outer half
/// of the argument compile-time: no `TokenCell` borrow derived from `tok` can
/// be alive across this call.
#[inline]
pub fn lend<R>(tok: &mut ScriptToken, f: impl FnOnce() -> R) -> R {
    let _ = tok;
    debug_assert_eq!(state(), State::Held, "lend: token not held by this frame");
    STATE.with(|s| s.set(State::Lent));
    let _restore = Restore(State::Held);
    f()
}

/// Why a callback prologue refused to mint a token.
#[derive(Debug, PartialEq, Eq)]
pub enum Reentry {
    /// Script ran without the token being lent: some path reached V8 without a
    /// token-taking wrapper (criterion 2 was bypassed), or a GC-time callback
    /// tried to act like a script callback.
    NotLent,
    /// No token at all on this thread (V8 called back outside any task).
    NoTask,
}

/// The generated callback prologue: mint a token for this callback frame, only
/// if the outer token is lent.
#[inline]
pub fn try_reenter<R>(f: impl for<'t> FnOnce(&'t mut ScriptToken) -> R) -> Result<R, Reentry> {
    match state() {
        State::Lent => {}
        State::Held => return Err(Reentry::NotLent),
        State::Idle => return Err(Reentry::NoTask),
    }
    STATE.with(|s| s.set(State::Held));
    let _restore = Restore(State::Lent);
    Ok(f(&mut ScriptToken::mint()))
}

/// Panicking form of [`try_reenter`].
#[inline]
pub fn reenter<R>(f: impl for<'t> FnOnce(&'t mut ScriptToken) -> R) -> R {
    match try_reenter(f) {
        Ok(r) => r,
        Err(e) => panic!("reenter: callback prologue refused to mint a token: {e:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_mints_and_restores() {
        assert_eq!(state(), State::Idle);
        enter(|_t| assert_eq!(state(), State::Held));
        assert_eq!(state(), State::Idle);
    }

    #[test]
    #[should_panic(expected = "already exists")]
    fn nested_enter_panics() {
        enter(|_t| enter(|_t2| ()));
    }

    #[test]
    fn reenter_needs_lend() {
        assert_eq!(try_reenter(|_| ()), Err(Reentry::NoTask));
        enter(|t| {
            assert_eq!(try_reenter(|_| ()), Err(Reentry::NotLent));
            lend(t, || {
                assert_eq!(try_reenter(|_| 7), Ok(7));
                // Inside the callback the token is held again, so a second
                // callback prologue nested without a lend is refused.
                reenter(|_inner| assert_eq!(try_reenter(|_| ()), Err(Reentry::NotLent)));
            });
            assert_eq!(state(), State::Held);
        });
    }

    #[test]
    fn deep_lend_reenter_chain() {
        fn level(t: &mut ScriptToken, n: u32) -> u32 {
            if n == 0 {
                return 0;
            }
            lend(t, || reenter(|t2| level(t2, n - 1) + 1))
        }
        enter(|t| assert_eq!(level(t, 64), 64));
        assert_eq!(state(), State::Idle);
    }

    #[test]
    fn state_restored_on_unwind() {
        let r =
            std::panic::catch_unwind(|| enter(|t| lend(t, || reenter(|_| panic!("script threw")))));
        assert!(r.is_err());
        assert_eq!(state(), State::Idle);
    }

    #[test]
    fn tokens_are_per_thread() {
        enter(|_t| {
            std::thread::spawn(|| enter(|_t| assert_eq!(state(), State::Held)))
                .join()
                .unwrap();
        });
    }
}
