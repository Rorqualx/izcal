//! Checking the overlay against itself and against webref.
//!
//! Two passes, and they fail for different reasons.
//!
//! The **first** checks the overlay alone: that every kind it names is
//! declared, that every computed-value text is classified once, that a
//! property's block is well formed. A failure here is a mistake in the
//! file.
//!
//! The **second** compares the overlay with the pinned `@webref/css`,
//! field by field, in both directions. A failure here is the one the
//! design exists for: the specification has moved, or the overlay was
//! written against something the specification never said. Each
//! difference is reported as a [`Disagreement`] naming the property, the
//! field, what the overlay says and what webref says, so a re-pin is
//! triaged line by line rather than accepted wholesale ([SP 6.6][SP],
//! ST-7: "Re-pin blocked until the overlay is fixed").
//!
//! The second pass needs the data file, which is pinned and fetched
//! rather than committed, so it runs only when the file is there. The
//! `html5lib` CI job's shape is followed: the data is fetched, and
//! `IZCAL_REQUIRE_CSS_DATA` makes its absence an error instead of a
//! skip.
//!
//! [SP]: ../../../reports/Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md

use std::collections::{BTreeMap, BTreeSet};

use crate::syntax::{Affect, AliasDecl, ComputedDecl, Error, File, KindDecl, PropertyDecl};
use crate::webref::Catalogue;

/// The kind a property whose computed value is defined by prose must
/// carry, and the only kind that takes a `hook`.
pub const PROSE_KIND: &str = "prose";

/// A checked overlay.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// The kinds, in the order declared.
    pub kinds: Vec<KindDecl>,
    /// Every computed-value text, classified, sorted by the text.
    pub computed: Vec<ComputedDecl>,
    /// The properties, sorted by name.
    pub properties: Vec<PropertyDecl>,
    /// The legacy aliases, sorted by name.
    pub aliases: Vec<AliasDecl>,
}

impl Model {
    /// The kind a property's computed-value text classifies to.
    ///
    /// A property webref gives no computed-value text for has none, and
    /// the overlay restates that absence, so this returns `None` for it.
    #[must_use]
    pub fn kind_of<'a>(&'a self, p: &'a PropertyDecl) -> Option<&'a str> {
        if let Some(k) = &p.kind {
            return Some(k.as_str());
        }
        let text = p.computed.as_ref()?;
        self.computed
            .iter()
            .find(|c| &c.text == text)
            .map(|c| c.kind.as_str())
    }

    /// The properties whose kind the overlay overrides, because the
    /// specification's own computed-value text is wrong for them.
    #[must_use]
    pub fn overrides(&self) -> Vec<&PropertyDecl> {
        // A `kind` where there is no text to classify is not an
        // override of anything; only one given *beside* a restated text
        // overrides it.
        self.properties
            .iter()
            .filter(|p| p.kind.is_some() && p.computed.is_some())
            .collect()
    }

    /// The properties that map to another rather than being stored.
    #[must_use]
    pub fn mappings(&self) -> Vec<&PropertyDecl> {
        self.properties
            .iter()
            .filter(|p| p.maps_to.is_some())
            .collect()
    }

    /// The properties whose initial value Izcal supplies because
    /// `@webref/css` extracted none.
    #[must_use]
    pub fn supplied_initials(&self) -> Vec<&PropertyDecl> {
        self.properties
            .iter()
            .filter(|p| p.initial_is.is_some())
            .collect()
    }

    /// How many properties still carry `affects unknown`.
    #[must_use]
    pub fn undecided(&self) -> usize {
        self.properties
            .iter()
            .filter(|p| p.affects.contains(&Affect::Unknown))
            .count()
    }
}

/// One difference between the overlay and webref.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Disagreement {
    /// The property, or the alias, it is about.
    pub subject: String,
    /// The field that differs.
    pub field: &'static str,
    /// What the overlay says, or `None` when it says nothing.
    pub overlay: Option<String>,
    /// What webref says, or `None` when it says nothing.
    pub webref: Option<String>,
}

impl std::fmt::Display for Disagreement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let show = |v: &Option<String>| v.clone().unwrap_or_else(|| "(nothing)".to_owned());
        write!(
            f,
            "{} {}: the overlay says {:?}, webref {:?}",
            self.subject,
            self.field,
            show(&self.overlay),
            show(&self.webref)
        )
    }
}

fn err(file: &str, line: usize, message: impl Into<String>) -> Error {
    Error {
        file: file.to_owned(),
        line,
        message: message.into(),
    }
}

/// Is this a CSS identifier the way a property is spelt: ASCII
/// lowercase, digits and hyphens, starting with a letter or the
/// vendor-prefix hyphen.
fn is_property_name(s: &str) -> bool {
    let mut cs = s.chars();
    let first = cs.next();
    matches!(first, Some(c) if c.is_ascii_lowercase() || c == '-')
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The first pass: the overlay against itself.
///
/// # Errors
/// [`Error`] for the first problem found, naming the file and line.
pub fn check(files: &[(String, File)]) -> Result<Model, Error> {
    let mut m = Model::default();
    let mut where_kind: BTreeMap<String, (String, usize)> = BTreeMap::new();
    let mut where_text: BTreeMap<String, (String, usize)> = BTreeMap::new();
    let mut where_prop: BTreeMap<String, (String, usize)> = BTreeMap::new();
    let mut where_alias: BTreeMap<String, (String, usize)> = BTreeMap::new();

    for (file, f) in files {
        for k in &f.kinds {
            if !is_property_name(&k.name) {
                return Err(err(
                    file,
                    k.line,
                    format!(
                        "the kind name `{}` is not lower-case and hyphenated",
                        k.name
                    ),
                ));
            }
            if where_kind
                .insert(k.name.clone(), (file.clone(), k.line))
                .is_some()
            {
                return Err(err(
                    file,
                    k.line,
                    format!("the kind `{}` is declared twice", k.name),
                ));
            }
            m.kinds.push(k.clone());
        }
        for c in &f.computed {
            if let Some((other, line)) = where_text.get(&c.text) {
                return Err(err(
                    file,
                    c.line,
                    format!("this computed-value text is already classified at {other}:{line}"),
                ));
            }
            where_text.insert(c.text.clone(), (file.clone(), c.line));
            m.computed.push(c.clone());
        }
        for p in &f.properties {
            if !is_property_name(&p.name) {
                return Err(err(
                    file,
                    p.line,
                    format!("`{}` is not spelt the way a CSS property is", p.name),
                ));
            }
            if let Some((other, line)) = where_prop.get(&p.name) {
                return Err(err(
                    file,
                    p.line,
                    format!("{} already has a block at {other}:{line}", p.name),
                ));
            }
            where_prop.insert(p.name.clone(), (file.clone(), p.line));
            let mut sorted = p.affects.clone();
            sorted.sort_unstable();
            sorted.dedup();
            if sorted.len() != p.affects.len() {
                return Err(err(
                    file,
                    p.line,
                    format!("{} names the same effect twice", p.name),
                ));
            }
            if p.affects.len() > 1 && p.affects.iter().any(|a| a.stands_alone()) {
                return Err(err(
                    file,
                    p.line,
                    format!(
                        "{} writes `{}` beside another effect; it stands alone",
                        p.name,
                        p.affects
                            .iter()
                            .find(|a| a.stands_alone())
                            .map_or("", |a| a.word())
                    ),
                ));
            }
            m.properties.push(p.clone());
        }
        for a in &f.aliases {
            if let Some((other, line)) = where_alias.get(&a.name) {
                return Err(err(
                    file,
                    a.line,
                    format!("{} is already an alias at {other}:{line}", a.name),
                ));
            }
            where_alias.insert(a.name.clone(), (file.clone(), a.line));
            m.aliases.push(a.clone());
        }
    }

    if !where_kind.contains_key(PROSE_KIND) {
        return Err(err(
            "",
            0,
            format!("no `{PROSE_KIND}` kind: it is the one a `hook` belongs to"),
        ));
    }

    for c in &m.computed {
        if !where_kind.contains_key(&c.kind) {
            let (file, line) = where_text.get(&c.text).cloned().unwrap_or_default();
            return Err(err(
                &file,
                line,
                format!("`{}` is not a declared kind", c.kind),
            ));
        }
    }

    let classified: BTreeMap<&str, &str> = m
        .computed
        .iter()
        .map(|c| (c.text.as_str(), c.kind.as_str()))
        .collect();
    for p in &m.properties {
        let (file, line) = where_prop.get(&p.name).cloned().unwrap_or_default();
        if let Some(k) = &p.kind
            && !where_kind.contains_key(k)
        {
            return Err(err(&file, line, format!("`{k}` is not a declared kind")));
        }
        if let Some(t) = &p.maps_to {
            if !where_prop.contains_key(t) {
                return Err(err(
                    &file,
                    line,
                    format!("{} maps to {t}, which has no block", p.name),
                ));
            }
            if !p.longhands.is_empty() {
                return Err(err(
                    &file,
                    line,
                    format!(
                        "{} maps to another property and cannot also be a shorthand",
                        p.name
                    ),
                ));
            }
        } else {
            // Nothing is stored under a mapping's name, so a mapping
            // needs neither. Everything else does: a property with no
            // initial value cannot be cascaded, and one with no kind
            // has no conversion.
            if p.initial.is_none() && p.initial_is.is_none() {
                return Err(err(
                    &file,
                    line,
                    format!(
                        "{} has no initial value: webref extracted none, so the overlay gives it \
                         with `initial-is`, or says what it maps to",
                        p.name
                    ),
                ));
            }
            if p.computed.is_none() && p.kind.is_none() {
                return Err(err(
                    &file,
                    line,
                    format!(
                        "{} has no computed-value text to classify, so the overlay gives its \
                         `kind`, or says what it maps to",
                        p.name
                    ),
                ));
            }
        }
        let kind = p.kind.as_deref().or_else(|| {
            p.computed
                .as_ref()
                .and_then(|t| classified.get(t.as_str()).copied())
        });
        if let Some(text) = &p.computed
            && kind.is_none()
        {
            return Err(err(
                &file,
                line,
                format!(
                    "{}'s computed-value text is not classified: {text:?}",
                    p.name
                ),
            ));
        }
        match (kind, &p.hook) {
            (Some(PROSE_KIND), None) => {
                return Err(err(
                    &file,
                    line,
                    format!(
                        "{} classifies as `{PROSE_KIND}` and names no `hook`; a computed value the \
                         table cannot carry needs a function that can",
                        p.name
                    ),
                ));
            }
            (k, Some(h)) if k != Some(PROSE_KIND) => {
                return Err(err(
                    &file,
                    line,
                    format!(
                        "{} names the hook `{h}` but classifies as `{}`; only `{PROSE_KIND}` \
                         takes one",
                        p.name,
                        k.unwrap_or("(none)")
                    ),
                ));
            }
            _ => {}
        }
        // A longhand may be a legacy alias. `font` names `font-stretch`
        // at the 8.7.9 pin, which is the alias of `font-width`; the
        // overlay restates webref's list as webref writes it, and the
        // table resolves it to the property the cascade stores.
        for l in &p.longhands {
            if !where_prop.contains_key(l) && !where_alias.contains_key(l) {
                return Err(err(
                    &file,
                    line,
                    format!(
                        "{} names the longhand {l}, which is neither a property nor an alias",
                        p.name
                    ),
                ));
            }
        }
    }

    for a in &m.aliases {
        if !where_prop.contains_key(&a.target) {
            let (file, line) = where_alias.get(&a.name).cloned().unwrap_or_default();
            return Err(err(
                &file,
                line,
                format!(
                    "the alias {} stands for {}, which has no block",
                    a.name, a.target
                ),
            ));
        }
        if where_prop.contains_key(&a.name) {
            let (file, line) = where_alias.get(&a.name).cloned().unwrap_or_default();
            return Err(err(
                &file,
                line,
                format!("{} is both an alias and a property", a.name),
            ));
        }
    }

    m.computed.sort_by(|a, b| a.text.cmp(&b.text));
    m.properties.sort_by(|a, b| a.name.cmp(&b.name));
    m.aliases.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(m)
}

/// The second pass: the overlay against the pinned catalogue.
///
/// Returns every difference, sorted, rather than the first: a re-pin is
/// triaged as a whole, and a caller that stopped at the first would make
/// that thirty rounds instead of one.
#[must_use]
pub fn against_webref(m: &Model, cat: &Catalogue) -> Vec<Disagreement> {
    let mut out = Vec::new();
    let mut d =
        |subject: &str, field: &'static str, overlay: Option<String>, webref: Option<String>| {
            out.push(Disagreement {
                subject: subject.to_owned(),
                field,
                overlay,
                webref,
            });
        };

    let overlay_names: BTreeSet<&str> = m.properties.iter().map(|p| p.name.as_str()).collect();
    for name in cat.properties.keys() {
        if !overlay_names.contains(name.as_str()) {
            d(
                name,
                "the property itself",
                None,
                Some("a property".to_owned()),
            );
        }
    }

    for p in &m.properties {
        let Some(w) = cat.properties.get(&p.name) else {
            d(
                &p.name,
                "the property itself",
                Some("a block".to_owned()),
                None,
            );
            continue;
        };
        if p.href != w.href {
            d(&p.name, "href", Some(p.href.clone()), Some(w.href.clone()));
        }
        if p.initial != w.initial {
            d(&p.name, "initial", p.initial.clone(), w.initial.clone());
        }
        if p.inherited != w.inherited {
            d(
                &p.name,
                "inherited",
                p.inherited.clone(),
                w.inherited.clone(),
            );
        }
        if p.animation != w.animation {
            d(
                &p.name,
                "animation",
                p.animation.clone(),
                w.animation.clone(),
            );
        }
        if p.syntax != w.syntax {
            d(&p.name, "syntax", p.syntax.clone(), w.syntax.clone());
        }
        if p.computed != w.computed {
            d(&p.name, "computed", p.computed.clone(), w.computed.clone());
        }
        if p.longhands != w.longhands {
            let join = |v: &[String]| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.join(", "))
                }
            };
            d(&p.name, "longhands", join(&p.longhands), join(&w.longhands));
        }
    }

    let overlay_aliases: BTreeMap<&str, &str> = m
        .aliases
        .iter()
        .map(|a| (a.name.as_str(), a.target.as_str()))
        .collect();
    for (name, target) in &cat.aliases {
        match overlay_aliases.get(name.as_str()) {
            None => d(name, "the alias itself", None, Some(target.clone())),
            Some(ours) if *ours != target.as_str() => {
                d(
                    name,
                    "alias target",
                    Some((*ours).to_owned()),
                    Some(target.clone()),
                );
            }
            Some(_) => {}
        }
    }
    for a in &m.aliases {
        if !cat.aliases.contains_key(&a.name) {
            d(&a.name, "the alias itself", Some(a.target.clone()), None);
        }
    }

    // The classification is exhaustive in both directions. A text webref
    // carries and nothing classifies would leave a property with no
    // conversion; a classification of a text webref no longer carries is
    // a rule that has quietly stopped applying to anything.
    let webref_texts: BTreeSet<&str> = cat
        .properties
        .values()
        .filter_map(|p| p.computed.as_deref())
        .collect();
    let ours: BTreeSet<&str> = m.computed.iter().map(|c| c.text.as_str()).collect();
    for t in webref_texts.difference(&ours) {
        d(t, "a computed-value text", None, Some("carried".to_owned()));
    }
    for t in ours.difference(&webref_texts) {
        d(
            t,
            "a computed-value text",
            Some("classified".to_owned()),
            None,
        );
    }

    out.sort();
    out
}
