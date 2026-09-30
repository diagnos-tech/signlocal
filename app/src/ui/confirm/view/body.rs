//! The body between header and footer: what the window shows in each state
//! of `docs/ux.md` §4.8, sections 16 px apart.

use egui::Ui;
use websign_ui_model::confirm::ConfirmState;
use websign_ui_model::confirm::port::Failure;

use super::{Screen, code, empty, error, list, outcome, pin, remember};
use crate::ui::theme::metrics;

pub fn show(ui: &mut Ui, s: &mut Screen<'_>) {
    ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
    match s.view.state {
        ConfirmState::Idle => {}
        ConfirmState::Success | ConfirmState::SiteCancelled | ConfirmState::Timeout => {
            outcome::show(ui, s);
        }
        ConfirmState::LoadingCerts => empty::loading(ui, s),
        ConfirmState::Empty => empty::show(ui, s),
        _ => request(ui, s),
    }
}

/// Choosing, ready, signing and the recoverable states: the error notice
/// (above the list, so another certificate is one click away), the code
/// card, the list, the PIN and "Remember".
fn request(ui: &mut Ui, s: &mut Screen<'_>) {
    let top = ui.cursor().top();
    let mut gap = false;
    let mut section = |ui: &mut Ui| {
        if gap {
            ui.add_space(metrics::SPACE_4);
        }
        gap = true;
    };
    // A locked PIN is told where the PIN field was (§4.6).
    if let Some(failure) = s
        .view
        .banner
        .clone()
        .filter(|f| !matches!(f, Failure::PinLocked { .. }))
    {
        section(ui);
        error::show(ui, s, &failure);
    }
    if code::visible(s.view) {
        section(ui);
        code::show(ui, s);
    }
    section(ui);
    let rows_box = list::show(ui, s);
    let mut field = None;
    if pin::visible(s.view) {
        section(ui);
        field = pin::show(ui, s);
    }
    match (field, rows_box) {
        (Some(field), Some(rows_box)) => {
            let around_rows = field.rect.bottom() - top - rows_box;
            let settled = s.session.fit.measured(ui.ctx(), around_rows);
            s.session.fit.reveal(ui, &field, settled);
        }
        _ => s.session.fit.no_field(),
    }
    if remember::visible(s.view) {
        section(ui);
        remember::show(ui, s);
    }
}
