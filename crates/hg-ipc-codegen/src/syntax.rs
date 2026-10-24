//! `.hgidl` syntax: the lexer, the tree and the parser.
//!
//! ```text
//! file      = doc* "protocol" NAME ";" item*
//! item      = doc* ( "error" NAME "{" variant* "}"
//!                  | "enum" NAME "{" variant* "}"
//!                  | "union" NAME "{" case* "}"
//!                  | "struct" NAME "{" field* "}"
//!                  | "interface" NAME binding "capacity" NUMBER "overflow" policy
//!                    "{" message* "}" )
//! variant   = doc* NAME ","
//! case      = doc* NAME [ "{" field* "}" ] ","
//! field     = doc* NAME ":" type ","
//! message   = doc* NAME "(" [ param { "," param } [","] ] ")" ";"
//! param     = doc* NAME ":" type
//! binding   = "bound" | "free"
//! policy    = "drop" | "terminate"
//! type      = "bool" | "u8" | "u16" | "u32" | "u64" | "i32" | "i64" | "f32" | "f64"
//!           | "bytes" "<" NUMBER ">" | "string" "<" NUMBER ">"
//!           | "vec" "<" type "," NUMBER ">" | "option" "<" type ">" | NAME
//! doc       = "///" text to end of line
//! ```
//!
//! `//` starts a comment. Numbers are decimal and may use `_`.

use std::fmt;

/// A position in the source, 1-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    /// Line.
    pub line: u32,
    /// Column, in characters.
    pub col: u32,
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// A generator error, at a source position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// Where.
    pub pos: Pos,
    /// What.
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.pos, self.message)
    }
}

impl std::error::Error for Error {}

pub(crate) fn err<T>(pos: Pos, message: impl Into<String>) -> Result<T, Error> {
    Err(Error {
        pos,
        message: message.into(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Name(String),
    Number(u64),
    Doc(String),
    Punct(char),
    End,
}

#[derive(Debug, Clone)]
struct Token {
    tok: Tok,
    pos: Pos,
}

fn lex(src: &str) -> Result<Vec<Token>, Error> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    let (mut line, mut col) = (1u32, 1u32);
    while let Some(&c) = chars.peek() {
        let pos = Pos { line, col };
        let mut bump = |chars: &mut std::iter::Peekable<std::str::Chars<'_>>| {
            let c = chars.next();
            if c == Some('\n') {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            c
        };
        if c.is_whitespace() {
            bump(&mut chars);
        } else if c == '/' {
            bump(&mut chars);
            if chars.peek() != Some(&'/') {
                return err(pos, "expected `//` or `///`");
            }
            bump(&mut chars);
            let doc = chars.peek() == Some(&'/');
            if doc {
                bump(&mut chars);
            }
            let mut text = String::new();
            while let Some(&c) = chars.peek() {
                if c == '\n' {
                    break;
                }
                text.push(c);
                bump(&mut chars);
            }
            if doc {
                out.push(Token {
                    tok: Tok::Doc(
                        text.strip_prefix(' ')
                            .unwrap_or(&text)
                            .trim_end()
                            .to_owned(),
                    ),
                    pos,
                });
            }
        } else if c.is_ascii_alphabetic() || c == '_' {
            let mut s = String::new();
            while let Some(&c) = chars.peek() {
                if !(c.is_ascii_alphanumeric() || c == '_') {
                    break;
                }
                s.push(c);
                bump(&mut chars);
            }
            out.push(Token {
                tok: Tok::Name(s),
                pos,
            });
        } else if c.is_ascii_digit() {
            let mut n: u64 = 0;
            while let Some(&c) = chars.peek() {
                if c == '_' {
                    bump(&mut chars);
                    continue;
                }
                let Some(d) = c.to_digit(10) else { break };
                n = match n.checked_mul(10).and_then(|n| n.checked_add(u64::from(d))) {
                    Some(n) => n,
                    None => return err(pos, "number too large"),
                };
                bump(&mut chars);
            }
            out.push(Token {
                tok: Tok::Number(n),
                pos,
            });
        } else if "{}()<>,;:".contains(c) {
            bump(&mut chars);
            out.push(Token {
                tok: Tok::Punct(c),
                pos,
            });
        } else {
            return err(pos, format!("unexpected character `{c}`"));
        }
    }
    out.push(Token {
        tok: Tok::End,
        pos: Pos { line, col },
    });
    Ok(out)
}

/// A field or parameter type, as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// `bool`, `u8` … `f64`.
    Prim(Prim),
    /// `bytes<N>`.
    Bytes(u64),
    /// `string<N>`.
    Str(u64),
    /// `vec<T, N>`.
    Vec(Box<Type>, u64),
    /// `option<T>`.
    Option(Box<Type>),
    /// A declared type.
    Named(String, Pos),
    /// `remote<Iface>`: the right to send `Iface`'s messages (a handle;
    /// message parameters only).
    Remote(String, Pos),
    /// `receiver<Iface>`: the right to receive them (a handle; message
    /// parameters only).
    Receiver(String, Pos),
}

impl Type {
    /// Whether it is a handle, carried beside the bytes, not in them.
    pub const fn is_handle(&self) -> bool {
        matches!(self, Self::Remote(..) | Self::Receiver(..))
    }
}

/// The fixed-width types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prim {
    /// `bool`.
    Bool,
    /// `u8`.
    U8,
    /// `u16`.
    U16,
    /// `u32`.
    U32,
    /// `u64`.
    U64,
    /// `i32`.
    I32,
    /// `i64`.
    I64,
    /// `f32`.
    F32,
    /// `f64`.
    F64,
}

impl Prim {
    fn from_name(s: &str) -> Option<Self> {
        Some(match s {
            "bool" => Self::Bool,
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u32" => Self::U32,
            "u64" => Self::U64,
            "i32" => Self::I32,
            "i64" => Self::I64,
            "f32" => Self::F32,
            "f64" => Self::F64,
            _ => return None,
        })
    }

    /// The Rust type.
    pub fn rust(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }

    /// Encoded size in bytes.
    pub fn size(self) -> u128 {
        match self {
            Self::Bool | Self::U8 => 1,
            Self::U16 => 2,
            Self::U32 | Self::I32 | Self::F32 => 4,
            Self::U64 | Self::I64 | Self::F64 => 8,
        }
    }

    /// Whether it can derive `Eq` and `Hash`.
    pub fn is_float(self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }
}

/// A named, documented thing.
#[derive(Debug, Clone)]
pub struct Named {
    /// The name.
    pub name: String,
    /// Its doc lines.
    pub doc: Vec<String>,
    /// Where it was declared.
    pub pos: Pos,
}

/// A struct field, union case field or message parameter.
#[derive(Debug, Clone)]
pub struct Field {
    /// Name and docs.
    pub id: Named,
    /// Its type.
    pub ty: Type,
}

/// A union case.
#[derive(Debug, Clone)]
pub struct Case {
    /// Name and docs.
    pub id: Named,
    /// Its fields (empty for a unit case).
    pub fields: Vec<Field>,
}

/// An interface's binding (IPC-R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    /// `bound`.
    Bound,
    /// `free`.
    Free,
}

/// An interface's overflow policy (CS-AS-6 (b)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    /// `drop`.
    Drop,
    /// `terminate`.
    Terminate,
}

/// A declaration.
#[derive(Debug, Clone)]
pub enum Item {
    /// The protocol's closed error enum (CS-ER-6).
    Error(Named, Vec<Named>),
    /// A closed enum.
    Enum(Named, Vec<Named>),
    /// A tagged union.
    Union(Named, Vec<Case>),
    /// A struct.
    Struct(Named, Vec<Field>),
    /// An interface: one direction of an endpoint.
    Interface {
        /// Name and docs.
        id: Named,
        /// Bound or free.
        binding: Binding,
        /// Queue capacity.
        capacity: u64,
        /// Overflow policy.
        overflow: Overflow,
        /// Its messages.
        messages: Vec<Case>,
    },
}

/// A parsed `.hgidl` file.
#[derive(Debug, Clone)]
pub struct File {
    /// The protocol.
    pub protocol: Named,
    /// Its declarations, in order.
    pub items: Vec<Item>,
}

struct Parser {
    toks: Vec<Token>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        let last = self.toks.len().saturating_sub(1);
        &self.toks[self.at.min(last)]
    }

    fn next(&mut self) -> Token {
        let t = self.peek().clone();
        if t.tok != Tok::End {
            self.at += 1;
        }
        t
    }

    fn docs(&mut self) -> Vec<String> {
        let mut docs = Vec::new();
        while let Tok::Doc(d) = &self.peek().tok {
            docs.push(d.clone());
            self.next();
        }
        docs
    }

    fn punct(&mut self, c: char) -> Result<(), Error> {
        let t = self.next();
        if t.tok == Tok::Punct(c) {
            Ok(())
        } else {
            err(t.pos, format!("expected `{c}`, found {}", show(&t.tok)))
        }
    }

    fn is_punct(&self, c: char) -> bool {
        self.peek().tok == Tok::Punct(c)
    }

    fn name(&mut self) -> Result<(String, Pos), Error> {
        let t = self.next();
        match t.tok {
            Tok::Name(n) => Ok((n, t.pos)),
            other => err(t.pos, format!("expected a name, found {}", show(&other))),
        }
    }

    fn keyword(&mut self, k: &str) -> Result<(), Error> {
        let (n, pos) = self.name()?;
        if n == k {
            Ok(())
        } else {
            err(pos, format!("expected `{k}`, found `{n}`"))
        }
    }

    fn number(&mut self) -> Result<(u64, Pos), Error> {
        let t = self.next();
        match t.tok {
            Tok::Number(n) => Ok((n, t.pos)),
            other => err(t.pos, format!("expected a number, found {}", show(&other))),
        }
    }

    fn named(&mut self, doc: Vec<String>) -> Result<Named, Error> {
        let (name, pos) = self.name()?;
        Ok(Named { name, doc, pos })
    }

    fn ty(&mut self) -> Result<Type, Error> {
        let (n, pos) = self.name()?;
        if let Some(p) = Prim::from_name(&n) {
            return Ok(Type::Prim(p));
        }
        Ok(match n.as_str() {
            "bytes" | "string" => {
                self.punct('<')?;
                let (len, _) = self.number()?;
                self.punct('>')?;
                if n == "bytes" {
                    Type::Bytes(len)
                } else {
                    Type::Str(len)
                }
            }
            "vec" => {
                self.punct('<')?;
                let inner = self.ty()?;
                self.punct(',')?;
                let (len, _) = self.number()?;
                self.punct('>')?;
                Type::Vec(Box::new(inner), len)
            }
            "option" => {
                self.punct('<')?;
                let inner = self.ty()?;
                self.punct('>')?;
                Type::Option(Box::new(inner))
            }
            "remote" | "receiver" => {
                self.punct('<')?;
                let (iface, ipos) = self.name()?;
                self.punct('>')?;
                if n == "remote" {
                    Type::Remote(iface, ipos)
                } else {
                    Type::Receiver(iface, ipos)
                }
            }
            _ => Type::Named(n, pos),
        })
    }

    /// `doc* NAME ":" type` then `end` (`,` for fields).
    fn field(&mut self) -> Result<Field, Error> {
        let doc = self.docs();
        let id = self.named(doc)?;
        self.punct(':')?;
        let ty = self.ty()?;
        Ok(Field { id, ty })
    }

    fn fields_block(&mut self) -> Result<Vec<Field>, Error> {
        self.punct('{')?;
        let mut fields = Vec::new();
        loop {
            if self.next_is_close() {
                break;
            }
            fields.push(self.field()?);
            self.punct(',')?;
        }
        self.punct('}')?;
        Ok(fields)
    }

    /// Whether the next non-doc token is `}` (a stray doc before it is
    /// an error the caller reports at the doc).
    fn next_is_close(&self) -> bool {
        self.is_punct('}')
    }

    fn variants(&mut self) -> Result<Vec<Named>, Error> {
        self.punct('{')?;
        let mut v = Vec::new();
        while !self.next_is_close() {
            let doc = self.docs();
            v.push(self.named(doc)?);
            self.punct(',')?;
        }
        self.punct('}')?;
        Ok(v)
    }

    fn cases(&mut self) -> Result<Vec<Case>, Error> {
        self.punct('{')?;
        let mut v = Vec::new();
        while !self.next_is_close() {
            let doc = self.docs();
            let id = self.named(doc)?;
            let fields = if self.is_punct('{') {
                self.fields_block()?
            } else {
                Vec::new()
            };
            self.punct(',')?;
            v.push(Case { id, fields });
        }
        self.punct('}')?;
        Ok(v)
    }

    fn messages(&mut self) -> Result<Vec<Case>, Error> {
        self.punct('{')?;
        let mut v = Vec::new();
        while !self.next_is_close() {
            let doc = self.docs();
            let id = self.named(doc)?;
            self.punct('(')?;
            let mut fields = Vec::new();
            while !self.is_punct(')') {
                fields.push(self.field()?);
                if !self.is_punct(')') {
                    self.punct(',')?;
                }
            }
            self.punct(')')?;
            self.punct(';')?;
            v.push(Case { id, fields });
        }
        self.punct('}')?;
        Ok(v)
    }

    fn file(&mut self) -> Result<File, Error> {
        let doc = self.docs();
        self.keyword("protocol")?;
        let protocol = self.named(doc)?;
        self.punct(';')?;
        let mut items = Vec::new();
        loop {
            let doc = self.docs();
            let t = self.next();
            let kw = match t.tok {
                Tok::End if doc.is_empty() => break,
                Tok::End => return err(t.pos, "a doc comment documents nothing"),
                Tok::Name(n) => n,
                other => {
                    return err(
                        t.pos,
                        format!("expected a declaration, found {}", show(&other)),
                    );
                }
            };
            let id = self.named(doc)?;
            items.push(match kw.as_str() {
                "error" => Item::Error(id, self.variants()?),
                "enum" => Item::Enum(id, self.variants()?),
                "union" => Item::Union(id, self.cases()?),
                "struct" => Item::Struct(id, self.fields_block()?),
                "interface" => {
                    let (b, bpos) = self.name()?;
                    let binding = match b.as_str() {
                        "bound" => Binding::Bound,
                        "free" => Binding::Free,
                        _ => return err(bpos, "an interface is `bound` or `free` (IPC-R2)"),
                    };
                    self.keyword("capacity")?;
                    let (capacity, _) = self.number()?;
                    self.keyword("overflow")?;
                    let (o, opos) = self.name()?;
                    let overflow = match o.as_str() {
                        "drop" => Overflow::Drop,
                        "terminate" => Overflow::Terminate,
                        _ => return err(opos, "overflow is `drop` or `terminate` (CS-AS-6 (b))"),
                    };
                    Item::Interface {
                        id,
                        binding,
                        capacity,
                        overflow,
                        messages: self.messages()?,
                    }
                }
                _ => return err(t.pos, format!("unknown declaration `{kw}`")),
            });
        }
        Ok(File { protocol, items })
    }
}

fn show(t: &Tok) -> String {
    match t {
        Tok::Name(n) => format!("`{n}`"),
        Tok::Number(n) => format!("`{n}`"),
        Tok::Doc(_) => "a doc comment".to_owned(),
        Tok::Punct(c) => format!("`{c}`"),
        Tok::End => "the end of the file".to_owned(),
    }
}

/// Parses `.hgidl` source.
///
/// # Errors
/// The first syntax error.
pub fn parse(src: &str) -> Result<File, Error> {
    Parser {
        toks: lex(src)?,
        at: 0,
    }
    .file()
}
