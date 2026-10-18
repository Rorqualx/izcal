//! F1 data plane: dense, generational-index struct-of-arrays holding only what
//! style and layout read, plus the log-application (flush) code.
//!
//! Spec rule: this module shares no helpers with `projection.rs`. It depends
//! only on the log's plain-data types and never sees a cppgc pointer.

use crate::log::{Atom, DeclBlock, Mutation, MutationLog, NodeId};
use std::sync::Arc;

pub const NONE: u32 = u32::MAX;

#[derive(Default)]
pub struct DataPlane {
  pub root: u32,
  pub generation: Vec<u32>,
  pub parent: Vec<u32>,
  pub first_child: Vec<u32>,
  pub last_child: Vec<u32>,
  pub prev_sibling: Vec<u32>,
  pub next_sibling: Vec<u32>,
  pub tag: Vec<Atom>,
  pub state: Vec<u32>,
  /// Attribute values are `Arc<str>` shared with the DOM (refcount, no copy).
  pub attrs: Vec<Box<[(Atom, Arc<str>)]>>,
  pub classes: Vec<Box<[Atom]>>,
  /// Reference to the inline declaration block, not a copy (F1 contract).
  pub inline_style: Vec<Option<Arc<DeclBlock>>>,
  /// Incremental metadata: per-node dirty bits set by flush.
  pub dirty: Vec<u8>,
}

pub const DIRTY_STYLE: u8 = 1;
pub const DIRTY_TREE: u8 = 2;

impl DataPlane {
  #[inline]
  fn slot(&self, id: NodeId) -> usize {
    let i = id.idx as usize;
    assert_eq!(self.generation[i], id.generation, "stale NodeId {id:?}");
    i
  }

  /// Apply and drain the log. The log is closed during a flush (F1 contract);
  /// the debug assertion checks nothing was appended concurrently.
  pub fn flush(&mut self, log: &mut MutationLog) {
    let n = log.entries.len();
    for m in log.entries.drain(..) {
      self.apply(m);
    }
    debug_assert!(log.entries.is_empty() && n >= log.entries.len());
  }

  fn apply(&mut self, m: Mutation) {
    match m {
      Mutation::SetRoot(id) => self.root = id.idx,
      Mutation::Create { id, tag } => {
        let i = id.idx as usize;
        if i == self.generation.len() {
          self.generation.push(id.generation);
          for v in [&mut self.parent, &mut self.first_child, &mut self.last_child, &mut self.prev_sibling, &mut self.next_sibling] {
            v.push(NONE);
          }
          self.tag.push(tag);
          self.state.push(0);
          self.attrs.push(Box::new([]));
          self.classes.push(Box::new([]));
          self.inline_style.push(None);
          self.dirty.push(DIRTY_STYLE | DIRTY_TREE);
        } else {
          // Slot reuse: a new generation in an existing slot.
          assert!(self.generation[i] < id.generation);
          self.generation[i] = id.generation;
          for v in [&mut self.parent, &mut self.first_child, &mut self.last_child, &mut self.prev_sibling, &mut self.next_sibling] {
            v[i] = NONE;
          }
          self.tag[i] = tag;
          self.state[i] = 0;
          self.attrs[i] = Box::new([]);
          self.classes[i] = Box::new([]);
          self.inline_style[i] = None;
          self.dirty[i] = DIRTY_STYLE | DIRTY_TREE;
        }
      }
      Mutation::Append { parent, child } => {
        let (p, c) = (self.slot(parent) as u32, self.slot(child) as u32);
        let last = self.last_child[p as usize];
        self.parent[c as usize] = p;
        self.prev_sibling[c as usize] = last;
        self.next_sibling[c as usize] = NONE;
        if last == NONE {
          self.first_child[p as usize] = c;
        } else {
          self.next_sibling[last as usize] = c;
        }
        self.last_child[p as usize] = c;
        self.dirty[p as usize] |= DIRTY_TREE;
        self.dirty[c as usize] |= DIRTY_STYLE | DIRTY_TREE;
      }
      Mutation::Remove { node } => {
        let i = self.slot(node);
        let p = self.parent[i];
        if p == NONE {
          return;
        }
        let (prev, next) = (self.prev_sibling[i], self.next_sibling[i]);
        if prev == NONE {
          self.first_child[p as usize] = next;
        } else {
          self.next_sibling[prev as usize] = next;
        }
        if next == NONE {
          self.last_child[p as usize] = prev;
        } else {
          self.prev_sibling[next as usize] = prev;
        }
        self.parent[i] = NONE;
        self.prev_sibling[i] = NONE;
        self.next_sibling[i] = NONE;
        self.dirty[p as usize] |= DIRTY_TREE;
      }
      Mutation::SetAttr { node, name, value } => {
        let i = self.slot(node);
        let mut v: Vec<_> = std::mem::take(&mut self.attrs[i]).into_vec();
        match v.iter_mut().find(|(k, _)| *k == name) {
          Some(s) => s.1 = value,
          None => v.push((name, value)),
        }
        self.attrs[i] = v.into_boxed_slice();
        self.dirty[i] |= DIRTY_STYLE;
      }
      Mutation::RemoveAttr { node, name } => {
        let i = self.slot(node);
        let v: Vec<_> = std::mem::take(&mut self.attrs[i]).into_vec().into_iter().filter(|(k, _)| *k != name).collect();
        self.attrs[i] = v.into_boxed_slice();
        self.dirty[i] |= DIRTY_STYLE;
      }
      Mutation::AddClass { node, class } => {
        let i = self.slot(node);
        let mut v = std::mem::take(&mut self.classes[i]).into_vec();
        v.push(class);
        self.classes[i] = v.into_boxed_slice();
        self.dirty[i] |= DIRTY_STYLE;
      }
      Mutation::RemoveClass { node, class } => {
        let i = self.slot(node);
        let mut v = std::mem::take(&mut self.classes[i]).into_vec();
        if let Some(k) = v.iter().position(|c| *c == class) {
          v.remove(k);
        }
        self.classes[i] = v.into_boxed_slice();
        self.dirty[i] |= DIRTY_STYLE;
      }
      Mutation::SetState { node, flags } => {
        let i = self.slot(node);
        self.state[i] = flags;
        self.dirty[i] |= DIRTY_STYLE;
      }
      Mutation::SetInlineStyle { node, style } => {
        let i = self.slot(node);
        self.inline_style[i] = style;
        self.dirty[i] |= DIRTY_STYLE;
      }
    }
  }

  pub fn shrink_to_fit(&mut self) {
    self.generation.shrink_to_fit();
    for v in [&mut self.parent, &mut self.first_child, &mut self.last_child, &mut self.prev_sibling, &mut self.next_sibling] {
      v.shrink_to_fit();
    }
    self.tag.shrink_to_fit();
    self.state.shrink_to_fit();
    self.attrs.shrink_to_fit();
    self.classes.shrink_to_fit();
    self.inline_style.shrink_to_fit();
    self.dirty.shrink_to_fit();
  }

  /// Per-node byte breakdown (exact element sizes, ignoring malloc rounding).
  pub fn breakdown(&self, n: usize) -> String {
    let n = n as f64;
    let attr_heap: usize = self.attrs.iter().map(|a| std::mem::size_of_val(&**a)).sum();
    let class_heap: usize = self.classes.iter().map(|c| std::mem::size_of_val(&**c)).sum();
    format!(
      "generation 4 + links 20 + tag 4 + state 4 + attrs ptr 16 + attrs heap {:.1} + classes ptr 16 + classes heap {:.1} + inline-style ref 8 + dirty 1",
      attr_heap as f64 / n,
      class_heap as f64 / n
    )
  }

  /// Bytes owned by the data plane: column capacities plus per-node heap
  /// allocations (boxed attribute / class slices). Shared `Arc` payloads
  /// (attribute strings, declaration blocks) are not counted: they are owned
  /// jointly with the DOM and not duplicated.
  pub fn owned_bytes(&self) -> usize {
    fn cap<T>(v: &Vec<T>) -> usize {
      v.capacity() * std::mem::size_of::<T>()
    }
    let cols = cap(&self.generation)
      + cap(&self.parent)
      + cap(&self.first_child)
      + cap(&self.last_child)
      + cap(&self.prev_sibling)
      + cap(&self.next_sibling)
      + cap(&self.tag)
      + cap(&self.state)
      + cap(&self.attrs)
      + cap(&self.classes)
      + cap(&self.inline_style)
      + cap(&self.dirty);
    let heap: usize = self.attrs.iter().map(|a| std::mem::size_of_val(&**a)).sum::<usize>()
      + self.classes.iter().map(|c| std::mem::size_of_val(&**c)).sum::<usize>();
    cols + heap
  }
}
