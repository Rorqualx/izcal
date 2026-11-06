//! The `.hgop` lexer (design review R0-O1, Section 3 of the language
//! note). Comments run from `//` to the end of the line.

use crate::diag::{Diag, Span};

/// One token.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// An identifier or keyword.
    Ident(String),
    /// `%Name`: an intrinsic's name, without the `%`.
    Intrinsic(String),
    /// An integer literal.
    Int(i128),
    /// A floating-point literal, as written (parsed by the checker).
    Float(String),
    /// A string literal, unescaped.
    Str(String),
    /// Punctuation.
    Punct(&'static str),
    /// The end of the file.
    Eof,
}

/// A token and where it starts.
#[derive(Debug, Clone)]
pub struct Token {
    /// The token.
    pub tok: Tok,
    /// Where it starts.
    pub span: Span,
}

/// Longest first, so `>>>` wins over `>>` and `>`.
const PUNCT: [&str; 34] = [
    ">>>", "==", "!=", "<=", ">=", "&&", "||", "<<", ">>", "=>", "->", "{", "}", "(", ")", "[",
    "]", "<", ">", ",", ";", ":", "=", ".", "|", "&", "^", "!", "~", "+", "-", "*", "/", "%",
];

/// Splits `src` (file number `file`) into tokens.
///
/// # Errors
/// A character that starts no token, or an unterminated string.
pub fn lex(src: &str, file: u16) -> Result<Vec<Token>, Diag> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let (mut i, mut line, mut col) = (0usize, 1u32, 1u32);
    let span = |line, col| Span { file, line, col };
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            i += 1;
            line += 1;
            col = 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            col += 1;
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        let start = span(line, col);
        if c.is_ascii_alphabetic() || c == '_' || c == '%' {
            let mut j = i + 1;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            let text: String = chars[i..j].iter().collect();
            let tok = if let Some(name) = text.strip_prefix('%') {
                if name.is_empty() {
                    return Err(Diag::new(start, "OC-1", "`%` must start an intrinsic name"));
                }
                Tok::Intrinsic(name.to_string())
            } else {
                Tok::Ident(text)
            };
            col += u32::try_from(j - i).unwrap_or(u32::MAX);
            i = j;
            out.push(Token { tok, span: start });
            continue;
        }
        if c.is_ascii_digit() {
            let mut j = i;
            let mut float = false;
            while j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '_') {
                j += 1;
            }
            if chars.get(j) == Some(&'.') && chars.get(j + 1).is_some_and(char::is_ascii_digit) {
                float = true;
                j += 1;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
            }
            if matches!(chars.get(j), Some('e' | 'E')) {
                let mut k = j + 1;
                if matches!(chars.get(k), Some('+' | '-')) {
                    k += 1;
                }
                if chars.get(k).is_some_and(char::is_ascii_digit) {
                    float = true;
                    j = k;
                    while j < chars.len() && chars[j].is_ascii_digit() {
                        j += 1;
                    }
                }
            }
            let text: String = chars[i..j].iter().filter(|c| **c != '_').collect();
            let tok = if float {
                Tok::Float(text)
            } else {
                match text.parse::<i128>() {
                    Ok(v) => Tok::Int(v),
                    Err(_) => return Err(Diag::new(start, "OC-1", "integer literal out of range")),
                }
            };
            col += u32::try_from(j - i).unwrap_or(u32::MAX);
            i = j;
            out.push(Token { tok, span: start });
            continue;
        }
        if c == '"' {
            let mut j = i + 1;
            let mut text = String::new();
            loop {
                match chars.get(j) {
                    None | Some('\n') => {
                        return Err(Diag::new(start, "OC-1", "unterminated string literal"));
                    }
                    Some('"') => break,
                    Some('\\') => {
                        match chars.get(j + 1) {
                            Some('"') => text.push('"'),
                            Some('\\') => text.push('\\'),
                            Some('n') => text.push('\n'),
                            _ => return Err(Diag::new(start, "OC-1", "unknown string escape")),
                        }
                        j += 2;
                    }
                    Some(&ch) => {
                        text.push(ch);
                        j += 1;
                    }
                }
            }
            col += u32::try_from(j + 1 - i).unwrap_or(u32::MAX);
            i = j + 1;
            out.push(Token {
                tok: Tok::Str(text),
                span: start,
            });
            continue;
        }
        let rest: String = chars[i..chars.len().min(i + 3)].iter().collect();
        let Some(p) = PUNCT.iter().find(|p| rest.starts_with(**p)) else {
            return Err(Diag::new(
                start,
                "OC-1",
                format!("unexpected character `{c}`"),
            ));
        };
        i += p.len();
        col += u32::try_from(p.len()).unwrap_or(u32::MAX);
        out.push(Token {
            tok: Tok::Punct(p),
            span: start,
        });
    }
    out.push(Token {
        tok: Tok::Eof,
        span: span(line, col),
    });
    Ok(out)
}
