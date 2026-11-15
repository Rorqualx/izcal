//! Reserved `.hgidl` shapes other specs fix ahead of their crates, kept
//! as golden files that must keep passing the generator.

/// Enterprise policy 6.6 and EP-M0: the reserved `policy.hgidl`, copied
/// from the spec's text, passes the checker and generates. It is not in
/// `hg-ipc-gen/idl`: no crate serves it until the policy adapter exists
/// (M5 or later).
#[test]
fn the_reserved_policy_protocol_generates() {
    let src = include_str!("../golden/policy.hgidl");
    let out = hg_ipc_codegen::generate(src, "crates/hg-ipc-codegen/tests/golden/policy.hgidl")
        .unwrap_or_else(|e| panic!("policy.hgidl: {e}"));
    assert!(out.contains("PolicyError"));
}

/// **The four generated crates' `src/sample.rs` are one file, so they
/// are checked to be one file.**
///
/// `hg-ipc-codegen` emits `impl crate::sample::Sample` under
/// `cfg(test)`, so every crate in `GENERATED_CRATES` must supply a
/// `sample` module, and the module is generic -- it names no generated
/// type. The four copies were byte-identical when
/// `hg-compositor-frame` was added (GR-M0, 15 November 2026) and
/// nothing checked that they stayed so: an edit to one would have left
/// the other three behind silently, and the round-trip and mutation
/// tests of each crate would then be sampling from different
/// distributions while reporting the same thing.
///
/// This is the cheap half of a question GR-M0's review puts to the
/// owner (its §9.2): whether duplication guarded by an identity test
/// is good enough, or whether the IDL should gain an `import`. Here
/// the duplication is of a hand-written support module rather than of
/// a wire format, the file is generic, and the test is four lines --
/// so guarding it is clearly right whatever is decided about the IDL.
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the identity check reads the checked-in files"
)]
fn oc20_every_generated_crates_sample_module_is_the_same_file() {
    let crates_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let names = [
        "hg-ipc-gen",
        "hg-os-types",
        "hg-display-list",
        "hg-compositor-frame",
    ];
    let mut first: Option<(String, String)> = None;
    for name in names {
        let p = crates_dir.join(name).join("src/sample.rs");
        let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        match &first {
            None => first = Some((name.to_owned(), text)),
            Some((other, want)) => assert!(
                &text == want,
                "crates/{name}/src/sample.rs differs from crates/{other}/src/sample.rs; \
                 they are one generic module and the generator requires all of them"
            ),
        }
    }
    assert!(first.is_some(), "no crates checked");
}
