//! The text assembler (bytecode note Section 7). It rejects what the
//! JS-M1 verifier will reject (Section 8): unknown opcodes, operands of
//! the wrong kind, registers outside the frame or in the header hole,
//! undeclared or wrongly kinded constants and slots, intrinsic arities,
//! `LoadInt` immediates outside the small-integer range, jumps against
//! their opcode's direction or off an instruction start, and a unit that
//! does not end in a terminator. Labels are resolved by iterative sizing:
//! every instruction starts at its smallest width and only grows.
//!
//! Two extensions of the note's literal syntax: `NaN`, `Infinity` and
//! `-Infinity` are number literals (JSON has none, and a unit may need
//! them), and a number literal is any JSON number.

use std::collections::BTreeMap;

use crate::unit::{Constant, Unit, UnitKind};
use crate::{Intrinsic, Op, OperandKind, Reg, SlotKind, WIDE2, WIDE4};

/// An assembly error, with its 1-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmError {
    /// The line.
    pub line: usize,
    /// What is wrong.
    pub msg: String,
}

impl core::fmt::Display for AsmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "line {}: {}", self.line, self.msg)
    }
}

fn err<T>(line: usize, msg: impl Into<String>) -> Result<T, AsmError> {
    Err(AsmError {
        line,
        msg: msg.into(),
    })
}

#[derive(Debug, Clone)]
enum Opnd {
    Value(i64),
    Label(String),
}

#[derive(Debug, Clone)]
struct Item {
    op: Op,
    forced: Option<u8>,
    operands: Vec<Opnd>,
    line: usize,
}

#[derive(Debug, Default)]
struct Func {
    name: String,
    params: u32,
    registers: u32,
    strict: bool,
    line: usize,
    consts: BTreeMap<u32, (Constant, usize)>,
    slots: BTreeMap<u32, (SlotKind, usize)>,
    items: Vec<Item>,
    labels: BTreeMap<String, (usize, usize)>,
}

/// Assembles every `.function ... .end` of `text`.
///
/// # Errors
/// The first error, with its line.
pub fn assemble(text: &str) -> Result<Vec<Unit>, AsmError> {
    let mut out = Vec::new();
    let mut cur: Option<Func> = None;
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        let mut s = strip_comment(raw).trim();
        if s.is_empty() {
            continue;
        }
        // A label: `name:` at the start.
        if let Some(colon) = s.find(':') {
            let head = &s[..colon];
            if !head.is_empty()
                && !head.starts_with('.')
                && head.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !s[..colon].contains(' ')
            {
                let Some(f) = cur.as_mut() else {
                    return err(line, "a label outside a function");
                };
                if f.labels
                    .insert(head.to_string(), (f.items.len(), line))
                    .is_some()
                {
                    return err(line, format!("label `{head}` is defined twice"));
                }
                s = s[colon + 1..].trim();
                if s.is_empty() {
                    continue;
                }
            }
        }
        if let Some(rest) = s.strip_prefix(".function") {
            if cur.is_some() {
                return err(line, "`.function` inside a function (missing `.end`)");
            }
            cur = Some(function_header(rest, line)?);
            continue;
        }
        if s == ".end" {
            let Some(f) = cur.take() else {
                return err(line, "`.end` outside a function");
            };
            out.push(finish(f, line)?);
            continue;
        }
        let Some(f) = cur.as_mut() else {
            return err(line, "an instruction or directive outside a function");
        };
        if let Some(rest) = s.strip_prefix(".const") {
            let (k, c) = const_directive(rest.trim(), line)?;
            if f.consts.insert(k, (c, line)).is_some() {
                return err(line, format!("constant k{k} is declared twice"));
            }
            continue;
        }
        if let Some(rest) = s.strip_prefix(".slot") {
            let mut parts = rest.split_whitespace();
            let (Some(r), Some(kind), None) = (parts.next(), parts.next(), parts.next()) else {
                return err(line, "expected `.slot sN kind`");
            };
            let n = index(r, 's', line)?;
            let Some(k) = SlotKind::parse(kind) else {
                return err(line, format!("unknown slot kind `{kind}`"));
            };
            if f.slots.insert(n, (k, line)).is_some() {
                return err(line, format!("slot s{n} is declared twice"));
            }
            continue;
        }
        if s.starts_with('.') {
            return err(line, format!("unknown directive `{s}`"));
        }
        f.items.push(instruction(s, line)?);
    }
    if let Some(f) = cur {
        return err(f.line, format!("function `{}` has no `.end`", f.name));
    }
    Ok(out)
}

fn strip_comment(s: &str) -> &str {
    let mut in_str = false;
    let mut esc = false;
    for (i, c) in s.char_indices() {
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
        } else if c == ';' {
            return &s[..i];
        }
    }
    s
}

fn function_header(rest: &str, line: usize) -> Result<Func, AsmError> {
    let mut parts = rest.split_whitespace();
    let Some(name) = parts.next() else {
        return err(line, "`.function` needs a name");
    };
    let mut f = Func {
        name: name.to_string(),
        line,
        ..Func::default()
    };
    for a in parts {
        if a == "strict" {
            f.strict = true;
        } else if let Some(v) = a.strip_prefix("params=") {
            f.params = uint(v, line)?;
        } else if let Some(v) = a.strip_prefix("registers=") {
            f.registers = uint(v, line)?;
        } else {
            return err(line, format!("unknown attribute `{a}`"));
        }
    }
    if f.params > crate::unit::MAX_PARAMS || f.registers > crate::unit::MAX_REGISTERS {
        return err(line, "too many parameters or registers");
    }
    Ok(f)
}

fn uint(s: &str, line: usize) -> Result<u32, AsmError> {
    match s.parse::<u32>() {
        Ok(v) if !s.starts_with('+') => Ok(v),
        _ => err(line, format!("`{s}` is not an unsigned integer")),
    }
}

fn index(s: &str, prefix: char, line: usize) -> Result<u32, AsmError> {
    match s.strip_prefix(prefix) {
        Some(n) => uint(n, line),
        None => err(line, format!("expected `{prefix}N`, found `{s}`")),
    }
}

fn const_directive(rest: &str, line: usize) -> Result<(u32, Constant), AsmError> {
    let mut it = rest.splitn(3, char::is_whitespace);
    let (Some(r), Some(kind), Some(lit)) = (it.next(), it.next(), it.next()) else {
        return err(line, "expected `.const kN kind literal`");
    };
    let k = index(r, 'k', line)?;
    let lit = lit.trim();
    let c = match kind {
        "number" => Constant::Number(number_literal(lit, line)?),
        "string" => Constant::String(json_string(lit, line)?),
        "key" => Constant::Key(json_string(lit, line)?),
        other => return err(line, format!("unknown constant kind `{other}`")),
    };
    Ok((k, c))
}

fn number_literal(s: &str, line: usize) -> Result<f64, AsmError> {
    match s {
        "NaN" => Ok(f64::NAN),
        "Infinity" => Ok(f64::INFINITY),
        "-Infinity" => Ok(f64::NEG_INFINITY),
        _ => match hg_js_number::parse_json_number(s) {
            Some(v) => Ok(v),
            None => err(line, format!("`{s}` is not a JSON number")),
        },
    }
}

/// A JSON string literal (ECMA-404) as UTF-16 code units; `\u` escapes
/// may name lone surrogates, which JS strings allow.
pub(crate) fn json_string(s: &str, line: usize) -> Result<Vec<u16>, AsmError> {
    let Some(inner) = s.strip_prefix('"').and_then(|x| x.strip_suffix('"')) else {
        return err(line, format!("`{s}` is not a JSON string"));
    };
    let mut out = Vec::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return err(line, "an unescaped `\"` inside a string"),
            '\\' => {
                let e = chars.next();
                match e {
                    Some('"') => out.push(0x22),
                    Some('\\') => out.push(0x5C),
                    Some('/') => out.push(0x2F),
                    Some('b') => out.push(0x08),
                    Some('f') => out.push(0x0C),
                    Some('n') => out.push(0x0A),
                    Some('r') => out.push(0x0D),
                    Some('t') => out.push(0x09),
                    Some('u') => {
                        let hex: String = chars.by_ref().take(4).collect();
                        match u16::from_str_radix(&hex, 16) {
                            Ok(v) if hex.len() == 4 => out.push(v),
                            _ => return err(line, "a `\\u` escape takes four hex digits"),
                        }
                    }
                    _ => return err(line, "unknown string escape"),
                }
            }
            c if u32::from(c) < 0x20 => return err(line, "a control character in a string"),
            c => {
                let mut buf = [0u16; 2];
                out.extend_from_slice(c.encode_utf16(&mut buf));
            }
        }
    }
    Ok(out)
}

fn instruction(s: &str, line: usize) -> Result<Item, AsmError> {
    let (head, rest) = match s.find(char::is_whitespace) {
        Some(i) => (&s[..i], s[i..].trim()),
        None => (s, ""),
    };
    let (mnemonic, forced) = if let Some(m) = head.strip_suffix(".w") {
        (m, Some(2u8))
    } else if let Some(m) = head.strip_suffix(".x") {
        (m, Some(4u8))
    } else {
        (head, None)
    };
    let Some(op) = Op::from_mnemonic(mnemonic) else {
        return err(line, format!("unknown mnemonic `{mnemonic}`"));
    };
    let kinds = op.info().operands;
    if forced.is_some() && kinds.is_empty() {
        return err(line, format!("`{mnemonic}` has no operands to widen"));
    }
    let toks: Vec<&str> = if rest.is_empty() {
        Vec::new()
    } else {
        rest.split(',').map(str::trim).collect()
    };
    let mut operands = Vec::new();
    let mut t = toks.iter();
    let mut k = 0;
    while k < kinds.len() {
        let Some(tok) = t.next() else {
            return err(line, format!("`{mnemonic}` takes more operands"));
        };
        match kinds[k] {
            OperandKind::Reg if kinds.get(k + 1) == Some(&OperandKind::Cnt) => {
                let Some((r, n)) = tok.split_once(':') else {
                    return err(
                        line,
                        format!("expected a register list `rN:count`, found `{tok}`"),
                    );
                };
                operands.push(Opnd::Value(i64::from(register(r, line)?.0)));
                operands.push(Opnd::Value(i64::from(uint(n, line)?)));
                k += 2;
                continue;
            }
            OperandKind::Reg => operands.push(Opnd::Value(i64::from(register(tok, line)?.0))),
            OperandKind::Cnt => operands.push(Opnd::Value(i64::from(uint(tok, line)?))),
            OperandKind::Imm => {
                let Some(v) = tok.strip_prefix('#') else {
                    return err(line, format!("expected an immediate `#N`, found `{tok}`"));
                };
                let Ok(v) = v.parse::<i64>() else {
                    return err(line, format!("`{tok}` is not an integer"));
                };
                if !(-(1i64 << 30)..(1i64 << 30)).contains(&v) {
                    return err(
                        line,
                        format!("`{tok}` is outside the small-integer range [-2^30, 2^30 - 1]"),
                    );
                }
                operands.push(Opnd::Value(v));
            }
            OperandKind::Const(_) => operands.push(Opnd::Value(i64::from(index(tok, 'k', line)?))),
            OperandKind::Slot(_) => operands.push(Opnd::Value(i64::from(index(tok, 's', line)?))),
            OperandKind::Fwd | OperandKind::Back => {
                let Some(l) = tok.strip_prefix('@') else {
                    return err(line, format!("expected a label `@name`, found `{tok}`"));
                };
                operands.push(Opnd::Label(l.to_string()));
            }
            OperandKind::Intr => {
                let Some(n) = tok.strip_prefix('%') else {
                    return err(
                        line,
                        format!("expected an intrinsic `%Name`, found `{tok}`"),
                    );
                };
                let Some(intr) = Intrinsic::from_name(n) else {
                    return err(line, format!("unknown intrinsic `%{n}`"));
                };
                operands.push(Opnd::Value(i64::from(intr.index())));
            }
        }
        k += 1;
    }
    if t.next().is_some() {
        return err(
            line,
            format!(
                "`{mnemonic}` takes {} operands",
                toks.len().saturating_sub(1)
            ),
        );
    }
    Ok(Item {
        op,
        forced,
        operands,
        line,
    })
}

fn register(s: &str, line: usize) -> Result<Reg, AsmError> {
    if s == "this" {
        return Ok(Reg::RECEIVER);
    }
    if let Some(n) = s.strip_prefix('r') {
        return Reg::local(uint(n, line)?).map_or_else(|| err(line, "register out of range"), Ok);
    }
    if let Some(n) = s.strip_prefix('a') {
        return Reg::param(uint(n, line)?).map_or_else(|| err(line, "parameter out of range"), Ok);
    }
    err(
        line,
        format!("expected a register (`rN`, `aN` or `this`), found `{s}`"),
    )
}

fn fits(v: i64, signed: bool, scale: u8) -> bool {
    let bits = 8 * u32::from(scale);
    if signed {
        let lim = 1i64 << (bits - 1);
        (-lim..lim).contains(&v)
    } else {
        v >= 0 && (bits >= 63 || v < (1i64 << bits))
    }
}

fn size(item: &Item, scale: u8) -> u32 {
    let n = u32::try_from(item.op.info().operands.len()).unwrap_or(0);
    u32::from(scale > 1) + 1 + n * u32::from(scale)
}

fn finish(f: Func, end_line: usize) -> Result<Unit, AsmError> {
    if f.items.is_empty() {
        return err(
            end_line,
            format!("function `{}` has no instructions", f.name),
        );
    }
    // Dense constant and slot tables.
    let mut constants = Vec::new();
    for (i, (k, (c, line))) in f.consts.iter().enumerate() {
        if u32::try_from(i).ok() != Some(*k) {
            return err(
                *line,
                format!("constants are numbered from k0 without gaps; k{i} is missing"),
            );
        }
        constants.push(c.clone());
    }
    let mut slots = Vec::new();
    for (i, (k, (s, line))) in f.slots.iter().enumerate() {
        if u32::try_from(i).ok() != Some(*k) {
            return err(
                *line,
                format!("slots are numbered from s0 without gaps; s{i} is missing"),
            );
        }
        slots.push(*s);
    }
    // Operand checks against the frame and the tables (bytecode note 8).
    for item in &f.items {
        let kinds = item.op.info().operands;
        for (j, (kind, o)) in kinds.iter().zip(&item.operands).enumerate() {
            let Opnd::Value(v) = o else { continue };
            match kind {
                OperandKind::Reg => {
                    let r = Reg(i32::try_from(*v).unwrap_or(i32::MIN));
                    let count = if kinds.get(j + 1) == Some(&OperandKind::Cnt) {
                        match item.operands.get(j + 1) {
                            Some(Opnd::Value(c)) => *c,
                            _ => 0,
                        }
                    } else {
                        1
                    };
                    check_regs(r, count, &f, item.line)?;
                }
                OperandKind::Const(ks) => {
                    let c = usize::try_from(*v).ok().and_then(|i| constants.get(i));
                    match c {
                        Some(c) if ks.contains(c.kind()) => {}
                        Some(c) => {
                            return err(
                                item.line,
                                format!(
                                    "`{}` does not accept a {} constant",
                                    item.op.info().mnemonic,
                                    c.kind().name()
                                ),
                            );
                        }
                        None => return err(item.line, format!("constant k{v} is not declared")),
                    }
                }
                OperandKind::Slot(want) => {
                    match usize::try_from(*v).ok().and_then(|i| slots.get(i)) {
                        Some(k) if k == want => {}
                        Some(k) => {
                            return err(
                                item.line,
                                format!(
                                    "slot s{v} is a `{}` slot; `{}` needs a `{}` slot",
                                    k.name(),
                                    item.op.info().mnemonic,
                                    want.name()
                                ),
                            );
                        }
                        None => return err(item.line, format!("slot s{v} is not declared")),
                    }
                }
                OperandKind::Intr => {
                    let intr = Intrinsic::from_index(u32::try_from(*v).unwrap_or(u32::MAX));
                    let count = match item.operands.get(j + 2) {
                        Some(Opnd::Value(c)) => *c,
                        _ => -1,
                    };
                    if let Some(intr) = intr
                        && i64::from(intr.arity()) != count
                    {
                        return err(
                            item.line,
                            format!(
                                "`%{}` takes {} arguments, not {count}",
                                intr.name(),
                                intr.arity()
                            ),
                        );
                    }
                }
                _ => {}
            }
        }
    }
    let last = f.items.last().map(|i| (i.op, i.line));
    if let Some((op, line)) = last
        && !op.info().terminator
    {
        return err(
            line,
            format!(
                "a unit ends in `Return`, `Jump` or `LoopBack`, not `{}`",
                op.info().mnemonic
            ),
        );
    }
    // Scales: the smallest that fits every non-jump operand, or forced.
    let mut scales: Vec<u8> = Vec::new();
    for item in &f.items {
        let kinds = item.op.info().operands;
        let mut s = 1u8;
        for (kind, o) in kinds.iter().zip(&item.operands) {
            if let Opnd::Value(v) = o {
                while !fits(*v, kind.signed(), s) {
                    s = if s == 1 { 2 } else { 4 };
                    if s == 4 && !fits(*v, kind.signed(), 4) {
                        return err(item.line, "an operand does not fit 4 bytes");
                    }
                }
            }
        }
        if let Some(fs) = item.forced {
            if fs < s {
                return err(item.line, "the forced width is too narrow for an operand");
            }
            s = fs;
        }
        scales.push(s);
    }
    // Iterative sizing of jumps: widths only grow.
    let mut starts = Vec::new();
    loop {
        starts.clear();
        let mut at = 0u32;
        for (item, s) in f.items.iter().zip(&scales) {
            starts.push(at);
            at = at.checked_add(size(item, *s)).ok_or(AsmError {
                line: item.line,
                msg: "code too long".into(),
            })?;
        }
        if at > crate::unit::MAX_CODE {
            return err(end_line, "code too long");
        }
        let mut changed = false;
        for (i, item) in f.items.iter().enumerate() {
            let kinds = item.op.info().operands;
            for (kind, o) in kinds.iter().zip(&item.operands) {
                let Opnd::Label(l) = o else { continue };
                let d = distance(&f, &starts, i, l, *kind, item.line)?;
                if !fits(d, false, scales[i]) {
                    if item.forced.is_some() || scales[i] == 4 {
                        return err(item.line, "the jump does not fit its forced width");
                    }
                    scales[i] = if scales[i] == 1 { 2 } else { 4 };
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    // Emit.
    let mut code = Vec::new();
    for (i, item) in f.items.iter().enumerate() {
        let s = scales[i];
        match s {
            2 => code.push(WIDE2),
            4 => code.push(WIDE4),
            _ => {}
        }
        code.push(item.op.byte());
        let kinds = item.op.info().operands;
        for (kind, o) in kinds.iter().zip(&item.operands) {
            let v = match o {
                Opnd::Value(v) => *v,
                Opnd::Label(l) => distance(&f, &starts, i, l, *kind, item.line)?,
            };
            let bytes = v.to_le_bytes();
            code.extend_from_slice(&bytes[..usize::from(s)]);
        }
    }
    Ok(Unit {
        name: f.name,
        kind: UnitKind::Function,
        strict: f.strict,
        params: f.params,
        registers: f.registers,
        code,
        slots,
        constants,
    })
}

fn check_regs(r: Reg, count: i64, f: &Func, line: usize) -> Result<(), AsmError> {
    if count < 0 {
        return err(line, "a negative register count");
    }
    if count == 0 {
        // An empty list names no register; it is written `r0:0`, so the
        // base never names a register outside the frame.
        if r.0 != 0 {
            return err(
                line,
                format!(
                    "an empty register list is written `r0:0`, not `{}:0`",
                    show_reg(r)
                ),
            );
        }
        return Ok(());
    }
    let first = r.resolve(f.params, f.registers);
    let Some(first) = first else {
        return err(
            line,
            format!(
                "register {} is outside the frame (or in the header)",
                show_reg(r)
            ),
        );
    };
    let last = i64::from(r.0) + count - 1;
    let last = Reg(i32::try_from(last).unwrap_or(i32::MAX));
    let ok = match (first, last.resolve(f.params, f.registers)) {
        (crate::RegKind::Local(_), Some(crate::RegKind::Local(_))) => true,
        // A list of parameters runs downward in index space; lists are of
        // locals at JS-M0.
        _ => count == 1,
    };
    if !ok {
        return err(
            line,
            format!("the register list {}:{count} leaves its range", show_reg(r)),
        );
    }
    Ok(())
}

fn show_reg(r: Reg) -> String {
    match r.0 {
        v if v >= 0 => format!("r{v}"),
        -13 => "this".into(),
        v if v <= -14 => format!("a{}", -14 - v),
        v => format!("header slot {v}"),
    }
}

fn distance(
    f: &Func,
    starts: &[u32],
    i: usize,
    label: &str,
    kind: OperandKind,
    line: usize,
) -> Result<i64, AsmError> {
    let Some((target, _)) = f.labels.get(label) else {
        return err(line, format!("undefined label `{label}`"));
    };
    let Some(&to) = starts.get(*target) else {
        return err(line, format!("label `{label}` is not on an instruction"));
    };
    let from = starts[i];
    match kind {
        OperandKind::Fwd => {
            if *target <= i {
                return err(
                    line,
                    format!(
                        "`@{label}` is a forward jump's target, but it is not after this instruction"
                    ),
                );
            }
            Ok(i64::from(to) - i64::from(from))
        }
        _ => {
            if *target > i {
                return err(
                    line,
                    format!(
                        "`@{label}` is a backward jump's target, but it is after this instruction"
                    ),
                );
            }
            Ok(i64::from(from) - i64::from(to))
        }
    }
}
