//! Bounds and layouts for the generated Kani harnesses (VER-1).
//!
//! [`loop_need`] is the most times any loop in a value's code can run for
//! an input of `body` bytes: a string's bytes, or a vec's elements, each
//! at least one byte (an element at least its type's smallest encoding).
//! It sets a harness's unwind bound. Kani checks unwinding assertions, so
//! a bound too small fails the proof rather than passing it.
//!
//! [`shapes`] lists every layout an accepted value of at most `budget`
//! bytes can have: each option's presence, each union's case, each
//! string's length and each vec's count, with those bytes fixed and the
//! rest (numbers, enum values, string contents) left free. A round-trip
//! proof over one symbolic input of any length makes every offset after
//! the first variable field symbolic, and CBMC's cost for that grew past
//! 10 GB for a 22-byte body (`os.OsInput.power`); with the layout fixed,
//! the same check took 7 s. Every accepted input has exactly one layout,
//! so proving the round trip for each layout proves it for every accepted
//! input; that decoding never panics on any input is a separate harness.
//!
//! # Proofs by part
//!
//! Some messages are too large for one proof (measured: they time out at
//! 15 minutes or exceed 10 GiB). For those, [`SPLIT`] names the messages
//! and [`SPLIT_TYPES`] the structs and unions to take apart, and the
//! generator proves each *part* instead: every field type the split
//! reaches, down to the first type not in [`SPLIT_TYPES`]. A part's
//! harness is the message's, on that type alone and on any input of the
//! same size.
//!
//! The joining step. A generated codec for a struct, a union case or a
//! message does nothing but call its fields' codecs, in field order, on
//! one reader or writer (and, for a union, read or write the `u32` tag
//! first); the `every_codec_is_its_fields_in_order` test checks this
//! against templates for every `.hgidl`. So, for a message `m` with
//! fields `f1 … fn`:
//!
//! - **Validation never panics.** `validate_m` runs `validate_f1`, then
//!   `validate_f2` on what is left, and so on, stopping at the first
//!   error. Each `validate_fi` sees a suffix of the input, which is itself
//!   an input of at most the harness's size, and the part harness proves
//!   `validate_fi` never panics on every such input. So `validate_m` never
//!   panics.
//! - **Round trip.** If `decode_m` accepts `b`, the fields' decoders
//!   consumed consecutive pieces `b = b1 ‖ … ‖ bn` (a reader only moves
//!   forward, and a decoder's result depends only on the bytes it
//!   consumes: a count is checked against the bytes left, and the
//!   elements it counts are among the bytes the decoder then consumes, so
//!   the same count passes on `bi` alone). So `decode_fi` accepts `bi`
//!   alone, whole, with the same value `vi`, and the part's round-trip
//!   proof gives `encode(vi) = bi`. `encode_m` writes `encode(v1) ‖ … ‖
//!   encode(vn)` after the ordinal, which is the ordinal followed by `b`.
//!
//! A union's tag is the first `u32` of `bi`; a case's fields join the
//! same way after it, so a split union's parts are its cases' field
//! types. An option is never split. A vector's loop is in `hg-ipc`, which
//! proves it for any element (one that consumes any allowed number of
//! bytes, or fails) at the bounds in [`VEC_LOOP_PROOFS`]; there, and for
//! validators only, a split vector's part is its element, proved by its
//! own part harness. Elsewhere a vector is proved whole, element
//! included.

use crate::check::Checked;
use crate::syntax::{Item, Type};

/// A message proved, in whole or in part, by parts (see the module
/// docs). Each entry is here because a whole-message proof was measured
/// over the limits on hueyj (15 minutes, 10 GiB);
/// measurements/milestones/m1-kani has the numbers. Whole-message proofs
/// that pass are kept: the parts are a fallback, not a replacement.
#[derive(Debug, Clone, Copy)]
pub struct Split {
    /// `protocol.Interface.message`.
    pub message: &'static str,
    /// Prove the validator by parts, instead of whole.
    pub validate: bool,
    /// Prove decoding and the round trip by parts as well.
    pub round_trip: bool,
    /// Keep the whole-message decoding and round-trip harnesses, except
    /// the layouts in `failing_layouts`.
    pub keep_whole_round_trip: bool,
    /// Whole-message round-trip layouts measured over the limits, left to
    /// the parts.
    pub failing_layouts: &'static [usize],
}

/// The messages proved by parts.
pub const SPLIT: &[Split] = &[
    Split {
        message: "os.OsCommand.register",
        validate: true,
        round_trip: true,
        keep_whole_round_trip: false,
        failing_layouts: &[],
    },
    Split {
        message: "os.OsInput.locale",
        validate: true,
        round_trip: false,
        keep_whole_round_trip: true,
        failing_layouts: &[],
    },
    Split {
        message: "os.OsInput.settings",
        validate: true,
        round_trip: false,
        keep_whole_round_trip: true,
        failing_layouts: &[],
    },
    Split {
        message: "os.OsInput.launch",
        validate: true,
        round_trip: false,
        keep_whole_round_trip: true,
        failing_layouts: &[],
    },
    Split {
        message: "os.OsCommand.post",
        validate: false,
        round_trip: true,
        keep_whole_round_trip: true,
        failing_layouts: &[18, 25],
    },
    Split {
        message: "os.OsInput.power",
        validate: false,
        round_trip: true,
        keep_whole_round_trip: true,
        failing_layouts: &[0],
    },
    Split {
        message: "os.OsInput.displays",
        validate: false,
        round_trip: true,
        keep_whole_round_trip: true,
        failing_layouts: &[1],
    },
    Split {
        message: "os.OsInput.default_status",
        validate: false,
        round_trip: true,
        keep_whole_round_trip: true,
        failing_layouts: &[1, 2, 4],
    },
];

/// The structs and unions a split takes apart into their fields.
pub const SPLIT_TYPES: &[&str] = &[
    "OsRegistration",
    "RegistrationKind",
    "LocaleValues",
    "SettingsValues",
    "LaunchRequest",
];

/// The split entry for `protocol.iface.message`, if any.
pub fn split_of(qualified: &str) -> Option<Split> {
    SPLIT.iter().copied().find(|s| s.message == qualified)
}

/// The vector loops `hg-ipc` proves against any element (`OpaqueMin`):
/// (`N`, the element's smallest encoding), each by the proof
/// `vec_of_any_element_validates_{N}_of_{min}_bytes` in `bounded.rs`. A
/// split validator takes a vector apart into its element only at one of
/// these bounds.
pub const VEC_LOOP_PROOFS: &[(u64, u128)] = &[(16, 4), (64, 8), (6, 8)];

/// The parts of `types` for a split: each type, or, for a struct or union
/// in [`SPLIT_TYPES`], the parts of its fields (a union's: of every
/// case's fields), and, for validators (`validate`), a vector of
/// variable-size elements at a bound in [`VEC_LOOP_PROOFS`] the parts of
/// its element. In order, without repeats.
pub fn parts<'a>(
    checked: &'a Checked,
    types: &[&'a Type],
    validate: bool,
    out: &mut Vec<&'a Type>,
) {
    for &t in types {
        let inner: Option<Vec<&Type>> = match t {
            Type::Vec(elem, n)
                if validate
                    && VEC_LOOP_PROOFS.contains(&(*n, min_len(checked, elem)))
                    && size_of(checked, elem).0 != size_of(checked, elem).1 =>
            {
                Some(vec![elem.as_ref()])
            }
            Type::Named(name, _) if SPLIT_TYPES.contains(&name.as_str()) => {
                match find(checked, name) {
                    Some(Item::Struct(_, fields)) => Some(fields.iter().map(|f| &f.ty).collect()),
                    Some(Item::Union(_, cases)) => Some(
                        cases
                            .iter()
                            .flat_map(|c| c.fields.iter().map(|f| &f.ty))
                            .collect(),
                    ),
                    _ => None,
                }
            }
            _ => None,
        };
        match inner {
            Some(fields) => parts(checked, &fields, validate, out),
            None => {
                if !out.contains(&t) {
                    out.push(t);
                }
            }
        }
    }
}

/// Whether `t`'s encoding has at most one variable-length piece, at its
/// end: then a round trip over one symbolic input of any length has no
/// symbolic offset after the variable piece, and needs no split by
/// layout (one string, or a union of single strings, for example).
pub fn tail_variable(checked: &Checked, t: &Type) -> bool {
    let fixed = |t: &Type| {
        let (lo, hi) = size_of(checked, t);
        lo == hi
    };
    let seq = |types: &[&Type]| match types.split_last() {
        None => true,
        Some((last, rest)) => rest.iter().all(|t| fixed(t)) && tail_variable(checked, last),
    };
    match t {
        Type::Prim(_) | Type::Str(_) | Type::Bytes(_) => true,
        Type::Option(inner) => tail_variable(checked, inner),
        Type::Vec(..) | Type::Remote(..) | Type::Receiver(..) => false,
        Type::Named(name, _) => match find(checked, name) {
            Some(Item::Struct(_, fields)) => {
                let types: Vec<&Type> = fields.iter().map(|f| &f.ty).collect();
                seq(&types)
            }
            Some(Item::Union(_, cases)) => cases.iter().all(|c| {
                let types: Vec<&Type> = c.fields.iter().map(|f| &f.ty).collect();
                seq(&types)
            }),
            Some(Item::Enum(..) | Item::Error(..)) => true,
            Some(Item::Interface { .. }) | None => false,
        },
    }
}

/// For a union whose cases each end in their only variable piece, and
/// at least two of which have one: its number of cases.
pub fn tail_union_cases(checked: &Checked, t: &Type) -> Option<usize> {
    let Type::Named(name, _) = t else { return None };
    let Some(Item::Union(_, cases)) = find(checked, name) else {
        return None;
    };
    let variable = |c: &crate::syntax::Case| {
        c.fields.iter().any(|f| {
            let (lo, hi) = size_of(checked, &f.ty);
            lo != hi
        })
    };
    (tail_variable(checked, t) && cases.iter().filter(|c| variable(c)).count() >= 2)
        .then_some(cases.len())
}

/// The fewest and most bytes `t` encodes to.
pub fn size_of(checked: &Checked, t: &Type) -> (u128, u128) {
    match t {
        Type::Prim(p) => (p.size(), p.size()),
        Type::Str(n) | Type::Bytes(n) => (4, 4u128.saturating_add(u128::from(*n))),
        Type::Vec(inner, n) => {
            let (_, most) = size_of(checked, inner);
            (4, 4u128.saturating_add(u128::from(*n).saturating_mul(most)))
        }
        Type::Option(inner) => {
            let (_, most) = size_of(checked, inner);
            (1, 1u128.saturating_add(most))
        }
        Type::Named(n, _) => checked
            .facts
            .get(n)
            .map_or((0, 0), |f| (f.size.min, f.size.max)),
        Type::Remote(..) | Type::Receiver(..) => (0, 0),
    }
}

/// The most layouts a message's round trip is split into; past this the
/// generator keeps the single harness.
pub const MAX_SHAPES: usize = 256;

/// One layout: its length and the bytes it fixes, by offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    /// Its length in bytes.
    pub len: u128,
    /// (offset, value) for each structural byte.
    pub fixed: Vec<(u128, u8)>,
}

fn find<'a>(checked: &'a Checked, name: &str) -> Option<&'a Item> {
    checked.file.items.iter().find(|i| match i {
        Item::Error(n, _) | Item::Enum(n, _) | Item::Union(n, _) | Item::Struct(n, _) => {
            n.name == name
        }
        Item::Interface { .. } => false,
    })
}

/// The fewest bytes `t` encodes to.
fn min_len(checked: &Checked, t: &Type) -> u128 {
    match t {
        Type::Prim(p) => p.size(),
        Type::Bytes(_) | Type::Str(_) | Type::Vec(..) => 4,
        Type::Option(_) => 1,
        Type::Named(n, _) => checked.facts.get(n).map_or(1, |f| f.size.min.max(1)),
        Type::Remote(..) | Type::Receiver(..) => 1,
    }
}

/// The most times a loop in `t`'s code runs on `body` bytes of input.
pub fn loop_need(checked: &Checked, t: &Type, body: u128) -> u128 {
    match t {
        Type::Prim(_) | Type::Bytes(_) | Type::Remote(..) | Type::Receiver(..) => 0,
        Type::Str(n) => u128::from(*n).min(body),
        Type::Vec(inner, n) => {
            let elems = body / min_len(checked, inner).max(1);
            u128::from(*n)
                .min(elems)
                .max(loop_need(checked, inner, body))
        }
        Type::Option(inner) => loop_need(checked, inner, body),
        Type::Named(name, _) => match find(checked, name) {
            Some(Item::Struct(_, fields)) => fields
                .iter()
                .map(|f| loop_need(checked, &f.ty, body))
                .max()
                .unwrap_or(0),
            Some(Item::Union(_, cases)) => cases
                .iter()
                .flat_map(|c| &c.fields)
                .map(|f| loop_need(checked, &f.ty, body))
                .max()
                .unwrap_or(0),
            Some(Item::Enum(..) | Item::Error(..) | Item::Interface { .. }) | None => 0,
        },
    }
}

/// How many bytes past its smallest encoding `t` needs to take each of
/// its non-minimal forms, one at a time: a string one byte, a vec or an
/// option one element (itself grown), a union its largest case.
pub fn grow(checked: &Checked, t: &Type) -> u128 {
    match t {
        Type::Prim(_) | Type::Remote(..) | Type::Receiver(..) => 0,
        Type::Str(_) | Type::Bytes(_) => 1,
        Type::Vec(inner, _) | Type::Option(inner) => {
            min_len(checked, inner).saturating_add(grow(checked, inner))
        }
        Type::Named(name, _) => match find(checked, name) {
            Some(Item::Struct(_, fields)) => fields
                .iter()
                .map(|f| grow(checked, &f.ty))
                .max()
                .unwrap_or(0),
            Some(Item::Union(_, cases)) => {
                let case_min = |c: &crate::syntax::Case| -> u128 {
                    c.fields.iter().map(|f| min_len(checked, &f.ty)).sum()
                };
                let least = cases.iter().map(case_min).min().unwrap_or(0);
                cases
                    .iter()
                    .map(|c| {
                        let inner = c.fields.iter().map(|f| grow(checked, &f.ty)).max();
                        case_min(c)
                            .saturating_sub(least)
                            .saturating_add(inner.unwrap_or(0))
                    })
                    .max()
                    .unwrap_or(0)
            }
            Some(Item::Enum(..) | Item::Error(..) | Item::Interface { .. }) | None => 0,
        },
    }
}

/// A harness's message size: `base`, raised so that every field can take
/// a non-minimal form (a list with an element, a present option, a
/// non-empty string), and capped at the largest encoding. Without the
/// raise, a message whose fixed part is near `base` is proved only with
/// its lists empty.
pub fn harness_bytes<'a>(
    checked: &Checked,
    types: impl IntoIterator<Item = &'a Type>,
    base: u128,
    min: u128,
    max: u128,
) -> u128 {
    let need = types
        .into_iter()
        .map(|t| grow(checked, t))
        .max()
        .unwrap_or(0);
    base.max(min.saturating_add(need)).min(max)
}

/// The unwind bound for code over `types` on `body` bytes.
pub fn unwind_for<'a>(
    checked: &Checked,
    types: impl IntoIterator<Item = &'a Type>,
    body: u128,
) -> u128 {
    types
        .into_iter()
        .map(|t| loop_need(checked, t, body))
        .max()
        .unwrap_or(0)
        .saturating_add(2)
}

fn shifted(s: &Shape, by: u128) -> impl Iterator<Item = (u128, u8)> + '_ {
    s.fixed.iter().map(move |&(o, v)| (o.saturating_add(by), v))
}

/// A `u32` field fixed to `value` at offset 0.
fn fixed_u32(value: u128) -> Option<Vec<(u128, u8)>> {
    let v = u32::try_from(value).ok()?;
    Some((0u128..).zip(v.to_le_bytes()).collect())
}

/// `prefix` (fixed bytes at 0..`plen`) followed by each of `rest`.
fn prefixed(prefix: &[(u128, u8)], plen: u128, rest: Vec<Shape>) -> Vec<Shape> {
    rest.into_iter()
        .map(|s| Shape {
            len: plen.saturating_add(s.len),
            fixed: prefix.iter().copied().chain(shifted(&s, plen)).collect(),
        })
        .collect()
}

/// Every layout of the values of `types`, in order, in at most `budget`
/// bytes; `None` if there are more than [`MAX_SHAPES`] or a handle.
pub fn shapes(checked: &Checked, types: &[&Type], budget: u128) -> Option<Vec<Shape>> {
    let Some((first, rest)) = types.split_first() else {
        return Some(vec![Shape {
            len: 0,
            fixed: Vec::new(),
        }]);
    };
    let mut out = Vec::new();
    for a in one(checked, first, budget)? {
        let tails = shapes(checked, rest, budget.saturating_sub(a.len))?;
        for b in tails {
            out.push(Shape {
                len: a.len.saturating_add(b.len),
                fixed: a.fixed.iter().copied().chain(shifted(&b, a.len)).collect(),
            });
            if out.len() > MAX_SHAPES {
                return None;
            }
        }
    }
    Some(out)
}

fn one(checked: &Checked, t: &Type, budget: u128) -> Option<Vec<Shape>> {
    let leaf = |len: u128| {
        if len <= budget {
            vec![Shape {
                len,
                fixed: Vec::new(),
            }]
        } else {
            Vec::new()
        }
    };
    Some(match t {
        Type::Prim(p) => leaf(p.size()),
        Type::Remote(..) | Type::Receiver(..) => return None,
        Type::Str(n) | Type::Bytes(n) => {
            let Some(room) = budget.checked_sub(4) else {
                return Some(Vec::new());
            };
            let mut v = Vec::new();
            for len in 0..=u128::from(*n).min(room) {
                v.push(Shape {
                    len: 4u128.saturating_add(len),
                    fixed: fixed_u32(len)?,
                });
                if v.len() > MAX_SHAPES {
                    return None;
                }
            }
            v
        }
        Type::Vec(inner, n) => {
            let Some(room) = budget.checked_sub(4) else {
                return Some(Vec::new());
            };
            let mut v = Vec::new();
            for count in 0..=u128::from(*n) {
                let elems = vec![inner.as_ref(); usize::try_from(count).ok()?];
                let bodies = shapes(checked, &elems, room)?;
                if bodies.is_empty() {
                    // Elements take at least one byte: more will not fit.
                    break;
                }
                v.extend(prefixed(&fixed_u32(count)?, 4, bodies));
                if v.len() > MAX_SHAPES {
                    return None;
                }
            }
            v
        }
        Type::Option(inner) => {
            if budget == 0 {
                return Some(Vec::new());
            }
            let mut v = vec![Shape {
                len: 1,
                fixed: vec![(0, 0)],
            }];
            v.extend(prefixed(&[(0, 1)], 1, one(checked, inner, budget - 1)?));
            v
        }
        Type::Named(name, _) => match find(checked, name)? {
            Item::Enum(..) | Item::Error(..) => leaf(4),
            Item::Struct(_, fields) => {
                let types: Vec<&Type> = fields.iter().map(|f| &f.ty).collect();
                shapes(checked, &types, budget)?
            }
            Item::Union(_, cases) => {
                let Some(room) = budget.checked_sub(4) else {
                    return Some(Vec::new());
                };
                let mut v = Vec::new();
                for (tag, case) in (0u128..).zip(cases) {
                    let types: Vec<&Type> = case.fields.iter().map(|f| &f.ty).collect();
                    v.extend(prefixed(
                        &fixed_u32(tag)?,
                        4,
                        shapes(checked, &types, room)?,
                    ));
                    if v.len() > MAX_SHAPES {
                        return None;
                    }
                }
                v
            }
            Item::Interface { .. } => return None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Prim;

    const SRC: &str = "/// P.\nprotocol p;\n/// E.\nerror E {\n/// A.\nA,\n}\n\
        /// Big.\nstruct Big {\n/// A.\na: u64,\n/// B.\nb: u64,\n/// C.\nc: u64,\n}\n\
        /// U.\nunion U {\n/// X.\nX {\n/// Q.\nq: u8,\n},\n/// Y.\nY,\n}\n";

    fn checked() -> Checked {
        crate::check::check(crate::syntax::parse(SRC).unwrap()).unwrap()
    }

    fn named(n: &str) -> Type {
        Type::Named(n.to_owned(), crate::syntax::Pos { line: 1, col: 1 })
    }

    #[test]
    fn an_option_has_two_layouts_when_both_fit() {
        let c = checked();
        let t = Type::Option(Box::new(Type::Prim(Prim::U64)));
        let s = shapes(&c, &[&t], 9).unwrap();
        assert_eq!(
            s,
            [
                Shape {
                    len: 1,
                    fixed: vec![(0, 0)]
                },
                Shape {
                    len: 9,
                    fixed: vec![(0, 1)]
                },
            ]
        );
        assert_eq!(shapes(&c, &[&t], 8).unwrap().len(), 1);
    }

    #[test]
    fn a_string_has_one_layout_per_length_that_fits() {
        let c = checked();
        let s = shapes(&c, &[&Type::Str(8192)], 6).unwrap();
        let lens: Vec<u128> = s.iter().map(|x| x.len).collect();
        assert_eq!(lens, [4, 5, 6]);
        assert_eq!(s[2].fixed, [(0, 2), (1, 0), (2, 0), (3, 0)]);
    }

    #[test]
    fn a_union_fixes_its_tag_and_later_fields_shift() {
        let c = checked();
        let (u, b) = (named("U"), Type::Prim(Prim::Bool));
        let s = shapes(&c, &[&u, &b], 6).unwrap();
        assert_eq!(
            s,
            [
                Shape {
                    len: 6,
                    fixed: vec![(0, 0), (1, 0), (2, 0), (3, 0)]
                },
                Shape {
                    len: 5,
                    fixed: vec![(0, 1), (1, 0), (2, 0), (3, 0)]
                },
            ]
        );
    }

    #[test]
    fn a_vec_stops_at_the_elements_that_fit_and_loops_no_further() {
        let c = checked();
        let t = Type::Vec(Box::new(named("Big")), 16);
        // 24-byte elements: none fit in 22 bytes, one in 30.
        assert_eq!(shapes(&c, &[&t], 22).unwrap().len(), 1);
        assert_eq!(shapes(&c, &[&t], 30).unwrap().len(), 2);
        assert_eq!(loop_need(&c, &t, 22), 0);
        assert_eq!(loop_need(&c, &Type::Str(8192), 22), 22);
        assert_eq!(loop_need(&c, &Type::Str(8), 22), 8);
    }

    #[test]
    fn a_list_near_the_size_limit_still_gets_an_element() {
        // A 24-byte struct and a vec<u8>: at the plain 24-byte harness
        // size only the empty list would fit.
        let c = checked();
        let (big, list) = (named("Big"), Type::Vec(Box::new(Type::Prim(Prim::U8)), 8));
        let min = 2 + 24 + 4;
        let k = harness_bytes(&c, [&big, &list], 24, min, 1000);
        assert_eq!(k, min + 1);
        let s = shapes(&c, &[&big, &list], k - 2).unwrap();
        assert!(s.iter().any(|x| x.fixed.contains(&(24, 1))), "{s:?}");
        // A union grows to its largest case; an option to its value.
        let u = named("U");
        assert_eq!(grow(&c, &u), 1);
        let o = Type::Option(Box::new(big));
        assert_eq!(grow(&c, &o), 24);
    }

    /// Each vector loop the generator relies on has its proof in hg-ipc.
    #[test]
    fn every_vec_loop_proof_exists() {
        let src = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../hg-ipc/src/bounded.rs"),
        )
        .unwrap();
        for (n, min) in VEC_LOOP_PROOFS {
            let name = format!("fn vec_of_any_element_validates_{n}_of_{min}_bytes()");
            assert!(src.contains(&name), "{name}");
            let instance = format!("validates::<BoundedVec<OpaqueMin<{min}, 62>, {n}>, 62>()");
            assert!(src.contains(&instance), "{instance}");
        }
    }

    #[test]
    fn too_many_layouts_keep_the_single_harness() {
        let c = checked();
        let s = Type::Str(8192);
        assert!(shapes(&c, &[&s, &s, &s], 60).is_none());
    }
}
