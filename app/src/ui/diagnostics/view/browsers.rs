//! "Browsers on this computer" (`docs/ux.md` §8.3): one row per installed
//! browser with the extension's connection, and the actions that fix it
//! (install the extension, repair the registration).

use egui::Ui;
use websign_i18n::k;
use websign_registration::detect::BrowserPackaging;
use websign_registration::status::RegistrationState;
use websign_registration::{Browser, Family};

use super::Screen;
use super::action::Trailing;
use super::row::{Extra, Row, Status};
use super::section;
use super::spans::Span;
use super::words::{tr, when};
use crate::ui::diagnostics::facts::{BrowserFact, Facts};
use crate::ui::diagnostics::state::{Action, Notice};
use crate::ui::icons::{self, Icon};
use crate::ui::theme::metrics;
use crate::ui::widgets::banner::Banner;
use crate::ui::widgets::tone::Tone;

/// Where "Install extension" leads. TODO(gustavo): the store pages, once
/// the extension is published (`project.toml` store IDs).
pub fn install_page() -> String {
    format!("{}download.html", websign_project::HOMEPAGE)
}

pub fn show(ui: &mut Ui, screen: &Screen<'_>, facts: &Facts, actions: &mut Vec<Action>) {
    let catalog = screen.catalog;
    section::label(ui, &tr(catalog, k::BROWSERS_SECTION), None);
    match screen.state.notice_at(screen.clock) {
        Some(Notice::Repaired(browser)) => {
            let text = catalog
                .tr(k::BROWSERS_REPAIRED)
                .arg("browser", browser)
                .to_string();
            ui.add(Banner::new(Tone::Success, &text));
            ui.add_space(metrics::SPACE_2);
        }
        Some(Notice::RepairFailed) => {
            let text = tr(catalog, k::ERRORS_INTERNAL_TITLE);
            ui.add(Banner::new(Tone::Danger, &text));
            ui.add_space(metrics::SPACE_2);
        }
        _ => {}
    }
    if facts.browsers.is_empty() {
        section::empty(ui, &tr(catalog, k::BROWSERS_NONE));
        return;
    }
    section::list(ui, &facts.browsers, |ui, browser, position| {
        let (status, extra, trailing, action) = describe(screen, browser);
        let row = Row {
            id: ui.id().with(browser.browser.label()),
            icon: Some(icons::BROWSER),
            title: title(browser),
            status: Some(status),
            extra,
            trailing: trailing
                .as_ref()
                .map(|(label, icon)| Trailing::Button { label, icon: *icon }),
            position,
        };
        if row.show(ui).clicked
            && let Some(action) = action
        {
            actions.push(action);
        }
    });
}

fn title(browser: &BrowserFact) -> Vec<Span> {
    let mut title = vec![Span::strong(browser.browser.label())];
    let major = browser
        .version
        .as_deref()
        .and_then(|version| version.split('.').next())
        .filter(|major| !major.is_empty());
    title.extend(major.map(Span::mono));
    title
}

type Described = (
    Status,
    Option<Extra>,
    Option<(String, Option<Icon>)>,
    Option<Action>,
);

/// Status line, hint, button and what the button does.
fn describe(screen: &Screen<'_>, browser: &BrowserFact) -> Described {
    let catalog = screen.catalog;
    let name = browser.browser.label();
    if browser.registration != RegistrationState::Registered {
        let text = catalog
            .tr(k::BROWSERS_HOST_MISSING)
            .arg("browser", name)
            .to_string();
        let status = Status {
            icon: Some(icons::ERROR),
            tone: Tone::Danger,
            text,
        };
        let button = (tr(catalog, k::BROWSERS_REPAIR), None);
        return (
            status,
            None,
            Some(button),
            Some(Action::Repair(name.to_owned())),
        );
    }
    let snap = (browser.packaging == BrowserPackaging::Snap && browser.browser == Browser::Firefox)
        .then(|| Extra {
            text: tr(catalog, k::BROWSERS_SNAP_HINT),
            mono: false,
        });
    match &browser.connection {
        Some(record) => {
            let text = catalog
                .tr(k::BROWSERS_CONNECTED)
                .arg("version", &record.extension_version)
                .arg(
                    "when",
                    when(catalog, record.last_seen, screen.now, screen.zone),
                )
                .to_string();
            let status = Status {
                icon: Some(icons::SUCCESS),
                tone: Tone::Success,
                text,
            };
            (status, snap, None, None)
        }
        None => {
            let status = Status {
                icon: Some(icons::ATTENTION),
                tone: Tone::Warning,
                text: tr(catalog, k::BROWSERS_NOT_DETECTED),
            };
            let hint = (browser.browser.family() == Family::Chromium).then(|| Extra {
                text: catalog
                    .tr(k::BROWSERS_EXTERNAL_PROMPT_HINT)
                    .arg("browser", name)
                    .to_string(),
                mono: false,
            });
            let button = (tr(catalog, k::BROWSERS_INSTALL), Some(icons::EXTERNAL_LINK));
            (
                status,
                snap.or(hint),
                Some(button),
                Some(Action::OpenUrl(install_page())),
            )
        }
    }
}
