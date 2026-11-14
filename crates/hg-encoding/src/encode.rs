//! Every encoder the Encoding Standard defines.
//!
//! As with the decoders, each `fn` is one of the standard's handlers,
//! in the order its prose writes it.
//!
//! Three encodings have no encoder: `replacement`, `UTF-16BE` and
//! `UTF-16LE`. That is not an omission — the standard's "get an output
//! encoding" (Section 4.3) maps all three to UTF-8, so nothing may
//! encode to them. [`Encoding::output_encoding`] is that algorithm and
//! [`Encoder::new`] refuses the three.
//!
//! **An unmappable code point is the caller's business.** The standard
//! returns "error with codePoint" and leaves what to do to whoever
//! asked: HTML form submission writes `&#NNN;`, URL percent-encoding
//! writes `%XX`. So [`Encoder::encode`] hands each one back rather than
//! choosing, which is also why there is no "replacement" error mode
//! here to match the decoder's.

use crate::generated::encodings::{Encoding, Family};
use crate::generated::indexes::{
    BIG5_ENCODE, EUC_KR_ENCODE, GB18030_ENCODE, ISO2022JP_KATAKANA, JIS0208_ENCODE, NONE16,
    SHIFT_JIS_ENCODE, single_byte_encode,
};

/// The ISO-2022-JP encoder's state (Section 12.2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Jis {
    #[default]
    Ascii,
    Roman,
    Jis0208,
}

/// An encoder for one encoding.
#[derive(Debug, Clone)]
pub struct Encoder {
    encoding: Encoding,
    /// gb18030's `is GBK`.
    is_gbk: bool,
    /// ISO-2022-JP's encoder state.
    jis: Jis,
}

/// What one code point encodes to.
enum Out {
    /// Bytes, at most four.
    Bytes([u8; 4], usize),
    /// The encoding cannot represent it.
    Unmappable(char),
    /// Emit these bytes and encode the same code point again, which is
    /// the standard's "restore codePoint to ioQueue" for ISO-2022-JP.
    Again([u8; 4], usize),
}

fn one(b: u8) -> Out {
    Out::Bytes([b, 0, 0, 0], 1)
}

fn two(a: u8, b: u8) -> Out {
    Out::Bytes([a, b, 0, 0], 2)
}

impl Encoder {
    /// An encoder for `encoding`, or `None` for the three the standard
    /// forbids encoding to.
    #[must_use]
    pub const fn new(encoding: Encoding) -> Option<Self> {
        match encoding.family() {
            Family::Replacement | Family::Utf16be | Family::Utf16le => None,
            Family::Big5
            | Family::EucJp
            | Family::EucKr
            | Family::Gb18030
            | Family::Gbk
            | Family::Iso2022Jp
            | Family::ShiftJis
            | Family::SingleByte
            | Family::Utf8
            | Family::XUserDefined => Some(Self {
                encoding,
                is_gbk: matches!(encoding.family(), Family::Gbk),
                jis: Jis::Ascii,
            }),
        }
    }

    /// The encoding it writes.
    #[must_use]
    pub const fn encoding(&self) -> Encoding {
        self.encoding
    }

    /// Encodes `input`, appending bytes to `out`.
    ///
    /// Each code point the encoding cannot represent is passed to
    /// `unmappable` with `out`, so the caller's own standard decides:
    /// HTML form submission appends `&#NNN;`, URL percent-encoding
    /// appends `%XX`.
    ///
    /// `last` says this is the final chunk. Only ISO-2022-JP does
    /// anything with it — it returns to ASCII, which is the one case
    /// where an encoder writes bytes of its own at the end.
    pub fn encode(
        &mut self,
        input: &str,
        last: bool,
        out: &mut Vec<u8>,
        unmappable: &mut dyn FnMut(char, &mut Vec<u8>),
    ) {
        for c in input.chars() {
            // `Again` asks for the same code point once more, after the
            // escape sequence just written. ISO-2022-JP is the only
            // encoder that asks, and it asks at most twice in a row: a
            // state change, then the character. Three rounds leaves a
            // spare, and running out is treated as unmappable rather
            // than dropping the code point, so a bug in the state
            // machine would show as mangled output and not as silence.
            let mut rounds = 0_u8;
            loop {
                rounds = rounds.saturating_add(1);
                if rounds > 3 {
                    unmappable(c, out);
                    break;
                }
                match self.step(c) {
                    Out::Bytes(bytes, n) => {
                        out.extend(bytes.iter().take(n));
                        break;
                    }
                    Out::Unmappable(bad) => {
                        unmappable(bad, out);
                        break;
                    }
                    Out::Again(bytes, n) => out.extend(bytes.iter().take(n)),
                }
            }
        }
        if last && self.jis != Jis::Ascii {
            self.jis = Jis::Ascii;
            out.extend([0x1B, 0x28, 0x42]);
        }
    }

    /// Encodes `input` whole, replacing what it cannot represent the
    /// way HTML's form submission does: `&#NNN;`.
    #[must_use]
    pub fn encode_all(encoding: Encoding, input: &str) -> Option<Vec<u8>> {
        let mut e = Self::new(encoding)?;
        let mut out = Vec::new();
        e.encode(input, true, &mut out, &mut |c, out| {
            out.extend(format!("&#{};", u32::from(c)).as_bytes());
        });
        Some(out)
    }

    fn step(&mut self, c: char) -> Out {
        match self.encoding.family() {
            Family::Utf8 => utf8(c),
            Family::SingleByte => single_byte(self.encoding, c),
            Family::Gbk | Family::Gb18030 => gb18030(self.is_gbk, c),
            Family::Big5 => big5(c),
            Family::EucJp => euc_jp(c),
            Family::Iso2022Jp => iso_2022_jp(&mut self.jis, c),
            Family::ShiftJis => shift_jis(c),
            Family::EucKr => euc_kr(c),
            Family::XUserDefined => x_user_defined(c),
            // Refused by `new`, so unreachable through the public API;
            // answering rather than panicking keeps this crate's "never
            // panic" rule (tier P) without an exception.
            Family::Replacement | Family::Utf16be | Family::Utf16le => Out::Unmappable(c),
        }
    }
}

/// `index pointer`, over a generated encode table.
fn pointer16(table: &[(u16, u16)], c: char) -> Option<u16> {
    let cp = u16::try_from(u32::from(c)).ok()?;
    let i = table.binary_search_by_key(&cp, |(k, _)| *k).ok()?;
    table.get(i).map(|(_, p)| *p)
}

/// The same, for the one table whose keys reach beyond U+FFFF.
fn pointer32(table: &[(u32, u16)], c: char) -> Option<u16> {
    let cp = u32::from(c);
    let i = table.binary_search_by_key(&cp, |(k, _)| *k).ok()?;
    table.get(i).map(|(_, p)| *p)
}

/// Section 8.1.2, the UTF-8 encoder.
fn utf8(c: char) -> Out {
    let mut bytes = [0_u8; 4];
    let n = c.encode_utf8(&mut bytes).len();
    Out::Bytes(bytes, n)
}

/// Section 9.2, the single-byte encoder.
fn single_byte(encoding: Encoding, c: char) -> Out {
    if c.is_ascii() {
        return one(u8::try_from(u32::from(c)).unwrap_or(0));
    }
    let Some(table) = single_byte_encode(encoding) else {
        return Out::Unmappable(c);
    };
    match pointer16(table, c) {
        Some(p) => u8::try_from(u32::from(p).saturating_add(0x80)).map_or(Out::Unmappable(c), one),
        None => Out::Unmappable(c),
    }
}

/// `index gb18030 ranges pointer`, Section 5.
fn gb18030_ranges_pointer(c: char) -> Option<u32> {
    use crate::generated::indexes::GB18030_RANGES;
    let cp = u32::from(c);
    if cp == 0xE7C7 {
        return Some(7457);
    }
    let i = match GB18030_RANGES.binary_search_by_key(&cp, |(_, k)| *k) {
        Ok(i) => i,
        Err(0) => return None,
        Err(i) => i.checked_sub(1)?,
    };
    let (pointer_offset, offset) = GB18030_RANGES.get(i).copied()?;
    pointer_offset.checked_add(cp.checked_sub(offset)?)
}

/// The gb18030 encoder's asymmetric table, Section 10.2.2. It keeps
/// these eighteen code points encoding the way GB18030-2005 encoded
/// them, which the index alone would not.
const GB18030_ASYMMETRIC: [(u32, u8, u8); 18] = [
    (0xE78D, 0xA6, 0xD9),
    (0xE78E, 0xA6, 0xDA),
    (0xE78F, 0xA6, 0xDB),
    (0xE790, 0xA6, 0xDC),
    (0xE791, 0xA6, 0xDD),
    (0xE792, 0xA6, 0xDE),
    (0xE793, 0xA6, 0xDF),
    (0xE794, 0xA6, 0xEC),
    (0xE795, 0xA6, 0xED),
    (0xE796, 0xA6, 0xF3),
    (0xE81E, 0xFE, 0x59),
    (0xE826, 0xFE, 0x61),
    (0xE82B, 0xFE, 0x66),
    (0xE82C, 0xFE, 0x67),
    (0xE832, 0xFE, 0x6D),
    (0xE843, 0xFE, 0x7E),
    (0xE854, 0xFE, 0x90),
    (0xE864, 0xFE, 0xA0),
];

/// Section 10.2.2, the gb18030 encoder. GBK's is this one with
/// `is GBK` true (Section 10.1.2).
fn gb18030(is_gbk: bool, c: char) -> Out {
    if c.is_ascii() {
        return one(u8::try_from(u32::from(c)).unwrap_or(0));
    }
    // Index gb18030 maps 0xA3 0xA0 to U+3000 rather than U+E5E5 for
    // compatibility, so U+E5E5 cannot round-trip.
    if c == '\u{E5E5}' {
        return Out::Unmappable(c);
    }
    if is_gbk && c == '\u{20AC}' {
        return one(0x80);
    }
    if let Some((_, a, b)) = GB18030_ASYMMETRIC
        .iter()
        .find(|(k, _, _)| *k == u32::from(c))
    {
        return two(*a, *b);
    }
    if let Some(p) = pointer16(&GB18030_ENCODE, c) {
        let p = u32::from(p);
        let leading = p.wrapping_div(190).saturating_add(0x81);
        let trailing = p.wrapping_rem(190);
        let offset = if trailing < 0x3F { 0x40 } else { 0x41 };
        return match (
            u8::try_from(leading),
            u8::try_from(trailing.saturating_add(offset)),
        ) {
            (Ok(a), Ok(b)) => two(a, b),
            _ => Out::Unmappable(c),
        };
    }
    if is_gbk {
        return Out::Unmappable(c);
    }
    let Some(mut p) = gb18030_ranges_pointer(c) else {
        return Out::Unmappable(c);
    };
    let byte1 = p.wrapping_div(10 * 126 * 10);
    p = p.wrapping_rem(10 * 126 * 10);
    let byte2 = p.wrapping_div(10 * 126);
    p = p.wrapping_rem(10 * 126);
    let byte3 = p.wrapping_div(10);
    let byte4 = p.wrapping_rem(10);
    let bytes = [
        u8::try_from(byte1.saturating_add(0x81)),
        u8::try_from(byte2.saturating_add(0x30)),
        u8::try_from(byte3.saturating_add(0x81)),
        u8::try_from(byte4.saturating_add(0x30)),
    ];
    match bytes {
        [Ok(a), Ok(b), Ok(d), Ok(e)] => Out::Bytes([a, b, d, e], 4),
        _ => Out::Unmappable(c),
    }
}

/// Section 11.1.2, the Big5 encoder.
fn big5(c: char) -> Out {
    if c.is_ascii() {
        return one(u8::try_from(u32::from(c)).unwrap_or(0));
    }
    let Some(p) = pointer32(&BIG5_ENCODE, c) else {
        return Out::Unmappable(c);
    };
    let p = u32::from(p);
    let leading = p.wrapping_div(157).saturating_add(0x81);
    let trailing = p.wrapping_rem(157);
    let offset = if trailing < 0x3F { 0x40 } else { 0x62 };
    match (
        u8::try_from(leading),
        u8::try_from(trailing.saturating_add(offset)),
    ) {
        (Ok(a), Ok(b)) => two(a, b),
        _ => Out::Unmappable(c),
    }
}

/// Section 12.1.2, the EUC-JP encoder.
fn euc_jp(c: char) -> Out {
    if c.is_ascii() {
        return one(u8::try_from(u32::from(c)).unwrap_or(0));
    }
    if c == '\u{00A5}' {
        return one(0x5C);
    }
    if c == '\u{203E}' {
        return one(0x7E);
    }
    let cp = u32::from(c);
    if (0xFF61..=0xFF9F).contains(&cp) {
        return u8::try_from(cp.saturating_sub(0xFF61).saturating_add(0xA1))
            .map_or(Out::Unmappable(c), |b| two(0x8E, b));
    }
    let c = if c == '\u{2212}' { '\u{FF0D}' } else { c };
    let Some(p) = pointer16(&JIS0208_ENCODE, c) else {
        return Out::Unmappable(c);
    };
    let p = u32::from(p);
    match (
        u8::try_from(p.wrapping_div(94).saturating_add(0xA1)),
        u8::try_from(p.wrapping_rem(94).saturating_add(0xA1)),
    ) {
        (Ok(a), Ok(b)) => two(a, b),
        _ => Out::Unmappable(c),
    }
}

/// Section 12.2.2, the ISO-2022-JP encoder.
///
/// The one encoder that writes bytes of its own: the escape sequences
/// that change mode. The standard notes that concatenating two of its
/// outputs can produce something its own decoder rejects, which is why
/// the state lives on the encoder and not on each call.
fn iso_2022_jp(state: &mut Jis, c: char) -> Out {
    // U+000E, U+000F and U+001B are refused as U+FFFD rather than as
    // themselves, "to prevent attacks": passing them through would let
    // content inject a mode change.
    if matches!(state, Jis::Ascii | Jis::Roman) && matches!(c, '\u{E}' | '\u{F}' | '\u{1B}') {
        return Out::Unmappable('\u{FFFD}');
    }
    if *state == Jis::Ascii && c.is_ascii() {
        return one(u8::try_from(u32::from(c)).unwrap_or(0));
    }
    if *state == Jis::Roman
        && ((c.is_ascii() && c != '\\' && c != '~') || c == '\u{00A5}' || c == '\u{203E}')
    {
        if c.is_ascii() {
            return one(u8::try_from(u32::from(c)).unwrap_or(0));
        }
        if c == '\u{00A5}' {
            return one(0x5C);
        }
        return one(0x7E);
    }
    if c.is_ascii() && *state != Jis::Ascii {
        *state = Jis::Ascii;
        return Out::Again([0x1B, 0x28, 0x42, 0], 3);
    }
    if (c == '\u{00A5}' || c == '\u{203E}') && *state != Jis::Roman {
        *state = Jis::Roman;
        return Out::Again([0x1B, 0x28, 0x4A, 0], 3);
    }
    let c = if c == '\u{2212}' { '\u{FF0D}' } else { c };
    let cp = u32::from(c);
    // Halfwidth katakana map to their fullwidth forms through the
    // index the standard keeps for this encoder alone.
    let c = if (0xFF61..=0xFF9F).contains(&cp) {
        let i = usize::try_from(cp.saturating_sub(0xFF61)).unwrap_or(usize::MAX);
        match ISO2022JP_KATAKANA
            .get(i)
            .copied()
            .filter(|v| *v != NONE16)
            .and_then(|v| char::from_u32(u32::from(v)))
        {
            Some(mapped) => mapped,
            None => return Out::Unmappable(c),
        }
    } else {
        c
    };
    let Some(p) = pointer16(&JIS0208_ENCODE, c) else {
        if *state == Jis::Jis0208 {
            *state = Jis::Ascii;
            return Out::Again([0x1B, 0x28, 0x42, 0], 3);
        }
        return Out::Unmappable(c);
    };
    if *state != Jis::Jis0208 {
        *state = Jis::Jis0208;
        return Out::Again([0x1B, 0x24, 0x42, 0], 3);
    }
    let p = u32::from(p);
    match (
        u8::try_from(p.wrapping_div(94).saturating_add(0x21)),
        u8::try_from(p.wrapping_rem(94).saturating_add(0x21)),
    ) {
        (Ok(a), Ok(b)) => two(a, b),
        _ => Out::Unmappable(c),
    }
}

/// Section 12.3.2, the Shift_JIS encoder.
fn shift_jis(c: char) -> Out {
    let cp = u32::from(c);
    if c.is_ascii() || cp == 0x80 {
        return u8::try_from(cp).map_or(Out::Unmappable(c), one);
    }
    if c == '\u{00A5}' {
        return one(0x5C);
    }
    if c == '\u{203E}' {
        return one(0x7E);
    }
    if (0xFF61..=0xFF9F).contains(&cp) {
        return u8::try_from(cp.saturating_sub(0xFF61).saturating_add(0xA1))
            .map_or(Out::Unmappable(c), one);
    }
    let c = if c == '\u{2212}' { '\u{FF0D}' } else { c };
    let Some(p) = pointer16(&SHIFT_JIS_ENCODE, c) else {
        return Out::Unmappable(c);
    };
    let p = u32::from(p);
    let leading = p.wrapping_div(188);
    let leading_offset = if leading < 0x1F { 0x81 } else { 0xC1 };
    let trailing = p.wrapping_rem(188);
    let offset = if trailing < 0x3F { 0x40 } else { 0x41 };
    match (
        u8::try_from(leading.saturating_add(leading_offset)),
        u8::try_from(trailing.saturating_add(offset)),
    ) {
        (Ok(a), Ok(b)) => two(a, b),
        _ => Out::Unmappable(c),
    }
}

/// Section 13.1.2, the EUC-KR encoder.
fn euc_kr(c: char) -> Out {
    if c.is_ascii() {
        return one(u8::try_from(u32::from(c)).unwrap_or(0));
    }
    let Some(p) = pointer16(&EUC_KR_ENCODE, c) else {
        return Out::Unmappable(c);
    };
    let p = u32::from(p);
    match (
        u8::try_from(p.wrapping_div(190).saturating_add(0x81)),
        u8::try_from(p.wrapping_rem(190).saturating_add(0x41)),
    ) {
        (Ok(a), Ok(b)) => two(a, b),
        _ => Out::Unmappable(c),
    }
}

/// Section 14.5.2, the x-user-defined encoder.
fn x_user_defined(c: char) -> Out {
    let cp = u32::from(c);
    if c.is_ascii() {
        return u8::try_from(cp).map_or(Out::Unmappable(c), one);
    }
    if (0xF780..=0xF7FF).contains(&cp) {
        return u8::try_from(cp.saturating_sub(0xF780).saturating_add(0x80))
            .map_or(Out::Unmappable(c), one);
    }
    Out::Unmappable(c)
}
