//! `hg-wpt-layout`: the WPT `css/CSS2` runner. Tools, tier T.
//!
//! # What this measures, and what it does not
//!
//! LY-M0 has no layout engine. A runner that reported a pass rate
//! today would be reporting zero and calling it a result. So what it
//! reports instead is **reachability**: for every test in the pinned
//! suite, what a browser would need to run it, and how many of each
//! kind there are.
//!
//! That is a number that can only go up, and it is a real regression
//! guard: when the tree builder lands, the count of runnable tests
//! moves and this says by how much. It is also the map LY-M1 needs —
//! it says what each formatting context buys, per chapter.
//!
//! # The three kinds
//!
//! - **[`Kind::Layout`]** — a `testharness` test carrying
//!   `data-expected-*` attributes, which state the used values a
//!   conforming engine must produce. These are **machine-readable
//!   layout expectations**: no rendering, no comparison image, just
//!   numbers. They are the first tests this track can run, and the
//!   count of them is the target the baseline moves toward.
//! - **[`Kind::Reftest`]** — a test with a `<link rel="match">` or
//!   `rel="mismatch">`. Running one needs a raster and a comparison,
//!   which is `hg-reftest-raster`'s job and then the graphics track's.
//! - **[`Kind::Harness`]** — a `testharness` test with no layout
//!   expectations, so it needs a script runtime and the DOM.
//!
//! Support files — references, images, stylesheets — are not tests and
//! are counted separately.

use std::path::{Path, PathBuf};

/// The pinned WPT commit.
pub const PIN: &str = "33926aca0f4313541f0ac5d67d4a4ab949d50b16";

/// The data set in `tools/test-data.toml`.
pub const DATA_SET: &str = "wpt-css2";

/// Where `css/CSS2` sits under the unpacked tree.
pub const SUITE_DIR: &str = "css/CSS2";

/// What a test needs to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// Layout expectations in `data-expected-*` attributes.
    Layout,
    /// A reference comparison.
    Reftest,
    /// A script runtime and the DOM.
    Harness,
    /// A **self-describing** test: it states in its own text what a
    /// conforming rendering looks like, and a person decides. The
    /// CSS 2.1 suite is largely these, and no runner can take them.
    Manual,
    /// Not a test: a reference, an image, a stylesheet.
    Support,
}

impl Kind {
    /// What it would take to run one.
    #[must_use]
    pub const fn needs(self) -> &'static str {
        match self {
            Self::Layout => "an HTML tree builder, the cascade and block layout",
            Self::Reftest => "the above, plus a raster and a reference comparison",
            Self::Harness => "the above, plus a script runtime and the DOM",
            Self::Manual => "a person",
            Self::Support => "nothing: it is not a test",
        }
    }

    /// Its name in a report.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Layout => "layout",
            Self::Reftest => "reftest",
            Self::Harness => "harness",
            Self::Manual => "manual",
            Self::Support => "support",
        }
    }
}

/// One file of the suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Its path, relative to `css/CSS2`.
    pub path: String,
    /// The chapter directory it sits in.
    pub chapter: String,
    /// What it needs.
    pub kind: Kind,
    /// How many `data-expected-*` attributes it carries.
    pub expectations: usize,
}

/// What the reference engine implements today, as the sections of
/// CSS 2.2 it was written from.
///
/// A list rather than a guess: each entry is a section this crate's
/// tests name. It is the other half of the baseline — a test is only
/// *reachable* when the sections it exercises are here.
pub const IMPLEMENTED_SECTIONS: [&str; 4] = [
    // The constraint equation for block-level non-replaced elements.
    "visudet.html#blockwidth",
    // min-width and max-width.
    "visudet.html#min-max-widths",
    // The automatic height of a block-level box.
    "visudet.html#normal-block",
    // Collapsing margins.
    "box.html#collapsing-margins",
];

/// Where the pinned suite lives.
///
/// # Errors
/// A message when neither `IZCAL_TEST_DATA` nor `HOME` is set.
pub fn suite_dir() -> Result<PathBuf, String> {
    let cache = match std::env::var_os("IZCAL_TEST_DATA") {
        Some(v) => PathBuf::from(v),
        None => {
            let home = std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .ok_or("neither IZCAL_TEST_DATA nor HOME is set")?;
            PathBuf::from(home).join(".cache/izcal/test-data")
        }
    };
    Ok(cache.join(format!("{DATA_SET}-{PIN}")).join(SUITE_DIR))
}

/// Whether a file name is a test rather than a support file.
fn is_test_name(name: &str) -> bool {
    let test_extension = name.ends_with(".html")
        || name.ends_with(".xht")
        || name.ends_with(".xhtml")
        || name.ends_with(".htm");
    if !test_extension {
        return false;
    }
    // WPT's own naming: a reference, a support file, or a file under
    // `support/` is not a test.
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
    !(stem.ends_with("-ref")
        || stem.ends_with("-notref")
        || stem.ends_with("-ref1")
        || stem.ends_with("-ref2"))
}

/// Classifies one file from its name and contents.
#[must_use]
pub fn classify(relative: &str, text: &str) -> Entry {
    // A file at the top level has no chapter directory of its own.
    let chapter = match relative.split_once('/') {
        Some((head, _)) => head.to_owned(),
        None => ".".to_owned(),
    };
    let name = relative.rsplit('/').next().unwrap_or(relative);
    let expectations = text.matches("data-expected-").count();

    // A support file: under a `support/` directory, a reference, or
    // simply not markup.
    let support =
        relative.contains("/support/") || relative.starts_with("support/") || !is_test_name(name);
    let kind = if support {
        Kind::Support
    } else if text.contains("rel=\"match\"")
        || text.contains("rel='match'")
        || text.contains("rel=\"mismatch\"")
        || text.contains("rel='mismatch'")
    {
        Kind::Reftest
    } else if expectations > 0 {
        Kind::Layout
    } else if text.contains("testharness.js") {
        Kind::Harness
    } else {
        // The CSS 2.1 suite's own manual tests carry the `flags`
        // metadata; anything with no reference and no harness is one.
        Kind::Manual
    };

    Entry {
        path: relative.to_owned(),
        chapter,
        kind,
        expectations,
    }
}

/// Reads and classifies every file of the suite.
///
/// # Errors
/// A message naming the directory when it cannot be read.
pub fn read_suite(dir: &Path) -> Result<Vec<Entry>, String> {
    if !dir.is_dir() {
        return Err(format!(
            "{} is not there; run `tools/fetch-test-data.sh {DATA_SET}`",
            dir.display()
        ));
    }
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(at) = stack.pop() {
        let entries = std::fs::read_dir(&at).map_err(|e| format!("{}: {e}", at.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", at.display()))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Ok(relative) = path.strip_prefix(dir) else {
                continue;
            };
            let relative = relative.to_string_lossy().replace('\\', "/");
            // Only the markup is read; an image's bytes say nothing
            // about what it needs.
            let text = if relative.ends_with(".html")
                || relative.ends_with(".htm")
                || relative.ends_with(".xht")
                || relative.ends_with(".xhtml")
            {
                std::fs::read_to_string(&path).unwrap_or_default()
            } else {
                String::new()
            };
            out.push(classify(&relative, &text));
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// What a run measured.
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    /// Every file.
    pub files: usize,
    /// Files by kind.
    pub by_kind: std::collections::BTreeMap<&'static str, usize>,
    /// Tests by chapter, support files excluded.
    pub by_chapter: std::collections::BTreeMap<String, usize>,
    /// How many `data-expected-*` attributes the suite holds in all.
    pub expectations: usize,
    /// How many tests this engine can run today.
    pub runnable: usize,
}

/// Counts a classified suite.
#[must_use]
pub fn measure(entries: &[Entry]) -> Ledger {
    let mut l = Ledger {
        files: entries.len(),
        ..Ledger::default()
    };
    for e in entries {
        *l.by_kind.entry(e.kind.name()).or_default() += 1;
        l.expectations = l.expectations.saturating_add(e.expectations);
        if !matches!(e.kind, Kind::Support) {
            *l.by_chapter.entry(e.chapter.clone()).or_default() += 1;
        }
    }
    // Nothing is runnable yet: every kind needs an HTML tree builder,
    // which is ST-M1's. The number is computed rather than written
    // down, so it moves on its own when the component lands.
    l.runnable = entries
        .iter()
        .filter(|e| matches!(e.kind, Kind::Layout) && HAS_TREE_BUILDER)
        .count();
    l
}

/// Whether an HTML tree builder exists to parse a test with.
///
/// `false` at LY-M0: `hg-html-syntax` has the tokenizer, and the tree
/// builder is ST-M1's. This constant is what the runnable count turns
/// on, so flipping it is the whole of "the number went up".
pub const HAS_TREE_BUILDER: bool = false;

/// The baseline: how many tests this engine runs.
///
/// **Zero**, and the report says why for each kind. A baseline that
/// can only go up is the honest evidence for a track whose engine
/// does not exist yet; what would not be honest is a pass rate over
/// the tests that happen to need nothing.
///
/// A zero baseline guards nothing on its own, so what the CI job
/// actually checks is [`check`]: that the suite still classifies as
/// recorded. A change in any of those counts is either the pin moving
/// or the classifier drifting, and both are things to look at.
pub const RUNNABLE_BASELINE: usize = 0;

/// What a recorded count disagrees about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    /// Which count.
    pub what: &'static str,
    /// What was recorded.
    pub recorded: usize,
    /// What was measured.
    pub measured: usize,
}

/// Checks a measurement against the recorded counts.
///
/// Returns every disagreement, empty when the suite is as recorded.
#[must_use]
pub fn check(l: &Ledger) -> Vec<Drift> {
    let kind = |name: &str| l.by_kind.get(name).copied().unwrap_or(0);
    [
        ("files", FILES, l.files),
        ("layout", LAYOUT_TESTS, kind("layout")),
        ("reftest", REFTESTS, kind("reftest")),
        ("harness", HARNESS, kind("harness")),
        ("manual", MANUAL, kind("manual")),
        ("support", SUPPORT, kind("support")),
        ("runnable", RUNNABLE_BASELINE, l.runnable),
    ]
    .into_iter()
    .filter(|(_, recorded, measured)| recorded != measured)
    .map(|(what, recorded, measured)| Drift {
        what,
        recorded,
        measured,
    })
    .collect()
}

/// How many files the pinned suite holds, measured.
pub const FILES: usize = 12_955;

/// How many of them are reference comparisons.
pub const REFTESTS: usize = 6_235;

/// How many are self-describing, so a person decides.
pub const MANUAL: usize = 2_930;

/// How many are `testharness` tests with no layout expectations.
pub const HARNESS: usize = 66;

/// How many are support files rather than tests.
pub const SUPPORT: usize = 3_711;

/// How many of them carry machine-readable layout expectations.
///
/// The first tests this track will be able to run, and the number the
/// baseline moves toward when the tree builder lands.
pub const LAYOUT_TESTS: usize = 13;
