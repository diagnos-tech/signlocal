//! Consistency rules between a locale file and the reference, used by
//! `cargo xtask check i18n` and by this crate's tests.

mod flatten;

use std::collections::BTreeSet;

use flatten::{CATEGORIES, Flat, Leaf, flatten, plural_paths};

use crate::message::placeholders;

/// Sections that end up in `chrome.i18n`, which has no plurals.
const EXTENSION_SECTIONS: [&str; 3] = ["popup.", "store.", "extension."];
/// The Chrome Web Store limit for an extension description.
const MAX_DESCRIPTION_CHARS: usize = 132;

/// One problem in a locale file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// A reference key the locale lacks.
    Missing { key: String },
    /// A key the reference does not have.
    Extra { key: String },
    /// Placeholders differ from the reference's (as sets); both lists are
    /// sorted and deduplicated.
    Placeholders {
        key: String,
        expected: Vec<String>,
        found: Vec<String>,
    },
    /// A plural lacks `other`, or has a category outside CLDR.
    Plural { key: String, detail: String },
    /// A `[popup]`/`[store]`/`[extension]` message is a plural (`chrome.i18n`
    /// has none), or the store description exceeds 132 characters.
    Extension { key: String, detail: String },
    /// Not valid TOML, or a value that is neither a string nor a table.
    Syntax { detail: String },
}

/// Compares `candidate` (a locale file's text) with `reference` (`en.toml`).
/// Every problem is reported, grouped by kind in the order of `SPEC.md` §7.
pub fn check_locale(reference: &str, candidate: &str) -> Vec<Problem> {
    let no_plurals = BTreeSet::new();
    let mut syntax = Vec::new();

    let mut reference_flat = Flat::new();
    match reference.parse::<toml::Table>() {
        Ok(table) => flatten(&table, "", &no_plurals, &mut reference_flat, &mut syntax),
        Err(error) => syntax.push(invalid("reference", &error)),
    }
    let mut candidate_flat = Flat::new();
    match candidate.parse::<toml::Table>() {
        Ok(table) => {
            let plurals = plural_paths(&reference_flat);
            flatten(&table, "", &plurals, &mut candidate_flat, &mut syntax);
        }
        Err(error) => syntax.push(invalid("candidate", &error)),
    }
    if !syntax.is_empty() {
        return syntax;
    }

    let mut missing = Vec::new();
    let mut extra = Vec::new();
    let mut placeholder = Vec::new();
    let mut plural = Vec::new();
    let mut extension = Vec::new();

    for (key, expected) in &reference_flat {
        match candidate_flat.get(key) {
            None => missing.push(Problem::Missing { key: key.clone() }),
            Some(found) => match (expected, found) {
                (Leaf::Text(a), Leaf::Text(b)) => {
                    compare_placeholders(key, &placeholders(a), &placeholders(b), &mut placeholder);
                }
                (Leaf::Plural(a), Leaf::Plural(b)) => {
                    let expected = a.get("other").map(|text| placeholders(text));
                    for form in b.values() {
                        let found = placeholders(form);
                        let expected = expected.as_deref().unwrap_or_default();
                        compare_placeholders(key, expected, &found, &mut placeholder);
                    }
                }
                _ => {
                    missing.push(Problem::Missing { key: key.clone() });
                    extra.push(Problem::Extra { key: key.clone() });
                }
            },
        }
    }
    for (key, found) in &candidate_flat {
        if !reference_flat.contains_key(key) {
            extra.push(Problem::Extra { key: key.clone() });
        }
        match found {
            Leaf::Plural(forms) => {
                plural.extend(plural_problems(key, forms.keys()));
                if EXTENSION_SECTIONS.iter().any(|s| key.starts_with(s)) {
                    extension.push(Problem::Extension {
                        key: key.clone(),
                        detail: "plurals are not available in chrome.i18n".into(),
                    });
                }
            }
            Leaf::Text(text) => {
                if key == "extension.description" && text.chars().count() > MAX_DESCRIPTION_CHARS {
                    extension.push(Problem::Extension {
                        key: key.clone(),
                        detail: format!("longer than {MAX_DESCRIPTION_CHARS} characters"),
                    });
                }
            }
        }
    }

    [missing, extra, placeholder, plural, extension].concat()
}

fn compare_placeholders(key: &str, expected: &[String], found: &[String], out: &mut Vec<Problem>) {
    if expected != found {
        out.push(Problem::Placeholders {
            key: key.to_owned(),
            expected: expected.to_vec(),
            found: found.to_vec(),
        });
    }
}

fn plural_problems<'a>(key: &str, categories: impl Iterator<Item = &'a String>) -> Vec<Problem> {
    let categories: Vec<&String> = categories.collect();
    let mut problems = Vec::new();
    if !categories.iter().any(|c| c.as_str() == "other") {
        problems.push(Problem::Plural {
            key: key.to_owned(),
            detail: "missing required category `other`".into(),
        });
    }
    for category in categories {
        if !CATEGORIES.contains(&category.as_str()) {
            problems.push(Problem::Plural {
                key: key.to_owned(),
                detail: format!("`{category}` is not a CLDR category"),
            });
        }
    }
    problems
}

fn invalid(which: &str, error: &toml::de::Error) -> Problem {
    Problem::Syntax {
        detail: format!("{which} is not valid TOML: {error}"),
    }
}

#[cfg(test)]
mod tests;
