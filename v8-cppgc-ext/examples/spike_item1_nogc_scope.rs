//! Item 1: no-GC + no-compaction held together across a worker phase,
//! including entry while concurrent marking is already running.
//!
//! The parent process runs each scenario in a child under a watchdog and
//! classifies the outcome (ok / assert-crash / deadlock-timeout).
//!
//! Ported from cppgc-spike to v8-cppgc-ext (no mangled symbols).
//!
//!   cargo run --release --example spike_item1_nogc_scope            # all scenarios
//!   cargo run --release --example spike_item1_nogc_scope -- child <scenario>

#[path = "spike/common.rs"]
mod common;
#[path = "spike/tree.rs"]
mod tree;
use common::*;
use tree::{TNode, FINALIZED, FINALIZED_LIVE_IDS};
use v8_cppgc_ext::DisallowGarbageCollectionScope;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use v8::cppgc::{EmbedderStackState, Heap, HeapCreateParams, Persistent};

const N: u32 = 20_000;
const GARBAGE: u32 = 5_000;
const WORKERS: usize = 8;
const SCOPE_WATCHDOG: Duration = Duration::from_secs(5);

const SCENARIOS: &[&str] = &[
  "detached-quiescent",
  "detached-marking",
  "detached-latency",
  "compaction-probe",
  "unified-marking",
  "unified-latency",
  "unified-alloc-only",
  "unified-blocked-owner",
  "unified-disallow",
];

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() >= 3 && args[1] == "child" {
    child(&args[2]);
    return;
  }
  println!("item1: V8 {} ; each scenario runs in its own process under a 120 s watchdog", v8::V8::get_version());
  for s in SCENARIOS {
    println!("\n===== scenario {s} =====");
    let (o, t) = run_child(&["child", s], Duration::from_secs(120));
    println!("===== scenario {s}: outcome={o:?} wall={t:?}");
  }
}

fn child(s: &str) {
  match s {
    "detached-quiescent" => detached(false),
    "detached-marking" => detached(true),
    "detached-latency" => detached_latency(),
    "compaction-probe" => compaction_probe(),
    "unified-marking" => unified_marking(true),
    "unified-alloc-only" => unified_marking(false),
    "unified-blocked-owner" => unified_blocked_owner(),
    "unified-latency" => unified_latency(),
    "unified-disallow" => unified_disallow(),
    _ => panic!("unknown scenario {s}"),
  }
}

/// Spawn reader threads that traverse the tree through raw pointers until
/// `stop` is set. Returns join handles yielding (traversals, first error).
fn spawn_readers(
  root: *const TNode,
  stop: Arc<AtomicBool>,
) -> Vec<std::thread::JoinHandle<(u64, Option<String>)>> {
  let root = SendPtr(root);
  let expect = tree::expected_checksum(N);
  (0..WORKERS)
    .map(|_| {
      let stop = stop.clone();
      std::thread::spawn(move || {
        let root = root;
        let mut iters = 0u64;
        loop {
          match tree::traverse(root.0) {
            Ok(r) if r == expect => {}
            Ok(r) => return (iters, Some(format!("checksum mismatch {r:?} != {expect:?}"))),
            Err(e) => return (iters, Some(e)),
          }
          iters += 1;
          if stop.load(Ordering::Acquire) && iters >= 3 {
            return (iters, None);
          }
        }
      })
    })
    .collect()
}

fn join_readers(hs: Vec<std::thread::JoinHandle<(u64, Option<String>)>>) -> (u64, Vec<String>) {
  let mut total = 0;
  let mut errs = vec![];
  for h in hs {
    let (n, e) = h.join().unwrap();
    total += n;
    errs.extend(e);
  }
  (total, errs)
}

fn sample_addrs(root: *const TNode) -> Vec<usize> {
  let mut v = vec![];
  let mut p = unsafe { (*root).first_child() };
  while let Some(n) = p {
    v.push(n as *const _ as usize);
    p = unsafe { n.first_child() };
  }
  v
}

/// Detached (standalone) CppHeap, GC driven by cppgc::testing::StandaloneTestingHeap.
fn detached(start_marking_first: bool) {
  let platform = init_v8("");
  let heap = Heap::create(platform, HeapCreateParams::default());
  heap.enable_detached_garbage_collections_for_testing();
  let h = heap_handle(&heap);
  let mut th = TestingHeap::new(&heap);

  let root = Persistent::new(&tree::build_tree(&heap, N, 4));
  let root_ptr = root.get().unwrap() as *const TNode;
  tree::alloc_garbage(&heap, GARBAGE);
  let addrs_before = sample_addrs(root_ptr);

  if start_marking_first {
    th.start();
    th.step(true);
    println!("before-entry: is_marking={} is_sweeping={}", is_marking(h), is_sweeping(h));
    assert!(is_marking(h), "expected incremental marking to be running");
  }

  let t0 = Instant::now();
  let scope = watchdog("NoGarbageCollectionScope::Enter", SCOPE_WATCHDOG, || NoGcScope::enter(h));
  let entry_ns = t0.elapsed().as_nanos();
  println!("scope-entry (single shot, incl. watchdog thread spawn): {entry_ns}ns is_marking_after_entry={}", is_marking(h));

  let fin_at_entry = FINALIZED.load(Ordering::SeqCst);
  let stop = Arc::new(AtomicBool::new(false));
  let readers = spawn_readers(root_ptr, stop.clone());

  // While workers read, the owning thread throws every GC request it has.
  let mut attempts = 0u32;
  let mut steps_done = 0u32;
  let mut marking_finished_by_step = false;
  let t = Instant::now();
  // Only calls that are legal inside a no-GC scope (debug V8 DCHECKs that the
  // StandaloneTestingHeap start/finalize entry points are *not* used in one):
  // marking steps, and CppHeap::CollectGarbageForTesting (an atomic GC request
  // that cppgc must refuse while the scope is held).
  while t.elapsed() < Duration::from_millis(1500) {
    if is_marking(h) {
      if th.step(true) {
        marking_finished_by_step = true;
      }
      steps_done += 1;
    }
    watchdog("collect_garbage_for_testing under scope", SCOPE_WATCHDOG, || unsafe {
      heap.collect_garbage_for_testing(EmbedderStackState::NoHeapPointers)
    });
    attempts += 1;
    std::thread::sleep(Duration::from_micros(200));
  }
  stop.store(true, Ordering::Release);
  let (traversals, errs) = join_readers(readers);
  let fin_in_scope = FINALIZED.load(Ordering::SeqCst) - fin_at_entry;
  println!(
    "in-scope: gc_attempts={attempts} marking_steps={steps_done} marking_done_flag={marking_finished_by_step} \
     still_marking={} finalized_during_scope={fin_in_scope} live_finalized={} reader_traversals={traversals} reader_errors={errs:?}",
    is_marking(h),
    FINALIZED_LIVE_IDS.load(Ordering::SeqCst)
  );
  let addrs_after = sample_addrs(root_ptr);
  println!("addresses-stable-in-scope={}", addrs_before == addrs_after);
  drop(scope);

  // Out of scope: finalization must now proceed and collect the garbage.
  if is_marking(h) {
    th.finalize(true);
  }
  unsafe { heap.collect_garbage_for_testing(EmbedderStackState::NoHeapPointers) };
  let addrs_after_gc = sample_addrs(root_ptr);
  println!(
    "after-scope: finalized_total={} (garbage={GARBAGE}) live_finalized={} addresses-stable-after-gc={} tree_ok={:?}",
    FINALIZED.load(Ordering::SeqCst),
    FINALIZED_LIVE_IDS.load(Ordering::SeqCst),
    addrs_before == addrs_after_gc,
    tree::traverse(root_ptr).map(|r| r == tree::expected_checksum(N))
  );
  let verdict = errs.is_empty() && fin_in_scope == 0 && addrs_before == addrs_after_gc;
  println!("RESULT {}", if verdict { "PASS" } else { "FAIL" });
  drop(root);
}

/// Can cppgc compaction move rusty_v8 objects at all? rusty_v8 creates its
/// CppHeap with no custom spaces (binding.cc: `v8::CppHeapCreateParams
/// params{{}}`), and only custom spaces can be compactable. Force compaction for
/// the next GC and check addresses. Debug V8 DCHECKs in Compactor::ShouldCompact
/// when compaction is forced on a heap with no compactable spaces.
fn compaction_probe() {
  let platform = init_v8("");
  let heap = Heap::create(platform, HeapCreateParams::default());
  heap.enable_detached_garbage_collections_for_testing();
  let mut th = TestingHeap::new(&heap);
  let root = Persistent::new(&tree::build_tree(&heap, N, 4));
  let root_ptr = root.get().unwrap() as *const TNode;
  tree::alloc_garbage(&heap, GARBAGE); // fragmentation, so compaction would have work
  let before = sample_addrs(root_ptr);
  for _ in 0..3 {
    th.force_compaction_next();
    th.start();
    while !th.step(true) {}
    th.finalize(true);
  }
  println!(
    "compaction forced for 3 incremental GCs: finalized={} addresses-stable={} tree_ok={:?}",
    FINALIZED.load(Ordering::SeqCst),
    before == sample_addrs(root_ptr),
    tree::traverse(root_ptr).map(|r| r == tree::expected_checksum(N))
  );
  drop(root);
}

// Latency loops: a hang shows up as the parent's 120 s watchdog firing, and a
// slow entry shows up in `max`; no per-entry watchdog thread (it would dominate
// the measurement).
fn detached_latency() {
  let platform = init_v8("");
  let heap = Heap::create(platform, HeapCreateParams::default());
  heap.enable_detached_garbage_collections_for_testing();
  let h = heap_handle(&heap);
  let mut th = TestingHeap::new(&heap);
  let root = Persistent::new(&tree::build_tree(&heap, N, 4));

  let mut idle = Vec::with_capacity(100_000);
  for _ in 0..100_000 {
    let t = Instant::now();
    let s = NoGcScope::enter(h);
    idle.push(t.elapsed().as_nanos() as u64);
    drop(s);
  }
  let mut marking = Vec::with_capacity(20_000);
  for i in 0..20_000 {
    if !is_marking(h) {
      tree::alloc_garbage(&heap, 50);
      th.start();
    }
    th.step(true);
    if !is_marking(h) {
      continue;
    }
    let t = Instant::now();
    let s = NoGcScope::enter(h);
    marking.push(t.elapsed().as_nanos() as u64);
    drop(s);
    if i % 50 == 49 {
      th.finalize(true);
    }
  }
  let t = Instant::now();
  for _ in 0..10_000_000 {
    drop(std::hint::black_box(NoGcScope::enter(h)));
  }
  println!("amortized enter+leave pair: {:.2}ns (10M pairs; Instant tick on this Mac is ~41.7ns)", t.elapsed().as_nanos() as f64 / 1e7);
  println!("{}", fmt_pct("detached entry latency, idle heap", idle));
  println!("{}", fmt_pct("detached entry latency, during incremental+concurrent marking", marking));
  drop(root);
}

/// Unified heap: the isolate's own CppHeap; V8 drives marking.
fn new_isolate() -> v8::OwnedIsolate {
  v8::Isolate::new(v8::CreateParams::default())
}

fn alloc_js_garbage(scope: &mut v8::PinScope, rounds: u32) {
  let src = format!(
    "{{ let a = []; for (let i = 0; i < {rounds}; i++) {{ a.push(new Array(64).fill(i)); if (a.length > 2000) a = []; }} }}"
  );
  let code = v8::String::new(scope, &src).unwrap();
  let script = v8::Script::compile(scope, code, None).unwrap();
  script.run(scope).unwrap();
}

fn unified_marking(explicit_full_gc: bool) {
  let _platform = init_v8("--expose-gc --stress-incremental-marking");
  let isolate = &mut new_isolate();
  let heap: &Heap = unsafe { &*(isolate.get_cpp_heap().unwrap() as *const Heap) };
  let h = heap_handle(heap);
  let root = Persistent::new(&tree::build_tree(heap, N, 4));
  let root_ptr = root.get().unwrap() as *const TNode;
  tree::alloc_garbage(heap, GARBAGE);

  v8::scope!(let hs, isolate);
  let ctx = v8::Context::new(hs, Default::default());
  let scope = &mut v8::ContextScope::new(hs, ctx);

  // Drive JS allocation until V8 starts incremental marking (which includes the CppHeap).
  let t = Instant::now();
  let mut rounds = 0;
  while !is_marking(h) && t.elapsed() < Duration::from_secs(20) {
    alloc_js_garbage(scope, 200);
    rounds += 1;
  }
  println!("marking-started={} after {rounds} JS alloc rounds ({:?})", is_marking(h), t.elapsed());
  if !is_marking(h) {
    println!("RESULT INCONCLUSIVE (could not observe unified marking start)");
    return;
  }

  let fin0 = FINALIZED.load(Ordering::SeqCst);
  let t0 = Instant::now();
  let ns = watchdog("NoGarbageCollectionScope::Enter (unified, marking)", SCOPE_WATCHDOG, || NoGcScope::enter(h));
  println!(
    "scope-entry (single shot, incl. watchdog thread spawn): {}ns is_marking_after_entry={}",
    t0.elapsed().as_nanos(),
    is_marking(h)
  );

  let stop = Arc::new(AtomicBool::new(false));
  let readers = spawn_readers(root_ptr, stop.clone());
  // Keep the mutator allocating JS (which drives and tries to finalize marking),
  // and also request explicit full GCs.
  let mut full_gc_requests = 0;
  let mut marking_ended_in_scope = false;
  let t = Instant::now();
  while t.elapsed() < Duration::from_millis(1500) {
    alloc_js_garbage(scope, 200);
    if !is_marking(h) {
      marking_ended_in_scope = true;
    }
    if explicit_full_gc && full_gc_requests < 5 {
      watchdog("request_garbage_collection_for_testing(Full) under scope", SCOPE_WATCHDOG, || {
        scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full)
      });
      full_gc_requests += 1;
    }
  }
  stop.store(true, Ordering::Release);
  let (traversals, errs) = join_readers(readers);
  let fin_in_scope = FINALIZED.load(Ordering::SeqCst) - fin0;
  println!(
    "in-scope: full_gc_requests={full_gc_requests} marking_ended_during_scope={marking_ended_in_scope} \
     finalized_during_scope={fin_in_scope} live_finalized={} reader_traversals={traversals} reader_errors={errs:?}",
    FINALIZED_LIVE_IDS.load(Ordering::SeqCst)
  );
  drop(ns);
  scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
  println!(
    "after-scope: finalized_total={} (garbage={GARBAGE}) live_finalized={} tree_ok={:?}",
    FINALIZED.load(Ordering::SeqCst),
    FINALIZED_LIVE_IDS.load(Ordering::SeqCst),
    tree::traverse(root_ptr).map(|r| r == tree::expected_checksum(N))
  );
  let ok = errs.is_empty() && FINALIZED_LIVE_IDS.load(Ordering::SeqCst) == 0;
  println!(
    "RESULT readers_ok={ok} scope_blocked_v8_initiated_finalization={} (cppgc objects finalized inside scope: {fin_in_scope})",
    fin_in_scope == 0
  );
  drop(root);
}

fn unified_latency() {
  let _platform = init_v8("--expose-gc --stress-incremental-marking");
  let isolate = &mut new_isolate();
  let heap: &Heap = unsafe { &*(isolate.get_cpp_heap().unwrap() as *const Heap) };
  let h = heap_handle(heap);
  let root = Persistent::new(&tree::build_tree(heap, N, 4));
  v8::scope!(let hs, isolate);
  let ctx = v8::Context::new(hs, Default::default());
  let scope = &mut v8::ContextScope::new(hs, ctx);
  let mut idle = vec![];
  let mut marking = vec![];
  let t = Instant::now();
  while (marking.len() < 20_000 || idle.len() < 20_000) && t.elapsed() < Duration::from_secs(60) {
    alloc_js_garbage(scope, 20);
    for _ in 0..50 {
      let m = is_marking(h);
      let t = Instant::now();
      let s = NoGcScope::enter(h);
      let d = t.elapsed().as_nanos() as u64;
      drop(s);
      if m { marking.push(d) } else { idle.push(d) }
    }
  }
  println!("{}", fmt_pct("unified entry latency, not marking", idle));
  println!("{}", fmt_pct("unified entry latency, V8 incremental marking active", marking));
  drop(root);
}

/// The pattern the spec actually relies on: the owning (script) thread holds
/// the scope(s) and *blocks* until workers finish, while V8 incremental +
/// concurrent marking is in progress. Both NoGarbageCollectionScope and
/// DisallowGarbageCollectionScope are held, so any GC finalization attempted
/// on the owning thread during the phase would crash (CHECK) rather than pass
/// silently.
fn unified_blocked_owner() {
  let _platform = init_v8("--expose-gc --stress-incremental-marking");
  let isolate = &mut new_isolate();
  let heap: &Heap = unsafe { &*(isolate.get_cpp_heap().unwrap() as *const Heap) };
  let h = heap_handle(heap);
  let root = Persistent::new(&tree::build_tree(heap, N, 4));
  let root_ptr = root.get().unwrap() as *const TNode;
  v8::scope!(let hs, isolate);
  let ctx = v8::Context::new(hs, Default::default());
  let scope = &mut v8::ContextScope::new(hs, ctx);
  let (mut phases, mut phases_marking, mut fin_in_phases, mut errors) = (0, 0, 0, vec![]);
  let mut traversals = 0;
  let t = Instant::now();
  while t.elapsed() < Duration::from_secs(10) {
    // Mutator work between phases: allocate JS + cppgc garbage so GCs happen.
    alloc_js_garbage(scope, 100);
    tree::alloc_garbage(heap, 200);
    let marking = is_marking(h);
    let fin0 = FINALIZED.load(Ordering::SeqCst);
    let worker_phase = || {
      let _ng = NoGcScope::enter(h);
      let _disallow = DisallowGarbageCollectionScope::new(h);
      let stop = Arc::new(AtomicBool::new(true)); // each worker does 3 traversals
      join_readers(spawn_readers(root_ptr, stop))
    };
    let (n, e) = watchdog("worker phase", SCOPE_WATCHDOG, worker_phase);
    traversals += n;
    errors.extend(e);
    fin_in_phases += FINALIZED.load(Ordering::SeqCst) - fin0;
    phases += 1;
    if marking {
      phases_marking += 1;
    }
  }
  scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
  println!(
    "blocked-owner: phases={phases} phases_started_during_marking={phases_marking} \
     finalized_during_phases={fin_in_phases} finalized_total={} live_finalized={} traversals={traversals} errors={errors:?}",
    FINALIZED.load(Ordering::SeqCst),
    FINALIZED_LIVE_IDS.load(Ordering::SeqCst)
  );
  let ok = errors.is_empty() && fin_in_phases == 0 && FINALIZED_LIVE_IDS.load(Ordering::SeqCst) == 0;
  println!("RESULT {}", if ok { "PASS" } else { "FAIL" });
  drop(root);
}

/// Informational: DisallowGarbageCollectionScope crashes on any GC trigger by
/// design, so it is unusable for a worker phase in which V8 may start a GC.
fn unified_disallow() {
  let _platform = init_v8("--expose-gc");
  let isolate = &mut new_isolate();
  let heap: &Heap = unsafe { &*(isolate.get_cpp_heap().unwrap() as *const Heap) };
  let h = heap_handle(heap);
  let _disallow = DisallowGarbageCollectionScope::new(h);
  println!(
    "disallow-scope entered; IsGarbageCollectionAllowed={}",
    DisallowGarbageCollectionScope::is_garbage_collection_allowed(h)
  );
  println!("requesting full GC inside DisallowGarbageCollectionScope (expect a crash)...");
  isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
  println!("RESULT full GC returned normally inside DisallowGarbageCollectionScope");
}
