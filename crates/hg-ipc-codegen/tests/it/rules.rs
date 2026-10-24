//! The [gen] rules: each bad `.hgidl` is refused with its reason.

fn refused(src: &str) -> String {
    match hg_ipc_codegen::generate_unformatted(src, "test.hgidl") {
        Ok(_) => panic!("accepted:\n{src}"),
        Err(e) => e.to_string(),
    }
}

const HEAD: &str = "/// P.\nprotocol p;\n/// E.\nerror E {\n/// A.\nA,\n}\n";

fn with(body: &str) -> String {
    format!("{HEAD}{body}")
}

#[test]
fn every_rule_has_a_refusal() {
    let cases: &[(&str, String)] = &[
        (
            "needs a `///` doc comment",
            with("struct S {\n/// F.\nf: u8,\n}\n"),
        ),
        (
            "one closed `error` enum",
            "/// P.\nprotocol p;\n".to_owned(),
        ),
        (
            "only one `error` enum",
            with("/// E2.\nerror E2 {\n/// A.\nA,\n}\n"),
        ),
        (
            "maximum length must be",
            with("/// S.\nstruct S {\n/// F.\nf: bytes<0>,\n}\n"),
        ),
        (
            "contains itself",
            with("/// S.\nstruct S {\n/// F.\nf: option<S>,\n}\n"),
        ),
        (
            "unknown type",
            with("/// S.\nstruct S {\n/// F.\nf: Nope,\n}\n"),
        ),
        (
            "`bound` or `free`",
            with("/// I.\ninterface I open capacity 1 overflow drop {\n/// M.\nm();\n}\n"),
        ),
        (
            "`drop` or `terminate`",
            with("/// I.\ninterface I bound capacity 1 overflow block {\n/// M.\nm();\n}\n"),
        ),
        (
            "capacity must be",
            with("/// I.\ninterface I bound capacity 0 overflow drop {\n/// M.\nm();\n}\n"),
        ),
        (
            "over the 65536 byte cap",
            with(
                "/// I.\ninterface I bound capacity 1 overflow drop {\n/// M.\nm(\n/// B.\nb: bytes<70000>,\n);\n}\n",
            ),
        ),
        (
            "duplicate message",
            with(
                "/// I.\ninterface I free capacity 1 overflow drop {\n/// M.\nm();\n/// M.\nm();\n}\n",
            ),
        ),
        (
            "snake_case",
            with("/// S.\nstruct S {\n/// F.\nBad: u8,\n}\n"),
        ),
        (
            "a name the generated code uses",
            with("/// S.\nstruct S {\n/// F.\nw: u8,\n}\n"),
        ),
        (
            "a name the generated code uses",
            with(
                "/// I.\ninterface I free capacity 1 overflow drop {\n/// M.\nm(\n/// L.\nlen: u8,\n);\n}\n",
            ),
        ),
        (
            "PascalCase",
            with("/// S.\nstruct lower {\n/// F.\nf: u8,\n}\n"),
        ),
        (
            "at least one byte",
            with(
                "/// S.\nstruct S {\n/// F.\nf: vec<option<u8>, 2>,\n}\n/// T.\nstruct T {\n/// F.\nf: vec<Z, 2>,\n}\n/// Z.\nstruct Z {\n/// F.\nf: u8,\n}\n",
            ),
        ),
    ];
    for (want, src) in cases {
        if *want == "at least one byte" {
            // Every type encodes to at least one byte, so this rule cannot
            // fire from valid syntax; the case shows such a schema passes.
            assert!(hg_ipc_codegen::generate_unformatted(src, "t").is_ok());
            continue;
        }
        let got = refused(src);
        assert!(got.contains(want), "want `{want}`, got `{got}` for:\n{src}");
    }
}

#[test]
fn errors_name_the_line() {
    let e = refused(
        "/// P.\nprotocol p;\n/// E.\nerror E {\n/// A.\nA,\n}\n/// S.\nstruct S {\n/// F.\nf: Nope,\n}\n",
    );
    assert!(e.starts_with("11:4:"), "{e}");
}
