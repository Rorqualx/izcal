//! Q-CS-5 probe: does a rusty_v8-linked binary run under Valgrind, and are
//! its instruction counts repeatable? `qcs5-probe <workload> [v8 flags...]`
//! (`--predictable` selects the fork's predictable platform) where workload is `init` (platform, isolate, context only), `script`
//! (a fixed JS workload), `script10` (it ten times) or `gc` (cppgc objects plus forced full GCs).

use std::sync::atomic::{AtomicUsize, Ordering};
use v8::cppgc::{GarbageCollected, Visitor};

static DROPS: AtomicUsize = AtomicUsize::new(0);

struct Payload {
    value: v8::TracedReference<v8::Value>,
}

// SAFETY: the only traced field is `value`, which `trace` visits.
unsafe impl GarbageCollected for Payload {
    fn trace(&self, visitor: &mut Visitor) {
        visitor.trace(&self.value);
    }
    fn get_name(&self) -> &'static std::ffi::CStr {
        c"QcsPayload"
    }
}

impl Drop for Payload {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

const SCRIPT: &str = r#"{
function fib(n) { return n < 2 ? n : fib(n - 1) + fib(n - 2); }
let acc = 0;
const objs = [];
for (let i = 0; i < 20000; i++) {
  objs.push({ i, s: "k" + (i % 97), a: [i, i * 2, i * 3] });
  acc = (acc + objs[i].a[2] + objs[i].s.length) | 0;
}
const m = new Map();
for (let i = 0; i < 5000; i++) m.set("key" + i, i);
for (let i = 0; i < 5000; i++) acc = (acc + m.get("key" + i)) | 0;
acc = (acc + fib(22)) | 0;
JSON.stringify({ acc, n: objs.length }).length + acc;
}"#;

fn run(scope: &mut v8::PinScope<'_, '_>, src: &str) -> String {
    let source = v8::String::new(scope, src).unwrap();
    let script = v8::Script::compile(scope, source, None).unwrap();
    script.run(scope).unwrap().to_rust_string_lossy(scope)
}

fn make(
    scope: &mut v8::PinScope<'_, '_>,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let payload = Payload {
        value: v8::TracedReference::new(scope, args.get(0)),
    };
    // SAFETY: the heap is this isolate's; the member is dropped at once, so
    // the object is garbage at the next full GC.
    let _member =
        unsafe { v8::cppgc::make_garbage_collected(scope.get_cpp_heap().unwrap(), payload) };
}

fn main() {
    let mut args = std::env::args().skip(1);
    let workload = args.next().unwrap_or_else(|| "init".into());
    let mut flags: Vec<String> = args.collect();
    if workload == "gc" {
        // RequestGarbageCollectionForTesting is fatal without it.
        flags.push("--expose-gc".into());
    }
    if !flags.is_empty() {
        v8::V8::set_flags_from_string(&flags.join(" "));
    }
    let predictable = flags.iter().any(|f| f == "--predictable");
    let single = flags.iter().any(|f| f == "--single-threaded");
    let platform = if predictable {
        // The fork's DST platform: synthetic clock, queued worker tasks.
        v8::V8::set_predictable_flags(1);
        v8::new_predictable_platform(None, v8::WorkerTaskMode::Queued, 0).make_shared()
    } else if single {
        v8::new_single_threaded_default_platform(false).make_shared()
    } else {
        v8::new_default_platform(0, false).make_shared()
    };
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
    {
        let hs = std::pin::pin!(v8::HandleScope::new(isolate));
        let hs = &mut hs.init();
        let context = v8::Context::new(hs, Default::default());
        let scope = &mut v8::ContextScope::new(hs, context);
        let out = match workload.as_str() {
            "init" => run(scope, "1 + 1"),
            "script" => run(scope, SCRIPT),
            "script10" => (0..10).map(|_| run(scope, SCRIPT)).last().unwrap(),
            "gc" => {
                let global = context.global(scope);
                let f = v8::Function::new(scope, make).unwrap();
                let name = v8::String::new(scope, "make").unwrap();
                global.set(scope, name.into(), f.into()).unwrap();
                run(scope, "for (let i = 0; i < 2000; i++) make({ i }); 0");
                for _ in 0..3 {
                    scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
                }
                format!("drops={}", DROPS.load(Ordering::SeqCst))
            }
            other => panic!("unknown workload {other}"),
        };
        println!("{workload} {out}");
    }
}
