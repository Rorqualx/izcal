//! Item 5: does cppgc catch a deliberately omitted write barrier written from Rust?
//!
//! Setup (detached CppHeap, GC driven step-by-step with StandaloneTestingHeap,
//! precise stack so only real references count):
//!   A: rooted by a Persistent.         B: allocated before marking starts,
//!   referenced only by a raw pointer the GC cannot see.
//!   1. start incremental marking, run marking steps to completion -> A is black, B white
//!   2. store B into A.first_child
//!        "barrier": through Member::set (C++ operator=, Dijkstra barrier)
//!        "omitted": copy the compressed pointer bits straight into A's Member slot
//!   3. finalize the GC
//! Correct GC: B survives. Missed barrier: B is swept while A still points to it
//! (dangling Member), unless a heap verifier catches it first.
//!
//!   cargo run --release --bin item5_write_barrier                       # release V8
//!   V8_FORCE_DEBUG=true cargo run --target-dir target-v8debug --bin item5_write_barrier   # debug V8

use cppgc_spike::tree::{TNode, FINALIZED_LIVE_IDS};
use cppgc_spike::*;
use std::sync::atomic::Ordering::SeqCst;
use std::time::Duration;
use v8::cppgc::{Heap, HeapCreateParams, Member, Persistent};

fn scenario(omit_barrier: bool) {
  let platform = init_v8("");
  let heap = Heap::create(platform, HeapCreateParams::default());
  heap.enable_detached_garbage_collections_for_testing();
  let h = heap_handle(&heap);
  let mut th = TestingHeap::new(h);

  let a = Persistent::new(&unsafe { v8::cppgc::make_garbage_collected(&heap, TNode::new(1, 1)) });
  // B: only a Box-held raw pointer; invisible to the GC (heap memory, precise stack).
  let b = Box::new(unsafe { v8::cppgc::make_garbage_collected(&heap, TNode::new(2, 42)) });
  let b_raw = unsafe { b.as_ref().as_ref() } as *const TNode as usize;

  th.start();
  let mut steps = 1;
  while !th.step(true) {
    steps += 1;
  }
  println!("marking steps until done: {steps}; is_marking={}", is_marking(h));

  let a_ref = a.get().unwrap();
  if omit_barrier {
    // Initializing construction has no Dijkstra barrier; then raw-copy the
    // 4-byte compressed pointer into A's live slot, bypassing operator=.
    let tmp = std::mem::ManuallyDrop::new(Member::<TNode>::new(&*b));
    unsafe {
      std::ptr::copy_nonoverlapping(
        &*tmp as *const Member<TNode> as *const u8,
        a_ref.first_child.get() as *mut u8,
        std::mem::size_of::<Member<TNode>>(),
      )
    };
  } else {
    unsafe { a_ref.set_first_child(&*b) };
  }
  let via_a = unsafe { a_ref.first_child() }.map(|n| n as *const TNode as usize);
  println!("A.first_child == B: {} (Member size {} bytes)", via_a == Some(b_raw), std::mem::size_of::<Member<TNode>>());
  drop(b);

  th.finalize(true);
  unsafe { heap.collect_garbage_for_testing(v8::cppgc::EmbedderStackState::NoHeapPointers) };
  let b_finalized = FINALIZED_LIVE_IDS.load(SeqCst) > 0;
  println!(
    "after GC: B finalized={b_finalized} while A.first_child still points at it={}",
    unsafe { a_ref.first_child.get().as_ref() }.is_some()
  );
  if b_finalized {
    println!("OUTCOME MISSED: barrier omission not detected; A now holds a dangling Member (use-after-free)");
  } else {
    println!("OUTCOME B survived");
  }
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() >= 2 {
    scenario(args[1] == "omitted");
    return;
  }
  for m in ["barrier", "omitted"] {
    println!("\n===== {m} =====");
    let (o, _) = run_child(&[m], Duration::from_secs(60));
    println!("===== {m}: process outcome={o:?}");
  }
}
