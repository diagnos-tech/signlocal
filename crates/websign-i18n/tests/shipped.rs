//! The repository's real locale files, checked the way CI does.

use std::path::PathBuf;
use websign_i18n::check::check_locale;
use websign_i18n::{ALL_KEYS, Locale};

fn read(tag: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../i18n")
        .join(format!("{tag}.toml"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn every_locale_file_matches_the_reference() {
    let reference = read("en");
    for locale in Locale::ALL {
        let problems = check_locale(&reference, &read(locale.tag()));
        assert!(problems.is_empty(), "{}: {problems:?}", locale.tag());
    }
}

#[test]
fn the_reference_checks_clean_against_itself() {
    let en = read("en");
    assert_eq!(check_locale(&en, &en), vec![]);
}

fn count_keys(t: &toml::Table) -> usize {
    let plural = t.contains_key("other")
        && t.keys()
            .all(|k| ["zero", "one", "two", "few", "many", "other"].contains(&k.as_str()));
    if plural {
        return 1;
    }
    t.values()
        .map(|v| match v {
            toml::Value::Table(sub) => count_keys(sub),
            _ => 1,
        })
        .sum()
}

#[test]
fn all_keys_cover_the_reference_leaves() {
    let en: toml::Table = read("en").parse().expect("en.toml parses");
    assert_eq!(ALL_KEYS.len(), count_keys(&en));
}
