//! Front matter parser tests. The inputs are written for these tests in
//! the shape CONTRIBUTING.md and INTERPRETING.md describe; none is copied
//! from test262.

use super::*;

fn parse(body: &str) -> Result<FrontMatter, FrontMatterError> {
    parse_test_file(&format!("// header\n/*---\n{body}\n---*/\ncode();\n"))
}

#[test]
fn the_plain_keys() {
    let fm = parse("description: Adds two numbers\nesid: sec-addition-operator").unwrap();
    assert_eq!(fm.description.as_deref(), Some("Adds two numbers"));
    assert_eq!(fm.esid.as_deref(), Some("sec-addition-operator"));
    assert_eq!(fm.flags, Flags::default());
    assert!(fm.includes.is_empty() && fm.features.is_empty() && fm.negative.is_none());
}

#[test]
fn every_flag() {
    let fm = parse(
        "description: d\nflags: [noStrict, async, generated, CanBlockIsFalse, non-deterministic]",
    )
    .unwrap();
    let f = fm.flags;
    assert!(
        f.no_strict && f.is_async && f.generated && f.can_block_is_false && f.non_deterministic
    );
    assert!(!f.only_strict && !f.module && !f.raw && !f.can_block_is_true);
    let fm = parse("description: d\nflags: [onlyStrict, CanBlockIsTrue]").unwrap();
    assert!(fm.flags.only_strict && fm.flags.can_block_is_true);
    let fm = parse("description: d\nflags: [module]").unwrap();
    assert!(fm.flags.module);
    let fm = parse("description: d\nflags: [raw]").unwrap();
    assert!(fm.flags.raw);
}

#[test]
fn flow_and_block_lists() {
    let fm = parse("description: d\nincludes: [compareArray.js, 'propertyHelper.js']\nfeatures:\n  - Symbol\n  - \"BigInt\"\nlocale: [en, en-US]").unwrap();
    assert_eq!(fm.includes, ["compareArray.js", "propertyHelper.js"]);
    assert_eq!(fm.features, ["Symbol", "BigInt"]);
    assert_eq!(fm.locale, ["en", "en-US"]);
    let fm = parse("description: d\nflags: []").unwrap();
    assert_eq!(fm.flags, Flags::default());
}

#[test]
fn negative_block_and_flow() {
    let fm = parse("description: d\nnegative:\n  phase: parse\n  type: SyntaxError").unwrap();
    assert_eq!(
        fm.negative,
        Some(Negative {
            phase: Phase::Parse,
            error_type: "SyntaxError".into()
        })
    );
    let fm = parse("description: d\nnegative: {phase: resolution, type: ReferenceError}").unwrap();
    assert_eq!(fm.negative.unwrap().phase, Phase::Resolution);
    let fm = parse("description: d\nnegative:\n  phase: runtime\n  type: Test262Error").unwrap();
    assert_eq!(fm.negative.unwrap().error_type, "Test262Error");
}

#[test]
fn block_scalars_and_folded_plain_scalars() {
    let fm = parse("description: |\n  line one\n    indented\n\n  after blank\ninfo: >\n  folded\n  text\nesid: x").unwrap();
    assert_eq!(
        fm.description.as_deref(),
        Some("line one\n  indented\n\nafter blank")
    );
    assert_eq!(fm.esid.as_deref(), Some("x"));
    let fm = parse("description: >-\n  a\n  b").unwrap();
    assert_eq!(fm.description.as_deref(), Some("a b"));
    let fm = parse("description: a long\n  description continued\nesid: y").unwrap();
    assert_eq!(
        fm.description.as_deref(),
        Some("a long description continued")
    );
    // An empty block scalar, as a harness file's header has it.
    let fm = parse("description: |\nesid: z").unwrap();
    assert_eq!(fm.description.as_deref(), Some(""));
}

#[test]
fn quoted_scalars_and_comments() {
    let fm = parse("# a comment\ndescription: 'it''s quoted: yes'\nesid: \"a\\\"b\" ").unwrap();
    assert_eq!(fm.description.as_deref(), Some("it's quoted: yes"));
    assert_eq!(fm.esid.as_deref(), Some("a\"b"));
    let fm = parse("description: plain # trailing comment").unwrap();
    assert_eq!(fm.description.as_deref(), Some("plain"));
}

#[test]
fn informative_keys_are_accepted() {
    let fm = parse("description: d\nauthor: Someone\nes5id: 1.2.3\nes6id: 4.5\ninfo: |\n  quoted spec text: with colons\n  - and dashes").unwrap();
    assert_eq!(fm.description.as_deref(), Some("d"));
}

#[test]
fn crlf_line_ends() {
    let fm =
        parse_test_file("/*---\r\ndescription: d\r\nflags: [onlyStrict]\r\n---*/\r\n").unwrap();
    assert!(fm.flags.only_strict);
    assert_eq!(fm.description.as_deref(), Some("d"));
}

fn refused(src: &str) -> String {
    match parse_test_file(src) {
        Ok(fm) => panic!("accepted {src:?} as {fm:?}"),
        Err(e) => e.to_string(),
    }
}

fn refused_body(body: &str) -> String {
    match parse(body) {
        Ok(fm) => panic!("accepted {body:?} as {fm:?}"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn refuses_missing_unclosed_and_repeated_blocks() {
    assert!(refused("code();").contains("no `/*---` block"));
    assert!(refused("/*---\ndescription: d\n").contains("never closed"));
    assert!(refused("/*---\ndescription: d\n---*/\n/*---\n---*/").contains("second `/*---`"));
    assert!(refused("/*--- description: d\n---*/").contains("text after `/*---`"));
}

#[test]
fn refuses_unknown_and_contradictory_flags() {
    assert!(
        refused_body("description: d\nflags: [strictOnly]").contains("unknown flag `strictOnly`")
    );
    assert!(
        refused_body("description: d\nflags: [onlyStrict, noStrict]")
            .contains("contradictory flags")
    );
    assert!(
        refused_body("description: d\nflags: [raw, onlyStrict]").contains("contradictory flags")
    );
    assert!(
        refused_body("description: d\nflags: [module, noStrict]").contains("contradictory flags")
    );
    assert!(
        refused_body("description: d\nflags: [CanBlockIsTrue, CanBlockIsFalse]")
            .contains("contradictory")
    );
    assert!(refused_body("description: d\nflags: [async, async]").contains("given twice"));
}

#[test]
fn refuses_bad_negative() {
    assert!(
        refused_body("description: d\nnegative:\n  phase: early\n  type: SyntaxError")
            .contains("not parse, resolution or runtime")
    );
    assert!(refused_body("description: d\nnegative:\n  phase: parse").contains("no `type`"));
    assert!(refused_body("description: d\nnegative:\n  type: SyntaxError").contains("no `phase`"));
    assert!(refused_body("description: d\nnegative: SyntaxError").contains("must be a mapping"));
    assert!(
        refused_body("description: d\nnegative:\n  phase: parse\n  type: Syntax Error")
            .contains("not a constructor name")
    );
    assert!(
        refused_body("description: d\nnegative:\n  phase: parse\n  type: SyntaxError\n  when: now")
            .contains("unknown key `when`")
    );
}

#[test]
fn refuses_malformed_yaml() {
    // The error names the file line: the body starts on line 3.
    let e = parse("description: d\nflags: [onlyStrict").unwrap_err();
    assert_eq!(e.line, 4);
    assert!(e.message.contains("not closed"));
    assert!(refused_body("description: d\ndescription: e").contains("given twice"));
    assert!(refused_body("description: d\nmystery: 1").contains("unknown key `mystery`"));
    assert!(refused_body("description d").contains("expected `key: value`"));
    assert!(refused_body("description: d\n\tflags: [raw]").contains("tab"));
    assert!(refused_body("description: d\nincludes: [assert]").contains("does not end in `.js`"));
    assert!(refused_body("description: d\nincludes: [a.js,, b.js]").contains("empty item"));
    assert!(refused_body("description: d\nfeatures: [[a]]").contains("nested flow"));
    assert!(refused_body("description: d\nfeatures: [a] b").contains("text after `]`"));
    assert!(refused_body("description: 'open").contains("unclosed single quote"));
    assert!(refused_body("description: d\n  more: x").contains("already has a value"));
    assert!(refused_body("description: d\nflags: [raw,\n  async]").contains("on one line"));
    assert!(refused_body("description: d\nfeatures:\n  - a\n  b").contains("expected `- item`"));
    assert!(refused_body("description: d\nflags: yes").contains("must be a list"));
    assert!(refused_body("description: [a]").contains("must be a string"));
}

#[test]
fn lone_cr_line_ends_and_an_indented_mapping() {
    let fm = parse_test_file("// c\r/*---\rdescription: d\rflags: [noStrict]\r---*/\r").unwrap();
    assert!(fm.flags.no_strict);
    let fm =
        parse_test_file("/*---\n description: >\n    folded\n    text\n flags: [raw]\n---*/\n")
            .unwrap();
    assert_eq!(fm.description.as_deref(), Some("folded text"));
    assert!(fm.flags.raw);
    let e = parse_test_file("/*---\n  description: d\n flags: [raw]\n---*/\n").unwrap_err();
    assert!(e.message.contains("less indented"));
    assert_eq!(e.line, 3);
}
