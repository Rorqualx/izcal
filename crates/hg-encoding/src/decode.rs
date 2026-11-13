//! Every decoder the Encoding Standard defines.
//!
//! Each `fn` below is one of the standard's *handlers*, written in the
//! order its prose writes it, with the section it comes from named. A
//! handler is given the queue and one byte — or end-of-queue — and
//! returns [`Step`]: finished, continue, an error, or one or two code
//! points. [`Decoder::decode`] is the standard's "process a queue"
//! around them.
//!
//! The one thing worth saying about the shape: a handler takes the
//! queue as well as the byte because several of them **restore** bytes
//! to it (Section 2's note on masking explains why — an illegal byte
//! combination must not swallow an ASCII byte, because an attacker who
//! can hide a `"` can change what a JSON document means). Restoring is
//! how the standard makes the swallowed byte come back.

use crate::generated::encodings::{Encoding, Family};
use crate::generated::indexes::{
    BIG5, EUC_KR, GB18030, GB18030_RANGES, JIS0208, JIS0212, NONE16, NONE32, single_byte,
};
use crate::queue::{Pushback, Queue};

/// What a handler returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// The queue is empty and the decoder is not mid-sequence.
    Finished,
    /// More bytes are needed.
    Continue,
    /// A decoder error.
    Error,
    /// One code point.
    One(char),
    /// Two, which only Big5 returns (Section 11.1.1's four pointers).
    Two(char, char),
}

/// What a decoder does with an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorMode {
    /// Emit U+FFFD and carry on. The standard's default, and what every
    /// document load uses.
    #[default]
    Replacement,
    /// Stop. `TextDecoder`'s `fatal` option, and what the standard's
    /// "UTF-8 decode without BOM or fail" hook needs.
    Fatal,
}

/// A decoder stopped because its error mode is [`ErrorMode::Fatal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeError;

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("the bytes are not valid in this encoding")
    }
}

impl core::error::Error for DecodeError {}

/// The ISO-2022-JP decoder's state (Section 12.2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Jis {
    #[default]
    Ascii,
    Roman,
    Katakana,
    Lead,
    Trail,
    EscapeStart,
    Escape,
}

/// Everything the decoders keep between bytes.
///
/// One struct rather than one per encoding: a decoder is built for one
/// encoding and uses only its own fields, and a single struct keeps the
/// handlers readable beside the prose they come from.
#[derive(Debug, Clone, Copy)]
struct State {
    /// UTF-8 code point, bytes seen, bytes needed (Section 8.1.1).
    cp: u32,
    seen: u8,
    needed: u8,
    /// UTF-8 lower and upper boundary.
    lo: u8,
    hi: u8,
    /// The leading byte of a two-byte sequence, and gb18030's second
    /// and third.
    first: u8,
    second: u8,
    third: u8,
    /// EUC-JP jis0212.
    jis0212: bool,
    /// ISO-2022-JP's decoder state, output state, leading byte and
    /// output flag.
    jis: Jis,
    jis_out_state: Jis,
    jis_lead: u8,
    jis_output: bool,
    /// The shared UTF-16 decoder's leading byte and leading surrogate.
    utf16_byte: Option<u8>,
    utf16_lead: Option<u16>,
    /// replacement error returned.
    replaced: bool,
}

impl State {
    const fn new() -> Self {
        Self {
            cp: 0,
            seen: 0,
            needed: 0,
            lo: 0x80,
            hi: 0xBF,
            first: 0,
            second: 0,
            third: 0,
            jis0212: false,
            jis: Jis::Ascii,
            jis_out_state: Jis::Ascii,
            jis_lead: 0,
            jis_output: false,
            utf16_byte: None,
            utf16_lead: None,
            replaced: false,
        }
    }
}

/// A decoder for one encoding.
#[derive(Debug, Clone)]
pub struct Decoder {
    encoding: Encoding,
    mode: ErrorMode,
    state: State,
    back: Pushback,
}

impl Decoder {
    /// A decoder for `encoding`, in `mode`.
    #[must_use]
    pub const fn new(encoding: Encoding, mode: ErrorMode) -> Self {
        Self {
            encoding,
            mode,
            state: State::new(),
            back: Pushback::new(),
        }
    }

    /// The encoding it decodes.
    #[must_use]
    pub const fn encoding(&self) -> Encoding {
        self.encoding
    }

    /// Decodes `input`, appending to `out`.
    ///
    /// `last` says this is the final chunk, which is the standard's
    /// end-of-queue: a decoder left mid-sequence reports one more
    /// error. A streaming caller passes `false` until it has no more
    /// bytes.
    ///
    /// # Errors
    /// [`DecodeError`] when the mode is [`ErrorMode::Fatal`] and the
    /// bytes are not valid. `out` then holds what was decoded before
    /// the error.
    pub fn decode(
        &mut self,
        input: &[u8],
        last: bool,
        out: &mut String,
    ) -> Result<(), DecodeError> {
        let mut q = Queue::new(&mut self.back, input);
        loop {
            let byte = q.next();
            if byte.is_none() && !last {
                return Ok(());
            }
            let step = handle(self.encoding, &mut self.state, &mut q, byte);
            match step {
                Step::Finished => return Ok(()),
                Step::Continue => {}
                Step::One(c) => out.push(c),
                Step::Two(a, b) => {
                    out.push(a);
                    out.push(b);
                }
                Step::Error => {
                    if self.mode == ErrorMode::Fatal {
                        return Err(DecodeError);
                    }
                    out.push('\u{FFFD}');
                }
            }
        }
    }

    /// Decodes `input` whole, in replacement mode.
    #[must_use]
    pub fn decode_all(encoding: Encoding, input: &[u8]) -> String {
        let mut out = String::new();
        let mut d = Self::new(encoding, ErrorMode::Replacement);
        // Replacement mode never errors.
        let _ = d.decode(input, true, &mut out);
        out
    }
}

/// Dispatches to the handler of `encoding`'s family.
fn handle(encoding: Encoding, s: &mut State, q: &mut Queue<'_>, byte: Option<u8>) -> Step {
    match encoding.family() {
        Family::Utf8 => utf8(s, q, byte),
        Family::SingleByte => single_byte_handler(encoding, byte),
        Family::Gbk | Family::Gb18030 => gb18030(s, q, byte),
        Family::Big5 => big5(s, q, byte),
        Family::EucJp => euc_jp(s, q, byte),
        Family::Iso2022Jp => iso_2022_jp(s, q, byte),
        Family::ShiftJis => shift_jis(s, q, byte),
        Family::EucKr => euc_kr(s, q, byte),
        Family::Replacement => replacement(s, byte),
        Family::Utf16be => utf16(s, q, byte, true),
        Family::Utf16le => utf16(s, q, byte, false),
        Family::XUserDefined => x_user_defined(byte),
    }
}

/// A code point from a table value, or an error when the table has none.
fn point(v: u32) -> Step {
    char::from_u32(v).map_or(Step::Error, Step::One)
}

/// Section 8.1.1, the UTF-8 decoder.
fn utf8(s: &mut State, q: &mut Queue<'_>, byte: Option<u8>) -> Step {
    let Some(byte) = byte else {
        if s.needed != 0 {
            s.needed = 0;
            return Step::Error;
        }
        return Step::Finished;
    };
    if s.needed == 0 {
        return match byte {
            0x00..=0x7F => point(u32::from(byte)),
            0xC2..=0xDF => {
                s.needed = 1;
                s.cp = u32::from(byte & 0x1F);
                Step::Continue
            }
            0xE0..=0xEF => {
                if byte == 0xE0 {
                    s.lo = 0xA0;
                }
                if byte == 0xED {
                    s.hi = 0x9F;
                }
                s.needed = 2;
                s.cp = u32::from(byte & 0xF);
                Step::Continue
            }
            0xF0..=0xF4 => {
                if byte == 0xF0 {
                    s.lo = 0x90;
                }
                if byte == 0xF4 {
                    s.hi = 0x8F;
                }
                s.needed = 3;
                s.cp = u32::from(byte & 0x7);
                Step::Continue
            }
            _ => Step::Error,
        };
    }
    if byte < s.lo || byte > s.hi {
        s.cp = 0;
        s.needed = 0;
        s.seen = 0;
        s.lo = 0x80;
        s.hi = 0xBF;
        q.restore(&[byte]);
        return Step::Error;
    }
    s.lo = 0x80;
    s.hi = 0xBF;
    s.cp = (s.cp << 6) | u32::from(byte & 0x3F);
    s.seen = s.seen.saturating_add(1);
    if s.seen != s.needed {
        return Step::Continue;
    }
    let cp = s.cp;
    s.cp = 0;
    s.needed = 0;
    s.seen = 0;
    point(cp)
}

/// Section 9.1, the single-byte decoder.
fn single_byte_handler(encoding: Encoding, byte: Option<u8>) -> Step {
    let Some(byte) = byte else {
        return Step::Finished;
    };
    if byte.is_ascii() {
        return point(u32::from(byte));
    }
    let Some(table) = single_byte(encoding) else {
        return Step::Error;
    };
    let Some(v) = table.get(usize::from(byte.saturating_sub(0x80))).copied() else {
        return Step::Error;
    };
    if v == NONE16 {
        return Step::Error;
    }
    point(u32::from(v))
}

/// `index gb18030 ranges code point`, Section 5.
fn gb18030_ranges_code_point(pointer: u32) -> Option<u32> {
    if (pointer > 39419 && pointer < 189_000) || pointer > 1_237_575 {
        return None;
    }
    if pointer == 7457 {
        return Some(0xE7C7);
    }
    let i = match GB18030_RANGES.binary_search_by_key(&pointer, |(p, _)| *p) {
        Ok(i) => i,
        Err(0) => return None,
        Err(i) => i.checked_sub(1)?,
    };
    let (offset, cp_offset) = GB18030_RANGES.get(i).copied()?;
    cp_offset.checked_add(pointer.checked_sub(offset)?)
}

/// Section 10.2.1, the gb18030 decoder. GBK's decoder is this one
/// (Section 10.1.1).
fn gb18030(s: &mut State, q: &mut Queue<'_>, byte: Option<u8>) -> Step {
    let Some(byte) = byte else {
        if s.first == 0 && s.second == 0 && s.third == 0 {
            return Step::Finished;
        }
        s.first = 0;
        s.second = 0;
        s.third = 0;
        return Step::Error;
    };
    if s.third != 0 {
        if !(0x30..=0x39).contains(&byte) {
            q.restore(&[s.second, s.third, byte]);
            s.first = 0;
            s.second = 0;
            s.third = 0;
            return Step::Error;
        }
        let pointer = u32::from(s.first.saturating_sub(0x81))
            .saturating_mul(10 * 126 * 10)
            .saturating_add(u32::from(s.second.saturating_sub(0x30)).saturating_mul(10 * 126))
            .saturating_add(u32::from(s.third.saturating_sub(0x81)).saturating_mul(10))
            .saturating_add(u32::from(byte.saturating_sub(0x30)));
        s.first = 0;
        s.second = 0;
        s.third = 0;
        return gb18030_ranges_code_point(pointer).map_or(Step::Error, point);
    }
    if s.second != 0 {
        if (0x81..=0xFE).contains(&byte) {
            s.third = byte;
            return Step::Continue;
        }
        q.restore(&[s.second, byte]);
        s.first = 0;
        s.second = 0;
        return Step::Error;
    }
    if s.first != 0 {
        if (0x30..=0x39).contains(&byte) {
            s.second = byte;
            return Step::Continue;
        }
        let leading = s.first;
        s.first = 0;
        let offset = if byte < 0x7F { 0x40 } else { 0x41 };
        let pointer = if (0x40..=0x7E).contains(&byte) || (0x80..=0xFE).contains(&byte) {
            Some(
                u32::from(leading.saturating_sub(0x81))
                    .saturating_mul(190)
                    .saturating_add(u32::from(byte.saturating_sub(offset))),
            )
        } else {
            None
        };
        let cp = pointer
            .and_then(|p| usize::try_from(p).ok())
            .and_then(|p| GB18030.get(p).copied())
            .filter(|v| *v != NONE16);
        if let Some(v) = cp {
            return point(u32::from(v));
        }
        if byte.is_ascii() {
            q.restore(&[byte]);
        }
        return Step::Error;
    }
    if byte.is_ascii() {
        return point(u32::from(byte));
    }
    if byte == 0x80 {
        return point(0x20AC);
    }
    if (0x81..=0xFE).contains(&byte) {
        s.first = byte;
        return Step::Continue;
    }
    Step::Error
}

/// The four Big5 pointers whose entry is two code points, Section
/// 11.1.1. An index holds one code point per pointer, so the standard
/// writes these four out instead.
const BIG5_PAIRS: [(u32, char, char); 4] = [
    (1133, '\u{00CA}', '\u{0304}'),
    (1135, '\u{00CA}', '\u{030C}'),
    (1164, '\u{00EA}', '\u{0304}'),
    (1166, '\u{00EA}', '\u{030C}'),
];

/// Section 11.1.1, the Big5 decoder.
fn big5(s: &mut State, q: &mut Queue<'_>, byte: Option<u8>) -> Step {
    let Some(byte) = byte else {
        if s.first == 0 {
            return Step::Finished;
        }
        s.first = 0;
        return Step::Error;
    };
    if s.first != 0 {
        let leading = s.first;
        s.first = 0;
        let offset = if byte < 0x7F { 0x40 } else { 0x62 };
        let pointer = if (0x40..=0x7E).contains(&byte) || (0xA1..=0xFE).contains(&byte) {
            Some(
                u32::from(leading.saturating_sub(0x81))
                    .saturating_mul(157)
                    .saturating_add(u32::from(byte.saturating_sub(offset))),
            )
        } else {
            None
        };
        if let Some(p) = pointer
            && let Some((_, a, b)) = BIG5_PAIRS.iter().find(|(k, _, _)| *k == p)
        {
            return Step::Two(*a, *b);
        }
        let cp = pointer
            .and_then(|p| usize::try_from(p).ok())
            .and_then(|p| BIG5.get(p).copied())
            .filter(|v| *v != NONE32);
        if let Some(v) = cp {
            return point(v);
        }
        if byte.is_ascii() {
            q.restore(&[byte]);
        }
        return Step::Error;
    }
    if byte.is_ascii() {
        return point(u32::from(byte));
    }
    if (0x81..=0xFE).contains(&byte) {
        s.first = byte;
        return Step::Continue;
    }
    Step::Error
}

/// Section 12.1.1, the EUC-JP decoder.
fn euc_jp(s: &mut State, q: &mut Queue<'_>, byte: Option<u8>) -> Step {
    let Some(byte) = byte else {
        if s.first == 0 {
            return Step::Finished;
        }
        s.first = 0;
        return Step::Error;
    };
    if s.first == 0x8E && (0xA1..=0xDF).contains(&byte) {
        s.first = 0;
        return point(
            0xFF61_u32
                .saturating_sub(0xA1)
                .saturating_add(u32::from(byte)),
        );
    }
    if s.first == 0x8F && (0xA1..=0xFE).contains(&byte) {
        s.jis0212 = true;
        s.first = byte;
        return Step::Continue;
    }
    if s.first != 0 {
        let leading = s.first;
        s.first = 0;
        let cp = if (0xA1..=0xFE).contains(&leading) && (0xA1..=0xFE).contains(&byte) {
            let pointer = usize::from(leading.saturating_sub(0xA1))
                .saturating_mul(94)
                .saturating_add(usize::from(byte.saturating_sub(0xA1)));
            let table: &[u16] = if s.jis0212 { &JIS0212 } else { &JIS0208 };
            table.get(pointer).copied().filter(|v| *v != NONE16)
        } else {
            None
        };
        s.jis0212 = false;
        if let Some(v) = cp {
            return point(u32::from(v));
        }
        if byte.is_ascii() {
            q.restore(&[byte]);
        }
        return Step::Error;
    }
    if byte.is_ascii() {
        return point(u32::from(byte));
    }
    if byte == 0x8E || byte == 0x8F || (0xA1..=0xFE).contains(&byte) {
        s.first = byte;
        return Step::Continue;
    }
    Step::Error
}

/// Section 12.2.1, the ISO-2022-JP decoder.
fn iso_2022_jp(s: &mut State, q: &mut Queue<'_>, byte: Option<u8>) -> Step {
    match s.jis {
        Jis::Ascii => match byte {
            Some(0x1B) => {
                s.jis = Jis::EscapeStart;
                Step::Continue
            }
            Some(b) if b <= 0x7F && b != 0x0E && b != 0x0F => {
                s.jis_output = false;
                point(u32::from(b))
            }
            None => Step::Finished,
            Some(_) => {
                s.jis_output = false;
                Step::Error
            }
        },
        Jis::Roman => match byte {
            Some(0x1B) => {
                s.jis = Jis::EscapeStart;
                Step::Continue
            }
            Some(0x5C) => {
                s.jis_output = false;
                point(0x00A5)
            }
            Some(0x7E) => {
                s.jis_output = false;
                point(0x203E)
            }
            Some(b) if b <= 0x7F && b != 0x0E && b != 0x0F && b != 0x5C && b != 0x7E => {
                s.jis_output = false;
                point(u32::from(b))
            }
            None => Step::Finished,
            Some(_) => {
                s.jis_output = false;
                Step::Error
            }
        },
        Jis::Katakana => match byte {
            Some(0x1B) => {
                s.jis = Jis::EscapeStart;
                Step::Continue
            }
            Some(b) if (0x21..=0x5F).contains(&b) => {
                s.jis_output = false;
                point(0xFF61_u32.saturating_sub(0x21).saturating_add(u32::from(b)))
            }
            None => Step::Finished,
            Some(_) => {
                s.jis_output = false;
                Step::Error
            }
        },
        Jis::Lead => match byte {
            Some(0x1B) => {
                s.jis = Jis::EscapeStart;
                Step::Continue
            }
            Some(b) if (0x21..=0x7E).contains(&b) => {
                s.jis_output = false;
                s.jis_lead = b;
                s.jis = Jis::Trail;
                Step::Continue
            }
            None => Step::Finished,
            Some(_) => {
                s.jis_output = false;
                Step::Error
            }
        },
        Jis::Trail => match byte {
            Some(0x1B) => {
                s.jis = Jis::EscapeStart;
                Step::Error
            }
            Some(b) if (0x21..=0x7E).contains(&b) => {
                s.jis = Jis::Lead;
                let pointer = usize::from(s.jis_lead.saturating_sub(0x21))
                    .saturating_mul(94)
                    .saturating_add(usize::from(b.saturating_sub(0x21)));
                JIS0208
                    .get(pointer)
                    .copied()
                    .filter(|v| *v != NONE16)
                    .map_or(Step::Error, |v| point(u32::from(v)))
            }
            None => {
                s.jis = Jis::Lead;
                Step::Error
            }
            Some(_) => {
                s.jis = Jis::Lead;
                Step::Error
            }
        },
        Jis::EscapeStart => {
            if byte == Some(0x24) || byte == Some(0x28) {
                s.jis_lead = byte.unwrap_or(0);
                s.jis = Jis::Escape;
                return Step::Continue;
            }
            if let Some(b) = byte {
                q.restore(&[b]);
            }
            s.jis_output = false;
            s.jis = s.jis_out_state;
            Step::Error
        }
        Jis::Escape => {
            let leading = s.jis_lead;
            s.jis_lead = 0;
            let state = match (leading, byte) {
                (0x28, Some(0x42)) => Some(Jis::Ascii),
                (0x28, Some(0x4A)) => Some(Jis::Roman),
                (0x28, Some(0x49)) => Some(Jis::Katakana),
                (0x24, Some(0x40 | 0x42)) => Some(Jis::Lead),
                _ => None,
            };
            if let Some(st) = state {
                s.jis = st;
                s.jis_out_state = st;
                let output = s.jis_output;
                s.jis_output = true;
                return if output { Step::Error } else { Step::Continue };
            }
            match byte {
                None => q.restore(&[leading]),
                Some(b) => q.restore(&[leading, b]),
            }
            s.jis_output = false;
            s.jis = s.jis_out_state;
            Step::Error
        }
    }
}

/// Section 12.3.1, the Shift_JIS decoder.
fn shift_jis(s: &mut State, q: &mut Queue<'_>, byte: Option<u8>) -> Step {
    let Some(byte) = byte else {
        if s.first == 0 {
            return Step::Finished;
        }
        s.first = 0;
        return Step::Error;
    };
    if s.first != 0 {
        let leading = s.first;
        s.first = 0;
        let offset = if byte < 0x7F { 0x40 } else { 0x41 };
        let leading_offset = if leading < 0xA0 { 0x81 } else { 0xC1 };
        let pointer = if (0x40..=0x7E).contains(&byte) || (0x80..=0xFC).contains(&byte) {
            Some(
                u32::from(leading.saturating_sub(leading_offset))
                    .saturating_mul(188)
                    .saturating_add(u32::from(byte.saturating_sub(offset))),
            )
        } else {
            None
        };
        // The EUDC range, interoperable legacy from Windows.
        if let Some(p) = pointer
            && (8836..=10715).contains(&p)
        {
            return point(0xE000_u32.saturating_sub(8836).saturating_add(p));
        }
        let cp = pointer
            .and_then(|p| usize::try_from(p).ok())
            .and_then(|p| JIS0208.get(p).copied())
            .filter(|v| *v != NONE16);
        if let Some(v) = cp {
            return point(u32::from(v));
        }
        if byte.is_ascii() {
            q.restore(&[byte]);
        }
        return Step::Error;
    }
    if byte.is_ascii() || byte == 0x80 {
        return point(u32::from(byte));
    }
    if (0xA1..=0xDF).contains(&byte) {
        return point(
            0xFF61_u32
                .saturating_sub(0xA1)
                .saturating_add(u32::from(byte)),
        );
    }
    if (0x81..=0x9F).contains(&byte) || (0xE0..=0xFC).contains(&byte) {
        s.first = byte;
        return Step::Continue;
    }
    Step::Error
}

/// Section 13.1.1, the EUC-KR decoder.
fn euc_kr(s: &mut State, q: &mut Queue<'_>, byte: Option<u8>) -> Step {
    let Some(byte) = byte else {
        if s.first == 0 {
            return Step::Finished;
        }
        s.first = 0;
        return Step::Error;
    };
    if s.first != 0 {
        let leading = s.first;
        s.first = 0;
        let cp = if (0x41..=0xFE).contains(&byte) {
            let pointer = usize::from(leading.saturating_sub(0x81))
                .saturating_mul(190)
                .saturating_add(usize::from(byte.saturating_sub(0x41)));
            EUC_KR.get(pointer).copied().filter(|v| *v != NONE16)
        } else {
            None
        };
        if let Some(v) = cp {
            return point(u32::from(v));
        }
        if byte.is_ascii() {
            q.restore(&[byte]);
        }
        return Step::Error;
    }
    if byte.is_ascii() {
        return point(u32::from(byte));
    }
    if (0x81..=0xFE).contains(&byte) {
        s.first = byte;
        return Step::Continue;
    }
    Step::Error
}

/// Section 14.1.1, the replacement decoder.
///
/// It exists to stop an attack: a label the server understands and the
/// client does not must not fall back to something that decodes.
fn replacement(s: &mut State, byte: Option<u8>) -> Step {
    if byte.is_none() {
        return Step::Finished;
    }
    if s.replaced {
        return Step::Finished;
    }
    s.replaced = true;
    Step::Error
}

/// Section 14.2.1, the shared UTF-16 decoder.
fn utf16(s: &mut State, q: &mut Queue<'_>, byte: Option<u8>, be: bool) -> Step {
    let Some(byte) = byte else {
        if s.utf16_byte.is_some() || s.utf16_lead.is_some() {
            s.utf16_byte = None;
            s.utf16_lead = None;
            return Step::Error;
        }
        return Step::Finished;
    };
    let Some(lead_byte) = s.utf16_byte else {
        s.utf16_byte = Some(byte);
        return Step::Continue;
    };
    let unit = if be {
        (u16::from(lead_byte) << 8) | u16::from(byte)
    } else {
        (u16::from(byte) << 8) | u16::from(lead_byte)
    };
    s.utf16_byte = None;
    if let Some(lead) = s.utf16_lead {
        s.utf16_lead = None;
        if (0xDC00..=0xDFFF).contains(&unit) {
            // "scalar value from surrogates".
            let v = 0x1_0000_u32
                .saturating_add(u32::from(lead.saturating_sub(0xD800)) << 10)
                .saturating_add(u32::from(unit.saturating_sub(0xDC00)));
            return point(v);
        }
        let hi = u8::try_from(unit >> 8).unwrap_or(0);
        let lo = u8::try_from(unit & 0x00FF).unwrap_or(0);
        if be {
            q.restore(&[hi, lo]);
        } else {
            q.restore(&[lo, hi]);
        }
        return Step::Error;
    }
    if (0xD800..=0xDBFF).contains(&unit) {
        s.utf16_lead = Some(unit);
        return Step::Continue;
    }
    if (0xDC00..=0xDFFF).contains(&unit) {
        return Step::Error;
    }
    point(u32::from(unit))
}

/// Section 14.5.1, the x-user-defined decoder.
fn x_user_defined(byte: Option<u8>) -> Step {
    let Some(byte) = byte else {
        return Step::Finished;
    };
    if byte.is_ascii() {
        return point(u32::from(byte));
    }
    point(
        0xF780_u32
            .saturating_add(u32::from(byte))
            .saturating_sub(0x80),
    )
}
