//! Value and pattern decoding tests.

use super::*;
use crate::lex::{Kind, Lexer};

/// The bytes of the first string token in `src`.
fn bytes(src: &str) -> Vec<u8> {
    let toks = Lexer::new(src).tokens().expect("should lex");
    let t = toks.iter().find(|t| t.kind == Kind::Str).expect("a string");
    string_bytes(src, t.span).expect("should decode")
}

#[test]
fn escapes_decode_to_the_bytes_they_name() {
    assert_eq!(bytes(r#""abc""#), b"abc");
    assert_eq!(bytes(r#""\t\n\r\"\'\\""#), b"\t\n\r\"'\\");
    assert_eq!(bytes(r#""\00asm""#), b"\0asm");
    assert_eq!(bytes(r#""\01\00\00\00""#), [1, 0, 0, 0]);
    assert_eq!(bytes(r#""\ff""#), [0xff]);
}

/// `\u{…}` names a character, which becomes its UTF-8; `\HH` names a
/// byte, which does not. The two must not be confused: a module's
/// binary form is built out of the second.
#[test]
fn a_unicode_escape_is_utf8_and_a_byte_escape_is_a_byte() {
    assert_eq!(bytes(r#""\u{41}""#), b"A");
    assert_eq!(bytes(r#""\u{e9}""#), "é".as_bytes());
    assert_eq!(bytes(r#""\e9""#), [0xe9]);
    assert_eq!(bytes(r#""\u{1f600}""#), "\u{1f600}".as_bytes());
}

/// A module's bytes need not be text, which is the point of the form:
/// `(module binary …)` exists to hand an engine something malformed.
#[test]
fn a_string_that_is_not_text_still_decodes_to_bytes() {
    assert_eq!(bytes(r#""\ff\fe""#), [0xff, 0xfe]);
    let toks = Lexer::new(r#""\ff""#).tokens().expect("should lex");
    assert!(string_text(r#""\ff""#, toks[0].span).is_err());
}

#[test]
fn integers_read_in_both_bases_with_separators() {
    assert_eq!(int("0", 32), Ok(0));
    assert_eq!(int("42", 32), Ok(42));
    assert_eq!(int("+42", 32), Ok(42));
    assert_eq!(int("0x2a", 32), Ok(42));
    assert_eq!(int("0xff_ff_ff_ff", 32), Ok(0xffff_ffff));
    assert_eq!(int("1_000", 32), Ok(1000));
}

/// The same value may be written in the signed or the unsigned range,
/// and both give the same two's-complement bits.
#[test]
fn a_negative_integer_is_the_same_value_as_its_unsigned_spelling() {
    assert_eq!(int("-1", 32), int("0xffffffff", 32));
    assert_eq!(int("-1", 64), int("0xffffffffffffffff", 64));
    assert_eq!(int("-2147483648", 32), Ok(0x8000_0000));
    assert_eq!(int("-128", 8), Ok(0x80));
    assert_eq!(int("255", 8), Ok(0xff));
}

#[test]
fn an_integer_outside_its_type_is_refused() {
    assert!(int("4294967296", 32).is_err());
    assert!(int("-2147483649", 32).is_err());
    assert!(int("256", 8).is_err());
    assert!(int("-129", 8).is_err());
    assert!(int("", 32).is_err());
    assert!(int("_1", 32).is_err());
    assert!(int("1_", 32).is_err());
    assert!(int("0x", 32).is_err());
}

#[test]
fn the_float_literals_that_are_not_numbers() {
    assert_eq!(float32("inf"), Ok(0x7f80_0000));
    assert_eq!(float32("-inf"), Ok(0xff80_0000));
    assert_eq!(float64("inf"), Ok(0x7ff0_0000_0000_0000));
    // `nan` is the canonical one: the payload's top bit alone.
    assert_eq!(float32("nan"), Ok(0x7fc0_0000));
    assert_eq!(float64("nan"), Ok(0x7ff8_0000_0000_0000));
    assert_eq!(float32("-nan"), Ok(0xffc0_0000));
    // An explicit payload is kept exactly, which is what makes the
    // float tests able to tell NaNs apart at all.
    assert_eq!(float32("nan:0x1"), Ok(0x7f80_0001));
    assert_eq!(float32("nan:0x200000"), Ok(0x7fa0_0000));
    assert_eq!(float64("nan:0x1"), Ok(0x7ff0_0000_0000_0001));
    assert!(float32("nan:0x0").is_err(), "a zero payload is an infinity");
    assert!(float32("nan:0x800000").is_err(), "the payload must fit");
}

/// A sign is not a factor: `-0` is a value in its own right, and
/// multiplying it away would lose a case the suite tests directly.
#[test]
fn negative_zero_survives() {
    assert_eq!(float32("-0"), Ok(0x8000_0000));
    assert_eq!(float32("-0.0"), Ok(0x8000_0000));
    assert_eq!(float32("-0x0p0"), Ok(0x8000_0000));
    assert_eq!(float64("-0"), Ok(0x8000_0000_0000_0000));
    assert_eq!(float32("0"), Ok(0));
}

#[test]
fn decimal_floats_decode() {
    assert_eq!(float32("1.0"), Ok(1.0f32.to_bits()));
    assert_eq!(float64("1.0"), Ok(1.0f64.to_bits()));
    assert_eq!(float64("1.234567"), Ok(1.234567f64.to_bits()));
    assert_eq!(float64("1e10"), Ok(1e10f64.to_bits()));
    assert_eq!(float64("1_0.0_1e1_0"), Ok(10.01e10f64.to_bits()));
    assert_eq!(float64("-1.5e-3"), Ok((-1.5e-3f64).to_bits()));
}

#[test]
fn hexadecimal_floats_decode() {
    assert_eq!(float64("0x1p0"), Ok(1.0f64.to_bits()));
    assert_eq!(float64("0x1.8p1"), Ok(3.0f64.to_bits()));
    assert_eq!(float64("0x10"), Ok(16.0f64.to_bits()));
    assert_eq!(float64("0x1p-1"), Ok(0.5f64.to_bits()));
    assert_eq!(float32("0x1.921fb6p+1"), Ok(0x4049_0fdb));
    // The largest finite double and the smallest normal one.
    assert_eq!(float64("0x1.fffffffffffffp+1023"), Ok(f64::MAX.to_bits()));
    assert_eq!(float64("0x1p-1022"), Ok(f64::MIN_POSITIVE.to_bits()));
    // Subnormals, down to the last bit.
    assert_eq!(float64("0x0.0000000000001p-1022"), Ok(1));
    assert_eq!(float32("0x1p-149"), Ok(1));
}

/// Beyond the largest finite value the specified result is an infinity,
/// and below the smallest subnormal it is a zero.
#[test]
fn hexadecimal_floats_overflow_and_underflow_as_specified() {
    assert_eq!(float64("0x1p+1024"), Ok(f64::INFINITY.to_bits()));
    assert_eq!(float32("0x1p+128"), Ok(f32::INFINITY.to_bits()));
    assert_eq!(float64("-0x1p+1024"), Ok(f64::NEG_INFINITY.to_bits()));
    assert_eq!(float32("0x1p-200"), Ok(0));
}

/// Rounding is to nearest, ties to even — at the tie itself, and either
/// side of it.
#[test]
fn rounding_at_a_tie_goes_to_even() {
    // 2^53 + 1 is not representable; the tie rounds to the even
    // neighbour, 2^53.
    assert_eq!(
        float64("0x1.00000000000008p+53"),
        Ok((9007199254740992.0f64).to_bits())
    );
    // 2^53 + 3 ties the other way, to 2^53 + 4.
    assert_eq!(
        float64("0x1.00000000000018p+53"),
        Ok((9007199254740996.0f64).to_bits())
    );
    // Just over a tie rounds up whatever the parity.
    assert_eq!(
        float64("0x1.000000000000081p+53"),
        Ok((9007199254740994.0f64).to_bits())
    );
}

/// Digits past the significand's width still decide a tie, so they
/// cannot simply be dropped. Without the sticky bit the first of these
/// would round down and give the wrong double.
#[test]
fn digits_below_the_significand_still_break_a_tie() {
    let with_tail = float64("0x1.00000000000008000000000000001p+53").expect("decodes");
    let exact_tie = float64("0x1.00000000000008p+53").expect("decodes");
    assert_ne!(with_tail, exact_tie);
    assert_eq!(with_tail, (9007199254740994.0f64).to_bits());
}

/// A differential inside the crate: wherever a hexadecimal literal's
/// significand fits in the format's precision and its exponent is in
/// range, the value is exactly `significand × 2^exponent` and can be
/// computed by a second, trivially correct route. The general path,
/// which rounds, must agree with it everywhere the simple one applies.
#[test]
fn the_rounding_path_agrees_with_exact_scaling_where_both_apply() {
    let mut checked = 0;
    for sig in [
        1u64,
        2,
        3,
        5,
        7,
        0xff,
        0x1234,
        0x1f_ffff,
        0x1f_ffff_ffff_ffff,
    ] {
        for exp in [-60i32, -30, -8, -1, 0, 1, 8, 30, 60] {
            for neg in [false, true] {
                let text = format!("{}0x{sig:x}p{exp}", if neg { "-" } else { "" });
                let got = float64(&text).expect("decodes");
                let want = (sig as f64) * (exp as f64).exp2();
                let want = if neg { -want } else { want };
                assert_eq!(
                    got,
                    want.to_bits(),
                    "{text}: {got:#018x} against {:#018x}",
                    want.to_bits()
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 9 * 9 * 2, "the grid shrank");
}

/// The same, for the narrower format, where rounding bites sooner.
#[test]
fn the_rounding_path_agrees_with_exact_scaling_for_f32() {
    for sig in [1u32, 3, 0xff, 0xff_ffff] {
        for exp in [-30i32, -8, 0, 8, 30] {
            let text = format!("0x{sig:x}p{exp}");
            let got = float32(&text).expect("decodes");
            let want = (sig as f32) * (exp as f32).exp2();
            assert_eq!(got, want.to_bits(), "{text}");
        }
    }
}

#[test]
fn a_float_that_is_not_a_number_is_refused() {
    assert!(float64("").is_err());
    assert!(float64("0x").is_err());
    assert!(float64("0xp1").is_err());
    assert!(float64("1.0e").is_err());
    assert!(float64("_1.0").is_err());
    assert!(float64("nan:").is_err());
}

#[test]
fn the_named_types_and_shapes_round_trip() {
    for t in [NumType::I32, NumType::I64, NumType::F32, NumType::F64] {
        assert_eq!(NumType::from_name(t.name()), Some(t));
    }
    for s in [
        Shape::I8x16,
        Shape::I16x8,
        Shape::I32x4,
        Shape::I64x2,
        Shape::F32x4,
        Shape::F64x2,
    ] {
        assert_eq!(Shape::from_name(s.name()), Some(s));
        assert_eq!(s.lanes() * (s.lane_bits() as usize), 128);
    }
    assert_eq!(NumType::from_name("v128"), None);
    assert_eq!(Shape::from_name("i32"), None);
}
