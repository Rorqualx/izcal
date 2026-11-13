//! Checking a parsed `.hgfont` file, and working out each table's
//! layout.
//!
//! Everything the emitter needs to be total is decided here, so the
//! generated reader has no case it cannot handle: record sizes are
//! constants, every name resolves, and every count expression names only
//! a parameter or an earlier scalar field of the same table.

use std::collections::{BTreeMap, BTreeSet};

use crate::syntax::{Error, Expr, Field, File, Record, Scalar, Table, Term, Ty};

/// A checked set of files, ready to emit.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// Records by name, with their byte size.
    pub records: BTreeMap<String, (Record, usize)>,
    /// Tables, in the order declared.
    pub tables: Vec<Table>,
    /// Every anchor any item or field carries, sorted and unique.
    pub anchors: BTreeSet<String>,
}

impl Model {
    /// The byte size of one element of `ty`, which is a constant for
    /// every type an array may hold.
    ///
    /// # Errors
    /// [`Error`] when a record name does not resolve.
    pub fn elem_size(&self, ty: &Ty, file: &str, line: usize) -> Result<usize, Error> {
        match ty {
            Ty::Scalar(s) => Ok(s.width()),
            Ty::Record(n) => self
                .records
                .get(n)
                .map(|(_, size)| *size)
                .ok_or_else(|| err(file, line, format!("no record called {n}"))),
            Ty::Array { .. } => Err(err(file, line, "an array may not hold an array")),
        }
    }
}

fn err(file: &str, line: usize, message: impl Into<String>) -> Error {
    Error {
        file: file.to_owned(),
        line,
        message: message.into(),
    }
}

fn is_snake(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && s.starts_with(|c: char| c.is_ascii_lowercase())
}

fn is_camel(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric())
        && s.starts_with(|c: char| c.is_ascii_uppercase())
}

/// Checks every file and builds the model.
///
/// # Errors
/// [`Error`] for the first problem found, naming the file and line.
pub fn check(files: &[(String, File)]) -> Result<Model, Error> {
    let mut m = Model::default();

    // Records first: a table may use any record, whatever the order.
    let mut pending: Vec<(String, Record)> = Vec::new();
    for (name, f) in files {
        for r in &f.records {
            if !is_camel(&r.name) {
                return Err(err(
                    name,
                    r.line,
                    format!("record {} is not UpperCamelCase", r.name),
                ));
            }
            if pending.iter().any(|(_, o)| o.name == r.name) {
                return Err(err(
                    name,
                    r.line,
                    format!("record {} declared twice", r.name),
                ));
            }
            pending.push((name.clone(), r.clone()));
        }
    }

    // A record holds only scalars and other records, so its size is a
    // constant. Resolving in dependency order also rejects cycles: a
    // pass that resolves nothing means every remaining record waits on
    // another.
    while !pending.is_empty() {
        let before = pending.len();
        let mut again = Vec::new();
        for (file, r) in pending {
            match record_size(&r, &m, &file) {
                Ok(Some(size)) => {
                    collect_anchors_record(&r, &mut m.anchors);
                    m.records.insert(r.name.clone(), (r, size));
                }
                Ok(None) => again.push((file, r)),
                Err(e) => return Err(e),
            }
        }
        pending = again;
        if pending.len() == before {
            let names: Vec<&str> = pending.iter().map(|(_, r)| r.name.as_str()).collect();
            let (file, r) = pending.first().ok_or_else(|| err("", 0, "unreachable"))?;
            return Err(err(
                file,
                r.line,
                format!(
                    "these records cannot be sized; one of them names a record that does not \
                     exist, or they refer to each other: {}",
                    names.join(", ")
                ),
            ));
        }
    }

    for (file, f) in files {
        for t in &f.tables {
            check_table(t, &mut m, file)?;
        }
    }
    Ok(m)
}

/// `Ok(None)` when a record it names is not sized yet.
fn record_size(r: &Record, m: &Model, file: &str) -> Result<Option<usize>, Error> {
    let mut size = 0_usize;
    let mut seen = BTreeSet::new();
    for f in &r.fields {
        if !is_snake(&f.name) {
            return Err(err(
                file,
                f.line,
                format!("field {} is not snake_case", f.name),
            ));
        }
        if !seen.insert(f.name.clone()) {
            return Err(err(
                file,
                f.line,
                format!("field {} declared twice in {}", f.name, r.name),
            ));
        }
        let w = match &f.ty {
            Ty::Scalar(s) => s.width(),
            Ty::Record(n) => match m.records.get(n) {
                Some((_, size)) => *size,
                None => return Ok(None),
            },
            Ty::Array { .. } => {
                return Err(err(
                    file,
                    f.line,
                    format!(
                        "record {} may not hold an array: a record's size must be a constant, so \
                         that an array of them can be indexed",
                        r.name
                    ),
                ));
            }
        };
        size = size
            .checked_add(w)
            .ok_or_else(|| err(file, f.line, format!("record {} is too large", r.name)))?;
    }
    Ok(Some(size))
}

fn check_table(t: &Table, m: &mut Model, file: &str) -> Result<(), Error> {
    if !is_camel(&t.name) {
        return Err(err(
            file,
            t.line,
            format!("table {} is not UpperCamelCase", t.name),
        ));
    }
    if m.tables.iter().any(|o| o.name == t.name) {
        return Err(err(
            file,
            t.line,
            format!("table {} declared twice", t.name),
        ));
    }

    let mut names: BTreeMap<String, Scalar> = BTreeMap::new();
    for (p, s) in &t.params {
        if !is_snake(p) {
            return Err(err(
                file,
                t.line,
                format!("parameter {p} of {} is not snake_case", t.name),
            ));
        }
        if names.insert(p.clone(), *s).is_some() {
            return Err(err(
                file,
                t.line,
                format!("parameter {p} of {} declared twice", t.name),
            ));
        }
    }

    let mut fields = BTreeSet::new();
    for f in &t.fields {
        if !is_snake(&f.name) {
            return Err(err(
                file,
                f.line,
                format!("field {} is not snake_case", f.name),
            ));
        }
        if !fields.insert(f.name.clone()) {
            return Err(err(
                file,
                f.line,
                format!("field {} declared twice in {}", f.name, t.name),
            ));
        }
        if names.contains_key(&f.name) {
            return Err(err(
                file,
                f.line,
                format!("field {} has the same name as a parameter", f.name),
            ));
        }
        match &f.ty {
            Ty::Scalar(s) => {
                names.insert(f.name.clone(), *s);
            }
            Ty::Record(n) => {
                if !m.records.contains_key(n) {
                    return Err(err(file, f.line, format!("no record called {n}")));
                }
            }
            Ty::Array { count, elem } => {
                check_expr(count, &names, file, f.line, &f.name)?;
                m.elem_size(elem, file, f.line)?;
                if matches!(**elem, Ty::Array { .. }) {
                    return Err(err(file, f.line, "an array may not hold an array"));
                }
            }
        }
        collect_anchor(&f.anchor, &mut m.anchors);
    }
    collect_anchor(&t.anchor, &mut m.anchors);
    m.tables.push(t.clone());
    Ok(())
}

/// A count expression may name only a parameter or an earlier scalar
/// field: anything else would need the reader to look forward, which it
/// cannot do without knowing the length it is computing.
fn check_expr(
    e: &Expr,
    in_scope: &BTreeMap<String, Scalar>,
    file: &str,
    line: usize,
    field: &str,
) -> Result<(), Error> {
    for (_, t) in &e.terms {
        if let Term::Name(n) = t {
            let Some(s) = in_scope.get(n) else {
                return Err(err(
                    file,
                    line,
                    format!(
                        "the length of {field} names {n}, which is not a parameter of this table \
                         or one of its earlier scalar fields"
                    ),
                ));
            };
            if matches!(s, Scalar::Tag) {
                return Err(err(
                    file,
                    line,
                    format!("the length of {field} names {n}, which is a tag, not a number"),
                ));
            }
        }
    }
    Ok(())
}

fn collect_anchor(a: &str, out: &mut BTreeSet<String>) {
    out.insert(a.to_owned());
}

fn collect_anchors_record(r: &Record, out: &mut BTreeSet<String>) {
    collect_anchor(&r.anchor, out);
    for f in &r.fields {
        collect_anchor(&f.anchor, out);
    }
}

/// Whether `f` is an array.
#[must_use]
pub const fn is_array(f: &Field) -> bool {
    matches!(f.ty, Ty::Array { .. })
}
