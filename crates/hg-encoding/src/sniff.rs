//! The sniffing helpers: deciding an encoding from the bytes
//! themselves when nothing else says.
//!
//! These are the HTML Standard's, not the Encoding Standard's, but
//! they live here because they are the other half of what the crate is
//! for and because they need the label table and nothing else (style
//! spec 6.15: `hg-encoding` is "decoders and encoders, generated
//! tables, **sniffing helpers**"). `hg-html-driver` will call
//! [`prescan`] from HTML's "determine the character encoding"; nothing
//! here parses HTML, and nothing here needs to.
//!
//! There is **no statistical autodetection** (ST-D16): the Encoding
//! Standard calls it "generally discouraged", and a detector that
//! guesses from byte frequencies can be steered by an attacker who
//! controls part of a page.
//!
//! Three algorithms:
//!
//! - [`prescan`] — "prescan a byte stream to determine its encoding",
//!   which scans the first bytes of a document for a `<meta>` that
//!   names one.
//! - [`extract_from_meta`] — "extracting a character encoding from a
//!   `meta` element", which reads a `content` attribute's value. It is
//!   also what a `Content-Type` header's `charset` goes through.
//! - [`xml_encoding`] — "get an XML encoding", which the prescan falls
//!   back to. The HTML Standard keeps it because "looking for syntax
//!   resembling an XML declaration, even in `text/html`, is necessary
//!   for compatibility with existing content".

use crate::Encoding;

/// How many bytes HTML's "determine the character encoding" prescans.
pub const HTML_LIMIT: usize = 1024;

/// ASCII whitespace, as HTML defines it.
const fn is_space(b: u8) -> bool {
    matches!(b, 0x09 | 0x0A | 0x0C | 0x0D | 0x20)
}

/// "Prescan a byte stream to determine its encoding", with `limit` as
/// the end condition.
///
/// HTML passes [`HTML_LIMIT`]; a caller that has the whole document and
/// wants the whole of it scanned passes its length.
///
/// The result is already adjusted the way the algorithm says: UTF-16BE
/// and UTF-16LE become UTF-8, and `x-user-defined` becomes
/// windows-1252. Both are there to stop a page choosing an encoding
/// that would make the bytes the parser already scanned mean something
/// else.
#[must_use]
pub fn prescan(bytes: &[u8], limit: usize) -> Option<Encoding> {
    let end = limit.min(bytes.len());
    let input = bytes.get(..end).unwrap_or(&[]);

    // "Prescan for UTF-16 XML declarations". The prefix is two bytes
    // longer than XML's own and the encoding name is not checked,
    // "for historical reasons".
    if input.starts_with(&[0x3C, 0x00, 0x3F, 0x00, 0x78, 0x00]) {
        return Some(Encoding::Utf16le);
    }
    if input.starts_with(&[0x00, 0x3C, 0x00, 0x3F, 0x00, 0x78]) {
        return Some(Encoding::Utf16be);
    }

    let mut at = 0_usize;
    while at < input.len() {
        let rest = input.get(at..).unwrap_or(&[]);
        if rest.starts_with(b"<!--") {
            // To the first `>` preceded by two `-`, which may be the
            // two in `<!--` itself.
            match find_comment_end(input, at) {
                Some(next) => at = next,
                None => break,
            }
            continue;
        }
        if starts_with_meta(rest) {
            match meta(input, at.saturating_add(5)) {
                Meta::Found(e) => return Some(e),
                Meta::Ran(next) => at = next,
                Meta::Out => break,
            }
            continue;
        }
        if is_tag_start(rest) {
            let mut p = at.saturating_add(1);
            while matches!(input.get(p), Some(b) if !is_space(*b) && *b != 0x3E) {
                p = p.saturating_add(1);
            }
            // Attributes are read and discarded: only `<meta>`'s
            // matter, but the algorithm still has to walk past them so
            // a `>` inside a quoted value does not end the tag.
            loop {
                match get_attribute(input, p) {
                    Attr::Got(_, _, next) => p = next,
                    // Running out inside a tag's attributes aborts the
                    // prescan too, not just the tag.
                    Attr::None => break,
                    Attr::Out => return xml_encoding(input),
                }
            }
            at = p;
            continue;
        }
        if rest.starts_with(b"<!") || rest.starts_with(b"</") || rest.starts_with(b"<?") {
            match input
                .get(at..)
                .and_then(|r| r.iter().position(|b| *b == 0x3E))
            {
                Some(i) => at = at.saturating_add(i),
                None => break,
            }
        }
        at = at.saturating_add(1);
    }
    // Running out of bytes, or reaching the end condition, is not a
    // failure: the algorithm falls back to the XML declaration.
    xml_encoding(input)
}

/// What one `<meta>` produced.
enum Meta {
    /// An encoding, which ends the prescan.
    Found(Encoding),
    /// Nothing; carry on from this position.
    Ran(usize),
    /// The bytes ran out inside the element, which aborts the whole
    /// prescan rather than this element alone.
    Out,
}

/// What "get an attribute" produced.
enum Attr {
    /// A name, a value, and where to carry on.
    Got(Vec<u8>, Vec<u8>, usize),
    /// A `>`: there is no attribute, and the element ends here.
    None,
    /// The bytes ran out part-way through.
    Out,
}

/// Whether `rest` starts `<meta` followed by a space or a slash,
/// case-insensitively.
fn starts_with_meta(rest: &[u8]) -> bool {
    let Some(head) = rest.get(..6) else {
        return false;
    };
    let Some((lt, tail)) = head.split_first() else {
        return false;
    };
    if *lt != 0x3C {
        return false;
    }
    let Some((last, name)) = tail.split_last() else {
        return false;
    };
    name.eq_ignore_ascii_case(b"meta") && (is_space(*last) || *last == 0x2F)
}

/// Whether `rest` starts a tag: `<`, optionally `/`, then a letter.
fn is_tag_start(rest: &[u8]) -> bool {
    if rest.first() != Some(&0x3C) {
        return false;
    }
    let after = if rest.get(1) == Some(&0x2F) { 2 } else { 1 };
    rest.get(after).is_some_and(u8::is_ascii_alphabetic)
}

/// The position just past the `-->` that ends a comment starting at
/// `at`, or `None` when it does not end.
fn find_comment_end(input: &[u8], at: usize) -> Option<usize> {
    // "the first 0x3E byte which is preceded by two 0x2D bytes ... and
    // comes after the 0x3C byte that was found". The two dashes may be
    // the ones in `<!--`, so the search starts at `at + 2`.
    let mut p = at.saturating_add(2);
    while let Some(b) = input.get(p) {
        if *b == 0x3E
            && input.get(p.wrapping_sub(1)) == Some(&0x2D)
            && input.get(p.wrapping_sub(2)) == Some(&0x2D)
        {
            return Some(p);
        }
        p = p.saturating_add(1);
    }
    None
}

/// The `<meta>` branch of the prescan, from just after `<meta`.
fn meta(input: &[u8], from: usize) -> Meta {
    let mut at = from;
    let mut seen: Vec<Vec<u8>> = Vec::new();
    let mut got_pragma = false;
    let mut need_pragma: Option<bool> = None;
    // Null, which is distinct from "an unrecognized encoding": an
    // unrecognized `charset` stops the search for this element.
    let mut charset: Option<Option<Encoding>> = None;

    loop {
        let (name, value, next) = match get_attribute(input, at) {
            Attr::Got(n, v, next) => (n, v, next),
            Attr::None => break,
            Attr::Out => return Meta::Out,
        };
        at = next;
        if seen.contains(&name) {
            continue;
        }
        if name == b"http-equiv" {
            if value == b"content-type" {
                got_pragma = true;
            }
        } else if name == b"content" {
            if let Some(e) = extract_from_meta(&value)
                && charset.is_none()
            {
                charset = Some(Some(e));
                need_pragma = Some(true);
            }
        } else if name == b"charset" {
            charset = Some(Encoding::for_label_bytes(&value));
            need_pragma = Some(false);
        }
        seen.push(name);
    }

    match need_pragma {
        None => Meta::Ran(at),
        Some(true) if !got_pragma => Meta::Ran(at),
        Some(_) => match charset.flatten() {
            // "If charset is failure, then jump to ... next byte."
            None => Meta::Ran(at),
            Some(e) => Meta::Found(adjust(e)),
        },
    }
}

/// The two substitutions the prescan makes before returning.
///
/// UTF-16BE/LE becomes UTF-8 and `x-user-defined` becomes
/// windows-1252. Both stop a page naming an encoding under which the
/// ASCII the prescan has already read would mean something else.
const fn adjust(e: Encoding) -> Encoding {
    // `if` and not `match`: the two substitutions are a rule about two
    // encodings, and spelling out the thirty-eight that are left would
    // say nothing and go stale.
    if matches!(e, Encoding::Utf16be | Encoding::Utf16le) {
        Encoding::Utf8
    } else if matches!(e, Encoding::XUserDefined) {
        Encoding::Windows1252
    } else {
        e
    }
}

/// "Get an attribute", from `at`. Returns its lower-cased name, its
/// value and where to carry on, or `None` when there is no attribute.
fn get_attribute(input: &[u8], at: usize) -> Attr {
    let mut p = at;
    while matches!(input.get(p), Some(b) if is_space(*b) || *b == 0x2F) {
        p = p.saturating_add(1);
    }
    if input.get(p) == Some(&0x3E) {
        return Attr::None;
    }
    if input.get(p).is_none() {
        return Attr::Out;
    }

    let mut name: Vec<u8> = Vec::new();
    let value: Vec<u8> = Vec::new();
    // The name.
    loop {
        match input.get(p) {
            None => return Attr::Out,
            Some(0x3D) if !name.is_empty() => {
                p = p.saturating_add(1);
                break;
            }
            Some(b) if is_space(*b) => return spaces(input, p, name, value),
            Some(0x2F | 0x3E) => return Attr::Got(name, value, p),
            Some(b) => {
                name.push(b.to_ascii_lowercase());
                p = p.saturating_add(1);
            }
        }
    }
    value_of(input, p, name)
}

/// The "spaces" label: whitespace after a name, which is an attribute
/// with an empty value unless an `=` follows.
fn spaces(input: &[u8], from: usize, name: Vec<u8>, value: Vec<u8>) -> Attr {
    let mut p = from;
    while matches!(input.get(p), Some(b) if is_space(*b)) {
        p = p.saturating_add(1);
    }
    if input.get(p).is_none() {
        return Attr::Out;
    }
    if input.get(p) != Some(&0x3D) {
        return Attr::Got(name, value, p);
    }
    value_of(input, p.saturating_add(1), name)
}

/// The "value" label, from just past the `=`.
fn value_of(input: &[u8], from: usize, name: Vec<u8>) -> Attr {
    let mut p = from;
    let mut value: Vec<u8> = Vec::new();
    while matches!(input.get(p), Some(b) if is_space(*b)) {
        p = p.saturating_add(1);
    }
    match input.get(p) {
        Some(q @ (0x22 | 0x27)) => {
            let quote = *q;
            p = p.saturating_add(1);
            loop {
                match input.get(p) {
                    None => return Attr::Out,
                    Some(b) if *b == quote => return Attr::Got(name, value, p.saturating_add(1)),
                    Some(b) => {
                        value.push(b.to_ascii_lowercase());
                        p = p.saturating_add(1);
                    }
                }
            }
        }
        Some(0x3E) => Attr::Got(name, value, p),
        None => Attr::Out,
        Some(b) => {
            value.push(b.to_ascii_lowercase());
            p = p.saturating_add(1);
            loop {
                match input.get(p) {
                    None => return Attr::Out,
                    Some(b) if is_space(*b) || *b == 0x3E => return Attr::Got(name, value, p),
                    Some(b) => {
                        value.push(b.to_ascii_lowercase());
                        p = p.saturating_add(1);
                    }
                }
            }
        }
    }
}

/// "Extracting a character encoding from a `meta` element": reads a
/// `content` attribute's value, or a `Content-Type` header's.
///
/// It is deliberately not HTTP's own parameter syntax. The HTML
/// Standard says so: HTTP "doesn't allow the use of single quotes and
/// requires supporting a backslash-escape mechanism that is not
/// supported by this algorithm", and implementations diverged long ago.
#[must_use]
pub fn extract_from_meta(s: &[u8]) -> Option<Encoding> {
    let mut at = 0_usize;
    loop {
        let found = find_ci(s, b"charset", at)?;
        let mut p = found.saturating_add(7);
        while matches!(s.get(p), Some(b) if is_space(*b)) {
            p = p.saturating_add(1);
        }
        if s.get(p) != Some(&0x3D) {
            // "move position to point just before that next character"
            // — the search resumes after this occurrence, not at it,
            // or the loop would not end.
            at = found.saturating_add(1);
            continue;
        }
        p = p.saturating_add(1);
        while matches!(s.get(p), Some(b) if is_space(*b)) {
            p = p.saturating_add(1);
        }
        return match s.get(p) {
            None => None,
            Some(q @ (0x22 | 0x27)) => {
                let rest = s.get(p.saturating_add(1)..)?;
                let end = rest.iter().position(|b| b == q)?;
                Encoding::for_label_bytes(rest.get(..end)?)
            }
            Some(_) => {
                let rest = s.get(p..)?;
                let end = rest
                    .iter()
                    .position(|b| is_space(*b) || *b == 0x3B)
                    .unwrap_or(rest.len());
                Encoding::for_label_bytes(rest.get(..end)?)
            }
        };
    }
}

/// The first ASCII case-insensitive occurrence of `needle` at or after
/// `from`.
fn find_ci(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    let end = haystack.len().checked_sub(needle.len())?;
    (from..=end).find(|i| {
        haystack
            .get(*i..i.saturating_add(needle.len()))
            .is_some_and(|w| w.eq_ignore_ascii_case(needle))
    })
}

/// "Get an XML encoding": the `encoding` pseudo-attribute of an XML
/// declaration, which the prescan falls back to.
#[must_use]
pub fn xml_encoding(bytes: &[u8]) -> Option<Encoding> {
    if !bytes.starts_with(b"<?xml") {
        return None;
    }
    let decl_end = bytes.iter().position(|b| *b == 0x3E)?;
    let mut p = find_ci(bytes.get(..decl_end)?, b"encoding", 0)?.saturating_add(8);
    // "While the byte at encodingPosition is less than or equal to
    // 0x20" — every ASCII space and control character, not only the
    // five HTML calls whitespace.
    while matches!(bytes.get(p), Some(b) if *b <= 0x20) {
        p = p.saturating_add(1);
    }
    if bytes.get(p) != Some(&0x3D) {
        return None;
    }
    p = p.saturating_add(1);
    while matches!(bytes.get(p), Some(b) if *b <= 0x20) {
        p = p.saturating_add(1);
    }
    let quote = *bytes.get(p)?;
    if quote != 0x22 && quote != 0x27 {
        return None;
    }
    let rest = bytes.get(p.saturating_add(1)..)?;
    let end = rest.iter().position(|b| *b == quote)?;
    let label = rest.get(..end)?;
    if label.iter().any(|b| *b <= 0x20) {
        return None;
    }
    let e = Encoding::for_label_bytes(label)?;
    Some(if matches!(e, Encoding::Utf16be | Encoding::Utf16le) {
        Encoding::Utf8
    } else {
        e
    })
}
