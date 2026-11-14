//! `hg-raster`'s tests: the exact ÷255 over its whole range, the
//! blend modes against the specification's own identities, coverage
//! of shapes whose answer is known by construction, and image
//! sampling.

#[cfg(test)]
mod blending;
#[cfg(test)]
mod coverage;
#[cfg(test)]
mod images;
