//! Prefinalizers (CPPGC_USING_PRE_FINALIZER), LivenessBroker and custom weak
//! callbacks for Rust-defined GC types. Includes the prefinalizer part of the
//! cppgc spike's item 4.

mod common;
use common::*;
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use v8::cppgc::{EmbedderStackState, GarbageCollected, Heap, Member, Persistent, Visitor};
use v8_cppgc_ext::testing::StandaloneTestingHeap;
use v8_cppgc_ext::*;

// ---- item 4 (prefinalizer part) ----

static DROPS: AtomicUsize = AtomicUsize::new(0);
static PREFIN_RAN: AtomicUsize = AtomicUsize::new(0);
static PREFIN_AFTER_DROP: AtomicUsize = AtomicUsize::new(0);
static PREFIN_NOT_IN_ATOMIC_PAUSE: AtomicUsize = AtomicUsize::new(0);
static PARTNER_ALIVE: AtomicUsize = AtomicUsize::new(0);
static PARTNER_DEAD: AtomicUsize = AtomicUsize::new(0);
static DROPS_AT_GC_START: AtomicUsize = AtomicUsize::new(0);
thread_local! {
  static HEAP: Cell<*const Heap> = const { Cell::new(std::ptr::null()) };
}

struct Elem {
  idx: usize,
  partner: Member<Elem>,
  #[allow(clippy::box_collection)] // stands in for Rust-owned per-element data
  _data: Box<Vec<u64>>,
}
unsafe impl GarbageCollected for Elem {
  fn trace(&self, v: &mut Visitor) {
    v.trace(&self.partner);
  }
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"Elem"
  }
}
impl Drop for Elem {
  fn drop(&mut self) {
    DROPS.fetch_add(1, SeqCst);
  }
}
impl PreFinalizer for Elem {
  fn pre_finalize(&self, broker: &LivenessBroker) {
    if DROPS.load(SeqCst) != DROPS_AT_GC_START.load(SeqCst) {
      PREFIN_AFTER_DROP.fetch_add(1, SeqCst);
    }
    let heap = HEAP.with(|h| h.get());
    if !heap.is_null() && !HeapState::is_in_atomic_pause(unsafe { &*heap }.heap_handle()) {
      PREFIN_NOT_IN_ATOMIC_PAUSE.fetch_add(1, SeqCst);
    }
    // The whole graph is readable; the broker says which part survives.
    if broker.is_heap_object_alive(&self.partner) {
      PARTNER_ALIVE.fetch_add(1, SeqCst);
    } else {
      PARTNER_DEAD.fetch_add(1, SeqCst);
    }
    let _ = unsafe { self.partner.get() }.map(|p| p.idx); // reading a dead partner is allowed
    PREFIN_RAN.fetch_add(1, SeqCst);
  }
}

const N: usize = 10_000;
const PREFIN_REGISTERED: usize = 200; // first 200: 100 live, 100 dead

fn reset() {
  for c in [&DROPS, &PREFIN_RAN, &PREFIN_AFTER_DROP, &PREFIN_NOT_IN_ATOMIC_PAUSE, &PARTNER_ALIVE, &PARTNER_DEAD] {
    c.store(0, SeqCst);
  }
}

/// Elements with even index are rooted. Every element points at an
/// "anchor" that stays alive until the last GC, so prefinalizers can observe
/// a partner that is alive (first GC) and one that is dead (last GC).
fn alloc_elems(heap: &Heap) -> Vec<Persistent<Elem>> {
  let mut keep = Vec::with_capacity(N / 2 + 1);
  let anchor = unsafe {
    v8::cppgc::make_garbage_collected(heap, Elem { idx: usize::MAX, partner: Member::empty(), _data: Box::new(vec![]) })
  };
  keep.push(Persistent::new(&anchor));
  for i in 0..N {
    let e = Elem { idx: i, partner: Member::new(&anchor), _data: Box::new(vec![i as u64; 32]) };
    let p = if i < PREFIN_REGISTERED {
      unsafe { make_garbage_collected_with_prefinalizer(heap, e) }
    } else {
      unsafe { v8::cppgc::make_garbage_collected(heap, e) }
    };
    if i % 2 == 0 {
      keep.push(Persistent::new(&p));
    }
  }
  keep
}

fn check(label: &str, expect_drops: usize, expect_pf: usize) {
  let (d, pf, late, notpause) =
    (DROPS.load(SeqCst), PREFIN_RAN.load(SeqCst), PREFIN_AFTER_DROP.load(SeqCst), PREFIN_NOT_IN_ATOMIC_PAUSE.load(SeqCst));
  println!(
    "{label}: drops={d} prefinalizers_run={pf} ran_after_a_destructor={late} outside_atomic_pause={notpause} partner_alive={} partner_dead={}",
    PARTNER_ALIVE.load(SeqCst),
    PARTNER_DEAD.load(SeqCst)
  );
  assert_eq!(d, expect_drops, "{label}: drops");
  assert_eq!(pf, expect_pf, "{label}: prefinalizers");
  assert_eq!(late, 0, "{label}: a prefinalizer ran after a destructor of the same cycle");
  assert_eq!(notpause, 0, "{label}: prefinalizer outside the atomic pause");
}

#[test]
fn item4_prefinalizers_detached_incremental() {
  let (p, _g) = init();
  let heap = detached_heap(p);
  HEAP.with(|h| h.set(&*heap));
  reset();
  let mut keep = alloc_elems(&heap);
  let mut th = unsafe { StandaloneTestingHeap::new(&heap) };
  DROPS_AT_GC_START.store(DROPS.load(SeqCst), SeqCst);
  th.start_garbage_collection();
  while !th.perform_marking_step(EmbedderStackState::NoHeapPointers) {}
  th.finalize_garbage_collection(EmbedderStackState::NoHeapPointers);
  check("detached half-collected", N / 2, PREFIN_REGISTERED / 2);
  // every dead registered element (odd) saw its partner (anchor) alive
  assert_eq!(PARTNER_ALIVE.load(SeqCst), PREFIN_REGISTERED / 2);
  keep.clear();
  DROPS_AT_GC_START.store(DROPS.load(SeqCst), SeqCst);
  precise_gc(&heap);
  check("detached all-collected", N + 1, PREFIN_REGISTERED);
  // second GC: the anchor died in the same cycle as the remaining elements
  assert_eq!(PARTNER_DEAD.load(SeqCst), PREFIN_REGISTERED / 2);
  HEAP.with(|h| h.set(std::ptr::null()));
}

#[test]
fn item4_prefinalizers_unified_precise_gc() {
  let (_p, _g) = init();
  let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
  let heap: &Heap = unsafe { &*(isolate.get_cpp_heap().unwrap() as *const Heap) };
  HEAP.with(|h| h.set(heap));
  reset();
  let mut keep = alloc_elems(heap);
  DROPS_AT_GC_START.store(DROPS.load(SeqCst), SeqCst);
  isolate.request_garbage_collection_for_testing_with_stack_state(v8::GarbageCollectionType::Full, EmbedderStackState::NoHeapPointers);
  check("unified half-collected", N / 2, PREFIN_REGISTERED / 2);
  keep.clear();
  DROPS_AT_GC_START.store(DROPS.load(SeqCst), SeqCst);
  isolate.request_garbage_collection_for_testing_with_stack_state(v8::GarbageCollectionType::Full, EmbedderStackState::NoHeapPointers);
  check("unified all-collected", N + 1, PREFIN_REGISTERED);
  assert_eq!(PARTNER_ALIVE.load(SeqCst), PREFIN_REGISTERED / 2);
  assert_eq!(PARTNER_DEAD.load(SeqCst), PREFIN_REGISTERED / 2);
  HEAP.with(|h| h.set(std::ptr::null()));
}

/// Registering after allocation (register_prefinalizer) works the same.
#[test]
fn register_prefinalizer_after_allocation() {
  let (p, _g) = init();
  let heap = detached_heap(p);
  HEAP.with(|h| h.set(std::ptr::null()));
  reset();
  for i in 0..10 {
    let e = unsafe { v8::cppgc::make_garbage_collected(&heap, Elem { idx: 2 * i + 1, partner: Member::empty(), _data: Box::new(vec![]) }) };
    unsafe { register_prefinalizer(&e) };
  }
  precise_gc(&heap);
  assert_eq!(PREFIN_RAN.load(SeqCst), 10);
  assert_eq!(DROPS.load(SeqCst), 10);
}

/// 16-byte aligned types use rusty_v8's RustObjButAlign16 path; the value
/// offset must still be found.
#[test]
fn prefinalizer_on_16_byte_aligned_type() {
  static RAN: AtomicUsize = AtomicUsize::new(0);
  #[repr(align(16))]
  struct Aligned(u128);
  unsafe impl GarbageCollected for Aligned {
    fn trace(&self, _: &mut Visitor) {}
    fn get_name(&self) -> &'static std::ffi::CStr {
      c"Aligned"
    }
  }
  impl PreFinalizer for Aligned {
    fn pre_finalize(&self, _: &LivenessBroker) {
      assert_eq!(self.0, 0xfeed_f00d_dead_beef_0123_4567_89ab_cdef);
      RAN.fetch_add(1, SeqCst);
    }
  }
  let (p, _g) = init();
  let heap = detached_heap(p);
  let keep = Persistent::new(&unsafe { make_garbage_collected_with_prefinalizer(&heap, Aligned(0xfeed_f00d_dead_beef_0123_4567_89ab_cdef)) });
  for _ in 0..5 {
    unsafe { make_garbage_collected_with_prefinalizer(&heap, Aligned(0xfeed_f00d_dead_beef_0123_4567_89ab_cdef)) };
  }
  precise_gc(&heap);
  assert_eq!(RAN.load(SeqCst), 5);
  drop(keep);
  precise_gc(&heap);
  assert_eq!(RAN.load(SeqCst), 6);
}

// ---- custom weak callbacks ----

/// Holds an *untraced* pointer and clears it in a weak callback once the
/// target dies (the C++ `UntracedMember` + `RegisterWeakCallbackMethod`
/// pattern).
struct WeakHolder {
  /// Untraced: the raw cppgc object pointer (null once cleared).
  target: Cell<*const std::ffi::c_void>,
  callbacks: Cell<usize>,
}
struct Target;
static TARGET_DROPS: AtomicUsize = AtomicUsize::new(0);
unsafe impl GarbageCollected for Target {
  fn trace(&self, _: &mut Visitor) {}
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"Target"
  }
}
impl Drop for Target {
  fn drop(&mut self) {
    TARGET_DROPS.fetch_add(1, SeqCst);
  }
}
unsafe impl GarbageCollected for WeakHolder {
  fn trace(&self, v: &mut Visitor) {
    v.register_weak_callback_method(self);
  }
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"WeakHolder"
  }
}
impl WeakCallbackMethod for WeakHolder {
  fn weak_callback(&self, broker: &LivenessBroker) {
    self.callbacks.set(self.callbacks.get() + 1);
    let t = self.target.get();
    if !t.is_null() && !unsafe { broker.is_heap_object_alive_raw(t) } {
      self.target.set(std::ptr::null());
    }
  }
}

#[test]
fn weak_callback_clears_dead_untraced_pointer() {
  let (p, _g) = init();
  let heap = detached_heap(p);
  TARGET_DROPS.store(0, SeqCst);
  let target = unsafe { v8::cppgc::make_garbage_collected(&heap, Target) };
  let target_root = Persistent::new(&target);
  let holder = Persistent::new(&unsafe {
    v8::cppgc::make_garbage_collected(&heap, WeakHolder { target: Cell::new(heap_consistency::object_ptr(&target)), callbacks: Cell::new(0) })
  });
  precise_gc(&heap);
  let h = holder.get().unwrap();
  assert_eq!(h.callbacks.get(), 1);
  assert!(!h.target.get().is_null(), "alive target must be kept");
  drop(target_root);
  precise_gc(&heap);
  assert_eq!(h.callbacks.get(), 2);
  assert!(h.target.get().is_null(), "dead target must be cleared by the weak callback");
  assert_eq!(TARGET_DROPS.load(SeqCst), 1);
}
