//! Where the pinned test data lives: `tools/test-data.toml`, fetched by
//! `tools/fetch-test-data.sh` into `$IZCAL_TEST_DATA/<name>-<commit>/`
//! (default `~/.cache/izcal/test-data`).

use std::path::{Path, PathBuf};

/// One `[name]` table of `tools/test-data.toml`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataSet {
    /// The table name (`test262`, `engine262`).
    pub name: String,
    /// The pinned commit.
    pub commit: String,
    /// The commit's date.
    pub date: String,
    /// The archive's SHA-256.
    pub sha256: String,
}

/// Reads the flat `[name]` / `key = "value"` subset the fetch script
/// reads; anything else is an error naming the line.
pub fn parse_test_data(text: &str) -> Result<Vec<DataSet>, String> {
    let mut out: Vec<DataSet> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            out.push(DataSet {
                name: name.to_string(),
                ..DataSet::default()
            });
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            return Err(format!(
                "test-data.toml line {}: expected `key = \"value\"`",
                i + 1
            ));
        };
        let v = v.trim();
        let Some(v) = v.strip_prefix('"').and_then(|v| v.strip_suffix('"')) else {
            return Err(format!(
                "test-data.toml line {}: values are quoted strings",
                i + 1
            ));
        };
        let Some(cur) = out.last_mut() else {
            return Err(format!(
                "test-data.toml line {}: a key before any [table]",
                i + 1
            ));
        };
        match k.trim() {
            "commit" => cur.commit = v.to_string(),
            "date" => cur.date = v.to_string(),
            "sha256" => cur.sha256 = v.to_string(),
            _ => {}
        }
    }
    Ok(out)
}

/// The repository's `tools/test-data.toml`.
pub fn default_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/test-data.toml")
}

/// The cache root: `$IZCAL_TEST_DATA`, else `~/.cache/izcal/test-data`.
pub fn cache_root() -> PathBuf {
    if let Some(d) = std::env::var_os("IZCAL_TEST_DATA") {
        return PathBuf::from(d);
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .unwrap_or_default();
    PathBuf::from(home).join(".cache/izcal/test-data")
}

/// The unpacked tree of a pinned data set, and its record.
pub fn locate(name: &str) -> Result<(PathBuf, DataSet), String> {
    let manifest = default_manifest();
    let text =
        std::fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
    let sets = parse_test_data(&text)?;
    let set = sets
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("no [{name}] in {}", manifest.display()))?;
    let dir = cache_root().join(format!("{}-{}", set.name, set.commit));
    let stamp = std::fs::read_to_string(dir.join(".izcal-sha256")).unwrap_or_default();
    if stamp.trim() != set.sha256 {
        return Err(format!(
            "{} is missing or not the pinned archive; run tools/fetch-test-data.sh {name}",
            dir.display()
        ));
    }
    Ok((dir, set))
}

/// Today's UTC date, `YYYY-MM-DD`, from the system clock.
pub fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let days = (secs / 86_400) as i64;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// The proleptic Gregorian date of a day count since 1970-01-01: whole
/// 400-year eras of 146,097 days, then the year in the era counted from
/// 1 March so the leap day falls last.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}
