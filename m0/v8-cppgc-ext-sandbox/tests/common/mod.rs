//! Shared test support: one V8/cppgc initialization per test binary, and a
//! lock so tests in one binary never use V8 concurrently.
#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use v8::cppgc::{GarbageCollected, Heap, HeapCreateParams, Member, Visitor};

static PLATFORM: OnceLock<v8::SharedRef<v8::Platform>> = OnceLock::new();
static LOCK: Mutex<()> = Mutex::new(());

pub const FLAGS: &str = "--expose-gc";

pub fn init() -> (v8::SharedRef<v8::Platform>, MutexGuard<'static, ()>) {
  let g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
  let p = PLATFORM
    .get_or_init(|| {
      v8::V8::set_flags_from_string(FLAGS);
      let platform = v8::new_default_platform(0, false).make_shared();
      v8::V8::initialize_platform(platform.clone());
      v8::cppgc::initialize_process(platform.clone());
      v8::V8::initialize();
      platform
    })
    .clone();
  (p, g)
}

/// A detached heap with testing GCs enabled.
pub fn detached_heap(platform: v8::SharedRef<v8::Platform>) -> v8::UniqueRef<Heap> {
  let heap = Heap::create(platform, HeapCreateParams::default());
  heap.enable_detached_garbage_collections_for_testing();
  heap
}

/// Simple GC type with a strong edge and a drop counter.
pub struct Node {
  pub id: u32,
  pub next: Member<Node>,
  pub drops: &'static AtomicUsize,
}

unsafe impl GarbageCollected for Node {
  fn trace(&self, v: &mut Visitor) {
    v.trace(&self.next);
  }
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"V8CppgcExtTestNode"
  }
}

impl Drop for Node {
  fn drop(&mut self) {
    self.drops.fetch_add(1, Ordering::SeqCst);
  }
}

pub fn node(heap: &Heap, id: u32, drops: &'static AtomicUsize) -> v8::cppgc::UnsafePtr<Node> {
  unsafe { v8::cppgc::make_garbage_collected(heap, Node { id, next: Member::empty(), drops }) }
}

pub fn precise_gc(heap: &Heap) {
  unsafe { heap.collect_garbage_for_testing(v8::cppgc::EmbedderStackState::NoHeapPointers) }
}
