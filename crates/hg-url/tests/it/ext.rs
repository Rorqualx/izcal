//! The `ext://` scheme Izcal registers (style spec 6.1, EX-D4).
//!
//! Registered in `hg-url` and nowhere else, because G31 allows the
//! browser one URL parser: a check elsewhere that special-cased
//! `ext://` would be the divergence G31 exists to prevent.

use hg_url::{Host, Url};

fn p(input: &str) -> Option<Url> {
    Url::parse(input)
}

#[test]
fn e01_an_extension_url_has_a_host_and_a_path() {
    let Some(u) = p("ext://abcdef/page.html") else {
        panic!("that parses");
    };
    assert_eq!(u.scheme, "ext");
    assert_eq!(u.host, Some(Host::Opaque("abcdef".to_owned())));
    assert_eq!(u.serialize_path(), "/page.html");
    assert_eq!(u.serialize(false), "ext://abcdef/page.html");
}

#[test]
fn e02_it_needs_no_slashes() {
    // `ext:abc/page` reaches the same URL as `ext://abc/page`.
    for input in ["ext:abcdef/page.html", "ext:/abcdef/page.html"] {
        let Some(u) = p(input) else {
            panic!("{input} parses");
        };
        assert_eq!(u.host, Some(Host::Opaque("abcdef".to_owned())), "{input}");
        assert_eq!(u.serialize(false), "ext://abcdef/page.html", "{input}");
    }
}

#[test]
fn e03_it_is_never_opaque_path() {
    // Without the registration this would be an opaque path, and then
    // `ext://<id>` would not be one origin: there would be no host to
    // derive a site key from.
    let Some(u) = p("ext:abcdef") else {
        panic!("that parses");
    };
    assert!(!u.has_opaque_path());
    assert_eq!(u.host, Some(Host::Opaque("abcdef".to_owned())));
}

#[test]
fn e04_it_is_not_a_special_scheme() {
    let Some(u) = p("ext://abcdef/a\\b") else {
        panic!("that parses");
    };
    // A backslash stays a path character, as for any non-special
    // scheme; registering a scheme does not make it special.
    assert_eq!(u.serialize_path(), "/a\\b");
    assert!(!u.is_special());
    // And it has no default port, so one that is written is kept.
    let Some(u) = p("ext://abcdef:80/") else {
        panic!("that parses");
    };
    assert_eq!(u.port, Some(80));
}

#[test]
fn e05_the_id_is_an_opaque_host_and_never_a_domain() {
    // "never becomes a registrable domain": the ID is not run through
    // the domain parser, so it is not lower-cased, not IDNA-mapped and
    // not read as an address.
    let Some(u) = p("ext://AbCdEf/") else {
        panic!("that parses");
    };
    assert_eq!(u.host, Some(Host::Opaque("AbCdEf".to_owned())));
    let Some(u) = p("ext://1.2.3.4/") else {
        panic!("that parses");
    };
    assert_eq!(u.host, Some(Host::Opaque("1.2.3.4".to_owned())));
}

#[test]
fn e06_registering_a_scheme_changes_nothing_for_any_other() {
    // The twin: the same inputs with an unregistered scheme take the
    // standard's own route, opaque path and all.
    let Some(u) = p("other:abcdef") else {
        panic!("that parses");
    };
    assert!(u.has_opaque_path());
    assert_eq!(u.host, None);
    let Some(u) = p("other:/abcdef/page.html") else {
        panic!("that parses");
    };
    assert_eq!(u.host, None);
    assert_eq!(u.serialize_path(), "/abcdef/page.html");
}

#[test]
fn e07_an_empty_extension_id_is_refused() {
    // `ext://<id>` is one origin and one site key (HGBA-2 I-1), so an
    // empty id would be an origin belonging to no extension.
    for input in ["ext://", "ext:", "ext://?x", "ext://#x", "ext://:80/"] {
        assert_eq!(p(input), None, "{input}");
    }
    // Extra slashes are eaten, as they are for a special scheme — the
    // standard's own `https://////example.com///` — so `ext:///page`
    // names the extension `page` and is not an empty id.
    let Some(u) = p("ext:///page") else {
        panic!("that parses");
    };
    assert_eq!(u.host, Some(Host::Opaque("page".to_owned())));
    // A non-special scheme that is not registered keeps the standard's
    // own answer, where an empty host is allowed.
    let Some(u) = p("other://") else {
        panic!("that parses");
    };
    assert_eq!(u.host, Some(Host::Empty));
}
