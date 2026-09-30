//! SPEC §1.3 and ux §16.5: the validity line and its tone.

use jiff::civil::{Date, date};
use websign_ui_model::certs::{Tone, ValidityLabel, validity_label};

const TODAY: Date = date(2026, 9, 29);
const LONG_AGO: Date = date(2025, 1, 1);

fn label(not_after: Date) -> (ValidityLabel, Tone) {
    validity_label(LONG_AGO, not_after, TODAY)
}

#[test]
fn spec_vectors_for_today_2026_09_29() {
    assert_eq!(
        label(date(2026, 10, 30)),
        (ValidityLabel::ValidUntil(date(2026, 10, 30)), Tone::Muted)
    );
    assert_eq!(
        label(date(2026, 10, 29)),
        (ValidityLabel::ExpiresInDays(30), Tone::Warning)
    );
    assert_eq!(
        label(date(2026, 10, 22)),
        (ValidityLabel::ExpiresInDays(23), Tone::Warning)
    );
    assert_eq!(
        label(date(2026, 10, 6)),
        (ValidityLabel::ExpiresInDays(7), Tone::Danger)
    );
    assert_eq!(
        label(date(2026, 9, 30)),
        (ValidityLabel::ExpiresTomorrow, Tone::Danger)
    );
    assert_eq!(
        label(date(2026, 9, 29)),
        (ValidityLabel::ExpiresToday, Tone::Danger)
    );
    assert_eq!(
        label(date(2026, 5, 10)),
        (ValidityLabel::ExpiredOn(date(2026, 5, 10)), Tone::Danger)
    );
    assert_eq!(
        validity_label(date(2026, 10, 1), date(2027, 10, 1), TODAY),
        (ValidityLabel::ValidFrom(date(2026, 10, 1)), Tone::Warning)
    );
}

#[test]
fn every_day_count_from_two_to_thirty_one_has_the_right_label_and_tone() {
    for days in 2..=31 {
        let not_after = TODAY.checked_add(jiff::Span::new().days(days)).unwrap();
        let expected = match days {
            2..=7 => (ValidityLabel::ExpiresInDays(days as u32), Tone::Danger),
            8..=30 => (ValidityLabel::ExpiresInDays(days as u32), Tone::Warning),
            _ => (ValidityLabel::ValidUntil(not_after), Tone::Muted),
        };
        assert_eq!(label(not_after), expected, "{days} days left");
    }
}

#[test]
fn eight_days_is_the_first_warning_and_seven_the_last_danger() {
    assert_eq!(label(date(2026, 10, 7)).1, Tone::Warning);
    assert_eq!(label(date(2026, 10, 6)).1, Tone::Danger);
}

#[test]
fn days_are_calendar_dates_across_a_year_boundary() {
    let result = validity_label(LONG_AGO, date(2027, 1, 1), date(2026, 12, 25));
    assert_eq!(result, (ValidityLabel::ExpiresInDays(7), Tone::Danger));
}

#[test]
fn days_count_the_leap_day() {
    let (label, _) = validity_label(LONG_AGO, date(2028, 3, 1), date(2028, 2, 28));
    assert_eq!(label, ValidityLabel::ExpiresInDays(2));
}

#[test]
fn valid_from_wins_over_every_expiry_rule() {
    // Not yet valid and, by the calendar, also "expires today".
    let result = validity_label(date(2026, 10, 1), date(2026, 9, 29), TODAY);
    assert_eq!(
        result,
        (ValidityLabel::ValidFrom(date(2026, 10, 1)), Tone::Warning)
    );
}

#[test]
fn the_first_valid_day_is_valid() {
    let (label, _) = validity_label(TODAY, date(2027, 1, 1), TODAY);
    assert_eq!(label, ValidityLabel::ValidUntil(date(2027, 1, 1)));
}

#[test]
fn the_day_after_expiry_is_expired() {
    let result = validity_label(LONG_AGO, date(2026, 9, 28), TODAY);
    assert_eq!(
        result,
        (ValidityLabel::ExpiredOn(date(2026, 9, 28)), Tone::Danger)
    );
}
