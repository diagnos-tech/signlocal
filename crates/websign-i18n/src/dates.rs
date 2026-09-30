//! Dates and times per locale, without ICU.

use jiff::civil::{Date, Time};

use crate::locale::Locale;

const MONTHS_EN: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// A calendar date: `"14/03/2027"` (pt, es, fr, it), `"14.03.2027"` (de),
/// `"14 Mar 2027"` (en).
pub fn format_date(locale: Locale, date: Date) -> String {
    let (year, month, day) = (date.year(), date.month(), date.day());
    match locale {
        Locale::En => {
            let index = usize::from(month.unsigned_abs()).saturating_sub(1);
            let name = MONTHS_EN.get(index).copied().unwrap_or_default();
            format!("{day} {name} {year:04}")
        }
        Locale::De => format!("{day:02}.{month:02}.{year:04}"),
        Locale::PtBr | Locale::PtPt | Locale::Es | Locale::Fr | Locale::It => {
            format!("{day:02}/{month:02}/{year:04}")
        }
    }
}

/// A clock time, 24-hour everywhere: `"14:02"`.
pub fn format_time(locale: Locale, time: Time) -> String {
    let _ = locale;
    format!("{:02}:{:02}", time.hour(), time.minute())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::{date, time};

    #[test]
    fn dates() {
        let d = date(2027, 3, 14);
        assert_eq!(format_date(Locale::En, d), "14 Mar 2027");
        assert_eq!(format_date(Locale::En, date(2027, 3, 4)), "4 Mar 2027");
        assert_eq!(format_date(Locale::PtBr, d), "14/03/2027");
        assert_eq!(format_date(Locale::Fr, date(2027, 3, 4)), "04/03/2027");
        assert_eq!(format_date(Locale::De, d), "14.03.2027");
    }

    #[test]
    fn times() {
        for locale in Locale::ALL {
            assert_eq!(format_time(locale, time(9, 5, 0, 0)), "09:05");
        }
    }
}
