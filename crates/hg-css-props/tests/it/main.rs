//! `hg-css-props`' tests: that the generated table answers the
//! questions the cascade and the parser ask of it.
//!
//! Nothing here restates the table's contents — that is what the
//! overlay and `hg-cssgen`'s cross-check are for. These are about the
//! lookups: that a name finds its property whatever its case, that a
//! legacy alias lands on the property it stands for, that a shorthand's
//! longhands are properties and not a dangling index, and that no row
//! is reachable that the table does not have.

use hg_css_props::{Affects, ComputedKind, Property};

#[test]
fn p01_a_name_finds_its_property() {
    assert_eq!(Property::from_name("color"), Some(Property::COLOR));
    assert_eq!(
        Property::from_name("margin-top"),
        Some(Property::MARGIN_TOP)
    );
    assert_eq!(Property::from_name("nonesuch"), None);
    assert_eq!(Property::from_name(""), None);
    // A custom property is not one of these.
    assert_eq!(Property::from_name("--x"), None);
}

#[test]
fn p02_property_names_are_ascii_case_insensitive() {
    for spelling in ["color", "Color", "COLOR", "cOlOr"] {
        assert_eq!(
            Property::from_name(spelling),
            Some(Property::COLOR),
            "{spelling} should find color"
        );
    }
    assert_eq!(
        Property::from_name("-WebKit-Text-Stroke-Color"),
        Property::from_name("-webkit-text-stroke-color")
    );
}

#[test]
fn p03_a_legacy_alias_lands_on_its_property() {
    // The Compatibility Standard's second spellings. The cascade never
    // sees two names for one thing.
    let found = Property::from_name("-webkit-align-content").unwrap();
    assert_eq!(found, Property::ALIGN_CONTENT);
    assert_eq!(found.name(), "align-content");
    // font-stretch is the alias; font-width is the property.
    assert_eq!(
        Property::from_name("font-stretch"),
        Some(Property::FONT_WIDTH)
    );
}

#[test]
fn p04_every_name_in_the_index_resolves_to_a_row_with_a_name() {
    // The index is generated beside the rows; a lookup that landed past
    // the end would come back with the absent row, whose name is empty.
    for p in Property::all() {
        assert!(!p.name().is_empty(), "{p:?} has no name");
        assert_eq!(
            Property::from_name(p.name()),
            Some(p),
            "{} does not find itself",
            p.name()
        );
    }
}

#[test]
fn p05_the_table_is_the_size_the_pin_says() {
    assert_eq!(Property::COUNT, 758);
    assert_eq!(Property::all().count(), Property::COUNT);
}

#[test]
fn p06_a_shorthand_expands_to_properties() {
    let margin = Property::MARGIN;
    assert!(margin.is_shorthand());
    let names: Vec<&str> = margin.longhands().iter().map(|p| p.name()).collect();
    assert_eq!(
        names,
        ["margin-top", "margin-right", "margin-bottom", "margin-left"]
    );
    assert!(!Property::MARGIN_TOP.is_shorthand());
    assert!(Property::MARGIN_TOP.longhands().is_empty());
}

#[test]
fn p07_no_longhand_is_a_dangling_index() {
    for p in Property::all() {
        for l in p.longhands() {
            assert!(
                !l.name().is_empty(),
                "{}'s longhand list has an index the table does not have",
                p.name()
            );
            assert_ne!(*l, p, "{} is its own longhand", p.name());
        }
    }
}

#[test]
fn p08_a_shorthand_does_not_inherit_as_itself() {
    // 76 shorthands say "see individual properties" in webref; the
    // overlay answers `no` for all of them, because the cascade stores
    // the longhands and they carry their own answer.
    for p in Property::all().filter(|p| p.is_shorthand()) {
        assert!(
            !p.inherits(),
            "{} is a shorthand and should not inherit",
            p.name()
        );
    }
}

#[test]
fn p09_inheritance_is_the_one_the_cascade_needs() {
    assert!(Property::COLOR.inherits());
    assert!(Property::FONT_SIZE.inherits());
    assert!(Property::VISIBILITY.inherits());
    assert!(!Property::WIDTH.inherits());
    assert!(!Property::BACKGROUND_COLOR.inherits());
    assert!(!Property::OPACITY.inherits());
}

#[test]
fn p10_the_shorthand_kind_is_the_one_with_no_value_of_its_own() {
    // Not every shorthand classifies as `shorthand`: seven carry a
    // computed value of their own at the pin, `background-position`
    // among them, and `all` has no longhand list at all while saying
    // "see individual properties". The kind follows the specification's
    // text, not the presence of a longhand list.
    assert_eq!(
        Property::MARGIN.computed_kind(),
        Some(ComputedKind::Shorthand)
    );
    assert_eq!(Property::ALL.computed_kind(), Some(ComputedKind::Shorthand));
    assert!(
        !Property::ALL.is_shorthand(),
        "webref gives `all` no longhand list"
    );
    assert_eq!(
        Property::BACKGROUND_POSITION.computed_kind(),
        Some(ComputedKind::Length),
        "a shorthand with a computed value of its own keeps it"
    );
    assert_eq!(Property::COLOR.computed_kind(), Some(ComputedKind::Color));
    assert_eq!(Property::WIDTH.computed_kind(), Some(ComputedKind::Length));
    assert_eq!(
        Property::OPACITY.computed_kind(),
        Some(ComputedKind::Number)
    );
}

#[test]
fn p11_only_the_prose_kind_names_a_hook() {
    let mut hooks = 0_usize;
    for p in Property::all() {
        if p.computed_kind() == Some(ComputedKind::Prose) {
            assert!(
                p.prose_hook().is_some(),
                "{} is prose and names no hook",
                p.name()
            );
            hooks = hooks.saturating_add(1);
        } else {
            assert_eq!(p.prose_hook(), None, "{} names a hook it may not", p.name());
        }
    }
    assert!(hooks > 0, "no property reached the prose kind at all");
    assert_eq!(Property::DISPLAY.prose_hook(), Some("display"));
}

#[test]
fn p12_affects_is_a_set_that_says_what_it_knows() {
    assert!(Property::WIDTH.affects().has(Affects::LAYOUT));
    assert!(Property::COLOR.affects().has(Affects::PAINT));
    assert!(Property::OPACITY.affects().has(Affects::COMPOSITE));
    assert!(!Property::WIDTH.affects().is_undecided());
    // NOTHING is the empty set, so `has` is the wrong question for it.
    assert!(Affects::LAYOUT.has(Affects::NOTHING));
    assert!(Affects::NOTHING.is_nothing());
    assert!(!Affects::LAYOUT.is_nothing());
    // The two that stand alone are not each other.
    assert!(!Affects::UNDECIDED.is_nothing());
    assert!(Affects::UNDECIDED.is_undecided());
}

#[test]
fn p13_a_property_with_no_initial_value_says_so() {
    // Nine properties at the pin: their specifications state it in a
    // form @webref/css did not extract. The table says None rather
    // than inventing one.
    let missing: Vec<&str> = Property::all()
        .filter(|p| p.initial().is_none())
        .map(|p| p.name())
        .collect();
    assert!(
        missing.contains(&"stop-color"),
        "stop-color should have no initial value at this pin, found {missing:?}"
    );
    assert_eq!(Property::COLOR.initial(), Some("CanvasText"));
}

#[test]
fn p14_every_kind_is_reachable_and_named() {
    for k in ComputedKind::ALL {
        assert!(!k.name().is_empty());
        assert!(
            Property::all().any(|p| p.computed_kind() == Some(k)),
            "no property has the kind {}",
            k.name()
        );
    }
}

#[cfg(feature = "reference")]
#[test]
fn p15_the_reference_data_lines_up_with_the_rows() {
    use hg_css_props::reference;
    assert_eq!(reference::grammar(Property::COLOR), Some("<color>"));
    assert!(reference::spec_url(Property::COLOR).contains("css-color-4"));
    for p in Property::all() {
        assert!(
            !reference::spec_url(p).is_empty(),
            "{} has no defining URL",
            p.name()
        );
    }
}
