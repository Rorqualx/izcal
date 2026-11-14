//! `hg-crypto`'s `pins.toml`: every primitive resolved, every licence
//! checked.
//!
//! The check lives here rather than beside the file because G3 forbids
//! ambient file access in tier P, and CS-AS-1 binds that to test
//! targets too -- so a tier P crate cannot read its own manifest
//! directory. This crate is tier T and reading pinned data is its job,
//! which makes it the right home anyway: `pins.toml` is a record to be
//! checked, and this is the crate that checks things.
//!
//! NW-M0 asks for "`hg-crypto`'s interfaces with every pin resolved
//! and licence-checked". A record nobody checks resolves nothing, so
//! this is the check — and it is deliberately a *text* check rather
//! than a parse into a type, because the file is read by people and a
//! schema would make it easy to add a field nobody reads.
//!
//! What it holds the file to:
//!
//! - every primitive TLS 1.3 and the trust pipeline need has a row;
//! - every row names a licence HGBA-2 2.3.2 allows;
//! - every row that says `status = "built"` names a primitive this
//!   crate actually exports, and every `pinned` one does not;
//! - no row names a crate that is in `Cargo.toml` but unused, and none
//!   is in `Cargo.toml` before its code is here.

use std::path::PathBuf;

/// `crates/hg-crypto`, from this crate's own manifest directory.
fn crypto_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../hg-crypto")
}

fn pins() -> String {
    let path = crypto_dir().join("pins.toml");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Each `[[primitive]]` block, as (name, field-text) pairs.
fn blocks(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        let t = line.trim();
        if t == "[[primitive]]" {
            if let Some(b) = current.take() {
                out.push(b);
            }
            current = Some((String::new(), String::new()));
        } else if let Some((name, body)) = current.as_mut() {
            if let Some(v) = t.strip_prefix("name = ") {
                *name = v.trim_matches('"').to_owned();
            }
            body.push_str(t);
            body.push('\n');
        }
    }
    if let Some(b) = current.take() {
        out.push(b);
    }
    out
}

fn field<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    body.lines()
        .find_map(|l| l.strip_prefix(&format!("{key} = ")))
        .map(|v| v.trim_matches('"'))
}

/// Every primitive the TLS 1.3 client and the trust pipeline need has
/// a row. The list is what NW-M1's own milestone row names, so a
/// primitive that arrives without a pin is caught here.
#[test]
fn cr20_every_primitive_the_stack_needs_has_a_row() {
    let text = pins();
    let rows = blocks(&text);
    let names: Vec<&str> = rows.iter().map(|(n, _)| n.as_str()).collect();
    for needed in [
        "sha-256",
        "sha-384",
        "hmac",
        "hkdf",
        "aes-gcm",
        "chacha20-poly1305",
        "x25519",
        "p-256",
        "p-384",
        "ed25519",
        "rsa",
        "ml-kem-768",
        "hpke",
    ] {
        assert!(names.contains(&needed), "no pin row for {needed}");
    }
    let mut sorted = names.clone();
    sorted.sort_unstable();
    let before = sorted.len();
    sorted.dedup();
    assert_eq!(sorted.len(), before, "a primitive appears twice");
}

/// Every row names a licence HGBA-2 2.3.2 allows: MIT, or a dual
/// licence with MIT in it. Anything else Izcal builds itself.
#[test]
fn cr21_every_pin_is_mit_or_dual_licensed_with_mit() {
    let text = pins();
    for (name, body) in blocks(&text) {
        let licence = field(&body, "licence").unwrap_or_else(|| panic!("{name}: no licence"));
        assert!(
            licence == "MIT" || licence.split(" OR ").any(|l| l == "MIT"),
            "{name}: {licence} is not MIT and not dual-licensed with MIT"
        );
        let source = field(&body, "source").unwrap_or_else(|| panic!("{name}: no source"));
        assert!(
            source == "izcal" || source == "pinned",
            "{name}: source {source}"
        );
        if source == "izcal" {
            assert_eq!(licence, "MIT", "{name}: Izcal's own code is MIT");
        } else {
            assert!(
                field(&body, "crate").is_some(),
                "{name}: a pinned source names its crate"
            );
        }
        assert!(
            field(&body, "standard").is_some(),
            "{name}: no standard named"
        );
        assert!(field(&body, "why").is_some(), "{name}: no reason given");
    }
}

/// `status = "built"` means the code is here. Exactly three rows say
/// so, and they are the three the crate exports.
#[test]
fn cr22_built_means_built() {
    let text = pins();
    let built: Vec<String> = blocks(&text)
        .into_iter()
        .filter(|(_, b)| field(b, "status") == Some("built"))
        .map(|(n, _)| n)
        .collect();
    assert_eq!(
        built,
        vec!["sha-256".to_owned(), "hmac".to_owned(), "hkdf".to_owned()],
        "the built rows are the primitives this crate implements"
    );
    for (name, body) in blocks(&text) {
        let status = field(&body, "status").unwrap_or_else(|| panic!("{name}: no status"));
        assert!(status == "built" || status == "pinned", "{name}: {status}");
    }
}

/// G36 and §9.1: a pinned dependency arrives with its primitive, not
/// before. No cryptography crate is in `Cargo.toml` while every row
/// that names one is still `pinned`.
#[test]
fn cr23_no_pinned_crate_is_a_dependency_yet() {
    let manifest = std::fs::read_to_string(crypto_dir().join("Cargo.toml")).expect("the manifest");
    let text = pins();
    for (name, body) in blocks(&text) {
        if field(&body, "status") == Some("pinned")
            && let Some(krate) = field(&body, "crate")
        {
            assert!(
                !manifest.contains(krate),
                "{name}: {krate} is a dependency but its row still says pinned"
            );
        }
    }
    // And nothing else crept in: this crate has no dependencies at
    // all, which is what makes "only hg-crypto depends on a
    // cryptography crate" checkable here rather than only in the
    // graph lint.
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap_or("")
        .split("\n[")
        .next()
        .unwrap_or("");
    assert_eq!(
        deps.trim(),
        "",
        "hg-crypto has a dependency; add it with its primitive and update pins.toml"
    );
}
