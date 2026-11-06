//! The generator's checks, each shown rejecting a description (language
//! note Section 6, OC-1 to OC-17). Each test starts from the checked-in
//! JS-M0 descriptions, which pass, and changes one thing.

use std::path::{Path, PathBuf};

use hg_opgen::{anchors, cli, diag::Diags};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sources() -> Vec<(String, String)> {
    cli::read_sources(&root()).unwrap()
}

/// Loads `src` and checks its anchors against the pins.
fn check(src: &[(String, String)]) -> Result<(), Diags> {
    let (m, _) = hg_opgen::load(src)?;
    let mut d = Diags::default();
    anchors::check(&m, &cli::pins(&root()).unwrap(), &mut d);
    if d.is_empty() { Ok(()) } else { Err(d) }
}

/// The sources with `from` replaced by `to` in the file ending `file`.
fn mutate(file: &str, from: &str, to: &str) -> Vec<(String, String)> {
    let mut s = sources();
    let f = s.iter_mut().find(|(n, _)| n.ends_with(file)).unwrap();
    assert!(f.1.contains(from), "{file} has no `{from}`");
    f.1 = f.1.replacen(from, to, 1);
    s
}

/// The sources with one more file.
fn with(extra: &str) -> Vec<(String, String)> {
    let mut s = sources();
    s.push(("crates/hg-js-ops/ops/99-test.hgop".into(), extra.into()));
    s
}

#[track_caller]
fn rejects(src: &[(String, String)], check_name: &str) -> String {
    let names: Vec<String> = src.iter().map(|s| s.0.clone()).collect();
    match check(src) {
        Ok(()) => panic!("accepted; expected an {check_name} rejection"),
        Err(d) => {
            let text: Vec<String> = d.0.iter().map(|x| x.render(&names)).collect();
            assert!(
                d.has(check_name),
                "expected {check_name}, got:\n{}",
                text.join("\n")
            );
            text.join("\n")
        }
    }
}

#[test]
fn the_checked_in_descriptions_pass() {
    check(&sources()).unwrap();
}

#[test]
fn oc1_rejects_unknown_names() {
    rejects(
        &mutate(
            "60-property.hgop",
            "rt_ordinary_get(d",
            "rt_ordinary_fetch(d",
        ),
        "OC-1",
    );
    rejects(
        &mutate(
            "01-prelude.hgop",
            "layout HostFn     { id: i32",
            "layout HostFn     { id: Frob",
        ),
        "OC-1",
    );
}

#[test]
fn oc2_rejects_a_match_that_misses_a_kind() {
    // ToBoolean without its Symbol arm: a path that does not handle
    // `Ref<Symbol>`.
    let t = rejects(
        &mutate("03-helpers.hgop", "Ref<Symbol>(_) => true,", ""),
        "OC-2",
    );
    assert!(t.contains("Symbol"), "{t}");
    // An ill-typed comparison.
    rejects(
        &mutate(
            "03-helpers.hgop",
            "Smi(i) => i != 0,",
            "Smi(i) => i != 0.5,",
        ),
        "OC-2",
    );
}

#[test]
fn oc3_rejects_fall_through_recursion_and_unchecked_overflow() {
    rejects(
        &mutate(
            "03-helpers.hgop",
            "path rope { return alloc Rope { left: a, right: b, length: u32(n) }; }",
            "path rope { }",
        ),
        "OC-3",
    );
    rejects(
        &mutate(
            "03-helpers.hgop",
            "Ref<Symbol>(_) => true,",
            "Ref<Symbol>(_) => ToBooleanValue(Undefined),",
        ),
        "OC-3",
    );
    // A product of two small integers can overflow i32: the description
    // must widen; and `smi(e)` needs a range proof.
    // (An i32 sum of two small integers is proved in range, so it passes.)
    check(&mutate(
        "30-arith.hgop",
        "acc = number(f64(i64(a) + i64(b)));",
        "acc = number(f64(a + b));",
    ))
    .unwrap();
    rejects(
        &mutate(
            "30-arith.hgop",
            "acc = number(f64(i64(a) + i64(b)));",
            "acc = number(f64(a * b));",
        ),
        "OC-3",
    );
    rejects(
        &mutate(
            "30-arith.hgop",
            "acc = number(f64(i64(a) + i64(b)));",
            "acc = smi(i64(a) + i64(b));",
        ),
        "OC-3",
    );
}

#[test]
fn oc4_rejects_frame_effects_that_disagree_with_the_declaration() {
    rejects(
        &mutate(
            "50-control.hgop",
            "op BranchTrue(target: fwd) acc in",
            "op BranchTrue(target: fwd) acc none",
        ),
        "OC-4",
    );
    rejects(
        &mutate(
            "50-control.hgop",
            "op Jump(target: fwd) acc none",
            "op Jump(target: fwd) acc out",
        ),
        "OC-4",
    );
    // A backward jump that skips the interrupt check.
    rejects(
        &mutate(
            "50-control.hgop",
            "op Jump(target: fwd)",
            "op Jump(target: back)",
        ),
        "OC-4",
    );
    rejects(
        &mutate("50-control.hgop", "  interrupt.poll();", ""),
        "OC-4",
    );
}

#[test]
fn oc5_rejects_an_undeclared_user_call_and_notes_a_wide_declaration() {
    let t = rejects(
        &mutate(
            "60-property.hgop",
            "effects { reads(Field(any)), alloc_internal, throw(any), call_user }",
            "effects { reads(Field(any)), alloc_internal, throw(TypeError) }",
        ),
        "OC-5",
    );
    assert!(t.contains("call_user"), "{t}");
    rejects(
        &mutate(
            "70-intrinsic.hgop",
            "effects { alloc(dict_units), alloc_internal, throw(TypeError) }\n  anchor ecma262 \"sec-object.create\"",
            "effects { alloc_internal, throw(TypeError) }\n  anchor ecma262 \"sec-object.create\"",
        ),
        "OC-5",
    );
    // Wider than inferred is allowed, and reported.
    let s = mutate(
        "50-control.hgop",
        "op Jump(target: fwd) acc none\n  effects { }",
        "op Jump(target: fwd) acc none\n  effects { writes(realm) }",
    );
    let (m, f) = hg_opgen::load(&s).unwrap();
    assert!(
        hg_opgen::notes(&m, &f)
            .iter()
            .any(|n| n.contains("`Jump` declares writes(realm)"))
    );
}

#[test]
fn oc6_rejects_an_unrooted_temporary_across_a_gc_point() {
    rejects(
        &with(
            "intrinsic %Unrooted(proto: Value) -> Value\n  effects { alloc(dict_units), alloc_internal }\n  anchor izcal \"test\"\n{\n  let d = rt_dict_new(Null);\n  charge(dict_units());\n  return d;\n}\n",
        ),
        "OC-6",
    );
    // A `frame` parameter given a value that is not frame-backed.
    rejects(
        &mutate(
            "30-arith.hgop",
            "acc = StringConcat(a, b);",
            "acc = StringConcat(rt_number_to_string(1.0), b);",
        ),
        "OC-6",
    );
}

#[test]
fn oc7_rejects_a_guard_after_an_effect() {
    let t = rejects(
        &mutate(
            "30-arith.hgop",
            "let y = acc;                           // frame-backed until acc is written\n  match",
            "let y = acc;\n  charge(dict_units());\n  match",
        ),
        "OC-7",
    );
    assert!(t.contains("speculable"), "{t}");
}

#[test]
fn oc8_rejects_representation_dependent_charges() {
    let s = mutate(
        "01-prelude.hgop",
        "layout String     { length: u32 immutable; }",
        "layout String     { length: u32 immutable; depth: u32 immutable repr; }",
    );
    let s = s
        .into_iter()
        .map(|(n, t)| {
            if n.ends_with("03-helpers.hgop") {
                let t = t.replacen(
                    "charge(string_units(n));",
                    "charge(string_units(n + i64(String.depth(a))));",
                    1,
                );
                (n, t)
            } else {
                (n, t)
            }
        })
        .collect::<Vec<_>>();
    rejects(&s, "OC-8");
    rejects(
        &mutate(
            "01-prelude.hgop",
            "units dict_units() = 4;",
            "units dict_units() = 4 + rope_depth;",
        ),
        "OC-8",
    );
}

#[test]
fn oc9_rejects_a_weak_read_kept_across_a_gc_point_and_a_hook() {
    // Language note 9.7's shape: a value read from mutable state, an
    // epoch-ending step, then a use of the old value.
    let gc = "intrinsic %Stale(this: Value) -> Value\n  effects { reads(weak), alloc(dict_units), alloc_internal, throw(TypeError) }\n  anchor izcal \"test\"\n{\n  match this {\n    Ref<WeakRef>(w) => path p anchor izcal \"test\" {\n      let t = WeakRef.target(w);\n      charge(dict_units());\n      match t { Empty => return Undefined; Value(v) => return v; }\n    }\n    _ => path q anchor izcal \"test\" { throw TypeError; }\n  }\n}\n";
    let t = rejects(&with(gc), "OC-9");
    assert!(t.contains("GC point"), "{t}");
    let hook = gc
        .replace(
            "alloc(dict_units), alloc_internal,",
            "host(HostSystemUTCEpochNanoseconds), input(clock),",
        )
        .replace("charge(dict_units());", "let now = rt_date_now();");
    let t = rejects(&with(&hook), "OC-9");
    assert!(t.contains("host hook"), "{t}");
    // Reading it again after the step is accepted.
    let fixed = gc.replace(
        "charge(dict_units());\n      match t {",
        "charge(dict_units());\n      match WeakRef.target(w) {",
    );
    check(&with(&fixed)).unwrap();
}

#[test]
fn oc11_rejects_a_clock_read_outside_its_hook() {
    rejects(
        &mutate(
            "02-runtime.hgop",
            "effects { host(HostSystemUTCEpochNanoseconds), input(clock) }",
            "effects { input(clock) }",
        ),
        "OC-11",
    );
    rejects(
        &mutate(
            "02-runtime.hgop",
            "effects { host(HostSystemUTCEpochNanoseconds), input(clock) }",
            "effects { host(HostSystemUTCEpochNanoseconds) }",
        ),
        "OC-11",
    );
}

#[test]
fn oc12_rejects_stale_and_empty_anchors() {
    let t = rejects(
        &mutate(
            "40-compare.hgop",
            "\"sec-islooselyequal\"",
            "\"sec-is-loosely-equal-old\"",
        ),
        "OC-12",
    );
    assert!(t.contains("sec-is-loosely-equal-old"), "{t}");
    rejects(
        &mutate(
            "50-control.hgop",
            "anchor ecma262 \"sec-loopcontinues\"",
            "anchor izcal \"\"",
        ),
        "OC-12",
    );
    rejects(
        &mutate(
            "60-property.hgop",
            "path nullish anchor ecma262 \"sec-getvalue\" step \"3.a\"",
            "path dictionary anchor ecma262 \"sec-getvalue\" step \"3.a\"",
        ),
        "OC-12",
    );
}

#[test]
fn oc13_rejects_encodings_the_verifier_could_not_check() {
    rejects(&mutate("50-control.hgop", "op Jump(", "op Wide2("), "OC-13");
    rejects(
        &mutate("60-property.hgop", "fb: slot<load>", "fb: slot<lode>"),
        "OC-13",
    );
    rejects(
        &mutate(
            "70-intrinsic.hgop",
            "op CallIntrinsic(id: intr, args: reg, count: cnt)",
            "op CallIntrinsic(id: intr, count: cnt, args: reg)",
        ),
        "OC-13",
    );
    rejects(
        &mutate(
            "10-load.hgop",
            "const<number|string>",
            "const<number|regex>",
        ),
        "OC-13",
    );
}

#[test]
fn oc14_rejects_representation_and_isa_dependent_results() {
    // A heap number made directly would skip the canonical form (JS-D12).
    rejects(
        &mutate(
            "70-intrinsic.hgop",
            "return number(rt_math_random());",
            "return alloc HeapNumber { value: rt_math_random() };",
        ),
        "OC-14",
    );
    rejects(
        &mutate(
            "30-arith.hgop",
            "acc = number(f64.add(a, b));",
            "acc = number(f64.fma(a, b, 1.0));",
        ),
        "OC-14",
    );
}

#[test]
fn oc15_generation_is_byte_identical_and_portable() {
    let (a, _) = cli::build(&root()).unwrap();
    let (b, _) = cli::build(&root()).unwrap();
    assert_eq!(a, b);
    for (path, text) in &a {
        assert!(!text.contains('\r'), "{path}");
        assert!(
            !text.contains(env!("CARGO_MANIFEST_DIR")),
            "{path} names a local path"
        );
    }
    // CRLF sources generate the same bytes.
    let crlf: Vec<(String, String)> = sources()
        .into_iter()
        .map(|(n, t)| (n, t.replace('\n', "\r\n").replace("\r\n", "\n")))
        .collect();
    assert_eq!(crlf, sources());
}

#[test]
fn oc16_the_checked_in_output_is_fresh_and_a_hand_edit_is_caught() {
    cli::run(&root(), &["check".into()]).unwrap();
    // A copy of the inputs and outputs with one generated file edited.
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("oc16");
    let _ = std::fs::remove_dir_all(&tmp);
    let (outs, _) = cli::build(&root()).unwrap();
    let mut files: Vec<String> = outs.iter().map(|o| o.0.clone()).collect();
    files.push(anchors::PIN_FILE.into());
    for (n, _) in sources() {
        files.push(n);
    }
    for f in &files {
        let to = tmp.join(f);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(root().join(f), &to).unwrap();
    }
    cli::run(&tmp, &["check".into()]).unwrap();
    let edited = tmp.join("crates/hg-js-ops/src/generated/facts.tsv");
    let mut t = std::fs::read_to_string(&edited).unwrap();
    t.push_str("# a hand edit\n");
    std::fs::write(&edited, t).unwrap();
    let e = cli::run(&tmp, &["check".into()]).unwrap_err();
    assert!(e.contains("OC-16") && e.contains("facts.tsv"), "{e}");
    cli::run(&tmp, &["generate".into()]).unwrap();
    cli::run(&tmp, &["check".into()]).unwrap();
}

#[test]
fn oc17_rejects_post_call_work() {
    // Section 6's example: `let r = rt_f(x); acc = bool(r == True);`.
    let t = rejects(
        &mutate(
            "40-compare.hgop",
            "acc = rt_loose_equal(x, y);",
            "let r = rt_loose_equal(x, y);\n      acc = bool(r == True);",
        ),
        "OC-17",
    );
    assert!(t.contains("user code"), "{t}");
    // A user call followed by more effects.
    rejects(
        &mutate(
            "60-property.hgop",
            "rt_set(o, k, acc, unit.strict());\n      next;",
            "rt_set(o, k, acc, unit.strict());\n      acc = Undefined;\n      next;",
        ),
        "OC-17",
    );
    // The after_call mode must match the form.
    rejects(
        &mutate(
            "60-property.hgop",
            "acc in after_call preserve\n  effects { reads(Field(any)), writes(Field(any)), alloc(property_units), alloc_internal",
            "acc in after_call result\n  effects { reads(Field(any)), writes(Field(any)), alloc(property_units), alloc_internal",
        ),
        "OC-17",
    );
}
