//! Rules the generator enforces, so no reviewer has to remember them
//! (security scope 6.6 [gen] rules; coding standard CS-ER-6, CS-ER-9,
//! CS-AS-6 (b), CS-TY-4).
//!
//! - Every name is documented: the generated tier P code denies
//!   `missing_docs`, so the docs come from the IDL.
//! - Exactly one closed `error` enum per protocol (CS-ER-6).
//! - Every variable-length type has a maximum, 1 to 2³²−1 (IPC-R4).
//! - Types are not recursive, so every encoding has a size bound.
//! - No zero-size struct, so a count can never be "free" on the wire.
//! - Every interface states a binding, a capacity and an overflow policy.
//! - Every message's largest encoding is at most `MAX_MESSAGE_LEN`.

use std::collections::BTreeMap;

use crate::syntax::{Binding, Case, Error, Field, File, Item, Named, Pos, Type, err};

/// Size bounds of a type's encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    /// The fewest bytes.
    pub min: u128,
    /// The most bytes.
    pub max: u128,
}

/// What the checker learned about each declared type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeFacts {
    /// Its size bounds.
    pub size: Size,
    /// Whether it holds a float (no `Eq` or `Hash`).
    pub has_float: bool,
    /// Whether it is a fieldless enum (`Copy` and `Hash`).
    pub is_plain_enum: bool,
    /// The most times any loop over it runs: the largest `N` of a
    /// `string<N>` or `vec<T, N>` inside it (0 if none). Kani's unwind
    /// bound is set from this, not from the input length.
    pub max_loop: u128,
    /// How many heap buffers deep it nests: 0 for fixed-size data, 1 for
    /// `bytes`, `string` or a `vec` of fixed-size data, 2 or more when a
    /// `vec` holds heap data. Kani proves round trips up to depth 1; past
    /// that, copies out of nested heap memory are beyond the solver.
    pub heap_depth: u8,
}

/// What the checker learned about one message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageFacts {
    /// Its size bounds, ordinal included.
    pub size: Size,
    /// As [`TypeFacts::max_loop`], over its fields.
    pub max_loop: u128,
    /// As [`TypeFacts::heap_depth`], over its fields.
    pub heap_depth: u8,
}

/// A checked file and its facts.
#[derive(Debug)]
pub struct Checked {
    /// The file.
    pub file: File,
    /// Facts per declared type.
    pub facts: BTreeMap<String, TypeFacts>,
    /// Each interface's messages, in declaration order.
    pub message_sizes: BTreeMap<String, Vec<MessageFacts>>,
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "box", "break", "const", "continue", "crate", "do", "dyn", "else",
    "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in", "let", "loop",
    "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return", "self",
    "Self", "static", "struct", "super", "trait", "true", "try", "type", "typeof", "union",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// Names the generated code binds next to fields (the reader, the writer,
/// and the Kani harnesses' locals); a field with one would shadow it.
const RESERVED_FIELDS: &[&str] = &[
    "r", "w", "len", "bytes", "out", "input", "decoded", "checked", "ipc", "handles", "h", "v",
    "ordinal", "msg", "from",
];

const RESERVED_TYPES: &[&str] = &[
    "Option",
    "Vec",
    "String",
    "Box",
    "Result",
    "Ok",
    "Err",
    "Some",
    "None",
    "Wire",
    "Reader",
    "Writer",
    "BadMessage",
    "Self",
    "Sample",
    "Gen",
];

fn is_snake(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_lowercase())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !s.contains("__")
        && !s.ends_with('_')
}

fn is_pascal(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_uppercase()) && s.chars().all(|c| c.is_ascii_alphanumeric())
}

fn documented(n: &Named, what: &str) -> Result<(), Error> {
    if n.doc.iter().all(|d| d.trim().is_empty()) {
        return err(
            n.pos,
            format!("{what} `{}` needs a `///` doc comment", n.name),
        );
    }
    Ok(())
}

fn snake(n: &Named, what: &str) -> Result<(), Error> {
    documented(n, what)?;
    if !is_snake(&n.name) || RUST_KEYWORDS.contains(&n.name.as_str()) {
        return err(
            n.pos,
            format!(
                "{what} `{}` must be snake_case and not a Rust keyword",
                n.name
            ),
        );
    }
    Ok(())
}

fn pascal(n: &Named, what: &str) -> Result<(), Error> {
    documented(n, what)?;
    if !is_pascal(&n.name) || RESERVED_TYPES.contains(&n.name.as_str()) {
        return err(
            n.pos,
            format!(
                "{what} `{}` must be PascalCase and not a reserved name",
                n.name
            ),
        );
    }
    Ok(())
}

fn unique<'a>(names: impl IntoIterator<Item = &'a Named>, what: &str) -> Result<(), Error> {
    let mut seen = BTreeMap::new();
    for n in names {
        if seen.insert(n.name.as_str(), n.pos).is_some() {
            return err(n.pos, format!("duplicate {what} `{}`", n.name));
        }
    }
    Ok(())
}

/// `message_name` → `MessageName`.
pub fn pascal_of(snake: &str) -> String {
    snake
        .split('_')
        .map(|part| {
            let mut c = part.chars();
            c.next().map_or_else(String::new, |f| {
                f.to_ascii_uppercase().to_string() + c.as_str()
            })
        })
        .collect()
}

fn bound(n: u64, pos: Pos) -> Result<(), Error> {
    if n == 0 || n > u64::from(u32::MAX) {
        return err(pos, "a maximum length must be 1 to 4294967295");
    }
    Ok(())
}

struct Ctx<'a> {
    decls: BTreeMap<&'a str, &'a Item>,
    facts: BTreeMap<String, TypeFacts>,
    visiting: Vec<String>,
}

impl Ctx<'_> {
    fn ty(&mut self, t: &Type, pos: Pos) -> Result<TypeFacts, Error> {
        let plain = |size: Size, has_float, max_loop, heap_depth| TypeFacts {
            size,
            has_float,
            is_plain_enum: false,
            max_loop,
            heap_depth,
        };
        Ok(match t {
            Type::Prim(p) => plain(
                Size {
                    min: p.size(),
                    max: p.size(),
                },
                p.is_float(),
                0,
                0,
            ),
            Type::Bytes(n) | Type::Str(n) => {
                bound(*n, pos)?;
                plain(
                    Size {
                        min: 4,
                        max: 4 + u128::from(*n),
                    },
                    false,
                    // Bytes are one copy; a string's validator loops.
                    if matches!(t, Type::Str(_)) {
                        u128::from(*n)
                    } else {
                        0
                    },
                    1,
                )
            }
            Type::Vec(inner, n) => {
                bound(*n, pos)?;
                let f = self.ty(inner, pos)?;
                if f.size.min == 0 {
                    return err(pos, "a vec element must encode to at least one byte");
                }
                plain(
                    Size {
                        min: 4,
                        max: 4u128.saturating_add(u128::from(*n).saturating_mul(f.size.max)),
                    },
                    f.has_float,
                    f.max_loop.max(u128::from(*n)),
                    f.heap_depth.saturating_add(1),
                )
            }
            Type::Option(inner) => {
                let f = self.ty(inner, pos)?;
                plain(
                    Size {
                        min: 1,
                        max: 1u128.saturating_add(f.size.max),
                    },
                    f.has_float,
                    f.max_loop,
                    f.heap_depth,
                )
            }
            Type::Named(name, npos) => self.named(name, *npos)?,
            Type::Remote(_, p) | Type::Receiver(_, p) => {
                return err(
                    *p,
                    "`remote<…>` and `receiver<…>` are handles: only a message's own parameters may be one",
                );
            }
        })
    }

    fn fields(&mut self, fields: &[Field]) -> Result<TypeFacts, Error> {
        let mut acc = TypeFacts {
            size: Size { min: 0, max: 0 },
            has_float: false,
            is_plain_enum: false,
            max_loop: 0,
            heap_depth: 0,
        };
        for f in fields {
            snake(&f.id, "field")?;
            if RESERVED_FIELDS.contains(&f.id.name.as_str()) {
                return err(
                    f.id.pos,
                    format!("field `{}` is a name the generated code uses", f.id.name),
                );
            }
            let t = self.ty(&f.ty, f.id.pos)?;
            acc.size.min = acc.size.min.saturating_add(t.size.min);
            acc.size.max = acc.size.max.saturating_add(t.size.max);
            acc.has_float |= t.has_float;
            acc.max_loop = acc.max_loop.max(t.max_loop);
            acc.heap_depth = acc.heap_depth.max(t.heap_depth);
        }
        unique(fields.iter().map(|f| &f.id), "field")?;
        Ok(acc)
    }

    fn cases(&mut self, cases: &[Case], tag: u128, what: &str) -> Result<TypeFacts, Error> {
        let mut acc = TypeFacts {
            size: Size {
                min: u128::MAX,
                max: 0,
            },
            has_float: false,
            is_plain_enum: false,
            max_loop: 0,
            heap_depth: 0,
        };
        for c in cases {
            let f = self.fields(&c.fields)?;
            acc.size.min = acc.size.min.min(f.size.min);
            acc.size.max = acc.size.max.max(f.size.max);
            acc.has_float |= f.has_float;
            acc.max_loop = acc.max_loop.max(f.max_loop);
            acc.heap_depth = acc.heap_depth.max(f.heap_depth);
            let _ = what;
        }
        acc.size.min = acc.size.min.saturating_add(tag);
        acc.size.max = acc.size.max.saturating_add(tag);
        Ok(acc)
    }

    fn named(&mut self, name: &str, pos: Pos) -> Result<TypeFacts, Error> {
        if let Some(f) = self.facts.get(name) {
            return Ok(*f);
        }
        if self.visiting.iter().any(|v| v == name) {
            return err(
                pos,
                format!("`{name}` contains itself; recursive types have no size bound"),
            );
        }
        let Some(item) = self.decls.get(name).copied() else {
            return err(pos, format!("unknown type `{name}`"));
        };
        self.visiting.push(name.to_owned());
        let facts = match item {
            Item::Error(id, vs) | Item::Enum(id, vs) => {
                if vs.is_empty() {
                    return err(id.pos, format!("`{}` needs at least one variant", id.name));
                }
                for v in vs {
                    pascal(v, "variant")?;
                }
                unique(vs, "variant")?;
                TypeFacts {
                    size: Size { min: 4, max: 4 },
                    has_float: false,
                    is_plain_enum: true,
                    max_loop: 0,
                    heap_depth: 0,
                }
            }
            Item::Union(id, cases) => {
                if cases.is_empty() {
                    return err(id.pos, format!("`{}` needs at least one case", id.name));
                }
                for c in cases {
                    pascal(&c.id, "union case")?;
                }
                unique(cases.iter().map(|c| &c.id), "union case")?;
                self.cases(cases, 4, "case")?
            }
            Item::Struct(id, fields) => {
                if fields.is_empty() {
                    return err(
                        id.pos,
                        format!("struct `{}` needs at least one field", id.name),
                    );
                }
                self.fields(fields)?
            }
            Item::Interface { id, .. } => {
                return err(pos, format!("`{}` is an interface, not a type", id.name));
            }
        };
        self.visiting.pop();
        self.facts.insert(name.to_owned(), facts);
        Ok(facts)
    }
}

fn item_id(item: &Item) -> &Named {
    match item {
        Item::Error(id, _)
        | Item::Enum(id, _)
        | Item::Union(id, _)
        | Item::Struct(id, _)
        | Item::Interface { id, .. } => id,
    }
}

/// Checks `file`.
///
/// # Errors
/// The first rule broken.
pub fn check(file: File) -> Result<Checked, Error> {
    snake(&file.protocol, "protocol")?;
    unique(file.items.iter().map(item_id), "declaration")?;
    let errors: Vec<&Named> = file
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Error(id, _) => Some(id),
            _ => None,
        })
        .collect();
    match errors.as_slice() {
        [] => {
            return err(
                file.protocol.pos,
                "a protocol declares one closed `error` enum (CS-ER-6)",
            );
        }
        [_] => {}
        [_, second, ..] => return err(second.pos, "a protocol declares only one `error` enum"),
    }
    let mut ctx = Ctx {
        decls: file
            .items
            .iter()
            .map(|i| (item_id(i).name.as_str(), i))
            .collect(),
        facts: BTreeMap::new(),
        visiting: Vec::new(),
    };
    let mut message_sizes = BTreeMap::new();
    for item in &file.items {
        let id = item_id(item);
        pascal(id, "declaration")?;
        match item {
            Item::Interface {
                id,
                capacity,
                messages,
                ..
            } => {
                if *capacity == 0 || *capacity > u64::from(u32::MAX) {
                    return err(id.pos, "capacity must be 1 to 4294967295");
                }
                if messages.is_empty() {
                    return err(
                        id.pos,
                        format!("interface `{}` needs at least one message", id.name),
                    );
                }
                if messages.len() > usize::from(u16::MAX) + 1 {
                    return err(id.pos, "an interface has at most 65536 messages");
                }
                for m in messages {
                    snake(&m.id, "message")?;
                }
                unique(messages.iter().map(|m| &m.id), "message")?;
                // Each message becomes a variant named `pascal_of(name)`, so
                // `x_1` and `x1` would both be `X1`.
                let mut variants = BTreeMap::new();
                for m in messages {
                    if let Some(other) = variants.insert(pascal_of(&m.id.name), &m.id.name) {
                        return err(
                            m.id.pos,
                            format!(
                                "messages `{other}` and `{}` both become `{}`",
                                m.id.name,
                                pascal_of(&m.id.name)
                            ),
                        );
                    }
                }
                let mut sizes = Vec::new();
                for m in messages {
                    unique(m.fields.iter().map(|f| &f.id), "parameter")?;
                    let mut handles = 0usize;
                    for f in m.fields.iter().filter(|f| f.ty.is_handle()) {
                        snake(&f.id, "parameter")?;
                        if RESERVED_FIELDS.contains(&f.id.name.as_str()) {
                            return err(
                                f.id.pos,
                                format!("field `{}` is a name the generated code uses", f.id.name),
                            );
                        }
                        let (Type::Remote(target, tpos) | Type::Receiver(target, tpos)) = &f.ty
                        else {
                            continue;
                        };
                        match ctx.decls.get(target.as_str()) {
                            Some(Item::Interface { binding, .. }) => {
                                if matches!(f.ty, Type::Receiver(..)) && *binding == Binding::Bound
                                {
                                    return err(
                                        *tpos,
                                        format!(
                                            "`{target}` is bound: its receiver stays with the process that serves it (IPC-R2)"
                                        ),
                                    );
                                }
                            }
                            Some(_) => {
                                return err(
                                    *tpos,
                                    format!("`{target}` is a type, not an interface"),
                                );
                            }
                            None => return err(*tpos, format!("unknown interface `{target}`")),
                        }
                        handles = handles.saturating_add(1);
                    }
                    if handles > hg_ipc::MAX_MESSAGE_HANDLES {
                        return err(
                            m.id.pos,
                            format!(
                                "message `{}` carries {handles} handles, over the {} cap",
                                m.id.name,
                                hg_ipc::MAX_MESSAGE_HANDLES
                            ),
                        );
                    }
                    let data: Vec<Field> = m
                        .fields
                        .iter()
                        .filter(|f| !f.ty.is_handle())
                        .cloned()
                        .collect();
                    let facts = ctx.fields(&data)?;
                    let size = facts.size.max.saturating_add(2);
                    sizes.push(MessageFacts {
                        size: Size {
                            min: facts.size.min.saturating_add(2),
                            max: size,
                        },
                        max_loop: facts.max_loop,
                        heap_depth: facts.heap_depth,
                    });
                    if size > hg_ipc::MAX_MESSAGE_LEN as u128 {
                        return err(
                            m.id.pos,
                            format!(
                                "message `{}` can encode to {size} bytes, over the {} byte cap; send bulk data as shared memory",
                                m.id.name,
                                hg_ipc::MAX_MESSAGE_LEN
                            ),
                        );
                    }
                }
                message_sizes.insert(id.name.clone(), sizes);
            }
            _ => {
                ctx.named(&id.name, id.pos)?;
            }
        }
    }
    Ok(Checked {
        facts: ctx.facts,
        file,
        message_sizes,
    })
}
