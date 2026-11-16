//! `import`: a protocol takes types from a sibling in the same `idl/`
//! directory, and the generated module re-exports them rather than
//! declaring a second copy.
//!
//! The owner's decision of 16 November 2026 (GR-M0's review §9.2,
//! option (a)). Every rule `check::resolve_imports` states is refused
//! here with its reason, and the accepting twin is checked beside it:
//! a refusal test whose twin is not shown cannot distinguish a rule
//! from a generator that refuses everything.

use std::collections::BTreeMap;

/// A resolver over a literal map, standing in for an `idl/` directory.
struct Map(BTreeMap<String, String>);

impl Map {
    fn new(pairs: &[(&str, &str)]) -> Self {
        Self(
            pairs
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        )
    }
}

impl hg_ipc_codegen::Imports for Map {
    fn source(&self, protocol: &str) -> Option<String> {
        self.0.get(protocol).cloned()
    }
}

/// A vocabulary protocol: one error enum and three types.
const VOCAB: &str = "\
/// Vocabulary.
protocol vocab;
/// Why.
error VocabError {
    /// A.
    A,
}
/// A point.
struct Point {
    /// X.
    x: i32,
    /// Y.
    y: i32,
}
/// A rule.
enum FillRule {
    /// NZ.
    NonZero,
}
/// A thing.
union Shape {
    /// A point.
    P { /// At.
        at: Point, },
}
/// An interface.
interface Sink bound capacity 8 overflow drop {
    /// Takes one.
    take(/// The point.
        p: Point);
}
";

fn user(imports: &str, body: &str) -> String {
    format!(
        "/// User.\nprotocol user;\n{imports}/// Why.\nerror UserError {{\n/// A.\nA,\n}}\n{body}"
    )
}

fn emitted(src: &str) -> Result<String, String> {
    let map = Map::new(&[("vocab", VOCAB)]);
    hg_ipc_codegen::generate_unformatted_with(src, "user.hgidl", &map).map_err(|e| e.to_string())
}

fn refused(src: &str) -> String {
    match emitted(src) {
        Ok(_) => panic!("accepted:\n{src}"),
        Err(e) => e,
    }
}

/// The accepting case, and what it emits.
#[test]
fn oc21_an_import_re_exports_and_does_not_redeclare() {
    let src = user(
        "import vocab { Point, FillRule };\n",
        "/// S.\nstruct S {\n/// F.\nf: Point,\n/// R.\nrule: FillRule,\n}\n",
    );
    let out = emitted(&src).unwrap_or_else(|e| panic!("{e}"));

    // Re-exported, in sorted order, from the sibling module.
    assert!(
        out.contains("pub use super::vocab::{FillRule, Point};"),
        "no re-export in:\n{out}"
    );
    // Not re-declared: one declaration, one set of harnesses, and the
    // type a consumer sees is the one the sibling encodes.
    assert!(
        !out.contains("pub struct Point"),
        "Point was declared a second time:\n{out}"
    );
    assert!(
        !out.contains("pub enum FillRule"),
        "FillRule was declared a second time:\n{out}"
    );
    // The importing protocol's own types are still emitted.
    assert!(out.contains("pub struct S"), "no local type:\n{out}");

    // The twin that makes the test worth having: the same file without
    // the import must be refused, so the acceptance above is the
    // import's doing and not the checker ignoring unknown names.
    let without = user(
        "",
        "/// S.\nstruct S {\n/// F.\nf: Point,\n/// R.\nrule: FillRule,\n}\n",
    );
    assert!(
        refused(&without).contains("unknown type `Point`"),
        "the same file without the import was not refused"
    );
}

/// `generate` resolves nothing, so an `import` is refused rather than
/// silently ignored.
#[test]
fn oc22_the_import_free_entry_point_refuses_an_import() {
    let src = user(
        "import vocab { Point };\n",
        "/// S.\nstruct S {\n/// F.\nf: Point,\n}\n",
    );
    let e = hg_ipc_codegen::generate_unformatted(&src, "user.hgidl")
        .expect_err("NoImports accepted an import");
    assert!(e.to_string().contains("no sibling protocol `vocab`"), "{e}");
}

/// Every rule in `resolve_imports`, each with its reason.
#[test]
fn oc23_every_import_rule_has_a_refusal() {
    let cases: &[(&str, String)] = &[
        (
            "no sibling protocol `absent`",
            user(
                "import absent { Point };\n",
                "/// S.\nstruct S {\n/// F.\nf: Point,\n}\n",
            ),
        ),
        (
            "cannot import itself",
            user(
                "import user { Point };\n",
                "/// S.\nstruct S {\n/// F.\nf: u8,\n}\n",
            ),
        ),
        (
            "declares no type `Missing`",
            user(
                "import vocab { Missing };\n",
                "/// S.\nstruct S {\n/// F.\nf: u8,\n}\n",
            ),
        ),
        (
            "is `vocab`'s error enum",
            user(
                "import vocab { VocabError };\n",
                "/// S.\nstruct S {\n/// F.\nf: u8,\n}\n",
            ),
        ),
        (
            "is an interface, not a type",
            user(
                "import vocab { Sink };\n",
                "/// S.\nstruct S {\n/// F.\nf: u8,\n}\n",
            ),
        ),
        (
            "imported and also declared here",
            user(
                "import vocab { Point };\n",
                "/// Point.\nstruct Point {\n/// F.\nf: u8,\n}\n",
            ),
        ),
        (
            "`Point` is imported twice",
            user(
                "import vocab { Point, Point };\n",
                "/// S.\nstruct S {\n/// F.\nf: Point,\n}\n",
            ),
        ),
        (
            "`vocab` is imported twice",
            user(
                "import vocab { Point };\nimport vocab { FillRule };\n",
                "/// S.\nstruct S {\n/// F.\nf: Point,\n}\n",
            ),
        ),
        (
            "takes at least one type",
            user(
                "import vocab { };\n",
                "/// S.\nstruct S {\n/// F.\nf: u8,\n}\n",
            ),
        ),
        (
            "comes before the declarations",
            user(
                "",
                "/// S.\nstruct S {\n/// F.\nf: u8,\n}\nimport vocab { Point };\n",
            ),
        ),
    ];
    for (reason, src) in cases {
        let got = refused(src);
        assert!(
            got.contains(reason),
            "wanted {reason:?}, got {got:?} for:\n{src}"
        );
    }
}

/// An imported protocol may not itself import, so the graph is one
/// level deep and a cycle cannot be written.
#[test]
fn oc24_an_imported_protocol_may_not_itself_import() {
    let middle = "\
/// Middle.
protocol middle;
import vocab { Point };
/// Why.
error MiddleError {
    /// A.
    A,
}
/// M.
struct M {
    /// F.
    f: Point,
}
";
    let map = Map::new(&[("vocab", VOCAB), ("middle", middle)]);
    let src = user(
        "import middle { M };\n",
        "/// S.\nstruct S {\n/// F.\nf: M,\n}\n",
    );
    let e = hg_ipc_codegen::generate_unformatted_with(&src, "user.hgidl", &map)
        .expect_err("a two-level import graph was accepted");
    assert!(e.to_string().contains("may not import"), "{e}");

    // The twin: `middle` without its own import is fine, so the
    // refusal is about the depth and not about `middle`.
    let flat = middle
        .replace("import vocab { Point };\n", "")
        .replace("    f: Point,", "    f: u8,");
    let map = Map::new(&[("vocab", VOCAB), ("middle", &flat)]);
    hg_ipc_codegen::generate_unformatted_with(&src, "user.hgidl", &map)
        .unwrap_or_else(|e| panic!("the one-level twin was refused: {e}"));
}

/// `pub use super::<protocol>` is only correct because both generated
/// modules are written side by side. This is that assumption, checked
/// against `pairs()` rather than assumed.
#[test]
fn oc25_a_crates_generated_modules_are_siblings() {
    let crates_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in [
        "hg-ipc-gen",
        "hg-os-types",
        "hg-display-list",
        "hg-compositor-frame",
    ] {
        let pairs = hg_ipc_codegen::pairs(&crates_dir.join(name), name).unwrap();
        let mut parents: Vec<_> = pairs.iter().filter_map(|p| p.rust.parent()).collect();
        parents.dedup();
        assert_eq!(
            parents.len(),
            1,
            "{name}: generated modules are not siblings, so `super::` would not reach them"
        );
        // And every import resolves in the directory the IDL sits in.
        for p in &pairs {
            assert_eq!(
                p.idl.parent().unwrap().file_name().unwrap(),
                "idl",
                "{name}: an .hgidl outside idl/"
            );
        }
    }
}

/// An error inside an imported file says which file it is in. Without
/// this the position is the sibling's and the label is the importer's,
/// so the message points at a line in the wrong file.
#[test]
fn oc26_an_error_in_an_imported_file_names_that_file() {
    // `vocab` with its error enum removed: valid syntax, refused by the
    // checker (CS-ER-6), and refused while being read as an import.
    let broken = VOCAB.replace("error VocabError {\n    /// A.\n    A,\n}\n", "");
    let map = Map::new(&[("vocab", &broken)]);
    let src = user(
        "import vocab { Point };\n",
        "/// S.\nstruct S {\n/// F.\nf: Point,\n}\n",
    );
    let e = hg_ipc_codegen::generate_unformatted_with(&src, "user.hgidl", &map)
        .expect_err("a broken import was accepted");
    let text = e.to_string();
    assert!(
        text.contains("in imported `vocab.hgidl`"),
        "the error does not say which file it is in: {text}"
    );
    assert!(
        text.contains("one closed `error` enum"),
        "the sibling's own reason was lost: {text}"
    );
}
