//! Changing one thing a script expects, on purpose.
//!
//! Running a script through an engine and getting the same verdict as
//! the engine gives the original proves less than it looks. A renderer
//! that wrote every assertion as something vacuous would agree just as
//! well, and so would one that dropped the assertions altogether: the
//! script would still pass, and the two runs would still match.
//!
//! So one expected value is changed and the run is required to *fail*.
//! That is the claim worth making — not "the rendering parses" but
//! "the rendering still means what the script meant", which only a run
//! that can be made to fail can show.

use crate::lex::position;
use crate::script::Cmd;
use crate::value::{Pattern, Value};

/// Changes one expected value somewhere in `cmds`, and says what it
/// changed and where. `None` when the script expects no exact number
/// anywhere, and so cannot be mutated this way.
///
/// The change is the lowest bit of the first exact number an
/// `assert_return` expects. The original script passed, so the engine's
/// answer equals what was expected; a different expectation must
/// therefore fail. Flipping the lowest bit is the smallest change that
/// guarantees it.
pub fn flip_one_expected(src: &str, cmds: &mut [Cmd]) -> Option<String> {
    for c in cmds.iter_mut() {
        let Cmd::AssertReturn { expected, span, .. } = c else {
            continue;
        };
        let at = position(src, span.start);
        for p in expected.iter_mut() {
            let Pattern::Exact(v) = p else {
                continue;
            };
            let what = match v {
                Value::I32(n) => {
                    let was = *n;
                    *n ^= 1;
                    format!("i32 {was:#x} became {n:#x}")
                }
                Value::I64(n) => {
                    let was = *n;
                    *n ^= 1;
                    format!("i64 {was:#x} became {n:#x}")
                }
                Value::F32(bits) => {
                    let was = *bits;
                    *bits ^= 1;
                    format!("f32 {was:#010x} became {bits:#010x}")
                }
                Value::F64(bits) => {
                    let was = *bits;
                    *bits ^= 1;
                    format!("f64 {was:#018x} became {bits:#018x}")
                }
                Value::V128(bytes) => {
                    let was = bytes[0];
                    bytes[0] ^= 1;
                    format!("v128 byte 0 {was:#x} became {:#x}", bytes[0])
                }
                // A reference's identity is not a number to nudge, and
                // a null is null; neither gives a reliably wrong value.
                Value::RefNull(_) | Value::RefExtern(_) | Value::RefHost(_) => continue,
            };
            return Some(format!("{}:{}: {what}", at.line, at.column));
        }
    }
    None
}

#[cfg(test)]
mod tests;
