//! HMAC (RFC 2104, FIPS 198-1), over any [`Hash`].
//!
//! Izcal's own code, written from the published standard.
//!
//! The construction is `H((K ⊕ opad) ‖ H((K ⊕ ipad) ‖ text))`, with a
//! key longer than a block replaced by its own hash and a shorter one
//! padded with zeros. Two details are worth having in the source
//! rather than only in the standard, because both have been got wrong
//! in shipped code:
//!
//! - **A key is hashed only when it is longer than the block**, not
//!   when it is longer than the output. Hashing at the wrong
//!   threshold gives a MAC that is wrong for keys in between, which
//!   functional tests with short keys never reach — which is why
//!   Wycheproof has vectors there.
//! - **Verification compares in constant time** ([`crate::ct::eq`]).
//!   A `==` on the tag leaks where the first difference is, and that
//!   is enough to forge one byte at a time.

use crate::ct;
use crate::hash::Hash;

/// A message authentication code over a hash.
pub trait Mac {
    /// The tag type.
    type Tag: AsRef<[u8]> + Copy;

    /// A MAC keyed with `key`.
    #[must_use]
    fn new(key: &[u8]) -> Self;
    /// Adds `data` to what is being authenticated.
    fn update(&mut self, data: &[u8]);
    /// Finishes, consuming the state.
    fn finish(self) -> Self::Tag;

    /// The tag of `data` under `key`.
    #[must_use]
    fn tag(key: &[u8], data: &[u8]) -> Self::Tag
    where
        Self: Sized,
    {
        let mut m = Self::new(key);
        m.update(data);
        m.finish()
    }

    /// Whether `expected` is the tag of `data` under `key`, compared
    /// in constant time.
    #[must_use]
    fn verify(key: &[u8], data: &[u8], expected: &[u8]) -> bool
    where
        Self: Sized,
    {
        let got = Self::tag(key, data);
        ct::eq(got.as_ref(), expected)
    }
}

const IPAD: u8 = 0x36;
const OPAD: u8 = 0x5c;

/// HMAC over `H`.
#[derive(Clone)]
pub struct Hmac<H: Hash> {
    inner: H,
    outer: H,
}

impl<H: Hash> Mac for Hmac<H> {
    type Tag = H::Output;

    fn new(key: &[u8]) -> Self {
        // RFC 2104 §2: a key longer than the *block* size is hashed
        // first; a shorter one is padded with zeros to the block size.
        let mut block = [0_u8; 128];
        let block_len = H::BLOCK.min(block.len());
        if key.len() > H::BLOCK {
            let digest = H::digest(key);
            let d = digest.as_ref();
            let take = d.len().min(block_len);
            if let (Some(slot), Some(src)) = (block.get_mut(..take), d.get(..take)) {
                slot.copy_from_slice(src);
            }
        } else {
            let take = key.len().min(block_len);
            if let (Some(slot), Some(src)) = (block.get_mut(..take), key.get(..take)) {
                slot.copy_from_slice(src);
            }
        }

        let mut inner = H::default();
        let mut outer = H::default();
        if let Some(padded) = block.get(..block_len) {
            let mut ipad = [0_u8; 128];
            let mut opad = [0_u8; 128];
            for ((i, o), k) in ipad.iter_mut().zip(opad.iter_mut()).zip(padded.iter()) {
                *i = k ^ IPAD;
                *o = k ^ OPAD;
            }
            if let (Some(i), Some(o)) = (ipad.get(..block_len), opad.get(..block_len)) {
                inner.update(i);
                outer.update(o);
            }
        }
        Self { inner, outer }
    }

    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    fn finish(mut self) -> H::Output {
        let inner = self.inner.finish();
        self.outer.update(inner.as_ref());
        self.outer.finish()
    }
}

/// HMAC-SHA-256.
pub type HmacSha256 = Hmac<crate::hash::Sha256>;
