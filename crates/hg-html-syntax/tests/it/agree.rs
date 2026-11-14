//! The two back ends against each other.
//!
//! They are compiled from the one description, so this does not check
//! the description — html5lib does that. It checks the *generator*,
//! whose two back ends share no code: a mistake in either one shows up
//! here as a disagreement.

use hg_html_syntax::generated::states;
use hg_html_syntax::{Back, State, Token, Tokenizer};

use crate::{Collect, run_driven, run_foreign};

/// Inputs chosen to reach every state the descriptions declare, which
/// the test below checks they do.
const CORPUS: &[&str] = &[
    "",
    "hello",
    "a\0b",
    "<p>text</p>",
    "<P CLASS=x ID='y' data-z=\"w\">",
    "<br/><hr />",
    "<a b=1 b=2 b=3>",
    "<a b= >",
    "<a b c>",
    "<a b/c>",
    "<a =b>",
    "<a b=`c`>",
    "<a b=\"c<d>e\">",
    "</>",
    "</ >",
    "</p attr>",
    "<?php echo 1; ?>",
    "<!>",
    "<!->",
    "<!-->",
    "<!--->",
    "<!---->",
    "<!-- a -- b -->",
    "<!--<!-->",
    "<!--<!--->",
    "<!--<<!-x-->",
    "<!--a--!>b",
    "<!--a--!",
    "<!--a-",
    "<!DOCTYPE>",
    "<!DOCTYPE html>",
    "<!doctype HTML>",
    "<!DOCTYPE\0html>",
    "<!DOCTYPEhtml>",
    "<!DOCTYPE html PUBLIC 'p' 's'>",
    "<!DOCTYPE html PUBLIC \"p\" \"s\">",
    "<!DOCTYPE html PUBLIC \"p",
    "<!DOCTYPE html PUBLIC'p'>",
    "<!DOCTYPE html SYSTEM \"s\">",
    "<!DOCTYPE html SYSTEM\"s\">",
    "<!DOCTYPE html public 'p' 's' x>",
    "<!DOCTYPE html PUBLIC 'p'\"s\">",
    "<!DOCTYPE html FOO>",
    "<!DOCTYPE html PUBLIC 'p",
    "<!DOCTYPE html SYSTEM 's",
    "<![CDATA[x]]>",
    "<![CDATA[]]]>",
    "<![CDATA[",
    "&amp;&AMP&notit;&abcd;&#;&#x;&#38;&#X26;&#x110000;&#xD800;&#0;&#x80;",
    "&",
    "&#",
    "&#x",
    "<a b='&amp'>",
    "<a b='&notit'>",
    "<a b='&notit;'>",
    "<a b=&amp>",
    "<title>a</b>b</title>",
    "<xmp></xm></xmp>",
    "<script>a<b</script>",
    "<script><!--a--></script>",
    "<script><!--<script>a</script>b</script>",
    "<script><!--a</script>b</script>",
    "<script><!--<script>a</scr</script>",
    "<script><!--<script>a-</script>",
    "<script><!--<script>a--></script>",
    "<script><!--<script>a<</script>",
    "<script><!--<script></scr</script>",
    "<script><!--<SCRIPT>x</SCRIPT >",
    "<script><!-<a",
    "<script></scr ipt>",
    "<plaintext>a<b>c",
    "<textarea>a</textarea>",
    "<style>a</style>",
    "<a",
    "</a",
    "<",
    "<!--",
    "<a b",
    "<a b=",
    "<a b=\"",
    "<a b='",
];

/// The cases that need the adjusted current node outside the HTML
/// namespace, which is the tokenizer's one question to a tree builder.
const FOREIGN: &[&str] = &[
    "<![CDATA[x]]>",
    "<![CDATA[]]]>",
    "<![CDATA[",
    "<![CDATA[]]a]]>",
];

#[test]
fn the_two_back_ends_agree_on_every_input() {
    for input in CORPUS {
        let a = run_driven(input, Back::Generated);
        let b = run_driven(input, Back::Table);
        assert_eq!(a.tokens, b.tokens, "tokens differ on {input:?}");
        assert_eq!(a.errors, b.errors, "errors differ on {input:?}");
    }
    for input in FOREIGN {
        let a = run_foreign(input, Back::Generated);
        let b = run_foreign(input, Back::Table);
        assert_eq!(a.tokens, b.tokens, "tokens differ on {input:?}");
        assert_eq!(a.errors, b.errors, "errors differ on {input:?}");
    }
}

#[test]
fn every_state_is_reached_by_the_corpus() {
    // Not a conformance claim — the corpus is small and html5lib is the
    // oracle. It keeps the agreement test honest: a state no input
    // reaches is a state the two back ends are never compared on, and
    // this names it.
    let mut seen: Vec<State> = Vec::new();
    let note = |t: &Tokenizer<Collect>, seen: &mut Vec<State>| {
        if !seen.contains(&t.state()) {
            seen.push(t.state());
        }
    };
    for input in CORPUS.iter().chain(FOREIGN) {
        for back in [Back::Generated, Back::Table] {
            let foreign = FOREIGN.contains(input);
            let sink = Collect {
                foreign,
                ..Collect::default()
            };
            let mut t = Tokenizer::new(sink, input, back);
            note(&t, &mut seen);
            let mut at = 0usize;
            while !t.is_done() {
                t.step();
                note(&t, &mut seen);
                let tokens = &t.sink().tokens;
                if !foreign && tokens.len() > at {
                    let next = tokens.get(at..).unwrap_or_default().to_vec();
                    at = tokens.len();
                    for token in next {
                        if let Token::StartTag { name, .. } = token
                            && let Some(e) = crate::switch_for(&name)
                        {
                            t.switch_to(e);
                        }
                    }
                }
            }
        }
    }
    let missed: Vec<&'static str> = states::ALL
        .iter()
        .filter(|s| !seen.contains(s))
        .map(|s| s.name())
        .collect();
    assert!(
        missed.is_empty(),
        "no input in the corpus reaches {} of the {} states: {missed:?}",
        missed.len(),
        states::ALL.len()
    );
}
