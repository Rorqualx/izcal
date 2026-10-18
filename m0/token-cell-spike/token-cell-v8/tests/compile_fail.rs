//! Criteria (1) and (4) through the V8 wrappers: misuse must not compile.
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
