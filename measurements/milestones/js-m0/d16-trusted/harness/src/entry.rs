//! The call path: what it costs to reach a unit and the three numbers a
//! caller needs from it before it can build a frame — the parameter
//! count, the register count and the canonical frame charge.
//!
//! JS-D16 (a) says the argument count comes from the trusted entry and
//! never from the cage, and prices itself as "one table indirection on
//! calls". (b) must take the same three numbers from the cage and
//! re-check them, because a wrong register count sizes a frame wrongly.

use std::hint::black_box;
use std::time::Instant;

use hg_heap::{TrustedId, TrustedKind, TrustedRegion};
use hg_js_bytecode::UnitView;

use crate::source::{Caged, Source};

/// What one cell of the entry measurement produced.
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    /// Entries measured.
    pub entries: u64,
    /// Nanoseconds for the whole cell.
    pub ns: u64,
    /// The three numbers, accumulated, which every variant must agree on.
    pub acc: u64,
}

impl Cell {
    /// Nanoseconds per entry.
    pub fn per_entry(&self) -> f64 {
        self.ns as f64 / self.entries as f64
    }
}

/// A header word of a trusted image: read, not checked. The bounds check
/// below is safe Rust's, not one the design needs — a production tier
/// reads the trusted entry's fields at fixed offsets — so (a)'s figure is
/// if anything an overstatement.
#[inline]
fn word(img: &[u8], off: usize) -> u32 {
    match img.get(off..off + 4) {
        Some(b) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        None => 0,
    }
}

/// (a): the trusted table indirection, then the three fields.
pub fn trusted_get(region: &TrustedRegion, id: TrustedId, n: u64) -> Option<Cell> {
    let mut acc = 0u64;
    let t0 = Instant::now();
    for _ in 0..n {
        let region = black_box(region);
        let id = black_box(id);
        let img = region.get(id, TrustedKind::BytecodeUnit)?;
        acc = acc
            .wrapping_add(u64::from(word(img, 16)))
            .wrapping_add(u64::from(word(img, 20)))
            .wrapping_add(u64::from(word(img, 40)));
    }
    let ns = t0.elapsed().as_nanos() as u64;
    Some(Cell {
        entries: n,
        ns,
        acc: black_box(acc),
    })
}

/// (a) through the API as it stands: `hg_js_bytecode::open`, which parses
/// the image again on every call.
pub fn trusted_open(region: &TrustedRegion, id: TrustedId, n: u64) -> Option<Cell> {
    let mut acc = 0u64;
    let t0 = Instant::now();
    for _ in 0..n {
        let region = black_box(region);
        let id = black_box(id);
        let v = hg_js_bytecode::open(region, id)?;
        acc = acc
            .wrapping_add(u64::from(v.params()))
            .wrapping_add(u64::from(v.registers()))
            .wrapping_add(v.frame_charge());
    }
    let ns = t0.elapsed().as_nanos() as u64;
    Some(Cell {
        entries: n,
        ns,
        acc: black_box(acc),
    })
}

/// (b) with per-use re-checks: no parse, every field re-read and
/// re-checked against a bound that was itself re-read.
pub fn caged_use(img: &[u8], n: u64) -> Option<Cell> {
    let mut acc = 0u64;
    let mut checks = 0u64;
    let t0 = Instant::now();
    for _ in 0..n {
        let u = Caged {
            img: black_box(img),
        };
        let params = u.params(&mut checks)?;
        let registers = u.registers(&mut checks)?;
        let charge = u.frame_charge(&mut checks)?;
        u.code(&mut checks)?;
        acc = acc
            .wrapping_add(u64::from(params))
            .wrapping_add(u64::from(registers))
            .wrapping_add(u64::from(charge));
    }
    let ns = t0.elapsed().as_nanos() as u64;
    Some(Cell {
        entries: n,
        ns,
        acc: black_box(acc),
    })
}

/// (b) revalidated at entry: the image is parsed again, after which its
/// fields may be used without further checks.
pub fn caged_parse(img: &[u8], n: u64) -> Option<Cell> {
    let mut acc = 0u64;
    let t0 = Instant::now();
    for _ in 0..n {
        let v = UnitView::parse(black_box(img)).ok()?;
        acc = acc
            .wrapping_add(u64::from(v.params()))
            .wrapping_add(u64::from(v.registers()))
            .wrapping_add(v.frame_charge());
    }
    let ns = t0.elapsed().as_nanos() as u64;
    Some(Cell {
        entries: n,
        ns,
        acc: black_box(acc),
    })
}

/// The write window: what it costs to put a verified unit into the
/// trusted region (a commit when needed, two protection flips and the
/// copy). It is (a)'s own cost, paid once per unit, not per call.
pub fn window(region: &mut TrustedRegion, image: &[u8], n: u64) -> Option<(u64, u64)> {
    let before = region.windows();
    let mut ids = Vec::with_capacity(n as usize);
    let t0 = Instant::now();
    for _ in 0..n {
        ids.push(region.insert(TrustedKind::BytecodeUnit, image).ok()?);
    }
    let ns = t0.elapsed().as_nanos() as u64;
    // Every id must still read back as what was written.
    for id in &ids {
        let got = region.get(*id, TrustedKind::BytecodeUnit)?;
        if got != image {
            return None;
        }
    }
    Some((ns, region.windows() - before))
}

/// Reading every constant of a unit: what a job's constant
/// materialization costs on each side. Under (a) the payload bounds were
/// checked when the unit was sealed; under (b) every read re-derives the
/// descriptor table, the literal pool and the payload bounds.
pub fn trusted_constants(v: &UnitView<'_>, n: u64) -> Option<Cell> {
    let mut acc = 0u64;
    let t0 = Instant::now();
    for _ in 0..n {
        let v = black_box(v);
        for i in 0..v.constant_count() {
            acc = acc.wrapping_add(match v.constant(i)? {
                hg_js_bytecode::ConstantRef::Number(x) => x.to_bits(),
                hg_js_bytecode::ConstantRef::String(u) | hg_js_bytecode::ConstantRef::Key(u) => {
                    u.len() as u64
                }
            });
        }
    }
    let ns = t0.elapsed().as_nanos() as u64;
    Some(Cell {
        entries: n,
        ns,
        acc: black_box(acc),
    })
}

/// The same reads under (b).
pub fn caged_constants(img: &[u8], count: u32, n: u64) -> Option<Cell> {
    let mut acc = 0u64;
    let mut checks = 0u64;
    let t0 = Instant::now();
    for _ in 0..n {
        let u = Caged {
            img: black_box(img),
        };
        for i in 0..count {
            let b = u.constant_bytes(i, &mut checks)?;
            acc = acc
                .wrapping_add(b.len() as u64)
                .wrapping_add(u64::from(b[0]));
        }
    }
    let ns = t0.elapsed().as_nanos() as u64;
    Some(Cell {
        entries: n,
        ns,
        acc: black_box(acc),
    })
}
