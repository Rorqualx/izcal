//! JS-D16 at the JS-M0 spike: what the trusted region costs on the call
//! and dispatch paths, against (b) — the same bytecode in the cage, with
//! the operand re-checks H-2 then forces.
//!
//! Subcommands: `gate` (the validity gate; every other subcommand refuses
//! to print a measurement until it passes), `dispatch`, `entry`, `e2e`,
//! `window`, `maps`, `seq`.
//!
//! Every timed loop is built with the check counter off, times its own
//! clock calibration, and prints raw rows: one line per repeat, so the
//! note can take minima and spreads without the harness choosing them.

#![forbid(unsafe_code)]

mod clock;
mod dispatch;
mod e2e;
mod entry;
mod progs;
mod source;

use std::hint::black_box;
use std::sync::Arc;

use hg_heap::{TrustedId, TrustedKind, TrustedRegion};
use hg_js_bytecode::{OperandKind, Unit, UnitView, decode};
use hg_js_ops::support::HostValue;
use hg_substrate::Memory;
use hg_substrate_os::memory::OsMemory;

use source::{Caged, Source, Trusted, admits};

/// The programs every mode runs, with the number of iterations the
/// arithmetic loop is built for.
fn programs() -> Vec<(String, Unit)> {
    vec![
        ("arith".to_string(), progs::arith(1000).expect("arith")),
        ("consts8".to_string(), progs::consts(8).expect("consts8")),
        ("consts64".to_string(), progs::consts(64).expect("consts64")),
        ("props".to_string(), progs::props().expect("props")),
        ("branchy".to_string(), progs::branchy().expect("branchy")),
    ]
}

fn os_memory() -> Arc<dyn Memory> {
    Arc::new(OsMemory::new())
}

/// A region holding one unit, with its id.
fn seal_one(mem: &Arc<dyn Memory>, unit: &Unit) -> (TrustedRegion, TrustedId) {
    let mut region = TrustedRegion::new(Arc::clone(mem), 4 << 20).expect("trusted region");
    let id = hg_js_bytecode::seal(&mut region, unit).expect("seal");
    (region, id)
}

// ---------------------------------------------------------------- the gate

/// The validity gate. A fast wrong number is the failure to guard
/// against, so nothing is timed until all of this holds.
fn gate(to_stderr: bool) -> bool {
    let mem = os_memory();
    let mut ok = true;
    let mut say = |name: &str, pass: bool, detail: String| {
        let line = format!("{} {name}\t{detail}", if pass { "PASS" } else { "FAIL" });
        // The measurement modes send the gate to stderr, so a results
        // file holds rows only; `gate` itself prints to stdout.
        if to_stderr {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
        ok = ok && pass;
    };

    // G1: the clock's own chain computed the arithmetic it must have, and
    // the frequency it implies is a real frequency.
    let c = clock::calibrate(200_000_000);
    // The chain's own cycles per add: about one is what a dependent
    // 64-bit add costs, and a figure far from it says the chain is not
    // what it is taken for.
    let cyc_per_add = c.cycles(c.ns_per_add);
    let plausible = c.ok
        && c.ghz > 0.3
        && c.ghz < 7.0
        && (c.ghz_source != "cpuinfo-max" || (0.8..3.0).contains(&cyc_per_add));
    say(
        "G1-clock",
        plausible,
        format!(
            "{:.4} ns/add = {cyc_per_add:.2} cyc/add at {:.3} GHz (from {}; with the chain as the clock that figure is 1.00 by construction and checks nothing); /proc/cpuinfo {:.0}-{:.0} MHz; governor {}",
            c.ns_per_add,
            c.ghz,
            c.ghz_source,
            c.mhz_min,
            c.mhz_max,
            clock::governor()
        ),
    );

    for (name, unit) in programs() {
        let image = unit.to_image();
        let (region, id) = seal_one(&mem, &unit);
        let sealed = hg_js_bytecode::open(&region, id).expect("open");
        let parsed = UnitView::parse(&image).expect("parse");
        let t = Trusted(sealed);
        let t_vec = Trusted(parsed);
        let c = Caged { img: &image };

        // G2: every variant walks the same path and agrees on the digest
        // of every instruction it decoded. A variant that read the wrong
        // bytes, or stopped early, cannot agree by accident.
        let steps = 20_000;
        let a_prod = dispatch::run::<_, 1, false, true>(&t, steps);
        let a_ref = dispatch::run::<_, 1, true, true>(&t, steps);
        let a_vec = dispatch::run::<_, 1, false, true>(&t_vec, steps);
        let b_use = dispatch::run::<_, 1, true, true>(&c, steps);
        let b_val = dispatch::run_revalidated::<1>(&image, steps);
        let cells = [a_prod, a_ref, a_vec, b_use, b_val];
        let agree = cells.iter().all(|x| x.is_some())
            && cells
                .iter()
                .all(|x| x.map(|c| (c.digest, c.instrs)) == cells[0].map(|c| (c.digest, c.instrs)));
        say(
            &format!("G2-agree[{name}]"),
            agree,
            match cells[0] {
                Some(c) => format!(
                    "{} instructions, digest {:#018x}, 5 variants agree: {}",
                    c.instrs,
                    c.digest,
                    cells
                        .iter()
                        .map(|x| match x {
                            Some(c) => format!("{:#x}", c.digest),
                            None => "none".to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                None => "a variant did not complete".to_string(),
            },
        );

        // G3: (b)'s re-checks really execute. The timed runs have the
        // counter off, so this run has it on and proves the work is there
        // to be timed: at least the code section, the two frame counts
        // and one check per operand, per instruction.
        let (bc, ac, apc) = (
            b_use.map_or(0, |c| c.checks),
            a_ref.map_or(0, |c| c.checks),
            a_prod.map_or(0, |c| c.checks),
        );
        let instrs = a_prod.map_or(1, |c| c.instrs);
        let live = bc >= instrs * 3 && bc > ac && apc == 0;
        say(
            &format!("G3-checks[{name}]"),
            live,
            format!(
                "{:.2} checks/instruction under (b), {:.2} under (a)+admits, {apc} under (a)",
                bc as f64 / instrs as f64,
                ac as f64 / instrs as f64
            ),
        );

        // G4: the re-checks are what catches a corrupted operand. One
        // operand byte of the image is flipped to a register outside the
        // frame: (b) must refuse it, (a)'s untouched trusted copy must
        // not notice, the harness's check and the real interpreter's own
        // `Cx::admits` must agree on refusing it.
        if let Some((bad, at)) = corrupt_reg(&image) {
            let bad_view = UnitView::parse(&bad);
            let bad_caged = Caged { img: &bad };
            let mut checks = 0;
            let i0 = decode(bad_view.map(|v| v.code()).unwrap_or(&[]), 0);
            let harness_refuses = match (bad_view, i0) {
                (Ok(v), Ok(_)) => {
                    let mut pc = 0u32;
                    let mut refused = false;
                    while let Ok(i) = decode(v.code(), pc) {
                        if !admits::<_, false>(
                            &i,
                            &Trusted(v),
                            v.params(),
                            v.registers(),
                            &mut checks,
                        ) {
                            refused = true;
                            break;
                        }
                        pc = i.end;
                        if usize::try_from(pc).unwrap_or(usize::MAX) >= v.code().len() {
                            break;
                        }
                    }
                    refused
                }
                _ => false,
            };
            let caged_refuses = dispatch::run::<_, 1, true, false>(&bad_caged, 2000).is_none();
            let trusted_runs = dispatch::run::<_, 1, false, false>(&t, 2000).is_some();
            say(
                &format!("G4-poison[{name}]"),
                harness_refuses && caged_refuses && trusted_runs,
                format!(
                    "operand byte at {at} made a register outside the frame: harness refuses {harness_refuses}, (b) refuses {caged_refuses}, untouched (a) still runs {trusted_runs}"
                ),
            );
        }
    }

    // G4b: the reference interpreter's own `Cx::admits` refuses exactly
    // what the harness's check refuses. Without this the harness could be
    // measuring a check that is not the engine's.
    let n = 1000u32;
    let arith = progs::arith(n).expect("arith");
    let good = arith.to_image();
    let intact = e2e::run_once(&mem, &good);
    let poisoned = corrupt_reg(&good).map(|(bad, _)| e2e::run_once(&mem, &bad));
    let want = f64::from(n) * (f64::from(n) + 1.0) / 2.0;
    let both = matches!(&intact, Ok(HostValue::Number(x)) if *x == want)
        && matches!(&poisoned, Some(Err(e)) if e.starts_with("BadBytecode"));
    say(
        "G4b-real-admits",
        both,
        format!("intact {intact:?}, corrupted {poisoned:?}"),
    );

    // G5: the real interpreter returns the same value under (a) and (b),
    // and that value is the arithmetic the program must have done.
    let a = e2e::run(&mem, &arith, 3, e2e::Where::Trusted);
    let b = e2e::run(&mem, &arith, 3, e2e::Where::Caged);
    let got_a = a.as_ref().and_then(|c| c.value.clone());
    let got_b = b.as_ref().and_then(|c| c.value.clone());
    let right = matches!(&got_a, Some(HostValue::Number(x)) if *x == want)
        && matches!(&got_b, Some(HostValue::Number(x)) if *x == want);
    say(
        "G5-interpreter",
        right,
        format!("sum 1..={n} = {want}; (a) {got_a:?}, (b) {got_b:?}"),
    );

    // G6: the trusted region's committed pages really are read-only
    // outside a write window (Linux; /proc/self/maps is the record).
    let mem2 = os_memory();
    let (region, id) = seal_one(&mem2, &arith);
    let lines = maps_for_region(4 << 20);
    let have = !lines.is_empty();
    say(
        "G6-readonly",
        have || !std::path::Path::new("/proc/self/maps").exists(),
        if have {
            format!("/proc/self/maps: {}", lines.join(" | "))
        } else {
            "no /proc/self/maps on this OS; not checked here".to_string()
        },
    );
    black_box(region.get(id, TrustedKind::BytecodeUnit));

    ok
}

/// An image with one register operand changed to a register outside the
/// frame, and the offset that was changed.
fn corrupt_reg(image: &[u8]) -> Option<(Vec<u8>, usize)> {
    let view = UnitView::parse(image).ok()?;
    let code = view.code();
    // Where the code section starts in the image, by address difference:
    // the view borrows these very bytes, so the subtraction is in range,
    // and a checked one keeps that an assumption this code states.
    let base = (code.as_ptr() as usize).checked_sub(image.as_ptr() as usize)?;
    let mut pc = 0u32;
    while let Ok(i) = decode(code, pc) {
        if i.scale == 1 {
            for (j, k) in i.op.info().operands.iter().enumerate() {
                if *k == OperandKind::Reg {
                    let at = base + usize::try_from(i.start).ok()? + 1 + j;
                    let mut out = image.to_vec();
                    *out.get_mut(at)? = 0x7F;
                    return Some((out, at));
                }
            }
        }
        pc = i.end;
        if usize::try_from(pc).ok()? >= code.len() {
            break;
        }
    }
    None
}

/// The `r--p` mapping followed by the `---p` remainder of a reservation
/// of `bytes`: the trusted region as the kernel sees it.
fn maps_for_region(bytes: usize) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string("/proc/self/maps") else {
        return Vec::new();
    };
    let lines: Vec<&str> = text.lines().collect();
    let span = |l: &str| -> Option<(usize, usize, String)> {
        let mut it = l.split_whitespace();
        let range = it.next()?;
        let perms = it.next()?.to_string();
        let (a, b) = range.split_once('-')?;
        Some((
            usize::from_str_radix(a, 16).ok()?,
            usize::from_str_radix(b, 16).ok()?,
            perms,
        ))
    };
    for w in lines.windows(2) {
        let (Some((a0, a1, p0)), Some((b0, b1, p1))) = (span(w[0]), span(w[1])) else {
            continue;
        };
        if p0.starts_with("r--")
            && p1.starts_with("---")
            && a1 == b0
            && (a1 - a0) + (b1 - b0) == bytes
        {
            return vec![w[0].to_string(), w[1].to_string()];
        }
    }
    Vec::new()
}

// --------------------------------------------------------- the measurements

/// `dispatch`: per-instruction cost, one row per repeat.
fn dispatch_mode(total: u64, reps: u32) {
    println!(
        "program\tvariant\tstreams\trep\tinstrs\tns\tns_per_instr\tghz\tcyc_per_instr\tdigest"
    );
    let mem = os_memory();
    for (name, unit) in programs() {
        let image = unit.to_image();
        let (region, id) = seal_one(&mem, &unit);
        let sealed = hg_js_bytecode::open(&region, id).expect("open");
        let t = Trusted(sealed);
        let t_vec = Trusted(UnitView::parse(&image).expect("parse"));
        let c = Caged { img: &image };
        for rep in 0..reps {
            // One calibration per cell group, next to the cells it scales.
            let cal = clock::calibrate(80_000_000);
            if !cal.ok {
                println!("FAIL calibration did not verify");
                return;
            }
            let row = |variant: &str, streams: usize, cell: Option<dispatch::Cell>| match cell {
                Some(x) => println!(
                    "{name}\t{variant}\t{streams}\t{rep}\t{}\t{}\t{:.4}\t{:.4}\t{:.4}\t{:#018x}",
                    x.instrs,
                    x.ns,
                    x.per_instr(),
                    cal.ghz,
                    cal.cycles(x.per_instr()),
                    x.digest
                ),
                None => println!("FAIL\t{name}\t{variant}\t{streams}\t{rep}"),
            };
            row(
                "a-trusted",
                1,
                dispatch::run::<_, 1, false, false>(&t, total),
            );
            row(
                "a-trusted-vec",
                1,
                dispatch::run::<_, 1, false, false>(&t_vec, total),
            );
            row(
                "a-ref-admits",
                1,
                dispatch::run::<_, 1, true, false>(&t, total),
            );
            row(
                "b-cage-use",
                1,
                dispatch::run::<_, 1, true, false>(&c, total),
            );
            row(
                "b-cage-revalidate",
                1,
                dispatch::run_revalidated::<1>(&image, total),
            );
            // The resolution diagnostic: work that reads nothing, so a
            // cell no slower than (a) says the loop had slack to spend.
            row("x-pad2", 1, dispatch::run_padded::<_, 1>(&t, total, 2));
            row("x-pad8", 1, dispatch::run_padded::<_, 1>(&t, total, 8));
            let q = total / 4;
            row("a-trusted", 4, dispatch::run::<_, 4, false, false>(&t, q));
            row("a-ref-admits", 4, dispatch::run::<_, 4, true, false>(&t, q));
            row("b-cage-use", 4, dispatch::run::<_, 4, true, false>(&c, q));
            row(
                "b-cage-revalidate",
                4,
                dispatch::run_revalidated::<4>(&image, q),
            );
            row("x-pad2", 4, dispatch::run_padded::<_, 4>(&t, q, 2));
            row("x-pad8", 4, dispatch::run_padded::<_, 4>(&t, q, 8));
        }
        // The mix the rows above were measured on.
        let hist = dispatch::histogram(&t, 2000);
        let mix: Vec<String> = hist.iter().map(|(m, n)| format!("{m}:{n}")).collect();
        println!("# mix[{name}] over 2000 instructions: {}", mix.join(" "));
    }
}

/// `entry`: the call path, one row per repeat.
fn entry_mode(n: u64, reps: u32) {
    println!("unit\tconstants\tvariant\trep\tentries\tns\tns_per_entry\tghz\tcyc_per_entry\tacc");
    let mem = os_memory();
    let units: Vec<(String, Unit)> = vec![
        ("tiny0".to_string(), progs::tiny(0).expect("tiny0")),
        ("tiny8".to_string(), progs::tiny(8).expect("tiny8")),
        ("tiny64".to_string(), progs::tiny(64).expect("tiny64")),
        ("arith".to_string(), progs::arith(1000).expect("arith")),
        ("consts64".to_string(), progs::consts(64).expect("consts64")),
    ];
    for (name, unit) in units {
        let image = unit.to_image();
        let consts = unit.constants.len();
        let (region, id) = seal_one(&mem, &unit);
        for rep in 0..reps {
            let cal = clock::calibrate(80_000_000);
            if !cal.ok {
                println!("FAIL calibration did not verify");
                return;
            }
            let row = |variant: &str, cell: Option<entry::Cell>| match cell {
                Some(x) => println!(
                    "{name}\t{consts}\t{variant}\t{rep}\t{}\t{}\t{:.4}\t{:.4}\t{:.4}\t{}",
                    x.entries,
                    x.ns,
                    x.per_entry(),
                    cal.ghz,
                    cal.cycles(x.per_entry()),
                    x.acc
                ),
                None => println!("FAIL\t{name}\t{variant}\t{rep}"),
            };
            row("a-table-get", entry::trusted_get(&region, id, n));
            row("a-open-reparse", entry::trusted_open(&region, id, n));
            row("b-cage-recheck", entry::caged_use(&image, n));
            row("b-cage-reparse", entry::caged_parse(&image, n));
            // Reading the constants a job materializes, per unit entry.
            let v = hg_js_bytecode::open(&region, id).expect("open");
            let nc = u32::try_from(consts).unwrap_or(0);
            row("a-const-read", entry::trusted_constants(&v, n / 8));
            row("b-const-read", entry::caged_constants(&image, nc, n / 8));
        }
    }
}

/// `e2e`: the real interpreter, so the deltas above have a denominator.
fn e2e_mode(reps: u32) {
    println!("program\tplace\trep\tjobs\tinstrs_per_job\tns\tns_per_job\tvalue");
    let mem = os_memory();
    // Two loop counts: the slope between them is the interpreter's
    // per-instruction cost with the per-job work cancelled out.
    for n in [1000u32, 2000] {
        let unit = progs::arith(n).expect("arith");
        let per_job = u64::from(n) * 10 + 6;
        for rep in 0..reps {
            for (label, place) in [
                ("trusted", e2e::Where::Trusted),
                ("caged", e2e::Where::Caged),
            ] {
                match e2e::run(&mem, &unit, 20, place) {
                    Some(c) => println!(
                        "arith{n}\t{label}\t{rep}\t{}\t{per_job}\t{}\t{:.1}\t{:?}",
                        c.jobs,
                        c.ns,
                        c.ns as f64 / c.jobs as f64,
                        c.value
                    ),
                    None => println!("FAIL\tarith{n}\t{label}\t{rep}"),
                }
            }
        }
    }
    // Many short jobs: where the per-entry difference shows up end to end.
    for (name, unit) in [
        ("tiny8", progs::tiny(8).expect("tiny8")),
        ("tiny64", progs::tiny(64).expect("tiny64")),
    ] {
        for rep in 0..reps {
            for (label, place) in [
                ("trusted", e2e::Where::Trusted),
                ("caged", e2e::Where::Caged),
            ] {
                match e2e::run(&mem, &unit, 20_000, place) {
                    Some(c) => println!(
                        "{name}\t{label}\t{rep}\t{}\t4\t{}\t{:.1}\t{:?}",
                        c.jobs,
                        c.ns,
                        c.ns as f64 / c.jobs as f64,
                        c.value
                    ),
                    None => println!("FAIL\t{name}\t{label}\t{rep}"),
                }
            }
        }
    }
}

/// `window`: what sealing a unit costs, (a)'s own price.
fn window_mode(reps: u32) {
    println!("unit\timage_bytes\trep\tinserts\tns\tns_per_insert\twindows");
    let mem = os_memory();
    for (name, unit) in programs() {
        let image = unit.to_image();
        for rep in 0..reps {
            let mut region = TrustedRegion::new(Arc::clone(&mem), 64 << 20).expect("region");
            let n = 2000;
            match entry::window(&mut region, &image, n) {
                Some((ns, windows)) => println!(
                    "{name}\t{}\t{rep}\t{n}\t{ns}\t{:.1}\t{windows}",
                    image.len(),
                    ns as f64 / n as f64
                ),
                None => println!("FAIL\t{name}\t{rep}"),
            }
        }
    }
}

/// `seq`: the sequences whose instruction count the note quotes, kept out
/// of line so the disassembly in `results/` can be read against them.
#[inline(never)]
fn seq_trusted_code(u: &Trusted<'_>) -> usize {
    let mut checks = 0;
    black_box(u.code(&mut checks)).map_or(0, <[u8]>::len)
}

#[inline(never)]
fn seq_caged_code(u: &Caged<'_>) -> usize {
    let mut checks = 0;
    black_box(u.code(&mut checks)).map_or(0, <[u8]>::len)
}

#[inline(never)]
fn seq_trusted_table(region: &TrustedRegion, id: TrustedId) -> usize {
    black_box(region.get(black_box(id), TrustedKind::BytecodeUnit)).map_or(0, <[u8]>::len)
}

#[inline(never)]
fn seq_caged_frame(u: &Caged<'_>) -> u32 {
    let mut checks = 0;
    black_box(u.frame_charge(&mut checks)).unwrap_or(0)
}

fn seq_mode() {
    let mem = os_memory();
    let unit = progs::arith(1000).expect("arith");
    let image = unit.to_image();
    let (region, id) = seal_one(&mem, &unit);
    let t = Trusted(hg_js_bytecode::open(&region, id).expect("open"));
    let c = Caged { img: &image };
    println!(
        "seq_trusted_code {} seq_caged_code {} seq_trusted_table {} seq_caged_frame {}",
        seq_trusted_code(black_box(&t)),
        seq_caged_code(black_box(&c)),
        seq_trusted_table(black_box(&region), id),
        seq_caged_frame(black_box(&c))
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("gate");
    let arg = |i: usize, d: u64| -> u64 { args.get(i).and_then(|s| s.parse().ok()).unwrap_or(d) };
    match mode {
        "gate" => {
            if !gate(false) {
                eprintln!("the validity gate failed: no measurement is valid from this build");
                std::process::exit(1);
            }
        }
        "dispatch" => {
            if !gate(true) {
                eprintln!("FAIL gate");
                std::process::exit(1);
            }
            dispatch_mode(arg(2, 4_000_000), arg(3, 5) as u32);
        }
        "entry" => {
            if !gate(true) {
                eprintln!("FAIL gate");
                std::process::exit(1);
            }
            entry_mode(arg(2, 2_000_000), arg(3, 5) as u32);
        }
        "e2e" => {
            if !gate(true) {
                eprintln!("FAIL gate");
                std::process::exit(1);
            }
            e2e_mode(arg(2, 5) as u32);
        }
        "window" => {
            if !gate(true) {
                eprintln!("FAIL gate");
                std::process::exit(1);
            }
            window_mode(arg(2, 5) as u32);
        }
        "seq" => seq_mode(),
        other => {
            eprintln!("unknown mode {other}: gate | dispatch | entry | e2e | window | seq");
            std::process::exit(2);
        }
    }
}
