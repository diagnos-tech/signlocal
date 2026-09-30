//! What a device's row says about its certificates (`docs/ux.md` §8.4):
//! "2 certificates", "No certificates · install {driver}" with the
//! download link for this OS, or "We can't tell if it has certificates".

use websign_i18n::{Catalog, k};

use super::row::{Extra, Status};
use super::words::tr;
use crate::ui::diagnostics::facts::{CertPresence, HintFact, TokenFact};
use crate::ui::icons;
use crate::ui::widgets::tone::Tone;

/// The status line of a device and, when its driver is missing, the
/// download page for this OS. `lead` prefixes the text ("Card: G&D …").
pub fn presence(
    catalog: &Catalog,
    presence: CertPresence,
    hint: Option<&HintFact>,
    lead: Option<String>,
) -> (Status, Option<String>) {
    let (icon, tone, text, download) = match presence {
        CertPresence::Found(count) => (
            icons::SUCCESS,
            Tone::Success,
            catalog
                .plural(k::DEVICES_CERTS_FOUND, i64::from(count))
                .to_string(),
            None,
        ),
        CertPresence::Missing => {
            let driver = hint.and_then(|hint| hint.driver.clone());
            let text = match &driver {
                Some(driver) => catalog
                    .tr(k::DEVICES_NO_CERTS)
                    .arg("driver", driver)
                    .to_string(),
                None => tr(catalog, k::POSSIBLE_NO_LINK),
            };
            let download = hint.and_then(|hint| hint.download.clone());
            (icons::ATTENTION, Tone::Warning, text, download)
        }
        CertPresence::Unknown => (
            icons::NOT_APPLICABLE,
            Tone::Neutral,
            tr(catalog, k::DEVICES_UNKNOWN_CERTS),
            None,
        ),
    };
    let text = match lead {
        Some(lead) => format!("{lead} · {text}"),
        None => text,
    };
    (
        Status {
            icon: Some(icon),
            tone,
            text,
        },
        download,
    )
}

/// "Look for the software on the website of the authority…" when a token
/// needs a named driver the database has no link for on this OS.
pub fn missing_link(catalog: &Catalog, token: &TokenFact, download: Option<&str>) -> Option<Extra> {
    let named = token
        .hint
        .as_ref()
        .is_some_and(|hint| hint.driver.is_some());
    let missing = token.certificates == CertPresence::Missing;
    (missing && named && download.is_none()).then(|| Extra {
        text: tr(catalog, k::POSSIBLE_NO_LINK),
        mono: false,
    })
}
