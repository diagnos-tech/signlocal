use std::collections::BTreeSet;

use super::*;

fn entries(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

fn problems(names: &[&str], summary: &str) -> Vec<String> {
    check_folder(&entries(names), Some(summary))
}

#[test]
fn a_complete_summary_has_no_problems() {
    let summary = "# a\n\n- `sub/` — folder\n- `x.rs` — file\n- `*.der` — certs\n";
    assert!(problems(&["sub/", "x.rs", "a.der", "b.der", "SUMMARY.md"], summary).is_empty());
}

#[test]
fn a_missing_summary_is_reported() {
    assert_eq!(check_folder(&entries(&["x"]), None), ["missing SUMMARY.md"]);
}

#[test]
fn uncovered_and_stale_rows_are_reported() {
    let found = problems(
        &["x.rs", "y.rs"],
        "# a\n\n- `x.rs` — file\n- `gone.rs` — old\n",
    );
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
        found
            .iter()
            .any(|p| p.contains("`y.rs` is not in SUMMARY.md"))
    );
    assert!(
        found
            .iter()
            .any(|p| p.contains("`gone.rs` which does not exist"))
    );
}

#[test]
fn a_literal_row_wins_over_a_glob_but_two_globs_conflict() {
    let ok = "# a\n\n- `a.der` — one\n- `*.der` — rest\n";
    assert!(problems(&["a.der", "b.der"], ok).is_empty());
    let clash = "# a\n\n- `a*` — one\n- `*.der` — two\n";
    assert!(problems(&["a.der"], clash)[0].contains("several rows"));
}

#[test]
fn a_folder_row_needs_the_trailing_slash() {
    let found = problems(&["sub/"], "# a\n\n- `sub` — folder\n");
    assert_eq!(found.len(), 2, "{found:?}");
}

#[test]
fn duplicate_literal_rows_are_reported() {
    let found = problems(&["x.rs"], "# a\n\n- `x.rs` — one\n- `x.rs` — two\n");
    assert!(found[0].contains("2 times"), "{found:?}");
}

#[test]
fn component_roots_include_crates_and_clients_children() {
    assert!(is_component_root("crates/websign-core"));
    assert!(is_component_root("sdk"));
    assert!(!is_component_root("crates/websign-core/src"));
    assert!(!is_component_root("docs"));
}
