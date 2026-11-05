//! The cases: each core operation that computes on values, with its
//! JavaScript equivalent, applied to every operand (or ordered pair) from
//! a fixed set of edge values.

use std::fmt::Write as _;

/// An operand.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// `undefined`.
    Undefined,
    /// `null`.
    Null,
    /// A Boolean.
    Bool(bool),
    /// A Number.
    Number(f64),
    /// A String, as UTF-16 code units.
    String(Vec<u16>),
    /// A fresh ordinary object with a null prototype: it has no
    /// `toString` or `valueOf`, so ToPrimitive on it throws a TypeError.
    Object,
}

impl Value {
    /// The value as a JSON literal, for the assembler (`.const`). It is
    /// also a valid JavaScript literal.
    pub fn number_literal(n: f64) -> String {
        if n.is_nan() {
            "NaN".into()
        } else if n == f64::INFINITY {
            "Infinity".into()
        } else if n == f64::NEG_INFINITY {
            "-Infinity".into()
        } else if n == 0.0 && n.is_sign_negative() {
            "-0".into()
        } else {
            // Rust's shortest round-trip form; JSON and JavaScript read it
            // back as the same double.
            format!("{n:?}")
        }
    }

    /// A JSON string literal with every code unit outside printable ASCII
    /// (and `"` and `\`) written as `\uXXXX`. It is also a JavaScript
    /// string literal, lone surrogates included.
    pub fn string_literal(s: &[u16]) -> String {
        let mut out = String::from("\"");
        for &u in s {
            match u {
                0x20..=0x7E if u != 0x22 && u != 0x5C => out.push(char::from(u as u8)),
                _ => {
                    let _ = write!(out, "\\u{u:04x}");
                }
            }
        }
        out.push('"');
        out
    }

    /// The value as a JavaScript expression.
    pub fn js(&self) -> String {
        match self {
            Self::Undefined => "undefined".into(),
            Self::Null => "null".into(),
            Self::Bool(b) => b.to_string(),
            Self::Number(n) => format!("({})", Self::number_literal(*n)),
            Self::String(s) => Self::string_literal(s),
            Self::Object => "Object.create(null)".into(),
        }
    }
}

/// An operation under test.
#[derive(Debug)]
pub struct OpSpec {
    /// Its bytecode mnemonic.
    pub mnemonic: &'static str,
    /// 1: the operand is in the accumulator. 2: the left operand is in a
    /// register and the right one in the accumulator.
    pub arity: u8,
    /// The JavaScript expression, over `a` (and `b`), that computes it.
    /// `a` and `b` are `let` bindings.
    pub js: &'static str,
}

/// The core set's value operations (design review R0-O1; bytecode note
/// 4.4). Loads, jumps, `Return`, properties and intrinsics are exercised
/// by the interpreter's own tests and by the probe.
pub const OPS: &[OpSpec] = &[
    OpSpec {
        mnemonic: "ToBoolean",
        arity: 1,
        js: "!!a",
    },
    OpSpec {
        mnemonic: "ToNumber",
        arity: 1,
        js: "+a",
    },
    // No BigInt in the core set: ToNumeric agrees with ToNumber on it.
    OpSpec {
        mnemonic: "ToNumeric",
        arity: 1,
        js: "+a",
    },
    OpSpec {
        mnemonic: "ToString",
        arity: 1,
        js: "`${a}`",
    },
    OpSpec {
        mnemonic: "Not",
        arity: 1,
        js: "!a",
    },
    OpSpec {
        mnemonic: "TypeOf",
        arity: 1,
        js: "typeof a",
    },
    OpSpec {
        mnemonic: "Neg",
        arity: 1,
        js: "-a",
    },
    OpSpec {
        mnemonic: "BitNot",
        arity: 1,
        js: "~a",
    },
    OpSpec {
        mnemonic: "Inc",
        arity: 1,
        js: "++a",
    },
    OpSpec {
        mnemonic: "Dec",
        arity: 1,
        js: "--a",
    },
    OpSpec {
        mnemonic: "Add",
        arity: 2,
        js: "a + b",
    },
    OpSpec {
        mnemonic: "Sub",
        arity: 2,
        js: "a - b",
    },
    OpSpec {
        mnemonic: "Mul",
        arity: 2,
        js: "a * b",
    },
    OpSpec {
        mnemonic: "Div",
        arity: 2,
        js: "a / b",
    },
    OpSpec {
        mnemonic: "Mod",
        arity: 2,
        js: "a % b",
    },
    OpSpec {
        mnemonic: "Exp",
        arity: 2,
        js: "a ** b",
    },
    OpSpec {
        mnemonic: "BitAnd",
        arity: 2,
        js: "a & b",
    },
    OpSpec {
        mnemonic: "BitOr",
        arity: 2,
        js: "a | b",
    },
    OpSpec {
        mnemonic: "BitXor",
        arity: 2,
        js: "a ^ b",
    },
    OpSpec {
        mnemonic: "Shl",
        arity: 2,
        js: "a << b",
    },
    OpSpec {
        mnemonic: "Sar",
        arity: 2,
        js: "a >> b",
    },
    OpSpec {
        mnemonic: "Shr",
        arity: 2,
        js: "a >>> b",
    },
    OpSpec {
        mnemonic: "LooseEq",
        arity: 2,
        js: "a == b",
    },
    OpSpec {
        mnemonic: "StrictEq",
        arity: 2,
        js: "a === b",
    },
    OpSpec {
        mnemonic: "Lt",
        arity: 2,
        js: "a < b",
    },
    OpSpec {
        mnemonic: "Gt",
        arity: 2,
        js: "a > b",
    },
    OpSpec {
        mnemonic: "Le",
        arity: 2,
        js: "a <= b",
    },
    OpSpec {
        mnemonic: "Ge",
        arity: 2,
        js: "a >= b",
    },
];

fn s(text: &str) -> Value {
    Value::String(text.encode_utf16().collect())
}

/// The edge values. Numbers cover the boundaries the conversions and
/// the small-integer paths care about (±0, ±2^30, ±2^31, 2^32, 2^53,
/// NaN, the infinities, subnormals, the largest double, the exponent
/// switch of Number::toString at 1e21 and 1e-7); strings cover
/// StringToNumber's grammar (white space, signs, prefixes, exponents,
/// `Infinity`, separators it refuses) and code units outside the BMP.
pub fn values() -> Vec<Value> {
    let mut v = vec![
        Value::Undefined,
        Value::Null,
        Value::Bool(true),
        Value::Bool(false),
        Value::Object,
    ];
    for n in [
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.5,
        -1.5,
        2.5,
        3.0,
        7.0,
        31.0,
        32.0,
        33.0,
        1_073_741_823.0,
        1_073_741_824.0,
        -1_073_741_824.0,
        -1_073_741_825.0,
        2_147_483_647.0,
        2_147_483_648.0,
        -2_147_483_648.0,
        -2_147_483_649.0,
        4_294_967_295.0,
        4_294_967_296.0,
        9_007_199_254_740_992.0,
        9_007_199_254_740_994.0,
        1e21,
        1e-7,
        123.456,
        0.1,
        5e-324,
        2.2250738585072014e-308,
        f64::MAX,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        v.push(Value::Number(n));
    }
    for t in [
        "",
        "0",
        "-0",
        "1",
        " 12 ",
        "\t\n\u{a0}\u{feff}7\u{2028}",
        "0x1F",
        "0X1f",
        "0b101",
        "0o17",
        "-0x10",
        "+0x10",
        "1e3",
        "1E-3",
        ".5",
        "5.",
        "+.5",
        "Infinity",
        "-Infinity",
        "+Infinity",
        "infinity",
        "1_000",
        "1,5",
        "abc",
        "NaN",
        "2147483648",
        "9007199254740993",
        "\u{661}",
        "a",
        "b",
        "aa",
        "\u{1F600}",
        "\u{ff}",
        "0.0000001",
    ] {
        v.push(s(t));
    }
    // A lone surrogate, which a JavaScript string may hold.
    v.push(Value::String(vec![0xD800]));
    v
}

/// One case: an operation on its operands.
#[derive(Clone, Debug)]
pub struct Case {
    /// The operation.
    pub op: &'static OpSpec,
    /// The operand, or the left operand.
    pub a: Value,
    /// The right operand, for a binary operation.
    pub b: Option<Value>,
}

impl Case {
    /// The case as a line of JavaScript, for a report.
    pub fn describe(&self) -> String {
        match &self.b {
            None => format!("{} on a = {}", self.op.mnemonic, self.a.js()),
            Some(b) => format!(
                "{} on a = {}, b = {}",
                self.op.mnemonic,
                self.a.js(),
                b.js()
            ),
        }
    }
}

/// Every case, grouped by operation in [`OPS`] order.
pub fn all() -> Vec<Case> {
    let vals = values();
    let mut out = Vec::new();
    for op in OPS {
        for a in &vals {
            if op.arity == 1 {
                out.push(Case {
                    op,
                    a: a.clone(),
                    b: None,
                });
            } else {
                for b in &vals {
                    out.push(Case {
                        op,
                        a: a.clone(),
                        b: Some(b.clone()),
                    });
                }
            }
        }
    }
    out
}
