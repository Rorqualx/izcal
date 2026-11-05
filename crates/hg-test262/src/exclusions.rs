//! Feature-exclusion lists: what an engine under test does not implement.
//!
//! One file per engine. Each line is `feature <name>` (a test262 feature
//! tag, `features.txt`) or `path <prefix>` (a prefix of the path under
//! `test/`, for suites gated by directory rather than tag, such as
//! `intl402/` for an engine without ECMA-402, INTERPRETING.md
//! "Implementations without ECMA-402"). `#` starts a comment, which should
//! say why the entry is there. A test matching any entry is reported as
//! excluded with the entry as its reason, never as a failure, and the
//! per-feature summary shows what the exclusions cover.

use std::fmt;

/// One exclusion entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Exclusion {
    /// A test262 feature tag.
    Feature(String),
    /// A path prefix under `test/`.
    Path(String),
}

impl fmt::Display for Exclusion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Exclusion::Feature(n) => write!(f, "feature {n}"),
            Exclusion::Path(p) => write!(f, "path {p}"),
        }
    }
}

/// A parsed exclusion list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exclusions {
    /// The entries, in file order.
    pub entries: Vec<Exclusion>,
}

impl Exclusions {
    /// Parses an exclusion list; errors name the line.
    pub fn parse(text: &str) -> Result<Exclusions, String> {
        let mut entries = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut words = line.split_whitespace();
            let (Some(kind), Some(value), None) = (words.next(), words.next(), words.next()) else {
                return Err(format!(
                    "line {}: expected `feature <name>` or `path <prefix>`, found `{line}`",
                    i + 1
                ));
            };
            let e = match kind {
                "feature" => Exclusion::Feature(value.to_string()),
                "path" => Exclusion::Path(value.trim_start_matches("test/").to_string()),
                other => return Err(format!("line {}: unknown entry kind `{other}`", i + 1)),
            };
            if entries.contains(&e) {
                return Err(format!("line {}: `{e}` listed twice", i + 1));
            }
            entries.push(e);
        }
        Ok(Exclusions { entries })
    }

    /// The first entry that excludes a test, if any.
    pub fn matches(&self, path: &str, features: &[String]) -> Option<&Exclusion> {
        self.entries.iter().find(|e| match e {
            Exclusion::Feature(n) => features.iter().any(|f| f == n),
            Exclusion::Path(p) => path.starts_with(p.as_str()),
        })
    }
}

#[cfg(test)]
mod tests;
