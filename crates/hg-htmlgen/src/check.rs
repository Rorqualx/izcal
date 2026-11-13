//! Checking a parsed `.hgtok` file.
//!
//! What is checked is what a generated tokenizer cannot recover from: a
//! state that does not exist, a state nothing reaches, a rule that can
//! never fire because an earlier one already matches it, and a state
//! with no "anything else" row — which the prose always has, and whose
//! absence would leave the generated loop with a case it cannot answer.

use std::collections::{BTreeMap, BTreeSet};

use crate::syntax::{Action, Class, Error, File, Match, State};

/// A checked description.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// The states, in the order written.
    pub states: Vec<State>,
    /// Every anchor, sorted and unique.
    pub anchors: BTreeSet<String>,
    /// Every parse-error name the description uses, sorted.
    pub errors: BTreeSet<String>,
    /// Every transition, as `(from, to)`, for the coverage table.
    pub transitions: BTreeSet<(String, String)>,
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

/// The states a list of actions can move to.
fn targets(actions: &[Action], out: &mut Vec<String>) {
    for a in actions {
        match a {
            Action::Switch(s) | Action::Reconsume(s) | Action::SetReturn(s) => {
                out.push(s.clone());
            }
            Action::If {
                then, otherwise, ..
            } => {
                targets(then, out);
                targets(otherwise, out);
            }
            _ => {}
        }
    }
}

/// Collects every parse-error name.
fn errors_of(actions: &[Action], out: &mut BTreeSet<String>) {
    for a in actions {
        match a {
            Action::Error(n) => {
                out.insert(n.clone());
            }
            Action::If {
                then, otherwise, ..
            } => {
                errors_of(then, out);
                errors_of(otherwise, out);
            }
            _ => {}
        }
    }
}

/// Whether an earlier match already covers a later one, which would make
/// the later rule dead.
fn covers(earlier: &Match, later: &Match) -> bool {
    match (earlier, later) {
        (Match::Anything, _) => true,
        (a, b) if a == b => true,
        (Match::Class(c), Match::Char(ch)) => match c {
            Class::Whitespace => matches!(ch, '\t' | '\n' | '\u{000C}' | ' '),
            Class::Null => *ch == '\0',
            Class::AsciiAlpha => ch.is_ascii_alphabetic(),
            Class::AsciiUpper => ch.is_ascii_uppercase(),
            Class::AsciiLower => ch.is_ascii_lowercase(),
            Class::AsciiDigit => ch.is_ascii_digit(),
            Class::AsciiHexDigit => ch.is_ascii_hexdigit(),
        },
        (Match::Class(Class::AsciiAlpha), Match::Class(c)) => {
            matches!(c, Class::AsciiUpper | Class::AsciiLower)
        }
        (Match::Class(Class::AsciiHexDigit), Match::Class(Class::AsciiDigit)) => true,
        _ => false,
    }
}

/// The state every description must start from.
pub const START: &str = "data";

/// Checks every file and builds the model.
///
/// # Errors
/// [`Error`] for the first problem found, naming the file and line.
pub fn check(files: &[(String, File)]) -> Result<Model, Error> {
    let mut m = Model::default();
    let mut where_declared: BTreeMap<String, String> = BTreeMap::new();

    for (file, f) in files {
        for s in &f.states {
            if !is_snake(&s.name) {
                return Err(err(
                    file,
                    s.line,
                    format!("state {} is not snake_case", s.name),
                ));
            }
            if where_declared
                .insert(s.name.clone(), file.clone())
                .is_some()
            {
                return Err(err(
                    file,
                    s.line,
                    format!("state {} declared twice", s.name),
                ));
            }
            m.anchors.insert(s.anchor.clone());
            m.states.push(s.clone());
        }
    }

    if !where_declared.contains_key(START) {
        return Err(err(
            "",
            0,
            format!("no `{START}` state: the tokenizer starts there"),
        ));
    }

    for s in &m.states {
        let file = where_declared.get(&s.name).map_or("", String::as_str);
        if s.external {
            continue;
        }
        if s.rules.is_empty() {
            return Err(err(
                file,
                s.line,
                format!(
                    "state {} has no rules; mark it `extern state` if the runtime implements it",
                    s.name
                ),
            ));
        }
        let mut seen: Vec<&Match> = Vec::new();
        let mut has_anything = false;
        for r in &s.rules {
            if let Some(prev) = seen.iter().find(|p| covers(p, &r.matches)) {
                return Err(err(
                    file,
                    r.line,
                    format!(
                        "this rule can never fire in state {}: an earlier rule for {:?} already \
                         matches it",
                        s.name, prev
                    ),
                ));
            }
            if matches!(r.matches, Match::Anything) {
                has_anything = true;
            }
            seen.push(&r.matches);

            if let Some(a) = &r.anchor {
                m.anchors.insert(a.clone());
            }
            errors_of(&r.actions, &mut m.errors);
            let mut to = Vec::new();
            targets(&r.actions, &mut to);
            for t in to {
                if !where_declared.contains_key(&t) {
                    return Err(err(
                        file,
                        r.line,
                        format!("state {} moves to {t}, which does not exist", s.name),
                    ));
                }
                m.transitions.insert((s.name.clone(), t));
            }
        }
        if !has_anything {
            return Err(err(
                file,
                s.line,
                format!(
                    "state {} has no `_` rule; every state in the prose ends with an \
                     \"anything else\" row",
                    s.name
                ),
            ));
        }
    }

    // Reachability, so a state nobody can enter is a mistake and not a
    // silent orphan. `extern` states count as reachable from whatever
    // routes into them, which the transitions already record.
    let mut reached: BTreeSet<String> = BTreeSet::new();
    reached.insert(START.to_owned());
    loop {
        let before = reached.len();
        for (from, to) in &m.transitions {
            if reached.contains(from) {
                reached.insert(to.clone());
            }
        }
        if reached.len() == before {
            break;
        }
    }
    let orphans: Vec<&str> = m
        .states
        .iter()
        .map(|s| s.name.as_str())
        .filter(|n| !reached.contains(*n))
        .collect();
    if !orphans.is_empty() {
        return Err(err(
            "",
            0,
            format!(
                "no path from `{START}` reaches: {}. A state nothing enters is either misspelt \
                 or not yet wired up",
                orphans.join(", ")
            ),
        ));
    }
    Ok(m)
}
