//! Reading WHATWG's `entities.json`.
//!
//! The file is one flat object: `"&amp;": {"codepoints": [38],
//! "characters": "&"}`. Only the name and `characters` are used, and
//! the name is stored without its leading ampersand, which the
//! tokenizer has consumed before the named character reference state.
//!
//! The scanner below is this crate's own, as every Izcal generator's
//! inputs are: `hg-htmlgen` has no dependencies.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What the data set is called in `tools/test-data.toml`.
pub const DATA_SET: &str = "html-entities";

/// The pin recorded there. The published file has no revision of its
/// own, so it is pinned by its own content hash.
pub const PIN: &str = "d741d877ac77";

/// The pinned `entities.json`, in the cache `tools/fetch-test-data.sh`
/// writes to: `$IZCAL_TEST_DATA`, or `~/.cache/izcal/test-data`.
///
/// # Errors
/// A message when neither `$IZCAL_TEST_DATA` nor `$HOME` is set.
pub fn path_from_env() -> Result<PathBuf, String> {
    let cache = match std::env::var_os("IZCAL_TEST_DATA") {
        Some(v) => PathBuf::from(v),
        None => {
            let home = std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .ok_or("neither IZCAL_TEST_DATA nor HOME is set")?;
            PathBuf::from(home).join(".cache/izcal/test-data")
        }
    };
    Ok(cache
        .join(format!("{DATA_SET}-{PIN}"))
        .join("entities.json"))
}

/// Reads the table: identifier without its ampersand, to the one or two
/// characters it stands for.
///
/// # Errors
/// A message naming the file when it cannot be read or is not the shape
/// above.
pub fn read(path: &Path) -> Result<BTreeMap<String, String>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parses the text of `entities.json`.
///
/// # Errors
/// A message when the text is not the shape above.
pub fn parse(text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut p = Scan {
        s: text.as_bytes(),
        at: 0,
    };
    let mut out = BTreeMap::new();
    p.ws();
    p.byte(b'{')?;
    p.ws();
    loop {
        p.ws();
        let name = p.string()?;
        p.ws();
        p.byte(b':')?;
        p.ws();
        let characters = p.entry()?;
        let key = name
            .strip_prefix('&')
            .ok_or_else(|| format!("{name:?} does not start with an ampersand"))?;
        if out.insert(key.to_owned(), characters).is_some() {
            return Err(format!("{name:?} appears twice"));
        }
        p.ws();
        match p.peek() {
            Some(b',') => p.at = p.at.saturating_add(1),
            Some(b'}') => {
                p.at = p.at.saturating_add(1);
                break;
            }
            other => return Err(format!("expected `,` or `}}`, found {other:?}")),
        }
    }
    p.ws();
    if p.peek().is_some() {
        return Err(format!("trailing text at byte {}", p.at));
    }
    if out.is_empty() {
        return Err("no named character references".to_owned());
    }
    Ok(out)
}

struct Scan<'a> {
    s: &'a [u8],
    at: usize,
}

impl Scan<'_> {
    fn peek(&self) -> Option<u8> {
        self.s.get(self.at).copied()
    }

    fn ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at = self.at.saturating_add(1);
        }
    }

    fn byte(&mut self, want: u8) -> Result<(), String> {
        if self.peek() == Some(want) {
            self.at = self.at.saturating_add(1);
            Ok(())
        } else {
            Err(format!(
                "expected {:?} at byte {}",
                char::from(want),
                self.at
            ))
        }
    }

    /// One entry's object, returning its `characters` value and ignoring
    /// `codepoints`, which says the same thing a second way.
    fn entry(&mut self) -> Result<String, String> {
        self.byte(b'{')?;
        let mut characters = None;
        loop {
            self.ws();
            let key = self.string()?;
            self.ws();
            self.byte(b':')?;
            self.ws();
            if key == "characters" {
                characters = Some(self.string()?);
            } else {
                self.skip_value()?;
            }
            self.ws();
            match self.peek() {
                Some(b',') => self.at = self.at.saturating_add(1),
                Some(b'}') => {
                    self.at = self.at.saturating_add(1);
                    break;
                }
                other => return Err(format!("expected `,` or `}}`, found {other:?}")),
            }
        }
        characters.ok_or_else(|| "an entry has no `characters`".to_owned())
    }

    /// Steps over a value this generator does not read. Only the shapes
    /// `entities.json` uses: a number, an array of numbers, a string.
    fn skip_value(&mut self) -> Result<(), String> {
        match self.peek() {
            Some(b'"') => {
                self.string()?;
                Ok(())
            }
            Some(b'[') => {
                self.at = self.at.saturating_add(1);
                loop {
                    self.ws();
                    match self.peek() {
                        Some(b']') => {
                            self.at = self.at.saturating_add(1);
                            return Ok(());
                        }
                        Some(b',') => self.at = self.at.saturating_add(1),
                        Some(_) => self.skip_value()?,
                        None => return Err("unterminated array".to_owned()),
                    }
                }
            }
            Some(c) if c.is_ascii_digit() || c == b'-' => {
                while matches!(self.peek(), Some(c) if c.is_ascii_digit()
                    || matches!(c, b'-' | b'+' | b'.' | b'e' | b'E'))
                {
                    self.at = self.at.saturating_add(1);
                }
                Ok(())
            }
            other => Err(format!("unexpected value starting with {other:?}")),
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.byte(b'"')?;
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                return Err("unterminated string".to_owned());
            };
            self.at = self.at.saturating_add(1);
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let Some(k) = self.peek() else {
                        return Err("unterminated escape".to_owned());
                    };
                    self.at = self.at.saturating_add(1);
                    match k {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => out.push(self.unicode_escape()?),
                        other => {
                            return Err(format!("unknown escape `\\{}`", char::from(other)));
                        }
                    }
                }
                // The file is UTF-8 and holds characters above ASCII
                // directly, so the bytes of one are copied through.
                _ => {
                    let start = self.at.saturating_sub(1);
                    let len = utf8_len(c)?;
                    let end = start.saturating_add(len);
                    let bytes = self
                        .s
                        .get(start..end)
                        .ok_or_else(|| "truncated UTF-8".to_owned())?;
                    let s = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
                    out.push_str(s);
                    self.at = end;
                }
            }
        }
    }

    /// `\uXXXX`, with the surrogate pair the format uses above the BMP.
    fn unicode_escape(&mut self) -> Result<char, String> {
        let first = self.hex4()?;
        if (0xD800..0xDC00).contains(&first) {
            self.byte(b'\\')?;
            self.byte(b'u')?;
            let second = self.hex4()?;
            let hi = first.saturating_sub(0xD800).saturating_mul(0x400);
            let lo = second.saturating_sub(0xDC00);
            let n = hi.saturating_add(lo).saturating_add(0x1_0000);
            return char::from_u32(n).ok_or_else(|| format!("{n:#x} is not a character"));
        }
        char::from_u32(first).ok_or_else(|| format!("{first:#x} is not a character"))
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let end = self.at.saturating_add(4);
        let bytes = self
            .s
            .get(self.at..end)
            .ok_or_else(|| "a short \\u escape".to_owned())?;
        let s = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
        let n = u32::from_str_radix(s, 16).map_err(|e| e.to_string())?;
        self.at = end;
        Ok(n)
    }
}

fn utf8_len(first: u8) -> Result<usize, String> {
    Ok(match first {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        other => return Err(format!("{other:#04x} does not start a UTF-8 character")),
    })
}
