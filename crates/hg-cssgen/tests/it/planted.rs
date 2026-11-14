//! The planted disagreement, and its twin.
//!
//! ST-M0 asks that `hg-cssgen` "imports webref with the overlay and
//! fails on a planted disagreement". A check that only ever passes
//! proves nothing: a cross-check that compared nothing, or that was
//! quietly disabled, would pass the real tree just as happily. So every
//! case here is a pair, the discipline JS-10 established — **the plant
//! is refused, and the same tree without the plant passes.**
//!
//! The plants are made in memory, against the real overlay and the real
//! pinned `@webref/css`, one field at a time. Each names the field it
//! moved and asserts that exactly that field is reported, so a check
//! that failed everything for the wrong reason would not pass either.
//!
//! The data is pinned and fetched rather than committed, so these are
//! skipped when it is not there. `IZCAL_REQUIRE_CSS_DATA=1` makes that
//! an error instead, which is what CI sets.

use std::path::{Path, PathBuf};

use hg_cssgen::check::against_webref;
use hg_cssgen::syntax::{Affect, AliasDecl, ComputedDecl, PropertyDecl};
use hg_cssgen::webref::Catalogue;
use hg_cssgen::{Model, cli};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The real overlay and the real catalogue, or `None` when the pinned
/// data is not in the cache.
fn real() -> Option<(Model, Catalogue)> {
    let m = cli::build(&root()).expect("the overlay checks");
    match cli::catalogue().expect("the catalogue parses") {
        Some(cat) => Some((m, cat)),
        None => {
            assert!(
                std::env::var_os("IZCAL_REQUIRE_CSS_DATA").is_none(),
                "IZCAL_REQUIRE_CSS_DATA=1 but {}",
                cli::missing_data_message()
            );
            println!("note: {}; skipped", cli::missing_data_message());
            None
        }
    }
}

/// Mutable access to one property of a copy of the model.
fn with<F: FnOnce(&mut PropertyDecl)>(m: &Model, name: &str, f: F) -> Model {
    let mut copy = m.clone();
    let p = copy
        .properties
        .iter_mut()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("{name} has no block"));
    f(p);
    copy
}

/// Asserts that a planted model is refused, that the difference
/// reported is the one planted, and that the model it was planted into
/// was itself clean.
fn plant(control: &Model, cat: &Catalogue, planted: &Model, subject: &str, field: &str) {
    assert!(
        against_webref(control, cat).is_empty(),
        "the twin without the plant must pass, or the plant proves nothing"
    );
    let found = against_webref(planted, cat);
    assert_eq!(
        found.len(),
        1,
        "exactly the planted difference should be found: {found:?}"
    );
    let d = found.first().unwrap();
    assert_eq!(d.subject, subject, "{d}");
    assert_eq!(d.field, field, "{d}");
}

#[test]
fn st_m0_the_real_overlay_and_the_pinned_webref_agree() {
    let Some((m, cat)) = real() else { return };
    let found = against_webref(&m, &cat);
    assert!(found.is_empty(), "{}", cli::report(&found));
    println!(
        "hg-cssgen: {} properties, {} aliases, {} computed-value texts, 0 disagreements",
        m.properties.len(),
        m.aliases.len(),
        m.computed.len()
    );
}

#[test]
fn pd01_a_changed_initial_value_is_caught() {
    let Some((m, cat)) = real() else { return };
    let planted = with(&m, "color", |p| p.initial = Some("red".to_owned()));
    plant(&m, &cat, &planted, "color", "initial");
}

#[test]
fn pd02_a_changed_inheritance_is_caught() {
    let Some((m, cat)) = real() else { return };
    // The restated *text*, which is what webref carries. Izcal's own
    // `inherits` answer has no counterpart there and is not compared.
    let planted = with(&m, "color", |p| p.inherited = Some("no".to_owned()));
    plant(&m, &cat, &planted, "color", "inherited");
}

#[test]
fn pd03_izcals_own_inherits_answer_is_not_compared() {
    let Some((m, cat)) = real() else { return };
    let planted = with(&m, "color", |p| p.inherits = Some(false));
    assert!(
        against_webref(&planted, &cat).is_empty(),
        "`inherits` is Izcal's decision; webref has no opinion to disagree with"
    );
}

#[test]
fn pd04_a_changed_grammar_is_caught() {
    let Some((m, cat)) = real() else { return };
    let planted = with(&m, "color", |p| {
        p.syntax = Some("<color> | auto".to_owned())
    });
    plant(&m, &cat, &planted, "color", "syntax");
}

#[test]
fn pd05_a_changed_animation_type_is_caught() {
    let Some((m, cat)) = real() else { return };
    let planted = with(&m, "color", |p| p.animation = Some("discrete".to_owned()));
    plant(&m, &cat, &planted, "color", "animation");
}

#[test]
fn pd06_a_changed_computed_value_text_is_caught() {
    let Some((m, cat)) = real() else { return };
    // Moved to another text the overlay already classifies, so the
    // classification is still exhaustive and only the property's own
    // restatement differs. A check that only compared the kind would
    // miss this, and the kind is the same on both sides.
    let planted = with(&m, "color", |p| {
        p.computed = Some("as specified".to_owned())
    });
    plant(&m, &cat, &planted, "color", "computed");
}

#[test]
fn pd07_a_changed_defining_url_is_caught() {
    let Some((m, cat)) = real() else { return };
    let planted = with(&m, "color", |p| {
        p.href = "https://example.invalid/#propdef-color".to_owned();
    });
    plant(&m, &cat, &planted, "color", "href");
}

#[test]
fn pd08_a_changed_longhand_list_is_caught() {
    let Some((m, cat)) = real() else { return };
    let planted = with(&m, "margin", |p| {
        p.longhands.truncate(3);
    });
    plant(&m, &cat, &planted, "margin", "longhands");
}

#[test]
fn pd09_a_property_webref_added_is_caught() {
    let Some((m, cat)) = real() else { return };
    let mut planted = m.clone();
    planted.properties.retain(|p| p.name != "color");
    plant(&m, &cat, &planted, "color", "the property itself");
}

#[test]
fn pd10_a_property_webref_dropped_is_caught() {
    let Some((m, cat)) = real() else { return };
    let mut planted = m.clone();
    planted.properties.push(PropertyDecl {
        name: "izcal-invented".to_owned(),
        href: "https://example.invalid/#propdef-izcal-invented".to_owned(),
        inherits: Some(false),
        affects: vec![Affect::Unknown],
        ..PropertyDecl::default()
    });
    plant(&m, &cat, &planted, "izcal-invented", "the property itself");
}

#[test]
fn pd11_a_changed_alias_is_caught() {
    let Some((m, cat)) = real() else { return };
    let mut planted = m.clone();
    let a = planted
        .aliases
        .iter_mut()
        .find(|a| a.name == "-webkit-align-content")
        .unwrap();
    a.target = "color".to_owned();
    plant(&m, &cat, &planted, "-webkit-align-content", "alias target");
}

#[test]
fn pd12_an_alias_webref_dropped_is_caught() {
    let Some((m, cat)) = real() else { return };
    let mut planted = m.clone();
    planted.aliases.push(AliasDecl {
        name: "-izcal-color".to_owned(),
        target: "color".to_owned(),
        line: 0,
    });
    plant(&m, &cat, &planted, "-izcal-color", "the alias itself");
}

#[test]
fn pd13_an_unclassified_computed_value_text_is_caught() {
    let Some((m, cat)) = real() else { return };
    let mut planted = m.clone();
    planted.computed.retain(|c| c.text != "as specified");
    // The text is still carried by webref and by 123 properties'
    // restatements, so dropping the classification leaves them with no
    // conversion. One difference: the text itself.
    plant(&m, &cat, &planted, "as specified", "a computed-value text");
}

#[test]
fn pd14_a_classification_of_a_text_webref_dropped_is_caught() {
    let Some((m, cat)) = real() else { return };
    let mut planted = m.clone();
    planted.computed.push(ComputedDecl {
        text: "a form the specifications stopped using".to_owned(),
        kind: "identity".to_owned(),
        line: 0,
    });
    plant(
        &m,
        &cat,
        &planted,
        "a form the specifications stopped using",
        "a computed-value text",
    );
}

#[test]
fn pd15_the_report_says_what_to_do_about_it() {
    let Some((m, cat)) = real() else { return };
    let planted = with(&m, "color", |p| p.initial = Some("red".to_owned()));
    let text = cli::report(&against_webref(&planted, &cat));
    assert!(text.contains("color initial"), "{text}");
    assert!(
        text.contains("ST-7"),
        "the report should name the rule: {text}"
    );
    assert!(
        text.contains("Never regenerate the overlay"),
        "the report should say what not to do: {text}"
    );
}
