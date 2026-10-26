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

use crate::check::Checked;
use crate::syntax::{Item, Type};

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
    fn too_many_layouts_keep_the_single_harness() {
        let c = checked();
        let s = Type::Str(8192);
        assert!(shapes(&c, &[&s, &s, &s], 60).is_none());
    }
}
