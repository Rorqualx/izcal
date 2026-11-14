//! Punycode, RFC 3492.
//!
//! The bootstring encoding IDNA uses to carry a Unicode label in the
//! ASCII the DNS accepts. It is written here from the RFC's own
//! pseudocode and its named parameters, not from any implementation.
//!
//! The parameters, from RFC 3492 Section 5:
//!
//! ```text
//! base = 36, tmin = 1, tmax = 26, skew = 38, damp = 700,
//! initial_bias = 72, initial_n = 128
//! ```
//!
//! The one thing the RFC is explicit about and that is easy to get
//! wrong: **overflow must be detected**. The delta arithmetic is
//! unbounded in principle, and a wrapped value would decode to the
//! wrong label — Section 6.4 says "an implementation must detect
//! overflow", so every step here is a checked one.
//!
//! What [`decode`] does *not* do is insist that its output re-encode
//! to its input. Neither RFC 3492 nor UTS #46 asks for that, and the
//! stricter rule would refuse labels a registry could legitimately
//! have issued. UTS #46 places its own conditions on a decoded label
//! afterwards, in the validity criteria.

/// Integer division that cannot divide by zero.
///
/// Every divisor here is a non-zero constant or a length that has been
/// forced to at least one, so the fallback is unreachable; it exists
/// because this crate is tier P and may not panic, and a division is
/// the one arithmetic operation that can.
const fn div(a: u32, b: u32) -> u32 {
    match a.checked_div(b) {
        Some(v) => v,
        None => 0,
    }
}

/// The remainder, under the same rule.
const fn rem(a: u32, b: u32) -> u32 {
    match a.checked_rem(b) {
        Some(v) => v,
        None => 0,
    }
}

/// The number of digits in the base.
const BASE: u32 = 36;
/// The least number of digits a delta may use.
const TMIN: u32 = 1;
/// The most.
const TMAX: u32 = 26;
/// The skew, which flattens the bias adaptation.
const SKEW: u32 = 38;
/// The damp, applied to the first delta only.
const DAMP: u32 = 700;
/// The bias a run starts with.
const INITIAL_BIAS: u32 = 72;
/// The code point a run starts from: the end of ASCII.
const INITIAL_N: u32 = 128;
/// The delimiter between the basic code points and the deltas.
pub const DELIMITER: char = '-';

/// What went wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// A digit outside the base, or a delimiter where none may be.
    BadInput,
    /// The arithmetic would have wrapped (RFC 3492 Section 6.4).
    Overflow,
    /// A delta named a code point that is not one.
    NotACodePoint,
}

/// RFC 3492's `adapt`.
fn adapt(delta: u32, numpoints: u32, firsttime: bool) -> u32 {
    let mut delta = if firsttime {
        div(delta, DAMP)
    } else {
        div(delta, 2)
    };
    delta = delta.saturating_add(div(delta, numpoints.max(1)));
    let mut k = 0_u32;
    let limit = div(BASE.saturating_sub(TMIN).saturating_mul(TMAX), 2);
    while delta > limit {
        delta = div(delta, BASE.saturating_sub(TMIN));
        k = k.saturating_add(BASE);
    }
    let numerator = BASE
        .saturating_sub(TMIN)
        .saturating_add(1)
        .saturating_mul(delta);
    k.saturating_add(div(numerator, delta.saturating_add(SKEW)))
}

/// A digit's value, or `None` when it is not a base-36 digit.
fn digit(c: char) -> Option<u32> {
    match c {
        'a'..='z' => Some(u32::from(c).checked_sub(u32::from('a'))?),
        'A'..='Z' => Some(u32::from(c).checked_sub(u32::from('A'))?),
        '0'..='9' => Some(u32::from(c).checked_sub(u32::from('0'))?.checked_add(26)?),
        _ => None,
    }
}

/// The character for a digit value, lower case.
fn digit_char(d: u32) -> Option<char> {
    let v = if d < 26 {
        u32::from(b'a').checked_add(d)?
    } else {
        u32::from(b'0').checked_add(d.checked_sub(26)?)?
    };
    char::from_u32(v)
}

/// The threshold for position `k`, RFC 3492 Section 6.
fn threshold(k: u32, bias: u32) -> u32 {
    if k <= bias.saturating_add(TMIN) {
        TMIN
    } else if k >= bias.saturating_add(TMAX) {
        TMAX
    } else {
        k.saturating_sub(bias)
    }
}

/// Decodes a Punycode string — the part **after** `xn--`.
///
/// # Errors
/// [`Error`] when the input is not something [`encode`] could have
/// produced.
pub fn decode(input: &str) -> Result<String, Error> {
    let mut n = INITIAL_N;
    let mut i = 0_u32;
    let mut bias = INITIAL_BIAS;
    // Everything before the last delimiter is literal; with no
    // delimiter, nothing is.
    let (basic, rest) = match input.rfind(DELIMITER) {
        Some(at) => (
            input.get(..at).ok_or(Error::BadInput)?,
            input.get(at.saturating_add(1)..).ok_or(Error::BadInput)?,
        ),
        None => ("", input),
    };
    if !basic.is_ascii() {
        return Err(Error::BadInput);
    }
    let mut output: Vec<char> = basic.chars().collect();

    let mut chars = rest.chars();
    while let Some(first) = chars.next() {
        let old_i = i;
        let mut w = 1_u32;
        let mut k = BASE;
        let mut c = first;
        loop {
            let d = digit(c).ok_or(Error::BadInput)?;
            i = i
                .checked_add(d.checked_mul(w).ok_or(Error::Overflow)?)
                .ok_or(Error::Overflow)?;
            let t = threshold(k, bias);
            if d < t {
                break;
            }
            w = w
                .checked_mul(BASE.checked_sub(t).ok_or(Error::Overflow)?)
                .ok_or(Error::Overflow)?;
            k = k.checked_add(BASE).ok_or(Error::Overflow)?;
            c = chars.next().ok_or(Error::BadInput)?;
        }
        let len = u32::try_from(output.len())
            .ok()
            .and_then(|l| l.checked_add(1))
            .ok_or(Error::Overflow)?;
        bias = adapt(i.saturating_sub(old_i), len, old_i == 0);
        n = n.checked_add(div(i, len)).ok_or(Error::Overflow)?;
        i = rem(i, len);
        let at = usize::try_from(i).ok().ok_or(Error::Overflow)?;
        if at > output.len() {
            return Err(Error::BadInput);
        }
        output.insert(at, char::from_u32(n).ok_or(Error::NotACodePoint)?);
        i = i.checked_add(1).ok_or(Error::Overflow)?;
    }
    Ok(output.into_iter().collect())
}

/// Encodes a label as Punycode — the part **after** `xn--`.
///
/// # Errors
/// [`Error::Overflow`] when the arithmetic would wrap.
pub fn encode(input: &str) -> Result<String, Error> {
    let chars: Vec<char> = input.chars().collect();
    let mut output: String = chars.iter().filter(|c| c.is_ascii()).collect();
    let basic = output.chars().count();
    let mut h = u32::try_from(basic).ok().ok_or(Error::Overflow)?;
    let b = h;
    if basic > 0 {
        output.push(DELIMITER);
    }
    let total = u32::try_from(chars.len()).ok().ok_or(Error::Overflow)?;

    let mut n = INITIAL_N;
    let mut delta = 0_u32;
    let mut bias = INITIAL_BIAS;
    while h < total {
        // The next code point to encode: the least one at or above n.
        let m = chars
            .iter()
            .map(|c| u32::from(*c))
            .filter(|v| *v >= n)
            .min()
            .ok_or(Error::Overflow)?;
        delta = delta
            .checked_add(
                m.checked_sub(n)
                    .ok_or(Error::Overflow)?
                    .checked_mul(h.checked_add(1).ok_or(Error::Overflow)?)
                    .ok_or(Error::Overflow)?,
            )
            .ok_or(Error::Overflow)?;
        n = m;
        for c in &chars {
            let v = u32::from(*c);
            if v < n {
                delta = delta.checked_add(1).ok_or(Error::Overflow)?;
            }
            if v != n {
                continue;
            }
            let mut q = delta;
            let mut k = BASE;
            loop {
                let t = threshold(k, bias);
                if q < t {
                    break;
                }
                let base_minus_t = BASE.checked_sub(t).ok_or(Error::Overflow)?;
                let d = t
                    .checked_add(rem(q.saturating_sub(t), base_minus_t))
                    .ok_or(Error::Overflow)?;
                output.push(digit_char(d).ok_or(Error::Overflow)?);
                q = div(q.saturating_sub(t), base_minus_t);
                k = k.checked_add(BASE).ok_or(Error::Overflow)?;
            }
            output.push(digit_char(q).ok_or(Error::Overflow)?);
            bias = adapt(delta, h.checked_add(1).ok_or(Error::Overflow)?, h == b);
            delta = 0;
            h = h.checked_add(1).ok_or(Error::Overflow)?;
        }
        delta = delta.checked_add(1).ok_or(Error::Overflow)?;
        n = n.checked_add(1).ok_or(Error::Overflow)?;
    }
    Ok(output)
}
