//! `check_locale` plural and browser-extension rules (SPEC §7, items 4-6).

use websign_i18n::check::{Problem, check_locale};

const REF: &str = include_str!("fixtures/reference.toml");
const DESCRIPTION: &str = "Sign documents with your certificate.";

fn check(candidate: &str) -> Vec<Problem> {
    check_locale(REF, candidate)
}

fn has(problems: &[Problem], f: impl Fn(&Problem) -> bool) -> bool {
    problems.iter().any(f)
}

#[test]
fn plural_category_placeholders_must_equal_reference_other() {
    let p = check(&REF.replace("Expires in {count} day\"", "Expires in one day\""));
    assert!(
        has(
            &p,
            |x| matches!(x, Problem::Placeholders { key, .. } if key.starts_with("cert.expires_in"))
        ),
        "{p:?}"
    );
}

#[test]
fn plural_may_omit_one_when_other_is_present() {
    let p = check(&REF.replace("one = \"Expires in {count} day\"\n", ""));
    assert_eq!(p, vec![]);
}

#[test]
fn plural_without_other_is_a_plural_problem() {
    let c = REF.replace(
        "other = \"Expires in {count} days\"",
        "many = \"Expires in {count} days\"",
    );
    let p = check(&c);
    assert!(
        has(
            &p,
            |x| matches!(x, Problem::Plural { key, .. } if key.starts_with("cert.expires_in"))
        ),
        "{p:?}"
    );
}

#[test]
fn plural_with_non_cldr_category_is_reported() {
    let p = check(&REF.replace("one = \"Expires", "several = \"Expires"));
    assert!(
        matches!(p.as_slice(), [Problem::Plural { key, .. }] if key == "cert.expires_in"),
        "{p:?}"
    );
}

#[test]
fn every_cldr_category_is_accepted_in_a_plural() {
    let c = REF.replace(
        "one = \"Expires in {count} day\"",
        "zero = \"{count}\"\none = \"{count}\"\ntwo = \"{count}\"\nfew = \"{count}\"\nmany = \"{count}\"",
    );
    assert_eq!(check(&c), vec![]);
}

#[test]
fn plural_under_popup_store_or_extension_is_an_extension_problem() {
    for section in ["popup", "store", "extension"] {
        let reference =
            format!("{REF}\n[{section}.n]\none = \"{{count}}\"\nother = \"{{count}}\"\n");
        let p = check_locale(&reference, &reference);
        assert!(
            has(&p, |x| matches!(x, Problem::Extension { .. })),
            "{section}: {p:?}"
        );
    }
}

#[test]
fn extension_description_limit_is_132_characters() {
    let with = |d: &str| REF.replace(DESCRIPTION, d);
    assert_eq!(check(&with(&"x".repeat(132))), vec![]);
    let p = check(&with(&"x".repeat(133)));
    assert!(
        has(
            &p,
            |x| matches!(x, Problem::Extension { key, .. } if key == "extension.description")
        ),
        "{p:?}"
    );
}

#[test]
fn description_limit_counts_characters_not_bytes() {
    let ok = REF.replace(DESCRIPTION, &"é".repeat(132));
    assert_eq!(check(&ok), vec![]);
    let long = REF.replace(DESCRIPTION, &"é".repeat(133));
    assert!(has(&check(&long), |x| matches!(
        x,
        Problem::Extension { .. }
    )));
}
