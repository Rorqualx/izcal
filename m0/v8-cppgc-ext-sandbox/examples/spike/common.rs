//! Support code shared by the ported cppgc-spike programs.
//!
//! This is the spike's `src/lib.rs` with `mod ffi` (mangled Itanium symbols,
//! hand-copied `HeapStatistics` layout) removed: every helper below is now a
//! thin alias over the `v8_cppgc_ext` crate, so the item programs needed only
//! minimal edits.
#![allow(dead_code)]

use std::sync::Once;
use std::time::{Duration, Instant};
use v8_cppgc_ext::testing::StandaloneTestingHeap;
use v8_cppgc_ext::{CppHeapExt, DetailLevel, HeapHandle, HeapState, IsolateExt, NoGarbageCollectionScope};

pub type Handle<'h> = HeapHandle<'h>;

/// Full unified GC with an explicit stack state (kNoHeapPointers = precise).
pub fn full_gc_precise(isolate: &mut v8::Isolate) {
  isolate.request_garbage_collection_for_testing_with_stack_state(
    v8::GarbageCollectionType::Full,
    v8::cppgc::EmbedderStackState::NoHeapPointers,
  );
}

pub fn heap_handle(heap: &v8::cppgc::Heap) -> HeapHandle<'_> {
  heap.heap_handle()
}

/// The spike's `NoGcScope::enter(h)` is now `NoGarbageCollectionScope::new(h)`.
pub struct NoGcScope;
impl NoGcScope {
  pub fn enter(h: HeapHandle<'_>) -> NoGarbageCollectionScope<'_> {
    NoGarbageCollectionScope::new(h)
  }
}

pub fn is_marking(h: HeapHandle<'_>) -> bool {
  HeapState::is_marking(h)
}
pub fn is_sweeping(h: HeapHandle<'_>) -> bool {
  HeapState::is_sweeping(h)
}

fn stack(no_heap_ptrs: bool) -> v8::cppgc::EmbedderStackState {
  if no_heap_ptrs {
    v8::cppgc::EmbedderStackState::NoHeapPointers
  } else {
    v8::cppgc::EmbedderStackState::MayContainHeapPointers
  }
}

/// The spike's `TestingHeap` over `v8_cppgc_ext::testing::StandaloneTestingHeap`.
pub struct TestingHeap<'h>(StandaloneTestingHeap<'h>);
impl<'h> TestingHeap<'h> {
  /// `heap` must be detached with detached testing GCs enabled.
  pub fn new(heap: &'h v8::cppgc::Heap) -> Self {
    TestingHeap(unsafe { StandaloneTestingHeap::new(heap) })
  }
  pub fn start(&mut self) {
    self.0.start_garbage_collection()
  }
  pub fn step(&mut self, no_heap_ptrs: bool) -> bool {
    self.0.perform_marking_step(stack(no_heap_ptrs))
  }
  pub fn finalize(&mut self, no_heap_ptrs: bool) {
    self.0.finalize_garbage_collection(stack(no_heap_ptrs))
  }
  pub fn toggle_main_thread_marking(&mut self, on: bool) {
    self.0.toggle_main_thread_marking(on)
  }
  pub fn force_compaction_next(&mut self) {
    self.0.force_compaction_for_next_garbage_collection()
  }
}

pub struct CppgcStats {
  pub committed: usize,
  pub resident: usize,
  pub used: usize,
}
/// `CppHeap::CollectStatistics(kBrief)` through the shim (Rust-owned result).
pub fn cppgc_stats(heap: &v8::cppgc::Heap) -> CppgcStats {
  let s = heap.collect_statistics(DetailLevel::Brief);
  assert!(s.space_stats.is_empty(), "kBrief should leave space_stats empty");
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
  unsafe {
    PLATFORM.clone().unwrap()
  }
}

/// p50 / p99 / p99.9 / max of a set of durations (ns).
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

/// Run `f` on the current thread; abort the process if it takes longer than
/// `timeout` (scope-entry deadlock watchdog).
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

pub fn _unused(_: &mut v8::Isolate) {
  let _ = <v8::Isolate as IsolateExt>::cpp_heap_handle;
}
