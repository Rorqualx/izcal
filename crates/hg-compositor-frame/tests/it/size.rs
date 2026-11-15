//! GR-18: the published size artifact is recomputed from the IDL, so
//! the format cannot grow without this test failing.
//!
//! And its G63 half: the format names no path, segment, fill rule,
//! gradient, filter, mask, ICC or shader vocabulary at all. "**0**
//! page-controlled filters, paths or ICC data parsed by the
//! compositor" is then a property of the format, checked, rather than
//! a sentence about it.

use std::path::Path;

use hg_compositor_frame::size;
use hg_ipc_codegen::syntax::{Item, Type, parse};

#[expect(
    clippy::disallowed_methods,
    reason = "the size artifact is computed from the checked-in IDL"
)]
fn idl() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("idl/compositor_frame.hgidl");
    std::fs::read_to_string(p).unwrap()
}

/// Every type name a field mentions, however deeply wrapped.
fn named_in(t: &Type, out: &mut Vec<String>) {
    match t {
        Type::Prim(_)
        | Type::Bytes(_)
        | Type::Str(_)
        | Type::Remote(_, _)
        | Type::Receiver(_, _)
        | Type::Region(_, _, _, _) => {}
        Type::Named(n, _) => out.push(n.clone()),
        Type::Vec(inner, _) | Type::Option(inner) => named_in(inner, out),
    }
}

struct Counts {
    types: usize,
    variants: usize,
    fields: usize,
    vectors: usize,
}

fn counts(items: &[Item]) -> Counts {
    let mut c = Counts {
        types: 0,
        variants: 0,
        fields: 0,
        vectors: 0,
    };
    for item in items {
        c.types = c.types.saturating_add(1);
        match item {
            Item::Error(_, vs) | Item::Enum(_, vs) => {
                c.variants = c.variants.saturating_add(vs.len());
            }
            Item::Union(_, cases) => {
                c.variants = c.variants.saturating_add(cases.len());
                for case in cases {
                    c.fields = c.fields.saturating_add(case.fields.len());
                    for f in &case.fields {
                        if matches!(f.ty, Type::Vec(_, _)) {
                            c.vectors = c.vectors.saturating_add(1);
                        }
                    }
                }
            }
            Item::Struct(_, fields) => {
                c.fields = c.fields.saturating_add(fields.len());
                for f in fields {
                    if matches!(f.ty, Type::Vec(_, _)) {
                        c.vectors = c.vectors.saturating_add(1);
                    }
                }
            }
            Item::Interface { .. } => {
                panic!("an interface in a format crate: the compositor frame is types only")
            }
        }
    }
    c
}

/// How deep one declared type nests inside another, from `root`.
fn depth_from(root: &str, items: &[Item], seen: &mut Vec<String>) -> usize {
    if seen.iter().any(|s| s == root) {
        return 0;
    }
    seen.push(root.to_owned());
    let mut names = Vec::new();
    for item in items {
        let fields = match item {
            Item::Struct(id, fields) if id.name == root => fields.clone(),
            Item::Union(id, cases) if id.name == root => {
                cases.iter().flat_map(|c| c.fields.clone()).collect()
            }
            Item::Struct(_, _) | Item::Union(_, _) | Item::Enum(_, _) | Item::Error(_, _) => {
                continue;
            }
            Item::Interface { .. } => continue,
        };
        for f in &fields {
            named_in(&f.ty, &mut names);
        }
    }
    let mut deepest = 0;
    for n in names {
        let d = depth_from(&n, items, seen).saturating_add(1);
        deepest = deepest.max(d);
    }
    seen.pop();
    deepest
}

#[test]
fn cf20_gr18_the_published_size_is_what_the_idl_says() {
    let file = parse(&idl()).unwrap();
    let c = counts(&file.items);
    assert_eq!(c.types, size::TYPES, "declared types");
    assert_eq!(c.variants, size::VARIANTS, "variants");
    assert_eq!(c.fields, size::FIELDS, "fields");
    assert_eq!(c.vectors, size::BOUNDED_VECTORS, "bounded vectors");
    assert_eq!(
        depth_from("Frame", &file.items, &mut Vec::new()),
        size::MAX_NESTING,
        "maximum nesting below Frame"
    );
}

#[test]
fn cf21_g63_the_format_has_no_page_controlled_pixel_vocabulary() {
    let file = parse(&idl()).unwrap();
    let mut names = Vec::new();
    for item in &file.items {
        match item {
            Item::Error(id, vs) | Item::Enum(id, vs) => {
                names.push(id.name.clone());
                names.extend(vs.iter().map(|v| v.name.clone()));
            }
            Item::Union(id, cases) => {
                names.push(id.name.clone());
                for case in cases {
                    names.push(case.id.name.clone());
                    names.extend(case.fields.iter().map(|f| f.id.name.clone()));
                }
            }
            Item::Struct(id, fields) => {
                names.push(id.name.clone());
                names.extend(fields.iter().map(|f| f.id.name.clone()));
            }
            Item::Interface { .. } => {}
        }
    }
    for name in &names {
        let lowered = name.to_lowercase().replace('_', "");
        for forbidden in size::FORBIDDEN_VOCABULARY {
            assert!(
                !lowered.contains(forbidden),
                "G63: `{name}` names `{forbidden}`, which the compositor must not parse"
            );
        }
    }

    // The twin: the check must be able to fail. A name the format does
    // not have, run through the same comparison, is caught.
    let planted = "FilterChain".to_lowercase();
    assert!(
        size::FORBIDDEN_VOCABULARY
            .iter()
            .any(|f| planted.contains(f)),
        "the vocabulary check would not catch a filter type"
    );
}
