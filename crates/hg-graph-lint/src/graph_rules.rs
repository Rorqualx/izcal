//! The rules that read one resolution of the package graph.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::config::{Reach, Tier, any_glob, glob};
use crate::metadata::{DepKind, Features, Graph};
use crate::{Inputs, Report};

const NORMAL_BUILD: [DepKind; 2] = [DepKind::Normal, DepKind::Build];

/// Runs every per-resolution rule on `g`.
pub fn check(inputs: &Inputs, g: &Graph, report: &mut Report) {
    let ctx = format!("{}/{}", g.triple, g.features.name());
    let cx = Ctx {
        inputs,
        g,
        ctx: &ctx,
    };
    if g.features == Features::All {
        cx.layers(report);
        cx.tiers(report);
        cx.simulator(report);
        cx.ambient(report);
        cx.tier_t_only(report);
        cx.proc_macros(report);
        cx.only_linked_by(report);
        cx.direct_only(report);
        cx.disjoint(report);
    }
    cx.forbid(report);
}

struct Ctx<'a> {
    inputs: &'a Inputs,
    g: &'a Graph,
    ctx: &'a str,
}

impl Ctx<'_> {
    fn name(&self, i: usize) -> &str {
        &self.g.packages[i].name
    }

    /// The declared row of a workspace member.
    fn declared(&self, i: usize) -> Option<&crate::config::Declared> {
        if self.g.packages[i].member {
            self.inputs.tiers.crates.get(self.name(i))
        } else {
            None
        }
    }

    fn add(&self, report: &mut Report, rule: &str, subject: usize, message: String) {
        report.add(rule, self.name(subject), message, Some(self.ctx));
    }

    /// Packages that build a binary or C-ABI library in this resolution.
    fn artifact_builders(&self) -> Vec<usize> {
        (0..self.g.packages.len())
            .filter(|&i| self.g.packages[i].builds_artifact())
            .collect()
    }

    fn shipping(&self) -> Vec<usize> {
        self.g
            .members()
            .into_iter()
            .filter(|&i| glob(&self.inputs.rules.shipping, self.name(i)))
            .collect()
    }

    /// Stage 1 2.1 and CS-ST-2: dependencies point to the same or a
    /// lower layer. Normal and build edges count; dev edges do not.
    fn layers(&self, report: &mut Report) {
        let rules = &self.inputs.rules;
        for a in self.g.members() {
            let Some(da) = self.declared(a) else { continue };
            let Some(&ra) = rules.layer_rank.get(&da.layer) else {
                self.add(
                    report,
                    "layers",
                    a,
                    format!(
                        "layer {:?} is not in graph-rules.toml's [layers] order",
                        da.layer
                    ),
                );
                continue;
            };
            let cap = rules.layer_cap.get(&da.layer).copied();
            for b in self.g.direct(a, &NORMAL_BUILD) {
                let Some(db) = self.declared(b) else { continue };
                let Some(&rb) = rules.layer_rank.get(&db.layer) else {
                    continue;
                };
                let excepted = rules
                    .layer_exceptions
                    .iter()
                    .any(|e| e.from_layer == da.layer && e.to == self.name(b));
                if excepted {
                    continue;
                }
                if rb > ra {
                    self.add(
                        report,
                        "layers",
                        a,
                        format!(
                            "{} ({}) depends on {} ({}), a higher layer",
                            self.name(a),
                            da.layer,
                            self.name(b),
                            db.layer
                        ),
                    );
                } else if let Some(c) = cap
                    && rb > c
                    && db.layer != da.layer
                {
                    self.add(
                        report,
                        "layers",
                        a,
                        format!(
                            "{} ({}) depends on {} ({}); {} may depend only on lower layers up to its cap",
                            self.name(a),
                            da.layer,
                            self.name(b),
                            db.layer,
                            da.layer
                        ),
                    );
                }
            }
        }
    }

    /// Coding standard 6.2: declared tier ≥ the strictest tier of any
    /// binary that links the crate; and no shipping binary links tier T.
    fn tiers(&self, report: &mut Report) {
        let builders = self.artifact_builders();
        let mut computed: BTreeMap<usize, (Tier, usize)> = BTreeMap::new();
        for &b in &builders {
            let Some(db) = self.declared(b) else { continue };
            let mut linked: Vec<usize> = self.g.linked(b, &|_| false).into_keys().collect();
            linked.push(b);
            for c in linked {
                let stricter = computed
                    .get(&c)
                    .is_none_or(|(t, _)| db.tier.strictness() > t.strictness());
                if stricter {
                    computed.insert(c, (db.tier, b));
                }
            }
        }
        for (c, (tier, by)) in computed {
            let Some(dc) = self.declared(c) else { continue };
            if dc.tier.strictness() < tier.strictness() {
                self.add(
                    report,
                    "tiers",
                    c,
                    format!(
                        "crate {} is now linked by {} (tier {}); declare tier {} in its own PR (declared {})",
                        self.name(c),
                        self.name(by),
                        tier.letter(),
                        tier.letter(),
                        dc.tier.letter()
                    ),
                );
            }
        }
        for s in self.shipping() {
            if self.declared(s).is_some_and(|d| d.tier == Tier::T) {
                self.add(
                    report,
                    "tier-T",
                    s,
                    "a shipping binary is declared tier T".into(),
                );
            }
            let preds = self.g.linked(s, &|_| false);
            for &c in preds.keys() {
                if self.declared(c).is_some_and(|d| d.tier == Tier::T) {
                    self.add(
                        report,
                        "tier-T",
                        s,
                        format!(
                            "shipping binary links tier T crate {}: {}",
                            self.name(c),
                            self.g.path_to(s, c, &preds)
                        ),
                    );
                }
            }
        }
    }

    /// Q-CS-1: only the listed binaries link the simulator.
    fn simulator(&self, report: &mut Report) {
        let rules = &self.inputs.rules;
        let sims = self.g.named(&rules.simulator);
        for b in self.artifact_builders() {
            if any_glob(&rules.sim_linkers, self.name(b)) {
                continue;
            }
            if sims.contains(&b) {
                self.add(
                    report,
                    "Q-CS-1",
                    b,
                    "the simulator itself builds a binary that is not on the Q-CS-1 list".into(),
                );
                continue;
            }
            let preds = self.g.linked(b, &|_| false);
            for &s in &sims {
                if preds.contains_key(&s) {
                    self.add(
                        report,
                        "Q-CS-1",
                        b,
                        format!(
                            "links {} but is not on the Q-CS-1 list ({}): {}",
                            rules.simulator,
                            rules.sim_linkers.join(", "),
                            self.g.path_to(b, s, &preds)
                        ),
                    );
                }
            }
        }
    }

    /// Reach from `root`: the first hop over normal, build and dev edges
    /// (G3 binds test targets), later hops over normal and build edges;
    /// `stop` packages are recorded but not looked through.
    fn reach_with_dev_first(
        &self,
        root: usize,
        stop: &dyn Fn(usize) -> bool,
    ) -> BTreeMap<usize, usize> {
        let mut seen = BTreeMap::new();
        let mut queue = VecDeque::from([root]);
        while let Some(n) = queue.pop_front() {
            if n != root && stop(n) {
                continue;
            }
            let kinds: &[DepKind] = if n == root {
                &[DepKind::Normal, DepKind::Build, DepKind::Dev]
            } else {
                &NORMAL_BUILD
            };
            for to in self.g.direct(n, kinds) {
                if to != root && !seen.contains_key(&to) {
                    seen.insert(to, n);
                    queue.push_back(to);
                }
            }
        }
        seen
    }

    /// CS-AS-1 (A.6): ambient-effect crates only through the substrate.
    fn ambient(&self, report: &mut Report) {
        let rules = &self.inputs.rules;
        for m in self.g.members() {
            let name = self.name(m);
            let tier_t = self.declared(m).is_some_and(|d| d.tier == Tier::T);
            if tier_t || any_glob(&rules.ambient_via, name) || any_glob(&rules.ambient_exempt, name)
            {
                continue;
            }
            let stop = |i: usize| {
                any_glob(&rules.ambient_via, self.name(i))
                    || self.declared(i).is_some_and(|d| d.tier == Tier::T)
            };
            let preds = self.reach_with_dev_first(m, &stop);
            for &c in preds.keys() {
                if !self.g.packages[c].member && any_glob(&rules.ambient_crates, self.name(c)) {
                    self.add(
                        report,
                        "CS-AS-1",
                        m,
                        format!(
                            "reaches {} other than through {}: {}",
                            self.name(c),
                            rules.ambient_via.join(", "),
                            self.g.path_to(m, c, &preds)
                        ),
                    );
                }
            }
        }
    }

    /// CS-ER-10 (A.6): `anyhow` only in tier T.
    fn tier_t_only(&self, report: &mut Report) {
        let rules = &self.inputs.rules;
        for m in self.g.members() {
            if self.declared(m).is_none_or(|d| d.tier == Tier::T) {
                continue;
            }
            let preds = self.g.reach(m, &NORMAL_BUILD, true, &|_| false);
            for &c in preds.keys() {
                if !self.g.packages[c].member && any_glob(&rules.tier_t_only, self.name(c)) {
                    self.add(
                        report,
                        "CS-ER-10",
                        m,
                        format!(
                            "{} is for tier T only: {}",
                            self.name(c),
                            self.g.path_to(m, c, &preds)
                        ),
                    );
                }
            }
        }
    }

    /// CS-ST-2: procedural macros only from the allowlist, and none (nor
    /// the banned crates) in the banned layers.
    fn proc_macros(&self, report: &mut Report) {
        let rules = &self.inputs.rules;
        for m in self.g.members() {
            let preds = self.g.reach(m, &NORMAL_BUILD, false, &|_| false);
            let banned_layer = self
                .declared(m)
                .is_some_and(|d| any_glob(&rules.proc_macro_banned_layers, &d.layer));
            for &c in preds.keys() {
                let p = &self.g.packages[c];
                if p.proc_macro && !p.member && !any_glob(&rules.proc_macro_allow, &p.name) {
                    self.add(
                        report,
                        "CS-ST-2",
                        m,
                        format!(
                            "procedural macro {} is not on the allowlist: {}",
                            p.name,
                            self.g.path_to(m, c, &preds)
                        ),
                    );
                }
                if banned_layer
                    && (p.proc_macro || any_glob(&rules.proc_macro_banned_crates, &p.name))
                {
                    self.add(
                        report,
                        "CS-ST-2",
                        m,
                        format!(
                            "L1–L3 crates use no procedural macro or {}: {}",
                            rules.proc_macro_banned_crates.join(", "),
                            self.g.path_to(m, c, &preds)
                        ),
                    );
                }
            }
        }
    }

    /// `[[forbid]]` rules.
    fn forbid(&self, report: &mut Report) {
        for f in &self.inputs.rules.forbid {
            if f.features != self.g.features {
                continue;
            }
            for a in 0..self.g.packages.len() {
                let name = self.name(a);
                if !any_glob(&f.from, name)
                    || any_glob(&f.from_except, name)
                    || !self.g.packages[a].is_built()
                {
                    continue;
                }
                let preds = match f.reach {
                    Reach::Linked => self.g.linked(a, &|_| false),
                    Reach::Direct => self
                        .g
                        .direct(a, &NORMAL_BUILD)
                        .into_iter()
                        .map(|b| (b, a))
                        .collect(),
                };
                for &b in preds.keys() {
                    let bn = self.name(b);
                    let by_name = any_glob(&f.crates, bn);
                    let by_layer = self
                        .declared(b)
                        .is_some_and(|d| f.layers.contains(&d.layer));
                    if (by_name || by_layer) && !any_glob(&f.except, bn) {
                        self.add(
                            report,
                            &f.rule,
                            a,
                            format!(
                                "must not depend on {bn} ({}): {}",
                                f.why,
                                self.g.path_to(a, b, &preds)
                            ),
                        );
                    }
                }
            }
        }
    }

    /// `[[only-linked-by]]` rules.
    fn only_linked_by(&self, report: &mut Report) {
        for o in &self.inputs.rules.only_linked_by {
            let guarded: BTreeSet<usize> = (0..self.g.packages.len())
                .filter(|&i| any_glob(&o.crates, self.name(i)))
                .collect();
            if guarded.is_empty() {
                continue;
            }
            for b in self.artifact_builders() {
                if any_glob(&o.binaries, self.name(b)) {
                    continue;
                }
                let preds = self.g.linked(b, &|_| false);
                for &c in &guarded {
                    if c == b || preds.contains_key(&c) {
                        self.add(
                            report,
                            &o.rule,
                            b,
                            format!(
                                "links {} ({}); only {} may: {}",
                                self.name(c),
                                o.why,
                                o.binaries.join(", "),
                                self.g.path_to(b, c, &preds)
                            ),
                        );
                    }
                }
            }
            if let Some(allowed) = &o.direct_dependents {
                for a in 0..self.g.packages.len() {
                    if any_glob(allowed, self.name(a)) {
                        continue;
                    }
                    for b in self.g.direct(a, &NORMAL_BUILD) {
                        if guarded.contains(&b) {
                            self.add(
                                report,
                                &o.rule,
                                a,
                                format!(
                                    "depends on {} directly ({}); only {} may",
                                    self.name(b),
                                    o.why,
                                    allowed.join(", ")
                                ),
                            );
                        }
                    }
                }
            }
        }
    }

    /// `[[direct-only]]` rules: the dependency half (the tier and
    /// `no_std` halves are workspace rules).
    fn direct_only(&self, report: &mut Report) {
        for d in &self.inputs.rules.direct_only {
            for a in 0..self.g.packages.len() {
                if !any_glob(&d.crates, self.name(a)) {
                    continue;
                }
                for b in self.g.direct(a, &NORMAL_BUILD) {
                    if !any_glob(&d.allow, self.name(b)) {
                        let allowed = if d.allow.is_empty() {
                            "none".to_owned()
                        } else {
                            d.allow.join(", ")
                        };
                        self.add(
                            report,
                            &d.rule,
                            a,
                            format!(
                                "depends on {} ({}); allowed: {allowed}",
                                self.name(b),
                                d.why
                            ),
                        );
                    }
                }
            }
        }
    }

    /// `[[disjoint]]` rules.
    fn disjoint(&self, report: &mut Report) {
        for d in &self.inputs.rules.disjoint {
            let side = |pats: &[String]| -> (Vec<usize>, BTreeSet<usize>) {
                let roots: Vec<usize> = (0..self.g.packages.len())
                    .filter(|&i| any_glob(pats, self.name(i)))
                    .collect();
                let mut all = BTreeSet::new();
                for &r in &roots {
                    all.extend(self.g.linked(r, &|_| false).into_keys());
                }
                (roots, all)
            };
            let (ra, la) = side(&d.a);
            let (rb, lb) = side(&d.b);
            if ra.is_empty() || rb.is_empty() {
                continue;
            }
            for &c in la.intersection(&lb) {
                if self.g.packages[c].member && !any_glob(&d.allow, self.name(c)) {
                    report.add(
                        &d.rule,
                        &format!("{} / {}", d.a.join(","), d.b.join(",")),
                        format!("both sides link {} ({})", self.name(c), d.why),
                        Some(self.ctx),
                    );
                }
            }
        }
    }
}
