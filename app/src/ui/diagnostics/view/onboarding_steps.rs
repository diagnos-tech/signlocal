//! What each step of "Getting started" that is still to do says, and the
//! one click that fixes it (`docs/ux.md` §8.2, §10): Repair for a missing
//! registration, the extension's install page, the command that starts the
//! card service, the token's driver download, the `.pfx` import, the test
//! page.

use websign_i18n::{Catalog, k};
use websign_ui_model::diagnostics::onboarding::{Step, StepState};

use super::browsers::install_page;
use super::devices::PCSCD_COMMAND;
use super::row::Status;
use super::words::{this_os, tr};
use crate::ui::diagnostics::facts::{CertPresence, Facts, HintFact};
use crate::ui::diagnostics::state::Action;
use crate::ui::icons::{self, Icon};
use crate::ui::widgets::tone::Tone;

/// A step to do, as its row shows it.
#[derive(Debug, Clone)]
pub struct Todo {
    pub icon: Icon,
    pub title: String,
    pub status: Status,
    /// The fix: its label and icon, and what it does.
    pub fix: Option<(String, Option<Icon>, Action)>,
}

/// The site's test page, which signs a sample message and shows the same
/// verification code.
pub fn test_page() -> String {
    format!("{}test/", websign_project::HOMEPAGE)
}

/// The site's page that starts the app from the browser and waits for the
/// extension to connect (`docs/ux.md` §9).
pub fn activate_page() -> String {
    format!("{}activate/", websign_project::HOMEPAGE)
}

/// The row of `step`, which is not done.
pub fn todo(catalog: &Catalog, facts: &Facts, step: Step, state: StepState) -> Todo {
    let (icon, title, body, fix) = match step {
        Step::App => (
            icons::ON_THIS_COMPUTER,
            tr(catalog, k::ONBOARDING_TODO_APP),
            tr(catalog, k::ONBOARDING_TODO_APP_BODY),
            repair(catalog, facts),
        ),
        Step::Extension => extension(catalog, facts, state),
        Step::CardService => (
            icons::ON_THIS_COMPUTER,
            tr(catalog, k::ONBOARDING_TODO_CARD_SERVICE),
            catalog
                .tr(k::ONBOARDING_TODO_CARD_SERVICE_BODY)
                .arg("command", PCSCD_COMMAND)
                .to_string(),
            Some((
                tr(catalog, k::ONBOARDING_COPY_COMMAND),
                Some(icons::COPY),
                Action::CopyText(PCSCD_COMMAND.to_owned()),
            )),
        ),
        Step::Driver => driver(catalog, facts),
        Step::Certificate => (
            icons::CERTIFICATE,
            tr(catalog, k::ONBOARDING_TODO_CERT),
            tr(catalog, k::CERTS_EMPTY_BODY),
            // Linux has no system store to import into (§8.5).
            (!cfg!(target_os = "linux"))
                .then(|| (tr(catalog, k::CERTS_TAB_IMPORT), None, Action::ImportPfx)),
        ),
        Step::TestSignature => (
            icons::SIGNATURE_REQUEST,
            tr(catalog, k::ONBOARDING_TODO_TEST),
            tr(catalog, k::ONBOARDING_TODO_TEST_BODY),
            Some((
                tr(catalog, k::ONBOARDING_TEST_BUTTON),
                Some(icons::EXTERNAL_LINK),
                Action::OpenUrl(test_page()),
            )),
        ),
    };
    let (status_icon, tone) = match state {
        StepState::Attention => (icons::ATTENTION, Tone::Warning),
        StepState::Pending | StepState::Done => (icons::NOT_APPLICABLE, Tone::Neutral),
    };
    Todo {
        icon,
        title,
        status: Status {
            icon: Some(status_icon),
            tone,
            text: body,
        },
        fix,
    }
}

type Parts = (Icon, String, String, Option<(String, Option<Icon>, Action)>);

/// [Repair] rewrites every browser's registration; the notice names the
/// first browser to restart.
fn repair(catalog: &Catalog, facts: &Facts) -> Option<(String, Option<Icon>, Action)> {
    let browser = facts.browsers.first()?.browser.label().to_owned();
    Some((
        tr(catalog, k::BROWSERS_REPAIR),
        None,
        Action::Repair(browser),
    ))
}

/// The browsers the extension has not connected from, by name; [Repair]
/// when a browser cannot start the app, else the install page.
fn extension(catalog: &Catalog, facts: &Facts, state: StepState) -> Parts {
    let waiting: Vec<&str> = facts
        .browsers
        .iter()
        .filter(|browser| browser.connection.is_none())
        .map(|browser| browser.browser.label())
        .collect();
    let body = if waiting.is_empty() {
        tr(catalog, k::BROWSERS_NONE)
    } else {
        catalog
            .tr(k::ONBOARDING_TODO_EXTENSION_BODY)
            .arg("browsers", waiting.join(", "))
            .to_string()
    };
    let install = || {
        (
            tr(catalog, k::BROWSERS_INSTALL),
            Some(icons::EXTERNAL_LINK),
            Action::OpenUrl(install_page()),
        )
    };
    let fix = match state {
        StepState::Attention => repair(catalog, facts),
        StepState::Pending | StepState::Done => Some(install()),
    };
    (
        icons::BROWSER,
        tr(catalog, k::ONBOARDING_TODO_EXTENSION),
        body,
        fix,
    )
}

/// The first connected device whose driver is missing, with its download.
fn driver(catalog: &Catalog, facts: &Facts) -> Parts {
    let devices = &facts.devices;
    let tokens = devices
        .tokens
        .iter()
        .map(|token| (token.certificates, token.hint.as_ref(), icons::TOKEN));
    let cards = devices.readers.iter().filter_map(|reader| {
        let card = reader.card.as_ref()?;
        Some((card.certificates, card.hint.as_ref(), icons::CARD))
    });
    let waiting = tokens
        .chain(cards)
        .find(|(presence, _, _)| *presence == CertPresence::Missing);
    let (icon, hint) = waiting.map_or((icons::DRIVER, None), |(_, hint, icon)| (icon, hint));
    let device = hint.map_or_else(
        || tr(catalog, k::DEVICES_GENERIC_CCID),
        |hint| hint.name.clone(),
    );
    let title = tr(catalog, k::ONBOARDING_TODO_DRIVER);
    let body = match hint.and_then(|hint| hint.driver.as_deref()) {
        Some(driver) => catalog
            .tr(k::ONBOARDING_TODO_DRIVER_BODY)
            .arg("device", device)
            .arg("driver", driver)
            .to_string(),
        None => catalog
            .tr(k::ONBOARDING_TODO_DRIVER_BODY_GENERIC)
            .arg("device", device)
            .to_string(),
    };
    (icon, title, body, download(catalog, hint))
}

/// "Download for {os}" when the device database knows the page.
fn download(catalog: &Catalog, hint: Option<&HintFact>) -> Option<(String, Option<Icon>, Action)> {
    let url = hint?.download.clone()?;
    let label = catalog
        .tr(k::COMMON_DOWNLOAD_FOR)
        .arg("os", this_os(catalog))
        .to_string();
    Some((label, Some(icons::DOWNLOAD), Action::OpenUrl(url)))
}
