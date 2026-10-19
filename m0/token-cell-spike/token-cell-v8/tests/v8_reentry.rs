//! Criteria (1), (3), (4) against the real fork V8: JS observers that re-enter
//! native DOM bindings, the runtime refusal of an unwrapped script call, and
//! MemberCell links under cppgc GC. The holes are in `holes_compile.rs`.
#![allow(clippy::disallowed_methods)] // tests call raw V8 on purpose (marked)

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::{Mutex, MutexGuard, OnceLock};
use token_cell::dom::{Document, MutationRecord, NodeId};
use token_cell::{ScriptToken, TokenCell};
use token_cell_v8::binding;
use token_cell_v8::event_loop::EventLoop;
use token_cell_v8::hg_script;
use v8::{FunctionCallbackArguments, Local, PinScope, ReturnValue, Value};

static PLATFORM: OnceLock<v8::SharedRef<v8::Platform>> = OnceLock::new();
static LOCK: Mutex<()> = Mutex::new(());

fn init() -> (v8::SharedRef<v8::Platform>, MutexGuard<'static, ()>) {
    let g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let p = PLATFORM
        .get_or_init(|| {
            let p = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(p.clone());
            v8::cppgc::initialize_process(p.clone());
            v8::V8::initialize();
            p
        })
        .clone();
    (p, g)
}

thread_local! {
    static DOC: Document = Document::new();
    static PENDING: TokenCell<Vec<MutationRecord>> = const { TokenCell::new(Vec::new()) };
    static DEPTH: RefCell<u32> = const { RefCell::new(0) };
}

// ---- bindings (what hg-webidl-codegen would emit) -------------------------

fn arg_u32(
    tok: &mut ScriptToken,
    scope: &PinScope<'_, '_>,
    args: &FunctionCallbackArguments,
    i: i32,
) -> u32 {
    hg_script::int32(tok, scope, args.get(i)).unwrap() as u32
}

fn set_text_impl<'s>(
    tok: &mut ScriptToken,
    scope: &mut PinScope<'s, '_>,
    args: &FunctionCallbackArguments<'s>,
    _rv: &mut ReturnValue<'s, Value>,
) {
    // Argument conversion can run script (valueOf), so it takes the token too.
    let id = arg_u32(tok, scope, args, 0);
    let text = hg_script::to_rust_string(tok, scope, args.get(1));
    DOC.with(|d| d.set_text(id, &text, tok));
}
binding!(fn set_text_cb = set_text_impl);

fn insert_before_impl<'s>(
    tok: &mut ScriptToken,
    scope: &mut PinScope<'s, '_>,
    args: &FunctionCallbackArguments<'s>,
    _rv: &mut ReturnValue<'s, Value>,
) {
    let p = arg_u32(tok, scope, args, 0);
    let c = arg_u32(tok, scope, args, 1);
    let r = arg_u32(tok, scope, args, 2);
    let r = if r == u32::MAX { None } else { Some(r) };
    DOC.with(|d| d.insert_before(p, c, r, tok)).unwrap();
    deliver(tok, scope); // [CEReactions]-style delivery before returning to JS
}
binding!(fn insert_before_cb = insert_before_impl);

/// Deliver pending records to `globalThis.observer(target, added)`.
fn deliver(tok: &mut ScriptToken, scope: &mut PinScope<'_, '_>) {
    let recs: Vec<MutationRecord> = PENDING.with(|p| std::mem::take(p.borrow_mut(tok)));
    if recs.is_empty() {
        return;
    }
    let global = scope.get_current_context().global(scope);
    let key = v8::String::new(scope, "observer").unwrap();
    let obs = hg_script::get(tok, scope, global, key.into()).unwrap();
    let Ok(obs) = Local::<v8::Function>::try_from(obs) else {
        return;
    };
    DEPTH.with(|d| *d.borrow_mut() += 1);
    for r in recs {
        let a = v8::Integer::new_from_unsigned(scope, r.target).into();
        let b = v8::Integer::new_from_unsigned(scope, r.added).into();
        let recv = v8::undefined(scope).into();
        hg_script::call(tok, scope, obs, recv, &[a, b]);
    }
}

fn setup_dom(tok: &mut ScriptToken) -> (NodeId, NodeId, NodeId, NodeId) {
    DOC.with(|d| {
        let root = d.create_node("root", tok);
        d.set_connected(root, tok);
        let a = d.create_node("a", tok);
        let b = d.create_node("b", tok);
        let x = d.create_node("x", tok);
        d.insert_before(root, a, None, tok).unwrap();
        d.insert_before(root, b, None, tok).unwrap();
        d.observe(
            Rc::new(|_d: &Document, rec: &MutationRecord, t: &mut ScriptToken| {
                PENDING.with(|p| p.borrow_mut(t).push(rec.clone()))
            }),
            tok,
        );
        PENDING.with(|p| p.borrow_mut(tok).clear());
        (root, a, b, x)
    })
}

fn install_bindings(tok: &mut ScriptToken, scope: &mut PinScope<'_, '_>) {
    let global = scope.get_current_context().global(scope);
    hg_script::install(tok, scope, global, "setText", set_text_cb);
    hg_script::install(tok, scope, global, "insertBefore", insert_before_cb);
}

macro_rules! with_context {
    (|$scope:ident| $body:block) => {{
        let (_p, _g) = init();
        let isolate = &mut v8::Isolate::new(Default::default());
        v8::scope!(let hs, isolate);
        let context = v8::Context::new(hs, Default::default());
        let $scope = &mut v8::ContextScope::new(hs, context);
        $body
    }};
}

// ---- criterion (4) with real JS --------------------------------------------

#[test]
fn js_observer_reenters_native_dom() {
    with_context!(|scope| {
        EventLoop::new().run_task(scope, |scope, tok| {
            let (root, a, b, x) = setup_dom(tok);
            install_bindings(tok, scope);
            hg_script::run(
                tok,
                scope,
                "globalThis.observer = (t, n) => setText(n, 'seen by observer of ' + t);",
            )
            .unwrap();
            DOC.with(|d| d.insert_before(root, x, Some(b), tok))
                .unwrap();
            deliver(tok, scope);
            DOC.with(|d| {
                assert_eq!(d.text(x, tok), "seen by observer of 0");
                assert_eq!(d.children(root, tok), vec![a, x, b]);
            });
        });
    });
}

#[test]
fn js_observer_nested_insert_before() {
    // Rust insertBefore -> JS observer -> native insertBefore -> JS observer
    // -> native setText: lend/reenter two levels deep through V8.
    with_context!(|scope| {
        EventLoop::new().run_task(scope, |scope, tok| {
            let (root, a, b, x) = setup_dom(tok);
            let y = DOC.with(|d| d.create_node("y", tok));
            install_bindings(tok, scope);
            let src = format!(
                "globalThis.observer = (t, n) => {{ if (n === {x}) insertBefore(t, {y}, {x}); else setText(n, 'nested'); }};"
            );
            hg_script::run(tok, scope, &src).unwrap();
            DOC.with(|d| d.insert_before(root, x, Some(b), tok))
                .unwrap();
            deliver(tok, scope);
            DOC.with(|d| {
                assert_eq!(d.children(root, tok), vec![a, y, x, b]);
                assert_eq!(d.child_count(root, tok), 4);
                assert_eq!(d.text(y, tok), "nested");
            });
            assert_eq!(DEPTH.with(|d| *d.borrow()), 2);
        });
    });
}

#[test]
fn microtask_reentry_through_checkpoint_wrapper() {
    with_context!(|scope| {
        EventLoop::new().run_task(scope, |scope, tok| {
            let (_root, a, _b, _x) = setup_dom(tok);
            install_bindings(tok, scope);
            // Explicit policy, so the job runs in the checkpoint wrapper and
            // not at the end of Script::run (V8's default kAuto).
            scope.set_microtasks_policy(v8::MicrotasksPolicy::Explicit);
            let src = format!("Promise.resolve().then(() => setText({a}, 'from microtask'));");
            hg_script::run(tok, scope, &src).unwrap();
            DOC.with(|d| assert_eq!(d.text(a, tok), "a"));
            hg_script::microtask_checkpoint(tok, scope);
            DOC.with(|d| assert_eq!(d.text(a, tok), "from microtask"));
        });
    });
}

// ---- criterion (1) runtime half: no token unless lent ------------------------

#[test]
fn unwrapped_script_call_is_refused() {
    with_context!(|scope| {
        EventLoop::new().run_task(scope, |scope, tok| {
            let (_root, a, _b, _x) = setup_dom(tok);
            install_bindings(tok, scope);
            let f = hg_script::run(
                tok,
                scope,
                &format!("() => setText({a}, 'should not happen')"),
            )
            .unwrap();
            let f = Local::<v8::Function>::try_from(f).unwrap();
            v8::tc_scope!(let tc, scope);
            let recv = v8::undefined(tc).into();
            // RAW call, no lend: what a path missing from disallowed-methods
            // would look like. The callback prologue sees state Held.
            assert!(f.call(tc, recv, &[]).is_none());
            let msg = tc
                .exception()
                .expect("expected an exception")
                .to_rust_string_lossy(tc);
            assert!(
                msg.contains("re-entrancy violation: script ran without a lent ScriptToken"),
                "{msg}"
            );
            DOC.with(|d| assert_eq!(d.text(a, tok), "a"));
        });
    });
}

#[test]
fn callback_outside_any_task_is_refused() {
    with_context!(|scope| {
        let f = v8::Function::new(scope, set_text_cb).unwrap();
        v8::tc_scope!(let tc, scope);
        let recv = v8::undefined(tc).into();
        assert!(f.call(tc, recv, &[]).is_none());
        let msg = tc
            .exception()
            .expect("expected an exception")
            .to_rust_string_lossy(tc);
        assert!(msg.contains("callback outside any task"), "{msg}");
    });
}

// ---- criterion (3): MemberCell links traced outside TokenCell ---------------

static DROPS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn member_cell_links_keep_nodes_alive_under_gc() {
    use token_cell_v8::member_cell::{DomNode, MemberCell};
    use v8::cppgc::*;
    let (p, _g) = init();
    let heap = Heap::create(p, HeapCreateParams::default());
    heap.enable_detached_garbage_collections_for_testing();
    let mk = |text: &str| {
        let n = DomNode {
            parent: MemberCell::empty(),
            first_child: MemberCell::empty(),
            next_sibling: MemberCell::empty(),
            prev_sibling: MemberCell::empty(),
            data: TokenCell::new(token_cell::dom::NodeData {
                text: text.into(),
                ..Default::default()
            }),
            drops: &DROPS,
        };
        // SAFETY: the pointer is immediately moved into a Persistent or a
        // traced MemberCell.
        unsafe { make_garbage_collected(&heap, n) }
    };
    DROPS.store(0, SeqCst);
    let root = Persistent::new(&mk("root"));
    {
        let a = mk("a");
        let b = mk("b");
        let r = root.get().unwrap();
        // insertBefore-style link writes through &self, no token.
        r.first_child.set(&a);
        // SAFETY: `a` is reachable from root's traced first_child.
        let ar = unsafe { a.as_ref() };
        ar.parent.set(&root);
        ar.next_sibling.set(&b);
        // SAFETY: `b` is reachable from a.next_sibling.
        let br = unsafe { b.as_ref() };
        br.prev_sibling.set(&a);
        br.parent.set(&root);
    }
    // SAFETY: no raw heap pointers on the stack are needed past this point.
    unsafe { heap.collect_garbage_for_testing(EmbedderStackState::NoHeapPointers) };
    assert_eq!(DROPS.load(SeqCst), 0, "linked nodes must survive");
    // Script-reachable state is still usable after GC, with the token.
    token_cell::enter(|tok| {
        let r = root.get().unwrap();
        let a = r.first_child.get().unwrap();
        // SAFETY: `a` is reachable (traced link).
        assert_eq!(unsafe { a.as_ref() }.data.borrow(tok).text, "a");
        // Unlink b (removeChild): only the links change.
        // SAFETY: as above.
        unsafe { a.as_ref() }.next_sibling.clear();
    });
    // SAFETY: as above.
    unsafe { heap.collect_garbage_for_testing(EmbedderStackState::NoHeapPointers) };
    // b is unreachable now; its TokenCell<NodeData> (a String) is dropped
    // during sweep without a token, which CS-V8-9 allows for plain buffers.
    assert_eq!(DROPS.load(SeqCst), 1, "unlinked node must be collected");
    drop(root);
    // SAFETY: as above.
    unsafe { heap.collect_garbage_for_testing(EmbedderStackState::NoHeapPointers) };
    assert_eq!(DROPS.load(SeqCst), 3);
}

// Silence unused warnings for helpers used only by some tests.
#[allow(dead_code)]
fn _unused(_: &PinScope<'_, '_>) {}
