//! CppHeap::CollectStatistics (kBrief and kDetailed) and the
//! no-compactable-spaces check.

mod common;
use common::*;
use std::sync::atomic::AtomicUsize;
use v8::cppgc::{Heap, Persistent};
use v8_cppgc_ext::*;

fn check_consistent(s: &HeapStatistics) {
  assert!(s.used_size_bytes <= s.resident_size_bytes, "{s:?}");
  assert!(s.resident_size_bytes <= s.committed_size_bytes, "{s:?}");
  if s.detail_level == DetailLevel::Detailed {
    let used: usize = s.space_stats.iter().map(|x| x.used_size_bytes).sum();
    let committed: usize = s.space_stats.iter().map(|x| x.committed_size_bytes).sum();
    assert_eq!(used, s.used_size_bytes);
    assert_eq!(committed + s.pooled_memory_size_bytes, s.committed_size_bytes);
    for sp in &s.space_stats {
      let pu: usize = sp.page_stats.iter().map(|p| p.used_size_bytes).sum();
      assert_eq!(pu, sp.used_size_bytes, "{}", sp.name);
      for pg in &sp.page_stats {
        assert!(pg.object_statistics.len() <= s.type_names.len());
        let bytes: usize = pg.object_statistics.iter().map(|e| e.allocated_bytes).sum();
        assert!(bytes <= pg.used_size_bytes, "{} page: objects {bytes} > used {}", sp.name, pg.used_size_bytes);
      }
      let fl = &sp.free_list_stats;
      assert_eq!(fl.bucket_size.len(), fl.free_count.len());
      assert_eq!(fl.bucket_size.len(), fl.free_size.len());
    }
  }
}

#[test]
fn brief_and_detailed_statistics_detached() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  let (p, _g) = init();
  let heap = detached_heap(p);
  let empty = heap.collect_statistics(DetailLevel::Brief);
  let keep: Vec<_> = (0..10_000).map(|i| Persistent::new(&node(&heap, i, &DROPS))).collect();
  precise_gc(&heap);
  let brief = heap.collect_statistics(DetailLevel::Brief);
  let detailed = heap.collect_statistics(DetailLevel::Detailed);
  println!("empty: {empty:?}");
  println!("brief: {brief:?}");
  println!(
    "detailed: used={} spaces={:?} types={:?}",
    detailed.used_size_bytes,
    detailed.space_stats.iter().map(|s| (&s.name, s.used_size_bytes, s.page_stats.len())).collect::<Vec<_>>(),
    detailed.type_names
  );
  assert_eq!(brief.detail_level, DetailLevel::Brief);
  assert!(brief.space_stats.is_empty() && brief.type_names.is_empty());
  assert_eq!(detailed.detail_level, DetailLevel::Detailed);
  assert_eq!(brief.used_size_bytes, detailed.used_size_bytes);
  assert!(brief.used_size_bytes >= empty.used_size_bytes + 10_000 * std::mem::size_of::<Node>());
  check_consistent(&brief);
  check_consistent(&detailed);
  let per_node = (brief.used_size_bytes - empty.used_size_bytes) as f64 / 10_000.0;
  println!("cppgc bytes per Node: {per_node:.1} (Rust size_of::<Node>() = {})", std::mem::size_of::<Node>());
  drop(keep);
  precise_gc(&heap);
  let after = heap.collect_statistics(DetailLevel::Brief);
  assert!(after.used_size_bytes < brief.used_size_bytes);
}

#[test]
fn statistics_of_isolate_cpp_heap() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  let (_p, _g) = init();
  let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
  let heap: &Heap = unsafe { &*(isolate.get_cpp_heap().unwrap() as *const Heap) };
  let keep: Vec<_> = (0..1000).map(|i| Persistent::new(&node(heap, i, &DROPS))).collect();
  let s = heap.collect_statistics(DetailLevel::Detailed);
  check_consistent(&s);
  assert!(s.type_names.iter().any(|n| n == "V8CppgcExtTestNode"), "{:?}", s.type_names);
  heap.assert_no_compactable_spaces();
  drop(keep);
}

#[test]
fn no_compactable_spaces_on_rusty_v8_heaps() {
  let (p, _g) = init();
  let heap = detached_heap(p);
  heap.assert_no_compactable_spaces();
}
