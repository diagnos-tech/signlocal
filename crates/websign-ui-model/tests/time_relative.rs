//! SPEC §4: relative times of the diagnostics window.

use jiff::civil::{DateTime, date, time};
use websign_ui_model::time::{Relative, relative};

fn at(y: i16, mo: i8, d: i8, h: i8, mi: i8, s: i8) -> DateTime {
    date(y, mo, d).at(h, mi, s, 0)
}

const NOW: DateTime = date(2026, 9, 29).at(15, 0, 0, 0);

#[test]
fn a_moment_in_the_future_is_just_now() {
    assert_eq!(relative(at(2026, 9, 29, 15, 5, 0), NOW), Relative::JustNow);
    assert_eq!(relative(at(2026, 9, 30, 9, 0, 0), NOW), Relative::JustNow);
}

#[test]
fn under_a_minute_is_just_now() {
    assert_eq!(relative(NOW, NOW), Relative::JustNow);
    assert_eq!(relative(at(2026, 9, 29, 14, 59, 1), NOW), Relative::JustNow);
}

#[test]
fn a_minute_is_one_minute_and_minutes_are_floored() {
    assert_eq!(
        relative(at(2026, 9, 29, 14, 59, 0), NOW),
        Relative::MinutesAgo(1)
    );
    assert_eq!(
        relative(at(2026, 9, 29, 14, 57, 1), NOW),
        Relative::MinutesAgo(2)
    );
    assert_eq!(
        relative(at(2026, 9, 29, 14, 0, 1), NOW),
        Relative::MinutesAgo(59)
    );
}

#[test]
fn an_hour_ago_is_today_at_the_time() {
    assert_eq!(
        relative(at(2026, 9, 29, 14, 0, 0), NOW),
        Relative::TodayAt(time(14, 0, 0, 0))
    );
    assert_eq!(
        relative(at(2026, 9, 29, 9, 10, 0), NOW),
        Relative::TodayAt(time(9, 10, 0, 0))
    );
}

#[test]
fn the_minute_rule_wins_across_midnight() {
    let now = at(2026, 9, 29, 0, 30, 0);
    assert_eq!(
        relative(at(2026, 9, 28, 23, 50, 0), now),
        Relative::MinutesAgo(40)
    );
    let just = at(2026, 9, 29, 0, 0, 30);
    assert_eq!(
        relative(at(2026, 9, 28, 23, 59, 50), just),
        Relative::JustNow
    );
}

#[test]
fn previous_date_beyond_an_hour_is_yesterday() {
    let now = at(2026, 9, 29, 0, 30, 0);
    assert_eq!(
        relative(at(2026, 9, 28, 23, 20, 0), now),
        Relative::YesterdayAt(time(23, 20, 0, 0))
    );
    assert_eq!(
        relative(at(2026, 9, 28, 17, 40, 0), NOW),
        Relative::YesterdayAt(time(17, 40, 0, 0))
    );
}

#[test]
fn yesterday_crosses_month_and_year_boundaries() {
    assert_eq!(
        relative(at(2026, 9, 30, 10, 0, 0), at(2026, 10, 1, 10, 0, 0)),
        Relative::YesterdayAt(time(10, 0, 0, 0))
    );
    assert_eq!(
        relative(at(2026, 12, 31, 22, 0, 0), at(2027, 1, 1, 8, 0, 0)),
        Relative::YesterdayAt(time(22, 0, 0, 0))
    );
}

#[test]
fn older_than_yesterday_is_the_date() {
    let then = at(2026, 9, 27, 23, 59, 0);
    assert_eq!(relative(then, NOW), Relative::On(then));
    let last_year = at(2025, 9, 29, 15, 0, 0);
    assert_eq!(relative(last_year, NOW), Relative::On(last_year));
}
