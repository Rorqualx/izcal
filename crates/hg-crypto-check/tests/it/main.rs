//! `hg-crypto-check`'s tests: the Wycheproof run against its recorded
//! baseline, the timing test against its planted leak, and
//! `hg-crypto`'s pins record.

#[cfg(test)]
mod pins;
#[cfg(test)]
mod timing;
#[cfg(test)]
mod wycheproof;
