//! The gate files the lint reads: `tools/tiers.toml`,
//! `tools/graph-rules.toml` and `unsafe-inventory.toml`. Each is read
//! strictly: an unknown key or a value of the wrong type is an error, so
//! a misspelt rule cannot switch itself off.

use std::collections::BTreeMap;
use std::path::Path;

use crate::metadata::Features;
use crate::toml;
use crate::value::Value;

/// A trust tier (coding standard 6.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// Tools and test infrastructure.
    T,
    /// Per-site processes.
    R,
    /// Shared processes.
    P,
    /// The OS substrate (`hg-substrate-os`), held to tier P's panic rule.
    O,
}

impl Tier {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "T" => Some(Self::T),
            "R" => Some(Self::R),
            "P" => Some(Self::P),
            "O" => Some(Self::O),
            _ => None,
        }
    }

    /// How strict the tier is. O is as strict as P (6.2: "Panics: as
    /// tier P"), so either satisfies the other.
    pub fn strictness(self) -> u8 {
        match self {
            Self::T => 0,
            Self::R => 1,
            Self::P | Self::O => 2,
        }
    }

    /// The tier's letter.
    pub fn letter(self) -> &'static str {
        match self {
            Self::T => "T",
            Self::R => "R",
            Self::P => "P",
            Self::O => "O",
        }
    }
}

/// One crate's row in `tools/tiers.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declared {
    /// Its tier.
    pub tier: Tier,
    /// Its layer (Stage 1 design 2.1), e.g. `L1`, `L2s` or `tools`.
    pub layer: String,
}

/// A configuration error: which file, and what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

/// Reads and parses a TOML file.
///
/// # Errors
/// [`ConfigError`] if it cannot be read or parsed.
pub fn read_toml(path: &Path) -> Result<Value, ConfigError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| ConfigError(format!("{}: {e}", path.display())))?;
    toml::parse(&text).map_err(|e| ConfigError(format!("{}: {e}", path.display())))
}

/// A view of one table that remembers which keys were read, so the
/// caller can reject the rest.
struct Strict<'v> {
    what: String,
    table: &'v BTreeMap<String, Value>,
    known: Vec<&'static str>,
}

impl<'v> Strict<'v> {
    fn new(what: impl Into<String>, v: &'v Value) -> Result<Self, ConfigError> {
        let what = what.into();
        let table = v
            .as_table()
            .ok_or_else(|| ConfigError(format!("{what}: expected a table, found {}", v.kind())))?;
        Ok(Self {
            what,
            table,
            known: Vec::new(),
        })
    }

    fn opt(&mut self, key: &'static str) -> Option<&'v Value> {
        self.known.push(key);
        self.table.get(key)
    }

    fn req(&mut self, key: &'static str) -> Result<&'v Value, ConfigError> {
        let what = self.what.clone();
        self.opt(key)
            .ok_or_else(|| ConfigError(format!("{what}: missing key {key:?}")))
    }

    fn string(&mut self, key: &'static str) -> Result<String, ConfigError> {
        let v = self.req(key)?;
        v.as_str()
            .map(str::to_owned)
            .ok_or_else(|| ConfigError(format!("{}.{key}: expected a string", self.what)))
    }

    fn opt_string(&mut self, key: &'static str) -> Result<Option<String>, ConfigError> {
        match self.opt(key) {
            None => Ok(None),
            Some(v) => v
                .as_str()
                .map(|s| Some(s.to_owned()))
                .ok_or_else(|| ConfigError(format!("{}.{key}: expected a string", self.what))),
        }
    }

    fn strings(&mut self, key: &'static str) -> Result<Vec<String>, ConfigError> {
        let what = self.what.clone();
        match self.opt(key) {
            None => Ok(Vec::new()),
            Some(v) => string_list(&format!("{what}.{key}"), v),
        }
    }

    fn bool(&mut self, key: &'static str) -> Result<bool, ConfigError> {
        match self.opt(key) {
            None => Ok(false),
            Some(v) => v
                .as_bool()
                .ok_or_else(|| ConfigError(format!("{}.{key}: expected a boolean", self.what))),
        }
    }

    fn finish(self) -> Result<(), ConfigError> {
        for k in self.table.keys() {
            if !self.known.contains(&k.as_str()) {
                return Err(ConfigError(format!("{}: unknown key {k:?}", self.what)));
            }
        }
        Ok(())
    }
}

fn string_list(what: &str, v: &Value) -> Result<Vec<String>, ConfigError> {
    v.as_array()
        .ok_or_else(|| ConfigError(format!("{what}: expected an array of strings")))?
        .iter()
        .map(|s| {
            s.as_str()
                .map(str::to_owned)
                .ok_or_else(|| ConfigError(format!("{what}: expected an array of strings")))
        })
        .collect()
}

/// `tools/tiers.toml`: every workspace crate's tier and layer.
#[derive(Debug, Clone, Default)]
pub struct Tiers {
    /// Rows by crate name.
    pub crates: BTreeMap<String, Declared>,
}

impl Tiers {
    /// Reads `tools/tiers.toml`.
    ///
    /// # Errors
    /// [`ConfigError`] for a malformed file.
    pub fn from_value(v: &Value) -> Result<Self, ConfigError> {
        let mut top = Strict::new("tiers.toml", v)?;
        let mut crates = BTreeMap::new();
        if let Some(c) = top.opt("crates") {
            let table = c
                .as_table()
                .ok_or_else(|| ConfigError("tiers.toml: [crates] must be a table".into()))?;
            for (name, row) in table {
                let mut r = Strict::new(format!("tiers.toml [crates.{name}]"), row)?;
                let tier_s = r.string("tier")?;
                let tier = Tier::parse(&tier_s).ok_or_else(|| {
                    ConfigError(format!(
                        "tiers.toml [crates.{name}]: unknown tier {tier_s:?}"
                    ))
                })?;
                let layer = r.string("layer")?;
                r.opt_string("why")?;
                r.finish()?;
                crates.insert(name.clone(), Declared { tier, layer });
            }
        }
        top.finish()?;
        Ok(Self { crates })
    }
}

/// How far a [`Forbid`] rule looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// Direct normal and build dependencies (what the crate can name).
    Direct,
    /// Everything the crate's artifacts link (normal edges, transitive).
    Linked,
}

/// A `[[forbid]]` rule: packages matching `from` must not depend on
/// packages matching `crates` (or in `layers`).
#[derive(Debug, Clone)]
pub struct Forbid {
    /// The rule's ID, e.g. `G260`.
    pub rule: String,
    /// Globs for the dependents.
    pub from: Vec<String>,
    /// Dependents exempt from the rule.
    pub from_except: Vec<String>,
    /// Globs for the forbidden dependencies.
    pub crates: Vec<String>,
    /// Layers whose crates are forbidden.
    pub layers: Vec<String>,
    /// Dependencies exempt from the rule.
    pub except: Vec<String>,
    /// Direct or transitive.
    pub reach: Reach,
    /// Which resolution the rule reads.
    pub features: Features,
    /// For the report.
    pub why: String,
}

/// An `[[only-linked-by]]` rule: only the named binaries may link the
/// crates, and optionally only the named crates may depend on them.
#[derive(Debug, Clone)]
pub struct OnlyLinkedBy {
    /// The rule's ID.
    pub rule: String,
    /// Globs for the guarded crates.
    pub crates: Vec<String>,
    /// Packages whose binaries may link them.
    pub binaries: Vec<String>,
    /// If set, the only packages that may depend on them directly.
    pub direct_dependents: Option<Vec<String>>,
    /// For the report.
    pub why: String,
}

/// A `[[direct-only]]` rule: the crate's direct normal and build
/// dependencies are a subset of `allow`.
#[derive(Debug, Clone)]
pub struct DirectOnly {
    /// The rule's ID.
    pub rule: String,
    /// Globs for the crates bound by it.
    pub crates: Vec<String>,
    /// Globs for the allowed dependencies.
    pub allow: Vec<String>,
    /// The tier the crate must declare, if any.
    pub tier: Option<Tier>,
    /// Whether its library root must be `#![no_std]`.
    pub no_std: bool,
    /// For the report.
    pub why: String,
}

/// A `[[disjoint]]` rule: two crate sets link no common workspace crate
/// except the allowed ones.
#[derive(Debug, Clone)]
pub struct Disjoint {
    /// The rule's ID.
    pub rule: String,
    /// One side.
    pub a: Vec<String>,
    /// The other side.
    pub b: Vec<String>,
    /// Shared crates that are allowed.
    pub allow: Vec<String>,
    /// For the report.
    pub why: String,
}

/// A layer exception: crates of `from_layer` may depend on `to`.
#[derive(Debug, Clone)]
pub struct LayerException {
    /// The dependent's layer.
    pub from_layer: String,
    /// The dependency.
    pub to: String,
}

/// `tools/graph-rules.toml`.
#[derive(Debug, Clone, Default)]
pub struct Rules {
    /// Layer name to rank: a crate may depend on equal or lower ranks.
    pub layer_rank: BTreeMap<String, usize>,
    /// Layer name to the highest rank it may depend on, where lower than
    /// its own (Stage 1 2.1: L3s depends only on L2).
    pub layer_cap: BTreeMap<String, usize>,
    /// Reviewed exceptions to layer order.
    pub layer_exceptions: Vec<LayerException>,
    /// Glob for shipping binaries.
    pub shipping: String,
    /// The simulator crate (Q-CS-1).
    pub simulator: String,
    /// Packages whose binaries may link the simulator.
    pub sim_linkers: Vec<String>,
    /// Names of the cryptographic-randomness trait (code and spec).
    pub crypto_traits: Vec<String>,
    /// The only crates that may implement it outside test code.
    pub crypto_implementers: Vec<String>,
    /// Ambient-effect crates reachable only through `ambient_via`.
    pub ambient_crates: Vec<String>,
    /// The crates that may link ambient-effect crates.
    pub ambient_via: Vec<String>,
    /// Workspace crates exempt as roots (the lint canary).
    pub ambient_exempt: Vec<String>,
    /// Crates only tier T may link.
    pub tier_t_only: Vec<String>,
    /// Procedural macros allowed anywhere they are not banned.
    pub proc_macro_allow: Vec<String>,
    /// Layers (globs) that may use no procedural macro.
    pub proc_macro_banned_layers: Vec<String>,
    /// Crates those layers may not link either (e.g. `syn`).
    pub proc_macro_banned_crates: Vec<String>,
    /// The directory every crate sits in, directly (flat layout).
    pub layout_dir: String,
    /// Required name prefix.
    pub layout_prefix: String,
    /// Required version.
    pub layout_version: String,
    /// Members exempt from the layout rules (e.g. `xtask`).
    pub layout_exempt: Vec<String>,
    /// `[[forbid]]` rules.
    pub forbid: Vec<Forbid>,
    /// `[[only-linked-by]]` rules.
    pub only_linked_by: Vec<OnlyLinkedBy>,
    /// `[[direct-only]]` rules.
    pub direct_only: Vec<DirectOnly>,
    /// `[[disjoint]]` rules.
    pub disjoint: Vec<Disjoint>,
}

fn table_list<'v>(what: &str, v: Option<&'v Value>) -> Result<Vec<&'v Value>, ConfigError> {
    match v {
        None => Ok(Vec::new()),
        Some(Value::Array(a)) => Ok(a.iter().collect()),
        Some(_) => Err(ConfigError(format!(
            "graph-rules.toml: {what} must be [[{what}]] tables"
        ))),
    }
}

impl Rules {
    /// Reads `tools/graph-rules.toml`.
    ///
    /// # Errors
    /// [`ConfigError`] for a malformed file.
    pub fn from_value(v: &Value) -> Result<Self, ConfigError> {
        let mut top = Strict::new("graph-rules.toml", v)?;
        let mut r = Self::default();

        let mut layers = Strict::new("[layers]", top.req("layers")?)?;
        let order = layers
            .req("order")?
            .as_array()
            .ok_or_else(|| ConfigError("[layers] order: expected an array of arrays".into()))?;
        for (rank, group) in order.iter().enumerate() {
            for name in string_list("[layers] order", group)? {
                if r.layer_rank.insert(name.clone(), rank).is_some() {
                    return Err(ConfigError(format!("[layers] order: {name} listed twice")));
                }
            }
        }
        for cap in table_list("layers.cap", layers.opt("cap"))? {
            let mut c = Strict::new("[[layers.cap]]", cap)?;
            let layer = c.string("layer")?;
            let max = c.string("max")?;
            c.opt_string("why")?;
            c.finish()?;
            let rank = *r
                .layer_rank
                .get(&max)
                .ok_or_else(|| ConfigError(format!("[[layers.cap]]: unknown layer {max}")))?;
            r.layer_cap.insert(layer, rank);
        }
        for ex in table_list("layers.exception", layers.opt("exception"))? {
            let mut e = Strict::new("[[layers.exception]]", ex)?;
            r.layer_exceptions.push(LayerException {
                from_layer: e.string("from-layer")?,
                to: e.string("to")?,
            });
            e.string("why")?;
            e.finish()?;
        }
        layers.finish()?;

        let mut s = Strict::new("[shipping]", top.req("shipping")?)?;
        r.shipping = s.string("binaries")?;
        s.opt_string("why")?;
        s.finish()?;

        let mut q = Strict::new("[q-cs-1]", top.req("q-cs-1")?)?;
        r.simulator = q.string("simulator")?;
        r.sim_linkers = q.strings("linkers")?;
        r.crypto_traits = q.strings("crypto-traits")?;
        r.crypto_implementers = q.strings("crypto-implementers")?;
        q.opt_string("why")?;
        q.finish()?;

        let mut a = Strict::new("[ambient]", top.req("ambient")?)?;
        r.ambient_crates = a.strings("crates")?;
        r.ambient_via = a.strings("via")?;
        r.ambient_exempt = a.strings("exempt")?;
        a.opt_string("why")?;
        a.finish()?;

        let mut t = Strict::new("[tier-t-only]", top.req("tier-t-only")?)?;
        r.tier_t_only = t.strings("crates")?;
        t.opt_string("why")?;
        t.finish()?;

        let mut p = Strict::new("[proc-macros]", top.req("proc-macros")?)?;
        r.proc_macro_allow = p.strings("allow")?;
        r.proc_macro_banned_layers = p.strings("banned-in-layers")?;
        r.proc_macro_banned_crates = p.strings("banned-crates")?;
        p.opt_string("why")?;
        p.finish()?;

        let mut l = Strict::new("[layout]", top.req("layout")?)?;
        r.layout_dir = l.string("dir")?;
        r.layout_prefix = l.string("prefix")?;
        r.layout_version = l.string("version")?;
        r.layout_exempt = l.strings("exempt")?;
        l.opt_string("why")?;
        l.finish()?;

        for f in table_list("forbid", top.opt("forbid"))? {
            let mut f = Strict::new("[[forbid]]", f)?;
            let rule = f.string("rule")?;
            let reach = match f.opt_string("reach")?.as_deref() {
                None | Some("linked") => Reach::Linked,
                Some("direct") => Reach::Direct,
                Some(other) => {
                    return Err(ConfigError(format!(
                        "[[forbid]] {rule}: unknown reach {other:?}"
                    )));
                }
            };
            let features = match f.opt_string("features")?.as_deref() {
                None | Some("all") => Features::All,
                Some("default") => Features::Default,
                Some(other) => {
                    return Err(ConfigError(format!(
                        "[[forbid]] {rule}: unknown features {other:?}"
                    )));
                }
            };
            let rule_out = Forbid {
                from: f.strings("from")?,
                from_except: f.strings("from-except")?,
                crates: f.strings("crates")?,
                layers: f.strings("layers")?,
                except: f.strings("except")?,
                reach,
                features,
                why: f.string("why")?,
                rule,
            };
            f.finish()?;
            r.forbid.push(rule_out);
        }
        for o in table_list("only-linked-by", top.opt("only-linked-by"))? {
            let mut o = Strict::new("[[only-linked-by]]", o)?;
            let direct = if o.opt("direct-dependents").is_some() {
                Some(o.strings("direct-dependents")?)
            } else {
                None
            };
            let rule_out = OnlyLinkedBy {
                rule: o.string("rule")?,
                crates: o.strings("crates")?,
                binaries: o.strings("binaries")?,
                direct_dependents: direct,
                why: o.string("why")?,
            };
            o.finish()?;
            r.only_linked_by.push(rule_out);
        }
        for d in table_list("direct-only", top.opt("direct-only"))? {
            let mut d = Strict::new("[[direct-only]]", d)?;
            let rule = d.string("rule")?;
            let tier = match d.opt_string("tier")? {
                None => None,
                Some(t) => Some(Tier::parse(&t).ok_or_else(|| {
                    ConfigError(format!("[[direct-only]] {rule}: unknown tier {t:?}"))
                })?),
            };
            let rule_out = DirectOnly {
                crates: d.strings("crates")?,
                allow: d.strings("allow")?,
                tier,
                no_std: d.bool("no-std")?,
                why: d.string("why")?,
                rule,
            };
            d.finish()?;
            r.direct_only.push(rule_out);
        }
        for d in table_list("disjoint", top.opt("disjoint"))? {
            let mut d = Strict::new("[[disjoint]]", d)?;
            let rule_out = Disjoint {
                rule: d.string("rule")?,
                a: d.strings("a")?,
                b: d.strings("b")?,
                allow: d.strings("allow")?,
                why: d.string("why")?,
            };
            d.finish()?;
            r.disjoint.push(rule_out);
        }
        top.finish()?;
        for layer in r.layer_cap.keys() {
            if !r.layer_rank.contains_key(layer) {
                return Err(ConfigError(format!(
                    "[[layers.cap]]: unknown layer {layer}"
                )));
            }
        }
        Ok(r)
    }
}

/// `unsafe-inventory.toml`: per crate, the modules (or bin files) that
/// may contain `unsafe` (G5, CS-TY-13).
#[derive(Debug, Clone, Default)]
pub struct Inventory {
    /// Entries by crate name.
    pub crates: BTreeMap<String, Vec<String>>,
}

impl Inventory {
    /// Reads `unsafe-inventory.toml`.
    ///
    /// # Errors
    /// [`ConfigError`] for a malformed file.
    pub fn from_value(v: &Value) -> Result<Self, ConfigError> {
        let mut top = Strict::new("unsafe-inventory.toml", v)?;
        let mut crates = BTreeMap::new();
        if let Some(c) = top.opt("crates") {
            let table = c.as_table().ok_or_else(|| {
                ConfigError("unsafe-inventory.toml: [crates] must be a table".into())
            })?;
            for (name, row) in table {
                let mut r = Strict::new(format!("unsafe-inventory.toml [crates.{name}]"), row)?;
                let entries = r.strings("entries")?;
                r.opt_string("owner")?;
                r.finish()?;
                crates.insert(name.clone(), entries);
            }
        }
        top.finish()?;
        Ok(Self { crates })
    }
}

/// Whether `name` matches `pattern`, where `*` matches any run of
/// characters (including none) and everything else matches itself.
pub fn glob(pattern: &str, name: &str) -> bool {
    let p = pattern.as_bytes();
    let n = name.as_bytes();
    let (mut pi, mut ni) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while ni < n.len() {
        if pi < p.len() && p[pi] == b'*' {
            star = Some((pi, ni));
            pi += 1;
        } else if pi < p.len() && p[pi] == n[ni] {
            pi += 1;
            ni += 1;
        } else if let Some((sp, sn)) = star {
            pi = sp + 1;
            ni = sn + 1;
            star = Some((sp, sn + 1));
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|&b| b == b'*')
}

/// Whether any pattern matches.
pub fn any_glob(patterns: &[String], name: &str) -> bool {
    patterns.iter().any(|p| glob(p, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globs() {
        assert!(glob("hg-*-main", "hg-browser-main"));
        assert!(glob("hg-ml*", "hg-ml"));
        assert!(glob("hg-ml*", "hg-ml-main"));
        assert!(!glob("hg-*-main", "hg-main-x"));
        assert!(glob("*", ""));
        assert!(glob("a*b*c", "aXbYbc"));
        assert!(!glob("hg-dst", "hg-dst2"));
    }

    #[test]
    fn unknown_keys_are_errors() {
        let v = toml::parse("[crates.hg-a]\ntier = \"P\"\nlayer = \"L1\"\ntypo = 1\n").unwrap();
        assert!(Tiers::from_value(&v).is_err());
        let v = toml::parse("[crates.hg-a]\ntier = \"Q\"\nlayer = \"L1\"\n").unwrap();
        assert!(Tiers::from_value(&v).is_err());
    }
}
