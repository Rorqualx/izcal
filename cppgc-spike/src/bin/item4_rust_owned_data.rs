//! Item 4: cppgc objects owning plain Rust data (Box / Arc, standing in for
//! Stylo per-element data). No Member is involved: the fields are ordinary
//! Rust values, and the Rust destructor must run when cppgc collects the
//! object. Counter-verified, plus a counting allocator showing the Rust heap
//! returns to baseline, plus which thread ran each Drop.
//!
//!   cargo run --release --bin item4_rust_owned_data

use cppgc_spike::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering::SeqCst};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use v8::cppgc::{GarbageCollected, Heap, HeapCreateParams, Persistent, Visitor};

struct Counting;
static LIVE_BYTES: AtomicIsize = AtomicIsize::new(0);
unsafe impl GlobalAlloc for Counting {
  unsafe fn alloc(&self, l: Layout) -> *mut u8 {
    LIVE_BYTES.fetch_add(l.size() as isize, SeqCst);
    unsafe { System.alloc(l) }
  }
  unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
    LIVE_BYTES.fetch_sub(l.size() as isize, SeqCst);
    unsafe { System.dealloc(p, l) }
  }
}
#[global_allocator]
static A: Counting = Counting;

static DROPS: AtomicUsize = AtomicUsize::new(0);
static DROP_THREADS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Stand-in for Stylo's per-element data (owned, non-GC).
struct ElementData {
  _computed: Vec<u64>,
}
impl Drop for ElementData {
  fn drop(&mut self) {
    DROPS.fetch_add(1, SeqCst);
    let name = format!("{:?}", std::thread::current().id());
    let mut t = DROP_THREADS.lock().unwrap();
    if !t.contains(&name) {
      t.push(name);
    }
  }
}
/// Stand-in for shared, immutable style data (e.g. a rule node / declaration block).
struct SharedRule {
  _decls: Vec<u8>,
}

struct Elem {
  _data: Box<ElementData>,
  _rule: Arc<SharedRule>,
}
unsafe impl GarbageCollected for Elem {
  fn trace(&self, _v: &mut Visitor) {} // no traced fields at all
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"Elem"
  }
}

const N: usize = 10_000;
const PREFIN_REGISTERED: usize = 200; // first 200 elems: 100 live, 100 dead

// ---- Prefinalizers (Q-1b(a) lists them). rusty_v8 has no API; the prebuilt
// library exports cppgc::internal::PrefinalizerRegistration's constructor and
// cppgc::LivenessBroker::IsHeapObjectAliveImpl (v8/include/cppgc/prefinalizer.h,
// liveness-broker.h), so we register directly. This is an *internal* cppgc API.
#[repr(C)]
struct LivenessBroker {
  _p: [u8; 0],
}
unsafe extern "C" {
  #[link_name = "_ZN5cppgc8internal24PrefinalizerRegistrationC1EPvPFbRKNS_14LivenessBrokerES2_E"]
  fn PrefinalizerRegistration_ctor(
    this: *mut u8,
    object: *mut std::ffi::c_void,
    cb: extern "C" fn(*const LivenessBroker, *mut std::ffi::c_void) -> bool,
  );
  #[link_name = "_ZNK5cppgc14LivenessBroker21IsHeapObjectAliveImplEPKv"]
  fn LivenessBroker_IsHeapObjectAliveImpl(this: *const LivenessBroker, obj: *const std::ffi::c_void) -> bool;
}
static PREFIN_RAN: AtomicUsize = AtomicUsize::new(0);
static PREFIN_RAN_AFTER_A_DROP_IN_SAME_GC: AtomicUsize = AtomicUsize::new(0);
static DROPS_AT_GC_START: AtomicUsize = AtomicUsize::new(0);
extern "C" fn prefinalizer(broker: *const LivenessBroker, obj: *mut std::ffi::c_void) -> bool {
  if unsafe { LivenessBroker_IsHeapObjectAliveImpl(broker, obj) } {
    return false; // object survives; keep the registration
  }
  // Dead object, destructors of this GC cycle have not run yet.
  if DROPS.load(SeqCst) != DROPS_AT_GC_START.load(SeqCst) {
    PREFIN_RAN_AFTER_A_DROP_IN_SAME_GC.fetch_add(1, SeqCst);
  }
  PREFIN_RAN.fetch_add(1, SeqCst);
  true
}

fn alloc_elems(heap: &Heap, rule: &Arc<SharedRule>) -> Vec<Persistent<Elem>> {
  let mut keep = Vec::with_capacity(N / 2);
  for i in 0..N {
    let e = Elem { _data: Box::new(ElementData { _computed: vec![i as u64; 32] }), _rule: rule.clone() };
    let p = unsafe { v8::cppgc::make_garbage_collected(heap, e) };
    if i < PREFIN_REGISTERED {
      use v8::cppgc::GetRustObj;
      let mut reg = 0u8;
      unsafe { PrefinalizerRegistration_ctor(&mut reg, p.get_rust_obj().cast(), prefinalizer) };
    }
    if i % 2 == 0 {
      keep.push(Persistent::new(&p));
    }
  }
  keep
}

fn report(label: &str, rule: &Arc<SharedRule>, base: isize, expect_drops: usize) -> bool {
  let drops = DROPS.load(SeqCst);
  let arc = Arc::strong_count(rule);
  let live = LIVE_BYTES.load(SeqCst) - base;
  let threads = DROP_THREADS.lock().unwrap().clone();
  let pf = PREFIN_RAN.load(SeqCst);
  let expect_pf = if expect_drops == N { PREFIN_REGISTERED } else { PREFIN_REGISTERED / 2 };
  let pf_late = PREFIN_RAN_AFTER_A_DROP_IN_SAME_GC.load(SeqCst);
  let ok = drops == expect_drops && arc == 1 + (N - expect_drops) && pf == expect_pf && pf_late == 0;
  println!(
    "{label}: drops={drops} (expect {expect_drops}) arc_strong={arc} (expect {}) prefinalizers_run={pf} (expect {expect_pf}, ran-after-a-destructor={pf_late}) rust_heap_delta={live}B drop_threads={threads:?} -> {}",
    1 + (N - expect_drops),
    if ok { "ok" } else { "MISMATCH" }
  );
  ok
}

fn detached() -> bool {
  let platform = init_v8("--expose-gc");
  let heap = Heap::create(platform, HeapCreateParams::default()); // concurrent sweeping enabled
  heap.enable_detached_garbage_collections_for_testing();
  let h = heap_handle(&heap);
  let mut th = TestingHeap::new(h);
  DROPS.store(0, SeqCst);
  PREFIN_RAN.store(0, SeqCst);
  DROP_THREADS.lock().unwrap().clear();
  let rule = Arc::new(SharedRule { _decls: vec![0; 256] });
  println!("main thread = {:?}", std::thread::current().id());
  let base = LIVE_BYTES.load(SeqCst);
  let mut keep = alloc_elems(&heap, &rule);
  println!("detached: allocated {N} Elem, rust heap +{}B", LIVE_BYTES.load(SeqCst) - base);

  // Incremental marking + (possibly concurrent) sweeping.
  DROPS_AT_GC_START.store(DROPS.load(SeqCst), SeqCst);
  th.start();
  while !th.step(true) {}
  th.finalize(true);
  println!("detached: right after finalize: drops={} is_sweeping={}", DROPS.load(SeqCst), is_sweeping(h));
  std::thread::sleep(Duration::from_millis(200));
  println!("detached: 200ms later (no mutator activity): drops={} is_sweeping={}", DROPS.load(SeqCst), is_sweeping(h));
  unsafe { heap.collect_garbage_for_testing(v8::cppgc::EmbedderStackState::NoHeapPointers) }; // finishes sweeping
  let ok1 = report("detached half-collected", &rule, base, N / 2);
  keep.clear();
  DROPS_AT_GC_START.store(DROPS.load(SeqCst), SeqCst);
  unsafe { heap.collect_garbage_for_testing(v8::cppgc::EmbedderStackState::NoHeapPointers) };
  let ok2 = report("detached all-collected", &rule, base, N);
  ok1 && ok2
}

fn unified() -> bool {
  let _p = init_v8("--expose-gc");
  let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
  let heap: &Heap = unsafe { &*(isolate.get_cpp_heap().unwrap() as *const Heap) };
  DROPS.store(0, SeqCst);
  PREFIN_RAN.store(0, SeqCst);
  DROP_THREADS.lock().unwrap().clear();
  let rule = Arc::new(SharedRule { _decls: vec![0; 256] });
  let base = LIVE_BYTES.load(SeqCst);
  let mut keep = alloc_elems(heap, &rule);
  v8::scope!(let hs, isolate);
  let ctx = v8::Context::new(hs, Default::default());
  let scope = &mut v8::ContextScope::new(hs, ctx);
  // Let V8's own heuristics collect: allocate JS garbage only (no explicit GC).
  let src = v8::String::new(scope, "globalThis.old ??= []; for (let i = 0; i < 2000; i++) { old.push(new Array(256).fill(i)); } if (old.length > 200000) old = [];").unwrap();
  let script = v8::Script::compile(scope, src, None).unwrap();
  let t = Instant::now();
  DROPS_AT_GC_START.store(DROPS.load(SeqCst), SeqCst);
  while DROPS.load(SeqCst) < N / 2 && t.elapsed() < Duration::from_secs(20) {
    script.run(scope).unwrap();
  }
  println!("unified: V8-scheduled GCs (JS allocation only) reached drops={} in {:?}", DROPS.load(SeqCst), t.elapsed());
  full_gc_precise(scope);
  let ok1 = report("unified half-collected", &rule, base, N / 2);
  keep.clear();
  DROPS_AT_GC_START.store(DROPS.load(SeqCst), SeqCst);
  full_gc_precise(scope);
  let ok2 = report("unified all-collected", &rule, base, N);
  ok1 && ok2
}

fn main() {
  let mode = std::env::args().nth(1).unwrap_or_default();
  if mode.is_empty() {
    let mut all = true;
    for m in ["detached", "unified"] {
      let (o, _) = run_child(&[m], Duration::from_secs(120));
      println!("[{m}] outcome={o:?}");
      all &= matches!(o, ChildOutcome::Ok);
    }
    println!("RESULT {}", if all { "PASS" } else { "FAIL" });
    return;
  }
  let ok = if mode == "detached" { detached() } else { unified() };
  std::process::exit(if ok { 0 } else { 1 });
}
