//! Item 3: DOM-to-JS edges through the unified heap.
//!
//! 3a  A cppgc "DOM node" holds a TracedReference to its JS wrapper; the node
//!     is rooted only from the DOM side (cppgc Persistent). After forced full
//!     GCs the wrapper must survive with its expando and identity intact.
//! 3b  Same pair rooted only from JS (globalThis.keep = wrapper): the node must
//!     survive through the wrapper's CppHeap pointer (Object::wrap).
//! 3c  DOM<->JS cycle (wrapper -> node via Object::wrap, node -> wrapper via
//!     TracedReference, plus a JS closure on the wrapper capturing it). Drop
//!     every root, force GC: both node and wrapper must be collected.
//! 3d  Negative control: a plain Rust struct (Box, not cppgc) holding the
//!     wrapper through v8::Global, with the wrapper pointing back through an
//!     internal-field External. Drop all JS roots, force GC: the cycle must
//!     NOT be collected (expected leak).
//!
//!   cargo run --release --bin item3_dom_js_edges

use cppgc_spike::init_v8;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use v8::cppgc::{GarbageCollected, Persistent, Visitor};

const TAG: u16 = 1;
static NODE_DROPS: AtomicUsize = AtomicUsize::new(0);
static RUST_NODE_DROPS: AtomicUsize = AtomicUsize::new(0);

struct DomNode {
  id: u32,
  wrapper: v8::TracedReference<v8::Object>,
}
unsafe impl GarbageCollected for DomNode {
  fn trace(&self, v: &mut Visitor) {
    v.trace(&self.wrapper);
  }
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"DomNode"
  }
}
impl Drop for DomNode {
  fn drop(&mut self) {
    NODE_DROPS.fetch_add(1, SeqCst);
  }
}

struct RustNode {
  _wrapper: Option<v8::Global<v8::Object>>,
}
impl Drop for RustNode {
  fn drop(&mut self) {
    RUST_NODE_DROPS.fetch_add(1, SeqCst);
  }
}

fn run(scope: &mut v8::PinScope, src: &str) -> v8::Local<'static, v8::Value> {
  let code = v8::String::new(scope, src).unwrap();
  let s = v8::Script::compile(scope, code, None).unwrap();
  unsafe { std::mem::transmute(s.run(scope).unwrap()) }
}

fn full_gc(scope: &mut v8::PinScope) {
  for _ in 0..3 {
    if std::env::var("ITEM3_CONSERVATIVE").is_ok() {
      scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
    } else {
      cppgc_spike::full_gc_precise(scope);
    }
  }
}

/// Create node + wrapper pair. Sets `expando = id` and `self = wrapper` and a
/// closure capturing the wrapper (a JS-internal cycle) on the wrapper.
#[inline(never)]
fn make_pair(scope: &mut v8::PinScope, id: u32) -> (v8::Weak<v8::Object>, *const DomNode, Option<Persistent<DomNode>>) {
  v8::scope!(let s, scope);
  fn empty(_: &mut v8::PinScope, _: v8::FunctionCallbackArguments, _: v8::ReturnValue<v8::Value>) {}
  let templ = v8::FunctionTemplate::new(s, empty);
  let obj = templ.get_function(s).unwrap().new_instance(s, &[]).unwrap();
  assert!(obj.is_api_wrapper());
  let node = DomNode { id, wrapper: v8::TracedReference::new(s, obj) };
  let heap = s.get_cpp_heap().unwrap() as *const v8::cppgc::Heap;
  let ptr = unsafe { v8::cppgc::make_garbage_collected(&*heap, node) };
  unsafe { v8::Object::wrap::<TAG, DomNode>(s, obj, &ptr) };
  let k = v8::String::new(s, "tmp").unwrap();
  s.get_current_context().global(s).set(s, k.into(), obj.into());
  run(s, &format!("tmp.expando = {id}; tmp.self = tmp; {{ const w = tmp; tmp.f = () => w; }} undefined"));
  let weak = v8::Weak::new(s, obj);
  let raw = unsafe { ptr.as_ref() } as *const DomNode;
  let p = Some(Persistent::new(&ptr));
  (weak, raw, p)
}

fn main() {
  let _p = init_v8("--expose-gc");
  let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
  v8::scope!(let hs, isolate);
  let ctx = v8::Context::new(hs, Default::default());
  let scope = &mut v8::ContextScope::new(hs, ctx);
  let mut results = vec![];

  // ---- 3a: rooted from the DOM side only ----
  let (weak_a, raw_a, pers_a) = make_pair(scope, 1);
  run(scope, "delete globalThis.tmp; undefined");
  full_gc(scope);
  let node = pers_a.as_ref().unwrap().get().unwrap();
  let ok_a = {
    v8::scope!(let s, scope);
    match node.wrapper.get(s) {
      Some(w) => {
        let k = v8::String::new(s, "expando").unwrap();
        let expando = w.get(s, k.into()).unwrap().int32_value(s).unwrap();
        let back = unsafe { v8::Object::unwrap::<TAG, DomNode>(s, w) }.map(|p| unsafe { p.as_ref() } as *const DomNode);
        println!("3a: wrapper alive, expando={expando}, unwrap==node: {}", back == Some(raw_a));
        expando == 1 && back == Some(raw_a) && !weak_a.is_empty()
      }
      None => false,
    }
  };
  println!("3a: node drops so far={} -> {}", NODE_DROPS.load(SeqCst), if ok_a { "PASS" } else { "FAIL" });
  results.push(ok_a && NODE_DROPS.load(SeqCst) == 0);
  drop(pers_a);
  full_gc(scope);
  println!(
    "3a': after dropping the DOM-side root too: wrapper collected={} node finalized={}",
    weak_a.is_empty(),
    NODE_DROPS.load(SeqCst)
  );

  // ---- 3b: rooted from JS only ----
  let drops0 = NODE_DROPS.load(SeqCst);
  let (weak_b, raw_b, mut pers_b) = make_pair(scope, 2);
  run(scope, "globalThis.keep = tmp; delete globalThis.tmp; undefined");
  pers_b.take(); // no DOM-side root
  full_gc(scope);
  let ok_b = {
    v8::scope!(let s, scope);
    let keep: v8::Local<v8::Object> = run(s, "globalThis.keep").try_into().unwrap();
    let back = unsafe { v8::Object::unwrap::<TAG, DomNode>(s, keep) }.map(|p| unsafe { p.as_ref() } as *const DomNode);
    let id = back.map(|p| unsafe { (*p).id });
    println!(
      "3b: node reachable via wrapper: id={id:?} same_ptr={} node_drops={}",
      back == Some(raw_b),
      NODE_DROPS.load(SeqCst) - drops0
    );
    id == Some(2) && NODE_DROPS.load(SeqCst) == drops0 && !weak_b.is_empty()
  };
  println!("3b: {}", if ok_b { "PASS" } else { "FAIL" });
  results.push(ok_b);

  // ---- 3c: drop all roots of a DOM<->JS cycle; must be collected ----
  let drops0 = NODE_DROPS.load(SeqCst);
  let (weak_c, _raw_c, mut pers_c) = make_pair(scope, 3);
  run(scope, "delete globalThis.tmp; delete globalThis.keep; undefined");
  pers_c.take();
  full_gc(scope);
  let collected_nodes = NODE_DROPS.load(SeqCst) - drops0;
  // Expect 2: the 3c node and the 3b node whose JS root was also deleted.
  let ok_c = weak_c.is_empty() && weak_b.is_empty() && collected_nodes == 2;
  println!(
    "3c: wrapper collected={} (3b wrapper collected={}) cppgc nodes finalized={collected_nodes} (expect 2) -> {}",
    weak_c.is_empty(),
    weak_b.is_empty(),
    if ok_c { "PASS" } else { "FAIL" }
  );
  results.push(ok_c);

  // ---- 3d: negative control, Rust-owned struct + v8::Global ----
  let (weak_d, rust_node) = {
    v8::scope!(let s, scope);
    let ot = v8::ObjectTemplate::new(s);
    ot.set_internal_field_count(1);
    let obj = ot.new_instance(s).unwrap();
    let rust_node = Box::into_raw(Box::new(RustNode { _wrapper: Some(v8::Global::new(s, obj)) }));
    let ext = v8::External::new(s, rust_node as *mut std::ffi::c_void);
    obj.set_internal_field(0, ext.into());
    let k = v8::String::new(s, "tmp").unwrap();
    s.get_current_context().global(s).set(s, k.into(), obj.into());
    run(s, "{ const w = tmp; tmp.f = () => w; } delete globalThis.tmp; undefined");
    (v8::Weak::new(s, obj), rust_node)
  };
  full_gc(scope);
  let leaked = !weak_d.is_empty() && RUST_NODE_DROPS.load(SeqCst) == 0;
  println!(
    "3d: after dropping all JS roots + GC: wrapper alive={} rust_node dropped={} -> cycle {} (expected: leaked)",
    !weak_d.is_empty(),
    RUST_NODE_DROPS.load(SeqCst),
    if leaked { "LEAKED (negative result confirmed)" } else { "collected (unexpected)" }
  );
  results.push(leaked);
  // Only manual intervention breaks it.
  unsafe { drop(Box::from_raw(rust_node)) };
  full_gc(scope);
  println!("3d: after manually dropping the Rust struct: wrapper collected={}", weak_d.is_empty());

  let all = results.iter().all(|r| *r);
  println!("RESULT {} ({results:?})", if all { "PASS" } else { "FAIL" });
}
