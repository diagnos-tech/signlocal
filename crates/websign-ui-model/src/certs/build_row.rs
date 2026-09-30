//! From a candidate to the row the window draws.

use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;
use websign_core::present::document::display_document;
use websign_core::present::holder::display_name;

use super::badge::badge;
use super::candidate::CertCandidate;
use super::location::location;
use super::row::{CertRow, RowStatus};
use super::validity::validity_label;

/// The row for `candidate`, or `None` when its certificate did not parse.
pub(super) fn make_row(
    candidate: &CertCandidate,
    status: RowStatus,
    today: Date,
) -> Option<CertRow> {
    let info = candidate.info.as_ref().ok()?;
    let (validity, _tone) = validity_label(
        local_date(info.not_before),
        local_date(info.not_after),
        today,
    );
    let issuer = info
        .issuer
        .common_name
        .as_ref()
        .or(info.issuer.organization.as_ref())
        .cloned()
        .unwrap_or_default();
    Some(CertRow {
        candidate: candidate.clone(),
        name: display_name(info),
        badge: badge(info),
        document: display_document(info),
        issuer,
        location: location(candidate),
        validity,
        status,
    })
}

/// The calendar date of `unix_secs` in the computer's time zone; instants
/// outside the representable range clamp to the nearest date.
fn local_date(unix_secs: i64) -> Date {
    match Timestamp::from_second(unix_secs) {
        Ok(instant) => instant.to_zoned(TimeZone::system()).date(),
        Err(_) if unix_secs < 0 => Date::MIN,
        Err(_) => Date::MAX,
    }
}
