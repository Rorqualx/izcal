//! Hosts, URL Standard Sections 3.2 to 3.7.
//!
//! A host is a domain, an IPv4 address, an IPv6 address, an opaque host
//! or the empty host. Which one a string becomes is decided by the host
//! parser, and the decision is not syntactic: `1.2.3.4` is an IPv4
//! address, `a.b.c.d` is a domain, and `0x7f.1` is an IPv4 address
//! because the standard's "ends in a number" checker says so.
//!
//! The domain branch runs UTS 46 through [`hg_unicode::idna`], which is
//! why this crate sits above `hg-unicode`: a URL's host is where IDNA
//! enters the browser.

use hg_unicode::idna::{self, Flags};

use crate::percent::{self, Set};

/// A parsed host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Host {
    /// A domain, already converted to ASCII by the domain parser.
    Domain(String),
    /// An IPv4 address, as the standard holds it: one 32-bit number.
    Ipv4(u32),
    /// An IPv6 address, as eight 16-bit pieces.
    Ipv6([u16; 8]),
    /// A host that is not a domain and not an address: everything a
    /// non-special URL's host can be.
    Opaque(String),
    /// The empty host, which only a non-special URL or a `file` URL may
    /// have.
    Empty,
}

/// A forbidden host code point (Section 3.2).
#[must_use]
pub const fn forbidden_host(c: char) -> bool {
    matches!(
        c,
        '\0' | '\t'
            | '\n'
            | '\r'
            | ' '
            | '#'
            | '/'
            | ':'
            | '<'
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '|'
    )
}

/// A forbidden domain code point (Section 3.2): a forbidden host code
/// point, a C0 control, `%` or DELETE.
#[must_use]
pub const fn forbidden_domain(c: char) -> bool {
    forbidden_host(c) || matches!(c, '\0'..='\u{1F}' | '%' | '\u{7F}')
}

/// The URL code points (Section 4.3).
///
/// Not a parsing decision: a code point outside this set is a
/// validation error, and the parser carries on.
#[must_use]
pub fn url_code_point(c: char) -> bool {
    if c.is_ascii_alphanumeric() {
        return true;
    }
    if matches!(
        c,
        '!' | '$'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | ':'
            | ';'
            | '='
            | '?'
            | '@'
            | '_'
            | '~'
    ) {
        return true;
    }
    ('\u{A0}'..='\u{10FFFD}').contains(&c) && !noncharacter(c)
}

/// An Infra noncharacter.
fn noncharacter(c: char) -> bool {
    let v = u32::from(c);
    if (0xFDD0..=0xFDEF).contains(&v) {
        return true;
    }
    matches!(v & 0xFFFF, 0xFFFE | 0xFFFF)
}

/// The domain parser (Section 3.3), which is UTS 46's ToASCII twice:
/// once strictly, to record a validation error, and once leniently,
/// whose answer is returned.
///
/// The strict pass is run for its error alone, as the standard says:
/// "This step does not return."
///
/// # The ASCII shortcut, including `xn--` labels
///
/// When `be_strict` is false and the domain is already ASCII, the
/// answer is the domain lowercased **whatever UTS 46 thinks** — the
/// standard's own words are "regardless of Unicode ToASCII's outcome,
/// due to web compatibility". That is not a loophole to be narrowed:
/// the note goes on to say that `IgnoreInvalidPunycode` "is not
/// sufficient on its own, as Punycode can decode successfully yet
/// still fail validity criteria", and that the shortcut is what makes
/// the domain parser and "domain to Unicode" round-trip "on input such
/// as xn--8i7caa".
///
/// So an ASCII label beginning `xn--` is **not** run through Punycode
/// here, and an invalid one is accepted as it stands. WPT's
/// `toascii.json` pins exactly this: `xn--a` carries the comment
/// "Invalid Punycode" and expects the output `xn--a`, and `xn--0.com`
/// and `xn--zn7c.com` ("U+FFFD character encoded in Punycode") expect
/// themselves too. Routing `xn--` labels through UTS 46 would fail
/// those cases.
///
/// What still guards the result is the two checks below: the empty
/// domain and the forbidden domain code points. A label's *meaning* is
/// a display question, which belongs to the shell's IDN display policy
/// and not to the URL parser.
#[must_use]
pub fn domain_to_ascii(domain: &str, be_strict: bool) -> Option<String> {
    if be_strict {
        // The strict pass is the whole answer when `beStrict` is true.
        return idna::to_ascii(domain, Flags::url(true), true);
    }
    // When `beStrict` is false the standard still runs the strict pass,
    // but only to record a `domain-to-ASCII` validation error — "This
    // step does not return." Validation errors are not surfaced by this
    // crate yet, so running UTS 46 a second time would buy nothing; the
    // step is named here so its absence is a recorded choice, not an
    // omission.
    let result = if domain.is_ascii() {
        domain.to_ascii_lowercase()
    } else {
        idna::to_ascii(domain, Flags::url(false), false)?
    };
    if result.is_empty() || result.chars().any(forbidden_domain) {
        return None;
    }
    Some(result)
}

/// The host parser (Section 3.5).
///
/// `is_opaque` is the parser's "url is not special".
#[must_use]
pub fn parse(input: &str, is_opaque: bool) -> Option<Host> {
    if let Some(rest) = input.strip_prefix('[') {
        let inner = rest.strip_suffix(']')?;
        return parse_ipv6(inner).map(Host::Ipv6);
    }
    if is_opaque {
        return parse_opaque(input);
    }
    if input.is_empty() {
        // The parser's own assertion. An empty host reaches the host
        // state only for a non-special URL, which took the branch
        // above; answering rather than asserting keeps tier P's rule.
        return Some(Host::Empty);
    }
    let decoded = percent::decode(input.as_bytes());
    let domain = String::from_utf8_lossy(&decoded).into_owned();
    let ascii = domain_to_ascii(&domain, false)?;
    if ends_in_a_number(&ascii) {
        return parse_ipv4(&ascii).map(Host::Ipv4);
    }
    Some(Host::Domain(ascii))
}

/// The opaque-host parser (Section 3.5).
#[must_use]
pub fn parse_opaque(input: &str) -> Option<Host> {
    if input.chars().any(forbidden_host) {
        return None;
    }
    if input.is_empty() {
        return Some(Host::Empty);
    }
    Some(Host::Opaque(percent::encode_str(input, Set::C0Control)))
}

/// The ends-in-a-number checker (Section 3.5).
#[must_use]
pub fn ends_in_a_number(input: &str) -> bool {
    let mut parts: Vec<&str> = input.split('.').collect();
    if parts.last().is_some_and(|p| p.is_empty()) {
        parts.pop();
    }
    let Some(last) = parts.last() else {
        return false;
    };
    if !last.is_empty() && last.bytes().all(|b| b.is_ascii_digit()) {
        return true;
    }
    parse_ipv4_number(last).is_some()
}

/// Saturating accumulation, so a number too large for `u64` still
/// compares greater than every threshold the IPv4 parser checks.
///
/// The standard speaks of a mathematical integer. Saturating at
/// `u64::MAX` gives the same answer, because every use of the value is
/// a comparison against a bound no larger than 256⁴.
fn push_digit(acc: u64, radix: u64, digit: u64) -> u64 {
    acc.saturating_mul(radix).saturating_add(digit)
}

/// The IPv4 number parser (Section 3.5): the number, and whether a
/// validation error was recorded.
#[must_use]
pub fn parse_ipv4_number(input: &str) -> Option<(u64, bool)> {
    if input.is_empty() {
        return None;
    }
    let mut rest = input;
    let mut validation_error = false;
    let mut radix = 10_u64;
    if rest.len() >= 2 && (rest.starts_with("0x") || rest.starts_with("0X")) {
        validation_error = true;
        rest = rest.get(2..)?;
        radix = 16;
    } else if rest.len() >= 2 && rest.starts_with('0') {
        validation_error = true;
        rest = rest.get(1..)?;
        radix = 8;
    }
    if rest.is_empty() {
        return Some((0, true));
    }
    let mut out = 0_u64;
    for b in rest.bytes() {
        let d = u64::from(char::from(b).to_digit(u32::try_from(radix).ok()?)?);
        out = push_digit(out, radix, d);
    }
    Some((out, validation_error))
}

/// The IPv4 parser (Section 3.5).
#[must_use]
pub fn parse_ipv4(input: &str) -> Option<u32> {
    let mut parts: Vec<&str> = input.split('.').collect();
    if parts.last().is_some_and(|p| p.is_empty()) && parts.len() > 1 {
        parts.pop();
    }
    if parts.len() > 4 {
        return None;
    }
    let mut numbers: Vec<u64> = Vec::with_capacity(parts.len());
    for part in &parts {
        let (n, _) = parse_ipv4_number(part)?;
        numbers.push(n);
    }
    let size = u32::try_from(numbers.len()).ok()?;
    let (last, head) = numbers.split_last()?;
    if head.iter().any(|n| *n > 255) {
        return None;
    }
    // 256 ^ (5 − size), which for size 1 to 4 is 256⁴ down to 256.
    let limit = 256_u64.checked_pow(5_u32.checked_sub(size)?)?;
    if *last >= limit {
        return None;
    }
    let mut ipv4 = *last;
    for (counter, n) in head.iter().enumerate() {
        let shift = 3_u32.checked_sub(u32::try_from(counter).ok()?)?;
        let scale = 256_u64.checked_pow(shift)?;
        ipv4 = ipv4.checked_add(n.checked_mul(scale)?)?;
    }
    u32::try_from(ipv4).ok()
}

/// Swaps two pieces without a path that can panic.
fn swap(pieces: &mut [u16; 8], i: usize, j: usize) {
    let (Some(a), Some(b)) = (pieces.get(i).copied(), pieces.get(j).copied()) else {
        return;
    };
    if let Some(s) = pieces.get_mut(i) {
        *s = b;
    }
    if let Some(s) = pieces.get_mut(j) {
        *s = a;
    }
}

/// The IPv6 parser (Section 3.5).
#[must_use]
pub fn parse_ipv6(input: &str) -> Option<[u16; 8]> {
    let cs: Vec<char> = input.chars().collect();
    let mut address = [0_u16; 8];
    let mut piece_index = 0_usize;
    let mut compress: Option<usize> = None;
    let mut at = 0_usize;

    if cs.get(at) == Some(&':') {
        if cs.get(at.checked_add(1)?) != Some(&':') {
            return None;
        }
        at = at.checked_add(2)?;
        piece_index = piece_index.checked_add(1)?;
        compress = Some(piece_index);
    }

    while let Some(c) = cs.get(at).copied() {
        if piece_index == 8 {
            return None;
        }
        if c == ':' {
            if compress.is_some() {
                return None;
            }
            at = at.checked_add(1)?;
            piece_index = piece_index.checked_add(1)?;
            compress = Some(piece_index);
            continue;
        }
        let mut value = 0_u32;
        let mut length = 0_usize;
        while length < 4
            && let Some(d) = cs.get(at).copied().and_then(|c| c.to_digit(16))
        {
            value = value.checked_mul(0x10)?.checked_add(d)?;
            at = at.checked_add(1)?;
            length = length.checked_add(1)?;
        }
        if cs.get(at) == Some(&'.') {
            if length == 0 {
                return None;
            }
            at = at.checked_sub(length)?;
            if piece_index > 6 {
                return None;
            }
            let mut numbers_seen = 0_usize;
            while at < cs.len() {
                let mut ipv4_piece: Option<u32> = None;
                if numbers_seen > 0 {
                    if cs.get(at) == Some(&'.') && numbers_seen < 4 {
                        at = at.checked_add(1)?;
                    } else {
                        return None;
                    }
                }
                if !cs.get(at).is_some_and(char::is_ascii_digit) {
                    return None;
                }
                while let Some(d) = cs.get(at).copied().and_then(|c| c.to_digit(10)) {
                    match ipv4_piece {
                        None => ipv4_piece = Some(d),
                        Some(0) => return None,
                        Some(p) => ipv4_piece = Some(p.checked_mul(10)?.checked_add(d)?),
                    }
                    if ipv4_piece.is_some_and(|p| p > 255) {
                        return None;
                    }
                    at = at.checked_add(1)?;
                }
                let piece = pieces_at(&address, piece_index)?;
                let v = u16::try_from(
                    u32::from(piece)
                        .checked_mul(0x100)?
                        .checked_add(ipv4_piece?)?,
                )
                .ok()?;
                *address.get_mut(piece_index)? = v;
                numbers_seen = numbers_seen.checked_add(1)?;
                if numbers_seen == 2 || numbers_seen == 4 {
                    piece_index = piece_index.checked_add(1)?;
                }
            }
            if numbers_seen != 4 {
                return None;
            }
            break;
        }
        if cs.get(at) == Some(&':') {
            at = at.checked_add(1)?;
            if at >= cs.len() {
                return None;
            }
        } else if at < cs.len() {
            return None;
        }
        // A leading zero in a piece (`length > 1` and `value` below
        // 0x10^(length−1)) is an `IPv6-piece-leading-zero` validation
        // error, not a failure, so nothing branches on it here.
        *address.get_mut(piece_index)? = u16::try_from(value).ok()?;
        piece_index = piece_index.checked_add(1)?;
    }

    if let Some(compress) = compress {
        let mut swaps = piece_index.checked_sub(compress)?;
        piece_index = 7;
        while piece_index != 0 && swaps > 0 {
            swap(
                &mut address,
                piece_index,
                compress.checked_add(swaps)?.checked_sub(1)?,
            );
            piece_index = piece_index.checked_sub(1)?;
            swaps = swaps.checked_sub(1)?;
        }
    } else if piece_index != 8 {
        return None;
    }
    Some(address)
}

fn pieces_at(address: &[u16; 8], i: usize) -> Option<u16> {
    address.get(i).copied()
}

/// The IPv4 serializer (Section 3.6).
#[must_use]
pub fn serialize_ipv4(address: u32) -> String {
    let mut out = String::new();
    let mut n = address;
    let mut parts = [0_u32; 4];
    for slot in parts.iter_mut().rev() {
        *slot = n.checked_rem(256).unwrap_or(0);
        n = n.checked_div(256).unwrap_or(0);
    }
    for (i, p) in parts.iter().enumerate() {
        if i > 0 {
            out.push('.');
        }
        out.push_str(&p.to_string());
    }
    out
}

/// The compressed piece index (Section 3.6): the start of the longest
/// run of two or more zero pieces, or `None`.
#[must_use]
pub fn compressed_piece_index(address: &[u16; 8]) -> Option<usize> {
    let mut longest_index: Option<usize> = None;
    let mut longest_size = 1_usize;
    let mut found_index: Option<usize> = None;
    let mut found_size = 0_usize;
    for (i, piece) in address.iter().enumerate() {
        if *piece != 0 {
            if found_size > longest_size {
                longest_index = found_index;
                longest_size = found_size;
            }
            found_index = None;
            found_size = 0;
        } else {
            if found_index.is_none() {
                found_index = Some(i);
            }
            found_size = found_size.saturating_add(1);
        }
    }
    // The last check is after the loop, not inside it: a run that ends
    // at the final piece is only compared once everything is seen. Run
    // inside the loop it would answer `1:2::5:0:0:0` for
    // `1:2:0:0:5:0:0:0`, taking the first long-enough run rather than
    // the longest.
    if found_size > longest_size {
        return found_index;
    }
    longest_index
}

/// The IPv6 serializer (Section 3.6), which is RFC 5952's form.
#[must_use]
pub fn serialize_ipv6(address: &[u16; 8]) -> String {
    let mut out = String::new();
    let compress = compressed_piece_index(address);
    let mut ignore0 = false;
    for (i, piece) in address.iter().enumerate() {
        if ignore0 && *piece == 0 {
            continue;
        }
        ignore0 = false;
        if compress == Some(i) {
            out.push_str(if i == 0 { "::" } else { ":" });
            ignore0 = true;
            continue;
        }
        out.push_str(&format!("{piece:x}"));
        if i != 7 {
            out.push(':');
        }
    }
    out
}

impl Host {
    /// The host serializer (Section 3.6).
    #[must_use]
    pub fn serialize(&self) -> String {
        match self {
            Self::Ipv4(a) => serialize_ipv4(*a),
            Self::Ipv6(a) => format!("[{}]", serialize_ipv6(a)),
            Self::Domain(s) | Self::Opaque(s) => s.clone(),
            Self::Empty => String::new(),
        }
    }
}

impl core::fmt::Display for Host {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.serialize())
    }
}
