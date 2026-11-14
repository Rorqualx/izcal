//! Reading the Encoding Standard's `encodings.json` and `indexes.json`.
//!
//! Both are published from the standard's own source and are not files
//! in the `whatwg/encoding` tree, so neither has a revision of its own
//! and each is pinned by its content hash (`tools/test-data.toml`).
//! They are read with [`hg_json`], as every generator's pinned inputs
//! are.
//!
//! `encodings.json` is a list of groups, each with a heading and its
//! encodings; an encoding has a `name` and its `labels`.
//!
//! `indexes.json` is one object: a name to a list. Most lists are code
//! points with `null` where the index has no entry; `gb18030-ranges` is
//! a list of `[pointer, code point]` pairs instead.

use std::collections::BTreeMap;
use std::path::PathBuf;

use hg_json::Json;

/// What the two data sets are called in `tools/test-data.toml`.
pub const ENCODINGS_SET: &str = "encoding-encodings";
/// The pin recorded there for `encodings.json`.
pub const ENCODINGS_PIN: &str = "078212b3697f";
/// What the index data set is called in `tools/test-data.toml`.
pub const INDEXES_SET: &str = "encoding-indexes";
/// The pin recorded there for `indexes.json`.
pub const INDEXES_PIN: &str = "b7c8961095f0";

/// One encoding: its name, its labels, and the group it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encoding {
    /// The standard's name for it, in the standard's own spelling.
    pub name: String,
    /// Every label that selects it, lower case, in the order written.
    pub labels: Vec<String>,
    /// The heading of the group it is listed under.
    pub group: String,
}

/// One index table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Index {
    /// Pointer to code point, with `None` where the index has no entry.
    Points(Vec<Option<u32>>),
    /// `gb18030-ranges`: (pointer, code point) pairs, ascending.
    Ranges(Vec<(u32, u32)>),
}

/// Everything the two files say.
#[derive(Debug, Clone, Default)]
pub struct Data {
    /// The encodings, in the order the standard lists them.
    pub encodings: Vec<Encoding>,
    /// The index tables, by the standard's name for each.
    pub indexes: BTreeMap<String, Index>,
}

/// The cache directory `tools/fetch-test-data.sh` writes to.
///
/// # Errors
/// A message when neither `$IZCAL_TEST_DATA` nor `$HOME` is set.
pub fn cache() -> Result<PathBuf, String> {
    match std::env::var_os("IZCAL_TEST_DATA") {
        Some(v) => Ok(PathBuf::from(v)),
        None => {
            let home = std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .ok_or("neither IZCAL_TEST_DATA nor HOME is set")?;
            Ok(PathBuf::from(home).join(".cache/izcal/test-data"))
        }
    }
}

/// Where the pinned `encodings.json` is.
///
/// # Errors
/// A message when the cache directory cannot be named.
pub fn encodings_path() -> Result<PathBuf, String> {
    Ok(cache()?
        .join(format!("{ENCODINGS_SET}-{ENCODINGS_PIN}"))
        .join("encodings.json"))
}

/// Where the pinned `indexes.json` is.
///
/// # Errors
/// A message when the cache directory cannot be named.
pub fn indexes_path() -> Result<PathBuf, String> {
    Ok(cache()?
        .join(format!("{INDEXES_SET}-{INDEXES_PIN}"))
        .join("indexes.json"))
}

/// Reads both files, when both are in the cache.
///
/// `Ok(None)` means they are not fetched.
///
/// # Errors
/// A message naming the file when one is there but cannot be read or is
/// not the shape above.
pub fn read() -> Result<Option<Data>, String> {
    let (e, i) = (encodings_path()?, indexes_path()?);
    if !e.is_file() || !i.is_file() {
        return Ok(None);
    }
    let et = std::fs::read_to_string(&e).map_err(|x| format!("{}: {x}", e.display()))?;
    let it = std::fs::read_to_string(&i).map_err(|x| format!("{}: {x}", i.display()))?;
    let mut d = Data {
        encodings: parse_encodings(&et).map_err(|x| format!("{}: {x}", e.display()))?,
        indexes: BTreeMap::new(),
    };
    d.indexes = parse_indexes(&it).map_err(|x| format!("{}: {x}", i.display()))?;
    Ok(Some(d))
}

/// Parses `encodings.json`.
///
/// # Errors
/// A message when the text is not the shape above.
pub fn parse_encodings(text: &str) -> Result<Vec<Encoding>, String> {
    let v = hg_json::parse(text).map_err(|e| e.to_string())?;
    let groups = v
        .as_arr()
        .ok_or_else(|| "encodings.json is not an array".to_owned())?;
    let mut out = Vec::new();
    for g in groups {
        let heading = g
            .get("heading")
            .and_then(Json::as_str)
            .ok_or_else(|| "a group has no `heading`".to_owned())?;
        let list = g
            .get("encodings")
            .and_then(Json::as_arr)
            .ok_or_else(|| format!("{heading} has no `encodings` array"))?;
        for e in list {
            let name = e
                .get("name")
                .and_then(Json::as_str)
                .ok_or_else(|| format!("an encoding in {heading} has no `name`"))?;
            let labels = e
                .get("labels")
                .and_then(Json::as_arr)
                .ok_or_else(|| format!("{name} has no `labels` array"))?;
            let mut names = Vec::new();
            for l in labels {
                let s = l
                    .as_str()
                    .ok_or_else(|| format!("{name} has a label that is not a string"))?;
                names.push(s.to_owned());
            }
            out.push(Encoding {
                name: name.to_owned(),
                labels: names,
                group: heading.to_owned(),
            });
        }
    }
    if out.is_empty() {
        return Err("encodings.json lists no encodings".to_owned());
    }
    Ok(out)
}

/// Parses `indexes.json`.
///
/// # Errors
/// A message when the text is not the shape above.
pub fn parse_indexes(text: &str) -> Result<BTreeMap<String, Index>, String> {
    let v = hg_json::parse(text).map_err(|e| e.to_string())?;
    let obj = v
        .as_obj()
        .ok_or_else(|| "indexes.json is not an object".to_owned())?;
    let mut out = BTreeMap::new();
    for (name, list) in obj {
        let items = list
            .as_arr()
            .ok_or_else(|| format!("index {name} is not an array"))?;
        // gb18030-ranges is the one index written as pairs.
        let index = if items.first().and_then(Json::as_arr).is_some() {
            let mut pairs = Vec::new();
            for it in items {
                let p = it
                    .as_arr()
                    .ok_or_else(|| format!("index {name} mixes pairs and values"))?;
                let (a, b) = (p.first(), p.get(1));
                pairs.push((
                    number(a).ok_or_else(|| format!("index {name} has a bad pointer"))?,
                    number(b).ok_or_else(|| format!("index {name} has a bad code point"))?,
                ));
            }
            Index::Ranges(pairs)
        } else {
            let mut points = Vec::new();
            for it in items {
                points.push(match it {
                    Json::Null => None,
                    other => Some(
                        number(Some(other))
                            .ok_or_else(|| format!("index {name} has a bad code point"))?,
                    ),
                });
            }
            Index::Points(points)
        };
        out.insert(name.clone(), index);
    }
    if out.is_empty() {
        return Err("indexes.json has no indexes".to_owned());
    }
    Ok(out)
}

fn number(v: Option<&Json>) -> Option<u32> {
    match v {
        Some(Json::Num(n)) => n.parse().ok(),
        _ => None,
    }
}
