//! The real reference interpreter, end to end: `hg_js_runtime::Agent`
//! running a job the assembler produced, with the unit image either in
//! the trusted region (a) or in an ordinary writable buffer re-validated
//! at entry (b). It gives the denominator the dispatch deltas are shares
//! of, and the one place the per-entry difference shows up in a whole
//! engine rather than in a loop.

use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use hg_heap::{Heap, HeapConfig, Mutator, Stress, TrustedRegion};
use hg_js_bytecode::{Unit, UnitView};
use hg_js_ops::support::{Host, HostValue};
use hg_js_runtime::{Agent, HostFunctions};
use hg_substrate::{
    CryptoRandom, DecisionKind, DecisionLog, Entropy, EntropyError, EntropyPurpose, Memory,
    Position, RandomStream,
};

/// A decision log that keeps nothing (the harness decides nothing).
struct NoLog;
impl DecisionLog for NoLog {
    fn record(&self, _kind: DecisionKind, _at: Position, _detail: u64) {}
}

/// SplitMix64, seeded: the realm's Random stream, fixed so a run is
/// reproducible.
struct SplitMix(u64);
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

struct FixedEntropy(NoCrypto);
impl Entropy for FixedEntropy {
    fn stream(&self, _purpose: EntropyPurpose) -> Box<dyn RandomStream> {
        Box::new(SplitMix(0x1234_5678))
    }
    fn crypto(&self) -> &dyn CryptoRandom {
        &self.0
    }
}

/// A host with a fixed clock and no jobs of its own.
#[derive(Default)]
struct FixedHost {
    callbacks: i64,
}
impl<'h> Host<'h> for FixedHost {
    fn make_job_callback(&mut self) -> i64 {
        self.callbacks += 1;
        self.callbacks
    }
    fn system_utc_epoch_nanoseconds(&mut self) -> i128 {
        1_700_000_000_000_000_000
    }
    fn enqueue_cleanup_job(&mut self, _job: u64) {}
}

/// No host functions: these programs call none.
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

/// Where the interpreter's unit image lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where {
    /// (a): sealed in the trusted region and opened by its id.
    Trusted,
    /// (b): in a writable buffer, parsed again at every entry.
    Caged,
}

/// What one end-to-end cell produced.
#[derive(Debug, Clone)]
pub struct Cell {
    /// Jobs run.
    pub jobs: u64,
    /// Nanoseconds for all of them.
    pub ns: u64,
    /// What the last job returned (the gate's check).
    pub value: Option<HostValue>,
}

/// Runs `unit` as `jobs` jobs and times them.
pub fn run(mem: &Arc<dyn Memory>, unit: &Unit, jobs: u64, place: Where) -> Option<Cell> {
    let image = unit.to_image();
    let mut region = TrustedRegion::new(Arc::clone(mem), 1 << 20).ok()?;
    let id = hg_js_bytecode::seal(&mut region, unit).ok()?;
    let mut heap = Heap::new(
        Arc::clone(mem),
        Arc::new(NoLog),
        HeapConfig {
            cage_bytes: 64 << 20,
            trigger: u64::MAX,
            stress: Stress::None,
            agent: 1,
        },
    )
    .ok()?;
    let out = heap.enter(|m: &mut Mutator<'_>| {
        m.scope(|m, s| {
            let mut agent = Agent::new(
                m,
                s,
                &FixedEntropy(NoCrypto),
                Box::new(NoFunctions),
                1 << 40,
            )
            .ok()?;
            let mut host = FixedHost::default();
            // One job before the clock starts: the first one pays for the
            // realm's and the constants' first touch.
            let warm = view(&region, id, &image, place)?;
            let h = agent.run_job(m, &mut host, &warm, None, &[]).ok()?;
            agent.release(m, h);
            let mut value = None;
            let t0 = Instant::now();
            for _ in 0..jobs {
                let v = view(&region, id, &image, place)?;
                let h = agent.run_job(m, &mut host, &v, None, &[]).ok()?;
                value = agent.host_value(m, h);
                agent.release(m, h);
            }
            let ns = t0.elapsed().as_nanos() as u64;
            Some(Cell {
                jobs,
                ns,
                value: black_box(value),
            })
        })
    });
    out
}

/// The view a job is given: (a) the trusted entry, (b) a parse of the
/// writable image.
fn view<'a>(
    region: &'a TrustedRegion,
    id: hg_heap::TrustedId,
    image: &'a [u8],
    place: Where,
) -> Option<UnitView<'a>> {
    match place {
        Where::Trusted => hg_js_bytecode::open(black_box(region), black_box(id)),
        Where::Caged => UnitView::parse(black_box(image)).ok(),
    }
}

/// One job, with whatever it ended as: the cross-check that the
/// harness's operand check and the reference interpreter's own
/// `Cx::admits` agree, on an intact image and on a corrupted one.
pub fn run_once(mem: &Arc<dyn Memory>, image: &[u8]) -> Result<HostValue, String> {
    let mut heap = Heap::new(
        Arc::clone(mem),
        Arc::new(NoLog),
        HeapConfig {
            cage_bytes: 16 << 20,
            trigger: u64::MAX,
            stress: Stress::None,
            agent: 1,
        },
    )
    .map_err(|e| format!("heap: {e:?}"))?;
    heap.enter(|m: &mut Mutator<'_>| {
        m.scope(|m, s| {
            let view = UnitView::parse(image).map_err(|e| format!("image: {e:?}"))?;
            let mut agent = Agent::new(
                m,
                s,
                &FixedEntropy(NoCrypto),
                Box::new(NoFunctions),
                1 << 40,
            )
            .map_err(|e| format!("agent: {e:?}"))?;
            let mut host = FixedHost::default();
            let h = agent
                .run_job(m, &mut host, &view, None, &[])
                .map_err(|e| format!("{e:?}"))?;
            agent.host_value(m, h).ok_or_else(|| "no value".to_string())
        })
    })
}
