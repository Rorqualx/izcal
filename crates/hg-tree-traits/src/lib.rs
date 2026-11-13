//! `hg-tree-traits`: Izcal's one read-only view of the DOM. L2
//! contracts, tier P.
//!
//! **G1.** `hg-style`, `hg-selectors`, `hg-box-tree`, `hg-layout`,
//! `hg-paint` and the semantic plane read the DOM *only* through this
//! crate. They never name `hg-dom`, `hg-heap` or any JS value type.
//! Two things follow, and both are the point:
//!
//! - **Workers never touch the cage or GC** (B4, H-2). A trait that
//!   hands out `&str` and plain ids can be implemented over
//!   `hg-node-store`'s columns and read from any thread, which a
//!   garbage-collected node reference could not.
//! - **The dual-implementation oracle becomes possible** (G20).
//!   `hg-style-reference` shares no code with `hg-style` *except* this
//!   trait and `hg-types`, so the two engines agreeing means something.
//!   If they shared a tree implementation, a tree bug would agree with
//!   itself.
//!
//! # What a tree is here
//!
//! [`Tree`] is the whole interface: shape ([`Tree::parent`] and the
//! four sibling and child accessors), identity ([`Tree::kind`],
//! [`Tree::local_name`]), attributes, element state, and text. Walks
//! over it — ancestors, element children, document order — are free
//! functions in this crate rather than trait methods, so an
//! implementation cannot get them subtly wrong and so a matcher and a
//! layout pass walk the tree the same way.
//!
//! # What a tree is not
//!
//! There is no write path, no event path, no node allocation and no
//! lifetime tied to a heap. A reader holds a `&Tree` for the length of
//! one pass, during which the tree does not change — that is the
//! contract, and it is what lets a pass be parallel and phase-pure
//! (ST 6.10).
//!
//! # Atoms
//!
//! Names come back as `&str`. Interning is real and will arrive with
//! `hg-node-store`'s columns and `hg-types`' atom table, but baking an
//! atom type into the contract now would force the reference engine to
//! share it — and a shared intern table is exactly the common-mode
//! risk G20 exists to keep small.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;

#[cfg(feature = "testing")]
pub mod testing;

/// A node's identity: a slot and the epoch it was allocated in.
///
/// Stage 1's node header (§5.2) is a 32-bit slot plus a 32-bit epoch,
/// so a stale id from a previous epoch can be told from a live one
/// rather than silently naming whatever now occupies the slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    /// The slot in the node columns.
    pub slot: u32,
    /// The epoch the slot was allocated in.
    pub epoch: u32,
}

impl NodeId {
    /// A node id.
    #[must_use]
    pub const fn new(slot: u32, epoch: u32) -> Self {
        Self { slot, epoch }
    }
}

/// What kind of node this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// An element.
    Element,
    /// A text node.
    Text,
    /// A comment.
    Comment,
    /// A processing instruction.
    ProcessingInstruction,
    /// The document.
    Document,
    /// A doctype.
    DocumentType,
    /// A document fragment, including a shadow root.
    DocumentFragment,
}

/// The namespaces the platform names, with their URLs.
pub mod ns {
    /// The HTML namespace.
    pub const HTML: &str = "http://www.w3.org/1999/xhtml";
    /// The SVG namespace.
    pub const SVG: &str = "http://www.w3.org/2000/svg";
    /// The MathML namespace.
    pub const MATHML: &str = "http://www.w3.org/1998/Math/MathML";
    /// The XLink namespace.
    pub const XLINK: &str = "http://www.w3.org/1999/xlink";
    /// The XML namespace.
    pub const XML: &str = "http://www.w3.org/XML/1998/namespace";
    /// The XMLNS namespace.
    pub const XMLNS: &str = "http://www.w3.org/2000/xmlns/";
}

/// Which namespace an attribute selector asks about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrNs<'a> {
    /// No namespace — the default, and what `[att]` means.
    None,
    /// One named namespace, as `[ns|att]` means.
    One(&'a str),
    /// Any namespace, as `[*|att]` means.
    Any,
}

/// The element state a selector can ask about.
///
/// Every bit here is state **outside the tree** (ST 6.7.1): it is
/// written by a named source on the script thread and logged with its
/// old bits, so a restyle it causes is reproducible. A matcher reads
/// the bits and never computes them — `:focus-visible`'s heuristic,
/// for instance, runs in the focus code, so its result is an input and
/// not a style-time decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ElementState(u32);

macro_rules! state_bits {
    ($($(#[$doc:meta])* $name:ident = $bit:expr, $set:ident;)*) => {
        impl ElementState {
            $(
                $(#[$doc])*
                pub const $name: Self = Self(1_u32 << $bit);
            )*

            /// Whether every bit of `other` is set here.
            #[must_use]
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }

            /// Both sets of bits.
            #[must_use]
            pub const fn union(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }

            /// The bits of `self` that are not in `other`.
            #[must_use]
            pub const fn without(self, other: Self) -> Self {
                Self(self.0 & !other.0)
            }

            /// Whether no bit is set.
            #[must_use]
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }

            /// The bits, for a side table or a log entry.
            #[must_use]
            pub const fn bits(self) -> u32 {
                self.0
            }

            /// A state from raw bits.
            #[must_use]
            pub const fn from_bits(bits: u32) -> Self {
                Self(bits)
            }

            $(
                #[doc = concat!("Sets or clears [`Self::", stringify!($name), "`].")]
                #[must_use]
                pub const fn $set(self, on: bool) -> Self {
                    if on { self.union(Self::$name) } else { self.without(Self::$name) }
                }
            )*
        }
    };
}

state_bits! {
    /// `:hover` — recomputed after layout from the recorded pointer
    /// position, so content moving under a still pointer changes it
    /// deterministically.
    HOVER = 0, with_hover;
    /// `:active`.
    ACTIVE = 1, with_active;
    /// `:focus`.
    FOCUS = 2, with_focus;
    /// `:focus-visible`, whose heuristic runs in the focus code.
    FOCUS_VISIBLE = 3, with_focus_visible;
    /// `:focus-within`.
    FOCUS_WITHIN = 4, with_focus_within;
    /// `:link` — an unvisited hyperlink.
    LINK = 5, with_link;
    /// `:visited`.
    VISITED = 6, with_visited;
    /// `:target`, set after navigation.
    TARGET = 7, with_target;
    /// `:enabled`.
    ENABLED = 8, with_enabled;
    /// `:disabled`.
    DISABLED = 9, with_disabled;
    /// `:checked`.
    CHECKED = 10, with_checked;
    /// `:indeterminate`.
    INDETERMINATE = 11, with_indeterminate;
    /// `:default`.
    DEFAULT = 12, with_default;
    /// `:required`.
    REQUIRED = 13, with_required;
    /// `:optional`.
    OPTIONAL = 14, with_optional;
    /// `:valid`.
    VALID = 15, with_valid;
    /// `:invalid`.
    INVALID = 16, with_invalid;
    /// `:in-range`.
    IN_RANGE = 17, with_in_range;
    /// `:out-of-range`.
    OUT_OF_RANGE = 18, with_out_of_range;
    /// `:read-only`.
    READ_ONLY = 19, with_read_only;
    /// `:read-write`.
    READ_WRITE = 20, with_read_write;
    /// `:placeholder-shown`.
    PLACEHOLDER_SHOWN = 21, with_placeholder_shown;
    /// `:autofill`, written by the shell.
    AUTOFILL = 22, with_autofill;
    /// `:defined`.
    DEFINED = 23, with_defined;
    /// `:open`.
    OPEN = 24, with_open;
    /// `:modal`.
    MODAL = 25, with_modal;
    /// `:fullscreen`.
    FULLSCREEN = 26, with_fullscreen;
    /// `:picture-in-picture`.
    PICTURE_IN_PICTURE = 27, with_picture_in_picture;
    /// `:muted`.
    MUTED = 28, with_muted;
    /// `:playing`, as against `:paused`.
    PLAYING = 29, with_playing;
    /// `:seeking`.
    SEEKING = 30, with_seeking;
    /// `:popover-open`.
    POPOVER_OPEN = 31, with_popover_open;
}

/// A read-only view of one document's tree, for the length of one
/// pass.
///
/// The tree does not change while a reader holds it. That is the
/// contract the parallel, phase-pure style pass rests on (ST 6.10), and
/// it is why nothing here is `&mut`.
pub trait Tree {
    /// How this implementation names a node.
    ///
    /// `Copy` and `Eq` because a matcher holds thousands of them and
    /// compares them constantly; nothing else is assumed.
    type Node: Copy + PartialEq;

    /// The document node.
    fn document(&self) -> Self::Node;

    /// Whether the document is an HTML document, which decides whether
    /// element and attribute names match case-insensitively.
    fn is_html_document(&self) -> bool;

    /// The node's identity.
    fn id(&self, node: Self::Node) -> NodeId;

    /// What kind of node it is.
    fn kind(&self, node: Self::Node) -> NodeKind;

    /// The parent, in the flat tree: a slotted node's parent is its
    /// slot's parent, not its host (DOM spec 6.4).
    fn parent(&self, node: Self::Node) -> Option<Self::Node>;

    /// The first child, in the flat tree.
    fn first_child(&self, node: Self::Node) -> Option<Self::Node>;

    /// The last child, in the flat tree.
    fn last_child(&self, node: Self::Node) -> Option<Self::Node>;

    /// The previous sibling, in the flat tree.
    fn previous_sibling(&self, node: Self::Node) -> Option<Self::Node>;

    /// The next sibling, in the flat tree.
    fn next_sibling(&self, node: Self::Node) -> Option<Self::Node>;

    /// An element's namespace, or `None` for a node that is not an
    /// element.
    fn namespace(&self, node: Self::Node) -> Option<&str>;

    /// An element's local name, or `None` for a node that is not an
    /// element.
    fn local_name(&self, node: Self::Node) -> Option<&str>;

    /// An attribute's value.
    ///
    /// This is the *one view* of an attribute: a value another crate
    /// overlaid — the SVG spec's animated-attribute overlay, SMIL's
    /// `animVal` — is what comes back here, so style reads whichever
    /// crate wrote last and never a second, staler copy (G1).
    fn attribute(&self, node: Self::Node, ns: AttrNs<'_>, local: &str) -> Option<&str>;

    /// The element's state bits.
    fn element_state(&self, node: Self::Node) -> ElementState;

    /// The text of a text node, or `None` for anything else.
    fn text(&self, node: Self::Node) -> Option<&str>;

    /// The value of the `id` attribute, which matching asks for often
    /// enough that an implementation will have it in a column.
    fn element_id(&self, node: Self::Node) -> Option<&str> {
        self.attribute(node, AttrNs::None, "id")
    }

    /// Whether the element carries `name` in its class list.
    ///
    /// Case sensitivity follows the document: quirks-mode HTML matches
    /// classes ASCII case-insensitively, and an implementation that
    /// tracks the mode overrides this.
    fn has_class(&self, node: Self::Node, name: &str, case_sensitive: bool) -> bool {
        let Some(value) = self.attribute(node, AttrNs::None, "class") else {
            return false;
        };
        value.split_ascii_whitespace().any(|c| {
            if case_sensitive {
                c == name
            } else {
                c.eq_ignore_ascii_case(name)
            }
        })
    }
}

/// Whether the node is an element.
pub fn is_element<T: Tree + ?Sized>(tree: &T, node: T::Node) -> bool {
    matches!(tree.kind(node), NodeKind::Element)
}

/// Whether the element is the document element — the `:root` of
/// Selectors 3.
pub fn is_root<T: Tree + ?Sized>(tree: &T, node: T::Node) -> bool {
    is_element(tree, node)
        && tree
            .parent(node)
            .is_some_and(|p| matches!(tree.kind(p), NodeKind::Document))
}

/// Whether the element is an HTML element in an HTML document, which
/// is what makes its name match case-insensitively.
pub fn is_html_element<T: Tree + ?Sized>(tree: &T, node: T::Node) -> bool {
    tree.is_html_document() && tree.namespace(node) == Some(ns::HTML)
}

/// Every child, in tree order.
pub fn children<T: Tree + ?Sized>(tree: &T, node: T::Node) -> Vec<T::Node> {
    let mut out = Vec::new();
    let mut at = tree.first_child(node);
    while let Some(n) = at {
        out.push(n);
        at = tree.next_sibling(n);
    }
    out
}

/// Every child that is an element, in tree order.
pub fn element_children<T: Tree + ?Sized>(tree: &T, node: T::Node) -> Vec<T::Node> {
    children(tree, node)
        .into_iter()
        .filter(|n| is_element(tree, *n))
        .collect()
}

/// Every ancestor, nearest first.
pub fn ancestors<T: Tree + ?Sized>(tree: &T, node: T::Node) -> Vec<T::Node> {
    let mut out = Vec::new();
    let mut at = tree.parent(node);
    while let Some(n) = at {
        out.push(n);
        at = tree.parent(n);
    }
    out
}

/// Every preceding sibling, nearest first.
pub fn preceding_siblings<T: Tree + ?Sized>(tree: &T, node: T::Node) -> Vec<T::Node> {
    let mut out = Vec::new();
    let mut at = tree.previous_sibling(node);
    while let Some(n) = at {
        out.push(n);
        at = tree.previous_sibling(n);
    }
    out
}

/// Every node of the subtree rooted at `node`, in tree order,
/// including `node` itself.
pub fn descendants<T: Tree + ?Sized>(tree: &T, node: T::Node) -> Vec<T::Node> {
    let mut out = Vec::new();
    let mut stack = alloc::vec![node];
    while let Some(n) = stack.pop() {
        out.push(n);
        let mut kids = children(tree, n);
        kids.reverse();
        stack.extend(kids);
    }
    out
}

/// Whether `node` has no element children and no non-empty text
/// children — Selectors 3's `:empty`.
///
/// "Non-empty" is the specification's: a text node of length zero does
/// not count, and a comment never counts.
pub fn is_empty_element<T: Tree + ?Sized>(tree: &T, node: T::Node) -> bool {
    children(tree, node)
        .into_iter()
        .all(|c| match tree.kind(c) {
            NodeKind::Element => false,
            NodeKind::Text => tree.text(c).is_none_or(str::is_empty),
            NodeKind::Comment
            | NodeKind::ProcessingInstruction
            | NodeKind::Document
            | NodeKind::DocumentType
            | NodeKind::DocumentFragment => true,
        })
}
