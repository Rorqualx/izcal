//! Writes the Rust for a checked file. The output is passed through
//! rustfmt by the caller, so this only has to be valid, deterministic
//! Rust.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::check::{Checked, TypeFacts, pascal_of};
use crate::syntax::{Binding, Case, Field, Item, Named, Overflow, Type};

/// The largest input, in bytes, each round-trip harness explores. Kani's
/// bound is stated with the result (security scope 6.6, "What VER-1
/// proves").
pub const KANI_INPUT_BYTES: u128 = 24;

/// The largest input, in bytes, each validator harness explores. The
/// validator allocates nothing, so it proves at larger sizes.
pub const KANI_VALIDATE_BYTES: u128 = 64;

struct Out {
    s: String,
    /// The part harnesses already written (by slug), so a type shared by
    /// several split messages is proved once.
    parts: BTreeSet<String>,
}

impl Out {
    fn line(&mut self, text: &str) {
        self.s.push_str(text);
        self.s.push('\n');
    }

    fn doc(&mut self, n: &Named) {
        for d in &n.doc {
            if d.is_empty() {
                self.line("///");
            } else {
                self.line(&format!("/// {d}"));
            }
        }
    }
}

fn rust_type(t: &Type) -> String {
    match t {
        Type::Prim(p) => p.rust().to_owned(),
        Type::Bytes(n) => format!("hg_ipc::BoundedBytes<{n}>"),
        Type::Str(n) => format!("hg_ipc::BoundedString<{n}>"),
        Type::Vec(inner, n) => format!("hg_ipc::BoundedVec<{}, {n}>", rust_type(inner)),
        Type::Option(inner) => format!("Option<{}>", rust_type(inner)),
        Type::Named(name, _) => name.clone(),
        Type::Remote(name, _) => format!("hg_ipc::Remote<{name}>"),
        Type::Receiver(name, _) => format!("hg_ipc::Endpoint<{name}>"),
    }
}

/// A parameter's Rust type: as [`rust_type`], except that a bound
/// interface's sender travels as a `Grant` (IPC-R2 (a)).
fn param_type(t: &Type, bound: &BTreeSet<String>) -> String {
    match t {
        Type::Remote(name, _) if bound.contains(name) => format!("hg_ipc::Grant<{name}>"),
        _ => rust_type(t),
    }
}

/// The handle kind a handle parameter crosses as: a bound interface's
/// sender is its own kind (IPC-R1, R2).
fn handle_kind(t: &Type, bound: &BTreeSet<String>) -> &'static str {
    match t {
        Type::Receiver(..) => "hg_ipc::HandleKind::ChannelReceiver",
        Type::Remote(name, _) if bound.contains(name) => "hg_ipc::HandleKind::BoundChannelSender",
        _ => "hg_ipc::HandleKind::ChannelSender",
    }
}

/// The expression taking a handle parameter from the iterator `h`.
fn handle_take(t: &Type, bound: &BTreeSet<String>) -> String {
    match t {
        Type::Remote(name, _) if bound.contains(name) => {
            format!("hg_ipc::endpoint::take_grant::<{name}>(h.next())?")
        }
        Type::Remote(name, _) => format!("hg_ipc::endpoint::take_remote::<{name}>(h.next(), ipc)?"),
        Type::Receiver(name, _) => {
            format!("hg_ipc::endpoint::take_endpoint::<{name}>(h.next(), ipc)?")
        }
        _ => String::new(),
    }
}

/// `<T as Wire>::MIN_LEN + …` for fields, as a const expression.
fn fields_len(fields: &[Field], which: &str) -> String {
    let mut e = "0".to_owned();
    for f in fields {
        e = format!(
            "hg_ipc::wire::len_add({e}, <{} as Wire>::{which})",
            rust_type(&f.ty)
        );
    }
    e
}

fn cases_len(cases: &[Case], tag: usize, which: &str, fold: &str) -> String {
    let mut parts = cases.iter().map(|c| fields_len(&c.fields, which));
    let first = parts.next().unwrap_or_else(|| "0".to_owned());
    let folded = parts.fold(first, |acc, p| format!("hg_ipc::wire::{fold}({acc}, {p})"));
    format!("hg_ipc::wire::len_add({tag}, {folded})")
}

fn derives(f: &TypeFacts) -> &'static str {
    if f.is_plain_enum {
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]"
    } else if f.has_float {
        "#[derive(Debug, Clone, PartialEq)]"
    } else {
        "#[derive(Debug, Clone, PartialEq, Eq)]"
    }
}

fn field_list(o: &mut Out, fields: &[Field], public: bool) {
    for f in fields {
        o.doc(&f.id);
        let vis = if public { "pub " } else { "" };
        o.line(&format!("{vis}{}: {},", f.id.name, rust_type(&f.ty)));
    }
}

fn binding_names(fields: &[Field]) -> String {
    fields
        .iter()
        .map(|f| f.id.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// `()`, `(a,)` or `(a, b)`: a tuple of the fields' names or types.
fn tuple(items: &[String]) -> String {
    match items {
        [] => "()".to_owned(),
        [one] => format!("({one},)"),
        _ => format!("({})", items.join(", ")),
    }
}

fn field_names(fields: &[Field]) -> Vec<String> {
    fields.iter().map(|f| f.id.name.clone()).collect()
}

/// `<T as Wire>::validate(r)?;` for each field, in order.
fn validate_fields(o: &mut Out, fields: &[Field]) {
    for f in fields {
        o.line(&format!("<{} as Wire>::validate(r)?;", rust_type(&f.ty)));
    }
}

fn decode_fields(fields: &[Field]) -> String {
    fields
        .iter()
        .map(|f| format!("{}: Wire::decode(r)?", f.id.name))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Emits a tagged type: an enum, a union or an interface's messages.
///
/// With `bodies`, each case's fields are read by its own `decode_<case>`
/// and written by its own `encode_<case>`, which the `Wire` impl calls
/// after the tag; a Kani harness can then reach one message without the
/// solver exploring the others.
fn tagged(
    o: &mut Out,
    name: &str,
    cases: &[(String, &[Field])],
    tag_ty: &str,
    unknown: &str,
    bodies: bool,
    wire: bool,
) {
    if bodies {
        o.line(&format!("impl {name} {{"));
        for (case, fields) in cases {
            let m = snake_of(case);
            let types: Vec<String> = fields.iter().map(|f| rust_type(&f.ty)).collect();
            o.line(&format!(
                "/// Reads [`Self::{case}`]'s fields, after its ordinal."
            ));
            o.line(&format!(
                "fn decode_{m}(r: &mut Reader<'_>) -> Result<{}, BadMessage> {{",
                tuple(&types)
            ));
            if fields.is_empty() {
                o.line("let _ = r;");
                o.line("Ok(())");
            } else {
                let reads: Vec<String> = fields
                    .iter()
                    .map(|_| "Wire::decode(r)?".to_owned())
                    .collect();
                o.line(&format!("Ok({})", tuple(&reads)));
            }
            o.line("}");
            o.line(&format!(
                "/// Checks [`Self::{case}`]'s fields, after its ordinal."
            ));
            o.line(&format!(
                "fn validate_{m}(r: &mut Reader<'_>) -> Result<(), BadMessage> {{"
            ));
            if fields.is_empty() {
                o.line("let _ = r;");
            }
            validate_fields(o, fields);
            o.line("Ok(())");
            o.line("}");
            o.line(&format!(
                "/// Writes [`Self::{case}`]'s fields, after its ordinal."
            ));
            let params: String = fields
                .iter()
                .zip(&types)
                .map(|(f, t)| format!(", {}: &{t}", f.id.name))
                .collect();
            o.line(&format!("fn encode_{m}(w: &mut Writer{params}) {{"));
            if fields.is_empty() {
                o.line("let _ = w;");
            }
            for f in *fields {
                o.line(&format!("{}.encode(w);", f.id.name));
            }
            o.line("}");
        }
        o.line("}");
        o.line("");
    }
    if !wire {
        return;
    }
    o.line(&format!("impl Wire for {name} {{"));
    let _ = (name, tag_ty);
    // MIN_LEN / MAX_LEN are written by the caller.
    o.line("fn encode(&self, w: &mut Writer) {");
    o.line("match self {");
    for (i, (case, fields)) in cases.iter().enumerate() {
        if bodies {
            let names = binding_names(fields);
            let pattern = if fields.is_empty() {
                format!("Self::{case}")
            } else {
                format!("Self::{case} {{ {names} }}")
            };
            let args: String = fields.iter().map(|f| format!(", {}", f.id.name)).collect();
            o.line(&format!(
                "{pattern} => {{ {i}{tag_ty}.encode(w); Self::encode_{}(w{args}); }}",
                snake_of(case)
            ));
        } else if fields.is_empty() {
            o.line(&format!("Self::{case} => {{ {i}{tag_ty}.encode(w); }}"));
        } else {
            o.line(&format!(
                "Self::{case} {{ {} }} => {{",
                binding_names(fields)
            ));
            o.line(&format!("{i}{tag_ty}.encode(w);"));
            for f in *fields {
                o.line(&format!("{}.encode(w);", f.id.name));
            }
            o.line("}");
        }
    }
    o.line("}");
    o.line("}");
    o.line("fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {");
    o.line(&format!("match {tag_ty}::decode(r)? {{"));
    for (i, (case, fields)) in cases.iter().enumerate() {
        if bodies {
            let names = field_names(fields);
            let build = if fields.is_empty() {
                format!("Self::{case}")
            } else {
                format!("Self::{case} {{ {} }}", names.join(", "))
            };
            o.line(&format!(
                "{i} => {{ let {} = Self::decode_{}(r)?; Ok({build}) }}",
                tuple(&names),
                snake_of(case)
            ));
        } else if fields.is_empty() {
            o.line(&format!("{i} => Ok(Self::{case}),"));
        } else {
            o.line(&format!(
                "{i} => Ok(Self::{case} {{ {} }}),",
                decode_fields(fields)
            ));
        }
    }
    o.line(&format!("_ => Err(BadMessage::{unknown}),"));
    o.line("}");
    o.line("}");
    o.line("fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {");
    o.line(&format!("match {tag_ty}::decode(r)? {{"));
    for (i, (case, fields)) in cases.iter().enumerate() {
        if bodies {
            o.line(&format!("{i} => Self::validate_{}(r),", snake_of(case)));
        } else if fields.is_empty() {
            o.line(&format!("{i} => Ok(()),"));
        } else {
            o.line(&format!("{i} => {{"));
            validate_fields(o, fields);
            o.line("Ok(())");
            o.line("}");
        }
    }
    o.line(&format!("_ => Err(BadMessage::{unknown}),"));
    o.line("}");
    o.line("}");
}

fn sample_fields(fields: &[Field]) -> String {
    fields
        .iter()
        .map(|f| format!("{}: Sample::sample(g)", f.id.name))
        .collect::<Vec<_>>()
        .join(", ")
}

fn sample_tagged(o: &mut Out, name: &str, cases: &[(String, &[Field])]) {
    o.line("#[cfg(test)]");
    o.line(&format!("impl crate::sample::Sample for {name} {{"));
    o.line("fn sample(g: &mut crate::sample::Gen) -> Self {");
    if cases.iter().any(|(_, fields)| !fields.is_empty()) {
        o.line("use crate::sample::Sample;");
    }
    // One case: no choice to make, and a match with only a wildcard arm
    // would be clippy's match_single_binding.
    if let [(case, fields)] = cases {
        if fields.is_empty() {
            o.line(&format!("let _ = g; Self::{case}"));
        } else {
            o.line(&format!("Self::{case} {{ {} }}", sample_fields(fields)));
        }
        o.line("}");
        o.line("}");
        return;
    }
    o.line(&format!("match g.below({}) {{", cases.len()));
    let last = cases.len().saturating_sub(1);
    for (i, (case, fields)) in cases.iter().enumerate() {
        let arm = if i == last {
            "_".to_owned()
        } else {
            i.to_string()
        };
        if fields.is_empty() {
            o.line(&format!("{arm} => Self::{case},"));
        } else {
            o.line(&format!(
                "{arm} => Self::{case} {{ {} }},",
                sample_fields(fields)
            ));
        }
    }
    o.line("}");
    o.line("}");
    o.line("}");
}

/// The Rust source for `checked`, before rustfmt. `source_label` names
/// the input in the header.
pub fn emit(checked: &Checked, source_label: &str) -> String {
    let file = &checked.file;
    let mut o = Out {
        s: String::new(),
        parts: BTreeSet::new(),
    };
    o.line(&format!(
        "// @generated by hg-ipc-codegen from {source_label}. Do not edit: change the"
    ));
    o.line("// .hgidl file and run `cargo run -p hg-ipc-codegen` (coding standard CS-ST-17).");
    for d in &file.protocol.doc {
        o.line(format!("//! {d}").trim_end());
    }
    o.line("");
    o.line("use hg_ipc::wire::{BadMessage, Reader, Wire, Writer};");
    o.line("");
    let bound: BTreeSet<String> = file
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Interface {
                id,
                binding: Binding::Bound,
                ..
            } => Some(id.name.clone()),
            _ => None,
        })
        .collect();
    let error_name = file
        .items
        .iter()
        .find_map(|i| match i {
            Item::Error(id, _) => Some(id.name.clone()),
            _ => None,
        })
        .unwrap_or_default();

    for item in &file.items {
        match item {
            Item::Error(id, vs) | Item::Enum(id, vs) => {
                let facts = checked.facts.get(&id.name).copied();
                o.doc(id);
                o.line(derives(&facts.unwrap_or(TypeFacts {
                    size: crate::check::Size { min: 4, max: 4 },
                    has_float: false,
                    is_plain_enum: true,
                    max_loop: 0,
                    heap_depth: 0,
                })));
                o.line(&format!("pub enum {} {{", id.name));
                for v in vs {
                    o.doc(v);
                    o.line(&format!("{},", v.name));
                }
                o.line("}");
                o.line("");
                let cases: Vec<(String, &[Field])> =
                    vs.iter().map(|v| (v.name.clone(), &[][..])).collect();
                tagged(&mut o, &id.name, &cases, "u32", "UnknownTag", false, true);
                o.line("const MIN_LEN: usize = 4;");
                o.line("const MAX_LEN: usize = 4;");
                o.line("}");
                o.line("");
                sample_tagged(&mut o, &id.name, &cases);
                o.line("");
            }
            Item::Union(id, cs) => {
                let facts = checked.facts.get(&id.name).copied();
                o.doc(id);
                if let Some(f) = facts {
                    o.line(derives(&f));
                }
                o.line(&format!("pub enum {} {{", id.name));
                for c in cs {
                    o.doc(&c.id);
                    if c.fields.is_empty() {
                        o.line(&format!("{},", c.id.name));
                    } else {
                        o.line(&format!("{} {{", c.id.name));
                        field_list(&mut o, &c.fields, false);
                        o.line("},");
                    }
                }
                o.line("}");
                o.line("");
                let cases: Vec<(String, &[Field])> = cs
                    .iter()
                    .map(|c| (c.id.name.clone(), c.fields.as_slice()))
                    .collect();
                tagged(&mut o, &id.name, &cases, "u32", "UnknownTag", false, true);
                o.line(&format!(
                    "const MIN_LEN: usize = {};",
                    cases_len(cs, 4, "MIN_LEN", "len_min")
                ));
                o.line(&format!(
                    "const MAX_LEN: usize = {};",
                    cases_len(cs, 4, "MAX_LEN", "len_max")
                ));
                o.line("}");
                o.line("");
                sample_tagged(&mut o, &id.name, &cases);
                o.line("");
            }
            Item::Struct(id, fields) => {
                let facts = checked.facts.get(&id.name).copied();
                o.doc(id);
                if let Some(f) = facts {
                    o.line(derives(&f));
                }
                o.line(&format!("pub struct {} {{", id.name));
                field_list(&mut o, fields, true);
                o.line("}");
                o.line("");
                o.line(&format!("impl Wire for {} {{", id.name));
                o.line(&format!(
                    "const MIN_LEN: usize = {};",
                    fields_len(fields, "MIN_LEN")
                ));
                o.line(&format!(
                    "const MAX_LEN: usize = {};",
                    fields_len(fields, "MAX_LEN")
                ));
                o.line("fn encode(&self, w: &mut Writer) {");
                for f in fields {
                    o.line(&format!("self.{}.encode(w);", f.id.name));
                }
                o.line("}");
                o.line("fn decode(r: &mut Reader<'_>) -> Result<Self, BadMessage> {");
                o.line(&format!("Ok(Self {{ {} }})", decode_fields(fields)));
                o.line("}");
                o.line("fn validate(r: &mut Reader<'_>) -> Result<(), BadMessage> {");
                validate_fields(&mut o, fields);
                o.line("Ok(())");
                o.line("}");
                o.line("}");
                o.line("");
                o.line("#[cfg(test)]");
                o.line(&format!("impl crate::sample::Sample for {} {{", id.name));
                o.line("fn sample(g: &mut crate::sample::Gen) -> Self {");
                o.line("use crate::sample::Sample;");
                o.line(&format!("Self {{ {} }}", sample_fields(fields)));
                o.line("}");
                o.line("}");
                o.line("");
            }
            Item::Interface {
                id,
                binding,
                capacity,
                overflow,
                messages,
            } => {
                let ctx = Iface {
                    protocol: &file.protocol.name,
                    name: &id.name,
                    bound: &bound,
                    error: &error_name,
                    is_bound: *binding == Binding::Bound,
                };
                let has_handles = messages
                    .iter()
                    .any(|m| m.fields.iter().any(|f| f.ty.is_handle()));
                let data: Vec<Case> = messages
                    .iter()
                    .map(|m| Case {
                        id: m.id.clone(),
                        fields: m
                            .fields
                            .iter()
                            .filter(|f| !f.ty.is_handle())
                            .cloned()
                            .collect(),
                    })
                    .collect();
                let has_float = messages
                    .iter()
                    .any(|m| m.fields.iter().any(|f| field_has_float(&f.ty, checked)));
                o.doc(id);
                o.line("///");
                o.line(&format!(
                    "/// Interface `{}.{}`: {}, capacity {capacity}, overflow `{}`.",
                    file.protocol.name,
                    id.name,
                    match binding {
                        Binding::Bound => "bound",
                        Binding::Free => "free",
                    },
                    match overflow {
                        Overflow::Drop => "drop",
                        Overflow::Terminate => "terminate",
                    }
                ));
                if has_handles {
                    // Handles are owned capabilities: no Clone or PartialEq.
                    o.line("#[derive(Debug)]");
                } else {
                    o.line(derives(&TypeFacts {
                        size: crate::check::Size { min: 0, max: 0 },
                        has_float,
                        is_plain_enum: false,
                        max_loop: 0,
                        heap_depth: 0,
                    }));
                }
                o.line(&format!("pub enum {} {{", id.name));
                for m in messages {
                    o.doc(&m.id);
                    let v = pascal_of(&m.id.name);
                    if m.fields.is_empty() {
                        o.line(&format!("{v},"));
                    } else {
                        o.line(&format!("{v} {{"));
                        for f in &m.fields {
                            o.doc(&f.id);
                            o.line(&format!("{}: {},", f.id.name, param_type(&f.ty, &bound)));
                        }
                        o.line("},");
                    }
                }
                o.line("}");
                o.line("");
                o.line(&format!("impl {} {{", id.name));
                o.line("/// What the `.hgidl` declared about this interface.");
                o.line("pub const INFO: hg_ipc::InterfaceInfo = hg_ipc::InterfaceInfo {");
                o.line(&format!("name: \"{}.{}\",", file.protocol.name, id.name));
                o.line(&format!(
                    "binding: hg_ipc::Binding::{},",
                    match binding {
                        Binding::Bound => "Bound",
                        Binding::Free => "Free",
                    }
                ));
                o.line(&format!("capacity: {capacity},"));
                o.line(&format!(
                    "overflow: hg_ipc::Overflow::{},",
                    match overflow {
                        Overflow::Drop => "Drop",
                        Overflow::Terminate => "Terminate",
                    }
                ));
                o.line(&format!(
                    "max_message_len: {},",
                    cases_len(&data, 2, "MAX_LEN", "len_max")
                ));
                o.line("};");
                o.line("}");
                o.line("");
                let cases: Vec<(String, &[Field])> = data
                    .iter()
                    .map(|m| (pascal_of(&m.id.name), m.fields.as_slice()))
                    .collect();
                tagged(
                    &mut o,
                    &id.name,
                    &cases,
                    "u16",
                    "UnknownOrdinal",
                    true,
                    !has_handles,
                );
                if !has_handles {
                    o.line(&format!(
                        "const MIN_LEN: usize = {};",
                        cases_len(&data, 2, "MIN_LEN", "len_min")
                    ));
                    o.line(&format!(
                        "const MAX_LEN: usize = {};",
                        cases_len(&data, 2, "MAX_LEN", "len_max")
                    ));
                    o.line("}");
                    o.line("");
                }
                interface_impl(&mut o, &ctx, messages);
                handler_trait(&mut o, &ctx, messages);
                sender_trait(&mut o, &ctx, messages);
                if !has_handles {
                    sample_tagged(&mut o, &id.name, &cases);
                    o.line("");
                }
                kani(&mut o, &file.protocol.name, &id.name, &data, checked);
            }
        }
    }
    o.s
}

/// What the per-interface emitters need to know.
struct Iface<'a> {
    protocol: &'a str,
    name: &'a str,
    bound: &'a BTreeSet<String>,
    error: &'a str,
    is_bound: bool,
}

/// `validate_bytes`, the `hg_ipc::Interface` impl (the message's wire
/// form: bytes plus handles in declaration order) and the free/bound
/// marker.
fn interface_impl(o: &mut Out, x: &Iface<'_>, messages: &[Case]) {
    let name = x.name;
    o.line(&format!("impl {name} {{"));
    o.line("/// Checks that `bytes` are one valid message, allocating nothing.");
    o.line("///");
    o.line("/// # Errors");
    o.line("/// [`BadMessage`] if they are not.");
    o.line("pub fn validate_bytes(bytes: &[u8]) -> Result<(), BadMessage> {");
    o.line("let mut r = Reader::new(bytes);");
    o.line("match u16::decode(&mut r)? {");
    for (i, m) in messages.iter().enumerate() {
        o.line(&format!(
            "{i} => Self::validate_{}(&mut r)?,",
            snake_of(&pascal_of(&m.id.name))
        ));
    }
    o.line("_ => return Err(BadMessage::UnknownOrdinal),");
    o.line("}");
    o.line("r.finish()");
    o.line("}");
    o.line("}");
    o.line("");

    let any_handles = messages
        .iter()
        .any(|m| m.fields.iter().any(|f| f.ty.is_handle()));
    o.line(&format!("impl hg_ipc::Interface for {name} {{"));
    o.line("const INFO: hg_ipc::InterfaceInfo = Self::INFO;");
    o.line("fn into_wire(self) -> (Vec<u8>, Vec<hg_ipc::Handle>) {");
    o.line("let mut w = Writer::new();");
    if any_handles {
        o.line("let mut handles = Vec::new();");
    }
    o.line("match self {");
    for (i, m) in messages.iter().enumerate() {
        let v = pascal_of(&m.id.name);
        let f = snake_of(&v);
        let all = binding_names(&m.fields);
        let pattern = if m.fields.is_empty() {
            format!("Self::{v}")
        } else {
            format!("Self::{v} {{ {all} }}")
        };
        let args: String = m
            .fields
            .iter()
            .filter(|p| !p.ty.is_handle())
            .map(|p| format!(", &{}", p.id.name))
            .collect();
        o.line(&format!("{pattern} => {{"));
        o.line(&format!("{i}u16.encode(&mut w);"));
        o.line(&format!("Self::encode_{f}(&mut w{args});"));
        for p in m.fields.iter().filter(|p| p.ty.is_handle()) {
            o.line(&format!("handles.push({}.into_handle());", p.id.name));
        }
        o.line("}");
    }
    o.line("}");
    if any_handles {
        o.line("(w.into_bytes(), handles)");
    } else {
        o.line("(w.into_bytes(), Vec::new())");
    }
    o.line("}");
    o.line("fn from_wire(");
    o.line("bytes: &[u8],");
    o.line("handles: Vec<hg_ipc::Handle>,");
    o.line("ipc: &std::sync::Arc<dyn hg_ipc::Ipc>,");
    o.line(") -> Result<Self, BadMessage> {");
    if !any_handles {
        o.line("let _ = ipc;");
    }
    o.line("// Every check before any decoding: the bytes by the validator,");
    o.line("// which allocates nothing, then the handles' kinds (IPC-R1).");
    o.line("Self::validate_bytes(bytes)?;");
    o.line("let mut r = Reader::new(bytes);");
    o.line("let msg = match u16::decode(&mut r)? {");
    for (i, m) in messages.iter().enumerate() {
        let v = pascal_of(&m.id.name);
        let f = snake_of(&v);
        let kinds: Vec<&str> = m
            .fields
            .iter()
            .filter(|p| p.ty.is_handle())
            .map(|p| handle_kind(&p.ty, x.bound))
            .collect();
        let data: Vec<String> = m
            .fields
            .iter()
            .filter(|p| !p.ty.is_handle())
            .map(|p| p.id.name.clone())
            .collect();
        o.line(&format!("{i} => {{"));
        o.line(&format!(
            "hg_ipc::endpoint::expect_kinds(handles.iter().map(hg_ipc::Handle::kind), &[{}])?;",
            kinds.join(", ")
        ));
        o.line(&format!(
            "let {} = Self::decode_{f}(&mut r)?;",
            tuple(&data)
        ));
        if kinds.is_empty() {
            if m.fields.is_empty() {
                o.line(&format!("Self::{v}"));
            } else {
                o.line(&format!("Self::{v} {{ {} }}", binding_names(&m.fields)));
            }
        } else {
            o.line("let mut h = handles.into_iter();");
            for p in m.fields.iter().filter(|p| p.ty.is_handle()) {
                o.line(&format!(
                    "let {} = {};",
                    p.id.name,
                    handle_take(&p.ty, x.bound)
                ));
            }
            o.line(&format!("Self::{v} {{ {} }}", binding_names(&m.fields)));
        }
        o.line("}");
    }
    o.line("_ => return Err(BadMessage::UnknownOrdinal),");
    o.line("};");
    o.line("r.finish()?;");
    o.line("Ok(msg)");
    o.line("}");
    o.line("}");
    o.line("");
    let marker = if x.is_bound {
        "BoundInterface"
    } else {
        "FreeInterface"
    };
    o.line(&format!("impl hg_ipc::{marker} for {name} {{}}"));
    o.line("");
}

/// The parameter list `name: Type, …` of a message, handles included.
fn params(m: &Case, bound: &BTreeSet<String>) -> String {
    m.fields
        .iter()
        .map(|f| format!(", {}: {}", f.id.name, param_type(&f.ty, bound)))
        .collect()
}

/// The handler trait, one method per message, and `dispatch`. A bound
/// interface's methods get the caller's identity (IPC-R2 (c)); a free
/// one's get none.
fn handler_trait(o: &mut Out, x: &Iface<'_>, messages: &[Case]) {
    let (name, error) = (x.name, x.error);
    let from = if x.is_bound {
        ", from: &hg_ipc::CallerIdentity"
    } else {
        ""
    };
    let from_arg = if x.is_bound { "from, " } else { "" };
    let _ = writeln!(
        o.s,
        "/// Receives `{}.{name}`'s messages, each already checked. An `Err`\n\
         /// closes the endpoint and reports the sender (IPC-R5).",
        x.protocol
    );
    o.line(&format!("pub trait {name}Handler {{"));
    for m in messages {
        o.doc(&m.id);
        o.line("///");
        o.line("/// # Errors");
        o.line("/// The protocol's error, which ends the conversation.");
        o.line(&format!(
            "fn {}(&mut self{from}{}) -> Result<(), {error}>;",
            m.id.name,
            params(m, x.bound)
        ));
    }
    o.line("}");
    o.line("");
    o.line(&format!("impl {name} {{"));
    o.line("/// Passes this message to its handler method.");
    o.line("///");
    o.line("/// # Errors");
    o.line("/// The handler's.");
    o.line(&format!(
        "pub fn dispatch<H: {name}Handler + ?Sized>(self, h: &mut H{from}) -> Result<(), {error}> {{"
    ));
    o.line("match self {");
    for m in messages {
        let v = pascal_of(&m.id.name);
        let all = binding_names(&m.fields);
        let pattern = if m.fields.is_empty() {
            format!("Self::{v}")
        } else {
            format!("Self::{v} {{ {all} }}")
        };
        let args = if m.fields.is_empty() {
            from_arg.trim_end_matches(", ").to_owned()
        } else {
            format!("{from_arg}{all}")
        };
        o.line(&format!("{pattern} => h.{}({args}),", m.id.name));
    }
    o.line("}");
    o.line("}");
    o.line("}");
    o.line("");
}

/// The sender trait on `hg_ipc::Remote<Iface>`, one method per message.
fn sender_trait(o: &mut Out, x: &Iface<'_>, messages: &[Case]) {
    let name = x.name;
    let _ = writeln!(
        o.s,
        "/// Sends `{}.{name}`'s messages; implemented for `hg_ipc::Remote<{name}>`.",
        x.protocol
    );
    o.line(&format!("pub trait {name}Remote {{"));
    for m in messages {
        o.doc(&m.id);
        o.line("///");
        o.line("/// # Errors");
        o.line("/// [`hg_ipc::SendError`]; the sender never blocks.");
        o.line(&format!(
            "fn {}(&self{}) -> Result<(), hg_ipc::SendError>;",
            m.id.name,
            params(m, x.bound)
        ));
    }
    o.line("}");
    o.line("");
    o.line(&format!("impl {name}Remote for hg_ipc::Remote<{name}> {{"));
    for m in messages {
        let v = pascal_of(&m.id.name);
        let built = if m.fields.is_empty() {
            format!("{name}::{v}")
        } else {
            format!("{name}::{v} {{ {} }}", binding_names(&m.fields))
        };
        o.line(&format!(
            "fn {}(&self{}) -> Result<(), hg_ipc::SendError> {{",
            m.id.name,
            params(m, x.bound)
        ));
        o.line(&format!("self.send({built})"));
        o.line("}");
    }
    o.line("}");
    o.line("");
}

fn field_has_float(t: &Type, checked: &Checked) -> bool {
    match t {
        Type::Prim(p) => p.is_float(),
        Type::Bytes(_) | Type::Str(_) => false,
        Type::Vec(inner, _) | Type::Option(inner) => field_has_float(inner, checked),
        Type::Named(n, _) => checked.facts.get(n).is_some_and(|f| f.has_float),
        Type::Remote(..) | Type::Receiver(..) => false,
    }
}

fn snake_of(pascal: &str) -> String {
    let mut s = String::new();
    for (i, c) in pascal.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                s.push('_');
            }
            s.push(c.to_ascii_lowercase());
        } else {
            s.push(c);
        }
    }
    s
}

/// Writes the validator harness for one message: on every body of up to
/// `KANI_VALIDATE_BYTES` bytes, validating never panics, overflows or
/// reads out of bounds (VER-1's three properties). The validator
/// allocates nothing, so this proves for nested messages too.
fn validator_harness(
    o: &mut Out,
    protocol: &str,
    rt: &RoundTrip<'_>,
    facts: &crate::check::MessageFacts,
    checked: &Checked,
) {
    let (iface, snake, name, body_fn, m) = (rt.iface, rt.snake, rt.name, rt.body_fn, rt.m);
    let size = facts.size;
    let k = crate::shape::harness_bytes(
        checked,
        m.fields.iter().map(|f| &f.ty),
        KANI_VALIDATE_BYTES,
        size.min,
        size.max,
    );
    let body = k.saturating_sub(2);
    let unwind = crate::shape::unwind_for(checked, m.fields.iter().map(|f| &f.ty), body);
    let _ = writeln!(
        o.s,
        "/// VER-1 for `{protocol}.{iface}.{name}` (IPC-R5): validating never\n\
         /// panics, overflows or reads out of bounds. Bounds: messages of 2 to\n\
         /// {k} bytes (its encodings are {} to {}); loops unwound {unwind} times.",
        size.min, size.max
    );
    o.line("#[cfg(kani)]");
    o.line("#[kani::proof]");
    o.line(&format!("#[kani::unwind({unwind})]"));
    o.line(&format!("fn verify_{snake}_{name}_validates() {{"));
    o.line(&format!("let bytes: [u8; {body}] = kani::any();"));
    o.line(&format!(
        "let len: usize = kani::any_where(|&n: &usize| n <= {body});"
    ));
    o.line("let Some(input) = bytes.get(..len) else { return };");
    o.line("let mut r = Reader::new(input);");
    o.line(&format!(
        "let checked = {iface}::validate_{body_fn}(&mut r).and_then(|()| r.finish());"
    ));
    o.line("kani::cover!(checked.is_ok(), \"some input is accepted\");");
    o.line("}");
    o.line("");
}

/// A harness name for a part type: its Rust type in lower snake case.
fn part_slug(t: &Type) -> String {
    let raw = rust_type(t).replace("hg_ipc::", "");
    let mut out = String::new();
    let mut prev_upper_or_sep = true;
    for c in raw.chars() {
        if c.is_ascii_uppercase() {
            if !prev_upper_or_sep {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
            prev_upper_or_sep = true;
        } else if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_upper_or_sep = false;
        } else {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
            prev_upper_or_sep = true;
        }
    }
    out.trim_end_matches('_').to_owned()
}

/// A part's validator harness: validating a `t` never panics on any input
/// of a message validator harness's size (see `shape`, "Proofs by part").
fn part_validator(o: &mut Out, checked: &Checked, t: &Type) {
    let slug = part_slug(t);
    if !o.parts.insert(format!("{slug}/validates")) {
        return;
    }
    let (min, max) = crate::shape::size_of(checked, t);
    let body = crate::shape::harness_bytes(
        checked,
        [t],
        KANI_VALIDATE_BYTES.saturating_sub(2),
        min,
        max,
    );
    let unwind = crate::shape::unwind_for(checked, [t], body);
    let ty = rust_type(t);
    let _ = writeln!(
        o.s,
        "/// VER-1 part `{ty}`: validating it never panics, overflows or reads\n\
         /// out of bounds, on every input of at most {body} bytes; loops\n\
         /// unwound {unwind} times."
    );
    o.line("#[cfg(kani)]");
    o.line("#[kani::proof]");
    o.line(&format!("#[kani::unwind({unwind})]"));
    o.line(&format!("fn verify_part_{slug}_validates() {{"));
    o.line(&format!("let bytes: [u8; {body}] = kani::any();"));
    o.line(&format!(
        "let len: usize = kani::any_where(|&n: &usize| n <= {body});"
    ));
    o.line("let Some(input) = bytes.get(..len) else { return };");
    o.line("let mut r = Reader::new(input);");
    o.line(&format!("let checked = <{ty} as Wire>::validate(&mut r);"));
    o.line("kani::cover!(checked.is_ok(), \"some input is accepted\");");
    o.line("}");
    o.line("");
}

/// A part's decoding and round-trip harnesses: decoding a `t` never
/// panics on any input of a round-trip harness's size, and every
/// accepted input of each layout re-encodes to the same bytes.
fn part_round_trip(o: &mut Out, checked: &Checked, t: &Type) {
    let slug = part_slug(t);
    if !o.parts.insert(format!("{slug}/round_trip")) {
        return;
    }
    let (min, max) = crate::shape::size_of(checked, t);
    let body =
        crate::shape::harness_bytes(checked, [t], KANI_INPUT_BYTES.saturating_sub(2), min, max);
    let unwind = crate::shape::unwind_for(checked, [t], body);
    let ty = rust_type(t);
    let header = |o: &mut Out, suffix: &str| {
        o.line("#[cfg(kani)]");
        o.line("#[kani::proof]");
        o.line(&format!("#[kani::unwind({unwind})]"));
        o.line(&format!("fn verify_part_{slug}_{suffix}() {{"));
    };
    let _ = writeln!(
        o.s,
        "/// VER-1 part `{ty}`: decoding it never panics, on every input of at\n\
         /// most {body} bytes; loops unwound {unwind} times."
    );
    header(o, "decodes");
    o.line(&format!("let bytes: [u8; {body}] = kani::any();"));
    o.line(&format!(
        "let len: usize = kani::any_where(|&n: &usize| n <= {body});"
    ));
    o.line("let Some(input) = bytes.get(..len) else { return };");
    o.line("let mut r = Reader::new(input);");
    o.line(&format!("let decoded = <{ty} as Wire>::decode(&mut r);"));
    o.line("kani::cover!(decoded.is_ok(), \"some input is accepted\");");
    o.line("}");
    o.line("");
    // A round trip over every length from `least` to `body`, with the
    // bytes in `fixed` fixed: for a value whose only variable piece is at
    // its end, no offset after it is symbolic.
    let tail_harness = |o: &mut Out, suffix: &str, fixed: &[(u128, u8)], least: u128| {
        header(o, suffix);
        if fixed.is_empty() {
            o.line(&format!("let bytes: [u8; {body}] = kani::any();"));
        } else {
            o.line(&format!("let mut bytes: [u8; {body}] = kani::any();"));
            for (at, v) in fixed {
                o.line(&format!(
                    "if let Some(b) = bytes.get_mut({at}) {{ *b = {v}; }}"
                ));
            }
        }
        o.line(&format!(
            "let len: usize = kani::any_where(|&n: &usize| ({least}..={body}).contains(&n));"
        ));
        o.line("let Some(input) = bytes.get(..len) else { return };");
        o.line("let mut r = Reader::new(input);");
        o.line(&format!(
            "let decoded = <{ty} as Wire>::decode(&mut r).and_then(|v| r.finish().map(|()| v));"
        ));
        o.line("kani::cover!(decoded.is_ok(), \"some input is accepted\");");
        o.line("if let Ok(v) = decoded {");
        o.line(&format!(
            "const {{ assert!({body} <= hg_ipc::wire::KANI_WRITER_CAPACITY) }};"
        ));
        o.line("let mut w = Writer::new();");
        o.line("v.encode(&mut w);");
        o.line("let out = w.into_bytes();");
        o.line("assert!(out.len() == len);");
        for i in 0..body {
            o.line(&format!(
                "if len > {i} {{ assert!(out.get({i}) == bytes.get({i})); }}"
            ));
        }
        o.line("}");
        o.line("}");
        o.line("");
    };
    // A union of cases that each end in their only variable piece: one
    // harness per case, its tag fixed (the cases' strings would otherwise
    // all be in one formula: `HandlerTarget`, three strings, ran out of
    // memory at 678 k steps).
    if let Some(cases) = crate::shape::tail_union_cases(checked, t) {
        for tag in 0..cases {
            let _ = writeln!(
                o.s,
                "/// VER-1 part `{ty}`, case {tag}: every accepted input of at most\n\
                 /// {body} bytes re-encodes to the same bytes."
            );
            let fixed: Vec<(u128, u8)> = (0u128..)
                .zip(u32::try_from(tag).unwrap_or(u32::MAX).to_le_bytes())
                .collect();
            tail_harness(o, &format!("round_trip_case_{tag}"), &fixed, 4);
        }
        return;
    }
    let shapes = if crate::shape::tail_variable(checked, t) {
        None
    } else {
        crate::shape::shapes(checked, &[t], body)
    };
    let Some(shapes) = shapes else {
        // One variable piece at the end, or too many layouts: one round
        // trip over every length instead.
        let _ = writeln!(
            o.s,
            "/// VER-1 part `{ty}`: every accepted input of at most {body} bytes\n\
             /// re-encodes to the same bytes."
        );
        tail_harness(o, "round_trip", &[], 0);
        return;
    };
    for (i, shape) in shapes.iter().enumerate() {
        let len = shape.len;
        let _ = writeln!(
            o.s,
            "/// VER-1 part `{ty}`: every accepted input of layout {i} ({len}\n\
             /// bytes) re-encodes to the same bytes."
        );
        header(o, &format!("round_trip_{i}"));
        if shape.fixed.is_empty() {
            o.line(&format!("let bytes: [u8; {len}] = kani::any();"));
        } else {
            o.line(&format!("let mut bytes: [u8; {len}] = kani::any();"));
            for (at, v) in &shape.fixed {
                o.line(&format!(
                    "if let Some(b) = bytes.get_mut({at}) {{ *b = {v}; }}"
                ));
            }
        }
        o.line("let mut r = Reader::new(&bytes);");
        o.line(&format!(
            "let decoded = <{ty} as Wire>::decode(&mut r).and_then(|v| r.finish().map(|()| v));"
        ));
        o.line("kani::cover!(decoded.is_ok(), \"this layout is accepted\");");
        o.line("if let Ok(v) = decoded {");
        o.line(&format!(
            "const {{ assert!({len} <= hg_ipc::wire::KANI_WRITER_CAPACITY) }};"
        ));
        o.line("let mut w = Writer::new();");
        o.line("v.encode(&mut w);");
        o.line("let out = w.into_bytes();");
        o.line(&format!("assert!(out.len() == {len});"));
        for i in 0..len {
            o.line(&format!("assert!(out.get({i}) == bytes.get({i}));"));
        }
        o.line("}");
        o.line("}");
        o.line("");
    }
}

/// What a message's round-trip harnesses need.
struct RoundTrip<'a> {
    snake: &'a str,
    iface: &'a str,
    name: &'a str,
    body_fn: &'a str,
    m: &'a Case,
    ordinal: usize,
    o0: u8,
    o1: u8,
    unwind: u128,
}

impl RoundTrip<'_> {
    fn header(&self, o: &mut Out, suffix: &str) {
        o.line("#[cfg(kani)]");
        o.line("#[kani::proof]");
        o.line(&format!("#[kani::unwind({})]", self.unwind));
        o.line(&format!(
            "fn verify_{}_{}_{suffix}() {{",
            self.snake, self.name
        ));
    }

    fn decode(&self, o: &mut Out, input: &str) {
        o.line(&format!("let mut r = Reader::new({input});"));
        o.line(&format!(
            "let decoded = {}::decode_{}(&mut r).and_then(|f| r.finish().map(|()| f));",
            self.iface, self.body_fn
        ));
    }

    /// Re-encodes `decoded`, and checks the ordinal and the length.
    /// `most` is the longest body the harness feeds: its encoding must
    /// fit the Kani writer's buffer, checked when the harness compiles.
    fn encode(&self, o: &mut Out, len: &str, most: u128) {
        let names = field_names(&self.m.fields);
        let args: String = names.iter().map(|n| format!(", &{n}")).collect();
        o.line(&format!("if let Ok({}) = decoded {{", tuple(&names)));
        o.line(&format!(
            "const {{ assert!({most} + 2 <= hg_ipc::wire::KANI_WRITER_CAPACITY) }};"
        ));
        o.line("let mut w = Writer::new();");
        o.line(&format!("{}u16.encode(&mut w);", self.ordinal));
        o.line(&format!(
            "{}::encode_{}(&mut w{args});",
            self.iface, self.body_fn
        ));
        o.line("let out = w.into_bytes();");
        o.line(&format!("assert!(out.len() == {len} + 2);"));
        // Byte by byte: a slice comparison is a `memcmp` loop.
        o.line(&format!("assert!(out.first() == Some(&{}));", self.o0));
        o.line(&format!("assert!(out.get(1) == Some(&{}));", self.o1));
    }
}

/// Decoding any input of at most `body` bytes never panics.
fn decode_harness(o: &mut Out, rt: &RoundTrip<'_>, body: u128) {
    rt.header(o, "decodes");
    o.line(&format!("let bytes: [u8; {body}] = kani::any();"));
    o.line(&format!(
        "let len: usize = kani::any_where(|&n: &usize| n <= {body});"
    ));
    o.line("let Some(input) = bytes.get(..len) else { return };");
    rt.decode(o, "input");
    o.line("kani::cover!(decoded.is_ok(), \"some input is accepted\");");
    o.line("}");
    o.line("");
}

/// Every accepted input of one layout re-encodes to the same bytes.
fn shape_harness(o: &mut Out, rt: &RoundTrip<'_>, i: usize, shape: &crate::shape::Shape) {
    let len = shape.len;
    rt.header(o, &format!("round_trip_{i}"));
    if shape.fixed.is_empty() {
        o.line(&format!("let bytes: [u8; {len}] = kani::any();"));
    } else {
        o.line(&format!("let mut bytes: [u8; {len}] = kani::any();"));
        for (at, v) in &shape.fixed {
            o.line(&format!(
                "if let Some(b) = bytes.get_mut({at}) {{ *b = {v}; }}"
            ));
        }
    }
    rt.decode(o, "&bytes");
    o.line("kani::cover!(decoded.is_ok(), \"this layout is accepted\");");
    rt.encode(o, &len.to_string(), len);
    for i in 0..len {
        o.line(&format!("assert!(out.get({}) == bytes.get({i}));", i + 2));
    }
    o.line("}");
    o.line("}");
    o.line("");
}

/// The single round-trip harness, over every input of at most `body`
/// bytes (for messages with handles, or too many layouts).
fn whole_round_trip(o: &mut Out, rt: &RoundTrip<'_>, body: u128) {
    rt.header(o, "round_trip");
    o.line(&format!("let bytes: [u8; {body}] = kani::any();"));
    o.line(&format!(
        "let len: usize = kani::any_where(|&n: &usize| n <= {body});"
    ));
    o.line("let Some(input) = bytes.get(..len) else { return };");
    rt.decode(o, "input");
    o.line("kani::cover!(decoded.is_ok(), \"some input is accepted\");");
    rt.encode(o, "len", body);
    for i in 0..body {
        o.line(&format!(
            "if len > {i} {{ assert!(out.get({}) == bytes.get({i})); }}",
            i + 2
        ));
    }
    o.line("}");
    o.line("}");
    o.line("");
}

/// Kani harnesses per message (Stage 1 4.1, VER-1): a validator harness
/// for every message, a round-trip harness for every message without
/// nested heap data, and one for the ordinals no message has.
///
/// Each message's harness calls that message's body decoder and encoder
/// directly, so the solver explores only them (through the interface
/// enum, it could not tell which message a decoded value held and
/// explored every encoder: 550k steps for a message with no fields).
/// It checks the whole encoding, ordinal included. Messages run up to `k` bytes: `KANI_INPUT_BYTES`, raised to
/// the message's smallest encoding so the harness always reaches an
/// accepted message, and capped at its largest. The `cover!` records
/// that it does.
fn kani(o: &mut Out, protocol: &str, iface: &str, messages: &[Case], checked: &Checked) {
    let snake = snake_of(iface);
    let sizes = checked
        .message_sizes
        .get(iface)
        .map_or(&[][..], Vec::as_slice);
    for (ordinal, (m, facts)) in messages.iter().zip(sizes).enumerate() {
        let size = facts.size;
        let name = &m.id.name;
        let body_fn = snake_of(&pascal_of(name));
        // The harness feeds the body only; the ordinal is the dispatcher's.
        let k = crate::shape::harness_bytes(
            checked,
            m.fields.iter().map(|f| &f.ty),
            KANI_INPUT_BYTES,
            size.min,
            size.max,
        );
        let body = k.saturating_sub(2);
        // The comparison below is unrolled here rather than looped; every
        // other loop is bounded as `shape::loop_need` says.
        let unwind = crate::shape::unwind_for(checked, m.fields.iter().map(|f| &f.ty), body);
        let [o0, o1] = u16::try_from(ordinal).unwrap_or(u16::MAX).to_le_bytes();
        let types: Vec<&Type> = m.fields.iter().map(|f| &f.ty).collect();
        let rt = RoundTrip {
            snake: &snake,
            iface,
            name,
            body_fn: &body_fn,
            m,
            ordinal,
            o0,
            o1,
            unwind,
        };
        let split = crate::shape::split_of(&format!("{protocol}.{iface}.{name}"));
        let mut part_types = Vec::new();
        if split.is_some() {
            crate::shape::parts(checked, &types, &mut part_types);
        }
        if split.is_some_and(|s| s.validate) {
            let _ = writeln!(
                o.s,
                "// `{protocol}.{iface}.{name}`'s validator is proved by parts (see\n\
                 // `hg-ipc-codegen`'s `shape` module): the part harnesses below.\n"
            );
            for t in &part_types {
                part_validator(o, checked, t);
            }
        } else {
            validator_harness(o, protocol, &rt, facts, checked);
        }
        if facts.heap_depth > 1 {
            let _ = writeln!(
                o.s,
                "// `{protocol}.{iface}.{name}` holds heap data inside heap data, so its\n\
                 // round trip is not a Kani harness: copies out of nested heap memory\n\
                 // are beyond the solver. Its parsing is proved by the validator\n\
                 // harness above, its round trip by the property tests.\n"
            );
            continue;
        }
        if split.is_some_and(|s| s.round_trip) {
            let _ = writeln!(
                o.s,
                "// `{protocol}.{iface}.{name}`'s decoding and round trip are also proved\n\
                 // by parts (see `hg-ipc-codegen`'s `shape` module).\n"
            );
            for t in &part_types {
                part_round_trip(o, checked, t);
            }
        }
        if split.is_some_and(|s| !s.keep_whole_round_trip) {
            continue;
        }
        let failing = split.map_or(&[][..], |s| s.failing_layouts);
        match crate::shape::shapes(checked, &types, body) {
            Some(shapes) if !shapes.is_empty() => {
                let _ = writeln!(
                    o.s,
                    "/// VER-1 for `{protocol}.{iface}.{name}` (IPC-R5): decoding never\n\
                     /// panics. Bounds: messages of 2 to {k} bytes (its encodings are\n\
                     /// {} to {}); loops unwound {unwind} times. Its round trip is\n\
                     /// proved per layout below ({} layouts).",
                    size.min,
                    size.max,
                    shapes.len()
                );
                decode_harness(o, &rt, body);
                for (i, shape) in shapes.iter().enumerate() {
                    if failing.contains(&i) {
                        let _ = writeln!(
                            o.s,
                            "// Layout {i} of `{protocol}.{iface}.{name}` ({} bytes) is over\n\
                             // Kani's limits whole; its parts prove it (`shape::SPLIT`).\n",
                            shape.len
                        );
                        continue;
                    }
                    let _ = writeln!(
                        o.s,
                        "/// VER-1 for `{protocol}.{iface}.{name}` (IPC-R6): every accepted\n\
                         /// input of layout {i} ({} bytes) re-encodes to the same bytes.",
                        shape.len
                    );
                    shape_harness(o, &rt, i, shape);
                }
            }
            _ => {
                let _ = writeln!(
                    o.s,
                    "/// VER-1 for `{protocol}.{iface}.{name}` (IPC-R5, IPC-R6): decoding\n\
                     /// never panics, and every accepted input re-encodes to the same\n\
                     /// bytes. Bounds: messages of 2 to {k} bytes (its encodings are {}\n\
                     /// to {}); loops unwound {unwind} times.",
                    size.min, size.max
                );
                whole_round_trip(o, &rt, body);
            }
        }
    }
    if messages.len() <= usize::from(u16::MAX) {
        let count = messages.len();
        // Each message's validator is replaced by one that may return
        // anything: the dispatcher alone decides this property, and with
        // the real validators in place the solver explores every one of
        // them on paths the ordinal rules out (OsInput's timed out at 15
        // minutes; stubbed, it takes under a second). The stubs allow
        // every result the real validators can give, so the assertion
        // holds for the real code: it holds only if no message's
        // validator runs. Needs `-Z stubbing`.
        let _ = writeln!(
            o.s,
            "/// For [`verify_{snake}_unknown_ordinal`]: a message validator that\n\
             /// may accept or refuse anything.\n\
             #[cfg(kani)]\n\
             fn kani_any_{snake}_validation(_r: &mut Reader<'_>) -> Result<(), BadMessage> {{\n\
             if kani::any() {{ Ok(()) }} else {{ Err(BadMessage::Truncated) }}\n\
             }}\n"
        );
        let _ = writeln!(
            o.s,
            "/// VER-1 for `{protocol}.{iface}`: an ordinal no message has is\n\
             /// refused by the validator, which `decode_exact` runs first,\n\
             /// whatever follows it (bounds: 2 to 4 bytes). The message\n\
             /// validators are stubbed: none may run."
        );
        o.line("#[cfg(kani)]");
        o.line("#[kani::proof]");
        o.line("#[kani::unwind(6)]");
        for m in messages {
            let body_fn = snake_of(&pascal_of(&m.id.name));
            o.line(&format!(
                "#[kani::stub({iface}::validate_{body_fn}, kani_any_{snake}_validation)]"
            ));
        }
        o.line(&format!("fn verify_{snake}_unknown_ordinal() {{"));
        o.line(&format!(
            "let ordinal: u16 = kani::any_where(|&n: &u16| usize::from(n) >= {count});"
        ));
        o.line("let tail: [u8; 2] = kani::any();");
        o.line("let len: usize = kani::any_where(|&n: &usize| (2..=4).contains(&n));");
        o.line("let [o0, o1] = ordinal.to_le_bytes();");
        o.line("let [t0, t1] = tail;");
        o.line("let bytes = [o0, o1, t0, t1];");
        o.line("let Some(input) = bytes.get(..len) else { return };");
        o.line(&format!(
            "assert!({iface}::validate_bytes(input) == Err(BadMessage::UnknownOrdinal));"
        ));
        o.line("}");
        o.line("");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Prim;

    /// The text with whitespace and commas removed, so layout does not
    /// matter but every other token, in order, does.
    fn squash(s: &str) -> String {
        s.chars()
            .filter(|c| !c.is_whitespace() && *c != ',')
            .collect()
    }

    /// The body, braces included, of the first `fn {name}(` at or after
    /// `from` in squashed text.
    fn body(text: &str, from: usize, name: &str) -> String {
        let head = format!("fn{name}(");
        let start = from
            + text[from..]
                .find(&head)
                .unwrap_or_else(|| panic!("no {head}"));
        let open = start + text[start..].find('{').unwrap();
        let mut depth = 0usize;
        for (i, c) in text[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return text[open..=open + i].to_owned();
                    }
                }
                _ => {}
            }
        }
        panic!("unbalanced {name}")
    }

    fn at(text: &str, marker: &str) -> usize {
        text.find(marker).unwrap_or_else(|| panic!("no {marker}"))
    }

    fn validates(fields: &[Field]) -> String {
        fields
            .iter()
            .map(|f| format!("<{}asWire>::validate(r)?;", squash(&rust_type(&f.ty))))
            .collect()
    }

    fn decodes(fields: &[Field]) -> String {
        fields
            .iter()
            .map(|f| format!("{}:Wire::decode(r)?", f.id.name))
            .collect()
    }

    fn encodes(fields: &[Field], prefix: &str) -> String {
        fields
            .iter()
            .map(|f| format!("{prefix}{}.encode(w);", f.id.name))
            .collect()
    }

    /// The joining step of the per-part proofs (see `shape`): every
    /// generated struct, union and message codec is exactly its fields'
    /// codecs, in field order, each on the same reader or writer, and
    /// nothing else. Checked against templates this test owns, for every
    /// `.hgidl` in the workspace.
    #[test]
    fn every_codec_is_its_fields_in_order() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let files = [
            root.join("../hg-ipc-gen/idl/process.hgidl"),
            root.join("../hg-ipc-gen/idl/sample.hgidl"),
            root.join("../hg-os-types/idl/os.hgidl"),
            root.join("tests/golden/policy.hgidl"),
        ];
        let mut checked_codecs = 0;
        for path in files {
            let src = std::fs::read_to_string(&path).unwrap();
            let checked = crate::check::check(crate::syntax::parse(&src).unwrap()).unwrap();
            let out = squash(&emit(&checked, "t"));
            for item in &checked.file.items {
                match item {
                    Item::Struct(n, fields) => {
                        let i = at(&out, &format!("implWirefor{}{{", n.name));
                        assert_eq!(
                            body(&out, i, "encode"),
                            format!("{{{}}}", encodes(fields, "self."))
                        );
                        assert_eq!(
                            body(&out, i, "decode"),
                            format!("{{Ok(Self{{{}}})}}", decodes(fields))
                        );
                        assert_eq!(
                            body(&out, i, "validate"),
                            format!("{{{}Ok(())}}", validates(fields))
                        );
                        checked_codecs += 3;
                    }
                    Item::Union(n, cases) => {
                        let i = at(&out, &format!("implWirefor{}{{", n.name));
                        let (mut enc, mut dec, mut val) =
                            (String::new(), String::new(), String::new());
                        for (tag, c) in cases.iter().enumerate() {
                            let name = &c.id.name;
                            if c.fields.is_empty() {
                                enc += &format!("Self::{name}=>{{{tag}u32.encode(w);}}");
                                dec += &format!("{tag}=>Ok(Self::{name})");
                                val += &format!("{tag}=>Ok(())");
                            } else {
                                let binds: String =
                                    c.fields.iter().map(|f| f.id.name.as_str()).collect();
                                enc += &format!(
                                    "Self::{name}{{{binds}}}=>{{{tag}u32.encode(w);{}}}",
                                    encodes(&c.fields, "")
                                );
                                dec +=
                                    &format!("{tag}=>Ok(Self::{name}{{{}}})", decodes(&c.fields));
                                val += &format!("{tag}=>{{{}Ok(())}}", validates(&c.fields));
                            }
                        }
                        let unknown = "_=>Err(BadMessage::UnknownTag)";
                        assert_eq!(body(&out, i, "encode"), format!("{{matchself{{{enc}}}}}"));
                        assert_eq!(
                            body(&out, i, "decode"),
                            format!("{{matchu32::decode(r)?{{{dec}{unknown}}}}}")
                        );
                        assert_eq!(
                            body(&out, i, "validate"),
                            format!("{{matchu32::decode(r)?{{{val}{unknown}}}}}")
                        );
                        checked_codecs += 3;
                    }
                    Item::Interface { id, messages, .. } => {
                        let i = at(&out, &format!("impl{}{{", id.name));
                        for m in messages {
                            let handles = m
                                .fields
                                .iter()
                                .any(|f| matches!(f.ty, Type::Remote(..) | Type::Receiver(..)));
                            if handles {
                                // Handles travel beside the bytes; not a
                                // per-part proof's concern.
                                continue;
                            }
                            let snake = snake_of(&pascal_of(&m.id.name));
                            let reads: String =
                                m.fields.iter().map(|_| "Wire::decode(r)?").collect();
                            let (dec, val, enc) = if m.fields.is_empty() {
                                (
                                    "{let_=r;Ok(())}".to_owned(),
                                    "{let_=r;Ok(())}".to_owned(),
                                    None,
                                )
                            } else {
                                (
                                    format!("{{Ok(({reads}))}}"),
                                    format!("{{{}Ok(())}}", validates(&m.fields)),
                                    Some(format!("{{{}}}", encodes(&m.fields, ""))),
                                )
                            };
                            assert_eq!(
                                body(&out, i, &format!("decode_{snake}")),
                                dec,
                                "{}",
                                m.id.name
                            );
                            assert_eq!(
                                body(&out, i, &format!("validate_{snake}")),
                                val,
                                "{}",
                                m.id.name
                            );
                            if let Some(enc) = enc {
                                assert_eq!(
                                    body(&out, i, &format!("encode_{snake}")),
                                    enc,
                                    "{}",
                                    m.id.name
                                );
                            }
                            checked_codecs += 3;
                        }
                    }
                    Item::Enum(..) | Item::Error(..) => {}
                }
            }
        }
        assert!(checked_codecs > 100, "{checked_codecs}");
    }

    #[test]
    fn prim_names_are_rust_names() {
        assert_eq!(rust_type(&Type::Prim(Prim::F64)), "f64");
        assert_eq!(
            rust_type(&Type::Vec(Box::new(Type::Str(8)), 3)),
            "hg_ipc::BoundedVec<hg_ipc::BoundedString<8>, 3>"
        );
    }
}
