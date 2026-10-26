//! This build's identity (distribution and updates 13 item 5, DU-M0a).
//!
//! `BuildId` is a truncated digest of the build's inputs, computed before
//! compilation by `hg-build-id` and passed to the compiler as the
//! environment variable `IZCAL_BUILD_ID` (16 hexadecimal digits). Every
//! binary of one build reads it here, so every process of the build
//! reports the same value (`process.hgidl`'s `ready`), and a child of
//! another build cannot report the parent's. Cargo rebuilds when the
//! variable changes.
//!
//! A build made without the variable carries [`UNPINNED_BUILD_ID`], the
//! `BuildId` of `hg-build-id`'s fixed "unpinned" inputs (its test checks
//! the value). Two unpinned builds cannot tell each other apart, which
//! only development builds accept: every distributed build is pinned.

/// The `BuildId` of a build made without `IZCAL_BUILD_ID`.
pub const UNPINNED_BUILD_ID: u64 = 0x7031_6f7b_90df_26cc;

/// This build's `BuildId`.
pub const BUILD_ID: u64 = match option_env!("IZCAL_BUILD_ID") {
    Some(hex) => parse_hex16(hex),
    None => UNPINNED_BUILD_ID,
};

/// Whether this build was made with `IZCAL_BUILD_ID`.
pub const BUILD_IS_PINNED: bool = option_env!("IZCAL_BUILD_ID").is_some();

/// Sixteen hexadecimal digits as a `u64`. Evaluated only at compile time,
/// for [`BUILD_ID`]: a malformed `IZCAL_BUILD_ID` stops the build.
#[expect(
    clippy::panic,
    reason = "compile-time only: a malformed IZCAL_BUILD_ID must stop the build"
)]
const fn parse_hex16(s: &str) -> u64 {
    let mut rest = s.as_bytes();
    if rest.len() != 16 {
        panic!("IZCAL_BUILD_ID must be 16 hexadecimal digits");
    }
    let mut v = 0u64;
    while let [c, tail @ ..] = rest {
        let d: u64 = match *c {
            b'0' => 0,
            b'1' => 1,
            b'2' => 2,
            b'3' => 3,
            b'4' => 4,
            b'5' => 5,
            b'6' => 6,
            b'7' => 7,
            b'8' => 8,
            b'9' => 9,
            b'a' | b'A' => 10,
            b'b' | b'B' => 11,
            b'c' | b'C' => 12,
            b'd' | b'D' => 13,
            b'e' | b'E' => 14,
            b'f' | b'F' => 15,
            _ => panic!("IZCAL_BUILD_ID must be 16 hexadecimal digits"),
        };
        v = v.wrapping_shl(4) | d;
        rest = tail;
    }
    v
}

#[cfg(test)]
mod tests {
    use super::parse_hex16;

    #[test]
    fn parses_both_cases() {
        assert_eq!(parse_hex16("0123456789abcdef"), 0x0123_4567_89ab_cdef);
        assert_eq!(parse_hex16("FEDCBA9876543210"), 0xfedc_ba98_7654_3210);
    }
}
