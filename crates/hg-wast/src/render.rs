//! Writing a parsed script back out as text.
//!
//! This exists to be checked, not to be pretty. A reader that misread a
//! script would still parse every file and still count every command;
//! what it would not do is write the script back out and have an engine
//! reach the same verdict on it. Rendering is what turns "it parsed"
//! into a claim an oracle can refute.
//!
//! So the rules here are about meaning, never about appearance:
//!
//! - A module is its own source text, byte for byte ([`crate::script`]
//!   keeps the span for this). Nothing is re-written inside one.
//! - A number is written in a form that is exact. Floats go out as
//!   hexadecimal, which names a binary value exactly and so cannot
//!   round on the way out and back; a decimal rendering would have to
//!   be proved round-trip safe, and `-0`, the subnormals and the NaN
//!   payloads are precisely where such a proof fails.
//! - A name is escaped byte by byte unless it is plain printable ASCII,
//!   so a name with a quotation mark, a backslash or a control
//!   character in it comes back as the same bytes.
//!
//! What this is not: a formatter. The output is for an engine to read.

use crate::script::{Action, Cmd, Module, TrapOf};
use crate::value::{Pattern, Shape, Value};

/// A whole script, rendered from what was parsed out of `src`.
#[must_use]
pub fn script(src: &str, cmds: &[Cmd]) -> String {
    let mut out = String::new();
    for c in cmds {
        cmd(src, c, &mut out);
        out.push('\n');
    }
    out
}

/// One command.
pub fn cmd(src: &str, c: &Cmd, out: &mut String) {
    match c {
        Cmd::Module(m) => out.push_str(m.text(src)),
        Cmd::Instance {
            instance, module, ..
        } => {
            out.push_str("(module instance");
            push_id(instance.as_deref(), out);
            push_id(module.as_deref(), out);
            out.push(')');
        }
        Cmd::Register { name, module, .. } => {
            out.push_str("(register ");
            push_name(name, out);
            push_id(module.as_deref(), out);
            out.push(')');
        }
        Cmd::Action(a) => action(a, out),
        Cmd::AssertReturn {
            action: a,
            expected,
            ..
        } => {
            out.push_str("(assert_return ");
            action(a, out);
            for p in expected {
                out.push(' ');
                pattern(p, out);
            }
            out.push(')');
        }
        Cmd::AssertTrap { of, message, .. } => {
            out.push_str("(assert_trap ");
            match of {
                TrapOf::Action(a) => action(a, out),
                TrapOf::Module(m) => module(src, m, out),
            }
            out.push(' ');
            push_name(message, out);
            out.push(')');
        }
        Cmd::AssertExhaustion {
            action: a, message, ..
        } => {
            out.push_str("(assert_exhaustion ");
            action(a, out);
            out.push(' ');
            push_name(message, out);
            out.push(')');
        }
        Cmd::AssertException { action: a, .. } => {
            out.push_str("(assert_exception ");
            action(a, out);
            out.push(')');
        }
        Cmd::AssertMalformed {
            module: m, message, ..
        } => assertion_about_module(src, "assert_malformed", m, message, out),
        Cmd::AssertInvalid {
            module: m, message, ..
        } => assertion_about_module(src, "assert_invalid", m, message, out),
        Cmd::AssertUnlinkable {
            module: m, message, ..
        } => assertion_about_module(src, "assert_unlinkable", m, message, out),
        // A meta command directs the interpreter's own tooling and is
        // not acted on here, so it goes back exactly as it came.
        Cmd::Meta { span, .. } => out.push_str(span.text(src)),
    }
}

fn assertion_about_module(src: &str, keyword: &str, m: &Module, message: &str, out: &mut String) {
    out.push('(');
    out.push_str(keyword);
    out.push(' ');
    module(src, m, out);
    out.push(' ');
    push_name(message, out);
    out.push(')');
}

fn module(src: &str, m: &Module, out: &mut String) {
    out.push_str(m.text(src));
}

/// An action.
pub fn action(a: &Action, out: &mut String) {
    match a {
        Action::Invoke {
            module, name, args, ..
        } => {
            out.push_str("(invoke");
            push_id(module.as_deref(), out);
            out.push(' ');
            push_name(name, out);
            for v in args {
                out.push(' ');
                value(v, out);
            }
            out.push(')');
        }
        Action::Get { module, name, .. } => {
            out.push_str("(get");
            push_id(module.as_deref(), out);
            out.push(' ');
            push_name(name, out);
            out.push(')');
        }
    }
}

/// One value, parenthesised as a script writes it.
pub fn value(v: &Value, out: &mut String) {
    match v {
        Value::I32(n) => out.push_str(&format!("(i32.const 0x{n:x})")),
        Value::I64(n) => out.push_str(&format!("(i64.const 0x{n:x})")),
        Value::F32(bits) => {
            out.push_str("(f32.const ");
            out.push_str(&float32(*bits));
            out.push(')');
        }
        Value::F64(bits) => {
            out.push_str("(f64.const ");
            out.push_str(&float64(*bits));
            out.push(')');
        }
        // Sixteen bytes are written as sixteen byte lanes: the one
        // shape that can say any `v128` exactly, whatever shape it was
        // written in.
        Value::V128(bytes) => {
            out.push_str("(v128.const i8x16");
            for b in bytes {
                out.push_str(&format!(" 0x{b:x}"));
            }
            out.push(')');
        }
        Value::RefNull(ty) => out.push_str(&format!("(ref.null {ty})")),
        Value::RefExtern(n) => out.push_str(&format!("(ref.extern {n})")),
        Value::RefHost(n) => out.push_str(&format!("(ref.host {n})")),
    }
}

/// One result pattern.
pub fn pattern(p: &Pattern, out: &mut String) {
    match p {
        Pattern::Exact(v) => value(v, out),
        Pattern::NanCanonical(ty) => out.push_str(&format!("({}.const nan:canonical)", ty.name())),
        Pattern::NanArithmetic(ty) => {
            out.push_str(&format!("({}.const nan:arithmetic)", ty.name()))
        }
        Pattern::V128(shape, lanes) => {
            out.push_str("(v128.const ");
            out.push_str(shape.name());
            for l in lanes {
                out.push(' ');
                lane(*shape, l, out);
            }
            out.push(')');
        }
        Pattern::RefOfKind(kind) => out.push_str(&format!("(ref.{kind})")),
        Pattern::Either(alts) => {
            out.push_str("(either");
            for a in alts {
                out.push(' ');
                pattern(a, out);
            }
            out.push(')');
        }
    }
}

/// One lane of a `v128` pattern, written bare rather than parenthesised.
fn lane(shape: Shape, p: &Pattern, out: &mut String) {
    match p {
        Pattern::NanCanonical(_) => out.push_str("nan:canonical"),
        Pattern::NanArithmetic(_) => out.push_str("nan:arithmetic"),
        Pattern::Exact(Value::F32(bits)) => out.push_str(&float32(*bits)),
        Pattern::Exact(Value::F64(bits)) => out.push_str(&float64(*bits)),
        Pattern::Exact(Value::I32(n)) => {
            if shape.is_float() {
                out.push_str(&float32(*n));
            } else {
                out.push_str(&format!("0x{n:x}"));
            }
        }
        Pattern::Exact(Value::I64(n)) => {
            if shape.is_float() {
                out.push_str(&float64(*n));
            } else {
                out.push_str(&format!("0x{n:x}"));
            }
        }
        // No other pattern can be a lane; writing it as a value keeps
        // the output readable if one ever is, and an engine will say so.
        other => pattern(other, out),
    }
}

/// An `f32`'s bits as a literal that reads back to the same bits.
#[must_use]
pub fn float32(bits: u32) -> String {
    // 23 stored mantissa bits is not a whole number of hexadecimal
    // digits, so the fraction is shifted up by one to make 24.
    float(u64::from(bits), 23, 8, 24)
}

/// An `f64`'s bits as a literal that reads back to the same bits.
#[must_use]
pub fn float64(bits: u64) -> String {
    float(bits, 52, 11, 52)
}

/// The shared rendering: an infinity, a NaN with its payload, a zero,
/// a subnormal or a normal number, in hexadecimal so that nothing
/// rounds.
fn float(bits: u64, mbits: u32, ebits: u32, frac_bits: u32) -> String {
    let sign = (bits >> (mbits + ebits)) & 1 == 1;
    let s = if sign { "-" } else { "" };
    let exp = (bits >> mbits) & ((1 << ebits) - 1);
    let mantissa = bits & ((1u64 << mbits) - 1);
    let exp_all = (1u64 << ebits) - 1;
    let bias = (1i64 << (ebits - 1)) - 1;

    if exp == exp_all {
        if mantissa == 0 {
            return format!("{s}inf");
        }
        // The payload is written out, never as the bare `nan`: two
        // different NaNs must not render alike, and telling them apart
        // is what half the float tests are for.
        return format!("{s}nan:0x{mantissa:x}");
    }
    if exp == 0 && mantissa == 0 {
        return format!("{s}0x0p+0");
    }
    // A subnormal has no leading one and the smallest exponent; a
    // normal number has a leading one.
    let (lead, e) = if exp == 0 {
        (0u64, 1 - bias)
    } else {
        (1u64, exp as i64 - bias)
    };
    let digits = (frac_bits as usize).div_ceil(4);
    let frac = mantissa << (frac_bits - mbits);
    let mut hex = format!("{frac:0digits$x}");
    while hex.len() > 1 && hex.ends_with('0') {
        hex.pop();
    }
    if frac == 0 {
        format!("{s}0x{lead}p{e:+}")
    } else {
        format!("{s}0x{lead}.{hex}p{e:+}")
    }
}

/// A name as a string literal that decodes to the same bytes.
///
/// Anything but plain printable ASCII goes out as `\HH` per byte, so a
/// name holding a quotation mark, a backslash or a character outside
/// ASCII survives exactly.
fn push_name(name: &str, out: &mut String) {
    out.push('"');
    for b in name.bytes() {
        if b.is_ascii_graphic() && b != b'"' && b != b'\\' || b == b' ' {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("\\{b:02x}"));
        }
    }
    out.push('"');
}

/// An optional identifier, with the space before it.
fn push_id(id: Option<&str>, out: &mut String) {
    if let Some(id) = id {
        out.push(' ');
        out.push_str(id);
    }
}

#[cfg(test)]
mod tests;
