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
    let _ = (not_before, not_after, today);
    todo!("SPEC.md §1.3")
}
