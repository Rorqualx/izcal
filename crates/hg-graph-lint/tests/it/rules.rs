//! Each rule passes on the baseline and fails when broken.

use std::path::Path;

use hg_graph_lint::{Options, Report, TRIPLES};

use crate::fixture::{Fixture, ast_grep};

const LINUX: &str = "x86_64-unknown-linux-gnu";
const WINDOWS: &str = "x86_64-pc-windows-msvc";

/// Asserts the report has a finding for `rule` whose subject is
/// `subject` and whose message contains `needle`.
fn assert_finding(r: &Report, rule: &str, subject: &str, needle: &str) {
    assert!(
        r.findings()
            .iter()
            .any(|f| f.rule == rule && f.subject == subject && f.message.contains(needle)),
        "expected [{rule}] {subject}: ...{needle}...; got:\n{}",
        r.render()
    );
}

fn assert_clean(r: &Report) {
    assert!(r.is_clean(), "expected clean; got:\n{}", r.render());
}

#[test]
fn the_baseline_passes_on_every_os() {
    let r = Fixture::baseline().lint(&TRIPLES, true);
    assert_clean(&r);
}

#[test]
fn the_real_workspace_passes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut opts = Options::new(root);
    // Not offline: on a fresh machine the other OSes' dependencies (such
    // as windows-sys on Linux) may not be downloaded yet.
    opts.cargo.locked = true;
    opts.ast_grep = ast_grep();
    let r = hg_graph_lint::run(&opts).unwrap();
    assert_clean(&r);
}

// --- Layers -------------------------------------------------------------

#[test]
fn an_l1_crate_depending_on_l4_fails() {
    let mut f = Fixture::baseline();
    f.lib("hg-html", "R", "L4", "", "//! L4.\n");
    f.dep("hg-substrate", "hg-html = { path = \"../hg-html\" }");
    let r = f.lint(&[], false);
    assert_finding(&r, "layers", "hg-substrate", "hg-html (L4), a higher layer");
}

#[test]
fn the_reviewed_layer_exception_is_narrow() {
    // L1 -> hg-types is the one exception; L1 -> another L2 crate fails.
    let mut f = Fixture::baseline();
    f.lib("hg-display-list", "P", "L2", "", "//! L2.\n");
    f.dep(
        "hg-substrate",
        "hg-display-list = { path = \"../hg-display-list\" }",
    );
    let r = f.lint(&[], false);
    assert_finding(&r, "layers", "hg-substrate", "hg-display-list (L2)");
}

#[test]
fn a_dev_dependency_on_a_higher_layer_is_not_a_layer_edge() {
    let mut f = Fixture::baseline();
    f.add_manifest(
        "hg-types",
        "[dev-dependencies]\nhg-substrate-os = { path = \"../hg-substrate-os\" }\n",
    );
    assert_clean(&f.lint(&[], false));
}

// --- Q-CS-1 -------------------------------------------------------------

#[test]
fn a_main_linking_the_simulator_directly_fails() {
    let mut f = Fixture::baseline();
    f.dep(
        "hg-x-main",
        "hg-substrate-sim = { path = \"../hg-substrate-sim\" }",
    );
    let r = f.lint(&[], false);
    assert_finding(&r, "Q-CS-1", "hg-x-main", "hg-x-main -> hg-substrate-sim");
}

#[test]
fn a_main_linking_the_simulator_transitively_fails() {
    let mut f = Fixture::baseline();
    f.lib(
        "hg-helper",
        "P",
        "L5",
        "hg-substrate-sim = { path = \"../hg-substrate-sim\" }\n",
        "//! L5.\n",
    );
    f.dep("hg-x-main", "hg-helper = { path = \"../hg-helper\" }");
    let r = f.lint(&[], false);
    assert_finding(
        &r,
        "Q-CS-1",
        "hg-x-main",
        "hg-x-main -> hg-helper -> hg-substrate-sim",
    );
}

#[test]
fn a_link_made_only_by_a_feature_fails() {
    let mut f = Fixture::baseline();
    f.dep(
        "hg-x-main",
        "hg-substrate-sim = { path = \"../hg-substrate-sim\", optional = true }",
    );
    f.add_manifest(
        "hg-x-main",
        "[features]\nreplay = [\"dep:hg-substrate-sim\"]\n",
    );
    let r = f.lint(&[], false);
    let q: Vec<_> = r
        .findings()
        .into_iter()
        .filter(|f| f.rule == "Q-CS-1")
        .collect();
    assert_eq!(q.len(), 1, "{}", r.render());
    // Seen only in the all-features resolution: default features do not
    // link it, and the lint does not need them to.
    assert_eq!(
        q[0].contexts.iter().collect::<Vec<_>>(),
        vec![&format!("{LINUX}/all")]
    );
}

#[test]
fn a_link_made_only_on_one_os_fails_on_that_os() {
    let mut f = Fixture::baseline();
    f.add_manifest(
        "hg-x-main",
        "[target.'cfg(windows)'.dependencies]\nhg-substrate-sim = { path = \"../hg-substrate-sim\" }\n",
    );
    let r = f.lint(&TRIPLES, false);
    let q: Vec<_> = r
        .findings()
        .into_iter()
        .filter(|f| f.rule == "Q-CS-1")
        .collect();
    assert_eq!(q.len(), 1, "{}", r.render());
    assert!(
        q[0].contexts.iter().all(|c| c.starts_with(WINDOWS)),
        "{:?}",
        q[0].contexts
    );
}

#[test]
fn a_dev_dependency_on_the_simulator_does_not_count() {
    let mut f = Fixture::baseline();
    f.add_manifest(
        "hg-x-main",
        "[dev-dependencies]\nhg-substrate-sim = { path = \"../hg-substrate-sim\" }\n",
    );
    f.add_manifest(
        "hg-substrate",
        "[dev-dependencies]\nhg-substrate-sim = { path = \"../hg-substrate-sim\" }\n",
    );
    assert_clean(&f.lint(&[], false));
}

#[test]
fn an_unlisted_tool_binary_linking_the_simulator_fails() {
    let mut f = Fixture::baseline();
    f.bin(
        "hg-bench",
        "T",
        "tools",
        "hg-substrate-sim = { path = \"../hg-substrate-sim\" }\n",
    );
    let r = f.lint(&[], false);
    assert_finding(&r, "Q-CS-1", "hg-bench", "not on the Q-CS-1 list");
}

#[test]
fn a_crypto_source_outside_the_substrates_fails() {
    let Some(_) = ast_grep() else { return };
    let mut f = Fixture::baseline();
    f.edit(
        "crates/hg-types/src/lib.rs",
        "//! Types.\n",
        "//! Types.\npub struct Seeded;\nimpl<T> crate::x::CryptoRandom<T> for Seeded {}\n",
    );
    let r = f.lint(&[], true);
    assert_finding(
        &r,
        "Q-CS-1",
        "hg-types",
        "implements the cryptographic-randomness trait",
    );

    // A rename hides nothing.
    let mut f = Fixture::baseline();
    f.edit(
        "crates/hg-types/src/lib.rs",
        "//! Types.\n",
        "//! Types.\nuse hg_substrate::CryptoRandom as Rng;\n",
    );
    assert_finding(&f.lint(&[], true), "Q-CS-1", "hg-types", "renames");

    // Neither does a macro.
    let mut f = Fixture::baseline();
    f.edit(
        "crates/hg-types/src/lib.rs",
        "//! Types.\n",
        "//! Types.\nmacro_rules! m { ($t:ty) => { impl CryptoRng for $t {} } }\n",
    );
    assert_finding(&f.lint(&[], true), "Q-CS-1", "hg-types", "names in a macro");
}

#[test]
fn test_only_crypto_fakes_are_allowed() {
    let Some(_) = ast_grep() else { return };
    let mut f = Fixture::baseline();
    // An inline #[cfg(test)] module (the baseline's hg-substrate has one),
    // a #[cfg(test)] item, a module file declared behind #[cfg(test)],
    // and an integration test.
    f.edit(
        "crates/hg-types/src/lib.rs",
        "//! Types.\n",
        "//! Types.\n#[cfg(test)]\n/// A fake.\nimpl hg_substrate::CryptoRandom for u8 {}\n#[cfg(test)]\nmod fakes;\n",
    );
    f.files.insert(
        "crates/hg-types/src/fakes.rs".into(),
        "struct F;\nimpl hg_substrate::CryptoRandom for F {}\n".into(),
    );
    f.files.insert(
        "crates/hg-types/tests/it/main.rs".into(),
        "struct F;\nimpl hg_substrate::CryptoRandom for F {}\n".into(),
    );
    assert_clean(&f.lint(&[], true));
}

// --- Tiers ----------------------------------------------------------------

#[test]
fn a_tier_t_crate_reached_from_a_main_fails() {
    let mut f = Fixture::baseline();
    f.lib("hg-corpus", "T", "tools", "", "//! A tool.\n");
    f.lib(
        "hg-helper",
        "P",
        "L5",
        "hg-corpus = { path = \"../hg-corpus\" }\n",
        "//! L5.\n",
    );
    f.dep("hg-x-main", "hg-helper = { path = \"../hg-helper\" }");
    let r = f.lint(&[], false);
    assert_finding(
        &r,
        "tier-T",
        "hg-x-main",
        "links tier T crate hg-corpus: hg-x-main -> hg-helper -> hg-corpus",
    );
}

#[test]
fn a_declared_tier_laxer_than_its_linker_fails() {
    let mut f = Fixture::baseline();
    f.lib("hg-helper", "R", "L5", "", "//! L5.\n");
    f.dep("hg-x-main", "hg-helper = { path = \"../hg-helper\" }");
    let r = f.lint(&[], false);
    assert_finding(
        &r,
        "tiers",
        "hg-helper",
        "crate hg-helper is now linked by hg-x-main (tier P); declare tier P in its own PR",
    );
}

#[test]
fn a_crate_without_a_tier_row_fails() {
    let mut f = Fixture::baseline();
    f.lib("hg-new", "P", "L2", "", "//! New.\n");
    f.tiers.remove("hg-new");
    assert_finding(
        &f.lint(&[], false),
        "tiers",
        "hg-new",
        "no row in tools/tiers.toml",
    );
}

// --- G5 -----------------------------------------------------------------

#[test]
fn a_crate_missing_forbid_that_is_not_inventoried_fails() {
    let mut f = Fixture::baseline();
    f.set_lints("hg-types", "unsafe_code = \"deny\"");
    let Some(_) = ast_grep() else { return };
    let r = f.lint(&[], true);
    assert_finding(&r, "G5", "hg-types", "does not forbid unsafe_code");

    // The crate-root attribute satisfies it as well as the manifest.
    f.edit(
        "crates/hg-types/src/lib.rs",
        "//! Types.\n",
        "//! Types.\n#![forbid(unsafe_code, missing_docs)]\n",
    );
    assert_clean(&f.lint(&[], true));
}

#[test]
fn inventory_entries_must_match_the_code() {
    let Some(_) = ast_grep() else { return };
    // An entry naming no file.
    let mut f = Fixture::baseline();
    f.edit(
        "unsafe-inventory.toml",
        "\"os::random\"",
        "\"os::random\", \"os::gone\"",
    );
    assert_finding(
        &f.lint(&[], true),
        "G5",
        "hg-substrate-os",
        "\"os::gone\" names no file",
    );

    // An expectation whose reason names another entry.
    let mut f = Fixture::baseline();
    f.edit(
        "crates/hg-substrate-os/src/os/random.rs",
        "G5: os::random",
        "G5: os::vm",
    );
    let r = f.lint(&[], true);
    assert_finding(&r, "G5", "hg-substrate-os", "must start \"G5: os::random\"");

    // Lowering the level bypasses the inventory, wherever unsafe_code
    // sits in the lint list, and inside cfg_attr.
    for attr in [
        "#[allow(unsafe_code)]\nfn f() {}\n",
        "#[allow(dead_code, unsafe_code)]\nfn f() {}\n",
        "#![warn(clippy::all, unsafe_code)]\n",
        "#![cfg_attr(unix, allow(unsafe_code))]\n",
    ] {
        let mut f = Fixture::baseline();
        f.edit(
            "crates/hg-substrate-os/src/lib.rs",
            "//! OS.\n",
            &format!("//! OS.\n{attr}"),
        );
        let r = f.lint(&[], true);
        assert_finding(&r, "G5", "hg-substrate-os", "allow or warn on unsafe_code");
    }

    // An expectation is found wherever unsafe_code sits in its list.
    let mut f = Fixture::baseline();
    f.edit(
        "crates/hg-substrate-os/src/lib.rs",
        "//! OS.\n",
        "//! OS.\n#[expect(dead_code, unsafe_code, reason = \"G5: os::random\")]\nfn f() {}\n",
    );
    let r = f.lint(&[], true);
    assert_finding(
        &r,
        "G5",
        "hg-substrate-os",
        "a file no unsafe-inventory.toml entry names",
    );
}

// --- Allowlist rules (graph-rules.toml) -----------------------------------

#[test]
fn g260_a_main_linking_a_compliance_tool_fails() {
    let mut f = Fixture::baseline();
    f.lib("hg-sbom", "T", "tools", "", "//! A tool.\n");
    f.lib(
        "hg-about",
        "P",
        "L5",
        "hg-sbom = { path = \"../hg-sbom\" }\n",
        "//! L5.\n",
    );
    f.dep("hg-x-main", "hg-about = { path = \"../hg-about\" }");
    let r = f.lint(&[], false);
    assert_finding(&r, "G260", "hg-x-main", "hg-x-main -> hg-about -> hg-sbom");
}

#[test]
fn g252_update_parsers_only_in_the_updater() {
    let mut f = Fixture::baseline();
    f.lib("hg-delta", "P", "L2s", "", "//! Deltas.\n");
    f.bin(
        "hg-updater-main",
        "P",
        "L6",
        "hg-delta = { path = \"../hg-delta\" }\n",
    );
    assert_clean(&f.lint(&[], false));
    f.dep("hg-x-main", "hg-delta = { path = \"../hg-delta\" }");
    let r = f.lint(&[], false);
    assert_finding(&r, "G252", "hg-x-main", "links hg-delta");
}

#[test]
fn g241_the_policy_parser_only_through_the_adapter() {
    let mut f = Fixture::baseline();
    f.lib("hg-policy-format", "P", "L2s", "", "//! Parser.\n");
    f.lib(
        "hg-policy",
        "P",
        "L5",
        "hg-policy-format = { path = \"../hg-policy-format\" }\n",
        "//! Policy.\n",
    );
    let r = f.lint(&[], false);
    assert_finding(&r, "G241", "hg-policy", "only hg-policy-adapter may");
}

#[test]
fn g231_g232_the_launcher_and_its_leaf_crates() {
    let mut f = Fixture::baseline();
    f.lib("hg-launch-wire", "P", "L2s", "", "//! IZLH.\n#![no_std]\n");
    f.bin(
        "hg-launch-main",
        "P",
        "L6",
        "hg-substrate-os = { path = \"../hg-substrate-os\" }\nhg-launch-wire = { path = \"../hg-launch-wire\" }\n",
    );
    assert_clean(&f.lint(&[], true));

    // G231: no hg-ipc in the launcher, directly or not.
    let mut g = f.clone();
    g.lib("hg-ipc", "P", "L1", "", "//! IPC.\n");
    g.dep("hg-launch-main", "hg-ipc = { path = \"../hg-ipc\" }");
    let r = g.lint(&[], false);
    assert_finding(&r, "G231/G255", "hg-launch-main", "depends on hg-ipc");
    assert_finding(&r, "G231", "hg-launch-main", "must not depend on hg-ipc");

    // G232: no dependencies, tier P, no_std.
    let mut g = f.clone();
    g.dep("hg-launch-wire", "hg-types = { path = \"../hg-types\" }");
    assert_finding(
        &g.lint(&[], false),
        "G232",
        "hg-launch-wire",
        "depends on hg-types",
    );
    let mut g = f.clone();
    g.tiers
        .insert("hg-launch-wire".into(), ("R".into(), "L2s".into()));
    assert_finding(
        &g.lint(&[], false),
        "G232",
        "hg-launch-wire",
        "declared tier P",
    );
    if ast_grep().is_some() {
        let mut g = f.clone();
        g.edit("crates/hg-launch-wire/src/lib.rs", "#![no_std]\n", "");
        assert_finding(&g.lint(&[], true), "G232", "hg-launch-wire", "#![no_std]");
    }
}

#[test]
fn g4_agent_and_ml_crates_with_default_features() {
    let mut f = Fixture::baseline();
    f.lib("hg-ml-tasks", "P", "L5", "", "//! ML.\n");
    // Behind a non-default feature: G4 is about default features.
    f.dep(
        "hg-x-main",
        "hg-ml-tasks = { path = \"../hg-ml-tasks\", optional = true }",
    );
    f.add_manifest("hg-x-main", "[features]\nai = [\"dep:hg-ml-tasks\"]\n");
    assert_clean(&f.lint(&[], false));
    // By default: G4 fails.
    let mut f = Fixture::baseline();
    f.lib("hg-ml-tasks", "P", "L5", "", "//! ML.\n");
    f.dep("hg-x-main", "hg-ml-tasks = { path = \"../hg-ml-tasks\" }");
    assert_finding(&f.lint(&[], false), "G4", "hg-x-main", "hg-ml-tasks");
    // hg-ml-main may.
    let mut f = Fixture::baseline();
    f.lib("hg-ml-tasks", "P", "L5", "", "//! ML.\n");
    f.bin(
        "hg-ml-main",
        "P",
        "L6",
        "hg-ml-tasks = { path = \"../hg-ml-tasks\" }\n",
    );
    assert_clean(&f.lint(&[], false));
}

// --- Dependencies and metadata ---------------------------------------------

#[test]
fn ambient_crates_only_through_the_os_substrate() {
    let mut f = Fixture::baseline();
    f.ext("rand", false);
    f.dep("hg-substrate-os", "rand = { path = \"ext:rand\" }");
    assert_clean(&f.lint(&[], false));
    // A dev-dependency counts: G3 binds tests.
    f.add_manifest(
        "hg-types",
        "[dev-dependencies]\nrand = { path = \"ext:rand\" }\n",
    );
    assert_finding(&f.lint(&[], false), "CS-AS-1", "hg-types", "reaches rand");
}

#[test]
fn anyhow_only_in_tier_t() {
    let mut f = Fixture::baseline();
    f.ext("anyhow", false);
    f.dep("hg-dst", "anyhow = { path = \"ext:anyhow\" }");
    assert_clean(&f.lint(&[], false));
    f.dep("hg-substrate-sim", "anyhow = { path = \"ext:anyhow\" }");
    assert_finding(
        &f.lint(&[], false),
        "CS-ER-10",
        "hg-substrate-sim",
        "anyhow",
    );
}

#[test]
fn proc_macros_need_the_allowlist() {
    let mut f = Fixture::baseline();
    f.ext("derive-thing", true);
    f.dep("hg-dst", "derive-thing = { path = \"ext:derive-thing\" }");
    let r = f.lint(&[], false);
    assert_finding(&r, "CS-ST-2", "hg-dst", "not on the allowlist");
}

#[test]
fn a_profile_without_panic_abort_fails() {
    let mut f = Fixture::baseline();
    f.edit("Cargo.toml", "[profile.release]\npanic = \"abort\"\n", "");
    let r = f.lint(&[], false);
    assert_finding(&r, "CS-ER-2", "profile.release", "unwind");
    assert_finding(&r, "CS-ER-2", "profile.checked", "unwind");

    let mut f = Fixture::baseline();
    f.edit(
        "Cargo.toml",
        "debug-assertions = true\n",
        "debug-assertions = true\npanic = \"unwind\"\n",
    );
    assert_finding(
        &f.lint(&[], false),
        "CS-ER-2",
        "profile.checked",
        "resolves to unwind",
    );
}

#[test]
fn layout_name_version_and_publish() {
    let mut f = Fixture::baseline();
    f.edit(
        "crates/hg-types/Cargo.toml",
        "version.workspace = true",
        "version = \"0.1.0\"",
    );
    f.edit(
        "crates/hg-types/Cargo.toml",
        "publish.workspace = true\n",
        "",
    );
    let r = f.lint(&[], false);
    assert_finding(&r, "CS-ST-14", "hg-types", "version is 0.1.0");
    assert_finding(&r, "CS-ST-14", "hg-types", "publish = false");
}
