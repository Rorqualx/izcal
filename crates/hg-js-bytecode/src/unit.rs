//! The bytecode unit (bytecode note 4.8, R0-B7) and its image form: a
//! fixed little-endian header, then sections located by offsets from the
//! image start. Nothing in it is a pointer or a heap reference. Parsing
//! treats the bytes as hostile: every count is limited and every offset
//! and length is checked with checked arithmetic (G18), so any bytes give
//! a [`UnitView`] or an [`ImageError`], never an out-of-bounds read.
//!
//! ```text
//!  0 magic "IZBU"          4 format id (u64)       12 flags (bit 0 strict, bits 1-2 kind)
//! 16 params               20 registers            24 slot count
//! 28 constant count       32 handler count (0)    36 code length
//! 40 frame charge         44 name offset          48 name length
//! 52 code offset          56 slots offset         60 constants offset
//! 64 literals offset      68 literals length      72 handlers offset
//! 76 positions offset     80 positions length     84 total length
//! 88 end of header
//! ```
//! A constant descriptor is 8 bytes: the kind byte, three zero bytes and
//! the offset of its payload in the literal pool (a number's 8 bytes, or
//! a string's u32 length followed by its UTF-16 code units).

use hg_heap::{TrustedError, TrustedId, TrustedKind, TrustedRegion};

use crate::{ConstKind, FORMAT_ID, SlotKind};

const MAGIC: [u8; 4] = *b"IZBU";
const HEADER: usize = 88;

/// Limits a unit's counts must keep (checked when parsing an image).
pub const MAX_PARAMS: u32 = 1 << 16;
/// The most locals a unit may have.
pub const MAX_REGISTERS: u32 = 1 << 16;
/// The longest code a unit may have.
pub const MAX_CODE: u32 = 1 << 26;
/// The most slots or constants a unit may have.
pub const MAX_TABLE: u32 = 1 << 20;

/// What kind of code a unit holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    /// A script's top level.
    Script,
    /// A function.
    Function,
}

/// A constant of a unit (bytecode note 4.5): its kind and literal. The
/// values are materialized per agent into a constant array in the cage.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    /// A Number.
    Number(f64),
    /// A String, as UTF-16 code units.
    String(Vec<u16>),
    /// A property key (interned when materialized), as UTF-16 code units.
    Key(Vec<u16>),
}

impl Constant {
    /// The constant's kind.
    pub fn kind(&self) -> ConstKind {
        match self {
            Self::Number(_) => ConstKind::Number,
            Self::String(_) => ConstKind::String,
            Self::Key(_) => ConstKind::Key,
        }
    }
}

/// One function's bytecode, as the assembler (and at JS-M1 the emitter)
/// makes it.
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    /// The function's name, for the disassembler.
    pub name: String,
    /// Script or function.
    pub kind: UnitKind,
    /// Strict mode (decides whether a failed `[[Set]]` throws).
    pub strict: bool,
    /// The parameter count.
    pub params: u32,
    /// The local register count.
    pub registers: u32,
    /// The instruction bytes.
    pub code: Vec<u8>,
    /// One kind per IC slot.
    pub slots: Vec<SlotKind>,
    /// The constant table.
    pub constants: Vec<Constant>,
}

/// The canonical frame charge's static part (bytecode note 4.8, R2-16):
/// `32 + 4 × (registers + params)` canonical depth bytes, the same on
/// every ISA and tier.
pub fn frame_charge(params: u32, registers: u32) -> u64 {
    32 + 4 * (u64::from(registers) + u64::from(params))
}

fn put32(out: &mut [u8], at: usize, v: u32) {
    if let Some(dst) = out.get_mut(at..at + 4) {
        dst.copy_from_slice(&v.to_le_bytes());
    }
}

fn len32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl Unit {
    /// The canonical frame charge of one activation.
    pub fn frame_charge(&self) -> u64 {
        frame_charge(self.params, self.registers)
    }

    /// The image form: a pointer-free byte string.
    pub fn to_image(&self) -> Vec<u8> {
        let mut literals = Vec::new();
        let mut descs = Vec::new();
        for c in &self.constants {
            let off = len32(literals.len());
            match c {
                Constant::Number(f) => literals.extend_from_slice(&f.to_bits().to_le_bytes()),
                Constant::String(u) | Constant::Key(u) => {
                    literals.extend_from_slice(&len32(u.len()).to_le_bytes());
                    for cu in u {
                        literals.extend_from_slice(&cu.to_le_bytes());
                    }
                }
            }
            descs.extend_from_slice(&[c.kind().byte(), 0, 0, 0]);
            descs.extend_from_slice(&off.to_le_bytes());
        }
        let name = self.name.as_bytes();
        let mut out = vec![0u8; HEADER];
        let name_off = out.len();
        out.extend_from_slice(name);
        let code_off = out.len();
        out.extend_from_slice(&self.code);
        let slots_off = out.len();
        out.extend(self.slots.iter().map(|k| k.byte()));
        while !out.len().is_multiple_of(4) {
            out.push(0);
        }
        let consts_off = out.len();
        out.extend_from_slice(&descs);
        let lit_off = out.len();
        out.extend_from_slice(&literals);
        let end = out.len();
        out[0..4].copy_from_slice(&MAGIC);
        if let Some(d) = out.get_mut(4..12) {
            d.copy_from_slice(&FORMAT_ID.to_le_bytes());
        }
        let kind_bits = match self.kind {
            UnitKind::Script => 0,
            UnitKind::Function => 2,
        };
        put32(&mut out, 12, u32::from(self.strict) | kind_bits);
        put32(&mut out, 16, self.params);
        put32(&mut out, 20, self.registers);
        put32(&mut out, 24, len32(self.slots.len()));
        put32(&mut out, 28, len32(self.constants.len()));
        put32(&mut out, 32, 0);
        put32(&mut out, 36, len32(self.code.len()));
        put32(
            &mut out,
            40,
            u32::try_from(self.frame_charge()).unwrap_or(u32::MAX),
        );
        put32(&mut out, 44, len32(name_off));
        put32(&mut out, 48, len32(name.len()));
        put32(&mut out, 52, len32(code_off));
        put32(&mut out, 56, len32(slots_off));
        put32(&mut out, 60, len32(consts_off));
        put32(&mut out, 64, len32(lit_off));
        put32(&mut out, 68, len32(literals.len()));
        put32(&mut out, 72, len32(end));
        put32(&mut out, 76, len32(end));
        put32(&mut out, 80, 0);
        put32(&mut out, 84, len32(end));
        out
    }
}

/// Why an image does not parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageError {
    /// Shorter than its header, or its total length disagrees.
    Truncated,
    /// Not a bytecode unit.
    BadMagic,
    /// Made for another opcode table (bytecode note 4.1).
    WrongFormat,
    /// A count exceeds its limit.
    TooLarge,
    /// A section lies outside the image.
    BadSection,
    /// A slot or constant kind byte is not a kind.
    BadKind,
    /// The frame charge disagrees with the counts.
    BadFrameCharge,
    /// The handler table is not empty (JS-M0 has no handlers).
    Handlers,
}

/// A parsed image: borrowed bytes with checked section bounds.
#[derive(Debug, Clone, Copy)]
pub struct UnitView<'a> {
    name: &'a [u8],
    code: &'a [u8],
    slots: &'a [u8],
    consts: &'a [u8],
    literals: &'a [u8],
    strict: bool,
    kind: UnitKind,
    params: u32,
    registers: u32,
}

fn get32(b: &[u8], at: usize) -> Result<u32, ImageError> {
    let s = b.get(at..at + 4).ok_or(ImageError::Truncated)?;
    let mut a = [0u8; 4];
    a.copy_from_slice(s);
    Ok(u32::from_le_bytes(a))
}

fn section(b: &[u8], off: u32, len: u32) -> Result<&[u8], ImageError> {
    let off = usize::try_from(off).map_err(|_| ImageError::BadSection)?;
    let len = usize::try_from(len).map_err(|_| ImageError::BadSection)?;
    let end = off.checked_add(len).ok_or(ImageError::BadSection)?;
    if off < HEADER {
        return Err(ImageError::BadSection);
    }
    b.get(off..end).ok_or(ImageError::BadSection)
}

impl<'a> UnitView<'a> {
    /// Parses `bytes` as a unit image (hostile input, G18).
    ///
    /// # Errors
    /// [`ImageError`] for anything that is not a well-formed image of this
    /// build's opcode table.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, ImageError> {
        if bytes.len() < HEADER {
            return Err(ImageError::Truncated);
        }
        if bytes.get(0..4) != Some(&MAGIC[..]) {
            return Err(ImageError::BadMagic);
        }
        let mut fid = [0u8; 8];
        fid.copy_from_slice(bytes.get(4..12).ok_or(ImageError::Truncated)?);
        if u64::from_le_bytes(fid) != FORMAT_ID {
            return Err(ImageError::WrongFormat);
        }
        let flags = get32(bytes, 12)?;
        let params = get32(bytes, 16)?;
        let registers = get32(bytes, 20)?;
        let nslots = get32(bytes, 24)?;
        let nconsts = get32(bytes, 28)?;
        let handlers = get32(bytes, 32)?;
        let code_len = get32(bytes, 36)?;
        let charge = get32(bytes, 40)?;
        if params > MAX_PARAMS
            || registers > MAX_REGISTERS
            || code_len > MAX_CODE
            || nslots > MAX_TABLE
            || nconsts > MAX_TABLE
        {
            return Err(ImageError::TooLarge);
        }
        if handlers != 0 {
            return Err(ImageError::Handlers);
        }
        if u64::from(charge) != frame_charge(params, registers) {
            return Err(ImageError::BadFrameCharge);
        }
        let total = get32(bytes, 84)?;
        if usize::try_from(total).ok() != Some(bytes.len()) {
            return Err(ImageError::Truncated);
        }
        let name = section(bytes, get32(bytes, 44)?, get32(bytes, 48)?)?;
        let code = section(bytes, get32(bytes, 52)?, code_len)?;
        let slots = section(bytes, get32(bytes, 56)?, nslots)?;
        let desc_len = nconsts.checked_mul(8).ok_or(ImageError::TooLarge)?;
        let consts = section(bytes, get32(bytes, 60)?, desc_len)?;
        let literals = section(bytes, get32(bytes, 64)?, get32(bytes, 68)?)?;
        if slots.iter().any(|b| SlotKind::from_byte(*b).is_none()) {
            return Err(ImageError::BadKind);
        }
        let kind = if flags & 6 == 2 {
            UnitKind::Function
        } else {
            UnitKind::Script
        };
        let v = Self {
            name,
            code,
            slots,
            consts,
            literals,
            strict: flags & 1 == 1,
            kind,
            params,
            registers,
        };
        for i in 0..nconsts {
            v.constant(i).ok_or(ImageError::BadKind)?;
        }
        Ok(v)
    }

    /// The instruction bytes.
    pub fn code(&self) -> &'a [u8] {
        self.code
    }

    /// The function's name (empty if not UTF-8).
    pub fn name(&self) -> &'a str {
        core::str::from_utf8(self.name).unwrap_or("")
    }

    /// Strict mode.
    pub fn strict(&self) -> bool {
        self.strict
    }

    /// Script or function.
    pub fn kind(&self) -> UnitKind {
        self.kind
    }

    /// The parameter count.
    pub fn params(&self) -> u32 {
        self.params
    }

    /// The local register count.
    pub fn registers(&self) -> u32 {
        self.registers
    }

    /// The canonical frame charge.
    pub fn frame_charge(&self) -> u64 {
        frame_charge(self.params, self.registers)
    }

    /// The number of IC slots.
    pub fn slot_count(&self) -> u32 {
        len32(self.slots.len())
    }

    /// Slot `i`'s kind.
    pub fn slot_kind(&self, i: u32) -> Option<SlotKind> {
        let i = usize::try_from(i).ok()?;
        SlotKind::from_byte(*self.slots.get(i)?)
    }

    /// The number of constants.
    pub fn constant_count(&self) -> u32 {
        len32(self.consts.len() / 8)
    }

    /// Constant `i`, if its descriptor and payload are well formed.
    pub fn constant(&self, i: u32) -> Option<ConstantRef<'a>> {
        let at = usize::try_from(i).ok()?.checked_mul(8)?;
        let d = self.consts.get(at..at.checked_add(8)?)?;
        let kind = ConstKind::from_byte(*d.first()?)?;
        let mut o = [0u8; 4];
        o.copy_from_slice(d.get(4..8)?);
        let off = usize::try_from(u32::from_le_bytes(o)).ok()?;
        match kind {
            ConstKind::Number => {
                let b = self.literals.get(off..off.checked_add(8)?)?;
                let mut a = [0u8; 8];
                a.copy_from_slice(b);
                Some(ConstantRef::Number(f64::from_bits(u64::from_le_bytes(a))))
            }
            ConstKind::String | ConstKind::Key => {
                let lb = self.literals.get(off..off.checked_add(4)?)?;
                let mut a = [0u8; 4];
                a.copy_from_slice(lb);
                let n = usize::try_from(u32::from_le_bytes(a)).ok()?;
                let start = off.checked_add(4)?;
                let end = start.checked_add(n.checked_mul(2)?)?;
                let units = Utf16(self.literals.get(start..end)?);
                Some(if kind == ConstKind::Key {
                    ConstantRef::Key(units)
                } else {
                    ConstantRef::String(units)
                })
            }
        }
    }

    /// The constant kind of constant `i`.
    pub fn constant_kind(&self, i: u32) -> Option<ConstKind> {
        self.constant(i).map(|c| c.kind())
    }

    /// An owned copy.
    pub fn to_unit(self) -> Unit {
        Unit {
            name: self.name().to_string(),
            kind: self.kind,
            strict: self.strict,
            params: self.params,
            registers: self.registers,
            code: self.code.to_vec(),
            slots: (0..self.slot_count())
                .filter_map(|i| self.slot_kind(i))
                .collect(),
            constants: (0..self.constant_count())
                .filter_map(|i| self.constant(i))
                .map(|c| match c {
                    ConstantRef::Number(f) => Constant::Number(f),
                    ConstantRef::String(u) => Constant::String(u.iter().collect()),
                    ConstantRef::Key(u) => Constant::Key(u.iter().collect()),
                })
                .collect(),
        }
    }
}

/// A constant as an image holds it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstantRef<'a> {
    /// A Number.
    Number(f64),
    /// A String.
    String(Utf16<'a>),
    /// A property key.
    Key(Utf16<'a>),
}

impl ConstantRef<'_> {
    /// The constant's kind.
    pub fn kind(&self) -> ConstKind {
        match self {
            Self::Number(_) => ConstKind::Number,
            Self::String(_) => ConstKind::String,
            Self::Key(_) => ConstKind::Key,
        }
    }
}

/// UTF-16 code units stored little-endian in an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Utf16<'a>(&'a [u8]);

impl Utf16<'_> {
    /// The number of code units.
    pub fn len(&self) -> usize {
        self.0.len() / 2
    }

    /// Whether there are none.
    pub fn is_empty(&self) -> bool {
        self.0.len() < 2
    }

    /// The code units.
    pub fn iter(&self) -> impl Iterator<Item = u16> + '_ {
        self.0
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
    }
}

/// Writes `unit`'s image into the trusted region (through its write
/// window), as a [`TrustedKind::BytecodeUnit`]. At JS-M1 the verifier runs
/// first; at JS-M0 the assembler is the only producer and rejects what the
/// verifier will.
///
/// # Errors
/// The region is full or a protection change failed.
pub fn seal(region: &mut TrustedRegion, unit: &Unit) -> Result<TrustedId, TrustedError> {
    region.insert(TrustedKind::BytecodeUnit, &unit.to_image())
}

/// The sealed unit `id`, if it is live, a bytecode unit and well formed.
pub fn open(region: &TrustedRegion, id: TrustedId) -> Option<UnitView<'_>> {
    UnitView::parse(region.get(id, TrustedKind::BytecodeUnit)?).ok()
}
