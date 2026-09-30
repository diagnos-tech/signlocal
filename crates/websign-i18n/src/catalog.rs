//! Loaded messages for one locale.

use crate::locale::Locale;
use crate::message::Message;
use crate::{Key, PluralKey};

/// The messages of one locale plus its fallbacks, parsed once.
#[derive(Debug)]
pub struct Catalog {
    locale: Locale,
}

impl Catalog {
    /// Parses the embedded files of `locale`'s fallback chain. The files are
    /// checked in CI, so a parse failure here is a bug: it falls back to
    /// English and logs, never panics.
    pub fn new(locale: Locale) -> Catalog {
        let _ = locale;
        todo!("SPEC.md §3")
    }

    /// The locale messages are looked up in first.
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// The message for `key`, ready for arguments.
    pub fn tr(&self, key: Key) -> Message<'_> {
        let _ = key;
        todo!("SPEC.md §3")
    }

    /// The plural form of `key` for `count`, with `{count}` already filled.
    pub fn plural(&self, key: PluralKey, count: i64) -> Message<'_> {
        let _ = (key, count);
        todo!("SPEC.md §3")
    }
}
