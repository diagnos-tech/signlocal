//! Fallback through the chain, with injected sources (SPEC §3).

use websign_i18n::{Catalog, Locale, k};

const EN: &str = r#"
[common]
cancel = "Cancel"
[cert.expires_in]
one = "en one {count}"
other = "en other {count}"
"#;
const PT_PT: &str = "[common]\ncancel = \"Cancelar (pt-PT)\"\n";

fn catalog(sources: &[(Locale, &str)]) -> Catalog {
    Catalog::from_sources(sources[0].0, sources)
}

#[test]
fn missing_key_falls_to_the_next_locale() {
    let c = catalog(&[(Locale::PtBr, ""), (Locale::PtPt, PT_PT), (Locale::En, EN)]);
    assert_eq!(c.tr(k::COMMON_CANCEL).to_string(), "Cancelar (pt-PT)");
    assert_eq!(c.locale(), Locale::PtBr);
}

#[test]
fn first_locale_with_the_key_wins() {
    let c = catalog(&[(Locale::PtPt, PT_PT), (Locale::En, EN)]);
    assert_eq!(c.tr(k::COMMON_CANCEL).to_string(), "Cancelar (pt-PT)");
}

#[test]
fn key_no_locale_has_renders_its_path() {
    let c = catalog(&[(Locale::De, ""), (Locale::En, "")]);
    assert_eq!(c.tr(k::COMMON_CANCEL).to_string(), "common.cancel");
    assert_eq!(
        c.plural(k::CERT_EXPIRES_IN, 3).to_string(),
        "cert.expires_in"
    );
}

#[test]
fn unparsable_source_is_skipped_without_panicking() {
    let c = catalog(&[(Locale::Fr, "cancel = = ["), (Locale::En, EN)]);
    assert_eq!(c.tr(k::COMMON_CANCEL).to_string(), "Cancel");
}

#[test]
fn plural_category_follows_the_locale_that_had_the_table() {
    // pt-BR says 0 is `one`; the table came from English, where 0 is `other`.
    let c = catalog(&[(Locale::PtBr, ""), (Locale::En, EN)]);
    assert_eq!(c.plural(k::CERT_EXPIRES_IN, 0).to_string(), "en other 0");
}

#[test]
fn missing_category_uses_other() {
    let c = catalog(&[(Locale::PtBr, EN)]);
    assert_eq!(
        c.plural(k::CERT_EXPIRES_IN, 1_000_000).to_string(),
        "en other 1000000"
    );
}

#[test]
fn plural_table_without_the_category_or_other_renders_the_path() {
    let c = catalog(&[(Locale::En, "[cert.expires_in]\nfew = \"x\"\n")]);
    assert_eq!(
        c.plural(k::CERT_EXPIRES_IN, 5).to_string(),
        "cert.expires_in"
    );
}

#[test]
fn a_string_where_a_plural_is_expected_falls_through() {
    let broken = "[cert]\nexpires_in = \"flat\"\n";
    let c = catalog(&[(Locale::Es, broken), (Locale::En, EN)]);
    assert_eq!(c.plural(k::CERT_EXPIRES_IN, 1).to_string(), "en one 1");
}
