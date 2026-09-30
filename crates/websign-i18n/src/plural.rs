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

impl PluralCategory {
    /// The CLDR name, used as the key inside a plural table of a locale file.
    pub const fn name(self) -> &'static str {
        match self {
            PluralCategory::Zero => "zero",
            PluralCategory::One => "one",
            PluralCategory::Two => "two",
            PluralCategory::Few => "few",
            PluralCategory::Many => "many",
            PluralCategory::Other => "other",
        }
    }
}

/// The category of the integer `count` in `locale` (CLDR 46 cardinal rules
/// for integers: `SPEC.md` §5 lists them per locale).
pub fn category(locale: Locale, count: i64) -> PluralCategory {
    // `unsigned_abs` keeps `i64::MIN` from overflowing.
    let n = count.unsigned_abs();
    let one = match locale {
        Locale::PtBr | Locale::Fr => n <= 1,
        Locale::En | Locale::PtPt | Locale::Es | Locale::It | Locale::De => n == 1,
    };
    let has_many = !matches!(locale, Locale::En | Locale::De);
    if one {
        PluralCategory::One
    } else if has_many && n != 0 && n.is_multiple_of(1_000_000) {
        PluralCategory::Many
    } else {
        PluralCategory::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PluralCategory::{Many, One, Other};

    #[test]
    fn vectors() {
        let cases = [
            (Locale::En, 0, Other),
            (Locale::En, 1, One),
            (Locale::En, 2, Other),
            (Locale::PtBr, 0, One),
            (Locale::PtBr, 1, One),
            (Locale::PtBr, 2, Other),
            (Locale::PtBr, 1_000_000, Many),
            (Locale::PtPt, 0, Other),
            (Locale::Fr, 0, One),
            (Locale::Fr, 2, Other),
            (Locale::De, 1_000_000, Other),
            (Locale::Es, 1_000_000, Many),
            (Locale::It, 2_000_000, Many),
            (Locale::Es, -1, One),
        ];
        for (locale, n, expected) in cases {
            assert_eq!(category(locale, n), expected, "{locale:?} {n}");
        }
    }

    #[test]
    fn extreme_values_do_not_panic() {
        assert_eq!(category(Locale::Es, i64::MIN), Other);
    }
}
