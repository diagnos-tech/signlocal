//! A certificate row's words (`docs/ux.md` §5.1–5.7): the model's row
//! turned into the localized texts the shared row widget draws.

use websign_core::present::document::DocumentLabel;
use websign_i18n::dates::format_date;
use websign_i18n::{Catalog, k};
use websign_ui_model::certs::{Badge, CertRow, DisabledReason, Place, RowStatus, ValidityLabel};

use super::words::tr;
use crate::ui::diagnostics::facts::ShownReason;
use crate::ui::icons::{self, Icon};
use crate::ui::widgets::cert_row::CertRowText;
use crate::ui::widgets::tone::Tone;

/// Owned texts of one row.
#[derive(Debug, Clone)]
pub struct RowWords {
    pub name: String,
    pub badge: String,
    pub detail: String,
    pub location_icon: Icon,
    pub location: String,
    pub status: String,
    pub tone: Tone,
    pub status_icon: Option<Icon>,
    pub accessible: String,
}

impl RowWords {
    /// The words of `row`; `hidden` names why a row the list hid is shown.
    pub fn of(catalog: &Catalog, row: &CertRow, hidden: Option<ShownReason>) -> RowWords {
        let (location_icon, location) = location(catalog, row);
        let (status, tone, status_icon) = match (hidden, row.status) {
            (Some(ShownReason::LoginOnly), _) => (
                tr(catalog, k::CERT_REASON_LOGIN_ONLY),
                Tone::Neutral,
                Some(icons::NOT_APPLICABLE),
            ),
            (Some(ShownReason::NoPrivateKey), _) => (
                tr(catalog, k::CERT_REASON_NO_KEY),
                Tone::Neutral,
                Some(icons::NOT_APPLICABLE),
            ),
            (None, RowStatus::Disabled(reason)) => disabled(catalog, row, reason),
            (None, RowStatus::Usable) => validity(catalog, row.validity),
        };
        let detail = [
            document(catalog, row.document.as_ref()),
            Some(row.issuer.clone()),
        ]
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
        let badge = badge(catalog, &row.badge);
        let accessible = [&row.name, &badge, &detail, &location, &status]
            .into_iter()
            .filter(|part| !part.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        RowWords {
            name: row.name.clone(),
            badge,
            detail,
            location_icon,
            location,
            status,
            tone,
            status_icon,
            accessible,
        }
    }

    /// Borrowed as the widget wants them.
    pub fn text(&self) -> CertRowText<'_> {
        CertRowText {
            name: &self.name,
            badge: &self.badge,
            detail: &self.detail,
            location_icon: self.location_icon,
            location: &self.location,
            status: &self.status,
            status_tone: self.tone,
            status_icon: self.status_icon,
            accessible_name: &self.accessible,
        }
    }
}

fn badge(catalog: &Catalog, badge: &Badge) -> String {
    match badge {
        Badge::IcpBrasil { class } => catalog.tr(k::CERT_KIND_ICP).arg("class", class).to_string(),
        Badge::IcpBrasilPlain => tr(catalog, k::CERT_KIND_ICP_PLAIN),
        Badge::EidasQualified => tr(catalog, k::CERT_KIND_EIDAS_QSCD),
        Badge::Eidas => tr(catalog, k::CERT_KIND_EIDAS),
        Badge::PtCitizenCard => tr(catalog, k::CERT_KIND_PT_CC),
        Badge::EsDnie => tr(catalog, k::CERT_KIND_ES_DNIE),
        Badge::Generic => tr(catalog, k::CERT_KIND_GENERIC),
    }
}

fn document(catalog: &Catalog, document: Option<&DocumentLabel>) -> Option<String> {
    Some(match document? {
        DocumentLabel::Cpf { masked, .. } => catalog
            .tr(k::CERT_DOC_CPF)
            .arg("masked", masked)
            .to_string(),
        DocumentLabel::Cnpj { formatted } => catalog
            .tr(k::CERT_DOC_CNPJ)
            .arg("cnpj", formatted)
            .to_string(),
        DocumentLabel::National { masked } => catalog
            .tr(k::CERT_DOC_GENERIC)
            .arg("masked", masked)
            .to_string(),
    })
}

fn location(catalog: &Catalog, row: &CertRow) -> (Icon, String) {
    let (icon, place) = match &row.location.place {
        Place::Computer => (icons::ON_THIS_COMPUTER, tr(catalog, k::CERT_WHERE_COMPUTER)),
        Place::Token { name } => (
            icons::TOKEN,
            catalog
                .tr(k::CERT_WHERE_TOKEN_NAMED)
                .arg("device", name)
                .to_string(),
        ),
        Place::CardInReader => (icons::CARD, tr(catalog, k::CERT_WHERE_CARD_IN_READER)),
        Place::UnknownHardware => (icons::TOKEN, tr(catalog, k::CERT_WHERE_UNKNOWN_HW)),
    };
    if row.location.via_driver {
        (
            icon,
            format!("{place} · {}", tr(catalog, k::CERT_VIA_DRIVER)),
        )
    } else {
        (icon, place)
    }
}

fn validity(catalog: &Catalog, label: ValidityLabel) -> (String, Tone, Option<Icon>) {
    let date = |date| format_date(catalog.locale(), date);
    match label {
        ValidityLabel::ValidUntil(day) => (
            catalog
                .tr(k::CERT_VALID_UNTIL)
                .arg("date", date(day))
                .to_string(),
            Tone::Neutral,
            None,
        ),
        ValidityLabel::ExpiresInDays(days) => (
            catalog
                .plural(k::CERT_EXPIRES_IN, i64::from(days))
                .to_string(),
            if days <= 7 {
                Tone::Danger
            } else {
                Tone::Warning
            },
            Some(icons::EXPIRES_SOON),
        ),
        ValidityLabel::ExpiresTomorrow => (
            tr(catalog, k::CERT_EXPIRES_TOMORROW),
            Tone::Danger,
            Some(icons::EXPIRES_SOON),
        ),
        ValidityLabel::ExpiresToday => (
            tr(catalog, k::CERT_EXPIRES_TODAY),
            Tone::Danger,
            Some(icons::EXPIRES_SOON),
        ),
        ValidityLabel::ExpiredOn(day) => (
            catalog
                .tr(k::CERT_EXPIRED_ON)
                .arg("date", date(day))
                .to_string(),
            Tone::Danger,
            Some(icons::ERROR),
        ),
        ValidityLabel::ValidFrom(day) => (
            catalog
                .tr(k::CERT_VALID_FROM)
                .arg("date", date(day))
                .to_string(),
            Tone::Warning,
            Some(icons::EXPIRES_SOON),
        ),
    }
}

fn disabled(
    catalog: &Catalog,
    row: &CertRow,
    reason: DisabledReason,
) -> (String, Tone, Option<Icon>) {
    match reason {
        DisabledReason::Expired | DisabledReason::NotYetValid => validity(catalog, row.validity),
        DisabledReason::PinLocked => (
            tr(catalog, k::CERT_REASON_PIN_LOCKED),
            Tone::Danger,
            Some(icons::PIN_LOCKED),
        ),
        DisabledReason::Incompatible => (
            tr(catalog, k::CERT_REASON_INCOMPATIBLE),
            Tone::Neutral,
            Some(icons::NOT_APPLICABLE),
        ),
        DisabledReason::Removed => (
            tr(catalog, k::CERT_REASON_REMOVED),
            Tone::Warning,
            Some(icons::ATTENTION),
        ),
    }
}
