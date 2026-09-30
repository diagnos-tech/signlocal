//! Devices tab, second half (`docs/ux.md` §8.4 "Token drivers"): each
//! PKCS#11 module, how loading it went and where it came from, [Remove] on
//! the ones the person added, and [Add driver…].
//!
//! "Add driver…" opens the OS file picker; where there is none (a Linux
//! desktop without `zenity` or `kdialog`) it shows a path field instead.
//! The module is loaded by the next scan, which runs right away, and the
//! result shows on its own row.

use std::path::PathBuf;

use egui::{Align, Layout, Ui};
use websign_i18n::{Catalog, k};

use super::Screen;
use super::action::Trailing;
use super::row::{Extra, Row, Status};
use super::section;
use super::spans::Span;
use super::words::tr;
use crate::ui::diagnostics::facts::{DriverFact, Facts};
use crate::ui::diagnostics::state::Action;
use crate::ui::icons;
use crate::ui::theme::metrics;
use crate::ui::widgets::button::Button;
use crate::ui::widgets::list::Position;
use crate::ui::widgets::text_field::TextField;
use crate::ui::widgets::tone::Tone;

/// The file types a module has, as a placeholder of the path field.
const EXAMPLE: &str = if cfg!(windows) {
    r"C:\Program Files\…\pkcs11.dll"
} else if cfg!(target_os = "macos") {
    "/Library/…/pkcs11.dylib"
} else {
    "/usr/lib/…/pkcs11.so"
};

pub fn show(ui: &mut Ui, screen: &Screen<'_>, facts: &Facts, actions: &mut Vec<Action>) {
    let catalog = screen.catalog;
    section::label(ui, &tr(catalog, k::DEVICES_SECTION_DRIVERS), None);
    if !facts.drivers.is_empty() {
        section::list(ui, &facts.drivers, |ui, driver, position| {
            row(ui, catalog, driver, position, actions);
        });
        ui.add_space(metrics::SPACE_3);
    }
    match &screen.state.driver_input {
        None => add_button(ui, catalog, screen.picking, actions),
        Some(path) => path_input(ui, catalog, path.clone(), actions),
    }
}

fn row(
    ui: &mut Ui,
    catalog: &Catalog,
    driver: &DriverFact,
    position: Position,
    actions: &mut Vec<Action>,
) {
    let (icon, tone, outcome) = match &driver.result {
        Ok(0) => (
            icons::NOT_APPLICABLE,
            Tone::Neutral,
            tr(catalog, k::DEVICES_DRIVER_NO_TOKEN),
        ),
        Ok(tokens) => (
            icons::SUCCESS,
            Tone::Success,
            catalog
                .plural(k::DEVICES_DRIVER_LOADED, i64::from(*tokens))
                .to_string(),
        ),
        Err(reason) => (
            icons::ERROR,
            Tone::Danger,
            catalog
                .tr(k::DEVICES_DRIVER_FAILED)
                .arg("reason", reason)
                .to_string(),
        ),
    };
    let origin = if driver.user_added {
        tr(catalog, k::DEVICES_DRIVER_USER)
    } else {
        tr(catalog, k::DEVICES_DRIVER_AUTO)
    };
    let remove = tr(catalog, k::DEVICES_DRIVER_REMOVE);
    let response = Row {
        id: ui.id().with(&driver.name),
        icon: Some(icons::DRIVER),
        title: vec![Span::strong(&driver.name)],
        status: Some(Status {
            icon: Some(icon),
            tone,
            text: format!("{outcome} · {origin}"),
        }),
        extra: (driver.path != driver.name).then(|| Extra {
            text: driver.path.clone(),
            mono: true,
        }),
        trailing: driver.setting.as_ref().map(|_| Trailing::Button {
            label: &remove,
            icon: None,
        }),
        position,
    }
    .show(ui);
    if response.clicked
        && let Some(setting) = &driver.setting
    {
        actions.push(Action::RemoveDriver(setting.clone()));
    }
}

fn add_button(ui: &mut Ui, catalog: &Catalog, picking: bool, actions: &mut Vec<Action>) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = metrics::SPACE_3;
        let label = tr(catalog, k::DEVICES_DRIVER_ADD);
        let button = Button::secondary(&label).icon(icons::ADD).busy(picking);
        if ui.add(button).clicked() && !picking {
            actions.push(Action::PickDriver);
        }
        section::note(ui, &tr(catalog, k::DEVICES_DRIVER_ADD_HELP));
    });
}

fn path_input(ui: &mut Ui, catalog: &Catalog, mut path: String, actions: &mut Vec<Action>) {
    let label = tr(catalog, k::DEVICES_DRIVER_ADD);
    let field_label = tr(catalog, k::DEVICES_DRIVER_PATH);
    let field = TextField::new(&mut path, &field_label)
        .hint(EXAMPLE)
        .icon(icons::DRIVER)
        .show(ui);
    let submitted = field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
    if field.changed() {
        actions.push(Action::EditDriverInput(path.clone()));
    }
    ui.add_space(metrics::SPACE_2);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = metrics::SPACE_2;
        let add = ui.add(
            Button::secondary(&label)
                .icon(icons::ADD)
                .enabled(!path.trim().is_empty()),
        );
        if (add.clicked() || submitted) && !path.trim().is_empty() {
            actions.push(Action::AddDriver(PathBuf::from(path.trim())));
        }
        let cancel = tr(catalog, k::COMMON_CANCEL);
        if ui.add(Button::ghost(&cancel)).clicked() {
            actions.push(Action::CancelDriverInput);
        }
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            section::note(ui, &tr(catalog, k::DEVICES_DRIVER_ADD_HELP));
        });
    });
}
