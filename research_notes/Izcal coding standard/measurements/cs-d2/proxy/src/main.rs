//! CS-D2 proxy driver: fixed-work renderer hot-path workloads over real inputs.
//! Usage: csd2-proxy <inputs_dir> <workload> <iters> [warmup]
//! Prints one line: workload iters warmup median_ns min_ns max_ns p25_ns p75_ns checksum

use std::fmt;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use cssparser::{Parser as CssParser, Token};
use euclid::{Scale, Size2D};
use html5ever::TokenizerResult;
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tokenizer::{
    BufferQueue, Token as HtmlToken, TokenSink, TokenSinkResult, Tokenizer,
};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::bloom::BloomFilter;
use selectors::context::{
    MatchingContext, MatchingForInvalidation, MatchingMode, NeedsSelectorFlags, SelectorCaches,
};
use selectors::matching::{ElementSelectorFlags, matches_selector};
use selectors::parser::Selector;
use selectors::{Element, OpaqueElement};
use servo_arc::Arc;
use style::context::QuirksMode;
use style::device::Device;
use style::device::servo::FontMetricsProvider;
use style::font_metrics::FontMetrics;
use style::media_queries::{MediaList, MediaType};
use style::properties::ComputedValues;
use style::properties::style_structs::Font;
use style::queries::values::PrefersColorScheme;
use style::selector_parser::{NonTSPseudoClass, PseudoElement, SelectorImpl};
use style::servo::media_features::PointerCapabilities;
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard, StylesheetGuards};
use style::stylesheets::{
    AllowImportRules, CssRule, DocumentStyleSheet, Origin, Stylesheet, UrlExtraData,
};
use style::stylist::Stylist;
use style::values::AtomIdent;
use style::values::AtomString;
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{CSSPixelLength, Length};
use style::values::specified::font::QueryFontMetricsFlags;
use web_atoms::{LocalName, Namespace};

fn read(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(name)).unwrap_or_else(|e| panic!("read {name}: {e}"))
}

// ---------------- cssparser ----------------
fn tokenize_all(input: &mut CssParser) -> usize {
    let mut n = 0usize;
    loop {
        let nested = match input.next_including_whitespace_and_comments() {
            Ok(t) => matches!(
                t,
                Token::Function(_)
                    | Token::ParenthesisBlock
                    | Token::SquareBracketBlock
                    | Token::CurlyBracketBlock
            ),
            Err(_) => break,
        };
        n += 1;
        if nested {
            let r: Result<usize, cssparser::ParseError<()>> =
                input.parse_nested_block(|i| Ok(tokenize_all(i)));
            n += r.unwrap();
        }
    }
    n
}

// ---------------- stylo ----------------
fn url_data() -> UrlExtraData {
    UrlExtraData::from(url::Url::parse("https://example.com/style.css").unwrap())
}

fn parse_sheet(css: &str, lock: &SharedRwLock, ud: &UrlExtraData) -> Stylesheet {
    let media = Arc::new(lock.wrap(MediaList::empty()));
    Stylesheet::from_str(
        css,
        ud.clone(),
        Origin::Author,
        media,
        lock.clone(),
        None,
        None,
        QuirksMode::NoQuirks,
        AllowImportRules::Yes,
    )
}

#[derive(Debug)]
struct Fm;
impl FontMetricsProvider for Fm {
    fn query_font_metrics(
        &self,
        _v: bool,
        _f: &Font,
        _b: CSSPixelLength,
        _fl: QueryFontMetricsFlags,
    ) -> FontMetrics {
        FontMetrics::default()
    }
    fn base_size_for_generic(&self, _g: GenericFontFamily) -> Length {
        Length::new(16.0)
    }
}

fn device() -> Device {
    let caps = PointerCapabilities::FINE | PointerCapabilities::HOVER;
    Device::new(
        MediaType::screen(),
        QuirksMode::NoQuirks,
        Size2D::new(1280.0, 800.0),
        Size2D::new(1280.0, 800.0),
        Scale::new(1.0),
        Box::new(Fm),
        ComputedValues::initial_values_with_font_override(Font::initial_values()),
        PrefersColorScheme::Light,
        caps,
        caps,
    )
}

fn collect_selectors(
    rules: &[CssRule],
    g: &SharedRwLockReadGuard,
    out: &mut Vec<Selector<SelectorImpl>>,
) {
    for r in rules {
        match r {
            CssRule::Style(sr) => {
                let sr = sr.read_with(g);
                for s in sr.selectors.slice().iter() {
                    if !s.has_pseudo_element() {
                        out.push(s.clone());
                    }
                }
            }
            CssRule::Media(m) => collect_selectors(&m.rules.read_with(g).0, g, out),
            _ => {}
        }
    }
}

// ---------------- DOM arena for selector matching ----------------
struct N {
    local: LocalName,
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<(LocalName, String)>,
    parent: Option<u32>,
    prev: Option<u32>,
    next: Option<u32>,
    first_child: Option<u32>,
    non_empty: bool,
}
struct Dom {
    nodes: Vec<N>,
}

fn build_dom(h: &Handle, parent: Option<u32>, dom: &mut Dom) -> Option<u32> {
    if let NodeData::Element { name, attrs, .. } = &h.data {
        let attrs = attrs.borrow();
        let mut id = None;
        let mut classes = vec![];
        let mut av = vec![];
        for a in attrs.iter() {
            let v = a.value.to_string();
            if &*a.name.local == "id" {
                id = Some(v.clone());
            }
            if &*a.name.local == "class" {
                classes = v.split_ascii_whitespace().map(|s| s.to_string()).collect();
            }
            av.push((a.name.local.clone(), v));
        }
        let idx = dom.nodes.len() as u32;
        dom.nodes.push(N {
            local: name.local.clone(),
            id,
            classes,
            attrs: av,
            parent,
            prev: None,
            next: None,
            first_child: None,
            non_empty: false,
        });
        let mut last: Option<u32> = None;
        let mut non_empty = false;
        for c in h.children.borrow().iter() {
            match &c.data {
                NodeData::Text { contents } => {
                    if !contents.borrow().is_empty() {
                        non_empty = true
                    }
                }
                NodeData::Element { .. } => {
                    non_empty = true;
                    if let Some(ci) = build_dom(c, Some(idx), dom) {
                        if let Some(l) = last {
                            dom.nodes[l as usize].next = Some(ci);
                            dom.nodes[ci as usize].prev = Some(l);
                        } else {
                            dom.nodes[idx as usize].first_child = Some(ci);
                        }
                        last = Some(ci);
                    }
                }
                _ => {}
            }
        }
        dom.nodes[idx as usize].non_empty = non_empty;
        Some(idx)
    } else {
        for c in h.children.borrow().iter() {
            build_dom(c, parent, dom);
        }
        None
    }
}

#[derive(Clone, Copy)]
struct El<'a> {
    dom: &'a Dom,
    i: u32,
}
impl fmt::Debug for El<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "El({})", self.i)
    }
}
impl<'a> El<'a> {
    fn n(&self) -> &'a N {
        &self.dom.nodes[self.i as usize]
    }
    fn at(&self, i: Option<u32>) -> Option<Self> {
        i.map(|i| El { dom: self.dom, i })
    }
}

impl<'a> Element for El<'a> {
    type Impl = SelectorImpl;
    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self.n())
    }
    fn parent_element(&self) -> Option<Self> {
        self.at(self.n().parent)
    }
    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }
    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }
    fn is_pseudo_element(&self) -> bool {
        false
    }
    fn prev_sibling_element(&self) -> Option<Self> {
        self.at(self.n().prev)
    }
    fn next_sibling_element(&self) -> Option<Self> {
        self.at(self.n().next)
    }
    fn first_element_child(&self) -> Option<Self> {
        self.at(self.n().first_child)
    }
    fn is_html_element_in_html_document(&self) -> bool {
        true
    }
    fn has_local_name(&self, l: &LocalName) -> bool {
        self.n().local == *l
    }
    fn has_namespace(&self, ns: &Namespace) -> bool {
        *ns == web_atoms::ns!(html)
    }
    fn is_same_type(&self, o: &Self) -> bool {
        self.n().local == o.n().local
    }
    fn attr_matches(
        &self,
        _ns: &NamespaceConstraint<&style::Namespace>,
        local_name: &style::LocalName,
        op: &AttrSelectorOperation<&AtomString>,
    ) -> bool {
        self.n()
            .attrs
            .iter()
            .any(|(n, v)| *n == local_name.0 && op.eval_str(v))
    }
    fn match_non_ts_pseudo_class(
        &self,
        pc: &NonTSPseudoClass,
        _c: &mut MatchingContext<SelectorImpl>,
    ) -> bool {
        match pc {
            NonTSPseudoClass::Link | NonTSPseudoClass::AnyLink => self.is_link(),
            _ => false,
        }
    }
    fn match_pseudo_element(
        &self,
        _pe: &PseudoElement,
        _c: &mut MatchingContext<SelectorImpl>,
    ) -> bool {
        false
    }
    fn apply_selector_flags(&self, _f: ElementSelectorFlags) {}
    fn is_link(&self) -> bool {
        &*self.n().local == "a" && self.n().attrs.iter().any(|(n, _)| &**n == "href")
    }
    fn is_html_slot_element(&self) -> bool {
        false
    }
    fn has_id(&self, id: &AtomIdent, cs: CaseSensitivity) -> bool {
        self.n()
            .id
            .as_deref()
            .is_some_and(|v| cs.eq(v.as_bytes(), id.as_bytes()))
    }
    fn has_class(&self, name: &AtomIdent, cs: CaseSensitivity) -> bool {
        self.n()
            .classes
            .iter()
            .any(|c| cs.eq(c.as_bytes(), name.as_bytes()))
    }
    fn has_custom_state(&self, _n: &AtomIdent) -> bool {
        false
    }
    fn imported_part(&self, _n: &AtomIdent) -> Option<AtomIdent> {
        None
    }
    fn is_part(&self, _n: &AtomIdent) -> bool {
        false
    }
    fn is_empty(&self) -> bool {
        !self.n().non_empty
    }
    fn is_root(&self) -> bool {
        self.n().parent.is_none()
    }
    fn add_element_unique_hashes(&self, _f: &mut BloomFilter) -> bool {
        false
    }
}

// ---------------- html5ever ----------------
struct Sink;
impl TokenSink for Sink {
    type Handle = ();
    fn process_token(&self, token: HtmlToken, _l: u64) -> TokenSinkResult<()> {
        black_box(token);
        TokenSinkResult::Continue
    }
}

fn chunked(s: &str) -> BufferQueue {
    // Same chunking as html5ever's own criterion bench: 1024-char chunks.
    let bq = BufferQueue::default();
    let chars: Vec<char> = s.chars().collect();
    for c in chars.chunks(1024) {
        let st: String = c.iter().collect();
        bq.push_back(StrTendril::from(st));
    }
    bq
}

fn parse_html(s: &str) -> RcDom {
    html5ever::parse_document(RcDom::default(), Default::default()).one(StrTendril::from(s))
}

fn collect_urls(h: &Handle, out: &mut Vec<String>) {
    if let NodeData::Element { attrs, .. } = &h.data {
        for a in attrs.borrow().iter() {
            let n = &*a.name.local;
            if n == "href" || n == "src" {
                out.push(a.value.to_string());
            }
        }
    }
    for c in h.children.borrow().iter() {
        collect_urls(c, out);
    }
}

type Work = Box<dyn FnMut() -> usize>;

fn setup(dir: &Path, w: &str) -> Work {
    match w {
        "cssparser_tokenize_bootstrap" | "cssparser_tokenize_bulma" => {
            let css = read(
                dir,
                if w.ends_with("bootstrap") {
                    "bootstrap-5.3.3.css"
                } else {
                    "bulma-1.0.2.css"
                },
            );
            Box::new(move || {
                let mut p = CssParser::new(&css);
                tokenize_all(&mut p)
            })
        }
        "stylo_parse_bootstrap" | "stylo_parse_bulma" => {
            let css = read(
                dir,
                if w.ends_with("bootstrap") {
                    "bootstrap-5.3.3.css"
                } else {
                    "bulma-1.0.2.css"
                },
            );
            let ud = url_data();
            Box::new(move || {
                let lock = SharedRwLock::new();
                let s = parse_sheet(&css, &lock, &ud);
                let g = lock.read();
                s.contents.read_with(&g).rules(&g).len()
            })
        }
        "stylo_stylist_bootstrap" | "stylo_stylist_bulma" => {
            let css = read(
                dir,
                if w.ends_with("bootstrap") {
                    "bootstrap-5.3.3.css"
                } else {
                    "bulma-1.0.2.css"
                },
            );
            let lock = SharedRwLock::new();
            let sheet = DocumentStyleSheet(Arc::new(parse_sheet(&css, &lock, &url_data())));
            Box::new(move || {
                let g = lock.read();
                let mut st = Stylist::new(device(), QuirksMode::NoQuirks);
                st.append_stylesheet(sheet.clone(), &g);
                let guards = StylesheetGuards::same(&g);
                st.flush(&guards);
                st.num_selectors()
            })
        }
        "selectors_match_bootstrap_navbar" => {
            let css = read(dir, "bootstrap-5.3.3.css");
            let html = read(dir, "bootstrap-docs-navbar.html");
            let lock = SharedRwLock::new();
            let sheet = parse_sheet(&css, &lock, &url_data());
            let mut sels = vec![];
            {
                let g = lock.read();
                collect_selectors(sheet.contents.read_with(&g).rules(&g), &g, &mut sels);
            }
            let rc = parse_html(&html);
            let mut dom = Dom { nodes: vec![] };
            build_dom(&rc.document, None, &mut dom);
            eprintln!("selectors={} elements={}", sels.len(), dom.nodes.len());
            let dom: &'static Dom = Box::leak(Box::new(dom));
            Box::new(move || {
                let mut caches = SelectorCaches::default();
                let mut ctx = MatchingContext::new(
                    MatchingMode::Normal,
                    None,
                    &mut caches,
                    QuirksMode::NoQuirks,
                    NeedsSelectorFlags::No,
                    MatchingForInvalidation::No,
                );
                let mut m = 0usize;
                for i in 0..dom.nodes.len() as u32 {
                    let e = El { dom, i };
                    for s in &sels {
                        if matches_selector(s, 0, None, e, &mut ctx) {
                            m += 1;
                        }
                    }
                }
                m
            })
        }
        "html5ever_treebuild_wikipedia" | "html5ever_treebuild_navbar" => {
            let html = read(
                dir,
                if w.ends_with("wikipedia") {
                    "wikipedia-web-browser.html"
                } else {
                    "bootstrap-docs-navbar.html"
                },
            );
            Box::new(move || {
                let d = parse_html(&html);
                let n = d.document.children.borrow().len();
                black_box(d);
                n
            })
        }
        w if w.starts_with("html5ever_tokenize_") => {
            // html5ever's own bench inputs (html5ever/data/bench/*.html)
            let name = &w["html5ever_tokenize_".len()..];
            let file = format!("{}.html", name.replace('_', "-"));
            let p = dir.join("../html5ever/html5ever/data/bench").join(&file);
            let s = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p:?}: {e}"));
            let bq = chunked(&s);
            Box::new(move || {
                let q = bq.clone();
                let tok = Tokenizer::new(Sink, Default::default());
                while tok.feed(&q) != TokenizerResult::Done {}
                tok.end();
                1
            })
        }
        "url_join_wikipedia" => {
            let html = read(dir, "wikipedia-web-browser.html");
            let rc = parse_html(&html);
            let mut urls = vec![];
            collect_urls(&rc.document, &mut urls);
            eprintln!("urls={}", urls.len());
            let base = url::Url::parse("https://en.wikipedia.org/wiki/Web_browser").unwrap();
            Box::new(move || {
                let mut n = 0usize;
                for u in &urls {
                    if let Ok(j) = base.join(black_box(u)) {
                        n += j.as_str().len();
                    }
                }
                n
            })
        }
        "url_parse_own_bench_strings" => {
            // The strings from url's own benches/parse_url.rs
            let strs = [
                "https://example.com/bench",
                "https://example.com/parkbench?tre=es&st=uff",
                "https://example.com/parkbench?tre=es&st=uff#fragment",
                "https://example.com/",
                "https://example.com:8080",
                "https://127.0.0.1/",
                "https://[::1]/",
                "https://xn--example-yr6d.com/",
            ];
            Box::new(move || {
                let mut n = 0usize;
                for _ in 0..1000 {
                    for s in &strs {
                        n += black_box(*s)
                            .parse::<url::Url>()
                            .map(|u| u.as_str().len())
                            .unwrap_or(0);
                    }
                }
                n
            })
        }
        _ => panic!("unknown workload {w}"),
    }
}

fn main() {
    // args: inputs_dir workload samples [warmup_calls] [batch]
    // Each timed sample runs the workload `batch` times; reported times are ns per call.
    let a: Vec<String> = std::env::args().collect();
    let dir = PathBuf::from(&a[1]);
    let w = &a[2];
    let samples: usize = a[3].parse().unwrap();
    let warm: usize = a.get(4).map(|s| s.parse().unwrap()).unwrap_or(0);
    let batch: usize = a.get(5).map(|s| s.parse().unwrap()).unwrap_or(1);
    let mut f = setup(&dir, w);
    let mut ck = 0usize;
    for _ in 0..warm {
        ck = ck.wrapping_add(black_box(f()));
    }
    let mut t = Vec::with_capacity(samples);
    for _ in 0..samples {
        let s = Instant::now();
        for _ in 0..batch {
            ck = ck.wrapping_add(black_box(f()));
        }
        t.push(s.elapsed().as_nanos() as f64 / batch as f64);
    }
    let raw: Vec<String> = t.iter().map(|x| format!("{x:.1}")).collect();
    t.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| {
        if t.is_empty() {
            0.0
        } else {
            t[((t.len() - 1) as f64 * p).round() as usize]
        }
    };
    println!(
        "{} samples={} warm={} batch={} median={:.1} min={:.1} max={:.1} p25={:.1} p75={:.1} ck={} raw={}",
        w,
        samples,
        warm,
        batch,
        q(0.5),
        q(0.0),
        q(1.0),
        q(0.25),
        q(0.75),
        ck,
        raw.join(",")
    );
}
