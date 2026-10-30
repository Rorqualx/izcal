//! The code-level checks, run through `ast-grep` (structural matching on
//! the Rust syntax tree, so comments and strings never match).
//!
//! One `ast-grep scan` over every member's directory evaluates all the
//! rules below at once; [`check`] then applies the G5 inventory rules and
//! the Q-CS-1 crypto rule to the matches. The crate-root rules
//! (`forbid(unsafe_code)`, `no_std`) are applied by the workspace rules.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::json;
use crate::value::Value;
use crate::{Inputs, Report};

/// One match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// The rule that matched (an `id` from [`rules_yaml`]).
    pub rule: String,
    /// The file, absolute.
    pub file: PathBuf,
    /// 1-based line.
    pub line: usize,
    /// The matched source text.
    pub text: String,
}

/// The matches of one scan.
#[derive(Debug, Clone, Default)]
pub struct Scan {
    /// Every match.
    pub matches: Vec<Match>,
}

impl Scan {
    /// Files with at least one match of `rule`.
    pub fn files(&self, rule: &str) -> BTreeSet<&Path> {
        self.matches
            .iter()
            .filter(|m| m.rule == rule)
            .map(|m| m.file.as_path())
            .collect()
    }
}

/// Matches an attribute `#[cfg(test)]`, exactly.
const CFG_TEST: &str = r"'^#\[\s*cfg\s*\(\s*test\s*\)\s*\]$'";

/// A `follows:` clause matching a `#[cfg(test)]` attribute among the
/// preceding siblings, looking back over other attributes and comments
/// (doc comments are comments in this grammar). `indent` is the column of
/// the `follows:` key; its first line carries no indent, so it can follow
/// a `- ` list marker.
fn after_cfg_test(indent: usize) -> String {
    let a = " ".repeat(indent + 2);
    let b = " ".repeat(indent + 4);
    let c = " ".repeat(indent + 6);
    format!(
        "follows:\n{a}stopBy:\n{b}not:\n{c}any: [{{kind: attribute_item}}, {{kind: line_comment}}, {{kind: block_comment}}]\n{a}kind: attribute_item\n{a}regex: {CFG_TEST}"
    )
}

/// The `ast-grep` rules. `traits` is a regex alternation of the crypto
/// trait names.
pub fn rules_yaml(traits: &str) -> String {
    format!(
        r"id: forbid-unsafe
language: rust
severity: hint
rule:
  kind: inner_attribute_item
  regex: '^#!\[\s*forbid\s*\([^)]*\bunsafe_code\b'
  inside:
    kind: source_file
---
id: no-std
language: rust
severity: hint
rule:
  kind: inner_attribute_item
  regex: '^#!\[\s*no_std\s*\]$'
  inside:
    kind: source_file
---
id: expect-unsafe
language: rust
severity: hint
rule:
  any:
    - kind: inner_attribute_item
    - kind: attribute_item
  regex: '\bexpect\s*\([^)]*\bunsafe_code\b'
---
id: lower-unsafe
language: rust
severity: hint
rule:
  any:
    - kind: inner_attribute_item
    - kind: attribute_item
  regex: '\b(allow|warn)\s*\([^)]*\bunsafe_code\b'
---
id: crypto-impl
language: rust
severity: hint
rule:
  kind: impl_item
  has:
    field: trait
    regex: '(^|::)({traits})(<.*>)?$'
  not:
    any:
      - {direct}
      - inside:
          stopBy: end
          any:
            - kind: mod_item
            - kind: function_item
            - kind: impl_item
          {nested}
---
id: crypto-alias
language: rust
severity: hint
rule:
  kind: use_as_clause
  has:
    field: path
    regex: '(^|::)({traits})$'
---
id: crypto-macro
language: rust
severity: hint
rule:
  kind: macro_definition
  regex: '\b({traits})\b'
---
id: cfg-test-mod-decl
language: rust
severity: hint
rule:
  kind: mod_item
  not:
    has:
      kind: declaration_list
  {decl}
",
        direct = after_cfg_test(8),
        nested = after_cfg_test(10),
        decl = after_cfg_test(2),
    )
}

/// Runs the scan over every workspace member's directory.
///
/// # Errors
/// A message when `ast-grep` cannot run or prints something unexpected.
pub fn scan(ast_grep: &Path, inputs: &Inputs) -> Result<Scan, String> {
    let g = inputs
        .graphs
        .first()
        .ok_or_else(|| "no package graph".to_owned())?;
    let mut dirs: Vec<PathBuf> = g
        .members()
        .into_iter()
        .map(|m| g.packages[m].dir().to_path_buf())
        .collect();
    dirs.sort();
    dirs.dedup();
    if dirs.is_empty() {
        return Ok(Scan::default());
    }
    let traits = inputs.rules.crypto_traits.join("|");
    let out = Command::new(ast_grep)
        .arg("scan")
        .arg("--inline-rules")
        .arg(rules_yaml(&traits))
        .arg("--json=compact")
        .args(&dirs)
        .current_dir(&inputs.root)
        .output()
        .map_err(|e| format!("cannot run {}: {e}", ast_grep.display()))?;
    if !out.status.success() {
        return Err(format!(
            "{} failed: {}",
            ast_grep.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let text = String::from_utf8(out.stdout).map_err(|_utf8| "output is not UTF-8".to_owned())?;
    let doc = json::parse(&text).map_err(|e| format!("output is not JSON: {e}"))?;
    let arr = doc
        .as_array()
        .ok_or_else(|| "output is not a JSON array".to_owned())?;
    let mut matches = Vec::new();
    for m in arr {
        let get = |k: &str| m.get(k).and_then(Value::as_str).map(str::to_owned);
        let (Some(rule), Some(file), Some(text)) = (get("ruleId"), get("file"), get("text")) else {
            return Err("a match without ruleId, file or text".into());
        };
        let line = m
            .path(&["range", "start", "line"])
            .and_then(Value::as_int)
            .and_then(|l| usize::try_from(l).ok())
            .ok_or_else(|| "a match without a line".to_owned())?;
        let file = PathBuf::from(file);
        let file = if file.is_absolute() {
            file
        } else {
            inputs.root.join(file)
        };
        matches.push(Match {
            rule,
            file,
            line: line + 1,
            text,
        });
    }
    Ok(Scan { matches })
}

/// The member that owns `file` (the deepest member directory holding it).
fn owner<'g>(members: &'g [(String, PathBuf)], file: &Path) -> Option<&'g (String, PathBuf)> {
    members
        .iter()
        .filter(|(_, dir)| file.starts_with(dir))
        .max_by_key(|(_, dir)| dir.components().count())
}

/// The file an inventory entry names: a path ending in `.rs` relative to
/// the crate, or a module path under `src/` (`a::b` is `src/a/b.rs` or
/// `src/a/b/mod.rs`).
pub fn entry_files(crate_dir: &Path, entry: &str) -> Vec<PathBuf> {
    if Path::new(entry)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("rs"))
    {
        return vec![crate_dir.join(entry)];
    }
    let rel: PathBuf = entry.split("::").collect();
    let base = crate_dir.join("src").join(rel);
    vec![base.with_extension("rs"), base.join("mod.rs")]
}

/// The `reason = "..."` string of an attribute's text, if any.
fn reason(text: &str) -> Option<String> {
    let at = text.find("reason")?;
    let rest = text[at + "reason".len()..].trim_start().strip_prefix('=')?;
    let rest = rest.trim_start().strip_prefix('"')?;
    let mut s = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(s),
            '\\' => s.push(chars.next()?),
            _ => s.push(c),
        }
    }
    None
}

/// Files that are test-only: under a member's `tests/`, `benches/` or
/// `examples/`, or a module declared behind `#[cfg(test)]`.
fn test_only_files(inputs: &Inputs, scan: &Scan) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    let Some(g) = inputs.graphs.first() else {
        return (dirs, files);
    };
    let mut roots = BTreeSet::new();
    for m in g.members() {
        let p = &g.packages[m];
        for d in ["tests", "benches", "examples"] {
            dirs.push(p.dir().join(d));
        }
        roots.extend(p.targets.iter().map(|t| t.src_path.clone()));
    }
    for m in scan
        .matches
        .iter()
        .filter(|m| m.rule == "cfg-test-mod-decl")
    {
        let Some(name) = mod_name(&m.text) else {
            continue;
        };
        let parent = m.file.parent().unwrap_or(Path::new("."));
        let stem = m.file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let base = if roots.contains(&m.file) || matches!(stem, "lib" | "main" | "mod") {
            parent.to_path_buf()
        } else {
            parent.join(stem)
        };
        files.push(base.join(format!("{name}.rs")));
        dirs.push(base.join(&name));
    }
    (dirs, files)
}

/// The module name in `mod name;` (after attributes and visibility).
fn mod_name(text: &str) -> Option<String> {
    let at = text.rfind("mod ")?;
    let rest = &text[at + 4..];
    let name: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// G5's inventory reasons and `allow(unsafe_code)`, and Q-CS-1's crypto
/// rule, on the scan's matches.
pub fn check(inputs: &Inputs, scan: &Scan, report: &mut Report) {
    let Some(g) = inputs.graphs.first() else {
        return;
    };
    let members: Vec<(String, PathBuf)> = g
        .members()
        .into_iter()
        .map(|m| {
            (
                g.packages[m].name.clone(),
                g.packages[m].dir().to_path_buf(),
            )
        })
        .collect();
    let rel = |p: &Path| {
        p.strip_prefix(&inputs.root)
            .unwrap_or(p)
            .display()
            .to_string()
    };

    // G5: every expect(unsafe_code) sits in an inventoried file and
    // names that entry; every entry still has one.
    let mut entry_used: BTreeMap<(String, String), bool> = BTreeMap::new();
    for (krate, entries) in &inputs.inventory.crates {
        for e in entries {
            entry_used.insert((krate.clone(), e.clone()), false);
        }
    }
    for m in &scan.matches {
        let Some((krate, dir)) = owner(&members, &m.file) else {
            continue;
        };
        let at = format!("{}:{}", rel(&m.file), m.line);
        match m.rule.as_str() {
            "lower-unsafe" => report.add(
                "G5",
                krate,
                format!("{at}: allow or warn on unsafe_code bypasses the inventory; use #[expect(unsafe_code, reason = \"G5: <entry> ...\")] in an inventoried module"),
                None,
            ),
            "expect-unsafe" => {
                let entries = inputs.inventory.crates.get(krate);
                let entry = entries.and_then(|es| {
                    es.iter()
                        .find(|e| entry_files(dir, e).contains(&m.file))
                });
                match (entries, entry) {
                    (None, _) => report.add(
                        "G5",
                        krate,
                        format!("{at}: expect(unsafe_code) in a crate that is not in unsafe-inventory.toml"),
                        None,
                    ),
                    (Some(_), None) => report.add(
                        "G5",
                        krate,
                        format!("{at}: expect(unsafe_code) in a file no unsafe-inventory.toml entry names"),
                        None,
                    ),
                    (Some(_), Some(e)) => {
                        let ok = reason(&m.text).is_some_and(|r| {
                            r.strip_prefix("G5: ")
                                .and_then(|r| r.strip_prefix(e.as_str()))
                                .is_some_and(|tail| tail.is_empty() || tail.starts_with(' '))
                        });
                        if ok {
                            entry_used.insert((krate.clone(), e.clone()), true);
                        } else {
                            report.add(
                                "G5",
                                krate,
                                format!("{at}: the reason must start \"G5: {e}\" (the inventory entry)"),
                                None,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
    for ((krate, e), used) in entry_used {
        let member = members.iter().any(|(n, _)| *n == krate);
        if member && !used {
            report.add(
                "G5",
                &krate,
                format!("unsafe-inventory.toml entry {e:?} has no #[expect(unsafe_code, reason = \"G5: {e} ...\")]; remove the entry or restore the expectation"),
                None,
            );
        }
    }

    // Q-CS-1: the crypto trait is implemented, aliased or named in a
    // macro only by the allowed crates, outside test code.
    let (test_dirs, test_files) = test_only_files(inputs, scan);
    for m in &scan.matches {
        if !matches!(
            m.rule.as_str(),
            "crypto-impl" | "crypto-alias" | "crypto-macro"
        ) {
            continue;
        }
        let Some((krate, _)) = owner(&members, &m.file) else {
            continue;
        };
        if inputs.rules.crypto_implementers.contains(krate) {
            continue;
        }
        let test_only =
            test_files.contains(&m.file) || test_dirs.iter().any(|d| m.file.starts_with(d));
        if test_only {
            continue;
        }
        let what = match m.rule.as_str() {
            "crypto-impl" => "implements",
            "crypto-alias" => "renames",
            _ => "names in a macro",
        };
        let first = m.text.lines().next().unwrap_or("").trim();
        report.add(
            "Q-CS-1",
            krate,
            format!(
                "{}:{}: {what} the cryptographic-randomness trait ({first}); only {} may, so a seeded crypto source exists only in the simulator",
                rel(&m.file),
                m.line,
                inputs.rules.crypto_implementers.join(" and ")
            ),
            None,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_reasons_and_mod_names() {
        assert_eq!(
            reason(r#"#![expect(unsafe_code, reason = "G5: os::vm (calls \"x\")")]"#).as_deref(),
            Some(r#"G5: os::vm (calls "x")"#)
        );
        assert_eq!(reason("#[expect(unsafe_code)]"), None);
        assert_eq!(
            mod_name("#[cfg(test)]\npub(crate) mod tests;").as_deref(),
            Some("tests")
        );
        assert_eq!(
            mod_name("#[cfg(test)]\n/// doc mod x\nmod y;").as_deref(),
            Some("y")
        );
    }

    #[test]
    fn entry_paths() {
        let d = Path::new("/w/crates/hg-x");
        assert_eq!(
            entry_files(d, "ipc::linux::sys"),
            vec![
                PathBuf::from("/w/crates/hg-x/src/ipc/linux/sys.rs"),
                PathBuf::from("/w/crates/hg-x/src/ipc/linux/sys/mod.rs")
            ]
        );
        assert_eq!(
            entry_files(d, "src/bin/a.rs"),
            vec![PathBuf::from("/w/crates/hg-x/src/bin/a.rs")]
        );
    }
}
