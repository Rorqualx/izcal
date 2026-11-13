//! Block layout, written from CSS 2.2's prose.
//!
//! Spec-literal on purpose. Every step below names the section it
//! comes from, and the order of the steps is the order the sections
//! give. This is the slow, obvious implementation whose answers the
//! production engine's caches, incremental paths and parallel paths
//! must equal (LY-D5, G42).
//!
//! What is here at LY-M0:
//!
//! - **§10.3.3** — the constraint equation, with the `auto` and
//!   over-constrained cases.
//! - **§10.4** — `min-width` and `max-width`, applied as the
//!   specification's three-step algorithm and not as a clamp.
//! - **§10.6.3** — `height: auto`, with its four ordered clauses.
//! - **§8.3.1** — collapsing margins, with the adjoining rules.

use hg_types::geometry::{LayoutUnit, PhysicalPoint, PhysicalRect, PhysicalSize};

use crate::style::{BoxStyle, Display, MaxSize, MinSize, Size};

/// A box the reference engine lays out: its style and its children.
///
/// Its **own** tree, not `hg-box-tree`'s (G41).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RefBox {
    /// A name, so a test can read a result.
    pub name: String,
    /// Its computed values.
    pub style: BoxStyle,
    /// Its children, in order.
    pub children: Vec<RefBox>,
    /// How many line boxes it holds, for §10.6.3's first clause.
    ///
    /// A number rather than a line list, because inline layout needs
    /// `hg-text`, which is TX-M1's. The clause it feeds is the one
    /// that matters to block layout, and taking the count as an input
    /// keeps the block algorithm honest about what it does not do.
    pub line_count: u32,
    /// The total height of those line boxes.
    pub line_height: LayoutUnit,
}

/// What a laid-out box came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefFragment {
    /// The box's name.
    pub name: String,
    /// Its border box, in the containing block's coordinates.
    pub rect: PhysicalRect,
    /// Its used margins, after collapsing.
    pub margin_top: LayoutUnit,
    /// Its used bottom margin, after collapsing.
    pub margin_bottom: LayoutUnit,
    /// Its children.
    pub children: Vec<RefFragment>,
}

/// The adjoining-margin summary of §8.3.1.
///
/// "When two or more margins collapse, the resulting margin width is
/// the maximum of the collapsing margins' widths." For negatives, "the
/// maximum of the absolute values of the negative adjoining margins is
/// deducted from the maximum of the positive adjoining margins" — so
/// two numbers are the whole state, and the result is their sum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Collapsed {
    /// The largest positive adjoining margin.
    pub positive: LayoutUnit,
    /// The most negative adjoining margin.
    pub negative: LayoutUnit,
}

impl Collapsed {
    /// Nothing adjoining.
    pub const EMPTY: Self = Self {
        positive: LayoutUnit::ZERO,
        negative: LayoutUnit::ZERO,
    };

    /// One more margin joined.
    #[must_use]
    pub const fn join(self, margin: LayoutUnit) -> Self {
        if margin.is_negative() {
            Self {
                negative: self.negative.min(margin),
                ..self
            }
        } else {
            Self {
                positive: self.positive.max(margin),
                ..self
            }
        }
    }

    /// Two summaries joined.
    #[must_use]
    pub const fn merge(self, other: Self) -> Self {
        Self {
            positive: self.positive.max(other.positive),
            negative: self.negative.min(other.negative),
        }
    }

    /// The collapsed margin this stands for.
    #[must_use]
    pub const fn value(self) -> LayoutUnit {
        self.positive.add(self.negative)
    }

    /// Whether nothing has joined.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.positive.is_zero() && self.negative.is_zero()
    }
}

/// The used widths and margins of §10.3.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsedWidth {
    /// The content width.
    pub width: LayoutUnit,
    /// The used left margin.
    pub margin_left: LayoutUnit,
    /// The used right margin.
    pub margin_right: LayoutUnit,
}

/// Whether the containing block's `direction` is left-to-right, which
/// decides which margin §10.3.3 ignores when over-constrained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dir {
    /// `ltr`.
    #[default]
    Ltr,
    /// `rtl`.
    Rtl,
}

/// §10.3.3, block-level non-replaced elements in normal flow.
///
/// The constraint equation is
///
/// ```text
/// margin-left + border-left + padding-left + width
///   + padding-right + border-right + margin-right
///   = width of containing block
/// ```
///
/// and the rules are, in the specification's own order:
///
/// 1. If `width` is not `auto` and the non-`auto` parts already exceed
///    the containing block's width, "any 'auto' values for
///    'margin-left' or 'margin-right' are, for the following rules,
///    treated as zero".
/// 2. "If there is exactly one value specified as 'auto', its used
///    value follows from the equality."
/// 3. Over-constrained: "the specified value of margin-right is
///    ignored and the value is calculated so as to make the equality
///    true" — `margin-left` instead when the direction is `rtl`.
/// 4. "If 'width' is set to 'auto', any other 'auto' values become '0'
///    and 'width' follows from the resulting equality."
/// 5. "If both 'margin-left' and 'margin-right' are 'auto', their used
///    values are equal", which centres the box.
#[must_use]
pub fn used_width(style: &BoxStyle, containing: LayoutUnit, dir: Dir) -> UsedWidth {
    let basis = Some(containing);
    let border = style.border();
    let padding = style.padding(basis);
    let fixed = border.horizontal().add(padding.horizontal());

    let mut left = style.margin_left;
    let mut right = style.margin_right;
    let width = style.width.resolve(basis);

    // Step 1: an over-wide box with `auto` margins treats them as zero.
    if let Some(w) = width {
        let non_auto = fixed
            .add(w)
            .add(left.resolve(basis).unwrap_or(LayoutUnit::ZERO))
            .add(right.resolve(basis).unwrap_or(LayoutUnit::ZERO));
        if non_auto.raw() > containing.raw() {
            if left.is_auto() {
                left = Size::Length(LayoutUnit::ZERO);
            }
            if right.is_auto() {
                right = Size::Length(LayoutUnit::ZERO);
            }
        }
    }

    let ml = left.resolve(basis);
    let mr = right.resolve(basis);
    let space = containing.sub(fixed);

    match (ml, width, mr) {
        // Step 4: `width: auto` makes every other `auto` zero.
        (_, None, _) => {
            let ml = ml.unwrap_or(LayoutUnit::ZERO);
            let mr = mr.unwrap_or(LayoutUnit::ZERO);
            UsedWidth {
                width: space.sub(ml).sub(mr).max(LayoutUnit::ZERO),
                margin_left: ml,
                margin_right: mr,
            }
        }
        // Step 5: both margins `auto` — equal, so the box is centred.
        (None, Some(w), None) => {
            let spare = space.sub(w);
            let half = LayoutUnit::from_raw(spare.raw().saturating_div(2));
            UsedWidth {
                width: w,
                margin_left: half,
                // The remainder goes to the right margin, so the three
                // still sum to the containing block on an odd spare.
                margin_right: spare.sub(half),
            }
        }
        // Step 2: exactly one `auto`.
        (None, Some(w), Some(mr)) => UsedWidth {
            width: w,
            margin_left: space.sub(w).sub(mr),
            margin_right: mr,
        },
        (Some(ml), Some(w), None) => UsedWidth {
            width: w,
            margin_left: ml,
            margin_right: space.sub(w).sub(ml),
        },
        // Step 3: over-constrained. One margin is ignored and
        // recomputed; which one is the containing block's direction.
        (Some(ml), Some(w), Some(mr)) => match dir {
            Dir::Ltr => UsedWidth {
                width: w,
                margin_left: ml,
                margin_right: space.sub(w).sub(ml),
            },
            Dir::Rtl => UsedWidth {
                width: w,
                margin_left: space.sub(w).sub(mr),
                margin_right: mr,
            },
        },
    }
}

/// §10.4, `min-width` and `max-width`.
///
/// The specification's own three steps, not a clamp: "The tentative
/// used width is calculated (without 'min-width' and 'max-width')";
/// if it exceeds `max-width`, "the rules above are applied again"
/// using `max-width` as the computed `width`; if the result is below
/// `min-width`, they are applied again using `min-width`.
///
/// The difference from a clamp is the **margins**: re-running the
/// rules recomputes an `auto` margin against the new width, so a
/// centred box whose width is cut by `max-width` stays centred. A
/// clamp would leave the margins where the tentative width put them.
#[must_use]
pub fn used_width_constrained(style: &BoxStyle, containing: LayoutUnit, dir: Dir) -> UsedWidth {
    let basis = Some(containing);
    let tentative = used_width(style, containing, dir);

    let max = style.max_width.resolve(basis);
    let mut result = tentative;
    if let Some(m) = max
        && tentative.width.raw() > m.raw()
    {
        let mut fixed = *style;
        fixed.width = Size::Length(m);
        result = used_width(&fixed, containing, dir);
    }

    let min = style.min_width.resolve(basis);
    if result.width.raw() < min.raw() {
        let mut fixed = *style;
        fixed.width = Size::Length(min);
        result = used_width(&fixed, containing, dir);
    }
    result
}

/// `min-height` and `max-height`, applied as §10.7's clamp.
#[must_use]
pub fn constrain_height(
    height: LayoutUnit,
    min: MinSize,
    max: MaxSize,
    basis: Option<LayoutUnit>,
) -> LayoutUnit {
    let mut h = height;
    if let Some(m) = max.resolve(basis)
        && h.raw() > m.raw()
    {
        h = m;
    }
    let lower = min.resolve(basis);
    if h.raw() < lower.raw() {
        h = lower;
    }
    h
}

/// What a child's layout handed back to its parent.
struct Laid {
    fragment: RefFragment,
    /// The margin at its block-start edge, still open to collapse
    /// with what came before.
    leading: Collapsed,
    /// The margin at its block-end edge, still open.
    trailing: Collapsed,
    /// Whether the box collapsed through: its own top and bottom
    /// margins are adjoining, so it contributes no height at all.
    collapsed_through: bool,
    /// Its border-box height, which is zero when it collapsed through.
    border_height: LayoutUnit,
}

/// Lays out `root` in a containing block of `containing` width.
///
/// The root's own margins do not collapse (§8.3.1: "Margins of the
/// root element's box do not collapse"), so the walk starts with a
/// closed margin run.
#[must_use]
pub fn layout(root: &RefBox, containing: LayoutUnit, dir: Dir) -> RefFragment {
    let laid = layout_box(root, containing, dir, true);
    laid.fragment
}

fn layout_box(b: &RefBox, containing: LayoutUnit, dir: Dir, is_root: bool) -> Laid {
    let basis = Some(containing);
    let width = used_width_constrained(&b.style, containing, dir);
    let border = b.style.border();
    let padding = b.style.padding(basis);

    // Whether this box's margins can collapse with its children's.
    // §8.3.1: "Margins of elements that establish new block formatting
    // contexts do not collapse with their in-flow children", and
    // "Margins of the root element's box do not collapse".
    let seals = is_root || b.style.establishes_bfc();
    // "no line boxes, no clearance, no padding and no border separate
    // them".
    let top_separated = !border.top.is_zero() || !padding.top.is_zero();
    let bottom_separated = !border.bottom.is_zero() || !padding.bottom.is_zero();
    let has_lines = b.line_count > 0;

    let own_top = b
        .style
        .margin_top
        .resolve(basis)
        .unwrap_or(LayoutUnit::ZERO);
    let own_bottom = b
        .style
        .margin_bottom
        .resolve(basis)
        .unwrap_or(LayoutUnit::ZERO);

    let content_width = width.width;
    let inner_left = border.left.add(padding.left);
    let inner_top = border.top.add(padding.top);

    // The box's own leading margin. Children's leading margins join it
    // only while the run is open — which it is not for a box that
    // seals, has a top border or padding, or holds line boxes.
    let mut leading = Collapsed::EMPTY.join(own_top);
    let mut leading_open = !seals && !top_separated && !has_lines;

    let mut children: Vec<RefFragment> = Vec::new();
    // The position of the next child's border-box top, inside the
    // content box, before the open margin run is added.
    let mut y = LayoutUnit::ZERO;
    let mut run = Collapsed::EMPTY;
    let mut placed_content = has_lines;

    for child in &b.children {
        if matches!(child.style.display, Display::None) {
            continue;
        }
        let laid = layout_box(child, content_width, dir, false);
        // Where the child's border box goes: after the run its leading
        // margin joins.
        let joined = if leading_open {
            // Still inside the box's own leading run, which sits
            // outside the box; so the child starts at the top edge.
            leading = leading.merge(laid.leading);
            Collapsed::EMPTY
        } else {
            run.merge(laid.leading)
        };
        let top = y.add(joined.value());

        let mut fragment = laid.fragment;
        fragment.rect = fragment
            .rect
            .translate(inner_left.add(fragment.rect.origin.x), inner_top.add(top));
        children.push(fragment);

        if laid.collapsed_through {
            // It adds no height, and both its margins join the run on
            // either side of it.
            if leading_open {
                leading = leading.merge(laid.trailing);
            } else {
                run = joined.merge(laid.trailing);
            }
            continue;
        }
        leading_open = false;
        placed_content = true;
        y = top.add(laid.border_height);
        run = laid.trailing;
    }

    // §10.6.3: the height when `height` is `auto`.
    let auto_height = if has_lines {
        // 1. "the bottom edge of the last line box".
        b.line_height
    } else if placed_content {
        // 2. "the bottom edge of the bottom (possibly collapsed)
        //    margin of its last in-flow child", when that margin does
        //    not collapse with this box's bottom margin; otherwise
        // 3. "the bottom border edge of the last in-flow child".
        if seals || bottom_separated {
            y.add(run.value())
        } else {
            y
        }
    } else {
        // 4. "zero, otherwise".
        LayoutUnit::ZERO
    };

    let specified = b.style.height.resolve(basis);
    let content_height = constrain_height(
        specified.unwrap_or(auto_height),
        b.style.min_height,
        b.style.max_height,
        basis,
    );

    // §8.3.1's fourth pair: "top and bottom margins of a box that does
    // not establish a new block formatting context, and that has zero
    // computed 'min-height', zero or 'auto' computed 'height', and no
    // in-flow children".
    let collapsed_through = !seals
        && !top_separated
        && !bottom_separated
        && !has_lines
        && content_height.is_zero()
        && matches!(b.style.min_height, MinSize::Zero)
        && !placed_content;

    let border_height = content_height
        .add(border.vertical())
        .add(padding.vertical());

    // The trailing run. The last child's bottom margin reaches through
    // only when nothing separates it and this box's height is
    // automatic — §8.3.1's third pair names that condition.
    let mut trailing = Collapsed::EMPTY.join(own_bottom);
    if !seals && !bottom_separated && !has_lines && specified.is_none() {
        trailing = trailing.merge(run);
    }
    if collapsed_through {
        // Its own two margins are adjoining, so what it hands upward
        // is one run holding both.
        trailing = trailing.merge(leading);
    }

    let outer_height = if collapsed_through {
        LayoutUnit::ZERO
    } else {
        border_height
    };

    let fragment = RefFragment {
        name: b.name.clone(),
        rect: PhysicalRect::new(
            PhysicalPoint::new(width.margin_left, LayoutUnit::ZERO),
            PhysicalSize::new(
                content_width
                    .add(border.horizontal())
                    .add(padding.horizontal()),
                outer_height,
            ),
        ),
        margin_top: leading.value(),
        margin_bottom: trailing.value(),
        children,
    };

    Laid {
        fragment,
        leading,
        trailing,
        collapsed_through,
        border_height: outer_height,
    }
}
