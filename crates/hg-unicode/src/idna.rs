//! UTS #46, Unicode IDNA Compatibility Processing.
//!
//! The algorithm a browser runs to turn a domain name a person typed
//! into the ASCII the DNS accepts, and back. It is written here from
//! UTS #46's own prose — Section 4, Processing, and Section 4.1, the
//! validity criteria — over the IDNA mapping table `hg-ucd` carries.
//!
//! [`to_ascii`] and [`to_unicode`] take the flags the standard names,
//! and [`Flags::url`] is the set the URL Standard's "domain to ASCII"
//! asks for: `CheckHyphens` off, `CheckBidi` and `CheckJoiners` on,
//! `Transitional_Processing` off, with `UseSTD3ASCIIRules` and
//! `VerifyDnsLength` following its own `beStrict`.
//!
//! **Transitional processing is implemented and deprecated.** UTS #46
//! says the transition "is complete and transitional processing is now
//! deprecated"; nothing in Izcal turns it on. It is here because
//! `IdnaTestV2.txt` measures it, and a conformance file with a column
//! unchecked is a conformance file half read.
//!
//! # What an error means
//!
//! Every operation returns the converted string **and** whether an
//! error was recorded, because UTS #46 says so: "Even if an error
//! occurs, the conversion of the string is performed as much as is
//! possible." [`to_ascii`] is the one that turns a recorded error into
//! a failure, as its own Section 4.2 step 5 requires.

use hg_ucd::{BidiClass, JoiningType, bidi_class, combining_class, idna, is_mark, joining_type};

use crate::punycode;

/// The prefix a Punycode label carries.
pub const ACE_PREFIX: &str = "xn--";

/// UTS #46's flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    /// Restrict ASCII to lower-case letters, digits and hyphen-minus.
    pub use_std3_ascii_rules: bool,
    /// Check the hyphen placement rules of IDNA2008.
    pub check_hyphens: bool,
    /// Check RFC 5893's right-to-left conditions.
    pub check_bidi: bool,
    /// Check RFC 5892's ContextJ rules.
    pub check_joiners: bool,
    /// Deprecated; map the deviation characters rather than passing
    /// them through.
    pub transitional_processing: bool,
    /// Accept a label whose Punycode does not decode, leaving it as it
    /// is rather than recording an error.
    pub ignore_invalid_punycode: bool,
}

impl Flags {
    /// What the URL Standard's "domain to ASCII" asks for.
    ///
    /// `beStrict` is the caller's: it turns on `UseSTD3ASCIIRules` and,
    /// in [`to_ascii`], `VerifyDnsLength`.
    #[must_use]
    pub const fn url(be_strict: bool) -> Self {
        Self {
            use_std3_ascii_rules: be_strict,
            check_hyphens: false,
            check_bidi: true,
            check_joiners: true,
            transitional_processing: false,
            ignore_invalid_punycode: false,
        }
    }

    /// The flags `IdnaTestV2.txt` is written against, other than the
    /// processing choice.
    ///
    /// Every check is on but `UseSTD3ASCIIRules`, which changes the
    /// converted *string* and so cannot be on against a file whose
    /// expected strings are written for the default.
    #[must_use]
    pub const fn conformance(transitional: bool) -> Self {
        Self {
            use_std3_ascii_rules: false,
            check_hyphens: true,
            check_bidi: true,
            check_joiners: true,
            transitional_processing: transitional,
            ignore_invalid_punycode: false,
        }
    }
}

/// A converted domain name, and whether an error was recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Processed {
    /// The converted string, produced whether or not there was an
    /// error.
    pub text: String,
    /// Whether any step recorded an error.
    pub error: bool,
}

/// UTS #46 Section 4, the main processing steps.
#[must_use]
pub fn process(domain: &str, flags: Flags) -> Processed {
    let mut error = false;

    // 1. Map.
    let mut mapped = String::new();
    for c in domain.chars() {
        let (status, mapping) = idna(c);
        match status {
            "disallowed" | "disallowed_STD3_valid" => {
                // Left unchanged; the Convert/Validate step catches it.
                mapped.push(c);
            }
            "ignored" => {}
            "mapped" => {
                if flags.transitional_processing && c == '\u{1E9E}' {
                    mapped.push_str("ss");
                } else {
                    mapped.extend(mapping);
                }
            }
            "disallowed_STD3_mapped" => {
                if flags.use_std3_ascii_rules {
                    mapped.push(c);
                } else {
                    mapped.extend(mapping);
                }
            }
            "deviation" => {
                if flags.transitional_processing {
                    mapped.extend(mapping);
                } else {
                    mapped.push(c);
                }
            }
            // "valid" and anything a future table adds.
            _ => mapped.push(c),
        }
    }

    // 2. Normalize to NFC.
    let normalized = crate::nfc(&mapped);

    // 3. Break into labels at U+002E, and 4. Convert/Validate.
    let mut out: Vec<String> = Vec::new();
    for label in normalized.split('.') {
        let (converted, label_error) = convert_label(label, flags);
        error |= label_error;
        out.push(converted);
    }

    // RFC 5893's conditions are about the whole name, not one label.
    let text = out.join(".");
    if flags.check_bidi && !bidi_domain_ok(&text) {
        error = true;
    }
    Processed { text, error }
}

/// One label's Convert/Validate step.
fn convert_label(label: &str, flags: Flags) -> (String, bool) {
    let Some(rest) = ace_suffix(label) else {
        // Not a Punycode label: validate as it is.
        return (label.to_owned(), !valid(label, flags, false));
    };
    if !label.is_ascii() {
        // "If the label contains any non-ASCII code point ... record
        // that there was an error, and continue with the next label."
        return (label.to_owned(), true);
    }
    let Ok(decoded) = punycode::decode(rest) else {
        return (label.to_owned(), !flags.ignore_invalid_punycode);
    };
    // "If the label is empty, or if the label contains only ASCII code
    // points, record that there was an error."
    let mut error = decoded.is_empty() || decoded.is_ascii();
    // A decoded label is always validated with Nontransitional
    // processing: "sources already in Punycode are validated without
    // mapping".
    if !valid(&decoded, flags, true) {
        error = true;
    }
    (decoded, error)
}

/// The part of a label after `xn--`, when it has that prefix.
fn ace_suffix(label: &str) -> Option<&str> {
    let head = label.get(..4)?;
    if head.eq_ignore_ascii_case(ACE_PREFIX) {
        label.get(4..)
    } else {
        None
    }
}

/// UTS #46 Section 4.1, the validity criteria, for one label.
///
/// `from_punycode` says the label came out of a Punycode decode, which
/// fixes the processing choice to Nontransitional whatever the flags
/// say.
fn valid(label: &str, flags: Flags, from_punycode: bool) -> bool {
    if label.is_empty() {
        // "Each of the following criteria must be satisfied for a
        // non-empty label": an empty one is vacuously valid, and the
        // empty root label is what a trailing dot leaves.
        return true;
    }
    // 1. The label must be in NFC.
    if !crate::is_normalized(label, crate::Form::Nfc) {
        return false;
    }
    let chars: Vec<char> = label.chars().collect();
    if flags.check_hyphens {
        // 2 and 3.
        if chars.get(2) == Some(&'-') && chars.get(3) == Some(&'-') {
            return false;
        }
        if chars.first() == Some(&'-') || chars.last() == Some(&'-') {
            return false;
        }
    } else if ace_suffix(label).is_some() {
        // 4. "If not CheckHyphens, the label must not begin with
        // `xn--`." A label that *came from* Punycode has had the
        // prefix removed, so this is about one that still has it.
        return false;
    }
    // 5. No full stop. Splitting on it already guarantees this, but a
    // Punycode decode can put one back.
    if label.contains('.') {
        return false;
    }
    // 6. Must not begin with a combining mark.
    if chars.first().copied().is_some_and(is_mark) {
        return false;
    }
    // 7. Every code point's status.
    let transitional = flags.transitional_processing && !from_punycode;
    for c in &chars {
        let (status, _) = idna(*c);
        let ok = match status {
            "valid" => true,
            "deviation" => !transitional,
            "disallowed_STD3_valid" => !flags.use_std3_ascii_rules,
            _ => false,
        };
        if !ok {
            return false;
        }
        if flags.use_std3_ascii_rules
            && c.is_ascii()
            && !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        {
            return false;
        }
    }
    // 8. ContextJ.
    if flags.check_joiners && !context_j_ok(&chars) {
        return false;
    }
    true
}

/// RFC 5892 Appendix A's rules for the two joiners.
///
/// A.1, ZERO WIDTH NON-JOINER: true if the preceding character has
/// `Canonical_Combining_Class` Virama, or if the label matches
/// `(L|D) T* ZWNJ T* (R|D)`.
///
/// A.2, ZERO WIDTH JOINER: true only if the preceding character has
/// `Canonical_Combining_Class` Virama.
fn context_j_ok(chars: &[char]) -> bool {
    /// The combining class the rules call Virama.
    const VIRAMA: u8 = 9;
    for (i, c) in chars.iter().enumerate() {
        if *c != '\u{200C}' && *c != '\u{200D}' {
            continue;
        }
        let before = i.checked_sub(1).and_then(|j| chars.get(j)).copied();
        if before.is_some_and(|b| combining_class(b) == VIRAMA) {
            continue;
        }
        if *c == '\u{200D}' {
            // A.2 has no second clause.
            return false;
        }
        // A.1's second clause, read backwards then forwards over the
        // transparent characters.
        let mut back = i;
        let left = loop {
            let Some(j) = back.checked_sub(1) else {
                break None;
            };
            back = j;
            let Some(b) = chars.get(j).copied() else {
                break None;
            };
            if joining_type(b) != JoiningType::T {
                break Some(joining_type(b));
            }
        };
        if !matches!(left, Some(JoiningType::L | JoiningType::D)) {
            return false;
        }
        let mut fore = i.saturating_add(1);
        let right = loop {
            let Some(b) = chars.get(fore).copied() else {
                break None;
            };
            fore = fore.saturating_add(1);
            if joining_type(b) != JoiningType::T {
                break Some(joining_type(b));
            }
        };
        if !matches!(right, Some(JoiningType::R | JoiningType::D)) {
            return false;
        }
    }
    true
}

/// Whether the whole domain satisfies RFC 5893, Section 2.
///
/// A *Bidi domain name* is one where some label holds a character of
/// class `R`, `AL` or `AN`. A name that is not one is unconstrained.
fn bidi_domain_ok(domain: &str) -> bool {
    let is_bidi = domain
        .chars()
        .any(|c| matches!(bidi_class(c), BidiClass::R | BidiClass::Al | BidiClass::An));
    if !is_bidi {
        return true;
    }
    domain.split('.').all(bidi_label_ok)
}

/// RFC 5893 Section 2's six numbered conditions, for one label.
fn bidi_label_ok(label: &str) -> bool {
    let chars: Vec<char> = label.chars().collect();
    let Some(first) = chars.first().copied() else {
        return true;
    };
    // 1. The first character must be L, R or AL, and that choice makes
    // the label an RTL or an LTR label.
    let rtl = match bidi_class(first) {
        BidiClass::R | BidiClass::Al => true,
        BidiClass::L => false,
        BidiClass::An
        | BidiClass::En
        | BidiClass::Es
        | BidiClass::Cs
        | BidiClass::Et
        | BidiClass::On
        | BidiClass::Bn
        | BidiClass::Nsm
        | BidiClass::Other => return false,
    };
    // The trailing NSMs are excluded from the "last character" rules
    // (conditions 3 and 6).
    let last = chars
        .iter()
        .rev()
        .find(|c| bidi_class(**c) != BidiClass::Nsm)
        .copied();
    if rtl {
        // 2. Only these classes may appear.
        for c in &chars {
            if !matches!(
                bidi_class(*c),
                BidiClass::R
                    | BidiClass::Al
                    | BidiClass::An
                    | BidiClass::En
                    | BidiClass::Es
                    | BidiClass::Cs
                    | BidiClass::Et
                    | BidiClass::On
                    | BidiClass::Bn
                    | BidiClass::Nsm
            ) {
                return false;
            }
        }
        // 3. The last non-NSM must be R, AL, AN or EN.
        if !last.is_some_and(|c| {
            matches!(
                bidi_class(c),
                BidiClass::R | BidiClass::Al | BidiClass::An | BidiClass::En
            )
        }) {
            return false;
        }
        // 4. EN and AN must not both appear.
        let has_en = chars.iter().any(|c| bidi_class(*c) == BidiClass::En);
        let has_an = chars.iter().any(|c| bidi_class(*c) == BidiClass::An);
        if has_en && has_an {
            return false;
        }
    } else {
        // 5. Only these classes may appear.
        for c in &chars {
            if !matches!(
                bidi_class(*c),
                BidiClass::L
                    | BidiClass::En
                    | BidiClass::Es
                    | BidiClass::Cs
                    | BidiClass::Et
                    | BidiClass::On
                    | BidiClass::Bn
                    | BidiClass::Nsm
            ) {
                return false;
            }
        }
        // 6. The last non-NSM must be L or EN.
        if !last.is_some_and(|c| matches!(bidi_class(c), BidiClass::L | BidiClass::En)) {
            return false;
        }
    }
    true
}

/// UTS #46 Section 4.3, ToUnicode.
#[must_use]
pub fn to_unicode(domain: &str, flags: Flags) -> Processed {
    process(domain, flags)
}

/// UTS #46 Section 4.2, ToASCII.
///
/// Returns `None` when an error was recorded, which is the standard's
/// failure value: "No DNS lookup should be done."
#[must_use]
pub fn to_ascii(domain: &str, flags: Flags, verify_dns_length: bool) -> Option<String> {
    let p = process(domain, flags);
    let mut error = p.error;
    let mut labels: Vec<String> = Vec::new();
    for label in p.text.split('.') {
        if label.is_ascii() {
            labels.push(label.to_owned());
            continue;
        }
        match punycode::encode(label) {
            Ok(encoded) => labels.push(format!("{ACE_PREFIX}{encoded}")),
            Err(_) => {
                error = true;
                labels.push(label.to_owned());
            }
        }
    }
    if verify_dns_length {
        // "The length of the domain name, excluding the root label and
        // its dot, is from 1 to 253. The length of each label is from 1
        // to 63."
        let joined = labels.join(".");
        let trimmed = joined.strip_suffix('.').unwrap_or(&joined);
        if trimmed.is_empty() || trimmed.len() > 253 {
            error = true;
        }
        for label in &labels {
            if label.is_empty() || label.len() > 63 {
                error = true;
            }
        }
    }
    if error {
        return None;
    }
    Some(labels.join("."))
}
