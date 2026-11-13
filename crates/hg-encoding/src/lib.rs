//! `hg-encoding`: Izcal's Encoding Standard implementation. L2s, tier P
//! (style spec 6.6, ST-D16).
//!
//! Every decoder and encoder the standard defines, over tables
//! `hg-encoding-build` compiles from the standard's own published
//! `encodings.json` and `indexes.json`. Forty encodings, 228 labels,
//! and the index tables the legacy ones are defined over.
//!
//! It is one crate for the whole browser: `TextDecoder` and
//! `TextEncoder`, `hg-url`'s query encoding, HTML form submission and
//! the parser's own decoding all read it. That is why it sits in **L2s**
//! and is written to **tier P** — the network services and the broker
//! link it through `hg-url`, so it may never panic (G24).
//!
//! `encoding_rs` is excluded under M-1: it was written for Gecko and
//! does browser-engine work (ST-D16). The tables here are WHATWG
//! standards data, which HGBA-2 2.3.2 allows Izcal to ship; the JSON
//! they come from is pinned, fetched and never committed.
//!
//! # What is here
//!
//! - [`Encoding`] and [`Encoding::for_label`] — the standard's "get an
//!   encoding", which is how a `charset` parameter or a `<meta>` becomes
//!   an encoding.
//! - [`Decoder`] and [`Encoder`] — the handlers of Sections 8 to 14,
//!   each written beside the prose it comes from.
//! - [`bom_sniff`] and [`decode`] — the standard's legacy hooks, which
//!   HTML's "determine the character encoding" needs.
//! - [`utf8_decode`], [`utf8_decode_without_bom`] and
//!   [`utf8_decode_without_bom_or_fail`] — the hooks Section 6 asks new
//!   formats to use.
//!
//! # What is not here yet
//!
//! HTML's "prescan a byte stream to determine its encoding" — the
//! `<meta charset>` sniffer — belongs to the HTML side and lands with
//! `hg-html-driver`. This crate carries the pieces it needs:
//! [`Encoding::for_label`] and [`bom_sniff`].

mod decode;
mod encode;
mod generated;
mod queue;

pub use decode::{DecodeError, Decoder, ErrorMode};
pub use encode::Encoder;
pub use generated::encodings::{Encoding, Family};

use generated::encodings::LABELS;

impl Encoding {
    /// The standard's "get an encoding" (Section 4.2): leading and
    /// trailing ASCII whitespace is removed and the rest is matched
    /// ASCII case-insensitively against the label table.
    ///
    /// `None` is the standard's failure. A caller that wants HTML's
    /// behaviour for an unknown label falls back to its own default;
    /// it must not invent an encoding, which is the attack
    /// [`Encoding::Replacement`] exists to stop.
    #[must_use]
    pub fn for_label(label: &str) -> Option<Self> {
        // "ASCII whitespace" is TAB, LF, FF, CR and SPACE.
        let trimmed = label.trim_matches(|c| matches!(c, '\t' | '\n' | '\u{C}' | '\r' | ' '));
        let found = LABELS.binary_search_by(|(l, _)| cmp_ascii_lowercase(l, trimmed));
        LABELS.get(found.ok()?).map(|(_, e)| *e)
    }

    /// The same, for a label that arrived as bytes — a `charset`
    /// parameter out of a header, or a `<meta>` attribute value.
    ///
    /// A label is never anything but ASCII, so bytes that are not UTF-8
    /// are simply not a label.
    #[must_use]
    pub fn for_label_bytes(label: &[u8]) -> Option<Self> {
        Self::for_label(core::str::from_utf8(label).ok()?)
    }

    /// The standard's "get an output encoding" (Section 4.3): an
    /// encoding suitable to encode *to*.
    ///
    /// `replacement`, `UTF-16BE` and `UTF-16LE` become `UTF-8`. Nothing
    /// may encode to them: `replacement` exists to refuse, and
    /// encoding to UTF-16 would produce bytes that an ASCII-reading
    /// consumer would misread.
    #[must_use]
    pub const fn output_encoding(self) -> Self {
        match self.family() {
            Family::Replacement | Family::Utf16be | Family::Utf16le => Self::Utf8,
            Family::Big5
            | Family::EucJp
            | Family::EucKr
            | Family::Gb18030
            | Family::Gbk
            | Family::Iso2022Jp
            | Family::ShiftJis
            | Family::SingleByte
            | Family::Utf8
            | Family::XUserDefined => self,
        }
    }

    /// Whether every ASCII byte decodes to the ASCII code point of the
    /// same value, with no leading byte in play.
    ///
    /// The standard's Section 2 calls the rest "ASCII-incompatible",
    /// and keeps only ISO-2022-JP and UTF-16BE/LE of them because
    /// deployed content requires it. A format that scans for ASCII
    /// delimiters before decoding is only safe for the compatible ones.
    #[must_use]
    pub const fn is_ascii_compatible(self) -> bool {
        !matches!(
            self.family(),
            Family::Utf16be | Family::Utf16le | Family::Replacement | Family::Iso2022Jp
        )
    }

    /// A decoder for it, in replacement mode.
    #[must_use]
    pub const fn new_decoder(self) -> Decoder {
        Decoder::new(self, ErrorMode::Replacement)
    }

    /// An encoder for it, or `None` for the three
    /// [`Encoding::output_encoding`] replaces.
    #[must_use]
    pub const fn new_encoder(self) -> Option<Encoder> {
        Encoder::new(self)
    }
}

/// Compares a label from the table, which is already ASCII lower case,
/// with one that may not be.
fn cmp_ascii_lowercase(table: &str, given: &str) -> core::cmp::Ordering {
    let mut ours = table.bytes();
    let mut theirs = given.bytes().map(|b| b.to_ascii_lowercase());
    loop {
        match (ours.next(), theirs.next()) {
            (None, None) => return core::cmp::Ordering::Equal,
            (None, Some(_)) => return core::cmp::Ordering::Less,
            (Some(_), None) => return core::cmp::Ordering::Greater,
            (Some(a), Some(b)) if a == b => {}
            (Some(a), Some(b)) => return a.cmp(&b),
        }
    }
}

/// The standard's "BOM sniff" (Section 6.1): the encoding a byte order
/// mark names, and how many bytes it occupies.
///
/// A byte order mark outranks a label, and the standard says why: it
/// "has been found to be more accurate in deployed content", even where
/// that contradicts a `Content-Type` header.
#[must_use]
pub fn bom_sniff(bytes: &[u8]) -> Option<(Encoding, usize)> {
    match bytes {
        [0xEF, 0xBB, 0xBF, ..] => Some((Encoding::Utf8, 3)),
        [0xFE, 0xFF, ..] => Some((Encoding::Utf16be, 2)),
        [0xFF, 0xFE, ..] => Some((Encoding::Utf16le, 2)),
        _ => None,
    }
}

/// The standard's "decode" (Section 6.1): decode `bytes` with
/// `fallback`, unless a byte order mark says otherwise.
///
/// Returns the text and the encoding actually used, which is what a
/// caller has to report — HTML's document `characterSet` is the
/// encoding the BOM chose, not the one the header named.
#[must_use]
pub fn decode(fallback: Encoding, bytes: &[u8]) -> (String, Encoding) {
    let (encoding, skip) = bom_sniff(bytes).unwrap_or((fallback, 0));
    let rest = bytes.get(skip..).unwrap_or(&[]);
    (Decoder::decode_all(encoding, rest), encoding)
}

/// The standard's "UTF-8 decode" hook (Section 6), for new formats: a
/// UTF-8 byte order mark is removed, and nothing else is sniffed.
#[must_use]
pub fn utf8_decode(bytes: &[u8]) -> String {
    let skip = usize::from(bytes.starts_with(&[0xEF, 0xBB, 0xBF])).saturating_mul(3);
    Decoder::decode_all(Encoding::Utf8, bytes.get(skip..).unwrap_or(&[]))
}

/// The standard's "UTF-8 decode without BOM" hook, for identifiers and
/// byte sequences inside a format.
#[must_use]
pub fn utf8_decode_without_bom(bytes: &[u8]) -> String {
    Decoder::decode_all(Encoding::Utf8, bytes)
}

/// The standard's "UTF-8 decode without BOM or fail" hook.
///
/// # Errors
/// [`DecodeError`] when `bytes` are not valid UTF-8.
pub fn utf8_decode_without_bom_or_fail(bytes: &[u8]) -> Result<String, DecodeError> {
    let mut out = String::new();
    Decoder::new(Encoding::Utf8, ErrorMode::Fatal).decode(bytes, true, &mut out)?;
    Ok(out)
}
