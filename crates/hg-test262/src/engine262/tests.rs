//! engine262 adapter tests that need no engine: the driver text, the
//! module copy and the report reader.

use super::*;
use crate::expand::{self, Mode};
use crate::frontmatter::parse_test_file;

fn job(src: &str, mode: Mode) -> Job {
    let fm = parse_test_file(src).unwrap();
    expand::job(
        "dir/t.js",
        Path::new("/t262/test/dir/t.js"),
        src,
        &fm,
        "HARNESS();\n",
        mode,
        Duration::from_secs(5),
    )
}

#[test]
fn js_string_escapes() {
    assert_eq!(js_string("a\"b\\c\nd\re"), "\"a\\\"b\\\\c\\nd\\re\"");
    assert_eq!(
        js_string("é\u{2028}😀\t"),
        "\"\\u00e9\\u2028\\ud83d\\ude00\\u0009\""
    );
}

#[test]
fn script_driver_puts_the_marker_after_the_strict_prefix() {
    let j = job("/*---\ndescription: d\n---*/\nbody();\n", Mode::Strict);
    let d = script_driver(&j);
    assert!(
        d.contains(&js_string(&format!("\"use strict\";\n{}\nHARNESS();\n", eval_marker()))[..30])
    );
    assert!(d.contains("$.evalScript(src)"));
    let raw = job(
        "/*---\ndescription: d\nflags: [raw]\n---*/\nbody();\n",
        Mode::Sloppy,
    );
    assert!(!script_driver(&raw).contains("evaluating"));
}

#[test]
fn module_copy_keeps_hashbang_first_and_raw_untouched() {
    let mut j = job(
        "/*---\ndescription: d\nflags: [module]\n---*/\nexport {};\n",
        Mode::Module,
    );
    assert!(module_copy(&j).starts_with(&eval_marker()));
    j.test = "#!/usr/bin/env x\nexport {};".into();
    assert!(module_copy(&j).starts_with("#!/usr/bin/env x\nprint("));
    j.raw = true;
    assert_eq!(module_copy(&j), j.test);
    assert!(module_driver(&j, "t.js").contains("import(\"./t.js\")"));
}

#[test]
fn finds_relative_specifiers() {
    let s = "import a from './a_FIXTURE.js';\nexport * from \"./b.js\";\nimport('./a_FIXTURE.js'); x = '../no'; y = './';";
    assert_eq!(relative_specifiers(s), ["./a_FIXTURE.js", "./b.js", "./"]);
}

fn report(lines: &[&str]) -> String {
    lines.iter().map(|l| format!("{l}\n")).collect()
}

#[test]
fn reads_reports() {
    let m = |s: &str| format!("{MARK}{s}");
    let ev = m("evaluating");
    // Runtime error: the marker printed first.
    let (c, prints, _) = parse_report(
        &report(&[
            &ev,
            "hello",
            &m("completion throw"),
            &m("name TypeError"),
            &m("stack at x"),
            "  at y",
            &m("message two"),
            "lines",
            &m("end"),
        ]),
        SourceKind::Script,
        false,
        None,
    );
    assert_eq!(
        c,
        Some(Completion::Threw {
            phase: Some(Phase::Runtime),
            name: "TypeError".into(),
            message: "two\nlines".into()
        })
    );
    assert_eq!(prints, ["hello"]);
    // Parse error: no marker.
    let throw = [
        m("completion throw"),
        m("name SyntaxError"),
        m("stack "),
        m("message bad"),
        m("end"),
    ];
    let throw: Vec<&str> = throw.iter().map(String::as_str).collect();
    let (c, _, _) = parse_report(&report(&throw), SourceKind::Script, false, None);
    assert!(matches!(
        c,
        Some(Completion::Threw {
            phase: Some(Phase::Parse),
            ..
        })
    ));
    // Raw: unknown phase.
    let (c, _, _) = parse_report(&report(&throw), SourceKind::Script, true, None);
    assert!(matches!(c, Some(Completion::Threw { phase: None, .. })));
    // Module: the test file's own location means parse, else resolution.
    let own = [
        m("completion throw"),
        m("name SyntaxError"),
        m("stack SyntaxError: x"),
        "/w/t.js:1:5".to_string(),
        m("message x"),
        m("end"),
    ];
    let own: Vec<&str> = own.iter().map(String::as_str).collect();
    let (c, _, _) = parse_report(&report(&own), SourceKind::Module, false, Some("/w/t.js"));
    assert!(matches!(
        c,
        Some(Completion::Threw {
            phase: Some(Phase::Parse),
            ..
        })
    ));
    let (c, _, _) = parse_report(
        &report(&own),
        SourceKind::Module,
        false,
        Some("/w/other.js"),
    );
    assert!(matches!(
        c,
        Some(Completion::Threw {
            phase: Some(Phase::Resolution),
            ..
        })
    ));
    // Normal, then async prints.
    let (c, prints, _) = parse_report(
        &report(&[&ev, &m("completion normal"), "Test262:AsyncTestComplete"]),
        SourceKind::Script,
        false,
        None,
    );
    assert_eq!(c, Some(Completion::Normal));
    assert_eq!(prints, ["Test262:AsyncTestComplete"]);
    // Nothing reported.
    assert_eq!(
        parse_report("crash\n", SourceKind::Script, false, None).0,
        None
    );
}

#[test]
fn reads_the_cli_uncaught_report() {
    let p = |s: &str| parse_uncaught(s);
    assert_eq!(
        p("TypeError: boom\n    at x.js:1:2"),
        Some(("TypeError".into(), "boom".into()))
    );
    assert_eq!(
        p("SyntaxError: Unexpected token\n/w/x.js:2:5\n"),
        Some(("SyntaxError".into(), "Unexpected token".into()))
    );
    assert_eq!(
        p("Test262Error { message: 'm' }"),
        Some(("Test262Error".into(), "{ message: 'm' }".into()))
    );
    assert_eq!(
        p("'Test262: This statement should not be evaluated.'")
            .unwrap()
            .0,
        "non-object:string"
    );
    assert_eq!(p("42").unwrap().0, "non-object:number");
    assert_eq!(p(""), None);
    assert_eq!(p("node:internal/x boom"), None);
}

#[test]
fn direct_script_and_instrumented_modules() {
    let j = job(
        "/*---\ndescription: d\n---*/\nimport('./a_FIXTURE.js');\n",
        Mode::Strict,
    );
    let d = direct_script(&j);
    assert!(d.starts_with("\"use strict\";\nprint("));
    assert!(d.ends_with("\n;print(\"#izcal-t262#completion normal\");\n"));
    assert_eq!(
        instrument_module("export {};"),
        format!("{}export {{}};", eval_marker())
    );
    assert!(names_location("SyntaxError: x\n/w/t.js:3:1", "/w/t.js"));
    assert!(!names_location(
        "Module \"/w/t.js\" has no export",
        "/w/t.js"
    ));
}

#[test]
fn parse_errors_and_instantiation_errors_are_told_apart() {
    // Shapes observed from engine262's `Error.stack` and CLI reports.
    assert!(stack_shows_parse_error(
        "SyntaxError: Unexpected token\nvar var = 1;\n    ^^^\n    at bound apply (native)"
    ));
    assert!(stack_shows_parse_error(
        "SyntaxError: x\n/w/t.js:2:5\nvar var;\n    ^^^\n"
    ));
    assert!(!stack_shows_parse_error(
        "SyntaxError: undefined is already declared\n    at <anonymous>\n    at bound apply (native)"
    ));
    let m = |s: &str| format!("{MARK}{s}\n");
    let gdi = [
        m("completion throw"),
        m("name SyntaxError"),
        m("stack SyntaxError: u"),
        "    at <anonymous>\n".into(),
        m("message u"),
        m("end"),
    ]
    .concat();
    let (c, _, _) = parse_report(&gdi, SourceKind::Script, false, None);
    assert!(matches!(
        c,
        Some(Completion::Threw {
            phase: Some(Phase::Runtime),
            ..
        })
    ));
}
