//! `hg-style-reference`: see the library docs.
//!
//! ```text
//! hg-style-reference --sheet FILE... --tree FILE [--property NAME]
//! ```
//!
//! Reads one tree literal and any number of stylesheets, then prints
//! the winning declarations for every element. It exists so a
//! differential can be run by hand, and so what the reference engine
//! could not read is visible rather than silent.
//!
//! Exit status: 0 when every sheet was read whole, 1 when a rule was
//! refused or an at-rule skipped, 2 when a file could not be read.

use std::process::ExitCode;

use hg_style_reference::cascade::{Cascade, Origin};
use hg_tree_traits::testing::TestTree;
use hg_tree_traits::{Tree, is_element};

const USAGE: &str = "usage: hg-style-reference --tree FILE [--sheet FILE]... [--origin ua|user|author] \
     [--property NAME]";

fn main() -> ExitCode {
    let mut tree_path: Option<String> = None;
    let mut sheets: Vec<(String, Origin)> = Vec::new();
    let mut origin = Origin::Author;
    let mut property: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--tree" => match args.next() {
                Some(v) => tree_path = Some(v),
                None => return usage(),
            },
            "--sheet" => match args.next() {
                Some(v) => sheets.push((v, origin)),
                None => return usage(),
            },
            "--origin" => match args.next().as_deref() {
                Some("ua") => origin = Origin::UserAgent,
                Some("user") => origin = Origin::User,
                Some("author") => origin = Origin::Author,
                _ => return usage(),
            },
            "--property" => match args.next() {
                Some(v) => property = Some(v),
                None => return usage(),
            },
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => return usage(),
        }
    }
    let Some(tree_path) = tree_path else {
        return usage();
    };
    let source = match std::fs::read_to_string(&tree_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("hg-style-reference: {tree_path}: {e}");
            return ExitCode::from(2);
        }
    };
    let mut tree = match TestTree::parse(&source) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("hg-style-reference: {tree_path}:{}: {}", e.line, e.message);
            return ExitCode::from(2);
        }
    };
    tree.set_html_document(true);

    let mut cascade = Cascade::new();
    for (path, origin) in &sheets {
        match std::fs::read_to_string(path) {
            Ok(s) => {
                cascade.add(&s, *origin);
            }
            Err(e) => {
                eprintln!("hg-style-reference: {path}: {e}");
                return ExitCode::from(2);
            }
        }
    }

    let mut incomplete = false;
    for sheet in cascade.sheets() {
        for s in &sheet.skipped {
            println!("  skipped: {s}");
            incomplete = true;
        }
        for r in &sheet.refused {
            println!("  refused: {r}");
            incomplete = true;
        }
    }

    for node in tree.elements() {
        if !is_element(&tree, node) {
            continue;
        }
        let name = tree.local_name(node).unwrap_or("?");
        let id = tree.element_id(node).unwrap_or("");
        let label = if id.is_empty() {
            name.to_owned()
        } else {
            format!("{name}#{id}")
        };
        match &property {
            Some(p) => {
                if let Some(v) = cascade.value(&tree, node, p) {
                    println!("{label}: {p}: {v}");
                }
            }
            None => {
                for (prop, candidate) in cascade.winners(&tree, node, None) {
                    println!("{label}: {prop}: {}", candidate.declaration.value);
                }
            }
        }
    }

    if incomplete {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
