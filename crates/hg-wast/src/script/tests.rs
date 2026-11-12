//! Script command tests.
//!
//! As in [`crate::lex`]'s tests, the sources here are written for these
//! tests: the suite is fetched data and is never committed.

use super::*;

fn cmds(src: &str) -> Vec<Cmd> {
    parse(src).unwrap_or_else(|e| panic!("should parse: {}", e.display(src)))
}

fn one(src: &str) -> Cmd {
    let mut c = cmds(src);
    assert_eq!(c.len(), 1, "expected one command from {src:?}");
    c.remove(0)
}

fn error(src: &str) -> String {
    parse(src).expect_err("should not parse").message
}

#[test]
fn a_module_keeps_its_identifier_and_its_whole_span() {
    let src = "(module $m (func))";
    let Cmd::Module(m) = one(src) else {
        panic!("a module");
    };
    assert_eq!(m.id.as_deref(), Some("$m"));
    assert_eq!(m.kind, ModuleKind::Text);
    assert!(!m.definition);
    assert_eq!(m.text(src), src, "the span is the whole command");
}

/// The bytes that went in are the bytes that come out: whatever is
/// inside a module, the runner hands an engine the source unchanged.
#[test]
fn a_module_span_is_verbatim_through_anything_inside_it() {
    let module = r#"(module (func (; ) ;) ) (data ")" "\00") )"#;
    let src = format!("{module} ;; a trailing comment with a )\n(module)");
    let c = cmds(&src);
    assert_eq!(c.len(), 2);
    let Cmd::Module(m) = &c[0] else {
        panic!("a module");
    };
    assert_eq!(m.text(&src), module);
}

#[test]
fn the_three_ways_to_give_a_module() {
    let Cmd::Module(m) = one(r#"(module binary "\00asm" "\01\00\00\00")"#) else {
        panic!("a module");
    };
    assert_eq!(m.kind, ModuleKind::Binary(b"\0asm\x01\0\0\0".to_vec()));

    let Cmd::Module(m) = one(r#"(module quote "(module)")"#) else {
        panic!("a module");
    };
    assert_eq!(m.kind, ModuleKind::Quote(b"(module)".to_vec()));

    let Cmd::Module(m) = one("(module definition $d (func))") else {
        panic!("a module");
    };
    assert!(m.definition);
    assert_eq!(m.id.as_deref(), Some("$d"));
}

#[test]
fn a_module_may_be_instantiated_by_name() {
    let Cmd::Instance {
        instance, module, ..
    } = one("(module instance $i $d)")
    else {
        panic!("an instance");
    };
    assert_eq!(instance.as_deref(), Some("$i"));
    assert_eq!(module.as_deref(), Some("$d"));
}

/// "In a source file, the toplevel `( module … )` surrounding the
/// module body may be omitted" — so a file of module fields is one
/// module and no commands at all.
#[test]
fn a_file_of_module_fields_is_one_inline_module() {
    let src = r#"(func) (memory 0) (func (export "f"))"#;
    let Cmd::Module(m) = one(src) else {
        panic!("a module");
    };
    assert_eq!(m.kind, ModuleKind::Text);
    assert_eq!(m.id, None);
    assert_eq!(m.text(src), src);
}

/// A file that opens with a command keyword is a script, not an inline
/// module, even though `module` could begin either.
#[test]
fn a_file_that_opens_with_a_command_is_a_script() {
    assert_eq!(cmds("(module)\n(module)").len(), 2);
    assert_eq!(cmds("(invoke \"f\")").len(), 1);
}

#[test]
fn register_names_an_instance() {
    let Cmd::Register { name, module, .. } = one(r#"(register "m" $i)"#) else {
        panic!("a register");
    };
    assert_eq!(name, "m");
    assert_eq!(module.as_deref(), Some("$i"));
    let Cmd::Register { module, .. } = one(r#"(register "m")"#) else {
        panic!("a register");
    };
    assert_eq!(module, None);
}

#[test]
fn the_two_actions() {
    let Cmd::Action(Action::Invoke {
        module, name, args, ..
    }) = one(r#"(invoke $m "f" (i32.const 1) (i64.const -2))"#)
    else {
        panic!("an invoke");
    };
    assert_eq!(module.as_deref(), Some("$m"));
    assert_eq!(name, "f");
    assert_eq!(args, [Value::I32(1), Value::I64(u64::MAX - 1)]);

    let Cmd::Action(Action::Get { name, .. }) = one(r#"(get "g")"#) else {
        panic!("a get");
    };
    assert_eq!(name, "g");
}

#[test]
fn assert_return_takes_any_number_of_results() {
    let Cmd::AssertReturn { expected, .. } = one(r#"(assert_return (invoke "f"))"#) else {
        panic!("an assert_return");
    };
    assert!(expected.is_empty());

    let Cmd::AssertReturn { expected, .. } =
        one(r#"(assert_return (invoke "f") (i32.const 1) (f64.const 2.0))"#)
    else {
        panic!("an assert_return");
    };
    assert_eq!(
        expected,
        [
            Pattern::Exact(Value::I32(1)),
            Pattern::Exact(Value::F64(2.0f64.to_bits()))
        ]
    );
}

/// `assert_trap` covers a trap while running and a trap while
/// instantiating, and the two are told apart by what follows it.
#[test]
fn assert_trap_is_about_an_action_or_a_module() {
    let Cmd::AssertTrap { of, message, .. } = one(r#"(assert_trap (invoke "f") "unreachable")"#)
    else {
        panic!("an assert_trap");
    };
    assert!(matches!(of, TrapOf::Action(_)));
    assert_eq!(message, "unreachable");

    let Cmd::AssertTrap { of, .. } = one(r#"(assert_trap (module (func)) "oops")"#) else {
        panic!("an assert_trap");
    };
    assert!(matches!(of, TrapOf::Module(_)));
}

#[test]
fn the_assertions_about_a_module() {
    assert!(matches!(
        one(r#"(assert_malformed (module quote "(") "unexpected")"#),
        Cmd::AssertMalformed { .. }
    ));
    assert!(matches!(
        one(r#"(assert_invalid (module (func (i32.add))) "type mismatch")"#),
        Cmd::AssertInvalid { .. }
    ));
    assert!(matches!(
        one(r#"(assert_unlinkable (module (import "a" "b" (func))) "unknown import")"#),
        Cmd::AssertUnlinkable { .. }
    ));
    assert!(matches!(
        one(r#"(assert_exhaustion (invoke "f") "call stack exhausted")"#),
        Cmd::AssertExhaustion { .. }
    ));
    assert!(matches!(
        one(r#"(assert_exception (invoke "f"))"#),
        Cmd::AssertException { .. }
    ));
}

/// An annotation may stand anywhere a token may, including before the
/// keyword that says what a command is. A reader that looked only after
/// the keyword would not find these.
#[test]
fn annotations_are_skipped_wherever_they_stand() {
    let Cmd::Module(m) = one("((@a) module (@b x y) $m (@c))") else {
        panic!("a module");
    };
    assert_eq!(m.id.as_deref(), Some("$m"));
    assert_eq!(cmds("(@a)\n(module)\n(@b)").len(), 1);
    // The forms the grammar allows for an annotation's own name.
    for a in ["(@a)", "(@)", "(@\"a\")", "(@ x)", "(@a (@b (@c)))"] {
        assert_eq!(cmds(&format!("{a} (module)")).len(), 1, "{a}");
    }
}

/// A space between the parenthesis and the `@` means it is not an
/// annotation and must not be skipped as one. Here that makes the file
/// an inline module rather than a script, which is what a file opening
/// with something that is not a command keyword always is.
#[test]
fn a_detached_at_sign_does_not_open_an_annotation() {
    let src = "( @a) (module)";
    let c = cmds(src);
    assert_eq!(c.len(), 1, "an inline module, not two commands");
    assert_eq!(c[0].span().text(src), src);
}

#[test]
fn the_number_patterns_a_result_may_use() {
    let Cmd::AssertReturn { expected, .. } =
        one(r#"(assert_return (invoke "f") (f32.const nan:canonical) (f64.const nan:arithmetic))"#)
    else {
        panic!("an assert_return");
    };
    assert_eq!(
        expected,
        [
            Pattern::NanCanonical(NumType::F32),
            Pattern::NanArithmetic(NumType::F64)
        ]
    );
}

#[test]
fn either_offers_alternatives() {
    let Cmd::AssertReturn { expected, .. } =
        one(r#"(assert_return (invoke "f") (either (i32.const 1) (i32.const 2)))"#)
    else {
        panic!("an assert_return");
    };
    assert_eq!(
        expected,
        [Pattern::Either(vec![
            Pattern::Exact(Value::I32(1)),
            Pattern::Exact(Value::I32(2)),
        ])]
    );
}

#[test]
fn references_are_values_with_a_payload_and_patterns_without() {
    let Cmd::AssertReturn { expected, .. } = one(
        r#"(assert_return (invoke "f") (ref.null extern) (ref.extern 3) (ref.func) (ref.extern))"#,
    ) else {
        panic!("an assert_return");
    };
    assert_eq!(
        expected,
        [
            Pattern::Exact(Value::RefNull("extern".to_string())),
            Pattern::Exact(Value::RefExtern(3)),
            Pattern::RefOfKind("func".to_string()),
            Pattern::RefOfKind("extern".to_string()),
        ]
    );
}

/// A `v128` of exact lanes is the sixteen bytes themselves, in
/// little-endian lane order; one with a NaN lane has no single byte
/// string and stays lane-wise.
#[test]
fn a_vector_is_bytes_when_it_can_be_and_lanes_when_it_cannot() {
    let Cmd::AssertReturn { expected, .. } =
        one(r#"(assert_return (invoke "f") (v128.const i32x4 1 2 3 4))"#)
    else {
        panic!("an assert_return");
    };
    let mut want = [0u8; 16];
    for (i, n) in [1u32, 2, 3, 4].iter().enumerate() {
        want[i * 4..i * 4 + 4].copy_from_slice(&n.to_le_bytes());
    }
    assert_eq!(expected, [Pattern::Exact(Value::V128(want))]);

    let Cmd::AssertReturn { expected, .. } =
        one(r#"(assert_return (invoke "f") (v128.const f32x4 1 nan:canonical 3 4))"#)
    else {
        panic!("an assert_return");
    };
    let Pattern::V128(shape, lanes) = &expected[0] else {
        panic!("lane-wise, because of the NaN");
    };
    assert_eq!(*shape, Shape::F32x4);
    assert_eq!(lanes.len(), 4);
    assert_eq!(lanes[1], Pattern::NanCanonical(NumType::F32));
}

#[test]
fn every_lane_shape_packs() {
    for (src, lanes) in [
        ("i8x16 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16", 16),
        ("i16x8 1 2 3 4 5 6 7 8", 8),
        ("i32x4 1 2 3 4", 4),
        ("i64x2 1 2", 2),
        ("f32x4 1 2 3 4", 4),
        ("f64x2 1 2", 2),
    ] {
        let cmd = one(&format!(
            r#"(assert_return (invoke "f") (v128.const {src}))"#
        ));
        let Cmd::AssertReturn { expected, .. } = cmd else {
            panic!("an assert_return");
        };
        assert!(
            matches!(expected[0], Pattern::Exact(Value::V128(_))),
            "{src} has {lanes} exact lanes and should pack"
        );
    }
}

/// An argument must be a value. A pattern there would mean an engine
/// was being asked to receive something underdetermined, which is not a
/// thing a call can do.
#[test]
fn a_pattern_is_refused_where_an_argument_belongs() {
    assert!(
        error(r#"(invoke "f" (f32.const nan:canonical))"#).contains("must be a value"),
        "a NaN pattern is not an argument"
    );
    assert!(error(r#"(invoke "f" (ref.func))"#).contains("must be a value"));
}

#[test]
fn what_is_not_a_command_is_named_as_such() {
    // In a script — a file that opened with a command keyword — an
    // unknown command is named. At the head of a file the same text is
    // an inline module instead, which the test above covers.
    assert!(error("(module) (frobnicate)").contains("not a script command"));
    assert!(error("(assert_return (frobnicate))").contains("not an action"));
    assert!(error("(assert_invalid (invoke \"f\") \"m\")").contains("expected a module"));
    assert!(error("(module").contains("never closed"));
}

/// The claim the whole design rests on. A span that is right parses
/// back to the module it came from; one that is wrong does not, and
/// this is what notices.
#[test]
fn a_module_span_parses_back_to_the_same_module() {
    let src = r#"(module $m (func)) (assert_malformed (module quote "(") "x")"#;
    for c in cmds(src) {
        for m in c.modules() {
            m.reparses(src).unwrap_or_else(|e| panic!("{e}"));
        }
    }
}

/// And the check has teeth: a span shortened by one byte is caught.
/// Without this test the re-parse could be vacuous and no one would
/// know.
#[test]
fn a_span_that_is_wrong_fails_the_re_parse() {
    let src = "(module $m (func))";
    let Cmd::Module(m) = one(src) else {
        panic!("a module");
    };
    let short = Module {
        span: Span {
            start: m.span.start,
            end: m.span.end - 1,
        },
        ..m.clone()
    };
    assert!(short.reparses(src).is_err(), "a truncated span must fail");

    let long = Module {
        span: Span {
            start: m.span.start,
            end: m.span.end,
        },
        id: Some("$other".to_string()),
        ..m.clone()
    };
    assert!(
        long.reparses(src).is_err(),
        "a span that parses as a different module must fail"
    );
}

#[test]
fn a_command_knows_its_own_keyword_and_span() {
    let src = r#"(module) (register "m") (invoke "f") (get "g")"#;
    let c = cmds(src);
    assert_eq!(
        c.iter().map(Cmd::keyword).collect::<Vec<_>>(),
        ["module", "register", "invoke", "get"]
    );
    assert_eq!(c[0].span().text(src), "(module)");
    assert_eq!(c[3].span().text(src), r#"(get "g")"#);
}

/// A meta command rewinds to its own `(` before taking the whole group.
/// Finding that parenthesis by subtracting from the current position is
/// wrong, because reading the keyword may have skipped an annotation on
/// the way: the subtraction then lands inside it. The suite has no meta
/// command at all, so nothing there would have shown this.
#[test]
fn a_meta_command_keeps_its_span_across_an_annotation() {
    let src = r#"(script "x")"#;
    let Cmd::Meta { name, span } = one(src) else {
        panic!("a meta command");
    };
    assert_eq!(name, "script");
    assert_eq!(span.text(src), src);

    let src = r#"((@a) script "x")"#;
    let Cmd::Meta { name, span } = one(src) else {
        panic!("a meta command");
    };
    assert_eq!(name, "script");
    assert_eq!(span.text(src), src);
}
