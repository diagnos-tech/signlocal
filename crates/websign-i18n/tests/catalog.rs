//! `Catalog` lookups against the shipped, embedded locales (SPEC §3).
//!
//! The non-English files are English copies until translated, so these tests
//! avoid asserting translated wording outside English.

use websign_i18n::{Catalog, Locale, k};

#[test]
fn english_message_with_argument() {
    let c = Catalog::new(Locale::En);
    let s = c.tr(k::CONFIRM_WINDOW_TITLE).arg("site", "a.b").to_string();
    assert_eq!(s, "Sign for a.b — WebeSign");
}

#[test]
fn english_plurals() {
    let c = Catalog::new(Locale::En);
    assert_eq!(
        c.plural(k::CERT_EXPIRES_IN, 1).to_string(),
        "Expires in 1 day"
    );
    assert_eq!(
        c.plural(k::CERT_EXPIRES_IN, 0).to_string(),
        "Expires in 0 days"
    );
    assert_eq!(
        c.plural(k::CERT_EXPIRES_IN, 23).to_string(),
        "Expires in 23 days"
    );
    assert_eq!(
        c.plural(k::CERT_EXPIRES_IN, 1_000_000).to_string(),
        "Expires in 1000000 days"
    );
    assert_eq!(
        c.plural(k::DEVICES_CERTS_FOUND, 1).to_string(),
        "1 certificate"
    );
    assert_eq!(
        c.plural(k::DEVICES_CERTS_FOUND, 2).to_string(),
        "2 certificates"
    );
}

#[test]
fn plural_count_is_a_plain_decimal_integer() {
    let c = Catalog::new(Locale::En);
    assert_eq!(
        c.plural(k::CERT_EXPIRES_IN, -5).to_string(),
        "Expires in -5 days"
    );
    assert_eq!(
        c.plural(k::CERT_EXPIRES_IN, -1).to_string(),
        "Expires in -1 day"
    );
    assert!(
        c.plural(k::CERT_EXPIRES_IN, 1234567)
            .to_string()
            .contains("1234567")
    );
}

#[test]
fn catalog_reports_its_locale() {
    for l in Locale::ALL {
        assert_eq!(Catalog::new(l).locale(), l);
    }
}

#[test]
fn every_locale_renders_a_common_key() {
    for l in Locale::ALL {
        let s = Catalog::new(l).tr(k::COMMON_CANCEL).to_string();
        assert!(!s.is_empty(), "{l:?}");
        assert_ne!(
            s,
            k::COMMON_CANCEL.path(),
            "{l:?} fell back to the key path"
        );
    }
}

#[test]
fn every_locale_fills_count_and_leaves_no_placeholder() {
    for l in Locale::ALL {
        let c = Catalog::new(l);
        for n in [0, 1, 2, 5, 1_000_000] {
            let s = c.plural(k::CERT_EXPIRES_IN, n).to_string();
            assert!(s.contains(&n.to_string()), "{l:?} {n}: {s}");
            assert!(!s.contains("{count}"), "{l:?} {n}: {s}");
        }
    }
}

#[test]
fn every_locale_renders_arguments() {
    for l in Locale::ALL {
        let c = Catalog::new(l);
        let s = c
            .tr(k::CONFIRM_WINDOW_TITLE)
            .arg("site", "host.example")
            .to_string();
        assert!(s.contains("host.example"), "{l:?}: {s}");
        assert!(!s.contains("{site}"), "{l:?}: {s}");
    }
}

#[test]
fn queue_title_fills_all_counters() {
    let c = Catalog::new(Locale::En);
    let s = c
        .tr(k::CONFIRM_WINDOW_TITLE_QUEUE)
        .arg("title", "T")
        .arg("current", 1)
        .arg("total", 3)
        .to_string();
    assert_eq!(s, "T (1 of 3)");
}

#[test]
fn unfilled_argument_stays_visible() {
    let c = Catalog::new(Locale::En);
    let s = c.tr(k::CONFIRM_WINDOW_TITLE).to_string();
    assert!(s.contains("{site}"), "{s}");
}

#[test]
fn english_language_name() {
    let c = Catalog::new(Locale::En);
    assert_eq!(c.tr(k::META_LANGUAGE_NAME).to_string(), "English");
}
