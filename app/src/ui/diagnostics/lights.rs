//! The traffic lights of each tab and of the whole window (`docs/ux.md`
//! §8.1), the tab the window opens on, and the "Getting started" strip
//! (§8.2), decided by `websign-ui-model` from the facts of a scan.

use websign_i18n::{Key, k};
use websign_protocol::messages::DiagnosticsTab;
use websign_registration::status::RegistrationState;
use websign_ui_model::certs::{CertRow, ValidityLabel};
use websign_ui_model::diagnostics::onboarding::{Onboarding, OnboardingFacts, onboarding};
use websign_ui_model::diagnostics::status::{
    BrowsersFacts, CertificatesFacts, DevicesFacts, Light, browsers_light, certificates_light,
    devices_light, overall,
};

use super::facts::{CertPresence, Facts};
use crate::ui::icons::{self, Icon};
use crate::ui::widgets::tone::Tone;

/// Every light of one scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lights {
    pub browsers: Light,
    pub devices: Light,
    pub certificates: Light,
}

impl Lights {
    pub fn of(facts: &Facts) -> Lights {
        Lights {
            browsers: browsers_light(&browsers_facts(facts)),
            devices: devices_light(&devices_facts(facts)),
            certificates: certificates_light(&CertificatesFacts {
                usable: count(facts.certificates.list.usable.len()),
                expiring_within_30_days: facts
                    .certificates
                    .list
                    .usable
                    .iter()
                    .filter(|row| expires_soon(row))
                    .count()
                    .try_into()
                    .unwrap_or(u32::MAX),
            }),
        }
    }

    /// The worst light.
    pub fn overall(self) -> Light {
        overall(&[self.browsers, self.devices, self.certificates])
    }

    /// The light of `tab`; Help has none.
    pub fn of_tab(self, tab: DiagnosticsTab) -> Option<Light> {
        match tab {
            DiagnosticsTab::Browsers => Some(self.browsers),
            DiagnosticsTab::Devices => Some(self.devices),
            DiagnosticsTab::Certificates => Some(self.certificates),
            DiagnosticsTab::Help => None,
        }
    }

    /// The first tab with a red light, else a yellow one, else Browsers.
    pub fn first_problem(self) -> DiagnosticsTab {
        let tabs = [
            DiagnosticsTab::Browsers,
            DiagnosticsTab::Devices,
            DiagnosticsTab::Certificates,
        ];
        [Light::Red, Light::Yellow]
            .into_iter()
            .find_map(|level| {
                tabs.into_iter()
                    .find(|&tab| self.of_tab(tab) == Some(level))
            })
            .unwrap_or(DiagnosticsTab::Browsers)
    }
}

/// 30 days or less left: the list's own warning and danger labels.
fn expires_soon(row: &CertRow) -> bool {
    matches!(
        row.validity,
        ValidityLabel::ExpiresInDays(_)
            | ValidityLabel::ExpiresTomorrow
            | ValidityLabel::ExpiresToday
    )
}

fn browsers_facts(facts: &Facts) -> BrowsersFacts {
    let connected = facts
        .browsers
        .iter()
        .filter(|browser| browser.connection.is_some())
        .count();
    let problems = facts
        .browsers
        .iter()
        .filter(|browser| {
            browser.connection.is_none() || browser.registration != RegistrationState::Registered
        })
        .count();
    BrowsersFacts {
        connected: count(connected),
        with_problems: count(problems),
        registration_missing_everywhere: !facts.browsers.is_empty()
            && facts
                .browsers
                .iter()
                .all(|browser| browser.registration != RegistrationState::Registered),
    }
}

fn devices_facts(facts: &Facts) -> DevicesFacts {
    let devices = &facts.devices;
    let cards = devices
        .readers
        .iter()
        .filter_map(|reader| reader.card.as_ref());
    let without = devices
        .tokens
        .iter()
        .map(|token| token.certificates)
        .chain(cards.map(|card| card.certificates))
        .filter(|presence| *presence == CertPresence::Missing)
        .count();
    DevicesFacts {
        devices_without_certificates: count(without),
        drivers_failed: count(facts.drivers.iter().filter(|d| d.result.is_err()).count()),
        complement_outdated: false,
        pcscd_stopped: devices.pcscd_running == Some(false),
    }
}

/// The "Getting started" strip.
pub fn getting_started(facts: &Facts, dismissed: bool, test_done: bool) -> Onboarding {
    let problem = facts
        .browsers
        .iter()
        .any(|browser| browser.registration != RegistrationState::Registered);
    onboarding(OnboardingFacts {
        any_extension_connected: facts.browsers.iter().any(|b| b.connection.is_some()),
        any_extension_problem: problem,
        usable_certificates: count(facts.certificates.list.usable.len()),
        test_signature_done: test_done,
        dismissed,
    })
}

/// How a light looks: fill icon, tone and the words for it (§8.1: color,
/// shape and text, never color alone).
pub fn look(light: Light) -> (Icon, Tone, Key) {
    match light {
        Light::Green => (icons::SUCCESS, Tone::Success, k::DIAG_STATUS_OK),
        Light::Yellow => (icons::ATTENTION, Tone::Warning, k::DIAG_STATUS_ATTENTION),
        Light::Red => (icons::ERROR, Tone::Danger, k::DIAG_STATUS_BLOCKED),
        Light::Gray => (icons::NOT_APPLICABLE, Tone::Neutral, k::DIAG_STATUS_NA),
    }
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}
