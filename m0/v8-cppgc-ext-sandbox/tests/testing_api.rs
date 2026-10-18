//! cppgc::testing: StandaloneTestingHeap, OverrideEmbedderStackStateScope,
//! IsHeapObjectOld; plus the 2-argument Isolate::RequestGarbageCollectionForTesting.

mod common;
use common::*;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use v8::cppgc::{EmbedderStackState, Heap, Persistent};
use v8_cppgc_ext::testing::{OverrideEmbedderStackStateScope, StandaloneTestingHeap, is_heap_object_old};
use v8_cppgc_ext::*;

#[test]
fn standalone_testing_heap_steps_and_toggles() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  let (p, _g) = init();
  let heap = detached_heap(p);
  let h = heap.heap_handle();
  let root = Persistent::new(&node(&heap, 0, &DROPS));
  for i in 0..20_000 {
    node(&heap, i, &DROPS);
  }
  let mut th = unsafe { StandaloneTestingHeap::new(&heap) };
  th.start_garbage_collection();
  assert!(HeapState::is_marking(h));
  th.start_garbage_collection(); // no-op while marking
  th.toggle_main_thread_marking(false);
  th.perform_marking_step(EmbedderStackState::NoHeapPointers);
  th.toggle_main_thread_marking(true);
  let mut steps = 1;
  while !th.perform_marking_step(EmbedderStackState::NoHeapPointers) {
    steps += 1;
  }
  assert_eq!(DROPS.load(SeqCst), 0, "nothing is swept before finalization");
  th.finalize_garbage_collection(EmbedderStackState::NoHeapPointers);
  println!("marking steps: {steps}");
  assert_eq!(DROPS.load(SeqCst), 20_000);
  assert!(!HeapState::is_marking(h));
  drop(root);
}

#[test]
#[should_panic(expected = "no GC in progress")]
fn marking_step_without_gc_panics_instead_of_crashing() {
  let (p, _g) = init();
  let heap = detached_heap(p);
  let mut th = unsafe { StandaloneTestingHeap::new(&heap) };
  th.perform_marking_step(EmbedderStackState::NoHeapPointers);
}

/// Forced compaction cannot move rusty_v8 objects: no compactable spaces.
/// (Debug V8 DCHECKs in Compactor::ShouldCompact for exactly that reason, so
/// this test only runs against the release prebuilt.)
#[test]
fn forced_compaction_does_not_move_objects() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  if v8_cppgc_ext::abi::DEBUG_V8 {
    println!("skipped on debug V8 (DCHECK(!enable_for_next_gc_for_testing_))");
    return;
  }
  let (p, _g) = init();
  let heap = detached_heap(p);
  heap.assert_no_compactable_spaces();
  let keep: Vec<_> = (0..5000).map(|i| Persistent::new(&node(&heap, i, &DROPS))).collect();
  for i in 0..5000 {
    node(&heap, i, &DROPS); // fragmentation
  }
  let before: Vec<usize> = keep.iter().map(|p| p.get().unwrap() as *const _ as usize).collect();
  let mut th = unsafe { StandaloneTestingHeap::new(&heap) };
  for _ in 0..3 {
    th.force_compaction_for_next_garbage_collection();
    th.start_garbage_collection();
    while !th.perform_marking_step(EmbedderStackState::NoHeapPointers) {}
    th.finalize_garbage_collection(EmbedderStackState::NoHeapPointers);
  }
  let after: Vec<usize> = keep.iter().map(|p| p.get().unwrap() as *const _ as usize).collect();
  assert_eq!(before, after);
  assert_eq!(DROPS.load(SeqCst), 5000);
}

/// The override applies to GCs V8 runs for an *attached* CppHeap (it becomes
/// an EmbedderStackStateScope on V8's heap). For a detached heap V8 only
/// remembers it until the heap is attached (cpp-heap.cc), so the detached
/// testing GC keeps using its explicit argument.
#[test]
fn override_embedder_stack_state_scope_makes_gc_precise() {
  let (_p, _g) = init();
  let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
  let h = unsafe { HeapHandle::from_isolate_unbounded(isolate) }.unwrap();
  // rusty_v8's 1-argument testing GC scans the stack conservatively...
  isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
  assert!(HeapState::previous_gc_was_conservative(h));
  {
    // ...unless the stack state is overridden.
    let _o = OverrideEmbedderStackStateScope::new(h, EmbedderStackState::NoHeapPointers);
    isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
    assert!(!HeapState::previous_gc_was_conservative(h), "override ignored");
  }
  isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
  assert!(HeapState::previous_gc_was_conservative(h));
}

#[test]
fn is_heap_object_old_follows_mark_bits() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  let (p, _g) = init();
  let heap = detached_heap(p);
  let n = node(&heap, 1, &DROPS);
  let root = Persistent::new(&n);
  assert!(!is_heap_object_old(&root), "fresh object is young (unmarked)");
  let mut th = unsafe { StandaloneTestingHeap::new(&heap) };
  th.start_garbage_collection();
  while !th.perform_marking_step(EmbedderStackState::NoHeapPointers) {}
  assert!(is_heap_object_old(&root), "reachable object is marked once marking is done");
  th.finalize_garbage_collection(EmbedderStackState::NoHeapPointers);
  println!("after full GC (young generation not enabled at runtime): old={}", is_heap_object_old(&root));
}

/// Item 3's finding: the 1-argument testing GC scans the stack conservatively;
/// the 2-argument overload with kNoHeapPointers is precise.
#[test]
fn isolate_testing_gc_with_explicit_stack_state() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  let (_p, _g) = init();
  let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
  let heap: &Heap = unsafe { &*(isolate.get_cpp_heap().unwrap() as *const Heap) };
  let h = unsafe { HeapHandle::from_isolate_unbounded(isolate) }.unwrap();

  // Conservative: the pointer is on our stack, the object must survive.
  let raw = std::hint::black_box(node(heap, 7, &DROPS));
  isolate.request_garbage_collection_for_testing_with_stack_state(
    v8::GarbageCollectionType::Full,
    EmbedderStackState::MayContainHeapPointers,
  );
  assert!(HeapState::previous_gc_was_conservative(h));
  assert_eq!(DROPS.load(SeqCst), 0, "conservative GC must keep stack-referenced object");
  assert_eq!(unsafe { raw.as_ref() }.id, 7);

  // Precise: same pointer still on the stack, but the GC ignores it.
  isolate.request_garbage_collection_for_testing_with_stack_state(
    v8::GarbageCollectionType::Full,
    EmbedderStackState::NoHeapPointers,
  );
  assert!(!HeapState::previous_gc_was_conservative(h));
  assert_eq!(DROPS.load(SeqCst), 1, "precise GC must collect the unreachable object");

  // 1-argument rusty_v8 API is conservative.
  isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
  assert!(HeapState::previous_gc_was_conservative(h));
  // Minor GC path also accepts the stack state.
  isolate.request_garbage_collection_for_testing_with_stack_state(
    v8::GarbageCollectionType::Minor,
    EmbedderStackState::NoHeapPointers,
  );
}
