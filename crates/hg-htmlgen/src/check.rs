//! Checking a parsed `.hgtok` file.
//!
//! What is checked is what a generated tokenizer cannot recover from: a
//! state that does not exist, a state nothing reaches, a rule that can
//! never fire because an earlier one already matches it, and a state
//! with no "anything else" row — which the prose always has, and whose
//! absence would leave the generated loop with a case it cannot answer.

use std::collections::{BTreeMap, BTreeSet};

use crate::syntax::{Action, Class, Error, File, Lookahead, Match, State};

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
///
/// Alternation is checked one alternative at a time, and *any* covered
/// alternative is a mistake — not only a wholly covered group. A
/// description that writes `'/' | ' '` under an earlier `whitespace`
/// rule has a dead alternative whether or not the rest of the group can
/// fire, and that is exactly the slip grouping invites. A lookahead
/// covers only an identical lookahead, because what a string of
/// characters shares with a code-point class is not decidable from the
/// description alone.
fn covers(earlier: &Match, later: &Match) -> bool {
    match (earlier, later) {
        (Match::Anything, _) => true,
        (a, b) if a == b => true,
        (_, Match::OneOf(ls)) => ls.iter().any(|l| covers(earlier, l)),
        (Match::OneOf(es), _) => es.iter().any(|e| covers(e, later)),
        (Match::Class(c), Match::Char(ch)) => match c {
            Class::Whitespace => matches!(ch, '\t' | '\n' | '\u{000C}' | ' '),
            Class::Null => *ch == '\0',
            Class::AsciiAlpha => ch.is_ascii_alphabetic(),
            Class::AsciiUpper => ch.is_ascii_uppercase(),
            Class::AsciiLower => ch.is_ascii_lowercase(),
            Class::AsciiDigit => ch.is_ascii_digit(),
            Class::AsciiHexDigit => ch.is_ascii_hexdigit(),
            Class::AsciiAlphanumeric => ch.is_ascii_alphanumeric(),
        },
        (Match::Class(Class::AsciiAlpha), Match::Class(c)) => {
            matches!(c, Class::AsciiUpper | Class::AsciiLower)
        }
        (Match::Class(Class::AsciiHexDigit), Match::Class(Class::AsciiDigit)) => true,
        (Match::Class(Class::AsciiAlphanumeric), Match::Class(c)) => !matches!(c, Class::Null),
        (
            Match::Lookahead(Lookahead {
                text: a,
                ci: true,
                from_current: fa,
            }),
            Match::Lookahead(Lookahead {
                text: b,
                from_current: fb,
                ..
            }),
        ) => fa == fb && a.eq_ignore_ascii_case(b),
        // A `from` lookahead is tested against the character the state
        // has already consumed, so an earlier rule that matches every
        // character it could start with leaves it dead.
        (Match::Class(_), Match::Lookahead(l)) if l.from_current => later
            .leading_chars()
            .is_some_and(|cs| cs.iter().all(|c| covers(earlier, &Match::Char(*c)))),
        _ => false,
    }
}

/// The first action in the list that needs a current input character,
/// named as the description writes it.
fn needs_current(actions: &[Action]) -> Option<&'static str> {
    for a in actions {
        let name = match a {
            Action::EmitCurrent => "emit_current",
            Action::Append(_) => "append",
            Action::AppendLower(_) => "append_lower",
            Action::Reconsume(_) => "reconsume",
            Action::ReconsumeReturn => "reconsume_return",
            Action::CharRefHexDigit => "char_ref_hex_digit",
            Action::CharRefDecDigit => "char_ref_dec_digit",
            Action::If {
                then, otherwise, ..
            } => return needs_current(then).or_else(|| needs_current(otherwise)),
            _ => continue,
        };
        return Some(name);
    }
    None
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
            // Its implementation is hand-written, so what it reaches is
            // a declaration; the targets are still checked and still
            // count for reachability.
            errors_of(&s.reaches, &mut m.errors);
            let mut to = Vec::new();
            targets(&s.reaches, &mut to);
            for t in to {
                if !where_declared.contains_key(&t) {
                    return Err(err(
                        file,
                        s.line,
                        format!(
                            "extern state {} declares it reaches {t}, which does not exist",
                            s.name
                        ),
                    ));
                }
                m.transitions.insert((s.name.clone(), t));
            }
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
        // The enter actions run before anything is consumed, so their
        // targets and errors belong to the state just as a rule's do.
        errors_of(&s.enter, &mut m.errors);
        let mut entered = Vec::new();
        targets(&s.enter, &mut entered);
        for t in entered {
            if !where_declared.contains_key(&t) {
                return Err(err(
                    file,
                    s.line,
                    format!(
                        "state {} enters by moving to {t}, which does not exist",
                        s.name
                    ),
                ));
            }
            m.transitions.insert((s.name.clone(), t));
        }

        let mut seen: Vec<&Match> = Vec::new();
        let mut has_anything = false;
        let mut past_lookahead = false;
        let mut unconsumed = false;
        for r in &s.rules {
            // `next` asks about characters nothing has consumed, so a
            // state that asks it consumes nothing at all — the
            // standard's own "anything else" for the markup declaration
            // open state ends "(do not consume anything)". Such a state
            // may hold nothing but those lookaheads and `_`.
            if r.matches.looks_ahead_unconsumed() {
                if past_lookahead {
                    return Err(err(
                        file,
                        r.line,
                        format!(
                            "in state {}, a `next` lookahead comes after a rule that consumes a \
                             character. A state that asks about the next few characters consumes \
                             nothing, so it may hold only those lookaheads and `_`",
                            s.name
                        ),
                    ));
                }
                unconsumed = true;
            } else if !matches!(r.matches, Match::Anything) {
                past_lookahead = true;
                if unconsumed {
                    return Err(err(
                        file,
                        r.line,
                        format!(
                            "in state {}, a rule that consumes a character follows a `next` \
                             lookahead. A state that asks about the next few characters consumes \
                             nothing, so it may hold only those lookaheads and `_`",
                            s.name
                        ),
                    ));
                }
            }
            if let Match::OneOf(ms) = &r.matches
                && ms.iter().any(|x| matches!(x, Match::Anything))
            {
                return Err(err(
                    file,
                    r.line,
                    format!(
                        "in state {}, `_` is an alternative of a larger match; \"anything else\" \
                         is a row of its own",
                        s.name
                    ),
                ));
            }
            if let Some(prev) = seen.iter().find(|p| covers(p, &r.matches)) {
                return Err(err(
                    file,
                    r.line,
                    format!(
                        "this rule, or one of its alternatives, can never fire in state {}: an \
                         earlier rule for {:?} already matches it",
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
        if unconsumed {
            // Nothing was consumed, so there is no current input
            // character to emit, append or reconsume.
            for r in &s.rules {
                if matches!(r.matches, Match::Anything)
                    && let Some(a) = needs_current(&r.actions)
                {
                    return Err(err(
                        file,
                        r.line,
                        format!(
                            "in state {}, `{a}` needs the current input character, and this state \
                             consumes none: it asks about the next few characters instead",
                            s.name
                        ),
                    ));
                }
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
    // An `entry state` is one the tree builder switches into, so it is
    // reached from outside the tokenizer's own transitions.
    for s in &m.states {
        if s.entry {
            reached.insert(s.name.clone());
        }
    }
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
