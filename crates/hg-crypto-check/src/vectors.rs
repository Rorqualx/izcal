//! Reading Wycheproof's test vectors.
//!
//! The files are JSON with a schema per primitive class
//! (`mac_test_schema_v1.json`, `hkdf_test_schema_v1.json` and so on).
//! What every class shares is the envelope — an algorithm, a schema
//! name, a test count, test groups, and in each group a list of cases
//! with a `tcId`, a `result` of `valid`, `invalid` or `acceptable`,
//! and a list of `flags` — so that is what this module models, and the
//! per-class fields stay as JSON for the runner that understands them.
//!
//! Modelling it the other way round, with a struct per schema, would
//! mean a new reader for every primitive and a file that silently
//! reads as empty when a schema gains a field.

use std::collections::BTreeMap;
use std::path::Path;

use hg_json::Json;

/// What a vector says the answer should be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    /// The operation must succeed and produce the given output.
    Valid,
    /// The operation must be refused.
    Invalid,
    /// Legacy or borderline: either answer is conforming, and which
    /// one Izcal gives is a decision rather than a bug.
    Acceptable,
}

impl Verdict {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "valid" => Some(Self::Valid),
            "invalid" => Some(Self::Invalid),
            "acceptable" => Some(Self::Acceptable),
            _ => None,
        }
    }
}

/// One test case.
#[derive(Debug, Clone)]
pub struct Case {
    /// Its number within the file.
    pub tc_id: i64,
    /// The comment Wycheproof gives it, which usually names the
    /// property being tested.
    pub comment: String,
    /// Its flags, which index the file's `notes`.
    pub flags: Vec<String>,
    /// What the answer should be.
    pub result: Verdict,
    /// Every other field, as JSON: the per-schema part.
    pub fields: BTreeMap<String, Json>,
}

impl Case {
    /// A field's hex string, decoded.
    #[must_use]
    pub fn hex(&self, key: &str) -> Option<Vec<u8>> {
        unhex(self.fields.get(key)?.as_str()?)
    }

    /// A field's integer.
    #[must_use]
    pub fn int(&self, key: &str) -> Option<i64> {
        self.fields.get(key)?.as_i64()
    }
}

/// One test group: cases that share parameters.
#[derive(Debug, Clone)]
pub struct Group {
    /// The group type, such as `MacTest`.
    pub ty: String,
    /// Group-level parameters, as JSON.
    pub fields: BTreeMap<String, Json>,
    /// Its cases.
    pub cases: Vec<Case>,
}

/// One vector file.
#[derive(Debug, Clone)]
pub struct Suite {
    /// The file's name, for reporting.
    pub name: String,
    /// The algorithm, as Wycheproof names it.
    pub algorithm: String,
    /// The schema the file declares.
    pub schema: String,
    /// The count the file declares, which is checked against the cases
    /// actually read.
    pub declared: i64,
    /// Its groups.
    pub groups: Vec<Group>,
}

impl Suite {
    /// How many cases it holds.
    #[must_use]
    pub fn case_count(&self) -> usize {
        self.groups.iter().map(|g| g.cases.len()).sum()
    }
}

/// Decodes a hex string, or `None` when it is not one.
#[must_use]
pub fn unhex(s: &str) -> Option<Vec<u8>> {
    let bytes = s.as_bytes();
    let (pairs, rest) = bytes.as_chunks::<2>();
    if !rest.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(pairs.len());
    for p in pairs {
        let text = std::str::from_utf8(p).ok()?;
        out.push(u8::from_str_radix(text, 16).ok()?);
    }
    Some(out)
}

fn object_fields(v: &Json, skip: &[&str]) -> BTreeMap<String, Json> {
    let mut out = BTreeMap::new();
    if let Some(pairs) = v.as_obj() {
        for (k, value) in pairs {
            if !skip.contains(&k.as_str()) {
                out.insert(k.clone(), value.clone());
            }
        }
    }
    out
}

/// Reads one vector file.
///
/// # Errors
/// A message naming the file when it cannot be read, is not JSON, or
/// does not have the envelope every Wycheproof schema shares.
pub fn read(path: &Path) -> Result<Suite, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let json = hg_json::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into(),
    );

    // Four of the pinned files -- the JSON Web Signature, Encryption,
    // Key and Crypto suites -- carry no `algorithm`, because each
    // covers a container format rather than one primitive. Their file
    // name is the name, and refusing them would mean the runner could
    // not report the 1,169 cases they hold at all.
    let algorithm = json
        .get("algorithm")
        .and_then(Json::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| {
            name.strip_suffix("_test.json")
                .unwrap_or(name.as_str())
                .to_owned()
        });
    let schema = json
        .get("schema")
        .and_then(Json::as_str)
        .unwrap_or("")
        .to_owned();
    let declared = json
        .get("numberOfTests")
        .and_then(Json::as_i64)
        .ok_or_else(|| format!("{name}: no `numberOfTests`"))?;
    let groups_json = json
        .get("testGroups")
        .and_then(Json::as_arr)
        .ok_or_else(|| format!("{name}: no `testGroups`"))?;

    let mut groups = Vec::new();
    for g in groups_json {
        let ty = g
            .get("type")
            .and_then(Json::as_str)
            .unwrap_or("")
            .to_owned();
        let cases_json = g
            .get("tests")
            .and_then(Json::as_arr)
            .ok_or_else(|| format!("{name}: a group with no `tests`"))?;
        let mut cases = Vec::new();
        for c in cases_json {
            let tc_id = c
                .get("tcId")
                .and_then(Json::as_i64)
                .ok_or_else(|| format!("{name}: a case with no `tcId`"))?;
            let result_text = c
                .get("result")
                .and_then(Json::as_str)
                .ok_or_else(|| format!("{name}: case {tc_id} has no `result`"))?;
            let result = Verdict::parse(result_text)
                .ok_or_else(|| format!("{name}: case {tc_id}: unknown result {result_text}"))?;
            let flags = c
                .get("flags")
                .and_then(Json::as_arr)
                .map(|a| {
                    a.iter()
                        .filter_map(|f| f.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            cases.push(Case {
                tc_id,
                comment: c
                    .get("comment")
                    .and_then(Json::as_str)
                    .unwrap_or("")
                    .to_owned(),
                flags,
                result,
                fields: object_fields(c, &["tcId", "comment", "flags", "result"]),
            });
        }
        groups.push(Group {
            ty,
            fields: object_fields(g, &["type", "tests"]),
            cases,
        });
    }

    Ok(Suite {
        name,
        algorithm,
        schema,
        declared,
        groups,
    })
}

/// The directory the pinned vectors are unpacked into.
///
/// # Errors
/// A message when neither `$IZCAL_TEST_DATA` nor `$HOME` is set.
pub fn vectors_dir() -> Result<std::path::PathBuf, String> {
    let base = match std::env::var("IZCAL_TEST_DATA") {
        Ok(v) if !v.is_empty() => std::path::PathBuf::from(v),
        _ => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| "neither IZCAL_TEST_DATA nor HOME is set".to_owned())?;
            std::path::PathBuf::from(home).join(".cache/izcal/test-data")
        }
    };
    Ok(base
        .join(format!("wycheproof-{PIN}"))
        .join("testvectors_v1"))
}

/// The pinned Wycheproof commit, as `tools/test-data.toml` gives it.
pub const PIN: &str = "65ca494727a52944c204441c860b039e6786b99b";

/// Every vector file in the pinned set, sorted by name.
///
/// # Errors
/// A message when the directory cannot be read.
pub fn all(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let p = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}
