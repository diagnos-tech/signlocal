//! One certificate row of the confirmation list and its keys
//! (`docs/ux.md` §4.9): a click or Space selects, ↑/↓/Home/End move the
//! selection over the rows that can sign, Enter only moves focus on (to the
//! PIN, else the primary button). No row gesture ever signs.

use egui::{EventFilter, Id, Key, Ui};
use websign_core::Fingerprint;
use websign_i18n::k;
use websign_ui_model::certs::{CertRow, RowStatus};
use websign_ui_model::confirm::UserInput;
use websign_ui_model::confirm::view::PinBlock;

use super::{Screen, details};
use crate::ui::confirm::row_text::RowText;
use crate::ui::confirm::session::FocusTarget;
use crate::ui::widgets::cert_row::CertRow as RowWidget;
use crate::ui::widgets::list::Position;

/// Draws `row`; `order` is the fingerprints the arrow keys move through.
pub fn show(
    ui: &mut Ui,
    s: &mut Screen<'_>,
    row: &CertRow,
    radio: bool,
    position: Position,
    order: &[Fingerprint],
) {
    let fingerprint = row.candidate.fingerprint;
    let selected = s.view.selected == Some(fingerprint);
    let text = RowText::of(s.tr, row);
    let details_label = s.tr.tr(k::COMMON_DETAILS).to_string();
    let enabled = row.status == RowStatus::Usable;
    let shown = RowWidget {
        text: text.widget(),
        selected,
        enabled,
        radio,
        details: enabled.then_some(details_label.as_str()),
        position,
    }
    .show(ui);
    let response = &shown.row;
    let focus_here = s.session.take_focus(FocusTarget::Row(fingerprint))
        || (selected && s.session.take_focus(FocusTarget::SelectedRow));
    if focus_here {
        response.request_focus();
        response.scroll_to_me(None);
    }
    if response.clicked() && !selected {
        s.input(UserInput::Select(fingerprint));
    }
    if shown.details.is_some_and(|details| details.clicked()) {
        s.session.details_open = !s.session.details_open;
    }
    if shown.enter {
        s.session.focus = Some(match s.view.pin {
            PinBlock::Field { .. } => FocusTarget::Pin,
            _ => FocusTarget::Primary,
        });
    }
    if response.has_focus() {
        arrows(ui, s, response.id, fingerprint, order);
    }
    if selected && enabled && s.session.details_open {
        details::show(ui, s, row);
    }
}

/// ↑/↓/Home/End on a focused row: select and focus the neighbor that can
/// sign. The row takes arrows from egui's own focus movement, which would
/// otherwise wander to the "Details" link.
fn arrows(ui: &mut Ui, s: &mut Screen<'_>, id: Id, current: Fingerprint, order: &[Fingerprint]) {
    ui.memory_mut(|memory| {
        memory.set_focus_lock_filter(
            id,
            EventFilter {
                vertical_arrows: true,
                ..EventFilter::default()
            },
        );
    });
    let Some(key) = ui.input_mut(|input| {
        [Key::ArrowUp, Key::ArrowDown, Key::Home, Key::End]
            .into_iter()
            .find(|key| input.consume_key(egui::Modifiers::NONE, *key))
    }) else {
        return;
    };
    let at = order.iter().position(|fingerprint| *fingerprint == current);
    let last = order.len().checked_sub(1);
    let target = match (key, at) {
        (Key::Home, _) => Some(0),
        (Key::End, _) => last,
        (Key::ArrowUp, Some(at)) => at.checked_sub(1),
        (Key::ArrowDown, Some(at)) => Some(at + 1).filter(|next| Some(*next) <= last),
        (_, None) => Some(0),
        _ => None,
    };
    if let Some(target) = target.and_then(|index| order.get(index)) {
        s.input(UserInput::Select(*target));
        s.session.focus = Some(FocusTarget::Row(*target));
    }
}
