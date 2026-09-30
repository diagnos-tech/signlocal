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
pub fn relative(then: DateTime, now: DateTime) -> Relative {
    let _ = (then, now);
    todo!("SPEC.md §4")
}
