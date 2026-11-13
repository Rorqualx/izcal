//! The cascade: origin and importance, element-attached styles,
//! layers, specificity and order of appearance.
//!
//! CSS Cascade 5 §6.1 gives the sorting order as six criteria "in
//! descending order of priority". All six are named here; two of them
//! are not yet modelled and say so rather than being silently dropped:
//!
//! | Criterion | Here |
//! |---|---|
//! | Origin and importance | [`Origin`], with `!important` |
//! | Context (encapsulation) | **not modelled** — shadow trees are ST-M1 |
//! | Element-attached styles | the `style` attribute |
//! | Layers | `@layer`, in declaration order |
//! | Specificity | Selectors 3 §9 |
//! | Order of appearance | the rule's index in its sheet |
//!
//! Two of the eight origin buckets are also absent: transition and
//! animation declarations, which have no source until the animation
//! track exists. The ranks below leave their numbers empty rather than
//! closing the gap, so adding them later moves nothing.
//!
//! **This is the slow way on purpose.** Every rule is matched against
//! every element and the whole list is sorted. `hg-style`'s rule index
//! and invalidation sets exist to get the same answer faster, and
//! ST-4 and ST-6 are the claims that they do.

use std::collections::BTreeMap;

use hg_tree_traits::{AttrNs, Tree};

use crate::selector::{Selector, SelectorList, Specificity};
use crate::sheet::{self, Sheet};

/// Where a declaration came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// The user agent's own sheet.
    UserAgent,
    /// The user's sheet.
    User,
    /// The page's.
    Author,
}

/// One declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    /// The property name, lower-cased.
    pub property: String,
    /// The value, as written.
    pub value: String,
    /// Whether it carries `!important`.
    pub important: bool,
}

/// One style rule.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Its selector list.
    pub selectors: SelectorList,
    /// Its declarations, in the order written.
    pub declarations: Vec<Declaration>,
    /// Its origin.
    pub origin: Origin,
    /// Its layer's index in the sheet's layer order, or `None` for an
    /// unlayered rule.
    pub layer: Option<usize>,
    /// Its index among the sheet's rules.
    pub order: usize,
}

/// Where in the cascade a declaration sits.
///
/// Ordered so that the greater key wins, which is what lets the whole
/// cascade be one `max_by_key`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rank {
    /// Origin and importance: the eight buckets of §6.1, as a number
    /// from 1 (normal user-agent) to 8 (transitions).
    pub origin: u8,
    /// Element-attached: 1 for the `style` attribute, 0 for a style
    /// rule. Not reversed for important declarations — the spec says
    /// "separately for normal and important declarations".
    pub attached: u8,
    /// The layer's rank within its bucket, later-wins for normal
    /// declarations and earlier-wins for important ones.
    pub layer: u32,
    /// Selectors 3 specificity.
    pub specificity: Specificity,
    /// Order of appearance. A `style` attribute is "placed after any
    /// style sheets", so it is given [`u32::MAX`].
    pub order: u32,
}

/// The eight buckets of CSS Cascade 5 §6.1, greatest first in the
/// spec's list and therefore greatest *number* here.
const TRANSITION: u8 = 8;
const IMPORTANT_USER_AGENT: u8 = 7;
const IMPORTANT_USER: u8 = 6;
const IMPORTANT_AUTHOR: u8 = 5;
const ANIMATION: u8 = 4;
const NORMAL_AUTHOR: u8 = 3;
const NORMAL_USER: u8 = 2;
const NORMAL_USER_AGENT: u8 = 1;

/// The bucket a declaration falls in.
#[must_use]
pub const fn origin_rank(origin: Origin, important: bool) -> u8 {
    match (origin, important) {
        (Origin::UserAgent, true) => IMPORTANT_USER_AGENT,
        (Origin::User, true) => IMPORTANT_USER,
        (Origin::Author, true) => IMPORTANT_AUTHOR,
        (Origin::Author, false) => NORMAL_AUTHOR,
        (Origin::User, false) => NORMAL_USER,
        (Origin::UserAgent, false) => NORMAL_USER_AGENT,
    }
}

/// The bucket a transition declaration would fall in. Nothing produces
/// one yet; the constant is here so the gap in the ranks is visible.
#[must_use]
pub const fn transition_rank() -> u8 {
    TRANSITION
}

/// The bucket an animation declaration would fall in.
#[must_use]
pub const fn animation_rank() -> u8 {
    ANIMATION
}

/// A layer's rank (§6.1, "Layers").
///
/// "When comparing declarations that belong to different layers, then
/// for normal rules the declaration whose cascade layer is last wins,
/// and for important rules the declaration whose cascade layer is
/// first wins." Unlayered styles are "an implicit final layer", so
/// they win among normal declarations and lose among important ones —
/// the same logic the origins use.
#[must_use]
pub fn layer_rank(layer: Option<usize>, layer_count: usize, important: bool) -> u32 {
    let count = u32::try_from(layer_count).unwrap_or(u32::MAX);
    match (layer, important) {
        // The implicit final layer, which is last in declaration order.
        (None, false) => count.saturating_add(1),
        (None, true) => 0,
        (Some(i), false) => u32::try_from(i).unwrap_or(0).saturating_add(1),
        (Some(i), true) => count.saturating_sub(u32::try_from(i).unwrap_or(0)),
    }
}

/// A declaration that reached an element, with where it sits.
#[derive(Debug, Clone)]
pub struct Candidate {
    /// The declaration.
    pub declaration: Declaration,
    /// Its place in the cascade.
    pub rank: Rank,
}

/// The sheets of one document, cascaded together.
#[derive(Debug, Clone, Default)]
pub struct Cascade {
    sheets: Vec<Sheet>,
}

impl Cascade {
    /// An empty cascade.
    #[must_use]
    pub const fn new() -> Self {
        Self { sheets: Vec::new() }
    }

    /// Adds a sheet's source in an origin.
    pub fn add(&mut self, source: &str, origin: Origin) -> &mut Self {
        self.sheets.push(sheet::parse(source, origin));
        self
    }

    /// The sheets, for a report of what was skipped or refused.
    #[must_use]
    pub fn sheets(&self) -> &[Sheet] {
        &self.sheets
    }

    /// Every declaration that reaches `node`, with its rank, in no
    /// particular order.
    ///
    /// `pseudo_element` selects declarations whose selector names that
    /// pseudo-element; `None` asks for the element itself.
    #[must_use]
    pub fn candidates<T: Tree + ?Sized>(
        &self,
        tree: &T,
        node: T::Node,
        pseudo_element: Option<&str>,
    ) -> Vec<Candidate> {
        let mut out = Vec::new();
        for s in &self.sheets {
            for rule in &s.rules {
                let Some(specificity) =
                    most_specific_match(tree, node, &rule.selectors, pseudo_element)
                else {
                    continue;
                };
                for declaration in &rule.declarations {
                    out.push(Candidate {
                        rank: Rank {
                            origin: origin_rank(rule.origin, declaration.important),
                            attached: 0,
                            layer: layer_rank(rule.layer, s.layers.len(), declaration.important),
                            specificity,
                            order: u32::try_from(rule.order).unwrap_or(u32::MAX),
                        },
                        declaration: declaration.clone(),
                    });
                }
            }
        }
        // The `style` attribute, which is an author declaration that
        // beats every style rule of the same importance.
        if pseudo_element.is_none()
            && let Some(text) = tree.attribute(node, AttrNs::None, "style")
        {
            for declaration in sheet::declarations(text) {
                out.push(Candidate {
                    rank: Rank {
                        origin: origin_rank(Origin::Author, declaration.important),
                        attached: 1,
                        layer: 0,
                        specificity: Specificity::default(),
                        order: u32::MAX,
                    },
                    declaration,
                });
            }
        }
        out
    }

    /// The winning declaration for every property that reaches
    /// `node`.
    #[must_use]
    pub fn winners<T: Tree + ?Sized>(
        &self,
        tree: &T,
        node: T::Node,
        pseudo_element: Option<&str>,
    ) -> BTreeMap<String, Candidate> {
        let mut out: BTreeMap<String, Candidate> = BTreeMap::new();
        for candidate in self.candidates(tree, node, pseudo_element) {
            match out.get(&candidate.declaration.property) {
                Some(held) if held.rank >= candidate.rank => {}
                _ => {
                    out.insert(candidate.declaration.property.clone(), candidate);
                }
            }
        }
        out
    }

    /// The winning value for one property, as written.
    #[must_use]
    pub fn value<T: Tree + ?Sized>(
        &self,
        tree: &T,
        node: T::Node,
        property: &str,
    ) -> Option<String> {
        self.winners(tree, node, None)
            .get(&property.to_ascii_lowercase())
            .map(|c| c.declaration.value.clone())
    }
}

/// The specificity of the most specific selector in `list` that
/// matches, restricted to those naming `pseudo_element`.
fn most_specific_match<T: Tree + ?Sized>(
    tree: &T,
    node: T::Node,
    list: &SelectorList,
    pseudo_element: Option<&str>,
) -> Option<Specificity> {
    list.selectors
        .iter()
        .filter(|s| s.pseudo_element() == pseudo_element)
        .filter(|s| crate::selector::matches(tree, node, s))
        .map(Selector::specificity)
        .max()
}
