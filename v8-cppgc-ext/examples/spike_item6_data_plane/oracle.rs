//! Projection-equality oracle (HGBA-3.0 "Oracles for components with no
//! predecessor", B4 row): the data plane must equal a fresh projection of the
//! cppgc DOM. Compares the data plane's connected tree, walked in document
//! order, against the projection's preorder sequence.

use crate::data_plane::{DataPlane, NONE};
use crate::projection::ProjNode;
use std::sync::Arc;

pub fn check(proj: &[ProjNode], dp: &DataPlane) -> Result<(), String> {
  let mut k = 0usize;
  let mut stack = vec![dp.root];
  while let Some(i) = stack.pop() {
    let iu = i as usize;
    let Some(p) = proj.get(k) else {
      return Err(format!("data plane has more connected nodes than the DOM (at slot {i})"));
    };
    let ctx = |what: &str| format!("node #{k} id={:?}: {what} differs", p.id);
    if p.id.idx != i || p.id.generation != dp.generation[iu] {
      return Err(ctx("identity / document order"));
    }
    let dp_parent = dp.parent[iu];
    let ok_parent = match p.parent {
      None => dp_parent == NONE,
      Some(pid) => dp_parent == pid.idx && dp.generation[dp_parent as usize] == pid.generation,
    };
    if !ok_parent {
      return Err(ctx("parent"));
    }
    if p.tag != dp.tag[iu] {
      return Err(ctx("tag"));
    }
    if p.state != dp.state[iu] {
      return Err(ctx("state flags"));
    }
    let a = &dp.attrs[iu];
    if p.attrs.len() != a.len() || p.attrs.iter().zip(a.iter()).any(|(x, y)| x.0 != y.0 || *x.1 != *y.1) {
      return Err(ctx("attributes"));
    }
    if p.classes[..] != dp.classes[iu][..] {
      return Err(ctx("classes"));
    }
    if p.inline_style != dp.inline_style[iu].as_ref().map(Arc::as_ptr) {
      return Err(ctx("inline style reference"));
    }
    let mut kids = vec![];
    let mut c = dp.first_child[iu];
    while c != NONE {
      kids.push(c);
      c = dp.next_sibling[c as usize];
    }
    stack.extend(kids.into_iter().rev());
    k += 1;
  }
  if k != proj.len() {
    return Err(format!("DOM has {} connected nodes, data plane {k}", proj.len()));
  }
  Ok(())
}
