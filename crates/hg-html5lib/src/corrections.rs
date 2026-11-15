//! The suite's known-wrong expectations, corrected at load (ST-D21).
//!
//! The owner's answer of 15 November 2026 to **ST-D21** was *"fork and
//! add patches"*. A pushed fork would be a second remote, which the
//! owner's standing rule reserves for their own asking, so the patches
//! live here instead: a table, committed, applied to each case as it is
//! read. The pin does not move, the fetched tree stays byte-identical to
//! upstream, and `tools/test-data.toml` keeps one digest.
//!
//! The defects are measured in `research_notes/ST-M0 design review/
//! html5lib-oracle-defects.md` and reported upstream as
//! html5lib/html5lib-tests#202. All of them are in `test3.test`.
//!
//! # Every entry names the value it replaces
//!
//! A correction carries the exact value it expects to find. If upstream
//! fixes a case, the entry stops matching and [`apply`] **fails the
//! load** rather than silently doing nothing. That is the point: a patch
//! set that quietly becomes a no-op is how a fork rots, and this one
//! cannot, because the next re-pin reports it as a finding. There is no
//! fuzz, no context matching and no "apply if present".
//!
//! # The three evidence classes, which are not equally strong
//!
//! - [`Class::LintProven`] — in `test3.test` the description *is* the
//!   input for 1,247 of its 1,606 cases, and a lint over that identity
//!   reports 12 disagreements, 3 of them benign. The 9 left are inputs
//!   that lost a character their own description still promises. This is
//!   the strongest class: the file contradicts itself and only one
//!   reading survives.
//! - [`Class::SpecDerived`] — the column follows from the standard.
//!   §13.2.5.75 and §13.2.5.76 both "Emit an end-of-file token" at EOF,
//!   so the PI token is never emitted and the error is at the end of the
//!   input. Cases 1158-1166 already have it there.
//! - [`Class::SeriesInferred`] — case 1174's stray `?`. Weaker, because
//!   nothing in the file contradicts itself: it is an argument about
//!   intent. It is gated by [`SERIES_INFERRED`] so it can be dropped
//!   without touching anything else, and [`Gate`] carries the column
//!   that case needs in each case.
//!
//! The argument is a **transposition**. Series 1 (1167-1174) is eight
//! inputs of the form `<?x x` and series 2 (1175-1182) eight of the form
//! `<?x x?`. 1174 is series 1's only member carrying a trailing `?`, and
//! 1182 is series 2's only member lacking one. Applying both hunks moves
//! one character between two rows, and removes the file's last duplicate
//! pair rather than creating one. A single character in the wrong row is
//! a much better account of the evidence than eight characters lost in
//! the same place, which is why this hunk is offered at all.

/// The file every correction here applies to.
pub const FILE: &str = "test3.test";

/// Whether the [`Class::SeriesInferred`] hunks are applied.
///
/// Set to `false` to run against the suite's own case 1174. Nothing else
/// needs to change: the [`Gate`] on each entry selects the column that
/// goes with the input either way, and [`check`] proves exactly one of
/// the two is live.
pub const SERIES_INFERRED: bool = true;

/// Which field of a case a correction changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// Its `input`, decoded.
    Input,
    /// Its `description`.
    Description,
    /// The column of its `errors[nth]`, after
    /// [`cases::NOT_THE_TOKENIZERS`](crate::cases::NOT_THE_TOKENIZERS)
    /// has dropped the ones this crate does not judge.
    ErrorCol {
        /// Which error, counting from zero, in the case's `errors`
        /// array as the file has it — before
        /// [`NOT_THE_TOKENIZERS`](crate::cases::NOT_THE_TOKENIZERS)
        /// drops any.
        nth: usize,
        /// The code that error must have. The index alone is the
        /// fragile part of a patch set, so the name carries with it.
        code: &'static str,
    },
}

/// A value a correction reads or writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    /// A string field.
    Text(&'static str),
    /// A column.
    Col(u32),
}

/// How well evidenced a correction is. See the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// The file contradicts itself and only one reading survives.
    LintProven,
    /// The standard fixes the value.
    SpecDerived,
    /// An argument about intent, from the shape of the series.
    SeriesInferred,
}

/// When a correction applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    /// Always.
    Always,
    /// Only when [`SERIES_INFERRED`] is set.
    WithSeriesInferred,
    /// Only when it is not.
    WithoutSeriesInferred,
}

impl Gate {
    /// Whether this gate is open.
    #[must_use]
    pub const fn open(self) -> bool {
        match self {
            Self::Always => true,
            Self::WithSeriesInferred => SERIES_INFERRED,
            Self::WithoutSeriesInferred => !SERIES_INFERRED,
        }
    }
}

/// One correction: a case, a field, the value it must find, and the
/// value it writes.
#[derive(Debug, Clone, Copy)]
pub struct Correction {
    /// Its index in the file's `tests` array, counting from zero.
    pub index: usize,
    /// Which field it changes.
    pub field: Field,
    /// The value it must find there.
    pub from: Value,
    /// The value it writes.
    pub to: Value,
    /// How well evidenced it is.
    pub class: Class,
    /// When it applies.
    pub gate: Gate,
}

/// The table.
pub const CORRECTIONS: &[Correction] = &[
    Correction {
        // the case's own description states the input, and in this file
        // description == input for 1,247 of 1,606 cases
        index: 1175,
        field: Field::Input,
        from: Value::Text("<?a a"),
        to: Value::Text("<?a a?"),
        class: Class::LintProven,
        gate: Gate::Always,
    },
    Correction {
        // the case's own description states the input, and in this file
        // description == input for 1,247 of 1,606 cases
        index: 1176,
        field: Field::Input,
        from: Value::Text("<?b b"),
        to: Value::Text("<?b b?"),
        class: Class::LintProven,
        gate: Gate::Always,
    },
    Correction {
        // the case's own description states the input, and in this file
        // description == input for 1,247 of 1,606 cases
        index: 1177,
        field: Field::Input,
        from: Value::Text("<?y y"),
        to: Value::Text("<?y y?"),
        class: Class::LintProven,
        gate: Gate::Always,
    },
    Correction {
        // the case's own description states the input, and in this file
        // description == input for 1,247 of 1,606 cases
        index: 1178,
        field: Field::Input,
        from: Value::Text("<?z z"),
        to: Value::Text("<?z z?"),
        class: Class::LintProven,
        gate: Gate::Always,
    },
    Correction {
        // the case's own description states the input, and in this file
        // description == input for 1,247 of 1,606 cases
        index: 1179,
        field: Field::Input,
        from: Value::Text("<?A A"),
        to: Value::Text("<?A A?"),
        class: Class::LintProven,
        gate: Gate::Always,
    },
    Correction {
        // the case's own description states the input, and in this file
        // description == input for 1,247 of 1,606 cases
        index: 1180,
        field: Field::Input,
        from: Value::Text("<?B B"),
        to: Value::Text("<?B B?"),
        class: Class::LintProven,
        gate: Gate::Always,
    },
    Correction {
        // the case's own description states the input, and in this file
        // description == input for 1,247 of 1,606 cases
        index: 1181,
        field: Field::Input,
        from: Value::Text("<?Y Y"),
        to: Value::Text("<?Y Y?"),
        class: Class::LintProven,
        gate: Gate::Always,
    },
    Correction {
        // the case's own description states the input, and in this file
        // description == input for 1,247 of 1,606 cases
        index: 1182,
        field: Field::Input,
        from: Value::Text("<?Z Z"),
        to: Value::Text("<?Z Z?"),
        class: Class::LintProven,
        gate: Gate::Always,
    },
    Correction {
        // the case's own description states the input, and in this file
        // description == input for 1,247 of 1,606 cases; written doubly
        // escaped here, so the NUL is literal
        index: 500,
        field: Field::Input,
        from: Value::Text("<!DOCTYPE a SYSTEM \u{0}"),
        to: Value::Text("<!DOCTYPE a SYSTEM x\u{0}"),
        class: Class::LintProven,
        gate: Gate::Always,
    },
    Correction {
        // the restored x moves the NUL one to the right: bogus DOCTYPE state,
        // unexpected-null-character
        index: 500,
        field: Field::ErrorCol {
            nth: 1,
            code: "unexpected-null-character",
        },
        from: Value::Col(20),
        to: Value::Col(21),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1167,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(6),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1168,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(6),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1169,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(6),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1170,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(6),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1171,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(6),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1172,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(6),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1173,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(6),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it; the end depends on the
        // series-inferred hunk below
        index: 1174,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(6),
        class: Class::SpecDerived,
        gate: Gate::WithSeriesInferred,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it; the input as the suite
        // has it, with its trailing ?
        index: 1174,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(7),
        class: Class::SpecDerived,
        gate: Gate::WithoutSeriesInferred,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1175,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(7),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1176,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(7),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1177,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(7),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1178,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(7),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1179,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(7),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1180,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(7),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1181,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(7),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // PI data and PI questionable states emit only an end-of-file token at
        // EOF (HTML 13.2.5.75, 13.2.5.76), so the column is the end of the
        // input, as cases 1158-1166 already have it
        index: 1182,
        field: Field::ErrorCol {
            nth: 0,
            code: "eof-in-processing-instruction",
        },
        from: Value::Col(4),
        to: Value::Col(7),
        class: Class::SpecDerived,
        gate: Gate::Always,
    },
    Correction {
        // a transposition: 1174 is series 1's only member carrying a trailing
        // ?, and 1182 is series 2's only member lacking one, so the character
        // is in the wrong row. Applying both hunks swaps it back and removes
        // the file's last duplicate
        index: 1174,
        field: Field::Input,
        from: Value::Text("<?Z Z?"),
        to: Value::Text("<?Z Z"),
        class: Class::SeriesInferred,
        gate: Gate::WithSeriesInferred,
    },
    Correction {
        // a transposition: 1174 is series 1's only member carrying a trailing
        // ?, and 1182 is series 2's only member lacking one, so the character
        // is in the wrong row. Applying both hunks swaps it back and removes
        // the file's last duplicate
        index: 1174,
        field: Field::Description,
        from: Value::Text("<?Z Z?"),
        to: Value::Text("<?Z Z"),
        class: Class::SeriesInferred,
        gate: Gate::WithSeriesInferred,
    },
];

/// The fields of one case a correction may change.
pub struct Target<'a> {
    /// Its description.
    pub description: &'a mut String,
    /// Its decoded input.
    pub input: &'a mut String,
    /// Its errors, as the file has them: before
    /// [`NOT_THE_TOKENIZERS`](crate::cases::NOT_THE_TOKENIZERS) drops
    /// any, so an entry's `nth` does not move when that list changes.
    pub errors: &'a mut [crate::ExpectedError],
}

/// Applies every open correction for one case, and returns how many.
///
/// # Errors
/// A message when an entry does not find the value it names. That is the
/// expiry: upstream has fixed the case, or renumbered it, or the entry
/// was wrong — each of which is a finding, and none of which may pass as
/// a silent no-op.
pub fn apply(file: &str, index: usize, t: Target<'_>) -> Result<usize, String> {
    if file != FILE {
        return Ok(0);
    }
    let mut n = 0;
    for c in CORRECTIONS
        .iter()
        .filter(|c| c.index == index && c.gate.open())
    {
        let where_ = format!("{FILE} case {index} ({:?})", c.field);
        match (c.field, c.from, c.to) {
            (Field::Input, Value::Text(from), Value::Text(to)) => {
                if t.input != from {
                    return Err(mismatch(&where_, from, t.input));
                }
                *t.input = to.to_owned();
            }
            (Field::Description, Value::Text(from), Value::Text(to)) => {
                if t.description != from {
                    return Err(mismatch(&where_, from, t.description));
                }
                *t.description = to.to_owned();
            }
            (Field::ErrorCol { nth, code }, Value::Col(from), Value::Col(to)) => {
                let Some(e) = t.errors.get_mut(nth) else {
                    return Err(format!(
                        "{where_}: no error {nth}; the case has {}",
                        t.errors.len()
                    ));
                };
                if e.code != code {
                    return Err(mismatch(&where_, code, &e.code));
                }
                if e.col != from {
                    return Err(mismatch(&where_, &from.to_string(), &e.col.to_string()));
                }
                e.col = to;
            }
            _ => return Err(format!("{where_}: the entry's field and values disagree")),
        }
        n += 1;
    }
    Ok(n)
}

fn mismatch(where_: &str, want: &str, got: &str) -> String {
    format!(
        "{where_}: expected {want:?} and found {got:?}. A correction that \
         no longer matches is a finding, not a no-op: the pin may have \
         moved, upstream may have fixed the case \
         (html5lib/html5lib-tests#202), or the entry was wrong. Re-measure \
         with `research_notes/ST-M0 design review/html5lib-oracle-defects.md` \
         before changing either."
    )
}

/// How many corrections are open, which is how many [`apply`] must have
/// made over a whole load.
#[must_use]
pub fn open_count() -> usize {
    CORRECTIONS.iter().filter(|c| c.gate.open()).count()
}

/// How many open corrections are of each class.
#[must_use]
pub fn by_class(class: Class) -> usize {
    CORRECTIONS
        .iter()
        .filter(|c| c.gate.open() && c.class == class)
        .count()
}

/// Checks the table itself, independently of any data.
///
/// # Errors
/// A message when two entries would fight over the same field, or when
/// the gates do not select exactly one value for a gated field.
pub fn check() -> Result<(), String> {
    let mut seen: Vec<(usize, String)> = Vec::new();
    for c in CORRECTIONS.iter().filter(|c| c.gate.open()) {
        let key = (c.index, format!("{:?}", c.field));
        if seen.contains(&key) {
            return Err(format!(
                "two open corrections change {FILE} case {} {:?}",
                c.index, c.field
            ));
        }
        seen.push(key);
    }
    // Every gated entry must have a counterpart under the other gate,
    // so dropping the series-inferred hunks leaves no field unanswered.
    for c in CORRECTIONS {
        let other = match c.gate {
            Gate::Always => continue,
            Gate::WithSeriesInferred => Gate::WithoutSeriesInferred,
            Gate::WithoutSeriesInferred => Gate::WithSeriesInferred,
        };
        if c.class == Class::SeriesInferred {
            // The hunk itself has no counterpart: dropping it means
            // leaving the field as the suite has it.
            continue;
        }
        if !CORRECTIONS
            .iter()
            .any(|o| o.index == c.index && o.field == c.field && o.gate == other)
        {
            return Err(format!(
                "{FILE} case {} {:?} is gated with no counterpart: dropping \
                 the series-inferred hunks would leave it uncorrected",
                c.index, c.field
            ));
        }
    }
    Ok(())
}
