//! Emits the opcode table for `hg-js-bytecode` (R0-O2: one list for the
//! decoder, assembler, disassembler and the verifier's kind tables), the
//! operation facts (`facts.rs`, `facts.tsv`) and the anchor index
//! (`anchors.tsv`) for `hg-js-ops`.

use std::fmt::Write as _;

use crate::anchors::Pins;
use crate::ast::{AccMode, AfterCall, Anchor};
use crate::flow::{Flows, ItemFlow};
use crate::model::*;
use crate::tir::Body;

/// FNV-1a, 64 bits: the format identifier's hash (a fixed, portable
/// function of the table's text, OC-15).
pub fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn acc_rust(a: AccMode) -> &'static str {
    match a {
        AccMode::None => "AccMode::None",
        AccMode::In => "AccMode::In",
        AccMode::Out => "AccMode::Out",
        AccMode::InOut => "AccMode::InOut",
    }
}

fn after_text(a: Option<AfterCall>) -> &'static str {
    match a {
        None => "none",
        Some(AfterCall::Result) => "result",
        Some(AfterCall::Preserve) => "preserve",
    }
}

/// The canonical text of the opcode table the format identifier hashes.
pub fn table_text(m: &Model) -> String {
    let mut t = String::new();
    for op in &m.ops {
        let ks: Vec<String> = op.operands.iter().map(|(_, k)| k.text()).collect();
        let _ = writeln!(
            t,
            "{:#04x} {} ({}) acc {}",
            op.opcode,
            op.name,
            ks.join(", "),
            op.acc.name()
        );
    }
    for (i, f) in m.intrinsics.iter().enumerate() {
        let _ = writeln!(t, "intrinsic {i} %{} {}", f.name, f.sig.params.len());
    }
    t
}

/// `crates/hg-js-bytecode/src/generated/opcodes.rs`.
pub fn emit_opcodes(m: &Model, flows: &Flows, header: &str) -> String {
    let mut o = String::from(header);
    o.push_str("//! The opcode table (design review R0-O2, bytecode note Section 5):\n//! numbers in declaration order, mnemonics, operand kinds, accumulator\n//! modes and the intrinsics, all from the `.hgop` operations.\n\n");
    let has_const = m.ops.iter().any(|op| {
        op.operands
            .iter()
            .any(|(_, k)| matches!(k, OperandKind::Const(_)))
    });
    let has_slot = m.ops.iter().any(|op| {
        op.operands
            .iter()
            .any(|(_, k)| matches!(k, OperandKind::Slot(_)))
    });
    let mut uses = vec!["AccMode", "OpInfo", "OperandKind"];
    if has_const {
        uses.insert(1, "ConstKinds");
    }
    if has_slot {
        uses.push("SlotKind");
    }
    let _ = writeln!(o, "use crate::{{{}}};\n", uses.join(", "));
    let fid = fnv1a(&table_text(m));
    let _ = writeln!(
        o,
        "/// The format identifier: a hash of this table, so a unit made for another\n/// opcode table is rejected on load (bytecode note 4.1).\npub const FORMAT_ID: u64 = {fid:#018x};\n"
    );
    let _ = writeln!(
        o,
        "/// The number of opcodes.\npub const OP_COUNT: usize = {};\n",
        m.ops.len()
    );
    o.push_str("/// An opcode of the JS-M0 core set.\n#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n#[repr(u8)]\npub enum Op {\n");
    for op in &m.ops {
        let _ = writeln!(o, "    /// `{}`: {}.", op.name, anchor_short(&op.anchor));
        let _ = writeln!(o, "    {} = {:#04x},", op.name, op.opcode);
    }
    o.push_str("}\n\nimpl Op {\n    /// Every opcode, in number order.\n");
    let names: Vec<String> = m.ops.iter().map(|o| format!("Self::{}", o.name)).collect();
    let _ = writeln!(
        o,
        "    pub const ALL: [Self; {}] = [{}];\n",
        m.ops.len(),
        names.join(", ")
    );
    o.push_str("    /// The opcode encoded as `b`, if any.\n    pub const fn from_byte(b: u8) -> Option<Self> {\n        match b {\n");
    for op in &m.ops {
        let _ = writeln!(
            o,
            "            {:#04x} => Some(Self::{}),",
            op.opcode, op.name
        );
    }
    o.push_str("            _ => None,\n        }\n    }\n\n    /// The opcode's byte.\n    pub const fn byte(self) -> u8 {\n        self as u8\n    }\n\n");
    o.push_str("    /// The opcode named `mnemonic`, if any.\n    pub fn from_mnemonic(mnemonic: &str) -> Option<Self> {\n        match mnemonic {\n");
    for op in &m.ops {
        let _ = writeln!(o, "            \"{}\" => Some(Self::{}),", op.name, op.name);
    }
    o.push_str("            _ => None,\n        }\n    }\n\n    /// The opcode's table entry.\n    pub const fn info(self) -> &'static OpInfo {\n        match self {\n");
    for (i, op) in m.ops.iter().enumerate() {
        let ks: Vec<String> = op.operands.iter().map(|(_, k)| k.rust()).collect();
        let f = &flows.ops[i];
        let terminator = !f.control.contains("next");
        let _ = writeln!(
            o,
            "            Self::{} => &OpInfo {{\n                mnemonic: \"{}\",\n                operands: &[{}],\n                acc: {},\n                terminator: {terminator},\n                may_call_user: {},\n            }},",
            op.name,
            op.name,
            ks.join(", "),
            acc_rust(op.acc),
            op.effects.user()
        );
    }
    o.push_str("        }\n    }\n}\n\n");
    o.push_str("/// An intrinsic, called through `CallIntrinsic` (bytecode note 4.9).\n#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\npub enum Intrinsic {\n");
    for f in &m.intrinsics {
        let _ = writeln!(o, "    /// `%{}`: {}.", f.name, anchor_short(&f.sig.anchor));
        let _ = writeln!(o, "    {},", f.name);
    }
    o.push_str("}\n\nimpl Intrinsic {\n");
    let names: Vec<String> = m
        .intrinsics
        .iter()
        .map(|f| format!("Self::{}", f.name))
        .collect();
    let _ = writeln!(
        o,
        "    /// Every intrinsic, in id order.\n    pub const ALL: [Self; {}] = [{}];\n",
        m.intrinsics.len(),
        names.join(", ")
    );
    o.push_str("    /// The intrinsic with id `i`, if any.\n    pub const fn from_index(i: u32) -> Option<Self> {\n        match i {\n");
    for (i, f) in m.intrinsics.iter().enumerate() {
        let _ = writeln!(o, "            {i} => Some(Self::{}),", f.name);
    }
    o.push_str("            _ => None,\n        }\n    }\n\n    /// The intrinsic's id.\n    pub const fn index(self) -> u32 {\n        match self {\n");
    for (i, f) in m.intrinsics.iter().enumerate() {
        let _ = writeln!(o, "            Self::{} => {i},", f.name);
    }
    o.push_str("        }\n    }\n\n    /// The intrinsic's name, without `%`.\n    pub const fn name(self) -> &'static str {\n        match self {\n");
    for f in &m.intrinsics {
        let _ = writeln!(o, "            Self::{} => \"{}\",", f.name, f.name);
    }
    o.push_str("        }\n    }\n\n    /// The intrinsic named `name` (without `%`), if any.\n    pub fn from_name(name: &str) -> Option<Self> {\n        match name {\n");
    for f in &m.intrinsics {
        let _ = writeln!(o, "            \"{}\" => Some(Self::{}),", f.name, f.name);
    }
    o.push_str("            _ => None,\n        }\n    }\n\n    /// The number of arguments the intrinsic takes (OC-13).\n    pub const fn arity(self) -> u32 {\n        match self {\n");
    for f in &m.intrinsics {
        let _ = writeln!(o, "            Self::{} => {},", f.name, f.sig.params.len());
    }
    o.push_str("        }\n    }\n}\n");
    o
}

fn anchor_short(a: &Anchor) -> String {
    match a {
        Anchor::Spec {
            spec, clause, step, ..
        } => match step {
            Some(s) => format!("{} `{clause}` step {s}", spec.name()),
            None => format!("{} `{clause}`", spec.name()),
        },
        Anchor::Izcal { reason, .. } => format!("izcal: {reason}"),
    }
}

fn anchor_cols(a: &Anchor, pins: &Pins) -> (String, String, String, String, &'static str) {
    match a {
        Anchor::Spec {
            spec, clause, step, ..
        } => (
            spec.name().to_string(),
            pins.revision.get(spec.name()).cloned().unwrap_or_default(),
            clause.clone(),
            step.clone().unwrap_or_default(),
            "spec",
        ),
        Anchor::Izcal { reason, .. } => (
            "izcal".into(),
            String::new(),
            reason.clone(),
            String::new(),
            "izcal",
        ),
    }
}

fn str_list(v: impl IntoIterator<Item = String>) -> String {
    let parts: Vec<String> = v.into_iter().map(|s| format!("{s:?}")).collect();
    format!("&[{}]", parts.join(", "))
}

fn effects_rust(e: &Effects) -> String {
    format!(
        "EffectFacts {{ alloc: {}, alloc_internal: {}, reads: {}, writes: {}, throws: {}, call_user: {}, host: {}, input: {}, interrupt: {}, gc: {} }}",
        str_list(e.alloc.iter().cloned()),
        e.alloc_internal,
        str_list(e.reads.iter().map(ToString::to_string)),
        str_list(e.writes.iter().map(ToString::to_string)),
        str_list(e.throws.iter().cloned()),
        e.call_user,
        str_list(e.host.iter().cloned()),
        str_list(e.input.iter().map(|i| input_name(*i).to_string())),
        e.interrupt,
        e.gc()
    )
}

struct ItemRow<'a> {
    kind: &'static str,
    name: String,
    opcode: Option<u8>,
    operands: Vec<String>,
    acc: &'static str,
    after_call: &'static str,
    declared: &'a Effects,
    facts: &'a ItemFacts,
    body: &'a Body,
    flow: &'a ItemFlow,
}

fn rows<'a>(m: &'a Model, flows: &'a Flows) -> Vec<ItemRow<'a>> {
    let mut out = Vec::new();
    for (i, op) in m.ops.iter().enumerate() {
        out.push(ItemRow {
            kind: "op",
            name: op.name.clone(),
            opcode: Some(op.opcode),
            operands: op
                .operands
                .iter()
                .map(|(n, k)| format!("{n}: {}", k.text()))
                .collect(),
            acc: op.acc.name(),
            after_call: after_text(op.after_call),
            declared: &op.effects,
            facts: &op.facts,
            body: &op.body,
            flow: &flows.ops[i],
        });
    }
    for (i, f) in m.intrinsics.iter().enumerate() {
        out.push(ItemRow {
            kind: "intrinsic",
            name: format!("%{}", f.name),
            opcode: None,
            operands: f
                .sig
                .params
                .iter()
                .map(|(n, t)| format!("{n}: {t}"))
                .collect(),
            acc: "out",
            after_call: after_text(f.sig.after_call),
            declared: &f.sig.effects,
            facts: &f.sig.facts,
            body: &f.body,
            flow: &flows.intrinsics[i],
        });
    }
    for (i, f) in m.helpers.iter().enumerate() {
        out.push(ItemRow {
            kind: "helper",
            name: f.name.clone(),
            opcode: None,
            operands: f
                .sig
                .params
                .iter()
                .map(|(n, t)| format!("{n}: {t}"))
                .collect(),
            acc: "none",
            after_call: "none",
            declared: &f.sig.effects,
            facts: &f.sig.facts,
            body: &f.body,
            flow: &flows.helpers[i],
        });
    }
    out
}

/// `crates/hg-js-ops/src/generated/facts.rs`.
pub fn emit_facts_rs(m: &Model, flows: &Flows, header: &str) -> String {
    let mut o = String::from(header);
    o.push_str(
        "//! The operation facts (language note Section 7): one record per\n\
         //! operation, intrinsic and helper, and one per path. T2's effect\n\
         //! lattice, the debugger's side-effect-free evaluation and the T0\n\
         //! generator read these, so none of them restates an effect by hand.\n\n\
         use crate::facts_types::{CategoryFacts, EffectFacts, ItemFacts, PathFacts};\n\n",
    );
    o.push_str("/// The heap category tree (R0-O12): name, parent, changed at GC points.\npub static CATEGORIES: &[CategoryFacts] = &[\n");
    for c in &m.cats.cats {
        let parent = c
            .parent
            .map(|p| format!("Some({:?})", m.cats.cats[p].name))
            .unwrap_or_else(|| "None".into());
        let _ = writeln!(
            o,
            "    CategoryFacts {{ name: {:?}, parent: {parent}, gc_mutable: {} }},",
            c.name, c.gc_mutable
        );
    }
    o.push_str("];\n\n/// Fact kinds and the categories that invalidate them.\npub static FACT_KINDS: &[(&str, &[&str])] = &[\n");
    for f in &m.fact_kinds {
        let _ = writeln!(
            o,
            "    ({:?}, {}),",
            f.name,
            str_list(f.on.iter().map(ToString::to_string))
        );
    }
    o.push_str("];\n\n");
    let all = rows(m, flows);
    for (label, kind) in [
        ("OPS", "op"),
        ("INTRINSICS", "intrinsic"),
        ("HELPERS", "helper"),
    ] {
        let these: Vec<&ItemRow<'_>> = all.iter().filter(|r| r.kind == kind).collect();
        let _ = writeln!(
            o,
            "/// The facts of every {kind}, in declaration order.\npub static {label}: &[ItemFacts] = &["
        );
        for r in these {
            item_rust(&mut o, r);
        }
        o.push_str("];\n\n");
    }
    o
}

fn item_rust(o: &mut String, r: &ItemRow<'_>) {
    let f = r.flow;
    let inferred = &f.effects;
    let control: Vec<String> = f.control.iter().map(|c| (*c).to_string()).collect();
    let opcode = r
        .opcode
        .map_or_else(|| "None".to_string(), |b| format!("Some({b:#04x})"));
    let _ = writeln!(o, "    ItemFacts {{");
    let _ = writeln!(
        o,
        "        kind: {:?},\n        name: {:?},\n        opcode: {opcode},",
        r.kind, r.name
    );
    let _ = writeln!(
        o,
        "        operands: {},\n        acc: {:?},\n        after_call: {:?},",
        str_list(r.operands.clone()),
        r.acc,
        r.after_call
    );
    let _ = writeln!(o, "        control: {},", str_list(control));
    let _ = writeln!(o, "        declared: {},", effects_rust(r.declared));
    let _ = writeln!(o, "        effects: {},", effects_rust(inferred));
    let _ = writeln!(
        o,
        "        terminator: {},",
        !f.control.contains("next") && r.kind == "op"
    );
    let _ = writeln!(o, "        debugger: {:?},", inferred.debugger_class());
    let _ = writeln!(
        o,
        "        invalidates: {},\n        results: {},\n        transfer: {:?},\n        smt: {:?},\n        frame_charge: {:?},",
        str_list(r.facts.invalidates.clone()),
        str_list(r.facts.results.clone()),
        r.facts.transfer,
        r.facts.smt,
        r.facts.frame_charge
    );
    let _ = writeln!(o, "        paths: &[");
    for (i, p) in r.body.paths.iter().enumerate() {
        let e = &f.path_effects[i];
        let pure = e.is_empty();
        let foldable = pure;
        let gcs: Vec<String> = f.path_gc[i]
            .iter()
            .map(|(w, l)| format!("({w:?}, {l:?})"))
            .collect();
        let spec = p
            .speculate
            .as_ref()
            .map(|(_, pr)| format!("Some({pr:?})"))
            .unwrap_or_else(|| "None".into());
        let parent = p
            .parent
            .map_or_else(|| "None".to_string(), |x| format!("Some({x})"));
        let _ = writeln!(
            o,
            "            PathFacts {{ name: {:?}, parent: {parent}, guard: {:?}, speculate: {spec}, effects: {}, gc_points: &[{}], units: {}, pure: {pure}, foldable: {foldable}, calls: {}, anchor: {:?} }},",
            p.name,
            p.guard,
            effects_rust(e),
            gcs.join(", "),
            str_list(f.path_units[i].iter().cloned()),
            str_list(f.path_calls[i].iter().cloned()),
            anchor_short(&p.anchor)
        );
    }
    let _ = writeln!(o, "        ],\n    }},");
}

fn tsv_cell(s: &str) -> String {
    s.replace(['\t', '\n'], " ")
}

/// `facts.tsv`, for review diffs: one row per item, then one per path.
pub fn emit_facts_tsv(m: &Model, flows: &Flows, header: &str) -> String {
    let mut o = String::new();
    for line in header.lines() {
        let _ = writeln!(o, "#{}", line.trim_start_matches("//"));
    }
    o.push_str("# row\tkind\titem\topcode\tpath\tguard\tspeculate\teffects\tgc_points\tunits\tcalls\tcontrol\tdebugger\tanchor\n");
    for r in rows(m, flows) {
        let f = r.flow;
        let control: Vec<&str> = f.control.iter().copied().collect();
        let _ = writeln!(
            o,
            "item\t{}\t{}\t{}\t\t\t\t{}\t\t\t\t{}\t{}\t{}",
            r.kind,
            r.name,
            r.opcode.map_or_else(String::new, |b| format!("{b:#04x}")),
            tsv_cell(&f.effects.text()),
            control.join(" "),
            f.effects.debugger_class(),
            tsv_cell(&anchor_short(&r.body.paths[0].anchor))
        );
        for (i, p) in r.body.paths.iter().enumerate() {
            let gcs: Vec<String> = f.path_gc[i]
                .iter()
                .map(|(w, l)| {
                    if l.is_empty() {
                        w.clone()
                    } else {
                        format!("{w} [{l}]")
                    }
                })
                .collect();
            let units: Vec<String> = f.path_units[i].iter().cloned().collect();
            let calls: Vec<String> = f.path_calls[i].iter().cloned().collect();
            let _ = writeln!(
                o,
                "path\t{}\t{}\t\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t\t\t{}",
                r.kind,
                r.name,
                p.name,
                tsv_cell(&p.guard),
                p.speculate.as_ref().map_or("", |s| s.1.as_str()),
                tsv_cell(&f.path_effects[i].text()),
                tsv_cell(&gcs.join("; ")),
                tsv_cell(&units.join("; ")),
                calls.join(" "),
                tsv_cell(&anchor_short(&p.anchor))
            );
        }
    }
    o
}

/// `anchors.tsv`: one entry per path (language note Section 7).
pub fn emit_anchors_tsv(m: &Model, pins: &Pins, header: &str) -> String {
    let mut o = String::new();
    for line in header.lines() {
        let _ = writeln!(o, "#{}", line.trim_start_matches("//"));
    }
    o.push_str("# item_kind\titem\tpath\tspec\trevision\tclause\tstep\tanchor_kind\town\n");
    for (kind, item, path, anchor, own) in crate::anchors::all_anchors(m) {
        let (spec, rev, clause, step, ak) = anchor_cols(&anchor, pins);
        let _ = writeln!(
            o,
            "{kind}\t{item}\t{path}\t{spec}\t{}\t{}\t{step}\t{ak}\t{}",
            tsv_cell(&rev),
            tsv_cell(&clause),
            if own { "own" } else { "inherited" }
        );
    }
    o
}
