//! Generated key constants (SPEC §1).

use websign_i18n::{ALL_KEYS, Key, PluralKey, SOURCES, k};

#[test]
fn constants_exist_with_dotted_paths() {
    assert_eq!(k::CONFIRM_WINDOW_TITLE.path(), "confirm.window_title");
    assert_eq!(k::COMMON_CANCEL.path(), "common.cancel");
    assert_eq!(k::META_LANGUAGE_NAME.path(), "meta.language_name");
    assert_eq!(k::TIME_TODAY_AT.path(), "time.today_at");
}

#[test]
fn plural_tables_are_plural_keys() {
    let keys: [PluralKey; 3] = [
        k::CERT_EXPIRES_IN,
        k::DEVICES_CERTS_FOUND,
        k::DEVICES_DRIVER_LOADED,
    ];
    assert_eq!(keys[0].path(), "cert.expires_in");
    assert_eq!(keys[1].path(), "devices.certs_found");
    assert_eq!(keys[2].path(), "devices.driver_loaded");
}

#[test]
fn string_leaves_are_plain_keys() {
    let _: Key = k::CONFIRM_WINDOW_TITLE_QUEUE;
    let _: Key = k::FOOTER_EXPIRES_IN;
}

#[test]
fn all_keys_lists_constants_and_is_unique() {
    assert!(ALL_KEYS.contains(&"confirm.window_title"));
    assert!(ALL_KEYS.contains(&"cert.expires_in"));
    assert!(
        !ALL_KEYS.contains(&"cert.expires_in.one"),
        "plural is one key"
    );
    let mut sorted = ALL_KEYS.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), ALL_KEYS.len());
}

#[test]
fn all_keys_are_lowercase_dotted_paths() {
    for key in ALL_KEYS {
        assert!(!key.is_empty());
        assert!(
            key.chars().all(|c| c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || matches!(c, '.' | '_' | '-')),
            "{key}"
        );
        assert!(
            !key.starts_with('.') && !key.ends_with('.') && !key.contains(".."),
            "{key}"
        );
    }
}

#[test]
fn sources_embed_the_seven_locales() {
    let tags: Vec<_> = SOURCES.iter().map(|(t, _)| *t).collect();
    for tag in ["en", "pt-BR", "pt-PT", "es", "fr", "it", "de"] {
        assert!(tags.contains(&tag), "{tag}");
    }
    for (tag, src) in SOURCES {
        assert!(!src.trim().is_empty(), "{tag}");
        assert!(
            src.parse::<toml::Table>().is_ok(),
            "{tag} is not valid TOML"
        );
    }
}
