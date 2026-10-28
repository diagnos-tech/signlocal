//! Loaded messages for one locale.

use crate::locale::Locale;
use crate::message::Message;
use crate::plural::{self, PluralCategory};
use crate::{Key, PluralKey, SOURCES};

/// The messages of one locale plus its fallbacks, parsed once.
#[derive(Debug)]
pub struct Catalog {
    locale: Locale,
    /// Parsed files in lookup order: the locale first, English last.
    chain: Vec<(Locale, toml::Table)>,
}

impl Catalog {
    /// Parses the embedded files of `locale`'s fallback chain. The files are
    /// checked in CI, so a parse failure here is a bug: that file is logged
    /// and skipped (its keys fall through to the next locale), never a panic.
    pub fn new(locale: Locale) -> Catalog {
        let sources: Vec<(Locale, &str)> = locale
            .fallback_chain()
            .iter()
            .filter_map(|&member| Some((member, embedded(member)?)))
            .collect();
        Catalog::from_sources(locale, &sources)
    }

    /// A catalog over explicit `(locale, TOML source)` pairs in lookup order,
    /// with the same parsing and fallback rules as [`Catalog::new`]. It exists
    /// so fallback can be tested: the shipped files never lack a key.
    #[doc(hidden)]
    pub fn from_sources(locale: Locale, sources: &[(Locale, &str)]) -> Catalog {
        let chain = sources
            .iter()
            .filter_map(|&(member, source)| match source.parse::<toml::Table>() {
                Ok(table) => Some((member, table)),
                Err(error) => {
                    log::error!("i18n: {} is not valid TOML, skipped: {error}", member.tag());
                    None
                }
            })
            .collect();
        Catalog { locale, chain }
    }

    /// The locale messages are looked up in first.
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// The message for `key`, ready for arguments. A key no locale has renders
    /// as its own path, which is visible and caught by review.
    pub fn tr(&self, key: Key) -> Message<'_> {
        let template = self
            .chain
            .iter()
            .find_map(|(_, table)| lookup(table, key.path())?.as_str())
            .unwrap_or_else(|| key.path());
        Message::new(template)
    }

    /// The plural form of `key` for `count`, with `{count}` already filled
    /// (`-5` stays `-5`). The table comes from the first locale that has it and
    /// the category from that locale's rules; a category the table lacks uses
    /// `other`, and a table without either renders the key path.
    pub fn plural(&self, key: PluralKey, count: i64) -> Message<'_> {
        let found = self.chain.iter().find_map(|(member, table)| {
            let forms = lookup(table, key.path())?.as_table()?;
            Some((*member, forms))
        });
        let template = found
            .and_then(|(member, forms)| {
                let category = plural::category(member, count);
                forms
                    .get(category.name())
                    .or_else(|| forms.get(PluralCategory::Other.name()))?
                    .as_str()
            })
            .unwrap_or_else(|| key.path());
        Message::new(template).arg("count", count)
    }
}

fn embedded(locale: Locale) -> Option<&'static str> {
    SOURCES
        .iter()
        .find(|(tag, _)| *tag == locale.tag())
        .map(|(_, source)| *source)
}

fn lookup<'a>(table: &'a toml::Table, path: &str) -> Option<&'a toml::Value> {
    let mut segments = path.split('.');
    let mut value = table.get(segments.next()?)?;
    for segment in segments {
        value = value.as_table()?.get(segment)?;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::k;

    #[test]
    fn english_plural_uses_count() {
        let catalog = Catalog::new(Locale::En);
        assert_eq!(
            catalog.plural(k::CERT_EXPIRES_IN, 1).to_string(),
            "Expires in 1 day"
        );
        assert_eq!(
            catalog.plural(k::CERT_EXPIRES_IN, 23).to_string(),
            "Expires in 23 days"
        );
    }

    #[test]
    fn every_locale_loads_every_key() {
        for locale in Locale::ALL {
            let catalog = Catalog::new(locale);
            assert_eq!(catalog.locale(), locale);
            for path in crate::ALL_KEYS {
                let found = catalog
                    .chain
                    .iter()
                    .any(|(_, table)| lookup(table, path).is_some());
                assert!(found, "{locale:?} lacks {path}");
            }
        }
    }

    #[test]
    fn unknown_key_renders_its_path() {
        let catalog = Catalog::new(Locale::De);
        assert_eq!(catalog.tr(Key("no.such.key")).to_string(), "no.such.key");
    }
}
