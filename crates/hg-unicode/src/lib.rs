//! `hg-unicode`: Unicode algorithms over `hg-ucd`'s tables.
//!
//! At TX-M0 this is **normalization** — the four forms of UAX #15 — which
//! the fonts and text spec puts first because the style spec's `hg-url`
//! needs it, and because TX-1 requires `NormalizationTest.txt` at 100%
//! from TX-M0. Bidi, line breaking and segmentation follow in the same
//! track.
//!
//! The algorithms are written from UAX #15's prose: the recursive
//! decomposition mappings (Section 3), the canonical ordering algorithm
//! (D108–D109), and canonical composition with the blocking rule (D115,
//! D117). Hangul syllables are composed and decomposed arithmetically
//! (Section 16 of the core specification, "Hangul Syllable
//! Decomposition"), so no table holds them.
//!
//! Tier P: no panic, no indexing, no `unsafe`. Every arithmetic step is
//! checked, and a step that cannot be completed leaves the character
//! unchanged rather than failing — a normalizer that refuses input is
//! worse than one that passes an unusual code point through.

use hg_ucd::{canonical, combining_class, compatibility, compose, quick_check};

/// The Unicode version the tables come from.
pub const VERSION: &str = hg_ucd::VERSION;

/// Which normal form to produce (UAX #15, Table 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Form {
    /// Canonical decomposition, then canonical composition.
    Nfc,
    /// Canonical decomposition.
    Nfd,
    /// Compatibility decomposition, then canonical composition.
    Nfkc,
    /// Compatibility decomposition.
    Nfkd,
}

impl Form {
    /// Whether the form decomposes compatibility mappings too.
    const fn compat(self) -> bool {
        matches!(self, Self::Nfkc | Self::Nfkd)
    }

    /// Whether the form composes after decomposing.
    const fn composes(self) -> bool {
        matches!(self, Self::Nfc | Self::Nfkc)
    }

    /// The bit position of this form's quick-check value.
    const fn quick_check_shift(self) -> u8 {
        match self {
            Self::Nfc => 0,
            Self::Nfd => 2,
            Self::Nfkc => 4,
            Self::Nfkd => 6,
        }
    }
}

// Hangul, from the core specification's "Hangul Syllable Decomposition".
const S_BASE: u32 = 0xAC00;
const L_BASE: u32 = 0x1100;
const V_BASE: u32 = 0x1161;
const T_BASE: u32 = 0x11A7;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;
const N_COUNT: u32 = V_COUNT * T_COUNT; // 588
const S_COUNT: u32 = L_COUNT * N_COUNT; // 11172

/// Normalizes `s` to `form`.
#[must_use]
pub fn normalize(s: &str, form: Form) -> String {
    if is_normalized(s, form) {
        return s.to_owned();
    }
    let mut buf: Vec<char> = Vec::with_capacity(s.len());
    for c in s.chars() {
        decompose_into(c, form.compat(), &mut buf);
    }
    canonical_order(&mut buf);
    if form.composes() {
        compose_in_place(&mut buf);
    }
    buf.into_iter().collect()
}

/// `normalize(s, Form::Nfc)`.
#[must_use]
pub fn nfc(s: &str) -> String {
    normalize(s, Form::Nfc)
}

/// `normalize(s, Form::Nfd)`.
#[must_use]
pub fn nfd(s: &str) -> String {
    normalize(s, Form::Nfd)
}

/// `normalize(s, Form::Nfkc)`.
#[must_use]
pub fn nfkc(s: &str) -> String {
    normalize(s, Form::Nfkc)
}

/// `normalize(s, Form::Nfkd)`.
#[must_use]
pub fn nfkd(s: &str) -> String {
    normalize(s, Form::Nfkd)
}

/// Whether `s` is already in `form`, by the quick check of UAX #15
/// Section 9: a string is in the form when every character's quick-check
/// value is `Yes` and the combining classes are in order. `Maybe` is not
/// an answer, so this returns `false` for it and the caller normalizes.
#[must_use]
pub fn is_normalized(s: &str, form: Form) -> bool {
    let shift = form.quick_check_shift();
    let mut last = 0_u8;
    for c in s.chars() {
        let ccc = combining_class(c);
        if last > ccc && ccc != 0 {
            return false;
        }
        if (quick_check(c) >> shift) & 0b11 != 0 {
            return false;
        }
        last = ccc;
    }
    true
}

/// Appends `c`'s decomposition, or `c` itself when it has none.
fn decompose_into(c: char, compat: bool, out: &mut Vec<char>) {
    if let Some(parts) = hangul_decompose(c) {
        out.extend(parts.iter().flatten());
        return;
    }
    let mapped = if compat {
        compatibility(c).or_else(|| canonical(c))
    } else {
        canonical(c)
    };
    match mapped {
        Some(parts) => out.extend_from_slice(parts),
        None => out.push(c),
    }
}

/// The two or three jamo of a Hangul syllable, or `None` when `c` is not
/// one. The third is `None` for a syllable with no trailing consonant.
fn hangul_decompose(c: char) -> Option<[Option<char>; 3]> {
    let s_index = u32::from(c).checked_sub(S_BASE)?;
    if s_index >= S_COUNT {
        return None;
    }
    let l = L_BASE.checked_add(s_index.checked_div(N_COUNT)?)?;
    let v = V_BASE.checked_add(s_index.checked_rem(N_COUNT)?.checked_div(T_COUNT)?)?;
    let t_index = s_index.checked_rem(T_COUNT)?;
    let t = if t_index == 0 {
        None
    } else {
        char::from_u32(T_BASE.checked_add(t_index)?)
    };
    Some([char::from_u32(l), char::from_u32(v), t])
}

/// The Hangul composition of `first` and `second`, or `None`.
fn hangul_compose(first: char, second: char) -> Option<char> {
    let a = u32::from(first);
    let b = u32::from(second);
    // L + V
    if let Some(l_index) = a.checked_sub(L_BASE)
        && l_index < L_COUNT
        && let Some(v_index) = b.checked_sub(V_BASE)
        && v_index < V_COUNT
    {
        let s = S_BASE
            .checked_add(l_index.checked_mul(N_COUNT)?)?
            .checked_add(v_index.checked_mul(T_COUNT)?)?;
        return char::from_u32(s);
    }
    // LV + T
    if let Some(s_index) = a.checked_sub(S_BASE)
        && s_index < S_COUNT
        && s_index.checked_rem(T_COUNT)? == 0
        && let Some(t_index) = b.checked_sub(T_BASE)
        && t_index > 0
        && t_index < T_COUNT
    {
        return char::from_u32(a.checked_add(t_index)?);
    }
    None
}

/// The canonical ordering algorithm (UAX #15, D108–D109): within each
/// run of non-starters, sort by combining class, stably.
fn canonical_order(buf: &mut [char]) {
    let mut start: Option<usize> = None;
    for i in 0..buf.len() {
        let is_starter = buf.get(i).copied().map(combining_class) == Some(0);
        match (is_starter, start) {
            (true, Some(s)) => {
                sort_run(buf, s, i);
                start = None;
            }
            (true, None) => {}
            (false, None) => start = Some(i),
            (false, Some(_)) => {}
        }
    }
    if let Some(s) = start {
        sort_run(buf, s, buf.len());
    }
}

/// A stable insertion sort of `buf[from..to]` by combining class. The
/// runs are short — the standard's own stability guarantee is what
/// matters, not the asymptotics.
fn sort_run(buf: &mut [char], from: usize, to: usize) {
    let Some(run) = buf.get_mut(from..to) else {
        return;
    };
    for i in 1..run.len() {
        let mut j = i;
        while j > 0 {
            let Some(prev) = run.get(j.wrapping_sub(1)).copied() else {
                break;
            };
            let Some(cur) = run.get(j).copied() else {
                break;
            };
            if combining_class(prev) <= combining_class(cur) {
                break;
            }
            run.swap(j.wrapping_sub(1), j);
            j = j.wrapping_sub(1);
        }
    }
}

/// Canonical composition (UAX #15, D115 and D117), in place.
fn compose_in_place(buf: &mut Vec<char>) {
    let mut out: Vec<char> = Vec::with_capacity(buf.len());
    let mut starter: Option<usize> = None;
    let mut last_ccc = 0_u8;
    let mut between = false;

    for c in buf.iter().copied() {
        let ccc = combining_class(c);
        if let Some(si) = starter {
            // C is blocked from the last starter when something lies
            // between them whose class is zero or not lower than C's.
            let blocked = between && (last_ccc == 0 || last_ccc >= ccc);
            if !blocked
                && let Some(l) = out.get(si).copied()
                && let Some(p) = compose_pair(l, c)
            {
                if let Some(slot) = out.get_mut(si) {
                    *slot = p;
                }
                continue;
            }
        }
        out.push(c);
        if ccc == 0 {
            starter = out.len().checked_sub(1);
            between = false;
            last_ccc = 0;
        } else {
            between = true;
            last_ccc = ccc;
        }
    }
    *buf = out;
}

/// The primary composite of a pair, Hangul included.
fn compose_pair(first: char, second: char) -> Option<char> {
    hangul_compose(first, second).or_else(|| compose(first, second))
}
