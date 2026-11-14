//! Reading WHATWG's `entities.json`.
//!
//! The file is one flat object: `"&amp;": {"codepoints": [38],
//! "characters": "&"}`. Only the name and `characters` are used, and
//! the name is stored without its leading ampersand, which the
//! tokenizer has consumed before the named character reference state.
//!
//! It is read with `hg-json`, Izcal's own reader, as every generator's
//! pinned inputs are: no generator takes a dependency outside the tree.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What the data set is called in `tools/test-data.toml`.
pub const DATA_SET: &str = "html-entities";

/// The pin recorded there. The published file has no revision of its
/// own, so it is pinned by its own content hash.
pub const PIN: &str = "d741d877ac77";

/// The pinned `entities.json`, in the cache `tools/fetch-test-data.sh`
/// writes to: `$IZCAL_TEST_DATA`, or `~/.cache/izcal/test-data`.
///
/// # Errors
/// A message when neither `$IZCAL_TEST_DATA` nor `$HOME` is set.
pub fn path_from_env() -> Result<PathBuf, String> {
    let cache = match std::env::var_os("IZCAL_TEST_DATA") {
        Some(v) => PathBuf::from(v),
        None => {
            let home = std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .ok_or("neither IZCAL_TEST_DATA nor HOME is set")?;
            PathBuf::from(home).join(".cache/izcal/test-data")
        }
    };
    Ok(cache
        .join(format!("{DATA_SET}-{PIN}"))
        .join("entities.json"))
}

/// Reads the table: identifier without its ampersand, to the one or two
/// characters it stands for.
///
/// # Errors
/// A message naming the file when it cannot be read or is not the shape
/// above.
pub fn read(path: &Path) -> Result<BTreeMap<String, String>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parses the text of `entities.json`.
///
/// # Errors
/// A message when the text is not the shape above.
pub fn parse(text: &str) -> Result<BTreeMap<String, String>, String> {
    let v = hg_json::parse(text).map_err(|e| e.to_string())?;
    let pairs = v
        .as_obj()
        .ok_or_else(|| "entities.json is not an object".to_owned())?;
    let mut out = BTreeMap::new();
    for (name, entry) in pairs {
        let characters = entry
            .get("characters")
            .and_then(hg_json::Json::as_str)
            .ok_or_else(|| format!("{name:?} has no `characters` string"))?;
        let key = name
            .strip_prefix('&')
            .ok_or_else(|| format!("{name:?} does not start with an ampersand"))?;
        if out.insert(key.to_owned(), characters.to_owned()).is_some() {
            return Err(format!("{name:?} appears twice"));
        }
    }
    if out.is_empty() {
        return Err("no named character references".to_owned());
    }
    Ok(out)
}
