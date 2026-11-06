//! OC-12: every path has an anchor (its own or the nearest enclosing
//! one), and every clause id exists at the pinned revision of its
//! specification.
//!
//! The ids come from one of two places. If `$IZCAL_TEST_DATA` holds a
//! pinned `spec.html` for a specification (the test-data cache of the
//! JS-M0 plan, Phase 0 item 5: `<dir>/<spec>/spec.html` or
//! `<dir>/<spec>-<revision>/spec.html`), its element ids are the source.
//! Otherwise the checked-in pin list `ops/anchors.pinned` is: the ids the
//! descriptions use, each verified against the rendered specification by
//! `hg-opgen pin`, with the revision recorded in its header. Nothing from
//! the specifications is copied but the ids.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{Anchor, Spec};
use crate::diag::{Diags, Span};
use crate::model::Model;

/// The pin list's path, relative to the workspace root.
pub const PIN_FILE: &str = "crates/hg-js-ops/ops/anchors.pinned";

/// Where the ids came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdSource {
    /// A pinned `spec.html` in the test-data cache.
    TestData(String),
    /// The checked-in pin list.
    PinList,
}

/// The ids of each specification, and where they came from.
#[derive(Debug, Clone, Default)]
pub struct Pins {
    /// Ids per specification name.
    pub ids: BTreeMap<String, BTreeSet<String>>,
    /// The revision text per specification.
    pub revision: BTreeMap<String, String>,
    /// The source per specification.
    pub source: BTreeMap<String, IdSource>,
}

/// Every element id in a rendered specification, except ecmarkup's
/// generated cross-reference ids (`_ref_N`).
pub fn html_ids(html: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let b = html.as_bytes();
    let mut i = 0;
    while let Some(at) = html[i..].find("id=") {
        let start = i + at;
        i = start + 3;
        // Only an attribute: preceded by whitespace.
        if start == 0 || !b[start - 1].is_ascii_whitespace() {
            continue;
        }
        let rest = &html[i..];
        let id = if let Some(q) = rest.strip_prefix('"') {
            q.split('"').next().unwrap_or("")
        } else {
            rest.split(|c: char| c.is_whitespace() || c == '>')
                .next()
                .unwrap_or("")
        };
        if !id.is_empty() && !id.starts_with("_ref_") {
            out.insert(id.to_string());
        }
    }
    out
}

/// Parses the pin list.
pub fn parse_pins(text: &str) -> Pins {
    let mut p = Pins::default();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rev) = line.strip_prefix("# revision ") {
            if let Some((spec, r)) = rev.split_once(' ') {
                p.revision.insert(spec.to_string(), r.to_string());
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((spec, id)) = line.split_once(' ') {
            p.ids
                .entry(spec.to_string())
                .or_default()
                .insert(id.to_string());
            p.source.insert(spec.to_string(), IdSource::PinList);
        }
    }
    p
}

/// Every anchor of the model: (item kind, item, path, anchor).
pub fn all_anchors(m: &Model) -> Vec<(&'static str, String, String, Anchor, bool)> {
    let mut out = Vec::new();
    for r in &m.runtimes {
        out.push((
            "runtime",
            r.name.clone(),
            "main".into(),
            r.sig.anchor.clone(),
            true,
        ));
    }
    for h in &m.helpers {
        for p in &h.body.paths {
            out.push((
                "helper",
                h.name.clone(),
                p.name.clone(),
                p.anchor.clone(),
                p.own_anchor,
            ));
        }
    }
    for i in &m.intrinsics {
        for p in &i.body.paths {
            out.push((
                "intrinsic",
                format!("%{}", i.name),
                p.name.clone(),
                p.anchor.clone(),
                p.own_anchor,
            ));
        }
    }
    for o in &m.ops {
        for p in &o.body.paths {
            out.push((
                "op",
                o.name.clone(),
                p.name.clone(),
                p.anchor.clone(),
                p.own_anchor,
            ));
        }
    }
    out
}

/// Checks every anchor of `m` against `pins` (OC-12).
pub fn check(m: &Model, pins: &Pins, diags: &mut Diags) {
    for (_, item, path, anchor, _) in all_anchors(m) {
        match &anchor {
            Anchor::Izcal { reason, span } => {
                if reason.trim().is_empty() {
                    diags.err(
                        *span,
                        "OC-12",
                        format!("`{item}` path `{path}`: an `izcal` anchor states its reason"),
                    );
                }
            }
            Anchor::Spec {
                spec, clause, span, ..
            } => {
                let name = spec.name();
                match pins.ids.get(name) {
                    Some(ids) if ids.contains(clause) => {}
                    Some(_) => diags.err(
                        *span,
                        "OC-12",
                        format!("`{item}` path `{path}`: `{clause}` is not a clause id of the pinned {name} ({})", source_text(pins, name)),
                    ),
                    None => diags.err(
                        *span,
                        "OC-12",
                        format!("`{item}` path `{path}`: no pinned revision of {name} to check `{clause}` against"),
                    ),
                }
            }
        }
    }
}

fn source_text(p: &Pins, spec: &str) -> String {
    match p.source.get(spec) {
        Some(IdSource::TestData(path)) => format!("from {path}"),
        _ => format!("from {PIN_FILE}"),
    }
}

/// The spec ids the model uses, per specification.
pub fn used(m: &Model) -> BTreeMap<&'static str, BTreeSet<String>> {
    let mut out: BTreeMap<&'static str, BTreeSet<String>> = BTreeMap::new();
    for (_, _, _, a, _) in all_anchors(m) {
        if let Anchor::Spec { spec, clause, .. } = a {
            out.entry(spec.name()).or_default().insert(clause);
        }
    }
    out
}

/// Writes the pin list for the ids `m` uses, verified against the given
/// rendered specifications (`spec name` → (revision text, html)).
///
/// # Errors
/// The ids no given specification has.
pub fn pin_text(
    m: &Model,
    specs: &BTreeMap<String, (String, String)>,
) -> Result<String, Vec<String>> {
    let mut out = String::from(
        "# The ECMA-262 and proposal clause ids the .hgop descriptions anchor to,\n\
         # each verified to exist in the rendered specification at the revision\n\
         # below (OC-12). Written by `hg-opgen pin`; do not edit. A new anchor\n\
         # needs a re-pin against the then-current pinned specification.\n",
    );
    let mut missing = Vec::new();
    let mut body = String::new();
    for (spec, ids) in used(m) {
        let Some((rev, html)) = specs.get(spec) else {
            missing.push(format!("{spec}: no specification text given"));
            continue;
        };
        out.push_str(&format!("# revision {spec} {rev}\n"));
        let have = html_ids(html);
        for id in ids {
            if have.contains(&id) {
                body.push_str(&format!("{spec} {id}\n"));
            } else {
                missing.push(format!("{spec} {id}"));
            }
        }
    }
    if !missing.is_empty() {
        return Err(missing);
    }
    out.push_str(&body);
    Ok(out)
}

/// The specification a `$IZCAL_TEST_DATA` directory name holds, if any.
pub fn spec_of_dir(name: &str) -> Option<Spec> {
    let base = name.split('-').next().unwrap_or(name);
    match base {
        "ecma262" => Some(Spec::Ecma262),
        "ecma402" => Some(Spec::Ecma402),
        "temporal" => Some(Spec::Temporal),
        _ if name.starts_with("proposal-temporal") => Some(Spec::Temporal),
        _ => None,
    }
}

/// A span for diagnostics with no position.
pub const NOWHERE: Span = Span {
    file: 0,
    line: 0,
    col: 0,
};
