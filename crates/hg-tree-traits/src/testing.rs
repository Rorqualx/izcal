//! A tree a test can write down. Behind the `testing` feature.
//!
//! This is a **fixture, not engine code**. G20 keeps `hg-style` and
//! `hg-style-reference` from sharing engine code; a differential needs
//! them to read the *same* tree, so the tree they read is deliberately
//! one implementation. Section 10 already records `hg-tree-traits` as a
//! remaining common-mode risk, and this is part of it: a bug in this
//! file would make both engines wrong together, which is why it holds
//! no matching, no cascade and no parsing of anything but its own
//! indented literal.
//!
//! # The literal
//!
//! One node per line, nesting by two-space indent. An element is a
//! name with optional attributes; `#id` and `.class` are shorthands
//! for the `id` and `class` attributes, in the order written.
//!
//! ```text
//! html
//!   body
//!     div#main.wide lang=en
//!       p.first
//!         "some text"
//!       p
//!         "more"
//!       <!-- a comment -->
//! ```
//!
//! A line that is a quoted string is a text node; `<!-- ... -->` is a
//! comment. Everything else is an element. A name may carry a
//! namespace prefix: `svg|circle`, with `svg`, `math`, `xlink`, `xml`
//! and `xmlns` known and anything else taken literally as a URL.

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::{AttrNs, ElementState, NodeId, NodeKind, Tree, ns};

/// One node of a test tree.
#[derive(Debug, Clone)]
pub struct Node {
    /// What kind of node it is.
    pub kind: NodeKind,
    /// The namespace, for an element.
    pub namespace: String,
    /// The local name, for an element.
    pub local_name: String,
    /// The attributes, in the order written: namespace, local, value.
    pub attributes: Vec<(String, String, String)>,
    /// The text, for a text node or a comment.
    pub text: String,
    /// The state bits a test sets with [`TestTree::set_state`].
    pub state: ElementState,
    /// The parent's index.
    pub parent: Option<usize>,
    /// The children's indices, in tree order.
    pub children: Vec<usize>,
}

/// A tree written down in a test.
#[derive(Debug, Clone)]
pub struct TestTree {
    nodes: Vec<Node>,
    html_document: bool,
}

/// What went wrong reading a tree literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// The 1-based line.
    pub line: usize,
    /// What was wrong with it.
    pub message: String,
}

fn namespace_of(prefix: &str) -> String {
    match prefix {
        "svg" => ns::SVG.to_owned(),
        "math" => ns::MATHML.to_owned(),
        "xlink" => ns::XLINK.to_owned(),
        "xml" => ns::XML.to_owned(),
        "xmlns" => ns::XMLNS.to_owned(),
        other => other.to_owned(),
    }
}

/// Splits a line into whitespace-separated fields, keeping a quoted
/// value whole.
fn fields(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for c in s.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => cur.push(c),
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c.is_ascii_whitespace() => {
                if !cur.is_empty() {
                    out.push(core::mem::take(&mut cur));
                }
            }
            None => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

impl TestTree {
    /// Reads a tree literal.
    ///
    /// # Errors
    /// [`Error`] naming the line, for bad indentation or an empty name.
    pub fn parse(source: &str) -> Result<Self, Error> {
        let mut nodes = alloc::vec![Node {
            kind: NodeKind::Document,
            namespace: String::new(),
            local_name: String::new(),
            attributes: Vec::new(),
            text: String::new(),
            state: ElementState::default(),
            parent: None,
            children: Vec::new(),
        }];
        // The node each indent depth is currently inside; depth 0 is
        // the document.
        let mut open: Vec<usize> = alloc::vec![0];
        for (i, raw) in source.lines().enumerate() {
            let line = i.saturating_add(1);
            if raw.trim().is_empty() {
                continue;
            }
            let indent = raw.len().saturating_sub(raw.trim_start().len());
            if indent % 2 != 0 {
                return Err(Error {
                    line,
                    message: format!("indent {indent} is not a multiple of two"),
                });
            }
            let depth = indent.checked_div(2).unwrap_or(0).saturating_add(1);
            if depth > open.len() {
                return Err(Error {
                    line,
                    message: "indented more than one level at once".to_owned(),
                });
            }
            open.truncate(depth);
            let Some(parent) = open.last().copied() else {
                return Err(Error {
                    line,
                    message: "no parent".to_owned(),
                });
            };
            let body = raw.trim();
            let node = if let Some(c) = body
                .strip_prefix("<!--")
                .and_then(|b| b.strip_suffix("-->"))
            {
                Node {
                    kind: NodeKind::Comment,
                    text: c.trim().to_owned(),
                    ..Self::blank(parent)
                }
            } else if (body.starts_with('"') && body.ends_with('"') && body.len() >= 2)
                || (body.starts_with('\'') && body.ends_with('\'') && body.len() >= 2)
            {
                let inner = body
                    .get(1..body.len().saturating_sub(1))
                    .unwrap_or_default()
                    .to_owned();
                Node {
                    kind: NodeKind::Text,
                    text: inner,
                    ..Self::blank(parent)
                }
            } else {
                Self::element(body, parent, line)?
            };
            let index = nodes.len();
            nodes.push(node);
            if let Some(p) = nodes.get_mut(parent) {
                p.children.push(index);
            }
            open.push(index);
        }
        Ok(Self {
            nodes,
            html_document: true,
        })
    }

    fn blank(parent: usize) -> Node {
        Node {
            kind: NodeKind::Text,
            namespace: String::new(),
            local_name: String::new(),
            attributes: Vec::new(),
            text: String::new(),
            state: ElementState::default(),
            parent: Some(parent),
            children: Vec::new(),
        }
    }

    fn element(body: &str, parent: usize, line: usize) -> Result<Node, Error> {
        let parts = fields(body);
        let Some(head) = parts.first() else {
            return Err(Error {
                line,
                message: "an empty line is not a node".to_owned(),
            });
        };
        let mut attributes: Vec<(String, String, String)> = Vec::new();
        // `#id` and `.class` shorthands, in the order written.
        let mut name = head.as_str();
        let mut id: Option<String> = None;
        let mut classes: Vec<String> = Vec::new();
        while let Some(cut) = name.rfind(['#', '.']) {
            let (before, token) = (
                name.get(..cut).unwrap_or_default(),
                name.get(cut..).unwrap_or_default(),
            );
            if let Some(v) = token.strip_prefix('#') {
                id = Some(v.to_owned());
            } else if let Some(v) = token.strip_prefix('.') {
                classes.insert(0, v.to_owned());
            }
            name = before;
        }
        let (namespace, local) = match name.split_once('|') {
            Some((prefix, local)) => (namespace_of(prefix), local.to_owned()),
            None => (ns::HTML.to_owned(), name.to_owned()),
        };
        if local.is_empty() {
            return Err(Error {
                line,
                message: "an element needs a name".to_owned(),
            });
        }
        if let Some(v) = id {
            attributes.push((String::new(), "id".to_owned(), v));
        }
        if !classes.is_empty() {
            attributes.push((String::new(), "class".to_owned(), classes.join(" ")));
        }
        for field in parts.iter().skip(1) {
            let (key, value) = field.split_once('=').unwrap_or((field.as_str(), ""));
            let (ans, local) = match key.split_once('|') {
                Some((prefix, local)) => (namespace_of(prefix), local.to_owned()),
                None => (String::new(), key.to_owned()),
            };
            // A second `class=` replaces the shorthand rather than
            // sitting beside it, so a test cannot write two.
            if let Some(slot) = attributes
                .iter_mut()
                .find(|(n, l, _)| *n == ans && *l == local)
            {
                slot.2 = value.to_owned();
            } else {
                attributes.push((ans, local, value.to_owned()));
            }
        }
        Ok(Node {
            kind: NodeKind::Element,
            namespace,
            local_name: local,
            attributes,
            ..Self::blank(parent)
        })
    }

    /// Whether the tree is an HTML document. Tests of namespace or
    /// case sensitivity turn it off.
    pub const fn set_html_document(&mut self, html: bool) {
        self.html_document = html;
    }

    /// Sets an element's state bits, by its `id` attribute.
    ///
    /// Returns whether an element with that id was found.
    pub fn set_state(&mut self, id: &str, state: ElementState) -> bool {
        let Some(index) = self.find(id) else {
            return false;
        };
        let Some(node) = self.nodes.get_mut(index) else {
            return false;
        };
        node.state = state;
        true
    }

    /// The index of the element with this `id`, in tree order.
    #[must_use]
    pub fn find(&self, id: &str) -> Option<usize> {
        self.nodes.iter().position(|n| {
            matches!(n.kind, NodeKind::Element)
                && n.attributes
                    .iter()
                    .any(|(ns, l, v)| ns.is_empty() && l == "id" && v == id)
        })
    }

    /// Every node, in the order they were written, which for this
    /// format is tree order.
    #[must_use]
    pub fn all(&self) -> Vec<usize> {
        (0..self.nodes.len()).collect()
    }

    /// Every element, in tree order.
    #[must_use]
    pub fn elements(&self) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.kind, NodeKind::Element))
            .map(|(i, _)| i)
            .collect()
    }

    fn at(&self, index: usize) -> Option<&Node> {
        self.nodes.get(index)
    }
}

impl Tree for TestTree {
    type Node = usize;

    fn document(&self) -> usize {
        0
    }

    fn is_html_document(&self) -> bool {
        self.html_document
    }

    fn id(&self, node: usize) -> NodeId {
        NodeId::new(u32::try_from(node).unwrap_or(u32::MAX), 0)
    }

    fn kind(&self, node: usize) -> NodeKind {
        self.at(node).map_or(NodeKind::Comment, |n| n.kind)
    }

    fn parent(&self, node: usize) -> Option<usize> {
        self.at(node).and_then(|n| n.parent)
    }

    fn first_child(&self, node: usize) -> Option<usize> {
        self.at(node).and_then(|n| n.children.first().copied())
    }

    fn last_child(&self, node: usize) -> Option<usize> {
        self.at(node).and_then(|n| n.children.last().copied())
    }

    fn previous_sibling(&self, node: usize) -> Option<usize> {
        let parent = self.parent(node)?;
        let kids = &self.at(parent)?.children;
        let i = kids.iter().position(|k| *k == node)?;
        kids.get(i.checked_sub(1)?).copied()
    }

    fn next_sibling(&self, node: usize) -> Option<usize> {
        let parent = self.parent(node)?;
        let kids = &self.at(parent)?.children;
        let i = kids.iter().position(|k| *k == node)?;
        kids.get(i.checked_add(1)?).copied()
    }

    fn namespace(&self, node: usize) -> Option<&str> {
        let n = self.at(node)?;
        matches!(n.kind, NodeKind::Element).then_some(n.namespace.as_str())
    }

    fn local_name(&self, node: usize) -> Option<&str> {
        let n = self.at(node)?;
        matches!(n.kind, NodeKind::Element).then_some(n.local_name.as_str())
    }

    fn attribute(&self, node: usize, ans: AttrNs<'_>, local: &str) -> Option<&str> {
        let n = self.at(node)?;
        n.attributes
            .iter()
            .find(|(a, l, _)| {
                l == local
                    && match ans {
                        AttrNs::None => a.is_empty(),
                        AttrNs::One(want) => a == want,
                        AttrNs::Any => true,
                    }
            })
            .map(|(_, _, v)| v.as_str())
    }

    fn element_state(&self, node: usize) -> ElementState {
        self.at(node)
            .map_or_else(ElementState::default, |n| n.state)
    }

    fn text(&self, node: usize) -> Option<&str> {
        let n = self.at(node)?;
        matches!(n.kind, NodeKind::Text).then_some(n.text.as_str())
    }
}
