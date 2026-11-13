//! `hg-url`: Izcal's URL Standard implementation. L2s, tier P
//! (style spec 6.4, G31).
//!
//! **The browser has one URL parser.** Every place a URL is read — a
//! link, a `fetch`, a redirect's `Location`, a `<base>`, a CSP source
//! expression, the network service's own routing, the broker's policy
//! checks — reads this crate. G31 exists because a second parser is a
//! security bug waiting to happen: the check and the use must agree on
//! what the URL *is*, and two implementations eventually will not.
//!
//! That is also why it is **tier P**: the network services and the
//! broker link it, so it may never panic (G24). No slice is indexed,
//! no arithmetic wraps, and every table lookup is checked.
//!
//! # What is here
//!
//! - [`Url`] — a URL record: scheme, username, password, host, port,
//!   path, query and fragment, with the standard's own invariants.
//! - [`parse`] and [`Url::parse`] — the URL parser, which is the basic
//!   URL parser plus the `blob` hook the standard adds on top.
//! - [`parse::basic_parse`] — the basic URL parser itself, with the
//!   `url` and `state_override` arguments the `URL` class's setters
//!   need.
//! - [`host::Host`] — a domain, an IPv4 address, an IPv6 address, an
//!   opaque host or the empty host, with the host parser and the host
//!   serializer.
//! - [`percent`] — the six percent-encode sets, percent-encoding and
//!   percent-decoding, and "percent-encode after encoding", which is
//!   where a legacy query encoding enters.
//!
//! # What is not here
//!
//! `application/x-www-form-urlencoded` and the `URL` and
//! `URLSearchParams` classes: those are the DOM track's, and they are
//! written over this crate rather than beside it.

pub mod host;
pub mod parse;
pub mod percent;
pub mod scheme;

pub use host::Host;
pub use parse::State;

use hg_encoding::Encoding;

/// A URL's path (Section 4.1).
///
/// A URL has either a list of segments or a single opaque string, and
/// which one it has changes what almost every other algorithm may do
/// to it — so the two are different shapes here rather than a list with
/// a flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Path {
    /// A list of zero or more segments.
    List(Vec<String>),
    /// One opaque segment, which `shorten` and `/..` never touch.
    Opaque(String),
}

impl Default for Path {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

/// A URL record (Section 4.1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Url {
    /// The scheme, lower-cased, without its `:`.
    pub scheme: String,
    /// The username, percent-encoded; empty when there is none.
    pub username: String,
    /// The password, percent-encoded; empty when there is none.
    pub password: String,
    /// The host, or `None` for a URL that has none.
    pub host: Option<Host>,
    /// The port, or `None` — which is also what a scheme's default
    /// port becomes.
    pub port: Option<u16>,
    /// The path.
    pub path: Path,
    /// The query, without its `?`.
    pub query: Option<String>,
    /// The fragment, without its `#`.
    pub fragment: Option<String>,
}

impl Url {
    /// The URL parser (Section 4.4), with UTF-8 and no base.
    ///
    /// # Errors
    /// `None` when the input is not a URL.
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        parse(input, None, Encoding::Utf8)
    }

    /// The URL parser against a base URL.
    ///
    /// # Errors
    /// `None` when the input is not a URL, relative to that base.
    #[must_use]
    pub fn parse_with_base(input: &str, base: &Self) -> Option<Self> {
        parse(input, Some(base), Encoding::Utf8)
    }

    /// Whether the scheme is a special scheme (Section 4.2).
    #[must_use]
    pub fn is_special(&self) -> bool {
        scheme::is_special(&self.scheme)
    }

    /// Whether the URL includes credentials: a non-empty username or
    /// password.
    #[must_use]
    pub fn includes_credentials(&self) -> bool {
        !self.username.is_empty() || !self.password.is_empty()
    }

    /// Whether the path is opaque.
    #[must_use]
    pub const fn has_opaque_path(&self) -> bool {
        matches!(self.path, Path::Opaque(_))
    }

    /// Whether the path holds no segments. An opaque path is never
    /// empty in this sense: it is one string, not a list.
    #[must_use]
    pub fn path_is_empty(&self) -> bool {
        match &self.path {
            Path::List(segments) => segments.is_empty(),
            Path::Opaque(_) => false,
        }
    }

    /// Appends a segment, or — for an opaque path — the text itself.
    pub fn push_segment(&mut self, segment: String) {
        match &mut self.path {
            Path::List(segments) => segments.push(segment),
            Path::Opaque(s) => s.push_str(&segment),
        }
    }

    /// Appends to an opaque path.
    pub fn push_opaque(&mut self, text: &str) {
        match &mut self.path {
            Path::Opaque(s) => s.push_str(text),
            Path::List(segments) => segments.push(text.to_owned()),
        }
    }

    /// Shortening a URL's path (Section 4.2).
    ///
    /// The `file` exception is the Windows drive letter quirk: `..`
    /// may not remove `C:` from `file:///C:/`, which is why `..`
    /// against `file:///C:/demo` gives `file:///C:/`.
    pub fn shorten_path(&mut self) {
        let is_file = self.scheme == "file";
        let Path::List(segments) = &mut self.path else {
            return;
        };
        if is_file
            && segments.len() == 1
            && segments
                .first()
                .is_some_and(|s| scheme::is_normalized_windows_drive_letter(s))
        {
            return;
        }
        segments.pop();
    }

    /// The URL path serializer (Section 4.5).
    #[must_use]
    pub fn serialize_path(&self) -> String {
        match &self.path {
            Path::Opaque(s) => s.clone(),
            Path::List(segments) => {
                let mut out = String::new();
                for segment in segments {
                    out.push('/');
                    out.push_str(segment);
                }
                out
            }
        }
    }

    /// The URL serializer (Section 4.5).
    #[must_use]
    pub fn serialize(&self, exclude_fragment: bool) -> String {
        let mut out = String::with_capacity(self.scheme.len().saturating_add(16));
        out.push_str(&self.scheme);
        out.push(':');
        if let Some(host) = &self.host {
            out.push_str("//");
            if self.includes_credentials() {
                out.push_str(&self.username);
                if !self.password.is_empty() {
                    out.push(':');
                    out.push_str(&self.password);
                }
                out.push('@');
            }
            out.push_str(&host.serialize());
            if let Some(port) = self.port {
                out.push(':');
                out.push_str(&port.to_string());
            }
        } else if !self.has_opaque_path()
            && let Path::List(segments) = &self.path
            && segments.len() > 1
            && segments.first().is_some_and(String::is_empty)
        {
            // Without this, `web+demo:/.//not-a-host/` would serialize
            // as `web+demo://not-a-host/` and parse back as a URL with
            // a host it never had.
            out.push_str("/.");
        }
        out.push_str(&self.serialize_path());
        if let Some(query) = &self.query {
            out.push('?');
            out.push_str(query);
        }
        if !exclude_fragment && let Some(fragment) = &self.fragment {
            out.push('#');
            out.push_str(fragment);
        }
        out
    }
}

impl core::fmt::Display for Url {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.serialize(false))
    }
}

/// The URL parser (Section 4.4).
///
/// It is the basic URL parser, plus the `blob` URL entry step. Izcal
/// has no blob store yet — that belongs with the DOM track's `Blob` —
/// so a `blob:` URL parses to its record with no entry resolved, which
/// is what the standard says to do when resolving fails.
#[must_use]
pub fn parse(input: &str, base: Option<&Url>, encoding: Encoding) -> Option<Url> {
    parse::basic_parse(input, base, encoding, None, None)
}
