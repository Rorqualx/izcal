//! The dispatch loop, as the reference interpreter writes it
//! (`hg_js_ops::generated::interp::run`): decode the instruction at `pc`,
//! admit its operands, then take the next `pc` from the flow. The
//! operation's semantics are left out — they are identical under (a) and
//! (b) and would hide the difference being measured — and replaced by a
//! digest of the decoded instruction, which every variant must agree on.
//!
//! Two shapes, because dispatch has both: one stream is the critical path
//! (each `pc` depends on the instruction before it), and several
//! interleaved streams are the pipelined shape (independent dispatches
//! that a core may overlap, which is what a production tier's unrolled
//! handlers look like from the memory system's side).

use std::hint::black_box;
use std::time::Instant;

use hg_js_bytecode::{Instr, Op, decode};

use crate::source::{Source, admits};

/// What one cell of the dispatch measurement produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// Instructions dispatched.
    pub instrs: u64,
    /// The digest every variant must agree on.
    pub digest: u64,
    /// Operand and section checks executed (0 when counting is off).
    pub checks: u64,
    /// Nanoseconds for the whole cell.
    pub ns: u64,
}

impl Cell {
    /// Nanoseconds per dispatched instruction.
    pub fn per_instr(&self) -> f64 {
        self.ns as f64 / self.instrs as f64
    }
}

/// The digest of one decoded instruction: identical under every variant,
/// so a variant that read the wrong bytes cannot agree by accident.
#[inline]
fn mix(d: u64, i: &Instr) -> u64 {
    // Cheap on purpose: a digest with a long dependency chain of its own
    // would dilute the difference being measured. One rotate, one
    // multiply and five xors is enough that a variant which read the
    // wrong byte cannot agree over twenty thousand instructions.
    d.rotate_left(13).wrapping_mul(0x2545_F491_4F6C_DD1D)
        ^ u64::from(i.op.byte())
        ^ (i.operand(0) as u64)
        ^ ((i.operand(1) as u64) << 13)
        ^ ((i.operand(2) as u64) << 27)
        ^ ((i.operand(3) as u64) << 41)
        ^ (u64::from(i.end) << 51)
}

/// The next `pc`, with the branch direction taken from a stream state
/// that does not depend on which variant is running, so every variant
/// walks the same path.
#[inline]
fn next_pc(i: &Instr, lcg: u64) -> u32 {
    match i.op {
        Op::Jump => i.forward_target(0),
        Op::LoopBack => i.backward_target(0),
        Op::BranchTrue | Op::BranchFalse | Op::BranchTruthy | Op::BranchFalsy => {
            if lcg >> 62 == 0 {
                i.forward_target(0)
            } else {
                i.end
            }
        }
        Op::Return => 0,
        _ => i.end,
    }
}

/// Dispatches `steps` × `K` instructions from `u`.
///
/// `CHECK` is the H-2 operand re-check: (b) must run it on every tier,
/// (a) runs it only in the reference interpreter. `COUNT` adds the
/// gate's check counter, and is off in every timed cell.
///
/// Returns `None` if anything was inadmissible, which the gate treats as
/// a failure rather than as a measurement.
pub fn run<S: Source, const K: usize, const CHECK: bool, const COUNT: bool>(
    u: &S,
    steps: u64,
) -> Option<Cell> {
    let mut pc = [0u32; K];
    let mut digest = [0u64; K];
    let mut lcg = [0u64; K];
    for l in lcg.iter_mut() {
        *l = 0x2545_F491_4F6C_DD1D;
    }
    let mut checks = 0u64;
    let mut instrs = 0u64;
    // One warm pass, so the first timed instruction is not the one that
    // faults the image in.
    for k in 0..K {
        let code = u.code(&mut checks)?;
        let _ = decode(code, pc[k]).ok()?;
    }
    checks = 0;
    let t0 = Instant::now();
    for _ in 0..steps {
        for k in 0..K {
            // The source is opaque at every instruction, so neither
            // variant may hoist its loads out of the loop: (a) re-loads
            // the code pointer, (b) re-derives and re-checks. Both pay
            // the same opacity.
            let u = black_box(u);
            let code = u.code(&mut checks)?;
            let i = decode(code, pc[k]).ok()?;
            if CHECK {
                // Under (b) the frame's counts are in the cage too, so
                // the bound a register operand is resolved against is
                // re-read per instruction; under (a) they are trusted.
                let params = u.params(&mut checks)?;
                let registers = u.registers(&mut checks)?;
                if !admits::<S, COUNT>(&i, u, params, registers, &mut checks) {
                    return None;
                }
            }
            digest[k] = mix(digest[k], &i);
            lcg[k] = lcg[k]
                .wrapping_mul(0x5851_F42D_4C95_7F2D)
                .wrapping_add(0x1405_7B7E_F767_814F);
            pc[k] = next_pc(&i, lcg[k]);
            instrs += 1;
        }
    }
    let ns = t0.elapsed().as_nanos() as u64;
    let mut d = 0u64;
    for k in 0..K {
        d = d.rotate_left(17) ^ black_box(digest[k]);
    }
    Some(Cell {
        instrs,
        digest: d,
        checks,
        ns,
    })
}

/// The op histogram of the path one stream walks, for the report: the mix
/// the numbers above were measured on.
pub fn histogram<S: Source>(u: &S, steps: u64) -> Vec<(&'static str, u64)> {
    let mut counts = vec![0u64; 256];
    let mut checks = 0u64;
    let mut pc = 0u32;
    let mut lcg = 0x2545_F491_4F6C_DD1Du64;
    for _ in 0..steps {
        let Some(code) = u.code(&mut checks) else {
            break;
        };
        let Ok(i) = decode(code, pc) else { break };
        if let Some(c) = counts.get_mut(usize::from(i.op.byte())) {
            *c += 1;
        }
        lcg = lcg
            .wrapping_mul(0x5851_F42D_4C95_7F2D)
            .wrapping_add(0x1405_7B7E_F767_814F);
        pc = next_pc(&i, lcg);
    }
    let mut out = Vec::new();
    for op in Op::ALL {
        let n = counts.get(usize::from(op.byte())).copied().unwrap_or(0);
        if n > 0 {
            out.push((op.info().mnemonic, n));
        }
    }
    out
}

/// The cheaper sound (b): bytecode in the cage, re-validated whole at
/// every point where script could have run and written it — a unit entry
/// and every instruction whose operation may call user code
/// (`Op::info().may_call_user`) — and trusted in between. It is sound only
/// under a cage no other thread writes; JS-D17 puts every agent of a
/// process in one cage, so an agent on another thread invalidates it
/// (README, "What an honest (b) has to do").
pub fn run_revalidated<const K: usize>(img: &[u8], steps: u64) -> Option<Cell> {
    let mut pc = [0u32; K];
    let mut digest = [0u64; K];
    let mut lcg = [0u64; K];
    for l in lcg.iter_mut() {
        *l = 0x2545_F491_4F6C_DD1D;
    }
    // The last validation's result, dropped whenever script could have
    // run since it was made.
    let mut view = [None::<hg_js_bytecode::UnitView<'_>>; K];
    let mut instrs = 0u64;
    let mut parses = 0u64;
    let t0 = Instant::now();
    for _ in 0..steps {
        for k in 0..K {
            let img = black_box(img);
            let v = match view[k] {
                Some(v) => v,
                None => {
                    // The revalidation: the whole image is parsed again,
                    // which is what makes its operands trustworthy for
                    // the stretch that follows.
                    parses += 1;
                    let v = hg_js_bytecode::UnitView::parse(img).ok()?;
                    view[k] = Some(v);
                    v
                }
            };
            let i = decode(v.code(), pc[k]).ok()?;
            digest[k] = mix(digest[k], &i);
            lcg[k] = lcg[k]
                .wrapping_mul(0x5851_F42D_4C95_7F2D)
                .wrapping_add(0x1405_7B7E_F767_814F);
            pc[k] = next_pc(&i, lcg[k]);
            if i.op.info().may_call_user || i.op == Op::Return {
                view[k] = None;
            }
            instrs += 1;
        }
    }
    let ns = t0.elapsed().as_nanos() as u64;
    let mut d = 0u64;
    for k in 0..K {
        d = d.rotate_left(17) ^ black_box(digest[k]);
    }
    Some(Cell {
        instrs,
        digest: d,
        checks: parses,
        ns,
    })
}

/// A diagnostic, not a variant of JS-D16: (a)'s loop with `pad`
/// dependent ALU operations added per instruction, reading nothing. It
/// answers the question the (a)-versus-(b) rows raise at one stream —
/// whether the loop has slack in which extra work is free — by adding
/// work that cannot possibly be cheaper than none.
pub fn run_padded<S: Source, const K: usize>(u: &S, steps: u64, pad: u32) -> Option<Cell> {
    let mut pc = [0u32; K];
    let mut digest = [0u64; K];
    let mut lcg = [0u64; K];
    for l in lcg.iter_mut() {
        *l = 0x2545_F491_4F6C_DD1D;
    }
    let mut checks = 0u64;
    let mut instrs = 0u64;
    let mut side = 1u64;
    let t0 = Instant::now();
    for _ in 0..steps {
        for k in 0..K {
            let u = black_box(u);
            let code = u.code(&mut checks)?;
            let i = decode(code, pc[k]).ok()?;
            for _ in 0..pad {
                side = side.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(5)
                    ^ u64::from(i.op.byte());
            }
            digest[k] = mix(digest[k], &i);
            lcg[k] = lcg[k]
                .wrapping_mul(0x5851_F42D_4C95_7F2D)
                .wrapping_add(0x1405_7B7E_F767_814F);
            pc[k] = next_pc(&i, lcg[k]);
            instrs += 1;
        }
    }
    let ns = t0.elapsed().as_nanos() as u64;
    let mut d = 0u64;
    for k in 0..K {
        d = d.rotate_left(17) ^ black_box(digest[k]);
    }
    black_box(side);
    Some(Cell {
        instrs,
        digest: d,
        checks,
        ns,
    })
}
