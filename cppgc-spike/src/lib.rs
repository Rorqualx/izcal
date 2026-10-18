//! Shared support for the cppgc spike binaries.
//!
//! `ffi` declares C++ symbols that the *prebuilt* rusty_v8 static library
//! already exports but that rusty_v8 does not wrap (no shim is compiled; we
//! link to the mangled Itanium-ABI names directly). Every use of these is a
//! finding in itself: rusty_v8 152.2.0 exposes none of them.

pub mod tree;

use std::sync::Once;
use std::time::{Duration, Instant};

pub mod ffi {
  //! Raw bindings to exported-but-unwrapped cppgc / CppHeap symbols.
  //!
  //! Headers: v8/include/cppgc/heap-consistency.h (NoGarbageCollectionScope,
  //! DisallowGarbageCollectionScope), cppgc/heap-state.h (HeapState),
  //! cppgc/testing.h (StandaloneTestingHeap), v8/include/v8-cppgc.h
  //! (CppHeap::GetHeapHandle, CppHeap::CollectStatistics).
  //! None of these appear in rusty_v8's src/cppgc.rs or src/binding.cc.

  #[repr(C)]
  pub struct HeapHandle {
    _p: [u8; 0],
  }

  /// cppgc::EmbedderStackState (enum class, int-sized).
  pub const STACK_MAY_CONTAIN_HEAP_POINTERS: i32 = 0;
  pub const STACK_NO_HEAP_POINTERS: i32 = 1;

  /// Layout of cppgc::testing::StandaloneTestingHeap: a single HeapHandle&.
  #[repr(C)]
  pub struct StandaloneTestingHeap {
    pub heap_handle: *mut HeapHandle,
  }

  /// Leading POD fields of cppgc::HeapStatistics (v8/include/cppgc/heap-statistics.h).
  /// The trailing two std::vector members (libc++, 3 words each) are kept as
  /// opaque words. With DetailLevel::kBrief they are empty, so nothing leaks.
  #[repr(C)]
  pub struct HeapStatisticsRaw {
    pub committed_size_bytes: usize,
    pub resident_size_bytes: usize,
    pub used_size_bytes: usize,
    pub pooled_memory_size_bytes: usize,
    pub detail_level: u8,
    pub space_stats: [usize; 3],
    pub type_names: [usize; 3],
  }

  unsafe extern "C" {
    /// v8::Isolate::RequestGarbageCollectionForTesting(GarbageCollectionType,
    /// cppgc::EmbedderStackState) — rusty_v8 only wraps the 1-arg overload.
    #[link_name = "_ZN2v87Isolate34RequestGarbageCollectionForTestingENS0_21GarbageCollectionTypeEN5cppgc18EmbedderStackStateE"]
    pub fn Isolate_RequestGarbageCollectionForTesting2(isolate: v8::UnsafeRawIsolatePtr, ty: i32, stack: i32);

    #[link_name = "_ZN2v87CppHeap13GetHeapHandleEv"]
    pub fn CppHeap_GetHeapHandle(heap: *mut v8::cppgc::Heap) -> *mut HeapHandle;

    #[link_name = "_ZN2v87CppHeap17CollectStatisticsEN5cppgc14HeapStatistics11DetailLevelE"]
    pub fn CppHeap_CollectStatistics(heap: *mut v8::cppgc::Heap, level: u8) -> HeapStatisticsRaw;

    #[link_name = "_ZN5cppgc6subtle24NoGarbageCollectionScope5EnterERNS_10HeapHandleE"]
    pub fn NoGarbageCollectionScope_Enter(h: *mut HeapHandle);
    #[link_name = "_ZN5cppgc6subtle24NoGarbageCollectionScope5LeaveERNS_10HeapHandleE"]
    pub fn NoGarbageCollectionScope_Leave(h: *mut HeapHandle);

    #[link_name = "_ZN5cppgc6subtle30DisallowGarbageCollectionScope5EnterERNS_10HeapHandleE"]
    pub fn DisallowGarbageCollectionScope_Enter(h: *mut HeapHandle);
    #[link_name = "_ZN5cppgc6subtle30DisallowGarbageCollectionScope5LeaveERNS_10HeapHandleE"]
    pub fn DisallowGarbageCollectionScope_Leave(h: *mut HeapHandle);
    #[link_name = "_ZN5cppgc6subtle30DisallowGarbageCollectionScope26IsGarbageCollectionAllowedERNS_10HeapHandleE"]
    pub fn DisallowGarbageCollectionScope_IsGarbageCollectionAllowed(h: *mut HeapHandle) -> bool;

    #[link_name = "_ZN5cppgc6subtle9HeapState9IsMarkingERKNS_10HeapHandleE"]
    pub fn HeapState_IsMarking(h: *const HeapHandle) -> bool;
    #[link_name = "_ZN5cppgc6subtle9HeapState10IsSweepingERKNS_10HeapHandleE"]
    pub fn HeapState_IsSweeping(h: *const HeapHandle) -> bool;
    #[link_name = "_ZN5cppgc6subtle9HeapState15IsInAtomicPauseERKNS_10HeapHandleE"]
    pub fn HeapState_IsInAtomicPause(h: *const HeapHandle) -> bool;

    #[link_name = "_ZN5cppgc7testing21StandaloneTestingHeap22StartGarbageCollectionEv"]
    pub fn STH_StartGarbageCollection(this: *mut StandaloneTestingHeap);
    #[link_name = "_ZN5cppgc7testing21StandaloneTestingHeap18PerformMarkingStepENS_18EmbedderStackStateE"]
    pub fn STH_PerformMarkingStep(this: *mut StandaloneTestingHeap, stack: i32) -> bool;
    #[link_name = "_ZN5cppgc7testing21StandaloneTestingHeap25FinalizeGarbageCollectionENS_18EmbedderStackStateE"]
    pub fn STH_FinalizeGarbageCollection(this: *mut StandaloneTestingHeap, stack: i32);
    #[link_name = "_ZN5cppgc7testing21StandaloneTestingHeap23ToggleMainThreadMarkingEb"]
    pub fn STH_ToggleMainThreadMarking(this: *mut StandaloneTestingHeap, on: bool);
    #[link_name = "_ZN5cppgc7testing21StandaloneTestingHeap39ForceCompactionForNextGarbageCollectionEv"]
    pub fn STH_ForceCompactionForNextGarbageCollection(this: *mut StandaloneTestingHeap);
  }
}

/// Full unified GC with an explicit stack state (kNoHeapPointers = precise:
/// raw pointers left on the Rust stack do not keep cppgc objects alive).
pub fn full_gc_precise(isolate: &mut v8::Isolate) {
  unsafe { ffi::Isolate_RequestGarbageCollectionForTesting2(isolate.as_raw_isolate_ptr(), 0, ffi::STACK_NO_HEAP_POINTERS) }
}

pub fn heap_handle(heap: &v8::cppgc::Heap) -> *mut ffi::HeapHandle {
  unsafe { ffi::CppHeap_GetHeapHandle(heap as *const _ as *mut _) }
}

/// RAII wrapper over cppgc::subtle::NoGarbageCollectionScope::Enter/Leave.
pub struct NoGcScope(*mut ffi::HeapHandle);
impl NoGcScope {
  pub fn enter(h: *mut ffi::HeapHandle) -> Self {
    unsafe { ffi::NoGarbageCollectionScope_Enter(h) };
    NoGcScope(h)
  }
}
impl Drop for NoGcScope {
  fn drop(&mut self) {
    unsafe { ffi::NoGarbageCollectionScope_Leave(self.0) };
  }
}

pub fn is_marking(h: *mut ffi::HeapHandle) -> bool {
  unsafe { ffi::HeapState_IsMarking(h) }
}
pub fn is_sweeping(h: *mut ffi::HeapHandle) -> bool {
  unsafe { ffi::HeapState_IsSweeping(h) }
}

/// Thin safe-ish wrapper for cppgc::testing::StandaloneTestingHeap. Only valid
/// for a detached CppHeap with detached GCs enabled.
pub struct TestingHeap(ffi::StandaloneTestingHeap);
impl TestingHeap {
  pub fn new(h: *mut ffi::HeapHandle) -> Self {
    TestingHeap(ffi::StandaloneTestingHeap { heap_handle: h })
  }
  pub fn start(&mut self) {
    unsafe { ffi::STH_StartGarbageCollection(&mut self.0) }
  }
  pub fn step(&mut self, no_heap_ptrs: bool) -> bool {
    unsafe { ffi::STH_PerformMarkingStep(&mut self.0, stack(no_heap_ptrs)) }
  }
  pub fn finalize(&mut self, no_heap_ptrs: bool) {
    unsafe { ffi::STH_FinalizeGarbageCollection(&mut self.0, stack(no_heap_ptrs)) }
  }
  pub fn toggle_main_thread_marking(&mut self, on: bool) {
    unsafe { ffi::STH_ToggleMainThreadMarking(&mut self.0, on) }
  }
  pub fn force_compaction_next(&mut self) {
    unsafe { ffi::STH_ForceCompactionForNextGarbageCollection(&mut self.0) }
  }
}
fn stack(no_heap_ptrs: bool) -> i32 {
  if no_heap_ptrs { ffi::STACK_NO_HEAP_POINTERS } else { ffi::STACK_MAY_CONTAIN_HEAP_POINTERS }
}

pub struct CppgcStats {
  pub committed: usize,
  pub resident: usize,
  pub used: usize,
}
pub fn cppgc_stats(heap: &v8::cppgc::Heap) -> CppgcStats {
  let s = unsafe { ffi::CppHeap_CollectStatistics(heap as *const _ as *mut _, 0) };
  assert_eq!(s.space_stats[0], s.space_stats[1], "kBrief should leave space_stats empty");
  CppgcStats { committed: s.committed_size_bytes, resident: s.resident_size_bytes, used: s.used_size_bytes }
}

static INIT: Once = Once::new();
static mut PLATFORM: Option<v8::SharedRef<v8::Platform>> = None;

/// Initialize V8 + cppgc once per process. `flags` are V8 flags.
pub fn init_v8(flags: &str) -> v8::SharedRef<v8::Platform> {
  INIT.call_once(|| {
    if !flags.is_empty() {
      v8::V8::set_flags_from_string(flags);
    }
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform.clone());
    v8::cppgc::initialize_process(platform.clone());
    v8::V8::initialize();
    unsafe { PLATFORM = Some(platform) };
  });
  #[allow(static_mut_refs)]
  unsafe { PLATFORM.clone().unwrap() }
}

/// p50 / p99 / max of a set of durations (ns).
pub fn percentiles(mut v: Vec<u64>) -> (u64, u64, u64, u64) {
  v.sort_unstable();
  let n = v.len();
  let p = |q: f64| v[((n as f64 - 1.0) * q).round() as usize];
  (p(0.5), p(0.99), p(0.999), v[n - 1])
}

pub fn fmt_pct(name: &str, v: Vec<u64>) -> String {
  let n = v.len();
  let (p50, p99, p999, max) = percentiles(v);
  format!("{name}: n={n} p50={p50}ns p99={p99}ns p99.9={p999}ns max={max}ns")
}

/// Outcome of running a scenario in a child process under a watchdog.
#[derive(Debug)]
pub enum ChildOutcome {
  Ok,
  ExitCode(i32),
  Signal(i32),
  TimedOut,
}

/// Re-exec the current binary with `args`, kill it after `timeout`.
/// Child stdout/stderr are inherited so the log shows everything.
pub fn run_child(args: &[&str], timeout: Duration) -> (ChildOutcome, Duration) {
  use std::os::unix::process::ExitStatusExt;
  let exe = std::env::current_exe().unwrap();
  let start = Instant::now();
  let mut child = std::process::Command::new(exe).args(args).spawn().expect("spawn child");
  loop {
    if let Some(st) = child.try_wait().unwrap() {
      let o = if st.success() {
        ChildOutcome::Ok
      } else if let Some(sig) = st.signal() {
        ChildOutcome::Signal(sig)
      } else {
        ChildOutcome::ExitCode(st.code().unwrap_or(-1))
      };
      return (o, start.elapsed());
    }
    if start.elapsed() > timeout {
      let _ = child.kill();
      let _ = child.wait();
      return (ChildOutcome::TimedOut, start.elapsed());
    }
    std::thread::sleep(Duration::from_millis(5));
  }
}

/// A raw pointer we promise to only dereference inside a no-GC scope.
pub struct SendPtr<T>(pub *const T);
impl<T> Clone for SendPtr<T> {
  fn clone(&self) -> Self {
    *self
  }
}
impl<T> Copy for SendPtr<T> {}
unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

/// Run `f` on a helper thread; panic (abort the process) if it takes longer
/// than `timeout`. Used as the scope-entry watchdog inside a child.
pub fn watchdog<R>(what: &str, timeout: Duration, f: impl FnOnce() -> R) -> R {
  let (tx, rx) = std::sync::mpsc::channel::<()>();
  let what = what.to_string();
  let guard = std::thread::spawn(move || {
    if rx.recv_timeout(timeout).is_err() {
      eprintln!("WATCHDOG: '{what}' did not complete within {timeout:?} -> treating as DEADLOCK");
      std::process::exit(86);
    }
  });
  let r = f();
  let _ = tx.send(());
  let _ = guard.join();
  r
}
