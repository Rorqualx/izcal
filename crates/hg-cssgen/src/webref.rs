//! Reading `@webref/css`'s `css.json`.
//!
//! The package is the W3C's consolidated extraction of CSS features
//! from the published specifications. Its README states that the data
//! "appears in a standalone `css.json` file within the package" and
//! "may be parsed as JSON in any programming language to yield the same
//! object" as its JavaScript API returns, so that file is what is read
//! here, with [`hg_json`]. The package's `css-tree` peer dependency
//! belongs to that JavaScript API; Izcal does not install it and does
//! not call it. Using webref's data is M-1-clean; using CSSTree or any
//! other CSS parser library is not.
//!
//! Only `properties` is read at ST-M0. `atrules`, `selectors`,
//! `functions` and `types` are the next slices of the same import.
//!
//! A property entry looks like this, with the optional fields absent
//! where the specification states them in a form webref did not
//! extract:
//!
//! ```json
//! {
//!   "name": "color",
//!   "href": "https://drafts.csswg.org/css-color-4/#propdef-color",
//!   "initial": "CanvasText",
//!   "inherited": "yes",
//!   "computedValue": "computed color, see resolving color values",
//!   "animationType": "by computed value type",
//!   "syntax": "<color>"
//! }
//! ```
//!
//! An entry with `legacyAliasOf` is not a property of its own: it is a
//! second spelling of one, and the Compatibility Standard defines 64 of
//! them. They are kept apart from the properties here, because a table
//! that mixed them would answer "how many properties are there" wrongly.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use hg_json::Json;

/// What the data set is called in `tools/test-data.toml`.
pub const DATA_SET: &str = "webref-css";

/// The version pinned there.
pub const PIN: &str = "8.7.9";

/// One property, as webref carries it. A field is `None` when the entry
/// does not have it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Property {
    /// Its name, as CSS spells it.
    pub name: String,
    /// The URL of its definition in the specification.
    pub href: String,
    /// Its initial value.
    pub initial: Option<String>,
    /// What it says about inheritance, as written. Usually `yes` or
    /// `no`, but at the 8.7.9 pin also "see individual properties",
    /// `yes?`, `?`, `n/a` and "no (but see prose)" — so it is kept as
    /// the text it is and the overlay decides what the cascade does.
    pub inherited: Option<String>,
    /// Its computed-value text, as the specification's prose writes it.
    pub computed: Option<String>,
    /// Its animation type.
    pub animation: Option<String>,
    /// Its value grammar, in value-definition syntax.
    pub syntax: Option<String>,
    /// Its longhands, in the order webref carries them.
    pub longhands: Vec<String>,
}

/// What `css.json` says, split as the module describes.
#[derive(Debug, Clone, Default)]
pub struct Catalogue {
    /// The properties, by name.
    pub properties: BTreeMap<String, Property>,
    /// The legacy aliases: the alias, to the property it stands for.
    pub aliases: BTreeMap<String, String>,
}

/// The pinned `css.json`, in the cache `tools/fetch-test-data.sh`
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
    Ok(cache.join(format!("{DATA_SET}-{PIN}")).join("css.json"))
}

/// Reads and splits the catalogue.
///
/// # Errors
/// A message naming the file when it cannot be read or is not the shape
/// above.
pub fn read(path: &Path) -> Result<Catalogue, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parses the text of `css.json`.
///
/// # Errors
/// A message when the text is not the shape above.
pub fn parse(text: &str) -> Result<Catalogue, String> {
    let v = hg_json::parse(text).map_err(|e| e.to_string())?;
    let entries = v
        .get("properties")
        .and_then(Json::as_arr)
        .ok_or_else(|| "css.json has no `properties` array".to_owned())?;
    let mut out = Catalogue::default();
    for e in entries {
        let name = e
            .get("name")
            .and_then(Json::as_str)
            .ok_or_else(|| "a property entry has no `name`".to_owned())?
            .to_owned();
        if let Some(target) = e.get("legacyAliasOf").and_then(Json::as_str) {
            if out
                .aliases
                .insert(name.clone(), target.to_owned())
                .is_some()
            {
                return Err(format!("the alias {name} appears twice"));
            }
            continue;
        }
        let href = e
            .get("href")
            .and_then(Json::as_str)
            .ok_or_else(|| format!("{name} has no `href`"))?
            .to_owned();
        let longhands = match e.get("longhands") {
            None => Vec::new(),
            Some(Json::Arr(items)) => items
                .iter()
                .map(|i| {
                    i.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| format!("{name} has a longhand that is not a string"))
                })
                .collect::<Result<Vec<String>, String>>()?,
            Some(_) => return Err(format!("{name}'s `longhands` is not an array")),
        };
        let text_of = |k: &str| e.get(k).and_then(Json::as_str).map(str::to_owned);
        let p = Property {
            href,
            initial: text_of("initial"),
            inherited: text_of("inherited"),
            computed: text_of("computedValue"),
            animation: text_of("animationType"),
            syntax: text_of("syntax"),
            longhands,
            name: name.clone(),
        };
        if out.properties.insert(name.clone(), p).is_some() {
            return Err(format!("the property {name} appears twice"));
        }
    }
    if out.properties.is_empty() {
        return Err("css.json carries no properties".to_owned());
    }
    Ok(out)
}
