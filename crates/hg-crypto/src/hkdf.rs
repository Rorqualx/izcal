//! HKDF (RFC 5869), and TLS 1.3's label over it (RFC 8446 §7.1).
//!
//! Izcal's own code, written from the published standards.
//!
//! HKDF is two steps and the separation is the point: **extract**
//! turns input keying material of any shape into a fixed-length
//! pseudorandom key, and **expand** turns that into as many output
//! bytes as are asked for. A key schedule that skipped extract and
//! expanded the shared secret directly would carry the secret's own
//! structure into every key derived from it.
//!
//! TLS 1.3 builds its whole schedule on this: `HKDF-Expand-Label` is
//! expand with the info field given a fixed shape, so that two
//! protocols that derive keys from the same secret cannot collide on
//! one. [`expand_label`] writes that shape out.

use crate::hash::Hash;
use crate::hmac::{Hmac, Mac};

/// A key derivation function.
pub trait Kdf {
    /// Extracts a pseudorandom key from `ikm` with an optional salt.
    ///
    /// Returns the PRK's bytes in `out`, which must be the hash's
    /// output length; `false` when it is not.
    fn extract(salt: &[u8], ikm: &[u8], out: &mut [u8]) -> bool;

    /// Expands `prk` into `out` bytes, bound to `info`.
    ///
    /// Returns `false` when `out` is longer than the standard allows
    /// (RFC 5869 §2.3: 255 times the hash length).
    fn expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> bool;
}

/// HKDF over `H`.
pub struct Hkdf<H: Hash> {
    _hash: core::marker::PhantomData<H>,
}

impl<H: Hash> Kdf for Hkdf<H> {
    /// RFC 5869 §2.2: `PRK = HMAC-Hash(salt, IKM)`, with a salt of
    /// `HashLen` zeros when none is given.
    fn extract(salt: &[u8], ikm: &[u8], out: &mut [u8]) -> bool {
        if out.len() != H::OUT {
            return false;
        }
        let zeros = [0_u8; 64];
        let salt = if salt.is_empty() {
            zeros.get(..H::OUT).unwrap_or(&zeros)
        } else {
            salt
        };
        let tag = Hmac::<H>::tag(salt, ikm);
        let bytes = tag.as_ref();
        if bytes.len() != out.len() {
            return false;
        }
        out.copy_from_slice(bytes);
        true
    }

    /// RFC 5869 §2.3: `T(n) = HMAC-Hash(PRK, T(n-1) ‖ info ‖ n)`,
    /// with `T(0)` empty and `n` counting from 1.
    fn expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> bool {
        // §2.3: "L: length of output keying material in octets
        // (<= 255*HashLen)". A longer request is refused rather than
        // truncated, because the counter is one byte and would wrap.
        let max = H::OUT.saturating_mul(255);
        if out.len() > max {
            return false;
        }
        let mut previous = [0_u8; 64];
        let mut previous_len = 0_usize;
        let mut counter: u8 = 1;
        let mut written = 0_usize;

        while written < out.len() {
            let mut m = Hmac::<H>::new(prk);
            if let Some(t) = previous.get(..previous_len) {
                m.update(t);
            }
            m.update(info);
            m.update(&[counter]);
            let tag = m.finish();
            let block = tag.as_ref();

            let remaining = out.len().saturating_sub(written);
            let take = block.len().min(remaining);
            let end = written.saturating_add(take);
            if let (Some(slot), Some(src)) = (out.get_mut(written..end), block.get(..take)) {
                slot.copy_from_slice(src);
            } else {
                return false;
            }
            written = end;

            let keep = block.len().min(previous.len());
            if let (Some(slot), Some(src)) = (previous.get_mut(..keep), block.get(..keep)) {
                slot.copy_from_slice(src);
            }
            previous_len = keep;

            // 255 blocks is the most the length check above allows, so
            // this cannot reach 0 before the loop ends; saturating
            // rather than wrapping so that a future change to the
            // check cannot silently repeat block 0.
            counter = counter.saturating_add(1);
        }
        true
    }
}

/// TLS 1.3's `HKDF-Expand-Label` (RFC 8446 §7.1).
///
/// The info field is the `HkdfLabel` structure:
///
/// ```text
/// struct {
///     uint16 length = Length;
///     opaque label<7..255> = "tls13 " + Label;
///     opaque context<0..255> = Context;
/// } HkdfLabel;
/// ```
///
/// The `"tls13 "` prefix is what keeps TLS's derivations from
/// colliding with another protocol's over the same secret, and the
/// two one-byte lengths are what keep a label and a context from
/// being read as one another.
///
/// Returns `false` when the label or context is too long for its
/// one-byte length, or when the output is longer than HKDF allows.
pub fn expand_label<H: Hash>(prk: &[u8], label: &[u8], context: &[u8], out: &mut [u8]) -> bool {
    const PREFIX: &[u8] = b"tls13 ";
    let label_len = PREFIX.len().saturating_add(label.len());
    if label_len > 255 || context.len() > 255 {
        return false;
    }
    let Ok(out_len) = u16::try_from(out.len()) else {
        return false;
    };
    let Ok(label_len_byte) = u8::try_from(label_len) else {
        return false;
    };
    let Ok(context_len_byte) = u8::try_from(context.len()) else {
        return false;
    };

    // The structure, serialized: 2 + 1 + label + 1 + context.
    let mut info = [0_u8; 2 + 1 + 255 + 1 + 255];
    let mut at = 0_usize;
    let mut put = |src: &[u8]| -> bool {
        let end = at.saturating_add(src.len());
        if let Some(slot) = info.get_mut(at..end) {
            slot.copy_from_slice(src);
            at = end;
            true
        } else {
            false
        }
    };
    if !put(&out_len.to_be_bytes())
        || !put(&[label_len_byte])
        || !put(PREFIX)
        || !put(label)
        || !put(&[context_len_byte])
        || !put(context)
    {
        return false;
    }
    let Some(info) = info.get(..at) else {
        return false;
    };
    Hkdf::<H>::expand(prk, info, out)
}

/// HKDF-SHA-256.
pub type HkdfSha256 = Hkdf<crate::hash::Sha256>;
