//! Smoke tests of the patched rusty_v8 API (v8::cppgc::*), one per feature.

// Links the shim that provides the patch's new C++ functions.
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::{Mutex, OnceLock};
use v8::cppgc::heap_consistency::*;
use v8::cppgc::testing::*;
use v8::cppgc::*;

static PLATFORM: OnceLock<v8::SharedRef<v8::Platform>> = OnceLock::new();
static LOCK: Mutex<()> = Mutex::new(());

fn init() -> (v8::SharedRef<v8::Platform>, std::sync::MutexGuard<'static, ()>) {
  let g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
  let p = PLATFORM
    .get_or_init(|| {
      v8::V8::set_flags_from_string("--expose-gc");
      let p = v8::new_default_platform(0, false).make_shared();
      v8::V8::initialize_platform(p.clone());
      initialize_process(p.clone());
      v8::V8::initialize();
      p
    })
    .clone();
  (p, g)
}

fn heap(p: v8::SharedRef<v8::Platform>) -> v8::UniqueRef<Heap> {
  let h = Heap::create(p, HeapCreateParams::default());
  h.enable_detached_garbage_collections_for_testing();
  h
}

static DROPS: AtomicUsize = AtomicUsize::new(0);
static PREFIN: AtomicUsize = AtomicUsize::new(0);
struct N {
  next: Member<N>,
}
unsafe impl GarbageCollected for N {
  fn trace(&self, v: &mut Visitor) {
    v.trace(&self.next);
    v.register_weak_callback_method(self);
  }
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"UpstreamN"
  }
}
impl Drop for N {
  fn drop(&mut self) {
    DROPS.fetch_add(1, SeqCst);
  }
}
impl PreFinalizer for N {
  fn pre_finalize(&self, broker: &LivenessBroker) {
    let _ = broker.is_heap_object_alive(&self.next);
    PREFIN.fetch_add(1, SeqCst);
  }
}
static WEAK: AtomicUsize = AtomicUsize::new(0);
impl WeakCallbackMethod for N {
  fn weak_callback(&self, _: &LivenessBroker) {
    WEAK.fetch_add(1, SeqCst);
  }
}

fn n(h: &Heap) -> UnsafePtr<N> {
  unsafe { make_garbage_collected(h, N { next: Member::empty() }) }
}

#[test]
fn upstream_everything() {
  let (p, _g) = init();
  let h = heap(p);
  let hh = h.heap_handle();
  DROPS.store(0, SeqCst);

  // scopes + heap state
  for _ in 0..100 {
    n(&h);
  }
  {
    let _s = NoGarbageCollectionScope::new(hh);
    let _d = DisallowGarbageCollectionScope::new(hh);
    assert!(!DisallowGarbageCollectionScope::is_garbage_collection_allowed(hh));
  }
  unsafe { NoGarbageCollectionScope::enter(hh) };
  unsafe { h.collect_garbage_for_testing(EmbedderStackState::NoHeapPointers) };
  assert_eq!(DROPS.load(SeqCst), 0);
  unsafe { NoGarbageCollectionScope::leave(hh) };
  unsafe { h.collect_garbage_for_testing(EmbedderStackState::NoHeapPointers) };
  assert_eq!(DROPS.load(SeqCst), 100);
  assert!(!HeapState::previous_gc_was_conservative(hh));

  // statistics
  let keep: Vec<_> = (0..50).map(|_| Persistent::new(&n(&h))).collect();
  let s = h.collect_statistics(DetailLevel::Detailed);
  let idx = s.type_names.iter().position(|t| t == "UpstreamN").unwrap();
  let count: usize =
    s.space_stats.iter().flat_map(|x| &x.page_stats).filter_map(|p| p.object_statistics.get(idx)).map(|e| e.object_count).sum();
  assert_eq!(count, 50);
  assert_eq!(h.collect_statistics(DetailLevel::Brief).detail_level, DetailLevel::Brief);
  h.assert_no_compactable_spaces();
  drop(keep);

  // prefinalizer + weak callback
  PREFIN.store(0, SeqCst);
  WEAK.store(0, SeqCst);
  let root = Persistent::new(&unsafe { make_garbage_collected_with_prefinalizer(&*h, N { next: Member::empty() }) });
  for _ in 0..10 {
    unsafe { make_garbage_collected_with_prefinalizer(&*h, N { next: Member::empty() }) };
  }
  unsafe { h.collect_garbage_for_testing(EmbedderStackState::NoHeapPointers) };
  assert_eq!(PREFIN.load(SeqCst), 10);
  assert!(WEAK.load(SeqCst) >= 1);

  // testing heap + barriers
  let a = Persistent::new(&n(&h));
  let b = n(&h);
  let slot = &a.get().unwrap().next as *const Member<N> as *const std::ffi::c_void;
  let mut th = unsafe { StandaloneTestingHeap::new(&h) };
  th.start_garbage_collection();
  th.toggle_main_thread_marking(true);
  while !th.perform_marking_step(EmbedderStackState::NoHeapPointers) {}
  assert!(is_heap_object_old(&a));
  assert_eq!(unsafe { get_write_barrier_type(slot, object_ptr(&b)) }, WriteBarrierType::Marking);
  assert!(unsafe { dijkstra_write_barrier(slot, object_ptr(&b)) });
  assert!(is_heap_object_old(&b));
  th.finalize_garbage_collection(EmbedderStackState::NoHeapPointers);
  drop(root);
}

#[test]
fn upstream_isolate_gc_and_override() {
  let (_p, _g) = init();
  let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
  let hh = unsafe { HeapHandle::from_isolate_unbounded(isolate) }.unwrap();
  isolate.request_garbage_collection_for_testing_with_stack_state(
    v8::GarbageCollectionType::Full,
    EmbedderStackState::NoHeapPointers,
  );
  assert!(!HeapState::previous_gc_was_conservative(hh));
  isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
  assert!(HeapState::previous_gc_was_conservative(hh));
  {
    let _o = OverrideEmbedderStackStateScope::new(hh, EmbedderStackState::NoHeapPointers);
    isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
    assert!(!HeapState::previous_gc_was_conservative(hh));
  }
}
