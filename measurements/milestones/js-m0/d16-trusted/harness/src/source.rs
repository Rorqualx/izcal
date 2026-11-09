//! The two ways an interpreter may reach a unit's operand tables, behind
//! one trait so that both pay exactly the same checks and differ only in
//! how the bytes are derived.
//!
//! - [`Trusted`]: JS-D16 (a). The image lives in the trusted region, so
//!   the one parse done when it was sealed stays valid: the section
//!   bounds, the counts and the kinds are read from a [`UnitView`] that
//!   was checked once.
//! - [`Caged`]: JS-D16 (b). The image lives in the cage, so by H-2
//!   nothing read from it may be trusted and no parse of it may be
//!   cached: every access re-reads the header word it needs, re-checks
//!   its limit and re-checks the section bounds before it reads a byte.
//!
//! What (b) must check, and why that is the minimum (README, "What an
//! honest (b) has to do"): the decoder is total, so a jump that lands
//! mid-instruction is a wrong program and not a memory error, and no
//! re-verification of instruction boundaries is needed. What is needed is
//! that every index the interpreter takes from the cage is re-checked
//! against a bound that was itself re-read from the cage: the code
//! section's offset and length, the parameter and register counts a
//! register operand is resolved against, the constant and slot counts, and
//! the kind bytes.

use hg_js_bytecode::{
    ConstKind, ConstKinds, Instr, Intrinsic, OperandKind, Reg, SlotKind, UnitView,
};
use hg_js_ops::heap::is_smi_range;
use hg_js_ops::support::RegList;

/// Image header offsets (`hg_js_bytecode::unit`'s documented layout).
mod at {
    pub const PARAMS: usize = 16;
    pub const REGISTERS: usize = 20;
    pub const SLOT_COUNT: usize = 24;
    pub const CONST_COUNT: usize = 28;
    pub const CODE_LEN: usize = 36;
    pub const FRAME_CHARGE: usize = 40;
    pub const CODE_OFF: usize = 52;
    pub const SLOTS_OFF: usize = 56;
    pub const CONSTS_OFF: usize = 60;
    pub const LITERALS_OFF: usize = 64;
    pub const LITERALS_LEN: usize = 68;
    pub const HEADER: usize = 88;
}

/// The limits the format sets, re-checked by [`Caged`] on every read.
mod limit {
    pub const PARAMS: u32 = 1 << 16;
    pub const REGISTERS: u32 = 1 << 16;
    pub const CODE: u32 = 1 << 26;
    pub const TABLE: u32 = 1 << 20;
}

/// Where a dispatch loop reads a unit's code and operand tables.
pub trait Source {
    /// The code bytes, or `None` if the section does not lie in the image.
    fn code(&self, checks: &mut u64) -> Option<&[u8]>;
    /// The parameter count.
    fn params(&self, checks: &mut u64) -> Option<u32>;
    /// The local register count.
    fn registers(&self, checks: &mut u64) -> Option<u32>;
    /// Constant `i`'s kind.
    fn constant_kind(&self, i: u32, checks: &mut u64) -> Option<ConstKind>;
    /// Slot `i`'s kind.
    fn slot_kind(&self, i: u32, checks: &mut u64) -> Option<SlotKind>;
    /// The canonical frame charge, as a caller needs it to size a frame.
    fn frame_charge(&self, checks: &mut u64) -> Option<u32>;
}

/// JS-D16 (a): a unit parsed once, in the trusted region.
#[derive(Debug, Clone, Copy)]
pub struct Trusted<'a>(pub UnitView<'a>);

impl Source for Trusted<'_> {
    #[inline]
    fn code(&self, _checks: &mut u64) -> Option<&[u8]> {
        Some(self.0.code())
    }
    #[inline]
    fn params(&self, _checks: &mut u64) -> Option<u32> {
        Some(self.0.params())
    }
    #[inline]
    fn registers(&self, _checks: &mut u64) -> Option<u32> {
        Some(self.0.registers())
    }
    #[inline]
    fn constant_kind(&self, i: u32, _checks: &mut u64) -> Option<ConstKind> {
        self.0.constant_kind(i)
    }
    #[inline]
    fn slot_kind(&self, i: u32, _checks: &mut u64) -> Option<SlotKind> {
        self.0.slot_kind(i)
    }
    #[inline]
    fn frame_charge(&self, _checks: &mut u64) -> Option<u32> {
        u32::try_from(self.0.frame_charge()).ok()
    }
}

/// JS-D16 (b): a unit image in the cage, re-derived on every access.
#[derive(Debug, Clone, Copy)]
pub struct Caged<'a> {
    /// The image bytes as the cage holds them.
    pub img: &'a [u8],
}

/// One little-endian u32 of the header, bounds-checked.
#[inline]
fn word(img: &[u8], off: usize) -> Option<u32> {
    let b = img.get(off..off.checked_add(4)?)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// A section of the image, with every bound re-read and re-checked.
#[inline]
fn section(img: &[u8], off: u32, len: u32) -> Option<&[u8]> {
    let off = usize::try_from(off).ok()?;
    let len = usize::try_from(len).ok()?;
    if off < at::HEADER {
        return None;
    }
    img.get(off..off.checked_add(len)?)
}

impl Caged<'_> {
    /// The constant descriptor table, re-derived.
    #[inline]
    fn consts(&self, checks: &mut u64) -> Option<&[u8]> {
        *checks += 1;
        let n = word(self.img, at::CONST_COUNT)?;
        if n > limit::TABLE {
            return None;
        }
        section(self.img, word(self.img, at::CONSTS_OFF)?, n.checked_mul(8)?)
    }

    /// The literal pool, re-derived.
    #[inline]
    fn literals(&self, checks: &mut u64) -> Option<&[u8]> {
        *checks += 1;
        section(
            self.img,
            word(self.img, at::LITERALS_OFF)?,
            word(self.img, at::LITERALS_LEN)?,
        )
    }

    /// Constant `i`'s payload, re-derived and re-checked: what reading a
    /// constant (not only admitting its operand) costs under (b).
    pub fn constant_bytes(&self, i: u32, checks: &mut u64) -> Option<&[u8]> {
        let d = self.consts(checks)?;
        let at = usize::try_from(i).ok()?.checked_mul(8)?;
        let d = d.get(at..at.checked_add(8)?)?;
        let kind = ConstKind::from_byte(*d.first()?)?;
        let off = usize::try_from(u32::from_le_bytes([d[4], d[5], d[6], d[7]])).ok()?;
        let lit = self.literals(checks)?;
        match kind {
            ConstKind::Number => lit.get(off..off.checked_add(8)?),
            ConstKind::String | ConstKind::Key => {
                let lb = lit.get(off..off.checked_add(4)?)?;
                let n = usize::try_from(u32::from_le_bytes([lb[0], lb[1], lb[2], lb[3]])).ok()?;
                let start = off.checked_add(4)?;
                lit.get(start..start.checked_add(n.checked_mul(2)?)?)
            }
        }
    }
}

impl Source for Caged<'_> {
    #[inline]
    fn code(&self, checks: &mut u64) -> Option<&[u8]> {
        *checks += 1;
        let len = word(self.img, at::CODE_LEN)?;
        if len > limit::CODE {
            return None;
        }
        section(self.img, word(self.img, at::CODE_OFF)?, len)
    }

    #[inline]
    fn params(&self, checks: &mut u64) -> Option<u32> {
        *checks += 1;
        let n = word(self.img, at::PARAMS)?;
        (n <= limit::PARAMS).then_some(n)
    }

    #[inline]
    fn registers(&self, checks: &mut u64) -> Option<u32> {
        *checks += 1;
        let n = word(self.img, at::REGISTERS)?;
        (n <= limit::REGISTERS).then_some(n)
    }

    #[inline]
    fn constant_kind(&self, i: u32, checks: &mut u64) -> Option<ConstKind> {
        let d = self.consts(checks)?;
        let at = usize::try_from(i).ok()?.checked_mul(8)?;
        ConstKind::from_byte(*d.get(at)?)
    }

    #[inline]
    fn slot_kind(&self, i: u32, checks: &mut u64) -> Option<SlotKind> {
        *checks += 1;
        let n = word(self.img, at::SLOT_COUNT)?;
        if n > limit::TABLE {
            return None;
        }
        let s = section(self.img, word(self.img, at::SLOTS_OFF)?, n)?;
        SlotKind::from_byte(*s.get(usize::try_from(i).ok()?)?)
    }

    #[inline]
    fn frame_charge(&self, checks: &mut u64) -> Option<u32> {
        *checks += 1;
        let charge = word(self.img, at::FRAME_CHARGE)?;
        let params = self.params(checks)?;
        let registers = self.registers(checks)?;
        // The charge is what a caller sizes a frame with, so it is
        // re-checked against the counts it must follow.
        let want = 32u64 + 4 * (u64::from(registers) + u64::from(params));
        (u64::from(charge) == want).then_some(charge)
    }
}

/// Whether instruction `i`'s operands are admissible, read from `u`.
///
/// This is the reference interpreter's own per-instruction check
/// (`hg_js_ops::support::Cx::admits`), expressed over [`Source`] so that
/// the trusted and caged paths run the same checks in the same order. Each
/// variant of the measurement chooses whether to call it: under (a) a
/// production tier does not (the verifier's result stays valid, JS-D16),
/// under (b) every tier must.
///
/// `COUNT` adds the check counter the validity gate reads; the timed runs
/// are built with it off, so no cell of the measurement pays for it.
#[inline]
pub fn admits<S: Source, const COUNT: bool>(
    i: &Instr,
    u: &S,
    params: u32,
    registers: u32,
    checks: &mut u64,
) -> bool {
    let kinds = i.op.info().operands;
    let Some(code) = u.code(checks) else {
        return false;
    };
    let code_len = u32::try_from(code.len()).unwrap_or(u32::MAX);
    let in_frame = |r: Reg| -> bool { r.resolve(params, registers).is_some() };
    for (j, k) in kinds.iter().enumerate() {
        if COUNT {
            *checks += 1;
        }
        let ok = match *k {
            OperandKind::Reg => {
                if kinds.get(j + 1) == Some(&OperandKind::Cnt) {
                    let n = i.index(j + 1);
                    let base = i.reg(j);
                    n == 0
                        || (in_frame(base)
                            && in_frame(RegList::new(base, n).at(n - 1))
                            && base.0 >= 0)
                } else {
                    in_frame(i.reg(j))
                }
            }
            OperandKind::Cnt => true,
            OperandKind::Imm => is_smi_range(i.operand(j)),
            OperandKind::Const(want) => u
                .constant_kind(i.index(j), checks)
                .is_some_and(|c| ConstKinds::contains(want, c)),
            OperandKind::Slot(want) => u.slot_kind(i.index(j), checks) == Some(want),
            OperandKind::Fwd => i.forward_target(j) < code_len && i.index(j) > 0,
            OperandKind::Back => i.backward_target(j) < code_len,
            OperandKind::Intr => match Intrinsic::from_index(i.index(j)) {
                Some(x) => {
                    kinds.get(j + 2) == Some(&OperandKind::Cnt) && i.index(j + 2) == x.arity()
                }
                None => false,
            },
        };
        if !ok {
            return false;
        }
    }
    true
}
