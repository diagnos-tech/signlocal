//! "Allowed sites" and "Allowed programs" (`docs/ux.md` §8.3): what the
//! person asked WebeSign to remember, with [Revoke]. Revoking asks for an
//! inline confirmation: the button turns into "Confirm revoke" for 4 s.

use egui::Ui;
use websign_i18n::{Key, k};

use super::Screen;
use super::action::Trailing;
use super::row::{Row, Status};
use super::section;
use super::spans::Span;
use super::words::{date, tr, when};
use crate::ui::diagnostics::remembered::{CallerLabel, Remembered};
use crate::ui::diagnostics::state::{Action, Notice};
use crate::ui::icons;
use crate::ui::theme::metrics;
use crate::ui::widgets::banner::Banner;
use crate::ui::widgets::tone::Tone;

pub fn show(ui: &mut Ui, screen: &Screen<'_>, actions: &mut Vec<Action>) {
    let catalog = screen.catalog;
    section::label(
        ui,
        &tr(catalog, k::SITES_SECTION),
        Some(icons::ALLOWED_SITES),
    );
    if let Some(Notice::Revoked(shown)) = screen.state.notice_at(screen.clock) {
        let text = catalog.tr(k::SITES_REVOKED).arg("site", shown).to_string();
        ui.add(Banner::new(Tone::Success, &text));
        ui.add_space(metrics::SPACE_2);
    }
    group(ui, screen, screen.sites, k::SITES_EMPTY, actions);
    if !screen.programs.is_empty() {
        section::label(
            ui,
            &tr(catalog, k::SITES_APPS_SECTION),
            Some(icons::ON_THIS_COMPUTER),
        );
        group(ui, screen, screen.programs, k::SITES_APPS_EMPTY, actions);
    }
}

fn group(
    ui: &mut Ui,
    screen: &Screen<'_>,
    entries: &[Remembered],
    empty: Key,
    actions: &mut Vec<Action>,
) {
    let catalog = screen.catalog;
    if entries.is_empty() {
        section::empty(ui, &tr(catalog, empty));
        return;
    }
    let revoke = tr(catalog, k::SITES_REVOKE);
    let confirm = tr(catalog, k::SITES_REVOKE_CONFIRM);
    section::list(ui, entries, |ui, entry, position| {
        let pending = screen.state.revoke_pending(&entry.key, screen.clock);
        let text = catalog
            .tr(k::SITES_ROW)
            .arg("date", date(catalog, entry.remembered_at, screen.zone))
            .arg(
                "when",
                when(catalog, entry.last_used_at, screen.now, screen.zone),
            )
            .to_string();
        let row = Row {
            id: ui.id().with(&entry.key),
            icon: None,
            title: title(&entry.label),
            status: Some(Status {
                icon: None,
                tone: Tone::Neutral,
                text,
            }),
            extra: None,
            trailing: Some(Trailing::Button {
                label: if pending { &confirm } else { &revoke },
                icon: None,
            }),
            position,
        };
        if row.show(ui).clicked {
            actions.push(if pending {
                Action::Revoke {
                    key: entry.key.clone(),
                    shown: entry.label.plain(),
                }
            } else {
                Action::ArmRevoke(entry.key.clone())
            });
        }
    });
}

fn title(label: &CallerLabel) -> Vec<Span> {
    match label {
        CallerLabel::Site {
            prefix,
            registrable,
            suffix,
        } => [
            Span::dim(prefix),
            Span::strong(registrable),
            Span::dim(suffix),
        ]
        .into_iter()
        .filter(|span| !span.text.is_empty())
        .collect(),
        CallerLabel::Program { name } => vec![Span::strong(name)],
    }
}
