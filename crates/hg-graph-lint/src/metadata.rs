//! The package graph, read from `cargo metadata --format-version 1` (the
//! Cargo Book, "cargo metadata": `packages`, `workspace_members` and the
//! `resolve` graph, whose `deps[].dep_kinds` say whether each edge is a
//! normal, build or dev dependency and for which platform).
//!
//! One graph is one resolution: one target triple (`--filter-platform`,
//! so `cfg`-gated dependencies of other OSes are gone) and one feature
//! set (default features, or `--all-features`).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::json;
use crate::value::Value;

/// Which features the resolution enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Features {
    /// Every package's default features (what a plain build links).
    Default,
    /// `--all-features`: every link any feature combination could make.
    All,
}

impl Features {
    /// The name used in reports and in `graph-rules.toml`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::All => "all",
        }
    }
}

/// The kind of a dependency edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DepKind {
    /// `[dependencies]`: linked into the dependent's artifacts.
    Normal,
    /// `[build-dependencies]`: runs on the host at build time.
    Build,
    /// `[dev-dependencies]`: tests, benches and examples only.
    Dev,
}

/// One build target of a package (`packages[].targets[]`).
#[derive(Debug, Clone)]
pub struct Target {
    /// The target's name.
    pub name: String,
    /// `lib`, `bin`, `test`, `bench`, `example`, `custom-build`, `proc-macro`, ...
    pub kinds: Vec<String>,
    /// `lib`, `rlib`, `cdylib`, `staticlib`, `bin`, `proc-macro`, ...
    pub crate_types: Vec<String>,
    /// The crate root.
    pub src_path: PathBuf,
    /// Features the target needs before Cargo builds it.
    pub required_features: Vec<String>,
}

impl Target {
    /// Whether building this target produces a linked artifact that runs
    /// or loads outside Cargo: a binary or a C-ABI library.
    pub fn is_linked_artifact(&self) -> bool {
        self.kinds.iter().any(|k| k == "bin")
            || self
                .crate_types
                .iter()
                .any(|t| matches!(t.as_str(), "cdylib" | "staticlib" | "dylib"))
    }
}

/// One package.
#[derive(Debug, Clone)]
pub struct Package {
    /// The package id (opaque; unique in the graph).
    pub id: String,
    /// The package name.
    pub name: String,
    /// The version.
    pub version: String,
    /// The manifest.
    pub manifest_path: PathBuf,
    /// Whether it is a workspace member.
    pub member: bool,
    /// `publish = false` (an empty `publish` list in the metadata).
    pub publish_false: bool,
    /// Its targets.
    pub targets: Vec<Target>,
    /// Whether it is a procedural-macro crate.
    pub proc_macro: bool,
    /// The features this resolution enabled for it.
    pub enabled_features: BTreeSet<String>,
}

impl Package {
    /// The package's directory.
    pub fn dir(&self) -> &Path {
        self.manifest_path.parent().unwrap_or(Path::new("."))
    }

    /// Whether this resolution builds the package at all: it has a
    /// library, or one of its binaries is switched on. A package whose
    /// every binary needs a feature this resolution did not enable (a
    /// `required-features` binary, such as an `ai`-only main) is not built.
    pub fn is_built(&self) -> bool {
        self.targets.iter().any(|t| {
            t.kinds
                .iter()
                .any(|k| k == "lib" || k == "rlib" || k == "proc-macro")
        }) || self.builds_artifact()
    }

    /// Whether this resolution builds a linked artifact for it: some
    /// binary or C-ABI target whose required features are all enabled.
    pub fn builds_artifact(&self) -> bool {
        self.targets.iter().any(|t| {
            t.is_linked_artifact()
                && t.required_features
                    .iter()
                    .all(|f| self.enabled_features.contains(f))
        })
    }
}

/// A dependency edge.
#[derive(Debug, Clone)]
pub struct Edge {
    /// Index of the dependency in [`Graph::packages`].
    pub to: usize,
    /// The edge's kinds (one dependency can be normal and dev at once).
    pub kinds: BTreeSet<DepKind>,
}

/// One resolution of the workspace.
#[derive(Debug, Clone)]
pub struct Graph {
    /// The target triple it was resolved for.
    pub triple: String,
    /// The feature set it was resolved with.
    pub features: Features,
    /// Every package in the resolution.
    pub packages: Vec<Package>,
    /// Outgoing edges per package.
    pub edges: Vec<Vec<Edge>>,
    /// The workspace root.
    pub workspace_root: PathBuf,
}

/// Why the graph could not be read.
#[derive(Debug)]
pub enum MetadataError {
    /// `cargo metadata` could not start or failed.
    Cargo(String),
    /// Its output was not the documented shape.
    Format(String),
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cargo(m) => write!(f, "cargo metadata failed: {m}"),
            Self::Format(m) => write!(f, "unexpected cargo metadata output: {m}"),
        }
    }
}

impl std::error::Error for MetadataError {}

/// How to invoke `cargo metadata`.
#[derive(Debug, Clone)]
pub struct CargoInvocation {
    /// The `cargo` executable.
    pub cargo: PathBuf,
    /// Pass `--locked` (CI and the hooks; the lockfile must be current).
    pub locked: bool,
    /// Pass `--offline`.
    pub offline: bool,
}

impl CargoInvocation {
    /// `$CARGO` when run under Cargo, else `cargo` on `PATH`.
    pub fn from_env() -> Self {
        Self {
            cargo: std::env::var_os("CARGO").map_or_else(|| PathBuf::from("cargo"), PathBuf::from),
            locked: false,
            offline: false,
        }
    }
}

/// Runs `cargo metadata` for the workspace at `root` and reads the graph.
///
/// # Errors
/// [`MetadataError`] when Cargo fails or prints something unexpected.
pub fn load(
    inv: &CargoInvocation,
    root: &Path,
    triple: &str,
    features: Features,
) -> Result<Graph, MetadataError> {
    let mut cmd = Command::new(&inv.cargo);
    cmd.arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--filter-platform")
        .arg(triple)
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .current_dir(root);
    if features == Features::All {
        cmd.arg("--all-features");
    }
    if inv.locked {
        cmd.arg("--locked");
    }
    if inv.offline {
        cmd.arg("--offline");
    }
    let out = cmd
        .output()
        .map_err(|e| MetadataError::Cargo(format!("{}: {e}", inv.cargo.display())))?;
    if !out.status.success() {
        return Err(MetadataError::Cargo(
            String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        ));
    }
    let text = String::from_utf8(out.stdout)
        .map_err(|_utf8| MetadataError::Format("output is not UTF-8".into()))?;
    from_json(&text, triple, features)
}

fn field<'v>(v: &'v Value, key: &str) -> Result<&'v Value, MetadataError> {
    v.get(key)
        .ok_or_else(|| MetadataError::Format(format!("missing field {key:?}")))
}

fn str_field<'v>(v: &'v Value, key: &str) -> Result<&'v str, MetadataError> {
    field(v, key)?
        .as_str()
        .ok_or_else(|| MetadataError::Format(format!("{key:?} is not a string")))
}

fn arr_field<'v>(v: &'v Value, key: &str) -> Result<&'v [Value], MetadataError> {
    field(v, key)?
        .as_array()
        .ok_or_else(|| MetadataError::Format(format!("{key:?} is not an array")))
}

fn strings(v: &[Value]) -> Vec<String> {
    v.iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

/// Reads a `cargo metadata --format-version 1` document.
///
/// # Errors
/// [`MetadataError::Format`] when a documented field is missing.
pub fn from_json(text: &str, triple: &str, features: Features) -> Result<Graph, MetadataError> {
    let doc = json::parse(text).map_err(|e| MetadataError::Format(e.to_string()))?;
    if doc.get("version").and_then(Value::as_int) != Some(1) {
        return Err(MetadataError::Format("not format version 1".into()));
    }
    let members: BTreeSet<&str> = arr_field(&doc, "workspace_members")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let resolve = field(&doc, "resolve")?;
    if matches!(resolve, Value::Null) {
        return Err(MetadataError::Format(
            "no resolve graph (was --no-deps passed?)".into(),
        ));
    }
    let mut packages = Vec::new();
    let mut index = BTreeMap::new();
    for p in arr_field(&doc, "packages")? {
        let id = str_field(p, "id")?.to_owned();
        let mut targets = Vec::new();
        for t in arr_field(p, "targets")? {
            targets.push(Target {
                name: str_field(t, "name")?.to_owned(),
                kinds: strings(arr_field(t, "kind")?),
                crate_types: strings(arr_field(t, "crate_types")?),
                src_path: PathBuf::from(str_field(t, "src_path")?),
                required_features: t
                    .get("required-features")
                    .and_then(Value::as_array)
                    .map(strings)
                    .unwrap_or_default(),
            });
        }
        let proc_macro = targets
            .iter()
            .any(|t| t.kinds.iter().any(|k| k == "proc-macro"));
        index.insert(id.clone(), packages.len());
        packages.push(Package {
            member: members.contains(id.as_str()),
            name: str_field(p, "name")?.to_owned(),
            version: str_field(p, "version")?.to_owned(),
            manifest_path: PathBuf::from(str_field(p, "manifest_path")?),
            publish_false: p
                .get("publish")
                .and_then(Value::as_array)
                .is_some_and(<[Value]>::is_empty),
            targets,
            proc_macro,
            enabled_features: BTreeSet::new(),
            id,
        });
    }
    let mut edges = vec![Vec::new(); packages.len()];
    for node in arr_field(resolve, "nodes")? {
        let from = *index
            .get(str_field(node, "id")?)
            .ok_or_else(|| MetadataError::Format("resolve node for an unknown package".into()))?;
        packages[from].enabled_features =
            strings(arr_field(node, "features")?).into_iter().collect();
        for d in arr_field(node, "deps")? {
            let to = *index
                .get(str_field(d, "pkg")?)
                .ok_or_else(|| MetadataError::Format("dependency on an unknown package".into()))?;
            let mut kinds = BTreeSet::new();
            for k in arr_field(d, "dep_kinds")? {
                kinds.insert(match k.get("kind") {
                    None | Some(Value::Null) => DepKind::Normal,
                    Some(Value::Str(s)) if s == "build" => DepKind::Build,
                    Some(Value::Str(s)) if s == "dev" => DepKind::Dev,
                    Some(other) => {
                        return Err(MetadataError::Format(format!(
                            "unknown dependency kind {other:?}"
                        )));
                    }
                });
            }
            edges[from].push(Edge { to, kinds });
        }
    }
    Ok(Graph {
        triple: triple.to_owned(),
        features,
        packages,
        edges,
        workspace_root: PathBuf::from(str_field(&doc, "workspace_root")?),
    })
}

impl Graph {
    /// Indexes of the packages named `name` (several versions may exist).
    pub fn named(&self, name: &str) -> Vec<usize> {
        (0..self.packages.len())
            .filter(|&i| self.packages[i].name == name)
            .collect()
    }

    /// Workspace members, in name order.
    pub fn members(&self) -> Vec<usize> {
        let mut m: Vec<usize> = (0..self.packages.len())
            .filter(|&i| self.packages[i].member)
            .collect();
        m.sort_by(|&a, &b| self.packages[a].name.cmp(&self.packages[b].name));
        m
    }

    /// Direct dependencies of `from` through edges of the given kinds.
    pub fn direct(&self, from: usize, kinds: &[DepKind]) -> Vec<usize> {
        self.edges[from]
            .iter()
            .filter(|e| kinds.iter().any(|k| e.kinds.contains(k)))
            .map(|e| e.to)
            .collect()
    }

    /// What `from`'s artifacts link: the packages reachable through
    /// normal edges, not counting `from` itself. A procedural macro runs
    /// in the compiler, so it and its own dependencies are not linked.
    /// `stop` packages are included but not looked through.
    pub fn linked(&self, from: usize, stop: &dyn Fn(usize) -> bool) -> BTreeMap<usize, usize> {
        self.reach(from, &[DepKind::Normal], true, stop)
    }

    /// Breadth-first reach from `from` over edges of `kinds`; the value
    /// is the predecessor on one shortest path, for [`Self::path_to`].
    /// With `skip_proc_macros`, procedural macros are neither entered nor
    /// reported.
    pub fn reach(
        &self,
        from: usize,
        kinds: &[DepKind],
        skip_proc_macros: bool,
        stop: &dyn Fn(usize) -> bool,
    ) -> BTreeMap<usize, usize> {
        let mut seen = BTreeMap::new();
        let mut queue = VecDeque::from([from]);
        while let Some(n) = queue.pop_front() {
            if n != from && stop(n) {
                continue;
            }
            for e in &self.edges[n] {
                if !kinds.iter().any(|k| e.kinds.contains(k)) {
                    continue;
                }
                if skip_proc_macros && self.packages[e.to].proc_macro {
                    continue;
                }
                if e.to != from && !seen.contains_key(&e.to) {
                    seen.insert(e.to, n);
                    queue.push_back(e.to);
                }
            }
        }
        seen
    }

    /// The path `from -> ... -> to` recorded by [`Self::reach`], as names.
    pub fn path_to(&self, from: usize, to: usize, preds: &BTreeMap<usize, usize>) -> String {
        let mut chain = vec![to];
        let mut cur = to;
        while let Some(&p) = preds.get(&cur) {
            if p == from || chain.len() > self.packages.len() {
                break;
            }
            chain.push(p);
            cur = p;
        }
        chain.push(from);
        chain
            .iter()
            .rev()
            .map(|&i| self.packages[i].name.as_str())
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r#"{
      "version": 1, "workspace_root": "/w",
      "workspace_members": ["a", "b"],
      "packages": [
        {"id": "a", "name": "hg-a", "version": "0.0.0", "manifest_path": "/w/crates/hg-a/Cargo.toml",
         "publish": [], "targets": [{"name": "hg-a", "kind": ["bin"], "crate_types": ["bin"], "src_path": "/w/crates/hg-a/src/main.rs", "required-features": ["x"]}]},
        {"id": "b", "name": "hg-b", "version": "0.0.0", "manifest_path": "/w/crates/hg-b/Cargo.toml",
         "publish": null, "targets": [{"name": "hg_b", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/w/crates/hg-b/src/lib.rs"}]},
        {"id": "m", "name": "derive", "version": "1.0.0", "manifest_path": "/r/derive/Cargo.toml",
         "publish": null, "targets": [{"name": "derive", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": "/r/derive/src/lib.rs"}]}
      ],
      "resolve": {"root": null, "nodes": [
        {"id": "a", "dependencies": ["b"], "features": [], "deps": [
          {"name": "hg_b", "pkg": "b", "dep_kinds": [{"kind": null, "target": null}, {"kind": "dev", "target": null}]}]},
        {"id": "b", "dependencies": ["m"], "features": [], "deps": [
          {"name": "derive", "pkg": "m", "dep_kinds": [{"kind": null, "target": "cfg(unix)"}]}]},
        {"id": "m", "dependencies": [], "features": [], "deps": []}
      ]}
    }"#;

    #[test]
    fn reads_packages_edges_and_kinds() {
        let g = from_json(DOC, "x86_64-unknown-linux-gnu", Features::All).unwrap();
        let a = g.named("hg-a")[0];
        let b = g.named("hg-b")[0];
        assert!(g.packages[a].publish_false && !g.packages[b].publish_false);
        assert!(g.edges[a][0].kinds.contains(&DepKind::Dev));
        // The bin needs feature x, which this resolution did not enable.
        assert!(!g.packages[a].builds_artifact());
        let linked = g.linked(a, &|_| false);
        assert!(linked.contains_key(&b));
        assert_eq!(linked.len(), 1, "the proc macro is not linked");
        assert_eq!(g.members(), vec![a, b]);
    }
}
