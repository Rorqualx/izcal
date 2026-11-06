//! Unit tests. std's float parsing and formatting appear here only as a
//! black-box oracle for cross-checking (never in the implementation).

use crate::big::Big;
use crate::dd::Dd;
use crate::{
    exponentiate, number_to_string, parse_json_number, remainder, string_to_number, to_int32,
    to_uint32,
};

/// A deterministic xorshift64 stream (no rand crate, no ambient entropy).
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    /// A finite double from random bits.
    fn finite(&mut self) -> f64 {
        loop {
            let x = f64::from_bits(self.next());
            if x.is_finite() {
                return x;
            }
        }
    }
}

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn s2n(s: &str) -> f64 {
    string_to_number(&utf16(s))
}

fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}

/// Distance in ulps between two finite doubles of the same sign.
fn ulps(a: f64, b: f64) -> u64 {
    let (a, b) = (a.to_bits() as i64, b.to_bits() as i64);
    (a - b).unsigned_abs()
}

// ---------------------------------------------------------------- toString

#[test]
fn to_string_spec_cases() {
    let cases: &[(f64, &str)] = &[
        (f64::NAN, "NaN"),
        (0.0, "0"),
        (-0.0, "0"),
        (f64::INFINITY, "Infinity"),
        (f64::NEG_INFINITY, "-Infinity"),
        (1.0, "1"),
        (-1.0, "-1"),
        (0.1, "0.1"),
        (0.30000000000000004, "0.30000000000000004"),
        (1.5, "1.5"),
        (100.0, "100"),
        (123.456, "123.456"),
        (1e20, "100000000000000000000"),
        (1e21, "1e+21"),
        (1.5e21, "1.5e+21"),
        (123456789012345680000.0, "123456789012345680000"),
        (1e-6, "0.000001"),
        (0.000001, "0.000001"),
        (1.2e-6, "0.0000012"),
        (1e-7, "1e-7"),
        (1.5e-7, "1.5e-7"),
        (123e-20, "1.23e-18"),
        (9007199254740992.0, "9007199254740992"),
        (5e-324, "5e-324"),
        (-5e-324, "-5e-324"),
        (1.7976931348623157e308, "1.7976931348623157e+308"),
        (2.2250738585072014e-308, "2.2250738585072014e-308"),
        (2.225073858507201e-308, "2.225073858507201e-308"),
        (1e23, "1e+23"),
        (9.5367431640625e-7, "9.5367431640625e-7"),
        (4.35, "4.35"),
        (0.5, "0.5"),
        (2.0f64.powi(-1022), "2.2250738585072014e-308"),
        (1e100, "1e+100"),
        (12345.0, "12345"),
        (-0.0000035, "-0.0000035"),
        (1e-5, "0.00001"),
    ];
    for &(x, want) in cases {
        assert_eq!(number_to_string(x), want, "toString({x:e})");
    }
}

/// The mantissa digits and exponent n of std's shortest `{:e}` output.
fn std_shortest(x: f64) -> (String, i64) {
    let s = format!("{x:e}");
    let (mant, exp) = s.split_once('e').unwrap();
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    (digits, exp.parse::<i64>().unwrap() + 1)
}

fn check_to_string(x: f64) {
    let s = number_to_string(x);
    let back: f64 = s.parse().unwrap();
    assert!(
        same(back, x) || (x == 0.0 && back == 0.0),
        "{x:e} → {s} → {back:e}"
    );
    if x > 0.0 {
        let (digits, n) = crate::to_string::shortest_digits(x);
        let mine: String = digits.iter().map(|d| char::from(b'0' + d)).collect();
        let (theirs, tn) = std_shortest(x);
        assert_eq!(mine.len(), theirs.len(), "{x:e}: {mine} vs {theirs}");
        if (&mine, n) != (&theirs, tn) {
            // Only an exact tie may differ: x is exactly half-way between
            // the two candidates and Note 2 takes the even one.
            assert_eq!(n, tn, "{x:e}");
            let bits = x.to_bits();
            let biased = ((bits >> 52) & 0x7ff) as i64;
            let fraction = bits & ((1u64 << 52) - 1);
            let (f, e) = if biased == 0 {
                (fraction, -1074)
            } else {
                (fraction | (1u64 << 52), biased - 1075)
            };
            let exact = exact_decimal(f, e);
            let sig = exact.split('e').next().unwrap().trim_end_matches('0');
            let sig = sig.trim_start_matches('0');
            assert_eq!(sig, format!("{mine}5"), "{x:e}: not a tie");
            assert_eq!(digits.last().unwrap() % 2, 0, "{x:e}: tie not even");
            assert_eq!(mine.len(), theirs.len());
        }
    }
}

#[test]
fn to_string_round_trips_random_bits() {
    let mut rng = XorShift(0x9e37_79b9_7f4a_7c15);
    for _ in 0..30_000 {
        check_to_string(rng.finite().abs());
    }
}

#[test]
fn to_string_round_trips_edge_values() {
    // powers of two (asymmetric intervals), subnormals, neighbours of
    // powers of ten, small integers
    for e in -1074..=1023 {
        let x = if e >= -1022 {
            crate::dd::pow2(e)
        } else {
            f64::from_bits(1u64 << (e + 1074))
        };
        check_to_string(x);
        check_to_string(f64::from_bits(x.to_bits() + 1));
        if x.to_bits() > 1 {
            check_to_string(f64::from_bits(x.to_bits() - 1));
        }
    }
    for bits in 1..2000u64 {
        check_to_string(f64::from_bits(bits));
    }
    let mut p = 1.0f64;
    for _ in 0..308 {
        p *= 10.0;
        check_to_string(p);
        check_to_string(f64::from_bits(p.to_bits() + 1));
        check_to_string(f64::from_bits(p.to_bits() - 1));
    }
    for i in 1..5000u32 {
        check_to_string(f64::from(i));
        check_to_string(f64::from(i) / 1000.0);
    }
    check_to_string(f64::MAX);
    check_to_string(f64::MIN_POSITIVE);
}

#[test]
fn to_string_negative_and_formats() {
    let mut rng = XorShift(12345);
    for _ in 0..2000 {
        let x = rng.finite();
        let s = number_to_string(x);
        if x < 0.0 {
            assert!(s.starts_with('-'));
            assert_eq!(&s[1..], number_to_string(-x));
        }
        // the exponent form appears exactly outside n in [-5, 21]
        if x != 0.0 {
            let (_, n) = crate::to_string::shortest_digits(x.abs());
            assert_eq!(s.contains('e'), !(-5..=21).contains(&n), "{s}");
        }
    }
}

// ------------------------------------------------------- StringToNumber

#[test]
fn string_to_number_grammar() {
    let cases: &[(&str, f64)] = &[
        ("", 0.0),
        (" \n", 0.0),
        (
            "\u{feff}\u{a0}\u{1680}\u{2000}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\t\u{b}\u{c}\r 7 ",
            7.0,
        ),
        ("0", 0.0),
        ("-0", -0.0),
        ("+0", 0.0),
        ("1", 1.0),
        ("+1.5", 1.5),
        ("-1.5", -1.5),
        ("+.5", 0.5),
        (".5", 0.5),
        ("5.", 5.0),
        ("5.e1", 50.0),
        (".5e-1", 0.05),
        ("00012", 12.0),
        ("1e3", 1000.0),
        ("1E+3", 1000.0),
        ("1e-3", 0.001),
        ("Infinity", f64::INFINITY),
        ("+Infinity", f64::INFINITY),
        ("-Infinity", f64::NEG_INFINITY),
        ("  Infinity  ", f64::INFINITY),
        ("0x10", 16.0),
        ("0X1f", 31.0),
        ("0xABCdef", 11_259_375.0),
        ("0o17", 15.0),
        ("0O17", 15.0),
        ("0b101", 5.0),
        ("0B101", 5.0),
        ("  0x10  ", 16.0),
        ("1e400", f64::INFINITY),
        ("-1e400", f64::NEG_INFINITY),
        ("1e-400", 0.0),
        ("-1e-400", -0.0),
        ("0e999999999999999999999", 0.0),
        ("1e999999999999999999999", f64::INFINITY),
        ("1e-999999999999999999999", 0.0),
        ("9007199254740993", 9007199254740992.0),
        ("9007199254740995", 9007199254740996.0),
        (
            "9007199254740993.0000000000000000000000001",
            9007199254740994.0,
        ),
        ("2.4703282292062327e-324", 0.0),
        ("2.4703282292062328e-324", 5e-324),
        ("4.9406564584124654e-324", 5e-324),
        ("1.7976931348623157e308", f64::MAX),
        ("1.7976931348623158e308", f64::MAX),
        ("1.7976931348623159e308", f64::INFINITY),
        ("0x1fffffffffffff", 9007199254740991.0),
        ("0x20000000000001", 9007199254740992.0),
        ("0x20000000000003", 9007199254740996.0),
    ];
    for &(s, want) in cases {
        let got = s2n(s);
        assert!(same(got, want), "{s:?}: got {got:e}, want {want:e}");
    }
    for s in [
        ".",
        "1e",
        "1e+",
        "e5",
        "0x",
        "0o",
        "0b",
        "-0x10",
        "+0x10",
        "0x1g",
        "0o8",
        "0b2",
        "1_000",
        "0x1_0",
        "infinity",
        "INFINITY",
        "-",
        "+",
        "1 2",
        "1..2",
        "1.2.3",
        "0x.1",
        "--1",
        "1e1.5",
        "NaN",
        "\u{85}1",
        "\u{200b}1",
        "1d",
        "Infinityx",
    ] {
        assert!(s2n(s).is_nan(), "{s:?} should be NaN");
    }
    // a lone surrogate or any non-ASCII digit is not part of the grammar
    assert!(string_to_number(&[0xd800]).is_nan());
    assert!(string_to_number(&[0x31, 0xd800]).is_nan());
}

#[test]
fn string_to_number_long_hex() {
    // 2^200 + 1: the low bit is far below the rounding position
    let mut s = String::from("0x1");
    s.push_str(&"0".repeat(49));
    s.push('1');
    assert!(same(s2n(&s), 2.0f64.powi(200)));
    // exactly half-way above 2^53 with a sticky bit far away
    let mut s = String::from("0x20000000000001");
    s.push_str(&"0".repeat(60));
    s.push('1');
    let want: f64 = "9007199254740994".parse().unwrap();
    assert!(same(s2n(&s), want * 2.0f64.powi(244)));
    // 0x + 300 f's overflows
    let s = format!("0x{}", "f".repeat(300));
    assert_eq!(s2n(&s), f64::INFINITY);
}

/// Exact decimal text of f × 2^e (f > 0).
fn exact_decimal(f: u64, e: i64) -> String {
    let mut n = Big::from_u64(f);
    if e >= 0 {
        n.shl(e as u64);
        n.to_decimal()
    } else {
        // f × 2^e = f × 5^-e / 10^-e
        n.mul_pow5(e.unsigned_abs());
        format!("{}e-{}", n.to_decimal(), e.unsigned_abs())
    }
}

#[test]
fn string_to_number_exact_midpoints() {
    let mut rng = XorShift(0xdead_beef);
    let check = |x: f64| {
        if x == 0.0 || x == f64::MAX {
            return;
        }
        let bits = x.to_bits();
        let biased = ((bits >> 52) & 0x7ff) as i64;
        let fraction = bits & ((1u64 << 52) - 1);
        let (f, e) = if biased == 0 {
            (fraction, -1074)
        } else {
            (fraction | (1u64 << 52), biased - 1075)
        };
        // midpoint to the next double up: (2f + 1) × 2^(e - 1)
        let mid = exact_decimal(2 * f + 1, e - 1);
        let next = f64::from_bits(bits + 1);
        let even = if f & 1 == 0 { x } else { next };
        assert!(same(s2n(&mid), even), "{mid}");
        assert!(same(parse_json_number(&mid).unwrap(), even));
        let oracle: f64 = mid.parse().unwrap();
        assert!(same(oracle, even), "oracle disagrees on {mid}");
        // a hair above the midpoint rounds up, a hair below rounds down
        let (mant, exp) = mid.split_once('e').unwrap_or((&mid, "0"));
        let above = format!("{mant}.000000000000000000000000001e{exp}");
        assert!(same(s2n(&above), next), "{above}");
        assert!(same(parse_json_number(&above).unwrap(), next), "{above}");
        if mid.contains('e') {
            // mid's last digit is 5 (odd × 5^k)
            let mut below = mant.as_bytes().to_vec();
            let last = below.len() - 1;
            assert_eq!(below[last], b'5');
            below[last] = b'4';
            let below = format!("{}.999999999999e{exp}", String::from_utf8(below).unwrap());
            assert!(same(s2n(&below), x), "{below}");
        }
    };
    for _ in 0..300 {
        check(rng.finite().abs());
    }
    check(5e-324);
    check(f64::MIN_POSITIVE);
    check(9007199254740992.0);
    check(1.0);
    check(f64::from_bits(f64::MAX.to_bits() - 1));
    // exactly 2^-1075 (half the smallest subnormal) ties to +0
    assert_eq!(s2n(&exact_decimal(1, -1075)).to_bits(), 0);
}

#[test]
fn string_to_number_matches_oracle_on_generated_decimals() {
    let mut rng = XorShift(0x0123_4567_89ab_cdef);
    for i in 0..20_000 {
        let len = match i % 4 {
            0 => 1 + rng.below(5),
            1 => 1 + rng.below(20),
            2 => 15 + rng.below(10),
            _ => 1 + rng.below(900),
        };
        let mut s = String::new();
        if rng.below(2) == 0 {
            s.push('-');
        }
        let point = rng.below(len + 1);
        for j in 0..len {
            if j == point && rng.below(2) == 0 {
                s.push('.');
            }
            s.push(char::from(b'0' + rng.below(10) as u8));
        }
        if rng.below(3) != 0 {
            s.push('e');
            let e = rng.below(700) as i64 - 350;
            s.push_str(&e.to_string());
        }
        let oracle: f64 = s.parse().unwrap();
        let got = s2n(&s);
        assert!(same(got, oracle), "{s}: got {got:e}, oracle {oracle:e}");
    }
    // the output of toString parses back exactly
    for _ in 0..5000 {
        let x = rng.finite();
        assert!(same(s2n(&number_to_string(x)), x));
    }
}

#[test]
fn string_to_number_very_long() {
    // a million digits of 1 then e-999990 ≈ 1.111e9 (std's parser
    // returns Infinity for this one, so the oracle is the short form)
    let s = format!("{}e-999990", "1".repeat(1_000_000));
    assert!(same(s2n(&s), 1.1111111111111112e9));
    let s = format!("0.{}1", "0".repeat(100_000));
    assert_eq!(s2n(&s), 0.0);
}

// --------------------------------------------------------------- JSON

#[test]
fn json_number_grammar() {
    let ok: &[(&str, f64)] = &[
        ("0", 0.0),
        ("-0", -0.0),
        ("1", 1.0),
        ("-12.5e-1", -1.25),
        ("1E2", 100.0),
        ("1e+2", 100.0),
        ("0.5", 0.5),
        ("123456789012345678901234567890", 1.2345678901234568e29),
        ("1e400", f64::INFINITY),
        ("-1e400", f64::NEG_INFINITY),
    ];
    for &(s, want) in ok {
        assert!(same(parse_json_number(s).unwrap(), want), "{s}");
    }
    for s in [
        "", "-", "+1", "01", "00", ".5", "5.", "1e", "1e+", "0x10", " 1", "1 ", "Infinity", "NaN",
        "-.5", "1.e5", "1_0", "--1", "1.5.2", "e5",
    ] {
        assert_eq!(parse_json_number(s), None, "{s:?}");
    }
}

// ----------------------------------------------------------- remainder

#[test]
fn remainder_spec_cases() {
    let nan = f64::NAN;
    let inf = f64::INFINITY;
    let cases: &[(f64, f64, f64)] = &[
        (nan, 1.0, nan),
        (1.0, nan, nan),
        (inf, 1.0, nan),
        (-inf, 1.0, nan),
        (1.0, inf, 1.0),
        (-1.5, -inf, -1.5),
        (1.0, 0.0, nan),
        (1.0, -0.0, nan),
        (0.0, 5.0, 0.0),
        (-0.0, 5.0, -0.0),
        (-0.0, inf, -0.0),
        (5.0, 3.0, 2.0),
        (-5.0, 3.0, -2.0),
        (5.0, -3.0, 2.0),
        (-5.0, -3.0, -2.0),
        (-6.0, 3.0, -0.0),
        (6.0, -3.0, 0.0),
        (5.5, 2.0, 1.5),
        (5e-324, 5e-324, 0.0),
        (1.0, 5e-324, 0.0),
        (0.3, 0.1, 0.09999999999999998),
    ];
    for &(n, d, want) in cases {
        let got = remainder(n, d);
        assert!(same(got, want), "{n:e} % {d:e}: got {got:e}, want {want:e}");
    }
}

#[test]
fn remainder_matches_oracle() {
    let mut rng = XorShift(77);
    for i in 0..50_000 {
        let (n, d) = if i % 2 == 0 {
            (rng.finite(), rng.finite())
        } else {
            // operands of nearby magnitude, where the result is interesting
            let n = rng.finite();
            let d = n * (f64::from(rng.below(1000) as u32) + 1.0) / 997.0;
            (n, if d == 0.0 { 1.0 } else { d })
        };
        let got = remainder(n, d);
        let oracle = n % d;
        assert!(same(got, oracle), "{n:e} % {d:e}: {got:e} vs {oracle:e}");
    }
}

// ------------------------------------------------------ ToInt32/ToUint32

#[test]
fn to_int32_cases() {
    let cases: &[(f64, i32)] = &[
        (f64::NAN, 0),
        (0.0, 0),
        (-0.0, 0),
        (f64::INFINITY, 0),
        (f64::NEG_INFINITY, 0),
        (1.9, 1),
        (-1.9, -1),
        (2147483647.0, 2147483647),
        (2147483648.0, -2147483648),
        (-2147483648.0, -2147483648),
        (-2147483649.0, 2147483647),
        (4294967295.0, -1),
        (4294967296.0, 0),
        (4294967296.5, 0),
        (4294967297.0, 1),
        (1e20, 1661992960),
        (-1e20, -1661992960),
        (5e-324, 0),
        (f64::MAX, 0),
    ];
    for &(x, want) in cases {
        assert_eq!(to_int32(x), want, "ToInt32({x:e})");
        assert_eq!(to_uint32(x), want as u32, "ToUint32({x:e})");
    }
}

#[test]
fn to_uint32_matches_integer_oracle() {
    let mut rng = XorShift(4242);
    for _ in 0..20_000 {
        let x = f64::from_bits(rng.next() & !(0x7ffu64 << 52) | ((rng.below(1023 + 130)) << 52));
        if !x.is_finite() {
            continue;
        }
        // `as u128` truncates toward zero and saturates at 2^128
        let t = x.abs() as u128;
        if x.abs() >= 2f64.powi(127) {
            continue;
        }
        let low = (t % (1u128 << 32)) as u32;
        let want = if x < 0.0 { low.wrapping_neg() } else { low };
        assert_eq!(to_uint32(x), want, "{x:e}");
        assert_eq!(to_int32(x), want as i32);
    }
}

// --------------------------------------------------------- exponentiate

#[test]
fn exponentiate_special_cases() {
    let nan = f64::NAN;
    let inf = f64::INFINITY;
    let cases: &[(f64, f64, f64)] = &[
        (1.0, nan, nan),
        (nan, 0.0, 1.0),
        (nan, -0.0, 1.0),
        (inf, 0.0, 1.0),
        (nan, 1.0, nan),
        (inf, 2.0, inf),
        (inf, -2.0, 0.0),
        (-inf, 3.0, -inf),
        (-inf, 2.0, inf),
        (-inf, 0.5, inf),
        (-inf, -3.0, -0.0),
        (-inf, -2.0, 0.0),
        (-inf, -0.5, 0.0),
        (0.0, 3.0, 0.0),
        (0.0, -3.0, inf),
        (-0.0, 3.0, -0.0),
        (-0.0, 2.0, 0.0),
        (-0.0, 0.5, 0.0),
        (-0.0, -3.0, -inf),
        (-0.0, -2.0, inf),
        (-0.0, -0.5, inf),
        (2.0, inf, inf),
        (0.5, inf, 0.0),
        (1.0, inf, nan),
        (-1.0, inf, nan),
        (1.0, -inf, nan),
        (-1.0, -inf, nan),
        (2.0, -inf, 0.0),
        (-0.5, -inf, inf),
        (-2.0, 0.5, nan),
        (-8.0, 1.0 / 3.0, nan),
        (1.0, 12345.678, 1.0),
        (-1.0, 3.0, -1.0),
        (-1.0, 4.0, 1.0),
        (-1.0, 1e300, 1.0),
        (2.0, 10.0, 1024.0),
        (3.0, 3.0, 27.0),
        (10.0, 22.0, 1e22),
        (2.0, -1074.0, 5e-324),
        (2.0, -1075.0, 0.0),
        (2.0, 1023.0, 8.98846567431158e307),
        (2.0, 1024.0, inf),
        (-2.0, 3.0, -8.0),
        (-2.0, -3.0, -0.125),
        (-2.0, 1025.0, -inf),
        (-2.0, -1075.0, -0.0),
        (0.5, 1074.0, 5e-324),
        (4.0, 0.5, 2.0),
        (4.0, -0.5, 0.5),
        (1e-300, 1e300, 0.0),
        (1e300, 1e300, inf),
        (1e300, -1e300, 0.0),
        (-3.0, 9007199254740992.0 * 4.0, inf),
    ];
    for &(b, e, want) in cases {
        let got = exponentiate(b, e);
        assert!(
            same(got, want),
            "{b:e} ** {e:e}: got {got:e}, want {want:e}"
        );
    }
}

#[test]
fn exponentiate_integral_powers_exact() {
    for base in 2u32..=40 {
        let mut exact: u128 = 1;
        for n in 1..200i32 {
            let Some(next) = exact.checked_mul(u128::from(base)) else {
                break;
            };
            exact = next;
            // `as f64` on an integer rounds correctly
            let want = exact as f64;
            let got = exponentiate(f64::from(base), f64::from(n));
            assert!(same(got, want), "{base} ** {n}: {got:e} vs {want:e}");
            let neg = exponentiate(-f64::from(base), f64::from(n));
            let want_neg = if n % 2 == 1 { -want } else { want };
            assert!(same(neg, want_neg), "(-{base}) ** {n}");
        }
    }
    for k in 1..=1074 {
        let want = if k <= 1022 {
            crate::dd::pow2(-k)
        } else {
            f64::from_bits(1u64 << (1074 - k))
        };
        assert!(same(exponentiate(2.0, -f64::from(k)), want), "2 ** -{k}");
        assert!(same(exponentiate(0.5, f64::from(k)), want), "0.5 ** {k}");
    }
    for k in 1..=323 {
        let want = s2n(&format!("1e-{k}"));
        let got = exponentiate(10.0, -f64::from(k));
        assert!(ulps(got, want) <= 1, "10 ** -{k}: {got:e} vs {want:e}");
        let want = s2n(&format!("1e{k}"));
        if want.is_finite() {
            assert!(same(exponentiate(10.0, f64::from(k)), want), "10 ** {k}");
        }
    }
    // 1.5 ** 3 = 3.375 and 0.1 ** 2 rounds once
    assert_eq!(exponentiate(1.5, 3.0), 3.375);
    assert_eq!(exponentiate(1.25, -2.0), 0.64);
}

#[test]
fn exponentiate_square_root_matches_sqrt() {
    let mut rng = XorShift(2718);
    let mut inexact = 0;
    for _ in 0..5000 {
        let x = rng.finite().abs();
        if x == 0.0 {
            continue;
        }
        let got = exponentiate(x, 0.5);
        let want = x.sqrt(); // IEEE sqrt: correctly rounded
        let d = ulps(got, want);
        assert!(d <= 1, "{x:e} ** 0.5: {got:e} vs {want:e}");
        inexact += d;
    }
    assert!(inexact <= 5, "{inexact} results off by one ulp");
}

#[test]
fn exponentiate_close_to_oracle() {
    // std's powf (platform libm) is only a black-box comparison here: the
    // good ones are within 1 ulp, so ours must be within 2 of them.
    let mut rng = XorShift(31337);
    for _ in 0..20_000 {
        let x = f64::from_bits((rng.next() >> 12) | ((1023 - 40 + rng.below(80)) << 52));
        let y = (rng.next() as i64 as f64) / 9.2e18 * 300.0;
        let got = exponentiate(x, y);
        let oracle = x.powf(y);
        if oracle.is_finite() && oracle > 1e-300 {
            assert!(
                ulps(got, oracle) <= 2,
                "{x:e} ** {y:e}: {got:e} vs {oracle:e}"
            );
        }
    }
    assert!(ulps(exponentiate(2.0, 0.5), std::f64::consts::SQRT_2) == 0);
    assert!(ulps(exponentiate(8.0, 1.0 / 3.0), 2.0) <= 1);
    let (b, y) = (1.0 + 2f64.powi(-52), 2f64.powi(60));
    assert!(ulps(exponentiate(b, y), b.powf(y)) <= 1);
    let (b, y) = (1.0 + 2f64.powi(-52), 2f64.powi(53));
    assert!(ulps(exponentiate(b, y), b.powf(y)) <= 1);
}

#[test]
fn ln2_constant_matches_series() {
    // ln 2 = 2 atanh(1/3) = 2 Σ 1/((2i+1) 3^(2i+1))
    let third = Dd::from(1.0).div(Dd::from(3.0));
    let ninth = third.mul(third);
    let mut power = third;
    let mut sum = third;
    for i in 1..60 {
        power = power.mul(ninth);
        sum = sum.add(power.div(Dd::from(f64::from(2 * i + 1))));
    }
    let ln2 = sum.mul_f64(2.0);
    assert_eq!(ln2.hi.to_bits(), crate::pow::LN2.hi.to_bits());
    assert!(
        ulps(ln2.lo, crate::pow::LN2.lo) <= 4,
        "{:e} vs {:e}",
        ln2.lo,
        crate::pow::LN2.lo
    );
}
