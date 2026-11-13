//! Reading the pinned Unicode Character Database.
//!
//! Every file here is read as the Unicode Standard's prose describes its
//! format (UAX #44, Section 4.2 for `UnicodeData.txt`'s fields; UAX #44
//! Section 4.2.1 for the `@missing` and range conventions; UTS #46
//! Section 5 for `IdnaMappingTable.txt`). No third-party source is read
//! (M-1 part 3).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One code point's canonical or compatibility decomposition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decomposition {
    /// `true` when `UnicodeData.txt` gave it a `<tag>`, which makes it a
    /// compatibility mapping rather than a canonical one.
    pub compat: bool,
    /// The mapping, as written in the database (one level only).
    pub to: Vec<char>,
}

/// What the database says about every code point we need.
#[derive(Debug, Default)]
pub struct Ucd {
    /// Canonical combining class, for the code points whose class is not
    /// zero.
    pub ccc: BTreeMap<u32, u8>,
    /// One-level decomposition mappings.
    pub decomp: BTreeMap<u32, Decomposition>,
    /// `Full_Composition_Exclusion`: the code points a canonical
    /// composition may never produce (UAX #44).
    pub full_exclusion: BTreeSet<u32>,
    /// Quick-check values that are not `Yes`, as `(form, value)` where
    /// form is 0 NFC, 1 NFD, 2 NFKC, 3 NFKD and value is `N` or `M`.
    pub quick_check: BTreeMap<u32, [QuickCheck; 4]>,
    /// UTS #46's status and mapping per code point range.
    pub idna: Vec<IdnaRow>,
    /// `General_Category`, for the code points that are a **mark**
    /// (`Mn`, `Mc`, `Me`). UTS #46's validity criteria need no other
    /// category: "the label must not begin with a combining mark".
    pub marks: Vec<(u32, u32)>,
    /// `Bidi_Class`, for every code point whose class is not `L`.
    /// RFC 5893's six conditions are written in these classes, and `L`
    /// is both the default and the commonest, so leaving it out halves
    /// the table.
    pub bidi: Vec<(u32, u32, BidiClass)>,
    /// `Joining_Type`, for every code point whose type is not `U`.
    /// RFC 5892's ContextJ rule for ZWNJ is written in these.
    pub joining: Vec<(u32, u32, JoiningType)>,
    /// The version string from `ReadMe.txt`, for the generated header.
    pub version: String,
}

/// A quick-check value (UAX #15, Table 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuickCheck {
    /// The code point never changes under the form.
    #[default]
    Yes,
    /// It may change, depending on its context.
    Maybe,
    /// It always changes.
    No,
}

/// A `Bidi_Class` value, as far as RFC 5893 distinguishes them.
///
/// Its six conditions name `L`, `R`, `AL`, `AN`, `EN`, `ES`, `CS`,
/// `ET`, `ON`, `BN` and `NSM` and nothing else, so everything else is
/// one value: a label holding one fails the conditions whatever it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BidiClass {
    /// Left-to-right.
    L,
    /// Right-to-left.
    R,
    /// Right-to-left Arabic.
    Al,
    /// Arabic number.
    An,
    /// European number.
    En,
    /// European separator.
    Es,
    /// Common separator.
    Cs,
    /// European terminator.
    Et,
    /// Other neutral.
    On,
    /// Boundary neutral.
    Bn,
    /// Non-spacing mark.
    Nsm,
    /// Anything RFC 5893 does not name.
    Other,
}

impl BidiClass {
    /// The UCD's abbreviation, read.
    #[must_use]
    pub fn from_abbr(s: &str) -> Self {
        match s {
            "L" => Self::L,
            "R" => Self::R,
            "AL" => Self::Al,
            "AN" => Self::An,
            "EN" => Self::En,
            "ES" => Self::Es,
            "CS" => Self::Cs,
            "ET" => Self::Et,
            "ON" => Self::On,
            "BN" => Self::Bn,
            "NSM" => Self::Nsm,
            _ => Self::Other,
        }
    }

    /// Its name in the generated table.
    #[must_use]
    pub const fn variant(self) -> &'static str {
        match self {
            Self::L => "L",
            Self::R => "R",
            Self::Al => "Al",
            Self::An => "An",
            Self::En => "En",
            Self::Es => "Es",
            Self::Cs => "Cs",
            Self::Et => "Et",
            Self::On => "On",
            Self::Bn => "Bn",
            Self::Nsm => "Nsm",
            Self::Other => "Other",
        }
    }
}

/// A `Joining_Type` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JoiningType {
    /// Join causing.
    C,
    /// Dual joining.
    D,
    /// Right joining.
    R,
    /// Left joining.
    L,
    /// Transparent.
    T,
    /// Non joining, the default.
    U,
}

impl JoiningType {
    /// The UCD's abbreviation, read.
    #[must_use]
    pub fn from_abbr(s: &str) -> Self {
        match s {
            "C" => Self::C,
            "D" => Self::D,
            "R" => Self::R,
            "L" => Self::L,
            "T" => Self::T,
            _ => Self::U,
        }
    }

    /// Its name in the generated table.
    #[must_use]
    pub const fn variant(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::D => "D",
            Self::R => "R",
            Self::L => "L",
            Self::T => "T",
            Self::U => "U",
        }
    }
}

/// One row of `IdnaMappingTable.txt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdnaRow {
    /// First code point of the range.
    pub first: u32,
    /// Last code point of the range, inclusive.
    pub last: u32,
    /// The status word, as written.
    pub status: String,
    /// The mapping, where the status has one.
    pub mapping: Vec<char>,
}

/// Strips a `#` comment and surrounding space.
fn content(line: &str) -> &str {
    line.split('#').next().unwrap_or("").trim()
}

/// Parses `XXXX` or `XXXX..YYYY` as an inclusive range.
fn range(field: &str) -> Option<(u32, u32)> {
    let f = field.trim();
    match f.split_once("..") {
        Some((a, b)) => Some((
            u32::from_str_radix(a.trim(), 16).ok()?,
            u32::from_str_radix(b.trim(), 16).ok()?,
        )),
        None => {
            let c = u32::from_str_radix(f, 16).ok()?;
            Some((c, c))
        }
    }
}

/// Parses a space-separated list of hexadecimal code points.
fn chars(field: &str) -> Vec<char> {
    field
        .split_whitespace()
        .filter_map(|t| u32::from_str_radix(t, 16).ok())
        .filter_map(char::from_u32)
        .collect()
}

fn read(dir: &Path, rel: &str) -> Result<String, String> {
    let p = dir.join(rel);
    std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))
}

impl Ucd {
    /// Reads every file this generator needs.
    ///
    /// `ucd` is the unpacked `UCD.zip` tree and `idna` the directory
    /// holding `IdnaMappingTable.txt`.
    ///
    /// # Errors
    /// A message naming the file when one cannot be read or parsed.
    pub fn read(ucd: &Path, idna: &Path) -> Result<Self, String> {
        let mut u = Self {
            version: Self::version(ucd)?,
            ..Self::default()
        };
        u.read_unicode_data(&read(ucd, "UnicodeData.txt")?)?;
        u.read_derived_props(&read(ucd, "DerivedNormalizationProps.txt")?);
        u.read_idna(&read(idna, "IdnaMappingTable.txt")?);
        u.read_marks(&read(ucd, "extracted/DerivedGeneralCategory.txt")?);
        u.read_bidi(&read(ucd, "extracted/DerivedBidiClass.txt")?);
        u.read_joining(&read(ucd, "extracted/DerivedJoiningType.txt")?);
        if u.decomp.is_empty() || u.ccc.is_empty() || u.full_exclusion.is_empty() {
            return Err("the UCD parsed to empty tables; is the tree the right one?".into());
        }
        Ok(u)
    }

    /// The version line of the UCD's `ReadMe.txt`.
    fn version(ucd: &Path) -> Result<String, String> {
        let text = read(ucd, "ReadMe.txt")?;
        for line in text.lines() {
            if let Some(at) = line.find("version ") {
                let rest = line
                    .get(at.saturating_add("version ".len())..)
                    .unwrap_or("");
                let v: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if v.split('.').count() == 3 {
                    return Ok(v);
                }
            }
        }
        Err("no version line in the UCD's ReadMe.txt".into())
    }

    /// `UnicodeData.txt`: field 3 is the combining class and field 5 the
    /// decomposition mapping (UAX #44 Section 4.2). Ranges are written as
    /// a pair of lines whose names end `, First>` and `, Last>`; neither
    /// kind of range carries a decomposition, and the CJK and Hangul
    /// ranges have combining class zero, so a range needs no expansion
    /// here.
    fn read_unicode_data(&mut self, text: &str) -> Result<(), String> {
        for line in text.lines() {
            let f: Vec<&str> = line.split(';').collect();
            let (Some(code), Some(name), Some(ccc), Some(decomp)) =
                (f.first(), f.get(1), f.get(3), f.get(5))
            else {
                continue;
            };
            let Ok(cp) = u32::from_str_radix(code.trim(), 16) else {
                continue;
            };
            if let Ok(c) = ccc.trim().parse::<u8>()
                && c != 0
            {
                self.ccc.insert(cp, c);
            }
            let d = decomp.trim();
            if d.is_empty() {
                continue;
            }
            if name.ends_with(", First>") || name.ends_with(", Last>") {
                return Err(format!("{code}: a range line carries a decomposition"));
            }
            let compat = d.starts_with('<');
            let body = if compat {
                d.split_once('>').map_or("", |(_, rest)| rest)
            } else {
                d
            };
            let to = chars(body);
            if to.is_empty() {
                return Err(format!("{code}: empty decomposition mapping"));
            }
            self.decomp.insert(cp, Decomposition { compat, to });
        }
        Ok(())
    }

    /// `DerivedNormalizationProps.txt`: `range ; property [; value]`.
    fn read_derived_props(&mut self, text: &str) {
        for line in text.lines() {
            let c = content(line);
            if c.is_empty() {
                continue;
            }
            let f: Vec<&str> = c.split(';').collect();
            let (Some(r), Some(prop)) = (f.first(), f.get(1)) else {
                continue;
            };
            let Some((first, last)) = range(r) else {
                continue;
            };
            let prop = prop.trim();
            if prop == "Full_Composition_Exclusion" {
                for cp in first..=last {
                    self.full_exclusion.insert(cp);
                }
                continue;
            }
            let form = match prop {
                "NFC_QC" => 0_usize,
                "NFD_QC" => 1,
                "NFKC_QC" => 2,
                "NFKD_QC" => 3,
                _ => continue,
            };
            let value = match f.get(2).map(|v| v.trim()) {
                Some("N") => QuickCheck::No,
                Some("M") => QuickCheck::Maybe,
                _ => continue,
            };
            for cp in first..=last {
                let e = self.quick_check.entry(cp).or_default();
                if let Some(slot) = e.get_mut(form) {
                    *slot = value;
                }
            }
        }
    }

    /// `extracted/DerivedGeneralCategory.txt`, keeping only the marks.
    fn read_marks(&mut self, text: &str) {
        for line in text.lines() {
            let c = content(line);
            let Some((span, rest)) = c.split_once(';') else {
                continue;
            };
            if !matches!(rest.trim(), "Mn" | "Mc" | "Me") {
                continue;
            }
            if let Some(r) = range(span) {
                self.marks.push(r);
            }
        }
        self.marks.sort_unstable();
    }

    /// `extracted/DerivedBidiClass.txt`, leaving out `L`.
    fn read_bidi(&mut self, text: &str) {
        for line in text.lines() {
            let c = content(line);
            let Some((span, rest)) = c.split_once(';') else {
                continue;
            };
            let class = BidiClass::from_abbr(rest.trim());
            if class == BidiClass::L {
                continue;
            }
            if let Some((a, b)) = range(span) {
                self.bidi.push((a, b, class));
            }
        }
        self.bidi.sort_unstable();
    }

    /// `extracted/DerivedJoiningType.txt`, leaving out `U`.
    fn read_joining(&mut self, text: &str) {
        for line in text.lines() {
            let c = content(line);
            let Some((span, rest)) = c.split_once(';') else {
                continue;
            };
            let ty = JoiningType::from_abbr(rest.trim());
            if ty == JoiningType::U {
                continue;
            }
            if let Some((a, b)) = range(span) {
                self.joining.push((a, b, ty));
            }
        }
        self.joining.sort_unstable();
    }

    /// `IdnaMappingTable.txt`: `range ; status [; mapping [; idna2008]]`
    /// (UTS #46 Section 5).
    fn read_idna(&mut self, text: &str) {
        for line in text.lines() {
            let c = content(line);
            if c.is_empty() {
                continue;
            }
            let f: Vec<&str> = c.split(';').collect();
            let (Some(r), Some(status)) = (f.first(), f.get(1)) else {
                continue;
            };
            let Some((first, last)) = range(r) else {
                continue;
            };
            self.idna.push(IdnaRow {
                first,
                last,
                status: status.trim().to_owned(),
                mapping: f.get(2).map(|m| chars(m)).unwrap_or_default(),
            });
        }
        self.idna.sort_by_key(|r| r.first);
    }

    /// The canonical decomposition of `cp`, applied until no character
    /// decomposes further (UAX #15: the mappings are applied
    /// recursively).
    #[must_use]
    pub fn canonical(&self, cp: u32) -> Vec<char> {
        let mut out = Vec::new();
        self.expand(cp, false, &mut out);
        out
    }

    /// The compatibility decomposition of `cp`, applied recursively.
    #[must_use]
    pub fn compatibility(&self, cp: u32) -> Vec<char> {
        let mut out = Vec::new();
        self.expand(cp, true, &mut out);
        out
    }

    fn expand(&self, cp: u32, compat: bool, out: &mut Vec<char>) {
        match self.decomp.get(&cp) {
            Some(d) if compat || !d.compat => {
                for c in &d.to {
                    self.expand(u32::from(*c), compat, out);
                }
            }
            _ => {
                if let Some(c) = char::from_u32(cp) {
                    out.push(c);
                }
            }
        }
    }

    /// The primary composites: `(first, second) -> composite` for every
    /// canonical decomposition of exactly two characters whose composite
    /// is not a `Full_Composition_Exclusion` (UAX #15, Section 15).
    #[must_use]
    pub fn compositions(&self) -> BTreeMap<(u32, u32), u32> {
        let mut m = BTreeMap::new();
        for (cp, d) in &self.decomp {
            if d.compat || self.full_exclusion.contains(cp) {
                continue;
            }
            if let [a, b] = d.to.as_slice() {
                m.insert((u32::from(*a), u32::from(*b)), *cp);
            }
        }
        m
    }
}
