//! Decoding one instruction (bytecode note 4.1): an optional scaling
//! prefix, an opcode byte, then the operands, each 1, 2 or 4 bytes as the
//! prefix says, little-endian and unaligned. Every read is bounds-checked;
//! any bytes give an instruction or an error, never a panic.

use crate::{EXTENDED, INVALID, Op, OperandKind, Reg, WIDE2, WIDE4};

/// A decoded instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instr {
    /// The opcode.
    pub op: Op,
    /// The operand width in bytes: 1, 2 or 4.
    pub scale: u8,
    /// The offset of its first byte (the prefix, if any).
    pub start: u32,
    /// The offset just past it.
    pub end: u32,
    operands: [i64; 4],
}

/// Why bytes do not decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// The offset is at or past the end of the code.
    End,
    /// `Invalid` (0x00) or an unassigned opcode byte.
    Invalid(u8),
    /// The reserved `Extended` prefix.
    Reserved,
    /// A prefix followed by a prefix, or by an opcode without operands.
    BadPrefix,
    /// The instruction runs past the end of the code.
    Truncated,
}

/// Decodes the instruction at `pc` in `code`.
///
/// # Errors
/// [`DecodeError`] for bytes that are not an instruction.
pub fn decode(code: &[u8], pc: u32) -> Result<Instr, DecodeError> {
    let at = usize::try_from(pc).map_err(|_| DecodeError::End)?;
    let first = *code.get(at).ok_or(DecodeError::End)?;
    let (scale, op_at) = match first {
        WIDE2 => (2u8, at.checked_add(1).ok_or(DecodeError::Truncated)?),
        WIDE4 => (4u8, at.checked_add(1).ok_or(DecodeError::Truncated)?),
        _ => (1u8, at),
    };
    let byte = *code.get(op_at).ok_or(DecodeError::Truncated)?;
    let op = match byte {
        WIDE2 | WIDE4 => return Err(DecodeError::BadPrefix),
        EXTENDED => return Err(DecodeError::Reserved),
        INVALID => return Err(DecodeError::Invalid(byte)),
        b => Op::from_byte(b).ok_or(DecodeError::Invalid(b))?,
    };
    let kinds = op.info().operands;
    if scale > 1 && kinds.is_empty() {
        return Err(DecodeError::BadPrefix);
    }
    let mut operands = [0i64; 4];
    let mut pos = op_at.checked_add(1).ok_or(DecodeError::Truncated)?;
    let width = usize::from(scale);
    for (i, kind) in kinds.iter().enumerate() {
        let end = pos.checked_add(width).ok_or(DecodeError::Truncated)?;
        let bytes = code.get(pos..end).ok_or(DecodeError::Truncated)?;
        let v = read(bytes, kind.signed());
        if let Some(slot) = operands.get_mut(i) {
            *slot = v;
        }
        pos = end;
    }
    let end = u32::try_from(pos).map_err(|_| DecodeError::Truncated)?;
    Ok(Instr {
        op,
        scale,
        start: pc,
        end,
        operands,
    })
}

fn read(bytes: &[u8], signed: bool) -> i64 {
    let mut u: u64 = 0;
    for (i, b) in bytes.iter().enumerate() {
        u |= u64::from(*b) << (8 * i);
    }
    if !signed {
        return i64::try_from(u).unwrap_or(0);
    }
    let bits = 8 * bytes.len();
    // Sign-extend from `bits`.
    let shift = 64 - bits;
    (i64::from_ne_bytes(u.to_ne_bytes()) << shift) >> shift
}

impl Instr {
    /// The number of operands.
    pub fn count(&self) -> usize {
        self.op.info().operands.len()
    }

    /// Operand `i`'s value (0 past the last).
    pub fn operand(&self, i: usize) -> i64 {
        self.operands.get(i).copied().unwrap_or(0)
    }

    /// Operand `i` as a register.
    pub fn reg(&self, i: usize) -> Reg {
        Reg(i32::try_from(self.operand(i)).unwrap_or(i32::MIN))
    }

    /// Operand `i` as an immediate.
    pub fn imm(&self, i: usize) -> i32 {
        i32::try_from(self.operand(i)).unwrap_or(0)
    }

    /// Operand `i` as an unsigned index (a count, a constant, a slot or
    /// an intrinsic id).
    pub fn index(&self, i: usize) -> u32 {
        u32::try_from(self.operand(i)).unwrap_or(u32::MAX)
    }

    /// The target of forward jump operand `i`: the distance from the
    /// instruction's first byte, its prefix included (bytecode note 4.7).
    /// `u32::MAX` if it overflows.
    pub fn forward_target(&self, i: usize) -> u32 {
        self.start.saturating_add(self.index(i))
    }

    /// The target of backward jump operand `i`; `u32::MAX` if it would be
    /// before the code.
    pub fn backward_target(&self, i: usize) -> u32 {
        self.start.checked_sub(self.index(i)).unwrap_or(u32::MAX)
    }

    /// The kind of operand `i`.
    pub fn kind(&self, i: usize) -> Option<OperandKind> {
        self.op.info().operands.get(i).copied()
    }
}
