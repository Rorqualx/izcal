//! `hg-fontc`'s tests: the generated writers against bytes transcribed
//! from the OpenType prose, against the generated readers, and against
//! the mistakes a font compiler makes.

#[cfg(test)]
mod bytes;
#[cfg(test)]
mod refusals;
#[cfg(test)]
mod roundtrip;
