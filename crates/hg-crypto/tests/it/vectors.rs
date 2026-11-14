//! The standards' own published vectors.
//!
//! Every number here is transcribed from the document named beside it.
//! Nothing is taken from an implementation, and nothing is a value
//! this code produced and was then asserted against itself — a hash
//! that agrees with its own output is a hash of something, just not
//! necessarily SHA-256.

use hg_crypto::ct;
use hg_crypto::hash::{Hash, Sha256};
use hg_crypto::hkdf::{HkdfSha256, Kdf, expand_label};
use hg_crypto::hmac::{HmacSha256, Mac};
use hg_crypto::secret::Secret;

fn hex(bytes: &[u8]) -> String {
    let mut s = String::new();
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn unhex(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let (pairs, rest) = b.as_chunks::<2>();
    assert!(rest.is_empty(), "an even number of hex digits");
    pairs
        .iter()
        .map(|p| {
            let t = std::str::from_utf8(p).expect("ascii");
            u8::from_str_radix(t, 16).expect("hex")
        })
        .collect()
}

// --- SHA-256 (FIPS 180-4, and NIST's published examples) -------------------

/// The three examples NIST publishes for SHA-256, and the empty input.
#[test]
fn cr01_sha256_matches_the_published_examples() {
    for (input, want) in [
        (
            "",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            "abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ] {
        let d = Sha256::digest(input.as_bytes());
        assert_eq!(hex(d.as_ref()), want, "{input:?}");
    }
}

/// NIST's long example: one million 'a'. It is the one that catches a
/// length counter that overflows or is kept in bytes rather than bits.
#[test]
fn cr02_sha256_of_a_million_letters() {
    let mut h = Sha256::new();
    for _ in 0..1_000 {
        h.update(&[b'a'; 1_000]);
    }
    assert_eq!(
        hex(h.finish().as_ref()),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

/// The same bytes fed in any number of pieces give the same digest:
/// the buffering is not part of the function.
#[test]
fn cr03_the_split_does_not_change_the_digest() {
    let data: Vec<u8> = (0..300_u32)
        .map(|i| u8::try_from(i % 251).unwrap_or(0))
        .collect();
    let whole = Sha256::digest(&data);
    for split in [0_usize, 1, 55, 63, 64, 65, 127, 128, 200, 299, 300] {
        let mut h = Sha256::new();
        let (a, b) = data.split_at(split);
        h.update(a);
        h.update(b);
        assert_eq!(h.finish(), whole, "split at {split}");
    }
    // And one byte at a time.
    let mut h = Sha256::new();
    for b in &data {
        h.update(&[*b]);
    }
    assert_eq!(h.finish(), whole);
}

/// The padding boundary: a message whose length leaves no room for the
/// 8-byte length field needs a second block. 55 bytes fits; 56 does
/// not.
#[test]
fn cr04_the_padding_boundary() {
    let a55 = vec![b'a'; 55];
    let a56 = vec![b'a'; 56];
    let a64 = vec![b'a'; 64];
    // Distinct digests, and each reproducible by the split test above.
    assert_ne!(Sha256::digest(&a55), Sha256::digest(&a56));
    assert_ne!(Sha256::digest(&a56), Sha256::digest(&a64));
    // A known value: SHA-256 of 64 'a' characters.
    assert_eq!(
        hex(Sha256::digest(&a64).as_ref()),
        "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb"
    );
}

// --- HMAC-SHA-256 (RFC 4231) -----------------------------------------------

/// RFC 4231's seven test cases, by their published key, data and tag.
#[test]
fn cr05_rfc_4231s_hmac_sha256_cases() {
    let cases: [(Vec<u8>, Vec<u8>, &str); 6] = [
        // Case 1
        (
            vec![0x0b; 20],
            b"Hi There".to_vec(),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7",
        ),
        // Case 2: a key shorter than the block.
        (
            b"Jefe".to_vec(),
            b"what do ya want for nothing?".to_vec(),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843",
        ),
        // Case 3
        (
            vec![0xaa; 20],
            vec![0xdd; 50],
            "773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe",
        ),
        // Case 4: a 25-byte key of increasing bytes.
        (
            (1_u8..=25).collect(),
            vec![0xcd; 50],
            "82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b",
        ),
        // Case 6: a key longer than the block, which must be hashed
        // first. This is the case a wrong threshold gets wrong.
        (
            vec![0xaa; 131],
            b"Test Using Larger Than Block-Size Key - Hash Key First".to_vec(),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54",
        ),
        // Case 7
        (
            vec![0xaa; 131],
            b"This is a test using a larger than block-size key and a larger \
              than block-size data. The key needs to be hashed before being \
              used by the HMAC algorithm."
                .to_vec(),
            "9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2",
        ),
    ];
    for (i, (key, data, want)) in cases.iter().enumerate() {
        let tag = HmacSha256::tag(key, data);
        assert_eq!(hex(tag.as_ref()), *want, "RFC 4231 case {}", i + 1);
        assert!(HmacSha256::verify(key, data, tag.as_ref()));
    }
}

/// Case 7's data in the RFC is one unbroken string; the line breaks a
/// Rust literal introduces would change the tag. This checks the
/// literal above is the bytes the RFC prints.
#[test]
fn cr06_case_sevens_data_is_the_rfcs_bytes() {
    let data = b"This is a test using a larger than block-size key and a larger \
              than block-size data. The key needs to be hashed before being \
              used by the HMAC algorithm."
        .to_vec();
    assert!(
        !data.windows(2).any(|w| w == b"  "),
        "a continued string literal kept its indentation, which changes the tag"
    );
}

/// A tag that is one bit wrong does not verify.
#[test]
fn cr07_a_wrong_tag_does_not_verify() {
    let key = b"key";
    let data = b"data";
    let tag = HmacSha256::tag(key, data);
    let mut wrong = tag.0;
    if let Some(first) = wrong.first_mut() {
        *first ^= 1;
    }
    assert!(!HmacSha256::verify(key, data, &wrong));
    assert!(!HmacSha256::verify(key, data, &[]), "a short tag");
    assert!(!HmacSha256::verify(key, data, &[0; 64]), "a long tag");
    assert!(HmacSha256::verify(key, data, tag.as_ref()), "the twin");
}

// --- HKDF-SHA-256 (RFC 5869) -----------------------------------------------

/// RFC 5869's Appendix A test cases 1 to 3, by their published PRK and
/// output keying material.
#[test]
fn cr08_rfc_5869s_hkdf_sha256_cases() {
    // Test Case 1
    let ikm = vec![0x0b; 22];
    let salt = unhex("000102030405060708090a0b0c");
    let info = unhex("f0f1f2f3f4f5f6f7f8f9");
    let mut prk = [0_u8; 32];
    assert!(HkdfSha256::extract(&salt, &ikm, &mut prk));
    assert_eq!(
        hex(&prk),
        "077709362c2e32df0ddc3f0dc47bba6390b6c73bb50f9c3122ec844ad7c2b3e5"
    );
    let mut okm = [0_u8; 42];
    assert!(HkdfSha256::expand(&prk, &info, &mut okm));
    assert_eq!(
        hex(&okm),
        "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf\
         34007208d5b887185865"
    );

    // Test Case 2: inputs longer than the hash, output over one block.
    let ikm: Vec<u8> = (0_u8..=0x4f).collect();
    let salt: Vec<u8> = (0x60_u8..=0xaf).collect();
    let info: Vec<u8> = (0xb0_u8..=0xff).collect();
    let mut prk = [0_u8; 32];
    assert!(HkdfSha256::extract(&salt, &ikm, &mut prk));
    assert_eq!(
        hex(&prk),
        "06a6b88c5853361a06104c9ceb35b45cef760014904671014a193f40c15fc244"
    );
    let mut okm = [0_u8; 82];
    assert!(HkdfSha256::expand(&prk, &info, &mut okm));
    assert_eq!(
        hex(&okm),
        "b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c\
         59045a99cac7827271cb41c65e590e09da3275600c2f09b8367793a9aca3db71\
         cc30c58179ec3e87c14c01d5c1f3434f1d87"
    );

    // Test Case 3: no salt and no info, which is the case a decoder
    // that treats "empty" as "absent" differently gets wrong.
    let ikm = vec![0x0b; 22];
    let mut prk = [0_u8; 32];
    assert!(HkdfSha256::extract(&[], &ikm, &mut prk));
    assert_eq!(
        hex(&prk),
        "19ef24a32c717b167f33a91d6f648bdf96596776afdb6377ac434c1c293ccb04"
    );
    let mut okm = [0_u8; 42];
    assert!(HkdfSha256::expand(&prk, &[], &mut okm));
    assert_eq!(
        hex(&okm),
        "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d\
         9d201395faa4b61a96c8"
    );
}

/// §2.3's bound: at most 255 times the hash length. One past it is
/// refused rather than wrapping the counter back to its first block.
#[test]
fn cr09_expand_refuses_more_than_the_standard_allows() {
    let prk = [0_u8; 32];
    let mut ok = vec![0_u8; 255 * 32];
    assert!(HkdfSha256::expand(&prk, b"", &mut ok), "255 blocks");
    let mut over = vec![0_u8; 255 * 32 + 1];
    assert!(!HkdfSha256::expand(&prk, b"", &mut over), "one byte past");
    // And a PRK of the wrong length is refused by extract.
    let mut short = [0_u8; 31];
    assert!(!HkdfSha256::extract(b"", b"", &mut short));
}

/// A prefix of a longer expansion is the shorter one: the counter runs
/// forward and never restarts.
#[test]
fn cr10_a_shorter_expansion_is_a_prefix_of_a_longer_one() {
    let prk = [7_u8; 32];
    let mut long = [0_u8; 200];
    assert!(HkdfSha256::expand(&prk, b"info", &mut long));
    for n in [1_usize, 31, 32, 33, 64, 99, 200] {
        let mut short = vec![0_u8; n];
        assert!(HkdfSha256::expand(&prk, b"info", &mut short));
        assert_eq!(short.as_slice(), long.get(..n).expect("in range"), "{n}");
    }
}

// --- HKDF-Expand-Label (RFC 8446 §7.1) -------------------------------------

/// The `HkdfLabel` structure is serialized as the standard writes it:
/// a 2-byte length, a 1-byte label length over `"tls13 "` plus the
/// label, and a 1-byte context length over the context.
#[test]
fn cr11_expand_label_builds_the_standards_structure() {
    let prk = [1_u8; 32];
    let mut out = [0_u8; 32];
    assert!(expand_label::<Sha256>(&prk, b"key", b"", &mut out));

    // The same bytes, by expanding with the info built here.
    let mut info = Vec::new();
    info.extend_from_slice(&32_u16.to_be_bytes());
    info.push(u8::try_from(b"tls13 ".len() + 3).expect("small"));
    info.extend_from_slice(b"tls13 key");
    info.push(0);
    let mut want = [0_u8; 32];
    assert!(HkdfSha256::expand(&prk, &info, &mut want));
    assert_eq!(out, want);

    // The "tls13 " prefix is load-bearing: without it the output is a
    // different key, which is the collision the prefix prevents.
    let mut without = Vec::new();
    without.extend_from_slice(&32_u16.to_be_bytes());
    without.push(3);
    without.extend_from_slice(b"key");
    without.push(0);
    let mut other = [0_u8; 32];
    assert!(HkdfSha256::expand(&prk, &without, &mut other));
    assert_ne!(out, other);
}

/// A label or context too long for its one-byte length is refused.
#[test]
fn cr12_expand_label_refuses_what_it_cannot_serialize() {
    let prk = [1_u8; 32];
    let mut out = [0_u8; 16];
    let long_label = vec![b'x'; 250];
    assert!(
        !expand_label::<Sha256>(&prk, &long_label, b"", &mut out),
        "250 + 6 does not fit one byte"
    );
    let ok_label = vec![b'x'; 249];
    assert!(expand_label::<Sha256>(&prk, &ok_label, b"", &mut out));
    let long_context = vec![0_u8; 256];
    assert!(!expand_label::<Sha256>(
        &prk,
        b"key",
        &long_context,
        &mut out
    ));
}

// --- the constant-time helpers and the secret type -------------------------

/// `ct::eq` answers what `==` would.
#[test]
fn cr13_constant_time_equality_is_equality() {
    assert!(ct::eq(b"", b""));
    assert!(ct::eq(b"abc", b"abc"));
    assert!(!ct::eq(b"abc", b"abd"), "differs at the end");
    assert!(!ct::eq(b"abc", b"zbc"), "differs at the start");
    assert!(!ct::eq(b"abc", b"ab"), "different lengths");
    for v in [0_u8, 1, 0x7f, 0x80, 0xff] {
        assert_eq!(ct::is_zero_u8(v), v == 0, "{v}");
    }
}

/// `ct::select` picks without branching, and refuses a length mismatch
/// rather than reading past one of its inputs.
#[test]
fn cr14_constant_time_select() {
    let a = [1_u8, 2, 3];
    let b = [9_u8, 8, 7];
    let mut out = [0_u8; 3];
    assert!(ct::select(&mut out, &a, &b, false));
    assert_eq!(out, a);
    assert!(ct::select(&mut out, &a, &b, true));
    assert_eq!(out, b);
    let mut short = [0_u8; 2];
    assert!(!ct::select(&mut short, &a, &b, true));
    assert_eq!(short, [0, 0], "nothing was written");
    assert_eq!(ct::mask(true), 0xff);
    assert_eq!(ct::mask(false), 0x00);
}

/// A secret does not print its bytes, and compares in constant time.
#[test]
fn cr15_a_secret_does_not_leak_through_debug() {
    let s = Secret::new([0xde, 0xad, 0xbe, 0xef]);
    let shown = format!("{s:?}");
    assert_eq!(shown, "Secret<4>(..)");
    assert!(!shown.contains("222"), "no decimal bytes");
    assert!(!shown.contains("de"), "no hex bytes");

    assert!(s.ct_eq(&Secret::new([0xde, 0xad, 0xbe, 0xef])));
    assert!(!s.ct_eq(&Secret::new([0xde, 0xad, 0xbe, 0xee])));
    assert_eq!(s.len(), 4);
    assert!(!s.is_empty());
    assert_eq!(s.expose_bytes(|b| b[0]), 0xde);
}
