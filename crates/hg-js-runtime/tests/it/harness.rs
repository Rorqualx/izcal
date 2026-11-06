//! The test host and the harness: a heap on test memory, an agent, a
//! recording host, and helpers that assemble and run programs.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use hg_heap::{Heap, HeapConfig, HeapStats, Mutator, Stress};
use hg_js_bytecode::{Op, OperandKind, UnitView, assemble};
use hg_js_ops::support::{ErrorKind, Host, HostValue};
use hg_js_runtime::{Agent, Handle, HostFunctions, JobError};
use hg_substrate::{
    CryptoRandom, DecisionKind, DecisionLog, Entropy, EntropyError, EntropyPurpose, Position,
    RandomStream,
};

/// A decision log that keeps nothing.
struct NoLog;
impl DecisionLog for NoLog {
    fn record(&self, _kind: DecisionKind, _at: Position, _detail: u64) {}
}

/// SplitMix64: a fixed, seeded stream (the test's `Entropy::stream`).
pub struct SplitMix(pub u64);
impl RandomStream for SplitMix {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for b in dest {
            *b = self.next_u64().to_le_bytes()[0];
        }
    }
}

struct NoCrypto;
impl CryptoRandom for NoCrypto {
    fn fill(&self, _dest: &mut [u8]) -> Result<(), EntropyError> {
        Err(EntropyError::Unavailable)
    }
}

/// Seeded entropy: the Random stream is SplitMix from `SEED`.
pub struct TestEntropy(NoCrypto);
pub const SEED: u64 = 0x1234_5678;
impl Entropy for TestEntropy {
    fn stream(&self, purpose: EntropyPurpose) -> Box<dyn RandomStream> {
        assert_eq!(purpose, EntropyPurpose::Random);
        Box::new(SplitMix(SEED))
    }
    fn crypto(&self) -> &dyn CryptoRandom {
        &self.0
    }
}

/// The host: a fixed clock, a counter for job callbacks, and the cleanup
/// jobs it was asked to enqueue.
#[derive(Debug, Default)]
pub struct TestHost {
    pub now_ns: i128,
    pub callbacks_made: i64,
    pub cleanup_jobs: Vec<u64>,
}

impl<'h> Host<'h> for TestHost {
    fn make_job_callback(&mut self) -> i64 {
        self.callbacks_made += 1;
        100 + self.callbacks_made
    }
    fn system_utc_epoch_nanoseconds(&mut self) -> i128 {
        self.now_ns
    }
    fn enqueue_cleanup_job(&mut self, job: u64) {
        self.cleanup_jobs.push(job);
    }
}

/// Host functions: each call is logged. Function 1 returns 42 (a
/// `valueOf`), 2 returns "obj" (a `toString`), 3 throws "boom", 4 returns
/// an object (so ToPrimitive goes on), others return undefined.
#[derive(Clone, Default)]
pub struct Calls(pub Rc<RefCell<Vec<Call>>>);

/// One logged call: the function and its arguments.
pub type Call = (i32, Vec<HostValue>);

impl HostFunctions for Calls {
    fn call(
        &mut self,
        id: i32,
        _this: &HostValue,
        args: &[HostValue],
    ) -> Result<HostValue, HostValue> {
        self.0.borrow_mut().push((id, args.to_vec()));
        match id {
            1 => Ok(HostValue::Number(42.0)),
            2 => Ok(HostValue::String("obj".encode_utf16().collect())),
            3 => Err(HostValue::String("boom".encode_utf16().collect())),
            4 => Ok(HostValue::Object),
            _ => Ok(HostValue::Undefined),
        }
    }
}

/// What a test sees after running: the host, the calls, the heap's
/// digest and counters, and the agent's cycle count.
pub struct After<T> {
    pub value: T,
    pub host: TestHost,
    pub calls: Vec<Call>,
    pub digest: u64,
    pub stats: HeapStats,
    pub cycles: u64,
}

/// Runs `f` with a fresh heap (trigger `u64::MAX`: cycles run at the
/// agent's decision points) and agent (`trigger` canonical units between
/// cycles).
pub fn with_agent<T>(
    stress: Stress,
    trigger: u64,
    f: impl for<'h, 's> FnOnce(&mut Mutator<'h>, &mut Agent<'h, 's>, &mut TestHost) -> T,
) -> After<T> {
    let mut heap = Heap::new(
        Arc::new(hg_heap::testing::TestMemory::default()),
        Arc::new(NoLog),
        HeapConfig {
            cage_bytes: 16 << 20,
            trigger: u64::MAX,
            stress,
            agent: 1,
        },
    )
    .unwrap();
    let calls = Calls::default();
    let log = Rc::clone(&calls.0);
    let mut host = TestHost {
        now_ns: 1_700_000_000_123_456_789,
        ..TestHost::default()
    };
    let (value, digest, cycles) = heap.enter(|m| {
        m.scope(|m, s| {
            let mut agent =
                Agent::new(m, s, &TestEntropy(NoCrypto), Box::new(calls), trigger).unwrap();
            let v = f(m, &mut agent, &mut host);
            let cycles = agent.pacer().cycles();
            (v, m.digest(), cycles)
        })
    });
    let calls = log.borrow().clone();
    After {
        value,
        host,
        calls,
        digest,
        stats: heap.stats(),
        cycles,
    }
}

/// Assembles `text`, numbering each `s?` placeholder as a new slot of
/// the kind its instruction's operand needs, so a test body never
/// declares slots by hand.
pub fn assemble_auto(text: &str) -> Vec<hg_js_bytecode::Unit> {
    let mut out = String::new();
    let mut slots = Vec::new();
    let mut body = String::new();
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with(".function") {
            out.push_str(&body);
            body.clear();
            slots.clear();
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if t == ".end" {
            for (i, k) in slots.iter().enumerate() {
                out.push_str(&format!(".slot s{i} {k}\n"));
            }
            out.push_str(&body);
            out.push_str(".end\n");
            body.clear();
            continue;
        }
        let mut l = line.to_string();
        if l.contains("s?") {
            let code = t.split(';').next().unwrap_or("");
            let mut toks = code.split_whitespace();
            let first = toks.next().unwrap_or("");
            let mnemonic = if first.ends_with(':') {
                toks.next().unwrap_or("")
            } else {
                first
            };
            let op = Op::from_mnemonic(mnemonic)
                .unwrap_or_else(|| panic!("no opcode `{mnemonic}` in `{line}`"));
            for k in op.info().operands {
                if let OperandKind::Slot(sk) = k {
                    l = l.replacen("s?", &format!("s{}", slots.len()), 1);
                    slots.push(sk.name());
                }
            }
        }
        body.push_str(&l);
        body.push('\n');
    }
    out.push_str(&body);
    assemble(&out).unwrap_or_else(|e| panic!("{e}\n{out}"))
}

/// The first unit of `text` run as a job with no arguments.
pub fn run_one<'h>(
    m: &mut Mutator<'h>,
    a: &mut Agent<'h, '_>,
    host: &mut TestHost,
    text: &str,
) -> Result<HostValue, Outcome> {
    let units = assemble_auto(text);
    let img = units[0].to_image();
    let unit = UnitView::parse(&img).unwrap();
    match a.run_job(m, host, &unit, None, &[]) {
        Ok(h) => Ok(a.host_value(m, h).unwrap()),
        Err(JobError::Thrown(h)) => Err(Outcome::Thrown(a.host_value(m, h).unwrap())),
        Err(JobError::Error(k)) => Err(Outcome::Error(k)),
        Err(e) => Err(Outcome::Other(e)),
    }
}

/// Runs unit `i` of `units` with `args`.
pub fn run_unit<'h>(
    m: &mut Mutator<'h>,
    a: &mut Agent<'h, '_>,
    host: &mut TestHost,
    unit: &hg_js_bytecode::Unit,
    args: &[Handle],
) -> Result<Handle, JobError> {
    let img = unit.to_image();
    let view = UnitView::parse(&img).unwrap();
    a.run_job(m, host, &view, None, args)
}

/// An abrupt end.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Thrown(HostValue),
    Error(ErrorKind),
    Other(JobError),
}

/// Evaluates a function body (`params=0 registers=8`, sloppy) with the
/// given constant lines; the body ends in `Return` by itself.
pub fn eval(consts: &str, body: &str) -> Result<HostValue, Outcome> {
    eval_with(Stress::None, consts, body, false)
}

/// `eval` in strict mode.
pub fn eval_strict(consts: &str, body: &str) -> Result<HostValue, Outcome> {
    eval_with(Stress::None, consts, body, true)
}

pub fn eval_with(
    stress: Stress,
    consts: &str,
    body: &str,
    strict: bool,
) -> Result<HostValue, Outcome> {
    let text = format!(
        ".function t params=0 registers=8{}\n{consts}\n{body}\n Return\n.end\n",
        if strict { " strict" } else { "" }
    );
    with_agent(stress, 1 << 20, |m, a, h| run_one(m, a, h, &text)).value
}

/// A Number result, compared by bits (so −0 and NaN are exact).
#[track_caller]
pub fn assert_num(r: Result<HostValue, Outcome>, want: f64) {
    match r {
        Ok(HostValue::Number(x)) => assert!(
            x.to_bits() == want.to_bits() || (x.is_nan() && want.is_nan()),
            "got {x:?} ({:#x}), want {want:?}",
            x.to_bits()
        ),
        other => panic!("got {other:?}, want the Number {want:?}"),
    }
}

/// A String result.
#[track_caller]
pub fn assert_str(r: Result<HostValue, Outcome>, want: &str) {
    let w: Vec<u16> = want.encode_utf16().collect();
    match r {
        Ok(HostValue::String(u)) => assert_eq!(
            u,
            w,
            "got {:?}, want {want:?}",
            String::from_utf16_lossy(&u)
        ),
        other => panic!("got {other:?}, want the String {want:?}"),
    }
}

/// A Boolean result.
#[track_caller]
pub fn assert_bool(r: Result<HostValue, Outcome>, want: bool) {
    assert_eq!(r, Ok(HostValue::Bool(want)));
}

/// A thrown error of kind `k`.
#[track_caller]
pub fn assert_error(r: Result<HostValue, Outcome>, k: ErrorKind) {
    assert_eq!(r, Err(Outcome::Error(k)));
}
