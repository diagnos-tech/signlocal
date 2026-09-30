//! CLDR plural rules for the shipped locales, as one pure function.

use crate::locale::Locale;

/// A CLDR plural category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

/// The category of the integer `count` in `locale` (CLDR 46 cardinal rules
/// for integers: `SPEC.md` §5 lists them per locale).
pub fn category(locale: Locale, count: i64) -> PluralCategory {
    let _ = (locale, count);
    todo!("SPEC.md §5")
}
