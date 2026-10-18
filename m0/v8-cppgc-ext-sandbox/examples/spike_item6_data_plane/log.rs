//! The mutation log: the only interface from the DOM (script side) to the data
//! plane. Plain data only: no cppgc pointers ever cross this boundary.

use std::sync::Arc;

pub type Atom = u32;

/// Generational node id, assigned by the DOM at node creation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NodeId {
  pub idx: u32,
  pub generation: u32,
}

/// Stand-in for Stylo's `Arc<Locked<PropertyDeclarationBlock>>`: shared, not copied.
#[derive(Debug)]
pub struct DeclBlock {
  pub _text: String,
}

#[derive(Clone, Debug)]
pub enum Mutation {
  SetRoot(NodeId),
  Create { id: NodeId, tag: Atom },
  Append { parent: NodeId, child: NodeId },
  Remove { node: NodeId },
  SetAttr { node: NodeId, name: Atom, value: Arc<str> },
  RemoveAttr { node: NodeId, name: Atom },
  AddClass { node: NodeId, class: Atom },
  RemoveClass { node: NodeId, class: Atom },
  SetState { node: NodeId, flags: u32 },
  SetInlineStyle { node: NodeId, style: Option<Arc<DeclBlock>> },
}

#[derive(Default)]
pub struct MutationLog {
  pub entries: Vec<Mutation>,
}

impl MutationLog {
  #[inline]
  pub fn append(&mut self, m: Mutation) {
    self.entries.push(m);
  }
  pub fn len(&self) -> usize {
    self.entries.len()
  }
}
