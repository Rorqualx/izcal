//! Mutation tests.

use super::*;
use crate::render;
use crate::script::parse;

#[test]
fn one_expected_value_changes_and_the_rest_do_not() {
    let src = r#"(assert_return (invoke "f") (i32.const 42) (i32.const 7))
(assert_return (invoke "g") (i32.const 9))"#;
    let mut cmds = parse(src).expect("should parse");
    let what = flip_one_expected(src, &mut cmds).expect("something to change");
    assert!(what.contains("i32 0x2a became 0x2b"), "{what}");
    let text = render::script(src, &cmds);
    assert!(text.contains("0x2b"), "{text}");
    // The others are untouched.
    assert!(text.contains("0x7"), "{text}");
    assert!(text.contains("0x9"), "{text}");
}

#[test]
fn a_script_with_nothing_to_change_says_so() {
    let src = r#"(module)
(assert_return (invoke "f"))
(assert_return (invoke "g") (f32.const nan:canonical))
(assert_return (invoke "h") (ref.null extern))
(assert_trap (invoke "i") "unreachable")"#;
    let mut cmds = parse(src).expect("should parse");
    assert_eq!(flip_one_expected(src, &mut cmds), None);
}

/// Every numeric kind can be nudged, so a script made only of one of
/// them is still mutable.
#[test]
fn each_kind_of_number_can_be_changed() {
    for (src, want) in [
        (r#"(assert_return (invoke "f") (i64.const 1))"#, "i64"),
        (r#"(assert_return (invoke "f") (f32.const 1))"#, "f32"),
        (r#"(assert_return (invoke "f") (f64.const 1))"#, "f64"),
        (
            r#"(assert_return (invoke "f") (v128.const i32x4 0 0 0 0))"#,
            "v128",
        ),
    ] {
        let mut cmds = parse(src).expect("should parse");
        let what = flip_one_expected(src, &mut cmds).expect("something to change");
        assert!(what.contains(want), "{src}: {what}");
        assert_ne!(
            render::script(src, &cmds),
            render::script(src, &parse(src).expect("again"))
        );
    }
}
