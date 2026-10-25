//! Fixture workspaces: a small copy of the real workspace's shape, written
//! to a temporary directory, linted with real `cargo metadata` runs. Each
//! test starts from [`Fixture::baseline`], which passes, and breaks one
//! rule.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use hg_graph_lint::{Options, Report};

/// The repository's own `tools/graph-rules.toml`, so the tests exercise
/// the shipped rule data, not a copy of it.
fn real_rules() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/graph-rules.toml");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// `ast-grep` for the code checks, or `None` (and a note on stderr) when
/// it is not installed. With `IZCAL_REQUIRE_AST_GREP=1` (CI's checks
/// job, which installs the pinned ast-grep) a missing tool fails the test
/// instead, so the code checks cannot be skipped where they must run.
pub fn ast_grep() -> Option<PathBuf> {
    let found = std::process::Command::new("ast-grep")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if found {
        return Some(PathBuf::from("ast-grep"));
    }
    assert!(
        std::env::var_os("IZCAL_REQUIRE_AST_GREP").is_none_or(|v| v != "1"),
        "IZCAL_REQUIRE_AST_GREP=1 but ast-grep is not on PATH"
    );
    eprintln!("note: ast-grep not on PATH; this test's code checks are skipped");
    None
}

/// A workspace as files, plus external crates kept outside it.
#[derive(Debug, Clone)]
pub struct Fixture {
    /// Workspace files by relative path.
    pub files: BTreeMap<String, String>,
    /// External (non-member) path crates: name to `src/lib.rs`, and
    /// whether it is a procedural macro.
    pub external: BTreeMap<String, bool>,
    /// Lines of tools/tiers.toml rows: crate to (tier, layer).
    pub tiers: BTreeMap<String, (String, String)>,
}

fn manifest(name: &str, deps: &str, extra: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion.workspace = true\nedition.workspace = true\npublish.workspace = true\n\n[dependencies]\n{deps}\n{extra}\n[lints.rust]\nunsafe_code = \"forbid\"\n"
    )
}

impl Fixture {
    /// A passing workspace with the real one's shape: types, the three
    /// substrate crates, a shipping main, the DST runner.
    pub fn baseline() -> Self {
        let mut f = Self {
            files: BTreeMap::new(),
            external: BTreeMap::new(),
            tiers: BTreeMap::new(),
        };
        f.files.insert(
            "Cargo.toml".into(),
            "[workspace]\nresolver = \"3\"\nmembers = [\"crates/*\"]\n\n[workspace.package]\nedition = \"2024\"\nversion = \"0.0.0\"\npublish = false\n\n[workspace.lints.rust]\nunsafe_code = \"deny\"\n\n[profile.release]\npanic = \"abort\"\n\n[profile.checked]\ninherits = \"release\"\ndebug-assertions = true\n".into(),
        );
        f.files
            .insert("tools/graph-rules.toml".into(), real_rules());
        f.files.insert(
            "unsafe-inventory.toml".into(),
            "[crates.hg-substrate-os]\nentries = [\"os::random\"]\n".into(),
        );
        f.lib("hg-types", "P", "L2", "", "//! Types.\n");
        f.lib(
            "hg-substrate",
            "P",
            "L1",
            "hg-types = { path = \"../hg-types\" }\n",
            "//! Traits.\npub trait CryptoRandom {}\n#[cfg(test)]\nmod tests {\n    struct Fake;\n    impl super::CryptoRandom for Fake {}\n}\n",
        );
        f.lib(
            "hg-substrate-os",
            "O",
            "L1",
            "hg-substrate = { path = \"../hg-substrate\" }\nhg-types = { path = \"../hg-types\" }\n",
            "//! OS.\npub mod os;\npub struct OsCrypto;\nimpl hg_substrate::CryptoRandom for OsCrypto {}\n",
        );
        f.set_lints("hg-substrate-os", "unsafe_code = \"deny\"");
        f.files.insert(
            "crates/hg-substrate-os/src/os.rs".into(),
            "//! OS calls.\npub mod random;\n".into(),
        );
        f.files.insert(
            "crates/hg-substrate-os/src/os/random.rs".into(),
            "//! The CSPRNG.\n#![expect(unsafe_code, reason = \"G5: os::random (the OS call)\")]\n/// Reads nothing.\npub fn fill() {\n    // SAFETY: an empty block.\n    unsafe {}\n}\n".into(),
        );
        f.lib(
            "hg-substrate-sim",
            "R",
            "L1",
            "hg-substrate = { path = \"../hg-substrate\" }\nhg-types = { path = \"../hg-types\" }\n",
            "//! Simulator.\nstruct SimCrypto;\nimpl hg_substrate::CryptoRandom for SimCrypto {}\n",
        );
        f.bin(
            "hg-x-main",
            "P",
            "L6",
            "hg-substrate = { path = \"../hg-substrate\" }\nhg-substrate-os = { path = \"../hg-substrate-os\" }\n",
        );
        f.bin(
            "hg-dst",
            "T",
            "tools",
            "hg-substrate-os = { path = \"../hg-substrate-os\" }\nhg-substrate-sim = { path = \"../hg-substrate-sim\" }\n",
        );
        f
    }

    /// Adds a library crate.
    pub fn lib(&mut self, name: &str, tier: &str, layer: &str, deps: &str, src: &str) {
        self.files.insert(
            format!("crates/{name}/Cargo.toml"),
            manifest(name, deps, ""),
        );
        self.files
            .insert(format!("crates/{name}/src/lib.rs"), src.to_owned());
        self.tiers
            .insert(name.to_owned(), (tier.to_owned(), layer.to_owned()));
    }

    /// Adds a binary crate.
    pub fn bin(&mut self, name: &str, tier: &str, layer: &str, deps: &str) {
        self.files.insert(
            format!("crates/{name}/Cargo.toml"),
            manifest(name, deps, ""),
        );
        self.files.insert(
            format!("crates/{name}/src/main.rs"),
            "//! A binary.\nfn main() {}\n".into(),
        );
        self.tiers
            .insert(name.to_owned(), (tier.to_owned(), layer.to_owned()));
    }

    /// Adds an external crate (outside the workspace).
    pub fn ext(&mut self, name: &str, proc_macro: bool) {
        self.external.insert(name.to_owned(), proc_macro);
    }

    /// Replaces the `[lints.rust]` body of a member.
    pub fn set_lints(&mut self, name: &str, body: &str) {
        let m = self.manifest_mut(name);
        let at = m.find("[lints.rust]").unwrap();
        m.truncate(at);
        m.push_str("[lints.rust]\n");
        m.push_str(body);
        m.push('\n');
    }

    /// The manifest of a member.
    pub fn manifest_mut(&mut self, name: &str) -> &mut String {
        self.files
            .get_mut(&format!("crates/{name}/Cargo.toml"))
            .unwrap_or_else(|| panic!("no crate {name}"))
    }

    /// Inserts `text` into a member's manifest just before `[lints.rust]`.
    pub fn add_manifest(&mut self, name: &str, text: &str) {
        let m = self.manifest_mut(name);
        let at = m.find("[lints.rust]").unwrap();
        m.insert_str(at, &format!("{text}\n"));
    }

    /// Adds a line to a member's `[dependencies]`.
    pub fn dep(&mut self, name: &str, line: &str) {
        let m = self.manifest_mut(name);
        let at = m.find("[dependencies]\n").unwrap() + "[dependencies]\n".len();
        m.insert_str(at, &format!("{line}\n"));
    }

    /// Replaces text in a file, which must contain it.
    pub fn edit(&mut self, path: &str, from: &str, to: &str) {
        let f = self
            .files
            .get_mut(path)
            .unwrap_or_else(|| panic!("no file {path}"));
        assert!(f.contains(from), "{path} has no {from:?}");
        *f = f.replacen(from, to, 1);
    }

    /// Writes the fixture and lints it for `triples` (Linux if empty).
    pub fn lint(&self, triples: &[&str], code: bool) -> Report {
        let dir = TempDir::new();
        let ws = dir.0.join("ws");
        let ext = dir.0.join("ext");
        for (name, proc_macro) in &self.external {
            let lib = if *proc_macro {
                "\n[lib]\nproc-macro = true\n"
            } else {
                ""
            };
            write(
                &ext.join(name).join("Cargo.toml"),
                &format!(
                    "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nedition = \"2024\"\n{lib}"
                ),
            );
            write(&ext.join(name).join("src/lib.rs"), "//! External.\n");
        }
        for (rel, text) in &self.files {
            // Path dependencies on external crates: `ext:NAME` in a
            // dependency line becomes the absolute path (a TOML literal
            // string, so Windows separators need no escaping).
            let mut text = text.clone();
            for name in self.external.keys() {
                let p = ext.join(name);
                text = text.replace(&format!("\"ext:{name}\""), &format!("'{}'", p.display()));
            }
            write(&ws.join(rel), &text);
        }
        let mut tiers = String::new();
        for (name, (tier, layer)) in &self.tiers {
            tiers.push_str(&format!(
                "[crates.{name}]\ntier = \"{tier}\"\nlayer = \"{layer}\"\n\n"
            ));
        }
        write(&ws.join("tools/tiers.toml"), &tiers);
        let mut opts = Options::new(&ws);
        opts.cargo.offline = true;
        opts.triples = if triples.is_empty() {
            vec!["x86_64-unknown-linux-gnu".to_owned()]
        } else {
            triples.iter().map(|t| (*t).to_owned()).collect()
        };
        opts.ast_grep = if code { ast_grep() } else { None };
        hg_graph_lint::run(&opts).unwrap_or_else(|e| panic!("lint failed to run: {e}"))
    }
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// A temporary directory, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("hg-graph-lint-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
