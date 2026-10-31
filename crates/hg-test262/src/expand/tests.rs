//! Expansion tests: modes, the strict prefix and harness order
//! (INTERPRETING.md, "Strict Mode", `flags`, `includes`).

use super::*;
use crate::frontmatter::parse_test_file;

fn fm(flags: &str, includes: &str) -> FrontMatter {
    parse_test_file(&format!(
        "/*---\ndescription: d\nflags: [{flags}]\nincludes: [{includes}]\n---*/\n"
    ))
    .unwrap()
}

#[test]
fn modes_follow_the_flags() {
    assert_eq!(modes(&fm("", "")), [Mode::Sloppy, Mode::Strict]);
    assert_eq!(
        modes(&fm("generated, async", "")),
        [Mode::Sloppy, Mode::Strict]
    );
    assert_eq!(modes(&fm("onlyStrict", "")), [Mode::Strict]);
    assert_eq!(modes(&fm("noStrict", "")), [Mode::Sloppy]);
    assert_eq!(modes(&fm("raw", "")), [Mode::Sloppy]);
    assert_eq!(modes(&fm("module", "")), [Mode::Module]);
    assert_eq!(modes(&fm("module, raw", "")), [Mode::Module]);
}

#[test]
fn harness_order() {
    assert_eq!(harness_files(&fm("", "")), ["assert.js", "sta.js"]);
    assert_eq!(
        harness_files(&fm("async", "b.js, a.js")),
        ["assert.js", "sta.js", "doneprintHandle.js", "b.js", "a.js"]
    );
    assert!(harness_files(&fm("raw", "a.js")).is_empty());
}

#[test]
fn strict_prefix_comes_first_and_raw_is_untouched() {
    let f = fm("", "");
    let strict = job(
        "a.js",
        Path::new("/t/a.js"),
        "body();\n",
        &f,
        "HARNESS\n",
        Mode::Strict,
        Duration::from_secs(1),
    );
    assert_eq!(
        strict.script_source(),
        "\"use strict\";\nHARNESS\nbody();\n"
    );
    let sloppy = job(
        "a.js",
        Path::new("/t/a.js"),
        "body();\n",
        &f,
        "HARNESS\n",
        Mode::Sloppy,
        Duration::from_secs(1),
    );
    assert_eq!(sloppy.script_source(), "HARNESS\nbody();\n");
    let r = fm("raw", "");
    let raw = job(
        "r.js",
        Path::new("/t/r.js"),
        "'use strict'\nx",
        &r,
        "",
        Mode::Sloppy,
        Duration::from_secs(1),
    );
    assert_eq!(raw.script_source(), "'use strict'\nx");
    assert!(raw.raw);
    let m = fm("module", "");
    let module = job(
        "m.js",
        Path::new("/t/m.js"),
        "export {};",
        &m,
        "H\n",
        Mode::Module,
        Duration::from_secs(1),
    );
    assert_eq!(module.kind, SourceKind::Module);
    assert_eq!(module.prefix, "");
}

#[test]
fn host_gaps() {
    let none = HostCapabilities {
        can_block: false,
        agent: false,
        is_html_dda: false,
    };
    let all = HostCapabilities {
        can_block: true,
        agent: true,
        is_html_dda: true,
    };
    assert!(host_gap(&fm("CanBlockIsTrue", ""), "", &none).is_some());
    assert!(host_gap(&fm("CanBlockIsTrue", ""), "", &all).is_none());
    assert!(host_gap(&fm("CanBlockIsFalse", ""), "", &all).is_some());
    assert!(host_gap(&fm("CanBlockIsFalse", ""), "", &none).is_none());
    assert!(
        host_gap(&fm("", ""), "$262.agent.start('')", &none)
            .unwrap()
            .contains("agent")
    );
    assert!(host_gap(&fm("", ""), "$262.agent.start('')", &all).is_none());
    let dda = parse_test_file("/*---\ndescription: d\nfeatures: [IsHTMLDDA]\n---*/").unwrap();
    assert!(host_gap(&dda, "", &none).unwrap().contains("IsHTMLDDA"));
    assert!(host_gap(&dda, "", &all).is_none());
}

#[test]
fn harness_reader_concatenates_and_refuses_escapes() {
    let dir = std::env::temp_dir().join(format!("hg-test262-harness-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("assert.js"), "A").unwrap();
    std::fs::write(dir.join("sta.js"), "S\n").unwrap();
    std::fs::write(dir.join("x.js"), "X").unwrap();
    let h = Harness::new(&dir);
    assert_eq!(h.text_for(&fm("", "x.js")).unwrap(), "A\nS\nX\n");
    assert!(
        h.text_for(&fm("", "missing.js"))
            .unwrap_err()
            .contains("missing.js")
    );
    assert!(h.file("../etc.js").is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}
