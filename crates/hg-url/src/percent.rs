//! Percent-encoding, URL Standard Section 1.3.
//!
//! A **percent-encode set** is a set of code points; UTF-8
//! percent-encoding a code point in the set writes each of its bytes as
//! `%XX`. The six sets the parser uses nest, each adding to the one
//! before, and the standard notes one place where they do not: "The
//! query percent-encode set cannot be defined in terms of the fragment
//! percent-encode set due to the omission of U+0060 (`)."

/// One of the standard's percent-encode sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Set {
    /// C0 controls and everything above U+007E.
    C0Control,
    /// The C0 control set, plus space, `"`, `<`, `>` and `` ` ``.
    Fragment,
    /// The C0 control set, plus space, `"`, `#`, `<` and `>`.
    Query,
    /// The query set, plus `'`.
    SpecialQuery,
    /// The query set, plus `?`, `^`, `` ` ``, `{` and `}`.
    Path,
    /// The path set, plus `/`, `:`, `;`, `=`, `@`, `[` to `]` and `|`.
    Userinfo,
    /// The userinfo set, plus `$` to `&`, `+` and `,`.
    ///
    /// Not used by the parser: HTML's `registerProtocolHandler()` uses
    /// it, and so may anything embedding data in a URL component.
    Component,
}

impl Set {
    /// Whether `c` is in this set, and so must be percent-encoded.
    #[must_use]
    pub const fn contains(self, c: char) -> bool {
        // The C0 control set is in every other set.
        // A C0 control, or anything past U+007E — the C0 control
        // percent-encode set, which every other set contains. Written
        // as code-point ranges rather than `as u32` because tier P
        // denies silent conversions.
        if matches!(c, '\0'..='\u{1F}' | '\u{7F}'..='\u{10FFFF}') {
            return true;
        }
        match self {
            Self::C0Control => false,
            Self::Fragment => matches!(c, ' ' | '"' | '<' | '>' | '`'),
            Self::Query => matches!(c, ' ' | '"' | '#' | '<' | '>'),
            Self::SpecialQuery => matches!(c, ' ' | '"' | '#' | '<' | '>' | '\''),
            Self::Path => matches!(c, ' ' | '"' | '#' | '<' | '>' | '?' | '^' | '`' | '{' | '}'),
            Self::Userinfo => matches!(
                c,
                ' ' | '"'
                    | '#'
                    | '<'
                    | '>'
                    | '?'
                    | '^'
                    | '`'
                    | '{'
                    | '}'
                    | '/'
                    | ':'
                    | ';'
                    | '='
                    | '@'
                    | '['
                    | '\\'
                    | ']'
                    | '|'
            ),
            Self::Component => matches!(
                c,
                ' ' | '"'
                    | '#'
                    | '<'
                    | '>'
                    | '?'
                    | '^'
                    | '`'
                    | '{'
                    | '}'
                    | '/'
                    | ':'
                    | ';'
                    | '='
                    | '@'
                    | '['
                    | '\\'
                    | ']'
                    | '|'
                    | '$'
                    | '%'
                    | '&'
                    | '+'
                    | ','
            ),
        }
    }
}

/// The hexadecimal digits percent-encoding writes, upper case.
const HEX: [u8; 16] = *b"0123456789ABCDEF";

/// Appends `byte` as `%XX`.
pub fn encode_byte(byte: u8, out: &mut String) {
    out.push('%');
    out.push(char::from(
        HEX.get(usize::from(byte >> 4)).copied().unwrap_or(b'0'),
    ));
    out.push(char::from(
        HEX.get(usize::from(byte & 0x0F)).copied().unwrap_or(b'0'),
    ));
}

/// UTF-8 percent-encodes `c` with `set`, appending to `out`.
pub fn encode(c: char, set: Set, out: &mut String) {
    if !set.contains(c) {
        out.push(c);
        return;
    }
    let mut buf = [0_u8; 4];
    for byte in c.encode_utf8(&mut buf).as_bytes() {
        encode_byte(*byte, out);
    }
}

/// UTF-8 percent-encodes every code point of `s` with `set`.
#[must_use]
pub fn encode_str(s: &str, set: Set) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        encode(c, set, &mut out);
    }
    out
}

/// A hexadecimal digit's value.
const fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b.wrapping_sub(b'0')),
        b'a'..=b'f' => Some(b.wrapping_sub(b'a').wrapping_add(10)),
        b'A'..=b'F' => Some(b.wrapping_sub(b'A').wrapping_add(10)),
        _ => None,
    }
}

/// Whether the two bytes after `at` are ASCII hex digits.
#[must_use]
pub fn two_hex_digits(bytes: &[u8], at: usize) -> bool {
    let a = bytes.get(at).copied().and_then(hex);
    let b = bytes.get(at.saturating_add(1)).copied().and_then(hex);
    a.is_some() && b.is_some()
}

/// Percent-decodes `input` to bytes (Section 1.3).
///
/// A `%` not followed by two hex digits is kept as itself, which is
/// what the standard says: the sequence is simply not an escape.
#[must_use]
pub fn decode(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0_usize;
    while let Some(b) = input.get(i).copied() {
        if b != b'%' {
            out.push(b);
            i = i.saturating_add(1);
            continue;
        }
        let hi = input.get(i.saturating_add(1)).copied().and_then(hex);
        let lo = input.get(i.saturating_add(2)).copied().and_then(hex);
        match (hi, lo) {
            (Some(h), Some(l)) => {
                out.push(h.wrapping_shl(4).wrapping_add(l));
                i = i.saturating_add(3);
            }
            _ => {
                out.push(b);
                i = i.saturating_add(1);
            }
        }
    }
    out
}

/// Whether `s` holds a percent-encoded byte: a `%` with two hex digits
/// after it.
#[must_use]
pub fn has_percent_encoded_byte(s: &str) -> bool {
    let b = s.as_bytes();
    b.iter()
        .enumerate()
        .any(|(i, c)| *c == b'%' && two_hex_digits(b, i.saturating_add(1)))
}

/// Percent-encode after encoding (Section 1.3).
///
/// The query is the one URL component HTML may ask to encode in a
/// legacy encoding, and that is why this cannot be done code point by
/// code point: the standard says so itself, "due to the stateful
/// ISO-2022-JP encoder". The encoder therefore runs over the whole
/// string, and a code point it cannot represent becomes
/// `%26%23` + the decimal scalar value + `%3B` — the percent-encoded
/// form of `&#N;` — at the byte position where it occurred.
#[must_use]
pub fn encode_after_encoding(encoding: hg_encoding::Encoding, input: &str, set: Set) -> String {
    if matches!(encoding, hg_encoding::Encoding::Utf8) {
        return encode_str(input, set);
    }
    let Some(mut encoder) = hg_encoding::Encoder::new(encoding) else {
        return encode_str(input, set);
    };
    let mut bytes = Vec::new();
    // Where each unrepresentable code point fell, so the escape lands
    // between the bytes before it and the bytes after it.
    let mut escapes: Vec<(usize, char)> = Vec::new();
    encoder.encode(input, true, &mut bytes, &mut |c, out| {
        escapes.push((out.len(), c));
    });
    let mut out = String::with_capacity(input.len());
    let mut next = 0_usize;
    for (i, byte) in bytes.iter().enumerate() {
        while escapes.get(next).is_some_and(|(at, _)| *at == i) {
            if let Some((_, c)) = escapes.get(next) {
                out.push_str("%26%23");
                out.push_str(&u32::from(*c).to_string());
                out.push_str("%3B");
            }
            next = next.saturating_add(1);
        }
        // Every byte is a code point below U+0080, and the assertion in
        // the standard is that the set holds every non-ASCII code
        // point — so the byte is tested as itself.
        let isomorph = char::from(*byte);
        if set.contains(isomorph) {
            encode_byte(*byte, &mut out);
        } else {
            out.push(isomorph);
        }
    }
    while let Some((_, c)) = escapes.get(next) {
        out.push_str("%26%23");
        out.push_str(&u32::from(*c).to_string());
        out.push_str("%3B");
        next = next.saturating_add(1);
    }
    out
}
