//! cppgc::subtle::HeapConsistency write barriers (port of the spike's item 5
//! scenario: a reference stored behind the marker's back).

mod common;
use common::*;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use v8::cppgc::{EmbedderStackState, GarbageCollected, Heap, Member, Persistent, UnsafePtr, Visitor};
use v8_cppgc_ext::heap_consistency::*;
use v8_cppgc_ext::testing::{StandaloneTestingHeap, is_heap_object_old};

static DROPS: AtomicUsize = AtomicUsize::new(0);

/// Like the spike's TNode: the Member slot is in an UnsafeCell because it is
/// written through a shared reference.
struct CNode {
  id: u32,
  next: UnsafeCell<Member<CNode>>,
}
unsafe impl GarbageCollected for CNode {
  fn trace(&self, v: &mut Visitor) {
    v.trace(unsafe { &*self.next.get() });
  }
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"CNode"
  }
}
impl Drop for CNode {
  fn drop(&mut self) {
    DROPS.fetch_add(1, SeqCst);
  }
}
fn cnode(heap: &Heap, id: u32) -> UnsafePtr<CNode> {
  unsafe { v8::cppgc::make_garbage_collected(heap, CNode { id, next: UnsafeCell::new(Member::empty()) }) }
}

/// Stores `b` into `a.next` by copying the compressed bits, i.e. without
/// Member::operator= and therefore without any barrier.
unsafe fn store_without_barrier(a: &CNode, b: &UnsafePtr<CNode>) {
  let tmp = std::mem::ManuallyDrop::new(Member::<CNode>::new(b));
  unsafe {
    std::ptr::copy_nonoverlapping(
      &*tmp as *const Member<CNode> as *const u8,
      a.next.get() as *mut u8,
      std::mem::size_of::<Member<CNode>>(),
    )
  };
}

/// Returns (B finalized, A.next still points at B).
fn scenario(run_barrier: bool) -> (bool, bool) {
  let (p, _g) = init();
  DROPS.store(0, SeqCst);
  let heap = detached_heap(p);
  let mut th = unsafe { StandaloneTestingHeap::new(&heap) };
  let a = Persistent::new(&cnode(&heap, 1));
  // B is only referenced from Rust heap memory (invisible to a precise GC).
  let b = Box::new(cnode(&heap, 2));
  let a_ref = a.get().unwrap();
  let slot = a_ref.next.get() as *const std::ffi::c_void;

  assert_eq!(unsafe { get_write_barrier_type(slot, object_ptr(&*b)) }, WriteBarrierType::None);
  th.start_garbage_collection();
  while !th.perform_marking_step(EmbedderStackState::NoHeapPointers) {}
  // A is black now; storing white B into it needs a marking barrier.
  assert!(is_heap_object_old(&a) && !is_heap_object_old(&*b));
  assert_eq!(unsafe { get_write_barrier_type(slot, object_ptr(&*b)) }, WriteBarrierType::Marking);
  unsafe { store_without_barrier(a_ref, &b) };
  assert_eq!(unsafe { (*a_ref.next.get()).get() }.map(|n| n.id), Some(2));
  if run_barrier {
    assert!(unsafe { dijkstra_write_barrier(slot, object_ptr(&*b)) });
    assert!(is_heap_object_old(&*b), "Dijkstra barrier must mark B");
    // young generation is compiled in but not enabled at runtime
    assert!(!unsafe { generational_barrier(slot, object_ptr(&*b)) });
  }
  drop(b);
  th.finalize_garbage_collection(EmbedderStackState::NoHeapPointers);
  let b_finalized = DROPS.load(SeqCst) > 0;
  let still_points = unsafe { (*a_ref.next.get()).get() }.is_some();
  if !b_finalized {
    // B must stay alive across further GCs through A.
    precise_gc(&heap);
    assert_eq!(DROPS.load(SeqCst), 0);
    assert_eq!(unsafe { (*a_ref.next.get()).get() }.unwrap().id, 2);
  }
  // Clear the (possibly dangling) slot before teardown.
  unsafe { std::ptr::write_bytes(a_ref.next.get() as *mut u8, 0, std::mem::size_of::<Member<CNode>>()) };
  (b_finalized, still_points)
}

#[test]
fn dijkstra_barrier_keeps_late_stored_object_alive() {
  assert_eq!(scenario(true), (false, true), "B was collected despite the barrier");
}

/// Control: without the barrier the release prebuilt collects B while A still
/// points at it (the spike's "MISSED" outcome). Debug V8's marking verifier
/// aborts instead, so this control only runs against the release prebuilt.
#[test]
fn control_without_barrier_loses_object() {
  if v8_cppgc_ext::abi::DEBUG_V8 {
    println!("skipped on debug V8 (MarkingVerifier aborts by design)");
    return;
  }
  assert_eq!(scenario(false), (true, true), "control: B should have been collected without a barrier");
}

/// Steele barrier: re-trace an already-marked object whose field changed.
#[test]
fn steele_barrier_retraces_marked_object() {
  let (p, _g) = init();
  DROPS.store(0, SeqCst);
  let heap = detached_heap(p);
  let mut th = unsafe { StandaloneTestingHeap::new(&heap) };
  let a = Persistent::new(&cnode(&heap, 1));
  let b = Box::new(cnode(&heap, 2));
  let a_ref = a.get().unwrap();
  th.start_garbage_collection();
  while !th.perform_marking_step(EmbedderStackState::NoHeapPointers) {}
  unsafe { store_without_barrier(a_ref, &b) };
  assert!(unsafe { steele_write_barrier(object_ptr(&a)) });
  drop(b);
  th.finalize_garbage_collection(EmbedderStackState::NoHeapPointers);
  assert_eq!(DROPS.load(SeqCst), 0, "B must survive via the re-traced A");
  unsafe { std::ptr::write_bytes(a_ref.next.get() as *mut u8, 0, std::mem::size_of::<Member<CNode>>()) };
}

#[test]
fn barriers_outside_gc_are_noops() {
  let (p, _g) = init();
  let heap = detached_heap(p);
  let a = Persistent::new(&cnode(&heap, 1));
  let b = cnode(&heap, 2);
  let slot = a.get().unwrap().next.get() as *const std::ffi::c_void;
  unsafe { write_barrier(slot, object_ptr(&b)) };
  assert!(!unsafe { dijkstra_write_barrier(slot, object_ptr(&b)) });
  assert!(!unsafe { steele_write_barrier(object_ptr(&a)) });
  assert!(!unsafe { generational_barrier(slot, object_ptr(&b)) });
}
