//! Toy cppgc DOM (script-thread side). Every mutation of a field that style or
//! layout reads is applied to the DOM *and* appended to the mutation log.

use crate::log::{Atom, DeclBlock, Mutation, MutationLog, NodeId};
use std::cell::{Cell, UnsafeCell};
use std::sync::Arc;
use v8::cppgc::{GarbageCollected, Heap, Member, Persistent, UnsafePtr, Visitor};

pub struct DomNode {
  pub id: NodeId,
  pub tag: Atom,
  pub state: Cell<u32>,
  pub parent: UnsafeCell<Member<DomNode>>,
  pub first_child: UnsafeCell<Member<DomNode>>,
  pub last_child: UnsafeCell<Member<DomNode>>,
  pub prev_sibling: UnsafeCell<Member<DomNode>>,
  pub next_sibling: UnsafeCell<Member<DomNode>>,
  pub attrs: UnsafeCell<Vec<(Atom, Arc<str>)>>,
  pub classes: UnsafeCell<Vec<Atom>>,
  pub inline_style: UnsafeCell<Option<Arc<DeclBlock>>>,
  /// JS wrapper slot, as a real DOM node has (left empty here).
  pub wrapper: v8::TracedReference<v8::Object>,
}

unsafe impl GarbageCollected for DomNode {
  fn trace(&self, v: &mut Visitor) {
    unsafe {
      v.trace(&*self.parent.get());
      v.trace(&*self.first_child.get());
      v.trace(&*self.last_child.get());
      v.trace(&*self.prev_sibling.get());
      v.trace(&*self.next_sibling.get());
    }
    v.trace(&self.wrapper);
  }
  fn get_name(&self) -> &'static std::ffi::CStr {
    c"DomNode"
  }
}

impl DomNode {
  pub fn link(cell: &UnsafeCell<Member<DomNode>>) -> Option<&DomNode> {
    // SAFETY: Members are traced; callers only run on the script thread with no
    // GC able to free reachable nodes.
    unsafe { (*cell.get()).get() }
  }
}

/// Script-side node handle: index into `Document::handles`. rusty_v8 cannot
/// turn a `&DomNode` back into a cppgc pointer (needed by `Member::set`), so
/// the script side keeps idx -> UnsafePtr. This table is reported separately
/// and excluded from both DOM and data-plane memory.
pub type NodeRef = u32;

pub struct Document<'h> {
  heap: &'h Heap,
  pub root: Persistent<DomNode>,
  pub handles: Vec<UnsafePtr<DomNode>>,
  pub log: MutationLog,
}

impl<'h> Document<'h> {
  pub fn new(heap: &'h Heap) -> Self {
    let mut d = Document { heap, root: Persistent::empty(), handles: Vec::new(), log: MutationLog::default() };
    let r = d.alloc(0);
    d.root = Persistent::new(&d.handles[r as usize]);
    let id = d.node(r).id;
    d.log.append(Mutation::SetRoot(id));
    d
  }
  pub fn node(&self, r: NodeRef) -> &DomNode {
    unsafe { self.handles[r as usize].as_ref() }
  }
  fn alloc(&mut self, tag: Atom) -> NodeRef {
    let idx = self.handles.len() as u32;
    let id = NodeId { idx, generation: 1 };
    let n = DomNode {
      id,
      tag,
      state: Cell::new(0),
      parent: UnsafeCell::new(Member::empty()),
      first_child: UnsafeCell::new(Member::empty()),
      last_child: UnsafeCell::new(Member::empty()),
      prev_sibling: UnsafeCell::new(Member::empty()),
      next_sibling: UnsafeCell::new(Member::empty()),
      attrs: UnsafeCell::new(Vec::new()),
      classes: UnsafeCell::new(Vec::new()),
      inline_style: UnsafeCell::new(None),
      wrapper: v8::TracedReference::empty(),
    };
    self.handles.push(unsafe { v8::cppgc::make_garbage_collected(self.heap, n) });
    self.log.append(Mutation::Create { id, tag });
    idx
  }
  fn set_link(&self, cell: &UnsafeCell<Member<DomNode>>, to: Option<NodeRef>) {
    unsafe {
      match to {
        Some(t) => (*cell.get()).set(&self.handles[t as usize]),
        None => (*cell.get()).set(&Member::<DomNode>::empty()),
      }
    }
  }
  fn idx_of(n: Option<&DomNode>) -> Option<NodeRef> {
    n.map(|n| n.id.idx)
  }
  /// createElement(tag) + parent.appendChild(new)
  pub fn create_child(&mut self, parent: NodeRef, tag: Atom) -> NodeRef {
    let c = self.alloc(tag);
    self.append(parent, c);
    c
  }
  pub fn append(&mut self, parent: NodeRef, child: NodeRef) {
    let (p, c) = (self.node(parent), self.node(child));
    debug_assert!(DomNode::link(&c.parent).is_none(), "child must be detached");
    self.set_link(&c.parent, Some(parent));
    match Self::idx_of(DomNode::link(&p.last_child)) {
      Some(last) => {
        self.set_link(&self.node(last).next_sibling, Some(child));
        self.set_link(&c.prev_sibling, Some(last));
      }
      None => self.set_link(&p.first_child, Some(child)),
    }
    self.set_link(&p.last_child, Some(child));
    let (pid, cid) = (p.id, c.id);
    self.log.append(Mutation::Append { parent: pid, child: cid });
  }
  /// node.remove(): unlink from the parent, keeping the node's subtree.
  pub fn remove(&mut self, node: NodeRef) {
    let n = self.node(node);
    let Some(p) = Self::idx_of(DomNode::link(&n.parent)) else { return };
    let prev = Self::idx_of(DomNode::link(&n.prev_sibling));
    let next = Self::idx_of(DomNode::link(&n.next_sibling));
    match prev {
      Some(pr) => self.set_link(&self.node(pr).next_sibling, next),
      None => self.set_link(&self.node(p).first_child, next),
    }
    match next {
      Some(nx) => self.set_link(&self.node(nx).prev_sibling, prev),
      None => self.set_link(&self.node(p).last_child, prev),
    }
    self.set_link(&n.parent, None);
    self.set_link(&n.prev_sibling, None);
    self.set_link(&n.next_sibling, None);
    let id = n.id;
    self.log.append(Mutation::Remove { node: id });
  }
  pub fn is_connected(&self, node: NodeRef) -> bool {
    let mut n = self.node(node);
    while let Some(p) = DomNode::link(&n.parent) {
      n = p;
    }
    n.id.idx == 0
  }
  pub fn set_attr(&mut self, node: NodeRef, name: Atom, value: Arc<str>) {
    let n = self.node(node);
    let attrs = unsafe { &mut *n.attrs.get() };
    match attrs.iter_mut().find(|(k, _)| *k == name) {
      Some(slot) => slot.1 = value.clone(),
      None => attrs.push((name, value.clone())),
    }
    let id = n.id;
    self.log.append(Mutation::SetAttr { node: id, name, value });
  }
  pub fn remove_attr(&mut self, node: NodeRef, name: Atom) {
    let n = self.node(node);
    let attrs = unsafe { &mut *n.attrs.get() };
    if let Some(i) = attrs.iter().position(|(k, _)| *k == name) {
      attrs.remove(i);
      let id = n.id;
      self.log.append(Mutation::RemoveAttr { node: id, name });
    }
  }
  pub fn toggle_class(&mut self, node: NodeRef, class: Atom) {
    let n = self.node(node);
    let cl = unsafe { &mut *n.classes.get() };
    let id = n.id;
    if let Some(i) = cl.iter().position(|c| *c == class) {
      cl.remove(i);
      self.log.append(Mutation::RemoveClass { node: id, class });
    } else {
      cl.push(class);
      self.log.append(Mutation::AddClass { node: id, class });
    }
  }
  pub fn set_state(&mut self, node: NodeRef, flags: u32) {
    let n = self.node(node);
    n.state.set(flags);
    let id = n.id;
    self.log.append(Mutation::SetState { node: id, flags });
  }
  pub fn set_inline_style(&mut self, node: NodeRef, style: Option<Arc<DeclBlock>>) {
    let n = self.node(node);
    unsafe { *n.inline_style.get() = style.clone() };
    let id = n.id;
    self.log.append(Mutation::SetInlineStyle { node: id, style });
  }
}
