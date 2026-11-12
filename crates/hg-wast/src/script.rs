//! The script commands.
//!
//! A `.wast` file is a sequence of commands: modules to define, names to
//! register, actions to perform and assertions about what they do. This
//! module reads that layer exactly and leaves module bodies alone — each
//! is kept as the byte range it occupies, to be handed to an engine
//! verbatim ([`Module::text`]).
//!
//! # Annotations
//!
//! The format admits annotations, `(@name …)`, anywhere a token may
//! appear, and a reader that does not understand one must skip it. The
//! suite's `annotations.wast` puts them in the one place a careless
//! parser will not look — before a command's own keyword, as in
//! `((@a) module (@a) $m …)` — so they are skipped by the cursor itself
//! rather than at each site that reads a token.

use crate::lex::{Error, Kind, Lexer, Span, Token};
use crate::value::{self, NumType, Pattern, Shape, Value};

/// A module command, in any of the forms a script may write one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module {
    /// The module's identifier, if it was named.
    pub id: Option<String>,
    /// How the module's content was given.
    pub kind: ModuleKind,
    /// `(module definition …)`: defined, not instantiated.
    pub definition: bool,
    /// The whole command, `(module` to its `)`.
    pub span: Span,
}

impl Module {
    /// Checks that the module's own span parses back to this module.
    ///
    /// The span is what an engine is handed, so this is the claim the
    /// whole design rests on: the bytes between those two offsets are a
    /// module command, the one written there, and nothing else. A span
    /// that stopped a byte early would still have parsed once, in the
    /// whole file, and gone unnoticed; parsing it again on its own is
    /// what makes the boundary itself testable.
    pub fn reparses(&self, src: &str) -> Result<(), String> {
        let text = self.span.text(src);
        let again = parse(text).map_err(|e| format!("the span does not parse: {e}"))?;
        match again.as_slice() {
            [Cmd::Module(n)]
                if n.id == self.id && n.kind == self.kind && n.definition == self.definition =>
            {
                Ok(())
            }
            [Cmd::Module(_)] => Err("the span parses as a different module".to_string()),
            other => Err(format!(
                "the span parses as {} command(s), not one module",
                other.len()
            )),
        }
    }

    /// The command's source text, to hand to an engine unchanged.
    ///
    /// This is why the parser does not build a tree of the module: the
    /// bytes that went in are the bytes that come out, so nothing the
    /// runner does to a module can change what is being tested.
    #[must_use]
    pub fn text<'a>(&self, src: &'a str) -> &'a str {
        self.span.text(src)
    }
}

/// How a module's content was given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleKind {
    /// Ordinary text, in the module's own syntax.
    Text,
    /// `binary` and string literals: the module's bytes, which the test
    /// may intend to be malformed.
    Binary(Vec<u8>),
    /// `quote` and string literals: the module's text, which the test
    /// may intend to be malformed.
    Quote(Vec<u8>),
}

/// Something a script does to an instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// `(invoke $m? "name" const*)`
    Invoke {
        /// The instance, if one was named.
        module: Option<String>,
        /// The export's name.
        name: String,
        /// The arguments.
        args: Vec<Value>,
        /// The whole action.
        span: Span,
    },
    /// `(get $m? "name")`
    Get {
        /// The instance, if one was named.
        module: Option<String>,
        /// The export's name.
        name: String,
        /// The whole action.
        span: Span,
    },
}

impl Action {
    /// The whole action's span.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Action::Invoke { span, .. } | Action::Get { span, .. } => *span,
        }
    }
}

/// What an `assert_trap` is about: a trap while running, or a trap
/// while instantiating.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrapOf {
    /// A trap during an action.
    Action(Action),
    /// A trap during instantiation.
    Module(Module),
}

/// One command of a script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cmd {
    /// Define, validate and — unless it is a `definition` — instantiate.
    Module(Module),
    /// `(module instance $i $m)`: instantiate a defined module.
    Instance {
        /// The new instance's identifier.
        instance: Option<String>,
        /// The defined module's identifier.
        module: Option<String>,
        /// The whole command.
        span: Span,
    },
    /// `(register "name" $m?)`: make an instance importable.
    Register {
        /// The name to register under.
        name: String,
        /// The instance, if one was named.
        module: Option<String>,
        /// The whole command.
        span: Span,
    },
    /// An action on its own, performed for its effect.
    Action(Action),
    /// `(assert_return action pattern*)`
    AssertReturn {
        /// What to do.
        action: Action,
        /// What each result must be.
        expected: Vec<Pattern>,
        /// The whole command.
        span: Span,
    },
    /// `(assert_trap … "message")`
    AssertTrap {
        /// What traps.
        of: TrapOf,
        /// The expected failure text.
        message: String,
        /// The whole command.
        span: Span,
    },
    /// `(assert_exhaustion action "message")`
    AssertExhaustion {
        /// What exhausts the engine.
        action: Action,
        /// The expected failure text.
        message: String,
        /// The whole command.
        span: Span,
    },
    /// `(assert_exception action)`
    AssertException {
        /// What throws.
        action: Action,
        /// The whole command.
        span: Span,
    },
    /// `(assert_malformed module "message")`
    AssertMalformed {
        /// The module that must not decode.
        module: Module,
        /// The expected failure text.
        message: String,
        /// The whole command.
        span: Span,
    },
    /// `(assert_invalid module "message")`
    AssertInvalid {
        /// The module that must not validate.
        module: Module,
        /// The expected failure text.
        message: String,
        /// The whole command.
        span: Span,
    },
    /// `(assert_unlinkable module "message")`
    AssertUnlinkable {
        /// The module that must not link.
        module: Module,
        /// The expected failure text.
        message: String,
        /// The whole command.
        span: Span,
    },
    /// A meta command: `script`, `input` or `output`. These direct the
    /// reference interpreter's own tooling rather than test an engine,
    /// so they are recognised and kept, not acted on.
    Meta {
        /// Which one.
        name: String,
        /// The whole command.
        span: Span,
    },
}

impl Cmd {
    /// The whole command's span.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Cmd::Module(m) => m.span,
            Cmd::Action(a) => a.span(),
            Cmd::Instance { span, .. }
            | Cmd::Register { span, .. }
            | Cmd::AssertReturn { span, .. }
            | Cmd::AssertTrap { span, .. }
            | Cmd::AssertExhaustion { span, .. }
            | Cmd::AssertException { span, .. }
            | Cmd::AssertMalformed { span, .. }
            | Cmd::AssertInvalid { span, .. }
            | Cmd::AssertUnlinkable { span, .. }
            | Cmd::Meta { span, .. } => *span,
        }
    }

    /// The modules this command names, in the order it names them.
    #[must_use]
    pub fn modules(&self) -> Vec<&Module> {
        match self {
            Cmd::Module(m)
            | Cmd::AssertMalformed { module: m, .. }
            | Cmd::AssertInvalid { module: m, .. }
            | Cmd::AssertUnlinkable { module: m, .. }
            | Cmd::AssertTrap {
                of: TrapOf::Module(m),
                ..
            } => vec![m],
            _ => vec![],
        }
    }

    /// The command's keyword, for ledgers and messages.
    #[must_use]
    pub fn keyword(&self) -> &'static str {
        match self {
            Cmd::Module(_) => "module",
            Cmd::Instance { .. } => "module instance",
            Cmd::Register { .. } => "register",
            Cmd::Action(Action::Invoke { .. }) => "invoke",
            Cmd::Action(Action::Get { .. }) => "get",
            Cmd::AssertReturn { .. } => "assert_return",
            Cmd::AssertTrap { .. } => "assert_trap",
            Cmd::AssertExhaustion { .. } => "assert_exhaustion",
            Cmd::AssertException { .. } => "assert_exception",
            Cmd::AssertMalformed { .. } => "assert_malformed",
            Cmd::AssertInvalid { .. } => "assert_invalid",
            Cmd::AssertUnlinkable { .. } => "assert_unlinkable",
            Cmd::Meta { .. } => "meta",
        }
    }
}

/// Reads a whole `.wast` source into its commands.
pub fn parse(src: &str) -> Result<Vec<Cmd>, Error> {
    let toks = Lexer::new(src).tokens()?;
    let mut cur = Cursor {
        src,
        toks: &toks,
        i: 0,
    };
    // "In a source file, the toplevel `( module … )` surrounding the
    // module body may be omitted" (text format, modules, Abbreviations).
    // The suite's `inline-module.wast` is exactly that: a whole file of
    // module fields. Such a file holds one module and no commands, so
    // the first keyword settles which of the two a file is.
    //
    // A file that opens with any other keyword is therefore an inline
    // module whatever follows, and `(frobnicate)` alone is one too.
    // That is not a gap being papered over: a module body is never read
    // here, so there is nothing this crate could say about its
    // contents, and an engine handed the file is what decides. A file
    // that does open with a command keyword is a script, and from there
    // an unknown command is named as one.
    if let Some(first) = cur.first_keyword()
        && !is_command_keyword(first)
    {
        let span = Span {
            start: toks.first().map_or(0, |t| t.span.start),
            end: toks.last().map_or(0, |t| t.span.end),
        };
        return Ok(vec![Cmd::Module(Module {
            id: None,
            kind: ModuleKind::Text,
            definition: false,
            span,
        })]);
    }
    let mut out = Vec::new();
    while cur.peek().is_some() {
        out.push(cur.command()?);
    }
    Ok(out)
}

/// True for the keywords that begin a script command. Everything else
/// at the top of a file begins a module field, and so an inline module.
fn is_command_keyword(word: &str) -> bool {
    matches!(
        word,
        "module"
            | "register"
            | "invoke"
            | "get"
            | "assert_return"
            | "assert_trap"
            | "assert_exhaustion"
            | "assert_exception"
            | "assert_malformed"
            | "assert_invalid"
            | "assert_unlinkable"
            | "script"
            | "input"
            | "output"
    )
}

/// A position in the token stream, which hides annotations.
struct Cursor<'a> {
    src: &'a str,
    toks: &'a [Token],
    i: usize,
}

impl<'a> Cursor<'a> {
    /// The keyword of the first command-shaped group, without moving.
    fn first_keyword(&mut self) -> Option<&'a str> {
        let save = self.i;
        self.skip_annotations();
        let open = self.toks.get(self.i).copied();
        let word = match (open, self.toks.get(self.i + 1)) {
            (Some(o), Some(w)) if o.kind == Kind::LParen && w.kind == Kind::Word => {
                Some(w.span.text(self.src))
            }
            _ => None,
        };
        self.i = save;
        word
    }

    /// The next token, with any annotation group before it skipped.
    fn peek(&mut self) -> Option<Token> {
        self.skip_annotations();
        self.toks.get(self.i).copied()
    }

    /// Skips `(@name …)` groups, which a reader that does not know an
    /// annotation must ignore wherever it appears.
    fn skip_annotations(&mut self) {
        while self.is_annotation_start() {
            let mut depth = 0usize;
            while let Some(t) = self.toks.get(self.i) {
                self.i += 1;
                match t.kind {
                    Kind::LParen => depth += 1,
                    Kind::RParen => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// True at a `(@…` that opens an annotation.
    ///
    /// The `@` must touch the parenthesis: `(@a …)` is an annotation
    /// and `( @a …)` is not. Its token may be a word — `(@a)` — or a
    /// reserved token — `(@"a")`, which the suite also writes, since
    /// `@` and a string make one run.
    fn is_annotation_start(&self) -> bool {
        let Some(open) = self.toks.get(self.i) else {
            return false;
        };
        if open.kind != Kind::LParen {
            return false;
        }
        matches!(self.toks.get(self.i + 1), Some(t)
            if t.span.start == open.span.end
                && matches!(t.kind, Kind::Word | Kind::Reserved)
                && t.span.text(self.src).starts_with('@'))
    }

    /// The offset to report an error at when the stream has run out.
    fn end_offset(&self) -> usize {
        self.toks.last().map_or(0, |t| t.span.end)
    }

    fn err<T>(&self, at: Option<Token>, message: impl Into<String>) -> Result<T, Error> {
        Err(Error {
            offset: at.map_or_else(|| self.end_offset(), |t| t.span.start),
            message: message.into(),
        })
    }

    fn expect(&mut self, kind: Kind, what: &str) -> Result<Token, Error> {
        let t = self.peek();
        match t {
            Some(t) if t.kind == kind => {
                self.i += 1;
                Ok(t)
            }
            other => self.err(other, format!("expected {what}")),
        }
    }

    /// The text of the next token if it is a word, without consuming it.
    fn peek_word(&mut self) -> Option<&'a str> {
        match self.peek() {
            Some(t) if t.kind == Kind::Word => Some(t.span.text(self.src)),
            _ => None,
        }
    }

    /// Consumes the next token if it is the word `w`.
    fn eat_word(&mut self, w: &str) -> bool {
        if self.peek_word() == Some(w) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    /// Consumes an identifier if one is next.
    fn opt_id(&mut self) -> Option<String> {
        match self.peek() {
            Some(t) if t.kind == Kind::Id => {
                self.i += 1;
                Some(t.span.text(self.src).to_string())
            }
            _ => None,
        }
    }

    /// Consumes a string and returns its text.
    fn name(&mut self) -> Result<String, Error> {
        let t = self.expect(Kind::Str, "a name in quotation marks")?;
        value::string_text(self.src, t.span)
    }

    /// Consumes a string and returns its bytes.
    fn bytes(&mut self) -> Result<Vec<u8>, Error> {
        let t = self.expect(Kind::Str, "a string")?;
        value::string_bytes(self.src, t.span)
    }

    /// Skips the balanced group that starts at the current `(`.
    fn skip_group(&mut self) -> Result<Span, Error> {
        let open = self.expect(Kind::LParen, "(")?;
        let mut depth = 1usize;
        loop {
            let Some(t) = self.toks.get(self.i) else {
                return Err(Error {
                    offset: open.span.start,
                    message: "this ( is never closed".to_string(),
                });
            };
            self.i += 1;
            match t.kind {
                Kind::LParen => depth += 1,
                Kind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(open.span.to(t.span));
                    }
                }
                _ => {}
            }
        }
    }

    /// Reads one command.
    fn command(&mut self) -> Result<Cmd, Error> {
        let open = self.expect(Kind::LParen, "a command")?;
        // Where that parenthesis is, so a command that wants the whole
        // group can go back to it. Counting backwards from here would
        // not do: reading the keyword may skip an annotation, and the
        // count would land inside it.
        let open_at = self.i - 1;
        let head = self.expect(Kind::Word, "a command's keyword")?;
        let word = head.span.text(self.src);
        match word {
            "module" => self.module_cmd(open),
            "register" => {
                let name = self.name()?;
                let module = self.opt_id();
                let close = self.expect(Kind::RParen, ")")?;
                Ok(Cmd::Register {
                    name,
                    module,
                    span: open.span.to(close.span),
                })
            }
            "invoke" | "get" => {
                let action = self.action_body(open, word)?;
                Ok(Cmd::Action(action))
            }
            "assert_return" => {
                let action = self.action()?;
                let mut expected = Vec::new();
                while self.peek().is_some_and(|t| t.kind == Kind::LParen) {
                    expected.push(self.pattern()?);
                }
                let close = self.expect(Kind::RParen, ")")?;
                Ok(Cmd::AssertReturn {
                    action,
                    expected,
                    span: open.span.to(close.span),
                })
            }
            "assert_trap" => {
                // Either an action that traps, or a module whose
                // instantiation traps.
                let of = if self.looks_like_module() {
                    TrapOf::Module(self.module()?)
                } else {
                    TrapOf::Action(self.action()?)
                };
                let message = self.name()?;
                let close = self.expect(Kind::RParen, ")")?;
                Ok(Cmd::AssertTrap {
                    of,
                    message,
                    span: open.span.to(close.span),
                })
            }
            "assert_exhaustion" => {
                let action = self.action()?;
                let message = self.name()?;
                let close = self.expect(Kind::RParen, ")")?;
                Ok(Cmd::AssertExhaustion {
                    action,
                    message,
                    span: open.span.to(close.span),
                })
            }
            "assert_exception" => {
                let action = self.action()?;
                let close = self.expect(Kind::RParen, ")")?;
                Ok(Cmd::AssertException {
                    action,
                    span: open.span.to(close.span),
                })
            }
            "assert_malformed" | "assert_invalid" | "assert_unlinkable" => {
                let module = self.module()?;
                let message = self.name()?;
                let close = self.expect(Kind::RParen, ")")?;
                let span = open.span.to(close.span);
                Ok(match word {
                    "assert_malformed" => Cmd::AssertMalformed {
                        module,
                        message,
                        span,
                    },
                    "assert_invalid" => Cmd::AssertInvalid {
                        module,
                        message,
                        span,
                    },
                    _ => Cmd::AssertUnlinkable {
                        module,
                        message,
                        span,
                    },
                })
            }
            "script" | "input" | "output" => {
                // Keep the whole command; the runner does not act on it.
                self.i = open_at;
                let span = self.skip_group()?;
                Ok(Cmd::Meta {
                    name: word.to_string(),
                    span,
                })
            }
            other => self.err(Some(head), format!("`{other}` is not a script command")),
        }
    }

    /// True when the next group is a module command.
    fn looks_like_module(&mut self) -> bool {
        self.skip_annotations();
        matches!(self.toks.get(self.i), Some(t) if t.kind == Kind::LParen)
            && matches!(self.toks.get(self.i + 1),
                Some(t) if t.kind == Kind::Word && t.span.text(self.src) == "module")
    }

    /// Reads a `(module …)` where one is expected.
    fn module(&mut self) -> Result<Module, Error> {
        let open = self.expect(Kind::LParen, "a module")?;
        let head = self.expect(Kind::Word, "`module`")?;
        if head.span.text(self.src) != "module" {
            return self.err(Some(head), "expected a module");
        }
        match self.module_cmd(open)? {
            Cmd::Module(m) => Ok(m),
            other => self.err(
                Some(head),
                format!("expected a module, not {}", other.keyword()),
            ),
        }
    }

    /// Reads a module command, with `(` and `module` already consumed.
    fn module_cmd(&mut self, open: Token) -> Result<Cmd, Error> {
        // `(module instance $i $m)` instantiates a defined module.
        if self.eat_word("instance") {
            let instance = self.opt_id();
            let module = self.opt_id();
            let close = self.expect(Kind::RParen, ")")?;
            return Ok(Cmd::Instance {
                instance,
                module,
                span: open.span.to(close.span),
            });
        }
        let definition = self.eat_word("definition");
        let id = self.opt_id();
        let kind = if self.eat_word("binary") {
            let mut bytes = Vec::new();
            while self.peek().is_some_and(|t| t.kind == Kind::Str) {
                bytes.extend_from_slice(&self.bytes()?);
            }
            ModuleKind::Binary(bytes)
        } else if self.eat_word("quote") {
            let mut bytes = Vec::new();
            while self.peek().is_some_and(|t| t.kind == Kind::Str) {
                bytes.extend_from_slice(&self.bytes()?);
            }
            ModuleKind::Quote(bytes)
        } else {
            ModuleKind::Text
        };
        // Whatever the form, the rest of the group belongs to the
        // module, not to the script: skip it without reading it.
        let mut depth = 1usize;
        let close = loop {
            let Some(t) = self.toks.get(self.i) else {
                return Err(Error {
                    offset: open.span.start,
                    message: "this module is never closed".to_string(),
                });
            };
            self.i += 1;
            match t.kind {
                Kind::LParen => depth += 1,
                Kind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        break *t;
                    }
                }
                _ => {}
            }
        };
        Ok(Cmd::Module(Module {
            id,
            kind,
            definition,
            span: open.span.to(close.span),
        }))
    }

    /// Reads a parenthesised action.
    fn action(&mut self) -> Result<Action, Error> {
        let open = self.expect(Kind::LParen, "an action")?;
        let head = self.expect(Kind::Word, "`invoke` or `get`")?;
        let word = head.span.text(self.src);
        if word != "invoke" && word != "get" {
            return self.err(Some(head), format!("`{word}` is not an action"));
        }
        self.action_body(open, word)
    }

    /// Reads an action with `(` and its keyword already consumed.
    fn action_body(&mut self, open: Token, word: &str) -> Result<Action, Error> {
        let module = self.opt_id();
        let name = self.name()?;
        if word == "get" {
            let close = self.expect(Kind::RParen, ")")?;
            return Ok(Action::Get {
                module,
                name,
                span: open.span.to(close.span),
            });
        }
        let mut args = Vec::new();
        while self.peek().is_some_and(|t| t.kind == Kind::LParen) {
            let at = self.peek();
            match self.pattern()? {
                Pattern::Exact(v) => args.push(v),
                _ => return self.err(at, "an argument must be a value, not a pattern"),
            }
        }
        let close = self.expect(Kind::RParen, ")")?;
        Ok(Action::Invoke {
            module,
            name,
            args,
            span: open.span.to(close.span),
        })
    }

    /// Reads one value or result pattern.
    ///
    /// Values and patterns share this one reader so that the two cannot
    /// drift apart: an action's argument is a pattern that turned out to
    /// be exact, checked at the one place that builds an argument.
    fn pattern(&mut self) -> Result<Pattern, Error> {
        self.expect(Kind::LParen, "a value")?;
        let head = self.expect(Kind::Word, "a value's keyword")?;
        let word = head.span.text(self.src);

        if word == "either" {
            let mut alts = Vec::new();
            while self.peek().is_some_and(|t| t.kind == Kind::LParen) {
                alts.push(self.pattern()?);
            }
            self.expect(Kind::RParen, ")")?;
            return Ok(Pattern::Either(alts));
        }

        if let Some(rest) = word.strip_suffix(".const") {
            let p = if rest == "v128" {
                self.v128()?
            } else {
                let ty = NumType::from_name(rest)
                    .ok_or_else(|| self.error_at(head, format!("`{rest}` is not a number type")))?;
                let lit = self.expect(Kind::Word, "a number")?;
                self.number(ty, lit)?
            };
            self.expect(Kind::RParen, ")")?;
            return Ok(p);
        }

        if let Some(kind) = word.strip_prefix("ref.") {
            let p = match kind {
                "null" => match self.peek() {
                    Some(t) if t.kind == Kind::Word => {
                        self.i += 1;
                        Pattern::Exact(Value::RefNull(t.span.text(self.src).to_string()))
                    }
                    // `(ref.null)` with no type is the pattern "a null
                    // reference of some type".
                    _ => Pattern::RefOfKind("null".to_string()),
                },
                "extern" | "host" => match self.peek() {
                    Some(t) if t.kind == Kind::Word => {
                        self.i += 1;
                        let n = value::int(t.span.text(self.src), 64)
                            .map_err(|m| self.error_at(t, m))?;
                        Pattern::Exact(if kind == "extern" {
                            Value::RefExtern(n)
                        } else {
                            Value::RefHost(n)
                        })
                    }
                    _ => Pattern::RefOfKind(kind.to_string()),
                },
                // `(ref.func)`, `(ref.array)`, `(ref.struct)`,
                // `(ref.i31)`, `(ref.eq)`, `(ref.any)`: a reference of
                // that kind, with nothing more said about it.
                other => Pattern::RefOfKind(other.to_string()),
            };
            self.expect(Kind::RParen, ")")?;
            return Ok(p);
        }

        self.err(Some(head), format!("`{word}` is not a value"))
    }

    /// Reads a `v128.const`'s shape and lanes.
    ///
    /// Lanes that are all exact become the sixteen bytes themselves; a
    /// lane that is a NaN pattern keeps the whole thing lane-wise,
    /// because such a result has no single byte string.
    fn v128(&mut self) -> Result<Pattern, Error> {
        let sh = self.expect(Kind::Word, "a lane shape")?;
        let shape = Shape::from_name(sh.span.text(self.src))
            .ok_or_else(|| self.error_at(sh, "this is not a lane shape"))?;
        let ty = match shape {
            Shape::F32x4 => NumType::F32,
            Shape::F64x2 => NumType::F64,
            Shape::I64x2 => NumType::I64,
            _ => NumType::I32,
        };
        let mut lanes = Vec::with_capacity(shape.lanes());
        for _ in 0..shape.lanes() {
            let lit = self.expect(Kind::Word, "a lane")?;
            let text = lit.span.text(self.src);
            lanes.push(if shape.is_float() {
                self.number(ty, lit)?
            } else {
                let bits =
                    value::int(text, shape.lane_bits()).map_err(|m| self.error_at(lit, m))?;
                Pattern::Exact(match shape.lane_bits() {
                    64 => Value::I64(bits),
                    _ => Value::I32(bits as u32),
                })
            });
        }
        if let Some(bytes) = pack(shape, &lanes) {
            return Ok(Pattern::Exact(Value::V128(bytes)));
        }
        Ok(Pattern::V128(shape, lanes))
    }

    /// Reads one number literal, which for a float may be a pattern.
    fn number(&mut self, ty: NumType, lit: Token) -> Result<Pattern, Error> {
        let text = lit.span.text(self.src);
        if ty.is_float() {
            match text {
                "nan:canonical" => return Ok(Pattern::NanCanonical(ty)),
                "nan:arithmetic" => return Ok(Pattern::NanArithmetic(ty)),
                _ => {}
            }
        }
        let v = match ty {
            NumType::I32 => value::int(text, 32).map(|b| Value::I32(b as u32)),
            NumType::I64 => value::int(text, 64).map(Value::I64),
            NumType::F32 => value::float32(text).map(Value::F32),
            NumType::F64 => value::float64(text).map(Value::F64),
        }
        .map_err(|m| self.error_at(lit, m))?;
        Ok(Pattern::Exact(v))
    }

    fn error_at(&self, t: Token, message: impl Into<String>) -> Error {
        Error {
            offset: t.span.start,
            message: message.into(),
        }
    }
}

/// The sixteen bytes of a `v128` whose lanes are all exact, or `None`
/// when one of them is a pattern.
fn pack(shape: Shape, lanes: &[Pattern]) -> Option<[u8; 16]> {
    let mut out = [0u8; 16];
    let width = (shape.lane_bits() / 8) as usize;
    for (i, lane) in lanes.iter().enumerate() {
        let Pattern::Exact(v) = lane else {
            return None;
        };
        let bits: u64 = match v {
            Value::I32(n) | Value::F32(n) => u64::from(*n),
            Value::I64(n) | Value::F64(n) => *n,
            _ => return None,
        };
        out[i * width..(i + 1) * width].copy_from_slice(&bits.to_le_bytes()[..width]);
    }
    Some(out)
}

#[cfg(test)]
mod tests;
