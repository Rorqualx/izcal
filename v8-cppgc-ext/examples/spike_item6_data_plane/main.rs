//! Item 6: data-plane memory and flush cost (HGBA-3.0 D8 / B4 / INC-3).
//!
//! Builds a toy cppgc DOM at 1k / 10k / 100k nodes (tag, ~4 attributes, class
//! list, state flags, tree links, inline-style reference, wrapper slot), the F1
//! Rust-owned data plane fed only through the mutation log, and measures:
//! bytes/node, log append cost, flush cost for N = 1 / 100 / 10k, and a fresh
//! projection + equality check.
//!
//! Ported from cppgc-spike to v8-cppgc-ext: the cppgc heap is measured with
//! `CppHeapExt::collect_statistics` (kBrief, plus a kDetailed per-type
//! breakdown that the spike could not read) instead of a hand-copied
//! `cppgc::HeapStatistics` layout.
//!
//!   cargo run --release --example spike_item6_data_plane

mod data_plane;
mod dom;
mod log;
mod oracle;
mod projection;

#[path = "../spike/common.rs"]
mod common;
use common::{cppgc_stats, init_v8};
use v8_cppgc_ext::{CppHeapExt, DetailLevel};
use dom::{Document, NodeRef};
use log::{DeclBlock, MutationLog};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::sync::Arc;
use std::time::Instant;

// ---- counting allocator: requested bytes and actual malloc_size bytes ----
unsafe extern "C" {
  fn malloc_size(p: *const std::ffi::c_void) -> usize;
}
struct Counting;
static REQ: AtomicUsize = AtomicUsize::new(0);
static ACT: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
  unsafe fn alloc(&self, l: Layout) -> *mut u8 {
    let p = unsafe { System.alloc(l) };
    REQ.fetch_add(l.size(), Relaxed);
    ACT.fetch_add(unsafe { malloc_size(p as _) }, Relaxed);
    p
  }
  unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
    REQ.fetch_sub(l.size(), Relaxed);
    ACT.fetch_sub(unsafe { malloc_size(p as _) }, Relaxed);
    unsafe { System.dealloc(p, l) }
  }
}
#[global_allocator]
static GA: Counting = Counting;
fn mem() -> (usize, usize) {
  (REQ.load(Relaxed), ACT.load(Relaxed))
}

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

const ATTR_NAMES: u32 = 12;
fn rand_str(r: &mut Rng, lo: usize, hi: usize) -> Arc<str> {
  let len = lo + r.below(hi - lo);
  let s: String = (0..len).map(|_| (b'a' + r.below(26) as u8) as char).collect();
  Arc::from(s)
}

/// One random DOM mutation, applied to the DOM and logged.
fn random_mutation(d: &mut Document, r: &mut Rng, values: &[Arc<str>]) {
  let n = r.below(d.handles.len()) as NodeRef;
  match r.below(100) {
    0..30 => d.set_attr(n, r.below(ATTR_NAMES as usize) as u32, values[r.below(values.len())].clone()),
    30..35 => d.remove_attr(n, r.below(ATTR_NAMES as usize) as u32),
    35..60 => d.toggle_class(n, r.below(300) as u32),
    60..75 => d.set_state(n, r.next() as u32 & 0xff),
    75..80 => {
      let s = (r.below(2) == 0).then(|| Arc::new(DeclBlock { _text: "color: red".into() }));
      d.set_inline_style(n, s)
    }
    80..86 => {
      if d.is_connected(n) {
        d.create_child(n, r.below(40) as u32);
      }
    }
    86..92 => {
      // remove a connected leaf (its slot stays until GC; no Destroy is logged here)
      if n != 0 && d.is_connected(n) && dom::DomNode::link(&d.node(n).first_child).is_none() {
        d.remove(n)
      }
    }
    _ => {
      // move a subtree under another connected node (remove + append)
      let p = r.below(d.handles.len()) as NodeRef;
      if n != 0 && n != p && d.is_connected(n) && d.is_connected(p) && !is_ancestor(d, n, p) {
        d.remove(n);
        d.append(p, n);
      }
    }
  }
}
fn is_ancestor(d: &Document, a: NodeRef, mut x: NodeRef) -> bool {
  loop {
    if x == a {
      return true;
    }
    match dom::DomNode::link(&d.node(x).parent) {
      Some(p) => x = p.id.idx,
      None => return false,
    }
  }
}

fn pct(mut v: Vec<u64>) -> String {
  v.sort_unstable();
  let n = v.len();
  let mean = v.iter().sum::<u64>() as f64 / n as f64;
  format!("mean={:.0}ns p50={}ns p99={}ns max={}ns (n={n})", mean, v[n / 2], v[(n - 1) * 99 / 100], v[n - 1])
}

fn run(heap: &v8::cppgc::Heap, nodes: usize) {
  println!("\n=================== DOM size {nodes} ===================");
  let mut r = Rng(0x2545F4914F6CDD1D ^ nodes as u64);
  let values: Vec<Arc<str>> = (0..1000).map(|_| rand_str(&mut r, 6, 30)).collect();
  let mut d = Document::new(heap);
  d.handles.reserve(nodes * 2);
  d.log.entries.reserve(nodes * 12);

  // ---------- build the DOM (logged) ----------
  let (cppgc0, (req0, act0)) = (cppgc_stats(heap).used, mem());
  let mut recent: Vec<NodeRef> = vec![0];
  for i in 1..nodes {
    let parent = recent[r.below(recent.len())];
    let c = d.create_child(parent, r.below(40) as u32);
    for _ in 0..(2 + r.below(5)) {
      // ~4 attributes per element, fresh strings (as a parser would allocate)
      d.set_attr(c, r.below(ATTR_NAMES as usize) as u32, rand_str(&mut r, 6, 30));
    }
    for _ in 0..r.below(5) {
      d.toggle_class(c, r.below(300) as u32);
    }
    if r.below(10) == 0 {
      d.set_state(c, 1 + r.below(15) as u32);
    }
    if r.below(20) == 0 {
      d.set_inline_style(c, Some(Arc::new(DeclBlock { _text: rand_str(&mut r, 40, 120).to_string() })));
    }
    recent.push(c);
    if recent.len() > 64 {
      recent.remove(r.below(recent.len()));
    }
    let _ = i;
  }
  let (cppgc1, (req1, act1)) = (cppgc_stats(heap).used, mem());
  let dom_cppgc = cppgc1 - cppgc0;
  {
    // kDetailed: per-type object statistics (new with v8-cppgc-ext).
    let det = heap.collect_statistics(DetailLevel::Detailed);
    // kBrief reports the allocation counter (includes the current linear
    // allocation buffer); kDetailed walks the pages after resetting LABs.
    let idx = det.type_names.iter().position(|n| n == "DomNode").expect("DomNode type");
    let (bytes, count) = det
      .space_stats
      .iter()
      .flat_map(|s| &s.page_stats)
      .filter_map(|p| p.object_statistics.get(idx))
      .fold((0, 0), |(b, c), e| (b + e.allocated_bytes, c + e.object_count));
    println!(
      "kDetailed: used={}B (kBrief said {}B) spaces={:?} DomNode objects on heap={count} ({:.1}B/object incl. header; heap may hold earlier runs' DOMs)",
      det.used_size_bytes,
      cppgc1,
      det.space_stats.iter().map(|s| (s.name.as_str(), s.used_size_bytes)).collect::<Vec<_>>(),
      bytes as f64 / count as f64
    );
  }
  let (dom_req, dom_act) = (req1 - req0, act1 - act0);
  let log_len = d.log.len();

  // ---------- initial flush builds the data plane ----------
  let mut dp = data_plane::DataPlane::default();
  let (req2, act2) = mem();
  let t = Instant::now();
  dp.flush(&mut d.log);
  let build_flush = t.elapsed();
  let (req3, act3) = mem();
  let (dp_req, dp_act, dp_owned) = (req3 - req2, act3 - act2, dp.owned_bytes());
  let n = nodes as f64;
  println!(
    "memory/node: cppgc DOM heap(used)={:.1}B  DOM Rust-side malloc={:.1}B req / {:.1}B actual  => DOM total={:.1}B",
    dom_cppgc as f64 / n,
    dom_req as f64 / n,
    dom_act as f64 / n,
    (dom_cppgc + dom_act) as f64 / n
  );
  println!(
    "memory/node: data plane={:.1}B owned (columns+boxes) / {:.1}B req / {:.1}B actual malloc",
    dp_owned as f64 / n,
    dp_req as f64 / n,
    dp_act as f64 / n
  );
  let total = dom_cppgc + dom_act + dp_act;
  println!(
    "data plane share of total node memory = {:.1}% (actual malloc; {:.1}% of DOM-only cppgc+malloc); vs cppgc heap alone {:.1}%",
    100.0 * dp_act as f64 / total as f64,
    100.0 * dp_act as f64 / (dom_cppgc + dom_act) as f64,
    100.0 * dp_act as f64 / dom_cppgc as f64
  );
  println!(
    "excluded: script-side handle table {}B/node; DomNode payload size_of={}B; attr strings + decl blocks are shared Arcs (counted on DOM side only)",
    std::mem::size_of::<v8::cppgc::UnsafePtr<dom::DomNode>>(),
    std::mem::size_of::<dom::DomNode>()
  );
  println!("initial flush: {log_len} logged mutations in {:?} ({:.1}ns/mutation)", build_flush, build_flush.as_nanos() as f64 / log_len as f64);
  let (mut tp, mut tc) = (vec![], vec![]);
  let mut res = Ok(());
  let mut conn = 0;
  for _ in 0..5 {
    let t = Instant::now();
    let proj = projection::project(d.root.get().unwrap());
    tp.push(t.elapsed().as_nanos() as u64);
    let t = Instant::now();
    res = oracle::check(&proj, &dp);
    tc.push(t.elapsed().as_nanos() as u64);
    conn = proj.len();
  }
  tp.sort();
  tc.sort();
  println!(
    "projection+equality on the full built tree ({conn} connected nodes): projection {:.3}ms + equality {:.3}ms = {:.3}ms (median of 5) oracle={res:?}",
    tp[2] as f64 / 1e6,
    tc[2] as f64 / 1e6,
    (tp[2] + tc[2]) as f64 / 1e6
  );
  dp.shrink_to_fit();
  let (req4, act4) = mem();
  println!(
    "data plane after shrink_to_fit: {:.1}B/node actual malloc ({:.1}B/node was Vec growth slack); share of total node memory {:.1}%",
    (act4 - act2) as f64 / n,
    (act3 as f64 - act4 as f64) / n,
    100.0 * (act4 - act2) as f64 / (dom_cppgc + dom_act + act4 - act2) as f64
  );
  let _ = req4;
  println!("data plane column breakdown (bytes/node): {}", dp.breakdown(nodes));

  // ---------- log append cost ----------
  // Record a realistic mix of mutations, then time appending clones of them
  // (clone = what the DOM call site builds: ids + Arc refcount bumps).
  for _ in 0..10_000 {
    random_mutation(&mut d, &mut r, &values);
  }
  let templates = std::mem::take(&mut d.log.entries);
  let mut log2 = MutationLog::default();
  let reps = 100;
  let t = Instant::now();
  for _ in 0..reps {
    for m in &templates {
      log2.append(m.clone());
    }
    log2.entries.clear(); // keeps capacity, as a steady-state log does
  }
  let per = t.elapsed().as_nanos() as f64 / (reps * templates.len()) as f64;
  println!("log append (steady-state capacity): {per:.1}ns/append, size_of::<Mutation>()={}B", std::mem::size_of::<log::Mutation>());
  // Apply the recorded templates so the data plane stays in sync.
  d.log.entries = templates;
  dp.flush(&mut d.log);

  // ---------- flush cost for N = 1, 100, 10k ----------
  for (batch, trials) in [(1usize, 5000usize), (100, 1000), (10_000, 20)] {
    let mut times = Vec::with_capacity(trials);
    for _ in 0..trials {
      for _ in 0..batch {
        random_mutation(&mut d, &mut r, &values);
      }
      let logged = d.log.len() as u64;
      let t = Instant::now();
      dp.flush(&mut d.log);
      times.push(t.elapsed().as_nanos() as u64);
      let _ = logged;
    }
    let proj = projection::project(d.root.get().unwrap());
    let ok = oracle::check(&proj, &dp);
    println!("flush N={batch:>5}: {}  | oracle after {trials} flushes: {ok:?}", pct(times));
  }

  // ---------- fresh projection + equality ----------
  let (mut tp, mut tc) = (vec![], vec![]);
  let mut connected = 0;
  for _ in 0..5 {
    let t = Instant::now();
    let proj = projection::project(d.root.get().unwrap());
    tp.push(t.elapsed().as_nanos() as u64);
    let t = Instant::now();
    oracle::check(&proj, &dp).expect("projection equality");
    tc.push(t.elapsed().as_nanos() as u64);
    connected = proj.len();
  }
  tp.sort();
  tc.sort();
  println!(
    "projection+equality after the flush runs: projection {:.3}ms + equality {:.3}ms = {:.3}ms ({connected} connected nodes, {} slots; median of 5)",
    tp[2] as f64 / 1e6,
    tc[2] as f64 / 1e6,
    (tp[2] + tc[2]) as f64 / 1e6,
    d.handles.len()
  );

  // ---------- negative control: a mutation that bypasses the log ----------
  let victim = (1..d.handles.len() as u32).find(|&i| d.is_connected(i)).unwrap();
  let before = d.log.len();
  d.set_state(victim, 0xdead);
  d.log.entries.truncate(before); // simulate a missing log append
  dp.flush(&mut d.log);
  let proj = projection::project(d.root.get().unwrap());
  println!("negative control (one unlogged mutation): oracle -> {:?}", oracle::check(&proj, &dp));
}

fn main() {
  let platform = init_v8("");
  // Detached CppHeap: no isolate, so no GC runs unless requested (none is).
  let heap = v8::cppgc::Heap::create(platform, v8::cppgc::HeapCreateParams::default());
  for n in [1_000, 10_000, 100_000] {
    run(&heap, n);
  }
}
