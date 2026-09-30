//! Devices tab, first half (`docs/ux.md` §8.4): the Linux card service,
//! tokens and cards, and card readers with the card in each.

use egui::Ui;
use websign_i18n::{Catalog, k};
use websign_ui_model::diagnostics::atr_mask::mask_atr;

use super::Screen;
use super::action::Trailing;
use super::presence::{missing_link, presence};
use super::row::{Extra, Row, Status};
use super::section;
use super::spans::Span;
use super::words::{this_os, tr};
use crate::ui::diagnostics::facts::{Facts, ReaderFact, TokenFact};
use crate::ui::diagnostics::state::Action;
use crate::ui::icons;
use crate::ui::widgets::list::Position;
use crate::ui::widgets::tone::Tone;

/// What to run when `pcscd` is stopped.
pub const PCSCD_COMMAND: &str = "sudo systemctl enable --now pcscd.socket";

pub fn show(ui: &mut Ui, screen: &Screen<'_>, facts: &Facts, actions: &mut Vec<Action>) {
    let catalog = screen.catalog;
    let devices = &facts.devices;
    if let Some(running) = devices.pcscd_running {
        section::label(ui, &tr(catalog, k::DEVICES_PCSCD), None);
        section::list(ui, &[running], |ui, running, position| {
            pcscd_row(ui, catalog, *running, position, actions);
        });
    }
    if devices.tokens.is_empty() && devices.readers.is_empty() {
        section::label(ui, &tr(catalog, k::DEVICES_SECTION_TOKENS), None);
        section::empty(ui, &tr(catalog, k::DEVICES_EMPTY));
        return;
    }
    if !devices.tokens.is_empty() {
        section::label(ui, &tr(catalog, k::DEVICES_SECTION_TOKENS), None);
        section::list(ui, &devices.tokens, |ui, token, position| {
            token_row(ui, catalog, token, position, actions);
        });
    }
    if !devices.readers.is_empty() {
        section::label(ui, &tr(catalog, k::DEVICES_SECTION_READERS), None);
        section::list(ui, &devices.readers, |ui, reader, position| {
            reader_row(ui, catalog, reader, position, actions);
        });
    }
}

fn pcscd_row(
    ui: &mut Ui,
    catalog: &Catalog,
    running: bool,
    position: Position,
    actions: &mut Vec<Action>,
) {
    let (icon, tone, text) = if running {
        (
            icons::SUCCESS,
            Tone::Success,
            tr(catalog, k::DEVICES_PCSCD_OK),
        )
    } else {
        let text = catalog
            .tr(k::DEVICES_PCSCD_STOPPED)
            .arg("command", PCSCD_COMMAND)
            .to_string();
        (icons::ERROR, Tone::Danger, text)
    };
    let copy = tr(catalog, k::COMMON_COPY);
    let row = Row {
        id: ui.id().with("pcscd"),
        icon: Some(icons::ON_THIS_COMPUTER),
        title: vec![Span::strong(tr(catalog, k::DEVICES_PCSCD))],
        status: Some(Status {
            icon: Some(icon),
            tone,
            text,
        }),
        extra: None,
        trailing: (!running).then_some(Trailing::Icon {
            icon: icons::COPY,
            name: &copy,
        }),
        position,
    };
    if row.show(ui).clicked {
        actions.push(Action::CopyText(PCSCD_COMMAND.to_owned()));
    }
}

fn token_row(
    ui: &mut Ui,
    catalog: &Catalog,
    token: &TokenFact,
    position: Position,
    actions: &mut Vec<Action>,
) {
    let name = token.hint.as_ref().map_or_else(
        || tr(catalog, k::DEVICES_GENERIC_CCID),
        |hint| hint.name.clone(),
    );
    let usb = format!("USB {}", token.vid_pid.to_uppercase());
    let (status, download) = presence(catalog, token.certificates, token.hint.as_ref(), None);
    let label = catalog
        .tr(k::COMMON_DOWNLOAD_FOR)
        .arg("os", this_os(catalog))
        .to_string();
    let row = Row {
        id: ui.id().with(&token.vid_pid),
        icon: Some(icons::TOKEN),
        title: vec![Span::strong(name), Span::mono(usb)],
        status: Some(status),
        extra: missing_link(catalog, token, download.as_deref()),
        trailing: download.as_ref().map(|_| Trailing::Button {
            label: &label,
            icon: Some(icons::DOWNLOAD),
        }),
        position,
    };
    if row.show(ui).clicked
        && let Some(url) = download
    {
        actions.push(Action::OpenUrl(url));
    }
}

fn reader_row(
    ui: &mut Ui,
    catalog: &Catalog,
    reader: &ReaderFact,
    position: Position,
    actions: &mut Vec<Action>,
) {
    let copy = tr(catalog, k::COMMON_COPY);
    let (status, extra, masked) = match &reader.card {
        None => (
            Status {
                icon: Some(icons::NOT_APPLICABLE),
                tone: Tone::Neutral,
                text: tr(catalog, k::DEVICES_READER_EMPTY),
            },
            None,
            None,
        ),
        Some(card) => {
            let card_name = card.hint.as_ref().map_or_else(
                || tr(catalog, k::DEVICES_READER_CARD_UNKNOWN),
                |hint| {
                    catalog
                        .tr(k::DEVICES_READER_CARD)
                        .arg("card", &hint.name)
                        .to_string()
                },
            );
            let (status, _) = presence(
                catalog,
                card.certificates,
                card.hint.as_ref(),
                Some(card_name),
            );
            let masked = card.atr.as_deref().map(mask_atr);
            let extra = masked.as_ref().map(|atr| Extra {
                text: format!("ATR {atr}"),
                mono: true,
            });
            (status, extra, masked)
        }
    };
    let row = Row {
        id: ui.id().with(&reader.name),
        icon: Some(icons::CARD),
        title: vec![Span::strong(&reader.name)],
        status: Some(status),
        extra,
        trailing: masked.as_ref().map(|_| Trailing::Icon {
            icon: icons::COPY,
            name: &copy,
        }),
        position,
    };
    if row.show(ui).clicked
        && let Some(atr) = masked
    {
        actions.push(Action::CopyText(atr));
    }
}
