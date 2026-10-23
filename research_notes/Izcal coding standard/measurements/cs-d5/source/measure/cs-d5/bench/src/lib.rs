//! CS-D5 measurement: CPU-bound kernels like the ones the Izcal perf gate
//! would cover (CSS tokenizing, selector matching, URL parsing, HTML
//! tokenizing). Inputs are generated from a fixed-seed LCG, so every run sees
//! the same bytes. No `HashMap` with a random seed, no threads.

use std::cell::Cell;

/// Fixed-seed linear congruential generator (deterministic inputs).
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Lcg(seed)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const TAGS: [&str; 10] = [
    "div", "span", "p", "a", "li", "ul", "section", "img", "button", "td",
];
const CLASSES: [&str; 12] = [
    "nav", "item", "active", "btn", "primary", "card", "header", "footer", "row", "col", "hidden",
    "title",
];

// ---------------------------------------------------------------- CSS input

pub fn make_css(rules: usize) -> String {
    let mut r = Lcg::new(0x5eed_c55);
    let mut s = String::new();
    for i in 0..rules {
        let tag = TAGS[r.below(10) as usize];
        let c1 = CLASSES[r.below(12) as usize];
        let c2 = CLASSES[r.below(12) as usize];
        match i % 4 {
            0 => s.push_str(&format!("{tag}.{c1} > .{c2}:hover, #id{i} {{")),
            1 => s.push_str(&format!(
                "@media (min-width: {}px) {{ .{c1} {tag} {{",
                300 + r.below(900)
            )),
            2 => s.push_str(&format!("{tag}[data-x=\"{c2}\"] ~ .{c1}::before {{")),
            _ => s.push_str(&format!(".{c1}.{c2} {tag}:nth-child(2n+{}) {{", r.below(5))),
        }
        s.push_str(&format!(
            " color: rgb({}, {}, {}); margin: {}px {}em; width: calc(100% - {}px);",
            r.below(256),
            r.below(256),
            r.below(256),
            r.below(40),
            r.below(4),
            r.below(200)
        ));
        s.push_str(" background: url(\"img/bg.png\") no-repeat; /* comment */ font: 12px/1.5 \"Helvetica Neue\", sans-serif; }");
        if i % 4 == 1 {
            s.push_str(" }");
        }
        s.push('\n');
    }
    s
}

fn css_walk(p: &mut cssparser::Parser<'_>, depth: u32) -> u64 {
    let mut n = 0u64;
    loop {
        let block = match p.next_including_whitespace_and_comments() {
            Err(_) => break,
            Ok(tok) => {
                n += 1;
                matches!(
                    tok,
                    cssparser::Token::Function(_)
                        | cssparser::Token::ParenthesisBlock
                        | cssparser::Token::SquareBracketBlock
                        | cssparser::Token::CurlyBracketBlock
                )
            }
        };
        if block {
            let r: Result<u64, cssparser::ParseError<()>> =
                p.parse_nested_block(|inner| Ok(css_walk(inner, depth + 1)));
            n += r.unwrap_or(0);
        }
    }
    n
}

/// Tokenizes a stylesheet with `cssparser`, descending into every block.
pub fn css_tokenize(css: &str) -> u64 {
    let mut p = cssparser::Parser::new(css);
    css_walk(&mut p, 0)
}

// ---------------------------------------------------------- selector matching

pub struct Elem {
    pub parent: Option<u32>,
    pub tag: u8,
    pub id: u32,
    pub classes: u16, // bitset over CLASSES
}

pub struct Compound {
    pub tag: Option<u8>,
    pub id: Option<u32>,
    pub classes: u16,
}

pub enum Comb {
    Descendant,
    Child,
}

/// Rightmost compound first, then (combinator, compound) pairs leftwards.
pub struct Selector {
    pub key: Compound,
    pub rest: Vec<(Comb, Compound)>,
}

pub fn make_dom(n: usize) -> Vec<Elem> {
    let mut r = Lcg::new(0xd0d0);
    let mut v: Vec<Elem> = Vec::with_capacity(n);
    for i in 0..n {
        let parent = if i == 0 {
            None
        } else {
            Some(r.below(i as u64) as u32)
        };
        let mut classes = 0u16;
        for _ in 0..r.below(4) {
            classes |= 1 << r.below(12);
        }
        v.push(Elem {
            parent,
            tag: r.below(10) as u8,
            id: i as u32,
            classes,
        });
    }
    v
}

fn rand_compound(r: &mut Lcg) -> Compound {
    let tag = if r.below(2) == 0 {
        Some(r.below(10) as u8)
    } else {
        None
    };
    let mut classes = 0u16;
    for _ in 0..r.below(3) {
        classes |= 1 << r.below(12);
    }
    let id = if r.below(20) == 0 {
        Some(r.below(2000) as u32)
    } else {
        None
    };
    Compound { tag, id, classes }
}

pub fn make_selectors(n: usize) -> Vec<Selector> {
    let mut r = Lcg::new(0x5e1e);
    (0..n)
        .map(|_| {
            let key = rand_compound(&mut r);
            let rest = (0..r.below(4))
                .map(|_| {
                    let c = if r.below(3) == 0 {
                        Comb::Child
                    } else {
                        Comb::Descendant
                    };
                    (c, rand_compound(&mut r))
                })
                .collect();
            Selector { key, rest }
        })
        .collect()
}

fn compound_matches(c: &Compound, e: &Elem) -> bool {
    c.tag.is_none_or(|t| t == e.tag)
        && c.id.is_none_or(|i| i == e.id)
        && (e.classes & c.classes) == c.classes
}

fn rest_matches(dom: &[Elem], rest: &[(Comb, Compound)], mut cur: u32) -> bool {
    let Some(((comb, comp), tail)) = rest.split_first() else {
        return true;
    };
    match comb {
        Comb::Child => match dom[cur as usize].parent {
            Some(p) if compound_matches(comp, &dom[p as usize]) => rest_matches(dom, tail, p),
            _ => false,
        },
        Comb::Descendant => {
            while let Some(p) = dom[cur as usize].parent {
                if compound_matches(comp, &dom[p as usize]) && rest_matches(dom, tail, p) {
                    return true;
                }
                cur = p;
            }
            false
        }
    }
}

/// Right-to-left matching of every selector against every element.
pub fn selector_match(dom: &[Elem], sels: &[Selector]) -> u64 {
    let mut n = 0u64;
    for (i, e) in dom.iter().enumerate() {
        for s in sels {
            if compound_matches(&s.key, e) && rest_matches(dom, &s.rest, i as u32) {
                n += 1;
            }
        }
    }
    n
}

// ------------------------------------------------------------------- URLs

pub fn make_urls(n: usize) -> Vec<String> {
    let mut r = Lcg::new(0x0e1);
    let hosts = [
        "example.com",
        "xn--bcher-kva.example",
        "sub.domain.test",
        "127.0.0.1",
        "[::1]",
    ];
    (0..n)
        .map(|i| match i % 4 {
            0 => format!(
                "https://{}/path/{}/../to/./file{}.html?q={}&x=%20y#frag",
                hosts[r.below(5) as usize],
                r.below(1000),
                i,
                r.below(99999)
            ),
            1 => format!("../rel/{}/img.png?v={}", r.below(100), i),
            2 => format!(
                "HTTP://User:Pass@{}:8080/A%2fB/c d/ü?ä=ö",
                hosts[r.below(5) as usize]
            ),
            _ => format!(
                "//cdn.{}/static/{}.js",
                hosts[r.below(3) as usize],
                r.below(5000)
            ),
        })
        .collect()
}

/// Parses each string against a base URL (absolute, relative, scheme-relative).
pub fn url_parse(urls: &[String]) -> u64 {
    let base = url::Url::parse("https://www.example.org/dir/sub/page.html").unwrap();
    let mut n = 0u64;
    for u in urls {
        if let Ok(p) = base.join(u) {
            n += p.as_str().len() as u64;
        }
    }
    n
}

// ------------------------------------------------------------------- HTML

pub fn make_html(blocks: usize) -> String {
    let mut r = Lcg::new(0x47e1);
    let mut s = String::from("<!DOCTYPE html><html><head><title>t</title></head><body>\n");
    for i in 0..blocks {
        let c = CLASSES[r.below(12) as usize];
        s.push_str(&format!(
            "<div class=\"{c} row\" id=\"b{i}\" data-n='{}'><p>Some text &amp; an entity &lt;{}&gt; \
             with <a href=\"/x/{}?a=1&b=2\">a link</a> and <img src=x.png alt=\"\" /> &nbsp;</p>\
             <!-- c{i} --><ul><li>one<li>two</ul><script>var x = \"</div>\" < {};</script></div>\n",
            r.below(100),
            r.below(1000),
            r.below(1000),
            r.below(10)
        ));
    }
    s.push_str("</body></html>");
    s
}

struct Counter {
    n: Cell<u64>,
}

impl html5ever::tokenizer::TokenSink for Counter {
    type Handle = ();
    fn process_token(
        &self,
        token: html5ever::tokenizer::Token,
        _line: u64,
    ) -> html5ever::tokenizer::TokenSinkResult<()> {
        use html5ever::tokenizer::Token;
        let add = match token {
            Token::TagToken(t) => 1 + t.attrs.len() as u64,
            Token::CharacterTokens(b) => b.len() as u64,
            _ => 1,
        };
        self.n.set(self.n.get() + add);
        html5ever::tokenizer::TokenSinkResult::Continue
    }
}

/// Tokenizes an HTML document with the `html5ever` tokenizer.
pub fn html_tokenize(html: &str) -> u64 {
    use html5ever::tendril::StrTendril;
    use html5ever::tokenizer::{BufferQueue, Tokenizer, TokenizerOpts};
    let input = BufferQueue::default();
    input.push_back(StrTendril::from_slice(html));
    let tok = Tokenizer::new(Counter { n: Cell::new(0) }, TokenizerOpts::default());
    let _ = tok.feed(&input);
    tok.end();
    tok.sink.n.get()
}

/// Setup for the selector benchmark: 2000 elements, 200 selectors.
pub fn make_match_input() -> (Vec<Elem>, Vec<Selector>) {
    (make_dom(2000), make_selectors(200))
}

#[cfg(test)]
mod tests {
    #[test]
    fn kernels_do_work() {
        let (d, s) = super::make_match_input();
        let r = [
            super::css_tokenize(&super::make_css(2000)),
            super::selector_match(&d, &s),
            super::url_parse(&super::make_urls(2000)),
            super::html_tokenize(&super::make_html(1000)),
        ];
        println!("{r:?}");
        assert!(r.iter().all(|&n| n > 0));
    }
}

/// Inserts and looks up `n` string keys in a `HashMap` with `std`'s
/// randomly seeded `RandomState`: the control case for CS-PF-4's rule that
/// benchmarked hashing uses a fixed key.
pub fn hashmap_random(keys: &[String]) -> u64 {
    let mut m = std::collections::HashMap::new();
    hash_work(&mut m, keys)
}

/// The same work with a fixed-key hasher.
pub fn hashmap_fixed(keys: &[String]) -> u64 {
    let mut m: std::collections::HashMap<
        &str,
        u64,
        std::hash::BuildHasherDefault<std::hash::DefaultHasher>,
    > = std::collections::HashMap::default();
    hash_work(&mut m, keys)
}

fn hash_work<'a, S: std::hash::BuildHasher>(
    m: &mut std::collections::HashMap<&'a str, u64, S>,
    keys: &'a [String],
) -> u64 {
    for (i, k) in keys.iter().enumerate() {
        m.insert(k.as_str(), i as u64);
    }
    keys.iter().filter_map(|k| m.get(k.as_str())).sum()
}

pub fn make_keys(n: usize) -> Vec<String> {
    let mut r = Lcg::new(0x4a54);
    (0..n).map(|i| format!("key-{}-{}", i, r.next())).collect()
}
