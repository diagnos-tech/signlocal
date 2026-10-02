//! Every folder has a `SUMMARY.md` that covers each entry exactly once.

use std::path::Path;

use super::glob::{is_pattern, matches};
use super::listing::{is_unchecked, tree};
use super::summary_file::{Row, parse};

/// Component roots that need a `README.md`; the children of `crates/` and
/// `clients/` are added by [`is_component_root`].
const COMPONENT_ROOTS: [&str; 8] = [
    "app",
    "xtask",
    "sdk",
    "extension",
    "e2e",
    "safari",
    "packaging",
    "i18n",
];

pub fn check(root: &Path) -> Result<Vec<String>, String> {
    let tree = tree(root)?;
    let mut problems = Vec::new();
    for (folder, entries) in &tree {
        if is_unchecked(folder) {
            continue;
        }
        let shown = if folder.is_empty() { "." } else { folder };
        let summary_path = root.join(folder).join("SUMMARY.md");
        let summary = std::fs::read_to_string(&summary_path).ok();
        problems.extend(
            check_folder(entries, summary.as_deref())
                .into_iter()
                .map(|p| format!("{shown}: {p}")),
        );
        if is_component_root(folder) && !entries.contains("README.md") {
            problems.push(format!("{shown}: component root without README.md"));
        }
    }
    Ok(problems)
}

fn is_component_root(folder: &str) -> bool {
    COMPONENT_ROOTS.contains(&folder)
        || folder.split_once('/').is_some_and(|(parent, child)| {
            matches!(parent, "crates" | "clients") && !child.contains('/')
        })
}

/// Problems of one folder: `entries` are its files and `name/` subfolders.
fn check_folder(
    entries: &std::collections::BTreeSet<String>,
    summary: Option<&str>,
) -> Vec<String> {
    let Some(summary) = summary else {
        return vec!["missing SUMMARY.md".to_owned()];
    };
    let rows = match parse(summary) {
        Ok(rows) => rows,
        Err(problems) => {
            return problems
                .into_iter()
                .map(|p| format!("SUMMARY.md {p}"))
                .collect();
        }
    };
    let mut problems = Vec::new();
    let owned: Vec<&String> = entries
        .iter()
        .filter(|e| e.as_str() != "SUMMARY.md")
        .collect();
    for row in &rows {
        if !owned.iter().any(|entry| row_matches(row, entry)) {
            problems.push(format!(
                "SUMMARY.md lists `{}` which does not exist",
                row.name
            ));
        }
    }
    for entry in owned {
        problems.extend(coverage(entry, &rows));
    }
    problems
}

/// Whether `row` names `entry`: same kind (folder or file) and the same
/// name or a matching glob.
fn row_matches(row: &Row, entry: &str) -> bool {
    let (row_name, row_is_dir) = split_kind(&row.name);
    let (entry_name, entry_is_dir) = split_kind(entry);
    row_is_dir == entry_is_dir
        && if is_pattern(row_name) {
            matches(row_name, entry_name)
        } else {
            row_name == entry_name
        }
}

fn split_kind(name: &str) -> (&str, bool) {
    (name.trim_end_matches('/'), name.ends_with('/'))
}

/// A literal row wins over globs; otherwise exactly one glob must match.
fn coverage(entry: &str, rows: &[Row]) -> Option<String> {
    let literal = rows
        .iter()
        .filter(|r| !is_pattern(&r.name) && row_matches(r, entry))
        .count();
    match literal {
        1 => return None,
        n if n > 1 => return Some(format!("SUMMARY.md lists `{entry}` {n} times")),
        _ => {}
    }
    let globs: Vec<&str> = rows
        .iter()
        .filter(|r| is_pattern(&r.name) && row_matches(r, entry))
        .map(|r| r.name.as_str())
        .collect();
    match globs.len() {
        0 => Some(format!("`{entry}` is not in SUMMARY.md")),
        1 => None,
        _ => Some(format!(
            "`{entry}` is matched by several rows: {}",
            globs.join(", ")
        )),
    }
}

#[cfg(test)]
mod tests;
