//! Thin layer over the HGBA rusty_v8 fork (feature `rusty_v8_fork`).
//!
//! Every type is the fork's own (`v8::cppgc::*`), so values flow freely
//! between code written against this crate and code written against the fork.
//! The extension traits remain for source compatibility; where the fork has an
//! inherent method of the same name, the inherent method wins at call sites
//! and the trait method forwards to it.

use std::ffi::c_void;

use v8::cppgc::{EmbedderStackState, GarbageCollected, GetRustObj, Heap, UnsafePtr, Visitor};

pub use v8::cppgc::{
  DetailLevel, DisallowGarbageCollectionScope, FreeListStatistics, HeapHandle, HeapState, HeapStatistics,
  LivenessBroker, NoGarbageCollectionScope, ObjectStatsEntry, PageStatistics, PreFinalizer, SpaceStatistics,
  WeakCallbackMethod, heap_consistency, make_garbage_collected_with_prefinalizer, register_prefinalizer, testing,
};

mod ffi {
  use super::*;
  unsafe extern "C" {
    // Provided by the fork's binding.cc (same C ABI as the standalone shim).
    pub fn cppgc__Visitor__RegisterWeakCallback(
      v: *mut Visitor,
      cb: unsafe extern "C" fn(&LivenessBroker, *const c_void),
      data: *const c_void,
    );
    pub fn cppgc__LivenessBroker__IsHeapObjectAlive(b: *const LivenessBroker, obj: *const c_void) -> bool;
    // Provided by this crate's shim (ABI checks only).
    pub fn v8_cppgc_ext__V8__Initialize_with_shim_build_config() -> bool;
    pub fn v8_cppgc_ext__shim_build_configuration() -> i32;
    pub fn v8_cppgc_ext__Member__assign_with_shim(slot: *mut c_void, obj: *mut c_void);
    pub fn v8_cppgc_ext__Member__construct_with_shim(slot: *mut c_void, obj: *mut c_void);
    pub fn v8_cppgc_ext__Member__get_with_shim(slot: *const c_void) -> *mut c_void;
    pub fn v8_cppgc_ext__Member__destruct_with_shim(slot: *mut c_void);
    pub fn v8_cppgc_ext__sizeof_Member() -> usize;
  }
}

fn rust_obj_ptr<T: GarbageCollected>(p: &impl GetRustObj<T>) -> *mut c_void {
  p.get_rust_obj() as *mut c_void
}

/// Extension methods for [`v8::cppgc::Heap`]; all are inherent in the fork.
pub trait CppHeapExt {
  fn heap_handle(&self) -> HeapHandle<'_>;
  fn collect_statistics(&self, detail_level: DetailLevel) -> HeapStatistics;
  fn assert_no_compactable_spaces(&self);
}

impl CppHeapExt for Heap {
  fn heap_handle(&self) -> HeapHandle<'_> {
    Heap::heap_handle(self)
  }
  fn collect_statistics(&self, detail_level: DetailLevel) -> HeapStatistics {
    Heap::collect_statistics(self, detail_level)
  }
  fn assert_no_compactable_spaces(&self) {
    Heap::assert_no_compactable_spaces(self)
  }
}

/// Extension methods for [`v8::cppgc::Visitor`].
pub trait VisitorExt {
  fn register_weak_callback_method<T: WeakCallbackMethod>(&mut self, object: &T);
  /// `Visitor::RegisterWeakCallback(callback, data)` (not wrapped by the fork).
  ///
  /// # Safety
  /// `data` must stay valid until the callback has run in this GC cycle.
  unsafe fn register_weak_callback(
    &mut self,
    callback: unsafe extern "C" fn(&LivenessBroker, *const c_void),
    data: *const c_void,
  );
}

impl VisitorExt for Visitor {
  fn register_weak_callback_method<T: WeakCallbackMethod>(&mut self, object: &T) {
    Visitor::register_weak_callback_method(self, object)
  }
  unsafe fn register_weak_callback(
    &mut self,
    callback: unsafe extern "C" fn(&LivenessBroker, *const c_void),
    data: *const c_void,
  ) {
    unsafe { ffi::cppgc__Visitor__RegisterWeakCallback(self, callback, data) }
  }
}

/// Extension methods for [`LivenessBroker`] that the fork does not wrap.
pub trait LivenessBrokerExt {
  /// `LivenessBroker::IsHeapObjectAlive` on a raw object pointer.
  ///
  /// # Safety
  /// Only inside a prefinalizer or weak callback; `object` must be null or
  /// point to a cppgc object of this heap.
  unsafe fn is_heap_object_alive_raw(&self, object: *const c_void) -> bool;
}

impl LivenessBrokerExt for LivenessBroker {
  unsafe fn is_heap_object_alive_raw(&self, object: *const c_void) -> bool {
    unsafe { ffi::cppgc__LivenessBroker__IsHeapObjectAlive(self, object) }
  }
}

/// Extension methods for [`v8::Isolate`].
pub trait IsolateExt {
  fn request_garbage_collection_for_testing_with_stack_state(
    &mut self,
    gc_type: v8::GarbageCollectionType,
    stack_state: EmbedderStackState,
  );
  fn cpp_heap_handle(&mut self) -> Option<HeapHandle<'_>>;
}

impl IsolateExt for v8::Isolate {
  fn request_garbage_collection_for_testing_with_stack_state(
    &mut self,
    gc_type: v8::GarbageCollectionType,
    stack_state: EmbedderStackState,
  ) {
    v8::Isolate::request_garbage_collection_for_testing_with_stack_state(self, gc_type, stack_state)
  }
  fn cpp_heap_handle(&mut self) -> Option<HeapHandle<'_>> {
    self.get_cpp_heap().map(HeapHandle::new)
  }
}

/// ABI self-checks (same as standalone mode). In fork mode the shim contains
/// only these functions.
#[doc(hidden)]
pub mod abi {
  use super::*;

  pub const SHIM_DEFINES: &str = env!("V8_CPPGC_EXT_DEFINES");
  pub const SHIM_CXX: &str = env!("V8_CPPGC_EXT_CXX");
  pub const DEBUG_V8: bool = cfg!(v8_cppgc_ext_debug_v8);

  pub const BIT_POINTER_COMPRESSION: i32 = 1 << 0;
  pub const BIT_31BIT_SMIS: i32 = 1 << 1;
  pub const BIT_SANDBOX: i32 = 1 << 2;
  pub const BIT_V8_ENABLE_CHECKS: i32 = 1 << 3;
  pub const BIT_CPPGC_CAGED_HEAP: i32 = 1 << 8;
  pub const BIT_CPPGC_POINTER_COMPRESSION: i32 = 1 << 9;
  pub const BIT_CPPGC_YOUNG_GENERATION: i32 = 1 << 10;

  pub fn shim_build_configuration() -> i32 {
    unsafe { ffi::v8_cppgc_ext__shim_build_configuration() }
  }

  /// # Safety
  /// Only in a process where V8 has not been initialized yet.
  pub unsafe fn initialize_v8_with_shim_build_config() -> bool {
    unsafe { ffi::v8_cppgc_ext__V8__Initialize_with_shim_build_config() }
  }

  pub fn sizeof_member() -> usize {
    unsafe { ffi::v8_cppgc_ext__sizeof_Member() }
  }

  pub fn member_roundtrip_shim_to_prebuilt<T: GarbageCollected>(obj: &UnsafePtr<T>) -> bool {
    let mut slot = std::mem::MaybeUninit::<v8::cppgc::Member<T>>::uninit();
    assert_eq!(std::mem::size_of::<v8::cppgc::Member<T>>(), sizeof_member());
    unsafe {
      ffi::v8_cppgc_ext__Member__construct_with_shim(slot.as_mut_ptr().cast(), rust_obj_ptr(obj));
      let m = slot.assume_init();
      let ok = rust_obj_ptr(&m) == rust_obj_ptr(obj);
      drop(m);
      ok
    }
  }

  pub fn member_roundtrip_prebuilt_to_shim<T: GarbageCollected>(obj: &UnsafePtr<T>) -> bool {
    let m = v8::cppgc::Member::new(obj);
    let got = unsafe { ffi::v8_cppgc_ext__Member__get_with_shim(&m as *const _ as *const c_void) };
    got == rust_obj_ptr(obj)
  }

  pub fn member_assign_with_shim<T: GarbageCollected>(member: &mut v8::cppgc::Member<T>, obj: &UnsafePtr<T>) -> bool {
    unsafe {
      ffi::v8_cppgc_ext__Member__assign_with_shim(member as *mut _ as *mut c_void, rust_obj_ptr(obj));
    }
    rust_obj_ptr(member) == rust_obj_ptr(obj)
  }

  #[allow(dead_code)]
  fn _uses() {
    let _ = ffi::v8_cppgc_ext__Member__destruct_with_shim;
  }
}
