//! The rules that do not depend on the resolution: declared rows, crate
//! layout and metadata, profiles, G5's manifest and inventory paths, and
//! the tier and `no_std` halves of `[[direct-only]]`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::code::{Scan, entry_files};
use crate::config::{ConfigError, any_glob, read_toml};
use crate::value::Value;
use crate::{Inputs, Report};

/// Runs the workspace rules. `scan` is `None` when code checks are off.
///
/// # Errors
/// [`ConfigError`] when a manifest cannot be read.
pub fn check(inputs: &Inputs, scan: Option<&Scan>, report: &mut Report) -> Result<(), ConfigError> {
    let Some(g) = inputs.graphs.first() else {
        return Ok(());
    };
    let root_manifest = read_toml(&inputs.root.join("Cargo.toml"))?;
    // `.cargo/config.toml` profile keys override the manifest's (the
    // Cargo Book, "Profiles": config profiles take precedence).
    let mut merged = profile_tables(&root_manifest);
    let config = inputs.root.join(".cargo/config.toml");
    if config.exists() {
        for (name, keys) in profile_tables(&read_toml(&config)?) {
            merged.entry(name).or_default().extend(keys);
        }
    }
    profiles(&merged, report);
    let rules = &inputs.rules;
    let forbid_files = scan.map(|s| s.files("forbid-unsafe"));
    let no_std_files = scan.map(|s| s.files("no-std"));
    let mut members = BTreeMap::new();
    for m in g.members() {
        let p = &g.packages[m];
        members.insert(p.name.clone(), m);
        let name = p.name.as_str();
        if !inputs.tiers.crates.contains_key(name) {
            report.add(
                "tiers",
                name,
                "no row in tools/tiers.toml: declare its tier and layer",
                None,
            );
        }

        // CS-ST-1, CS-ST-14 and CS-TE-2 (A.6).
        let exempt = any_glob(&rules.layout_exempt, name);
        let want_dir = if exempt {
            inputs.root.join(name)
        } else {
            inputs.root.join(&rules.layout_dir).join(name)
        };
        if p.dir() != want_dir {
            report.add(
                "CS-ST-1",
                name,
                format!(
                    "flat layout: the crate must live in {}",
                    want_dir
                        .strip_prefix(&inputs.root)
                        .unwrap_or(&want_dir)
                        .display()
                ),
                None,
            );
        }
        if !exempt && !name.starts_with(&rules.layout_prefix) {
            report.add(
                "CS-ST-1",
                name,
                format!("crate names start with {:?}", rules.layout_prefix),
                None,
            );
        }
        if p.version != rules.layout_version {
            report.add(
                "CS-ST-14",
                name,
                format!(
                    "version is {}; workspace crates are {}",
                    p.version, rules.layout_version
                ),
                None,
            );
        }
        if !p.publish_false {
            report.add("CS-ST-14", name, "publish = false is required", None);
        }
        let tests: Vec<_> = p
            .targets
            .iter()
            .filter(|t| t.kinds.iter().any(|k| k == "test"))
            .collect();
        if tests.len() > 1 {
            let names: Vec<&str> = tests.iter().map(|t| t.name.as_str()).collect();
            report.add(
                "CS-TE-2",
                name,
                format!(
                    "one integration-test binary per crate (tests/it/main.rs); found {}",
                    names.join(", ")
                ),
                None,
            );
        }
        for t in &tests {
            if !t.src_path.starts_with(p.dir().join("tests")) {
                report.add(
                    "CS-TE-2",
                    name,
                    format!("integration test {} is outside tests/", t.name),
                    None,
                );
            }
        }

        // G5.
        let manifest = read_toml(&p.manifest_path)?;
        let level = unsafe_code_level(&manifest, &root_manifest);
        if let Some(entries) = inputs.inventory.crates.get(name) {
            if !matches!(level.as_deref(), Some("deny" | "forbid")) {
                report.add(
                    "G5",
                    name,
                    "an inventoried crate sets unsafe_code = \"deny\" in its manifest, so only the inventoried modules' #[expect]s admit unsafe",
                    None,
                );
            }
            for e in entries {
                if !entry_files(p.dir(), e).iter().any(|f| f.is_file()) {
                    report.add(
                        "G5",
                        name,
                        format!("unsafe-inventory.toml entry {e:?} names no file in the crate"),
                        None,
                    );
                }
            }
        } else if level.as_deref() != Some("forbid") {
            for t in &p.targets {
                match &forbid_files {
                    Some(files) if files.contains(t.src_path.as_path()) => {}
                    Some(_) => report.add(
                        "G5",
                        name,
                        format!(
                            "target {} ({}) does not forbid unsafe_code: add #![forbid(unsafe_code)] to its root, set unsafe_code = \"forbid\" in [lints.rust], or inventory it in unsafe-inventory.toml",
                            t.name,
                            rel(&inputs.root, &t.src_path)
                        ),
                        None,
                    ),
                    None => {}
                }
            }
        }
    }
    for krate in inputs.inventory.crates.keys() {
        if !members.contains_key(krate) {
            report.add(
                "G5",
                krate,
                "unsafe-inventory.toml lists a crate that is not in the workspace",
                None,
            );
        }
    }

    // [[direct-only]]: declared tier and no_std.
    for d in &rules.direct_only {
        for (name, &m) in &members {
            if !any_glob(&d.crates, name) {
                continue;
            }
            if let Some(want) = d.tier {
                let have = inputs.tiers.crates.get(name).map(|r| r.tier);
                if have != Some(want) {
                    report.add(
                        &d.rule,
                        name,
                        format!("must be declared tier {} ({})", want.letter(), d.why),
                        None,
                    );
                }
            }
            if d.no_std
                && let Some(files) = &no_std_files
            {
                let p = &g.packages[m];
                for t in p
                    .targets
                    .iter()
                    .filter(|t| t.kinds.iter().any(|k| k == "lib"))
                {
                    if !files.contains(t.src_path.as_path()) {
                        report.add(
                            &d.rule,
                            name,
                            format!(
                                "the library root {} must be #![no_std] ({})",
                                rel(&inputs.root, &t.src_path),
                                d.why
                            ),
                            None,
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).display().to_string()
}

/// A lint level, written as `"forbid"` or `{ level = "forbid", ... }`.
fn level(v: &Value) -> Option<String> {
    match v {
        Value::Str(s) => Some(s.clone()),
        Value::Table(_) => v.get("level").and_then(Value::as_str).map(str::to_owned),
        _ => None,
    }
}

/// The `unsafe_code` level a member's manifest sets for all its targets:
/// its own `[lints.rust]`, or the workspace's when it inherits.
pub fn unsafe_code_level(manifest: &Value, root_manifest: &Value) -> Option<String> {
    let lints = manifest.get("lints")?;
    if lints.get("workspace").and_then(Value::as_bool) == Some(true) {
        return root_manifest
            .path(&["workspace", "lints", "rust", "unsafe_code"])
            .and_then(level);
    }
    lints.path(&["rust", "unsafe_code"]).and_then(level)
}

type ProfileTable = BTreeMap<String, BTreeMap<String, Value>>;

/// The `[profile.*]` tables of a manifest or config file.
fn profile_tables(doc: &Value) -> ProfileTable {
    doc.get("profile")
        .and_then(Value::as_table)
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_table().map(|v| (k.clone(), v.clone())))
                .collect()
        })
        .unwrap_or_default()
}

/// CS-ER-2 (A.1, A.6): `release` and every custom profile resolve to
/// `panic = "abort"`. `dev` and the test profiles unwind (Cargo forces
/// unwinding for tests, and `dev` matches it so dependencies build once).
fn profiles(profiles: &ProfileTable, report: &mut Report) {
    let builtin_unwind = ["dev", "test", "bench", "doc"];
    let mut names: Vec<&str> = profiles.keys().map(String::as_str).collect();
    if !profiles.contains_key("release") {
        names.push("release");
    }
    names.retain(|n| !builtin_unwind.contains(n));
    for name in names {
        let mut cur = name;
        let mut hops = 0;
        let resolved = loop {
            let p = profiles.get(cur);
            if let Some(v) = p.and_then(|p| p.get("panic")).and_then(Value::as_str) {
                break Some(v.to_owned());
            }
            match p.and_then(|p| p.get("inherits")).and_then(Value::as_str) {
                Some(parent) if hops < 16 => {
                    cur = parent;
                    hops += 1;
                }
                // Cargo's built-in default for `release` and `dev`.
                _ => break None,
            }
        };
        if resolved.as_deref() != Some("abort") {
            report.add(
                "CS-ER-2",
                &format!("profile.{name}"),
                format!(
                    "builds shipping or simulation binaries, so it must resolve to panic = \"abort\" (resolves to {})",
                    resolved.as_deref().unwrap_or("Cargo's default, unwind")
                ),
                None,
            );
        }
    }
}
