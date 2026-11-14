//! `hg-raster-ref`'s tests: the exact rasterizer against shapes whose
//! answer is arithmetic, and `hg-raster` against the exact one.

#[cfg(test)]
mod differential;
#[cfg(test)]
mod exactness;
