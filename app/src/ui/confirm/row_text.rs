//! The localized texts of one certificate row (`docs/ux.md` §5.1–§5.7) and
//! its full sentence for screen readers (§14).

use websign_core::present::document::DocumentLabel;
use websign_i18n::{Catalog, dates, k};
use websign_ui_model::certs::{Badge, CertRow, DisabledReason, Place, RowStatus, ValidityLabel};

use crate::ui::icons::{self, Icon};
use crate::ui::widgets::cert_row::CertRowText;
use crate::ui::widgets::tone::Tone;

/// Owned texts; [`RowText::widget`] lends them to the row widget.
#[derive(Debug, Clone)]
pub struct RowText {
    name: String,
    badge: String,
    detail: String,
    location_icon: Icon,
    location: String,
    status: String,
    status_tone: Tone,
    status_icon: Option<Icon>,
    accessible: String,
}

impl RowText {
    pub fn of(tr: &Catalog, row: &CertRow) -> RowText {
        let badge = badge(tr, &row.badge);
        let (document, spoken_document) = document(tr, row.document.as_ref());
        let detail = match &document {
            Some(document) => format!("{document} · {}", row.issuer),
            None => row.issuer.clone(),
        };
        let (location_icon, mut location) = place(tr, &row.location.place);
        if row.location.via_driver {
            location = format!("{location} · {}", tr.tr(k::CERT_VIA_DRIVER));
        }
        let (status, status_tone, status_icon) = status(tr, row);
        let accessible = [
            Some(row.name.as_str()),
            Some(badge.as_str()),
            spoken_document.as_deref(),
            Some(row.issuer.as_str()).filter(|issuer| !issuer.is_empty()),
            Some(location.as_str()),
            Some(status.as_str()),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ");
        RowText {
            name: row.name.clone(),
            badge,
            detail,
            location_icon,
            location,
            status,
            status_tone,
            status_icon,
            accessible,
        }
    }

    pub fn widget(&self) -> CertRowText<'_> {
        CertRowText {
            name: &self.name,
            badge: &self.badge,
            detail: &self.detail,
            location_icon: self.location_icon,
            location: &self.location,
            status: &self.status,
            status_tone: self.status_tone,
            status_icon: self.status_icon,
            accessible_name: &self.accessible,
        }
    }
}

fn badge(tr: &Catalog, badge: &Badge) -> String {
    let key = match badge {
        Badge::PtCitizenCard => k::CERT_KIND_PT_CC,
        Badge::EsDnie => k::CERT_KIND_ES_DNIE,
        Badge::IcpBrasil { class } => {
            return tr.tr(k::CERT_KIND_ICP).arg("class", class).to_string();
        }
        Badge::IcpBrasilPlain => k::CERT_KIND_ICP_PLAIN,
        Badge::EidasQualified => k::CERT_KIND_EIDAS_QSCD,
        Badge::Eidas => k::CERT_KIND_EIDAS,
        Badge::Generic => k::CERT_KIND_GENERIC,
    };
    tr.tr(key).to_string()
}

/// The shown document and how a screen reader says it.
fn document(tr: &Catalog, label: Option<&DocumentLabel>) -> (Option<String>, Option<String>) {
    let Some(label) = label else {
        return (None, None);
    };
    let (shown, spoken) = match label {
        DocumentLabel::Cpf { masked, visible } => (
            tr.tr(k::CERT_DOC_CPF).arg("masked", masked).to_string(),
            tr.tr(k::CERT_DOC_CPF_A11Y)
                .arg("visible", visible)
                .to_string(),
        ),
        DocumentLabel::Cnpj { formatted } => {
            let text = tr.tr(k::CERT_DOC_CNPJ).arg("cnpj", formatted).to_string();
            (text.clone(), text)
        }
        DocumentLabel::National { masked } => {
            let text = tr.tr(k::CERT_DOC_GENERIC).arg("masked", masked).to_string();
            (text.clone(), text)
        }
    };
    (Some(shown), Some(spoken))
}

fn place(tr: &Catalog, place: &Place) -> (Icon, String) {
    match place {
        Place::Computer => (
            icons::ON_THIS_COMPUTER,
            tr.tr(k::CERT_WHERE_COMPUTER).to_string(),
        ),
        Place::Token { name } => (
            icons::TOKEN,
            tr.tr(k::CERT_WHERE_TOKEN_NAMED)
                .arg("device", name)
                .to_string(),
        ),
        Place::CardInReader => (icons::CARD, tr.tr(k::CERT_WHERE_CARD_IN_READER).to_string()),
        Place::UnknownHardware => (icons::TOKEN, tr.tr(k::CERT_WHERE_UNKNOWN_HW).to_string()),
    }
}

/// Validity, or why a disabled row cannot sign (§5.6, §5.8).
fn status(tr: &Catalog, row: &CertRow) -> (String, Tone, Option<Icon>) {
    let reason = match row.status {
        RowStatus::Disabled(DisabledReason::PinLocked) => Some(k::CERT_REASON_PIN_LOCKED),
        RowStatus::Disabled(DisabledReason::Incompatible) => Some(k::CERT_REASON_INCOMPATIBLE),
        RowStatus::Disabled(DisabledReason::Removed) => Some(k::CERT_REASON_REMOVED),
        _ => None,
    };
    if let Some(key) = reason {
        return (tr.tr(key).to_string(), Tone::Danger, Some(icons::ERROR));
    }
    let date = |date| dates::format_date(tr.locale(), date);
    match row.validity {
        ValidityLabel::ValidUntil(until) => (
            tr.tr(k::CERT_VALID_UNTIL)
                .arg("date", date(until))
                .to_string(),
            Tone::Neutral,
            None,
        ),
        ValidityLabel::ExpiresInDays(days) => (
            tr.plural(k::CERT_EXPIRES_IN, i64::from(days)).to_string(),
            if days <= 7 {
                Tone::Danger
            } else {
                Tone::Warning
            },
            Some(icons::EXPIRES_SOON),
        ),
        ValidityLabel::ExpiresTomorrow => soon(tr, k::CERT_EXPIRES_TOMORROW),
        ValidityLabel::ExpiresToday => soon(tr, k::CERT_EXPIRES_TODAY),
        ValidityLabel::ExpiredOn(on) => (
            tr.tr(k::CERT_EXPIRED_ON).arg("date", date(on)).to_string(),
            Tone::Danger,
            Some(icons::ERROR),
        ),
        ValidityLabel::ValidFrom(from) => (
            tr.tr(k::CERT_VALID_FROM)
                .arg("date", date(from))
                .to_string(),
            Tone::Warning,
            Some(icons::EXPIRES_SOON),
        ),
    }
}

fn soon(tr: &Catalog, key: websign_i18n::Key) -> (String, Tone, Option<Icon>) {
    (
        tr.tr(key).to_string(),
        Tone::Danger,
        Some(icons::EXPIRES_SOON),
    )
}
