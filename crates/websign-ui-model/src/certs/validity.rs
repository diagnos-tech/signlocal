//! The validity line (`docs/ux.md` §5.6, vectors §16.5).

use jiff::civil::Date;

/// What line 3 says about validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidityLabel {
    /// More than 30 days left.
    ValidUntil(Date),
    /// 2 to 30 days left.
    ExpiresInDays(u32),
    ExpiresTomorrow,
    ExpiresToday,
    ExpiredOn(Date),
    ValidFrom(Date),
}

/// The color of the label; never the only signal (text says the same).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Muted,
    Warning,
    Danger,
}

/// The label and tone for a certificate valid from `not_before` to
/// `not_after`, both already converted to **local calendar dates**, seen on
/// local date `today`. Days are calendar-date differences, not 24-hour spans.
pub fn validity_label(not_before: Date, not_after: Date, today: Date) -> (ValidityLabel, Tone) {
    if today < not_before {
        return (ValidityLabel::ValidFrom(not_before), Tone::Warning);
    }
    if today > not_after {
        return (ValidityLabel::ExpiredOn(not_after), Tone::Danger);
    }
    let days = (not_after - today).get_days();
    let count = days.unsigned_abs();
    match days {
        0 => (ValidityLabel::ExpiresToday, Tone::Danger),
        1 => (ValidityLabel::ExpiresTomorrow, Tone::Danger),
        2..=7 => (ValidityLabel::ExpiresInDays(count), Tone::Danger),
        8..=30 => (ValidityLabel::ExpiresInDays(count), Tone::Warning),
        _ => (ValidityLabel::ValidUntil(not_after), Tone::Muted),
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    const TODAY: Date = date(2026, 9, 29);
    const LONG_AGO: Date = date(2020, 1, 1);

    fn label(not_after: Date) -> (ValidityLabel, Tone) {
        validity_label(LONG_AGO, not_after, TODAY)
    }

    #[test]
    fn spec_vectors() {
        let until = date(2026, 10, 30);
        assert_eq!(
            label(until),
            (ValidityLabel::ValidUntil(until), Tone::Muted)
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
        let expired = date(2026, 5, 10);
        assert_eq!(
            label(expired),
            (ValidityLabel::ExpiredOn(expired), Tone::Danger)
        );
        let from = date(2026, 10, 1);
        assert_eq!(
            validity_label(from, date(2027, 1, 1), TODAY),
            (ValidityLabel::ValidFrom(from), Tone::Warning)
        );
    }

    #[test]
    fn eight_days_is_the_warning_boundary() {
        assert_eq!(
            label(date(2026, 10, 7)),
            (ValidityLabel::ExpiresInDays(8), Tone::Warning)
        );
    }
}
