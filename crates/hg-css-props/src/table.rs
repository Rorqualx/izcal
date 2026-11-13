//! The vocabulary the generated table is written in.
//!
//! This is the only hand-written code in the crate, and it holds no
//! data: [`Property`] is an index into the generated rows, and every
//! accessor reads one.
//!
//! [`Property`] is two bytes because a declaration block is a list of
//! (property, value) pairs and there are a great many of them on a
//! page. It can only be made from a generated constant or from
//! [`Property::from_name`], so the index it carries is always one the
//! table has — but nothing here indexes a slice to prove it, because
//! this crate denies `indexing_slicing` and a table lookup is not the
//! place to make an exception.

use crate::generated::kinds::ComputedKind;
use crate::generated::properties::{BY_NAME, ROWS};

/// What a change to a property dirties.
///
/// A set, because a property can do more than one: an SVG geometry
/// attribute changes the SVG layout pass *and* paint. [`Affects::NOTHING`]
/// and [`Affects::UNDECIDED`] stand alone — the first says a change is
/// invisible downstream, the second that the decision has not been
/// made, which the overlay writes down rather than guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Affects(u8);

impl Affects {
    /// Layout must run again.
    pub const LAYOUT: Self = Self(0b0000_0001);
    /// Paint must run again; layout need not.
    pub const PAINT: Self = Self(0b0000_0010);
    /// Only the compositor's inputs change.
    pub const COMPOSITE: Self = Self(0b0000_0100);
    /// The SVG layout pass must run again (the SVG spec's R5).
    pub const SVG_GEOMETRY: Self = Self(0b0000_1000);
    /// The SVG reference graph must be rebuilt (the SVG spec's R5).
    pub const SVG_RESOURCE: Self = Self(0b0001_0000);
    /// Nothing downstream changes.
    pub const NOTHING: Self = Self(0);
    /// Not yet decided. The overlay writes this where the answer is
    /// open, and `hg-cssgen` reports how many properties carry it.
    pub const UNDECIDED: Self = Self(0b1000_0000);

    /// Both.
    #[must_use]
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether this set contains all of `other`.
    ///
    /// [`Affects::NOTHING`] is the empty set, so every set contains it;
    /// ask [`Affects::is_nothing`] instead.
    #[must_use]
    pub const fn has(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether a change is invisible to layout, paint and the
    /// compositor.
    #[must_use]
    pub const fn is_nothing(self) -> bool {
        self.0 == 0
    }

    /// Whether the decision is still open.
    #[must_use]
    pub const fn is_undecided(self) -> bool {
        self.0 == Self::UNDECIDED.0
    }
}

/// One property's row. Built only by `hg-cssgen`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Row {
    pub(crate) name: &'static str,
    pub(crate) initial: Option<&'static str>,
    pub(crate) inherits: bool,
    pub(crate) kind: Option<ComputedKind>,
    pub(crate) hook: Option<&'static str>,
    pub(crate) affects: Affects,
    pub(crate) longhands: &'static [Property],
    pub(crate) animation: Option<&'static str>,
}

/// The row a property with an index the table does not have would use.
///
/// [`Property`] cannot be built with such an index, so nothing reaches
/// this. It exists because the alternative is a panic in a lookup, and
/// a table whose accessors can panic is a table every caller must
/// defend against.
static ABSENT: Row = Row {
    name: "",
    initial: None,
    inherits: false,
    kind: None,
    hook: None,
    affects: Affects::UNDECIDED,
    longhands: &[],
    animation: None,
};

/// One CSS property.
///
/// Made from a generated constant (`Property::COLOR`) or looked up by
/// name. A legacy alias — `-webkit-align-content` and the sixty-three
/// others the Compatibility Standard defines — resolves to the property
/// it stands for, so the cascade never sees two spellings of one thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Property(pub(crate) u16);

impl Property {
    /// The property a declaration names, or `None` when nothing does.
    ///
    /// CSS property names are ASCII case-insensitive, so `Color` and
    /// `COLOR` both find `color`. A custom property (`--x`) is not one
    /// of these and is not found here.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let found = BY_NAME.binary_search_by(|(n, _)| cmp_ascii_lowercase(n, name));
        BY_NAME.get(found.ok()?).map(|(_, i)| Self(*i))
    }

    fn row(self) -> &'static Row {
        ROWS.get(usize::from(self.0)).unwrap_or(&ABSENT)
    }

    /// Its index in the table, which is also its identity.
    #[must_use]
    pub const fn index(self) -> u16 {
        self.0
    }

    /// Its name, in the spelling the specification uses.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.row().name
    }

    /// Its initial value, as the specification writes it.
    ///
    /// `None` for the handful whose specifications state it in a form
    /// `@webref/css` did not extract.
    #[must_use]
    pub fn initial(self) -> Option<&'static str> {
        self.row().initial
    }

    /// Whether it inherits.
    #[must_use]
    pub fn inherits(self) -> bool {
        self.row().inherits
    }

    /// Which conversion takes its specified value to a computed one.
    #[must_use]
    pub fn computed_kind(self) -> Option<ComputedKind> {
        self.row().kind
    }

    /// The hand-written function that computes its value, for the few
    /// whose specifications define it in prose a table cannot carry.
    #[must_use]
    pub fn prose_hook(self) -> Option<&'static str> {
        self.row().hook
    }

    /// What a change to it dirties.
    #[must_use]
    pub fn affects(self) -> Affects {
        self.row().affects
    }

    /// Its animation type, as the specification writes it.
    #[must_use]
    pub fn animation(self) -> Option<&'static str> {
        self.row().animation
    }

    /// The longhands it expands to; empty when it is not a shorthand.
    #[must_use]
    pub fn longhands(self) -> &'static [Self] {
        self.row().longhands
    }

    /// Whether it is a shorthand.
    #[must_use]
    pub fn is_shorthand(self) -> bool {
        !self.row().longhands.is_empty()
    }

    /// Every property, in name order.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..ROWS.len())
            .filter_map(|i| u16::try_from(i).ok())
            .map(Self)
    }
}

/// Compares a table name, which is already ASCII lower case, with a name
/// a declaration wrote, which may not be.
fn cmp_ascii_lowercase(table: &str, given: &str) -> std::cmp::Ordering {
    let mut ours = table.bytes();
    let mut theirs = given.bytes().map(|b| b.to_ascii_lowercase());
    loop {
        match (ours.next(), theirs.next()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(a), Some(b)) if a == b => {}
            (Some(a), Some(b)) => return a.cmp(&b),
        }
    }
}

/// The value grammars and defining URLs, behind the `reference`
/// feature: `hg-style-reference` interprets the grammars itself (G20),
/// and DevTools shows the URLs.
#[cfg(feature = "reference")]
pub mod reference {
    use super::Property;
    use crate::generated::grammars::{GRAMMARS, SPEC_URLS};

    /// A property's value grammar, in CSS value-definition syntax.
    #[must_use]
    pub fn grammar(p: Property) -> Option<&'static str> {
        GRAMMARS.get(usize::from(p.index())).copied().flatten()
    }

    /// The URL of a property's definition in the specification.
    #[must_use]
    pub fn spec_url(p: Property) -> &'static str {
        SPEC_URLS.get(usize::from(p.index())).copied().unwrap_or("")
    }
}
