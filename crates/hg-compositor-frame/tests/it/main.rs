//! `hg-compositor-frame`'s tests: the fifteen things the wire format
//! cannot say, the ten counts it can, GR-18's published size, and the
//! checked-in generated code.

#[cfg(test)]
mod fresh;
#[cfg(test)]
mod size;
#[cfg(test)]
mod validation;
#[cfg(test)]
mod wire;
