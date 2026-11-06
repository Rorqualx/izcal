//! Every JS-M0 opcode, run by assembled programs. Each expected result is
//! derived by hand from the ECMA-262 clause named beside it (tc39.es
//! ecma262, the revision pinned in `crates/hg-js-ops/ops/anchors.pinned`),
//! not from running another engine.

use hg_js_ops::support::{ErrorKind, HostValue};

use crate::harness::*;

// --- Loads and register transfer (0x01-0x09) --------------------------------

#[test]
fn loads() {
    // 6.1.1-6.1.3: the Undefined, Null and Boolean values.
    assert_eq!(eval("", " LoadUndefined"), Ok(HostValue::Undefined));
    assert_eq!(eval("", " LoadNull"), Ok(HostValue::Null));
    assert_bool(eval("", " LoadTrue"), true);
    assert_bool(eval("", " LoadFalse"), false);
    // 13.2.3.1 (numeric literals): the immediate is a small integer.
    assert_num(eval("", " LoadInt #-7"), -7.0);
    assert_num(eval("", " LoadInt.x #-1073741824"), -1_073_741_824.0);
    // Constants keep −0 and non-integral values exactly (6.1.6.1).
    assert_num(eval(".const k0 number -0", " LoadConst k0"), -0.0);
    assert_num(eval(".const k0 number 0.5", " LoadConst k0"), 0.5);
    assert_str(eval(".const k0 string \"hi\"", " LoadConst k0"), "hi");
}

#[test]
fn register_transfer() {
    assert_num(
        eval("", " LoadInt #3\n StoreReg r0\n LoadUndefined\n LoadReg r0"),
        3.0,
    );
    assert_num(
        eval(
            "",
            " LoadInt #3\n StoreReg r0\n Copy r0, r5\n LoadUndefined\n LoadReg r5",
        ),
        3.0,
    );
    // Locals start undefined (bytecode note 4.3).
    assert_eq!(eval("", " LoadReg r7"), Ok(HostValue::Undefined));
    // The receiver of a job is undefined.
    assert_eq!(eval("", " LoadReg this"), Ok(HostValue::Undefined));
}

// --- Conversions (0x0A-0x0F) -------------------------------------------------

#[test]
fn to_boolean() {
    // 7.1.2 ToBoolean.
    let cases = [
        (".const k0 string \"\"", " LoadConst k0", false),
        (".const k0 string \"a\"", " LoadConst k0", true),
        ("", " LoadInt #0", false),
        ("", " LoadInt #-1", true),
        (".const k0 number NaN", " LoadConst k0", false),
        (".const k0 number -0", " LoadConst k0", false),
        ("", " LoadNull", false),
        ("", " LoadUndefined", false),
        (
            "",
            " LoadNull\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1",
            true,
        ),
    ];
    for (c, b, want) in cases {
        assert_bool(eval(c, &format!("{b}\n ToBoolean")), want);
    }
}

#[test]
fn to_number_and_numeric() {
    // 7.1.4 ToNumber and 7.1.4.1.1 StringToNumber: white space is trimmed,
    // the empty string is 0, hex is allowed, junk is NaN.
    assert_num(
        eval(".const k0 string \" 12 \"", " LoadConst k0\n ToNumber s?"),
        12.0,
    );
    assert_num(
        eval(".const k0 string \"0x10\"", " LoadConst k0\n ToNumber s?"),
        16.0,
    );
    assert_num(
        eval(".const k0 string \"\"", " LoadConst k0\n ToNumber s?"),
        0.0,
    );
    assert_num(
        eval(".const k0 string \"1e\"", " LoadConst k0\n ToNumber s?"),
        f64::NAN,
    );
    assert_num(
        eval(
            ".const k0 string \"-Infinity\"",
            " LoadConst k0\n ToNumber s?",
        ),
        f64::NEG_INFINITY,
    );
    assert_num(eval("", " LoadUndefined\n ToNumber s?"), f64::NAN);
    assert_num(eval("", " LoadTrue\n ToNumber s?"), 1.0);
    // 7.1.3 ToNumeric of null is +0.
    assert_num(eval("", " LoadNull\n ToNumeric s?"), 0.0);
    // An object: OrdinaryToPrimitive (7.1.1.1) with hint number calls
    // valueOf first; host function 1 returns 42.
    let obj = ".const k0 key \"valueOf\"\n";
    let body = " LoadNull\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1\n StoreReg r1\n LoadInt #1\n StoreReg r2\n CallIntrinsic %CreateHostFunction, r2:1\n SetNamed r1, k0, s?\n LoadReg r1\n ToNumber s?";
    assert_num(eval(obj, body), 42.0);
    // Without valueOf or toString, ToPrimitive throws a TypeError.
    assert_error(
        eval(
            "",
            " LoadNull\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1\n ToNumber s?",
        ),
        ErrorKind::TypeError,
    );
}

#[test]
fn to_string() {
    // 7.1.17 ToString and 6.1.6.1.20 Number::toString.
    assert_str(eval("", " LoadInt #5\n ToString"), "5");
    assert_str(
        eval(".const k0 number 1e21", " LoadConst k0\n ToString"),
        "1e+21",
    );
    assert_str(
        eval(
            ".const k0 number 123456789012345680000",
            " LoadConst k0\n ToString",
        ),
        "123456789012345680000",
    );
    assert_str(eval(".const k0 number -0", " LoadConst k0\n ToString"), "0");
    assert_str(
        eval(".const k0 number 0.000001", " LoadConst k0\n ToString"),
        "0.000001",
    );
    assert_str(
        eval(".const k0 number 1e-7", " LoadConst k0\n ToString"),
        "1e-7",
    );
    assert_str(
        eval(".const k0 number NaN", " LoadConst k0\n ToString"),
        "NaN",
    );
    assert_str(eval("", " LoadNull\n ToString"), "null");
    assert_str(eval("", " LoadTrue\n ToString"), "true");
    // 0.1 + 0.2: the shortest digits that read back as the sum.
    let c = ".const k0 number 0.1\n.const k1 number 0.2";
    assert_str(
        eval(
            c,
            " LoadConst k0\n StoreReg r0\n LoadConst k1\n Add r0, s?\n ToString",
        ),
        "0.30000000000000004",
    );
}

#[test]
fn not_and_typeof() {
    // 13.5.7.1: `!` is the negation of ToBoolean.
    assert_bool(eval("", " LoadInt #0\n Not"), true);
    assert_bool(eval(".const k0 string \"x\"", " LoadConst k0\n Not"), false);
    // 13.5.3.1 typeof: Table 41.
    let t = |c: &str, b: &str| eval(c, &format!("{b}\n TypeOf"));
    assert_str(t("", " LoadUndefined"), "undefined");
    assert_str(t("", " LoadNull"), "object");
    assert_str(t("", " LoadInt #1"), "number");
    assert_str(t(".const k0 number 0.5", " LoadConst k0"), "number");
    assert_str(t(".const k0 string \"\"", " LoadConst k0"), "string");
    assert_str(t("", " LoadFalse"), "boolean");
    assert_str(
        t(
            "",
            " LoadNull\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1",
        ),
        "object",
    );
    assert_str(
        t(
            "",
            " LoadInt #9\n StoreReg r0\n CallIntrinsic %CreateHostFunction, r0:1",
        ),
        "function",
    );
}

// --- Arithmetic and bitwise (0x10-0x1F) ---------------------------------------

/// `lhs op rhs` with both operands as Number constants.
fn bin(op: &str, l: f64, r: f64) -> Result<HostValue, Outcome> {
    let c = format!(".const k0 number {}\n.const k1 number {}", lit(l), lit(r));
    eval(
        &c,
        &format!(" LoadConst k0\n StoreReg r0\n LoadConst k1\n {op} r0, s?"),
    )
}

fn lit(x: f64) -> String {
    if x.is_nan() {
        "NaN".into()
    } else if x.is_infinite() {
        if x > 0.0 {
            "Infinity".into()
        } else {
            "-Infinity".into()
        }
    } else if x == 0.0 && x.is_sign_negative() {
        "-0".into()
    } else {
        format!("{x:e}")
    }
}

#[test]
fn add() {
    // 6.1.6.1.7 Number::add.
    assert_num(
        eval("", " LoadInt #1\n StoreReg r0\n LoadInt #2\n Add r0, s?"),
        3.0,
    );
    // A small-integer sum past 2^30 − 1 is a heap Number, the same value.
    assert_num(
        eval(
            "",
            " LoadInt.x #1073741823\n StoreReg r0\n LoadInt #1\n Add r0, s?",
        ),
        1_073_741_824.0,
    );
    assert_num(bin("Add", -0.0, -0.0), -0.0);
    assert_num(bin("Add", -0.0, 0.0), 0.0);
    assert_num(bin("Add", f64::INFINITY, f64::NEG_INFINITY), f64::NAN);
    // 13.15.3 step 1.c: a String operand concatenates (ToString of the other).
    let c = ".const k0 string \"a\"";
    assert_str(
        eval(c, " LoadConst k0\n StoreReg r0\n LoadInt #1\n Add r0, s?"),
        "a1",
    );
    assert_str(
        eval(c, " LoadNull\n StoreReg r0\n LoadConst k0\n Add r0, s?"),
        "nulla",
    );
    // Long enough for a rope (ROPE_MIN_LENGTH 9), and a rope of ropes;
    // the value is the same as a flat concatenation.
    let c = ".const k0 string \"abcde\"\n.const k1 string \"fghij\"";
    assert_str(
        eval(
            c,
            " LoadConst k0\n StoreReg r0\n LoadConst k1\n Add r0, s?\n StoreReg r1\n Add r1, s?",
        ),
        "abcdefghijabcdefghij",
    );
    // Booleans and undefined go through ToNumber.
    assert_num(
        eval("", " LoadTrue\n StoreReg r0\n LoadInt #1\n Add r0, s?"),
        2.0,
    );
    assert_num(
        eval("", " LoadUndefined\n StoreReg r0\n LoadInt #1\n Add r0, s?"),
        f64::NAN,
    );
    // An object with toString only (host function 2 returns "obj"):
    // ToPrimitive default hint tries valueOf first, finds none, then toString.
    let c = ".const k0 key \"toString\"\n.const k1 string \"<\"";
    let body = " LoadNull\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1\n StoreReg r1\n LoadInt #2\n StoreReg r2\n CallIntrinsic %CreateHostFunction, r2:1\n SetNamed r1, k0, s?\n LoadConst k1\n StoreReg r3\n LoadReg r1\n Add r3, s?";
    assert_str(eval(c, body), "<obj");
}

#[test]
fn sub_mul_div_mod_exp() {
    // 6.1.6.1.8 Number::subtract, with ToNumber of a String (13.15.3).
    assert_num(
        eval("", " LoadInt #5\n StoreReg r0\n LoadInt #7\n Sub r0, s?"),
        -2.0,
    );
    assert_num(
        eval(
            ".const k0 string \"5\"",
            " LoadConst k0\n StoreReg r0\n LoadInt #2\n Sub r0, s?",
        ),
        3.0,
    );
    // 6.1.6.1.4 Number::multiply: the sign of a zero product.
    assert_num(
        eval("", " LoadInt #0\n StoreReg r0\n LoadInt #-1\n Mul r0, s?"),
        -0.0,
    );
    assert_num(
        eval("", " LoadInt #-3\n StoreReg r0\n LoadInt #4\n Mul r0, s?"),
        -12.0,
    );
    assert_num(
        eval(
            "",
            " LoadInt.x #65536\n StoreReg r0\n LoadInt.x #65536\n Mul r0, s?",
        ),
        4_294_967_296.0,
    );
    // 6.1.6.1.5 Number::divide.
    assert_num(
        eval("", " LoadInt #1\n StoreReg r0\n LoadInt #0\n Div r0, s?"),
        f64::INFINITY,
    );
    assert_num(
        eval("", " LoadInt #-1\n StoreReg r0\n LoadInt #0\n Div r0, s?"),
        f64::NEG_INFINITY,
    );
    assert_num(
        eval("", " LoadInt #0\n StoreReg r0\n LoadInt #0\n Div r0, s?"),
        f64::NAN,
    );
    assert_num(
        eval("", " LoadInt #7\n StoreReg r0\n LoadInt #2\n Div r0, s?"),
        3.5,
    );
    assert_num(
        eval("", " LoadInt #0\n StoreReg r0\n LoadInt #-5\n Div r0, s?"),
        -0.0,
    );
    // 6.1.6.1.6 Number::remainder: the sign of the dividend.
    assert_num(
        eval("", " LoadInt #5\n StoreReg r0\n LoadInt #3\n Mod r0, s?"),
        2.0,
    );
    assert_num(
        eval("", " LoadInt #-5\n StoreReg r0\n LoadInt #3\n Mod r0, s?"),
        -2.0,
    );
    assert_num(
        eval("", " LoadInt #-4\n StoreReg r0\n LoadInt #2\n Mod r0, s?"),
        -0.0,
    );
    assert_num(
        eval("", " LoadInt #5\n StoreReg r0\n LoadInt #0\n Mod r0, s?"),
        f64::NAN,
    );
    assert_num(bin("Mod", 5.5, 2.0), 1.5);
    assert_num(bin("Mod", 1.0, f64::INFINITY), 1.0);
    // 6.1.6.1.3 Number::exponentiate.
    assert_num(
        eval("", " LoadInt #2\n StoreReg r0\n LoadInt #10\n Exp r0, s?"),
        1024.0,
    );
    assert_num(
        eval("", " LoadInt #2\n StoreReg r0\n LoadInt #-1\n Exp r0, s?"),
        0.5,
    );
    assert_num(bin("Exp", f64::NAN, 0.0), 1.0);
    assert_num(bin("Exp", 1.0, f64::INFINITY), f64::NAN);
    assert_num(bin("Exp", -8.0, 1.0 / 3.0), f64::NAN);
    assert_num(bin("Exp", -0.0, -1.0), f64::NEG_INFINITY);
}

#[test]
fn bitwise_and_shifts() {
    // 6.1.6.1.17-19 NumberBitwiseOp on ToInt32 values.
    assert_num(
        eval("", " LoadInt #6\n StoreReg r0\n LoadInt #3\n BitAnd r0, s?"),
        2.0,
    );
    assert_num(
        eval("", " LoadInt #6\n StoreReg r0\n LoadInt #3\n BitOr r0, s?"),
        7.0,
    );
    assert_num(
        eval("", " LoadInt #6\n StoreReg r0\n LoadInt #3\n BitXor r0, s?"),
        5.0,
    );
    assert_num(bin("BitOr", 4_294_967_296.0 + 5.0, 0.0), 5.0);
    assert_num(bin("BitAnd", -1.5, 255.0), 255.0);
    // 6.1.6.1.9-11: shift counts are taken modulo 32.
    assert_num(
        eval("", " LoadInt #1\n StoreReg r0\n LoadInt #31\n Shl r0, s?"),
        -2_147_483_648.0,
    );
    assert_num(
        eval("", " LoadInt #1\n StoreReg r0\n LoadInt #32\n Shl r0, s?"),
        1.0,
    );
    assert_num(
        eval("", " LoadInt #-8\n StoreReg r0\n LoadInt #1\n Sar r0, s?"),
        -4.0,
    );
    assert_num(
        eval("", " LoadInt #-1\n StoreReg r0\n LoadInt #0\n Shr r0, s?"),
        4_294_967_295.0,
    );
    assert_num(
        eval("", " LoadInt #-1\n StoreReg r0\n LoadInt #28\n Shr r0, s?"),
        15.0,
    );
    assert_num(bin("Shl", 1.0, 33.5), 2.0);
}

#[test]
fn unary() {
    // 6.1.6.1.1 Number::unaryMinus: −0 for +0.
    assert_num(eval("", " LoadInt #0\n Neg s?"), -0.0);
    assert_num(
        eval(".const k0 string \"3\"", " LoadConst k0\n Neg s?"),
        -3.0,
    );
    assert_num(
        eval("", " LoadInt.x #-1073741824\n Neg s?"),
        1_073_741_824.0,
    );
    // 6.1.6.1.2 Number::bitwiseNOT.
    assert_num(eval("", " LoadInt #5\n BitNot s?"), -6.0);
    assert_num(
        eval(".const k0 number 4294967295", " LoadConst k0\n BitNot s?"),
        0.0,
    );
    // 13.4.4.1 and 13.4.5.1: ++ and -- add and subtract 1 (Number::add).
    assert_num(eval("", " LoadInt.x #1073741823\n Inc s?"), 1_073_741_824.0);
    assert_num(
        eval("", " LoadInt.x #-1073741824\n Dec s?"),
        -1_073_741_825.0,
    );
    assert_num(
        eval(".const k0 string \"x\"", " LoadConst k0\n Inc s?"),
        f64::NAN,
    );
    assert_num(eval("", " LoadNull\n Dec s?"), -1.0);
}

// --- Comparison (0x20-0x25) ----------------------------------------------------

#[test]
fn equality() {
    // 7.2.14 IsLooselyEqual.
    assert_bool(
        eval(
            "",
            " LoadNull\n StoreReg r0\n LoadUndefined\n LooseEq r0, s?",
        ),
        true,
    );
    assert_bool(
        eval(
            ".const k0 string \"1\"",
            " LoadConst k0\n StoreReg r0\n LoadInt #1\n LooseEq r0, s?",
        ),
        true,
    );
    assert_bool(
        eval("", " LoadTrue\n StoreReg r0\n LoadInt #1\n LooseEq r0, s?"),
        true,
    );
    assert_bool(
        eval("", " LoadNull\n StoreReg r0\n LoadInt #0\n LooseEq r0, s?"),
        false,
    );
    assert_bool(bin("LooseEq", f64::NAN, f64::NAN), false);
    assert_bool(
        eval(
            ".const k0 string \"\"",
            " LoadConst k0\n StoreReg r0\n LoadFalse\n LooseEq r0, s?",
        ),
        true,
    );
    // An object equals a primitive through ToPrimitive (step 11):
    // valueOf (host function 1) returns 42.
    let c = ".const k0 key \"valueOf\"";
    let body = " LoadNull\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1\n StoreReg r1\n LoadInt #1\n StoreReg r2\n CallIntrinsic %CreateHostFunction, r2:1\n SetNamed r1, k0, s?\n LoadInt #42\n LooseEq r1, s?";
    assert_bool(eval(c, body), true);
    // 7.2.15 IsStrictlyEqual: no conversions; NaN is unequal; ±0 equal;
    // strings by code units; objects by identity.
    assert_bool(
        eval(
            ".const k0 string \"1\"",
            " LoadConst k0\n StoreReg r0\n LoadInt #1\n StrictEq r0, s?",
        ),
        false,
    );
    assert_bool(bin("StrictEq", f64::NAN, f64::NAN), false);
    assert_bool(bin("StrictEq", 0.0, -0.0), true);
    assert_bool(bin("StrictEq", 1.5, 1.5), true);
    let c =
        ".const k0 string \"abcdefghij\"\n.const k1 string \"abcde\"\n.const k2 string \"fghij\"";
    assert_bool(
        eval(
            c,
            " LoadConst k1\n StoreReg r0\n LoadConst k2\n Add r0, s?\n StoreReg r1\n LoadConst k0\n StrictEq r1, s?",
        ),
        true,
    );
    let objs = " LoadNull\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1\n StoreReg r1\n CallIntrinsic %ObjectCreate, r0:1\n StrictEq r1, s?";
    assert_bool(eval("", objs), false);
    assert_bool(
        eval(
            "",
            " LoadNull\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1\n StoreReg r1\n StrictEq r1, s?",
        ),
        true,
    );
}

#[test]
fn relational() {
    // 7.2.13 IsLessThan and 13.10.1.
    assert_bool(
        eval("", " LoadInt #1\n StoreReg r0\n LoadInt #2\n Lt r0, s?"),
        true,
    );
    assert_bool(
        eval("", " LoadInt #1\n StoreReg r0\n LoadInt #2\n Gt r0, s?"),
        false,
    );
    assert_bool(
        eval("", " LoadInt #2\n StoreReg r0\n LoadInt #2\n Le r0, s?"),
        true,
    );
    assert_bool(
        eval("", " LoadInt #2\n StoreReg r0\n LoadInt #2\n Ge r0, s?"),
        true,
    );
    // NaN: IsLessThan is undefined, so every comparison is false.
    for op in ["Lt", "Gt", "Le", "Ge"] {
        assert_bool(bin(op, f64::NAN, 1.0), false);
    }
    // Two strings compare by code units: "10" < "9".
    let c = ".const k0 string \"10\"\n.const k1 string \"9\"";
    assert_bool(
        eval(c, " LoadConst k0\n StoreReg r0\n LoadConst k1\n Lt r0, s?"),
        true,
    );
    // A string and a number compare as numbers: 10 < "9" is false.
    assert_bool(
        eval(c, " LoadInt #10\n StoreReg r0\n LoadConst k1\n Lt r0, s?"),
        false,
    );
    // A prefix is less (step 3, the lexicographic order).
    let c = ".const k0 string \"ab\"\n.const k1 string \"abc\"";
    assert_bool(
        eval(c, " LoadConst k0\n StoreReg r0\n LoadConst k1\n Lt r0, s?"),
        true,
    );
    assert_bool(
        eval(c, " LoadConst k1\n StoreReg r0\n LoadConst k0\n Ge r0, s?"),
        true,
    );
    assert_bool(bin("Lt", -0.0, 0.0), false);
    assert_bool(bin("Le", f64::NEG_INFINITY, -1e308), true);
}

// --- Control (0x26-0x2C) -------------------------------------------------------

#[test]
fn jumps_and_branches() {
    // 14.6.2 (if): a forward jump skips its target's predecessors.
    assert_num(
        eval("", " LoadInt #1\n Jump @l\n LoadInt #2\nl: Return"),
        1.0,
    );
    // BranchTrue / BranchFalse take exactly the Booleans.
    assert_num(
        eval(
            "",
            " LoadTrue\n BranchTrue @t\n LoadInt #0\n Return\nt: LoadInt #1",
        ),
        1.0,
    );
    assert_num(
        eval(
            "",
            " LoadInt #1\n BranchTrue @t\n LoadInt #0\n Return\nt: LoadInt #1",
        ),
        0.0,
    );
    assert_num(
        eval(
            "",
            " LoadFalse\n BranchFalse @t\n LoadInt #0\n Return\nt: LoadInt #1",
        ),
        1.0,
    );
    // BranchTruthy / BranchFalsy take ToBoolean (7.1.2).
    let c = ".const k0 string \"x\"";
    assert_num(
        eval(
            c,
            " LoadConst k0\n BranchTruthy @t\n LoadInt #0\n Return\nt: LoadInt #1",
        ),
        1.0,
    );
    assert_num(
        eval(
            "",
            " LoadInt #0\n BranchFalsy @t\n LoadInt #0\n Return\nt: LoadInt #1",
        ),
        1.0,
    );
    assert_num(
        eval(
            "",
            " LoadNull\n BranchTruthy @t\n LoadInt #7\n Return\nt: LoadInt #1",
        ),
        7.0,
    );
}

#[test]
fn loops_and_return() {
    // 14.7.4.2 (for): sum = 0; for (i = 1; i <= 10; i++) sum += i.
    let body = " LoadInt #0\n StoreReg r0\n LoadInt #1\n StoreReg r1\n\
        top: LoadInt #10\n Le r1, s?\n BranchFalse @done\n LoadReg r1\n Add r0, s?\n StoreReg r0\n LoadReg r1\n Inc s?\n StoreReg r1\n LoopBack @top, s?\n\
        done: LoadReg r0";
    assert_num(eval("", body), 55.0);
    // A Return in the middle ends the unit with the accumulator.
    assert_num(eval("", " LoadInt #4\n Return\n LoadInt #5"), 4.0);
}

#[test]
fn interrupt_stops_a_loop() {
    // The interrupt check at LoopBack (bytecode note 4.10).
    let text = ".function t params=0 registers=1\nl: LoopBack @l, s?\n.end\n";
    let r = with_agent(hg_heap::Stress::None, 1 << 20, |m, a, h| {
        a.pacer().fuel = Some(100);
        run_one(m, a, h, text)
    });
    assert_eq!(
        r.value,
        Err(Outcome::Other(hg_js_runtime::JobError::Interrupted))
    );
}

// --- Properties (0x2D-0x30) ----------------------------------------------------

const OBJ: &str = " LoadNull\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1\n StoreReg r1\n";

#[test]
fn named_properties() {
    // 10.1.9 OrdinarySet then 10.1.8 OrdinaryGet.
    let c = ".const k0 key \"x\"\n.const k1 key \"y\"";
    assert_num(
        eval(
            c,
            &format!(
                "{OBJ} LoadInt #5\n SetNamed r1, k0, s?\n LoadUndefined\n GetNamed r1, k0, s?"
            ),
        ),
        5.0,
    );
    // A missing property is undefined.
    assert_eq!(
        eval(c, &format!("{OBJ} GetNamed r1, k1, s?")),
        Ok(HostValue::Undefined)
    );
    // An assignment's value is the right-hand side (13.15.2).
    assert_num(
        eval(c, &format!("{OBJ} LoadInt #5\n SetNamed r1, k0, s?")),
        5.0,
    );
    // Overwriting keeps one property.
    assert_num(
        eval(
            c,
            &format!(
                "{OBJ} LoadInt #5\n SetNamed r1, k0, s?\n LoadInt #6\n SetNamed r1, k0, s?\n GetNamed r1, k0, s?"
            ),
        ),
        6.0,
    );
    // The prototype chain: Object.create(p) sees p's x.
    let chain = format!(
        "{OBJ} LoadInt #5\n SetNamed r1, k0, s?\n CallIntrinsic %ObjectCreate, r1:1\n StoreReg r2\n GetNamed r2, k0, s?"
    );
    assert_num(eval(c, &chain), 5.0);
    // Setting on the child adds an own property and leaves p alone.
    let shadow = format!("{chain}\n LoadInt #9\n SetNamed r2, k0, s?\n GetNamed r1, k0, s?");
    assert_num(eval(c, &shadow), 5.0);
    // Many properties grow the table.
    let mut many = String::from(OBJ);
    let mut consts = String::new();
    for i in 0..20 {
        consts.push_str(&format!(".const k{i} key \"p{i}\"\n"));
        many.push_str(&format!(" LoadInt #{i}\n SetNamed r1, k{i}, s?\n"));
    }
    many.push_str(" GetNamed r1, k13, s?");
    assert_num(eval(&consts, &many), 13.0);
    // GetValue step 3.a: ToObject throws on undefined and null.
    assert_error(
        eval(c, " LoadUndefined\n StoreReg r0\n GetNamed r0, k0, s?"),
        ErrorKind::TypeError,
    );
    assert_error(
        eval(
            c,
            " LoadNull\n StoreReg r0\n LoadInt #1\n SetNamed r0, k0, s?",
        ),
        ErrorKind::TypeError,
    );
}

#[test]
fn primitive_bases() {
    // 10.4.3: a String's own `length` and index properties.
    let c = ".const k0 string \"hello\"\n.const k1 key \"length\"\n.const k2 key \"1\"\n.const k3 key \"x\"";
    assert_num(
        eval(c, " LoadConst k0\n StoreReg r0\n GetNamed r0, k1, s?"),
        5.0,
    );
    assert_str(
        eval(c, " LoadConst k0\n StoreReg r0\n GetNamed r0, k2, s?"),
        "e",
    );
    // JS-M0 has no String.prototype: another name is undefined.
    assert_eq!(
        eval(c, " LoadConst k0\n StoreReg r0\n GetNamed r0, k3, s?"),
        Ok(HostValue::Undefined)
    );
    // PutValue on a primitive: the receiver is not an object, so [[Set]]
    // returns false: ignored in sloppy code, a TypeError in strict code
    // (6.2.5.6 step 3.e).
    let c = ".const k0 key \"x\"";
    assert_num(
        eval(
            c,
            " LoadInt #1\n StoreReg r0\n LoadInt #2\n SetNamed r0, k0, s?",
        ),
        2.0,
    );
    assert_error(
        eval_strict(
            c,
            " LoadInt #1\n StoreReg r0\n LoadInt #2\n SetNamed r0, k0, s?",
        ),
        ErrorKind::TypeError,
    );
}

#[test]
fn keyed_properties() {
    // GetValue / PutValue step 3.c: ToPropertyKey of the key value, so the
    // number 1 and the string "1" are one key.
    let c = ".const k0 string \"1\"";
    assert_num(
        eval(
            c,
            &format!(
                "{OBJ} LoadInt #1\n StoreReg r2\n LoadInt #5\n SetKeyed r1, r2, s?\n LoadConst k0\n GetKeyed r1, s?"
            ),
        ),
        5.0,
    );
    // A string indexed by a number.
    let c = ".const k0 string \"abc\"";
    assert_str(
        eval(
            c,
            " LoadConst k0\n StoreReg r0\n LoadInt #2\n GetKeyed r0, s?",
        ),
        "c",
    );
    assert_eq!(
        eval(
            c,
            " LoadConst k0\n StoreReg r0\n LoadInt #3\n GetKeyed r0, s?"
        ),
        Ok(HostValue::Undefined)
    );
    // A key that is an object: ToPrimitive with hint string calls toString
    // (host function 2 returns "obj").
    let c = ".const k0 key \"toString\"\n.const k1 key \"obj\"";
    let body = format!(
        "{OBJ} LoadInt #2\n StoreReg r2\n CallIntrinsic %CreateHostFunction, r2:1\n SetNamed r1, k0, s?\n\
         CallIntrinsic %ObjectCreate, r0:1\n StoreReg r3\n LoadInt #8\n SetKeyed r3, r1, s?\n GetNamed r3, k1, s?"
    );
    assert_num(eval(c, &body), 8.0);
    // A throwing toString: the host's value is the completion (step 3.c's ?).
    let c = ".const k0 key \"toString\"";
    let body = format!(
        "{OBJ} LoadInt #3\n StoreReg r2\n CallIntrinsic %CreateHostFunction, r2:1\n SetNamed r1, k0, s?\n LoadReg r1\n GetKeyed r1, s?"
    );
    assert_eq!(
        eval(c, &body),
        Err(Outcome::Thrown(HostValue::String(
            "boom".encode_utf16().collect()
        )))
    );
    // Undefined base.
    assert_error(
        eval(
            "",
            " LoadUndefined\n StoreReg r0\n LoadInt #0\n GetKeyed r0, s?",
        ),
        ErrorKind::TypeError,
    );
    assert_error(
        eval(
            "",
            " LoadNull\n StoreReg r0\n LoadInt #0\n StoreReg r1\n SetKeyed r0, r1, s?",
        ),
        ErrorKind::TypeError,
    );
}

// --- CallIntrinsic (0x31) ------------------------------------------------------

#[test]
fn intrinsics() {
    // 20.1.2.2 Object.create: a prototype that is not an object or null.
    assert_error(
        eval(
            "",
            " LoadInt #1\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1",
        ),
        ErrorKind::TypeError,
    );
    // 26.1.1.1 WeakRef(target): a primitive cannot be held weakly.
    assert_error(
        eval(
            "",
            " LoadInt #1\n StoreReg r0\n CallIntrinsic %WeakRefCreate, r0:1",
        ),
        ErrorKind::TypeError,
    );
    // 26.1.3.2 deref: the target while it lives.
    let body = format!(
        "{OBJ} CallIntrinsic %WeakRefCreate, r1:1\n StoreReg r2\n CallIntrinsic %WeakRefDeref, r2:1\n StrictEq r1, s?"
    );
    assert_bool(eval("", &body), true);
    assert_error(
        eval(
            "",
            " LoadInt #1\n StoreReg r0\n CallIntrinsic %WeakRefDeref, r0:1",
        ),
        ErrorKind::TypeError,
    );
    // 26.2.1.1 FinalizationRegistry: the callback must be callable.
    assert_error(
        eval(
            "",
            &format!("{OBJ} CallIntrinsic %FinalizationRegistryCreate, r1:1"),
        ),
        ErrorKind::TypeError,
    );
    // 26.2.3.2 register: target and held value may not be the same (step 4),
    // and an unregister token must be holdable or undefined (step 5).
    let reg = format!(
        "{OBJ} LoadInt #7\n StoreReg r2\n CallIntrinsic %CreateHostFunction, r2:1\n StoreReg r2\n\
         CallIntrinsic %FinalizationRegistryCreate, r2:1\n StoreReg r2\n Copy r1, r3\n"
    );
    assert_error(
        eval(
            "",
            &format!(
                "{reg} Copy r1, r4\n LoadUndefined\n StoreReg r5\n CallIntrinsic %FinalizationRegistryRegister, r2:4"
            ),
        ),
        ErrorKind::TypeError,
    );
    assert_error(
        eval(
            "",
            &format!(
                "{reg} LoadInt #1\n StoreReg r4\n LoadInt #2\n StoreReg r5\n CallIntrinsic %FinalizationRegistryRegister, r2:4"
            ),
        ),
        ErrorKind::TypeError,
    );
    // A token: register returns undefined, unregister finds the cell once
    // (26.2.3.3: true, then false).
    let with_token = format!(
        "{reg} LoadInt #1\n StoreReg r4\n CallIntrinsic %ObjectCreate, r0:1\n StoreReg r5\n CallIntrinsic %FinalizationRegistryRegister, r2:4\n StoreReg r7\n Copy r5, r3\n"
    );
    assert_eq!(
        eval("", &format!("{with_token} LoadReg r7")),
        Ok(HostValue::Undefined)
    );
    assert_bool(
        eval(
            "",
            &format!("{with_token} CallIntrinsic %FinalizationRegistryUnregister, r2:2"),
        ),
        true,
    );
    assert_bool(
        eval(
            "",
            &format!(
                "{with_token} CallIntrinsic %FinalizationRegistryUnregister, r2:2\n CallIntrinsic %FinalizationRegistryUnregister, r2:2"
            ),
        ),
        false,
    );
    // 21.3.2.27 Math.random: the realm stream's 53 high bits times 2^−53.
    let mut draw = SplitMix(SEED);
    let want = f64::from_bits(0) + ((draw.next() >> 11) as f64) / 9_007_199_254_740_992.0;
    assert_num(eval("", " CallIntrinsic %MathRandom, r0:0"), want);
    // Date.now: floor(HostSystemUTCEpochNanoseconds / 10^6).
    assert_num(
        eval("", " CallIntrinsic %DateNow, r0:0"),
        1_700_000_000_123.0,
    );
    // A host function's id must be a small integer.
    assert_error(
        eval(
            "",
            " LoadNull\n StoreReg r0\n CallIntrinsic %CreateHostFunction, r0:1",
        ),
        ErrorKind::TypeError,
    );
}

trait Next {
    fn next(&mut self) -> u64;
}
impl Next for SplitMix {
    fn next(&mut self) -> u64 {
        hg_substrate::RandomStream::next_u64(self)
    }
}
