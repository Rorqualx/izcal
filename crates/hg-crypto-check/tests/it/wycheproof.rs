//! The pinned Wycheproof set, read and run.
//!
//! Two different things are held here and it is worth keeping them
//! apart. The **classification** — how many files, algorithms and
//! cases the pinned set holds — can regress when the reader starts
//! dropping something, and it does so silently; that is what the
//! counts are for. The **pass counts** are the oracle proper, and they
//! can only go up.

use hg_crypto_check::run::Summary;
use hg_crypto_check::{
    ALGORITHMS, CASES, FILES, HKDF_SHA256_BASELINE, HMAC_SHA256_BASELINE, run, vectors,
};

/// `None` when the pinned set has not been fetched, so a checkout
/// without it is not a failure — unless `IZCAL_REQUIRE_CRYPTO_DATA`
/// says the job must have it, as CI's does.
fn summary() -> Option<Summary> {
    let required = std::env::var("IZCAL_REQUIRE_CRYPTO_DATA").is_ok_and(|v| v == "1");
    let dir = match vectors::vectors_dir() {
        Ok(d) => d,
        Err(e) => {
            assert!(!required, "IZCAL_REQUIRE_CRYPTO_DATA=1 but: {e}");
            return None;
        }
    };
    let files = match vectors::all(&dir) {
        Ok(f) => f,
        Err(e) => {
            assert!(
                !required,
                "IZCAL_REQUIRE_CRYPTO_DATA=1 but the vectors are missing: {e}"
            );
            eprintln!("skipping: {e}; run `tools/fetch-test-data.sh wycheproof`");
            return None;
        }
    };
    let mut s = Summary::default();
    let mut unreadable = Vec::new();
    for p in &files {
        match vectors::read(p) {
            Ok(suite) => run::run_suite(&suite, &mut s),
            Err(e) => unreadable.push(e),
        }
    }
    assert!(unreadable.is_empty(), "unreadable: {unreadable:?}");
    assert_eq!(files.len(), FILES, "the pinned set's file count");
    Some(s)
}

/// Every file's declared `numberOfTests` agrees with the cases read.
/// A reader that dropped a group would otherwise lower the pass count
/// and raise the rate at the same time.
#[test]
fn cc01_no_file_declares_more_cases_than_it_holds() {
    let Some(s) = summary() else { return };
    assert!(
        s.miscounted.is_empty(),
        "files whose count does not match: {:?}",
        s.miscounted
    );
}

/// The classification, which can regress.
#[test]
fn cc02_the_pinned_set_is_what_it_was() {
    let Some(s) = summary() else { return };
    assert_eq!(s.by_algorithm.len(), ALGORITHMS, "algorithms");
    assert_eq!(s.total(), CASES, "cases");
}

/// The oracle: every case of the two primitives Izcal has.
#[test]
fn cc03_hmac_and_hkdf_pass_every_vector() {
    let Some(s) = summary() else { return };
    let hmac = s
        .by_algorithm
        .get("HMACSHA256")
        .expect("the set has HMAC-SHA-256");
    assert_eq!(hmac.fail, 0, "failures: {:?}", hmac.failures);
    assert!(
        hmac.pass >= HMAC_SHA256_BASELINE,
        "HMAC-SHA-256 passed {} of {}, baseline {HMAC_SHA256_BASELINE}",
        hmac.pass,
        hmac.total()
    );
    assert_eq!(hmac.unimplemented, 0, "every case ran");

    let hkdf = s
        .by_algorithm
        .get("HKDF-SHA-256")
        .expect("the set has HKDF-SHA-256");
    assert_eq!(hkdf.fail, 0, "failures: {:?}", hkdf.failures);
    assert!(
        hkdf.pass >= HKDF_SHA256_BASELINE,
        "HKDF-SHA-256 passed {} of {}, baseline {HKDF_SHA256_BASELINE}",
        hkdf.pass,
        hkdf.total()
    );
    assert_eq!(hkdf.unimplemented, 0, "every case ran");

    println!(
        "hg-crypto-check: HMACSHA256 {}/{}, HKDF-SHA-256 {}/{}, at {}",
        hmac.pass,
        hmac.total(),
        hkdf.pass,
        hkdf.total(),
        vectors::PIN
    );
}

/// Exactly two algorithms run today. The assertion is on the *list*
/// rather than the count, so a primitive that lands shows up here as a
/// name to add rather than as a number that moved.
#[test]
fn cc04_the_runner_covers_what_hg_crypto_implements() {
    let Some(s) = summary() else { return };
    assert_eq!(
        s.implemented_algorithms(),
        vec!["HKDF-SHA-256", "HMACSHA256"],
        "add the new primitive's algorithm here when it lands"
    );
    assert!(
        s.unimplemented() > 100_000,
        "the rest of the set is classified, not silently dropped"
    );
}

/// A file the reader cannot understand is an error, not a skip: a
/// Wycheproof schema change that made every file unreadable would
/// otherwise look like a clean run over nothing.
#[test]
fn cc05_an_unreadable_file_is_an_error() {
    let dir = std::env::temp_dir().join("izcal-cc05");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("broken_test.json");
    std::fs::write(&path, "{\"algorithm\": \"X\"}").expect("write");
    let err = vectors::read(&path).expect_err("no numberOfTests");
    assert!(err.contains("numberOfTests"), "got: {err}");

    // The twin: the same file with the envelope it needs.
    std::fs::write(
        &path,
        "{\"algorithm\":\"X\",\"numberOfTests\":0,\"testGroups\":[]}",
    )
    .expect("write");
    let suite = vectors::read(&path).expect("reads");
    assert_eq!(suite.algorithm, "X");
    assert_eq!(suite.case_count(), 0);
}
