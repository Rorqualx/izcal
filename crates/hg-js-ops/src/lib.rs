//! `hg-js-ops`: the JS operations hg-opgen generates from `ops/*.hgop`
//! (JS spec 6.3; design review R0-O1, R0-O9). Layer L2e, tier R.
//!
//! - [`generated::interp`]: the reference interpreter, a `match` loop
//!   generic over the generated [`Runtime`] trait;
//! - [`generated::facts`]: the operation facts (also `facts.tsv`), and
//!   `anchors.tsv`, the anchor index;
//! - [`heap`]: the value layer over `hg-heap` (layouts, typed
//!   references, the typed heap operations);
//! - [`support`]: the frame, realm, constants, hooks and pacing.

#![forbid(unsafe_code)]

pub mod facts_types;
pub mod generated;
pub mod heap;
pub mod support;

pub use generated::interp::{Runtime, run};
