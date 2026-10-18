//! Item 2: cross-thread reads of cppgc objects through raw pointers (no
//! CrossThreadPersistent), inside the item-1 pattern, under stress.
//!
//! Each cycle: the script thread mutates the tree with barriered Member writes
//! (insert/remove subtrees, creating garbage) while V8 incremental+concurrent
//! marking is running, then enters NoGarbageCollectionScope +
//! DisallowGarbageCollectionScope, hands raw `*const TNode` to worker threads,
//! and blocks until they finish. Workers verify every node (magic word,
//! payload, checksum equal to the owner's own traversal).
//!
//! Ported from cppgc-spike to v8-cppgc-ext (no mangled symbols).
//!
//!   cargo run --release --example spike_item2_crossthread_raw [-- --secs 20] [--racy-control]
//!
//! `--racy-control` adds a deliberate unsynchronized Rust write from a worker
//! to a field other workers read, to prove TSAN (when enabled) sees Rust races.

#[path = "spike/common.rs"]
mod common;
#[path = "spike/tree.rs"]
mod tree;
use common::*;
use tree::{TNode, FINALIZED, FINALIZED_LIVE_IDS};
use v8_cppgc_ext::DisallowGarbageCollectionScope;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use v8::cppgc::{Heap, Persistent};

const WORKERS: usize = 8;

struct Rng(u64);
impl Rng {
  fn next(&mut self) -> u64 {
    self.0 ^= self.0 << 13;
    self.0 ^= self.0 >> 7;
    self.0 ^= self.0 << 17;
    self.0
  }
  fn below(&mut self, n: usize) -> usize {
    (self.next() % n as u64) as usize
  }
}

fn collect(root: *const TNode) -> Vec<*const TNode> {
  let mut out = vec![];
  let mut st = vec![root];
  while let Some(p) = st.pop() {
    out.push(p);
    unsafe {
      if let Some(c) = (*p).first_child() {
        st.push(c);
      }
      if let Some(s) = (*p).next_sibling() {
        st.push(s);
      }
    }
  }
  out
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let secs: u64 = args.iter().position(|a| a == "--secs").map(|i| args[i + 1].parse().unwrap()).unwrap_or(20);
  let racy = args.iter().any(|a| a == "--racy-control");

  let _p = init_v8("--expose-gc --stress-incremental-marking");
  let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
  let heap: &Heap = unsafe { &*(isolate.get_cpp_heap().unwrap() as *const Heap) };
  let h = heap_handle(heap);
  let root = Persistent::new(&tree::build_tree(heap, 5_000, 4));
  let root_ptr = SendPtr(root.get().unwrap() as *const TNode);
  v8::scope!(let hs, isolate);
  let ctx = v8::Context::new(hs, Default::default());
  let scope = &mut v8::ContextScope::new(hs, ctx);
  let js = v8::String::new(scope, "{ let a = []; for (let i = 0; i < 100; i++) a.push(new Array(64).fill(i)); }").unwrap();
  let js = v8::Script::compile(scope, js, None).unwrap();

  let mut rng = Rng(0x9E3779B97F4A7C15);
  let mut next_id = 1_000_000u32;
  let (mut cycles, mut cycles_marking, mut inserts, mut removes, mut reads) = (0u64, 0u64, 0u64, 0u64, 0u64);
  let mut fin_during_phases = 0usize;
  let mut errors: Vec<String> = vec![];
  let start = Instant::now();
  while start.elapsed() < Duration::from_secs(secs) && errors.is_empty() {
    // ---- mutator phase (script thread only; barriered writes) ----
    js.run(scope).unwrap();
    let nodes = collect(root_ptr.0);
    for _ in 0..20 {
      let p = unsafe { &*nodes[rng.below(nodes.len())] };
      if rng.below(2) == 0 {
        // insert a 3-node chain as the new first child
        let a = unsafe { v8::cppgc::make_garbage_collected(heap, TNode::new(7, next_id)) };
        let b = unsafe { v8::cppgc::make_garbage_collected(heap, TNode::new(8, next_id + 1)) };
        next_id += 2;
        unsafe {
          a.as_ref().set_next_sibling(&b);
          // b.next_sibling = p.first_child (may be empty); p.first_child = a
          (*b.as_ref().next_sibling.get()).set(&*p.first_child.get());
          p.set_first_child(&a);
        }
        inserts += 1;
      } else {
        // remove the first child's whole subtree (becomes garbage)
        unsafe {
          if let Some(c) = p.first_child() {
            let next = &*c.next_sibling.get();
            (*p.first_child.get()).set(next);
            removes += 1;
          }
        }
      }
    }
    let expect = tree::traverse(root_ptr.0).expect("owner traversal");
    let marking = is_marking(h);

    // ---- worker phase: owner holds both scopes and blocks ----
    let fin0 = FINALIZED.load(Ordering::SeqCst);
    let _ng = NoGcScope::enter(h);
    let disallow = DisallowGarbageCollectionScope::new(h);
    let results: Vec<_> = (0..WORKERS)
      .map(|w| {
        std::thread::spawn(move || {
          let root = root_ptr;
          if racy && w == 0 {
            // Deliberate data race for the TSAN control run.
            unsafe {
              let p1 = (&(*root.0).payload as *const [u64; 2] as *mut u64).add(1);
              std::ptr::write(p1, 7 * (*root.0).id as u64);
            }
          }
          let mut out = Ok(());
          for _ in 0..2 {
            match tree::traverse(root.0) {
              Ok(r) if r == expect => {}
              Ok(r) => out = Err(format!("worker {w}: {r:?} != {expect:?}")),
              Err(e) => out = Err(format!("worker {w}: {e}")),
            }
          }
          out
        })
      })
      .collect::<Vec<_>>()
      .into_iter()
      .map(|j| j.join().unwrap())
      .collect();
    drop(disallow);
    drop(_ng);
    fin_during_phases += FINALIZED.load(Ordering::SeqCst) - fin0;
    for r in results {
      match r {
        Ok(()) => reads += 2,
        Err(e) => errors.push(e),
      }
    }
    cycles += 1;
    cycles_marking += marking as u64;
  }
  scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
  println!(
    "item2: secs={secs} racy_control={racy} cycles={cycles} cycles_with_marking_active={cycles_marking} \
     inserts={inserts} removes={removes} worker_traversals={reads} finalized_total={} \
     finalized_during_worker_phases={fin_during_phases} removed_nodes_finalized={}",
    FINALIZED.load(Ordering::SeqCst),
    FINALIZED_LIVE_IDS.load(Ordering::SeqCst)
  );
  println!("errors={errors:?}");
  println!("RESULT {}", if errors.is_empty() && fin_during_phases == 0 { "PASS" } else { "FAIL" });
  drop(root);
}
