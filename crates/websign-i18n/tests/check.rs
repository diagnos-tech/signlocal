//! `check_locale` diagnostics (SPEC §7).

use websign_i18n::check::{Problem, check_locale};

const REF: &str = include_str!("fixtures/reference.toml");
const QUEUE_LINE: &str = "queue = \"{current} of {total}\"\n";
const PLURAL_TABLE: &str =
    "[cert.expires_in]\none = \"Expires in {count} day\"\nother = \"Expires in {count} days\"\n";

fn check(candidate: &str) -> Vec<Problem> {
    check_locale(REF, candidate)
}

fn has(problems: &[Problem], f: impl Fn(&Problem) -> bool) -> bool {
    problems.iter().any(f)
}

#[test]
fn identical_file_has_no_problems() {
    assert_eq!(check(REF), vec![]);
}

#[test]
fn translated_text_and_comments_do_not_matter() {
    let c = r#"
[meta]
language_name = "Deutsch"
[confirm]
window_title = "Unterschreiben für {site}"
queue = "{total} — {current}"
[cert.expires_in]
one = "Läuft in {count} Tag ab"
other = "Läuft in {count} Tagen ab"
[popup]
ready_title = "Bereit"
[extension]
name = "WebeSign"
description = "Dokumente signieren."
"#;
    assert_eq!(check(c), vec![]);
}

#[test]
fn invalid_toml_is_a_syntax_problem() {
    let p = check("this is = = not toml [");
    assert!(has(&p, |x| matches!(x, Problem::Syntax { .. })), "{p:?}");
}

#[test]
fn non_string_non_table_value_is_a_syntax_problem() {
    let p = check(&REF.replace("ready_title = \"Ready\"", "ready_title = 5"));
    assert!(has(&p, |x| matches!(x, Problem::Syntax { .. })), "{p:?}");
}

#[test]
fn missing_key_is_reported_by_full_path() {
    let p = check(&REF.replace(QUEUE_LINE, ""));
    assert!(
        p.contains(&Problem::Missing {
            key: "confirm.queue".into()
        }),
        "{p:?}"
    );
}

#[test]
fn missing_plural_table_is_reported() {
    let p = check(&REF.replace(PLURAL_TABLE, ""));
    assert!(
        has(
            &p,
            |x| matches!(x, Problem::Missing { key } if key.starts_with("cert.expires_in"))
        ),
        "{p:?}"
    );
}

#[test]
fn extra_key_is_reported() {
    let p = check(&format!("{REF}\n[confirm.more]\nnew = \"x\"\n"));
    assert!(
        p.contains(&Problem::Extra {
            key: "confirm.more.new".into()
        }),
        "{p:?}"
    );
    let p = check(&REF.replace("[popup]\n", "[popup]\nsurplus = \"x\"\n"));
    assert!(
        p.contains(&Problem::Extra {
            key: "popup.surplus".into()
        }),
        "{p:?}"
    );
}

#[test]
fn all_problems_are_reported_not_just_the_first() {
    let c = REF
        .replace(QUEUE_LINE, "")
        .replace("[popup]\n", "[popup]\nsurplus = \"x\"\n")
        .replace("Sign for {site}", "Sign for {host}");
    let p = check(&c);
    assert!(has(&p, |x| matches!(x, Problem::Missing { .. })), "{p:?}");
    assert!(has(&p, |x| matches!(x, Problem::Extra { .. })), "{p:?}");
    assert!(
        has(&p, |x| matches!(x, Problem::Placeholders { .. })),
        "{p:?}"
    );
}

#[test]
fn missing_is_detected_before_extra() {
    let c = REF
        .replace(QUEUE_LINE, "")
        .replace("[popup]\n", "[popup]\nsurplus = \"x\"\n");
    let p = check(&c);
    let missing = p.iter().position(|x| matches!(x, Problem::Missing { .. }));
    let extra = p.iter().position(|x| matches!(x, Problem::Extra { .. }));
    assert!(missing.is_some() && extra.is_some(), "{p:?}");
    assert!(missing < extra, "{p:?}");
}

#[test]
fn table_versus_string_is_one_missing_and_one_extra() {
    let c = REF.replace(PLURAL_TABLE, "[cert]\nexpires_in = \"Expires {count}\"\n");
    let p = check(&c);
    let missing = p
        .iter()
        .filter(|x| matches!(x, Problem::Missing { .. }))
        .count();
    let extra = p
        .iter()
        .filter(|x| matches!(x, Problem::Extra { .. }))
        .count();
    assert_eq!((missing, extra), (1, 1), "{p:?}");
}

#[test]
fn renamed_placeholder_is_a_placeholder_problem() {
    let p = check(&REF.replace("Sign for {site}", "Sign for {host}"));
    let hit = p.iter().find_map(|x| match x {
        Problem::Placeholders {
            key,
            expected,
            found,
        } if key == "confirm.window_title" => Some((expected.clone(), found.clone())),
        _ => None,
    });
    let (expected, found) = hit.unwrap_or_else(|| panic!("{p:?}"));
    assert_eq!(expected, ["site"]);
    assert_eq!(found, ["host"]);
}

#[test]
fn dropped_and_added_placeholders_are_detected() {
    for text in ["Sign", "Sign for {site} {extra}"] {
        let p = check(&REF.replace("Sign for {site}", text));
        assert!(
            has(
                &p,
                |x| matches!(x, Problem::Placeholders { key, .. } if key == "confirm.window_title")
            ),
            "{text}: {p:?}"
        );
    }
}

#[test]
fn placeholders_compare_as_sets_so_order_and_repeats_are_fine() {
    let p = check(&REF.replace("{current} of {total}", "{total} {current} {total}"));
    assert_eq!(p, vec![]);
}

#[test]
fn placeholder_lists_are_sorted_and_deduplicated() {
    let p = check(&REF.replace("{current} of {total}", "{total} {x} {total}"));
    assert_eq!(
        p,
        vec![Problem::Placeholders {
            key: "confirm.queue".into(),
            expected: vec!["current".into(), "total".into()],
            found: vec!["total".into(), "x".into()],
        }]
    );
}

#[test]
fn braces_that_are_not_placeholders_are_ignored() {
    let p = check(&REF.replace("Sign for {site}", "Sign for {site} { x } {Name}"));
    assert_eq!(p, vec![]);
}

#[test]
fn empty_candidate_reports_every_reference_key_missing() {
    let p = check("");
    let missing: Vec<&str> = p
        .iter()
        .filter_map(|x| match x {
            Problem::Missing { key } => Some(key.as_str()),
            _ => None,
        })
        .collect();
    for key in [
        "meta.language_name",
        "confirm.window_title",
        "confirm.queue",
        "popup.ready_title",
        "extension.name",
        "extension.description",
    ] {
        assert!(missing.contains(&key), "{key} in {p:?}");
    }
    assert!(missing.iter().any(|k| k.starts_with("cert.expires_in")));
}

#[test]
fn checker_does_not_panic_on_hostile_input() {
    for c in [
        "\u{0}",
        "[[a]]\nb = 1",
        "a = { b = 1 }",
        "[confirm]\nwindow_title = [1,2]",
        "= 1",
        "\"\" = \"\"",
    ] {
        let _ = check(c);
        let _ = check_locale(c, REF);
    }
}

#[test]
fn deeply_nested_input_does_not_panic() {
    let dotted = format!("{} = \"x\"", vec!["a"; 5_000].join("."));
    let arrays = format!("a = {}1{}", "[".repeat(5_000), "]".repeat(5_000));
    for c in [dotted, arrays] {
        let _ = check(&c);
        let _ = check_locale(&c, &c);
    }
}
