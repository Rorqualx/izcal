// Copyright 2019-2021 the Deno authors. All rights reserved. MIT license

use crate::Data;
use crate::TracedReference;
use crate::binding::RustObj;
use crate::platform::Platform;
use crate::support::Opaque;
use crate::support::SharedRef;
use crate::support::UniqueRef;
use crate::support::int;
use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::ffi::c_char;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

unsafe extern "C" {
  fn cppgc__initialize_process(platform: *mut Platform);
  fn cppgc__shutdown_process();

  fn v8__CppHeap__Create(
    platform: *mut Platform,
    marking_support: MarkingType,
    sweeping_support: SweepingType,
  ) -> *mut Heap;
  fn v8__CppHeap__Terminate(heap: *mut Heap);
  fn v8__CppHeap__DELETE(heap: *mut Heap);
  fn cppgc__make_garbage_collectable(
    heap: *mut Heap,
    size: usize,
    alignment: usize,
  ) -> *mut RustObj;

  fn cppgc__heap__enable_detached_garbage_collections_for_testing(
    heap: *mut Heap,
  );
  fn cppgc__heap__collect_garbage_for_testing(
    heap: *mut Heap,
    stack_state: EmbedderStackState,
  );

  fn cppgc__Visitor__Trace__Member(
    visitor: *mut Visitor,
    member: *const MemberInner,
  );
  fn cppgc__Visitor__Trace__WeakMember(
    visitor: *mut Visitor,
    member: *const WeakMemberInner,
  );
  fn cppgc__Visitor__Trace__TracedReference(
    visitor: *mut Visitor,
    reference: *const TracedReference<Data>,
  );

  fn cppgc__Member__CONSTRUCT(member: *mut MemberInner, obj: *mut RustObj);
  fn cppgc__Member__DESTRUCT(member: *mut MemberInner);
  fn cppgc__Member__Get(member: *const MemberInner) -> *mut RustObj;
  fn cppgc__Member__Assign(member: *mut MemberInner, other: *mut RustObj);

  fn cppgc__WeakMember__CONSTRUCT(
    member: *mut WeakMemberInner,
    obj: *mut RustObj,
  );
  fn cppgc__WeakMember__DESTRUCT(member: *mut WeakMemberInner);
  fn cppgc__WeakMember__Get(member: *const WeakMemberInner) -> *mut RustObj;
  fn cppgc__WeakMember__Assign(
    member: *mut WeakMemberInner,
    other: *mut RustObj,
  );

  fn cppgc__Persistent__CONSTRUCT(obj: *mut RustObj) -> *mut PersistentInner;
  fn cppgc__Persistent__DESTRUCT(this: *mut PersistentInner);
  fn cppgc__Persistent__Assign(this: *mut PersistentInner, ptr: *mut RustObj);
  fn cppgc__Persistent__Get(this: *const PersistentInner) -> *mut RustObj;

  fn cppgc__WeakPersistent__CONSTRUCT(
    obj: *mut RustObj,
  ) -> *mut WeakPersistentInner;
  fn cppgc__WeakPersistent__DESTRUCT(this: *mut WeakPersistentInner);
  fn cppgc__WeakPersistent__Assign(
    this: *mut WeakPersistentInner,
    ptr: *mut RustObj,
  );
  fn cppgc__WeakPersistent__Get(
    this: *const WeakPersistentInner,
  ) -> *mut RustObj;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn rusty_v8_RustObj_trace(
  obj: *const RustObj,
  visitor: *mut Visitor,
) {
  unsafe {
    let r = get_rust_obj(obj);
    r.trace(&mut *visitor);
  }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn rusty_v8_RustObj_get_name(
  obj: *const RustObj,
) -> *const c_char {
  let r = unsafe { get_rust_obj(obj) };
  r.get_name().as_ptr()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn rusty_v8_RustObj_drop(obj: *mut RustObj) {
  unsafe {
    let r = get_rust_obj_mut(obj);
    std::ptr::drop_in_place(r);
  }
}

/// Process-global initialization of the garbage collector. Must be called before
/// creating a Heap.
///
/// Can be called multiple times when paired with `ShutdownProcess()`.
pub fn initialize_process(platform: SharedRef<Platform>) {
  unsafe {
    cppgc__initialize_process(&*platform as *const Platform as *mut _);
  }
}

/// # Safety
///
/// Must be called after destroying the last used heap. Some process-global
/// metadata may not be returned and reused upon a subsequent
/// `initialize_process()` call.
pub unsafe fn shutdown_process() {
  unsafe {
    cppgc__shutdown_process();
  }
}

/// Visitor passed to trace methods. All managed pointers must have called the
/// Visitor's trace method on them.
///
/// ```
/// use v8::cppgc::{Member, Visitor, GarbageCollected};
/// use std::ffi::CStr;
///
/// struct Foo { foo: Member<Foo> }
///
/// unsafe impl GarbageCollected for Foo {
///   fn trace(&self, visitor: &mut Visitor) {
///     visitor.trace(&self.foo);
///   }
///
///   fn get_name(&self) -> &'static std::ffi::CStr {
///     c"Foo"
///   }
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct Visitor(Opaque);

impl Visitor {
  /// Trace a managed object.
  #[inline(always)]
  pub fn trace(&mut self, member: &impl Traced) {
    member.trace(self);
  }
}

/// Trait for inlined objects that are not allocated
/// themselves but otherwise follow managed heap layout.
pub trait Traced {
  /// Called by the `Visitor` when tracing managed objects.
  fn trace(&self, visitor: &mut Visitor);
}

impl<T: Traced> Traced for Option<T> {
  fn trace(&self, visitor: &mut Visitor) {
    if let Some(value) = self {
      visitor.trace(value);
    }
  }
}

impl<T> Traced for TracedReference<T> {
  fn trace(&self, visitor: &mut Visitor) {
    unsafe {
      cppgc__Visitor__Trace__TracedReference(
        visitor,
        self as *const TracedReference<T> as *const TracedReference<Data>,
      );
    }
  }
}

#[repr(C)]
pub enum EmbedderStackState {
  /// Stack may contain interesting heap pointers.
  MayContainHeapPointers,
  /// Stack does not contain any interesting heap pointers.
  NoHeapPointers,
}

/// Specifies supported marking types.
#[repr(u8)]
pub enum MarkingType {
  /// Atomic stop-the-world marking. This option does not require any write barriers but is the most intrusive in terms of jank.
  Atomic,
  /// Incremental marking interleaves marking with the rest of the application workload on the same thread.
  Incremental,
  /// Incremental and concurrent marking.
  IncrementalAndConcurrent,
}

/// Specifies supported sweeping types.
#[repr(u8)]
pub enum SweepingType {
  /// Atomic stop-the-world sweeping. All of sweeping is performed at once.
  Atomic,
  /// Incremental sweeping interleaves sweeping with the rest of the application workload on the same thread.
  Incremental,
  /// Incremental and concurrent sweeping. Sweeping is split and interleaved with the rest of the application.
  IncrementalAndConcurrent,
}

pub type InternalFieldIndex = int;

pub struct HeapCreateParams {
  /// Specifies which kind of marking are supported by the heap.
  pub marking_support: MarkingType,
  /// Specifies which kind of sweeping are supported by the heap.
  pub sweeping_support: SweepingType,
}

impl Default for HeapCreateParams {
  fn default() -> Self {
    Self {
      marking_support: MarkingType::IncrementalAndConcurrent,
      sweeping_support: SweepingType::IncrementalAndConcurrent,
    }
  }
}

/// A heap for allocating managed C++ objects.
///
/// Similar to v8::Isolate, the heap may only be accessed from one thread at a
/// time.
#[repr(C)]
#[derive(Debug)]
pub struct Heap(Opaque);

impl Drop for Heap {
  fn drop(&mut self) {
    unsafe {
      v8__CppHeap__DELETE(self);
    }
  }
}

impl Heap {
  pub fn create(
    platform: SharedRef<Platform>,
    params: HeapCreateParams,
  ) -> UniqueRef<Heap> {
    unsafe {
      UniqueRef::from_raw(v8__CppHeap__Create(
        &*platform as *const Platform as *mut _,
        params.marking_support,
        params.sweeping_support,
      ))
    }
  }

  pub unsafe fn collect_garbage_for_testing(
    &self,
    stack_state: EmbedderStackState,
  ) {
    unsafe {
      cppgc__heap__collect_garbage_for_testing(
        self as *const Heap as *mut _,
        stack_state,
      );
    }
  }

  pub fn enable_detached_garbage_collections_for_testing(&self) {
    unsafe {
      cppgc__heap__enable_detached_garbage_collections_for_testing(
        self as *const Heap as *mut _,
      );
    }
  }

  pub fn terminate(&mut self) {
    unsafe {
      v8__CppHeap__Terminate(self);
    }
  }
}

/// Base trait for objects supporting garbage collection.
///
/// # Safety
///
/// implementors must guarantee that the `trace()`
/// method correctly visits all [`Member`], [`WeakMember`], and
/// [`TraceReference`] pointers held by this object. Failing to do so will leave
/// dangling pointers in the heap as objects are garbage collected.
pub unsafe trait GarbageCollected {
  /// `trace` must call [`Visitor::trace`] for each
  /// [`Member`], [`WeakMember`], or [`TracedReference`] reachable
  /// from `self`.
  fn trace(&self, visitor: &mut Visitor);

  /// Specifies a name for the garbage-collected object. Such names will never
  /// be hidden, as they are explicitly specified by the user of this API.
  ///
  /// V8 may call this function while generating a heap snapshot or at other
  /// times. If V8 is currently generating a heap snapshot (according to
  /// HeapProfiler::IsTakingSnapshot), then the returned string must stay alive
  /// until the snapshot generation has completed. Otherwise, the returned string
  /// must stay alive forever. If you need a place to store a temporary string
  /// during snapshot generation, use HeapProfiler::CopyNameForHeapSnapshot.
  fn get_name(&self) -> &'static CStr;
}

#[repr(C)]
struct RustObjConcrete<T> {
  head: RustObj,
  dynamic: *mut dyn GarbageCollected,
  value: T,
}

unsafe fn get_rust_obj<'s>(obj: *const RustObj) -> &'s dyn GarbageCollected {
  unsafe {
    let obj = &*(obj as *const RustObjConcrete<()>);
    &*obj.dynamic
  }
}

unsafe fn get_rust_obj_mut<'s>(
  obj: *mut RustObj,
) -> &'s mut dyn GarbageCollected {
  unsafe {
    let obj = &mut *(obj as *mut RustObjConcrete<()>);
    &mut *obj.dynamic
  }
}

/// Constructs an instance of T, which is a garbage collected type.
///
/// The object will be allocated on the heap and managed by cppgc. During
/// marking, the object will be traced by calling the `trace` method on it.
///
/// During sweeping, the destructor will be called and the memory will be
/// freed.
///
/// # Safety
///
/// The caller must ensure that the returned pointer is always stored on
/// the stack, or is safely moved into one of the other cppgc pointer types.
pub unsafe fn make_garbage_collected<T: GarbageCollected + 'static>(
  heap: &Heap,
  obj: T,
) -> UnsafePtr<T> {
  const {
    // max alignment in cppgc is 16
    assert!(std::mem::align_of::<T>() <= 16);
    assert!(
      std::mem::offset_of!(RustObjConcrete<T>, dynamic)
        == std::mem::offset_of!(RustObjConcrete<()>, dynamic)
    );
  }

  let additional_bytes =
    std::mem::size_of::<RustObjConcrete<T>>() - std::mem::size_of::<RustObj>();

  let pointer = unsafe {
    cppgc__make_garbage_collectable(
      heap as *const Heap as *mut _,
      additional_bytes,
      std::mem::align_of::<RustObjConcrete<T>>(),
    )
  };

  assert!(!pointer.is_null());

  unsafe {
    let pointer = &mut *(pointer as *mut RustObjConcrete<T>);
    let value_ptr = std::ptr::addr_of_mut!(pointer.value);
    value_ptr.write(obj);
    std::ptr::addr_of_mut!(pointer.dynamic).write(value_ptr as _);
  }

  UnsafePtr {
    pointer: unsafe { NonNull::new_unchecked(pointer) },
    _phantom: PhantomData,
  }
}

/// # Safety
///
/// T must be the correct type for this specific RustObj
unsafe fn get_value_from_rust_obj<T: GarbageCollected>(
  obj: *mut RustObj,
) -> *const T {
  unsafe {
    let obj = &mut *(obj as *mut RustObjConcrete<T>);
    std::ptr::addr_of_mut!(obj.value)
  }
}

/// UnsafePtr is used to refer to an on-heap object from the stack.
#[derive(Clone, Copy)]
pub struct UnsafePtr<T: GarbageCollected> {
  pointer: NonNull<RustObj>,
  _phantom: PhantomData<T>,
}

impl<T: GarbageCollected> UnsafePtr<T> {
  /// Create a new UnsafePtr.
  ///
  /// # Safety
  ///
  /// The caller must ensure that the returned pointer is always stored on
  /// the stack, or is safely moved into one of the other cppgc pointer types.
  pub unsafe fn new(value: &impl GetRustObj<T>) -> Option<UnsafePtr<T>> {
    NonNull::new(value.get_rust_obj()).map(|pointer| UnsafePtr {
      pointer,
      _phantom: PhantomData,
    })
  }

  pub unsafe fn as_ref(&self) -> &T {
    unsafe { &*get_value_from_rust_obj(self.pointer.as_ptr()) }
  }
}

impl<T: GarbageCollected> GetRustObj<T> for UnsafePtr<T> {
  fn get_rust_obj(&self) -> *mut RustObj {
    self.pointer.as_ptr()
  }
}

#[doc(hidden)]
pub trait GetRustObj<T: GarbageCollected> {
  fn get_rust_obj(&self) -> *mut RustObj;
}

impl<T: GarbageCollected> GetRustObj<T> for *mut RustObj {
  fn get_rust_obj(&self) -> *mut RustObj {
    *self
  }
}

macro_rules! member {
  ($( # $attr:tt )* $name:ident) => {
    paste::paste! {
      #[repr(transparent)]
      struct [< $name Inner >]([u8; crate::binding:: [< cppgc__ $name _SIZE >]]);

      impl [< $name Inner >] {
        fn new(ptr: *mut RustObj) -> Self {
          let mut this = std::mem::MaybeUninit::uninit();
          unsafe {
            [< cppgc__ $name __CONSTRUCT >](this.as_mut_ptr(), ptr);
            this.assume_init()
          }
        }

        #[inline(always)]
        fn get(&self) -> *mut RustObj {
          // Member may be a compressed pointer, so just read it from C++
          unsafe { [< cppgc__ $name __Get >](self) }
        }

        #[inline(always)]
        fn assign(&mut self, ptr: *mut RustObj) {
          // Assignment has write barriers in the GC, so call into C++
          unsafe {
            [< cppgc__ $name __Assign >](self, ptr);
          }
        }
      }

      impl Drop for [< $name Inner >] {
        fn drop(&mut self) {
          unsafe {
            [< cppgc__ $name __DESTRUCT >](self);
          }
        }
      }

      $( # $attr )*
      #[repr(transparent)]
      pub struct $name<T: GarbageCollected> {
        inner: [< $name Inner >],
        _phantom: PhantomData<T>,
      }

      impl<T: GarbageCollected> $name<T> {
        #[doc = "Create a new empty "]
        #[doc = stringify!($name)]
        #[doc = " which may be set later."]
        pub fn empty() -> Self {
          Self {
            inner: [< $name Inner >]::new(std::ptr::null_mut()),
            _phantom: PhantomData,
          }
        }

        #[doc = "Create a new "]
        #[doc = stringify!($name)]
        #[doc = " and initialize it with an object."]
        pub fn new(other: &impl GetRustObj<T>) -> Self {
          Self {
            inner: [< $name Inner >]::new(other.get_rust_obj()),
            _phantom: PhantomData,
          }
        }

        #[doc = "Set the object pointed to by this "]
        #[doc = stringify!($name)]
        #[doc = "."]
        pub fn set(&mut self, other: &impl GetRustObj<T>) {
          let ptr = other.get_rust_obj();
          self.inner.assign(ptr);
        }

        #[doc = "Get the object pointed to by this "]
        #[doc = stringify!($name)]
        #[doc = ", returning `None` if the pointer is empty or has been garbage-collected."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "The caller must ensure that this pointer is being traced correctly by appearing in the [`trace`](GarbageCollected::trace)"]
        #[doc = "implementation of the object that owns the pointer. Between initializing the pointer and calling `get()`, the pointer must be reachable by the garbage collector."]
        pub unsafe fn get(&self) -> Option<&T> {
          let ptr = self.inner.get();
          if ptr.is_null() {
            None
          } else {
            // SAFETY: Either this is a strong reference and the pointer is valid according
            // to the safety contract of this method, or this is a weak reference and the
            // ptr will be null if it was collected.
            Some(unsafe { &*get_value_from_rust_obj(ptr) })
          }
        }
      }

      impl<T: GarbageCollected> GetRustObj<T> for $name<T> {
        fn get_rust_obj(&self) -> *mut RustObj {
          self.inner.get()
        }
      }

      impl<T: GarbageCollected> Traced for $name<T> {
        fn trace(&self, visitor: &mut Visitor) {
          unsafe { [< cppgc__Visitor__Trace__ $name >](visitor, &self.inner) }
        }
      }

      impl<T: GarbageCollected> std::fmt::Debug for $name<T> {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
          fmt.debug_struct(stringify!($name)).finish()
        }
      }
    }
  }
}

member! {
  /// Members are used in classes to contain strong pointers to other garbage
  /// collected objects. All Member fields of a class must be traced in the class'
  /// trace method.
  Member
}

member! {
  /// WeakMember is similar to Member in that it is used to point to other garbage
  /// collected objects. However instead of creating a strong pointer to the
  /// object, the WeakMember creates a weak pointer, which does not keep the
  /// pointee alive. Hence if all pointers to to a heap allocated object are weak
  /// the object will be garbage collected. At the time of GC the weak pointers
  /// will automatically be set to null.
  WeakMember
}

macro_rules! persistent {
  ($( # $attr:tt )* $name:ident) => {
    paste::paste! {
      // PersistentBase is extremely particular about move and copy semantics,
      // so we allocate it on the heap and only interact with it via calls to C++.
      #[repr(C)]
      struct [< $name Inner >](Opaque);

      $( # $attr )*
      pub struct $name<T: GarbageCollected> {
        inner: *mut [< $name Inner >],
        _phantom: PhantomData<T>,
      }

      impl<T: GarbageCollected> $name<T> {
        #[doc = "Create a new empty "]
        #[doc = stringify!($name)]
        #[doc = " which may be set later."]
        pub fn empty() -> Self {
          let this = unsafe { [< cppgc__ $name __CONSTRUCT >](std::ptr::null_mut()) };
          Self {
            inner: this,
            _phantom: PhantomData,
          }
        }

        #[doc = "Create a new "]
        #[doc = stringify!($name)]
        #[doc = " and initialize it with an object."]
        pub fn new(other: &impl GetRustObj<T>) -> Self {
          let this = unsafe { [< cppgc__ $name __CONSTRUCT >](other.get_rust_obj()) };
          Self {
            inner: this,
            _phantom: PhantomData,
          }
        }

        #[doc = "Set the object pointed to by this "]
        #[doc = stringify!($name)]
        #[doc = "."]
        pub fn set(&mut self, other: &impl GetRustObj<T>) {
          let ptr = other.get_rust_obj();
          self.assign(ptr);
        }

        #[doc = "Borrow the object pointed to by this "]
        #[doc = stringify!($name)]
        #[doc = "."]
        pub fn get(&self) -> Option<&T> {
          let ptr = self.get_rust_obj();
          if ptr.is_null() {
            None
          } else {
            // SAFETY: Either this is a strong reference and the pointer is always valid
            // or this is a weak reference and the ptr will be null if it was collected.
            Some(unsafe { &*get_value_from_rust_obj(ptr) })
          }
        }

        #[inline(always)]
        fn assign(&mut self, ptr: *mut RustObj) {
          unsafe {
            [< cppgc__ $name __Assign >](self.inner, ptr);
          }
        }
      }

      impl<T: GarbageCollected> Drop for $name<T> {
        fn drop(&mut self) {
          unsafe {
            [< cppgc__ $name __DESTRUCT >](self.inner);
          }
        }
      }

      impl<T: GarbageCollected> std::fmt::Debug for $name<T> {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
          fmt.debug_struct(stringify!($name)).finish()
        }
      }

      impl<T: GarbageCollected> GetRustObj<T> for $name<T> {
        fn get_rust_obj(&self) -> *mut RustObj {
          unsafe {
            [< cppgc__ $name __Get >](self.inner)
          }
        }
      }
    }
  };
}

persistent! {
  /// Persistent is a way to create a strong pointer from an off-heap object to
  /// another on-heap object. As long as the Persistent handle is alive the GC will
  /// keep the object pointed to alive. The Persistent handle is always a GC root
  /// from the point of view of the GC. Persistent must be constructed and
  /// destructed in the same thread.
  Persistent
}

persistent! {
  /// WeakPersistent is a way to create a weak pointer from an off-heap object to
  /// an on-heap object. The pointer is automatically cleared when the pointee gets
  /// collected. WeakPersistent must be constructed and destructed in the same
  /// thread.
  WeakPersistent
}

/// `GcCell` is a zero-overhead memory cell that provides interior mutability
/// for garbage collected types.
///
/// Mutable access to the value inside the `GcCell` is granted by delivering
/// proof that the caller has mutable access to the V8 Isolate (such as through
/// a [`HandleScope`](crate::HandleScope)). This statically guarantees that
/// access patterns follow the rules of both the garbage collector and the Rust
/// borrow checker.
///
/// `GcCell` also implements [`Traced`], which means that it can hold other
/// garbage collected references, like [`Member`] or [`WeakMember`]. If the
/// `GcCell` holds other garbage collected objects, those objects cannot be
/// accessed while the `GcCell` is borrowed, and the caller must construct
/// temporary pointers ([`UnsafePtr<T>`]) on the stack in order to access nested
/// objects in the object graph.
pub struct GcCell<T> {
  // Contents guarded by access to the `Isolate`.
  value: UnsafeCell<T>,
}

impl<T> std::fmt::Debug for GcCell<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("GcCell").finish()
  }
}

unsafe impl<T: Send> Send for GcCell<T> {}
unsafe impl<T: Sync> Sync for GcCell<T> {}

impl<T> GcCell<T> {
  pub fn new(value: T) -> Self {
    Self {
      value: UnsafeCell::new(value),
    }
  }

  pub fn set(&self, isolate: &mut crate::Isolate, value: T) {
    _ = isolate;
    unsafe {
      // SAFETY: The `isolate` argument is proof that we have mutable access to
      // the value.
      *self.value.get() = value;
    }
  }
}

impl<T> GcCell<T> {
  pub fn get<'a>(&'a self, isolate: &'a crate::Isolate) -> &'a T {
    _ = isolate;
    unsafe {
      // SAFETY: The `isolate` argument is proof that we have access to the
      // value, and the returned reference binds the isolate's lifetime.
      &*self.value.get()
    }
  }

  pub fn get_mut<'a>(&'a self, isolate: &'a mut crate::Isolate) -> &'a mut T {
    _ = isolate;
    unsafe {
      // SAFETY: The `isolate` argument is proof that we have mutable access to
      // the value, and the returned reference binds the isolate's lifetime.
      &mut *self.value.get()
    }
  }

  pub fn with<'a, 's, R>(
    &'a self,
    scope: &'a mut crate::HandleScope<'s>,
    f: impl FnOnce(&'a mut crate::HandleScope<'s>, &'a mut T) -> R,
  ) -> R {
    f(scope, unsafe { &mut *self.value.get() })
  }
}

impl<T: Traced> Traced for GcCell<T> {
  fn trace(&self, visitor: &mut Visitor) {
    unsafe {
      // SAFETY: This is invoked by the GC, so access is guaranteed to follow
      // its rules.
      visitor.trace(&(*self.value.get()));
    }
  }
}

// =============================================================================
// Heap handles, GC scopes, heap state, statistics, prefinalizers, weak
// callbacks, cppgc::testing and HeapConsistency.
// =============================================================================

#[repr(C)]
struct HeapHandleInner(Opaque);

#[repr(C)]
struct HeapStatisticsInner(Opaque);

#[repr(C)]
struct StandaloneTestingHeapInner(Opaque);

#[repr(C)]
struct OverrideEmbedderStackStateScopeInner(Opaque);

#[repr(C)]
#[derive(Default)]
struct HeapStatisticsTotals {
  committed_size_bytes: usize,
  resident_size_bytes: usize,
  used_size_bytes: usize,
  pooled_memory_size_bytes: usize,
  space_count: usize,
  type_name_count: usize,
  detail_level: u8,
}

#[repr(C)]
struct HeapStatisticsSpace {
  name: *const c_char,
  name_len: usize,
  committed_size_bytes: usize,
  resident_size_bytes: usize,
  used_size_bytes: usize,
  page_count: usize,
}

#[repr(C)]
struct HeapStatisticsPage {
  committed_size_bytes: usize,
  resident_size_bytes: usize,
  used_size_bytes: usize,
  object_statistics: *const ObjectStatsEntry,
  object_statistics_len: usize,
}

#[repr(C)]
struct HeapStatisticsFreeList {
  bucket_size: *const usize,
  bucket_size_len: usize,
  free_count: *const usize,
  free_count_len: usize,
  free_size: *const usize,
  free_size_len: usize,
}

type PrefinalizerCallback =
  unsafe extern "C" fn(broker: &LivenessBroker, object: *mut RustObj) -> bool;
type WeakCallback =
  unsafe extern "C" fn(broker: &LivenessBroker, data: *const c_void);

unsafe extern "C" {
  fn v8__CppHeap__GetHeapHandle(heap: *mut Heap) -> *mut HeapHandleInner;

  fn cppgc__NoGarbageCollectionScope__Enter(handle: *mut HeapHandleInner);
  fn cppgc__NoGarbageCollectionScope__Leave(handle: *mut HeapHandleInner);
  fn cppgc__DisallowGarbageCollectionScope__Enter(handle: *mut HeapHandleInner);
  fn cppgc__DisallowGarbageCollectionScope__Leave(handle: *mut HeapHandleInner);
  fn cppgc__DisallowGarbageCollectionScope__IsGarbageCollectionAllowed(
    handle: *mut HeapHandleInner,
  ) -> bool;

  fn cppgc__HeapState__IsMarking(handle: *const HeapHandleInner) -> bool;
  fn cppgc__HeapState__IsSweeping(handle: *const HeapHandleInner) -> bool;
  fn cppgc__HeapState__IsSweepingOnOwningThread(
    handle: *const HeapHandleInner,
  ) -> bool;
  fn cppgc__HeapState__IsInAtomicPause(handle: *const HeapHandleInner) -> bool;
  fn cppgc__HeapState__PreviousGCWasConservative(
    handle: *const HeapHandleInner,
  ) -> bool;

  fn v8__CppHeap__CollectStatistics(
    heap: *mut Heap,
    detail_level: u8,
  ) -> *mut HeapStatisticsInner;
  fn cppgc__HeapStatistics__DELETE(this: *mut HeapStatisticsInner);
  fn cppgc__HeapStatistics__GetTotals(
    this: *const HeapStatisticsInner,
    out: *mut HeapStatisticsTotals,
  );
  fn cppgc__HeapStatistics__GetSpace(
    this: *const HeapStatisticsInner,
    space_index: usize,
    out: *mut HeapStatisticsSpace,
  ) -> bool;
  fn cppgc__HeapStatistics__GetPage(
    this: *const HeapStatisticsInner,
    space_index: usize,
    page_index: usize,
    out: *mut HeapStatisticsPage,
  ) -> bool;
  fn cppgc__HeapStatistics__GetFreeList(
    this: *const HeapStatisticsInner,
    space_index: usize,
    out: *mut HeapStatisticsFreeList,
  ) -> bool;
  fn cppgc__HeapStatistics__GetTypeName(
    this: *const HeapStatisticsInner,
    index: usize,
    len: *mut usize,
  ) -> *const c_char;

  fn cppgc__RustObj__RegisterPrefinalizer(
    obj: *mut RustObj,
    callback: PrefinalizerCallback,
  );
  fn cppgc__LivenessBroker__IsHeapObjectAlive(
    broker: *const LivenessBroker,
    obj: *const RustObj,
  ) -> bool;
  fn cppgc__Visitor__RegisterWeakCallback(
    visitor: *mut Visitor,
    callback: WeakCallback,
    data: *const c_void,
  );

  fn cppgc__testing__OverrideEmbedderStackStateScope__NEW(
    handle: *mut HeapHandleInner,
    state: EmbedderStackState,
  ) -> *mut OverrideEmbedderStackStateScopeInner;
  fn cppgc__testing__OverrideEmbedderStackStateScope__DELETE(
    this: *mut OverrideEmbedderStackStateScopeInner,
  );
  fn cppgc__testing__StandaloneTestingHeap__NEW(
    handle: *mut HeapHandleInner,
  ) -> *mut StandaloneTestingHeapInner;
  fn cppgc__testing__StandaloneTestingHeap__DELETE(
    this: *mut StandaloneTestingHeapInner,
  );
  fn cppgc__testing__StandaloneTestingHeap__StartGarbageCollection(
    this: *mut StandaloneTestingHeapInner,
  );
  fn cppgc__testing__StandaloneTestingHeap__PerformMarkingStep(
    this: *mut StandaloneTestingHeapInner,
    stack_state: EmbedderStackState,
  ) -> bool;
  fn cppgc__testing__StandaloneTestingHeap__FinalizeGarbageCollection(
    this: *mut StandaloneTestingHeapInner,
    stack_state: EmbedderStackState,
  );
  fn cppgc__testing__StandaloneTestingHeap__ToggleMainThreadMarking(
    this: *mut StandaloneTestingHeapInner,
    should_mark: bool,
  );
  fn cppgc__testing__StandaloneTestingHeap__ForceCompactionForNextGarbageCollection(
    this: *mut StandaloneTestingHeapInner,
  );
  fn cppgc__testing__IsHeapObjectOld(obj: *mut RustObj) -> bool;

  fn cppgc__HeapConsistency__GetWriteBarrierType(
    slot: *const c_void,
    value: *const c_void,
  ) -> u8;
  fn cppgc__HeapConsistency__WriteBarrier(
    slot: *const c_void,
    value: *const c_void,
  );
  fn cppgc__HeapConsistency__DijkstraWriteBarrier(
    slot: *const c_void,
    value: *const c_void,
  ) -> bool;
  fn cppgc__HeapConsistency__SteeleWriteBarrier(object: *const c_void) -> bool;
  fn cppgc__HeapConsistency__GenerationalBarrier(
    slot: *const c_void,
    value: *const c_void,
  ) -> bool;
}

/// A `cppgc::HeapHandle` for a [`Heap`]. Borrows the heap and is
/// `!Send + !Sync`: every operation must happen on the heap's thread.
#[derive(Clone, Copy, Debug)]
pub struct HeapHandle<'h> {
  ptr: NonNull<HeapHandleInner>,
  _heap: PhantomData<&'h Heap>,
  _not_send: PhantomData<*mut ()>,
}

impl<'h> HeapHandle<'h> {
  /// `v8::CppHeap::GetHeapHandle()`.
  pub fn new(heap: &'h Heap) -> Self {
    let ptr =
      unsafe { v8__CppHeap__GetHeapHandle(heap as *const Heap as *mut Heap) };
    Self {
      ptr: NonNull::new(ptr).unwrap(),
      _heap: PhantomData,
      _not_send: PhantomData,
    }
  }

  /// The handle of an isolate's CppHeap with a lifetime that is not tied to
  /// the `&mut Isolate` borrow, so scopes can be held while JS runs.
  ///
  /// # Safety
  ///
  /// The isolate must outlive `'a`, and the handle must only be used on the
  /// thread that owns the isolate.
  pub unsafe fn from_isolate_unbounded<'a>(
    isolate: &mut crate::Isolate,
  ) -> Option<HeapHandle<'a>> {
    let heap = isolate.get_cpp_heap()?;
    Some(HeapHandle::new(unsafe { &*(heap as *const Heap) }))
  }

  fn raw(&self) -> *mut HeapHandleInner {
    self.ptr.as_ptr()
  }
}

impl Heap {
  /// `CppHeap::GetHeapHandle()`.
  pub fn heap_handle(&self) -> HeapHandle<'_> {
    HeapHandle::new(self)
  }

  /// `CppHeap::CollectStatistics()`, copied into Rust-owned structures. Must
  /// be called on the heap's thread, outside of a garbage collection.
  pub fn collect_statistics(
    &self,
    detail_level: DetailLevel,
  ) -> HeapStatistics {
    HeapStatistics::collect(self, detail_level)
  }

  /// Panics if the heap has custom spaces. Only custom spaces can be
  /// compactable, so this guarantees cppgc never moves objects of this heap.
  /// Heaps created by rusty_v8 never have custom spaces.
  pub fn assert_no_compactable_spaces(&self) {
    let stats = self.collect_statistics(DetailLevel::Detailed);
    assert!(!stats.space_stats.is_empty());
    for space in &stats.space_stats {
      assert!(
        !space.name.starts_with("CustomSpace"),
        "cppgc heap has custom space {}",
        space.name
      );
    }
  }
}

/// RAII form of `cppgc::subtle::NoGarbageCollectionScope`.
///
/// Defers garbage collections initiated by cppgc while held. It does not stop
/// V8's own garbage collections from finalizing an attached CppHeap; combine
/// with [`DisallowGarbageCollectionScope`] to turn those into a crash.
#[must_use]
#[derive(Debug)]
pub struct NoGarbageCollectionScope<'h> {
  handle: HeapHandle<'h>,
}

impl<'h> NoGarbageCollectionScope<'h> {
  pub fn new(handle: HeapHandle<'h>) -> Self {
    unsafe { cppgc__NoGarbageCollectionScope__Enter(handle.raw()) };
    Self { handle }
  }

  /// # Safety
  ///
  /// Must be balanced by one [`Self::leave`] on the same heap and thread.
  pub unsafe fn enter(handle: HeapHandle<'_>) {
    unsafe { cppgc__NoGarbageCollectionScope__Enter(handle.raw()) }
  }

  /// # Safety
  ///
  /// Must pair with a previous [`Self::enter`].
  pub unsafe fn leave(handle: HeapHandle<'_>) {
    unsafe { cppgc__NoGarbageCollectionScope__Leave(handle.raw()) }
  }
}

impl Drop for NoGarbageCollectionScope<'_> {
  fn drop(&mut self) {
    unsafe { cppgc__NoGarbageCollectionScope__Leave(self.handle.raw()) }
  }
}

/// RAII form of `cppgc::subtle::DisallowGarbageCollectionScope`: any garbage
/// collection reaching the heap while held is a fatal error.
#[must_use]
#[derive(Debug)]
pub struct DisallowGarbageCollectionScope<'h> {
  handle: HeapHandle<'h>,
}

impl<'h> DisallowGarbageCollectionScope<'h> {
  pub fn new(handle: HeapHandle<'h>) -> Self {
    unsafe { cppgc__DisallowGarbageCollectionScope__Enter(handle.raw()) };
    Self { handle }
  }

  /// # Safety
  ///
  /// Must be balanced by one [`Self::leave`] on the same heap and thread.
  pub unsafe fn enter(handle: HeapHandle<'_>) {
    unsafe { cppgc__DisallowGarbageCollectionScope__Enter(handle.raw()) }
  }

  /// # Safety
  ///
  /// Must pair with a previous [`Self::enter`].
  pub unsafe fn leave(handle: HeapHandle<'_>) {
    unsafe { cppgc__DisallowGarbageCollectionScope__Leave(handle.raw()) }
  }

  pub fn is_garbage_collection_allowed(handle: HeapHandle<'_>) -> bool {
    unsafe {
      cppgc__DisallowGarbageCollectionScope__IsGarbageCollectionAllowed(
        handle.raw(),
      )
    }
  }
}

impl Drop for DisallowGarbageCollectionScope<'_> {
  fn drop(&mut self) {
    unsafe { cppgc__DisallowGarbageCollectionScope__Leave(self.handle.raw()) }
  }
}

/// `cppgc::subtle::HeapState`.
pub struct HeapState(());

impl HeapState {
  pub fn is_marking(handle: HeapHandle<'_>) -> bool {
    unsafe { cppgc__HeapState__IsMarking(handle.raw()) }
  }
  pub fn is_sweeping(handle: HeapHandle<'_>) -> bool {
    unsafe { cppgc__HeapState__IsSweeping(handle.raw()) }
  }
  pub fn is_sweeping_on_owning_thread(handle: HeapHandle<'_>) -> bool {
    unsafe { cppgc__HeapState__IsSweepingOnOwningThread(handle.raw()) }
  }
  pub fn is_in_atomic_pause(handle: HeapHandle<'_>) -> bool {
    unsafe { cppgc__HeapState__IsInAtomicPause(handle.raw()) }
  }
  pub fn previous_gc_was_conservative(handle: HeapHandle<'_>) -> bool {
    unsafe { cppgc__HeapState__PreviousGCWasConservative(handle.raw()) }
  }
}

/// `cppgc::HeapStatistics::DetailLevel`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DetailLevel {
  Brief = 0,
  Detailed = 1,
}

/// `cppgc::HeapStatistics::ObjectStatsEntry`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct ObjectStatsEntry {
  pub allocated_bytes: usize,
  pub object_count: usize,
}

/// `cppgc::HeapStatistics::PageStatistics`.
#[derive(Clone, Debug, Default)]
pub struct PageStatistics {
  pub committed_size_bytes: usize,
  pub resident_size_bytes: usize,
  pub used_size_bytes: usize,
  pub object_statistics: Vec<ObjectStatsEntry>,
}

/// `cppgc::HeapStatistics::FreeListStatistics`.
#[derive(Clone, Debug, Default)]
pub struct FreeListStatistics {
  pub bucket_size: Vec<usize>,
  pub free_count: Vec<usize>,
  pub free_size: Vec<usize>,
}

/// `cppgc::HeapStatistics::SpaceStatistics`.
#[derive(Clone, Debug, Default)]
pub struct SpaceStatistics {
  pub name: String,
  pub committed_size_bytes: usize,
  pub resident_size_bytes: usize,
  pub used_size_bytes: usize,
  pub page_stats: Vec<PageStatistics>,
  pub free_list_stats: FreeListStatistics,
}

/// `cppgc::HeapStatistics`, owned by Rust.
#[derive(Clone, Debug)]
pub struct HeapStatistics {
  pub committed_size_bytes: usize,
  pub resident_size_bytes: usize,
  pub used_size_bytes: usize,
  pub pooled_memory_size_bytes: usize,
  pub detail_level: DetailLevel,
  pub space_stats: Vec<SpaceStatistics>,
  pub type_names: Vec<String>,
}

impl HeapStatistics {
  fn collect(heap: &Heap, detail_level: DetailLevel) -> Self {
    unsafe fn to_vec<T: Copy>(ptr: *const T, len: usize) -> Vec<T> {
      if len == 0 || ptr.is_null() {
        Vec::new()
      } else {
        unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
      }
    }
    unsafe fn to_string(ptr: *const c_char, len: usize) -> String {
      String::from_utf8_lossy(&unsafe { to_vec(ptr as *const u8, len) })
        .into_owned()
    }
    struct Owned(*mut HeapStatisticsInner);
    impl Drop for Owned {
      fn drop(&mut self) {
        unsafe { cppgc__HeapStatistics__DELETE(self.0) }
      }
    }
    let owned = Owned(unsafe {
      v8__CppHeap__CollectStatistics(
        heap as *const Heap as *mut Heap,
        detail_level as u8,
      )
    });
    let stats = owned.0 as *const HeapStatisticsInner;
    unsafe {
      let mut totals = HeapStatisticsTotals::default();
      cppgc__HeapStatistics__GetTotals(stats, &mut totals);
      let mut space_stats = Vec::with_capacity(totals.space_count);
      for i in 0..totals.space_count {
        let mut space = std::mem::MaybeUninit::uninit();
        assert!(cppgc__HeapStatistics__GetSpace(
          stats,
          i,
          space.as_mut_ptr()
        ));
        let space: HeapStatisticsSpace = space.assume_init();
        let mut page_stats = Vec::with_capacity(space.page_count);
        for j in 0..space.page_count {
          let mut page = std::mem::MaybeUninit::uninit();
          assert!(cppgc__HeapStatistics__GetPage(
            stats,
            i,
            j,
            page.as_mut_ptr()
          ));
          let page: HeapStatisticsPage = page.assume_init();
          page_stats.push(PageStatistics {
            committed_size_bytes: page.committed_size_bytes,
            resident_size_bytes: page.resident_size_bytes,
            used_size_bytes: page.used_size_bytes,
            object_statistics: to_vec(
              page.object_statistics,
              page.object_statistics_len,
            ),
          });
        }
        let mut fl = std::mem::MaybeUninit::uninit();
        assert!(cppgc__HeapStatistics__GetFreeList(
          stats,
          i,
          fl.as_mut_ptr()
        ));
        let fl: HeapStatisticsFreeList = fl.assume_init();
        space_stats.push(SpaceStatistics {
          name: to_string(space.name, space.name_len),
          committed_size_bytes: space.committed_size_bytes,
          resident_size_bytes: space.resident_size_bytes,
          used_size_bytes: space.used_size_bytes,
          page_stats,
          free_list_stats: FreeListStatistics {
            bucket_size: to_vec(fl.bucket_size, fl.bucket_size_len),
            free_count: to_vec(fl.free_count, fl.free_count_len),
            free_size: to_vec(fl.free_size, fl.free_size_len),
          },
        });
      }
      let mut type_names = Vec::with_capacity(totals.type_name_count);
      for i in 0..totals.type_name_count {
        let mut len = 0;
        let ptr = cppgc__HeapStatistics__GetTypeName(stats, i, &mut len);
        type_names.push(to_string(ptr, len));
      }
      Self {
        committed_size_bytes: totals.committed_size_bytes,
        resident_size_bytes: totals.resident_size_bytes,
        used_size_bytes: totals.used_size_bytes,
        pooled_memory_size_bytes: totals.pooled_memory_size_bytes,
        detail_level: if totals.detail_level == 0 {
          DetailLevel::Brief
        } else {
          DetailLevel::Detailed
        },
        space_stats,
        type_names,
      }
    }
  }
}

/// `cppgc::LivenessBroker`, available inside prefinalizers and weak
/// callbacks.
#[repr(C)]
pub struct LivenessBroker(Opaque);

impl LivenessBroker {
  /// `LivenessBroker::IsHeapObjectAlive`. Empty pointers count as alive.
  pub fn is_heap_object_alive<T: GarbageCollected>(
    &self,
    object: &impl GetRustObj<T>,
  ) -> bool {
    unsafe {
      cppgc__LivenessBroker__IsHeapObjectAlive(self, object.get_rust_obj())
    }
  }
}

/// The equivalent of `CPPGC_USING_PRE_FINALIZER`. `pre_finalize` runs on the
/// heap's thread in the atomic pause of the GC that finds the object dead,
/// before any destructor of that cycle runs. It may read the whole object
/// graph (dead or alive) and must not allocate on the cppgc heap.
pub trait PreFinalizer: GarbageCollected {
  fn pre_finalize(&self, broker: &LivenessBroker);
}

unsafe extern "C" fn invoke_pre_finalizer<T: PreFinalizer>(
  broker: &LivenessBroker,
  object: *mut RustObj,
) -> bool {
  unsafe {
    if cppgc__LivenessBroker__IsHeapObjectAlive(broker, object) {
      return false;
    }
    (*get_value_from_rust_obj::<T>(object)).pre_finalize(broker);
  }
  true
}

/// Registers `T::pre_finalize` for `ptr`.
///
/// # Safety
///
/// `ptr` must point to a live object on a heap owned by the current thread;
/// register at most once per object and not from inside a prefinalizer.
pub unsafe fn register_prefinalizer<T: PreFinalizer + 'static>(
  ptr: &UnsafePtr<T>,
) {
  unsafe {
    cppgc__RustObj__RegisterPrefinalizer(
      ptr.pointer.as_ptr(),
      invoke_pre_finalizer::<T>,
    )
  }
}

/// [`make_garbage_collected`] followed by [`register_prefinalizer`].
///
/// # Safety
///
/// See [`make_garbage_collected`].
pub unsafe fn make_garbage_collected_with_prefinalizer<
  T: PreFinalizer + 'static,
>(
  heap: &Heap,
  obj: T,
) -> UnsafePtr<T> {
  unsafe {
    let ptr = make_garbage_collected(heap, obj);
    register_prefinalizer(&ptr);
    ptr
  }
}

/// A per-object weak callback (`Visitor::RegisterWeakCallbackMethod`).
pub trait WeakCallbackMethod {
  fn weak_callback(&self, broker: &LivenessBroker);
}

unsafe extern "C" fn invoke_weak_callback_method<T: WeakCallbackMethod>(
  broker: &LivenessBroker,
  data: *const c_void,
) {
  unsafe { (*(data as *const T)).weak_callback(broker) }
}

impl Visitor {
  /// `Visitor::RegisterWeakCallbackMethod`: call from
  /// [`GarbageCollected::trace`] with the object being traced.
  pub fn register_weak_callback_method<T: WeakCallbackMethod>(
    &mut self,
    object: &T,
  ) {
    unsafe {
      cppgc__Visitor__RegisterWeakCallback(
        self,
        invoke_weak_callback_method::<T>,
        object as *const T as *const c_void,
      )
    }
  }
}

/// `cppgc/testing.h`.
pub mod testing {
  use super::*;

  /// `cppgc::testing::OverrideEmbedderStackStateScope`. Applies to garbage
  /// collections V8 runs for an attached heap.
  #[must_use]
  pub struct OverrideEmbedderStackStateScope<'h> {
    inner: NonNull<OverrideEmbedderStackStateScopeInner>,
    _handle: HeapHandle<'h>,
  }

  impl<'h> OverrideEmbedderStackStateScope<'h> {
    pub fn new(handle: HeapHandle<'h>, state: EmbedderStackState) -> Self {
      let inner = unsafe {
        cppgc__testing__OverrideEmbedderStackStateScope__NEW(
          handle.raw(),
          state,
        )
      };
      Self {
        inner: NonNull::new(inner).unwrap(),
        _handle: handle,
      }
    }
  }

  impl Drop for OverrideEmbedderStackStateScope<'_> {
    fn drop(&mut self) {
      unsafe {
        cppgc__testing__OverrideEmbedderStackStateScope__DELETE(
          self.inner.as_ptr(),
        )
      }
    }
  }

  /// `cppgc::testing::StandaloneTestingHeap`.
  pub struct StandaloneTestingHeap<'h> {
    inner: NonNull<StandaloneTestingHeapInner>,
    handle: HeapHandle<'h>,
  }

  impl<'h> StandaloneTestingHeap<'h> {
    /// # Safety
    ///
    /// `heap` must be a detached heap on which
    /// [`Heap::enable_detached_garbage_collections_for_testing`] was called.
    pub unsafe fn new(heap: &'h Heap) -> Self {
      let handle = heap.heap_handle();
      let inner =
        unsafe { cppgc__testing__StandaloneTestingHeap__NEW(handle.raw()) };
      Self {
        inner: NonNull::new(inner).unwrap(),
        handle,
      }
    }

    pub fn start_garbage_collection(&mut self) {
      unsafe {
        cppgc__testing__StandaloneTestingHeap__StartGarbageCollection(
          self.inner.as_ptr(),
        )
      }
    }

    /// Returns true once marking is done. Panics if no GC is in progress.
    pub fn perform_marking_step(
      &mut self,
      stack_state: EmbedderStackState,
    ) -> bool {
      assert!(HeapState::is_marking(self.handle), "no GC in progress");
      unsafe {
        cppgc__testing__StandaloneTestingHeap__PerformMarkingStep(
          self.inner.as_ptr(),
          stack_state,
        )
      }
    }

    /// Panics if no GC is in progress.
    pub fn finalize_garbage_collection(
      &mut self,
      stack_state: EmbedderStackState,
    ) {
      assert!(HeapState::is_marking(self.handle), "no GC in progress");
      unsafe {
        cppgc__testing__StandaloneTestingHeap__FinalizeGarbageCollection(
          self.inner.as_ptr(),
          stack_state,
        )
      }
    }

    /// Panics if no GC is in progress.
    pub fn toggle_main_thread_marking(&mut self, should_mark: bool) {
      assert!(HeapState::is_marking(self.handle), "no GC in progress");
      unsafe {
        cppgc__testing__StandaloneTestingHeap__ToggleMainThreadMarking(
          self.inner.as_ptr(),
          should_mark,
        )
      }
    }

    pub fn force_compaction_for_next_garbage_collection(&mut self) {
      unsafe {
        cppgc__testing__StandaloneTestingHeap__ForceCompactionForNextGarbageCollection(
          self.inner.as_ptr(),
        )
      }
    }
  }

  impl Drop for StandaloneTestingHeap<'_> {
    fn drop(&mut self) {
      unsafe {
        cppgc__testing__StandaloneTestingHeap__DELETE(self.inner.as_ptr())
      }
    }
  }

  /// `cppgc::testing::IsHeapObjectOld`.
  pub fn is_heap_object_old<T: GarbageCollected>(
    object: &impl GetRustObj<T>,
  ) -> bool {
    let ptr = object.get_rust_obj();
    assert!(!ptr.is_null());
    unsafe { cppgc__testing__IsHeapObjectOld(ptr) }
  }
}

/// `cppgc::subtle::HeapConsistency` write barriers, for references stored
/// without going through [`Member::set`].
pub mod heap_consistency {
  use super::*;

  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum WriteBarrierType {
    None,
    Marking,
    Generational,
  }

  /// # Safety
  ///
  /// `slot` must be a valid address; `value` null or a cppgc object.
  pub unsafe fn get_write_barrier_type(
    slot: *const c_void,
    value: *const c_void,
  ) -> WriteBarrierType {
    match unsafe { cppgc__HeapConsistency__GetWriteBarrierType(slot, value) } {
      0 => WriteBarrierType::None,
      1 => WriteBarrierType::Marking,
      _ => WriteBarrierType::Generational,
    }
  }

  /// The barrier `Member::operator=` runs after storing `value` in `slot`.
  ///
  /// # Safety
  ///
  /// See [`get_write_barrier_type`]; heap thread only.
  pub unsafe fn write_barrier(slot: *const c_void, value: *const c_void) {
    unsafe { cppgc__HeapConsistency__WriteBarrier(slot, value) }
  }

  /// # Safety
  ///
  /// See [`get_write_barrier_type`]; heap thread only.
  pub unsafe fn dijkstra_write_barrier(
    slot: *const c_void,
    value: *const c_void,
  ) -> bool {
    unsafe { cppgc__HeapConsistency__DijkstraWriteBarrier(slot, value) }
  }

  /// # Safety
  ///
  /// `object` must be a live cppgc object; heap thread only.
  pub unsafe fn steele_write_barrier(object: *const c_void) -> bool {
    unsafe { cppgc__HeapConsistency__SteeleWriteBarrier(object) }
  }

  /// # Safety
  ///
  /// See [`get_write_barrier_type`]; heap thread only.
  pub unsafe fn generational_barrier(
    slot: *const c_void,
    value: *const c_void,
  ) -> bool {
    unsafe { cppgc__HeapConsistency__GenerationalBarrier(slot, value) }
  }

  /// The object pointer of a cppgc handle, for use as `value`/`object`.
  pub fn object_ptr<T: GarbageCollected>(
    handle: &impl GetRustObj<T>,
  ) -> *const c_void {
    handle.get_rust_obj() as *const c_void
  }
}
