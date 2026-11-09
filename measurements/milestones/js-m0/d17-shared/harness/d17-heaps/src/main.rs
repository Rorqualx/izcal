//! The JS-D17 spike: what several agent heaps in one `ProcessCage` cost.
//!
//! JS-D17 chose (a) one cage per process, each agent heap in its own
//! blocks, with a later shared heap as further blocks in the same cage.
//! Its reversal trigger is "address-space or allocation contention
//! between agents measured on a reference device". This harness measures
//! the two halves of that against the alternative the row rejected,
//! (b) one cage per agent:
//!
//! * `gate` — the validity gate. Nothing below is worth reading unless
//!   this prints PASS on every line.
//! * `space` — address space as agents multiply: the substrate's ledger
//!   against the kernel's own view of the one reservation.
//! * `space-max` — how many agent heaps a process can actually carry.
//! * `reservations` — how many 16 GiB reservations one process can make,
//!   which is what (b) would need.
//! * `churn-maps` — what agents coming and going do to the mappings the
//!   one reservation has become.
//! * `contend` — several agent heaps allocating at once from one cage,
//!   against one cage each and against one cage and one memory each.
//! * `interleave` — what a heap's own pointer chase costs when its
//!   chunks are interleaved with other agents' in the shared cage.
//!
//! Every timing cell measures the clock itself and prints FAIL in place
//! of a measurement when a gate on that cell does not hold.

mod clock;
mod osview;
mod spy;
mod work;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Instant;

use hg_heap::{Heap, ProcessCage};
use hg_substrate::{Memory, MemoryKind};

use spy::SpyMemory;
use work::{CAGE_BYTES, Did};

/// Chunk bytes the cage hands out at a time (`hg-heap`'s `CHUNK`, not
/// public, so it is restated here and checked by the gate).
const CHUNK: u64 = 1 << 20;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("");
    let rest = &args[args.len().min(1)..];
    match cmd {
        "gate" => gate(),
        "space" => space(rest),
        "space-max" => space_max(rest),
        "reservations" => reservations(rest),
        "churn-maps" => churn_maps(rest),
        "maps" => maps(rest),
        "contend" => contend(rest),
        "interleave" => interleave(rest),
        _ => {
            eprintln!(
                "usage: d17-heaps <gate|space|space-max|reservations|churn-maps|maps|contend|interleave> [args]"
            );
            std::process::exit(2);
        }
    }
}

/// `--name value` from the argument list, or `default`.
fn opt<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> T {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == name {
            if let Some(v) = it.next()
                && let Ok(p) = v.parse::<T>()
            {
                return p;
            }
            return default;
        }
    }
    default
}

/// A comma-separated `--name 1,2,4` list, or `default`.
fn list(args: &[String], name: &str, default: &[u64]) -> Vec<u64> {
    let s: String = opt(args, name, String::new());
    if s.is_empty() {
        return default.to_vec();
    }
    s.split(',').filter_map(|p| p.trim().parse().ok()).collect()
}

/// The cage offset a reference slot decodes to, by the public formula of
/// JS spec 6.1: the address is `base + 4·v − 4`, so the offset is
/// `4·v − 4`. Every object of every agent heap decodes this way, with no
/// heap identity anywhere in it: that is what "one base for every
/// reference" means.
fn offset_of(slot: hg_heap::Slot) -> u64 {
    4 * u64::from(slot.raw()) - 4
}

/// The chunk an offset lies in.
fn chunk_of(off: u64) -> u64 {
    off / CHUNK
}

/// The chunks `n` cells of one agent heap landed in, and their offsets.
fn chunks_used(h: &mut Heap, n: u64) -> (Vec<u64>, Vec<u64>) {
    h.enter(|m| {
        m.scope(|m, s| {
            let mut offs = Vec::new();
            for _ in 0..n {
                if let Ok(c) = m.alloc::<work::Cell>(s, 0) {
                    offs.push(offset_of(c.get(m).slot()));
                }
            }
            let mut cs: Vec<u64> = offs.iter().map(|o| chunk_of(*o)).collect();
            cs.sort_unstable();
            cs.dedup();
            (cs, offs)
        })
    })
}

// ---------------------------------------------------------------- gate

/// The validity gate. A fast wrong number is this spike's main failure
/// mode, so every claim the legs below rest on is checked first, and
/// each check prints PASS or FAIL.
fn gate() {
    let mut all = true;
    let mut check = |name: &str, ok: bool, detail: String| {
        println!("{}\t{name}\t{detail}", if ok { "PASS" } else { "FAIL" });
        all &= ok;
    };

    // 1. One cage per process: four agent heaps, one reservation.
    let mem = Arc::new(SpyMemory::new());
    let cage = ProcessCage::new(mem.clone(), CAGE_BYTES).expect("16 GiB cage");
    let mut heaps: Vec<Heap> = (0..4)
        .map(|i| work::heap_in(&cage, work::no_gc(i)))
        .collect();
    // Grow them turn by turn, so a pool shared between them interleaves.
    let mut used: Vec<Vec<u64>> = Vec::new();
    for h in &mut heaps {
        let (cs, _) = chunks_used(h, 64);
        used.push(cs);
    }
    for h in &mut heaps {
        let (cs, _) = chunks_used(h, 200_000);
        used.push(cs);
    }
    let res = mem.reservations();
    check(
        "one-reservation",
        res.len() == 1 && res[0].len as u64 == CAGE_BYTES,
        format!(
            "{} reservation(s), {} bytes",
            res.len(),
            res.first().map(|r| r.len).unwrap_or(0)
        ),
    );

    // 2. One base: every object of every heap decodes to an offset inside
    //    the one cage, by the one formula, with no heap identity in it.
    let inside = used
        .iter()
        .flatten()
        .all(|c| c * CHUNK < CAGE_BYTES && *c > 0);
    check(
        "offsets-in-one-cage",
        inside,
        format!(
            "{} chunk(s) seen, all in [1, {})",
            used.iter().flatten().count(),
            CAGE_BYTES / CHUNK
        ),
    );

    // 3. Each agent owns its own chunks: no chunk is used by two heaps.
    let mut owner: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    let mut shared_chunk = None;
    for (i, cs) in used.iter().enumerate() {
        let agent = i % 4;
        for c in cs {
            match owner.get(c) {
                Some(&a) if a != agent => shared_chunk = Some((*c, a, agent)),
                _ => {
                    owner.insert(*c, agent);
                }
            }
        }
    }
    check(
        "chunks-disjoint",
        shared_chunk.is_none(),
        match shared_chunk {
            None => format!("{} chunks, each one agent's", owner.len()),
            Some((c, a, b)) => format!("chunk {c} used by agents {a} and {b}"),
        },
    );

    // 4. The agents really draw from one pool: their chunk runs
    //    interleave, which a cage per agent could not produce.
    let mut runs = 0u32;
    let mut sorted: Vec<(u64, usize)> = owner.iter().map(|(c, a)| (*c, *a)).collect();
    sorted.sort_unstable();
    let mut prev = None;
    for (_, a) in &sorted {
        if prev != Some(*a) {
            runs += 1;
        }
        prev = Some(*a);
    }
    check(
        "pool-interleaved",
        runs > 4,
        format!("{runs} runs of ownership across the cage's chunks (>4 means one pool)"),
    );

    // 5. The chunk size this harness assumes is the cage's own.
    let mut h = work::heap_in(&cage, work::no_gc(99));
    let (cs, _) = chunks_used(&mut h, 100_000);
    let contiguous = cs.windows(2).all(|w| w[1] == w[0] + 1);
    check(
        "chunk-size",
        cs.len() > 1 && contiguous,
        format!(
            "{} chunks of {} bytes, contiguous: {contiguous}",
            cs.len(),
            CHUNK
        ),
    );
    drop(h);
    drop(heaps);
    drop(cage);

    // 6. The work is not elided: the checksum is the one the arithmetic
    //    must give.
    let mem = Arc::new(SpyMemory::new());
    let cage = ProcessCage::new(mem, CAGE_BYTES).expect("cage");
    let mut h = work::heap_in(&cage, work::no_gc(1));
    let n = 1000u64;
    let did = work::grow(&mut h, n);
    let want: u64 = (0..n).map(|i| i & 0xFFFF).sum();
    check(
        "work-checked",
        did.checksum == want && did.allocs == n + 1 && did.refused == 0,
        format!(
            "checksum {} (want {want}), {} allocs, {} refused",
            did.checksum, did.allocs, did.refused
        ),
    );
    drop(h);
    drop(cage);

    // 7. The clock's two readings agree, so cycle figures mean something.
    let c = clock::measure(300_000_000);
    check(
        "clock",
        c.sound(),
        format!(
            "OS max {:.0} MHz, mean {:.0} MHz; reference {:.3} ns/add = {:.2} cyc/add; checked {}; cycles available: {}",
            c.mhz_os_max,
            c.mhz_os_mean,
            c.ref_ns_per_add,
            c.ref_cycles_per_add,
            c.checked,
            c.has_cycles()
        ),
    );

    // 8. The OS's own view is readable, or the space leg can only report
    //    the substrate's ledger.
    let v = osview::read(None);
    check(
        "os-view",
        osview::available() && v.mappings > 0,
        format!(
            "{} mappings, VmSize {} B, max_map_count {}",
            v.mappings, v.virtual_bytes, v.max_map_count
        ),
    );

    // 9. Concurrency is real: four threads barrier-synchronised, each
    //    reporting its own window, must overlap almost entirely.
    let (ov, dur) = overlap_probe(4);
    check(
        "concurrency",
        ov >= 0.9,
        format!("overlap {ov:.3} of a {dur:.3} s window"),
    );

    println!("{}\tgate", if all { "PASS" } else { "FAIL" });
    if !all {
        std::process::exit(1);
    }
}

/// Runs `t` barrier-synchronised threads that spin for a fixed time and
/// returns how much of their windows overlapped.
fn overlap_probe(t: usize) -> (f64, f64) {
    let bar = Arc::new(Barrier::new(t));
    let starts: Arc<Vec<AtomicU64>> = Arc::new((0..t).map(|_| AtomicU64::new(0)).collect());
    let ends: Arc<Vec<AtomicU64>> = Arc::new((0..t).map(|_| AtomicU64::new(0)).collect());
    let t0 = Instant::now();
    std::thread::scope(|sc| {
        for i in 0..t {
            let (bar, starts, ends) = (bar.clone(), starts.clone(), ends.clone());
            sc.spawn(move || {
                bar.wait();
                starts[i].store(t0.elapsed().as_nanos() as u64, Ordering::Relaxed);
                let mut x = 0u64;
                let until = Instant::now();
                while until.elapsed().as_millis() < 200 {
                    for _ in 0..10_000 {
                        x = x.wrapping_add(std::hint::black_box(1));
                    }
                }
                std::hint::black_box(x);
                ends[i].store(t0.elapsed().as_nanos() as u64, Ordering::Relaxed);
            });
        }
    });
    window_overlap(&starts, &ends)
}

/// The fraction of each thread's window that every thread was inside:
/// `(min end − max start) / mean duration`.
fn window_overlap(starts: &[AtomicU64], ends: &[AtomicU64]) -> (f64, f64) {
    let s: Vec<u64> = starts.iter().map(|a| a.load(Ordering::Relaxed)).collect();
    let e: Vec<u64> = ends.iter().map(|a| a.load(Ordering::Relaxed)).collect();
    let max_start = s.iter().copied().max().unwrap_or(0);
    let min_end = e.iter().copied().min().unwrap_or(0);
    let mean: f64 = s
        .iter()
        .zip(&e)
        .map(|(a, b)| b.saturating_sub(*a) as f64)
        .sum::<f64>()
        / s.len().max(1) as f64;
    let common = min_end.saturating_sub(max_start) as f64;
    if mean <= 0.0 {
        return (0.0, 0.0);
    }
    (common / mean, mean / 1e9)
}

// --------------------------------------------------------------- space

/// Address space as agents multiply: the substrate's ledger beside the
/// kernel's own view of the single reservation.
fn space(args: &[String]) {
    let counts = list(args, "--agents", &[1, 2, 4, 8, 16, 32, 64, 128, 256]);
    let mib = opt(args, "--mib-per-agent", 2u64);
    let per_agent_cells = mib * CHUNK / 32; // a Cell is well under 32 bytes
    println!(
        "agents\treservations\treserved_B\tcommitted_B\tcommitted_process_B\tcommitted_peragent_B\tcommit_calls\tchunks\tmappings\tmappings_in_cage\tvmsize_B\tvmrss_B\tvmpeak_B\tmax_map_count\tper_agent_committed_B\tper_agent_mappings\tchecksum_ok"
    );
    for a in counts {
        let mem = Arc::new(SpyMemory::new());
        let base = osview::read(None);
        let cage = ProcessCage::new(mem.clone(), CAGE_BYTES).expect("16 GiB cage");
        let cage_range = mem.reservations().first().map(|r| (r.base, r.len));
        let mut heaps: Vec<Heap> = (0..a)
            .map(|i| work::heap_in(&cage, work::no_gc(u32::try_from(i).unwrap_or(0))))
            .collect();
        // Round-robin, as agents of one process actually grow.
        let mut ok = true;
        let rounds = 8u64;
        for _ in 0..rounds {
            for h in &mut heaps {
                let d = work::grow(h, per_agent_cells / rounds);
                ok &= d.refused == 0;
            }
        }
        let st = mem.stats();
        let calls = mem.calls();
        let v = osview::read(cage_range);
        let chunks = calls.commit_bytes / CHUNK;
        let per_agent = st.committed_by_kind[MemoryKind::PerAgent.index()] / a.max(1);
        println!(
            "{a}\t{}\t{}\t{}\t{}\t{}\t{}\t{chunks}\t{}\t{}\t{}\t{}\t{}\t{}\t{per_agent}\t{:.3}\t{ok}",
            calls.reserves,
            st.reserved,
            st.committed,
            st.committed_by_kind[MemoryKind::ProcessPart.index()],
            st.committed_by_kind[MemoryKind::PerAgent.index()],
            calls.commits,
            v.mappings,
            v.mappings_in_range,
            v.virtual_bytes,
            v.resident_bytes,
            v.peak_virtual_bytes,
            v.max_map_count,
            (v.mappings as f64 - base.mappings as f64) / a as f64,
        );
        drop(heaps);
        drop(cage);
    }
}

/// How many agent heaps a process can carry in one cage. Each agent
/// allocates one object, which is the floor: one chunk.
fn space_max(args: &[String]) {
    let limit = opt(args, "--limit", 4096u64);
    let cap = opt(args, "--commit-cap-mib", 4096u64) * CHUNK;
    let mem = Arc::new(SpyMemory::new());
    let cage = ProcessCage::new(mem.clone(), CAGE_BYTES).expect("16 GiB cage");
    let cage_range = mem.reservations().first().map(|r| (r.base, r.len));
    let mut heaps: Vec<Heap> = Vec::new();
    let mut stopped = "limit";
    println!("agents\tcommitted_B\tchunks\tmappings\tmappings_in_cage\tvmrss_B\tstopped");
    for i in 0..limit {
        let mut h = work::heap_in(&cage, work::no_gc(u32::try_from(i & 0xFFFF).unwrap_or(0)));
        let did = work::grow(&mut h, 1);
        if did.refused > 0 {
            stopped = "cage-full";
            break;
        }
        heaps.push(h);
        let st = mem.stats();
        if st.committed > cap {
            stopped = "commit-cap";
            break;
        }
        if (i + 1).is_power_of_two() || i + 1 == limit {
            let v = osview::read(cage_range);
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\tgoing",
                i + 1,
                st.committed,
                mem.calls().commit_bytes / CHUNK,
                v.mappings,
                v.mappings_in_range,
                v.resident_bytes,
            );
        }
    }
    let st = mem.stats();
    let v = osview::read(cage_range);
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{stopped}",
        heaps.len(),
        st.committed,
        mem.calls().commit_bytes / CHUNK,
        v.mappings,
        v.mappings_in_range,
        v.resident_bytes,
    );
    // The arithmetic ceiling, independent of commit: chunk 0 is never
    // handed out, so a cage of CAGE_BYTES has this many chunks to give.
    eprintln!(
        "arithmetic ceiling: {} chunks of {} bytes = {} agents at the 1-chunk floor",
        CAGE_BYTES / CHUNK - 1,
        CHUNK,
        CAGE_BYTES / CHUNK - 1
    );
}

/// How many 16 GiB reservations one process can make: what alternative
/// (b), one cage per agent, would have to buy.
fn reservations(args: &[String]) {
    let limit = opt(args, "--limit", 64u64);
    let bytes = opt(args, "--gib", 16u64) << 30;
    let mem = Arc::new(SpyMemory::new());
    let mut cages: Vec<ProcessCage> = Vec::new();
    println!("cages\treserved_B\tmappings\tvmsize_B\tvmrss_B\tstatus");
    let mut status = "limit";
    for i in 0..limit {
        match ProcessCage::new(mem.clone(), bytes) {
            Ok(c) => cages.push(c),
            Err(_) => {
                status = "reserve-failed";
                break;
            }
        }
        if (i + 1).is_power_of_two() || i + 1 == limit {
            let v = osview::read(None);
            let st = mem.stats();
            println!(
                "{}\t{}\t{}\t{}\t{}\tgoing",
                i + 1,
                st.reserved,
                v.mappings,
                v.virtual_bytes,
                v.resident_bytes
            );
        }
    }
    let v = osview::read(None);
    let st = mem.stats();
    println!(
        "{}\t{}\t{}\t{}\t{}\t{status}",
        cages.len(),
        st.reserved,
        v.mappings,
        v.virtual_bytes,
        v.resident_bytes
    );
}

/// What agents coming and going do to the one reservation: each round
/// makes an agent heap, takes `blocks` blocks and drops it, which
/// decommits its chunks. On Linux a decommit is an `mmap` `MAP_FIXED`
/// `PROT_NONE` over the range, so a hole in a committed run splits the
/// mapping; this counts what that comes to.
fn churn_maps(args: &[String]) {
    let rounds = opt(args, "--rounds", 2048u64);
    let blocks = opt(args, "--blocks", 40u64);
    let live = opt(args, "--live", 4u64);
    let mem = Arc::new(SpyMemory::new());
    let cage = ProcessCage::new(mem.clone(), CAGE_BYTES).expect("cage");
    let cage_range = mem.reservations().first().map(|r| (r.base, r.len));
    // `live` long-lived agents that keep growing, so the short-lived ones
    // punch holes between their chunks.
    let mut resident: Vec<Heap> = (0..live)
        .map(|i| work::heap_in(&cage, work::no_gc(u32::try_from(i).unwrap_or(0))))
        .collect();
    println!(
        "round\tcommitted_B\tcommit_calls\tdecommit_calls\tdecommit_failed\tmappings\tmappings_in_cage\tmax_map_count"
    );
    for r in 0..rounds {
        for h in &mut resident {
            h.enter(|m| work::take_chunks(m, 2));
        }
        let mut h = work::heap_in(&cage, work::no_gc(1000));
        h.enter(|m| work::take_chunks(m, blocks));
        drop(h);
        if (r + 1).is_power_of_two() || r + 1 == rounds {
            let st = mem.stats();
            let c = mem.calls();
            let v = osview::read(cage_range);
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                r + 1,
                st.committed,
                c.commits,
                c.decommits,
                c.decommit_failures,
                v.mappings,
                v.mappings_in_range,
                v.max_map_count
            );
        }
    }
}

// ------------------------------------------------------------- contend

/// The three arms: what each agent gets of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arm {
    /// JS-D17 (a): one cage, one memory, every agent heap inside it.
    SharedCage,
    /// JS-D17 (b): a cage per agent, one memory (so the substrate's own
    /// ledger lock is still shared, as it would be in one process).
    CagePerAgent,
    /// The no-interference ideal: a cage and a memory per agent, so
    /// nothing in the substrate is shared.
    CageAndMemPerAgent,
}

impl Arm {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "shared" => Some(Self::SharedCage),
            "percage" => Some(Self::CagePerAgent),
            "private" => Some(Self::CageAndMemPerAgent),
            _ => None,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::SharedCage => "shared",
            Self::CagePerAgent => "percage",
            Self::CageAndMemPerAgent => "private",
        }
    }
}

/// Several agent heaps allocating at once. Per-agent throughput as
/// agents are added, in each arm, so the cost of sharing one cage is the
/// difference between the arms rather than an absolute figure.
fn contend(args: &[String]) {
    let counts = list(args, "--agents", &[1, 2, 4, 8]);
    let mode: String = opt(args, "--mode", "grow".to_string());
    let reps = opt(args, "--reps", 3u64);
    let n = opt(args, "--n", 400_000u64);
    let arms: Vec<Arm> = opt(args, "--arms", "shared,percage,private".to_string())
        .split(',')
        .filter_map(Arm::parse)
        .collect();
    println!(
        "mode\tarm\tagents\trep\tops_per_agent\tsecs\tops_per_s_per_agent\tns_per_op\tmhz_os_max\tmhz_os_mean\tref_cyc_per_add\tcycles_per_op\toverlap\tcommit_calls\tdecommit_calls\tchecksum_ok\tverdict"
    );
    for a in &counts {
        for arm in &arms {
            for rep in 0..reps {
                let c = clock::measure(200_000_000);
                let r = contend_cell(*arm, *a, &mode, n);
                let ok = r.ok && c.sound() && r.overlap >= 0.9;
                let per = if r.secs > 0.0 {
                    r.ops as f64 / r.secs / *a as f64
                } else {
                    0.0
                };
                let ns = if per > 0.0 { 1e9 / per } else { 0.0 };
                println!(
                    "{mode}\t{}\t{a}\t{rep}\t{}\t{:.6}\t{:.1}\t{:.2}\t{:.0}\t{:.0}\t{:.2}\t{:.2}\t{:.3}\t{}\t{}\t{}\t{}",
                    arm.name(),
                    r.ops / *a,
                    r.secs,
                    per,
                    ns,
                    c.mhz_os_max,
                    c.mhz_os_mean,
                    c.ref_cycles_per_add,
                    ns * c.ghz(),
                    r.overlap,
                    r.commits,
                    r.decommits,
                    r.ok,
                    if ok { "ok" } else { "FAIL" }
                );
            }
        }
    }
}

/// One contention cell's outcome.
struct CellResult {
    ops: u64,
    secs: f64,
    overlap: f64,
    commits: u64,
    decommits: u64,
    ok: bool,
}

fn contend_cell(arm: Arm, agents: u64, mode: &str, n: u64) -> CellResult {
    let t = usize::try_from(agents).unwrap_or(1);
    // One memory for the shared arms; one per agent for `private`.
    let shared_mem = Arc::new(SpyMemory::new());
    let mems: Vec<Arc<SpyMemory>> = match arm {
        Arm::CageAndMemPerAgent => (0..t).map(|_| Arc::new(SpyMemory::new())).collect(),
        _ => (0..t).map(|_| shared_mem.clone()).collect(),
    };
    let cages: Vec<ProcessCage> = match arm {
        Arm::SharedCage => {
            let c = ProcessCage::new(shared_mem.clone(), CAGE_BYTES).expect("cage");
            (0..t).map(|_| c.clone()).collect()
        }
        _ => mems
            .iter()
            .map(|m| ProcessCage::new(m.clone(), CAGE_BYTES).expect("cage"))
            .collect(),
    };

    let bar = Arc::new(Barrier::new(t));
    let starts: Arc<Vec<AtomicU64>> = Arc::new((0..t).map(|_| AtomicU64::new(0)).collect());
    let ends: Arc<Vec<AtomicU64>> = Arc::new((0..t).map(|_| AtomicU64::new(0)).collect());
    let t0 = Instant::now();
    let mut ops = 0u64;
    let mut ok = true;
    let results: Vec<Did> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..t)
            .map(|i| {
                let (bar, starts, ends) = (bar.clone(), starts.clone(), ends.clone());
                let cage = cages[i].clone();
                let mode = mode.to_string();
                sc.spawn(move || {
                    let agent = u32::try_from(i).unwrap_or(0);
                    bar.wait();
                    starts[i].store(t0.elapsed().as_nanos() as u64, Ordering::Relaxed);
                    let did = match mode.as_str() {
                        "steady" => {
                            let mut h = work::heap_in(&cage, work::with_gc(agent, 4096));
                            work::steady(&mut h, n)
                        }
                        "churn" => work::churn(&cage, agent, n, 40),
                        _ => {
                            let mut h = work::heap_in(&cage, work::no_gc(agent));
                            work::grow(&mut h, n)
                        }
                    };
                    ends[i].store(t0.elapsed().as_nanos() as u64, Ordering::Relaxed);
                    did
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("worker")).collect()
    });
    for d in &results {
        ops += d.allocs;
        ok &= d.refused == 0;
    }
    let (overlap, mean) = window_overlap(&starts, &ends);
    let (mut commits, mut decommits) = (0, 0);
    for m in mems
        .iter()
        .take(if arm == Arm::CageAndMemPerAgent { t } else { 1 })
    {
        let c = m.calls();
        commits += c.commits;
        decommits += c.decommits;
    }
    CellResult {
        ops,
        secs: mean,
        overlap,
        commits,
        decommits,
        ok,
    }
}

// ----------------------------------------------------------- interleave

/// What one agent heap's own pointer chase costs when its chunks sit
/// next to each other (it is alone in the cage) against when `others`
/// agents grew alongside it and its chunks are a megabyte apart. Sharing
/// a cage cannot change the encoding — one base, by construction — but it
/// can change where an agent's objects land, which is the other half of
/// "address-space contention between agents".
fn interleave(args: &[String]) {
    let others = list(args, "--others", &[0, 1, 3, 7, 15]);
    let cells = opt(args, "--cells", 400_000u64);
    let laps = opt(args, "--laps", 20u64);
    let reps = opt(args, "--reps", 3u64);
    let rounds = opt(args, "--rounds", 16u64);
    println!(
        "others\trep\tcells\tchunks_spanned\tchunk_stride\tlaps\tns_per_step\tmhz_os_max\tmhz_os_mean\tref_cyc_per_add\tcycles_per_step\tchecksum_ok\tverdict"
    );
    for o in others {
        for rep in 0..reps {
            let c = clock::measure(200_000_000);
            let mem = Arc::new(SpyMemory::new());
            let cage = ProcessCage::new(mem, CAGE_BYTES).expect("cage");
            let mut mine = work::heap_in(&cage, work::no_gc(0));
            let mut rest: Vec<Heap> = (0..o)
                .map(|i| work::heap_in(&cage, work::no_gc(u32::try_from(i + 1).unwrap_or(0))))
                .collect();
            let (ns, span, stride, ok) =
                chase_interleaved(&mut mine, &mut rest, cells, rounds, laps);
            let good = ok && c.sound();
            println!(
                "{o}\t{rep}\t{cells}\t{span}\t{stride}\t{laps}\t{:.3}\t{:.0}\t{:.0}\t{:.2}\t{:.2}\t{ok}\t{}",
                ns,
                c.mhz_os_max,
                c.mhz_os_mean,
                c.ref_cycles_per_add,
                ns * c.ghz(),
                if good { "ok" } else { "FAIL" }
            );
            drop(rest);
            drop(mine);
            drop(cage);
        }
    }
}

/// Builds a chain of `cells` cells in `mine`, in `rounds` rounds with
/// each of the `rest` agents growing by as much between rounds, so
/// `mine`'s chunks are separated by theirs in the shared cage. Then
/// walks the chain for `laps` laps, one dependent load per step.
///
/// Returns nanoseconds per step, the chunks the chain spans, the mean
/// gap in chunks between the ones it uses, and whether every lap walked
/// the same number of steps and the sum came out right — a walk that
/// fell out of the chain would be fast and wrong.
fn chase_interleaved(
    mine: &mut Heap,
    rest: &mut [Heap],
    cells: u64,
    rounds: u64,
    laps: u64,
) -> (f64, u64, u64, bool) {
    let per_round = cells / rounds.max(1);
    mine.enter(|m| {
        m.scope(|m, s| {
            let head = match m.alloc::<work::Cell>(s, 0) {
                Ok(c) => c,
                Err(_) => return (0.0, 0, 0, false),
            };
            let mut n = 0u64;
            let mut want = 0u64;
            let mut chunks = std::collections::BTreeSet::new();
            for _ in 0..rounds {
                for _ in 0..per_round {
                    let Ok(c) = m.alloc::<work::Cell>(s, 0) else {
                        break;
                    };
                    let cg = c.get(m);
                    let v = u32::try_from(n & 0xFFFF).unwrap_or(0);
                    cg.set_raw(m, work::Cell::N, v);
                    chunks.insert(chunk_of(offset_of(cg.slot())));
                    // Prepended to the chain, so the walk runs in reverse
                    // allocation order: one dependent load per step,
                    // whatever order the chunks came in.
                    let hd = head.get(m);
                    cg.set(m, work::Cell::NEXT, hd.get(m, work::Cell::NEXT));
                    hd.set(m, work::Cell::NEXT, Some(cg));
                    want += u64::from(v);
                    n += 1;
                }
                for h in rest.iter_mut() {
                    work::grow(h, per_round);
                }
            }
            let span = u64::try_from(chunks.len()).unwrap_or(0);
            let stride = match (chunks.first(), chunks.last()) {
                (Some(a), Some(b)) if span > 1 => (b - a) / (span - 1),
                _ => 0,
            };
            let mut steps = 0u64;
            let mut sum = 0u64;
            let t = Instant::now();
            for _ in 0..laps {
                let mut cur = head.get(m).get(m, work::Cell::NEXT);
                let mut this = 0u64;
                while let Some(cg) = cur {
                    sum += u64::from(cg.raw(m, work::Cell::N));
                    this += 1;
                    cur = cg.get(m, work::Cell::NEXT);
                }
                if this != n {
                    return (0.0, span, stride, false);
                }
                steps += this;
            }
            let secs = t.elapsed().as_secs_f64();
            std::hint::black_box(sum);
            let ok = steps == n * laps && sum == want * laps && n > 0;
            (
                if steps > 0 {
                    secs * 1e9 / steps as f64
                } else {
                    0.0
                },
                span,
                stride,
                ok,
            )
        })
    })
}

// ---------------------------------------------------------------- maps

/// The kernel's own lines for the cage's address range, after `agents`
/// agent heaps have grown in it and `churn` short-lived ones have come
/// and gone. This is the raw evidence behind the mapping counts: what one
/// 16 GiB reservation holding several agent heaps actually looks like to
/// the OS.
fn maps(args: &[String]) {
    let agents = opt(args, "--agents", 8u64);
    let rounds = opt(args, "--churn", 64u64);
    let mem = Arc::new(SpyMemory::new());
    let cage = ProcessCage::new(mem.clone(), CAGE_BYTES).expect("cage");
    let (base, len) = mem
        .reservations()
        .first()
        .map(|r| (r.base, r.len))
        .unwrap_or((0, 0));
    let mut heaps: Vec<Heap> = (0..agents)
        .map(|i| work::heap_in(&cage, work::no_gc(u32::try_from(i).unwrap_or(0))))
        .collect();
    for _ in 0..8 {
        for h in &mut heaps {
            work::grow(h, 20_000);
        }
    }
    for _ in 0..rounds {
        let mut h = work::heap_in(&cage, work::no_gc(999));
        h.enter(|m| work::take_chunks(m, 40));
        drop(h);
        for h in &mut heaps {
            work::grow(h, 2_000);
        }
    }
    let st = mem.stats();
    let c = mem.calls();
    println!(
        "# cage base 0x{base:x} len {len} ({} GiB); {agents} agents, {rounds} churn rounds",
        len >> 30
    );
    println!(
        "# substrate ledger: {} reservation(s), reserved {} B, committed {} B, {} commits, {} decommits ({} refused)",
        c.reserves, st.reserved, st.committed, c.commits, c.decommits, c.decommit_failures
    );
    #[cfg(target_os = "linux")]
    if let Ok(text) = std::fs::read_to_string("/proc/self/maps") {
        println!("# /proc/self/maps lines inside the cage:");
        for line in text.lines() {
            if let Some((start, _)) = line.split_once('-')
                && let Ok(a) = usize::from_str_radix(start, 16)
                && a >= base
                && a < base.saturating_add(len)
            {
                println!("{line}");
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let pid = std::process::id().to_string();
        if let Ok(out) = std::process::Command::new("/usr/bin/vmmap")
            .arg("-w")
            .arg(&pid)
            .output()
        {
            println!("# vmmap lines inside the cage:");
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if in_cage_line(line, base, len) {
                    println!("{line}");
                }
            }
        }
    }
    let v = osview::read(Some((base, len)));
    println!(
        "# OS view: {} mappings in all, {} inside the cage, VmSize {} B, VmRSS {} B, max_map_count {}",
        v.mappings, v.mappings_in_range, v.virtual_bytes, v.resident_bytes, v.max_map_count
    );
}

/// Whether a `vmmap` line's region starts inside `[base, base + len)`.
#[cfg(target_os = "macos")]
fn in_cage_line(line: &str, base: usize, len: usize) -> bool {
    for field in line.split_whitespace() {
        if let Some((a, b)) = field.split_once('-')
            && a.len() >= 9
            && b.len() >= 9
            && a.bytes().all(|c| c.is_ascii_hexdigit())
            && let Ok(addr) = usize::from_str_radix(a, 16)
        {
            return addr >= base && addr < base.saturating_add(len);
        }
    }
    false
}
