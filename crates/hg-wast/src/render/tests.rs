//! Rendering tests.
//!
//! The important ones are round trips. Rendering is only worth doing if
//! what comes back out means what went in, and for a float that is a
//! claim about every bit pattern, not about a few examples.

use super::*;
use crate::script::parse;
use crate::value::{float32 as read32, float64 as read64};

/// Parse, render, parse again: the commands must be equal.
fn round_trip(src: &str) -> String {
    let first = parse(src).unwrap_or_else(|e| panic!("should parse: {}", e.display(src)));
    let text = script(src, &first);
    let second = parse(&text)
        .unwrap_or_else(|e| panic!("the rendering should parse: {}\n{text}", e.display(&text)));
    assert_eq!(
        first.len(),
        second.len(),
        "a command was lost in rendering\n{text}"
    );
    for (a, b) in first.iter().zip(&second) {
        assert_eq!(a.keyword(), b.keyword(), "\n{text}");
    }
    text
}

/// Every `f32` bit pattern that matters renders to a literal that reads
/// back to the same bits. A float rendering that rounds would be a
/// silent corruption of the test being run.
#[test]
fn every_f32_shape_round_trips() {
    let mut cases: Vec<u32> = vec![
        0,           // +0
        0x8000_0000, // -0
        0x0000_0001, // the smallest subnormal
        0x007f_ffff, // the largest subnormal
        0x0080_0000, // the smallest normal
        0x7f7f_ffff, // the largest finite
        0x7f80_0000, // +inf
        0xff80_0000, // -inf
        0x7fc0_0000, // the canonical NaN
        0x7f80_0001, // a NaN with the smallest payload
        0x7fff_ffff, // a NaN with the largest payload
        0x3f80_0000, // 1.0
        0xbf80_0000, // -1.0
        0x4049_0fdb, // a value with a full mantissa
    ];
    // A spread of ordinary values as well, so the exponent and the
    // mantissa are both exercised across their ranges.
    for e in 0..=254u32 {
        cases.push((e << 23) | 0x0055_5555);
    }
    for bits in cases {
        let text = float32(bits);
        assert_eq!(
            read32(&text),
            Ok(bits),
            "{bits:#010x} rendered as {text} and read back differently"
        );
    }
}

#[test]
fn every_f64_shape_round_trips() {
    let mut cases: Vec<u64> = vec![
        0,
        0x8000_0000_0000_0000,
        1,
        0x000f_ffff_ffff_ffff,
        0x0010_0000_0000_0000,
        0x7fef_ffff_ffff_ffff,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_0000,
        0x7ff0_0000_0000_0001,
        0x3ff0_0000_0000_0000,
    ];
    for e in 0..=2046u64 {
        cases.push((e << 52) | 0x0005_5555_5555_5555);
    }
    for bits in cases {
        let text = float64(bits);
        assert_eq!(
            read64(&text),
            Ok(bits),
            "{bits:#018x} rendered as {text} and read back differently"
        );
    }
}

/// Two different NaNs must not render alike. Writing a bare `nan` for
/// all of them would quietly turn a payload test into nothing.
#[test]
fn nans_with_different_payloads_render_differently() {
    assert_ne!(float32(0x7fc0_0000), float32(0x7f80_0001));
    assert_ne!(
        float64(0x7ff8_0000_0000_0000),
        float64(0x7ff0_0000_0000_0001)
    );
    assert_eq!(read32(&float32(0x7f80_0001)), Ok(0x7f80_0001));
    // And a sign on a NaN survives.
    assert_eq!(read32(&float32(0xffc0_0000)), Ok(0xffc0_0000));
}

#[test]
fn a_module_is_rendered_as_its_own_source() {
    let module_text = r#"(module $m (func) (; ) ;) (data ")" "\00"))"#;
    let src = format!(r#"{module_text} (invoke "f")"#);
    let text = round_trip(&src);
    assert!(
        text.contains(module_text),
        "the module should go out byte for byte: {text}"
    );
}

#[test]
fn a_whole_script_round_trips() {
    let src = r#"
(module $m (func (export "f") (result i32) (i32.const 1)))
(register "m" $m)
(invoke $m "f" (i32.const -1) (f64.const 0x1p-1) (ref.extern 2))
(get "g")
(assert_return (invoke "f") (i32.const 42))
(assert_return (invoke "f") (f32.const nan:canonical) (ref.func))
(assert_return (invoke "f") (either (i32.const 1) (i32.const 2)))
(assert_return (invoke "f") (v128.const i32x4 1 2 3 4))
(assert_return (invoke "f") (v128.const f32x4 1 nan:canonical 3 4))
(assert_trap (invoke "f") "unreachable")
(assert_trap (module (func)) "oops")
(assert_exhaustion (invoke "f") "call stack exhausted")
(assert_exception (invoke "f"))
(assert_malformed (module quote "(") "unexpected")
(assert_invalid (module (func)) "type mismatch")
(assert_unlinkable (module (func)) "unknown import")
(module definition $d (func))
(module instance $i $d)
"#;
    round_trip(src);
}

/// A name is bytes, not a word: a quotation mark, a backslash, a
/// control character or a character outside ASCII must come back as
/// itself.
#[test]
fn awkward_names_survive() {
    let src = "(register \"a\\22b\\5cc\\00d\\u{e9}\")";
    let text = round_trip(src);
    let again = parse(&text).expect("should parse");
    let (Cmd::Register { name: a, .. }, Cmd::Register { name: b, .. }) =
        (&parse(src).expect("should parse")[0], &again[0])
    else {
        panic!("two registers");
    };
    assert_eq!(a, b, "{text}");
}

/// The values an action carries round-trip through a rendering too,
/// which is what makes the rendering usable as evidence.
#[test]
fn an_actions_arguments_survive() {
    let src = r#"(invoke "f" (i32.const 0xffffffff) (i64.const -1) (f32.const 0x1p-149) (f64.const -0x0p+0) (v128.const i8x16 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15) (ref.null extern) (ref.host 7))"#;
    let first = parse(src).expect("should parse");
    let text = script(src, &first);
    let second = parse(&text).expect("the rendering should parse");
    let (Cmd::Action(Action::Invoke { args: a, .. }), Cmd::Action(Action::Invoke { args: b, .. })) =
        (&first[0], &second[0])
    else {
        panic!("two invokes");
    };
    assert_eq!(a, b, "{text}");
}

/// The patterns an assertion carries survive as well, NaN patterns and
/// `either` included.
#[test]
fn an_assertions_patterns_survive() {
    let src = r#"(assert_return (invoke "f") (f32.const nan:canonical) (f64.const nan:arithmetic) (v128.const f64x2 1 nan:canonical) (either (ref.func) (ref.null)) (i32.const 7))"#;
    let first = parse(src).expect("should parse");
    let text = script(src, &first);
    let second = parse(&text).expect("the rendering should parse");
    let (Cmd::AssertReturn { expected: a, .. }, Cmd::AssertReturn { expected: b, .. }) =
        (&first[0], &second[0])
    else {
        panic!("two assertions");
    };
    assert_eq!(a, b, "{text}");
}
