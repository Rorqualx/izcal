//! HGBA M0 V8 determinism probe (Stage 1 design doc, Section 3.3).
//!
//! Usage:
//!   determinism-probe <variant> <runs>            # runs in one process, fresh isolate each
//!   determinism-probe spawn <variant> <procs>     # <procs> fresh processes, 1 run each
//!
//! Variants:
//!   pred          predictable platform (virtual clock, queued worker tasks)
//!                 + --predictable --single-threaded --random-seed=42
//!   pred-inline   same, worker tasks run inline
//!   pred-noflags  predictable platform (virtual clock, queued worker tasks)
//!                 + --random-seed=42 only (no --predictable/--single-threaded)
//!   default-flags default platform (worker threads, real clock)
//!                 + --predictable --single-threaded --random-seed=42
//!   default       default platform (worker threads, real clock) + --random-seed=42
//!
//! All variants add --expose-gc (needed for the embedder's forced GCs).

use std::process::Command;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Instant;

const SEED: i32 = 42;
const STEPS: u32 = 60;
const T0_US: u64 = 1_700_000_000_000_000; // 2023-11-14T22:13:20Z, virtual
const STEP_US: u64 = 16_000;
static NOW_US: AtomicU64 = AtomicU64::new(T0_US);
static WORKER_TASKS: AtomicU64 = AtomicU64::new(0);

const PROBE_JS: &str = include_str!("probe.js");

fn fnv1a(s: &str) -> u64 {
  let mut h: u64 = 0xcbf29ce484222325;
  for b in s.as_bytes() {
    h ^= *b as u64;
    h = h.wrapping_mul(0x100000001b3);
  }
  h
}

struct Outcome {
  total: u64,
  r: u64,
  d: u64,
  g: u64,
  fin: String,
}

fn call(scope: &mut v8::PinScope, name: &str, arg: Option<u32>) -> String {
  let global = scope.get_current_context().global(scope);
  let key = v8::String::new(scope, name).unwrap();
  let f = global.get(scope, key.into()).unwrap();
  let f = v8::Local::<v8::Function>::try_from(f).unwrap();
  let recv = v8::undefined(scope).into();
  let args: Vec<v8::Local<v8::Value>> = match arg {
    Some(a) => vec![v8::Integer::new_from_unsigned(scope, a).into()],
    None => vec![],
  };
  let r = f.call(scope, recv, &args).expect("JS threw");
  r.to_rust_string_lossy(scope)
}

fn drain(platform: &v8::SharedRef<v8::Platform>, scope: &mut v8::PinScope) {
  loop {
    let mut ran = false;
    while v8::Platform::pump_message_loop(platform, scope, false) {
      ran = true;
    }
    if let Some(n) = v8::Platform::run_worker_tasks(platform) {
      WORKER_TASKS.fetch_add(n as u64, Ordering::Relaxed);
      ran |= n > 0;
    }
    scope.perform_microtask_checkpoint();
    if !ran {
      break;
    }
  }
}

fn run_once(platform: &v8::SharedRef<v8::Platform>) -> Outcome {
  NOW_US.store(T0_US, Ordering::SeqCst);
  let isolate = &mut v8::Isolate::new(Default::default());
  isolate.set_microtasks_policy(v8::MicrotasksPolicy::Explicit);
  v8::scope!(let scope, isolate);
  let context = v8::Context::new(scope, Default::default());
  let scope = &mut v8::ContextScope::new(scope, context);
  let src = v8::String::new(scope, PROBE_JS).unwrap();
  let script = v8::Script::compile(scope, src, None).unwrap();
  script.run(scope).unwrap();
  for i in 0..STEPS {
    call(scope, "step", Some(i));
    NOW_US.fetch_add(STEP_US, Ordering::SeqCst);
    // End of the "task": run microtasks (also clears WeakRef keep-alive
    // sets), then an embedder-forced GC point, then deliver its consequences
    // (FinalizationRegistry cleanup tasks).
    drain(platform, scope);
    if i % 10 == 9 {
      scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
      drain(platform, scope);
    }
  }
  scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
  drain(platform, scope);
  let log = call(scope, "finish", None);
  // Split channels out of the JSON without a JSON dependency: re-evaluate.
  let chan = |scope: &mut v8::PinScope, c: &str| -> String {
    let code = format!("JSON.stringify(LOG.{c})");
    let s = v8::String::new(scope, &code).unwrap();
    let sc = v8::Script::compile(scope, s, None).unwrap();
    sc.run(scope).unwrap().to_rust_string_lossy(scope)
  };
  let r = chan(scope, "R");
  let d = chan(scope, "D");
  let g = chan(scope, "G");
  if std::env::var_os("PROBE_DUMP").is_some() {
    eprintln!("R={}\nD={}\nG={}", &r[..r.len().min(200)], &d[..d.len().min(200)], &g[..g.len().min(4000)]);
  }
  let fin = {
    let s = v8::String::new(scope, "String(finalized.length)").unwrap();
    let sc = v8::Script::compile(scope, s, None).unwrap();
    sc.run(scope).unwrap().to_rust_string_lossy(scope)
  };
  Outcome { total: fnv1a(&log), r: fnv1a(&r), d: fnv1a(&d), g: fnv1a(&g), fin }
}

fn init(variant: &str) -> v8::SharedRef<v8::Platform> {
  let clock = || NOW_US.load(Ordering::SeqCst) as f64 / 1e6;
  let platform = match variant {
    "pred" | "pred-inline" => {
      v8::V8::set_predictable_flags(SEED);
      let mode = if variant == "pred" { v8::WorkerTaskMode::Queued } else { v8::WorkerTaskMode::Inline };
      v8::new_predictable_platform(Some(Box::new(clock)), mode, 1)
    }
    "pred-noflags" => {
      v8::V8::set_flags_from_string(&format!("--random-seed={SEED}"));
      v8::new_predictable_platform(Some(Box::new(clock)), v8::WorkerTaskMode::Queued, 4)
    }
    "default-flags" => {
      v8::V8::set_predictable_flags(SEED);
      v8::new_default_platform(0, false)
    }
    "default" => {
      v8::V8::set_flags_from_string(&format!("--random-seed={SEED}"));
      v8::new_default_platform(0, false)
    }
    v => panic!("unknown variant {v}"),
  }
  .make_shared();
  v8::V8::set_flags_from_string("--expose-gc");
  v8::V8::initialize_platform(platform.clone());
  v8::V8::initialize();
  platform
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args[1] == "spawn" {
    let variant = &args[2];
    let procs: usize = args[3].parse().unwrap();
    let exe = std::env::current_exe().unwrap();
    let t = Instant::now();
    let mut lines = vec![];
    for _ in 0..procs {
      let out = Command::new(&exe).args([variant.as_str(), "1", "--quiet"]).output().unwrap();
      assert!(out.status.success(), "child failed: {}", String::from_utf8_lossy(&out.stderr));
      let s = String::from_utf8(out.stdout).unwrap();
      lines.push(s.lines().find(|l| l.starts_with("run=")).unwrap().to_string());
    }
    summarize(&format!("{variant} (fresh process x{procs})"), &lines, t.elapsed().as_secs_f64());
    return;
  }
  let variant = &args[1];
  let runs: usize = args[2].parse().unwrap();
  let quiet = args.iter().any(|a| a == "--quiet");
  let platform = init(variant);
  eprintln!("V8 {}", v8::V8::get_version());
  let t = Instant::now();
  let mut lines = vec![];
  for k in 0..runs {
    let o = run_once(&platform);
    lines.push(format!(
      "run={k} total={:016x} R={:016x} D={:016x} G={:016x} finalized={}",
      o.total, o.r, o.d, o.g, o.fin
    ));
  }
  eprintln!("worker tasks drained by the embedder: {} total over {runs} runs", WORKER_TASKS.load(Ordering::Relaxed));
  if quiet {
    println!("{}", lines[0]);
  } else {
    summarize(&format!("{variant} (in-process x{runs})"), &lines, t.elapsed().as_secs_f64());
  }
  // PROBE_EXIT=dispose (default) | nodispose | noplatform: shutdown diagnostics only;
  // results are printed above and do not depend on this.
  match std::env::var("PROBE_EXIT").as_deref() {
    Ok("nodispose") => {}
    Ok("dropfirst") => {
      drop(platform);
      unsafe { v8::V8::dispose() };
      v8::V8::dispose_platform();
      return;
    }
    Ok("droplast") => {
      unsafe { v8::V8::dispose() };
      v8::V8::dispose_platform();
      drop(platform);
      eprintln!("platform ref dropped");
      return;
    }
    Ok("leak") => {
      unsafe { v8::V8::dispose() };
      v8::V8::dispose_platform();
      std::mem::forget(platform);
      return;
    }
    Ok("noplatform") => {
      unsafe { v8::V8::dispose() };
    }
    _ => {
      unsafe { v8::V8::dispose() };
      v8::V8::dispose_platform();
    }
  }
  if std::env::var_os("PROBE_EXIT_NOW").is_some() {
    use std::io::Write;
    std::io::stdout().flush().ok();
    std::process::exit(0);
  }
}

fn summarize(label: &str, lines: &[String], secs: f64) {
  use std::collections::BTreeMap;
  let field = |l: &str, k: &str| -> String {
    l.split(' ').find(|p| p.starts_with(&format!("{k}="))).unwrap()[k.len() + 1..].to_string()
  };
  println!("== {label}: {} runs in {secs:.1}s", lines.len());
  for k in ["total", "R", "D", "G", "finalized"] {
    let mut m: BTreeMap<String, usize> = BTreeMap::new();
    for l in lines {
      *m.entry(field(l, k)).or_default() += 1;
    }
    let top: Vec<String> = m.iter().take(5).map(|(v, c)| format!("{v}x{c}")).collect();
    println!("  {k:9} distinct={:4}  {}", m.len(), top.join(" "));
  }
  println!("  first: {}", lines[0]);
}
