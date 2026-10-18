//! Builds `src/binding.cc`, the C++ shim, against rusty_v8 152.2.0's V8 headers
//! and Chromium libc++ (`std::__Cr`), with the same ABI-relevant defines that
//! rusty_v8's prebuilt static library was built with.
//!
//! The defines are derived from V8's `BUILD.gn` (`enabled_external_v8_defines`,
//! `enabled_external_cppgc_defines`) and Chromium's `build/config/c++` for the
//! gn args rusty_v8 uses (see `build_v8()` in rusty_v8's build.rs and its
//! `.gn`). They are checked three ways (see README "ABI matching"):
//!   1. static_asserts against the sizes baked into rusty_v8's own bindgen
//!      output (`gen/src_binding_*.rs`) for this target/profile;
//!   2. link-time probes of define-gated symbols in the prebuilt;
//!   3. runtime checks in the test-suite (V8's own build-configuration check,
//!      Member compression round trips, statistics sanity).
//!
//! Environment variables:
//!   V8_FORCE_DEBUG            same meaning as for rusty_v8 (debug prebuilt)
//!   V8_CPPGC_EXT_RUSTY_V8_DIR path to the unpacked v8-152.2.0 crate
//!   V8_CPPGC_EXT_CLANG_DIR    a clang install dir (with bin/clang++) able to
//!                             parse Chromium's libc++ (clang >= 21). Defaults
//!                             to CLANG_BASE_PATH, then a cached download of
//!                             the Chromium clang pinned by rusty_v8.
//!   RUSTY_V8_SRC_BINDING_PATH honoured like rusty_v8 does, for the size checks.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const V8_CRATE: &str = "v8-152.2.0";

fn main() {
  for v in [
    "V8_FORCE_DEBUG",
    "V8_CPPGC_EXT_RUSTY_V8_DIR",
    "V8_CPPGC_EXT_CLANG_DIR",
    "CLANG_BASE_PATH",
    "RUSTY_V8_SRC_BINDING_PATH",
    "GN_ARGS",
  ] {
    println!("cargo:rerun-if-env-changed={v}");
  }
  println!("cargo:rerun-if-changed=src/binding.cc");
  println!("cargo:rerun-if-changed=build.rs");

  let target = env::var("TARGET").unwrap();
  let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
  let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  if env::var_os("DOCS_RS").is_some() {
    return;
  }
  if target_os == "windows" {
    // rusty_v8's Windows prebuilt is clang-cl + Chromium libc++ with the MSVC
    // ABI; the flag set below has not been worked out or tested for it.
    panic!("v8-cppgc-ext: Windows is not supported yet (see README)");
  }

  let v8_dir = find_rusty_v8_dir();
  println!("v8-cppgc-ext: using rusty_v8 sources at {}", v8_dir.display());

  // Mirrors rusty_v8 build.rs `prebuilt_profile()`.
  let is_debug = target_os != "windows" && env_bool("V8_FORCE_DEBUG");
  let ptr_compr = env::var_os("CARGO_FEATURE_V8_ENABLE_POINTER_COMPRESSION").is_some();
  let sandbox = env::var_os("CARGO_FEATURE_V8_ENABLE_SANDBOX").is_some();
  if env::var("GN_ARGS").map(|a| !a.is_empty()).unwrap_or(false) {
    println!(
      "cargo:warning=v8-cppgc-ext: GN_ARGS is set (V8 built from source?). The shim's defines \
       assume rusty_v8's default gn args; verify with `cargo test --test abi`."
    );
  }

  let mut defines: Vec<(String, Option<String>)> = vec![];
  let mut def = |k: &str, v: Option<&str>| defines.push((k.to_string(), v.map(str::to_string)));

  // ---- V8 external defines (v8/BUILD.gn: enabled_external_v8_defines) ----
  // rusty_v8 .gn default_args + build.rs gn args.
  def("V8_ARRAY_BUFFER_INTERNAL_FIELD_COUNT", Some("2"));
  def("V8_ARRAY_BUFFER_VIEW_INTERNAL_FIELD_COUNT", Some("2"));
  def("V8_PROMISE_INTERNAL_FIELD_COUNT", Some("1"));
  def("V8_USE_DEFAULT_HASHER_SECRET", Some("false"));
  // v8_enable_v8_checks=false is passed explicitly by rusty_v8 -> no V8_ENABLE_CHECKS.
  if is_debug {
    // v8_enable_memory_accounting_checks = verification_features || dcheck_always_on
    def("V8_ENABLE_MEMORY_ACCOUNTING_CHECKS", None);
  }
  if ptr_compr || sandbox {
    def("V8_COMPRESS_POINTERS", None);
    def("V8_COMPRESS_POINTERS_IN_SHARED_CAGE", None);
    def("V8_31BIT_SMIS_ON_64BIT_ARCH", None);
  }
  if sandbox {
    def("V8_ENABLE_SANDBOX", None);
  }
  // v8_cppgc_microtask_queue defaults to true (gni/v8.gni).
  def("V8_CPPGC_MICROTASK_QUEUE", None);
  match target_os.as_str() {
    "android" => def("V8_TARGET_OS_ANDROID", None),
    "ios" => def("V8_TARGET_OS_IOS", None),
    "linux" => def("V8_TARGET_OS_LINUX", None),
    "macos" => def("V8_TARGET_OS_MACOS", None),
    "windows" => def("V8_TARGET_OS_WIN", None),
    other => panic!("v8-cppgc-ext: unsupported target_os {other}"),
  }
  def("V8_HAVE_TARGET_OS", None);
  if target_arch == "aarch64" {
    def("V8_TARGET_ARCH_ARM64", None);
  }
  if is_debug {
    // rusty_v8's BUILD.gn adds DEBUG for is_debug, and V8 compiles itself with
    // it (v8_enable_debugging_features); v8config.h / v8-source-location.h read it.
    def("DEBUG", None);
  }

  // ---- cppgc external defines (v8/BUILD.gn: enabled_external_cppgc_defines) ----
  let is_64 = matches!(target_arch.as_str(), "x86_64" | "aarch64" | "riscv64" | "loongarch64");
  if is_64 {
    // cppgc_enable_caged_heap => young generation + pointer compression.
    def("CPPGC_CAGED_HEAP", None);
    def("CPPGC_YOUNG_GENERATION", None);
    def("CPPGC_POINTER_COMPRESSION", None);
  }
  if is_debug {
    def("CPPGC_ENABLE_API_CHECKS", None); // cppgc_enable_api_checks = is_debug
  }
  if target_os != "ios" {
    def("CPPGC_ENABLE_LARGER_CAGE", None);
  }
  // cppgc_enable_object_names = !(component && win) && !android && !fuchsia
  if target_os != "android" {
    def("CPPGC_SUPPORTS_OBJECT_NAMES", None);
  }
  if target_os == "linux" || target_os == "macos" {
    def("CPPGC_ENABLE_OBJECT_SECTION_GCINFO", None);
  }
  def("CPPGC_SLIM_WRITE_BARRIER", None);
  if env::var_os("CARGO_FEATURE_RUSTY_V8_FORK").is_some() {
    // The fork's binding.cc already defines the UPSTREAM block's functions.
    def("V8CE_UPSTREAM_IN_RUSTY_V8", None);
  }

  // ---- Chromium libc++ configuration (build/config/c++/BUILD.gn,
  // build/config/compiler/BUILD.gn). ABI namespace/version come from
  // buildtools/third_party/libc++/__config_site (std::__Cr, ABI v2). ----
  {
    def("_LIBCPP_HARDENING_MODE", Some("_LIBCPP_HARDENING_MODE_EXTENSIVE"));
    def("_LIBCPP_DISABLE_VISIBILITY_ANNOTATIONS", None);
    def("_LIBCXXABI_DISABLE_VISIBILITY_ANNOTATIONS", None);
    def("_LIBCPP_INSTRUMENTED_WITH_ASAN", Some("0"));
  }

  // ---- size expectations from rusty_v8's bindgen output ----
  let expectations = binding_expectations(&v8_dir, &target, is_debug, ptr_compr, sandbox);
  let gen_header = out_dir.join("v8_cppgc_ext_expectations.h");
  fs::write(&gen_header, expectations).unwrap();

  // ---- compiler ----
  let clang_dir = find_clang(&v8_dir, &out_dir);
  let cxx = clang_dir.join("bin").join("clang++");
  assert!(cxx.exists(), "v8-cppgc-ext: compiler not found at {}", cxx.display());

  let mut b = cc::Build::new();
  b.cpp(true)
    .compiler(&cxx)
    .cpp_link_stdlib(None) // libc++ (std::__Cr) comes from librusty_v8.a
    .file("src/binding.cc")
    .warnings(true)
    .extra_warnings(false)
    .flag_if_supported("-Wno-unused-parameter")
    .flag_if_supported("-Wno-unknown-warning-option");
  {
    b.flag("-std=c++20")
      .flag("-fno-exceptions")
      .flag("-fno-rtti")
      .flag("-nostdinc++")
      .flag("-isystem")
      .flag(v8_dir.join("buildtools/third_party/libc++").to_str().unwrap())
      .flag("-isystem")
      .flag(v8_dir.join("third_party/libc++/src/include").to_str().unwrap())
      .flag("-isystem")
      .flag(v8_dir.join("third_party/libc++abi/src/include").to_str().unwrap());
  }
  if (target_os == "macos" || target_os == "ios")
    && let Ok(o) = Command::new("xcrun").arg("--show-sdk-path").output()
  {
    let sdk = String::from_utf8(o.stdout).unwrap();
    b.flag("-isysroot").flag(sdk.trim());
  }
  b.include(v8_dir.join("v8/include"))
    .include(v8_dir.join("src")) // support.h (RustObj)
    .include(&v8_dir)
    .include(&out_dir);
  for (k, v) in &defines {
    b.define(k, v.as_deref());
  }
  b.compile("v8_cppgc_ext");

  // Expose the configuration to the Rust side (cfg + runtime self-description).
  println!("cargo:rustc-check-cfg=cfg(v8_cppgc_ext_debug_v8)");
  if is_debug {
    println!("cargo:rustc-cfg=v8_cppgc_ext_debug_v8");
  }
  let summary = defines
    .iter()
    .map(|(k, v)| match v {
      Some(v) => format!("{k}={v}"),
      None => k.clone(),
    })
    .collect::<Vec<_>>()
    .join(" ");
  println!("cargo:rustc-env=V8_CPPGC_EXT_DEFINES={summary}");
  println!("cargo:rustc-env=V8_CPPGC_EXT_CXX={}", cxx.display());
}

fn env_bool(key: &str) -> bool {
  matches!(env::var(key).unwrap_or_default().as_str(), "true" | "1" | "yes")
}

fn cargo_home() -> PathBuf {
  if let Some(h) = env::var_os("CARGO_HOME") {
    return PathBuf::from(h);
  }
  PathBuf::from(env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")).expect("HOME")).join(".cargo")
}

/// Locates the unpacked `v8-152.2.0` crate (headers, libc++, gen/ bindings).
fn find_rusty_v8_dir() -> PathBuf {
  if let Some(d) = env::var_os("V8_CPPGC_EXT_RUSTY_V8_DIR") {
    return PathBuf::from(d);
  }
  // Vendored sources next to the workspace?
  let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
  for anc in manifest.ancestors() {
    let p = anc.join("vendor").join("v8");
    if p.join("v8/include/v8-cppgc.h").exists() {
      return p;
    }
  }
  let reg = cargo_home().join("registry").join("src");
  if let Ok(rd) = fs::read_dir(&reg) {
    for e in rd.flatten() {
      let p = e.path().join(V8_CRATE);
      if p.join("v8/include/v8-cppgc.h").exists() {
        return p;
      }
    }
  }
  panic!(
    "v8-cppgc-ext: could not find the {V8_CRATE} sources under {}; set V8_CPPGC_EXT_RUSTY_V8_DIR",
    reg.display()
  );
}

/// Chromium's clang revision pinned by rusty_v8 (tools/clang/scripts/update.py).
fn chromium_clang_package(v8_dir: &Path) -> String {
  let s = fs::read_to_string(v8_dir.join("tools/clang/scripts/update.py")).expect("update.py");
  let grab = |key: &str| -> String {
    let line = s.lines().find(|l| l.starts_with(key)).unwrap_or_else(|| panic!("{key} not in update.py"));
    line.split('=').nth(1).unwrap().trim().trim_matches('\'').trim_matches('"').to_string()
  };
  format!("{}-{}", grab("CLANG_REVISION ="), grab("CLANG_SUB_REVISION ="))
}

fn find_clang(v8_dir: &Path, out_dir: &Path) -> PathBuf {
  for var in ["V8_CPPGC_EXT_CLANG_DIR", "CLANG_BASE_PATH"] {
    if let Some(d) = env::var_os(var) {
      return PathBuf::from(d);
    }
  }
  let pkg = chromium_clang_package(v8_dir);
  let home = PathBuf::from(env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")).expect("HOME"));
  let cache = home.join(".cache").join("v8-cppgc-ext").join(format!("clang-{pkg}"));
  if cache.join("bin").exists() {
    return cache;
  }
  // Same package rusty_v8's build.rs would download (clang_download()).
  let host = match (env::consts::OS, env::consts::ARCH) {
    ("macos", "aarch64") => "Mac_arm64",
    ("macos", _) => "Mac",
    ("linux", _) => "Linux_x64",
    ("windows", _) => "Win",
    (os, arch) => panic!("v8-cppgc-ext: no Chromium clang for host {os}/{arch}; set V8_CPPGC_EXT_CLANG_DIR"),
  };
  let url = format!("https://commondatastorage.googleapis.com/chromium-browser-clang/{host}/clang-{pkg}.tar.xz");
  println!("cargo:warning=v8-cppgc-ext: downloading Chromium clang {pkg} (~47 MB) to {}", cache.display());
  fs::create_dir_all(&cache).unwrap();
  let tarball = out_dir.join("clang.tar.xz");
  let ok = Command::new("curl")
    .args(["-fsSL", "--retry", "3", "-o"])
    .arg(&tarball)
    .arg(&url)
    .status()
    .map(|s| s.success())
    .unwrap_or(false)
    && Command::new("tar").arg("xJf").arg(&tarball).arg("-C").arg(&cache).status().map(|s| s.success()).unwrap_or(false);
  let _ = fs::remove_file(&tarball);
  if !ok {
    let _ = fs::remove_dir_all(&cache);
    panic!(
      "v8-cppgc-ext: failed to download {url}. Chromium's libc++ headers need clang >= 21 \
       (Apple clang 17 cannot parse them). Set V8_CPPGC_EXT_CLANG_DIR to a suitable LLVM install."
    );
  }
  cache
}

/// Pulls size/alignment facts that rusty_v8's bindgen recorded (with the real
/// GN defines) out of `gen/src_binding_<features>_<profile>_<target>.rs` and
/// turns them into C++ static_asserts.
fn binding_expectations(v8_dir: &Path, target: &str, debug: bool, ptr_compr: bool, sandbox: bool) -> String {
  let mut features = String::new();
  if ptr_compr || sandbox {
    features.push_str("_ptrcomp");
  }
  if sandbox {
    features.push_str("_sandbox");
  }
  let profile = if debug { "debug" } else { "release" };
  let path = env::var_os("RUSTY_V8_SRC_BINDING_PATH")
    .map(PathBuf::from)
    .unwrap_or_else(|| v8_dir.join("gen").join(format!("src_binding{features}_{profile}_{target}.rs")));
  let mut out = String::from("// Generated by build.rs from rusty_v8's bindgen output.\n#pragma once\n");
  out += &format!("#define V8CE_BINDING_FILE \"{}\"\n", path.display());
  let Ok(src) = fs::read_to_string(&path) else {
    println!(
      "cargo:warning=v8-cppgc-ext: {} not found; skipping bindgen size cross-checks",
      path.display()
    );
    out += "#define V8CE_HAVE_EXPECTATIONS 0\n";
    return out;
  };
  out += "#define V8CE_HAVE_EXPECTATIONS 1\n";
  // `pub const NAME: usize = N;`
  for name in [
    "cppgc__Member_SIZE",
    "cppgc__WeakMember_SIZE",
    "v8__TracedReference_SIZE",
    "v8__Eternal_SIZE",
    "v8__String__ValueView_SIZE",
    "v8__ScriptOrigin_SIZE",
  ] {
    let needle = format!("pub const {name}: usize = ");
    if let Some(i) = src.find(&needle) {
      let rest = &src[i + needle.len()..];
      let n: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
      out += &format!("#define V8CE_EXPECT_{name} {n}\n");
    }
  }
  // bindgen layout tests: ["Size of X"][size_of::<X>() - Nusize]
  for (rust, key) in [
    ("RustObj", "RustObj"),
    ("v8_HeapStatistics", "v8_HeapStatistics"),
    ("v8_HeapSpaceStatistics", "v8_HeapSpaceStatistics"),
    ("v8_Object_Wrappable", "v8_Object_Wrappable"),
    ("cppgc_NameProvider", "cppgc_NameProvider"),
  ] {
    for (kind, prefix) in [("Size", "SIZE"), ("Alignment", "ALIGN")] {
      let needle = format!("\"{kind} of {rust}\"]");
      if let Some(i) = src.find(&needle) {
        let rest = &src[i..];
        if let Some(j) = rest.find(" - ") {
          let n: String = rest[j + 3..].chars().take_while(|c| c.is_ascii_digit()).collect();
          if !n.is_empty() {
            out += &format!("#define V8CE_EXPECT_{prefix}_{key} {n}\n");
          }
        }
      }
    }
  }
  out
}
