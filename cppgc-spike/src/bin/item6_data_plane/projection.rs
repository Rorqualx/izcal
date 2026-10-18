//! Fresh, independent, non-incremental projection of the cppgc DOM: one full
//! traversal reading exactly the fields the data plane mirrors.
//!
//! Spec rule: this module shares no helpers with `data_plane.rs` (the
//! log-application code). It reads the DOM directly and knows nothing about
//! the log or the data plane.

use crate::dom::DomNode;
use crate::log::{Atom, DeclBlock, NodeId};
use std::sync::Arc;

pub struct ProjNode {
  pub id: NodeId,
  pub parent: Option<NodeId>,
  pub tag: Atom,
  pub state: u32,
  pub attrs: Vec<(Atom, Arc<str>)>,
  pub classes: Vec<Atom>,
  pub inline_style: Option<*const DeclBlock>,
}

/// Preorder (document order) projection of the connected tree under `root`.
pub fn project(root: &DomNode) -> Vec<ProjNode> {
  let mut out = Vec::new();
  let mut stack: Vec<&DomNode> = vec![root];
  while let Some(n) = stack.pop() {
    // SAFETY: script thread only; reachable nodes cannot be freed meanwhile.
    let (attrs, classes, style) = unsafe { (&*n.attrs.get(), &*n.classes.get(), &*n.inline_style.get()) };
    let parent = unsafe { (*n.parent.get()).get() }.map(|p| p.id);
    out.push(ProjNode {
      id: n.id,
      parent,
      tag: n.tag,
      state: n.state.get(),
      attrs: attrs.clone(),
      classes: classes.clone(),
      inline_style: style.as_ref().map(Arc::as_ptr),
    });
    // Push children in reverse so they pop in document order.
    let mut kids = Vec::new();
    let mut c = unsafe { (*n.first_child.get()).get() };
    while let Some(k) = c {
      kids.push(k);
      c = unsafe { (*k.next_sibling.get()).get() };
    }
    stack.extend(kids.into_iter().rev());
  }
  out
}
