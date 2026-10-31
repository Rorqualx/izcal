//! `js-probe`: JS-M0's decision-point probe (Stage 1 3.3, M1's exit; JS
//! spec 9.2, JS-M0 row). Reference-interpreter programs run over
//! `hg-heap` under the simulator: they allocate, create WeakRefs and a
//! FinalizationRegistry, draw `Math.random` from the substrate's seeded
//! entropy, read the simulated clock, and leave garbage, so canonical GC
//! cycles run at decision points and clear weak references (JS-D15,
//! R0-B11). The seeded scheduler orders the program's rounds.
//!
//! The last line is the `hg-dst` report line plus `state=`: a hash of the
//! heap's address-independent digest, every deref result, every cleanup
//! call and the cycle count. The trace hash covers the decision log,
//! since each GC completion is a trace line. `hg-dst repeat` checks that
//! 1,000 runs of a seed give one line.
//!
//! The probe's rounds run inside one simulated task, in the order the
//! scheduler decided: the JS-M0 runtime's agent lives in one heap scope,
//! so one job per task waits for JS-M1's event loop.
//!
//! ```text
//! js-probe --seed N [--rounds R] [--stress none|move|safepoint] [--trigger UNITS]
//! ```

use core::hash::Hasher as _;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use hg_heap::{Heap, HeapConfig, Stress};
use hg_js_bytecode::{Op, OperandKind, Unit, UnitView, assemble};
use hg_js_ops::support::{Host, HostValue};
use hg_js_runtime::{Agent, HostFunctions, JobError};
use hg_substrate::{EntropyPurpose, Priority, Substrate, Task};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{Limits, SimConfig, World};
use hg_types::{Duration, Instant, ProcessHasher};

/// The probe program: `setup` makes a target held by a WeakRef and
/// registered in a FinalizationRegistry, stamps the clock, and returns a
/// holder; `churn` makes a seeded amount of garbage and strings, then
/// derefs the WeakRef. `s?` placeholders become slots (see `slots`).
const PROGRAM: &str = r#"
.function setup params=0 registers=9
.const k0 key "weakRef"
.const k1 key "registry"
.const k2 key "t"
        LoadNull
        StoreReg r0
        CallIntrinsic %ObjectCreate, r0:1
        StoreReg r1                           ; r1 = target
        CallIntrinsic %WeakRefCreate, r1:1
        StoreReg r2                           ; r2 = WeakRef
        LoadInt #7
        StoreReg r3
        CallIntrinsic %CreateHostFunction, r3:1
        StoreReg r3                           ; r3 = cleanup callback 7
        CallIntrinsic %FinalizationRegistryCreate, r3:1
        StoreReg r3                           ; r3 = registry
        Copy r1, r4
        CallIntrinsic %MathRandom, r0:0
        StoreReg r5                           ; r5 = held value: a random Number
        LoadUndefined
        StoreReg r6
        CallIntrinsic %FinalizationRegistryRegister, r3:4
        CallIntrinsic %ObjectCreate, r0:1
        StoreReg r7                           ; r7 = holder
        LoadReg r2
        SetNamed r7, k0, s?
        LoadReg r3
        SetNamed r7, k1, s?
        CallIntrinsic %DateNow, r0:0
        SetNamed r7, k2, s?
        LoadReg r7
        Return
.end

.function churn params=1 registers=6
.const k0 key "weakRef"
.const k1 string "x"
        LoadNull
        StoreReg r0
        CallIntrinsic %MathRandom, r0:0
        StoreReg r2
        LoadInt #900
        Mul r2, s?
        StoreReg r2
        LoadInt #0
        BitOr r2, s?
        StoreReg r2                           ; r2 = floor(random * 900)
        LoadInt #0
        StoreReg r1                           ; i = 0
        LoadConst k1
        StoreReg r3                           ; r3 = "x"
loop:   CallIntrinsic %ObjectCreate, r0:1     ; garbage: dict units each
        LoadReg r1
        Add r3, s?                            ; "x" + i: string units
        StoreReg r4
        LoadReg r1
        Inc s?
        StoreReg r1
        LoadReg r2
        Lt r1, s?                             ; i < r2
        BranchFalse @done
        LoopBack @loop, s?
done:   GetNamed a0, k0, s?
        StoreReg r5
        CallIntrinsic %WeakRefDeref, r5:1
        Return
.end
"#;

/// Numbers each `s?` placeholder as a new slot of the kind its
/// instruction's operand needs.
fn slots(text: &str) -> String {
    let mut out = String::new();
    let mut body = String::new();
    let mut kinds: Vec<&'static str> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with(".function") {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if t == ".end" {
            for (i, k) in kinds.iter().enumerate() {
                out.push_str(&format!(".slot s{i} {k}\n"));
            }
            out.push_str(&body);
            out.push_str(".end\n");
            body.clear();
            kinds.clear();
            continue;
        }
        let mut l = line.to_owned();
        if l.contains("s?") {
            let code = t.split(';').next().unwrap_or("");
            let mut toks = code.split_whitespace();
            let first = toks.next().unwrap_or("");
            let mnemonic = if first.ends_with(':') {
                toks.next().unwrap_or("")
            } else {
                first
            };
            let op = Op::from_mnemonic(mnemonic).expect("a known mnemonic");
            for k in op.info().operands {
                if let OperandKind::Slot(sk) = k {
                    l = l.replacen("s?", &format!("s{}", kinds.len()), 1);
                    kinds.push(sk.name());
                }
            }
        }
        body.push_str(&l);
        body.push('\n');
    }
    out
}

/// The host: the simulated wall clock, and the cleanup jobs to run.
struct ProbeHost {
    s: Substrate,
    callbacks: i64,
    cleanup: Vec<u64>,
}

impl<'h> Host<'h> for ProbeHost {
    fn make_job_callback(&mut self) -> i64 {
        self.callbacks += 1;
        self.callbacks
    }
    fn system_utc_epoch_nanoseconds(&mut self) -> i128 {
        i128::from(self.s.clock.wall().as_unix_nanos())
    }
    fn enqueue_cleanup_job(&mut self, job: u64) {
        self.cleanup.push(job);
    }
}

/// One host-function call: its id and arguments.
type Call = (i32, Vec<HostValue>);

/// Host functions: every call is logged.
#[derive(Clone, Default)]
struct Calls(Rc<RefCell<Vec<Call>>>);

impl HostFunctions for Calls {
    fn call(
        &mut self,
        id: i32,
        _this: &HostValue,
        args: &[HostValue],
    ) -> Result<HostValue, HostValue> {
        self.0.borrow_mut().push((id, args.to_vec()));
        Ok(HostValue::Undefined)
    }
}

/// Feeds one state item to the hash; with `JS_PROBE_EXPLAIN` set, also
/// prints it, so a reader can see what the hash covers.
fn feed(h: &mut ProcessHasher, item: &str) {
    if std::env::var_os("JS_PROBE_EXPLAIN").is_some() {
        eprintln!("js-probe: {item}");
    }
    h.write(item.as_bytes());
}

fn write_value(h: &mut ProcessHasher, v: &Result<HostValue, JobError>) {
    feed(h, &format!("deref {v:?}"));
}

/// Runs the rounds in `order` and returns the state hash.
fn run_rounds(s: &Substrate, units: &[Unit], order: &[u64], stress: Stress, trigger: u64) -> u64 {
    let mut heap = Heap::new(
        Arc::clone(&s.mem),
        Arc::clone(&s.decisions),
        HeapConfig {
            cage_bytes: 256 << 20,
            trigger: u64::MAX,
            stress,
            agent: 1,
        },
    )
    .expect("a cage");
    let calls = Calls::default();
    let log = Rc::clone(&calls.0);
    let mut host = ProbeHost {
        s: s.clone(),
        callbacks: 0,
        cleanup: Vec::new(),
    };
    let images: Vec<Vec<u8>> = units.iter().map(Unit::to_image).collect();
    let mut state = ProcessHasher::with_keys(0x4A53_5052_4F42_0001, 0x4A53_5052_4F42_0002);
    let (digest, cycles) = heap.enter(|m| {
        m.scope(|m, sc| {
            let mut agent =
                Agent::new(m, sc, s.entropy.as_ref(), Box::new(calls), trigger).expect("an agent");
            let setup = UnitView::parse(&images[0]).expect("setup unit");
            let churn = UnitView::parse(&images[1]).expect("churn unit");
            for &round in order {
                feed(&mut state, &format!("round {round}"));
                let holder = agent.run_job(m, &mut host, &setup, None, &[]);
                let Ok(holder) = holder else {
                    write_value(&mut state, &holder.map(|_| HostValue::Undefined));
                    continue;
                };
                // Odd rounds end their synchronous run first, so the
                // target is held only weakly when `churn` runs.
                if round % 2 == 1 {
                    agent.clear_kept_objects(m);
                }
                let deref = agent
                    .run_job(m, &mut host, &churn, None, &[holder])
                    .map(|h| {
                        let v = agent.host_value(m, h).unwrap_or(HostValue::Undefined);
                        // The handle would root the target for the
                        // agent's life; release it so a later cycle can
                        // clear the target and run its cleanup.
                        agent.release(m, h);
                        v
                    });
                write_value(&mut state, &deref);
                agent.release(m, holder);
                agent.clear_kept_objects(m);
                for job in std::mem::take(&mut host.cleanup) {
                    let r = agent.run_cleanup_job(m, &mut host, job);
                    feed(&mut state, &format!("cleanup {job} {r:?}"));
                }
            }
            (m.digest(), agent.pacer().cycles())
        })
    });
    for (id, args) in log.borrow().iter() {
        feed(&mut state, &format!("call {id} {args:?}"));
    }
    feed(&mut state, &format!("digest {digest:016x} cycles {cycles}"));
    state.finish()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut seed = None;
    let mut rounds = 6u64;
    let mut stress = Stress::None;
    let mut trigger = 4_000u64;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--seed" => seed = args.next().and_then(|v| v.parse().ok()),
            "--trigger" => {
                trigger = args.next().and_then(|v| v.parse().ok()).unwrap_or(trigger);
            }
            "--rounds" => rounds = args.next().and_then(|v| v.parse().ok()).unwrap_or(rounds),
            "--stress" => {
                stress = match args.next().as_deref() {
                    Some("move") => Stress::MoveOnAlloc,
                    Some("safepoint") => Stress::MoveAtSafepoint,
                    _ => Stress::None,
                };
            }
            _ => {
                eprintln!(
                    "usage: js-probe --seed N [--rounds R] [--stress none|move|safepoint] [--trigger UNITS]"
                );
                std::process::exit(2);
            }
        }
    }
    let Some(seed) = seed else {
        eprintln!(
            "usage: js-probe --seed N [--rounds R] [--stress none|move|safepoint] [--trigger UNITS]"
        );
        std::process::exit(2);
    };
    let units = assemble(&slots(PROGRAM)).expect("the probe program assembles");
    let mut config = SimConfig::new(seed);
    // JS_PROBE_TRACE prints the trace's lines, for comparing two runs.
    config.keep_trace_text = std::env::var_os("JS_PROBE_TRACE").is_some();
    let world = World::new(config);
    let state: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(None));
    let out = Arc::clone(&state);
    world.spawn("renderer", Arc::new(OsMemory::new()), false, move |s| {
        // The scheduler decides the rounds' order: each round is a task
        // posted with a seeded delay, and records itself when it runs.
        let order: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
        let seq = s.exec.sequence("rounds", Priority::UserVisible);
        let mut rng = s.entropy.stream(EntropyPurpose::Scheduling);
        for r in 0..rounds {
            let order = Arc::clone(&order);
            let delay = Duration::from_micros(rng.next_u64() % 10_000);
            let _ = s.exec.post(
                seq,
                Task::new(move || order.lock().expect("order").push(r)),
                Some(delay),
                Duration::from_micros(500),
            );
        }
        let s2 = s.clone();
        let units = units.clone();
        let out = Arc::clone(&out);
        let _ = s.exec.post(
            seq,
            Task::new(move || {
                let order = order.lock().expect("order").clone();
                let h = run_rounds(&s2, &units, &order, stress, trigger);
                *out.lock().expect("state") = Some(h);
            }),
            Some(Duration::from_millis(20)),
            Duration::from_micros(500),
        );
    });
    let _ = world.run(Limits {
        max_steps: 100_000,
        max_time: Instant::from_nanos(60_000_000_000),
    });
    let Some(h) = *state.lock().expect("state") else {
        eprintln!("js-probe: the rounds did not run");
        std::process::exit(1);
    };
    if let Some(text) = world.trace_text() {
        eprint!("{text}");
    }
    println!("{} state={h:016x}", world.summary().report_line());
}
