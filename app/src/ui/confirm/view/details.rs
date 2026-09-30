//! "Details" of the selected certificate (`docs/ux.md` §5.12): label/value
//! pairs under the row, with the fingerprint copyable.

use egui::{Grid, Margin, Ui};
use jiff::Timestamp;
use websign_core::{CertInfo, Curve, PublicKeyKind};
use websign_i18n::{Catalog, dates, k};
use websign_ui_model::certs::{CertRow, KeySource};
use websign_ui_model::confirm::UserInput;

use super::{Action, Screen};
use crate::ui::icons;
use crate::ui::theme::{self, typography};
use crate::ui::widgets::button::{Button, Size};

pub fn show(ui: &mut Ui, s: &mut Screen<'_>, row: &CertRow) {
    let Ok(info) = &row.candidate.info else {
        return;
    };
    let c = theme::colors(ui.ctx());
    let tr = s.tr;
    let fingerprint = grouped(&info.fingerprint.to_string());
    let mut pairs = vec![
        (
            tr.tr(k::CERT_DETAILS_SUBJECT).to_string(),
            info.subject.common_name.clone().unwrap_or_default(),
        ),
        (tr.tr(k::CERT_DETAILS_ISSUER).to_string(), issuer(info)),
        (
            tr.tr(k::CERT_DETAILS_VALIDITY).to_string(),
            validity(tr, info),
        ),
        (tr.tr(k::CERT_DETAILS_USAGE).to_string(), usage(tr, info)),
        (tr.tr(k::CERT_DETAILS_KEY).to_string(), key(&info.key)),
        (
            tr.tr(k::CERT_DETAILS_ACCESS).to_string(),
            access(tr, &row.candidate.source),
        ),
    ];
    if row
        .candidate
        .alternates
        .iter()
        .any(|path| matches!(path.source, KeySource::Driver { .. }))
    {
        pairs.push((
            String::new(),
            tr.tr(k::CERT_DETAILS_ALSO_VIA_DRIVER).to_string(),
        ));
    }
    egui::Frame::new()
        .inner_margin(Margin {
            left: 44,
            right: 16,
            top: 0,
            bottom: 12,
        })
        .show(ui, |ui| {
            Grid::new("confirm.details")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    for (label, value) in &pairs {
                        ui.label(typography::SMALL.rich(label).color(c.fg_subtle));
                        ui.add(egui::Label::new(typography::SMALL.rich(value).color(c.fg)).wrap());
                        ui.end_row();
                    }
                    ui.label(
                        typography::SMALL
                            .rich(tr.tr(k::CERT_DETAILS_FINGERPRINT).to_string())
                            .color(c.fg_subtle),
                    );
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Label::new(typography::MONO.rich(&fingerprint).color(c.fg))
                                .wrap()
                                .selectable(true),
                        );
                        let copy = tr.tr(k::COMMON_COPY).to_string();
                        if Button::ghost(&copy)
                            .icon(icons::COPY)
                            .size(Size::Small)
                            .show(ui)
                            .response
                            .clicked()
                        {
                            s.out.push(Action::Copy(fingerprint.clone()));
                        }
                    });
                    ui.end_row();
                    ui.label("");
                    view_in_system(ui, s);
                    ui.end_row();
                });
        });
}

/// The OS viewer shows everything the pairs above leave out; the host opens
/// it with the certificate it holds (`docs/ux.md` §5.12).
fn view_in_system(ui: &mut Ui, s: &mut Screen<'_>) {
    let label = s.tr.tr(k::CERT_DETAILS_VIEW_IN_SYSTEM).to_string();
    if Button::ghost(&label)
        .icon(icons::CERTIFICATE)
        .size(Size::Small)
        .show(ui)
        .response
        .clicked()
    {
        s.input(UserInput::ViewCertificate);
    }
}

fn issuer(info: &CertInfo) -> String {
    [
        &info.issuer.common_name,
        &info.issuer.organization,
        &info.issuer.country,
    ]
    .into_iter()
    .flatten()
    .cloned()
    .collect::<Vec<_>>()
    .join(", ")
}

fn validity(tr: &Catalog, info: &CertInfo) -> String {
    let date = |secs: i64| {
        Timestamp::from_second(secs)
            .map(|at| {
                dates::format_date(
                    tr.locale(),
                    at.to_zoned(jiff::tz::TimeZone::system()).date(),
                )
            })
            .unwrap_or_default()
    };
    tr.tr(k::CERT_DETAILS_VALIDITY_RANGE)
        .arg("from", date(info.not_before))
        .arg("to", date(info.not_after))
        .to_string()
}

fn usage(tr: &Catalog, info: &CertInfo) -> String {
    let Some(usage) = &info.key_usage else {
        return String::new();
    };
    let mut parts = Vec::new();
    if usage.digital_signature {
        parts.push(tr.tr(k::CERT_DETAILS_USAGE_SIGN).to_string());
    }
    if usage.non_repudiation {
        parts.push(tr.tr(k::CERT_DETAILS_USAGE_NONREP).to_string());
    }
    parts.join(", ")
}

fn key(key: &PublicKeyKind) -> String {
    match key {
        PublicKeyKind::Rsa { bits } => format!("RSA {bits}"),
        PublicKeyKind::Ec { curve } => format!("ECDSA {}", curve_name(*curve)),
        PublicKeyKind::Unsupported { oid } => oid.clone(),
    }
}

fn curve_name(curve: Curve) -> &'static str {
    match curve {
        Curve::P256 => "P-256",
        Curve::P384 => "P-384",
        Curve::P521 => "P-521",
        Curve::BrainpoolP256r1 => "brainpoolP256r1",
        Curve::BrainpoolP384r1 => "brainpoolP384r1",
        Curve::BrainpoolP512r1 => "brainpoolP512r1",
    }
}

fn access(tr: &Catalog, source: &KeySource) -> String {
    match source {
        KeySource::Windows => tr.tr(k::CERT_DETAILS_ACCESS_WINDOWS).to_string(),
        KeySource::MacosKeychain | KeySource::MacosToken => {
            tr.tr(k::CERT_DETAILS_ACCESS_MACOS).to_string()
        }
        KeySource::Driver { path } => tr
            .tr(k::CERT_DETAILS_ACCESS_DRIVER)
            .arg("path", path)
            .to_string(),
    }
}

/// 64 hex digits in 16 groups of 4.
fn grouped(hex: &str) -> String {
    let upper = hex.to_ascii_uppercase();
    let chars: Vec<char> = upper.chars().filter(char::is_ascii_hexdigit).collect();
    chars
        .chunks(4)
        .map(|group| group.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}
