//! Rows for certificates the confirmation list leaves out but the
//! Certificates tab still shows under "Can't sign", with their reason
//! (`docs/ux.md` §8.5: "Made for login, not for signing", "No private key on
//! this computer").

use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;
use websign_core::present::document::display_document;
use websign_core::present::holder::display_name;
use websign_ui_model::certs::{
    CertCandidate, CertList, CertRow, HiddenReason, ListContext, RowStatus, badge, location,
    validity_label,
};

/// Why a hidden certificate is shown anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShownReason {
    LoginOnly,
    NoPrivateKey,
}

/// The hidden entries of `list` worth showing, as rows in list order.
pub fn hidden_rows(
    candidates: &[CertCandidate],
    list: &CertList,
    context: &ListContext,
) -> Vec<(CertRow, ShownReason)> {
    list.hidden
        .iter()
        .filter_map(|(fingerprint, reason)| {
            let shown = match reason {
                HiddenReason::KeyUsage
                | HiddenReason::ExtendedKeyUsage
                | HiddenReason::LoginSibling => ShownReason::LoginOnly,
                HiddenReason::NoPrivateKey => ShownReason::NoPrivateKey,
                _ => return None,
            };
            let candidate = candidates.iter().find(|c| c.fingerprint == *fingerprint)?;
            Some((row(candidate, context)?, shown))
        })
        .collect()
}

/// The row `websign-ui-model` would build, for a candidate it hid.
fn row(candidate: &CertCandidate, context: &ListContext) -> Option<CertRow> {
    let info = candidate.info.as_ref().ok()?;
    let zone = &context.time_zone;
    let (validity, _) = validity_label(
        local_date(info.not_before, zone),
        local_date(info.not_after, zone),
        context.today,
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
        status: RowStatus::Usable,
    })
}

fn local_date(unix_secs: i64, zone: &TimeZone) -> Date {
    match Timestamp::from_second(unix_secs) {
        Ok(instant) => instant.to_zoned(zone.clone()).date(),
        Err(_) if unix_secs < 0 => Date::MIN,
        Err(_) => Date::MAX,
    }
}
