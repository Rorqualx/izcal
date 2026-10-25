//! The `BuildId` definition (distribution 13 item 5): what it depends on,
//! and the constant `hg-types` compiles in.

use std::collections::BTreeSet;
use std::path::Path;

use hg_build_id::{Inputs, collect, schema_hash, unpinned};

fn base() -> Inputs {
    Inputs {
        commit: "0123456789abcdef0123456789abcdef01234567".into(),
        specs_lock: Some(b"wpt 1\n".to_vec()),
        downloads_lock: None,
        toolchain: b"[toolchain]\nchannel = \"1.99.1\"\n".to_vec(),
        profile: "release".into(),
        target: "x86_64-unknown-linux-gnu".into(),
        channel: "stable".into(),
        version: "1.0.0".into(),
        schema: [7; 32],
    }
}

#[test]
fn the_unpinned_constant_is_the_unpinned_inputs_digest() {
    assert_eq!(hg_types::UNPINNED_BUILD_ID, unpinned().build_id());
}

#[test]
fn this_test_binary_reports_its_build() {
    // Built without the variable, it carries the unpinned value; built
    // with it, that value.
    match std::env::var("IZCAL_BUILD_ID") {
        Ok(hex) if hg_types::BUILD_IS_PINNED => {
            assert_eq!(format!("{:016x}", hg_types::BUILD_ID), hex.to_lowercase());
        }
        _ => assert_eq!(hg_types::BUILD_ID, hg_types::UNPINNED_BUILD_ID),
    }
}

#[test]
fn every_input_changes_the_id() {
    let b = base();
    let id = b.build_id();
    let mut variants = vec![];
    let mut v = b.clone();
    v.commit.push('x');
    variants.push(v);
    let mut v = b.clone();
    v.specs_lock = None;
    variants.push(v);
    let mut v = b.clone();
    v.downloads_lock = Some(Vec::new());
    variants.push(v);
    let mut v = b.clone();
    v.toolchain.push(b'\n');
    variants.push(v);
    let mut v = b.clone();
    v.profile = "dev".into();
    variants.push(v);
    let mut v = b.clone();
    v.target = "aarch64-apple-darwin".into();
    variants.push(v);
    let mut v = b.clone();
    v.channel = "beta".into();
    variants.push(v);
    let mut v = b.clone();
    v.version = "1.0.1".into();
    variants.push(v);
    let mut v = b.clone();
    v.schema[0] ^= 1;
    variants.push(v);
    for v in variants {
        assert_ne!(v.build_id(), id, "{v:?}");
    }
}

#[test]
fn values_cannot_slide_between_inputs() {
    // An absent lock and an empty one differ, and moving bytes from one
    // field to the next changes the encoding.
    let mut a = base();
    a.downloads_lock = Some(Vec::new());
    assert_ne!(a.encoding(), base().encoding());
    let mut a = base();
    a.profile = "releasex".into();
    a.target = "86_64-unknown-linux-gnu".into();
    assert_ne!(a.build_id(), base().build_id());
}

#[test]
fn many_builds_do_not_collide() {
    // DU's T1 check, on synthetic inputs: 20,000 builds differing in
    // commit and version give 20,000 distinct 64-bit ids.
    let mut seen = BTreeSet::new();
    for i in 0..20_000u32 {
        let mut b = base();
        b.commit = format!("{i:040x}");
        b.version = format!("1.{}.{}", i / 100, i % 100);
        assert!(seen.insert(b.build_id()), "collision at {i}");
    }
}

#[test]
fn the_checkout_has_a_build_id() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    // Two reads of one checkout agree, and the schema hash covers the
    // .hgidl files that exist.
    let a = collect(&root, "release", "x86_64-unknown-linux-gnu", "dev", "0.0.0").unwrap();
    let b = collect(&root, "release", "x86_64-unknown-linux-gnu", "dev", "0.0.0").unwrap();
    assert_eq!(a.build_id(), b.build_id());
    assert_eq!(a.schema, schema_hash(&root).unwrap());
    assert_ne!(a.schema, [0; 32]);
    let other = collect(&root, "release", "aarch64-apple-darwin", "dev", "0.0.0").unwrap();
    assert_ne!(a.build_id(), other.build_id());
}

#[test]
fn no_policy_is_the_digest_of_the_empty_policy() {
    // Enterprise policy 6.21: the encoding of the empty entry set with the
    // catalogue version fixed at 0 is the tag `izcal-policy-v1` and the
    // version as a u32 little-endian, with no entries.
    let mut enc = b"izcal-policy-v1".to_vec();
    enc.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(hg_types::NO_POLICY.0, hg_build_id::sha256::sha256(&enc));
}

/// A scratch repository named `name` with one commit of `files`; its
/// path and a function that runs git in it.
fn scratch(name: &str, files: &[(&str, &str)]) -> (std::path::PathBuf, impl Fn(&[&str])) {
    let dir = std::env::temp_dir().join(format!("hg-build-id-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("crates/x/idl")).unwrap();
    std::fs::write(dir.join("rust-toolchain.toml"), "[toolchain]\n").unwrap();
    std::fs::write(dir.join("crates/x/idl/a.hgidl"), "protocol a;\n").unwrap();
    for (path, text) in files {
        std::fs::write(dir.join(path), text).unwrap();
    }
    let at = dir.clone();
    let git = move |args: &[&str]| {
        let ok = std::process::Command::new("git")
            .arg("-C")
            .arg(&at)
            .args(args)
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "git {args:?}");
    };
    git(&["init", "-q"]);
    git(&["add", "-A"]);
    git(&[
        "-c",
        "user.email=t@t",
        "-c",
        "user.name=t",
        "commit",
        "-qm",
        "t",
    ]);
    (dir, git)
}

#[test]
fn an_untracked_file_changes_the_commit_input() {
    let (dir, _git) = scratch("untracked", &[]);
    let clean = collect(&dir, "release", "t", "dev", "0.0.0").unwrap();
    assert!(!clean.commit.contains("+dirty"));
    std::fs::write(dir.join("crates/x/new.rs"), "fn f() {}\n").unwrap();
    let dirty = collect(&dir, "release", "t", "dev", "0.0.0").unwrap();
    assert!(dirty.commit.contains("+dirty:"));
    assert_ne!(clean.build_id(), dirty.build_id());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_dirty_tree_has_one_id_whatever_the_git_config() {
    let (dir, git) = scratch("config", &[("a.txt", "one\ntwo\n"), ("b.txt", "gone\n")]);
    // A changed file, a deleted one and an untracked one.
    std::fs::write(dir.join("a.txt"), "one\nthree\n").unwrap();
    std::fs::remove_file(dir.join("b.txt")).unwrap();
    std::fs::write(dir.join("c.txt"), "new\n").unwrap();
    let before = collect(&dir, "release", "t", "dev", "0.0.0").unwrap();
    assert!(before.commit.contains("+dirty:"));
    // Settings that change what `git diff` or `git status` print, or which
    // untracked files they see.
    let ignore = dir.with_extension("ignore");
    std::fs::write(&ignore, "c.txt\n").unwrap();
    std::fs::write(dir.join(".git/info/exclude"), "c.txt\n").unwrap();
    for (key, value) in [
        ("diff.external", "echo hostile"),
        ("color.ui", "always"),
        ("color.diff", "always"),
        ("color.status", "always"),
        ("diff.noprefix", "true"),
        ("diff.mnemonicPrefix", "true"),
        ("diff.algorithm", "patience"),
        ("diff.renames", "copies"),
        ("diff.context", "9"),
        ("status.renames", "copies"),
        ("status.showUntrackedFiles", "no"),
        ("core.excludesFile", ignore.to_str().unwrap()),
    ] {
        git(&["config", key, value]);
    }
    let after = collect(&dir, "release", "t", "dev", "0.0.0").unwrap();
    assert_eq!(before.commit, after.commit);
    std::fs::remove_dir_all(&dir).unwrap();
    std::fs::remove_file(&ignore).unwrap();
}

#[test]
fn a_nested_checkout_counts_by_its_commit_and_changes() {
    // Git lists a nested checkout (or a submodule) as one directory
    // entry; what is in it must still reach the id.
    let (dir, _git) = scratch("nested", &[]);
    let inner = dir.join("inner");
    std::fs::create_dir_all(&inner).unwrap();
    std::fs::write(inner.join("x.txt"), "one\n").unwrap();
    let git_in = |args: &[&str]| {
        let ok = std::process::Command::new("git")
            .arg("-C")
            .arg(&inner)
            .args(args)
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "git {args:?}");
    };
    git_in(&["init", "-q"]);
    git_in(&["add", "-A"]);
    git_in(&[
        "-c",
        "user.email=t@t",
        "-c",
        "user.name=t",
        "commit",
        "-qm",
        "t",
    ]);
    let first = collect(&dir, "release", "t", "dev", "0.0.0").unwrap();
    // A change inside it, then a new commit of that change.
    std::fs::write(inner.join("x.txt"), "two\n").unwrap();
    let edited = collect(&dir, "release", "t", "dev", "0.0.0").unwrap();
    assert_ne!(first.commit, edited.commit);
    git_in(&[
        "-c",
        "user.email=t@t",
        "-c",
        "user.name=t",
        "commit",
        "-qam",
        "u",
    ]);
    let committed = collect(&dir, "release", "t", "dev", "0.0.0").unwrap();
    assert_ne!(first.commit, committed.commit);
    assert_ne!(edited.commit, committed.commit);
    std::fs::remove_dir_all(&dir).unwrap();
}
