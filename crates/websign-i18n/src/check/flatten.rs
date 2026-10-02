//! A locale file as a flat map from dotted key path to message.

use std::collections::{BTreeMap, BTreeSet};

use super::Problem;

pub(super) const CATEGORIES: [&str; 6] = ["zero", "one", "two", "few", "many", "other"];

/// One message: a string, or a plural table of category to string.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Leaf {
    Text(String),
    Plural(BTreeMap<String, String>),
}

pub(super) type Flat = BTreeMap<String, Leaf>;

/// Flattens `table`. A table is a plural when `known_plurals` (the reference's
/// plurals) says so at that path, or when it looks like one, so a candidate
/// plural that lost its `other` is reported as such instead of as noise.
/// Values that are neither strings nor tables are reported as `Syntax`.
pub(super) fn flatten(
    table: &toml::Table,
    prefix: &str,
    known_plurals: &BTreeSet<String>,
    out: &mut Flat,
    syntax: &mut Vec<Problem>,
) {
    for (name, value) in table {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        match value {
            toml::Value::String(text) => {
                out.insert(path, Leaf::Text(text.clone()));
            }
            toml::Value::Table(inner) if known_plurals.contains(&path) || looks_plural(inner) => {
                let mut forms = BTreeMap::new();
                for (category, form) in inner {
                    match form.as_str() {
                        Some(text) => {
                            forms.insert(category.clone(), text.to_owned());
                        }
                        None => syntax.push(not_a_message(&format!("{path}.{category}"))),
                    }
                }
                out.insert(path, Leaf::Plural(forms));
            }
            toml::Value::Table(inner) => flatten(inner, &path, known_plurals, out, syntax),
            _ => syntax.push(not_a_message(&path)),
        }
    }
}

/// Paths of the plural messages in `flat`.
pub(super) fn plural_paths(flat: &Flat) -> BTreeSet<String> {
    flat.iter()
        .filter(|(_, leaf)| matches!(leaf, Leaf::Plural(_)))
        .map(|(path, _)| path.clone())
        .collect()
}

fn looks_plural(table: &toml::Table) -> bool {
    table.contains_key("other")
        && table
            .iter()
            .all(|(name, value)| CATEGORIES.contains(&name.as_str()) && value.is_str())
}

fn not_a_message(path: &str) -> Problem {
    Problem::Syntax {
        detail: format!("{path} must be a string or a table"),
    }
}
