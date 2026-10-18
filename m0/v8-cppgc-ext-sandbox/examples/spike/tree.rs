#![allow(dead_code)]
//! Toy cppgc "DOM" tree shared by items 1, 2 and 5: first-child / next-sibling
//! links as traced `Member`s, plus plain (untraced) payload fields.

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use v8::cppgc::{GarbageCollected, Member, UnsafePtr, Visitor};

pub const LIVE: u64 = 0x11FE_11FE_11FE_11FE;
pub const DEAD: u64 = 0xDEAD_DEAD_DEAD_DEAD;

pub static FINALIZED: AtomicUsize = AtomicUsize::new(0);
pub static FINALIZED_LIVE_IDS: AtomicUsize = AtomicUsize::new(0);

pub struct TNode {
  pub magic: UnsafeCell<u64>,
  pub tag: u32,
  pub id: u32,
  pub first_child: UnsafeCell<Member<TNode>>,
  pub next_sibling: UnsafeCell<Member<TNode>>,
  pub payload: [u64; 2],
}

unsafe impl GarbageCollected for TNode {
  fn trace(&self, v: &mut Visitor) {
    unsafe {
      v.trace(&*self.first_child.get());
      v.trace(&*self.next_sibling.get());
    }
  }
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"TNode"
  }
}

impl Drop for TNode {
  fn drop(&mut self) {
    FINALIZED.fetch_add(1, Ordering::SeqCst);
    // ids < 1<<31 are "live tree" nodes; garbage uses the high bit.
    if self.id & 0x8000_0000 == 0 {
      FINALIZED_LIVE_IDS.fetch_add(1, Ordering::SeqCst);
    }
    *self.magic.get_mut() = DEAD;
  }
}

impl TNode {
  pub fn new(tag: u32, id: u32) -> Self {
    TNode {
      magic: UnsafeCell::new(LIVE),
      tag,
      id,
      first_child: UnsafeCell::new(Member::empty()),
      next_sibling: UnsafeCell::new(Member::empty()),
      payload: [id as u64 * 3, id as u64 * 7],
    }
  }
  /// Barriered write (C++ Member::operator=). Main thread only.
  pub unsafe fn set_first_child(&self, c: &UnsafePtr<TNode>) {
    unsafe { (*self.first_child.get()).set(c) }
  }
  pub unsafe fn set_next_sibling(&self, c: &UnsafePtr<TNode>) {
    unsafe { (*self.next_sibling.get()).set(c) }
  }
  pub unsafe fn first_child(&self) -> Option<&TNode> {
    unsafe { (*self.first_child.get()).get() }
  }
  pub unsafe fn next_sibling(&self) -> Option<&TNode> {
    unsafe { (*self.next_sibling.get()).get() }
  }
}

/// Build a tree of `n` nodes with fan-out `fanout` (breadth-first ids).
/// Returns the root; all other nodes are reachable only through Members.
pub fn build_tree(heap: &v8::cppgc::Heap, n: u32, fanout: u32) -> UnsafePtr<TNode> {
  // The Vec below is invisible to the GC, so no GC may finalize while building.
  let _no_gc = crate::NoGcScope::enter(crate::heap_handle(heap));
  unsafe {
    let mut nodes: Vec<UnsafePtr<TNode>> = Vec::with_capacity(n as usize);
    for i in 0..n {
      nodes.push(v8::cppgc::make_garbage_collected(heap, TNode::new(i % 64, i)));
    }
    for i in 0..n {
      let first = i * fanout + 1;
      if first < n {
        nodes[i as usize].as_ref().set_first_child(&nodes[first as usize]);
        let last = (first + fanout).min(n);
        for c in first..last - 1 {
          nodes[c as usize].as_ref().set_next_sibling(&nodes[c as usize + 1]);
        }
      }
    }
    nodes.swap_remove(0)
  }
}

/// Allocate unreachable garbage nodes (ids with the high bit set).
pub fn alloc_garbage(heap: &v8::cppgc::Heap, n: u32) {
  for i in 0..n {
    unsafe {
      v8::cppgc::make_garbage_collected(heap, TNode::new(99, 0x8000_0000 | i));
    }
  }
}

/// Read-only traversal through raw pointers. Returns (count, checksum) or an
/// error string if a node fails its integrity check.
pub fn traverse(root: *const TNode) -> Result<(u64, u64), String> {
  let mut stack = vec![root];
  let (mut count, mut sum) = (0u64, 0u64);
  while let Some(p) = stack.pop() {
    let n = unsafe { &*p };
    let magic = unsafe { std::ptr::read_volatile(n.magic.get()) };
    if magic != LIVE {
      return Err(format!("node id={} magic={magic:#x} (finalized while reachable)", n.id));
    }
    debug_assert_eq!(n.payload[0], n.id as u64 * 3);
    if n.payload[1] != n.id as u64 * 7 {
      return Err(format!("node id={} payload corrupted", n.id));
    }
    count += 1;
    sum = sum.wrapping_add((n.id as u64) << 8 | n.tag as u64);
    unsafe {
      if let Some(c) = n.first_child() {
        stack.push(c);
      }
      if let Some(s) = n.next_sibling() {
        stack.push(s);
      }
    }
  }
  Ok((count, sum))
}

pub fn expected_checksum(n: u32) -> (u64, u64) {
  let mut sum = 0u64;
  for i in 0..n {
    sum = sum.wrapping_add((i as u64) << 8 | (i % 64) as u64);
  }
  (n as u64, sum)
}
