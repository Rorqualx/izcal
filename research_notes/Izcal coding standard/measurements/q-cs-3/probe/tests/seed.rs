//! Q-CS-3 probe: which proptest configuration gives a fixed, printed seed.
//! Each property appends every generated value to the file named by
//! `QCS3_OUT.<test>`, so two runs can be compared.
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};
use std::io::Write;

fn record(tag: &str, v: &(u64, String)) {
    if let Ok(path) = std::env::var("QCS3_OUT") {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{path}.{tag}"))
            .unwrap();
        f.write_all(format!("{} {:?}\n", v.0, v.1).as_bytes())
            .unwrap();
    }
}

proptest! {
    // Default config: seed source decided by proptest (env or random).
    #[test]
    fn default_config(v in (any::<u64>(), "[a-z]{0,8}")) {
        record("default", &v);
    }
}

proptest! {
    #![proptest_config(Config { rng_seed: RngSeed::Fixed(0x1234_5678), ..Config::default() })]
    #[test]
    fn fixed_in_config(v in (any::<u64>(), "[a-z]{0,8}")) {
        record("fixed", &v);
    }
}

proptest! {
    // Fails on purpose when QCS3_FAIL is set, to see what a failure prints.
    #[test]
    fn failing(v in (any::<u64>(), "[a-z]{0,8}")) {
        record("failing", &v);
        if std::env::var("QCS3_FAIL").is_ok() {
            prop_assert!(v.0 % 7 != 3, "value {} hit the bug", v.0);
        }
    }
}
