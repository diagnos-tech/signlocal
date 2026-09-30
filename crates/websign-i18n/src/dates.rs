//! Dates and times per locale, without ICU.

use jiff::civil::{Date, Time};

use crate::locale::Locale;

/// A calendar date: `"14/03/2027"` (pt, es, fr, it), `"14.03.2027"` (de),
/// `"14 Mar 2027"` (en).
pub fn format_date(locale: Locale, date: Date) -> String {
    let _ = (locale, date);
    todo!("SPEC.md §6")
}

/// A clock time, 24-hour everywhere: `"14:02"`.
pub fn format_time(locale: Locale, time: Time) -> String {
    let _ = (locale, time);
    todo!("SPEC.md §6")
}
