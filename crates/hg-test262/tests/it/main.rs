//! `hg-test262` integration tests (CS-TE-2: one binary per crate): the
//! runner end to end over bundled synthetic test files (`t262/`, written
//! for these tests in test262's layout, none copied from test262) with a
//! fake engine adapter that follows `// FAKE:` directives instead of
//! running JavaScript.

#[cfg(test)]
mod fake;
#[cfg(test)]
mod run;
