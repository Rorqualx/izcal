//! The vocabulary the reference stepper's generated table is written in.
//!
//! These types hold no behaviour: `hg-htmlgen` emits a `&'static` value
//! of [`StateDesc`] per state, and [`crate::tokenizer::Tokenizer::step_table`]
//! interprets them. The generated tokenizer compiles the same model into
//! Rust instead, so the two back ends share the description and share no
//! code.

use crate::State;

/// A named class of code points, as the prose names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Tab, line feed, form feed or space.
    Whitespace,
    /// U+0000.
    Null,
    /// `A`–`Z` or `a`–`z`.
    AsciiAlpha,
    /// `A`–`Z`.
    AsciiUpper,
    /// `a`–`z`.
    AsciiLower,
    /// `0`–`9`.
    AsciiDigit,
    /// `0`–`9`, `A`–`F` or `a`–`f`.
    AsciiHexDigit,
    /// `A`–`Z`, `a`–`z` or `0`–`9`.
    AsciiAlphanumeric,
}

impl Class {
    /// Whether `c` is in the class.
    #[must_use]
    pub const fn has(self, c: char) -> bool {
        match self {
            Self::Whitespace => matches!(c, '\t' | '\n' | '\u{000C}' | ' '),
            Self::Null => c == '\0',
            Self::AsciiAlpha => c.is_ascii_alphabetic(),
            Self::AsciiUpper => c.is_ascii_uppercase(),
            Self::AsciiLower => c.is_ascii_lowercase(),
            Self::AsciiDigit => c.is_ascii_digit(),
            Self::AsciiHexDigit => c.is_ascii_hexdigit(),
            Self::AsciiAlphanumeric => c.is_ascii_alphanumeric(),
        }
    }
}

/// A buffer the tokenizer appends to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Buf {
    /// The current tag token's name.
    TagName,
    /// The current attribute's name.
    AttrName,
    /// The current attribute's value.
    AttrValue,
    /// The current comment token's data.
    Comment,
    /// The current DOCTYPE token's name.
    DoctypeName,
    /// Its public identifier.
    PublicId,
    /// Its system identifier.
    SystemId,
    /// The temporary buffer.
    Temp,
}

/// A flag a rule may set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// The current tag's self-closing flag.
    SelfClosing,
    /// The current DOCTYPE's force-quirks flag.
    ForceQuirks,
    /// Its public identifier, set to the empty string.
    PublicIdEmpty,
    /// Its system identifier, set to the empty string.
    SystemIdEmpty,
}

/// A condition a rule may branch on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    /// "If the current end tag token is an appropriate end tag token".
    Appropriate,
    /// "If the temporary buffer is the string `script`".
    TempIsScript,
    /// "If the character reference was consumed as part of an attribute".
    InAttribute,
    /// "If the adjusted current node is not an element in the HTML
    /// namespace".
    AdjustedNotHtml,
}

/// What a rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pat {
    /// One code point.
    Char(char),
    /// A named class.
    Class(Class),
    /// End of file.
    Eof,
    /// Any of several, as the prose groups them.
    OneOf(&'static [Pat]),
    /// A string tested against the input. `from_current` says whether it
    /// starts at the character the state has consumed.
    Lookahead {
        /// The characters to compare.
        text: &'static str,
        /// Whether it starts at the current input character.
        from_current: bool,
        /// Whether the comparison is ASCII case-insensitive.
        ci: bool,
    },
    /// "Anything else".
    Anything,
}

/// One action, in the prose's own vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// "Switch to the X state".
    Switch(State),
    /// "Reconsume in the X state".
    Reconsume(State),
    /// "Set the return state to X".
    SetReturn(State),
    /// "Switch to the return state".
    SwitchReturn,
    /// "Reconsume in the return state".
    ReconsumeReturn,
    /// "Emit the current input character as a character token".
    EmitCurrent,
    /// "Emit a U+XXXX character token".
    EmitChar(char),
    /// "Emit an end-of-file token".
    EmitEof,
    /// "Emit the current tag, comment or DOCTYPE token".
    EmitToken,
    /// "Emit the temporary buffer as character tokens".
    EmitTemp,
    /// "Create a new start tag token".
    CreateStartTag,
    /// "Create a new end tag token".
    CreateEndTag,
    /// "Create a comment token".
    CreateComment,
    /// "Create a new DOCTYPE token".
    CreateDoctype,
    /// "Start a new attribute".
    StartAttribute,
    /// "Append the current input character to X".
    Append(Buf),
    /// "Append the lowercase version of the current input character".
    AppendLower(Buf),
    /// "Append U+XXXX to X".
    AppendChar(Buf, char),
    /// "… whose data is the `[CDATA[` string".
    AppendStr(Buf, &'static str),
    /// "Set the temporary buffer to the empty string".
    ClearTemp,
    /// "Flush code points consumed as a character reference".
    FlushTemp,
    /// Set a flag.
    Set(Flag),
    /// "Ignore the character".
    Ignore,
    /// "Set the character reference code to zero".
    CharRefReset,
    /// Multiply the character reference code by 16 and add the digit.
    CharRefHexDigit,
    /// The same, by 10.
    CharRefDecDigit,
    /// The duplicate-attribute comparison, on leaving the attribute name
    /// state.
    FinishAttrName,
    /// "This is a *named* parse error".
    Error(&'static str),
    /// A branch the prose writes as "if … otherwise".
    If {
        /// What is tested.
        cond: Cond,
        /// Taken when it holds.
        then: &'static [Op],
        /// Taken when it does not.
        otherwise: &'static [Op],
    },
}

/// One rule of a state.
#[derive(Debug, Clone, Copy)]
pub struct Rule {
    /// What it matches.
    pub pat: Pat,
    /// What it does.
    pub ops: &'static [Op],
    /// The clause it came from, when the rule carries its own.
    pub anchor: Option<&'static str>,
}

/// One state.
#[derive(Debug, Clone, Copy)]
pub struct StateDesc {
    /// Its name, as the description writes it.
    pub name: &'static str,
    /// The clause that defines it.
    pub anchor: &'static str,
    /// Whether the runtime implements it instead.
    pub external: bool,
    /// Whether the tree builder may switch the tokenizer into it.
    pub entry: bool,
    /// Whether it consumes nothing, asking about the next few
    /// characters instead.
    pub consumes: bool,
    /// What it does on entry, before consuming.
    pub enter: &'static [Op],
    /// Its rules, in the prose's order.
    pub rules: &'static [Rule],
}
