//! The reader over whole files.

use std::path::{Path, PathBuf};

use hg_wast::script::{self, Cmd, ModuleKind};

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it/wast")
}

fn read(name: &str) -> String {
    let p = dir().join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn parse(name: &str) -> (String, Vec<Cmd>) {
    let src = read(name);
    let cmds = script::parse(&src).unwrap_or_else(|e| panic!("{name}: {}", e.display(&src)));
    (src, cmds)
}

/// How many of each command a file holds, as the ledger would say it.
fn census(cmds: &[Cmd]) -> Vec<(&'static str, usize)> {
    let mut out: Vec<(&'static str, usize)> = Vec::new();
    for c in cmds {
        match out.iter_mut().find(|(k, _)| *k == c.keyword()) {
            Some((_, n)) => *n += 1,
            None => out.push((c.keyword(), 1)),
        }
    }
    out.sort_unstable();
    out
}

/// Every command the reader knows, in one file, counted.
///
/// The count is the point: a reader that quietly stopped recognising a
/// command would still parse this file, and would still report success.
#[test]
fn a_whole_script_is_read_command_by_command() {
    let (_, cmds) = parse("script.wast");
    assert_eq!(
        census(&cmds),
        [
            ("assert_exception", 1),
            ("assert_exhaustion", 1),
            ("assert_invalid", 1),
            ("assert_malformed", 1),
            ("assert_return", 5),
            ("assert_trap", 2),
            ("assert_unlinkable", 1),
            ("get", 1),
            ("invoke", 1),
            ("module", 3),
            ("module instance", 1),
            ("register", 1),
        ]
    );
}

#[test]
fn the_module_forms_survive_the_round_trip_to_disk() {
    let (src, cmds) = parse("script.wast");
    let kinds: Vec<&ModuleKind> = cmds
        .iter()
        .flat_map(Cmd::modules)
        .map(|m| &m.kind)
        .collect();
    assert!(kinds.iter().any(|k| **k == ModuleKind::Text));
    assert!(kinds.iter().any(|k| matches!(k, ModuleKind::Quote(_))));
    assert!(
        kinds
            .iter()
            .any(|k| matches!(k, ModuleKind::Binary(b) if b.starts_with(b"\0asm"))),
        "the binary module's bytes should decode"
    );
    for c in &cmds {
        for m in c.modules() {
            m.reparses(&src).unwrap_or_else(|e| panic!("{e}"));
        }
    }
}

/// The lexical cases. If any of these were read as raw bytes instead of
/// tokens, a module's span would run to the wrong place and the file
/// would fail to parse — or, worse, parse into the wrong commands.
#[test]
fn the_awkward_file_reads_as_three_commands() {
    let (src, cmds) = parse("awkward.wast");
    assert_eq!(
        census(&cmds),
        [("assert_malformed", 1), ("assert_return", 1), ("module", 1)]
    );
    let Cmd::Module(m) = &cmds[0] else {
        panic!("the first command is the module");
    };
    assert_eq!(m.id.as_deref(), Some("$keyword"));
    assert!(
        m.text(&src).ends_with(')'),
        "the span should end at the module's own parenthesis"
    );
    for c in &cmds {
        for m in c.modules() {
            m.reparses(&src).unwrap_or_else(|e| panic!("{e}"));
        }
    }
}

#[test]
fn a_file_of_module_fields_is_one_module() {
    let (src, cmds) = parse("inline.wast");
    assert_eq!(census(&cmds), [("module", 1)]);
    let Cmd::Module(m) = &cmds[0] else {
        panic!("a module");
    };
    assert_eq!(m.text(&src).trim(), src.trim());
}
