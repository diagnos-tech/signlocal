//! Certificates tab (`docs/ux.md` §8.5): [Import .pfx file…] where the OS
//! imports it, the usable certificates grouped by where they are reached,
//! and a collapsed "Can't sign (n)" group with each one's reason.

use egui::{Label, Sense, Ui};
use websign_i18n::{Catalog, Key, k};
use websign_ui_model::certs::{CertRow, KeySource};

use super::Screen;
use super::cert_text::RowWords;
use super::section;
use super::words::tr;
use crate::ui::diagnostics::facts::{Facts, ShownReason};
use crate::ui::diagnostics::state::Action;
use crate::ui::icons;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::button::Button;
use crate::ui::widgets::cert_row::CertRow as CertRowWidget;
use crate::ui::widgets::focus;
use crate::ui::widgets::list::{self, Position};

/// The groups, in order.
const GROUPS: [(Origin, Key); 3] = [
    (Origin::Windows, k::CERTS_TAB_GROUP_WINDOWS),
    (Origin::Mac, k::CERTS_TAB_GROUP_MACOS),
    (Origin::Driver, k::CERTS_TAB_GROUP_DRIVER),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Origin {
    Windows,
    Mac,
    Driver,
}

fn origin(row: &CertRow) -> Origin {
    match row.candidate.source {
        KeySource::Windows => Origin::Windows,
        KeySource::MacosKeychain | KeySource::MacosToken => Origin::Mac,
        KeySource::Driver { .. } => Origin::Driver,
    }
}

pub fn show(ui: &mut Ui, screen: &Screen<'_>, facts: &Facts, actions: &mut Vec<Action>) {
    let catalog = screen.catalog;
    import_bar(ui, catalog, screen.picking, actions);
    let fact = &facts.certificates;
    let hidden_count = fact.list.disabled.len() + fact.hidden_rows.len();
    if fact.list.usable.is_empty() && hidden_count == 0 {
        ui.add_space(metrics::SPACE_5);
        section::empty(ui, &tr(catalog, k::CERTS_TAB_EMPTY));
        return;
    }
    for (group, key) in GROUPS {
        let rows: Vec<&CertRow> = fact
            .list
            .usable
            .iter()
            .filter(|row| origin(row) == group)
            .collect();
        if rows.is_empty() {
            continue;
        }
        section::label(ui, &format!("{} ({})", tr(catalog, key), rows.len()), None);
        let words: Vec<RowWords> = rows
            .iter()
            .map(|row| RowWords::of(catalog, row, None))
            .collect();
        cert_list(ui, catalog, &rows, &words, true, actions);
    }
    if hidden_count > 0 {
        hidden_group(ui, screen, facts, hidden_count, actions);
    }
}

/// [Import .pfx file…] and what the OS will do with it; on Linux, where
/// there is no import, only the note pointing to tokens.
fn import_bar(ui: &mut Ui, catalog: &Catalog, picking: bool, actions: &mut Vec<Action>) {
    ui.add_space(metrics::SPACE_3);
    if cfg!(target_os = "linux") {
        section::note(ui, &tr(catalog, k::CERTS_TAB_IMPORT_LINUX));
        return;
    }
    let help = if cfg!(windows) {
        k::CERTS_TAB_IMPORT_HELP_WINDOWS
    } else {
        k::CERTS_TAB_IMPORT_HELP_MACOS
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = metrics::SPACE_3;
        let label = tr(catalog, k::CERTS_TAB_IMPORT);
        let button = Button::secondary(&label).icon(icons::IMPORT).busy(picking);
        if ui.add(button).clicked() && !picking {
            actions.push(Action::ImportPfx);
        }
        section::note(ui, &tr(catalog, help));
    });
}

fn hidden_group(
    ui: &mut Ui,
    screen: &Screen<'_>,
    facts: &Facts,
    count: usize,
    actions: &mut Vec<Action>,
) {
    let c = theme::colors(ui.ctx());
    let catalog = screen.catalog;
    let open = screen.state.hidden_open;
    ui.add_space(20.0);
    let caret = if open { icons::COLLAPSE } else { icons::EXPAND };
    let title = catalog
        .tr(k::CERTS_TAB_HIDDEN_GROUP)
        .arg("count", count)
        .to_string();
    let text = format!("{}  {title}", caret.glyph());
    let toggle =
        ui.add(Label::new(typography::CAPTION.rich(text).color(c.fg_muted)).sense(Sense::click()));
    focus::ring(ui, &toggle, toggle.rect, metrics::RADIUS_SM);
    let expanded = open;
    ui.ctx().accesskit_node_builder(toggle.id, |node| {
        node.set_role(egui::accesskit::Role::Button);
        // Not the caret glyph egui filed as the label's value.
        node.clear_value();
        node.set_label(title.clone());
        node.set_expanded(expanded);
    });
    if toggle.clicked() {
        actions.push(Action::ToggleHidden);
    }
    if !open {
        return;
    }
    ui.add_space(metrics::SPACE_2);
    let fact = &facts.certificates;
    let rows: Vec<&CertRow> = fact
        .list
        .disabled
        .iter()
        .chain(fact.hidden_rows.iter().map(|(row, _)| row))
        .collect();
    let reasons = fact
        .list
        .disabled
        .iter()
        .map(|_| None)
        .chain(fact.hidden_rows.iter().map(|(_, reason)| Some(*reason)));
    let words: Vec<RowWords> = rows
        .iter()
        .zip(reasons)
        .map(|(row, reason): (&&CertRow, Option<ShownReason>)| RowWords::of(catalog, row, reason))
        .collect();
    cert_list(ui, catalog, &rows, &words, false, actions);
}

fn cert_list(
    ui: &mut Ui,
    catalog: &Catalog,
    rows: &[&CertRow],
    words: &[RowWords],
    enabled: bool,
    actions: &mut Vec<Action>,
) {
    let details = tr(catalog, k::COMMON_DETAILS);
    list::show(ui, |ui| {
        for (index, (row, words)) in rows.iter().zip(words).enumerate() {
            let response = CertRowWidget {
                text: words.text(),
                selected: false,
                enabled,
                radio: false,
                details: enabled.then_some(details.as_str()),
                position: Position::of(index, rows.len()),
            }
            .show(ui);
            if response.details.is_some_and(|link| link.clicked()) {
                actions.push(Action::Details(row.candidate.fingerprint));
            }
        }
    });
}
