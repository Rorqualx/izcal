//! `hg-graph-lint`: the crate-graph rules (Stage 1 design 2.2, G1–G8 and
//! the specs' registered ranges) and the coding standard's graph checks
//! (A.6), run on `cargo metadata` in every commit and pull request.
//! Tier T, layer `tools` (coding standard 6.2; M1 phase F).
//!
//! # What it reads
//!
//! - `cargo metadata --format-version 1 --filter-platform <triple>` for
//!   each reference OS (Linux x86-64, macOS arm64, Windows x86-64),
//!   because dependencies are `cfg`-gated per OS, and each twice: with
//!   default features and with `--all-features`. The all-features graph
//!   is a superset, so every rule reads it except G4, which the spec
//!   words for "a default-feature `hg-*-main` binary".
//! - `tools/tiers.toml` (each crate's tier and layer; normative, coding
//!   standard 6.2), `tools/graph-rules.toml` (layer order and the
//!   allowlist rules as data, so a rule whose crates do not exist yet
//!   binds the moment they do), `unsafe-inventory.toml` (G5), and the
//!   root `Cargo.toml`'s profiles and lint tables.
//! - With code checks on, the crate sources through `ast-grep` (the
//!   project's structural search tool, pinned in CI), for the few rules
//!   that are about code rather than the graph: crate-root
//!   `#![forbid(unsafe_code)]`, the inventory's `#[expect(unsafe_code)]`
//!   reasons, `#![no_std]` for G232, and the Q-CS-1 crypto check below.
//!
//! # What counts as linking
//!
//! A binary links what its normal dependencies reach, transitively.
//! Dev-dependencies build only tests, benches and examples, so they never
//! count as linkage (the CS-AS-1 ambient-crate rule still follows them,
//! because G3 binds test targets too). Build-dependencies run on the host
//! at build time and are not linked either; the layer and direct-only
//! rules still count them, since the crate's build names them. A
//! procedural macro runs inside the compiler: it and its dependencies
//! are not linked, and are checked by the proc-macro rule instead.
//!
//! # The rules
//!
//! | ID | Check | Source |
//! |---|---|---|
//! | `layers` | A crate depends (normal or build) only on crates of its own or lower layers, with reviewed exceptions in `graph-rules.toml` | S1 2.1; CS-ST-2 |
//! | `tiers` | Every member has a row in `tiers.toml`; declared tier ≥ the strictest tier of any binary that links it | CS 6.2, CS-ST-10 |
//! | `tier-T` | No shipping binary (`hg-*-main`) links a tier T crate | CS 6.2, A.6 |
//! | `Q-CS-1` | Only the listed packages' binaries link `hg-substrate-sim`; outside test code, only `hg-substrate-os` and `hg-substrate-sim` implement the cryptographic-randomness trait | S1 8.1 M1; CS Q-CS-1 |
//! | `CS-AS-1` | `rand`, `getrandom`, `num_cpus`, `tokio`, `smol` reachable from a non-tier-T crate only through `hg-substrate-os` | CS-AS-1, A.6 |
//! | `CS-ER-10` | `anyhow` linked only by tier T | CS-ER-10, A.6 |
//! | `CS-ST-2` | Procedural macros only from the allowlist, and none (nor `syn`) in L1–L3 | CS-ST-2 |
//! | `CS-ER-2` | `release` and every custom profile resolve to `panic = "abort"` | CS-ER-2, A.1 |
//! | `G5` | Every target of a crate not in `unsafe-inventory.toml` forbids `unsafe_code` (manifest level or crate root); inventoried entries name files that exist and carry the matching `expect` reason; no `allow(unsafe_code)` | S1 G5; CS-TY-13 |
//! | `CS-ST-1`, `CS-ST-14`, `CS-TE-2` | Flat `crates/<name>` layout, `hg-` prefix, version `0.0.0`, `publish = false`, at most one integration-test target, under `tests/` | A.6 |
//! | `[[forbid]]`, `[[only-linked-by]]`, `[[direct-only]]`, `[[disjoint]]` | The data rules of `graph-rules.toml`: G1, G2, G4, G6, G7, G8, G230–G232, G241, G252, G255, G260 | the owning specs |
//!
//! # Q-CS-1 and seeded randomness
//!
//! Q-CS-1's concern is a *seeded cryptographic source* in a binary that
//! touches the network. The seeded source is `SimCrypto`, private to
//! `hg-substrate-sim` (`pub(crate)`), reachable only through that crate's
//! `Entropy::crypto()`. Two checks together keep it there:
//!
//! 1. **Package level**: only the binaries `graph-rules.toml` lists
//!    (`hg-replay-main`, and the `hg-dst-sim` package, which holds the
//!    simulation binaries phase E split out of `hg-dst`) may link
//!    `hg-substrate-sim`, directly or not, in any feature combination, on
//!    any of the three OSes.
//! 2. **Code level**: a crypto source exists only where a type
//!    implements the cryptographic-randomness trait (`CryptoRandom` in
//!    the code; Stage 1 3.1 calls it `CryptoRng`, so both names are
//!    checked). Outside test code, only `hg-substrate-os` (the OS CSPRNG,
//!    in the inventoried `os::random`) and `hg-substrate-sim` may
//!    implement it. An alias of the trait (`use ... as`) or a macro that
//!    names it counts too, so a rename or a generated impl cannot hide
//!    one. Test code (`#[cfg(test)]` items and modules, and files under
//!    `tests/`, `benches/` and `examples/`) is exempt: it never links
//!    into a shipped binary, and test fakes of the trait are expected.
//!
//! `hg_substrate::prng::Xoshiro256StarStar::from_seed` is public in a
//! crate every binary links, and stays out of scope on purpose: it is a
//! non-cryptographic stream generator (the `RandomStream` trait, "Not
//! cryptographic"), which production seeds from the OS source. Making a
//! seeded stream cryptographic would need an implementation of the
//! crypto trait, which check 2 catches.
//!
//! # Not checked here
//!
//! G3's ambient-API rule on code is clippy's (`clippy.toml`); G9 is
//! `hg-anchor`'s. A.6 items that wait for their inputs: the CS-AS-13
//! nondeterminism inventory (no `nondeterminism-inventory.toml` yet),
//! the `parser` tag's fuzz targets, unsafe block counts (CS-T10), the
//! generated lint tables (CS-ST-10, no `cargo xtask lints` yet) and
//! CS-ST-17's expectation list. Module-level confinements (G248's
//! `hg-substrate-os::policy`, G251's broker-only launch API, G230's
//! per-OS modules) are below crate granularity.

pub mod code;
pub mod config;
pub mod graph_rules;
pub mod json;
pub mod metadata;
pub mod toml;
pub mod value;
pub mod workspace_rules;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use config::{ConfigError, Inventory, Rules, Tiers};
use metadata::{CargoInvocation, Features, Graph};

/// The reference OSes' target triples (Stage 1 design; CI's three).
pub const TRIPLES: [&str; 3] = [
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
];

/// One rule violation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// The rule's ID (`G260`, `Q-CS-1`, `layers`, ...).
    pub rule: String,
    /// The crate (or file) at fault.
    pub subject: String,
    /// What is wrong, with the dependency path where there is one.
    pub message: String,
    /// The resolutions it was seen in (`<triple>/<features>`); empty for
    /// rules that do not depend on the resolution.
    pub contexts: BTreeSet<String>,
}

/// The lint's result.
#[derive(Debug, Clone, Default)]
pub struct Report {
    findings: BTreeMap<(String, String, String), BTreeSet<String>>,
    /// Things the run could not check, which a reader must know about.
    pub notes: Vec<String>,
    /// How many target triples were read.
    pub triples: usize,
}

impl Report {
    /// Records a violation. The same violation seen in several
    /// resolutions is one finding listing them.
    pub fn add(
        &mut self,
        rule: &str,
        subject: &str,
        message: impl Into<String>,
        context: Option<&str>,
    ) {
        let e = self
            .findings
            .entry((rule.to_owned(), subject.to_owned(), message.into()))
            .or_default();
        if let Some(c) = context {
            e.insert(c.to_owned());
        }
    }

    /// Whether nothing was found.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    /// Every finding, sorted by rule, subject and message.
    pub fn findings(&self) -> Vec<Finding> {
        self.findings
            .iter()
            .map(|((rule, subject, message), contexts)| Finding {
                rule: rule.clone(),
                subject: subject.clone(),
                message: message.clone(),
                contexts: contexts.clone(),
            })
            .collect()
    }

    /// Whether some finding is for `rule`.
    pub fn has(&self, rule: &str) -> bool {
        self.findings.keys().any(|(r, _, _)| r == rule)
    }

    /// The report as text, one finding per line.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for f in self.findings() {
            let _ = write!(out, "error[{}] {}: {}", f.rule, f.subject, f.message);
            if !f.contexts.is_empty() {
                let mut triples = BTreeSet::new();
                let mut features = BTreeSet::new();
                for c in &f.contexts {
                    let (t, feat) = c.split_once('/').unwrap_or((c, ""));
                    triples.insert(t);
                    features.insert(feat);
                }
                let os = if self.triples > 1 && triples.len() == self.triples {
                    "every OS".to_owned()
                } else {
                    triples.into_iter().collect::<Vec<_>>().join(", ")
                };
                let feats = features.into_iter().collect::<Vec<_>>().join(" and ");
                let _ = write!(out, " ({os}; {feats} features)");
            }
            out.push('\n');
        }
        for n in &self.notes {
            let _ = writeln!(out, "note: {n}");
        }
        out
    }
}

/// How to run the lint.
#[derive(Debug, Clone)]
pub struct Options {
    /// The workspace root (holds `Cargo.toml`, `tools/` and
    /// `unsafe-inventory.toml`).
    pub root: PathBuf,
    /// Target triples to resolve for.
    pub triples: Vec<String>,
    /// How to run `cargo metadata`.
    pub cargo: CargoInvocation,
    /// The `ast-grep` executable for the code checks, or `None` to skip
    /// them (reported as a note).
    pub ast_grep: Option<PathBuf>,
}

impl Options {
    /// The defaults for `root`: all three triples, `cargo` from the
    /// environment, `ast-grep` from `PATH`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            triples: TRIPLES.iter().map(|t| (*t).to_owned()).collect(),
            cargo: CargoInvocation::from_env(),
            ast_grep: Some(PathBuf::from("ast-grep")),
        }
    }
}

/// Why the lint could not run (as opposed to finding violations).
#[derive(Debug)]
pub enum LintError {
    /// A gate file is missing or malformed.
    Config(ConfigError),
    /// `cargo metadata` failed.
    Metadata(metadata::MetadataError),
    /// `ast-grep` failed.
    Code(String),
}

impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(e) => write!(f, "{e}"),
            Self::Metadata(e) => write!(f, "{e}"),
            Self::Code(e) => write!(f, "code checks: {e}"),
        }
    }
}

impl std::error::Error for LintError {}

impl From<ConfigError> for LintError {
    fn from(e: ConfigError) -> Self {
        Self::Config(e)
    }
}

impl From<metadata::MetadataError> for LintError {
    fn from(e: metadata::MetadataError) -> Self {
        Self::Metadata(e)
    }
}

/// Everything the rules read.
#[derive(Debug)]
pub struct Inputs {
    /// The workspace root.
    pub root: PathBuf,
    /// `tools/tiers.toml`.
    pub tiers: Tiers,
    /// `tools/graph-rules.toml`.
    pub rules: Rules,
    /// `unsafe-inventory.toml`.
    pub inventory: Inventory,
    /// One graph per (triple, feature set).
    pub graphs: Vec<Graph>,
}

/// Reads the gate files under `root`.
///
/// # Errors
/// [`ConfigError`] if one is missing or malformed.
pub fn read_config(root: &Path) -> Result<(Tiers, Rules, Inventory), ConfigError> {
    let tiers = Tiers::from_value(&config::read_toml(&root.join("tools/tiers.toml"))?)?;
    let rules = Rules::from_value(&config::read_toml(&root.join("tools/graph-rules.toml"))?)?;
    let inventory =
        Inventory::from_value(&config::read_toml(&root.join("unsafe-inventory.toml"))?)?;
    Ok((tiers, rules, inventory))
}

/// Runs every check on the workspace at `opts.root`.
///
/// # Errors
/// [`LintError`] when an input cannot be read; violations are in the
/// [`Report`].
pub fn run(opts: &Options) -> Result<Report, LintError> {
    let (tiers, rules, inventory) = read_config(&opts.root)?;
    let mut graphs = Vec::new();
    for triple in &opts.triples {
        for features in [Features::Default, Features::All] {
            graphs.push(metadata::load(&opts.cargo, &opts.root, triple, features)?);
        }
    }
    // Cargo's own spelling of the root (absolute, symlinks resolved), so
    // it compares equal to the manifest paths it reports.
    let root = graphs
        .first()
        .map_or_else(|| opts.root.clone(), |g| g.workspace_root.clone());
    let inputs = Inputs {
        root,
        tiers,
        rules,
        inventory,
        graphs,
    };
    let mut report = Report {
        triples: opts.triples.len(),
        ..Report::default()
    };
    for g in &inputs.graphs {
        graph_rules::check(&inputs, g, &mut report);
    }
    let scan = match &opts.ast_grep {
        Some(bin) => Some(code::scan(bin, &inputs).map_err(LintError::Code)?),
        None => {
            report.notes.push(
                "code checks skipped (no ast-grep): crate-root forbid(unsafe_code), the \
                 inventory's expect reasons, no_std (G232) and the Q-CS-1 crypto check"
                    .into(),
            );
            None
        }
    };
    workspace_rules::check(&inputs, scan.as_ref(), &mut report)?;
    if let Some(s) = &scan {
        code::check(&inputs, s, &mut report);
    }
    Ok(report)
}
