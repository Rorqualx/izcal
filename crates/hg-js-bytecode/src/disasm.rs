//! The disassembler: a unit back to the assembler's text, so that
//! reassembling the text gives the same bytes (bytecode note Section 7,
//! target R0T-6). Jump targets get labels `L0`, `L1`, … in offset order;
//! an instruction written with a prefix keeps it as `.w` or `.x`.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::unit::{Constant, Unit};
use crate::{Intrinsic, OperandKind, Reg, decode};

/// The text of `unit`. Bytes that do not decode end the listing with a
/// comment naming the offset.
pub fn disassemble(unit: &Unit) -> String {
    let mut o = String::new();
    let _ = writeln!(
        o,
        ".function {} params={} registers={}{}",
        unit.name,
        unit.params,
        unit.registers,
        if unit.strict { " strict" } else { "" }
    );
    for (i, c) in unit.constants.iter().enumerate() {
        let lit = match c {
            Constant::Number(f) => number(*f),
            Constant::String(u) | Constant::Key(u) => json_string(u),
        };
        let _ = writeln!(o, ".const k{i} {} {lit}", c.kind().name());
    }
    for (i, s) in unit.slots.iter().enumerate() {
        let _ = writeln!(o, ".slot s{i} {}", s.name());
    }
    // First pass: instructions and jump targets.
    let mut instrs = Vec::new();
    let mut targets: BTreeMap<u32, usize> = BTreeMap::new();
    let mut pc = 0u32;
    let mut bad = None;
    while usize::try_from(pc).is_ok_and(|p| p < unit.code.len()) {
        match decode(&unit.code, pc) {
            Ok(i) => {
                for (j, k) in i.op.info().operands.iter().enumerate() {
                    match k {
                        OperandKind::Fwd => {
                            targets.insert(i.forward_target(j), 0);
                        }
                        OperandKind::Back => {
                            targets.insert(i.backward_target(j), 0);
                        }
                        _ => {}
                    }
                }
                pc = i.end;
                instrs.push(i);
            }
            Err(e) => {
                bad = Some((pc, e));
                break;
            }
        }
    }
    for (n, v) in targets.values_mut().enumerate() {
        *v = n;
    }
    let label = |t: u32| {
        targets
            .get(&t)
            .map_or_else(|| format!("bad{t}"), |n| format!("L{n}"))
    };
    for i in &instrs {
        let prefix = targets
            .get(&i.start)
            .map_or_else(String::new, |n| format!("L{n}:"));
        let scale = match i.scale {
            2 => ".w",
            4 => ".x",
            _ => "",
        };
        let kinds = i.op.info().operands;
        let mut ops = Vec::new();
        let mut j = 0;
        while j < kinds.len() {
            match kinds[j] {
                OperandKind::Reg if kinds.get(j + 1) == Some(&OperandKind::Cnt) => {
                    ops.push(format!("{}:{}", reg(i.reg(j)), i.index(j + 1)));
                    j += 2;
                    continue;
                }
                OperandKind::Reg => ops.push(reg(i.reg(j))),
                OperandKind::Cnt => ops.push(i.index(j).to_string()),
                OperandKind::Imm => ops.push(format!("#{}", i.imm(j))),
                OperandKind::Const(_) => ops.push(format!("k{}", i.index(j))),
                OperandKind::Slot(_) => ops.push(format!("s{}", i.index(j))),
                OperandKind::Fwd => ops.push(format!("@{}", label(i.forward_target(j)))),
                OperandKind::Back => ops.push(format!("@{}", label(i.backward_target(j)))),
                OperandKind::Intr => ops.push(match Intrinsic::from_index(i.index(j)) {
                    Some(x) => format!("%{}", x.name()),
                    None => format!("%unknown{}", i.index(j)),
                }),
            }
            j += 1;
        }
        let ops = if ops.is_empty() {
            String::new()
        } else {
            format!(" {}", ops.join(", "))
        };
        let _ = writeln!(o, "{prefix:<8}{}{scale}{ops}", i.op.info().mnemonic);
    }
    if let Some((pc, e)) = bad {
        let _ = writeln!(o, "; undecodable bytes at {pc}: {e:?}");
    }
    o.push_str(".end\n");
    o
}

fn reg(r: Reg) -> String {
    match r.0 {
        v if v >= 0 => format!("r{v}"),
        -13 => "this".into(),
        v if v <= -14 => format!("a{}", -14i64 - i64::from(v)),
        v => format!("header{v}"),
    }
}

fn number(f: f64) -> String {
    if f.is_nan() {
        return "NaN".into();
    }
    if f.is_infinite() {
        return if f > 0.0 {
            "Infinity".into()
        } else {
            "-Infinity".into()
        };
    }
    if f == 0.0 && f.is_sign_negative() {
        return "-0".into();
    }
    // Number::toString's shortest round-trip digits are a JSON number.
    hg_js_number::number_to_string(f)
}

fn json_string(units: &[u16]) -> String {
    let mut o = String::from("\"");
    let mut i = 0;
    while i < units.len() {
        let u = units[i];
        // A valid surrogate pair is written as the character.
        if (0xD800..0xDC00).contains(&u)
            && let Some(&lo) = units.get(i + 1)
            && (0xDC00..0xE000).contains(&lo)
        {
            let c = 0x10000 + ((u32::from(u) - 0xD800) << 10) + (u32::from(lo) - 0xDC00);
            if let Some(ch) = char::from_u32(c) {
                o.push(ch);
            }
            i += 2;
            continue;
        }
        match u {
            0x22 => o.push_str("\\\""),
            0x5C => o.push_str("\\\\"),
            0x0A => o.push_str("\\n"),
            0x09 => o.push_str("\\t"),
            0x0D => o.push_str("\\r"),
            u if u < 0x20 || (0xD800..0xE000).contains(&u) || u >= 0x7F => {
                let _ = write!(o, "\\u{u:04x}");
            }
            u => o.push(char::from(u8::try_from(u).unwrap_or(b'?'))),
        }
        i += 1;
    }
    o.push('"');
    o
}
