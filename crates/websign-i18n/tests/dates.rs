//! Date and time formats per locale (SPEC §6).

use jiff::civil::{date, time};
use websign_i18n::Locale;
use websign_i18n::dates::{format_date, format_time};

const SLASH: [Locale; 5] = [
    Locale::PtBr,
    Locale::PtPt,
    Locale::Es,
    Locale::Fr,
    Locale::It,
];

#[test]
fn spec_date_examples() {
    let d = date(2027, 3, 14);
    assert_eq!(format_date(Locale::En, d), "14 Mar 2027");
    for l in SLASH {
        assert_eq!(format_date(l, d), "14/03/2027", "{l:?}");
    }
    assert_eq!(format_date(Locale::De, d), "14.03.2027");
}

#[test]
fn numeric_forms_zero_pad_day_and_month() {
    let d = date(2027, 1, 4);
    for l in SLASH {
        assert_eq!(format_date(l, d), "04/01/2027", "{l:?}");
    }
    assert_eq!(format_date(Locale::De, d), "04.01.2027");
}

#[test]
fn english_does_not_pad_the_day() {
    assert_eq!(format_date(Locale::En, date(2027, 3, 4)), "4 Mar 2027");
}

#[test]
fn english_month_abbreviations() {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    for (i, m) in months.iter().enumerate() {
        let d = date(2030, i as i8 + 1, 20);
        assert_eq!(format_date(Locale::En, d), format!("20 {m} 2030"));
    }
}

#[test]
fn year_edges_and_leap_day() {
    assert_eq!(format_date(Locale::De, date(2028, 2, 29)), "29.02.2028");
    assert_eq!(format_date(Locale::En, date(2028, 12, 31)), "31 Dec 2028");
    assert_eq!(format_date(Locale::Fr, date(2000, 1, 1)), "01/01/2000");
}

#[test]
fn time_is_24_hour_zero_padded_in_every_locale() {
    for l in Locale::ALL {
        assert_eq!(format_time(l, time(14, 2, 0, 0)), "14:02", "{l:?}");
        assert_eq!(format_time(l, time(0, 0, 0, 0)), "00:00", "{l:?}");
        assert_eq!(format_time(l, time(9, 5, 0, 0)), "09:05", "{l:?}");
        assert_eq!(
            format_time(l, time(23, 59, 59, 999_999_999)),
            "23:59",
            "{l:?}"
        );
        assert_eq!(format_time(l, time(12, 0, 0, 0)), "12:00", "{l:?}");
    }
}

#[test]
fn seconds_are_not_shown() {
    assert_eq!(format_time(Locale::En, time(8, 30, 45, 0)), "08:30");
}
