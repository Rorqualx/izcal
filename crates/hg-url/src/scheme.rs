//! Special schemes, Section 4.2.
//!
//! Six schemes are special, and what makes them special is woven
//! through the parser: a backslash counts as a slash, an empty host is
//! a failure, `//` may be omitted, and the default port is dropped.
//! The table is the standard's own.

/// The special schemes and their default ports, in the standard's
/// order.
pub const SPECIAL: [(&str, Option<u16>); 6] = [
    ("ftp", Some(21)),
    ("file", None),
    ("http", Some(80)),
    ("https", Some(443)),
    ("ws", Some(80)),
    ("wss", Some(443)),
];

/// Whether `scheme` is a special scheme.
#[must_use]
pub fn is_special(scheme: &str) -> bool {
    SPECIAL.iter().any(|(s, _)| *s == scheme)
}

/// A scheme's default port, or `None` — both for a scheme with no
/// default and for `file`, which is special and has none.
#[must_use]
pub fn default_port(scheme: &str) -> Option<u16> {
    SPECIAL
        .iter()
        .find(|(s, _)| *s == scheme)
        .and_then(|(_, p)| *p)
}

/// The schemes Izcal registers beyond the standard's.
///
/// `ext://<id>/` is the extensions spec's extension origin (EX-D4),
/// registered here because G31 allows exactly one URL parser: a check
/// elsewhere that special-cased `ext://` would be the divergence G31
/// exists to prevent (style spec 6.1).
///
/// A registered scheme is **not** a special scheme. It gains two rules
/// and nothing else:
///
/// - **it needs no slashes** — `ext:abc/page` parses its authority just
///   as `ext://abc/page` does;
/// - **it is never opaque-path** — it always has a host, so
///   `ext://<id>` is one origin and one site key (HGBA-2 I-1).
///
/// It does not gain a default port, backslash-as-slash, an
/// empty-host failure, or domain parsing: an extension ID is an opaque
/// host, because the style spec states the scheme "never becomes a
/// registrable domain".
pub const REGISTERED: [&str; 1] = ["ext"];

/// Whether `scheme` is one Izcal registers (see [`REGISTERED`]).
#[must_use]
pub fn is_registered(scheme: &str) -> bool {
    REGISTERED.contains(&scheme)
}

/// A Windows drive letter (Section 4.2): an ASCII alpha then `:` or
/// `|`.
#[must_use]
pub fn is_windows_drive_letter(s: &str) -> bool {
    let mut cs = s.chars();
    let (Some(a), Some(b), None) = (cs.next(), cs.next(), cs.next()) else {
        return false;
    };
    a.is_ascii_alphabetic() && (b == ':' || b == '|')
}

/// A normalized Windows drive letter: one whose second code point is
/// `:`. Only this form is conforming.
#[must_use]
pub fn is_normalized_windows_drive_letter(s: &str) -> bool {
    let mut cs = s.chars();
    let (Some(a), Some(b), None) = (cs.next(), cs.next(), cs.next()) else {
        return false;
    };
    a.is_ascii_alphabetic() && b == ':'
}

/// Whether `cs` from `at` starts with a Windows drive letter
/// (Section 4.2): two code points that are one, then the end or one of
/// `/`, `\`, `?`, `#`.
#[must_use]
pub fn starts_with_windows_drive_letter(cs: &[char], at: usize) -> bool {
    let (Some(a), Some(b)) = (
        cs.get(at).copied(),
        at.checked_add(1).and_then(|i| cs.get(i)).copied(),
    ) else {
        return false;
    };
    if !(a.is_ascii_alphabetic() && (b == ':' || b == '|')) {
        return false;
    }
    match at.checked_add(2).and_then(|i| cs.get(i)).copied() {
        None => true,
        Some(c) => matches!(c, '/' | '\\' | '?' | '#'),
    }
}

/// A single-dot URL path segment (Section 4.1).
#[must_use]
pub fn is_single_dot(segment: &str) -> bool {
    segment == "." || segment.eq_ignore_ascii_case("%2e")
}

/// A double-dot URL path segment (Section 4.1).
#[must_use]
pub fn is_double_dot(segment: &str) -> bool {
    segment == ".."
        || segment.eq_ignore_ascii_case(".%2e")
        || segment.eq_ignore_ascii_case("%2e.")
        || segment.eq_ignore_ascii_case("%2e%2e")
}
