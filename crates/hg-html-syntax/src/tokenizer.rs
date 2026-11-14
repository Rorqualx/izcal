//! The tokenizer's runtime: everything the two generated back ends call,
//! and the interpreter the reference stepper's table is run by.
//!
//! Nothing here decides a transition. The states live in `states/*.hgtok`
//! and the generator turns them into code and into data; this module
//! holds the buffers, the effects, and the two states the description
//! declares `extern` because the prose does not write them as a table.

use crate::State;
use crate::generated::entities;
use crate::generated::states::{EntryState, Extern};
use crate::table::{Buf, Cond, Flag, Op, Pat, StateDesc};
use crate::token::{Attribute, Sink, Token};

/// Which generated back end runs the states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Back {
    /// The tokenizer: generated Rust, one function per state.
    Generated,
    /// The reference stepper: the generated table, interpreted here.
    Table,
}

/// Which token is being built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Current {
    None,
    Tag,
    Comment,
    Doctype,
    Pi,
}

/// The HTML tokenizer.
pub struct Tokenizer<S: Sink> {
    sink: S,
    back: Back,
    input: Vec<char>,
    at: usize,
    /// What the last [`Self::consume`] returned: the current input
    /// character, or `None` at the end of the file.
    last: Option<char>,
    pending_reconsume: bool,
    done: bool,
    state: State,
    return_state: State,

    temp: String,
    current: Current,
    tag_name: String,
    tag_is_end: bool,
    tag_attrs: Vec<Attribute>,
    attr_dropped: bool,
    self_closing: bool,
    comment: String,
    pi_target: String,
    pi_data: String,
    dt_name: Option<String>,
    dt_public: Option<String>,
    dt_system: Option<String>,
    dt_quirks: bool,
    char_ref: u32,
    last_start_tag: String,

    line: u32,
    column: u32,
    started: bool,
    /// Whether this step has consumed anything, and whether the state
    /// has asked for its errors to be reported one further on. See
    /// [`Self::error`].
    consumed_this_step: bool,
    error_after_match: bool,
}

impl<S: Sink> Tokenizer<S> {
    /// A tokenizer over `input`, running `back`, starting in the data
    /// state.
    ///
    /// The input stream's newline normalization (§13.2.3.5) is applied
    /// here: CRLF and a lone CR each become one LF. The rest of the
    /// input stream — decoding, `document.write`'s insertion point, the
    /// surrogate and control parse errors — belongs to `hg-html-driver`.
    #[must_use]
    pub fn new(sink: S, input: &str, back: Back) -> Self {
        let mut chars = Vec::with_capacity(input.len());
        let mut after_cr = false;
        for c in input.chars() {
            match c {
                '\r' => {
                    chars.push('\n');
                    after_cr = true;
                }
                '\n' if after_cr => after_cr = false,
                _ => {
                    chars.push(c);
                    after_cr = false;
                }
            }
        }
        Self {
            sink,
            back,
            input: chars,
            at: 0,
            last: None,
            pending_reconsume: false,
            done: false,
            state: State::Data,
            return_state: State::Data,
            temp: String::new(),
            current: Current::None,
            tag_name: String::new(),
            tag_is_end: false,
            tag_attrs: Vec::new(),
            attr_dropped: false,
            self_closing: false,
            comment: String::new(),
            pi_target: String::new(),
            pi_data: String::new(),
            dt_name: None,
            dt_public: None,
            dt_system: None,
            dt_quirks: false,
            char_ref: 0,
            last_start_tag: String::new(),
            line: 1,
            column: 0,
            started: false,
            consumed_this_step: false,
            error_after_match: false,
        }
    }

    /// A tokenizer started in any state, with the last start tag
    /// already set — what html5lib-tests' `initialStates` and
    /// `lastStartTag` describe.
    ///
    /// Behind a feature because a shipping build has no use for it: the
    /// tree builder switches to the five [`EntryState`]s and no others,
    /// and nothing but a conformance harness can set the last start tag
    /// without emitting one.
    #[cfg(feature = "conformance")]
    #[must_use]
    pub fn for_conformance(
        sink: S,
        input: &str,
        back: Back,
        state: State,
        last_start_tag: &str,
    ) -> Self {
        let mut t = Self::new(sink, input, back);
        t.state = state;
        t.last_start_tag = last_start_tag.to_owned();
        t
    }

    /// Switches the tokenizer into one of the five states the tree
    /// construction stage may switch it to, and no others — "switch the
    /// tokenizer to the RAWTEXT state" and its four siblings.
    pub fn switch_to(&mut self, state: EntryState) {
        self.state = state.state();
    }

    /// The state it is in.
    #[must_use]
    pub const fn state(&self) -> State {
        self.state
    }

    /// Gives the sink back.
    #[must_use]
    pub fn into_sink(self) -> S {
        self.sink
    }

    /// The sink, while the tokenizer still owns it — what a driver
    /// reads between steps to decide whether to switch states.
    #[must_use]
    pub const fn sink(&self) -> &S {
        &self.sink
    }

    /// Whether the end-of-file token has been emitted.
    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.done
    }

    /// Runs until the end-of-file token is emitted.
    pub fn run(&mut self) {
        while !self.done {
            self.step();
        }
    }

    /// One state, by whichever back end this tokenizer runs.
    pub fn step(&mut self) {
        self.consumed_this_step = false;
        self.error_after_match = false;
        match self.back {
            Back::Generated => self.generated_step(),
            Back::Table => self.step_table(),
        }
    }

    // -- the input ----------------------------------------------------

    /// Consumes the next input character, or reports the end of the
    /// file. A pending "reconsume" returns the same character again.
    pub(crate) fn consume(&mut self) -> Option<char> {
        self.consumed_this_step = true;
        if self.pending_reconsume {
            self.pending_reconsume = false;
            return self.last;
        }
        let c = self.input.get(self.at).copied();
        if c.is_some() {
            self.at = self.at.saturating_add(1);
        }
        // The position is the one of the character just consumed, so a
        // parse error names where it happened.
        // Columns count UTF-16 code units, not code points: that is
        // what every reported parse-error position in the wild means,
        // html5lib-tests' among them, and an astral character is two.
        if self.started {
            if self.last == Some('\n') {
                self.line = self.line.saturating_add(1);
                self.column = 1;
            } else {
                let width = self
                    .last
                    .map_or(1, |c| u32::try_from(c.len_utf16()).unwrap_or(1));
                self.column = self.column.saturating_add(width);
            }
        } else {
            self.started = true;
            self.column = 1;
        }
        self.last = c;
        c
    }

    /// Reports this state's parse errors at the character after the
    /// run it consumed, which the named character reference state
    /// wants: it consumed a whole identifier, not a character.
    pub(crate) const fn report_after_match(&mut self) {
        self.error_after_match = true;
    }

    /// The `n`-th character from the current position, counting the one
    /// a pending reconsume would hand back as the zeroth.
    fn peek(&self, n: usize) -> Option<char> {
        if self.pending_reconsume {
            if n == 0 {
                return self.last;
            }
            let k = n.checked_sub(1)?;
            return self.input.get(self.at.checked_add(k)?).copied();
        }
        self.input.get(self.at.checked_add(n)?).copied()
    }

    fn same(a: char, b: char, ci: bool) -> bool {
        if ci {
            a.eq_ignore_ascii_case(&b)
        } else {
            a == b
        }
    }

    /// "If the next few characters are …": nothing has been consumed,
    /// and the whole string is consumed when it matches.
    pub(crate) fn match_next(&mut self, text: &str, ci: bool) -> bool {
        let mut n: usize = 0;
        for want in text.chars() {
            match self.peek(n) {
                Some(got) if Self::same(got, want, ci) => n = n.saturating_add(1),
                _ => return false,
            }
        }
        for _ in 0..n {
            self.consume();
        }
        true
    }

    /// "If the … characters starting from the current input character
    /// are …": the first has been consumed, the rest are consumed when
    /// the whole string matches.
    pub(crate) fn match_from(&mut self, text: &str, ci: bool) -> bool {
        let mut want = text.chars();
        let (Some(first), Some(got)) = (want.next(), self.last) else {
            return false;
        };
        if !Self::same(got, first, ci) {
            return false;
        }
        let mut n: usize = 0;
        for w in want {
            match self.peek(n) {
                Some(g) if Self::same(g, w, ci) => n = n.saturating_add(1),
                _ => return false,
            }
        }
        for _ in 0..n {
            self.consume();
        }
        true
    }

    // -- transitions --------------------------------------------------

    pub(crate) const fn switch(&mut self, s: State) {
        self.state = s;
    }

    pub(crate) const fn reconsume_in(&mut self, s: State) {
        self.state = s;
        self.pending_reconsume = true;
    }

    pub(crate) const fn set_return(&mut self, s: State) {
        self.return_state = s;
    }

    pub(crate) const fn switch_return(&mut self) {
        self.state = self.return_state;
    }

    pub(crate) const fn reconsume_return(&mut self) {
        self.state = self.return_state;
        self.pending_reconsume = true;
    }

    // -- effects ------------------------------------------------------

    /// Reports a parse error where it happened.
    ///
    /// A state that consumed a character reports *that* character's
    /// position. A state with no current input character of its own
    /// reports the position of the next one — the markup declaration
    /// open state, which consumes nothing, and the two hand-written
    /// states, which are not per-character states at all.
    pub(crate) fn error(&mut self, code: &'static str) {
        // A state that consumed a character reports *that* character's
        // position. A state that consumed nothing reports the next
        // character's — the markup declaration open state, whose own
        // "anything else" says "don't consume anything", and the
        // numeric character reference end state, which checks a number.
        // Two things leave the position where it is: the end of the
        // file, which has no next character, and a pending reconsume,
        // where the next character is the one already there.
        let next = (self.error_after_match || !self.consumed_this_step)
            && self.last.is_some()
            && !self.pending_reconsume;
        let column = self.column.saturating_add(u32::from(next));
        self.sink.error(code, self.line, column);
    }

    fn emit(&mut self, t: Token) {
        self.sink.token(t);
    }

    pub(crate) fn emit_current(&mut self) {
        if let Some(c) = self.last {
            self.emit(Token::Character(c));
        }
    }

    pub(crate) fn emit_char(&mut self, c: char) {
        self.emit(Token::Character(c));
    }

    pub(crate) fn emit_eof(&mut self) {
        self.done = true;
        self.emit(Token::Eof);
    }

    pub(crate) fn emit_temp(&mut self) {
        let t = core::mem::take(&mut self.temp);
        for c in t.chars() {
            self.emit(Token::Character(c));
        }
        self.temp = t;
    }

    pub(crate) fn emit_token(&mut self) {
        let token = match self.current {
            Current::None => return,
            Current::Tag => {
                let name = core::mem::take(&mut self.tag_name);
                let attrs = core::mem::take(&mut self.tag_attrs);
                if self.tag_is_end {
                    Token::EndTag {
                        name,
                        attrs,
                        self_closing: self.self_closing,
                    }
                } else {
                    self.last_start_tag.clone_from(&name);
                    Token::StartTag {
                        name,
                        attrs,
                        self_closing: self.self_closing,
                    }
                }
            }
            Current::Comment => Token::Comment(core::mem::take(&mut self.comment)),
            Current::Pi => Token::ProcessingInstruction {
                target: core::mem::take(&mut self.pi_target),
                data: core::mem::take(&mut self.pi_data),
            },
            Current::Doctype => Token::Doctype {
                name: self.dt_name.take(),
                public_id: self.dt_public.take(),
                system_id: self.dt_system.take(),
                force_quirks: self.dt_quirks,
            },
        };
        self.current = Current::None;
        self.emit(token);
    }

    pub(crate) fn create_tag(&mut self, end: bool) {
        self.current = Current::Tag;
        self.tag_name.clear();
        self.tag_attrs.clear();
        self.tag_is_end = end;
        self.self_closing = false;
        self.attr_dropped = false;
    }

    pub(crate) fn create_comment(&mut self) {
        self.current = Current::Comment;
        self.comment.clear();
    }

    /// "Create a processing instruction token whose target is target
    /// and data is the empty string", target being the temporary
    /// buffer's code points in the order they were added.
    pub(crate) fn create_pi(&mut self) {
        self.current = Current::Pi;
        self.pi_target.clone_from(&self.temp);
        self.pi_data.clear();
    }

    /// "To convert the temporary buffer to a comment, create a comment
    /// token whose data is the concatenation of `?` and the code points
    /// in the temporary buffer" — what a processing instruction with a
    /// target the standard disallows becomes.
    pub(crate) fn temp_to_comment(&mut self) {
        self.current = Current::Comment;
        self.comment.clear();
        self.comment.push('?');
        let t = core::mem::take(&mut self.temp);
        self.comment.push_str(&t);
        self.temp = t;
    }

    pub(crate) fn create_doctype(&mut self) {
        self.current = Current::Doctype;
        self.dt_name = None;
        self.dt_public = None;
        self.dt_system = None;
        self.dt_quirks = false;
    }

    pub(crate) fn start_attribute(&mut self) {
        self.attr_dropped = false;
        self.tag_attrs.push(Attribute::default());
    }

    /// The comparison the prose hangs on *leaving* the attribute name
    /// state: a duplicate is a parse error and the new attribute is
    /// removed, so everything appended to its value afterwards goes
    /// nowhere.
    pub(crate) fn finish_attr_name(&mut self) {
        let Some(last) = self.tag_attrs.last() else {
            return;
        };
        let name = last.name.clone();
        let earlier = self.tag_attrs.iter().rev().skip(1).any(|a| a.name == name);
        if earlier {
            self.error("duplicate-attribute");
            self.tag_attrs.pop();
            self.attr_dropped = true;
        }
    }

    pub(crate) fn push(&mut self, b: Buf, c: char) {
        match b {
            Buf::TagName => self.tag_name.push(c),
            Buf::AttrName => {
                if let Some(a) = self.tag_attrs.last_mut() {
                    a.name.push(c);
                }
            }
            Buf::AttrValue => {
                if !self.attr_dropped
                    && let Some(a) = self.tag_attrs.last_mut()
                {
                    a.value.push(c);
                }
            }
            Buf::Comment => self.comment.push(c),
            Buf::DoctypeName => self.dt_name.get_or_insert_default().push(c),
            Buf::PublicId => self.dt_public.get_or_insert_default().push(c),
            Buf::SystemId => self.dt_system.get_or_insert_default().push(c),
            Buf::Temp => self.temp.push(c),
            Buf::PiData => self.pi_data.push(c),
        }
    }

    pub(crate) fn push_current(&mut self, b: Buf) {
        if let Some(c) = self.last {
            self.push(b, c);
        }
    }

    pub(crate) fn push_current_lower(&mut self, b: Buf) {
        if let Some(c) = self.last {
            self.push(b, c.to_ascii_lowercase());
        }
    }

    pub(crate) fn push_str(&mut self, b: Buf, s: &str) {
        for c in s.chars() {
            self.push(b, c);
        }
    }

    pub(crate) fn clear_temp(&mut self) {
        self.temp.clear();
    }

    /// "Flush code points consumed as a character reference": into the
    /// attribute's value when that is what consumed them, otherwise as
    /// character tokens.
    pub(crate) fn flush_temp(&mut self) {
        let t = core::mem::take(&mut self.temp);
        if self.char_ref_in_attribute() {
            for c in t.chars() {
                self.push(Buf::AttrValue, c);
            }
        } else {
            for c in t.chars() {
                self.emit(Token::Character(c));
            }
        }
        self.temp = t;
    }

    pub(crate) fn set_flag(&mut self, f: Flag) {
        match f {
            Flag::SelfClosing => self.self_closing = true,
            Flag::ForceQuirks => self.dt_quirks = true,
            Flag::PublicIdEmpty => self.dt_public = Some(String::new()),
            Flag::SystemIdEmpty => self.dt_system = Some(String::new()),
        }
    }

    pub(crate) const fn char_ref_reset(&mut self) {
        self.char_ref = 0;
    }

    /// "Multiply the character reference code by `base`. Add a numeric
    /// version of the current input character to it." Saturating is
    /// faithful: the end state's only question above 0x10FFFF is
    /// whether it is above 0x10FFFF.
    pub(crate) fn char_ref_digit(&mut self, base: u32) {
        let d = self.last.and_then(|c| c.to_digit(base)).unwrap_or(0);
        self.char_ref = self.char_ref.saturating_mul(base).saturating_add(d);
    }

    // -- conditions ---------------------------------------------------

    const fn char_ref_in_attribute(&self) -> bool {
        matches!(
            self.return_state,
            State::AttributeValueDoubleQuoted
                | State::AttributeValueSingleQuoted
                | State::AttributeValueUnquoted
        )
    }

    pub(crate) fn test(&mut self, c: Cond) -> bool {
        match c {
            // "An appropriate end tag token is one whose tag name
            // matches the tag name of the last start tag to have been
            // emitted from this tokenizer."
            Cond::Appropriate => self.tag_is_end && self.tag_name == self.last_start_tag,
            Cond::TempIsScript => self.temp == "script",
            Cond::DisallowedPiTarget => {
                self.temp.eq_ignore_ascii_case("xml")
                    || self.temp.eq_ignore_ascii_case("xml-stylesheet")
            }
            Cond::InAttribute => self.char_ref_in_attribute(),
            Cond::AdjustedNotHtml => self.sink.adjusted_node_not_html(),
        }
    }

    // -- the two states the description declares extern ---------------

    /// The hand-written states. [`Extern`] is generated, so adding one
    /// to a description fails to compile here until it is written.
    pub(crate) fn external_state(&mut self) {
        match self.state.external() {
            Some(Extern::NamedCharacterReference) => self.named_character_reference(),
            Some(Extern::NumericCharacterReferenceEnd) => self.numeric_character_reference_end(),
            None => {}
        }
    }

    /// "Consume the maximum number of characters possible, where the
    /// consumed characters are one of the identifiers in the first
    /// column of the named character references table."
    ///
    /// A longest match, not a transition, which is why the description
    /// declares this state `extern`. The scan is linear in the longest
    /// identifier and binary over the 2,231 entries; the structure that
    /// does it in one pass is ST-M1's.
    fn named_character_reference(&mut self) {
        let mut name = String::new();
        let mut best: Option<(usize, &'static str)> = None;
        let mut n: usize = 0;
        while n < entities::LONGEST_NAME {
            let Some(c) = self.peek(n) else { break };
            name.push(c);
            n = n.saturating_add(1);
            if let Some(rep) = entities::lookup(&name) {
                best = Some((n, rep));
            }
        }

        let Some((len, replacement)) = best else {
            // Nothing matched, so nothing is consumed: the temporary
            // buffer still holds the ampersand alone.
            self.flush_temp();
            self.switch(State::AmbiguousAmpersand);
            return;
        };

        // It consumed an identifier, not a character, so its errors
        // are reported at the character after it.
        self.report_after_match();
        let mut semicolon = false;
        for _ in 0..len {
            if let Some(c) = self.consume() {
                self.push(Buf::Temp, c);
                semicolon = c == ';';
            }
        }

        // "If the character reference was consumed as part of an
        // attribute, and the last character matched is not a semicolon,
        // and the next input character is either an equals sign or an
        // ASCII alphanumeric, then, for historical reasons, flush …"
        if self.char_ref_in_attribute()
            && !semicolon
            && self
                .peek(0)
                .is_some_and(|c| c == '=' || c.is_ascii_alphanumeric())
        {
            self.flush_temp();
            self.switch_return();
            return;
        }
        if !semicolon {
            self.error("missing-semicolon-after-character-reference");
        }
        self.temp.clear();
        self.push_str(Buf::Temp, replacement);
        self.flush_temp();
        self.switch_return();
    }

    /// "Check the character reference code" — a table of ranges over a
    /// number, consuming nothing, which is why this state is `extern`
    /// too.
    fn numeric_character_reference_end(&mut self) {
        let mut code = self.char_ref;
        if code == 0 {
            self.error("null-character-reference");
            code = 0xFFFD;
        }
        if code > 0x0010_FFFF {
            self.error("character-reference-outside-unicode-range");
            code = 0xFFFD;
        }
        if (0xD800..=0xDFFF).contains(&code) {
            self.error("surrogate-character-reference");
            code = 0xFFFD;
        }
        if is_noncharacter(code) {
            self.error("noncharacter-character-reference");
        }
        if code == 0x0D || (is_control(code) && !is_ascii_whitespace(code)) {
            self.error("control-character-reference");
            if let Some(r) = c1_replacement(code) {
                code = r;
            }
        }
        self.temp.clear();
        if let Some(c) = char::from_u32(code) {
            self.push(Buf::Temp, c);
        }
        self.flush_temp();
        self.switch_return();
    }

    // -- the reference stepper ----------------------------------------

    /// One state, by interpreting the generated table.
    ///
    /// The prose's own order: a state that asks about the next few
    /// characters consumes nothing and tests those first; every other
    /// state consumes, then takes the first rule that matches.
    pub fn step_table(&mut self) {
        let d: &'static StateDesc = self.state.desc();
        if d.external {
            self.external_state();
            return;
        }
        self.run_ops(d.enter);
        if !d.consumes {
            for r in d.rules {
                let fired = match r.pat {
                    Pat::Lookahead { text, ci, .. } => self.match_next(text, ci),
                    Pat::Anything => true,
                    Pat::Char(_) | Pat::Class(_) | Pat::Eof | Pat::OneOf(_) => true,
                };
                if fired {
                    self.run_ops(r.ops);
                    return;
                }
            }
            return;
        }
        let c = self.consume();
        for r in d.rules {
            if self.pat_matches(r.pat, c) {
                self.run_ops(r.ops);
                return;
            }
        }
    }

    fn pat_matches(&mut self, p: Pat, c: Option<char>) -> bool {
        match p {
            Pat::Char(x) => c == Some(x),
            Pat::Class(k) => c.is_some_and(|c| k.has(c)),
            Pat::Eof => c.is_none(),
            Pat::OneOf(ps) => {
                for q in ps {
                    if self.pat_matches(*q, c) {
                        return true;
                    }
                }
                false
            }
            Pat::Lookahead {
                text,
                from_current,
                ci,
            } => from_current && self.match_from(text, ci),
            Pat::Anything => true,
        }
    }

    fn run_ops(&mut self, ops: &'static [Op]) {
        for op in ops {
            match *op {
                Op::Switch(s) => self.switch(s),
                Op::Reconsume(s) => self.reconsume_in(s),
                Op::SetReturn(s) => self.set_return(s),
                Op::SwitchReturn => self.switch_return(),
                Op::ReconsumeReturn => self.reconsume_return(),
                Op::EmitCurrent => self.emit_current(),
                Op::EmitChar(c) => self.emit_char(c),
                Op::EmitEof => self.emit_eof(),
                Op::EmitToken => self.emit_token(),
                Op::EmitTemp => self.emit_temp(),
                Op::CreateStartTag => self.create_tag(false),
                Op::CreateEndTag => self.create_tag(true),
                Op::CreateComment => self.create_comment(),
                Op::CreateDoctype => self.create_doctype(),
                Op::CreatePi => self.create_pi(),
                Op::TempToComment => self.temp_to_comment(),
                Op::StartAttribute => self.start_attribute(),
                Op::Append(b) => self.push_current(b),
                Op::AppendLower(b) => self.push_current_lower(b),
                Op::AppendChar(b, c) => self.push(b, c),
                Op::AppendStr(b, s) => self.push_str(b, s),
                Op::ClearTemp => self.clear_temp(),
                Op::FlushTemp => self.flush_temp(),
                Op::Set(f) => self.set_flag(f),
                Op::Ignore => {}
                Op::CharRefReset => self.char_ref_reset(),
                Op::CharRefHexDigit => self.char_ref_digit(16),
                Op::CharRefDecDigit => self.char_ref_digit(10),
                Op::FinishAttrName => self.finish_attr_name(),
                Op::Error(code) => self.error(code),
                Op::If {
                    cond,
                    then,
                    otherwise,
                } => {
                    if self.test(cond) {
                        self.run_ops(then);
                    } else {
                        self.run_ops(otherwise);
                    }
                }
            }
        }
    }
}

const fn is_control(code: u32) -> bool {
    code <= 0x1F || (code >= 0x7F && code <= 0x9F)
}

const fn is_ascii_whitespace(code: u32) -> bool {
    matches!(code, 0x09 | 0x0A | 0x0C | 0x0D | 0x20)
}

/// The Unicode noncharacters: U+FDD0 to U+FDEF, and the last two code
/// points of every plane.
const fn is_noncharacter(code: u32) -> bool {
    if code >= 0xFDD0 && code <= 0xFDEF {
        return true;
    }
    matches!(code & 0xFFFF, 0xFFFE | 0xFFFF) && code <= 0x0010_FFFF
}

/// The numeric character reference end state's own table, for the 27
/// code points in the C1 range the standard remaps.
const fn c1_replacement(code: u32) -> Option<u32> {
    Some(match code {
        0x80 => 0x20AC,
        0x82 => 0x201A,
        0x83 => 0x0192,
        0x84 => 0x201E,
        0x85 => 0x2026,
        0x86 => 0x2020,
        0x87 => 0x2021,
        0x88 => 0x02C6,
        0x89 => 0x2030,
        0x8A => 0x0160,
        0x8B => 0x2039,
        0x8C => 0x0152,
        0x8E => 0x017D,
        0x91 => 0x2018,
        0x92 => 0x2019,
        0x93 => 0x201C,
        0x94 => 0x201D,
        0x95 => 0x2022,
        0x96 => 0x2013,
        0x97 => 0x2014,
        0x98 => 0x02DC,
        0x99 => 0x2122,
        0x9A => 0x0161,
        0x9B => 0x203A,
        0x9C => 0x0153,
        0x9E => 0x017E,
        0x9F => 0x0178,
        _ => return None,
    })
}
