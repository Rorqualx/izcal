//! Izcal's side: each case assembled into a unit and run as a job by the
//! generated reference interpreter, on an ordinary heap and again under
//! the moving test collector.

use std::sync::Arc;

use hg_heap::{Heap, HeapConfig, Stress};
use hg_js_bytecode::{Op, OperandKind, UnitView, assemble};
use hg_js_ops::support::{Host, HostValue};
use hg_js_runtime::{Agent, HostFunctions, JobError};
use hg_substrate::{DecisionKind, DecisionLog, Position};
use hg_substrate_os::entropy::OsEntropy;
use hg_substrate_os::memory::OsMemory;

use crate::cases::{Case, Value};

/// Izcal's answer to one case, on each heap.
#[derive(Debug)]
pub struct Answer {
    /// On an ordinary heap.
    pub plain: String,
    /// Under the moving test collector.
    pub moving: String,
}

/// The result encoding both sides print: `u`, `z` (null), `t`, `f`,
/// `NaN`, `n` + the double's 16 hex digits, `s` + each code unit's 4 hex
/// digits, `o` for an object, `E` + an error's constructor name.
pub fn encode(v: &HostValue) -> String {
    match v {
        HostValue::Undefined => "u".into(),
        HostValue::Null => "z".into(),
        HostValue::Bool(true) => "t".into(),
        HostValue::Bool(false) => "f".into(),
        HostValue::Number(n) if n.is_nan() => "NaN".into(),
        HostValue::Number(n) => format!("n{:016x}", n.to_bits()),
        HostValue::String(s) => {
            let mut out = String::from("s");
            for u in s {
                out.push_str(&format!("{u:04x}"));
            }
            out
        }
        HostValue::Object => "o".into(),
    }
}

struct NoLog;
impl DecisionLog for NoLog {
    fn record(&self, _kind: DecisionKind, _at: Position, _detail: u64) {}
}

/// No host hook is reached by a value operation.
struct NoHost;
impl<'h> Host<'h> for NoHost {
    fn make_job_callback(&mut self) -> i64 {
        0
    }
    fn system_utc_epoch_nanoseconds(&mut self) -> i128 {
        0
    }
    fn enqueue_cleanup_job(&mut self, _job: u64) {}
}

struct NoFunctions;
impl HostFunctions for NoFunctions {
    fn call(
        &mut self,
        _id: i32,
        _this: &HostValue,
        _args: &[HostValue],
    ) -> Result<HostValue, HostValue> {
        Ok(HostValue::Undefined)
    }
}

/// Loads `v` into the accumulator; `k` numbers the constants (from k0,
/// without gaps, as the assembler requires).
fn load(v: &Value, k: &mut usize, consts: &mut String, body: &mut String) {
    match v {
        Value::Undefined => body.push_str("  LoadUndefined\n"),
        Value::Null => body.push_str("  LoadNull\n"),
        Value::Bool(true) => body.push_str("  LoadTrue\n"),
        Value::Bool(false) => body.push_str("  LoadFalse\n"),
        Value::Number(n) => {
            consts.push_str(&format!(
                ".const k{k} number {}\n",
                Value::number_literal(*n)
            ));
            body.push_str(&format!("  LoadConst k{k}\n"));
            *k += 1;
        }
        Value::String(s) => {
            consts.push_str(&format!(
                ".const k{k} string {}\n",
                Value::string_literal(s)
            ));
            body.push_str(&format!("  LoadConst k{k}\n"));
            *k += 1;
        }
        Value::Object => {
            body.push_str("  LoadNull\n  StoreReg r3\n  CallIntrinsic %ObjectCreate, r3:1\n")
        }
    }
}

/// The case as assembler text: the operand(s) loaded, the operation,
/// `Return`. Slots are declared in the order the operation names them.
fn unit_text(c: &Case) -> Result<String, String> {
    let op =
        Op::from_mnemonic(c.op.mnemonic).ok_or_else(|| format!("no opcode `{}`", c.op.mnemonic))?;
    let mut consts = String::new();
    let mut body = String::new();
    let mut k = 0;
    load(&c.a, &mut k, &mut consts, &mut body);
    if let Some(b) = &c.b {
        body.push_str("  StoreReg r0\n");
        load(b, &mut k, &mut consts, &mut body);
    }
    let mut operands = Vec::new();
    let mut slots = String::new();
    for k in op.info().operands {
        match k {
            OperandKind::Reg => operands.push("r0".to_owned()),
            OperandKind::Slot(sk) => {
                let i = slots.lines().count();
                slots.push_str(&format!(".slot s{i} {}\n", sk.name()));
                operands.push(format!("s{i}"));
            }
            other => return Err(format!("`{}` takes a {other:?} operand", c.op.mnemonic)),
        }
    }
    body.push_str(&format!(
        "  {} {}\n  Return\n",
        c.op.mnemonic,
        operands.join(", ")
    ));
    Ok(format!(
        ".function case params=0 registers=4\n{consts}{slots}{body}.end\n"
    ))
}

/// Runs every case on one heap with `stress`.
fn run_all(cases: &[Case], stress: Stress) -> Result<Vec<String>, String> {
    let mut images = Vec::with_capacity(cases.len());
    for c in cases {
        let text = unit_text(c)?;
        let units = assemble(&text).map_err(|e| format!("{e:?}\n{text}"))?;
        let unit = units.first().ok_or("an empty unit")?;
        images.push(unit.to_image());
    }
    let mut heap = Heap::new(
        Arc::new(OsMemory::new()),
        Arc::new(NoLog),
        HeapConfig {
            cage_bytes: 256 << 20,
            trigger: u64::MAX,
            stress,
            agent: 1,
        },
    )
    .map_err(|e| format!("a heap: {e:?}"))?;
    heap.enter(|m| {
        m.scope(|m, sc| {
            // `Math.random` is not under test; the OS seeds its stream.
            let entropy = OsEntropy::new();
            let mut agent = Agent::new(m, sc, &entropy, Box::new(NoFunctions), 1 << 16)
                .map_err(|e| format!("an agent: {e:?}"))?;
            let mut out = Vec::with_capacity(cases.len());
            let mut host = NoHost;
            for img in &images {
                let unit = UnitView::parse(img).map_err(|e| format!("{e:?}"))?;
                let r = match agent.run_job(m, &mut host, &unit, None, &[]) {
                    Ok(h) => {
                        let v = agent.host_value(m, h);
                        agent.release(m, h);
                        v.map_or_else(|| "X(no value)".to_owned(), |v| encode(&v))
                    }
                    Err(JobError::Error(k)) => format!("E{k:?}"),
                    Err(JobError::Thrown(h)) => {
                        let v = agent.host_value(m, h);
                        agent.release(m, h);
                        format!("T{v:?}")
                    }
                    Err(e) => format!("X{e:?}"),
                };
                out.push(r);
            }
            Ok(out)
        })
    })
}

/// Izcal's answers, on an ordinary heap and under the moving collector.
pub fn run(cases: &[Case]) -> Result<Vec<Answer>, String> {
    let plain = run_all(cases, Stress::None)?;
    let moving = run_all(cases, Stress::MoveOnAlloc)?;
    Ok(plain
        .into_iter()
        .zip(moving)
        .map(|(plain, moving)| Answer { plain, moving })
        .collect())
}
