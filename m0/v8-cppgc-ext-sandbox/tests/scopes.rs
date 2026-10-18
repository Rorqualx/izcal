//! NoGarbageCollectionScope, DisallowGarbageCollectionScope, HeapState.

mod common;
use common::*;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use v8::cppgc::EmbedderStackState;
use v8_cppgc_ext::testing::StandaloneTestingHeap;
use v8_cppgc_ext::*;

#[test]
fn no_gc_scope_defers_cppgc_gc() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  let (p, _g) = init();
  let heap = detached_heap(p);
  let h = heap.heap_handle();
  for i in 0..1000 {
    node(&heap, i, &DROPS);
  }
  {
    let _s = NoGarbageCollectionScope::new(h);
    for _ in 0..10 {
      precise_gc(&heap);
    }
    assert_eq!(DROPS.load(SeqCst), 0, "GC ran inside NoGarbageCollectionScope");
    // nested + static forms
    let _s2 = NoGarbageCollectionScope::new(h);
    unsafe { NoGarbageCollectionScope::enter(h) };
    precise_gc(&heap);
    unsafe { NoGarbageCollectionScope::leave(h) };
    assert_eq!(DROPS.load(SeqCst), 0);
  }
  precise_gc(&heap);
  assert_eq!(DROPS.load(SeqCst), 1000, "garbage not collected after leaving the scope");
}

#[test]
fn static_no_gc_enter_leave_balance() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  let (p, _g) = init();
  let heap = detached_heap(p);
  let h = heap.heap_handle();
  node(&heap, 1, &DROPS);
  unsafe { NoGarbageCollectionScope::enter(h) };
  precise_gc(&heap);
  assert_eq!(DROPS.load(SeqCst), 0);
  unsafe { NoGarbageCollectionScope::leave(h) };
  precise_gc(&heap);
  assert_eq!(DROPS.load(SeqCst), 1);
}

#[test]
fn disallow_scope_reports_gc_forbidden() {
  let (p, _g) = init();
  let heap = detached_heap(p);
  let h = heap.heap_handle();
  assert!(DisallowGarbageCollectionScope::is_garbage_collection_allowed(h));
  {
    let _d = DisallowGarbageCollectionScope::new(h);
    assert!(!DisallowGarbageCollectionScope::is_garbage_collection_allowed(h));
    {
      let _d2 = DisallowGarbageCollectionScope::new(h);
      assert!(!DisallowGarbageCollectionScope::is_garbage_collection_allowed(h));
    }
    assert!(!DisallowGarbageCollectionScope::is_garbage_collection_allowed(h));
  }
  assert!(DisallowGarbageCollectionScope::is_garbage_collection_allowed(h));
  unsafe { DisallowGarbageCollectionScope::enter(h) };
  assert!(!DisallowGarbageCollectionScope::is_garbage_collection_allowed(h));
  unsafe { DisallowGarbageCollectionScope::leave(h) };
  assert!(DisallowGarbageCollectionScope::is_garbage_collection_allowed(h));
  // NoGC does not forbid (it defers).
  let _n = NoGarbageCollectionScope::new(h);
  assert!(DisallowGarbageCollectionScope::is_garbage_collection_allowed(h));
}

#[test]
fn heap_state_tracks_incremental_gc() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  let (p, _g) = init();
  let heap = detached_heap(p);
  let h = heap.heap_handle();
  let root = v8::cppgc::Persistent::new(&node(&heap, 0, &DROPS));
  for i in 0..5000 {
    node(&heap, i, &DROPS);
  }
  assert!(!HeapState::is_marking(h));
  assert!(!HeapState::is_sweeping(h));
  assert!(!HeapState::is_in_atomic_pause(h));
  let mut th = unsafe { StandaloneTestingHeap::new(&heap) };
  th.start_garbage_collection();
  assert!(HeapState::is_marking(h));
  assert!(!HeapState::is_in_atomic_pause(h));
  // A no-GC scope entered during marking does not stop marking.
  {
    let _s = NoGarbageCollectionScope::new(h);
    th.perform_marking_step(EmbedderStackState::NoHeapPointers);
    assert!(HeapState::is_marking(h));
  }
  while !th.perform_marking_step(EmbedderStackState::NoHeapPointers) {}
  th.finalize_garbage_collection(EmbedderStackState::NoHeapPointers);
  assert!(!HeapState::is_marking(h));
  assert!(!HeapState::is_sweeping(h), "FinalizeGarbageCollection finishes sweeping");
  assert!(!HeapState::previous_gc_was_conservative(h));
  assert_eq!(DROPS.load(SeqCst), 5000);

  // Conservative (stack-scanning) GC is reported as such.
  unsafe { heap.collect_garbage_for_testing(EmbedderStackState::MayContainHeapPointers) };
  assert!(HeapState::previous_gc_was_conservative(h));
  precise_gc(&heap);
  assert!(!HeapState::previous_gc_was_conservative(h));
  drop(root);
}

/// Destructors run during sweeping on the owning thread: HeapState sees it.
#[test]
fn heap_state_inside_destructor() {
  use std::cell::Cell;
  thread_local! {
    static HANDLE: Cell<*mut std::ffi::c_void> = const { Cell::new(std::ptr::null_mut()) };
    static SEEN: Cell<(bool, bool, bool)> = const { Cell::new((false, false, false)) };
  }
  struct Probe;
  unsafe impl v8::cppgc::GarbageCollected for Probe {
    fn trace(&self, _: &mut v8::cppgc::Visitor) {}
    fn get_name(&self) -> &'static std::ffi::CStr {
      c"Probe"
    }
  }
  impl Drop for Probe {
    fn drop(&mut self) {
      // Re-derive a handle from the raw pointer stashed by the test body.
      let raw = HANDLE.with(|h| h.get());
      let heap: &v8::cppgc::Heap = unsafe { &*(raw as *const v8::cppgc::Heap) };
      let h = heap.heap_handle();
      SEEN.with(|s| {
        s.set((HeapState::is_sweeping(h), HeapState::is_sweeping_on_owning_thread(h), HeapState::is_marking(h)))
      });
    }
  }
  let (p, _g) = init();
  let heap = detached_heap(p);
  HANDLE.with(|h| h.set(&*heap as *const v8::cppgc::Heap as *mut _));
  unsafe { v8::cppgc::make_garbage_collected(&heap, Probe) };
  precise_gc(&heap);
  let (sweeping, on_owner, marking) = SEEN.with(|s| s.get());
  assert!(sweeping && on_owner && !marking, "in Drop: sweeping={sweeping} on_owner={on_owner} marking={marking}");
}
