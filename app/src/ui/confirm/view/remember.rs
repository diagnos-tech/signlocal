//! "Remember this site / program on this computer" (`docs/ux.md` §4.10):
//! unchecked by default; disabled, with the reason, for numeric and IDN
//! addresses and for interpreters and shells; absent when the selected
//! certificate is already in the caller's consent.

use egui::Ui;
use websign_i18n::k;
use websign_ui_model::confirm::port::CallerView;
use websign_ui_model::confirm::view::RememberBox;
use websign_ui_model::confirm::{ConfirmView, UserInput};

use super::{Screen, checkbox};

pub fn visible(view: &ConfirmView) -> bool {
    view.remember != RememberBox::Hidden
}

pub fn show(ui: &mut Ui, s: &mut Screen<'_>) {
    let tr = s.tr;
    let label = match s.view.header.caller {
        CallerView::Web { .. } => tr.tr(k::CONSENT_REMEMBER),
        CallerView::Desktop { .. } => tr.tr(k::CONSENT_REMEMBER_APP),
    }
    .to_string();
    let (checked, enabled, help) = match s.view.remember {
        RememberBox::Enabled { checked } => (checked, true, tr.tr(k::CONSENT_REMEMBER_HELP)),
        RememberBox::Disabled => {
            let reason = match s.view.header.caller {
                CallerView::Web { .. } => k::CONSENT_REMEMBER_DISABLED,
                CallerView::Desktop { .. } => k::CONSENT_REMEMBER_DISABLED_SCRIPT,
            };
            (false, false, tr.tr(reason))
        }
        RememberBox::Hidden => return,
    };
    let response = checkbox::show(ui, checked, enabled, &label, &help.to_string());
    if response.clicked() {
        s.input(UserInput::Remember(!checked));
    }
}
