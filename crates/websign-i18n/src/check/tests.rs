use super::*;
use crate::SOURCES;

const REFERENCE: &str = r#"
a = "Hello {name}"
[n]
one = "{count} day"
other = "{count} days"
[popup]
title = "T"
"#;

fn kinds(candidate: &str) -> Vec<Problem> {
    check_locale(REFERENCE, candidate)
}

#[test]
fn identical_is_clean() {
    assert!(kinds(REFERENCE).is_empty());
}

#[test]
fn shipped_locales_are_consistent() {
    let (_, reference) = SOURCES[0];
    for (tag, source) in SOURCES {
        assert_eq!(check_locale(reference, source), vec![], "{tag}");
    }
}

#[test]
fn reports_missing_and_extra() {
    let problems = kinds("a = \"Hi {name}\"\nz = \"x\"\n[n]\nother = \"{count}\"\n");
    assert_eq!(
        problems,
        vec![
            Problem::Missing {
                key: "popup.title".into()
            },
            Problem::Extra { key: "z".into() },
        ]
    );
}

#[test]
fn reports_placeholder_mismatch() {
    let problems =
        kinds("a = \"Hi\"\n[n]\none = \"1\"\nother = \"{count}\"\n[popup]\ntitle = \"T\"\n");
    assert_eq!(problems.len(), 2);
    assert!(
        problems
            .iter()
            .all(|p| matches!(p, Problem::Placeholders { .. }))
    );
}

#[test]
fn reports_plural_without_other() {
    let problems = kinds("a = \"{name}\"\n[n]\none = \"{count}\"\n[popup]\ntitle = \"T\"\n");
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::Plural { key, .. } if key == "n"))
    );
}

#[test]
fn reports_syntax() {
    assert!(matches!(
        kinds("a = [").as_slice(),
        [Problem::Syntax { .. }]
    ));
    assert!(matches!(
        kinds("a = 3").as_slice(),
        [Problem::Syntax { .. }]
    ));
}

#[test]
fn reports_kind_mismatch_once_each() {
    let problems = kinds("a = \"{name}\"\nn = \"x\"\n[popup]\ntitle = \"T\"\n");
    assert_eq!(
        problems,
        vec![
            Problem::Missing { key: "n".into() },
            Problem::Extra { key: "n".into() },
        ]
    );
}

#[test]
fn reports_extension_rules() {
    let reference = "[extension]\ndescription = \"d\"\n";
    let long = "x".repeat(133);
    let problems = check_locale(
        reference,
        &format!("[extension]\ndescription = \"{long}\"\n"),
    );
    assert!(matches!(problems.as_slice(), [Problem::Extension { .. }]));

    let reference = "[popup.n]\none = \"a\"\nother = \"b\"\n";
    let problems = check_locale(reference, reference);
    assert!(matches!(problems.as_slice(), [Problem::Extension { .. }]));
}
