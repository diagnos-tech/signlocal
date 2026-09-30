//! Localized words the tabs share: relative times, dates and the OS name.

use jiff::Timestamp;
use jiff::tz::TimeZone;
use websign_i18n::dates::{format_date, format_time};
use websign_i18n::{Catalog, Key, k};
use websign_ui_model::time::{Relative, relative};

/// The message of `key` with no arguments.
pub fn tr(catalog: &Catalog, key: Key) -> String {
    catalog.tr(key).to_string()
}

/// "3 min ago", "today, 14:02", "yesterday, 17:40", "12 Sep 2026".
pub fn when(catalog: &Catalog, then: i64, now: Timestamp, zone: &TimeZone) -> String {
    let Ok(then) = Timestamp::from_second(then) else {
        return String::new();
    };
    let then = then.to_zoned(zone.clone()).datetime();
    let now = now.to_zoned(zone.clone()).datetime();
    let locale = catalog.locale();
    match relative(then, now) {
        Relative::JustNow => tr(catalog, k::TIME_JUST_NOW),
        Relative::MinutesAgo(minutes) => catalog
            .tr(k::TIME_MINUTES_AGO)
            .arg("count", minutes)
            .to_string(),
        Relative::TodayAt(time) => catalog
            .tr(k::TIME_TODAY_AT)
            .arg("time", format_time(locale, time))
            .to_string(),
        Relative::YesterdayAt(time) => catalog
            .tr(k::TIME_YESTERDAY_AT)
            .arg("time", format_time(locale, time))
            .to_string(),
        Relative::On(at) => format_date(locale, at.date()),
    }
}

/// The local calendar date of `unix_seconds`.
pub fn date(catalog: &Catalog, unix_seconds: i64, zone: &TimeZone) -> String {
    Timestamp::from_second(unix_seconds)
        .map(|at| format_date(catalog.locale(), at.to_zoned(zone.clone()).date()))
        .unwrap_or_default()
}

/// This OS's name, for "Download for {os}".
pub fn this_os(catalog: &Catalog) -> String {
    let key = if cfg!(windows) {
        k::OS_WINDOWS
    } else if cfg!(target_os = "macos") {
        k::OS_MACOS
    } else {
        k::OS_LINUX
    };
    tr(catalog, key)
}
