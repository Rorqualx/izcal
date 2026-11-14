//! The basic URL parser, Section 4.4.
//!
//! One state machine over the input's code points, written state by
//! state in the standard's own order. A pointer walks the input; a
//! state may move it backwards, which is why it is signed here, and
//! the scheme state may send it back to the start.
//!
//! The `url` and `state_override` arguments exist for the `URL` class's
//! setters, which parse a fragment of a URL into an existing one. They
//! are carried here because leaving them out would mean writing the
//! machine twice.

use hg_encoding::Encoding;

use crate::host::{self, Host};
use crate::percent::{self, Set};
use crate::scheme::{
    self, is_double_dot, is_normalized_windows_drive_letter, is_single_dot,
    is_windows_drive_letter, starts_with_windows_drive_letter,
};
use crate::{Path, Url};

/// A state of the basic URL parser.
///
/// The variants a `state override` may name are the ones the `URL`
/// class's setters use; the rest are reachable only from within a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Before the scheme's first code point.
    SchemeStart,
    /// Inside the scheme.
    Scheme,
    /// After an input with no scheme, which only a base can rescue.
    NoScheme,
    /// A special scheme equal to the base's: `//` or relative.
    SpecialRelativeOrAuthority,
    /// After `scheme:/` for a non-special URL.
    PathOrAuthority,
    /// Relative to the base.
    Relative,
    /// After a `/` in the relative state.
    RelativeSlash,
    /// Expecting `//` after a special scheme.
    SpecialAuthoritySlashes,
    /// Skipping any further slashes a special scheme was given.
    SpecialAuthorityIgnoreSlashes,
    /// Collecting username, password and host together, because only an
    /// `@` later on says which is which.
    Authority,
    /// The host, with the port still possible.
    Host,
    /// The host, with a `:` a failure rather than a port — what the
    /// `hostname` setter needs.
    Hostname,
    /// The port.
    Port,
    /// A `file` URL, before anything after the scheme.
    File,
    /// After one slash in a `file` URL.
    FileSlash,
    /// A `file` URL's host, where a drive letter is not one.
    FileHost,
    /// Before the path's first segment.
    PathStart,
    /// A path segment.
    Path,
    /// The path of a URL that has no host and no segments.
    OpaquePath,
    /// The query.
    Query,
    /// The fragment.
    Fragment,
}

/// What a state's run asks the loop to do next.
enum Flow {
    /// Carry on: advance the pointer unless it is at the end.
    Continue,
    /// Terminate with failure.
    Fail,
    /// Start over from the first code point, in the no scheme state.
    Restart,
}

/// The code point at `at`, or `None` for the EOF code point.
fn code_point(cs: &[char], at: i64) -> Option<char> {
    usize::try_from(at).ok().and_then(|i| cs.get(i)).copied()
}

/// The code points after `at` — the standard's "remaining".
fn remaining(cs: &[char], at: i64) -> &[char] {
    let Some(i) = at.checked_add(1).and_then(|n| usize::try_from(n).ok()) else {
        return &[];
    };
    cs.get(i..).unwrap_or(&[])
}

/// Whether `cs` starts with `prefix`.
fn starts_with(cs: &[char], prefix: &[char]) -> bool {
    cs.len() >= prefix.len() && cs.iter().zip(prefix).all(|(a, b)| a == b)
}

/// A C0 control or space, which the parser strips from both ends.
const fn c0_or_space(c: char) -> bool {
    matches!(c, '\0'..='\u{20}')
}

/// The basic URL parser (Section 4.4).
///
/// `url` and `state_override` are the arguments "only for use by
/// various APIs": with them the algorithm modifies `url` and may
/// terminate without a result, which here is reported by returning the
/// modified `url` either way.
///
/// Returns `None` for failure.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "the standard's one state machine; splitting it would hide \
              the shared pointer, buffer and flags the states read"
)]
pub fn basic_parse(
    input: &str,
    base: Option<&Url>,
    encoding: Encoding,
    url: Option<Url>,
    state_override: Option<State>,
) -> Option<Url> {
    let mut url = url.unwrap_or_default();
    let has_override = state_override.is_some();

    let trimmed = if has_override {
        input
    } else {
        input
            .trim_start_matches(c0_or_space)
            .trim_end_matches(c0_or_space)
    };
    let cs: Vec<char> = trimmed
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();

    let mut state = state_override.unwrap_or(State::SchemeStart);
    let mut encoding = encoding.output_encoding();
    let mut buffer = String::new();
    let mut at_sign_seen = false;
    let mut inside_brackets = false;
    let mut password_token_seen = false;
    let mut ptr = 0_i64;
    let len = i64::try_from(cs.len()).ok()?;

    loop {
        let c = code_point(&cs, ptr);
        let flow = match state {
            State::SchemeStart => match c {
                Some(ch) if ch.is_ascii_alphabetic() => {
                    buffer.push(ch.to_ascii_lowercase());
                    state = State::Scheme;
                    Flow::Continue
                }
                _ if !has_override => {
                    state = State::NoScheme;
                    ptr = ptr.checked_sub(1)?;
                    Flow::Continue
                }
                _ => Flow::Fail,
            },
            State::Scheme => match c {
                Some(ch) if ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.') => {
                    buffer.push(ch.to_ascii_lowercase());
                    Flow::Continue
                }
                Some(':') => {
                    if has_override {
                        if scheme::is_special(&url.scheme) != scheme::is_special(&buffer) {
                            return Some(url);
                        }
                        if (url.includes_credentials() || url.port.is_some()) && buffer == "file" {
                            return Some(url);
                        }
                        if url.scheme == "file" && url.host == Some(Host::Empty) {
                            return Some(url);
                        }
                    }
                    url.scheme = core::mem::take(&mut buffer);
                    if has_override {
                        if url.port == scheme::default_port(&url.scheme) {
                            url.port = None;
                        }
                        return Some(url);
                    }
                    if url.scheme == "file" {
                        state = State::File;
                    } else if url.is_special() && base.is_some_and(|b| b.scheme == url.scheme) {
                        state = State::SpecialRelativeOrAuthority;
                    } else if url.is_special() || scheme::is_registered(&url.scheme) {
                        // A registered scheme takes the same route for
                        // its authority, which is what "needs no
                        // slashes, never opaque-path" means: the two
                        // authority-slash states accept zero, one or
                        // two slashes, and nothing below reaches the
                        // opaque path state.
                        state = State::SpecialAuthoritySlashes;
                    } else if starts_with(remaining(&cs, ptr), &['/']) {
                        state = State::PathOrAuthority;
                        ptr = ptr.checked_add(1)?;
                    } else {
                        url.path = Path::Opaque(String::new());
                        state = State::OpaquePath;
                    }
                    Flow::Continue
                }
                _ if !has_override => Flow::Restart,
                _ => Flow::Fail,
            },
            State::NoScheme => {
                let opaque_base = base.is_some_and(Url::has_opaque_path);
                match base {
                    None => Flow::Fail,
                    Some(_) if opaque_base && c != Some('#') => Flow::Fail,
                    Some(b) if opaque_base => {
                        url.scheme.clone_from(&b.scheme);
                        url.path = b.path.clone();
                        url.query.clone_from(&b.query);
                        url.fragment = Some(String::new());
                        state = State::Fragment;
                        Flow::Continue
                    }
                    Some(b) => {
                        state = if b.scheme == "file" {
                            State::File
                        } else {
                            State::Relative
                        };
                        ptr = ptr.checked_sub(1)?;
                        Flow::Continue
                    }
                }
            }
            State::SpecialRelativeOrAuthority => {
                if c == Some('/') && starts_with(remaining(&cs, ptr), &['/']) {
                    state = State::SpecialAuthorityIgnoreSlashes;
                    ptr = ptr.checked_add(1)?;
                } else {
                    state = State::Relative;
                    ptr = ptr.checked_sub(1)?;
                }
                Flow::Continue
            }
            State::PathOrAuthority => {
                if c == Some('/') {
                    state = State::Authority;
                } else {
                    state = State::Path;
                    ptr = ptr.checked_sub(1)?;
                }
                Flow::Continue
            }
            State::Relative => {
                let b = base?;
                url.scheme.clone_from(&b.scheme);
                if c == Some('/') || (url.is_special() && c == Some('\\')) {
                    state = State::RelativeSlash;
                } else {
                    url.username.clone_from(&b.username);
                    url.password.clone_from(&b.password);
                    url.host.clone_from(&b.host);
                    url.port = b.port;
                    url.path = b.path.clone();
                    url.query.clone_from(&b.query);
                    match c {
                        Some('?') => {
                            url.query = Some(String::new());
                            state = State::Query;
                        }
                        Some('#') => {
                            url.fragment = Some(String::new());
                            state = State::Fragment;
                        }
                        Some(_) => {
                            url.query = None;
                            url.shorten_path();
                            state = State::Path;
                            ptr = ptr.checked_sub(1)?;
                        }
                        None => {}
                    }
                }
                Flow::Continue
            }
            State::RelativeSlash => {
                if url.is_special() && (c == Some('/') || c == Some('\\')) {
                    state = State::SpecialAuthorityIgnoreSlashes;
                } else if c == Some('/') {
                    state = State::Authority;
                } else {
                    let b = base?;
                    url.username.clone_from(&b.username);
                    url.password.clone_from(&b.password);
                    url.host.clone_from(&b.host);
                    url.port = b.port;
                    state = State::Path;
                    ptr = ptr.checked_sub(1)?;
                }
                Flow::Continue
            }
            State::SpecialAuthoritySlashes => {
                if c == Some('/') && starts_with(remaining(&cs, ptr), &['/']) {
                    state = State::SpecialAuthorityIgnoreSlashes;
                    ptr = ptr.checked_add(1)?;
                } else {
                    state = State::SpecialAuthorityIgnoreSlashes;
                    ptr = ptr.checked_sub(1)?;
                }
                Flow::Continue
            }
            State::SpecialAuthorityIgnoreSlashes => {
                // A backslash counts as a slash only for a special
                // scheme. For the standard's own schemes this is the
                // same test it writes, since every URL that reaches
                // here through the standard is special.
                if c != Some('/') && !(url.is_special() && c == Some('\\')) {
                    state = State::Authority;
                    ptr = ptr.checked_sub(1)?;
                }
                Flow::Continue
            }
            State::Authority => {
                if c == Some('@') {
                    if at_sign_seen {
                        buffer.insert_str(0, "%40");
                    }
                    at_sign_seen = true;
                    for cp in buffer.chars() {
                        if cp == ':' && !password_token_seen {
                            password_token_seen = true;
                            continue;
                        }
                        let mut encoded = String::new();
                        percent::encode(cp, Set::Userinfo, &mut encoded);
                        if password_token_seen {
                            url.password.push_str(&encoded);
                        } else {
                            url.username.push_str(&encoded);
                        }
                    }
                    buffer.clear();
                    Flow::Continue
                } else if matches!(c, None | Some('/' | '?' | '#'))
                    || (url.is_special() && c == Some('\\'))
                {
                    if at_sign_seen && buffer.is_empty() {
                        return None;
                    }
                    let back = i64::try_from(buffer.chars().count()).ok()?.checked_add(1)?;
                    ptr = ptr.checked_sub(back)?;
                    buffer.clear();
                    state = State::Host;
                    Flow::Continue
                } else {
                    buffer.push(c?);
                    Flow::Continue
                }
            }
            State::Host | State::Hostname => {
                if has_override && url.scheme == "file" {
                    ptr = ptr.checked_sub(1)?;
                    state = State::FileHost;
                    Flow::Continue
                } else if c == Some(':') && !inside_brackets {
                    if buffer.is_empty() {
                        return None;
                    }
                    if state_override == Some(State::Hostname) {
                        return None;
                    }
                    url.host = Some(host::parse(&buffer, !url.is_special())?);
                    buffer.clear();
                    state = State::Port;
                    Flow::Continue
                } else if matches!(c, None | Some('/' | '?' | '#'))
                    || (url.is_special() && c == Some('\\'))
                {
                    ptr = ptr.checked_sub(1)?;
                    if (url.is_special() || scheme::is_registered(&url.scheme)) && buffer.is_empty()
                    {
                        // A registered scheme fails on an empty host
                        // for the reason a special scheme does, read
                        // from its own grammar: `ext://<id>/` names the
                        // id, and "ext://<id> is one origin and one
                        // site key" (HGBA-2 I-1). An empty id would be
                        // an origin belonging to no extension, which is
                        // a site key two extensions could share.
                        //
                        // This is a decision, not a quoted rule: the
                        // style spec states the two rules in
                        // `scheme::REGISTERED` and is silent here. It
                        // goes to ST-M0's design review with the
                        // mis-cite above it.
                        return None;
                    }
                    if has_override
                        && buffer.is_empty()
                        && (url.includes_credentials() || url.port.is_some())
                    {
                        return None;
                    }
                    url.host = Some(host::parse(&buffer, !url.is_special())?);
                    buffer.clear();
                    state = State::PathStart;
                    if has_override {
                        return Some(url);
                    }
                    Flow::Continue
                } else {
                    let ch = c?;
                    if ch == '[' {
                        inside_brackets = true;
                    }
                    if ch == ']' {
                        inside_brackets = false;
                    }
                    buffer.push(ch);
                    Flow::Continue
                }
            }
            State::Port => {
                if c.is_some_and(|ch| ch.is_ascii_digit()) {
                    buffer.push(c?);
                    Flow::Continue
                } else if matches!(c, None | Some('/' | '?' | '#'))
                    || (url.is_special() && c == Some('\\'))
                    || has_override
                {
                    if !buffer.is_empty() {
                        let mut port = 0_u32;
                        for d in buffer.chars() {
                            port = port
                                .saturating_mul(10)
                                .saturating_add(d.to_digit(10).unwrap_or(0));
                        }
                        let port = u16::try_from(port).ok()?;
                        url.port = if Some(port) == scheme::default_port(&url.scheme) {
                            None
                        } else {
                            Some(port)
                        };
                        buffer.clear();
                    }
                    if has_override {
                        return Some(url);
                    }
                    state = State::PathStart;
                    ptr = ptr.checked_sub(1)?;
                    Flow::Continue
                } else {
                    Flow::Fail
                }
            }
            State::File => {
                url.scheme = "file".to_owned();
                url.host = Some(Host::Empty);
                if c == Some('/') || c == Some('\\') {
                    state = State::FileSlash;
                } else if let Some(b) = base.filter(|b| b.scheme == "file") {
                    url.host.clone_from(&b.host);
                    url.path = b.path.clone();
                    url.query.clone_from(&b.query);
                    match c {
                        Some('?') => {
                            url.query = Some(String::new());
                            state = State::Query;
                        }
                        Some('#') => {
                            url.fragment = Some(String::new());
                            state = State::Fragment;
                        }
                        Some(_) => {
                            url.query = None;
                            if starts_with_windows_drive_letter(&cs, usize::try_from(ptr).ok()?) {
                                url.path = Path::List(Vec::new());
                            } else {
                                url.shorten_path();
                            }
                            state = State::Path;
                            ptr = ptr.checked_sub(1)?;
                        }
                        None => {}
                    }
                } else {
                    state = State::Path;
                    ptr = ptr.checked_sub(1)?;
                }
                Flow::Continue
            }
            State::FileSlash => {
                if c == Some('/') || c == Some('\\') {
                    state = State::FileHost;
                } else {
                    if let Some(b) = base.filter(|b| b.scheme == "file") {
                        url.host.clone_from(&b.host);
                        let starts_drive = usize::try_from(ptr)
                            .ok()
                            .is_some_and(|i| starts_with_windows_drive_letter(&cs, i));
                        if !starts_drive
                            && let Path::List(segments) = &b.path
                            && segments
                                .first()
                                .is_some_and(|s| is_normalized_windows_drive_letter(s))
                            && let Some(first) = segments.first()
                        {
                            url.push_segment(first.clone());
                        }
                    }
                    state = State::Path;
                    ptr = ptr.checked_sub(1)?;
                }
                Flow::Continue
            }
            State::FileHost => {
                if matches!(c, None | Some('/' | '\\' | '?' | '#')) {
                    ptr = ptr.checked_sub(1)?;
                    if !has_override && is_windows_drive_letter(&buffer) {
                        // The buffer is deliberately not cleared: the
                        // path state reads it as the first segment.
                        state = State::Path;
                    } else if buffer.is_empty() {
                        url.host = Some(Host::Empty);
                        if has_override {
                            return Some(url);
                        }
                        state = State::PathStart;
                    } else {
                        let parsed = host::parse(&buffer, !url.is_special())?;
                        url.host = Some(if parsed == Host::Domain("localhost".to_owned()) {
                            Host::Empty
                        } else {
                            parsed
                        });
                        if has_override {
                            return Some(url);
                        }
                        buffer.clear();
                        state = State::PathStart;
                    }
                    Flow::Continue
                } else {
                    buffer.push(c?);
                    Flow::Continue
                }
            }
            State::PathStart => {
                if url.is_special() {
                    state = State::Path;
                    if c != Some('/') && c != Some('\\') {
                        ptr = ptr.checked_sub(1)?;
                    }
                } else if !has_override && c == Some('?') {
                    url.query = Some(String::new());
                    state = State::Query;
                } else if !has_override && c == Some('#') {
                    url.fragment = Some(String::new());
                    state = State::Fragment;
                } else if c.is_some() {
                    state = State::Path;
                    if c != Some('/') {
                        ptr = ptr.checked_sub(1)?;
                    }
                } else if has_override && url.host.is_none() {
                    url.push_segment(String::new());
                }
                Flow::Continue
            }
            State::Path => {
                let boundary = matches!(c, None | Some('/'))
                    || (url.is_special() && c == Some('\\'))
                    || (!has_override && matches!(c, Some('?' | '#')));
                if boundary {
                    let slash_like = c == Some('/') || (url.is_special() && c == Some('\\'));
                    if is_double_dot(&buffer) {
                        url.shorten_path();
                        if !slash_like {
                            url.push_segment(String::new());
                        }
                    } else if is_single_dot(&buffer) {
                        if !slash_like {
                            url.push_segment(String::new());
                        }
                    } else {
                        if url.scheme == "file"
                            && url.path_is_empty()
                            && is_windows_drive_letter(&buffer)
                            && let Some(first) = buffer.chars().next()
                        {
                            buffer = format!("{first}:");
                        }
                        url.push_segment(core::mem::take(&mut buffer));
                    }
                    buffer.clear();
                    if c == Some('?') {
                        url.query = Some(String::new());
                        state = State::Query;
                    }
                    if c == Some('#') {
                        url.fragment = Some(String::new());
                        state = State::Fragment;
                    }
                } else {
                    percent::encode(c?, Set::Path, &mut buffer);
                }
                Flow::Continue
            }
            State::OpaquePath => match c {
                Some('?') => {
                    url.query = Some(String::new());
                    state = State::Query;
                    Flow::Continue
                }
                Some('#') => {
                    url.fragment = Some(String::new());
                    state = State::Fragment;
                    Flow::Continue
                }
                Some(' ') => {
                    let rest = remaining(&cs, ptr);
                    if starts_with(rest, &['?']) || starts_with(rest, &['#']) {
                        url.push_opaque("%20");
                    } else {
                        url.push_opaque(" ");
                    }
                    Flow::Continue
                }
                Some(ch) => {
                    let mut piece = String::new();
                    percent::encode(ch, Set::C0Control, &mut piece);
                    url.push_opaque(&piece);
                    Flow::Continue
                }
                None => Flow::Continue,
            },
            State::Query => {
                if encoding != Encoding::Utf8
                    && (!url.is_special() || url.scheme == "ws" || url.scheme == "wss")
                {
                    encoding = Encoding::Utf8;
                }
                if (!has_override && c == Some('#')) || c.is_none() {
                    let set = if url.is_special() {
                        Set::SpecialQuery
                    } else {
                        Set::Query
                    };
                    let encoded = percent::encode_after_encoding(encoding, &buffer, set);
                    url.query.get_or_insert_with(String::new).push_str(&encoded);
                    buffer.clear();
                    if c == Some('#') {
                        url.fragment = Some(String::new());
                        state = State::Fragment;
                    }
                } else if let Some(ch) = c {
                    buffer.push(ch);
                }
                Flow::Continue
            }
            State::Fragment => {
                if let Some(ch) = c {
                    let slot = url.fragment.get_or_insert_with(String::new);
                    percent::encode(ch, Set::Fragment, slot);
                }
                Flow::Continue
            }
        };

        match flow {
            Flow::Continue => {}
            Flow::Fail => return None,
            Flow::Restart => {
                buffer.clear();
                state = State::NoScheme;
                ptr = -1;
            }
        }
        if ptr >= len {
            break;
        }
        ptr = ptr.checked_add(1)?;
    }
    Some(url)
}
