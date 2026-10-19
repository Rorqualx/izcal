//! A DOM-like model for criterion (4): multi-node mutation under TokenCell.
//!
//! Layout follows the CS-D3 design:
//! - Tree links (`parent`, `first_child`, ...) are traced `Member`s in the
//!   cppgc DOM. They live **outside** any `TokenCell` (criterion 3). Here the
//!   stand-in is `Cell<Option<NodeId>>`: a slot that never lends a reference
//!   into itself, like the V8 crate's `MemberCell` sketch.
//! - Script-reachable, non-traced node state lives in `TokenCell<NodeData>`.
//! - Nodes are reached through cloned-out handles (`Rc<Node>` here, GC handles
//!   in the real DOM), never through a borrow of the document's node table.
//!
//! Observers get the token. One that runs "script" must lend it (`lend`, which
//! `hg_script`'s wrappers do around every V8 call); script that touches the
//! DOM comes back through `reenter`, which is what a generated binding prologue
//! does when JS calls back into Rust. The DOM crate itself never lends.
//!
//! The three patterns from CS-D3 (4) all appear in `insert_before`:
//! per-node split (step 2), scoped multi-borrow (step 4), clone-out (steps 0
//! and 5).

use crate::cell::{TokenCell, borrow_mut_2, borrow_mut_3};
use crate::token::ScriptToken;
use std::cell::Cell;
use std::rc::Rc;

pub type NodeId = u32;

crate::no_traced! {
    #[derive(Default, Debug, Clone)]
    pub struct NodeData {
        pub child_count: u32,
        pub tree_version: u64,
        pub text: String,
        pub connected: bool,
        /// Cached index among siblings, invalidated on mutation.
        pub index_hint: Option<u32>,
    }
}

pub struct Node {
    pub id: NodeId,
    parent: Cell<Option<NodeId>>,
    first_child: Cell<Option<NodeId>>,
    last_child: Cell<Option<NodeId>>,
    prev_sibling: Cell<Option<NodeId>>,
    next_sibling: Cell<Option<NodeId>>,
    pub data: TokenCell<NodeData>,
}

impl Node {
    pub fn parent(&self) -> Option<NodeId> {
        self.parent.get()
    }
    pub fn next_sibling(&self) -> Option<NodeId> {
        self.next_sibling.get()
    }
    pub fn first_child(&self) -> Option<NodeId> {
        self.first_child.get()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationRecord {
    pub target: NodeId,
    pub added: NodeId,
    pub next_sibling: Option<NodeId>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DomError {
    HierarchyRequest,
    NotFound,
}

/// Reaction hook (custom-element reaction / observer delivery). It gets the
/// token; to run script it lends it, and script re-enters through `reenter`.
pub type Observer = Rc<dyn Fn(&Document, &MutationRecord, &mut ScriptToken)>;

#[derive(Default)]
pub struct Document {
    nodes: TokenCell<Vec<Rc<Node>>>,
    observers: TokenCell<Vec<Observer>>,
    pub records: TokenCell<Vec<MutationRecord>>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_node(&self, text: &str, tok: &mut ScriptToken) -> NodeId {
        let nodes = self.nodes.borrow_mut(tok);
        let id = nodes.len() as NodeId;
        nodes.push(Rc::new(Node {
            id,
            parent: Cell::new(None),
            first_child: Cell::new(None),
            last_child: Cell::new(None),
            prev_sibling: Cell::new(None),
            next_sibling: Cell::new(None),
            data: TokenCell::new(NodeData {
                text: text.into(),
                ..Default::default()
            }),
        }));
        id
    }

    /// Clone-out: a handle that outlives any borrow of the node table.
    pub fn node(&self, id: NodeId, tok: &ScriptToken) -> Rc<Node> {
        self.nodes.borrow(tok)[id as usize].clone()
    }

    pub fn set_connected(&self, id: NodeId, tok: &mut ScriptToken) {
        let n = self.node(id, tok);
        n.data.borrow_mut(tok).connected = true;
    }

    pub fn observe(&self, obs: Observer, tok: &mut ScriptToken) {
        self.observers.borrow_mut(tok).push(obs);
    }

    pub fn text(&self, id: NodeId, tok: &ScriptToken) -> String {
        self.node(id, tok).data.borrow(tok).text.clone()
    }

    pub fn set_text(&self, id: NodeId, text: &str, tok: &mut ScriptToken) {
        let n = self.node(id, tok);
        let d = n.data.borrow_mut(tok);
        d.text.clear();
        d.text.push_str(text);
        d.tree_version += 1;
    }

    pub fn children(&self, id: NodeId, tok: &ScriptToken) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut cur = self.node(id, tok).first_child.get();
        while let Some(c) = cur {
            out.push(c);
            cur = self.node(c, tok).next_sibling.get();
        }
        out
    }

    pub fn child_count(&self, id: NodeId, tok: &ScriptToken) -> u32 {
        self.node(id, tok).data.borrow(tok).child_count
    }

    fn is_inclusive_ancestor(&self, anc: NodeId, mut of: NodeId, tok: &ScriptToken) -> bool {
        loop {
            if of == anc {
                return true;
            }
            match self.node(of, tok).parent.get() {
                Some(p) => of = p,
                None => return false,
            }
        }
    }

    /// Detach `n` from its parent's child list (links only).
    fn unlink(&self, n: &Node, tok: &ScriptToken) {
        let Some(p) = n.parent.get() else { return };
        let parent = self.node(p, tok);
        match n.prev_sibling.get() {
            Some(prev) => self.node(prev, tok).next_sibling.set(n.next_sibling.get()),
            None => parent.first_child.set(n.next_sibling.get()),
        }
        match n.next_sibling.get() {
            Some(next) => self.node(next, tok).prev_sibling.set(n.prev_sibling.get()),
            None => parent.last_child.set(n.prev_sibling.get()),
        }
        n.parent.set(None);
        n.prev_sibling.set(None);
        n.next_sibling.set(None);
    }

    /// `parent.insertBefore(child, reference)`, touching parent, child,
    /// reference node and (if any) the child's old parent, then running
    /// "script" (observers) synchronously, as a custom-element reaction would.
    pub fn insert_before(
        &self,
        parent_id: NodeId,
        child_id: NodeId,
        ref_id: Option<NodeId>,
        tok: &mut ScriptToken,
    ) -> Result<(), DomError> {
        // 0. Clone out handles; no borrow of the node table survives this.
        let parent = self.node(parent_id, tok);
        let child = self.node(child_id, tok);
        let mut ref_id = ref_id;
        if ref_id == Some(child_id) {
            ref_id = child.next_sibling.get();
        }
        let reference = ref_id.map(|r| self.node(r, tok));

        // 1. Validity: links only, shared token.
        if self.is_inclusive_ancestor(child_id, parent_id, tok) {
            return Err(DomError::HierarchyRequest);
        }
        if let Some(r) = &reference {
            if r.parent.get() != Some(parent_id) {
                return Err(DomError::NotFound);
            }
        }

        // 2. Per-node split: remove from the old parent with one short borrow.
        if let Some(old) = child.parent.get() {
            self.unlink(&child, tok);
            let old = self.node(old, tok);
            let d = old.data.borrow_mut(tok);
            d.child_count -= 1;
            d.tree_version += 1;
        }

        // 3. Links (traced fields; no token needed to write them).
        let prev = match &reference {
            Some(r) => r.prev_sibling.get(),
            None => parent.last_child.get(),
        };
        child.parent.set(Some(parent_id));
        child.prev_sibling.set(prev);
        child.next_sibling.set(ref_id);
        match prev {
            Some(p) => self.node(p, tok).next_sibling.set(Some(child_id)),
            None => parent.first_child.set(Some(child_id)),
        }
        match &reference {
            Some(r) => r.prev_sibling.set(Some(child_id)),
            None => parent.last_child.set(Some(child_id)),
        }

        // 4. Scoped multi-borrow: three nodes' script-reachable state at once.
        match &reference {
            Some(r) => {
                let (p, c, rd) = borrow_mut_3(&parent.data, &child.data, &r.data, tok);
                p.child_count += 1;
                p.tree_version += 1;
                c.connected = p.connected;
                c.index_hint = rd.index_hint;
                rd.index_hint = rd.index_hint.map(|i| i + 1);
            }
            None => {
                let (p, c) = borrow_mut_2(&parent.data, &child.data, tok);
                p.child_count += 1;
                p.tree_version += 1;
                c.connected = p.connected;
                c.index_hint = Some(p.child_count - 1);
            }
        }

        // 5. Reactions: run script. Clone the record and observer list out
        //    first; nothing borrowed from a TokenCell is alive across the hooks,
        //    which take `&mut ScriptToken` and may run script.
        let rec = MutationRecord {
            target: parent_id,
            added: child_id,
            next_sibling: ref_id,
        };
        self.records.borrow_mut(tok).push(rec.clone());
        self.notify(&rec, tok);

        // 6. After script: anything read before step 5 may be stale; re-borrow.
        let _ = parent.data.borrow(tok).child_count;
        Ok(())
    }

    fn notify(&self, rec: &MutationRecord, tok: &mut ScriptToken) {
        let observers: Vec<Observer> = self.observers.borrow(tok).clone();
        for o in observers {
            o(self, rec, tok);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{enter, lend, reenter};

    fn tree(doc: &Document, t: &mut ScriptToken) -> (NodeId, NodeId, NodeId, NodeId) {
        let root = doc.create_node("root", t);
        doc.set_connected(root, t);
        let a = doc.create_node("a", t);
        let b = doc.create_node("b", t);
        let x = doc.create_node("x", t);
        doc.insert_before(root, a, None, t).unwrap();
        doc.insert_before(root, b, None, t).unwrap();
        (root, a, b, x)
    }

    #[test]
    fn insert_before_links_and_counts() {
        let doc = Document::new();
        enter(|t| {
            let (root, a, b, x) = tree(&doc, t);
            doc.insert_before(root, x, Some(b), t).unwrap();
            assert_eq!(doc.children(root, t), vec![a, x, b]);
            assert_eq!(doc.child_count(root, t), 3);
            // Move x before a: same parent, so count stays 3.
            doc.insert_before(root, x, Some(a), t).unwrap();
            assert_eq!(doc.children(root, t), vec![x, a, b]);
            assert_eq!(doc.child_count(root, t), 3);
            assert!(doc.node(x, t).data.borrow(t).connected);
            assert_eq!(
                doc.insert_before(x, root, None, t),
                Err(DomError::HierarchyRequest)
            );
            assert_eq!(doc.insert_before(a, b, Some(x), t), Err(DomError::NotFound));
        });
    }

    #[test]
    fn observer_reenters_and_mutates() {
        let doc = Document::new();
        enter(|t| {
            let (root, _a, b, x) = tree(&doc, t);
            // The "MutationObserver": JS that calls back into a native binding.
            doc.observe(
                Rc::new(
                    |doc: &Document, rec: &MutationRecord, t: &mut ScriptToken| {
                        // Run the JS observer (lend); JS calls a native binding
                        // (reenter).
                        lend(t, || {
                            reenter(|t| {
                                let seen = format!("seen:{}", doc.text(rec.target, t));
                                doc.set_text(rec.added, &seen, t);
                            })
                        })
                    },
                ),
                t,
            );
            doc.insert_before(root, x, Some(b), t).unwrap();
            assert_eq!(doc.text(x, t), "seen:root");
        });
    }

    #[test]
    fn observer_reenters_insert_before_recursively() {
        // A reaction that itself mutates the tree (nested insertBefore, which
        // runs observers again): two levels of lend/reenter.
        let doc = Document::new();
        enter(|t| {
            let (root, a, b, x) = tree(&doc, t);
            let y = doc.create_node("y", t);
            let depth = Rc::new(Cell::new(0u32));
            let d2 = depth.clone();
            doc.observe(
                Rc::new(
                    move |doc: &Document, rec: &MutationRecord, t: &mut ScriptToken| {
                        if rec.added == x {
                            d2.set(d2.get() + 1);
                            lend(t, || {
                                reenter(|t| doc.insert_before(rec.target, y, Some(x), t))
                            })
                            .unwrap();
                        }
                    },
                ),
                t,
            );
            doc.insert_before(root, x, Some(b), t).unwrap();
            assert_eq!(depth.get(), 1);
            assert_eq!(doc.children(root, t), vec![a, y, x, b]);
            assert_eq!(doc.child_count(root, t), 4);
            assert_eq!(doc.records.borrow(t).len(), 2 + 1 + 1);
        });
    }

    #[test]
    fn observer_moves_the_reference_node() {
        // Script moves the node that the outer insertBefore used as reference.
        // Step 6 re-borrows, so the outer frame sees the new state.
        let doc = Document::new();
        enter(|t| {
            let (root, a, b, x) = tree(&doc, t);
            let other = doc.create_node("other", t);
            doc.observe(
                Rc::new(
                    move |doc: &Document, rec: &MutationRecord, t: &mut ScriptToken| {
                        if rec.added == x {
                            lend(t, || reenter(|t| doc.insert_before(other, b, None, t))).unwrap();
                        }
                    },
                ),
                t,
            );
            doc.insert_before(root, x, Some(b), t).unwrap();
            assert_eq!(doc.children(root, t), vec![a, x]);
            assert_eq!(doc.children(other, t), vec![b]);
            assert_eq!(doc.child_count(root, t), 2);
            assert_eq!(doc.child_count(other, t), 1);
        });
    }
}
