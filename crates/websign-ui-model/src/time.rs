//! Relative times for diagnostics ("3 min ago", "yesterday, 17:40").

use jiff::civil::{DateTime, Time};

/// A relative description of a past moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relative {
    /// Less than a minute ago.
    JustNow,
    /// 1 to 59 minutes ago.
    MinutesAgo(u32),
    /// Earlier today.
    TodayAt(Time),
    YesterdayAt(Time),
    /// Older: the renderer shows the date.
    On(DateTime),
}

/// Describes `then` as seen at `now`, both local.
///
/// A `then` in the future is `JustNow`: clocks drift, and "in 2 minutes" in a
/// support report would only confuse.
pub fn relative(then: DateTime, now: DateTime) -> Relative {
    let seconds = now.duration_since(then).as_secs();
    if seconds < 60 {
        return Relative::JustNow;
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return Relative::MinutesAgo(u32::try_from(minutes).unwrap_or(59));
    }
    if then.date() == now.date() {
        return Relative::TodayAt(then.time());
    }
    if now.date().yesterday().is_ok_and(|day| day == then.date()) {
        return Relative::YesterdayAt(then.time());
    }
    Relative::On(then)
}

#[cfg(test)]
mod tests {
    use jiff::civil::datetime;

    use super::*;

    fn now() -> DateTime {
        datetime(2026, 9, 29, 14, 30, 0, 0)
    }

    #[test]
    fn under_a_minute_and_future_are_just_now() {
        assert_eq!(
            relative(datetime(2026, 9, 29, 14, 29, 30, 0), now()),
            Relative::JustNow
        );
        assert_eq!(
            relative(datetime(2026, 9, 29, 15, 0, 0, 0), now()),
            Relative::JustNow
        );
    }

    #[test]
    fn minutes_are_floored() {
        assert_eq!(
            relative(datetime(2026, 9, 29, 14, 28, 1, 0), now()),
            Relative::MinutesAgo(1)
        );
        assert_eq!(
            relative(datetime(2026, 9, 29, 13, 31, 0, 0), now()),
            Relative::MinutesAgo(59)
        );
    }

    #[test]
    fn today_yesterday_and_older() {
        let at = datetime(2026, 9, 29, 9, 5, 0, 0);
        assert_eq!(relative(at, now()), Relative::TodayAt(at.time()));
        let y = datetime(2026, 9, 28, 17, 40, 0, 0);
        assert_eq!(relative(y, now()), Relative::YesterdayAt(y.time()));
        let old = datetime(2026, 9, 1, 8, 0, 0, 0);
        assert_eq!(relative(old, now()), Relative::On(old));
    }

    #[test]
    fn crossing_midnight_within_an_hour_is_minutes() {
        let late = datetime(2026, 9, 30, 0, 10, 0, 0);
        assert_eq!(
            relative(datetime(2026, 9, 29, 23, 50, 0, 0), late),
            Relative::MinutesAgo(20)
        );
    }
}
