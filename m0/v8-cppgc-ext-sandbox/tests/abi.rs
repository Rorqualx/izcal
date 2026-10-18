//! ABI cross-checks between the shim (compiled by this crate) and the
//! prebuilt librusty_v8.a. See README "ABI matching".

mod common;
use common::*;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use v8_cppgc_ext::abi;
use v8_cppgc_ext::*;

const CHILD_ENV: &str = "V8_CPPGC_EXT_BUILD_CONFIG_CHILD";

/// Runs in a child process (V8 must not be initialized yet): the shim's
/// header-inline `v8::V8::Initialize()` passes the shim's build configuration
/// to V8, which FATALs ("Embedder-vs-V8 build configuration mismatch") if
/// pointer compression / 31-bit Smis / sandbox / V8_ENABLE_CHECKS differ.
#[test]
fn build_config_child() {
  if std::env::var_os(CHILD_ENV).is_none() {
    return;
  }
  let platform = v8::new_default_platform(0, false).make_shared();
  v8::V8::initialize_platform(platform);
  let ok = unsafe { abi::initialize_v8_with_shim_build_config() };
  assert!(ok);
  println!("CHILD-OK shim build configuration accepted by V8: {:#x}", abi::shim_build_configuration());
}

#[test]
fn v8_accepts_shim_build_configuration() {
  let exe = std::env::current_exe().unwrap();
  let out = std::process::Command::new(exe)
    .args(["--exact", "build_config_child", "--nocapture", "--test-threads=1"])
    .env(CHILD_ENV, "1")
    .output()
    .unwrap();
  let stdout = String::from_utf8_lossy(&out.stdout);
  let stderr = String::from_utf8_lossy(&out.stderr);
  assert!(
    out.status.success() && stdout.contains("CHILD-OK"),
    "V8 rejected the shim's build configuration\nstatus={:?}\nstdout={stdout}\nstderr={stderr}",
    out.status
  );
}

#[test]
fn shim_configuration_matches_expectations() {
  let c = abi::shim_build_configuration();
  println!("shim defines: {}", abi::SHIM_DEFINES);
  println!("shim compiler: {}", abi::SHIM_CXX);
  println!("shim build configuration bits: {c:#x} (debug V8: {})", abi::DEBUG_V8);
  // rusty_v8 default features: no V8 pointer compression, no sandbox.
  let ptrcomp = cfg!(feature = "v8_enable_pointer_compression");
  assert_eq!(c & abi::BIT_POINTER_COMPRESSION != 0, ptrcomp);
  assert_eq!(c & abi::BIT_SANDBOX != 0, cfg!(feature = "v8_enable_sandbox"));
  assert_eq!(c & abi::BIT_V8_ENABLE_CHECKS, 0);
  // cppgc caged heap + compression on 64-bit.
  assert_ne!(c & abi::BIT_CPPGC_CAGED_HEAP, 0);
  assert_ne!(c & abi::BIT_CPPGC_POINTER_COMPRESSION, 0);
  assert_ne!(c & abi::BIT_CPPGC_YOUNG_GENERATION, 0);
  assert_eq!(abi::sizeof_member(), std::mem::size_of::<v8::cppgc::Member<Node>>());
}

/// Member compression is header-inline: the shim's compression must agree
/// with the prebuilt's, in both directions, for objects all over the cage.
#[test]
fn member_compression_roundtrips_with_prebuilt() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  let (p, _g) = init();
  let heap = detached_heap(p);
  let mut keep = vec![];
  let mut member = v8::cppgc::Member::<Node>::empty();
  for i in 0..20_000u32 {
    let n = node(&heap, i, &DROPS);
    if i % 97 == 0 {
      // spread objects over normal and large pages
      let _big = unsafe { v8::cppgc::make_garbage_collected(&heap, Big([0u64; 4096])) };
    }
    assert!(abi::member_roundtrip_shim_to_prebuilt(&n), "shim->prebuilt mismatch at {i}");
    assert!(abi::member_roundtrip_prebuilt_to_shim(&n), "prebuilt->shim mismatch at {i}");
    assert!(abi::member_assign_with_shim(&mut member, &n), "shim operator= mismatch at {i}");
    if i % 1000 == 0 {
      keep.push(v8::cppgc::Persistent::new(&n));
    }
  }
  assert_eq!(keep[3].get().unwrap().id, 3000);
}

struct Big(#[allow(dead_code)] [u64; 4096]);
unsafe impl v8::cppgc::GarbageCollected for Big {
  fn trace(&self, _: &mut v8::cppgc::Visitor) {}
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"Big"
  }
}

/// CollectStatistics returns std::vector / std::string built by the prebuilt
/// and destroyed by the shim: if the libc++ ABI differed, the names, counts
/// and per-type tallies below would be garbage (or crash).
#[test]
fn statistics_cross_the_libcxx_boundary_intact() {
  static DROPS: AtomicUsize = AtomicUsize::new(0);
  let (p, _g) = init();
  let heap = detached_heap(p);
  let keep: Vec<_> = (0..777).map(|i| v8::cppgc::Persistent::new(&node(&heap, i, &DROPS))).collect();
  precise_gc(&heap);
  let s = heap.collect_statistics(DetailLevel::Detailed);
  let names: Vec<_> = s.space_stats.iter().map(|x| x.name.clone()).collect();
  assert_eq!(names, ["NormalPageSpace0", "NormalPageSpace1", "NormalPageSpace2", "NormalPageSpace3", "LargePageSpace"]);
  let idx = s.type_names.iter().position(|n| n == "V8CppgcExtTestNode").expect("type name");
  let count: usize = s
    .space_stats
    .iter()
    .flat_map(|sp| &sp.page_stats)
    .filter_map(|pg| pg.object_statistics.get(idx))
    .map(|e| e.object_count)
    .sum();
  assert_eq!(count, 777);
  drop(keep);
  assert_eq!(DROPS.load(SeqCst), 0);
}
