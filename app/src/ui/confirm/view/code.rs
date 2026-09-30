//! The verification-code slot (`docs/ux.md` §4.4, §4.10): the code once the
//! digest is known, a skeleton while the caller prepares it, the Continue
//! hint before a new caller may see the certificate (D11), or what a site
//! learns in Choose mode.

use egui::Ui;
use websign_i18n::k;
use websign_ui_model::confirm::ConfirmView;
use websign_ui_model::confirm::port::Mode;
use websign_ui_model::confirm::view::CodeCard as Slot;

use super::Screen;
use crate::ui::confirm::words;
use crate::ui::icons;
use crate::ui::widgets::banner::Banner;
use crate::ui::widgets::code_card::{CodeCard, CodeState};
use crate::ui::widgets::tone::Tone;

pub fn visible(view: &ConfirmView) -> bool {
    view.code != Slot::Hidden
}

pub fn show(ui: &mut Ui, s: &mut Screen<'_>) {
    let tr = s.tr;
    let label = tr.tr(k::CODE_LABEL).to_string();
    let help = tr.tr(k::CODE_HELP).to_string();
    let hash = match s.view.mode {
        Mode::Sign { hash } => words::hash(hash),
        Mode::Choose => "",
    };
    // Under an error notice the help line gives its room to the notice.
    let compact = s.view.banner.is_some();
    let card = |state| CodeCard {
        label: &label,
        hash,
        help: &help,
        state,
        compact,
    };
    match &s.view.code {
        Slot::Hidden => {}
        Slot::ContinueHint => {
            let hint = tr.tr(k::CODE_CONTINUE_HINT).to_string();
            ui.add(Banner::new(Tone::Info, &hint).icon(icons::VERIFICATION_CODE));
        }
        Slot::SelectShares => {
            let shares = tr.tr(k::CONSENT_SELECT_SHARES).to_string();
            ui.add(Banner::new(Tone::Info, &shares));
        }
        Slot::Preparing { skeleton } => {
            let preparing = tr.tr(k::CODE_PREPARING).to_string();
            // Under 150 ms nothing is shown, but the space is kept so the
            // list does not jump when the card appears.
            ui.scope(|ui| {
                if !skeleton {
                    ui.set_invisible();
                }
                card(CodeState::Preparing {
                    preparing: &preparing,
                })
                .show(ui);
            });
        }
        Slot::Ready { code, .. } => {
            let spoken = tr
                .tr(k::CODE_A11Y)
                .arg("spelled", words::spelled(&code.text))
                .to_string();
            card(CodeState::Ready {
                code,
                spoken: &spoken,
            })
            .show(ui);
        }
    }
}
