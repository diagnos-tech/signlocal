//! The catalog the windows read their text from: the system locale, unless
//! `WEBSIGN_LOCALE` overrides it.
//!
//! The override exists for people whose OS language differs from the one
//! they want to read (a shared clinic computer), for support ("send me a
//! screenshot in English") and for the per-locale screenshot tests.

use websign_i18n::{Catalog, Locale};

/// Environment variable naming a locale (`pt-BR`, `de`, `es_MX.UTF-8`, …).
pub const LOCALE_ENV: &str = "WEBSIGN_LOCALE";

/// The catalog for the chosen locale.
pub fn catalog() -> Catalog {
    Catalog::new(locale())
}

/// The override if it names a shipped language, else the system locale.
pub fn locale() -> Locale {
    let requested = std::env::var(LOCALE_ENV).ok();
    resolve(requested.as_deref()).unwrap_or_else(Locale::from_system)
}

/// The shipped locale an override names; an unknown one is ignored with a
/// warning so a typo falls back to the system language instead of English.
fn resolve(requested: Option<&str>) -> Option<Locale> {
    let tag = requested?.trim();
    if tag.is_empty() {
        return None;
    }
    Locale::match_tag(tag).or_else(|| {
        log::warn!("ignoring {LOCALE_ENV}={tag:?}: not a shipped locale");
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_picks_the_closest_shipped_locale() {
        assert_eq!(resolve(Some("pt-BR")), Some(Locale::PtBr));
        assert_eq!(resolve(Some("de_AT.UTF-8")), Some(Locale::De));
        assert_eq!(resolve(Some(" fr ")), Some(Locale::Fr));
    }

    #[test]
    fn missing_empty_or_unknown_override_defers_to_the_system() {
        assert_eq!(resolve(None), None);
        assert_eq!(resolve(Some("")), None);
        assert_eq!(resolve(Some("zh-CN")), None);
    }
}
